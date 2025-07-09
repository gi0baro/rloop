#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

use anyhow::Result;
use mio::{
    Interest,
    net::{TcpListener, TcpStream},
};
use pyo3::{IntoPyObjectExt, buffer::PyBuffer, prelude::*, types::PyBytes};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::Read,
    sync::atomic,
};

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    handles::{BoxedHandle, CBHandle, Handle},
    log::LogExc,
    py::{asyncio_proto_buf, copy_context},
    sock::SocketWrapper,
    utils::syscall,
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
            for stream_fd in &streams.pin() {
                transports.push(event_loop.get_tcp_transport(*stream_fd, py));
            }
        });
        for transport in transports {
            transport.borrow(py).close(py);
        }
    }

    pub(crate) fn streams_abort(&self, py: Python, event_loop: &EventLoop) {
        let mut transports = Vec::new();
        event_loop.with_tcp_listener_streams(self.fd as usize, |streams| {
            for stream_fd in &streams.pin() {
                transports.push(event_loop.get_tcp_transport(*stream_fd, py));
            }
        });
        for transport in transports {
            transport.borrow(py).abort(py);
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
    pub(crate) fn new_stream(&self, py: Python, stream: TcpStream) -> (Py<TCPTransport>, BoxedHandle) {
        let proto = self.proto_factory.bind(py).call0().unwrap();

        let transport = TCPTransport::new(
            py,
            self.pyloop.clone_ref(py),
            stream,
            proto,
            self.sfamily,
            Some(self.fd),
        );
        let conn_made = transport
            .proto
            .getattr(py, pyo3::intern!(py, "connection_made"))
            .unwrap();
        let pytransport = Py::new(py, transport).unwrap();
        let conn_handle = Py::new(
            py,
            CBHandle::new1(conn_made, pytransport.clone_ref(py).into_any(), copy_context(py)),
        )
        .unwrap();

        (pytransport, Box::new(conn_handle))
    }
}
struct TCPTransportState {
    stream: TcpStream,
    write_buf: VecDeque<Box<[u8]>>,
    write_buf_dsize: usize,
}

#[pyclass(frozen, unsendable, module = "rloop._rloop")]
pub(crate) struct TCPTransport {
    pub fd: usize,
    pub lfd: Option<usize>,
    state: RefCell<TCPTransportState>,
    pyloop: Py<EventLoop>,
    // atomics
    closing: atomic::AtomicBool,
    paused: atomic::AtomicBool,
    water_hi: atomic::AtomicUsize,
    water_lo: atomic::AtomicUsize,
    weof: atomic::AtomicBool,
    // py protocol fields
    proto: PyObject,
    proto_buffered: bool,
    proto_paused: atomic::AtomicBool,
    protom_buf_get: PyObject,
    protom_conn_lost: PyObject,
    protom_recv_data: PyObject,
    // py extras
    extra: HashMap<String, PyObject>,
    sock: Py<SocketWrapper>,
}

impl TCPTransport {
    fn new(
        py: Python,
        pyloop: Py<EventLoop>,
        stream: TcpStream,
        pyproto: Bound<PyAny>,
        socket_family: i32,
        lfd: Option<usize>,
    ) -> Self {
        let fd = stream.as_raw_fd() as usize;
        let state = TCPTransportState {
            stream,
            write_buf: VecDeque::new(),
            write_buf_dsize: 0,
        };

        let wh = 1024 * 64;
        let wl = wh / 4;

        let mut proto_buffered = false;
        let protom_buf_get: PyObject;
        let protom_recv_data: PyObject;
        if pyproto.is_instance(asyncio_proto_buf(py).unwrap()).unwrap() {
            proto_buffered = true;
            protom_buf_get = pyproto.getattr(pyo3::intern!(py, "get_buffer")).unwrap().unbind();
            protom_recv_data = pyproto.getattr(pyo3::intern!(py, "buffer_updated")).unwrap().unbind();
        } else {
            protom_buf_get = py.None();
            protom_recv_data = pyproto.getattr(pyo3::intern!(py, "data_received")).unwrap().unbind();
        }
        let protom_conn_lost = pyproto.getattr(pyo3::intern!(py, "connection_lost")).unwrap().unbind();
        let proto = pyproto.unbind();

        Self {
            fd,
            lfd,
            state: RefCell::new(state),
            pyloop,
            closing: false.into(),
            paused: false.into(),
            water_hi: wh.into(),
            water_lo: wl.into(),
            weof: false.into(),
            proto,
            proto_buffered,
            proto_paused: false.into(),
            protom_buf_get,
            protom_conn_lost,
            protom_recv_data,
            extra: HashMap::new(),
            sock: SocketWrapper::from_fd(py, fd, socket_family, socket2::Type::STREAM, 0),
        }
    }

    pub(crate) fn from_py(py: Python, pyloop: &Py<EventLoop>, pysock: (i32, i32), proto_factory: PyObject) -> Self {
        let sock = unsafe { socket2::Socket::from_raw_fd(pysock.0) };
        _ = sock.set_nonblocking(true);
        let stdl: std::net::TcpStream = sock.into();
        let stream = TcpStream::from_std(stdl);

        let proto = proto_factory.bind(py).call0().unwrap();

        Self::new(py, pyloop.clone_ref(py), stream, proto, pysock.1, None)
    }

    pub(crate) fn attach(pyself: &Py<Self>, py: Python) -> PyResult<PyObject> {
        let rself = pyself.borrow(py);
        rself
            .proto
            .call_method1(py, pyo3::intern!(py, "connection_made"), (pyself.clone_ref(py),))?;
        Ok(rself.proto.clone_ref(py))
    }

    #[inline]
    fn write_buf_size_decr(pyself: &Py<Self>, py: Python) {
        let rself = pyself.borrow(py);
        if rself.state.borrow().write_buf_dsize <= rself.water_lo.load(atomic::Ordering::Relaxed)
            && rself
                .proto_paused
                .compare_exchange(true, false, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
                .is_ok()
        {
            Self::proto_resume(pyself, py);
        }
    }

    #[inline]
    fn close_from_read_handle(&self, py: Python, event_loop: &EventLoop) -> bool {
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_err()
        {
            return false;
        }

        if !self.state.borrow_mut().write_buf.is_empty() {
            return false;
        }

        event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
        _ = self.protom_conn_lost.call1(py, (py.None(),));
        true
    }

    #[inline]
    fn close_from_write_handle(&self, py: Python, errored: bool) -> Option<bool> {
        if self.closing.load(atomic::Ordering::Relaxed) {
            _ = self.protom_conn_lost.call1(
                py,
                #[allow(clippy::obfuscated_if_else)]
                (errored
                    .then(|| {
                        pyo3::exceptions::PyRuntimeError::new_err("socket transport failed")
                            .into_py_any(py)
                            .unwrap()
                    })
                    .unwrap_or_else(|| py.None()),),
            );
            return Some(true);
        }
        self.weof.load(atomic::Ordering::Relaxed).then_some(false)
    }

    #[inline(always)]
    fn call_conn_lost(&self, py: Python, err: Option<PyErr>) {
        _ = self.protom_conn_lost.call1(py, (err,));
        self.pyloop.get().tcp_stream_close(py, self.fd);
    }

    fn try_write(pyself: &Py<Self>, py: Python, data: &[u8]) -> PyResult<()> {
        let rself = pyself.borrow(py);

        if rself.weof.load(atomic::Ordering::Relaxed) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err("Cannot write after EOF"));
        }
        if data.is_empty() {
            return Ok(());
        }

        let mut state = rself.state.borrow_mut();
        let buf_added = match state.write_buf_dsize {
            #[allow(clippy::cast_possible_wrap)]
            0 => match syscall!(write(rself.fd as i32, data.as_ptr().cast(), data.len())) {
                Ok(written) if written as usize == data.len() => 0,
                Ok(written) => {
                    let written = written as usize;
                    state.write_buf.push_back((&data[written..]).into());
                    data.len() - written
                }
                Err(err)
                    if err.kind() == std::io::ErrorKind::Interrupted
                        || err.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    state.write_buf.push_back(data.into());
                    data.len()
                }
                Err(err) => {
                    if state.write_buf_dsize > 0 {
                        // reset buf_dsize?
                        rself.pyloop.get().tcp_stream_rem(rself.fd, Interest::WRITABLE);
                    }
                    if rself
                        .closing
                        .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
                        .is_ok()
                    {
                        rself.pyloop.get().tcp_stream_rem(rself.fd, Interest::READABLE);
                    }
                    rself.call_conn_lost(py, Some(pyo3::exceptions::PyRuntimeError::new_err(err.to_string())));
                    0
                }
            },
            _ => {
                state.write_buf.push_back(data.into());
                data.len()
            }
        };
        if buf_added > 0 {
            if state.write_buf_dsize == 0 {
                rself.pyloop.get().tcp_stream_add(rself.fd, Interest::WRITABLE);
            }
            state.write_buf_dsize += buf_added;
            if state.write_buf_dsize > rself.water_hi.load(atomic::Ordering::Relaxed)
                && rself
                    .proto_paused
                    .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
                    .is_ok()
            {
                Self::proto_pause(pyself, py);
            }
        }

        Ok(())
    }

    fn proto_pause(pyself: &Py<Self>, py: Python) {
        let rself = pyself.borrow(py);
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
        let rself = pyself.borrow(py);
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
impl TCPTransport {
    #[pyo3(signature = (name, default = None))]
    fn get_extra_info(&self, py: Python, name: &str, default: Option<PyObject>) -> Option<PyObject> {
        match name {
            "socket" => Some(self.sock.clone_ref(py).into_any()),
            "sockname" => self.sock.call_method0(py, pyo3::intern!(py, "getsockname")).ok(),
            "peername" => self.sock.call_method0(py, pyo3::intern!(py, "getpeername")).ok(),
            _ => self.extra.get(name).map(|v| v.clone_ref(py)).or(default),
        }
    }

    fn is_closing(&self) -> bool {
        self.closing.load(atomic::Ordering::Relaxed)
    }

    fn close(&self, py: Python) {
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let event_loop = self.pyloop.get();
        event_loop.tcp_stream_rem(self.fd, Interest::READABLE);
        if self.state.borrow().write_buf_dsize == 0 {
            // set conn lost?
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
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
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
            .compare_exchange(true, false, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
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

        let rself = pyself.borrow(py);
        rself.water_hi.store(wh, atomic::Ordering::Relaxed);
        rself.water_lo.store(wl, atomic::Ordering::Relaxed);

        if rself.state.borrow().write_buf_dsize > wh
            && rself
                .proto_paused
                .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
                .is_ok()
        {
            Self::proto_pause(&pyself, py);
        }

        Ok(())
    }

    fn get_write_buffer_size(&self) -> usize {
        self.state.borrow().write_buf_dsize
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
        if self.closing.load(atomic::Ordering::Relaxed) {
            return;
        }
        if self
            .weof
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let state = self.state.borrow();
        if state.write_buf_dsize == 0 {
            _ = state.stream.shutdown(std::net::Shutdown::Write);
        }
    }

    fn can_write_eof(&self) -> bool {
        true
    }

    fn abort(&self, py: Python) {
        if self.state.borrow().write_buf_dsize > 0 {
            self.pyloop.get().tcp_stream_rem(self.fd, Interest::WRITABLE);
        }
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_ok()
        {
            self.pyloop.get().tcp_stream_rem(self.fd, Interest::READABLE);
        }
        self.call_conn_lost(py, None);
    }
}

pub(crate) struct TCPReadHandle {
    pub fd: usize,
}

impl TCPReadHandle {
    #[inline]
    fn recv_direct(&self, py: Python, transport: &TCPTransport, buf: &mut [u8]) -> (Option<PyObject>, bool) {
        let (read, closed) = self.read_into(&mut transport.state.borrow_mut().stream, buf);
        if read > 0 {
            let rbuf = &buf[..read];
            let pydata = unsafe { PyBytes::from_ptr(py, rbuf.as_ptr(), read) };
            return (Some(pydata.into_any().unbind()), closed);
        }
        (None, closed)
    }

    #[inline]
    fn recv_buffered(&self, py: Python, transport: &TCPTransport) -> (Option<PyObject>, bool) {
        // NOTE: `PuBuffer.as_mut_slice` exists, but it returns a slice of `Cell<u8>`,
        //       which is smth we can't really use to read from `TcpStream`.
        //       So even if this sucks, we copy data back and forth, at least until
        //       we figure out a way to actually use `PyBuffer` directly.
        let pybuf: PyBuffer<u8> = PyBuffer::get(&transport.protom_buf_get.bind(py).call1((-1,)).unwrap()).unwrap();
        let mut vbuf = pybuf.to_vec(py).unwrap();
        let (read, closed) = self.read_into(&mut transport.state.borrow_mut().stream, vbuf.as_mut_slice());
        if read > 0 {
            _ = pybuf.copy_from_slice(py, &vbuf[..]);
            return (Some(read.into_py_any(py).unwrap()), closed);
        }
        (None, closed)
    }

    #[inline(always)]
    fn read_into(&self, stream: &mut TcpStream, buf: &mut [u8]) -> (usize, bool) {
        let mut len = 0;
        let mut closed = false;

        loop {
            match stream.read(&mut buf[len..]) {
                Ok(0) => {
                    if len < buf.len() {
                        closed = true;
                    }
                    break;
                }
                Ok(readn) => len += readn,
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                _ => break,
            }
        }

        (len, closed)
    }

    #[inline]
    fn recv_eof(&self, py: Python, event_loop: &EventLoop, transport: &TCPTransport) -> bool {
        event_loop.tcp_stream_rem(self.fd, Interest::READABLE);
        if let Ok(pyr) = transport.proto.call_method0(py, pyo3::intern!(py, "eof_received")) {
            if let Ok(true) = pyr.is_truthy(py) {
                return false;
            }
        }
        transport.close_from_read_handle(py, event_loop)
    }
}

impl Handle for TCPReadHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState) {
        let pytransport = event_loop.get_tcp_transport(self.fd, py);
        let transport = pytransport.borrow(py);

        // NOTE: we need to consume all the data coming from the socket even when it exceeds the buffer,
        //       otherwise we won't get another readable event from the poller
        let mut close = false;
        loop {
            let (data, eof) = match transport.proto_buffered {
                true => self.recv_buffered(py, &transport),
                false => self.recv_direct(py, &transport, &mut state.read_buf),
            };

            if let Some(data) = data {
                _ = transport.protom_recv_data.call1(py, (data,));
                if !eof {
                    continue;
                }
            }

            if eof {
                close = self.recv_eof(py, event_loop, &transport);
            }

            break;
        }

        if close {
            event_loop.tcp_stream_close(py, self.fd);
        }
    }
}

pub(crate) struct TCPWriteHandle {
    pub fd: usize,
}

impl TCPWriteHandle {
    #[inline]
    fn write(&self, transport: &TCPTransport) -> Option<usize> {
        #[allow(clippy::cast_possible_wrap)]
        let fd = self.fd as i32;
        let mut ret = 0;
        let mut state = transport.state.borrow_mut();
        while let Some(data) = state.write_buf.pop_front() {
            match syscall!(write(fd, data.as_ptr().cast(), data.len())) {
                Ok(written) if (written as usize) < data.len() => {
                    let written = written as usize;
                    state.write_buf.push_front((&data[written..]).into());
                    ret += written;
                    break;
                }
                Ok(written) => ret += written as usize,
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                    state.write_buf.push_front(data);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    state.write_buf.push_front(data);
                    break;
                }
                _ => {
                    state.write_buf.clear();
                    state.write_buf_dsize = 0;
                    return None;
                }
            }
        }
        state.write_buf_dsize -= ret;
        Some(ret)
    }
}

impl Handle for TCPWriteHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let pytransport = event_loop.get_tcp_transport(self.fd, py);
        let transport = pytransport.borrow(py);
        let stream_close;

        if let Some(written) = self.write(&transport) {
            if written > 0 {
                TCPTransport::write_buf_size_decr(&pytransport, py);
            }
            stream_close = match transport.state.borrow().write_buf.is_empty() {
                true => transport.close_from_write_handle(py, false),
                false => None,
            };
        } else {
            stream_close = transport.close_from_write_handle(py, true);
        }

        if transport.state.borrow().write_buf.is_empty() {
            event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
        }

        match stream_close {
            Some(true) => event_loop.tcp_stream_close(py, self.fd),
            Some(false) => {
                _ = transport.state.borrow().stream.shutdown(std::net::Shutdown::Write);
            }
            _ => {}
        }
    }
}
