#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Result;
use mio::{
    net::{TcpListener, TcpStream},
    Interest,
};
use pyo3::{buffer::PyBuffer, prelude::*, types::PyBytes, IntoPyObjectExt};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    io::Read,
    sync::{atomic, Arc, Mutex},
};

use crate::{
    event_loop::EventLoop,
    handles::CBHandle,
    log::LogExc,
    py::{asyncio_proto_buf, copy_context, run_in_ctx0},
    sock::SocketWrapper,
};

pub(crate) struct TCPServer {
    pub fd: i32,
    sfamily: i32,
    backlog: i32,
    protocol_factory: PyObject,
}

impl TCPServer {
    pub(crate) fn from_fd(fd: i32, sfamily: i32, backlog: i32, protocol_factory: PyObject) -> Self {
        Self {
            fd,
            sfamily,
            backlog,
            protocol_factory,
        }
    }

    pub(crate) fn listen(&self, py: Python, pyloop: Py<EventLoop>) -> Result<()> {
        // TODO: handle errors
        let sock = unsafe { socket2::Socket::from_raw_fd(self.fd) };
        sock.listen(self.backlog)?;

        let stdl: std::net::TcpListener = sock.into();
        let listener = TcpListener::from_std(stdl);
        let sref = TCPServerRef {
            pyloop: pyloop.clone_ref(py),
            listener: listener.into(),
            sfamily: self.sfamily,
            proto_factory: self.protocol_factory.clone_ref(py),
        };
        pyloop.get().tcp_listener_add(sref);

        Ok(())
    }

    pub(crate) fn close(&self, event_loop: &EventLoop) -> Result<()> {
        let closed = event_loop.tcp_listener_rem(self.fd as usize)?;
        if closed {}
        Ok(())
    }
}

pub(crate) struct TCPServerRef {
    pyloop: Py<EventLoop>,
    pub listener: Arc<TcpListener>,
    sfamily: i32,
    proto_factory: PyObject,
}

impl TCPServerRef {
    pub(crate) fn transport(&self, py: Python, stream: TcpStream) -> TCPTransport {
        TCPTransport::new(
            py,
            stream,
            self.sfamily,
            self.pyloop.clone_ref(py),
            self.proto_factory.clone_ref(py),
        )
    }
}

#[pyclass(frozen)]
pub(crate) struct TCPTransport {
    pub stream: Arc<Mutex<TcpStream>>,
    pub fd: usize,
    sock: Py<SocketWrapper>,
    pyloop: Py<EventLoop>,
    proto: PyObject,
    extra: HashMap<String, PyObject>,
    buffered_proto: bool,
    closing: atomic::AtomicBool,
    paused: atomic::AtomicBool,
    paused_proto: atomic::AtomicBool,
    buffer: VecDeque<Vec<u8>>,
    water_hi: atomic::AtomicUsize,
    water_lo: atomic::AtomicUsize,
    weof: atomic::AtomicBool,
    pym_conn_made: PyObject,
    pym_conn_lost: PyObject,
    pym_recv_data: PyObject,
    pym_recv_eof: PyObject,
    pym_buf_get: PyObject,
    pym_write_pause: PyObject,
    pym_write_resume: PyObject,
}

impl TCPTransport {
    pub(crate) fn new(
        py: Python,
        stream: TcpStream,
        sfamily: i32,
        pyloop: Py<EventLoop>,
        proto_factory: PyObject,
    ) -> Self {
        let fd = stream.as_raw_fd() as usize;
        let proto = proto_factory.bind(py).call0().unwrap();
        let mut buffered_proto = false;
        let pym_recv_data: PyObject;
        let pym_buf_get: PyObject;

        if proto.is_instance(asyncio_proto_buf(py).unwrap()).unwrap() {
            buffered_proto = true;
            pym_recv_data = proto.getattr(pyo3::intern!(py, "buffer_updated")).unwrap().unbind();
            pym_buf_get = proto.getattr(pyo3::intern!(py, "get_buffer")).unwrap().unbind();
        } else {
            pym_recv_data = proto.getattr(pyo3::intern!(py, "data_received")).unwrap().unbind();
            pym_buf_get = py.None();
        }
        let pym_conn_made = proto.getattr(pyo3::intern!(py, "connection_made")).unwrap().unbind();
        let pym_conn_lost = proto.getattr(pyo3::intern!(py, "connection_lost")).unwrap().unbind();
        let pym_recv_eof = proto.getattr(pyo3::intern!(py, "eof_received")).unwrap().unbind();
        let pym_write_pause = proto.getattr(pyo3::intern!(py, "pause_writing")).unwrap().unbind();
        let pym_write_resume = proto.getattr(pyo3::intern!(py, "resume_writing")).unwrap().unbind();

        let wh = 1024 * 64;
        let wl = wh / 4;

        Self {
            stream: Arc::new(Mutex::new(stream)),
            fd,
            sock: SocketWrapper::from_fd(py, fd, sfamily, socket2::Type::STREAM, 0),
            pyloop,
            proto: proto.unbind(),
            extra: HashMap::new(),
            buffered_proto,
            closing: false.into(),
            paused: false.into(),
            paused_proto: false.into(),
            buffer: VecDeque::new(),
            water_hi: wh.into(),
            water_lo: wl.into(),
            weof: false.into(),
            pym_conn_made,
            pym_conn_lost,
            pym_recv_data,
            pym_recv_eof,
            pym_buf_get,
            pym_write_pause,
            pym_write_resume,
        }
    }

