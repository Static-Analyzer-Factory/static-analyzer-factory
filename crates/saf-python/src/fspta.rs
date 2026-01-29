//! Python bindings for flow-sensitive pointer analysis.
//!
//! Provides access to `FlowSensitivePtaResult` with points-to queries,
//! flow-sensitive alias queries, diagnostics, and JSON export.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use saf_analysis::PtaResult;
use saf_analysis::PtsRepresentation;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::fspta::{self, FlowSensitivePtaResult, FsPtaConfig, FsSvfgBuilder};
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::SvfgBuilder;
use saf_core::air::AirModule;

use crate::id_parse::{parse_loc_id, parse_value_id};

/// Python wrapper for flow-sensitive PTA results.
///
/// Provides per-value flow-sensitive points-to queries, alias analysis
/// at specific program points, diagnostics, and JSON export.
///
/// Obtain via `Project.flow_sensitive_pta()`.
#[pyclass(name = "FlowSensitivePtaResult")]
pub struct PyFlowSensitivePtaResult {
    inner: FlowSensitivePtaResult,
}

impl PyFlowSensitivePtaResult {
    /// Build from project internals.
    #[allow(clippy::must_use_candidate)]
    pub fn build(
        module: &AirModule,
        callgraph: &CallGraph,
        defuse: &DefUseGraph,
        pta: &PtaResult,
        mssa_pta: PtaResult,
    ) -> Self {
        Self::build_with_repr(
            module,
            callgraph,
            defuse,
            pta,
            mssa_pta,
            PtsRepresentation::Auto,
        )
    }

    /// Build from project internals with specific representation.
    #[allow(clippy::similar_names, clippy::must_use_candidate)]
    pub fn build_with_repr(
        module: &AirModule,
        callgraph: &CallGraph,
        defuse: &DefUseGraph,
        pta: &PtaResult,
        mssa_pta: PtaResult,
        repr: PtsRepresentation,
    ) -> Self {
        let cfgs = crate::helpers::build_cfgs(module);

        // Build SVFG (needs its own MSSA)
        let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, callgraph);
        let (svfg, _program_points) =
            SvfgBuilder::new(module, defuse, callgraph, pta, &mut mssa).build();

        // Build FsSvfg (needs its own MSSA + PTA)
        let mssa_pta2 = pta.clone();
        let mut mssa2 = MemorySsa::build(module, &cfgs, mssa_pta2, callgraph);
        let fs_svfg = FsSvfgBuilder::new(module, &svfg, pta, &mut mssa2, callgraph).build();

        let config = FsPtaConfig::default().with_pts_representation(repr);
        let result = fspta::solve_flow_sensitive(module, &fs_svfg, pta, callgraph, &config);

        Self { inner: result }
    }
}

#[pymethods]
impl PyFlowSensitivePtaResult {
    /// Get the flow-sensitive points-to set for a value.
    ///
    /// Args:
    ///     value_hex: Value ID as hex string (e.g., "0x00000...").
    ///
    /// Returns:
    ///     list[str]: Location IDs that the value may point to (hex).
    fn points_to(&self, py: Python<'_>, value_hex: &str) -> PyResult<Py<PyList>> {
        let vid = parse_value_id(value_hex)?;
        let pts = self.inner.points_to(vid);
        let locs: Vec<String> = pts.iter().map(|l| format!("0x{:032x}", l.raw())).collect();
        Ok(PyList::new(py, &locs)?.into())
    }

    /// Get the dataflow points-to set for a location at a specific SVFG node.
    ///
    /// Args:
    ///     loc_hex: Location ID as hex string.
    ///     node_hex: SVFG node ID as hex string.
    ///
    /// Returns:
    ///     list[str]: Location IDs in the dfIn set (hex).
    fn points_to_at(&self, py: Python<'_>, loc_hex: &str, node_hex: &str) -> PyResult<Py<PyList>> {
        let loc = parse_loc_id(loc_hex)?;
        let node_vid = parse_value_id(node_hex)?;
        let node = saf_analysis::svfg::SvfgNodeId::Value(node_vid);
        let pts = self.inner.points_to_at(loc, node);
        let locs: Vec<String> = pts.iter().map(|l| format!("0x{:032x}", l.raw())).collect();
        Ok(PyList::new(py, &locs)?.into())
    }

    /// Check if two pointers may alias at a specific SVFG node.
    ///
    /// Args:
    ///     p_hex: First pointer value ID as hex.
    ///     q_hex: Second pointer value ID as hex.
    ///     node_hex: SVFG node ID as hex.
    ///
    /// Returns:
    ///     bool: True if the pointers may alias.
    #[allow(clippy::similar_names)]
    fn may_alias_at(&self, p_hex: &str, q_hex: &str, node_hex: &str) -> PyResult<bool> {
        let p = parse_value_id(p_hex)?;
        let q = parse_value_id(q_hex)?;
        let node_vid = parse_value_id(node_hex)?;
        let node = saf_analysis::svfg::SvfgNodeId::Value(node_vid);
        let result = self.inner.may_alias_at(p, q, node);
        Ok(result.may_alias_conservative())
    }

    /// Get analysis diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys "iterations", "iteration_limit_hit",
    ///           "strong_updates", "weak_updates", "fs_svfg_nodes", etc.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let diag = self.inner.diagnostics();
        let dict = PyDict::new(py);
        dict.set_item("iterations", diag.iterations)?;
        dict.set_item("iteration_limit_hit", diag.iteration_limit_hit)?;
        dict.set_item("strong_updates", diag.strong_updates)?;
        dict.set_item("weak_updates", diag.weak_updates)?;
        dict.set_item("fs_svfg_nodes", diag.fs_svfg_nodes)?;
        dict.set_item("fs_svfg_edges", diag.fs_svfg_edges)?;
        dict.set_item("store_nodes", diag.store_nodes)?;
        dict.set_item("load_nodes", diag.load_nodes)?;
        Ok(dict.into())
    }

    /// Export the result to a dictionary.
    ///
    /// Returns:
    ///     dict: Full export with schema_version, diagnostics, points_to.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let export = self.inner.export();
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        let diag = self.inner.diagnostics();
        format!(
            "FlowSensitivePtaResult(iterations={}, strong_updates={}, weak_updates={})",
            diag.iterations, diag.strong_updates, diag.weak_updates
        )
    }
}

/// Register fspta types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFlowSensitivePtaResult>()?;
    Ok(())
}
