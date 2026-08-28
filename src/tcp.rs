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

// PyO3 removed private CPython's APIs
unsafe extern "C" {
    fn _PyBytes_Resize(bytes: *mut *mut pyo3::ffi::PyObject, newsize: pyo3::ffi::Py_ssize_t) -> std::os::raw::c_int;
}

pub(crate) struct TCPServer {
    pub fd: i32,
    sfamily: i32,
    backlog: i32,
    protocol_factory: Py<PyAny>,
}

impl TCPServer {
    pub(crate) fn from_fd(fd: i32, sfamily: i32, backlog: i32, protocol_factory: Py<PyAny>) -> Self {
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
                if let Some(transport) = event_loop.get_tcp_transport(*stream_fd, py) {
                    transports.push(transport);
                }
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
                if let Some(transport) = event_loop.get_tcp_transport(*stream_fd, py) {
                    transports.push(transport);
                }
            }
        });
        for transport in transports {
            transport.borrow(py).abort(py);
        }
    }
}

impl Drop for TCPTransport {
    fn drop(&mut self) {
        Python::attach(|py| self.sock.get().detach_inner(py));
    }
}

pub(crate) struct TCPServerRef {
    pub fd: usize,
    pyloop: Py<EventLoop>,
    sfamily: i32,
    proto_factory: Py<PyAny>,
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

enum WriteChunkData {
    Py { _obj: Py<PyBytes> },
    Owned { _data: Box<[u8]> },
}

struct WriteChunk {
    _data: WriteChunkData,
    ptr: *const u8,
    len: usize,
    off: usize,
}

impl WriteChunk {
    #[inline]
    fn py(obj: &Bound<PyBytes>, off: usize) -> Self {
        let bytes = obj.as_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            off,
            _data: WriteChunkData::Py {
                _obj: obj.clone().unbind(),
            },
        }
    }

    #[inline]
    fn owned(data: Box<[u8]>) -> Self {
        Self {
            ptr: data.as_ptr(),
            len: data.len(),
            off: 0,
            _data: WriteChunkData::Owned { _data: data },
        }
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.len - self.off
    }

    #[inline]
    fn as_iovec(&self) -> libc::iovec {
        libc::iovec {
            iov_base: unsafe { self.ptr.add(self.off) }.cast_mut().cast(),
            iov_len: self.len - self.off,
        }
    }
}

struct TCPTransportState {
    stream: TcpStream,
    write_buf: VecDeque<WriteChunk>,
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
    conn_lost: atomic::AtomicBool,
    paused: atomic::AtomicBool,
    water_hi: atomic::AtomicUsize,
    water_lo: atomic::AtomicUsize,
    weof: atomic::AtomicBool,
    // py protocol fields
    proto: Py<PyAny>,
    proto_buffered: bool,
    proto_paused: atomic::AtomicBool,
    protom_buf_get: Py<PyAny>,
    protom_conn_lost: Py<PyAny>,
    protom_recv_data: Py<PyAny>,
    // py extras
    extra: HashMap<String, Py<PyAny>>,
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
        if socket_family == libc::AF_INET || socket_family == libc::AF_INET6 {
            _ = stream.set_nodelay(true);
        }
        let state = TCPTransportState {
            stream,
            write_buf: VecDeque::new(),
            write_buf_dsize: 0,
        };

        let wh = 1024 * 64;
        let wl = wh / 4;

        let mut proto_buffered = false;
        let protom_buf_get: Py<PyAny>;
        let protom_recv_data: Py<PyAny>;
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
            conn_lost: false.into(),
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

    pub(crate) fn from_py(py: Python, pyloop: &Py<EventLoop>, pysock: (i32, i32), proto_factory: Py<PyAny>) -> Self {
        let sock = unsafe { socket2::Socket::from_raw_fd(pysock.0) };
        _ = sock.set_nonblocking(true);
        let stdl: std::net::TcpStream = sock.into();
        let stream = TcpStream::from_std(stdl);

        let proto = proto_factory.bind(py).call0().unwrap();

        Self::new(py, pyloop.clone_ref(py), stream, proto, pysock.1, None)
    }

    pub(crate) fn attach(pyself: &Py<Self>, py: Python) -> PyResult<Py<PyAny>> {
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
        self.call_conn_lost(py, None);
        true
    }

    #[inline]
    fn close_from_write_handle(&self, py: Python, errored: bool) -> Option<bool> {
        if self.closing.load(atomic::Ordering::Relaxed) {
            self.call_conn_lost(
                py,
                errored.then(|| pyo3::exceptions::PyRuntimeError::new_err("socket transport failed")),
            );
            return Some(true);
        }
        self.weof.load(atomic::Ordering::Relaxed).then_some(false)
    }

    #[inline(always)]
    fn call_conn_lost(&self, py: Python, err: Option<PyErr>) {
        if self
            .conn_lost
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        {
            let mut state = self.state.borrow_mut();
            state.write_buf.clear();
            state.write_buf_dsize = 0;
        }
        _ = self.protom_conn_lost.call1(py, (err,));
        self.pyloop.get().tcp_stream_close(py, self.fd);
    }

