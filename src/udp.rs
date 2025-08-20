#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};

use mio::net::UdpSocket;
use pyo3::{IntoPyObject, prelude::*, types::PyBytes};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, io::ErrorKind, net::SocketAddr, sync::atomic};

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    handles::Handle,
    sock::SocketWrapper,
};

struct UDPTransportState {
    socket: UdpSocket,
    remote_addr: Option<SocketAddr>,
}

#[pyclass(frozen, unsendable, module = "rloop._rloop")]
pub(crate) struct UDPTransport {
    pub fd: usize,
    state: RefCell<UDPTransportState>,
    pyloop: Py<EventLoop>,
    // atomics
    closing: atomic::AtomicBool,
    // py protocol fields
    proto: PyObject,
    protom_conn_lost: PyObject,
    protom_datagram_received: PyObject,
    protom_error_received: PyObject,
    // py extras
    extra: HashMap<String, PyObject>,
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
        let state = UDPTransportState { socket, remote_addr };

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
            proto,
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
        proto_factory: PyObject,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        let sock = unsafe { socket2::Socket::from_raw_fd(pysock.0) };
        _ = sock.set_nonblocking(true);
        let std_socket: std::net::UdpSocket = sock.into();
        let socket = UdpSocket::from_std(std_socket);

        let proto = proto_factory.bind(py).call0().unwrap();

        Self::new(py, pyloop.clone_ref(py), socket, proto, pysock.1, remote_addr)
    }

    pub(crate) fn attach(pyself: &Py<Self>, py: Python) -> PyResult<PyObject> {
        let rself = pyself.borrow(py);
        rself
            .proto
            .call_method1(py, pyo3::intern!(py, "connection_made"), (pyself.clone_ref(py),))?;
        Ok(rself.proto.clone_ref(py))
    }

    #[inline]
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
}

#[pymethods]
impl UDPTransport {
    #[pyo3(signature = (name, default = None))]
    fn get_extra_info(&self, py: Python, name: &str, default: Option<PyObject>) -> Option<PyObject> {
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
        event_loop.udp_socket_rem(self.fd);
        self.call_conn_lost(py, None);
    }

    fn abort(&self, py: Python) {
        self.close(py);
    }

    fn set_protocol(&self, _protocol: PyObject) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "UDPTransport protocol cannot be changed",
        ))
    }

    fn get_protocol(&self, py: Python) -> PyObject {
        self.proto.clone_ref(py)
    }

    fn sendto(&self, py: Python, data: Cow<[u8]>, addr: Option<PyObject>) -> PyResult<()> {
        if self.closing.load(atomic::Ordering::Relaxed) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Cannot send on closing transport",
            ));
        }

        let target_addr = match addr {
            Some(addr_obj) => {
                // Parse the address from Python tuple (host, port)
                let addr_tuple: (String, u16) = addr_obj.extract(py)?;
                Some(
                    format!("{}:{}", addr_tuple.0, addr_tuple.1)
                        .parse::<SocketAddr>()
                        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid address: {e}")))?,
                )
            }
            None => {
                // Get remote addr without holding the borrow longer than necessary
                self.state.borrow().remote_addr
            }
        };

        match target_addr {
            Some(addr) => {
                // Temporarily borrow just for the send operation
                match self.state.borrow().socket.send_to(&data, addr) {
                    Ok(_) => Ok(()),
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        // For UDP, we don't buffer writes like TCP - just drop the packet or return error
                        Err(pyo3::exceptions::PyBlockingIOError::new_err("Socket would block"))
                    }
                    Err(err) => Err(pyo3::exceptions::PyOSError::new_err(err.to_string())),
                }
            }
            None => Err(pyo3::exceptions::PyValueError::new_err("No remote address specified")),
        }
    }
}

pub(crate) struct UDPHandle {
    fd: usize,
}

impl UDPHandle {
    pub(crate) fn new(fd: usize) -> Self {
        Self { fd }
    }
}

impl Handle for UDPHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let pytransport = event_loop.get_udp_transport(self.fd, py);
        let transport = pytransport.borrow(py);

        // Read datagrams from the socket
        let mut buf = vec![0u8; 65536].into_boxed_slice(); // Max UDP packet size

        loop {
            // Limit the scope of the borrow
            let recv_result = {
                let state = transport.state.borrow();
                state.socket.recv_from(&mut buf)
            };

            match recv_result {
                Ok((size, addr)) => {
                    // Call the protocol's datagram_received method
                    // Now state is not borrowed, so sendto can work
                    transport.call_datagram_received(py, &buf[..size], addr);
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
                    event_loop.udp_socket_close(py, self.fd);
                    break;
                }
            }
        }
    }
}
