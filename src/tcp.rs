#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Result;
use mio::{
    event::Event,
    net::{TcpListener, TcpStream},
    Interest,
};
use pyo3::{buffer::PyBuffer, prelude::*, types::PyBytes};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    sync::{atomic, Arc},
};

use crate::{
    event_loop::EventLoop,
    handles::{CBHandle, Handle, HandleRef},
    log::LogExc,
    py::{asyncio_proto_buf, copy_context},
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
        let sock = unsafe { socket2::Socket::from_raw_fd(self.fd) };
        sock.listen(self.backlog)?;

        let stdl: std::net::TcpListener = sock.into();
        let listener = TcpListener::from_std(stdl);
        let sref = TCPServerRef {
            fd: self.fd as usize,
            pyloop: pyloop.clone_ref(py),
            sfamily: self.sfamily,
            proto_factory: self.protocol_factory.clone_ref(py),
        };
        pyloop.get().tcp_listener_add(listener, sref);

        Ok(())
    }

    pub(crate) fn close(&self, py: Python, event_loop: &EventLoop) {
        self.streams_abort(py, event_loop);
        _ = event_loop.tcp_listener_rem(self.fd as usize);
        // if closed {}
        // Ok(())
    }

    pub(crate) fn streams_close(&self, py: Python, event_loop: &EventLoop) {
        let mut transports = Vec::new();
        event_loop.with_tcp_listener_streams(self.fd as usize, |streams| {
            for stream_fd in streams {
                event_loop.with_tcp_stream(*stream_fd, |stream| {
                    transports.push(stream.pytransport.clone_ref(py));
                });
            }
        });
        for transport in transports {
            transport.get().close(py);
        }
    }

    pub(crate) fn streams_abort(&self, py: Python, event_loop: &EventLoop) {
        let mut transports = Vec::new();
        event_loop.with_tcp_listener_streams(self.fd as usize, |streams| {
            for stream_fd in streams {
                event_loop.with_tcp_stream(*stream_fd, |stream| {
                    transports.push(stream.pytransport.clone_ref(py));
                });
            }
        });
        for transport in transports {
            transport.get().abort(py);
        }
    }
}

pub(crate) struct TCPServerRef {
    pub fd: usize,
    pyloop: Py<EventLoop>,
    sfamily: i32,
    proto_factory: PyObject,
}

impl TCPServerRef {
    #[inline]
    pub(crate) fn new_stream(&self, py: Python, stream: TcpStream) -> (TCPStream, HandleRef) {
        let proto = self.proto_factory.bind(py).call0().unwrap();
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
        let pyproto = proto.unbind();
        let pytransport = PyTCPTransport::new(
            py,
            stream.as_raw_fd() as usize,
            self.sfamily,
            self.pyloop.clone_ref(py),
            pyproto.clone_ref(py),
        );
        let conn_handle = CBHandle::new1(
            pyproto.getattr(py, pyo3::intern!(py, "connection_made")).unwrap(),
            pytransport.clone_ref(py).into_any(),
            copy_context(py),
        );

        (
            TCPStream::new(
                self.fd,
                stream,
                pytransport.into(),
                buffered_proto,
                pym_recv_data.into(),
                pym_buf_get,
            ),
            Arc::new(conn_handle),
        )
    }
}

pub(crate) struct TCPStream {
    pub lfd: usize,
    pub io: TcpStream,
    pub pytransport: Arc<Py<PyTCPTransport>>,
    read_buffered: bool,
    write_buffer: VecDeque<Box<[u8]>>,
    pym_recv_data: Arc<PyObject>,
    pym_buf_get: PyObject,
}

impl TCPStream {
    fn new(
        lfd: usize,
        stream: TcpStream,
        pytransport: Arc<Py<PyTCPTransport>>,
        read_buffered: bool,
        pym_recv_data: Arc<PyObject>,
        pym_buf_get: PyObject,
    ) -> Self {
        Self {
            lfd,
            io: stream,
            pytransport,
            read_buffered,
            write_buffer: VecDeque::new(),
            pym_recv_data,
            pym_buf_get,
        }
    }
}

impl TCPStream {
    pub(crate) fn recv(&mut self, event: &Event, fd: usize) -> HandleRef {
        // TODO: set eof in transport?
        if event.is_read_closed() {
            Arc::new(TCPHandleRecvEof {
                transport: self.pytransport.clone(),
                fd,
                send_buf_empty: self.write_buffer.is_empty(),
            })
        } else {
            match self.read_buffered {
                true => self.recv_buffered(fd),
                false => self.recv_direct(fd),
            }
        }
    }

