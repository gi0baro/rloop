#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Result;
use mio::net::{TcpListener, TcpStream};
use pyo3::{
    buffer::PyBuffer,
    prelude::*,
    types::{PyBytes, PyTuple},
    IntoPyObjectExt,
};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    io::Read,
    sync::{atomic, Arc, Mutex},
};

use crate::{
    event_loop::EventLoop,
    log::LogExc,
    py::{asyncio_proto_buf, copy_context, run_in_ctx0},
};

pub(crate) struct TCPServer {
    fd: i32,
    backlog: i32,
    protocol_factory: PyObject,
}

impl TCPServer {
    pub(crate) fn from_fd(fd: i32, backlog: i32, protocol_factory: PyObject) -> Self {
        Self {
            fd,
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

// #[pyclass(frozen)]
pub(crate) struct TCPServerRef {
    pyloop: Py<EventLoop>,
    pub listener: Arc<TcpListener>,
    proto_factory: PyObject,
}

impl TCPServerRef {
    pub(crate) fn transport(&self, py: Python, stream: TcpStream) -> TCPTransport {
        TCPTransport::new(py, stream, self.pyloop.clone_ref(py), self.proto_factory.clone_ref(py))
    }
}

#[pyclass(frozen)]
pub(crate) struct TCPTransport {
    pub stream: Arc<Mutex<TcpStream>>,
    pub fd: usize,
    pyloop: Py<EventLoop>,
    proto: PyObject,
    extra: HashMap<String, PyObject>,
    buffered_proto: bool,
    closing: atomic::AtomicBool,
    reading: atomic::AtomicBool,
    paused_proto: atomic::AtomicBool,
    buffer: VecDeque<Vec<u8>>,
    water_hi: atomic::AtomicUsize,
    water_lo: atomic::AtomicUsize,
    pym_conn_made: PyObject,
    pym_recv_data: PyObject,
    pym_recv_eof: PyObject,
    pym_buf_get: PyObject,
    pym_write_pause: PyObject,
    pym_write_resume: PyObject,
}

impl TCPTransport {
    pub(crate) fn new(py: Python, stream: TcpStream, pyloop: Py<EventLoop>, proto_factory: PyObject) -> Self {
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
        let pym_recv_eof = proto.getattr(pyo3::intern!(py, "eof_received")).unwrap().unbind();
        let pym_write_pause = proto.getattr(pyo3::intern!(py, "pause_writing")).unwrap().unbind();
        let pym_write_resume = proto.getattr(pyo3::intern!(py, "resume_writing")).unwrap().unbind();

        let wh = 1024 * 64;
        let wl = wh / 4;

        Self {
            stream: Arc::new(Mutex::new(stream)),
            fd,
            pyloop,
            proto: proto.unbind(),
            extra: HashMap::new(),
            buffered_proto,
            closing: false.into(),
            reading: false.into(),
            paused_proto: false.into(),
            buffer: VecDeque::new(),
            water_hi: wh.into(),
            water_lo: wl.into(),
            pym_conn_made,
            pym_recv_data,
            pym_recv_eof,
            pym_buf_get,
            pym_write_pause,
            pym_write_resume,
        }
    }

    pub(crate) fn attach(&self, py: Python) -> (PyObject, PyObject) {
        (self.pym_conn_made.clone_ref(py), PyTuple::empty(py).into_any().unbind())
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

        let pyctx = copy_context(py).unwrap().as_ptr();
        let cb = rself.pym_write_pause.as_ptr();
        let res = run_in_ctx0!(py, pyctx, cb);
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

        let pyctx = copy_context(py).unwrap().as_ptr();
        let cb = rself.pym_write_resume.as_ptr();
        let res = run_in_ctx0!(py, pyctx, cb);
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

    pub(crate) fn recv(&self, py: Python) -> (PyObject, PyObject) {
        match self.buffered_proto {
            true => {
                // NOTE: `PuBuffer.as_mut_slice` exists, but it returns a slice of `Cell<u8>`,
                //       which is smth we can't really use to read from `TcpStream`.
                //       So even if this sucks, we copy data back and forth, at least until
                //       we figure out a way to actually use `PyBuffer` directly.
                let pybuf: PyBuffer<u8> = PyBuffer::get(&self.pym_buf_get.bind(py).call0().unwrap()).unwrap();
                let mut vbuf = pybuf.to_vec(py).unwrap();
                let read = py.allow_threads(|| self.read_into(vbuf.as_mut_slice()));
                pybuf.copy_from_slice(py, &vbuf[..]).unwrap();
                let cb = self.pym_recv_data.clone_ref(py);
                let args = (read,).into_py_any(py).unwrap();
                (cb, args)
            }
            false => {
                let data = py.allow_threads(|| self.read());
                let cb: PyObject;
                let args: PyObject;
                if data.len() == 0 {
                    cb = self.pym_recv_eof.clone_ref(py);
                    args = PyTuple::empty(py).into_any().unbind();
                } else {
                    cb = self.pym_recv_data.clone_ref(py);
                    args = (&data[..],).into_py_any(py).unwrap();
                }
                (cb, args)
            }
        }
    }

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

    // fn write2(&self) {}

    // fn force_close(&self) {}
}

#[pymethods]
impl TCPTransport {
    #[pyo3(signature = (name, default = None))]
    fn get_extra_info(&self, py: Python, name: &str, default: Option<PyObject>) -> Option<PyObject> {
        self.extra.get(name).map(|v| v.clone_ref(py)).or(default)
    }

    fn is_closing(&self) -> bool {
        self.closing.load(atomic::Ordering::Relaxed)
    }

    fn close(&self) {
        todo!()
    }

    fn set_protocol(&self, _protocol: PyObject) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(""))
    }

    fn get_protocol(&self, py: Python) -> PyObject {
        self.proto.clone_ref(py)
    }

    fn is_reading(&self) -> bool {
        self.reading.load(atomic::Ordering::Relaxed)
    }

    fn pause_reading(&self) {
        todo!()
    }

    fn resume_reading(&self) {
        todo!()
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
        todo!()
    }

    fn can_write_eof(&self) -> bool {
        true
    }

    fn abort(&self) {
        todo!()
    }
}

// pub(super) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
//     // module.add_class::<TCPServer>()?;

//     Ok(())
// }
