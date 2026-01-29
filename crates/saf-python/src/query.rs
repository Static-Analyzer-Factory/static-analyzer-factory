//! Python bindings for Query class.
//!
//! Provides the Query class for performing taint flow and other analysis queries.

use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::display::DisplayResolver;
use saf_analysis::selector::{Selector as RustSelector, resolve_selectors};
use saf_analysis::{
    Finding, PtaResult, QueryLimits, ValueFlowConfig, ValueFlowGraph, build_valueflow,
};
use saf_core::air::AirModule;

use crate::exceptions::{codes, query_error};
use crate::finding::PyFinding;
use crate::id_parse::parse_value_id;
use crate::selector::{PySelectorSet, SelectorOrSet};

/// Query interface for running analysis queries.
///
/// Created from a `Project` via `project.query()`.
#[pyclass(name = "Query")]
#[allow(clippy::module_name_repetitions)]
pub struct PyQuery {
    pub(crate) module: Arc<AirModule>,
    pub(crate) valueflow: Arc<ValueFlowGraph>,
    pub(crate) pta_result: Arc<PtaResult>,
    /// Call graph (needed for on-demand precise VF rebuild).
    pub(crate) callgraph: Arc<CallGraph>,
    /// Def-use graph (needed for on-demand precise VF rebuild).
    pub(crate) defuse: Arc<DefUseGraph>,
    /// Lazily built precise-mode VF graph for sanitizer queries.
    pub(crate) precise_vf: OnceLock<Arc<ValueFlowGraph>>,
}

impl PyQuery {
    /// Create a new query interface.
    #[allow(clippy::must_use_candidate)]
    pub fn new(
        module: Arc<AirModule>,
        valueflow: Arc<ValueFlowGraph>,
        pta_result: Arc<PtaResult>,
        callgraph: Arc<CallGraph>,
        defuse: Arc<DefUseGraph>,
    ) -> Self {
        Self {
            module,
            valueflow,
            pta_result,
            callgraph,
            defuse,
            precise_vf: OnceLock::new(),
        }
    }
}

#[pymethods]
impl PyQuery {
    /// Find taint flows from sources to sinks.
    ///
    /// Args:
    ///     sources: A Selector or SelectorSet identifying source values.
    ///     sinks: A Selector or SelectorSet identifying sink values.
    ///     sanitizers: Optional Selector or SelectorSet identifying sanitizer values.
    ///     limit: Maximum number of findings to return (default 1000).
    ///
    /// Returns:
    ///     List of Finding objects representing taint flows from sources to sinks.
    ///
    /// Raises:
    ///     QueryError: If selectors are invalid or cannot be resolved.
    #[pyo3(signature = (sources, sinks, sanitizers=None, *, limit=1000))]
    fn taint_flow(
        &self,
        py: Python<'_>,
        sources: SelectorOrSet,
        sinks: SelectorOrSet,
        sanitizers: Option<SelectorOrSet>,
        limit: usize,
    ) -> PyResult<Vec<PyFinding>> {
        // Convert to selector sets
        let sources_set = sources.into_selector_set();
        let sinks_set = sinks.into_selector_set();
        let sanitizers_set = sanitizers
            .map(super::selector::SelectorOrSet::into_selector_set)
            .unwrap_or_default();

        // Resolve selectors to value IDs
        let source_values = self.resolve_selectors(py, &sources_set)?;
        let sink_values = self.resolve_selectors(py, &sinks_set)?;
        let sanitizer_values = self.resolve_selectors(py, &sanitizers_set)?;

        // Choose VF graph: precise when sanitizers are present, fast otherwise.
        // Fast mode routes all memory through a single `unknown_mem` node, which
        // can create spurious shortcuts that bypass sanitizer nodes entirely.
        let vf: &ValueFlowGraph = if sanitizer_values.is_empty() {
            &self.valueflow
        } else {
            self.precise_vf.get_or_init(|| {
                let config = ValueFlowConfig::precise();
                let graph = build_valueflow(
                    &config,
                    &self.module,
                    &self.defuse,
                    &self.callgraph,
                    Some(&self.pta_result),
                );
                Arc::new(graph)
            })
        };

        // Run the taint flow query
        let limits = QueryLimits::new(100, limit);
        let flows = vf.taint_flow(&source_values, &sink_values, &sanitizer_values, &limits);

        // Convert to findings with display resolution
        let resolver = DisplayResolver::with_analysis(&self.module, Some(&self.pta_result), None);
        let findings: Vec<PyFinding> = flows
            .into_iter()
            .map(|flow| {
                let finding = Finding::from_flow(flow, None);
                PyFinding::from_finding_with_resolver(&finding, &self.module, &resolver)
            })
            .collect();

        Ok(findings)
    }

