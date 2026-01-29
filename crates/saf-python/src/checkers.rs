//! Python bindings for the checker framework.
//!
//! Exposes `Project.check()`, `Project.check_all()`, `Project.check_custom()`,
//! `PyCheckerFinding`, `PyResourceTable`, and related types.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use saf_analysis::PtaResult;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::checkers::{
    self, CheckerFinding, CheckerResult, CheckerSpec, PathSensitiveConfig, PathSensitiveResult,
    ReachabilityMode, ResourceRole, ResourceTable, Severity, SitePattern, SolverConfig,
};
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::{Svfg, SvfgBuilder};
use saf_core::air::AirModule;

// ---------------------------------------------------------------------------
// PyCheckerFinding
// ---------------------------------------------------------------------------

/// A finding from a checker.
///
/// Represents a source-to-sink reachability violation on the SVFG.
#[pyclass(name = "CheckerFinding")]
pub struct PyCheckerFinding {
    pub(crate) inner: CheckerFinding,
}

#[pymethods]
impl PyCheckerFinding {
    /// Checker name that produced this finding.
    #[getter]
    fn checker(&self) -> &str {
        &self.inner.checker_name
    }

    /// Finding severity (info, warning, error, critical).
    #[getter]
    fn severity(&self) -> &str {
        self.inner.severity.name()
    }

    /// CWE ID if applicable, or None.
    #[getter]
    fn cwe(&self) -> Option<u32> {
        self.inner.cwe
    }

    /// Human-readable message describing the finding.
    #[getter]
    fn message(&self) -> &str {
        &self.inner.message
    }

    /// Source SVFG node hex ID.
    #[getter]
    fn source(&self) -> String {
        self.inner.source_node.to_hex()
    }

    /// Sink SVFG node hex ID.
    #[getter]
    fn sink(&self) -> String {
        self.inner.sink_node.to_hex()
    }

    /// Trace (path) from source to sink as list of hex node IDs.
    #[getter]
    fn trace(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let ids: Vec<String> = self
            .inner
            .trace
            .iter()
            .map(saf_analysis::svfg::SvfgNodeId::to_hex)
            .collect();
        Ok(PyList::new(py, &ids)?.into())
    }

    /// Per-sink traces for `MultiReach` findings (e.g., double-free).
    ///
    /// Returns a list of dicts, each with ``"sink"`` (hex ID) and ``"trace"``
    /// (list of hex IDs from source to that sink). Empty for non-`MultiReach`.
    #[getter]
    fn sink_traces(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let list = PyList::empty(py);
        for (sink, trace) in &self.inner.sink_traces {
            let d = PyDict::new(py);
            d.set_item("sink", sink.to_hex())?;
            let trace_ids: Vec<String> = trace
                .iter()
                .map(saf_analysis::svfg::SvfgNodeId::to_hex)
                .collect();
            d.set_item("trace", trace_ids)?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    /// Convert to a dictionary.
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("checker", &self.inner.checker_name)?;
        dict.set_item("severity", self.inner.severity.name())?;
        dict.set_item("cwe", self.inner.cwe)?;
        dict.set_item("message", &self.inner.message)?;
        dict.set_item("source", self.inner.source_node.to_hex())?;
        dict.set_item("sink", self.inner.sink_node.to_hex())?;
        let trace: Vec<String> = self
            .inner
            .trace
            .iter()
            .map(saf_analysis::svfg::SvfgNodeId::to_hex)
            .collect();
        dict.set_item("trace", trace)?;
        let sink_traces = PyList::empty(py);
        for (sink, st) in &self.inner.sink_traces {
            let d = PyDict::new(py);
            d.set_item("sink", sink.to_hex())?;
            let ids: Vec<String> = st
                .iter()
                .map(saf_analysis::svfg::SvfgNodeId::to_hex)
                .collect();
            d.set_item("trace", ids)?;
            sink_traces.append(d)?;
        }
        dict.set_item("sink_traces", sink_traces)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "CheckerFinding(checker='{}', severity='{}', cwe={:?})",
            self.inner.checker_name,
            self.inner.severity.name(),
            self.inner.cwe
        )
    }
}

// ---------------------------------------------------------------------------
// PyResourceTable
// ---------------------------------------------------------------------------

