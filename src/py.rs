use pyo3::{prelude::*, sync::PyOnceLock};
use std::convert::Into;

static ASYNCIO: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
static ASYNCIO_PROTO_BUF: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
static SOCKET: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
static WEAKREF: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

fn asyncio(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>> {
    Ok(ASYNCIO
        .get_or_try_init(py, || py.import("asyncio").map(Into::into))?
        .bind(py))
}

fn socket(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>> {
    Ok(SOCKET
        .get_or_try_init(py, || py.import("socket").map(Into::into))?
        .bind(py))
}

pub(crate) fn asyncio_proto_buf(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>> {
    Ok(ASYNCIO_PROTO_BUF
        .get_or_try_init(py, || {
            let ret = asyncio(py)?.getattr("protocols")?.getattr("BufferedProtocol")?;
            Ok::<Py<PyAny>, PyErr>(ret.unbind())
        })?
        .bind(py))
}

fn weakref(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>> {
    Ok(WEAKREF
        .get_or_try_init(py, || py.import("weakref").map(Into::into))?
        .bind(py))
}

pub(crate) fn copy_context(py: Python) -> Py<PyAny> {
    let ctx = unsafe {
        let ptr = pyo3::ffi::PyContext_CopyCurrent();
        Bound::from_owned_ptr(py, ptr)
    };
    ctx.unbind()
}

pub(crate) fn sock(py: Python) -> PyResult<Bound<PyAny>> {
    socket(py)?.getattr(pyo3::intern!(py, "socket"))
}

pub(crate) fn weakset(py: Python) -> PyResult<Bound<PyAny>> {
    weakref(py)?.getattr("WeakSet")?.call0()
}

// pub(crate) fn weakvaldict(py: Python) -> PyResult<Bound<PyAny>> {
//     weakref(py)?.getattr("WeakValueDictionary")?.call0()
// }

macro_rules! run_in_ctx0 {
    ($py:expr, $ctx:expr, $cb:expr) => {
        unsafe {
            pyo3::ffi::PyContext_Enter($ctx);
            let ptr = pyo3::ffi::compat::PyObject_CallNoArgs($cb);
            pyo3::ffi::PyContext_Exit($ctx);
            Bound::from_owned_ptr_or_err($py, ptr)
        }
    };
}

#[cfg(not(PyPy))]
macro_rules! run_in_ctx1 {
    ($py:expr, $ctx:expr, $cb:expr, $arg:expr) => {
        unsafe {
            pyo3::ffi::PyContext_Enter($ctx);
            let ptr = pyo3::ffi::PyObject_CallOneArg($cb, $arg);
            pyo3::ffi::PyContext_Exit($ctx);
            Bound::from_owned_ptr_or_err($py, ptr)
        }
    };
}

macro_rules! run_in_ctx {
    ($py:expr, $ctx:expr, $cb:expr, $args:expr) => {
        unsafe {
            pyo3::ffi::PyContext_Enter($ctx);
            let ptr = pyo3::ffi::PyObject_CallObject($cb, $args);
            pyo3::ffi::PyContext_Exit($ctx);
            Bound::from_owned_ptr_or_err($py, ptr)
        }
    };
}

pub(crate) use run_in_ctx;
pub(crate) use run_in_ctx0;

#[cfg(not(PyPy))]
pub(crate) use run_in_ctx1;
