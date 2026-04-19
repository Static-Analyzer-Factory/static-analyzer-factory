//! Python bindings for Project class.
//!
//! The Project class is the main entry point for SAF analysis in Python.

use std::path::Path;
use std::sync::{Arc, OnceLock};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::PtsRepresentation;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::cg_refinement::RefinementConfig;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::pipeline::{PipelineConfig, run_pipeline};
use saf_analysis::svfg::Svfg;
use saf_analysis::{
    FieldSensitivity, PtaConfig, PtaResult, ValueFlowConfig, ValueFlowGraph, ValueFlowMode,
};
use saf_core::air::AirModule;
use saf_core::config::Config;
use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;
#[cfg(any(feature = "llvm-18", feature = "llvm-22"))]
use saf_frontends::llvm::LlvmFrontend;

use crate::absint::{self as py_absint, PyAbstractInterpResult, PyNumericFinding};
use crate::air::PyAirModule;
use crate::cg_refinement::{self, PyRefinementResult};
use crate::checkers::{
    self as py_checkers, PyCheckerFinding, PyPathSensitiveResult, PyResourceTable,
};
use crate::cspta::PyCsPtaResult;
use crate::exceptions::{codes, frontend_error};
use crate::fspta::PyFlowSensitivePtaResult;
use crate::graphs::PyGraphStore;
use crate::id_parse::{parse_block_id, parse_function_id, parse_value_id};
use crate::ide;
use crate::ifds::{self, PyIfdsResult};
use crate::mssa::PyMemorySsa;
use crate::pta::PyPtaResult;
use crate::query::PyQuery;
use crate::schema::build_schema;
use crate::selector::SelectorOrSet;
use crate::svfg::PySvfg;
use crate::z3_refine;

/// A SAF analysis project opened from input files.
///
/// The Project class is the main entry point for SAF analysis. It loads
/// and analyzes input files, then provides access to query interfaces.
///
/// Example:
///     proj = Project.open("program.ll")
///     q = proj.query()
///     findings = q.taint_flow(sources, sinks)
#[pyclass(name = "Project")]
pub struct Project {
    /// The analyzed AIR module.
    module: Arc<AirModule>,
    /// The call graph.
    callgraph: Arc<CallGraph>,
    /// The def-use graph.
    defuse: Arc<DefUseGraph>,
    /// The PTA result.
    pta_result: Arc<PtaResult>,
    /// The value flow graph.
    valueflow: Arc<ValueFlowGraph>,
    /// Cached SVFG — built lazily on first checker call.
    svfg_cache: OnceLock<Svfg>,
}

