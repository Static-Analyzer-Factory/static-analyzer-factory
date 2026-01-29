//! Python bindings for the Sparse Value-Flow Graph (SVFG).
//!
//! Provides access to the unified value-flow graph that combines direct
//! (register/SSA) and indirect (memory) value-flow edges.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use saf_analysis::PtaResult;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::{Svfg, SvfgBuilder};
use saf_core::air::AirModule;

use crate::id_parse::parse_value_id;

/// Python wrapper for the Sparse Value-Flow Graph.
///
/// Unifies direct (register) and indirect (memory) value-flow into one graph.
/// Provides reachability queries and JSON export.
///
/// Obtain via `Project.svfg()`.
#[pyclass(name = "Svfg")]
#[allow(clippy::module_name_repetitions)]
pub struct PySvfg {
    inner: Svfg,
}

impl PySvfg {
    /// Build SVFG from project internals.
    #[allow(clippy::must_use_candidate)]
    pub fn build(
        module: &AirModule,
        callgraph: &CallGraph,
        defuse: &DefUseGraph,
        pta: &PtaResult,
        mssa_pta: PtaResult,
    ) -> Self {
        let cfgs = crate::helpers::build_cfgs(module);

        let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, callgraph);

        let (svfg, _program_points) =
            SvfgBuilder::new(module, defuse, callgraph, pta, &mut mssa).build();
        Self { inner: svfg }
    }
}

#[pymethods]
impl PySvfg {
    /// Get the total number of nodes.
    #[getter]
    fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Get the total number of edges.
    #[getter]
    fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Get construction diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys "direct_edge_count", "indirect_edge_count",
    ///           "mem_phi_count", "skipped_call_clobbers", "skipped_live_on_entry".
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let diag = self.inner.diagnostics();
        let dict = PyDict::new(py);
        dict.set_item("direct_edge_count", diag.direct_edge_count)?;
        dict.set_item("indirect_edge_count", diag.indirect_edge_count)?;
        dict.set_item("mem_phi_count", diag.mem_phi_count)?;
        dict.set_item("skipped_call_clobbers", diag.skipped_call_clobbers)?;
        dict.set_item("skipped_live_on_entry", diag.skipped_live_on_entry)?;
        Ok(dict.into())
    }

    /// Check if a value can reach another value via forward BFS.
    ///
    /// Args:
    ///     from_id: Source value ID as hex string.
    ///     to_id: Target value ID as hex string.
    ///
    /// Returns:
    ///     True if `to_id` is reachable from `from_id`.
    fn reachable(&self, from_id: &str, to_id: &str) -> PyResult<bool> {
        let from = parse_value_id(from_id)?;
        let to = parse_value_id(to_id)?;
        Ok(self.inner.reachable(from, to))
    }

    /// Find all value nodes reachable forward from a value.
    ///
    /// Args:
    ///     from_id: Source value ID as hex string.
    ///
    /// Returns:
    ///     list[str]: Hex IDs of all reachable value nodes (excluding MemPhi nodes).
    fn forward_reachable(&self, py: Python<'_>, from_id: &str) -> PyResult<Py<PyList>> {
        let from = parse_value_id(from_id)?;
        let reachable = self
            .inner
            .forward_reachable(saf_analysis::svfg::SvfgNodeId::Value(from));
        let values: Vec<String> = reachable
            .iter()
            .filter_map(saf_analysis::svfg::SvfgNodeId::as_value)
            .map(|v| format!("0x{:032x}", v.raw()))
            .collect();
        Ok(PyList::new(py, &values)?.into())
    }

    /// Find a value-flow path between two values.
    ///
    /// Args:
    ///     from_id: Source value ID as hex string.
    ///     to_id: Target value ID as hex string.
    ///     max_depth: Maximum BFS depth (default: 1000).
    ///
    /// Returns:
    ///     list[str] | None: Hex IDs of nodes on the path, or None if unreachable.
    #[pyo3(signature = (from_id, to_id, max_depth=1000))]
    fn value_flow_path(
        &self,
        py: Python<'_>,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> PyResult<Option<Py<PyList>>> {
        let from = parse_value_id(from_id)?;
        let to = parse_value_id(to_id)?;
        match self.inner.value_flow_path(from, to, max_depth) {
            Some(path) => {
                let ids: Vec<String> = path
                    .iter()
                    .map(saf_analysis::svfg::SvfgNodeId::to_hex)
                    .collect();
                Ok(Some(PyList::new(py, &ids)?.into()))
            }
            None => Ok(None),
        }
    }

    /// Export the SVFG to a dictionary.
    ///
    /// Returns:
    ///     dict: Full SVFG export with schema_version, nodes, edges, diagnostics.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let export = self.inner.export();
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        format!(
            "Svfg(nodes={}, edges={})",
            self.inner.node_count(),
            self.inner.edge_count()
        )
    }
}

/// Register SVFG types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySvfg>()?;
    Ok(())
}
