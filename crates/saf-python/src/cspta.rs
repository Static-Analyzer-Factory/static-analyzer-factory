//! Python bindings for context-sensitive pointer analysis.
//!
//! Provides access to `CsPtaResult` with context-qualified points-to queries,
//! alias analysis, diagnostics, and JSON export.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use saf_analysis::PtsRepresentation;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::cspta::{CsPtaConfig, CsPtaResult, solve_context_sensitive};
use saf_core::air::AirModule;

use crate::id_parse::parse_value_id;

/// Python wrapper for context-sensitive PTA results.
///
/// Provides context-qualified and context-insensitive points-to queries,
/// alias analysis, diagnostics, and JSON export.
///
/// Obtain via `Project.context_sensitive_pta()`.
#[pyclass(name = "CsPtaResult")]
pub struct PyCsPtaResult {
    inner: CsPtaResult,
    config: CsPtaConfig,
}

impl PyCsPtaResult {
    /// Build from project internals.
    #[allow(clippy::must_use_candidate)]
    pub fn build(module: &AirModule, callgraph: &CallGraph, k: u32) -> Self {
        Self::build_with_repr(module, callgraph, k, PtsRepresentation::Auto)
    }

    /// Build from project internals with specific representation.
    #[allow(clippy::must_use_candidate)]
    pub fn build_with_repr(
        module: &AirModule,
        callgraph: &CallGraph,
        k: u32,
        repr: PtsRepresentation,
    ) -> Self {
        let config = CsPtaConfig {
            k,
            ..CsPtaConfig::default()
        }
        .with_pts_representation(repr);
        let result = solve_context_sensitive(module, callgraph, &config);
        Self {
            inner: result,
            config,
        }
    }
}

#[pymethods]
impl PyCsPtaResult {
    /// Get the context-insensitive points-to set for a value (union across all contexts).
    ///
    /// Args:
    ///     value_hex: Value ID as hex string (e.g., "0x00000...").
    ///
    /// Returns:
    ///     list[str]: Location IDs that the value may point to (hex).
    fn points_to(&self, py: Python<'_>, value_hex: &str) -> PyResult<Py<PyList>> {
        let vid = parse_value_id(value_hex)?;
        let pts = self.inner.points_to_any(vid);
        let locs: Vec<String> = pts.iter().map(|l| format!("0x{:032x}", l.raw())).collect();
        Ok(PyList::new(py, &locs)?.into())
    }

    /// Get the context-specific points-to set for a value with a given context.
    ///
    /// Args:
    ///     value_hex: Value ID as hex string.
    ///     context: List of call-site instruction IDs (hex strings), or empty for CI.
    ///
    /// Returns:
    ///     list[str]: Location IDs in the points-to set (hex).
    fn points_to_in_context(
        &self,
        py: Python<'_>,
        value_hex: &str,
        context: Vec<String>,
    ) -> PyResult<Py<PyList>> {
        let vid = parse_value_id(value_hex)?;
        let ctx = parse_context(&context, self.config.k)?;
        let pts = self.inner.points_to(vid, &ctx);
        let locs: Vec<String> = pts.iter().map(|l| format!("0x{:032x}", l.raw())).collect();
        Ok(PyList::new(py, &locs)?.into())
    }

    /// Check if two pointers may alias (context-insensitive, union across contexts).
    ///
    /// Args:
    ///     p_hex: First pointer value ID as hex.
    ///     q_hex: Second pointer value ID as hex.
    ///
    /// Returns:
    ///     str: "may", "no", or "unknown".
    #[allow(clippy::similar_names)]
    fn may_alias(&self, p_hex: &str, q_hex: &str) -> PyResult<String> {
        let p = parse_value_id(p_hex)?;
        let q = parse_value_id(q_hex)?;
        let result = self.inner.may_alias_any(p, q);
        Ok(crate::helpers::alias_result_to_str(result).to_string())
    }

    /// Check if two pointers may alias in specific contexts.
    ///
    /// Args:
    ///     p_hex: First pointer value ID as hex.
    ///     p_context: Context for p (list of call-site hex strings).
    ///     q_hex: Second pointer value ID as hex.
    ///     q_context: Context for q (list of call-site hex strings).
    ///
    /// Returns:
    ///     str: "may", "no", or "unknown".
    #[allow(clippy::similar_names)]
    fn may_alias_in_context(
        &self,
        p_hex: &str,
        p_context: Vec<String>,
        q_hex: &str,
        q_context: Vec<String>,
    ) -> PyResult<String> {
        let p = parse_value_id(p_hex)?;
        let q = parse_value_id(q_hex)?;
        let p_ctx = parse_context(&p_context, self.config.k)?;
        let q_ctx = parse_context(&q_context, self.config.k)?;
        let result = self.inner.may_alias(p, &p_ctx, q, &q_ctx);
        Ok(crate::helpers::alias_result_to_str(result).to_string())
    }

    /// List all contexts seen for a value.
    ///
    /// Args:
    ///     value_hex: Value ID as hex string.
    ///
    /// Returns:
    ///     list[list[str]]: Each inner list is a context (call-site hex strings).
    fn contexts_for(&self, py: Python<'_>, value_hex: &str) -> PyResult<Py<PyList>> {
        let vid = parse_value_id(value_hex)?;
        let contexts = self.inner.contexts_for(vid);
        let outer = PyList::empty(py);
        for ctx in &contexts {
            let hex_vec = ctx.to_hex_vec();
            let inner = PyList::new(py, &hex_vec)?;
            outer.append(inner)?;
        }
        Ok(outer.into())
    }

    /// Get analysis diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys "iterations", "iteration_limit_hit",
    ///           "context_count", "max_pts_size", "scc_function_count",
    ///           "constraint_count", "location_count", "heap_clone_count".
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let diag = self.inner.diagnostics();
        let dict = PyDict::new(py);
        dict.set_item("iterations", diag.iterations)?;
        dict.set_item("iteration_limit_hit", diag.iteration_limit_hit)?;
        dict.set_item("context_count", diag.context_count)?;
        dict.set_item("max_pts_size", diag.max_pts_size)?;
        dict.set_item("scc_function_count", diag.scc_function_count)?;
        dict.set_item("constraint_count", diag.constraint_count)?;
        dict.set_item("location_count", diag.location_count)?;
        dict.set_item("heap_clone_count", diag.heap_clone_count)?;
        Ok(dict.into())
    }

    /// Export the result to a dictionary.
    ///
    /// Returns:
    ///     dict: Full export with schema_version, config, diagnostics,
    ///           contexts, and ci_summary.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let export = self.inner.export(&self.config);
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        let diag = self.inner.diagnostics();
        format!(
            "CsPtaResult(k={}, contexts={}, iterations={})",
            self.config.k, diag.context_count, diag.iterations
        )
    }
}

fn parse_context(hex_strings: &[String], k: u32) -> PyResult<saf_analysis::cspta::CallSiteContext> {
    use saf_core::ids::InstId;

    let mut ctx = saf_analysis::cspta::CallSiteContext::empty();
    for hex_str in hex_strings {
        let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let raw = u128::from_str_radix(s, 16).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid inst ID: {hex_str}"))
        })?;
        ctx = ctx.push(InstId::new(raw), k);
    }
    Ok(ctx)
}

/// Register cspta types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCsPtaResult>()?;
    Ok(())
}