#[pymethods]
impl Project {
    /// Open a project from the given path.
    ///
    /// The frontend is selected automatically based on the file extension:
    /// - `.air.json` → AIR-JSON frontend
    /// - `.ll` or `.bc` → LLVM frontend (requires LLVM feature)
    ///
    /// Args:
    ///     path: Path to the input file (`.air.json`, `.ll`, or `.bc`).
    ///     vf_mode: ValueFlow mode — `"fast"` (default) routes all memory
    ///         through a single unknown node for robust taint analysis;
    ///         `"precise"` uses points-to analysis to resolve memory
    ///         locations (may miss flows through unresolved pointers).
    ///     pta_solver: PTA solver to use — `"worklist"` (default) uses the
    ///         imperative worklist-based solver; `"datalog"` uses the
    ///         Ascent Datalog fixpoint solver.
    ///     pta_max_iterations: Maximum PTA solver iterations (default: 10000).
    ///     field_sensitivity_depth: Field sensitivity depth (0 = disabled,
    ///         default: 2). Higher values track deeper nested struct fields.
    ///     max_refinement_iterations: Maximum CG refinement iterations
    ///         (default: 10). Lower values trade precision for speed.
    ///
    /// Returns:
    ///     A Project instance ready for querying.
    ///
    /// Raises:
    ///     FrontendError: If the input file cannot be parsed or the
    ///         required frontend is not available.
    #[staticmethod]
    #[pyo3(signature = (path, *, vf_mode="fast", pta_solver="worklist", pta_max_iterations=None, field_sensitivity_depth=None, max_refinement_iterations=None))]
    fn open(
        py: Python<'_>,
        path: &str,
        vf_mode: &str,
        pta_solver: &str,
        pta_max_iterations: Option<usize>,
        field_sensitivity_depth: Option<u32>,
        max_refinement_iterations: Option<usize>,
    ) -> PyResult<Self> {
        let vf_mode_enum = match vf_mode {
            "fast" => ValueFlowMode::Fast,
            "precise" => ValueFlowMode::Precise,
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown vf_mode '{other}'. Expected 'fast' or 'precise'."
                )));
            }
        };

        // Validate pta_solver parameter
        let use_datalog = match pta_solver {
            "datalog" | "ascent" => true,
            "worklist" | "legacy" => false,
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown pta_solver '{other}'. Expected 'datalog' or 'worklist'."
                )));
            }
        };

        let file_path = Path::new(path);

        // Check file exists
        if !file_path.exists() {
            return Err(frontend_error(
                py,
                codes::FRONTEND_IO_ERROR,
                &format!("File not found: {path}"),
                Some(path),
            ));
        }

        let saf_config = Config::default();

        // Dispatch based on file extension
        let bundle = if path.ends_with(".air.json") {
            AirJsonFrontend
                .ingest(&[file_path], &saf_config)
                .map_err(|e| {
                    frontend_error(
                        py,
                        codes::FRONTEND_PARSE_ERROR,
                        &format!("Failed to parse input: {e}"),
                        Some(path),
                    )
                })?
        } else if path.to_ascii_lowercase().ends_with(".ll")
            || path.to_ascii_lowercase().ends_with(".bc")
        {
            #[cfg(any(feature = "llvm-18", feature = "llvm-22"))]
            {
                LlvmFrontend::new()
                    .ingest(&[file_path], &saf_config)
                    .map_err(|e| {
                        frontend_error(
                            py,
                            codes::FRONTEND_PARSE_ERROR,
                            &format!("Failed to parse LLVM IR: {e}"),
                            Some(path),
                        )
                    })?
            }
            #[cfg(not(any(feature = "llvm-18", feature = "llvm-22")))]
            {
                return Err(frontend_error(
                    py,
                    codes::FRONTEND_NOT_FOUND,
                    "LLVM support not compiled in. Rebuild with the llvm-18 or llvm-22 feature.",
                    Some(path),
                ));
            }
        } else {
            return Err(frontend_error(
                py,
                codes::FRONTEND_NOT_FOUND,
                &format!(
                    "Unsupported file extension for '{path}'. Expected .air.json, .ll, or .bc"
                ),
                Some(path),
            ));
        };

        let module = Arc::new(bundle.module);

        // Build analysis graphs
        let (callgraph, defuse, pta_result, valueflow) = build_analysis(
            &module,
            vf_mode_enum,
            use_datalog,
            pta_max_iterations,
            field_sensitivity_depth,
            max_refinement_iterations,
        );

        Ok(Self {
            module,
            callgraph: Arc::new(callgraph),
            defuse: Arc::new(defuse),
            pta_result: Arc::new(pta_result),
            valueflow: Arc::new(valueflow),
            svfg_cache: OnceLock::new(),
        })
    }

    /// Get a query interface for this project.
    ///
    /// Returns:
    ///     A Query object for running analysis queries.
    fn query(&self) -> PyQuery {
        PyQuery::new(
            Arc::clone(&self.module),
            Arc::clone(&self.valueflow),
            Arc::clone(&self.pta_result),
            Arc::clone(&self.callgraph),
            Arc::clone(&self.defuse),
        )
    }

    /// Get the schema describing all SAF capabilities.
    ///
    /// Returns a dictionary with information about frontends, graphs,
    /// queries, selectors, and configuration options. Designed for
    /// AI agent discoverability.
    ///
    /// Returns:
    ///     dict: Schema dictionary.
    fn schema(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        build_schema(py)
    }

    /// Get the analyzed AIR module.
    ///
    /// Provides mid-level access to the intermediate representation.
    ///
    /// Returns:
    ///     AirModule: The analyzed module.
    fn air(&self) -> PyAirModule {
        PyAirModule::new(Arc::clone(&self.module))
    }

    /// Get the graph store.
    ///
    /// Provides access to CFG, call graph, def-use graph, and value flow graph.
    ///
    /// Returns:
    ///     GraphStore: Access to all analysis graphs.
    fn graphs(&self) -> PyGraphStore {
        PyGraphStore::new(
            Arc::clone(&self.module),
            Arc::clone(&self.callgraph),
            Arc::clone(&self.defuse),
            Arc::clone(&self.valueflow),
        )
    }

    /// Get the points-to analysis result.
    ///
    /// Returns:
    ///     PtaResult: The points-to analysis result.
    fn pta_result(&self) -> PyPtaResult {
        PyPtaResult::new(Arc::clone(&self.pta_result))
    }

    /// Get a display resolver for translating hex IDs to human-readable names.
    ///
    /// The resolver maps opaque hex IDs (functions, blocks, instructions,
    /// values, globals, PTA locations, SVFG nodes) to descriptive labels
    /// with names, source locations, and context.
    ///
    /// Returns:
    ///     DisplayResolver: A resolver instance.
    fn display_resolver(&self) -> crate::display::PyDisplayResolver {
        crate::display::PyDisplayResolver::new(
            Arc::clone(&self.module),
            Some(Arc::clone(&self.pta_result)),
            None,
        )
    }

    /// Run IFDS-based taint analysis.
    ///
    /// Uses the IFDS framework (Reps/Horwitz/Sagiv tabulation algorithm)
    /// for precise interprocedural taint tracking. This is more precise
    /// than BFS-based taint_flow for inter-procedural flows.
    ///
    /// Args:
    ///     sources: Selector or SelectorSet for taint sources.
    ///     sinks: Selector or SelectorSet for taint sinks.
    ///     sanitizers: Optional Selector or SelectorSet for sanitizers.
    ///
    /// Returns:
    ///     IfdsResult: The analysis result with taint facts at each point.
    #[pyo3(signature = (sources, sinks, sanitizers=None))]
    fn ifds_taint(
        &self,
        py: Python<'_>,
        sources: SelectorOrSet,
        sinks: SelectorOrSet,
        sanitizers: Option<SelectorOrSet>,
    ) -> PyResult<PyIfdsResult> {
        let source_sels = sources.into_selector_set().inner().to_vec();
        let sink_sels = sinks.into_selector_set().inner().to_vec();
        let sanitizer_sels = sanitizers
            .map(|s| s.into_selector_set().inner().to_vec())
            .unwrap_or_default();

        ifds::run_ifds_taint(
            py,
            &self.module,
            &self.callgraph,
            source_sels,
            sink_sels,
            sanitizer_sels,
        )
    }

    /// Run typestate analysis with a built-in specification.
    ///
    /// Uses the IDE framework (Sagiv/Reps/Horwitz TCS'96) to track
    /// per-resource state machines across the program.
    ///
    /// Args:
    ///     spec: Name of a built-in typestate spec.
    ///         Available: ``"file_io"``, ``"mutex_lock"``, ``"memory_alloc"``.
    ///
    /// Returns:
    ///     TypestateResult: Analysis result with violation findings.
    fn typestate(&self, spec: &str) -> PyResult<ide::PyTypestateResult> {
        ide::run_typestate_builtin(&self.module, &self.callgraph, spec)
    }

    /// Run typestate analysis with a custom specification.
    ///
    /// Args:
    ///     spec: A `TypestateSpec` object defining the state machine.
    ///
    /// Returns:
    ///     TypestateResult: Analysis result with violation findings.
    fn typestate_custom(&self, spec: &ide::PyTypestateSpec) -> PyResult<ide::PyTypestateResult> {
        ide::run_typestate_custom(&self.module, &self.callgraph, spec)
    }

    /// Run iterative call graph refinement (CHA + PTA).
    ///
    /// Builds a class hierarchy (for C++ virtual dispatch), then runs
    /// iterative PTA-based indirect call resolution until the reachable
    /// set stabilises.
    ///
    /// Args:
    ///     entry_points: Entry point strategy — `"all"` for all defined
    ///         functions, or a comma-separated list of function names.
    ///     max_iterations: Maximum PTA refinement iterations (default: 10).
    ///
    /// Returns:
    ///     RefinementResult: The refined call graph, CHA, PTA result, and
    ///         resolved indirect call sites.
    #[pyo3(signature = (*, entry_points="all", max_iterations=10))]
    fn refine_call_graph(&self, entry_points: &str, max_iterations: usize) -> PyRefinementResult {
        cg_refinement::run_refinement(&self.module, entry_points, max_iterations)
    }

    /// Build Memory SSA for this project.
    ///
    /// Constructs memory def-use chains for all address-taken variables
    /// using a hybrid approach: location-partitioned skeleton with
    /// demand-driven clobber disambiguation.
    ///
    /// Returns:
    ///     MemorySsa: The Memory SSA representation.
    fn memory_ssa(&self) -> PyMemorySsa {
        let pta_clone = (*self.pta_result).clone();
        PyMemorySsa::build(&self.module, &self.callgraph, pta_clone)
    }

    /// Build the Sparse Value-Flow Graph (SVFG) for this project.
    ///
    /// Constructs a unified graph combining direct (register/SSA) and
    /// indirect (memory) value-flow edges using Memory SSA clobber analysis.
    ///
    /// Returns:
    ///     Svfg: The Sparse Value-Flow Graph.
    fn svfg(&self) -> PySvfg {
        let mssa_pta = (*self.pta_result).clone();
        PySvfg::build(
            &self.module,
            &self.callgraph,
            &self.defuse,
            &self.pta_result,
            mssa_pta,
        )
    }

    /// Run flow-sensitive pointer analysis.
    ///
    /// Builds the full pipeline (SVFG → FsSvfg → SFS solver) and returns
    /// per-value flow-sensitive points-to sets. More precise than Andersen's
    /// flow-insensitive analysis for programs with pointer reassignment.
    ///
    /// Args:
    ///     pts_repr: Points-to set representation — "auto" (default), "btreeset",
    ///         "bitvector", or "bdd". Auto selects based on allocation site count.
    ///
    /// Returns:
    ///     FlowSensitivePtaResult: Flow-sensitive points-to analysis result.
    #[pyo3(signature = (*, pts_repr="auto"))]
    fn flow_sensitive_pta(&self, pts_repr: &str) -> PyResult<PyFlowSensitivePtaResult> {
        let repr = parse_pts_repr(pts_repr)?;
        let mssa_pta = (*self.pta_result).clone();
        Ok(PyFlowSensitivePtaResult::build_with_repr(
            &self.module,
            &self.callgraph,
            &self.defuse,
            &self.pta_result,
            mssa_pta,
            repr,
        ))
    }

    /// Run context-sensitive pointer analysis (k-CFA).
    ///
    /// Qualifies each pointer value by a bounded call-site context,
    /// distinguishing calls to the same function from different sites.
    /// More precise than Andersen's context-insensitive analysis for
    /// wrapper functions and factory patterns.
    ///
    /// Args:
    ///     k: Context depth (1, 2, or 3). Default: 1.
    ///     pts_repr: Points-to set representation — "auto" (default), "btreeset",
    ///         "bitvector", or "bdd". Auto selects based on allocation site count.
    ///
    /// Returns:
    ///     CsPtaResult: Context-sensitive points-to analysis result.
    #[pyo3(signature = (*, k=1, pts_repr="auto"))]
    fn context_sensitive_pta(&self, k: u32, pts_repr: &str) -> PyResult<PyCsPtaResult> {
        let repr = parse_pts_repr(pts_repr)?;
        Ok(PyCsPtaResult::build_with_repr(
            &self.module,
            &self.callgraph,
            k,
            repr,
        ))
    }

    /// Run demand-driven pointer analysis.
    ///
    /// Computes points-to information only for explicitly queried pointers,
    /// using CFL-reachability for context-sensitive backward traversal on SVFG.
    /// More scalable than whole-program analysis for targeted queries.
    ///
    /// Args:
    ///     max_steps: Maximum traversal steps per query (0 = unlimited). Default: 100000.
    ///     max_context_depth: Maximum call-string depth (0 = unlimited). Default: 10.
    ///     timeout_ms: Timeout in milliseconds per query (0 = unlimited). Default: 5000.
    ///     enable_strong_updates: Enable strong update optimization. Default: True.
    ///     pts_repr: Points-to set representation — "auto" (default), "btreeset",
    ///         "bitvector", or "bdd". Auto selects based on allocation site count.
    ///
    /// Returns:
    ///     DdaPtaResult: Demand-driven pointer analysis handle.
    #[pyo3(signature = (*, max_steps=100_000, max_context_depth=10, timeout_ms=5000, enable_strong_updates=true, pts_repr="auto"))]
    fn demand_pta(
        &self,
        max_steps: usize,
        max_context_depth: usize,
        timeout_ms: u64,
        enable_strong_updates: bool,
        pts_repr: &str,
    ) -> PyResult<crate::dda::PyDdaPtaResult> {
        let repr = parse_pts_repr(pts_repr)?;
        Ok(crate::dda::PyDdaPtaResult::build_with_repr(
            &self.module,
            &self.pta_result,
            &self.callgraph,
            max_steps,
            max_context_depth,
            timeout_ms,
            enable_strong_updates,
            repr,
        ))
    }

    /// Run one or more named built-in checkers.
    ///
    /// Uses SVFG-based reachability analysis to detect memory safety,
    /// resource safety, and other bug patterns.
    ///
    /// Args:
    ///     name: A checker name (str) or list of checker names.
    ///         Available: "memory-leak", "use-after-free", "double-free",
    ///         "null-deref", "file-descriptor-leak", "uninit-use",
    ///         "stack-escape", "lock-not-released", "generic-resource-leak".
    ///
    /// Returns:
    ///     list[CheckerFinding]: Findings from the checker(s).
    fn check(&self, py: Python<'_>, name: &Bound<'_, PyAny>) -> PyResult<Py<pyo3::types::PyList>> {
        let names = crate::helpers::extract_checker_names(name)?;

        let table = saf_analysis::checkers::ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let result = py_checkers::run_check_with_svfg(&self.module, svfg, &table, names);

        let (findings_list, _diag) = py_checkers::findings_to_py(py, result)?;
        Ok(findings_list)
    }

    /// Run all built-in checkers.
    ///
    /// Runs all 9 built-in checkers (memory-leak, use-after-free,
    /// double-free, null-deref, file-descriptor-leak, uninit-use,
    /// stack-escape, lock-not-released, generic-resource-leak).
    ///
    /// Returns:
    ///     list[CheckerFinding]: All findings from all checkers.
    fn check_all(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        let table = saf_analysis::checkers::ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let result = py_checkers::run_check_all_with_svfg(&self.module, svfg, &table);

        let (findings_list, _diag) = py_checkers::findings_to_py(py, result)?;
        Ok(findings_list)
    }

    /// Run a custom checker with user-defined source/sink/sanitizer specs.
    ///
    /// Args:
    ///     name: Custom checker name.
    ///     mode: Reachability mode — "may_reach" or "must_not_reach".
    ///     source_role: Resource role for sources (e.g., "allocator", "acquire").
    ///     source_match_return: If True, match return value; if False, match first arg.
    ///     sink_role: Resource role for sinks, or None for function exits.
    ///     sink_is_exit: If True, sinks are function exits.
    ///     sanitizer_role: Resource role for sanitizers, or None.
    ///     sanitizer_match_return: If True, match return value; if False, match first arg.
    ///     cwe: Optional CWE ID.
    ///     severity: Severity level — "info", "warning", "error", "critical".
    ///
    /// Returns:
    ///     list[CheckerFinding]: Findings from the custom checker.
    #[pyo3(signature = (name, *, mode="must_not_reach", source_role="allocator",
                        source_match_return=true, sink_role=None, sink_is_exit=true,
                        sanitizer_role=None, sanitizer_match_return=false,
                        cwe=None, severity="warning"))]
    // INVARIANT: PyO3 requires each checker parameter as a separate argument
    // for Python keyword-argument support with defaults.
    #[allow(clippy::too_many_arguments)]
    fn check_custom(
        &self,
        py: Python<'_>,
        name: &str,
        mode: &str,
        source_role: &str,
        source_match_return: bool,
        sink_role: Option<&str>,
        sink_is_exit: bool,
        sanitizer_role: Option<&str>,
        sanitizer_match_return: bool,
        cwe: Option<u32>,
        severity: &str,
    ) -> PyResult<Py<pyo3::types::PyList>> {
        let spec = py_checkers::build_custom_spec(
            name,
            mode,
            source_role,
            source_match_return,
            sink_role,
            sink_is_exit,
            sanitizer_role,
            sanitizer_match_return,
            cwe,
            severity,
        )?;

        let table = saf_analysis::checkers::ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let result = py_checkers::run_check_custom_with_svfg(&self.module, svfg, &table, spec);

        let (findings_list, _diag) = py_checkers::findings_to_py(py, result)?;
        Ok(findings_list)
    }

    /// Run one or more named checkers with path-sensitive filtering.
    ///
    /// Uses Z3 SMT solver to verify path feasibility of each finding.
    /// Infeasible paths (contradictory branch conditions) are classified
    /// as false positives and separated from genuine findings.
    ///
    /// Args:
    ///     name: A checker name (str) or list of checker names.
    ///     z3_timeout_ms: Z3 solver timeout per finding in milliseconds (default: 1000).
    ///     max_guards: Maximum branch guards per trace before skipping Z3 (default: 64).
    ///
    /// Returns:
    ///     PathSensitiveResult: Result with feasible, infeasible, and unknown findings.
    #[pyo3(signature = (name, *, z3_timeout_ms=1000, max_guards=64))]
    fn check_path_sensitive(
        &self,
        name: &Bound<'_, PyAny>,
        z3_timeout_ms: u64,
        max_guards: usize,
    ) -> PyResult<PyPathSensitiveResult> {
        let names = crate::helpers::extract_checker_names(name)?;

        let table = saf_analysis::checkers::ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let result = py_checkers::run_check_path_sensitive_with_svfg(
            &self.module,
            svfg,
            &table,
            names,
            z3_timeout_ms,
            max_guards,
        );
        Ok(PyPathSensitiveResult { inner: result })
    }

    /// Run all built-in checkers with path-sensitive filtering.
    ///
    /// Uses Z3 SMT solver to verify path feasibility of each finding,
    /// separating genuine bugs from false positives caused by infeasible
    /// paths.
    ///
    /// Args:
    ///     z3_timeout_ms: Z3 solver timeout per finding in milliseconds (default: 1000).
    ///     max_guards: Maximum branch guards per trace before skipping Z3 (default: 64).
    ///
    /// Returns:
    ///     PathSensitiveResult: Result with feasible, infeasible, and unknown findings.
    #[pyo3(signature = (*, z3_timeout_ms=1000, max_guards=64))]
    fn check_all_path_sensitive(
        &self,
        z3_timeout_ms: u64,
        max_guards: usize,
    ) -> PyPathSensitiveResult {
        let table = saf_analysis::checkers::ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let result = py_checkers::run_check_all_path_sensitive_with_svfg(
            &self.module,
            svfg,
            &table,
            z3_timeout_ms,
            max_guards,
        );
        PyPathSensitiveResult { inner: result }
    }

    /// Post-filter existing findings for path feasibility.
    ///
    /// Takes a list of ``CheckerFinding`` objects (from ``check()`` or
    /// ``check_all()``) and applies Z3-based path feasibility checking.
    ///
    /// Args:
    ///     findings: List of ``CheckerFinding`` objects to filter.
    ///     z3_timeout_ms: Z3 solver timeout per finding in milliseconds (default: 1000).
    ///     max_guards: Maximum branch guards per trace before skipping Z3 (default: 64).
    ///
    /// Returns:
    ///     PathSensitiveResult: Result with feasible, infeasible, and unknown findings.
    #[pyo3(signature = (findings, *, z3_timeout_ms=1000, max_guards=64))]
    fn filter_infeasible(
        &self,
        findings: Vec<PyRef<'_, PyCheckerFinding>>,
        z3_timeout_ms: u64,
        max_guards: usize,
    ) -> PyPathSensitiveResult {
        let rust_findings: Vec<saf_analysis::checkers::CheckerFinding> =
            findings.iter().map(|f| f.inner.clone()).collect();
        let result = py_checkers::run_filter_infeasible(
            rust_findings,
            &self.module,
            z3_timeout_ms,
            max_guards,
        );
        PyPathSensitiveResult { inner: result }
    }

    /// Get checker diagnostics from running all checkers.
    ///
    /// Returns:
    ///     dict: Diagnostics including checkers_run, classified_sites,
    ///           source_nodes, sink_nodes, sanitizer_nodes, total_findings.
    fn checker_diagnostics(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyDict>> {
        let table = saf_analysis::checkers::ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let result = py_checkers::run_check_all_with_svfg(&self.module, svfg, &table);

        let (_findings_list, diag) = py_checkers::findings_to_py(py, result)?;
        Ok(diag)
    }

    /// Get the resource table with all built-in entries.
    ///
    /// Returns:
    ///     ResourceTable: The resource table.
    fn resource_table(&self) -> PyResourceTable {
        PyResourceTable {
            inner: saf_analysis::checkers::ResourceTable::new(),
        }
    }

    /// Get the checker schema — all built-in checker names and descriptions.
    ///
    /// Returns:
    ///     dict: Checker schema with names, descriptions, CWE IDs,
    ///           modes, and severity levels.
    fn checker_schema(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new(py);
        let checkers = saf_analysis::checkers::builtin_checkers();
        let list = pyo3::types::PyList::empty(py);

        for spec in &checkers {
            let entry = pyo3::types::PyDict::new(py);
            entry.set_item("name", &spec.name)?;
            entry.set_item("description", &spec.description)?;
            entry.set_item("cwe", spec.cwe)?;
            entry.set_item("severity", spec.severity.name())?;
            let mode = match spec.mode {
                saf_analysis::checkers::ReachabilityMode::MayReach => "may_reach",
                saf_analysis::checkers::ReachabilityMode::MustNotReach => "must_not_reach",
                saf_analysis::checkers::ReachabilityMode::MultiReach => "multi_reach",
                saf_analysis::checkers::ReachabilityMode::NeverReachSink => "never_reach_sink",
            };
            entry.set_item("mode", mode)?;
            list.append(entry)?;
        }

        dict.set_item("checkers", list)?;
        dict.set_item("count", checkers.len())?;
        Ok(dict.into())
    }

    /// Run abstract interpretation on the module.
    ///
    /// Computes numeric value ranges at every program point using interval
    /// analysis with widening and narrowing.
    ///
    /// Args:
    ///     max_widening: Maximum widening iterations per loop (default: 100).
    ///     narrowing_iterations: Narrowing iterations after convergence (default: 3).
    ///     use_thresholds: Use program-constant widening thresholds (default: True).
    ///
    /// Returns:
    ///     AbstractInterpResult: Computed invariants queryable at blocks and instructions.
    #[pyo3(signature = (*, max_widening=None, narrowing_iterations=None, use_thresholds=None))]
    fn abstract_interp(
        &self,
        max_widening: Option<u32>,
        narrowing_iterations: Option<u32>,
        use_thresholds: Option<bool>,
    ) -> PyAbstractInterpResult {
        py_absint::run_abstract_interp_with_config(
            &self.module,
            max_widening,
            narrowing_iterations,
            use_thresholds,
        )
    }

    /// Run combined PTA + abstract interpretation analysis.
    ///
    /// This analysis combines pointer analysis with abstract interpretation
    /// for alias-aware numeric analysis, indirect call resolution, and
    /// selective memory invalidation.
    ///
    /// Args:
    ///     enable_refinement: Enable bidirectional refinement loop (default True).
    ///     max_refinement_iterations: Maximum refinement iterations (default 3).
    ///
    /// Returns:
    ///     CombinedAnalysisResult: Combined analysis result with methods:
    ///         - `interval_at(value_hex)`: Get numeric interval for a value.
    ///         - `points_to(ptr_hex)`: Get points-to set for a pointer.
    ///         - `may_alias(ptr_a, ptr_b)`: Check alias relationship.
    ///
    /// Example:
    ///     result = project.analyze_combined()
    ///     interval = result.interval_at("0x1234...")
    ///     alias = result.may_alias("0x5678...", "0x9abc...")
    #[pyo3(signature = (*, enable_refinement=None, max_refinement_iterations=None))]
    fn analyze_combined(
        &self,
        enable_refinement: Option<bool>,
        max_refinement_iterations: Option<usize>,
    ) -> crate::combined::PyCombinedAnalysisResult {
        crate::combined::run_analyze_combined_with_config(
            &self.module,
            enable_refinement,
            max_refinement_iterations,
        )
    }

    /// Run a named numeric checker.
    ///
    /// Runs abstract interpretation and then checks for numeric bugs.
    ///
    /// Args:
    ///     name: Checker name — one of:
    ///         - "buffer_overflow" (CWE-120)
    ///         - "integer_overflow" (CWE-190)
    ///         - "division_by_zero" (CWE-369)
    ///         - "shift_count" (CWE-682)
    ///
    /// Returns:
    ///     list[NumericFinding]: Findings from the checker.
    ///
    /// Raises:
    ///     ValueError: If the checker name is unknown.
    fn check_numeric(&self, name: &str) -> PyResult<Vec<PyNumericFinding>> {
        py_absint::run_check_numeric(&self.module, name)
    }

    /// Run all numeric checkers (buffer overflow, integer overflow, division by zero, shift count).
    ///
    /// Returns:
    ///     list[NumericFinding]: Combined findings from all numeric checkers.
    fn check_all_numeric(&self) -> Vec<PyNumericFinding> {
        py_absint::run_check_all_numeric(&self.module)
    }

    // -----------------------------------------------------------------
    // Z3-enhanced analysis methods
    // -----------------------------------------------------------------

    /// Prove or disprove assertions in the program using Z3.
    ///
    /// Scans for ``assert()``-like calls, extracts dominating branch
    /// guards, and uses Z3 to check if the assertion failure path is
    /// feasible.
    ///
    /// Args:
    ///     z3_timeout_ms: Z3 solver timeout per assertion in ms (default: 1000).
    ///     max_guards: Maximum branch guards before skipping (default: 64).
    ///     assert_functions: Custom assert function names (default: built-in set).
    ///
    /// Returns:
    ///     AssertionResult: Result with proven, may_fail, and unknown assertions.
    #[pyo3(signature = (*, z3_timeout_ms=1000, max_guards=64, assert_functions=None))]
    fn prove_assertions(
        &self,
        z3_timeout_ms: u64,
        max_guards: usize,
        assert_functions: Option<Vec<String>>,
    ) -> z3_refine::PyAssertionResult {
        let fns = assert_functions.unwrap_or_default();
        z3_refine::run_prove_assertions(&self.module, None, z3_timeout_ms, max_guards, &fns)
    }

    /// Refine a may-alias query using Z3 path constraints.
    ///
    /// Given two pointer value IDs, a program point (block), and a function,
    /// checks if the aliasing is feasible on any concrete execution path.
    ///
    /// Args:
    ///     p: First pointer value ID (hex string).
    ///     q: Second pointer value ID (hex string).
    ///     at_block: Block ID where the alias query is evaluated (hex string).
    ///     func_id: Function ID containing the block (hex string).
    ///     z3_timeout_ms: Z3 solver timeout in ms (default: 1000).
    ///     max_guards: Maximum branch guards before skipping (default: 64).
    ///
    /// Returns:
    ///     AliasRefinementResult: Verdict (confirmed_alias, no_alias, unknown).
    #[pyo3(signature = (p, q, at_block, func_id, *, z3_timeout_ms=1000, max_guards=64))]
    #[allow(clippy::similar_names)]
    fn refine_alias(
        &self,
        p: &str,
        q: &str,
        at_block: &str,
        func_id: &str,
        z3_timeout_ms: u64,
        max_guards: usize,
    ) -> PyResult<z3_refine::PyAliasRefinementResult> {
        let p_id = parse_value_id(p)?;
        let q_id = parse_value_id(q)?;
        let block = parse_block_id(at_block)?;
        let func = parse_function_id(func_id)?;

        Ok(z3_refine::run_refine_alias(
            &self.module,
            &self.pta_result,
            p_id,
            q_id,
            block,
            func,
            z3_timeout_ms,
            max_guards,
        ))
    }

    /// Check if a feasible execution path exists between two blocks.
    ///
    /// Enumerates CFG paths and uses Z3 to check branch guard feasibility.
    ///
    /// Args:
    ///     from_block: Source block ID (hex string).
    ///     to_block: Target block ID (hex string).
    ///     func_id: Function ID (hex string).
    ///     z3_timeout_ms: Z3 solver timeout in ms (default: 1000).
    ///     max_guards: Maximum branch guards per path (default: 64).
    ///     max_paths: Maximum paths to enumerate (default: 100).
    ///
    /// Returns:
    ///     PathReachabilityResult: Verdict with optional witness path.
    #[pyo3(signature = (from_block, to_block, func_id, *, z3_timeout_ms=1000, max_guards=64, max_paths=100))]
    fn check_path_reachable(
        &self,
        from_block: &str,
        to_block: &str,
        func_id: &str,
        z3_timeout_ms: u64,
        max_guards: usize,
        max_paths: usize,
    ) -> PyResult<z3_refine::PyPathReachabilityResult> {
        let from = parse_block_id(from_block)?;
        let to = parse_block_id(to_block)?;
        let func = parse_function_id(func_id)?;

        Ok(z3_refine::run_check_path_reachable(
            &self.module,
            from,
            to,
            func,
            z3_timeout_ms,
            max_guards,
            max_paths,
        ))
    }

    /// Handle a JSON protocol request against the `ProgramDatabase`.
    ///
    /// Accepts a JSON string and returns a JSON string response.
    /// This is the primary interface for LLM agents.
    ///
    /// Example:
    ///     resp = proj.request('{"action": "schema"}')
    ///     data = json.loads(resp)
    ///     assert data["status"] == "ok"
    ///
    /// Args:
    ///     json_request: A JSON string containing the request.
    ///
    /// Returns:
    ///     str: A JSON string containing the response.
    #[pyo3(signature = (json_request))]
    fn request(&self, json_request: &str) -> PyResult<String> {
        let db = self.build_database();
        db.handle_request(json_request)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON error: {e}")))
    }

    fn __repr__(&self) -> String {
        let name = self.module.name.as_deref().unwrap_or("<unnamed>");
        format!("Project(module='{name}')")
    }
}