    pub(crate) fn send(&mut self, event: &Event, fd: usize) -> HandleRef {
        // println!("tcp stream send {:?}", transport.fd);
        if event.is_write_closed() {
            self.write_buffer.clear();
            Arc::new(TCPHandleSend {
                transport: self.pytransport.clone(),
                fd,
                written: Some(0),
                send_buf_empty: true,
            })
        } else {
            match self.write() {
                Some(written) => Arc::new(TCPHandleSend {
                    transport: self.pytransport.clone(),
                    fd,
                    written: Some(written),
                    send_buf_empty: self.write_buffer.is_empty(),
                }),
                _ => Arc::new(TCPHandleSend {
                    transport: self.pytransport.clone(),
                    fd,
                    written: None,
                    send_buf_empty: true,
                }),
            }
        }
    }

    #[inline]
    fn recv_direct(&mut self, fd: usize) -> HandleRef {
        let data = self.read();
        if data.len() == 0 {
            return Arc::new(TCPHandleRecvEof {
                transport: self.pytransport.clone(),
                fd,
                send_buf_empty: self.write_buffer.is_empty(),
            });
        }
        Arc::new(TCPHandleRecv {
            cb: self.pym_recv_data.clone(),
            data,
        })
    }

    #[inline]
    fn recv_buffered(&mut self, fd: usize) -> HandleRef {
        // NOTE: `PuBuffer.as_mut_slice` exists, but it returns a slice of `Cell<u8>`,
        //       which is smth we can't really use to read from `TcpStream`.
        //       So even if this sucks, we copy data back and forth, at least until
        //       we figure out a way to actually use `PyBuffer` directly.
        // let (mut eofr, mut eofw) = (false, false);
        let (pybuf, mut vbuf) = Python::with_gil(|py| {
            let pybuf: PyBuffer<u8> = PyBuffer::get(&self.pym_buf_get.bind(py).call1((-1,)).unwrap()).unwrap();
            let vbuf = pybuf.to_vec(py).unwrap();
            (pybuf, vbuf)
        });

        let read = self.read_into(vbuf.as_mut_slice());
        if read == 0 {
            return Arc::new(TCPHandleRecvEof {
                transport: self.pytransport.clone(),
                fd,
                send_buf_empty: self.write_buffer.is_empty(),
            });
        }
        _ = Python::with_gil(|py| pybuf.copy_from_slice(py, &vbuf[..]));
        Arc::new(TCPHandleRecvBuf {
            cb: self.pym_recv_data.clone(),
            data: read,
        })
    }

    #[inline]
    fn read(&mut self) -> Box<[u8]> {
        let mut len = 0;
        let mut buf = [0; 262_144];
        loop {
            match self.io.read(&mut buf[len..]) {
                Ok(readn) if readn != 0 => {
                    len += readn;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                _ => break,
            }
        }
        buf[..len].into()
    }

    #[inline]
    fn read_into(&mut self, buf: &mut [u8]) -> usize {
        let mut len = 0;
        loop {
            match self.io.read(&mut buf[len..]) {
                Ok(readn) if readn != 0 => {
                    len += readn;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                _ => break,
            }
        }
        len
    }

    #[inline]
    fn write(&mut self) -> Option<usize> {
        let mut ret = 0;
        while let Some(data) = self.write_buffer.pop_front() {
            match self.io.write(&data) {
                Ok(written) if written != data.len() => {
                    self.write_buffer.push_front((&data[written..]).into());
                    ret += written;
                    break;
                }
                Ok(written) => ret += written,
                Err(err) if err.kind() != std::io::ErrorKind::Interrupted => {
                    self.write_buffer.clear();
                    return None;
                }
                _ => {
                    self.write_buffer.push_front(data);
                    break;
                }
            }
        }
        Some(ret)
    }
}

#[pyclass(frozen)]
pub(crate) struct PyTCPTransport {
    pub fd: usize,
    extra: HashMap<String, PyObject>,
    sock: Py<SocketWrapper>,
    pyloop: Py<EventLoop>,
    proto: PyObject,
    closing: atomic::AtomicBool,
    paused: atomic::AtomicBool,
    paused_proto: atomic::AtomicBool,
    water_hi: atomic::AtomicUsize,
    water_lo: atomic::AtomicUsize,
    weof: atomic::AtomicBool,
    write_buf_size: atomic::AtomicUsize,
    pym_conn_lost: PyObject,
}

impl PyTCPTransport {
    fn new(py: Python, fd: usize, socket_family: i32, pyloop: Py<EventLoop>, proto: PyObject) -> Py<Self> {
        let wh = 1024 * 64;
        let wl = wh / 4;
        let pym_conn_lost = proto.getattr(py, pyo3::intern!(py, "connection_lost")).unwrap();

        Py::new(
            py,
            Self {
                fd,
                extra: HashMap::new(),
                sock: SocketWrapper::from_fd(py, fd, socket_family, socket2::Type::STREAM, 0),
                pyloop,
                proto,
                closing: false.into(),
                paused: false.into(),
                paused_proto: false.into(),
                water_hi: wh.into(),
                water_lo: wl.into(),
                weof: false.into(),
                write_buf_size: 0.into(),
                pym_conn_lost,
            },
        )
        .unwrap()
    }

    #[inline]
    fn write_buf_size_decr(pyself: &Py<Self>, py: Python, val: usize) {
        // println!("tcp write_buf_size_decr {:?}", val);
        let rself = pyself.get();
        let buf_size = rself.write_buf_size.fetch_sub(val, atomic::Ordering::Release) - val;
        if buf_size <= rself.water_lo.load(atomic::Ordering::Relaxed)
            && rself
                .paused_proto
                .compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed)
                .is_ok()
        {
            Self::proto_resume(pyself, py);
        }
    }

    #[inline]
    fn write_buf_eof(&self, py: Python, errored: bool) -> bool {
        if self.closing.load(atomic::Ordering::Relaxed) {
            self.call_conn_lost(
                py,
                errored.then(|| pyo3::exceptions::PyRuntimeError::new_err("socket transport failed")),
            );
            return false;
        }
        self.weof.load(atomic::Ordering::Relaxed)
    }

    #[inline]
    fn call_conn_lost(&self, py: Python, err: Option<PyErr>) {
        _ = self.pym_conn_lost.call1(py, (err,));
        self.pyloop.get().tcp_stream_close(self.fd);
    }

    fn try_write(pyself: &Py<Self>, py: Python, data: &[u8]) -> PyResult<()> {
        // println!("tcp try_write");

        let rself = pyself.get();
        if rself.weof.load(atomic::Ordering::Relaxed) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err("Cannot write after EOF"));
        }
        if data.is_empty() {
            return Ok(());
        }

