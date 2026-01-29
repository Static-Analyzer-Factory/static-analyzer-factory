//! Python bindings for the IDE framework and typestate analysis.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::ifds::{
    IfdsConfig, TypestateFinding, TypestateFindingKind, TypestateIdeProblem, TypestateSpec,
    TypestateTransition, builtin_typestate_spec, solve_ide,
};
use saf_core::air::AirModule;

/// A single typestate finding (violation).
#[pyclass(name = "TypestateFinding")]
#[derive(Clone)]
pub struct PyTypestateFinding {
    finding: TypestateFinding,
}

#[pymethods]
impl PyTypestateFinding {
    /// The resource value ID as a hex string.
    #[getter]
    fn resource(&self) -> String {
        self.finding.resource.to_hex()
    }

    /// The state the resource is in (e.g., "error", "opened", "locked").
    #[getter]
    fn state(&self) -> &str {
        &self.finding.state
    }

    /// The instruction ID where the finding was detected, as hex string.
    #[getter]
    fn inst(&self) -> String {
        self.finding.inst.to_hex()
    }

    /// The kind of finding: "error_state" or "non_accepting_at_exit".
    #[getter]
    fn kind(&self) -> &str {
        match self.finding.kind {
            TypestateFindingKind::ErrorState => "error_state",
            TypestateFindingKind::NonAcceptingAtExit => "non_accepting_at_exit",
        }
    }

    /// The spec that produced this finding (e.g., "file_io").
    #[getter]
    fn spec_name(&self) -> &str {
        &self.finding.spec_name
    }

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("resource", self.resource())?;
        d.set_item("state", self.state())?;
        d.set_item("inst", self.inst())?;
        d.set_item("kind", self.kind())?;
        d.set_item("spec_name", self.spec_name())?;
        Ok(d)
    }

    fn __repr__(&self) -> String {
        format!(
            "TypestateFinding(spec={}, kind={}, state={}, resource={})",
            self.finding.spec_name,
            self.kind(),
            self.finding.state,
            self.finding.resource.to_hex(),
        )
    }
}

/// Result of a typestate analysis.
#[pyclass(name = "TypestateResult")]
pub struct PyTypestateResult {
    findings: Vec<TypestateFinding>,
    diagnostics: saf_analysis::ifds::IdeDiagnostics,
}

#[pymethods]
impl PyTypestateResult {
    /// Get all findings.
    ///
    /// Returns:
    ///     list[TypestateFinding]: All typestate violations detected.
    fn findings(&self) -> Vec<PyTypestateFinding> {
        self.findings
            .iter()
            .map(|f| PyTypestateFinding { finding: f.clone() })
            .collect()
    }

    /// Get only error-state findings (e.g., double-close, use-after-close).
    ///
    /// Returns:
    ///     list[TypestateFinding]: Findings where the resource reached an error state.
    fn error_findings(&self) -> Vec<PyTypestateFinding> {
        self.findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::ErrorState)
            .map(|f| PyTypestateFinding { finding: f.clone() })
            .collect()
    }

    /// Get only non-accepting-at-exit findings (e.g., file leak, held lock).
    ///
    /// Returns:
    ///     list[TypestateFinding]: Findings where a resource was not in an
    ///         accepting state at function exit.
    fn leak_findings(&self) -> Vec<PyTypestateFinding> {
        self.findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::NonAcceptingAtExit)
            .map(|f| PyTypestateFinding { finding: f.clone() })
            .collect()
    }

    /// Check if any violations were found.
    ///
    /// Returns:
    ///     bool: True if there are any findings.
    fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Get the number of findings.
    fn __len__(&self) -> usize {
        self.findings.len()
    }

    /// Get solver diagnostics.
    ///
    /// Returns:
    ///     dict: IDE solver diagnostics (jump_fn_updates, value_propagations, etc.).
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("jump_fn_updates", self.diagnostics.jump_fn_updates)?;
        d.set_item("value_propagations", self.diagnostics.value_propagations)?;
        d.set_item("jump_fn_entries", self.diagnostics.jump_fn_entries)?;
        Ok(d)
    }

    fn __repr__(&self) -> String {
        let errors = self
            .findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::ErrorState)
            .count();
        let leaks = self
            .findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::NonAcceptingAtExit)
            .count();
        format!(
            "TypestateResult(findings={}, errors={}, leaks={})",
            self.findings.len(),
            errors,
            leaks,
        )
    }
}