impl Project {
    /// Get-or-build the SVFG, caching the result for subsequent calls.
    fn get_or_build_svfg(&self) -> &Svfg {
        self.svfg_cache.get_or_init(|| {
            let mssa_pta = (*self.pta_result).clone();
            py_checkers::build_svfg_for_checker(
                &self.module,
                &self.callgraph,
                &self.defuse,
                &self.pta_result,
                mssa_pta,
            )
        })
    }

    /// Build a `ProgramDatabase` from the project's existing graphs.
    fn build_database(&self) -> saf_analysis::database::ProgramDatabase {
        saf_analysis::database::ProgramDatabase::from_parts(
            Arc::clone(&self.module),
            (*self.callgraph).clone(),
            saf_analysis::icfg::Icfg::build(&self.module, &self.callgraph),
            Some((*self.pta_result).clone()),
            (*self.defuse).clone(),
            (*self.valueflow).clone(),
            saf_analysis::pipeline::PipelineStats {
                defuse_build_secs: 0.0,
                pta_solve_secs: 0.0,
                refinement_iterations: 0,
                valueflow_build_secs: 0.0,
                total_secs: 0.0,
                constraint_counts: [0; 5],
                post_hvn_constraint_counts: [0; 5],
                pta_iterations: 0,
                constraint_diff_added: 0,
                constraint_diff_removed: 0,
                changed_module_count: 0,
            },
        )
    }
}

