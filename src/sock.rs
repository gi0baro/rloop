use mio::Interest;
use pyo3::{IntoPyObjectExt, buffer::PyBuffer, prelude::*, types::PyTuple};
use std::os::raw::c_int;
use std::sync::{Arc, Mutex, atomic};

use crate::{
    event_loop::{EventLoop, EventLoopRunState},
    handles::Handle,
    py::{fut_done, fut_set_exc, fut_set_pyerr, fut_set_result, sock},
    utils::{WriteChunk, syscall},
};

// PyO3 removed private CPython's APIs
unsafe extern "C" {
    fn _PyBytes_Resize(bytes: *mut *mut pyo3::ffi::PyObject, newsize: pyo3::ffi::Py_ssize_t) -> std::os::raw::c_int;
}

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

pub(crate) enum PySockOp {
    Recv { fut: Py<PyAny>, n: usize },
    RecvInto { fut: Py<PyAny>, buf: Py<PyAny> },
    SendAll { fut: Py<PyAny>, chunk: WriteChunk },
}

unsafe impl Send for PySockOp {}

pub(crate) struct PySockHandleData {
    pub fd: i32,
    pub interest: Interest,
    pub reader: Option<PySockOp>,
    pub writer: Option<PySockOp>,
}

impl PySockHandleData {
    pub(crate) fn new(fd: i32, op: PySockOp, interest: Interest) -> Self {
        let (reader, writer) = match interest.is_writable() {
            true => (None, Some(op)),
            false => (Some(op), None),
        };
        Self {
            fd,
            interest,
            reader,
            writer,
        }
    }
}

enum PySockOpOutcome {
    Pending,
    Done,
}

impl PySockOp {
    #[inline]
    fn fut(&self) -> &Py<PyAny> {
        match self {
            Self::Recv { fut, .. } | Self::RecvInto { fut, .. } | Self::SendAll { fut, .. } => fut,
        }
    }

    fn run(&mut self, py: Python, fd: i32) -> PySockOpOutcome {
        if fut_done(py, self.fut()) {
            return PySockOpOutcome::Done;
        }

        match self {
            Self::Recv { fut, n } => Self::recv(py, fd, fut, *n),
            Self::RecvInto { fut, buf } => Self::recv_into(py, fd, fut, buf),
            Self::SendAll { fut, chunk } => Self::sendall(py, fd, fut, chunk),
        }
    }

    fn recv(py: Python, fd: i32, fut: &Py<PyAny>, n: usize) -> PySockOpOutcome {
        //: alloc an uninitialized `PyBytes` and read into it: single copy (kernel -> bytes), matching asyncio/CPython's `sock.recv`
        #[allow(clippy::cast_possible_wrap)]
        let mut obj = unsafe { pyo3::ffi::PyBytes_FromStringAndSize(std::ptr::null(), n as pyo3::ffi::Py_ssize_t) };
        if obj.is_null() {
            unsafe { pyo3::ffi::PyErr_Clear() };
            fut_set_exc(py, fut, std::io::Error::from(std::io::ErrorKind::OutOfMemory));
            return PySockOpOutcome::Done;
        }
        let buf = unsafe { pyo3::ffi::PyBytes_AsString(obj) };

        loop {
            match syscall!(recv(fd, buf.cast(), n, 0)) {
                #[allow(clippy::cast_sign_loss)]
                Ok(read) => {
                    let read = read as usize;
                    #[allow(clippy::cast_possible_wrap)]
                    if read < n && unsafe { _PyBytes_Resize(&raw mut obj, read as pyo3::ffi::Py_ssize_t) } != 0 {
                        unsafe { pyo3::ffi::PyErr_Clear() };
                        fut_set_exc(py, fut, std::io::Error::from(std::io::ErrorKind::OutOfMemory));
                        return PySockOpOutcome::Done;
                    }
                    let pydata = unsafe { Bound::from_owned_ptr(py, obj) };
                    fut_set_result(py, fut, pydata.unbind());
                    return PySockOpOutcome::Done;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    unsafe { pyo3::ffi::Py_DECREF(obj) };
                    return PySockOpOutcome::Pending;
                }
                Err(err) => {
                    unsafe { pyo3::ffi::Py_DECREF(obj) };
                    fut_set_exc(py, fut, err);
                    return PySockOpOutcome::Done;
                }
            }
        }
    }

    fn recv_into(py: Python, fd: i32, fut: &Py<PyAny>, buf: &Py<PyAny>) -> PySockOpOutcome {
        let pybuf: PyBuffer<u8> = match PyBuffer::get(buf.bind(py)) {
            Ok(v) => v,
            Err(err) => {
                fut_set_pyerr(py, fut, err);
                return PySockOpOutcome::Done;
            }
        };
        if !pybuf.is_c_contiguous() || pybuf.readonly() {
            fut_set_pyerr(
                py,
                fut,
                pyo3::exceptions::PyTypeError::new_err("a contiguous writable buffer is required"),
            );
            return PySockOpOutcome::Done;
        }

        loop {
            match syscall!(recv(fd, pybuf.buf_ptr(), pybuf.len_bytes(), 0)) {
                #[allow(clippy::cast_sign_loss)]
                Ok(read) => {
                    fut_set_result(py, fut, (read as usize).into_py_any(py).unwrap());
                    return PySockOpOutcome::Done;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => return PySockOpOutcome::Pending,
                Err(err) => {
                    fut_set_exc(py, fut, err);
                    return PySockOpOutcome::Done;
                }
            }
        }
    }

    fn sendall(py: Python, fd: i32, fut: &Py<PyAny>, chunk: &mut WriteChunk) -> PySockOpOutcome {
        loop {
            let data = chunk.as_slice();
            if data.is_empty() {
                fut_set_result(py, fut, py.None());
                return PySockOpOutcome::Done;
            }
            match syscall!(write(fd, data.as_ptr().cast(), data.len())) {
                #[allow(clippy::cast_sign_loss)]
                Ok(written) => {
                    chunk.off += written as usize;
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => return PySockOpOutcome::Pending,
                Err(err) => {
                    fut_set_exc(py, fut, err);
                    return PySockOpOutcome::Done;
                }
            }
        }
    }
}

pub(crate) struct PySockOpHandle {
    pub data: Arc<Mutex<PySockHandleData>>,
    pub write: bool,
}

impl Handle for PySockOpHandle {
    fn run(&self, py: Python, event_loop: &EventLoop, _state: &mut EventLoopRunState) {
        {
            let mut guard = self.data.lock().unwrap();
            let fd = guard.fd;
            let slot = match self.write {
                true => &mut guard.writer,
                false => &mut guard.reader,
            };
            let Some(op) = slot else {
                return;
            };
            match op.run(py, fd) {
                PySockOpOutcome::Pending => return,
                PySockOpOutcome::Done => {
                    *slot = None;
                }
            }
        }
        event_loop.py_sock_cleanup(&self.data);
    }
}
