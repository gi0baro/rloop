use pyo3::{prelude::*, types::PyBytes};

enum WriteChunkData {
    Py { _obj: Py<PyBytes> },
    Owned { _data: Box<[u8]> },
}

pub(crate) struct WriteChunk {
    _data: WriteChunkData,
    ptr: *const u8,
    len: usize,
    pub off: usize,
}

impl WriteChunk {
    #[inline]
    pub(crate) fn py(obj: &Bound<PyBytes>, off: usize) -> Self {
        let bytes = obj.as_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            off,
            _data: WriteChunkData::Py {
                _obj: obj.clone().unbind(),
            },
        }
    }

    #[inline]
    pub(crate) fn owned(data: Box<[u8]>) -> Self {
        Self {
            ptr: data.as_ptr(),
            len: data.len(),
            off: 0,
            _data: WriteChunkData::Owned { _data: data },
        }
    }

    #[inline]
    pub(crate) fn remaining(&self) -> usize {
        self.len - self.off
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.add(self.off), self.len - self.off) }
    }

    #[inline]
    pub(crate) fn as_iovec(&self) -> libc::iovec {
        libc::iovec {
            iov_base: unsafe { self.ptr.add(self.off) }.cast_mut().cast(),
            iov_len: self.len - self.off,
        }
    }
}

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub(crate) use syscall;
