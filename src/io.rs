#[cfg(unix)]
use mio::unix::SourceFd;
#[cfg(unix)]
use std::os::fd::RawFd;
#[cfg(windows)]
use std::os::windows::io::RawSocket;

use mio::{Interest, Registry, Token, event::Source as MioSource, net::TcpListener};
use std::sync::atomic;

//: the general strateg is per-registration persistent I/O state.
//  Adapted from TonIO
//
//  source: https://github.com/gi0baro/tonio, `src/io/schedule.rs`
//  copyright: Giovanni Barillari

pub(crate) const TOKEN_WAKER: usize = 0;
pub(crate) const TOKEN_SIGNALS: usize = 1;

const READY_R: u8 = 0b01;
const READY_W: u8 = 0b10;

pub(crate) struct ScheduledIO {
    pub(crate) fd: i32,
    readiness: atomic::AtomicU8,
    armed_r: atomic::AtomicBool,
    armed_w: atomic::AtomicBool,
}

impl ScheduledIO {
    pub(crate) fn new(fd: i32, armed_r: bool, armed_w: bool) -> Self {
        Self {
            fd,
            readiness: atomic::AtomicU8::new(0),
            armed_r: atomic::AtomicBool::new(armed_r),
            armed_w: atomic::AtomicBool::new(armed_w),
        }
    }

    #[inline(always)]
    pub(crate) fn armed_r(&self) -> bool {
        self.armed_r.load(atomic::Ordering::Relaxed)
    }

    #[inline(always)]
    pub(crate) fn armed_w(&self) -> bool {
        self.armed_w.load(atomic::Ordering::Relaxed)
    }

    #[inline(always)]
    pub(crate) fn arm_r(&self) {
        self.armed_r.store(true, atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn disarm_r(&self) {
        self.armed_r.store(false, atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn arm_w(&self) {
        self.armed_w.store(true, atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn disarm_w(&self) {
        self.armed_w.store(false, atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn set_ready_r(&self) {
        self.readiness.fetch_or(READY_R, atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn set_ready_w(&self) {
        self.readiness.fetch_or(READY_W, atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub(crate) fn consume_ready_r(&self) -> bool {
        self.readiness.fetch_and(!READY_R, atomic::Ordering::Relaxed) & READY_R != 0
    }

    #[inline(always)]
    pub(crate) fn clear_ready_w(&self) {
        self.readiness.fetch_and(!READY_W, atomic::Ordering::Relaxed);
    }
}

pub(crate) enum Source {
    #[cfg(unix)]
    FD(RawFd),
    #[cfg(windows)]
    FD(RawSocket),
    TCPListener(TcpListener),
    // #[cfg(unix)]
    // TCPStream(RawFd),
    // #[cfg(windows)]
    // TCPStream(RawSocket),
    // #[cfg(unix)]
    // UDPSocket(RawFd),
    // #[cfg(windows)]
    // UDPSocket(RawSocket),
}

#[cfg(windows)]
#[derive(Debug)]
pub struct SourceRawSocket<'a>(pub &'a RawSocket);

// NOTE: this won't work as `selector()` is not exposed on win
#[cfg(windows)]
impl<'a> MioSource for SourceRawSocket<'a> {
    fn register(&mut self, registry: &Registry, token: Token, interests: Interest) -> std::io::Result<()> {
        registry.selector().register(*self.0, token, interests)
    }

    fn reregister(&mut self, registry: &Registry, token: Token, interests: Interest) -> std::io::Result<()> {
        registry.selector().reregister(*self.0, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        registry.selector().deregister(*self.0)
    }
}

impl MioSource for Source {
    #[inline]
    fn register(&mut self, registry: &Registry, token: Token, interests: Interest) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::FD(inner) => SourceFd(inner).register(registry, token, interests),
            #[cfg(windows)]
            Self::FD(inner) => SourceRawSocket(inner).register(registry, token, interests),
            Self::TCPListener(inner) => inner.register(registry, token, interests),
        }
    }

    #[inline]
    fn reregister(&mut self, registry: &Registry, token: Token, interests: Interest) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::FD(inner) => SourceFd(inner).reregister(registry, token, interests),
            #[cfg(windows)]
            Self::FD(inner) => SourceRawSocket(inner).register(registry, token, interests),
            Self::TCPListener(inner) => inner.reregister(registry, token, interests),
        }
    }

    #[inline]
    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::FD(inner) => SourceFd(inner).deregister(registry),
            #[cfg(windows)]
            Self::FD(inner) => SourceRawSocket(inner).register(registry, token, interests),
            Self::TCPListener(inner) => inner.deregister(registry),
        }
    }
}
