//! Python bindings for demand-driven pointer analysis (DDA).
//!
//! Provides access to `DdaPta` with demand-driven points-to queries,
//! alias analysis, reachability, diagnostics, and JSON export.

use std::sync::{Arc, Mutex};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use saf_analysis::callgraph::CallGraph;
use saf_analysis::dda::{DdaCache, DdaConfig, DdaPta};
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::module_index::ModuleIndex;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::{Svfg, SvfgBuilder};
use saf_analysis::{PtaResult, PtsRepresentation};
use saf_core::air::AirModule;

use crate::id_parse::parse_value_id;

/// Aggregated DDA diagnostics counters.
#[derive(Default)]
struct DdaDiagnostics {
    queries: usize,
    cache_hits: usize,
    fallbacks: usize,
    strong_updates: usize,
    total_steps: usize,
    context_terminations: usize,
    cache_entries: usize,
}

/// Python wrapper for demand-driven pointer analysis.
///
/// Provides on-demand points-to queries, alias analysis, reachability checks,
/// diagnostics, and JSON export.
///
/// Obtain via `Project.demand_pta()`.
#[pyclass(name = "DdaPtaResult")]
pub struct PyDdaPtaResult {
    // Owned data borrowed by DdaPta in create_solver()
    module: AirModule,
    pta_result: PtaResult,
    callgraph: CallGraph,
    mssa: MemorySsa,
    svfg: Svfg,
    index: ModuleIndex,
    config: DdaConfig,
    // Persistent DDA cache reused across Python calls
    cache: Mutex<DdaCache>,
    // Aggregated diagnostics (single lock for all counters)
    diagnostics: Arc<Mutex<DdaDiagnostics>>,
}

impl PyDdaPtaResult {
    /// Build from project internals.
    #[allow(clippy::too_many_arguments, clippy::must_use_candidate)]
    pub fn build(
        module: &AirModule,
        pta_result: &Arc<PtaResult>,
        callgraph: &CallGraph,
        max_steps: usize,
        max_context_depth: usize,
        timeout_ms: u64,
        enable_strong_updates: bool,
    ) -> Self {
        Self::build_with_repr(
            module,
            pta_result,
            callgraph,
            max_steps,
            max_context_depth,
            timeout_ms,
            enable_strong_updates,
            PtsRepresentation::Auto,
        )
    }

    /// Build from project internals with specific representation.
    #[allow(clippy::too_many_arguments, clippy::must_use_candidate)]
    pub fn build_with_repr(
        module: &AirModule,
        pta_result: &Arc<PtaResult>,
        callgraph: &CallGraph,
        max_steps: usize,
        max_context_depth: usize,
        timeout_ms: u64,
        enable_strong_updates: bool,
        repr: PtsRepresentation,
    ) -> Self {
        let config = DdaConfig {
            max_steps,
            max_context_depth,
            timeout_ms,
            enable_strong_updates,
            ..DdaConfig::default()
        }
        .with_pts_representation(repr);

        // Clone data we need to own
        let owned_module = module.clone();
        let owned_pta_result = (**pta_result).clone();
        let owned_callgraph = callgraph.clone();

        let cfgs = crate::helpers::build_cfgs(&owned_module);

        let defuse = DefUseGraph::build(&owned_module);

        // Build MSSA (we'll rebuild it once for SVFG building)
        let mut mssa = MemorySsa::build(
            &owned_module,
            &cfgs,
            owned_pta_result.clone(),
            &owned_callgraph,
        );

        // Build SVFG (requires mutable mssa reference)
        let (svfg, _program_points) = SvfgBuilder::new(
            &owned_module,
            &defuse,
            &owned_callgraph,
            &owned_pta_result,
            &mut mssa,
        )
        .build();

        let index = ModuleIndex::build(&owned_module);

        Self {
            module: owned_module,
            pta_result: owned_pta_result,
            callgraph: owned_callgraph,
            mssa,
            svfg,
            index,
            config,
            cache: Mutex::new(DdaCache::new()),
            diagnostics: Arc::new(Mutex::new(DdaDiagnostics::default())),
        }
    }

    /// Create a `DdaPta` solver reusing the persistent cache.
    ///
    /// Takes the cache from the mutex so the solver owns it during
    /// the query. Call [`return_solver`](Self::return_solver) after
    /// the query to put the cache back and update diagnostics.
    fn create_solver(&self) -> DdaPta<'_> {
        let cache = self
            .cache
            .lock()
            .map(|mut guard| std::mem::take(&mut *guard))
            .unwrap_or_default();
        DdaPta::new_with_cache(
            &self.svfg,
            &self.mssa,
            &self.pta_result,
            &self.module,
            &self.callgraph,
            &self.index,
            self.config.clone(),
            cache,
        )
    }

    /// Return the solver's cache and update diagnostics.
    ///
    /// Consumes the solver, extracts its cache for reuse in future
    /// queries, and accumulates diagnostics counters.
    fn return_solver(&self, solver: DdaPta<'_>) {
        let diag = solver.diagnostics().clone();
        let cache_entries = solver.cache_stats().tl_entries;
        let cache = solver.take_cache();
        if let Ok(mut guard) = self.cache.lock() {
            *guard = cache;
        }
        if let Ok(mut d) = self.diagnostics.lock() {
            d.queries += diag.queries;
            d.cache_hits += diag.cache_hits;
            d.fallbacks += diag.fallbacks;
            d.strong_updates += diag.strong_updates;
            d.total_steps += diag.total_steps;
            d.context_terminations += diag.context_terminations;
            d.cache_entries = cache_entries;
        }
    }
}