        // needed?
        // if rself._conn_lost:
        //     if rself._conn_lost >= constants.LOG_THRESHOLD_FOR_CONNLOST_WRITES:
        //         logger.warning('socket.send() raised exception.')
        //     rself._conn_lost += 1
        //     return

        let buf_added = py.allow_threads(|| {
            match rself.write_buf_size.load(atomic::Ordering::Relaxed) {
                0 => rself.pyloop.get().with_tcp_stream(rself.fd, |stream| {
                    match stream.io.write(data) {
                        Ok(written) if written == data.len() => 0,
                        Ok(written) => {
                            stream.write_buffer.push_back((&data[written..]).into());
                            data.len() - written
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                            stream.write_buffer.push_back(data.into());
                            data.len()
                        }
                        Err(_) => {
                            // TODO: log exc
                            0
                        }
                    }
                }),
                _ => rself.pyloop.get().with_tcp_stream(rself.fd, |stream| {
                    stream.write_buffer.push_back(data.into());
                    data.len()
                }),
            }
        });
        if buf_added > 0 {
            let buf_size = rself.write_buf_size.fetch_add(buf_added, atomic::Ordering::Release) + buf_added;
            if buf_size > rself.water_hi.load(atomic::Ordering::Relaxed)
                && rself
                    .paused_proto
                    .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
                    .is_ok()
            {
                Self::proto_pause(pyself, py);
            }
        }

        Ok(())
    }

    fn proto_pause(pyself: &Py<Self>, py: Python) {
        // println!("tcp proto_pause");
        let rself = pyself.get();
        if let Err(err) = rself.proto.call_method0(py, pyo3::intern!(py, "pause_writing")) {
            let err_ctx = LogExc::transport(
                err,
                "protocol.pause_writing() failed".into(),
                rself.proto.clone_ref(py),
                pyself.clone_ref(py).into_any(),
            );
            _ = rself.pyloop.get().log_exception(py, err_ctx);
        }
    }

    fn proto_resume(pyself: &Py<Self>, py: Python) {
        // println!("tcp proto_resume");
        let rself = pyself.get();
        if let Err(err) = rself.proto.call_method0(py, pyo3::intern!(py, "resume_writing")) {
            let err_ctx = LogExc::transport(
                err,
                "protocol.resume_writing() failed".into(),
                rself.proto.clone_ref(py),
                pyself.clone_ref(py).into_any(),
            );
            _ = rself.pyloop.get().log_exception(py, err_ctx);
        }
    }
}

#[pymethods]
impl PyTCPTransport {
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

