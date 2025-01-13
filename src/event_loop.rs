use std::{
    collections::{BinaryHeap, VecDeque},
    io::{Read, Write},
    mem,
    os::fd::{AsRawFd, FromRawFd},
    sync::{atomic, Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use anyhow::Result;
use dashmap::DashMap;
use mio::{event, Interest, Poll, Token};
use pyo3::prelude::*;

use crate::{
    handles::{CBHandle, TimerHandle},
    io::Source,
    log::{log_exc_to_py_ctx, LogExc},
    py::{copy_context, weakset, weakvaldict},
    server::Server,
    tcp::{TCPServer, TCPServerRef, TCPTransport},
    time::Timer,
};

enum Handle {
    Internal(Source),
    Py(PyHandleData),
    TCPListener(TCPListenerHandleData),
    TCPStream(TCPStreamHandleData),
}

struct PyHandleData {
    source: Source,
    interest: Interest,
    cbr: Option<Py<CBHandle>>,
    cbw: Option<Py<CBHandle>>,
}

struct TCPListenerHandleData {
    source: Source,
    server: TCPServerRef,
}

struct TCPStreamHandleData {
    source: Source,
    interest: Interest,
    transport: Py<TCPTransport>,
}

#[pyclass(frozen, subclass)]
pub struct EventLoop {
    idle: atomic::AtomicBool,
    io: Arc<Mutex<Poll>>,
    handles_io: Arc<DashMap<Token, Handle>>,
    handles_ready: Arc<Mutex<VecDeque<Py<CBHandle>>>>,
    handles_sched: Arc<Mutex<BinaryHeap<Timer>>>,
    epoch: Instant,
    counter_ready: atomic::AtomicUsize,
    counter_io: atomic::AtomicU16,
    ssock: Arc<RwLock<Option<(socket2::Socket, socket2::Socket)>>>,
    tick_last_poll: atomic::AtomicU64,
    closed: atomic::AtomicBool,
    exc_handler: Arc<RwLock<PyObject>>,
    exception_handler: Arc<RwLock<PyObject>>,
    executor: Arc<RwLock<PyObject>>,
    sig_handlers: Arc<DashMap<u8, Py<CBHandle>>>,
    sig_listening: atomic::AtomicBool,
    sig_loop_handled: atomic::AtomicBool,
    sig_wfd: Arc<RwLock<PyObject>>,
    stopping: atomic::AtomicBool,
    shutdown_called_asyncgens: atomic::AtomicBool,
    shutdown_called_executor: atomic::AtomicBool,
    ssock_r: Arc<RwLock<PyObject>>,
    ssock_w: Arc<RwLock<PyObject>>,
    task_factory: Arc<RwLock<PyObject>>,
    thread_id: atomic::AtomicI64,
    watcher_child: Arc<RwLock<PyObject>>,
    #[pyo3(get)]
    _asyncgens: PyObject,
    #[pyo3(get)]
    _base_ctx: PyObject,
    #[pyo3(get)]
    _transports: PyObject,
}

impl EventLoop {
    #[inline]
    fn step(&self, py: Python) -> std::result::Result<(), std::io::Error> {
        let mut io_events = event::Events::with_capacity(128);
        let mut sched_time: Option<u64> = None;
        let mut skip_poll = false;

        // compute poll timeout based on scheduled work
        if self.counter_ready.load(atomic::Ordering::Relaxed) > 0 {
            sched_time = Some(0);
            // we want to skip polling when unnecessary:
            // if work is ready we check the time since last poll and skip for max 250μs
            let tick = Instant::now().duration_since(self.epoch).as_micros() as u64;
            if (tick - self.tick_last_poll.load(atomic::Ordering::Relaxed)) < 250 {
                skip_poll = true;
            }
        } else {
            let guard_sched = self.handles_sched.lock().unwrap();
            if let Some(timer) = guard_sched.peek() {
                let tick = Instant::now().duration_since(self.epoch).as_micros();
                if timer.when > tick {
                    let dt = ((timer.when - tick) / 1000) as u64;
                    sched_time = Some(dt);
                }
            }
            drop(guard_sched);
        }

        // I/O
        let poll_result = match skip_poll {
            true => Ok(()),
            false => py.allow_threads(|| {
                let mut io = self.io.lock().unwrap();
                if sched_time.is_none() {
                    self.idle.store(true, atomic::Ordering::Relaxed);
                }
                let res = io.poll(&mut io_events, sched_time.map(Duration::from_millis));
                if let Err(ref err) = res {
                    if err.kind() == std::io::ErrorKind::Interrupted {
                        // if we got an interrupt, we retry ready events (as we might need to process signals)
                        let _ = io.poll(&mut io_events, Some(Duration::from_millis(0)));
                    }
                }
                self.idle.store(false, atomic::Ordering::Relaxed);
                self.tick_last_poll.store(
                    Instant::now().duration_since(self.epoch).as_micros() as u64,
                    atomic::Ordering::Relaxed,
                );
                res
            }),
        };
        let mut guard_cb = self.handles_ready.lock().unwrap();
        let mut cb_handles = mem::replace(&mut *guard_cb, VecDeque::with_capacity(128));
        self.counter_ready
            .fetch_sub(cb_handles.len(), atomic::Ordering::Relaxed);
        drop(guard_cb);

        for event in &io_events {
            // NOTE: cancellation is not necessary as we have custom futures
            if let Some(io_handle) = self.handles_io.get(&event.token()) {
                match io_handle.value() {
                    Handle::Py(handle) => self.handle_io_py(py, event, handle, &mut cb_handles),
                    Handle::TCPListener(handle) => self.handle_io_tcpl(py, handle, &mut cb_handles),
                    Handle::TCPStream(handle) => self.handle_io_tcps(py, event, handle, &mut cb_handles),
                    Handle::Internal(_) => self.handle_io_internal(py, &mut cb_handles),
                }
            }
        }

        // timers
        let mut guard_sched = self.handles_sched.lock().unwrap();
        if let Some(timer) = guard_sched.peek() {
            let tick = Instant::now().duration_since(self.epoch).as_micros();
            if timer.when <= tick {
                while let Some(timer) = guard_sched.peek() {
                    if timer.when > tick {
                        break;
                    }
                    cb_handles.push_back(guard_sched.pop().unwrap().handle);
                }
            }
        }
        drop(guard_sched);

        // callbacks
        while let Some(cb_handle) = cb_handles.pop_front() {
            // let handle = match cb_handle {
            //     Handle::Callback(ref v) => v.get(),
            //     Handle::IO(ref v) => v,
            //     // _ => unreachable!()
            // };
            let handle = cb_handle.get();
            if !handle.cancelled.load(atomic::Ordering::Relaxed) {
                if let Some((err, msg)) = handle.run(py) {
                    let err_ctx = LogExc::cb_handle(err, msg, cb_handle.clone_ref(py).into_any());
                    let _ = self.log_exception(py, err_ctx);
                }
            }
        }

        poll_result
    }

    #[inline]
    fn handle_io_py(
        &self,
        py: Python,
        event: &event::Event,
        handle: &PyHandleData,
        handles: &mut VecDeque<Py<CBHandle>>,
    ) {
        if let Some(cbr) = &handle.cbr {
            if event.is_readable() {
                handles.push_back(cbr.clone_ref(py));
            }
        }
        if let Some(cbw) = &handle.cbw {
            if event.is_writable() {
                handles.push_back(cbw.clone_ref(py));
            }
        }
    }

    #[inline]
    fn handle_io_tcpl(&self, py: Python, handle: &TCPListenerHandleData, handles: &mut VecDeque<Py<CBHandle>>) {
        py.allow_threads(|| loop {
            match handle.server.listener.accept() {
                Ok((stream, _)) => {
                    self.tcp_stream_add(
                        Python::with_gil(|pyi| {
                            let fd = stream.as_raw_fd();
                            let transport =
                                Py::new(pyi, handle.server.transport(pyi, stream)).expect("cannot build transport");
                            let _ = self._transports.bind(pyi).set_item(fd, transport.clone_ref(pyi));
                            let handle = TCPTransport::attach(transport.clone_ref(pyi), pyi);
                            self.schedule_cbhandle(pyi, handles, handle)
                                .expect("cannot schedule handle");
                            transport
                        }),
                        Interest::READABLE,
                    );
                }
                // Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                // Err(_) => {}
                Err(_) => break,
            }
        });
    }

    #[inline]
    fn handle_io_tcps(
        &self,
        py: Python,
        event: &event::Event,
        handle: &TCPStreamHandleData,
        handles: &mut VecDeque<Py<CBHandle>>,
    ) {
        // let transport = handle.transport.get();

        if event.is_readable() {
            let handle = TCPTransport::recv(handle.transport.clone_ref(py), py);
            handles.push_back(Py::new(py, handle).unwrap());
        } else if event.is_writable() {
        } else if event.is_read_closed() {
        } else if event.is_write_closed() {
        }
    }

    #[inline]
    fn handle_io_internal(&self, py: Python, handles_ready: &mut VecDeque<Py<CBHandle>>) {
        let sigs = py.allow_threads(|| self.read_from_self());
        for sig in sigs {
            self.sig_handle(py, sig, handles_ready);
        }
    }

    #[inline]
    fn sig_handle(&self, py: Python, sig: u8, handles_ready: &mut VecDeque<Py<CBHandle>>) {
        if let Some(pyhandle) = self.sig_handlers.get(&sig) {
            self.sig_loop_handled.store(true, atomic::Ordering::Relaxed);

            if pyhandle.get().cancelled.load(atomic::Ordering::Relaxed) {
                self._sig_rem(sig);
            } else {
                handles_ready.push_back(pyhandle.clone_ref(py));
            }
        }
    }

    pub(crate) fn schedule_cbhandle(
        &self,
        py: Python,
        handles: &mut VecDeque<Py<CBHandle>>,
        handle: CBHandle,
    ) -> Result<()> {
        handles.push_back(Py::new(py, handle)?);
        Ok(())
    }

    #[inline(always)]
    fn wake(&self) {
        let mut guard = self.ssock.write().unwrap();
        if let Some((_, sockw)) = guard.as_mut() {
            let _ = sockw.write(b"\0");
        }
    }

    fn read_from_self(&self) -> Vec<u8> {
        let mut guard = self.ssock.write().unwrap();
        if let Some((sockr, _)) = guard.as_mut() {
            let mut buf = [0; 4096];
            let mut len = 0;
            loop {
                match sockr.read(&mut buf[len..]) {
                    Ok(0) => break,
                    Ok(readn) => {
                        len += readn;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
            if self.sig_listening.load(atomic::Ordering::Relaxed) {
                let mut sigs = Vec::with_capacity(len);
                for sig in &buf[..len] {
                    if *sig == 0 {
                        continue;
                    }
                    sigs.push(*sig);
                }
                return sigs;
            }
        }
        Vec::new()
    }

    #[inline]
    fn reader_rem(&self, token: Token) -> Result<bool> {
        if let Some((_, Handle::Py(mut item))) = self.handles_io.remove(&token) {
            let guard_poll = self.io.lock().unwrap();
            match item.interest {
                Interest::READABLE => {
                    self.counter_io.fetch_sub(1, atomic::Ordering::Relaxed);
                    guard_poll.registry().deregister(&mut item.source)?;
                }
                _ => {
                    let interest = Interest::WRITABLE;
                    guard_poll.registry().reregister(&mut item.source, token, interest)?;
                    self.handles_io.insert(
                        token,
                        Handle::Py(PyHandleData {
                            source: item.source,
                            interest,
                            cbr: None,
                            cbw: item.cbw,
                        }),
                    );
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    #[inline]
    fn writer_rem(&self, token: Token) -> Result<bool> {
        if let Some((_, Handle::Py(mut item))) = self.handles_io.remove(&token) {
            let guard_poll = self.io.lock().unwrap();
            match item.interest {
                Interest::WRITABLE => {
                    self.counter_io.fetch_sub(1, atomic::Ordering::Relaxed);
                    guard_poll.registry().deregister(&mut item.source)?;
                }
                _ => {
                    let interest = Interest::READABLE;
                    guard_poll.registry().reregister(&mut item.source, token, interest)?;
                    self.handles_io.insert(
                        token,
                        Handle::Py(PyHandleData {
                            source: item.source,
                            interest,
                            cbr: item.cbr,
                            cbw: None,
                        }),
                    );
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn tcp_listener_add(&self, server: TCPServerRef) {
        let token = Token(server.listener.as_raw_fd() as usize);
        let mut source = Source::TCPListener(server.listener.clone());

        let guard_poll = self.io.lock().unwrap();
        let _ = guard_poll.registry().register(&mut source, token, Interest::READABLE);
        self.handles_io
            .insert(token, Handle::TCPListener(TCPListenerHandleData { source, server }));
    }

    pub(crate) fn tcp_listener_rem(&self, fd: usize) -> Result<bool> {
        let token = Token(fd);
        if let Some((_, handle)) = self.handles_io.remove(&token) {
            match handle {
                Handle::TCPListener(mut item) => {
                    let guard_poll = self.io.lock().unwrap();
                    guard_poll.registry().deregister(&mut item.source)?;
                    return Ok(true);
                }
                _ => unreachable!(),
            }
        }
        Ok(false)
    }

    pub(crate) fn tcp_stream_add(&self, transport: Py<TCPTransport>, interest: Interest) {
        let rtransport = transport.get();
        let token = Token(rtransport.fd);
        let guard_poll = self.io.lock().unwrap();
        match self.handles_io.remove(&token) {
            Some((_, Handle::TCPStream(mut item))) => {
                if item.interest == interest {
                    return;
                }

                let interests = interest | item.interest;
                let _ = guard_poll.registry().reregister(&mut item.source, token, interests);
                drop(guard_poll);

                self.handles_io.insert(
                    token,
                    Handle::TCPStream(TCPStreamHandleData {
                        source: item.source,
                        interest: interests,
                        transport: item.transport,
                    }),
                );
            }
            _ => {
                let mut source = Source::TCPStream(rtransport.stream.clone());
                let _ = guard_poll.registry().register(&mut source, token, interest);
                drop(guard_poll);

                self.handles_io.insert(
                    token,
                    Handle::TCPStream(TCPStreamHandleData {
                        source,
                        interest,
                        transport,
                    }),
                );
            }
        }
    }

    pub(crate) fn tcp_stream_rem(&self, transport: Py<TCPTransport>, interest: Interest) {
        let rtransport = transport.get();
        let token = Token(rtransport.fd);
        let guard_poll = self.io.lock().unwrap();

        if let Some((_, Handle::TCPStream(mut item))) = self.handles_io.remove(&token) {
            let interests = item.interest.remove(interest);
            if interests.is_none() {
                let _ = guard_poll.registry().deregister(&mut item.source);
                return;
            }

            let _ = guard_poll
                .registry()
                .reregister(&mut item.source, token, interests.unwrap());
            drop(guard_poll);

            self.handles_io.insert(
                token,
                Handle::TCPStream(TCPStreamHandleData {
                    source: item.source,
                    interest: interests.unwrap(),
                    transport: item.transport,
                }),
            );
        }
    }

    pub(crate) fn log_exception(&self, py: Python, ctx: LogExc) -> PyResult<PyObject> {
        let handler = self.exc_handler.read().unwrap();
        handler.call1(
            py,
            (
                log_exc_to_py_ctx(py, ctx),
                self.exception_handler.read().unwrap().clone_ref(py),
            ),
        )
    }

    pub fn schedule0(&self, py: Python, callback: PyObject, context: Option<PyObject>) {
        let pyctx = context.unwrap_or(self._base_ctx.clone_ref(py));
        let handle = Py::new(py, CBHandle::new0(callback, pyctx)).expect("cannot create handle");
        let mut handles = self.handles_ready.lock().expect("cannot lock handles");
        handles.push_back(handle);
    }

    pub fn schedule1(&self, py: Python, callback: PyObject, arg: PyObject, context: Option<PyObject>) {
        let pyctx = context.unwrap_or(self._base_ctx.clone_ref(py));
        let handle = Py::new(py, CBHandle::new1(callback, arg, pyctx)).expect("cannot create handle");
        let mut handles = self.handles_ready.lock().expect("cannot lock handles");
        handles.push_back(handle);
    }

    pub fn schedule(&self, py: Python, callback: PyObject, args: PyObject, context: Option<PyObject>) {
        let pyctx = context.unwrap_or(self._base_ctx.clone_ref(py));
        let handle = Py::new(py, CBHandle::new(callback, args, pyctx)).expect("cannot create handle");
        let mut handles = self.handles_ready.lock().expect("cannot lock handles");
        handles.push_back(handle);
    }

    pub fn schedule_later0(&self, py: Python, delay: Duration, callback: PyObject, context: Option<PyObject>) {
        let pyctx = context.unwrap_or(self._base_ctx.clone_ref(py));
        let when = (Instant::now().duration_since(self.epoch) + delay).as_micros();
        let handle = Py::new(py, CBHandle::new0(callback, pyctx)).expect("cannot create handle");
        let mut guard = self.handles_sched.lock().expect("cannot lock handles");
        guard.push(Timer { handle, when });
    }

    pub fn schedule_later1(
        &self,
        py: Python,
        delay: Duration,
        callback: PyObject,
        arg: PyObject,
        context: Option<PyObject>,
    ) {
        let pyctx = context.unwrap_or(self._base_ctx.clone_ref(py));
        let when = (Instant::now().duration_since(self.epoch) + delay).as_micros();
        let handle = Py::new(py, CBHandle::new1(callback, arg, pyctx)).expect("cannot create handle");
        let mut guard = self.handles_sched.lock().expect("cannot lock handles");
        guard.push(Timer { handle, when });
    }

    pub fn schedule_later(
        &self,
        py: Python,
        delay: Duration,
        callback: PyObject,
        args: PyObject,
        context: Option<PyObject>,
    ) {
        let pyctx = context.unwrap_or(self._base_ctx.clone_ref(py));
        let when = (Instant::now().duration_since(self.epoch) + delay).as_micros();
        let handle = Py::new(py, CBHandle::new(callback, args, pyctx)).expect("cannot create handle");
        let mut guard = self.handles_sched.lock().expect("cannot lock handles");
        guard.push(Timer { handle, when });
    }
}

#[pymethods]
impl EventLoop {
    #[new]
    fn new(py: Python) -> PyResult<Self> {
        Ok(Self {
            idle: atomic::AtomicBool::new(false),
            io: Arc::new(Mutex::new(Poll::new()?)),
            handles_io: Arc::new(DashMap::with_capacity(128)),
            handles_ready: Arc::new(Mutex::new(VecDeque::with_capacity(128))),
            handles_sched: Arc::new(Mutex::new(BinaryHeap::with_capacity(32))),
            epoch: Instant::now(),
            counter_ready: atomic::AtomicUsize::new(0),
            counter_io: atomic::AtomicU16::new(0),
            ssock: Arc::new(RwLock::new(None)),
            tick_last_poll: atomic::AtomicU64::new(0),
            closed: atomic::AtomicBool::new(false),
            exc_handler: Arc::new(RwLock::new(py.None())),
            exception_handler: Arc::new(RwLock::new(py.None())),
            executor: Arc::new(RwLock::new(py.None())),
            sig_handlers: Arc::new(DashMap::with_capacity(32)),
            sig_listening: atomic::AtomicBool::new(false),
            sig_loop_handled: atomic::AtomicBool::new(false),
            sig_wfd: Arc::new(RwLock::new(py.None())),
            stopping: atomic::AtomicBool::new(false),
            shutdown_called_asyncgens: atomic::AtomicBool::new(false),
            shutdown_called_executor: atomic::AtomicBool::new(false),
            ssock_r: Arc::new(RwLock::new(py.None())),
            ssock_w: Arc::new(RwLock::new(py.None())),
            task_factory: Arc::new(RwLock::new(py.None())),
            thread_id: atomic::AtomicI64::new(0),
            watcher_child: Arc::new(RwLock::new(py.None())),
            _asyncgens: weakset(py)?.unbind(),
            _base_ctx: copy_context(py),
            _transports: weakvaldict(py)?.unbind(),
        })
    }

    #[getter(_clock)]
    fn _get_clock(&self) -> u128 {
        Instant::now().duration_since(self.epoch).as_micros()
    }

    #[getter(_thread_id)]
    fn _get_thread_id(&self) -> i64 {
        self.thread_id.load(atomic::Ordering::Relaxed)
    }

    #[setter(_thread_id)]
    fn _set_thread_id(&self, val: i64) {
        self.thread_id.store(val, atomic::Ordering::Relaxed);
    }

    #[getter(_closed)]
    fn _get_closed(&self) -> bool {
        self.closed.load(atomic::Ordering::Relaxed)
    }

    #[setter(_closed)]
    fn _set_closed(&self, val: bool) {
        self.closed.store(val, atomic::Ordering::Relaxed);
    }

    #[getter(_stopping)]
    fn _get_stopping(&self) -> bool {
        self.stopping.load(atomic::Ordering::Relaxed)
    }

    #[setter(_stopping)]
    fn _set_stopping(&self, val: bool) {
        self.stopping.store(val, atomic::Ordering::Relaxed);
    }

    #[getter(_asyncgens_shutdown_called)]
    fn _get_asyncgens_shutdown_called(&self) -> bool {
        self.shutdown_called_asyncgens.load(atomic::Ordering::Relaxed)
    }

    #[setter(_asyncgens_shutdown_called)]
    fn _set_asyncgens_shutdown_called(&self, val: bool) {
        self.shutdown_called_asyncgens.store(val, atomic::Ordering::Relaxed);
    }

    #[getter(_default_executor)]
    fn _get_default_executor(&self, py: Python) -> PyObject {
        self.executor.read().unwrap().clone_ref(py)
    }

    #[setter(_default_executor)]
    fn _set_default_executor(&self, val: PyObject) {
        let mut guard = self.executor.write().unwrap();
        *guard = val;
    }

    #[getter(_exc_handler)]
    fn _get_exc_handler(&self, py: Python) -> PyObject {
        self.exc_handler.read().unwrap().clone_ref(py)
    }

    #[setter(_exc_handler)]
    fn _set_exc_handler(&self, val: PyObject) {
        let mut guard = self.exc_handler.write().unwrap();
        *guard = val;
    }

    #[getter(_exception_handler)]
    fn _get_exception_handler(&self, py: Python) -> PyObject {
        self.exception_handler.read().unwrap().clone_ref(py)
    }

    #[setter(_exception_handler)]
    fn _set_exception_handler(&self, val: PyObject) {
        let mut guard = self.exception_handler.write().unwrap();
        *guard = val;
    }

    #[getter(_executor_shutdown_called)]
    fn _get_executor_shutdown_called(&self) -> bool {
        self.shutdown_called_executor.load(atomic::Ordering::Relaxed)
    }

    #[setter(_executor_shutdown_called)]
    fn _set_executor_shutdown_called(&self, val: bool) {
        self.shutdown_called_executor.store(val, atomic::Ordering::Relaxed);
    }

    #[getter(_sig_listening)]
    fn _get_sig_listening(&self) -> bool {
        self.sig_listening.load(atomic::Ordering::Relaxed)
    }

    #[setter(_sig_listening)]
    fn _set_sig_listening(&self, val: bool) {
        self.sig_listening.store(val, atomic::Ordering::Relaxed);
    }

    #[getter(_sig_wfd)]
    fn _get_sig_wfd(&self, py: Python) -> PyObject {
        self.sig_wfd.read().unwrap().clone_ref(py)
    }

    #[setter(_sig_wfd)]
    fn _set_sig_wfd(&self, val: PyObject) {
        let mut guard = self.sig_wfd.write().unwrap();
        *guard = val;
    }

    #[getter(_ssock_r)]
    fn _get_ssock_r(&self, py: Python) -> PyObject {
        self.ssock_r.read().unwrap().clone_ref(py)
    }

    #[setter(_ssock_r)]
    fn _set_ssock_r(&self, val: PyObject) {
        let mut guard = self.ssock_r.write().unwrap();
        *guard = val;
    }

    #[getter(_ssock_w)]
    fn _get_ssock_w(&self, py: Python) -> PyObject {
        self.ssock_w.read().unwrap().clone_ref(py)
    }

    #[setter(_ssock_w)]
    fn _set_ssock_w(&self, val: PyObject) {
        let mut guard = self.ssock_w.write().unwrap();
        *guard = val;
    }

    #[getter(_task_factory)]
    fn _get_task_factory(&self, py: Python) -> PyObject {
        self.task_factory.read().unwrap().clone_ref(py)
    }

    #[setter(_task_factory)]
    fn _set_task_factory(&self, factory: PyObject) {
        let mut guard = self.task_factory.write().unwrap();
        *guard = factory;
    }

    #[getter(_watcher_child)]
    fn _get_watcher_child(&self, py: Python) -> PyObject {
        self.watcher_child.read().unwrap().clone_ref(py)
    }

    #[setter(_watcher_child)]
    fn _set_watcher_child(&self, factory: PyObject) {
        let mut guard = self.watcher_child.write().unwrap();
        *guard = factory;
    }

    fn _ssock_set(&self, fd_r: usize, fd_w: usize) -> PyResult<()> {
        let mut guard = self.ssock.write().unwrap();
        *guard = Some(unsafe {
            (
                #[allow(clippy::cast_possible_wrap)]
                socket2::Socket::from_raw_fd(fd_r as i32),
                #[allow(clippy::cast_possible_wrap)]
                socket2::Socket::from_raw_fd(fd_w as i32),
            )
        });

        let token = Token(fd_r);
        let mut source = Source::FD(fd_r.try_into()?);
        let interest = Interest::READABLE;
        let guard_poll = self.io.lock().unwrap();
        guard_poll.registry().register(&mut source, token, interest)?;
        drop(guard_poll);
        self.handles_io.insert(token, Handle::Internal(source));

        Ok(())
    }

    fn _ssock_del(&self, fd_r: usize) -> PyResult<()> {
        let token = Token(fd_r);
        if let Some((_, Handle::Internal(mut source))) = self.handles_io.remove(&token) {
            let guard_poll = self.io.lock().unwrap();
            guard_poll.registry().deregister(&mut source)?;
        }
        self.ssock.write().unwrap().take();
        Ok(())
    }

    #[pyo3(signature = (callback, *args, context=None))]
    fn call_soon(
        &self,
        py: Python,
        callback: PyObject,
        args: PyObject,
        context: Option<PyObject>,
    ) -> PyResult<Py<CBHandle>> {
        let handle = Py::new(
            py,
            CBHandle::new(callback, args, context.unwrap_or_else(|| copy_context(py))),
        )?;
        let mut guard = self.handles_ready.lock().unwrap();
        guard.push_back(handle.clone_ref(py));
        self.counter_ready.fetch_add(1, atomic::Ordering::Relaxed);
        drop(guard);
        Ok(handle)
    }

    #[pyo3(signature = (callback, *args, context=None))]
    fn call_soon_threadsafe(
        &self,
        py: Python,
        callback: PyObject,
        args: PyObject,
        context: Option<PyObject>,
    ) -> PyResult<Py<CBHandle>> {
        let handle = Py::new(
            py,
            CBHandle::new(callback, args, context.unwrap_or_else(|| self._base_ctx.clone_ref(py))),
        )?;
        let mut guard = self.handles_ready.lock().unwrap();
        guard.push_back(handle.clone_ref(py));
        self.counter_ready.fetch_add(1, atomic::Ordering::Relaxed);
        drop(guard);
        // wake when necessary
        if self.idle.load(atomic::Ordering::Relaxed) {
            py.allow_threads(|| self.wake());
        }
        Ok(handle)
    }

    fn _call_later(
        &self,
        py: Python,
        delay: u64,
        callback: PyObject,
        args: PyObject,
        context: PyObject,
    ) -> PyResult<Py<TimerHandle>> {
        let when = Instant::now().duration_since(self.epoch).as_micros() + u128::from(delay);
        let handle = Py::new(py, CBHandle::new(callback, args, context))?;
        let thandle = Py::new(py, TimerHandle::new(handle.clone_ref(py), when))?;
        let mut guard = self.handles_sched.lock().unwrap();
        guard.push(Timer { handle, when });
        drop(guard);
        Ok(thandle)
    }

    fn _reader_add(
        &self,
        py: Python,
        fd: usize,
        callback: PyObject,
        args: PyObject,
        context: PyObject,
    ) -> PyResult<Py<CBHandle>> {
        let token = Token(fd);
        let handle = Py::new(py, CBHandle::new(callback, args, context))?;
        if let Some(mut item) = self.handles_io.get_mut(&token) {
            if let Handle::Py(data) = item.value_mut() {
                let interest = data.interest | Interest::READABLE;
                let guard_poll = self.io.lock().unwrap();
                guard_poll.registry().reregister(&mut data.source, token, interest)?;
                drop(guard_poll);
                data.interest = interest;
                data.cbr = Some(handle.clone_ref(py));
                return Ok(handle);
            }
        }

        let mut source = Source::FD(fd.try_into()?);
        let interest = Interest::READABLE;
        let guard_poll = self.io.lock().unwrap();
        guard_poll.registry().register(&mut source, token, interest)?;
        drop(guard_poll);
        self.handles_io.insert(
            token,
            Handle::Py(PyHandleData {
                source,
                interest,
                cbr: Some(handle.clone_ref(py)),
                cbw: None,
            }),
        );
        self.counter_io.fetch_add(1, atomic::Ordering::Relaxed);

        Ok(handle)
    }

    fn _reader_rem(&self, fd: usize) -> Result<bool> {
        let token = Token(fd);
        self.reader_rem(token)
    }

    fn _writer_add(
        &self,
        py: Python,
        fd: usize,
        callback: PyObject,
        args: PyObject,
        context: PyObject,
    ) -> PyResult<Py<CBHandle>> {
        let token = Token(fd);
        let handle = Py::new(py, CBHandle::new(callback, args, context))?;

        if let Some(mut item) = self.handles_io.get_mut(&token) {
            if let Handle::Py(data) = item.value_mut() {
                let interest = data.interest | Interest::WRITABLE;
                let guard_poll = self.io.lock().unwrap();
                guard_poll.registry().reregister(&mut data.source, token, interest)?;
                drop(guard_poll);
                data.interest = interest;
                data.cbw = Some(handle.clone_ref(py));
                return Ok(handle);
            }
        }

        let mut source = Source::FD(fd.try_into()?);
        let interest = Interest::WRITABLE;
        let guard_poll = self.io.lock().unwrap();
        guard_poll.registry().register(&mut source, token, interest)?;
        drop(guard_poll);
        self.handles_io.insert(
            token,
            Handle::Py(PyHandleData {
                source,
                interest,
                cbr: None,
                cbw: Some(handle.clone_ref(py)),
            }),
        );
        self.counter_io.fetch_add(1, atomic::Ordering::Relaxed);

        Ok(handle)
    }

    fn _writer_rem(&self, fd: usize) -> Result<bool> {
        let token = Token(fd);
        self.writer_rem(token)
    }

    fn _tcp_server(
        pyself: Py<Self>,
        py: Python,
        socks: PyObject,
        rsocks: Vec<(i32, i32)>,
        protocol_factory: PyObject,
        backlog: i32,
    ) -> PyResult<Py<Server>> {
        let mut servers = Vec::new();
        for (fd, family) in rsocks {
            servers.push(TCPServer::from_fd(fd, family, backlog, protocol_factory.clone_ref(py)));
        }
        let server = Server::tcp(pyself.clone_ref(py), socks, servers);
        Py::new(py, server)
    }

    fn _sig_add(&self, py: Python, sig: u8, callback: PyObject, args: PyObject, context: PyObject) -> Result<()> {
        let handle = Py::new(py, CBHandle::new(callback, args, context))?;
        self.sig_handlers.insert(sig, handle);
        Ok(())
    }

    fn _sig_rem(&self, sig: u8) -> bool {
        self.sig_handlers.remove(&sig).is_some()
    }

    fn _sig_clear(&self) {
        self.sig_handlers.clear();
    }

    fn _run(&self, py: Python) -> PyResult<()> {
        loop {
            if self.stopping.load(atomic::Ordering::Relaxed) {
                break;
            }
            if let Err(err) = self.step(py) {
                if err.kind() == std::io::ErrorKind::Interrupted
                    && self.sig_loop_handled.swap(false, atomic::Ordering::Relaxed)
                {
                    continue;
                }
                return Err(err.into());
            }
        }

        Ok(())
    }
}

pub(crate) fn init_pymodule(module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<EventLoop>()?;

    Ok(())
}
