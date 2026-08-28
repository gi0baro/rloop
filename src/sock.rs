use pyo3::{prelude::*, types::PyTuple};
use std::os::raw::c_int;
use std::sync::{Arc, atomic};

use crate::py::sock;

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct SocketWrapper {
    sock: Py<PyAny>,
}

impl SocketWrapper {
    pub fn from_fd(py: Python, fd: usize, family: i32, r#type: socket2::Type, proto: usize) -> Py<Self> {
        Py::new(
            py,
            Self {
                sock: sock(py)
                    .unwrap()
                    .call1::<(i32, c_int, usize, usize)>((family, r#type.into(), proto, fd))
                    .unwrap()
                    .unbind(),
            },
        )
        .unwrap()
    }

    pub(crate) fn detach_inner(&self, py: Python) {
        _ = self.sock.call_method0(py, pyo3::intern!(py, "detach"));
    }
}

#[pymethods]
impl SocketWrapper {
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<Py<PyAny>> {
        self.sock.getattr(py, name)
    }
}

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct PseudoSocket {
    fd: i32,
    family: i32,
    stype: i32,
    proto: i32,
    closed: Arc<atomic::AtomicBool>,
}

impl PseudoSocket {
    pub(crate) fn new(fd: i32, family: i32, stype: i32, proto: i32, closed: Arc<atomic::AtomicBool>) -> Self {
        Self {
            fd,
            family,
            stype,
            proto,
            closed,
        }
    }

    fn with_sock<T>(&self, py: Python, func: impl FnOnce(&Bound<PyAny>) -> PyResult<T>) -> PyResult<T> {
        if self.closed.load(atomic::Ordering::Acquire) {
            return Err(pyo3::exceptions::PyOSError::new_err((
                libc::EBADF,
                "Bad file descriptor",
            )));
        }
        let pysock = sock(py)?.call1((self.family, self.stype, self.proto, self.fd))?;
        let ret = func(&pysock);
        let detached = pysock.call_method0(pyo3::intern!(py, "detach"));
        match ret {
            Err(err) => Err(err),
            Ok(v) => detached.map(|_| v),
        }
    }
}

#[pymethods]
impl PseudoSocket {
    #[getter]
    fn family(&self) -> i32 {
        self.family
    }

    #[getter]
    fn r#type(&self) -> i32 {
        self.stype
    }

    #[getter]
    fn proto(&self) -> i32 {
        self.proto
    }

    fn fileno(&self) -> i32 {
        if self.closed.load(atomic::Ordering::Acquire) {
            -1
        } else {
            self.fd
        }
    }

    fn dup(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| Ok(s.call_method0(pyo3::intern!(py, "dup"))?.unbind()))
    }

    fn get_inheritable(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| {
            Ok(s.call_method0(pyo3::intern!(py, "get_inheritable"))?.unbind())
        })
    }

    fn getsockname(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| Ok(s.call_method0(pyo3::intern!(py, "getsockname"))?.unbind()))
    }

    fn getpeername(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| Ok(s.call_method0(pyo3::intern!(py, "getpeername"))?.unbind()))
    }

    #[pyo3(signature = (*args))]
    fn getsockopt(&self, py: Python, args: Bound<PyTuple>) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| {
            Ok(s.call_method1(pyo3::intern!(py, "getsockopt"), args)?.unbind())
        })
    }

    #[pyo3(signature = (*args))]
    fn setsockopt(&self, py: Python, args: Bound<PyTuple>) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| {
            Ok(s.call_method1(pyo3::intern!(py, "setsockopt"), args)?.unbind())
        })
    }

    #[pyo3(signature = (*args))]
    fn shutdown(&self, py: Python, args: Bound<PyTuple>) -> PyResult<Py<PyAny>> {
        self.with_sock(py, |s| {
            Ok(s.call_method1(pyo3::intern!(py, "shutdown"), args)?.unbind())
        })
    }
}
