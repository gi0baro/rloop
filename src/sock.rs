use pyo3::prelude::*;
use std::os::raw::c_int;

use crate::py::sock;

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct SocketWrapper {
    sock: PyObject,
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
}

#[pymethods]
impl SocketWrapper {
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        self.sock.getattr(py, name)
    }
}

// #[pyclass(frozen)]
// pub(crate) struct PseudoSocket {
//     fd: usize,
//     #[pyo3(get)]
//     family: i32,
//     stype: socket2::Type,
//     #[pyo3(get)]
//     proto: usize,
// }

// impl PseudoSocket {
//     pub(crate) fn tcp_stream(fd: usize, family: i32, proto: usize) -> Self {
//         Self {
//             fd,
//             family,
//             stype: socket2::Type::STREAM,
//             proto,
//         }
//     }
// }

// #[pymethods]
// impl PseudoSocket {
//     fn r#type(&self) -> c_int {
//         self.stype.into()
//     }

//     fn getsockname<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
//         let pysock = sock(py)?.call1((self.family, self.r#type(), self.proto, self.fd))?;
//         let ret = pysock.call_method0(pyo3::intern!(py, "getsockname"));
//         pysock.call_method0(pyo3::intern!(py, "detach"))?;
//         ret
//     }

//     fn getpeername<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
//         let pysock = sock(py)?.call1((self.family, self.r#type(), self.proto, self.fd))?;
//         let ret = pysock.call_method0(pyo3::intern!(py, "getpeername"));
//         pysock.call_method0(pyo3::intern!(py, "detach"))?;
//         ret
//     }
// }