/// Resource table mapping function names to resource management roles.
///
/// Ships with built-in entries for C stdlib, C++ operators, POSIX I/O,
/// pthreads. Users can add custom entries.
#[pyclass(name = "ResourceTable")]
pub struct PyResourceTable {
    pub(crate) inner: ResourceTable,
}

#[pymethods]
impl PyResourceTable {
    /// Add a custom resource entry.
    ///
    /// Args:
    ///     name: Function name.
    ///     role: Role string — one of "allocator", "deallocator", "reallocator",
    ///           "acquire", "release", "null_source", "dereference".
    fn add(&mut self, name: &str, role: &str) -> PyResult<()> {
        let r = parse_role(role)?;
        self.inner.add(name, r);
        Ok(())
    }

    /// Check if a function has a specific role.
    ///
    /// Args:
    ///     name: Function name.
    ///     role: Role string.
    ///
    /// Returns:
    ///     bool: True if the function has that role.
    fn has_role(&self, name: &str, role: &str) -> PyResult<bool> {
        let r = parse_role(role)?;
        Ok(self.inner.has_role(name, r))
    }

    /// Get all function names in the table.
    ///
    /// Returns:
    ///     list[str]: Sorted function names.
    fn function_names(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let names = self.inner.function_names();
        Ok(PyList::new(py, &names)?.into())
    }

    /// Get the number of entries.
    #[getter]
    fn size(&self) -> usize {
        self.inner.len()
    }

    /// Export the table as a list of dicts.
    ///
    /// Returns:
    ///     list[dict]: Each dict has "name" and "roles" keys.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let entries = self.inner.export();
        let list = PyList::empty(py);
        for entry in &entries {
            let d = PyDict::new(py);
            d.set_item("name", &entry.name)?;
            let roles: Vec<&str> = entry
                .roles
                .iter()
                .map(saf_analysis::checkers::ResourceRole::name)
                .collect();
            d.set_item("roles", roles)?;
            list.append(d)?;
        }
        Ok(list.into())
    }

    fn __repr__(&self) -> String {
        format!("ResourceTable(entries={})", self.inner.len())
    }
}

// ---------------------------------------------------------------------------
// PyPathSensitiveResult
// ---------------------------------------------------------------------------

/// Result of path-sensitive checker analysis.
///
/// Contains findings classified as feasible, infeasible, or unknown based on
/// Z3 SMT solver analysis of path conditions along SVFG traces.
///
/// - ``feasible``: Confirmed findings where the path is satisfiable.
/// - ``infeasible``: False positives where the path is provably impossible.
/// - ``unknown``: Findings where the solver timed out or couldn't decide.
#[pyclass(name = "PathSensitiveResult")]
pub struct PyPathSensitiveResult {
    pub(crate) inner: PathSensitiveResult,
}

#[pymethods]
impl PyPathSensitiveResult {
    /// Findings confirmed as feasible (real bugs).
    #[getter]
    fn feasible(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        findings_vec_to_py(py, &self.inner.feasible)
    }

    /// Findings proven infeasible (false positives filtered out).
    #[getter]
    fn infeasible(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        findings_vec_to_py(py, &self.inner.infeasible)
    }

    /// Findings where the solver couldn't decide (conservatively kept).
    #[getter]
    fn unknown(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        findings_vec_to_py(py, &self.inner.unknown)
    }

    /// Diagnostics dictionary with counters and statistics.
    #[getter]
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let d = &self.inner.diagnostics;
        let dict = PyDict::new(py);
        dict.set_item("total_findings", d.total_findings)?;
        dict.set_item("feasible_count", d.feasible_count)?;
        dict.set_item("infeasible_count", d.infeasible_count)?;
        dict.set_item("unknown_count", d.unknown_count)?;
        dict.set_item("guards_extracted", d.guards_extracted)?;
        dict.set_item("z3_calls", d.z3_calls)?;
        dict.set_item("z3_timeouts", d.z3_timeouts)?;
        dict.set_item("skipped_too_many_guards", d.skipped_too_many_guards)?;
        dict.set_item("joint_feasibility_filtered", d.joint_feasibility_filtered)?;
        Ok(dict.into())
    }

    /// Total number of findings (feasible + infeasible + unknown).
    #[getter]
    fn total(&self) -> usize {
        self.inner.diagnostics.total_findings
    }

    /// Number of false positives filtered out.
    #[getter]
    fn false_positives_filtered(&self) -> usize {
        self.inner.diagnostics.infeasible_count
    }

    fn __repr__(&self) -> String {
        let d = &self.inner.diagnostics;
        format!(
            "PathSensitiveResult(total={}, feasible={}, infeasible={}, unknown={})",
            d.total_findings, d.feasible_count, d.infeasible_count, d.unknown_count
        )
    }
}