#[pymethods]
impl PyDdaPtaResult {
    /// Query the points-to set for a value (on-demand).
    ///
    /// Args:
    ///     value_hex: Value ID as hex string (e.g., "0x00000...").
    ///
    /// Returns:
    ///     list[str]: Location IDs that the value may point to (hex).
    fn points_to(&self, py: Python<'_>, value_hex: &str) -> PyResult<Py<PyList>> {
        let vid = parse_value_id(value_hex)?;
        let mut solver = self.create_solver();
        let pts = solver.points_to(vid);
        self.return_solver(solver);
        let locs: Vec<String> = pts.iter().map(|l| format!("0x{:032x}", l.raw())).collect();
        Ok(PyList::new(py, &locs)?.into())
    }

    /// Check if two pointers may alias (on-demand).
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
        let mut solver = self.create_solver();
        let result = solver.may_alias(p, q);
        self.return_solver(solver);
        Ok(crate::helpers::alias_result_to_str(result).to_owned())
    }

    /// Check if there is a value-flow path from source to sink.
    ///
    /// Args:
    ///     src_hex: Source value ID as hex.
    ///     sink_hex: Sink value ID as hex.
    ///
    /// Returns:
    ///     bool: True if the source may flow to the sink.
    fn reachable(&self, src_hex: &str, sink_hex: &str) -> PyResult<bool> {
        let src = parse_value_id(src_hex)?;
        let sink = parse_value_id(sink_hex)?;
        let mut solver = self.create_solver();
        let result = solver.reachable(src, sink);
        self.return_solver(solver);
        Ok(result)
    }

    /// Check reachability with detailed result.
    ///
    /// Args:
    ///     src_hex: Source value ID as hex.
    ///     sink_hex: Sink value ID as hex.
    ///
    /// Returns:
    ///     dict: Reachability result with keys:
    ///         - "reachable": bool
    ///         - "via_alias": bool
    ///         - "via_svfg": bool
    ///         - "src_pts_count": int
    ///         - "sink_pts_count": int
    fn reachable_refined(
        &self,
        py: Python<'_>,
        src_hex: &str,
        sink_hex: &str,
    ) -> PyResult<Py<PyDict>> {
        let src = parse_value_id(src_hex)?;
        let sink = parse_value_id(sink_hex)?;
        let mut solver = self.create_solver();
        let result = solver.reachable_refined(src, sink);
        self.return_solver(solver);

        let dict = PyDict::new(py);
        dict.set_item("reachable", result.reachable)?;
        dict.set_item("via_alias", result.via_alias)?;
        dict.set_item("via_svfg", result.via_svfg)?;
        dict.set_item("src_pts_count", result.src_pts_count)?;
        dict.set_item("sink_pts_count", result.sink_pts_count)?;
        Ok(dict.into())
    }

    /// Get analysis diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys "queries", "cache_hits", "fallbacks",
    ///           "strong_updates", "total_steps", "context_terminations".
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let d = self
            .diagnostics
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let dict = PyDict::new(py);
        dict.set_item("queries", d.queries)?;
        dict.set_item("cache_hits", d.cache_hits)?;
        dict.set_item("fallbacks", d.fallbacks)?;
        dict.set_item("strong_updates", d.strong_updates)?;
        dict.set_item("total_steps", d.total_steps)?;
        dict.set_item("context_terminations", d.context_terminations)?;
        Ok(dict.into())
    }

    /// Get cache statistics.
    ///
    /// Returns:
    ///     dict: Cache stats with key "tl_entries" (top-level cache entries).
    fn cache_stats(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let d = self
            .diagnostics
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let dict = PyDict::new(py);
        dict.set_item("tl_entries", d.cache_entries)?;
        Ok(dict.into())
    }

    /// Export the result to a dictionary.
    ///
    /// Returns:
    ///     dict: Full export with schema_version, config, diagnostics, cache_stats.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let solver = self.create_solver();
        let export = solver.export();
        self.return_solver(solver);
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        let d = self
            .diagnostics
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        format!(
            "DdaPtaResult(queries={}, cache_hits={}, fallbacks={})",
            d.queries, d.cache_hits, d.fallbacks
        )
    }
}

/// Register DDA types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDdaPtaResult>()?;
    Ok(())
}