/// A typestate specification (for custom specs).
#[pyclass(name = "TypestateSpec")]
#[derive(Clone)]
pub struct PyTypestateSpec {
    inner: TypestateSpec,
}

#[pymethods]
impl PyTypestateSpec {
    /// Create a new typestate specification.
    ///
    /// Args:
    ///     name: Name of the checker.
    ///     states: List of state names.
    ///     initial_state: The initial state for new resources.
    ///     error_states: States indicating a bug.
    ///     accepting_states: States acceptable at program exit.
    ///     transitions: List of (from_state, call_name, to_state) tuples.
    ///     constructors: Function names that create resources.
    #[new]
    #[pyo3(signature = (name, states, initial_state, error_states, accepting_states, transitions, constructors))]
    // INVARIANT: PyO3 constructor requires each typestate spec field as a
    // separate argument for Python keyword-argument support.
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: String,
        states: Vec<String>,
        initial_state: String,
        error_states: Vec<String>,
        accepting_states: Vec<String>,
        transitions: Vec<(String, String, String)>,
        constructors: Vec<String>,
    ) -> PyResult<Self> {
        let transitions: Vec<TypestateTransition> = transitions
            .into_iter()
            .map(|(from, call, to)| TypestateTransition { from, call, to })
            .collect();

        let spec = TypestateSpec {
            name,
            states,
            initial_state,
            error_states,
            accepting_states,
            transitions,
            constructors,
        };

        if let Err(e) = spec.validate() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid typestate spec: {e}"
            )));
        }

        Ok(Self { inner: spec })
    }

    fn __repr__(&self) -> String {
        format!(
            "TypestateSpec(name={}, states={}, transitions={})",
            self.inner.name,
            self.inner.states.len(),
            self.inner.transitions.len(),
        )
    }
}

/// Run typestate analysis with a built-in spec.
///
/// This is called from `Project.typestate()`.
#[allow(clippy::must_use_candidate)]
pub fn run_typestate_builtin(
    module: &Arc<AirModule>,
    callgraph: &CallGraph,
    spec_name: &str,
) -> PyResult<PyTypestateResult> {
    let spec = builtin_typestate_spec(spec_name).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown typestate spec: '{spec_name}'. Available: file_io, mutex_lock, memory_alloc"
        ))
    })?;

    run_typestate_with_spec(module, callgraph, spec)
}

/// Run typestate analysis with a custom spec.
///
/// This is called from `Project.typestate_custom()`.
#[allow(clippy::must_use_candidate)]
pub fn run_typestate_custom(
    module: &Arc<AirModule>,
    callgraph: &CallGraph,
    spec: &PyTypestateSpec,
) -> PyResult<PyTypestateResult> {
    run_typestate_with_spec(module, callgraph, spec.inner.clone())
}

fn run_typestate_with_spec(
    module: &Arc<AirModule>,
    callgraph: &CallGraph,
    spec: TypestateSpec,
) -> PyResult<PyTypestateResult> {
    let problem = TypestateIdeProblem::new(module, spec);
    let icfg = Icfg::build(module, callgraph);
    let config = IfdsConfig::default();
    let result = solve_ide(&problem, &icfg, callgraph, &config);
    let diagnostics = result.diagnostics.clone();
    let findings = problem.collect_findings(&result);

    Ok(PyTypestateResult {
        findings,
        diagnostics,
    })
}

/// Get available built-in typestate spec names.
#[pyfunction]
pub fn typestate_specs() -> Vec<String> {
    vec![
        "file_io".to_string(),
        "mutex_lock".to_string(),
        "memory_alloc".to_string(),
    ]
}

/// Register IDE/typestate types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTypestateResult>()?;
    m.add_class::<PyTypestateFinding>()?;
    m.add_class::<PyTypestateSpec>()?;
    m.add_function(pyo3::wrap_pyfunction!(typestate_specs, m)?)?;
    Ok(())
}