/// Convert a `Vec<CheckerFinding>` reference to a Python list of `PyCheckerFinding`.
fn findings_vec_to_py(py: Python<'_>, findings: &[CheckerFinding]) -> PyResult<Py<PyList>> {
    let list = PyList::empty(py);
    for f in findings {
        let py_f = Py::new(py, PyCheckerFinding { inner: f.clone() })?;
        list.append(py_f)?;
    }
    Ok(list.into())
}

// ---------------------------------------------------------------------------
// Helper: build SVFG from project internals
// ---------------------------------------------------------------------------

pub(crate) fn build_svfg_for_checker(
    module: &AirModule,
    callgraph: &CallGraph,
    defuse: &DefUseGraph,
    pta: &PtaResult,
    mssa_pta: PtaResult,
) -> Svfg {
    let cfgs = crate::helpers::build_cfgs(module);

    let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, callgraph);
    let (svfg, _program_points) =
        SvfgBuilder::new(module, defuse, callgraph, pta, &mut mssa).build();
    svfg
}

// ---------------------------------------------------------------------------
// Project methods (added via check_*, checker_*, resource_table)
// ---------------------------------------------------------------------------

/// Run a named built-in checker or list of checkers using a pre-built SVFG.
#[allow(clippy::must_use_candidate)]
pub fn run_check_with_svfg(
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    names: Vec<String>,
) -> CheckerResult {
    let config = SolverConfig::default();

    let specs: Vec<CheckerSpec> = names
        .iter()
        .filter_map(|n| checkers::builtin_checker(n))
        .collect();

    checkers::run_checkers(&specs, module, svfg, table, &config)
}

/// Run all built-in checkers using a pre-built SVFG.
#[allow(clippy::must_use_candidate)]
pub fn run_check_all_with_svfg(
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
) -> CheckerResult {
    let config = SolverConfig::default();
    let specs = checkers::builtin_checkers();
    checkers::run_checkers(&specs, module, svfg, table, &config)
}

/// Run a custom checker using a pre-built SVFG.
#[allow(clippy::must_use_candidate)]
pub fn run_check_custom_with_svfg(
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    spec: CheckerSpec,
) -> CheckerResult {
    let config = SolverConfig::default();
    checkers::run_checkers(&[spec], module, svfg, table, &config)
}

/// Run named checkers with path-sensitive filtering using a pre-built SVFG.
#[allow(clippy::must_use_candidate)]
pub fn run_check_path_sensitive_with_svfg(
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    names: Vec<String>,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> PathSensitiveResult {
    let specs: Vec<CheckerSpec> = names
        .iter()
        .filter_map(|n| checkers::builtin_checker(n))
        .collect();
    let config = PathSensitiveConfig {
        z3_timeout_ms,
        max_guards_per_trace: max_guards,
        ..Default::default()
    };
    checkers::run_checkers_path_sensitive(&specs, module, svfg, table, &config)
}

/// Run all built-in checkers with path-sensitive filtering using a pre-built SVFG.
#[allow(clippy::must_use_candidate)]
pub fn run_check_all_path_sensitive_with_svfg(
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> PathSensitiveResult {
    let specs = checkers::builtin_checkers();
    let config = PathSensitiveConfig {
        z3_timeout_ms,
        max_guards_per_trace: max_guards,
        ..Default::default()
    };
    checkers::run_checkers_path_sensitive(&specs, module, svfg, table, &config)
}

/// Post-filter existing findings for path feasibility.
#[allow(clippy::must_use_candidate)]
pub fn run_filter_infeasible(
    findings: Vec<CheckerFinding>,
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> PathSensitiveResult {
    let config = PathSensitiveConfig {
        z3_timeout_ms,
        max_guards_per_trace: max_guards,
        ..Default::default()
    };
    checkers::filter_infeasible(&findings, module, &config)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_role(role: &str) -> PyResult<ResourceRole> {
    match role {
        "allocator" => Ok(ResourceRole::Allocator),
        "deallocator" => Ok(ResourceRole::Deallocator),
        "reallocator" => Ok(ResourceRole::Reallocator),
        "acquire" => Ok(ResourceRole::Acquire),
        "release" => Ok(ResourceRole::Release),
        "lock" => Ok(ResourceRole::Lock),
        "unlock" => Ok(ResourceRole::Unlock),
        "null_source" => Ok(ResourceRole::NullSource),
        "dereference" => Ok(ResourceRole::Dereference),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown role '{role}'. Expected: allocator, deallocator, reallocator, acquire, release, lock, unlock, null_source, dereference"
        ))),
    }
}

fn parse_reachability_mode(mode: &str) -> PyResult<ReachabilityMode> {
    match mode {
        "may_reach" => Ok(ReachabilityMode::MayReach),
        "must_not_reach" => Ok(ReachabilityMode::MustNotReach),
        "never_reach_sink" => Ok(ReachabilityMode::NeverReachSink),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown mode '{mode}'. Expected: may_reach, must_not_reach, never_reach_sink"
        ))),
    }
}

