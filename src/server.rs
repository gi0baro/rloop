use pyo3::prelude::*;
use std::sync::atomic;

use crate::event_loop::EventLoop;
use crate::tcp::TCPServer;

enum ServerType {
    TCP(TCPServer),
    // UDP,
    // Unix,
}

#[pyclass(frozen, module = "rloop._rloop")]
pub(crate) struct Server {
    #[pyo3(get)]
    _loop: Py<EventLoop>,
    #[pyo3(get)]
    _sockets: PyObject,
    closed: atomic::AtomicBool,
    serving: atomic::AtomicBool,
    servers: Vec<ServerType>,
    // serve_forever_fut: RwLock<Option<PyObject>>,
}

impl Server {
    pub(crate) fn tcp(event_loop: Py<EventLoop>, sockets: PyObject, servers: Vec<TCPServer>) -> Self {
        let srv: Vec<ServerType> = servers.into_iter().map(ServerType::TCP).collect();

        Self {
            _loop: event_loop,
            _sockets: sockets,
            closed: false.into(),
            serving: false.into(),
            servers: srv,
            // serve_forever_fut: RwLock::new(None),
            // waiters: RwLock::new(Vec::new()),
        }
    }
}

#[pymethods]
impl Server {
    // needed?
    // fn _add_waiter(&self, waiter: PyObject) {
    //     let mut guard = self.waiters.write().unwrap();
    //     guard.push(waiter);
    // }

    // #[getter(_sff)]
    // fn _get_sff(&self, py: Python) -> Option<PyObject> {
    //     let guard = self.serve_forever_fut.read().unwrap();
    //     guard.as_ref().map(|v| v.clone_ref(py))
    // }

    // #[setter(_sff)]
    // fn _set_sff(&self, val: PyObject) {
    //     let mut guard = self.serve_forever_fut.write().unwrap();
    //     *guard = Some(val);
    // }

    fn _start_serving(&self, py: Python) -> PyResult<()> {
        for server in &self.servers {
            match server {
                ServerType::TCP(inner) => inner.listen(py, self._loop.clone_ref(py))?,
            }
        }
        self.serving.store(true, atomic::Ordering::Release);
        Ok(())
    }

    fn _is_serving(&self) -> bool {
        self.serving.load(atomic::Ordering::Relaxed)
    }

    fn _close(&self, py: Python) {
        if self
            .closed
            .compare_exchange(false, true, atomic::Ordering::Release, atomic::Ordering::Relaxed)
            .is_ok()
        {
            let event_loop = self._loop.get();
            for server in &self.servers {
                match server {
                    ServerType::TCP(inner) => inner.close(py, event_loop),
                    // _ => {}
                }
            }
        }
        self.serving.store(false, atomic::Ordering::Release);
        // Ok(())
    }

    fn _streams_close(&self, py: Python) {
        let event_loop = self._loop.get();
        for server in &self.servers {
            match server {
                ServerType::TCP(inner) => inner.streams_close(py, event_loop),
            }
        }
    }

    fn _streams_abort(&self, py: Python) {
        let event_loop = self._loop.get();
        for server in &self.servers {
            match server {
                ServerType::TCP(inner) => inner.streams_abort(py, event_loop),
            }
        }
    }
}

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<Server>()?;

    Ok(())
}
