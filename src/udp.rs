#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

use mio::{Interest, net::UdpSocket};
use pyo3::{IntoPyObject, IntoPyObjectExt, prelude::*, types::PyBytes};
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::ErrorKind,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::atomic,
};

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    handles::Handle,
    log::LogExc,
    sock::SocketWrapper,
};

struct UDPTransportState {
    socket: UdpSocket,
    remote_addr: Option<SocketAddr>,
    write_buf: VecDeque<(Box<[u8]>, SocketAddr)>,
    write_buf_dsize: usize,
}

#[pyclass(frozen, unsendable, module = "rloop._rloop")]
pub(crate) struct UDPTransport {
    pub fd: usize,
    state: RefCell<UDPTransportState>,
    pyloop: Py<EventLoop>,
    // atomics
    closing: atomic::AtomicBool,
    water_hi: atomic::AtomicUsize,
    water_lo: atomic::AtomicUsize,
    // py protocol fields
    proto: Py<PyAny>,
    proto_paused: atomic::AtomicBool,
    protom_conn_lost: Py<PyAny>,
    protom_datagram_received: Py<PyAny>,
    protom_error_received: Py<PyAny>,
    // py extras
    extra: HashMap<String, Py<PyAny>>,
    sock: Py<SocketWrapper>,
}

impl UDPTransport {
    fn new(
        py: Python,
        pyloop: Py<EventLoop>,
        socket: UdpSocket,
        pyproto: Bound<PyAny>,
        socket_family: i32,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        let fd = socket.as_raw_fd() as usize;
        let state = UDPTransportState {
            socket,
            remote_addr,
            write_buf: VecDeque::new(),
            write_buf_dsize: 0,
        };

        let wh = 1024 * 64;
        let wl = wh / 4;

        let protom_conn_lost = pyproto.getattr(pyo3::intern!(py, "connection_lost")).unwrap().unbind();
        let protom_datagram_received = pyproto
            .getattr(pyo3::intern!(py, "datagram_received"))
            .unwrap()
            .unbind();
        let protom_error_received = pyproto.getattr(pyo3::intern!(py, "error_received")).unwrap().unbind();
        let proto = pyproto.unbind();

        Self {
            fd,
            state: RefCell::new(state),
            pyloop,
            closing: false.into(),
            water_hi: wh.into(),
            water_lo: wl.into(),
            proto,
            proto_paused: false.into(),
            protom_conn_lost,
            protom_datagram_received,
            protom_error_received,
            extra: HashMap::new(),
            sock: SocketWrapper::from_fd(py, fd, socket_family, socket2::Type::DGRAM, 0),
        }
    }

    pub(crate) fn from_py(
        py: Python,
        pyloop: &Py<EventLoop>,
        pysock: (i32, i32),
        proto_factory: Py<PyAny>,
        remote_addr_tup: Option<(String, u16)>,
    ) -> Self {
        let sock = unsafe { socket2::Socket::from_raw_fd(pysock.0) };
        _ = sock.set_nonblocking(true);
        let stds: std::net::UdpSocket = sock.into();
        let socket = UdpSocket::from_std(stds);

        let remote_addr = remote_addr_tup.map(|v| SocketAddr::new(IpAddr::from_str(&v.0).unwrap(), v.1));
        let proto = proto_factory.bind(py).call0().unwrap();

        Self::new(py, pyloop.clone_ref(py), socket, proto, pysock.1, remote_addr)
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
    fn close_from_write_handle(&self, py: Python, errored: bool) -> bool {
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
            return true;
        }
        false
    }

    #[inline(always)]
    fn call_conn_lost(&self, py: Python, exc: Option<PyErr>) {
        _ = self.protom_conn_lost.call1(py, (exc,));
    }

    #[inline]
    fn call_datagram_received(&self, py: Python, data: &[u8], addr: SocketAddr) {
        let py_data = PyBytes::new(py, data);
        let py_addr = (addr.ip().to_string(), addr.port()).into_pyobject(py).unwrap();
        _ = self.protom_datagram_received.call1(py, (py_data, py_addr));
    }

    #[inline]
    fn call_error_received(&self, py: Python, exc: PyErr) {
        _ = self.protom_error_received.call1(py, (exc,));
    }

    fn write(pyself: &Py<Self>, py: Python, data: &[u8], addr: SocketAddr) {
        let rself = pyself.borrow(py);
        let mut state = rself.state.borrow_mut();

        let buf_added = match state.write_buf_dsize {
            0 => {
                match rself.state.borrow().socket.send_to(data, addr) {
                    Ok(written) if written == data.len() => 0,
                    Ok(written) => {
                        state.write_buf.push_back(((&data[written..]).into(), addr));
                        data.len() - written
                    }
                    Err(err)
                        if err.kind() == std::io::ErrorKind::Interrupted
                            || err.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        state.write_buf.push_back((data.into(), addr));
                        data.len()
                    }
                    Err(err) => {
                        if state.write_buf_dsize > 0 {
                            // reset buf_dsize?
                            rself.pyloop.get().udp_socket_rem(rself.fd, Interest::WRITABLE);
                        }
                        if rself
                            .closing
                            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
                            .is_ok()
                        {
                            rself.pyloop.get().udp_socket_rem(rself.fd, Interest::READABLE);
                        }
                        rself.call_conn_lost(py, Some(pyo3::exceptions::PyOSError::new_err(err.to_string())));
                        0
                    }
                }
            }
            _ => {
                state.write_buf.push_back((data.into(), addr));
                data.len()
            }
        };