fn parse_severity(severity: &str) -> PyResult<Severity> {
    match severity {
        "info" => Ok(Severity::Info),
        "warning" => Ok(Severity::Warning),
        "error" => Ok(Severity::Error),
        "critical" => Ok(Severity::Critical),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown severity '{severity}'. Expected: info, warning, error, critical"
        ))),
    }
}

/// Convert findings to a Python list of `PyCheckerFinding`.
pub(crate) fn findings_to_py(
    py: Python<'_>,
    result: CheckerResult,
) -> PyResult<(Py<PyList>, Py<PyDict>)> {
    let list = PyList::empty(py);
    for f in result.findings {
        let py_f = Py::new(py, PyCheckerFinding { inner: f })?;
        list.append(py_f)?;
    }

    let diag = PyDict::new(py);
    diag.set_item("checkers_run", result.diagnostics.checkers_run)?;
    diag.set_item("classified_sites", result.diagnostics.classified_sites)?;
    diag.set_item("source_nodes", result.diagnostics.source_nodes)?;
    diag.set_item("sink_nodes", result.diagnostics.sink_nodes)?;
    diag.set_item("sanitizer_nodes", result.diagnostics.sanitizer_nodes)?;
    diag.set_item("total_findings", result.diagnostics.total_findings)?;

    Ok((list.into(), diag.into()))
}

/// Build a custom `CheckerSpec` from Python arguments.
// INVARIANT: PyO3 API requires each checker parameter as a separate argument
// for Python keyword-argument support.
#[allow(clippy::too_many_arguments)]
pub fn build_custom_spec(
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
) -> PyResult<CheckerSpec> {
    let sources = vec![SitePattern::Role {
        role: parse_role(source_role)?,
        match_return: source_match_return,
    }];

    let sinks = if sink_is_exit {
        vec![SitePattern::FunctionExit]
    } else if let Some(role_str) = sink_role {
        vec![SitePattern::Role {
            role: parse_role(role_str)?,
            match_return: false,
        }]
    } else {
        vec![SitePattern::AnyUseOf]
    };

    let sanitizers = if let Some(role_str) = sanitizer_role {
        vec![SitePattern::Role {
            role: parse_role(role_str)?,
            match_return: sanitizer_match_return,
        }]
    } else {
        vec![]
    };

    Ok(CheckerSpec {
        name: name.to_string(),
        description: format!("Custom checker: {name}"),
        cwe,
        severity: parse_severity(severity)?,
        mode: parse_reachability_mode(mode)?,
        sources,
        sinks,
        sanitizers,
    })
}

/// Register checker types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCheckerFinding>()?;
    m.add_class::<PyResourceTable>()?;
    m.add_class::<PyPathSensitiveResult>()?;

    // Role constants
    m.add("Allocator", "allocator")?;
    m.add("Deallocator", "deallocator")?;
    m.add("Reallocator", "reallocator")?;
    m.add("Acquire", "acquire")?;
    m.add("Release", "release")?;
    m.add("Lock", "lock")?;
    m.add("Unlock", "unlock")?;
    m.add("NullSource", "null_source")?;
    m.add("Dereference", "dereference")?;

    // Reachability modes
    m.add("MayReach", "may_reach")?;
    m.add("MustNotReach", "must_not_reach")?;

    // Severity levels
    m.add("Info", "info")?;
    m.add("Warning", "warning")?;
    m.add("Error", "error")?;
    m.add("Critical", "critical")?;

    Ok(())
}
