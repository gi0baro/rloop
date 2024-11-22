use pyo3::prelude::*;
use std::sync::atomic;

// pub(crate) enum Handle {
//     Callback(Py<CBHandle>),
//     // Timer(Py<TimerHandle>),
//     IO(Arc<CBHandle>),
// }

#[pyclass(frozen)]
pub(crate) struct CBHandle {
    callback: PyObject,
    args: PyObject,
    context: PyObject,
    pub cancelled: atomic::AtomicBool,
}

impl CBHandle {
    pub fn new(callback: PyObject, args: PyObject, context: PyObject) -> Self {
        Self {
            callback,
            args,
            context,
            cancelled: atomic::AtomicBool::new(false),
        }
    }

    pub fn run(&self, py: Python) -> Option<(PyErr, String)> {
        let ctx = self.context.as_ptr();
        let cb = self.callback.as_ptr();
        let args = self.args.as_ptr();

        let res: PyResult<Bound<PyAny>> = unsafe {
            pyo3::ffi::PyContext_Enter(ctx);
            let ptr = pyo3::ffi::PyObject_CallObject(cb, args);
            pyo3::ffi::PyContext_Exit(ctx);
            Bound::from_owned_ptr_or_err(py, ptr)
        };

        if let Err(err) = res {
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