    #[inline]
    pub(crate) fn attach(pyself: Py<Self>, py: Python) -> CBHandle {
        CBHandle::new1(
            pyself.get().pym_conn_made.clone_ref(py),
            pyself.into_any(),
            copy_context(py),
        )
    }

    // pub(crate) fn detach(&self) {
    // }

    fn try_proto_pause(pyself: Py<Self>, py: Python) {
        let rself = pyself.get();

        if rself.get_write_buffer_size() <= rself.water_hi.load(atomic::Ordering::Relaxed) {
            return;
        }
        if rself
            .paused_proto
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let cb = rself.pym_write_pause.as_ptr();
        let res = run_in_ctx0!(py, copy_context(py).as_ptr(), cb);
        if let Err(err) = res {
            let err_ctx = LogExc::transport(
                err,
                "protocol.pause_writing() failed".into(),
                rself.proto.clone_ref(py),
                pyself.clone_ref(py).into_any(),
            );
            let _ = rself.pyloop.get().log_exception(py, err_ctx);
        }
    }

    fn try_proto_resume(pyself: Py<Self>, py: Python) {
        let rself = pyself.get();

        if rself.get_write_buffer_size() > rself.water_lo.load(atomic::Ordering::Relaxed) {
            return;
        }
        if rself
            .paused_proto
            .compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let cb = rself.pym_write_resume.as_ptr();
        let res = run_in_ctx0!(py, copy_context(py).as_ptr(), cb);
        if let Err(err) = res {
            let err_ctx = LogExc::transport(
                err,
                "protocol.resume_writing() failed".into(),
                rself.proto.clone_ref(py),
                pyself.clone_ref(py).into_any(),
            );
            let _ = rself.pyloop.get().log_exception(py, err_ctx);
        }
    }

    #[inline]
    pub(crate) fn recv(pyself: Py<Self>, py: Python) -> CBHandle {
        let rself = pyself.get();
        match rself.buffered_proto {
            true => {
                // NOTE: `PuBuffer.as_mut_slice` exists, but it returns a slice of `Cell<u8>`,
                //       which is smth we can't really use to read from `TcpStream`.
                //       So even if this sucks, we copy data back and forth, at least until
                //       we figure out a way to actually use `PyBuffer` directly.
                let pybuf: PyBuffer<u8> = PyBuffer::get(&rself.pym_buf_get.bind(py).call0().unwrap()).unwrap();
                let mut vbuf = pybuf.to_vec(py).unwrap();
                let read = py.allow_threads(|| rself.read_into(vbuf.as_mut_slice()));
                if read == 0 {
                    let cb = TCPTransportOnEof {
                        transport: pyself.clone_ref(py),
                    };
                    CBHandle::new0(Py::new(py, cb).unwrap().into_any(), copy_context(py))
                } else {
                    pybuf.copy_from_slice(py, &vbuf[..]).unwrap();
                    CBHandle::new1(
                        rself.pym_recv_data.clone_ref(py),
                        read.into_py_any(py).unwrap(),
                        copy_context(py),
                    )
                }
            }
            false => {
                let data = py.allow_threads(|| rself.read());
                if data.len() == 0 {
                    let cb = TCPTransportOnEof {
                        transport: pyself.clone_ref(py),
                    };
                    CBHandle::new0(Py::new(py, cb).unwrap().into_any(), copy_context(py))
                } else {
                    CBHandle::new1(
                        rself.pym_recv_data.clone_ref(py),
                        (&data[..]).into_py_any(py).unwrap(),
                        copy_context(py),
                    )
                }
            }
        }
    }