/// Parse a points-to set representation string.
fn parse_pts_repr(s: &str) -> PyResult<PtsRepresentation> {
    PtsRepresentation::from_str_opt(s).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown pts_repr '{s}'. Expected 'auto', 'btreeset', 'bitvector', or 'bdd'."
        ))
    })
}

/// Build all analysis graphs for a module using the unified pipeline.
fn build_analysis(
    module: &AirModule,
    vf_mode: ValueFlowMode,
    use_datalog: bool,
    pta_max_iterations: Option<usize>,
    field_sensitivity_depth: Option<u32>,
    max_refinement_iterations: Option<usize>,
) -> (CallGraph, DefUseGraph, PtaResult, ValueFlowGraph) {
    let field_sensitivity = if let Some(depth) = field_sensitivity_depth {
        if depth == 0 {
            FieldSensitivity::None
        } else {
            FieldSensitivity::StructFields { max_depth: depth }
        }
    } else {
        FieldSensitivity::StructFields { max_depth: 2 }
    };

    // Keep a clone for the Ascent solver path before field_sensitivity is moved into config
    let field_sensitivity_for_ascent = field_sensitivity.clone();

    let config = PipelineConfig {
        refinement: RefinementConfig {
            max_iterations: max_refinement_iterations.unwrap_or(10),
            pta_config: PtaConfig {
                field_sensitivity: field_sensitivity.clone(),
                max_iterations: pta_max_iterations.unwrap_or(10_000),
                ..PtaConfig::default()
            },
            field_sensitivity,
            ..RefinementConfig::default()
        },
        valueflow: ValueFlowConfig {
            mode: vf_mode,
            ..ValueFlowConfig::default()
        },
        specs: None,
        build_valueflow: true,
    };
    let result = run_pipeline(module, &config);

    // CG refinement always produces a PTA result; provide a defensive fallback
    // via expect — this path is unreachable in practice.
    let pta_result = result
        .pta_result
        .expect("CG refinement always produces a PTA result");

    // Optionally replace with Ascent Datalog solver result
    let pta_result = if use_datalog {
        let pta_config = PtaConfig {
            field_sensitivity: field_sensitivity_for_ascent,
            max_iterations: pta_max_iterations.unwrap_or(10_000),
            ..PtaConfig::default()
        };
        let ascent_result = saf_datalog::pta::analyze_with_ascent(module, &pta_config, None);
        PtaResult::new(
            ascent_result.pts,
            std::sync::Arc::new(ascent_result.factory),
            ascent_result.diagnostics,
        )
    } else {
        pta_result
    };

    (
        result.call_graph,
        result.defuse,
        pta_result,
        result.valueflow,
    )
}