        if buf_added > 0 {
            if state.write_buf_dsize == 0 {
                rself.pyloop.get().udp_socket_add(rself.fd, Interest::WRITABLE);
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
impl UDPTransport {
    #[pyo3(signature = (name, default = None))]
    fn get_extra_info(&self, py: Python, name: &str, default: Option<Py<PyAny>>) -> Option<Py<PyAny>> {
        match name {
            "socket" => Some(self.sock.clone_ref(py).into_any()),
            "sockname" => self.sock.call_method0(py, pyo3::intern!(py, "getsockname")).ok(),
            "peername" => {
                if self.state.borrow().remote_addr.is_some() {
                    self.sock.call_method0(py, pyo3::intern!(py, "getpeername")).ok()
                } else {
                    default
                }
            }
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
        event_loop.udp_socket_rem(self.fd, Interest::READABLE);
        if self.state.borrow().write_buf_dsize == 0 {
            event_loop.udp_socket_rem(self.fd, Interest::WRITABLE);
            self.call_conn_lost(py, None);
        }
    }

    fn abort(&self, py: Python) {
        if self.state.borrow().write_buf_dsize > 0 {
            self.pyloop.get().udp_socket_rem(self.fd, Interest::WRITABLE);
        }
        if self
            .closing
            .compare_exchange(false, true, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed)
            .is_ok()
        {
            self.pyloop.get().udp_socket_rem(self.fd, Interest::READABLE);
        }
        self.call_conn_lost(py, None);
    }

    fn set_protocol(&self, _protocol: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "UDPTransport protocol cannot be changed",
        ))
    }

    fn get_protocol(&self, py: Python) -> Py<PyAny> {
        self.proto.clone_ref(py)
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

    fn sendto(pyself: Py<Self>, py: Python, data: Cow<[u8]>, addr: Option<(String, u16)>) -> PyResult<()> {
        let rself = pyself.borrow(py);

        if rself.closing.load(atomic::Ordering::Relaxed) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Cannot send on closing transport",
            ));
        }
        if data.is_empty() {
            return Ok(());
        }

        match addr
            .map(|v| SocketAddr::new(IpAddr::from_str(&v.0).unwrap(), v.1))
            .or_else(|| rself.state.borrow().remote_addr)
        {
            Some(addr) => {
                Self::write(&pyself, py, &data, addr);
                Ok(())
            }
            None => Err(pyo3::exceptions::PyValueError::new_err("No remote address specified")),
        }
    }
}

pub(crate) struct UDPReadHandle {
    pub fd: usize,
}

impl Handle for UDPReadHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState) {
        let pytransport = event_loop.get_udp_transport(self.fd, py);
        let transport = pytransport.borrow(py);

        loop {
            match {
                let trxstate = transport.state.borrow();
                trxstate.socket.recv_from(&mut state.read_buf)
            } {
                Ok((size, addr)) => {
                    // Call the protocol's datagram_received method
                    // Now state is not borrowed, so sendto can work
                    transport.call_datagram_received(py, &state.read_buf[..size], addr);
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    // No more data available
                    break;
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => {
                    // Interrupted by signal, continue
                }
                Err(err) => {
                    // Other error - call error_received and close
                    let py_err = pyo3::exceptions::PyOSError::new_err(err.to_string());
                    transport.call_error_received(py, py_err);
                    event_loop.udp_socket_close(self.fd);
                    break;
                }
            }
        }
    }
}

pub(crate) struct UDPWriteHandle {
    pub fd: usize,
}

impl UDPWriteHandle {
    #[inline]
    fn write(&self, transport: &UDPTransport) -> Option<usize> {
        let mut ret = 0;
        let mut state = transport.state.borrow_mut();

        while let Some((data, addr)) = state.write_buf.pop_front() {
            match state.socket.send_to(&data, addr) {
                Ok(written) if written < data.len() => {
                    state.write_buf.push_front(((&data[written..]).into(), addr));
                    ret += written;
                    break;
                }
                Ok(written) => ret += written,
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                    state.write_buf.push_front((data, addr));
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    state.write_buf.push_front((data, addr));
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

impl Handle for UDPWriteHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let pytransport = event_loop.get_udp_transport(self.fd, py);
        let transport = pytransport.borrow(py);
        let stream_close;

        if let Some(written) = self.write(&transport) {
            if written > 0 {
                UDPTransport::write_buf_size_decr(&pytransport, py);
            }
            stream_close = match transport.state.borrow().write_buf.is_empty() {
                true => transport.close_from_write_handle(py, false),
                false => false,
            };
        } else {
            stream_close = transport.close_from_write_handle(py, true);
        }

        if transport.state.borrow().write_buf.is_empty() {
            event_loop.udp_socket_rem(self.fd, Interest::WRITABLE);
        }
        if stream_close {
            event_loop.udp_socket_close(self.fd);
        }
    }
}