    fn try_write(pyself: &Py<Self>, py: Python, data: &[u8], chunk: impl FnOnce(usize) -> WriteChunk) -> PyResult<()> {
        let rself = pyself.borrow(py);

        if rself.weof.load(atomic::Ordering::Relaxed) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err("Cannot write after EOF"));
        }
        if data.is_empty() {
            return Ok(());
        }

        let mut state = rself.state.borrow_mut();
        let mut conn_err = None;
        let buf_added = match state.write_buf_dsize {
            #[allow(clippy::cast_possible_wrap)]
            0 => match syscall!(write(rself.fd as i32, data.as_ptr().cast(), data.len())) {
                Ok(written) if written as usize == data.len() => 0,
                Ok(written) => {
                    let written = written as usize;
                    state.write_buf.push_back(chunk(written));
                    data.len() - written
                }
                Err(err)
                    if err.kind() == std::io::ErrorKind::Interrupted
                        || err.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    state.write_buf.push_back(chunk(0));
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
                    conn_err = Some(err);
                    0
                }
            },
            _ => {
                state.write_buf.push_back(chunk(0));
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
        drop(state);

        if let Some(err) = conn_err {
            rself.call_conn_lost(py, Some(pyo3::exceptions::PyRuntimeError::new_err(err.to_string())));
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
    fn __traverse__(&self, visit: pyo3::gc::PyVisit<'_>) -> Result<(), pyo3::gc::PyTraverseError> {
        visit.call(&self.proto)?;
        visit.call(&self.protom_buf_get)?;
        visit.call(&self.protom_conn_lost)?;
        visit.call(&self.protom_recv_data)?;
        Ok(())
    }

    #[pyo3(signature = (name, default = None))]
    fn get_extra_info(&self, py: Python, name: &str, default: Option<Py<PyAny>>) -> Option<Py<PyAny>> {
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
        let buf_empty = self.state.borrow().write_buf_dsize == 0;
        if buf_empty {
            event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
            self.call_conn_lost(py, None);
        }
    }

    fn set_protocol(&self, _protocol: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "TCPTransport protocol cannot be changed",
        ))
    }

    fn get_protocol(&self, py: Python) -> Py<PyAny> {
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

    fn write(pyself: Py<Self>, py: Python, data: &Bound<PyAny>) -> PyResult<()> {
        if let Ok(pybytes) = data.cast::<PyBytes>() {
            return Self::try_write(&pyself, py, pybytes.as_bytes(), |off| WriteChunk::py(pybytes, off));
        }
        let bytes = data.extract::<Cow<[u8]>>()?;
        Self::try_write(&pyself, py, &bytes, |off| WriteChunk::owned((&bytes[off..]).into()))
    }

    fn writelines(pyself: Py<Self>, py: Python, data: &Bound<PyAny>) -> PyResult<()> {
        let pybytes = PyBytes::new(py, &[0; 0]);
        let pybytesj = pybytes.call_method1(pyo3::intern!(py, "join"), (data,))?;
        Self::write(pyself, py, &pybytesj)
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
    pub transport: Py<TCPTransport>,
}

impl TCPReadHandle {
    #[inline]
    fn recv_direct(&self, py: Python, transport: &TCPTransport) -> (Option<Py<PyAny>>, bool, bool) {
        const BUF_MAXSIZE: usize = 262_144;
        //: alloc an uninitialized `PyBytes` and read into it: single copy (kernel -> bytes), matching asyncio/CPython's `sock.recv`
        #[allow(clippy::cast_possible_wrap)]
        let mut obj =
            unsafe { pyo3::ffi::PyBytes_FromStringAndSize(std::ptr::null(), BUF_MAXSIZE as pyo3::ffi::Py_ssize_t) };
        if obj.is_null() {
            unsafe { pyo3::ffi::PyErr_Clear() };
            return (None, false, false);
        }
        let buf = unsafe { std::slice::from_raw_parts_mut(pyo3::ffi::PyBytes_AsString(obj).cast::<u8>(), BUF_MAXSIZE) };

        let (read, closed) = self.read_into(&mut transport.state.borrow_mut().stream, buf);
        if read > 0 {
            #[allow(clippy::cast_possible_wrap)]
            if read < BUF_MAXSIZE && unsafe { _PyBytes_Resize(&raw mut obj, read as pyo3::ffi::Py_ssize_t) } != 0 {
                unsafe { pyo3::ffi::PyErr_Clear() };
                return (None, closed, false);
            }
            let pydata = unsafe { Bound::from_owned_ptr(py, obj).unbind() };
            // `read == BUF_MAXSIZE` means the buffer filled, so more data may be waiting.
            return (Some(pydata), closed, read == BUF_MAXSIZE);
        }
        unsafe { pyo3::ffi::Py_DECREF(obj) };
        (None, closed, false)
    }

    #[inline]
    fn recv_buffered(&self, py: Python, transport: &TCPTransport) -> (Option<Py<PyAny>>, bool, bool) {
        let pybuf: PyBuffer<u8> = PyBuffer::get(&transport.protom_buf_get.bind(py).call1((-1,)).unwrap()).unwrap();
        let max_len = pybuf.len_bytes();

        let (read, closed) = if pybuf.is_c_contiguous() && !pybuf.readonly() {
            // Read straight into the protocol's buffer, no intermediate allocation or
            // copy. The GIL is held for the whole call, so nothing else can touch the
            // buffer concurrently, and we only access it through this slice.
            let buf = unsafe { std::slice::from_raw_parts_mut(pybuf.buf_ptr().cast::<u8>(), max_len) };
            self.read_into(&mut transport.state.borrow_mut().stream, buf)
        } else {
            // Fallback for non-contiguous/read-only buffers: copy through a temp Vec.
            let mut vbuf = pybuf.to_vec(py).unwrap();
            let (read, closed) = self.read_into(&mut transport.state.borrow_mut().stream, vbuf.as_mut_slice());
            if read > 0 {
                _ = pybuf.copy_from_slice(py, &vbuf[..read]);
            }
            (read, closed)
        };

        if read > 0 {
            return (Some(read.into_py_any(py).unwrap()), closed, read == max_len);
        }
        (None, closed, false)
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
        event_loop.tcp_stream_rem(transport.fd, Interest::READABLE);
        if let Ok(pyr) = transport.proto.call_method0(py, pyo3::intern!(py, "eof_received"))
            && let Ok(true) = pyr.is_truthy(py)
        {
            return false;
        }
        transport.close_from_read_handle(py, event_loop)
    }
}

impl Handle for TCPReadHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let transport = self.transport.borrow(py);
        if transport.conn_lost.load(atomic::Ordering::Relaxed) {
            return;
        }

        // NOTE: we need to consume all the data coming from the socket even when it exceeds the buffer,
        //       otherwise we won't get another readable event from the poller
        let mut close = false;
        loop {
            let (data, eof, full) = match transport.proto_buffered {
                true => self.recv_buffered(py, &transport),
                false => self.recv_direct(py, &transport),
            };

            if let Some(data) = data {
                _ = transport.protom_recv_data.call1(py, (data,));
                if !eof && full {
                    continue;
                }
            }

            if eof {
                close = self.recv_eof(py, event_loop, &transport);
            }

            break;
        }

        if close {
            event_loop.tcp_stream_close(py, transport.fd);
        }
    }
}

