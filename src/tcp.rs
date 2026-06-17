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
    sync::{Arc, atomic},
};

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    handles::{BoxedHandle, CBHandle, Handle},
    log::LogExc,
    py::{asyncio_proto_buf, copy_context},
    sock::SocketWrapper,
    utils::syscall,
};

use rustls::Connection as TLSConnection;

/// No-op used for SSL connections where connection_made is called later.
struct NoOpHandle;

impl Handle for NoOpHandle {
    fn run(&self, _py: Python, _event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        log::debug!("NoOpHandle::run() called");
        // Doing nothing
    }

    fn cancelled(&self) -> bool {
        false
    }
}

pub(crate) struct TCPServer {
    pub fd: i32,
    sfamily: i32,
    backlog: i32,
    protocol_factory: Py<PyAny>,
    ssl_config: Option<rustls::ServerConfig>,
}

impl TCPServer {
    pub(crate) fn from_fd(fd: i32, sfamily: i32, backlog: i32, protocol_factory: Py<PyAny>) -> Self {
        Self {
            fd,
            sfamily,
            backlog,
            protocol_factory,
            ssl_config: None,
        }
    }

    pub(crate) fn from_fd_ssl(
        fd: i32,
        sfamily: i32,
        backlog: i32,
        protocol_factory: Py<PyAny>,
        ssl_config: rustls::ServerConfig,
    ) -> Self {
        Self {
            fd,
            sfamily,
            backlog,
            protocol_factory,
            ssl_config: Some(ssl_config),
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
            ssl_config: self.ssl_config.clone(),
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

pub(crate) struct TCPServerRef {
    pub fd: usize,
    pyloop: Py<EventLoop>,
    sfamily: i32,
    proto_factory: Py<PyAny>,
    ssl_config: Option<rustls::ServerConfig>,
}

impl TCPServerRef {
    #[inline]
    pub(crate) fn new_stream(&self, py: Python, stream: TcpStream) -> (Py<TCPTransport>, BoxedHandle) {
        log::debug!("SSL server: accepting new connection");
        let proto = self.proto_factory.bind(py).call0().unwrap();

        let transport = TCPTransport::new(
            py,
            self.pyloop.clone_ref(py),
            stream,
            proto,
            self.sfamily,
            Some(self.fd),
        );

        // Initialize TLS if this is an SSL server
        if let Some(ref ssl_config) = self.ssl_config {
            log::debug!("SSL server: initializing TLS for new connection");
            transport.initialize_tls_server(ssl_config.clone());
        }

        let pytransport = Py::new(py, transport).unwrap();

        // For SSL connections, delay connection_made until handshake completes
        let is_ssl = self.ssl_config.is_some();
        log::debug!(
            "new_stream: is_ssl = {}, ssl_config.is_some() = {}",
            is_ssl,
            self.ssl_config.is_some()
        );
        let conn_handle: BoxedHandle = if is_ssl {
            // For SSL connections, wait the handshake before scheduling callbacks
            // connection_made will be called later when handshake completes
            log::debug!("Creating NoOpHandle for SSL connection");
            Box::new(NoOpHandle)
        } else {
            // For non-SSL connections, call connection_made immediately
            let conn_made = pytransport
                .borrow(py)
                .proto
                .getattr(py, pyo3::intern!(py, "connection_made"))
                .unwrap();
            Box::new(
                Py::new(
                    py,
                    CBHandle::new1(conn_made, pytransport.clone_ref(py).into_any(), copy_context(py)),
                )
                .unwrap(),
            )
        };

        (pytransport, conn_handle)
    }

    #[inline]
    fn get_ssl_config(&self, py: Python) -> Option<rustls::ServerConfig> {
        None // TODO: Pass SSL config through the server reference
    }
}

struct TCPTransportState {
    stream: TcpStream,
    tls_conn: Option<TLSConnection>,
    handshake_complete: bool,
    connection_made_called: bool,
    write_buf: VecDeque<Box<[u8]>>,
    write_buf_dsize: usize,
    tls_close_sent: bool,
    tls_close_received: bool,
    tls_close_sent_time: Option<std::time::Instant>,
    tls_pending_close: bool,
    ssl_close_timeout: u16,
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

        let ssl_close_timeout = std::env::var("RLOOP_SSL_CLOSE_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        let state = TCPTransportState {
            stream,
            tls_conn: None,
            handshake_complete: false,
            connection_made_called: false,
            write_buf: VecDeque::new(),
            write_buf_dsize: 0,
            tls_close_sent: false,
            tls_close_received: false,
            tls_close_sent_time: None,
            tls_pending_close: false,
            ssl_close_timeout,
        };

        let wh = 1024 * 64;
        let wl = wh / 4;

        let proto_buffered = pyproto.is_instance(asyncio_proto_buf(py).unwrap()).unwrap();
        let protom_conn_lost = pyproto.getattr(pyo3::intern!(py, "connection_lost")).unwrap().unbind();

        let protom_buf_get: Py<PyAny>;
        let protom_recv_data: Py<PyAny>;

        if proto_buffered {
            protom_buf_get = pyproto.getattr(pyo3::intern!(py, "get_buffer")).unwrap().unbind();
            protom_recv_data = pyproto.getattr(pyo3::intern!(py, "buffer_updated")).unwrap().unbind();
        } else {
            protom_buf_get = py.None();
            protom_recv_data = pyproto.getattr(pyo3::intern!(py, "data_received")).unwrap().unbind();
        }

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
        // For SSL connections, delay connection_made until handshake completes
        if rself.state.borrow().tls_conn.is_none() {
            rself
                .proto
                .call_method1(py, pyo3::intern!(py, "connection_made"), (pyself.clone_ref(py),))?;
        }
        Ok(rself.proto.clone_ref(py))
    }

    pub(crate) fn initialize_tls_server(&self, ssl_config: rustls::ServerConfig) {
        let mut state = self.state.borrow_mut();
        state.tls_conn = Some(TLSConnection::Server(
            rustls::ServerConnection::new(Arc::new(ssl_config)).unwrap(),
        ));
        state.handshake_complete = false;
    }

    pub(crate) fn initialize_tls_client(&self, ssl_config: rustls::ClientConfig, server_name: String) {
        log::debug!(
            "SSL client: Initializing TLS for fd {} with server '{}'",
            self.fd,
            server_name
        );
        let mut state = self.state.borrow_mut();
        let server_name = rustls::pki_types::ServerName::try_from(server_name).unwrap();
        let conn = rustls::ClientConnection::new(Arc::new(ssl_config), server_name).unwrap();
        state.tls_conn = Some(TLSConnection::Client(conn));
        state.handshake_complete = false;

        // Check if the client needs to send initial handshake data
        if let Some(ref tls_conn) = state.tls_conn {
            if tls_conn.wants_write() {
                log::debug!(
                    "SSL client: fd {} wants to write immediately after handshake init",
                    self.fd
                );
                self.pyloop.get().tcp_stream_add(self.fd, Interest::WRITABLE);
            } else {
                log::debug!(
                    "SSL client: fd {} does not want to write immediately after handshake init",
                    self.fd
                );
            }
        }
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

        // For TLS connections, call close() to send TLS close alerts
        if self.state.borrow().tls_conn.is_some() {
            self.close(py);
            return true; // Handled by TLS specific close path
        }

        if !self.state.borrow_mut().write_buf.is_empty() {
            // Need mutable borrow for check
            log::debug!(
                "TCP close_from_read_handle: fd {} has pending write data, not closing yet",
                self.fd
            );
            return false;
        }

        log::debug!("TCP close_from_read_handle: fd {} closing now", self.fd);
        event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
        _ = self.protom_conn_lost.call1(py, (py.None(),));
        true
    }

    #[inline]
    fn close_from_write_handle(&self, py: Python, errored: bool) -> Option<bool> {
        if self.closing.load(atomic::Ordering::Relaxed) {
            log::debug!(
                "TCP close_from_write_handle: fd {} already closing. Errored: {}",
                self.fd,
                errored
            );
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
        let weof = self.weof.load(atomic::Ordering::Relaxed); // Store to avoid multiple loads
        if weof {
            log::debug!(
                "TCP close_from_write_handle: fd {} WEOF true. Errored: {}",
                self.fd,
                errored
            );
        } else {
            log::debug!(
                "TCP close_from_write_handle: fd {} WEOF false. Errored: {}. Closing due to write EOF.",
                self.fd,
                errored
            );
        }
        weof.then_some(false) // if weof is true, return Some(false), else None
    }

    #[inline(always)]
    fn call_conn_lost(&self, py: Python, err: Option<PyErr>) {
        log::debug!(
            "TCPTransport::call_conn_lost called for fd {}. Error present: {:?}",
            self.fd,
            err.is_some()
        );
        let err_arg = match err {
            Some(e) => e.into_py_any(py).unwrap(),
            None => py.None(),
        };
        _ = self.protom_conn_lost.call1(py, (err_arg,));
        // tcp_stream_close will trigger actual socket closure and subsequent Python callback
        self.pyloop.get().tcp_stream_close(py, self.fd);
    }

    fn call_conn_lost_py(&self, py: Python) {
        self.call_conn_lost(py, None);
    }

    fn try_write(pyself: &Py<Self>, py: Python, data: &[u8]) -> PyResult<()> {
        let rself = pyself.borrow(py);

        if rself.weof.load(atomic::Ordering::Relaxed) {
            log::debug!("TCP/SSL try_write: fd {} EOF set for write", rself.fd);
            return Err(pyo3::exceptions::PyRuntimeError::new_err("Cannot write after EOF"));
        }
        if data.is_empty() {
            log::debug!("TCP/SSL try_write: fd {} empty data", rself.fd);
            return Ok(());
        }

        let is_tls = rself.state.borrow().tls_conn.is_some();
        if is_tls {
            log::debug!(
                "SSL write (try_write): called for fd {} with {} bytes of application data",
                rself.fd,
                data.len()
            );
        } else {
            log::debug!(
                "TCP write (try_write): called for fd {} with {} bytes of application data",
                rself.fd,
                data.len()
            );
        }

        let mut state = rself.state.borrow_mut();

        // For TLS connections, never write directly to socket - always buffer for encryption
        let buf_added = if is_tls {
            log::debug!(
                "SSL write (try_write): buffering {} bytes for encryption on fd {}",
                data.len(),
                rself.fd
            );
            state.write_buf.push_back(data.into());
            data.len()
        } else {
            match state.write_buf_dsize {
                #[allow(clippy::cast_possible_wrap)]
                0 => match syscall!(write(rself.fd as i32, data.as_ptr().cast(), data.len())) {
                    Ok(written) if written as usize == data.len() => {
                        log::debug!(
                            "TCP write (try_write): wrote all {} bytes directly on fd {}",
                            data.len(),
                            rself.fd
                        );
                        0 // All data written
                    }
                    Ok(written) => {
                        let written = written as usize;
                        log::debug!(
                            "TCP write (try_write): partial direct write on fd {}: {}/{}",
                            rself.fd,
                            written,
                            data.len()
                        );
                        state.write_buf.push_back((&data[written..]).into());
                        // Amount buffered
                        data.len() - written
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                        log::debug!(
                            "TCP write (try_write): interrupted on fd {}. Buffering all {} bytes.",
                            rself.fd,
                            data.len()
                        );
                        state.write_buf.push_back(data.into()); // Buffer all
                        data.len()
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        log::debug!(
                            "TCP write (try_write): would block on fd {}. Buffering all {} bytes.",
                            rself.fd,
                            data.len()
                        );
                        state.write_buf.push_back(data.into()); // Buffer all
                        data.len()
                    }
                    Err(err) => {
                        log::error!("TCP write (try_write): syscall error for fd {}: {:?}", rself.fd, err);
                        if state.write_buf_dsize > 0 {
                            // reset buf_dsize?
                            rself.pyloop.get().tcp_stream_rem(rself.fd, Interest::WRITABLE);
                        }
                        if rself
                            .closing
                            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
                            .is_ok()
                        {
                            log::debug!(
                                "TCP write (try_write): error on fd {}, setting closing and removing READ interest",
                                rself.fd
                            );
                            rself.pyloop.get().tcp_stream_rem(rself.fd, Interest::READABLE);
                        }
                        rself.call_conn_lost(
                            py,
                            Some(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())),
                        );
                        // Connection closed
                        0
                    }
                },
                _ => {
                    // Buffer already had data, append new data
                    log::debug!(
                        "SSL write (try_write): appending {} bytes to existing buffer on fd {}",
                        data.len(),
                        rself.fd
                    );
                    state.write_buf.push_back(data.into());
                    data.len()
                }
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
        log::debug!("TCP/SSL proto_pause called for fd {}", rself.fd); // Use rself.fd
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
        log::debug!("TCP/SSL proto_resume called for fd {}", rself.fd); // Use rself.fd
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

    pub(crate) fn is_tls(&self) -> bool {
        self.state.borrow().tls_conn.is_some()
    }

    pub(crate) fn close(&self, py: Python) {
        log::debug!("TCPTransport::close() called for fd {}", self.fd);
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_err()
        {
            log::debug!("TCPTransport::close() already closing, returning");
            return;
        }

        // For TLS connections
        if self.state.borrow().tls_conn.is_some() {
            let has_pending_data = !self.state.borrow().write_buf.is_empty();
            if has_pending_data {
                log::debug!(
                    "SSL close: pending data in write buffer, deferring close alert for fd {}",
                    self.fd
                );
                // Mark that we want to close after pending data is sent
                self.state.borrow_mut().tls_pending_close = true;
                return;
            } else {
                log::debug!("SSL close: no pending data, sending TLS close alert for fd {}", self.fd);
                // Send close alert immediately since no pending data
                let mut tls_buf = Vec::new();
                {
                    let mut state = self.state.borrow_mut();
                    if let Some(ref mut tls_conn) = state.tls_conn {
                        // Send close notify to initiate TLS close handshake
                        tls_conn.send_close_notify();
                        let _ = tls_conn.write_tls(&mut tls_buf);
                        // Mark that we've sent our close alert
                        state.tls_close_sent = true;
                        state.tls_close_sent_time = Some(std::time::Instant::now());
                    }
                }
                if !tls_buf.is_empty() {
                    log::trace!("SSL close: TLS buffer: {:02x?}", &tls_buf[..tls_buf.len().min(64)]);
                    let fd = self.fd as i32;
                    let _ = syscall!(write(fd, tls_buf.as_ptr().cast(), tls_buf.len()));
                }

                log::debug!("SSL close: sent close alert, waiting for peer response");
                let event_loop = self.pyloop.get();
                event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
                // Keep readable interest to detect peer's close

                // Notify the connection closing
                let pytransport = event_loop.get_tcp_transport(self.fd, py).unwrap();
                pytransport
                    .getattr(py, pyo3::intern!(py, "call_connection_lost"))
                    .unwrap();

                return;
            }
        }

        let event_loop = self.pyloop.get();
        event_loop.tcp_stream_rem(self.fd, Interest::READABLE);
        if self.state.borrow().write_buf_dsize == 0 || self.state.borrow().tls_conn.is_some() {
            // For TLS connections, close immediately after sending close alert
            // even if write buffer is not empty (close alert will be sent by TCPWriteHandle)
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
            log::error!(
                "TCPTransport::set_write_buffer_limits for fd {}: Error: high ({}) must be >= low ({}). Current values not changed.",
                pyself.borrow(py).fd,
                wh,
                wl
            );
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
        let size = self.state.borrow().write_buf_dsize;
        log::debug!(
            "TCPTransport::get_write_buffer_size called for fd {}. Size: {}",
            self.fd,
            size
        );
        size
    }

    fn get_write_buffer_limits(&self) -> (usize, usize) {
        let limits = (
            self.water_lo.load(atomic::Ordering::Relaxed),
            self.water_hi.load(atomic::Ordering::Relaxed),
        );
        log::debug!(
            "TCPTransport::get_write_buffer_limits called for fd {}. Limits: {:?}",
            self.fd,
            limits
        );
        limits
    }

    fn write(pyself: Py<Self>, py: Python, data: Cow<[u8]>) -> PyResult<()> {
        log::debug!(
            "TCPTransport::write (PyO3) called for fd {:?} with {} bytes",
            pyself.borrow(py).fd,
            data.len()
        );
        Self::try_write(&pyself, py, &data)
    }

    fn writelines(pyself: Py<Self>, py: Python, data: &Bound<PyAny>) -> PyResult<()> {
        log::debug!(
            "TCPTransport::writelines (PyO3) called for fd {:?}",
            pyself.borrow(py).fd
        );
        let pybytes = PyBytes::new(py, &[0; 0]);
        let pybytesj = pybytes.call_method1(pyo3::intern!(py, "join"), (data,))?;
        let bytes: Cow<[u8]> = pybytesj.extract()?;
        log::debug!(
            "TCPTransport::writelines (PyO3) for fd {:?} joined to {} bytes",
            pyself.borrow(py).fd,
            bytes.len()
        );
        Self::try_write(&pyself, py, &bytes)
    }

    fn write_eof(&self) {
        log::debug!("TCPTransport::write_eof called for fd {}", self.fd);
        if self.closing.load(atomic::Ordering::Relaxed) {
            log::debug!("TCPTransport::write_eof: fd {} closing, returning.", self.fd);
            return;
        }
        // weof -> write end of file: no more writes will be done.
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
        let can = !self.weof.load(atomic::Ordering::Relaxed); // Can write EOF if not already set
        log::debug!("TCPTransport::can_write_eof called for fd {}. Value: {}", self.fd, can);
        can
    }

    fn abort(&self, py: Python) {
        log::debug!("TCPTransport::abort called for fd {}", self.fd);
        if self.state.borrow().write_buf_dsize > 0 {
            log::debug!(
                "TCPTransport::abort: fd {} has write_buf_dsize > 0. Removing WRITABLE interest.",
                self.fd
            );
            self.pyloop.get().tcp_stream_rem(self.fd, Interest::WRITABLE);
        }
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_ok()
        {
            log::debug!(
                "TCPTransport::abort: fd {} set closing. Removing READ interest.",
                self.fd
            );
            self.pyloop.get().tcp_stream_rem(self.fd, Interest::READABLE);
        } else {
            log::debug!("TCPTransport::abort: fd {} was already closing.", self.fd);
        }
        log::debug!(
            "TCPTransport::abort: fd {} calling call_conn_lost due to abort.",
            self.fd
        );
        self.call_conn_lost(py, None);
    }

    fn call_connection_lost(&self, py: Python) {
        self.call_conn_lost_py(py);
        log::debug!(
            "TCPTransport::call_connection_lost (Python API) called for fd {}",
            self.fd
        );
    }
}

pub(crate) struct TCPReadHandle {
    pub fd: usize,
}

impl TCPReadHandle {
    #[inline]
    fn recv_direct(&self, py: Python, transport: &TCPTransport, buf: &mut [u8]) -> (Option<Py<PyAny>>, bool) {
        // Check if this is a TLS connection first
        let is_tls = transport.state.borrow().tls_conn.is_some();

        if is_tls {
            log::debug!("SSL read: processing TLS data for fd {}", self.fd);
            // Handle TLS connections
            let read = {
                let mut state = transport.state.borrow_mut();
                let (read, _) = self.read_into(&mut state.stream, buf);
                read
            };

            log::debug!("SSL read: received {read} bytes of raw data");

            if read > 0 {
                // Process TLS data
                {
                    let mut state = transport.state.borrow_mut();
                    let tls_conn = state.tls_conn.as_mut().unwrap();

                    // Feed raw bytes to TLS connection
                    let mut rd = std::io::Cursor::new(&buf[..read]);
                    if let Err(e) = tls_conn.read_tls(&mut rd) {
                        log::debug!("SSL read: TLS read_tls error: {e:?}");
                        // TLS error - close connection
                        return (None, true);
                    }

                    // Process the new packets
                    match tls_conn.process_new_packets() {
                        Ok(io_state) => {
                            // Check if we received a close alert from the peer
                            if io_state.peer_has_closed() {
                                log::debug!("SSL read: peer has closed the connection (received close alert)");
                                // FIX: Always close immediately when receiving close alert to prevent hanging
                                // This handles both TLS 1.2 and TLS 1.3 properly
                                log::debug!("SSL read: closing connection immediately to prevent hanging");
                                tls_conn.send_close_notify();
                                state.tls_close_received = true;
                                return (None, true);
                            }
                        }
                        Err(e) => {
                            log::debug!("SSL read: TLS process_new_packets error: {e:?}");
                            // TLS error - close connection
                            return (None, true);
                        }
                    }
                }

                // Check and update handshake status
                {
                    let mut state = transport.state.borrow_mut();
                    let tls_conn = state.tls_conn.as_ref().unwrap();
                    if !state.handshake_complete && !tls_conn.is_handshaking() {
                        state.handshake_complete = true;
                        log::debug!("SSL read: handshake completed");
                        log::trace!(
                            "RLOOP_TLS_DBG_HANDSHAKE_CMPL: fd={}, connection_made_called={}",
                            self.fd,
                            state.connection_made_called
                        );

                        // For SSL connections, call connection_made after handshake completes
                        if !state.connection_made_called {
                            state.connection_made_called = true;
                            // Schedule connection_made callback through the event loop
                            let transport_arg = transport
                                .pyloop
                                .get()
                                .get_tcp_transport(self.fd, py)
                                .map(|t| t.clone_ref(py).into_any())
                                .unwrap_or_else(|| py.None());
                            if let Ok(conn_made) = transport.proto.getattr(py, pyo3::intern!(py, "connection_made")) {
                                let _ = transport.pyloop.get().schedule1(
                                    conn_made,
                                    transport_arg,
                                    None, // Use default context
                                );
                            }
                        }
                    } else if !state.handshake_complete {
                        log::debug!("SSL read: still handshaking");
                    }
                }

                // Check if there is pending TLS data to write (handshake, etc.)
                {
                    let state = transport.state.borrow();
                    if let Some(ref tls_conn) = state.tls_conn
                        && tls_conn.wants_write()
                    {
                        log::debug!("SSL read: server wants to write (handshake data), adding writable interest");
                        transport.pyloop.get().tcp_stream_add(transport.fd, Interest::WRITABLE);
                    }
                }

                // Check if handshake is complete and read decrypted data
                let handshake_complete = transport.state.borrow().handshake_complete;
                if handshake_complete {
                    let mut app_data = Vec::new();
                    {
                        let mut state = transport.state.borrow_mut();
                        let tls_conn = state.tls_conn.as_mut().unwrap();

                        let mut temp_buf = [0u8; 4096];
                        loop {
                            match tls_conn.reader().read(&mut temp_buf) {
                                Ok(0) => break,
                                Ok(n) => app_data.extend_from_slice(&temp_buf[..n]),
                                Err(_) => break,
                            }
                        }
                    }

                    if !app_data.is_empty() {
                        log::debug!("SSL read: decrypted {} bytes of application data", app_data.len());
                        let pydata = PyBytes::new(py, &app_data);
                        return (Some(pydata.into_any().unbind()), false);
                    }
                }
            }

            // Check if connection is closed
            let closed = {
                let mut state = transport.state.borrow_mut();
                let (bytes_read, closed) = self.read_into(&mut state.stream, &mut []);
                log::debug!(
                    "SSL read: connection closed check - bytes_read={}, closed={}, tls_close_sent={}",
                    bytes_read,
                    closed,
                    state.tls_close_sent
                );
                // If we read 0 bytes and we've sent our close alert, consider the connection closed
                let peer_closed_after_our_alert = bytes_read == 0 && state.tls_close_sent;
                if peer_closed_after_our_alert {
                    log::debug!(
                        "SSL read: peer closed TCP connection after receiving our close alert - completing handshake"
                    );
                    // Peer closed TCP after receiving our close alert - this completes the handshake
                    return (None, true);
                }
                closed
            };
            return (None, closed);
        }

        // Non-TLS connection
        let mut state = transport.state.borrow_mut();
        let (read, closed) = self.read_into(&mut state.stream, buf);
        if read > 0 {
            let rbuf = &buf[..read];
            let pydata = unsafe { PyBytes::from_ptr(py, rbuf.as_ptr(), read) };
            return (Some(pydata.into_any().unbind()), closed);
        }
        (None, closed)
    }

    #[inline]
    fn recv_buffered(&self, py: Python, transport: &TCPTransport) -> (Option<Py<PyAny>>, bool) {
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
        if let Ok(pyr) = transport.proto.call_method0(py, pyo3::intern!(py, "eof_received"))
            && let Ok(true) = pyr.is_truthy(py)
        {
            return false;
        }

        // For TLS connections that have sent a close alert, call connection_lost when TCP closes
        if transport.state.borrow().tls_conn.is_some() && transport.state.borrow().tls_close_sent {
            transport.call_conn_lost(py, None);
            return true;
        }

        transport.close_from_read_handle(py, event_loop)
    }
}

impl Handle for TCPReadHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState) {
        // if None: transport was closed
        let Some(pytransport) = event_loop.get_tcp_transport(self.fd, py) else {
            return;
        };

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
        log::debug!("DEBUG: TCPWriteHandle::write called for fd {}", self.fd);
        #[allow(clippy::cast_possible_wrap)]
        let fd = self.fd as i32;

        // Check if this is a TLS connection first
        let is_tls = transport.state.borrow().tls_conn.is_some();

        if is_tls {
            log::debug!("SSL write: handling TLS write for fd {}", self.fd);
            // Handle TLS connections
            let mut tls_buf = Vec::new();
            {
                let mut state = transport.state.borrow_mut();
                let tls_conn = state.tls_conn.as_mut().unwrap();

                // First, handle any pending TLS writes (handshake or encrypted data)
                if let Err(e) = tls_conn.write_tls(&mut tls_buf) {
                    log::debug!("SSL write: TLS write_tls error: {e:?}");
                    // TLS error
                    return None;
                }
            }

            if tls_buf.is_empty() {
                log::debug!("SSL write: no TLS data to send");
            } else {
                log::trace!("SSL write: TLS buffer: {:02x?}", &tls_buf[..tls_buf.len().min(64)]);
                log::debug!("SSL write: sending {} bytes of TLS data", tls_buf.len());
                match syscall!(write(fd, tls_buf.as_ptr().cast(), tls_buf.len())) {
                    Ok(written) if written as usize == tls_buf.len() => {
                        log::debug!("SSL write: TLS data sent successfully");
                        // TLS data written successfully
                    }
                    Ok(_written) => {
                        log::debug!("SSL write: partial TLS write");
                        // Partial write - this is complex for TLS, just fail for now
                        return None;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        log::debug!("SSL write: TLS write would block");
                        // Would block - need to retry later
                        return Some(0);
                    }
                    _ => {
                        log::debug!("SSL write: TLS write failed");
                        return None;
                    }
                }
            }
            // Check if handshake is complete
            let handshake_complete = transport.state.borrow().handshake_complete;

            if handshake_complete {
                // Write data one by one to avoid borrowing conflicts
                let mut ret = 0;
                loop {
                    let data = {
                        let state = transport.state.borrow();
                        state.write_buf.front().cloned()
                    };

                    if let Some(data) = data {
                        {
                            let mut state = transport.state.borrow_mut();
                            let tls_conn = state.tls_conn.as_mut().unwrap();

                            if std::io::Write::write_all(&mut tls_conn.writer(), &data).is_err() {
                                // TLS write error - put data back
                                return None;
                            }
                        }

                        {
                            let mut state = transport.state.borrow_mut();
                            state.write_buf.pop_front();
                            state.write_buf_dsize -= data.len();
                        }

                        ret += data.len();
                    } else {
                        break;
                    }
                }

                // Write any newly encrypted data
                let mut tls_buf = Vec::new();
                {
                    let mut state = transport.state.borrow_mut();
                    let tls_conn = state.tls_conn.as_mut().unwrap();
                    if tls_conn.write_tls(&mut tls_buf).is_err() {
                        return None;
                    }
                }

                if !tls_buf.is_empty() {
                    log::trace!(
                        "SSL write: Application data TLS buffer: {:02x?}",
                        &tls_buf[..tls_buf.len().min(64)]
                    );
                    match syscall!(write(fd, tls_buf.as_ptr().cast(), tls_buf.len())) {
                        Ok(written) if written as usize == tls_buf.len() => {}
                        Ok(_) => return None, // Partial write
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            return Some(ret);
                        }
                        _ => return None,
                    }
                }

                return Some(ret);
            } else {
                // Handshake not complete, just wrote handshake data
                return Some(0);
            }
        }

        // Non-TLS connection
        let mut state = transport.state.borrow_mut();
        let mut ret = 0;
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
        let Some(pytransport) = event_loop.get_tcp_transport(self.fd, py) else {
            return;
        };
        let transport = pytransport.borrow(py);
        let stream_close;

        // Check if we need to timeout waiting for peer's close alert
        {
            let state = transport.state.borrow();
            if state.tls_close_sent && state.tls_conn.is_some() {
                log::debug!("SSL close: already sent. Waiting a response with TCP open.");
                if let Some(sent_time) = state.tls_close_sent_time {
                    let elapsed = sent_time.elapsed();
                    if elapsed > std::time::Duration::from_millis(transport.state.borrow().ssl_close_timeout.into()) {
                        log::debug!(
                            "SSL close: timeout waiting for peer's close alert ({}ms), closing connection",
                            elapsed.as_millis()
                        );
                        // Force close the connection
                        drop(state);
                        event_loop.tcp_stream_rem(self.fd, Interest::READABLE);
                        event_loop.tcp_stream_rem(self.fd, Interest::WRITABLE);
                        transport.call_conn_lost(py, None);
                        return;
                    }
                }
            }
        }

        if let Some(written) = self.write(&transport) {
            if written > 0 {
                TCPTransport::write_buf_size_decr(&pytransport, py);
            }
            let (write_buf_empty, pending_ssl_close) = {
                let state = transport.state.borrow();
                (state.write_buf.is_empty(), state.tls_pending_close)
            };
            stream_close = match write_buf_empty {
                true => {
                    if pending_ssl_close {
                        log::debug!(
                            "SSL write: write buffer empty, sending pending close alert for fd {}",
                            self.fd
                        );
                        // Send the close alert now that buffer is empty
                        let mut tls_buf = Vec::new();
                        {
                            let mut state = transport.state.borrow_mut();
                            if let Some(ref mut tls_conn) = state.tls_conn {
                                // Send close notify to initiate TLS close handshake
                                tls_conn.send_close_notify();
                                let _ = tls_conn.write_tls(&mut tls_buf);
                                state.tls_close_sent = true;
                                state.tls_close_sent_time = Some(std::time::Instant::now());
                                state.tls_pending_close = false; // Clear the flag
                            }
                        }
                        if !tls_buf.is_empty() {
                            log::trace!("SSL close: TLS buffer: {:02x?}", &tls_buf[..tls_buf.len().min(64)]);
                            let fd = self.fd as i32;
                            let _ = syscall!(write(fd, tls_buf.as_ptr().cast(), tls_buf.len()));
                        }
                        // Now close the connection
                        Some(true)
                    } else {
                        transport.close_from_write_handle(py, false)
                    }
                }
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
