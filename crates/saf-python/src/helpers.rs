//! Shared helper functions for Python bindings.
//!
//! Consolidates common patterns used across multiple PyO3 binding modules
//! to reduce code duplication.

use std::collections::BTreeMap;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;

use saf_analysis::AliasResult;
use saf_analysis::cfg::Cfg;
use saf_core::air::AirModule;
use saf_core::ids::FunctionId;

/// Serialize a value to a Python dict via JSON round-trip.
///
/// Converts any `Serialize` type to `serde_json` string, then parses it
/// back into a Python dict using `json.loads`. This is the standard
/// pattern for returning structured data to Python from Rust.
pub(crate) fn serde_to_py_dict<T: Serialize>(py: Python<'_>, value: &T) -> PyResult<Py<PyDict>> {
    let json_str = serde_json::to_string(value)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    let json_mod = py.import("json")?;
    let result = json_mod.call_method1("loads", (json_str,))?;
    result.extract::<Py<PyDict>>()
}

/// Build CFGs for all non-declaration functions in a module.
///
/// Filters out external declarations and builds a `Cfg` for each
/// concrete function definition, returning them keyed by `FunctionId`.
pub(crate) fn build_cfgs(module: &AirModule) -> BTreeMap<FunctionId, Cfg> {
    module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect()
}

/// Convert an `AliasResult` to its string representation.
///
/// Returns one of `"must"`, `"partial"`, `"may"`, `"no"`, or `"unknown"`.
pub(crate) fn alias_result_to_str(result: AliasResult) -> &'static str {
    match result {
        AliasResult::Must => "must",
        AliasResult::Partial => "partial",
        AliasResult::May => "may",
        AliasResult::No => "no",
        AliasResult::Unknown => "unknown",
    }
}

/// Extract checker names from a Python argument that is either a single
/// string or a list of strings.
///
/// Used by `Project.check()` and `Project.check_path_sensitive()` to
/// accept flexible checker name arguments.
pub(crate) fn extract_checker_names(name: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
    if let Ok(s) = name.extract::<String>() {
        Ok(vec![s])
    } else if let Ok(list) = name.extract::<Vec<String>>() {
        Ok(list)
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a string or list of strings for checker name(s)",
        ))
    }
}