pub(crate) struct TCPWriteHandle {
    pub transport: Py<TCPTransport>,
}

impl TCPWriteHandle {
    #[inline]
    fn write(&self, transport: &TCPTransport, iovecs: &mut Vec<libc::iovec>) -> Option<usize> {
        const MAX_IOV: usize = 1024;

        #[allow(clippy::cast_possible_wrap)]
        let fd = transport.fd as i32;
        let mut ret = 0;
        let mut state = transport.state.borrow_mut();
        while !state.write_buf.is_empty() {
            iovecs.clear();
            let mut requested = 0;
            for chunk in state.write_buf.iter().take(MAX_IOV) {
                requested += chunk.remaining();
                iovecs.push(chunk.as_iovec());
            }

            #[allow(clippy::cast_possible_wrap)]
            match syscall!(writev(fd, iovecs.as_ptr(), iovecs.len() as i32)) {
                Ok(written) => {
                    let written = written as usize;
                    ret += written;
                    let mut consumed = written;
                    let mut drained = 0;
                    for iov in iovecs.iter() {
                        if consumed < iov.iov_len {
                            break;
                        }
                        consumed -= iov.iov_len;
                        drained += 1;
                    }
                    if drained > 0 {
                        state.write_buf.drain(..drained);
                    }
                    if consumed > 0 {
                        state.write_buf.front_mut().unwrap().off += consumed;
                    }
                    if written < requested {
                        break;
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
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
    fn run(&self, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState) {
        let transport = self.transport.borrow(py);
        if transport.conn_lost.load(atomic::Ordering::Relaxed) {
            return;
        }
        let stream_close;

        if let Some(written) = self.write(&transport, &mut state.iovecs) {
            if written > 0 {
                TCPTransport::write_buf_size_decr(&self.transport, py);
            }
            let buf_empty = transport.state.borrow().write_buf.is_empty();
            stream_close = match buf_empty {
                true => transport.close_from_write_handle(py, false),
                false => None,
            };
        } else {
            stream_close = transport.close_from_write_handle(py, true);
        }

        let buf_empty = transport.state.borrow().write_buf.is_empty();
        if buf_empty {
            event_loop.tcp_stream_rem(transport.fd, Interest::WRITABLE);
        }

        match stream_close {
            Some(true) => event_loop.tcp_stream_close(py, transport.fd),
            Some(false) => {
                _ = transport.state.borrow().stream.shutdown(std::net::Shutdown::Write);
            }
            _ => {}
        }
    }
}
