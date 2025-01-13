use pyo3::{prelude::*, types::PyDict};

pub(crate) enum LogExc {
    CBHandle(LogExcCBHandleData),
    Transport(LogExcTransportData),
}

impl LogExc {
    pub(crate) fn cb_handle(exc: PyErr, msg: String, handle: PyObject) -> Self {
        Self::CBHandle(LogExcCBHandleData {
            base: LogExcBaseData { exc, msg },
            handle,
        })
    }

    pub(crate) fn transport(exc: PyErr, msg: String, protocol: PyObject, transport: PyObject) -> Self {
        Self::Transport(LogExcTransportData {
            base: LogExcBaseData { exc, msg },
            protocol,
            transport,
        })
    }
}

struct LogExcBaseData {
    exc: PyErr,
    msg: String,
}

pub(crate) struct LogExcCBHandleData {
    base: LogExcBaseData,
    handle: PyObject,
}

pub(crate) struct LogExcTransportData {
    base: LogExcBaseData,
    protocol: PyObject,
    transport: PyObject,
}

macro_rules! log_exc_base_data_to_dict {
    ($py:expr, $dict:expr, $data:expr) => {
        let _ = $dict.set_item(pyo3::intern!($py, "exception"), $data.exc);
        let _ = $dict.set_item(pyo3::intern!($py, "message"), $data.msg);
    };
}

pub(crate) fn log_exc_to_py_ctx(py: Python, exc: LogExc) -> Py<PyDict> {
    let dict = PyDict::new(py);
    match exc {
        LogExc::CBHandle(data) => {
            log_exc_base_data_to_dict!(py, dict, data.base);
            let _ = dict.set_item(pyo3::intern!(py, "handle"), data.handle);
        }
        LogExc::Transport(data) => {
            log_exc_base_data_to_dict!(py, dict, data.base);
            let _ = dict.set_item(pyo3::intern!(py, "protocol"), data.protocol);
            let _ = dict.set_item(pyo3::intern!(py, "transport"), data.transport);
        }
    }

    dict.unbind()
}
