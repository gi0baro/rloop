use pyo3::prelude::*;
use std::sync::OnceLock;

pub mod event_loop;
pub mod handles;
mod io;
mod log;
mod py;
mod server;
mod sock;
mod tcp;
mod time;
mod utils;

pub(crate) fn get_lib_version() -> &'static str {
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
    server::init_pymodule(module)?;

    Ok(())
}
