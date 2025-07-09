use pyo3::{IntoPyObjectExt, prelude::*};
use std::sync::atomic;

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    log::LogExc,
    py::{run_in_ctx, run_in_ctx0},
};

#[cfg(not(PyPy))]
use crate::py::run_in_ctx1;

pub trait Handle {
    fn run(&self, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState);
    fn cancelled(&self) -> bool {
        false
    }
}

pub(crate) type BoxedHandle = Box<dyn Handle + Send>;

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct CBHandle {
    callback: PyObject,
    args: PyObject,
    context: PyObject,
    cancelled: atomic::AtomicBool,
}

#[pyclass(frozen, module = "rloop._rloop", name = "CBHandle0")]
pub(crate) struct CBHandleNoArgs {
    callback: PyObject,
    context: PyObject,
    cancelled: atomic::AtomicBool,
}

#[pyclass(frozen, module = "rloop._rloop", name = "CBHandle1")]
pub(crate) struct CBHandleOneArg {
    callback: PyObject,
    arg: PyObject,
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

    #[allow(dead_code)]
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

macro_rules! cbhandle_cancel_impl {
    ($handle:ident) => {
        #[pymethods]
        impl $handle {
            fn cancel(&self) {
                self.cancelled.store(true, atomic::Ordering::Relaxed);
            }
        }
    };
}

macro_rules! cbhandle_cancelled_impl {
    () => {
        #[inline]
        fn cancelled(&self) -> bool {
            self.get().cancelled.load(atomic::Ordering::Relaxed)
        }
    };
}

cbhandle_cancel_impl!(CBHandle);
cbhandle_cancel_impl!(CBHandleNoArgs);
cbhandle_cancel_impl!(CBHandleOneArg);

impl Handle for Py<CBHandle> {
    cbhandle_cancelled_impl!();

    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let rself = self.get();
        let ctx = rself.context.as_ptr();
        let cb = rself.callback.as_ptr();
        let args = rself.args.as_ptr();

        if let Err(err) = run_in_ctx!(py, ctx, cb, args) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", rself.callback.bind(py)),
                self.clone_ref(py).into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }
}

impl Handle for Py<CBHandleNoArgs> {
    cbhandle_cancelled_impl!();

    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let rself = self.get();
        let ctx = rself.context.as_ptr();
        let cb = rself.callback.as_ptr();

        if let Err(err) = run_in_ctx0!(py, ctx, cb) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", rself.callback.bind(py)),
                self.clone_ref(py).into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }
}

impl Handle for Py<CBHandleOneArg> {
    #[cfg(not(PyPy))]
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let rself = self.get();
        let ctx = rself.context.as_ptr();
        let cb = rself.callback.as_ptr();
        let arg = rself.arg.as_ptr();

        if let Err(err) = run_in_ctx1!(py, ctx, cb, arg) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", rself.callback.bind(py)),
                self.clone_ref(py).into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }

    #[cfg(PyPy)]
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        let rself = self.get();
        let ctx = rself.context.as_ptr();
        let cb = rself.callback.as_ptr();
        let args = (rself.arg.clone_ref(py),).into_py_any(py).unwrap().into_ptr();

        if let Err(err) = run_in_ctx!(py, ctx, cb, args) {
            let err_ctx = LogExc::cb_handle(
                err,
                format!("Exception in callback {:?}", rself.callback.bind(py)),
                self.clone_ref(py).into_py_any(py).unwrap(),
            );
            _ = event_loop.log_exception(py, err_ctx);
        }
    }
}

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct TimerHandle {
    pub handle: Py<CBHandle>,
    #[pyo3(get)]
    when: f64,
}

impl TimerHandle {
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn new(handle: Py<CBHandle>, when: u128) -> Self {
        Self {
            handle,
            when: (when as f64) / 1_000_000.0,
        }
    }
}

#[pymethods]
impl TimerHandle {
    fn cancel(&self) {
        self.handle.get().cancel();
    }

    fn cancelled(&self) -> bool {
        self.handle.cancelled()
    }
}

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<CBHandle>()?;
    module.add_class::<TimerHandle>()?;

    Ok(())
}
