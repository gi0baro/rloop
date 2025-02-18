#[cfg(not(Py_GIL_DISABLED))]
macro_rules! py_allow_threads {
    ($py:expr, $func:tt) => {
        $py.allow_threads(|| $func)
    };
}

#[cfg(Py_GIL_DISABLED)]
macro_rules! py_allow_threads {
    ($py:expr, $func:tt) => {
        $func
    };
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

pub(super) use py_allow_threads;
pub(crate) use syscall;