// =============================================================================
// AnalysisSession — incremental analysis entry point
// =============================================================================

/// Incremental analysis session for multi-file projects.
///
/// Tracks analysis state across runs, caching per-module constraints
/// so that unchanged modules skip re-extraction on subsequent runs.
///
/// Example:
///     session = AnalysisSession(cache_dir=".saf-cache")
///     project = session.analyze(["src/main.ll", "src/parser.ll"])
///     q = project.query()
#[pyclass(name = "AnalysisSession")]
pub struct PyAnalysisSession {
    cache_dir: std::path::PathBuf,
    mode: String,
}

#[pymethods]
impl PyAnalysisSession {
    /// Create a new incremental analysis session.
    ///
    /// Args:
    ///     cache_dir: Directory for caching analysis state (default: ".saf-cache").
    ///     mode: Analysis mode — "sound" or "best-effort" (default: "best-effort").
    ///
    /// Returns:
    ///     An AnalysisSession instance.
    ///
    /// Raises:
    ///     ValueError: If mode is not "sound" or "best-effort".
    #[new]
    #[pyo3(signature = (*, cache_dir=".saf-cache", mode="best-effort"))]
    fn new(cache_dir: &str, mode: &str) -> PyResult<Self> {
        match mode {
            "sound" | "best-effort" => {}
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown mode '{other}'. Expected 'sound' or 'best-effort'."
                )));
            }
        }
        Ok(Self {
            cache_dir: std::path::PathBuf::from(cache_dir),
            mode: mode.to_string(),
        })
    }

    /// Run incremental analysis on the given input files.
    ///
    /// Each file is ingested via the appropriate frontend (selected by
    /// extension), linked into a multi-module program, and analyzed
    /// using the incremental pipeline. Unchanged modules (same content
    /// fingerprint) use cached constraints from previous runs.
    ///
    /// Args:
    ///     paths: List of input file paths (`.ll`, `.bc`, or `.air.json`).
    ///     vf_mode: ValueFlow mode — "fast" (default) or "precise".
    ///
    /// Returns:
    ///     A Project instance ready for querying.
    ///
    /// Raises:
    ///     FrontendError: If any input file cannot be parsed.
    ///     ValueError: If paths is empty.
    #[pyo3(signature = (paths, *, vf_mode="fast"))]
    #[allow(clippy::too_many_lines)] // PyO3 method with setup + analysis + result conversion
    fn analyze(&self, py: Python<'_>, paths: Vec<String>, vf_mode: &str) -> PyResult<Project> {
        use saf_analysis::pipeline::{PipelineConfig, run_pipeline_incremental};
        use saf_analysis::session::AnalysisSession;
        use saf_core::program::AirProgram;

        if paths.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "At least one input path is required.",
            ));
        }

        let vf_mode_enum = match vf_mode {
            "fast" => ValueFlowMode::Fast,
            "precise" => ValueFlowMode::Precise,
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown vf_mode '{other}'. Expected 'fast' or 'precise'."
                )));
            }
        };

        let saf_config = saf_core::config::Config::default();

        // Ingest each file as a separate bundle
        let mut bundles = Vec::with_capacity(paths.len());
        for path_str in &paths {
            let file_path = Path::new(path_str);
            if !file_path.exists() {
                return Err(frontend_error(
                    py,
                    codes::FRONTEND_IO_ERROR,
                    &format!("File not found: {path_str}"),
                    Some(path_str),
                ));
            }

            let bundle = if path_str.ends_with(".air.json") {
                AirJsonFrontend
                    .ingest(&[file_path], &saf_config)
                    .map_err(|e| {
                        frontend_error(
                            py,
                            codes::FRONTEND_PARSE_ERROR,
                            &format!("Failed to parse input: {e}"),
                            Some(path_str),
                        )
                    })?
            } else if path_str.to_ascii_lowercase().ends_with(".ll")
                || path_str.to_ascii_lowercase().ends_with(".bc")
            {
                #[cfg(any(feature = "llvm-18", feature = "llvm-22"))]
                {
                    LlvmFrontend::new()
                        .ingest(&[file_path], &saf_config)
                        .map_err(|e| {
                            frontend_error(
                                py,
                                codes::FRONTEND_PARSE_ERROR,
                                &format!("Failed to parse LLVM IR: {e}"),
                                Some(path_str),
                            )
                        })?
                }
                #[cfg(not(any(feature = "llvm-18", feature = "llvm-22")))]
                {
                    return Err(frontend_error(
                        py,
                        codes::FRONTEND_NOT_FOUND,
                        "LLVM support not compiled in. Rebuild with the llvm-18 or llvm-22 feature.",
                        Some(path_str),
                    ));
                }
            } else {
                return Err(frontend_error(
                    py,
                    codes::FRONTEND_NOT_FOUND,
                    &format!(
                        "Unsupported file extension for '{path_str}'. Expected .air.json, .ll, or .bc"
                    ),
                    Some(path_str),
                ));
            };
            bundles.push(bundle);
        }

        // Link into a multi-module program
        let program = AirProgram::link(bundles);

        // Load or create session
        let mut session = AnalysisSession::load(&self.cache_dir);

        // Configure pipeline
        let config = PipelineConfig {
            refinement: RefinementConfig {
                pta_config: PtaConfig {
                    field_sensitivity: FieldSensitivity::StructFields { max_depth: 2 },
                    ..PtaConfig::default()
                },
                ..RefinementConfig::default()
            },
            valueflow: ValueFlowConfig {
                mode: vf_mode_enum,
                ..ValueFlowConfig::default()
            },
            specs: None,
            build_valueflow: true,
        };

        // Run incremental pipeline
        let result = run_pipeline_incremental(&program, &config, &mut session);

        // Save session state
        if let Err(e) = session.save() {
            eprintln!("warning: failed to save analysis session: {e}");
        }

        let pta_result = result
            .pta_result
            .expect("CG refinement always produces a PTA result");

        let module = Arc::new(program.merged_view());

        Ok(Project {
            module,
            callgraph: Arc::new(result.call_graph),
            defuse: Arc::new(result.defuse),
            pta_result: Arc::new(pta_result),
            valueflow: Arc::new(result.valueflow),
            svfg_cache: OnceLock::new(),
        })
    }

    /// Clear the analysis cache.
    ///
    /// Removes all cached state from the cache directory. The next
    /// `analyze()` call will perform a full analysis from scratch.
    fn clean(&self) -> PyResult<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to clean cache: {e}"))
            })?;
        }
        Ok(())
    }

    /// Get the cache directory path.
    ///
    /// Returns:
    ///     str: The cache directory path.
    #[getter]
    fn cache_dir(&self) -> String {
        self.cache_dir.display().to_string()
    }

    /// Get the analysis mode.
    ///
    /// Returns:
    ///     str: The analysis mode ("sound" or "best-effort").
    #[getter]
    fn mode(&self) -> &str {
        &self.mode
    }

    fn __repr__(&self) -> String {
        format!(
            "AnalysisSession(cache_dir='{}', mode='{}')",
            self.cache_dir.display(),
            self.mode
        )
    }
}

/// Register project types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Project>()?;
    m.add_class::<PyAnalysisSession>()?;
    Ok(())
}