    /// Find data flows from sources to sinks (without sanitizer filtering).
    ///
    /// Args:
    ///     sources: A Selector or SelectorSet identifying source values.
    ///     sinks: A Selector or SelectorSet identifying sink values.
    ///     limit: Maximum number of findings to return (default 1000).
    ///
    /// Returns:
    ///     List of Finding objects representing flows from sources to sinks.
    #[pyo3(signature = (sources, sinks, *, limit=1000))]
    fn flows(
        &self,
        py: Python<'_>,
        sources: SelectorOrSet,
        sinks: SelectorOrSet,
        limit: usize,
    ) -> PyResult<Vec<PyFinding>> {
        // Convert to selector sets
        let sources_set = sources.into_selector_set();
        let sinks_set = sinks.into_selector_set();

        // Resolve selectors to value IDs
        let source_values = self.resolve_selectors(py, &sources_set)?;
        let sink_values = self.resolve_selectors(py, &sinks_set)?;

        // Run the flows query
        let limits = QueryLimits::new(100, limit);
        let flows = self.valueflow.flows(&source_values, &sink_values, &limits);

        // Convert to findings with display resolution
        let resolver = DisplayResolver::with_analysis(&self.module, Some(&self.pta_result), None);
        let findings: Vec<PyFinding> = flows
            .into_iter()
            .map(|flow| {
                let finding = Finding::from_flow(flow, None);
                PyFinding::from_finding_with_resolver(&finding, &self.module, &resolver)
            })
            .collect();

        Ok(findings)
    }

    /// Get the points-to set for a value.
    ///
    /// Args:
    ///     ptr: Value ID as hex string (e.g., "0x00000001").
    ///
    /// Returns:
    ///     List of location IDs as hex strings that the pointer may point to.
    #[allow(clippy::similar_names)]
    fn points_to(&self, ptr: &str) -> PyResult<Vec<String>> {
        let value_id = parse_value_id(ptr)?;
        let pts = self.pta_result.points_to(value_id);
        Ok(pts
            .iter()
            .map(|loc| format!("0x{:032x}", loc.raw()))
            .collect())
    }

    /// Check if two pointers may alias.
    ///
    /// Args:
    ///     p: First pointer value ID as hex string.
    ///     q: Second pointer value ID as hex string.
    ///
    /// Returns:
    ///     True if the pointers may alias, False otherwise.
    #[allow(clippy::similar_names)]
    fn may_alias(&self, p: &str, q: &str) -> PyResult<bool> {
        let p_id = parse_value_id(p)?;
        let q_id = parse_value_id(q)?;
        Ok(self
            .pta_result
            .may_alias(p_id, q_id)
            .may_alias_conservative())
    }

    /// Export a graph to a dictionary.
    ///
    /// Args:
    ///     name: Graph name ("valueflow" or "pta").
    ///
    /// Returns:
    ///     dict: Graph in JSON-compatible format.
    fn export_graph(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        match name {
            "valueflow" => {
                // Build export manually from the graph
                dict.set_item("type", "valueflow")?;
                dict.set_item("node_count", self.valueflow.node_count())?;
                dict.set_item("edge_count", self.valueflow.edge_count())?;

                // Export nodes using Debug format
                let nodes: Vec<String> = self
                    .valueflow
                    .nodes()
                    .iter()
                    .map(|n| format!("{n:?}"))
                    .collect();
                dict.set_item("nodes", nodes)?;

                // Export edges
                let mut edges = Vec::new();
                for node in self.valueflow.nodes() {
                    if let Some(succs) = self.valueflow.successors_of(*node) {
                        for (kind, succ) in succs {
                            edges.push((
                                format!("{node:?}"),
                                format!("{kind:?}"),
                                format!("{succ:?}"),
                            ));
                        }
                    }
                }
                dict.set_item("edges", edges)?;

                Ok(dict.into())
            }
            "pta" => {
                let config = saf_analysis::PtaConfig::default();
                let export = self.pta_result.export(&config);
                crate::helpers::serde_to_py_dict(py, &export)
            }
            _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown graph type: {name}. Available: valueflow, pta"
            ))),
        }
    }

    fn __repr__(&self) -> String {
        "Query()".to_string()
    }
}

impl PyQuery {
    /// Resolve a selector set to value IDs.
    fn resolve_selectors(
        &self,
        py: Python<'_>,
        selector_set: &PySelectorSet,
    ) -> PyResult<BTreeSet<saf_core::ids::ValueId>> {
        let selectors: Vec<RustSelector> = selector_set.inner().to_vec();

        resolve_selectors(&selectors, &self.module).map_err(|e| {
            query_error(
                py,
                codes::INVALID_SELECTOR,
                &format!("Failed to resolve selector: {e}"),
            )
        })
    }
}

/// Register query types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQuery>()?;
    Ok(())
}