    fn close(&self, py: Python) {
        // println!("tcp close");
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let event_loop = self.pyloop.get();
        event_loop.tcp_stream_rem(self.fd, Interest::READABLE);
        if self.write_buf_size.load(atomic::Ordering::Relaxed) == 0 {
            // TODO: set conn lost?
            event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
            self.call_conn_lost(py, None);
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

    fn pause_reading(&self) {
        if self.closing.load(atomic::Ordering::Relaxed) {
            return;
        }
        if self
            .paused
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        self.pyloop.get().tcp_stream_rem(self.fd, Interest::READABLE);
    }

    fn resume_reading(&self) {
        if self.closing.load(atomic::Ordering::Relaxed) {
            return;
        }
        if self
            .paused
            .compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        self.pyloop.get().tcp_stream_add(self.fd, Interest::READABLE);
    }

    #[pyo3(signature = (high = None, low = None))]
    fn set_write_buffer_limits(pyself: Py<Self>, py: Python, high: Option<usize>, low: Option<usize>) -> PyResult<()> {
        let wh = match high {
            None => match low {
                None => 1024 * 64,
                Some(v) => v * 4,
            },
            Some(v) => v,
        };
        let wl = match low {
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

        if rself.write_buf_size.load(atomic::Ordering::Relaxed) > wh
            && rself
                .paused_proto
                .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
                .is_ok()
        {
            Self::proto_pause(&pyself, py);
        }

        Ok(())
    }

    fn get_write_buffer_size(&self) -> usize {
        self.write_buf_size.load(atomic::Ordering::Relaxed)
    }

    fn get_write_buffer_limits(&self) -> (usize, usize) {
        (
            self.water_lo.load(atomic::Ordering::Relaxed),
            self.water_hi.load(atomic::Ordering::Relaxed),
        )
    }

    fn write(pyself: Py<Self>, py: Python, data: Cow<[u8]>) -> PyResult<()> {
        Self::try_write(&pyself, py, &data)
    }

    fn writelines(pyself: Py<Self>, py: Python, data: &Bound<PyAny>) -> PyResult<()> {
        let pybytes = PyBytes::new(py, &[0; 0]);
        let pybytesj = pybytes.call_method1(pyo3::intern!(py, "join"), (data,))?;
        let bytes = pybytesj.extract::<Cow<[u8]>>()?;
        Self::try_write(&pyself, py, &bytes)
    }

    fn write_eof(&self) {
        // println!("tcp write_eof");
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

        if self.write_buf_size.load(atomic::Ordering::Relaxed) == 0 {
            self.pyloop.get().tcp_stream_shutdown(self.fd);
        }
    }

    fn can_write_eof(&self) -> bool {
        true
    }

    fn abort(&self, py: Python) {
        if self.write_buf_size.load(atomic::Ordering::Relaxed) > 0 {
            self.pyloop.get().tcp_stream_rem(self.fd, Interest::WRITABLE);
        }
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_ok()
        {
            self.pyloop.get().tcp_stream_rem(self.fd, Interest::READABLE);
        }
        self.call_conn_lost(py, None);
    }
}

// impl Drop for PyTCPTransport {
//     fn drop(&mut self) {
//         println!("PyTCPTransport drop");
//     }
// }

struct TCPHandleRecv {
    cb: Arc<PyObject>,
    data: Box<[u8]>,
}

impl Handle for TCPHandleRecv {
    fn run(self: Arc<Self>, py: Python, _event_loop: &EventLoop) {
        // TODO handle error
        _ = self.cb.call1(py, (&self.data[..],));
    }
}

struct TCPHandleRecvBuf {
    cb: Arc<PyObject>,
    data: usize,
}

impl Handle for TCPHandleRecvBuf {
    fn run(self: Arc<Self>, py: Python, _event_loop: &EventLoop) {
        _ = self.cb.call1(py, (self.data,));
    }
}

struct TCPHandleRecvEof {
    transport: Arc<Py<PyTCPTransport>>,
    fd: usize,
    send_buf_empty: bool,
}

impl Handle for TCPHandleRecvEof {
    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop) {
        // println!("TCPHandleRecvEof {:?}", self.fd);
        event_loop.tcp_stream_rem(self.fd, Interest::READABLE);
        if let Ok(pyr) = self
            .transport
            .get()
            .proto
            .call_method0(py, pyo3::intern!(py, "eof_received"))
        {
            // println!("recv_eof res {:?}", pyr);
            if let Ok(false) = pyr.is_truthy(py) {
                if !self.send_buf_empty {
                    return;
                }
            }
        }
        // println!("TCPHandleRecvEof {:?} dropw", self.fd);
        event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
        // shudtdown?
    }
}

struct TCPHandleSend {
    transport: Arc<Py<PyTCPTransport>>,
    fd: usize,
    written: Option<usize>,
    send_buf_empty: bool,
}

impl Handle for TCPHandleSend {
    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop) {
        if self.send_buf_empty {
            event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
            if self.transport.get().write_buf_eof(py, self.written.is_none()) {
                event_loop.tcp_stream_shutdown(self.fd);
            }
            return;
        }
        PyTCPTransport::write_buf_size_decr(&self.transport, py, self.written.unwrap());
    }
}
