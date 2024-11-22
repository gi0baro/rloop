#[cfg(not(any(target_os = "freebsd", target_os = "windows")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(any(target_os = "freebsd", target_os = "windows"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use pyo3::prelude::*;
use std::sync::OnceLock;

mod event_loop;
mod handles;
mod io;
mod py;

pub fn get_lib_version() -> &'static str {
    static LIB_VERSION: OnceLock<String> = OnceLock::new();

    LIB_VERSION.get_or_init(|| {
        let version = env!("CARGO_PKG_VERSION");
        version.replace("-alpha", "a").replace("-beta", "b")
    })
}

#[pymodule(gil_used = false)]
fn _rloop(_py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add("__version__", get_lib_version())?;

    event_loop::init_pymodule(module)?;
    handles::init_pymodule(module)?;
    Ok(())
}
