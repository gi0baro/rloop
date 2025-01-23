use pyo3::{prelude::*, IntoPyObjectExt};
use std::sync::{atomic, Arc};

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    log::LogExc,
    py::{run_in_ctx, run_in_ctx0},
};

#[cfg(not(PyPy))]
use crate::py::run_in_ctx1;

pub(crate) trait Handle {
    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState);
    fn cancel(&self) {}
    fn cancelled(&self) -> bool {
        false
    }
}

pub(crate) type HandleRef = Arc<dyn Handle + Send + Sync>;

pub(crate) struct CBHandle {
    callback: PyObject,
    args: PyObject,
    context: PyObject,
    cancelled: atomic::AtomicBool,
}

impl CBHandle {
    pub(crate) fn new(callback: PyObject, args: PyObject, context: PyObject) -> Self {
        Self {
            callback,
            args,
            context,
            cancelled: false.into(),
        }
    }

    pub(crate) fn new0(callback: PyObject, context: PyObject) -> CBHandleNoArgs {
        CBHandleNoArgs {
            callback,
            context,
            cancelled: false.into(),
        }
    }

    pub(crate) fn new1(callback: PyObject, arg: PyObject, context: PyObject) -> CBHandleOneArg {
        CBHandleOneArg {
            callback,
            arg,
            context,
            cancelled: false.into(),
        }
    }
}

pub(crate) struct CBHandleNoArgs {
    callback: PyObject,
    context: PyObject,
    cancelled: atomic::AtomicBool,
}

pub(crate) struct CBHandleOneArg {
    callback: PyObject,
    arg: PyObject,
    context: PyObject,
    cancelled: atomic::AtomicBool,
}

macro_rules! handle_cancel_impl {
    () => {
        #[inline]
        fn cancel(&self) {
            self.cancelled.store(true, atomic::Ordering::Release);
        }

        #[inline]
        fn cancelled(&self) -> bool {
            self.cancelled.load(atomic::Ordering::Relaxed)
        }
    };
}

impl Handle for CBHandle {
    handle_cancel_impl!();

    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let ctx = self.context.as_ptr();
        let cb = self.callback.as_ptr();
        let args = self.args.as_ptr();

        if let Err(err) = run_in_ctx!(py, ctx, cb, args) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", self.callback.bind(py)),
                PyHandle { handle: self }.into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }
}

impl Handle for CBHandleNoArgs {
    handle_cancel_impl!();

    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let ctx = self.context.as_ptr();
        let cb = self.callback.as_ptr();

        if let Err(err) = run_in_ctx0!(py, ctx, cb) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", self.callback.bind(py)),
                PyHandle { handle: self }.into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }
}

impl Handle for CBHandleOneArg {
    handle_cancel_impl!();

    #[cfg(not(PyPy))]
    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let ctx = self.context.as_ptr();
        let cb = self.callback.as_ptr();
        let arg = self.arg.as_ptr();

        if let Err(err) = run_in_ctx1!(py, ctx, cb, arg) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", self.callback.bind(py)),
                PyHandle { handle: self }.into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }

    #[cfg(PyPy)]
    fn run(self: Arc<Self>, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let ctx = self.context.as_ptr();
        let cb = self.callback.as_ptr();
        let args = (self.arg.clone_ref(py),).into_pyobject(py).unwrap().into_ptr();

        if let Err(err) = run_in_ctx!(py, ctx, cb, args) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", self.callback.bind(py)),
                PyHandle { handle: self }.into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }
}

#[pyclass(frozen, name = "CBHandle", module = "rloop._rloop")]
pub(crate) struct PyHandle {
    pub handle: HandleRef,
}

#[pymethods]
impl PyHandle {
    fn cancel(&self) {
        self.handle.cancel();
    }

    fn cancelled(&self) -> bool {
        self.handle.cancelled()
    }
}

#[pyclass(frozen, name = "TimerHandle", module = "rloop._rloop")]
pub(crate) struct PyTimerHandle {
    pub handle: HandleRef,
    #[pyo3(get)]
    when: f64,
}

impl PyTimerHandle {
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn new(handle: HandleRef, when: u128) -> Self {
        Self {
            handle,
            when: (when as f64) / 1_000_000.0,
        }
    }
}

#[pymethods]
impl PyTimerHandle {
    fn cancel(&self) {
        self.handle.cancel();
    }

    fn cancelled(&self) -> bool {
        self.handle.cancelled()
    }
}

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<PyHandle>()?;
    module.add_class::<PyTimerHandle>()?;

    Ok(())
}
