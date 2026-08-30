use pyo3::{IntoPyObjectExt, prelude::*};
use std::sync::{Arc, Mutex, atomic};

use crate::{
    event_loop::{EventLoop, EventLoopRunState, PyHandleData},
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
    callback: Py<PyAny>,
    args: Py<PyAny>,
    context: Py<PyAny>,
    cancelled: atomic::AtomicBool,
}

#[pyclass(frozen, module = "rloop._rloop", name = "CBHandle0")]
pub(crate) struct CBHandleNoArgs {
    callback: Py<PyAny>,
    context: Py<PyAny>,
    cancelled: atomic::AtomicBool,
}

#[pyclass(frozen, module = "rloop._rloop", name = "CBHandle1")]
pub(crate) struct CBHandleOneArg {
    callback: Py<PyAny>,
    arg: Py<PyAny>,
    context: Py<PyAny>,
    cancelled: atomic::AtomicBool,
}

impl CBHandle {
    pub(crate) fn new(callback: Py<PyAny>, args: Py<PyAny>, context: Py<PyAny>) -> Self {
        Self {
            callback,
            args,
            context,
            cancelled: false.into(),
        }
    }

    pub(crate) fn new0(callback: Py<PyAny>, context: Py<PyAny>) -> CBHandleNoArgs {
        CBHandleNoArgs {
            callback,
            context,
            cancelled: false.into(),
        }
    }

    pub(crate) fn new1(callback: Py<PyAny>, arg: Py<PyAny>, context: Py<PyAny>) -> CBHandleOneArg {
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

macro_rules! cb_handle_match_args {
    ($py:expr, $callback:expr, $args:expr, $context:expr) => {{
        let (handle, bhandle): (Py<PyAny>, BoxedHandle) = match $args.len() {
            0 => {
                let h = Py::new($py, CBHandle::new0($callback, $context)).unwrap();
                (h.clone_ref($py).into_any(), Box::new(h) as BoxedHandle)
            }
            1 => {
                let arg = $args.get_item(0).unwrap().unbind();
                let h = Py::new($py, CBHandle::new1($callback, arg, $context)).unwrap();
                (h.clone_ref($py).into_any(), Box::new(h) as BoxedHandle)
            }
            _ => {
                let h = Py::new(
                    $py,
                    CBHandle::new($callback, $args.clone().unbind().into_any(), $context),
                )
                .unwrap();
                (h.clone_ref($py).into_any(), Box::new(h) as BoxedHandle)
            }
        };
        (handle, bhandle)
    }};
}

pub(crate) use cb_handle_match_args;

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

//: level-triggered emulation wrapper for `add_reader`/`add_writer` callbacks
pub(crate) struct CBLTHandle {
    pub data: Arc<Mutex<PyHandleData>>,
    pub write: bool,
}

impl CBLTHandle {
    fn cb(&self, py: Python) -> Option<Py<CBHandle>> {
        let guard = self.data.lock().unwrap();
        let slot = match self.write {
            true => &guard.cbw,
            false => &guard.cbr,
        };
        match slot {
            Some(cb) if !Handle::cancelled(cb) => Some(cb.clone_ref(py)),
            _ => None,
        }
    }

    #[inline(always)]
    fn fd_ready(fd: i32, write: bool) -> bool {
        let events = match write {
            true => libc::POLLOUT,
            false => libc::POLLIN,
        };
        let mut pfd = libc::pollfd { fd, events, revents: 0 };
        match crate::utils::syscall!(poll(&raw mut pfd, 1, 0)) {
            Ok(n) if n > 0 => pfd.revents & (events | libc::POLLERR | libc::POLLHUP) != 0,
            _ => false,
        }
    }
}

impl Handle for CBLTHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, state: &mut EventLoopRunState) {
        let Some(cb) = self.cb(py) else {
            return;
        };
        cb.run(py, event_loop, state);
        let fd = self.data.lock().unwrap().fd;
        if CBLTHandle::fd_ready(fd, self.write) && self.cb(py).is_some() {
            _ = event_loop.schedule_handle(
                CBLTHandle {
                    data: self.data.clone(),
                    write: self.write,
                },
                None,
            );
        }
    }
}

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct TimerHandle {
    callback: Py<PyAny>,
    args: Py<PyAny>,
    context: Py<PyAny>,
    cancelled: atomic::AtomicBool,
    when: f64,
}

impl TimerHandle {
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn new(callback: Py<PyAny>, args: Py<PyAny>, context: Py<PyAny>, when: u128) -> Self {
        Self {
            callback,
            args,
            context,
            cancelled: false.into(),
            when: (when as f64) / 1_000_000.0,
        }
    }
}

#[pymethods]
impl TimerHandle {
    fn cancel(&self) {
        self.cancelled.store(true, atomic::Ordering::Relaxed);
    }

    fn cancelled(&self) -> bool {
        self.cancelled.load(atomic::Ordering::Relaxed)
    }

    fn when(&self) -> f64 {
        self.when
    }
}

impl Handle for Py<TimerHandle> {
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

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<CBHandle>()?;
    module.add_class::<TimerHandle>()?;

    Ok(())
}