    #[inline]
    fn read(&self) -> Box<[u8]> {
        let mut guard = self.stream.lock().unwrap();
        let mut len = 0;
        let mut buf = [0; 262144];
        loop {
            match guard.read(&mut buf[len..]) {
                Ok(0) => break,
                Ok(readn) => {
                    len += readn;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        buf[..len].into()
    }

    #[inline]
    fn read_into(&self, buf: &mut [u8]) -> usize {
        let mut guard = self.stream.lock().unwrap();
        let mut len = 0;
        loop {
            match guard.read(&mut buf[len..]) {
                Ok(0) => break,
                Ok(readn) => {
                    len += readn;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        len
    }

    #[inline]
    fn cb_eof(pyself: Py<Self>, py: Python) {
        let rself = pyself.get();
        match rself.pym_recv_eof.call0(py) {
            Ok(ret) => match ret.is_truthy(py).unwrap() {
                true => {
                    rself
                        .pyloop
                        .get()
                        .tcp_stream_rem(pyself.clone_ref(py), Interest::READABLE);
                }
                false => TCPTransport::close(pyself, py),
            },
            Err(err) => {
                todo!()
                // log fatal error
                // force close
            }
        }
    }

    // fn write2(&self) {}

    // fn force_close(&self) {}
}

#[pymethods]
impl TCPTransport {
    #[pyo3(signature = (name, default = None))]
    fn get_extra_info(&self, py: Python, name: &str, default: Option<PyObject>) -> Option<PyObject> {
        match name {
            "socket" => Some(self.sock.clone_ref(py).into_any()),
            "sockname" => match self.sock.call_method0(py, pyo3::intern!(py, "getsockname")) {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            "peername" => match self.sock.call_method0(py, pyo3::intern!(py, "getpeername")) {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            _ => self.extra.get(name).map(|v| v.clone_ref(py)).or(default),
        }
    }

    fn is_closing(&self) -> bool {
        self.closing.load(atomic::Ordering::Relaxed)
    }

    fn close(pyself: Py<Self>, py: Python) {
        let rself = pyself.get();
        if rself
            .closing
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let event_loop = rself.pyloop.get();
        event_loop.tcp_stream_rem(pyself.clone_ref(py), Interest::READABLE);
        if rself.buffer.len() == 0 {
            // TODO: set conn lost
            event_loop.tcp_stream_rem(pyself.clone_ref(py), Interest::WRITABLE);
            event_loop.schedule0(py, rself.pym_conn_lost.clone_ref(py), Some(copy_context(py)));
        }
    }

    fn set_protocol(&self, _protocol: PyObject) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "TCPTransport protocol cannot be changed",
        ))
    }

    fn get_protocol(&self, py: Python) -> PyObject {
        self.proto.clone_ref(py)
    }

    fn is_reading(&self) -> bool {
        !self.closing.load(atomic::Ordering::Relaxed) && !self.paused.load(atomic::Ordering::Relaxed)
    }

    fn pause_reading(pyself: Py<Self>, py: Python) {
        let rself = pyself.get();
        if rself.closing.load(atomic::Ordering::Relaxed) {
            return;
        }
        if rself
            .paused
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        rself
            .pyloop
            .get()
            .tcp_stream_rem(pyself.clone_ref(py), Interest::READABLE);
    }

    fn resume_reading(pyself: Py<Self>, py: Python) {
        let rself = pyself.get();
        if rself.closing.load(atomic::Ordering::Relaxed) {
            return;
        }
        if rself
            .paused
            .compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        rself
            .pyloop
            .get()
            .tcp_stream_add(pyself.clone_ref(py), Interest::READABLE);
    }

    #[pyo3(signature = (high = None, low = None))]
    fn set_write_buffer_limits(pyself: Py<Self>, py: Python, high: Option<usize>, low: Option<usize>) -> PyResult<()> {
        let wh: usize;
        let wl: usize;

        wh = match high {
            None => match low {
                None => 1024 * 64,
                Some(v) => v * 4,
            },
            Some(v) => v,
        };
        wl = match low {
            None => wh / 4,
            Some(v) => v,
        };

        if wh < wl {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "high must be >= low must be >= 0",
            ));
        }

        let rself = pyself.get();
        rself.water_hi.store(wh, atomic::Ordering::Relaxed);
        rself.water_lo.store(wl, atomic::Ordering::Relaxed);
        TCPTransport::try_proto_pause(pyself, py);
        Ok(())
    }

    fn get_write_buffer_size(&self) -> usize {
        let mut size = 0;
        for item in self.buffer.iter() {
            size += item.len();
        }
        size
    }

    fn get_write_buffer_limits(&self) -> (usize, usize) {
        (
            self.water_lo.load(atomic::Ordering::Relaxed),
            self.water_hi.load(atomic::Ordering::Relaxed),
        )
    }

    fn write(&self, data: Cow<[u8]>) {
        todo!()
    }

    fn writelines(&self, py: Python, data: &Bound<PyAny>) -> PyResult<()> {
        let pybytes = PyBytes::new(py, &[0; 0]);
        let pybytesj = pybytes.call_method1(pyo3::intern!(py, "join"), (data,))?;
        let bytes = pybytesj.extract::<Cow<[u8]>>()?;
        self.write(bytes);
        Ok(())
    }

    fn write_eof(&self) {
        if self.closing.load(atomic::Ordering::Relaxed) {
            return;
        }
        if self
            .weof
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        if self.buffer.len() == 0 {
            let stream = self.stream.lock().unwrap();
            let _ = stream.shutdown(std::net::Shutdown::Write);
        }
    }

    fn can_write_eof(&self) -> bool {
        true
    }

    fn abort(&self) {
        todo!()
    }
}

#[pyclass(frozen)]
struct TCPTransportOnEof {
    transport: Py<TCPTransport>,
}

#[pymethods]
impl TCPTransportOnEof {
    fn __call__(&self, py: Python) {
        TCPTransport::cb_eof(self.transport.clone_ref(py), py);
    }
}
