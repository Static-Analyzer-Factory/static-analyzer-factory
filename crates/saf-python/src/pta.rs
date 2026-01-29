//! Python bindings for PTA results.
//!
//! Provides access to points-to analysis results.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::{PathSensitiveConfig, PtaConfig, PtaResult};

use crate::id_parse::parse_value_id;

/// Python wrapper for PTA result.
#[pyclass(name = "PtaResult")]
pub struct PyPtaResult {
    pub(crate) inner: Arc<PtaResult>,
}

impl PyPtaResult {
    #[allow(clippy::must_use_candidate)]
    pub fn new(inner: Arc<PtaResult>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyPtaResult {
    /// Get the number of values with points-to sets.
    #[getter]
    fn value_count(&self) -> usize {
        self.inner.value_count()
    }

    /// Get the number of abstract locations.
    #[getter]
    fn location_count(&self) -> usize {
        self.inner.location_count()
    }

    /// Get the points-to set for a value.
    ///
    /// Args:
    ///     ptr: Value ID as hex string (e.g., "0x00000001").
    ///
    /// Returns:
    ///     List of location IDs as hex strings.
    #[allow(clippy::similar_names)]
    fn points_to(&self, ptr: &str) -> PyResult<Vec<String>> {
        let value_id = parse_value_id(ptr)?;
        let pts = self.inner.points_to(value_id);
        Ok(pts
            .iter()
            .map(|loc| format!("0x{:032x}", loc.raw()))
            .collect())
    }

    /// Check if two pointers may alias.
    ///
    /// Args:
    ///     p: First pointer value ID.
    ///     q: Second pointer value ID.
    ///
    /// Returns:
    ///     True if the pointers may alias, False otherwise.
    #[allow(clippy::similar_names)]
    fn may_alias(&self, p: &str, q: &str) -> PyResult<bool> {
        let p_id = parse_value_id(p)?;
        let q_id = parse_value_id(q)?;
        Ok(self.inner.may_alias(p_id, q_id).may_alias_conservative())
    }

    /// Check if two pointers definitely don't alias.
    ///
    /// Args:
    ///     p: First pointer value ID.
    ///     q: Second pointer value ID.
    ///
    /// Returns:
    ///     True if the pointers definitely don't alias, False otherwise.
    #[allow(clippy::similar_names)]
    fn no_alias(&self, p: &str, q: &str) -> PyResult<bool> {
        let p_id = parse_value_id(p)?;
        let q_id = parse_value_id(q)?;
        Ok(self.inner.may_alias(p_id, q_id).no_alias())
    }

    /// Get the five-valued alias result.
    ///
    /// Args:
    ///     p: First pointer value ID.
    ///     q: Second pointer value ID.
    ///
    /// Returns:
    ///     One of "must", "partial", "may", "no", or "unknown".
    #[allow(clippy::similar_names)]
    fn alias_result(&self, p: &str, q: &str) -> PyResult<String> {
        let p_id = parse_value_id(p)?;
        let q_id = parse_value_id(q)?;
        let result = self.inner.may_alias(p_id, q_id);
        Ok(crate::helpers::alias_result_to_str(result).to_owned())
    }

    /// Export the PTA result to a dictionary.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let config = PtaConfig::default();
        let export = self.inner.export(&config);
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        format!(
            "PtaResult(values={}, locations={})",
            self.inner.value_count(),
            self.inner.location_count()
        )
    }
}

/// Python wrapper for path-sensitive alias configuration.
#[pyclass(name = "PathSensitiveConfig")]
#[derive(Clone)]
pub struct PyPathSensitiveConfig {
    pub(crate) inner: PathSensitiveConfig,
}

#[pymethods]
impl PyPathSensitiveConfig {
    /// Create a new path-sensitive configuration.
    ///
    /// Args:
    ///     enabled: Whether path-sensitive analysis is enabled (default: True).
    ///     max_paths: Maximum paths to enumerate per query (default: 16).
    ///     timeout_ms: Timeout in milliseconds for Z3 checks (default: 500).
    #[new]
    #[pyo3(signature = (enabled=true, max_paths=16, timeout_ms=500))]
    fn new(enabled: bool, max_paths: usize, timeout_ms: u64) -> Self {
        Self {
            inner: PathSensitiveConfig {
                enabled,
                max_paths,
                timeout_ms,
            },
        }
    }

    #[getter]
    fn enabled(&self) -> bool {
        self.inner.enabled
    }

    #[getter]
    fn max_paths(&self) -> usize {
        self.inner.max_paths
    }

    #[getter]
    fn timeout_ms(&self) -> u64 {
        self.inner.timeout_ms
    }

    fn __repr__(&self) -> String {
        format!(
            "PathSensitiveConfig(enabled={}, max_paths={}, timeout_ms={})",
            self.inner.enabled, self.inner.max_paths, self.inner.timeout_ms
        )
    }
}

/// Register PTA types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPtaResult>()?;
    m.add_class::<PyPathSensitiveConfig>()?;
    Ok(())
}
