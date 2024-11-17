use pyo3::{prelude::*, sync::GILOnceCell};
use std::convert::Into;

static CONTEXTVARS: GILOnceCell<PyObject> = GILOnceCell::new();
static WEAKREF: GILOnceCell<PyObject> = GILOnceCell::new();

fn contextvars(py: Python) -> PyResult<&Bound<PyAny>> {
    Ok(CONTEXTVARS
        .get_or_try_init(py, || py.import_bound("contextvars").map(Into::into))?
        .bind(py))
}

fn weakref(py: Python) -> PyResult<&Bound<PyAny>> {
    Ok(WEAKREF
        .get_or_try_init(py, || py.import_bound("weakref").map(Into::into))?
        .bind(py))
}

pub(crate) fn copy_context(py: Python) -> PyResult<Bound<PyAny>> {
    contextvars(py)?.call_method0("copy_context")
}

pub(crate) fn weakset(py: Python) -> PyResult<Bound<PyAny>> {
    weakref(py)?.getattr("WeakSet")?.call0()
}
