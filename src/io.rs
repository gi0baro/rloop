#[cfg(unix)]
use mio::unix::SourceFd;
#[cfg(unix)]
use std::os::fd::RawFd;
#[cfg(windows)]
use std::os::windows::io::RawSocket;

use mio::{Interest, Registry, Token, event::Source as MioSource, net::TcpListener};

pub(crate) enum Source {
    TCPListener(TcpListener),
    #[cfg(unix)]
    TCPStream(RawFd),
    #[cfg(windows)]
    TCPStream(RawSocket),
    #[cfg(unix)]
    FD(RawFd),
    #[cfg(windows)]
    FD(RawSocket),
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
            Self::TCPListener(inner) => inner.register(registry, token, interests),
            Self::TCPStream(inner) => SourceFd(inner).register(registry, token, interests),
            #[cfg(unix)]
            Self::FD(inner) => SourceFd(inner).register(registry, token, interests),
            #[cfg(windows)]
            Self::FD(inner) => SourceRawSocket(inner).register(registry, token, interests),
        }
    }

    #[inline]
    fn reregister(&mut self, registry: &Registry, token: Token, interests: Interest) -> std::io::Result<()> {
        match self {
            Self::TCPListener(inner) => inner.reregister(registry, token, interests),
            Self::TCPStream(inner) => SourceFd(inner).reregister(registry, token, interests),
            #[cfg(unix)]
            Self::FD(inner) => SourceFd(inner).reregister(registry, token, interests),
            #[cfg(windows)]
            Self::FD(inner) => SourceRawSocket(inner).register(registry, token, interests),
        }
    }

    #[inline]
    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        match self {
            Self::TCPListener(inner) => inner.deregister(registry),
            Self::TCPStream(inner) => SourceFd(inner).deregister(registry),
            #[cfg(unix)]
            Self::FD(inner) => SourceFd(inner).deregister(registry),
            #[cfg(windows)]
            Self::FD(inner) => SourceRawSocket(inner).register(registry, token, interests),
        }
    }
}
