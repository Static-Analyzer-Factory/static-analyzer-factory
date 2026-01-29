//! Python bindings for graph access.
//!
//! Provides access to analysis graphs (CFG, CallGraph, DefUse, ValueFlow)
//! using the unified [`PropertyGraph`](saf_analysis::export::PropertyGraph)
//! export format. Supports export to dict, DOT, and interactive HTML.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::ValueFlowGraph;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::export::PropertyGraph;
use saf_core::air::AirModule;

/// Python wrapper for graph storage.
#[pyclass(name = "GraphStore")]
pub struct PyGraphStore {
    pub(crate) module: Arc<AirModule>,
    pub(crate) callgraph: Arc<CallGraph>,
    pub(crate) defuse: Arc<DefUseGraph>,
    pub(crate) valueflow: Arc<ValueFlowGraph>,
}

impl PyGraphStore {
    #[allow(clippy::must_use_candidate)]
    pub fn new(
        module: Arc<AirModule>,
        callgraph: Arc<CallGraph>,
        defuse: Arc<DefUseGraph>,
        valueflow: Arc<ValueFlowGraph>,
    ) -> Self {
        Self {
            module,
            callgraph,
            defuse,
            valueflow,
        }
    }

    /// Build a [`PropertyGraph`] for the specified graph type.
    ///
    /// This is the shared helper used by `export`, `to_dot`, and `to_html`.
    fn build_pg(&self, name: &str, function: Option<&str>) -> PyResult<PropertyGraph> {
        match name {
            "callgraph" => Ok(self.callgraph.to_pg(&self.module, None)),
            "cfg" => {
                let mut merged = PropertyGraph::new("cfg");
                for func in &self.module.functions {
                    if func.is_declaration {
                        continue;
                    }
                    if let Some(f) = function {
                        if func.name != f {
                            continue;
                        }
                    }
                    let cfg = saf_analysis::cfg::Cfg::build(func);
                    let pg = cfg.to_pg(func, &self.module.source_files, None);
                    merged.nodes.extend(pg.nodes);
                    merged.edges.extend(pg.edges);
                }
                Ok(merged)
            }
            "defuse" => Ok(self.defuse.to_pg(&self.module, None)),
            "valueflow" => Ok(saf_analysis::to_property_graph(
                &self.valueflow,
                &self.module,
                None,
            )),
            _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown graph type: {name}. Available: cfg, callgraph, defuse, valueflow"
            ))),
        }
    }
}

#[pymethods]
impl PyGraphStore {
    /// List available graph types.
    fn available(&self) -> Vec<&'static str> {
        vec!["cfg", "callgraph", "defuse", "valueflow"]
    }

    /// Export a graph to a `PropertyGraph` dictionary.
    ///
    /// Args:
    ///     name: Graph name ("cfg", "callgraph", "defuse", "valueflow")
    ///     function: For CFG, the function name to export (optional, exports all if None)
    ///
    /// Returns:
    ///     dict: Graph in unified `PropertyGraph` format with schema_version,
    ///           graph_type, nodes, edges, and optional metadata.
    #[pyo3(signature = (name, function=None))]
    fn export(&self, py: Python<'_>, name: &str, function: Option<&str>) -> PyResult<Py<PyDict>> {
        let pg = self.build_pg(name, function)?;
        crate::helpers::serde_to_py_dict(py, &pg)
    }

    /// Export a graph to Graphviz DOT format.
    ///
    /// Args:
    ///     name: Graph name ("cfg", "callgraph", "defuse", "valueflow")
    ///     function: For CFG, the function name to export (optional, exports all if None)
    ///
    /// Returns:
    ///     str: DOT representation of the graph.
    #[pyo3(signature = (name, function=None))]
    fn to_dot(&self, name: &str, function: Option<&str>) -> PyResult<String> {
        let pg = self.build_pg(name, function)?;
        Ok(pg.to_dot())
    }

    /// Export a graph to a self-contained interactive HTML page.
    ///
    /// The HTML page includes Cytoscape.js visualization with layout selection,
    /// zoom, search, edge type filters, hover tooltips, and click-to-highlight.
    ///
    /// Args:
    ///     name: Graph name ("cfg", "callgraph", "defuse", "valueflow")
    ///     function: For CFG, the function name to export (optional, exports all if None)
    ///
    /// Returns:
    ///     str: Self-contained HTML page with interactive graph visualization.
    #[pyo3(signature = (name, function=None))]
    fn to_html(&self, name: &str, function: Option<&str>) -> PyResult<String> {
        let pg = self.build_pg(name, function)?;
        Ok(pg.to_html())
    }

    fn __repr__(&self) -> String {
        "GraphStore(cfg, callgraph, defuse, valueflow)".to_string()
    }
}

/// Register graph types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraphStore>()?;
    Ok(())
}
