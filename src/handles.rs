use pyo3::prelude::*;
use std::sync::atomic;

use crate::py::{run_in_ctx, run_in_ctx0, run_in_ctx1};

enum CBHandleCallType {
    Full(PyObject),
    OneArg(PyObject),
    NoArgs,
}

#[pyclass(frozen)]
pub(crate) struct CBHandle {
    cbtype: CBHandleCallType,
    callback: PyObject,
    // args: PyObject,
    context: PyObject,
    pub cancelled: atomic::AtomicBool,
}

impl CBHandle {
    pub(crate) fn new(callback: PyObject, args: PyObject, context: PyObject) -> Self {
        Self {
            cbtype: CBHandleCallType::Full(args),
            callback,
            context,
            cancelled: atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn new0(callback: PyObject, context: PyObject) -> Self {
        Self {
            cbtype: CBHandleCallType::NoArgs,
            callback,
            context,
            cancelled: atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn new1(callback: PyObject, arg: PyObject, context: PyObject) -> Self {
        Self {
            cbtype: CBHandleCallType::OneArg(arg),
            callback,
            context,
            cancelled: atomic::AtomicBool::new(false),
        }
    }

    pub fn run(&self, py: Python) -> Option<(PyErr, String)> {
        let ctx = self.context.as_ptr();
        let cb = self.callback.as_ptr();

        if let Err(err) = match &self.cbtype {
            CBHandleCallType::Full(obj) => {
                let args = obj.as_ptr();
                run_in_ctx!(py, ctx, cb, args)
            }
            CBHandleCallType::OneArg(obj) => {
                let arg = obj.as_ptr();
                run_in_ctx1!(py, ctx, cb, arg)
            }
            CBHandleCallType::NoArgs => {
                run_in_ctx0!(py, ctx, cb)
            }
        } {
            // TODO: better format for callback repr
            let msg = format!("Exception in callback {:?}", self.callback);
            return Some((err, msg));
        }

        None
    }
}

#[pymethods]
impl CBHandle {
    fn cancel(&self) {
        self.cancelled.store(true, atomic::Ordering::Relaxed);
    }

    fn cancelled(&self) -> bool {
        self.cancelled.load(atomic::Ordering::Relaxed)
    }
}

#[pyclass]
pub(crate) struct TimerHandle {
    handle: Py<CBHandle>,
    when: u128,
}

impl TimerHandle {
    pub fn new(handle: Py<CBHandle>, when: u128) -> Self {
        Self { handle, when }
    }
}

#[pymethods]
impl TimerHandle {
    fn cancel(&self) {
        self.handle.get().cancel();
    }

    fn cancelled(&self) -> bool {
        self.handle.get().cancelled()
    }

    #[getter(when)]
    #[allow(clippy::cast_precision_loss)]
    fn _get_when(&self) -> f64 {
        (self.when as f64) / 1_000_000.0
    }
}

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<CBHandle>()?;
    module.add_class::<TimerHandle>()?;

    Ok(())
}
