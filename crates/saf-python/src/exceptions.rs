//! Python exception types for SAF errors.
//!
//! Provides a hierarchy of exceptions that map to SAF's error types with
//! structured `.code` and `.details` attributes for agent-friendly consumption.

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

// Base exception for all SAF errors
create_exception!(
    saf,
    SafError,
    PyException,
    "Base exception for all SAF errors. Has .code and .details attributes."
);

// Exception subclasses
create_exception!(
    saf,
    FrontendError,
    SafError,
    "Error from frontend ingestion (parsing, I/O, unsupported features)."
);
create_exception!(
    saf,
    AnalysisError,
    SafError,
    "Error during analysis (PTA timeout, ValueFlow build error)."
);
create_exception!(
    saf,
    QueryError,
    SafError,
    "Error during query execution (invalid selector, no match)."
);
create_exception!(
    saf,
    ConfigError,
    SafError,
    "Error in configuration (invalid field, incompatible options)."
);

/// Error codes for structured error reporting.
#[allow(dead_code)]
pub mod codes {
    // Frontend errors
    pub const FRONTEND_NOT_FOUND: &str = "FRONTEND_NOT_FOUND";
    pub const FRONTEND_PARSE_ERROR: &str = "FRONTEND_PARSE_ERROR";
    pub const FRONTEND_IO_ERROR: &str = "FRONTEND_IO_ERROR";

    // Analysis errors
    pub const PTA_TIMEOUT: &str = "PTA_TIMEOUT";
    pub const VALUEFLOW_BUILD_ERROR: &str = "VALUEFLOW_BUILD_ERROR";

    // Query errors
    pub const INVALID_SELECTOR: &str = "INVALID_SELECTOR";
    pub const SELECTOR_NO_MATCH: &str = "SELECTOR_NO_MATCH";

    // Config errors
    pub const CONFIG_INVALID_FIELD: &str = "CONFIG_INVALID_FIELD";
}

/// Create a `FrontendError` with code and details.
#[allow(deprecated)] // to_object → IntoPyObject migration tracked separately
pub fn frontend_error(py: Python<'_>, code: &str, message: &str, path: Option<&str>) -> PyErr {
    let details = path.map(|p| {
        let dict = pyo3::types::PyDict::new(py);
        let _ = dict.set_item("path", p);
        dict
    });

    let exc = FrontendError::new_err(message.to_string());
    let _ = exc.value(py).setattr("code", code);
    let _ = exc.value(py).setattr(
        "details",
        details.map_or_else(|| py.None(), |d| d.to_object(py)),
    );
    exc
}

/// Create a QueryError with code and details.
pub fn query_error(py: Python<'_>, code: &str, message: &str) -> PyErr {
    let exc = QueryError::new_err(message.to_string());
    let _ = exc.value(py).setattr("code", code);
    let _ = exc.value(py).setattr("details", py.None());
    exc
}

/// Register exception types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("SafError", m.py().get_type::<SafError>())?;
    m.add("FrontendError", m.py().get_type::<FrontendError>())?;
    m.add("AnalysisError", m.py().get_type::<AnalysisError>())?;
    m.add("QueryError", m.py().get_type::<QueryError>())?;
    m.add("ConfigError", m.py().get_type::<ConfigError>())?;
    Ok(())
}
