//! Python bindings for Z3-enhanced analysis features.
//!
//! Provides Python wrappers for seven Z3-based analysis features:
//!
//! 1. **IFDS taint Z3 refinement** — filter IFDS taint witness paths via Z3.
//! 2. **ValueFlow taint Z3 refinement** — filter ValueFlow taint flows via Z3.
//! 3. **Typestate Z3 refinement** — filter typestate findings via Z3.
//! 4. **Numeric Z3 refinement** — filter numeric checker findings via Z3.
//! 5. **Assertion prover** — prove or disprove assertions via Z3.
//! 6. **Alias refinement** — refine may-alias queries via Z3.
//! 7. **Path reachability** — check block-to-block feasible reachability via Z3.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::Flow;
use saf_analysis::PtaResult;
use saf_analysis::TaintFlowZ3Result;
use saf_analysis::absint::NumericFinding;
use saf_analysis::absint::numeric_z3::NumericZ3Result;
use saf_analysis::ifds::TypestateFinding;
use saf_analysis::ifds::taint_z3::{TaintWitnessPath, TaintZ3Result};
use saf_analysis::ifds::typestate_z3::TypestateZ3Result;
use saf_analysis::z3_utils::{
    AliasRefinement, AliasRefinementResult, AssertionDiagnostics, AssertionFinding,
    AssertionResult, AssertionStatus, PathReachability, PathReachabilityResult,
    Z3FilterDiagnostics, check_path_reachable, prove_assertions, refine_alias,
};
use saf_core::air::AirModule;
use saf_core::ids::{BlockId, FunctionId, ValueId};

// ---------------------------------------------------------------------------
// Shared: Z3FilterDiagnostics → PyDict
// ---------------------------------------------------------------------------

/// Convert `Z3FilterDiagnostics` into a Python dictionary.
fn z3_filter_diagnostics_to_dict<'py>(
    py: Python<'py>,
    d: &Z3FilterDiagnostics,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("total_items", d.total_items)?;
    dict.set_item("feasible_count", d.feasible_count)?;
    dict.set_item("infeasible_count", d.infeasible_count)?;
    dict.set_item("unknown_count", d.unknown_count)?;
    dict.set_item("guards_extracted", d.guards_extracted)?;
    dict.set_item("z3_calls", d.z3_calls)?;
    dict.set_item("z3_timeouts", d.z3_timeouts)?;
    dict.set_item("skipped_too_many_guards", d.skipped_too_many_guards)?;
    Ok(dict)
}

/// Convert `AssertionDiagnostics` into a Python dictionary.
fn assertion_diagnostics_to_dict<'py>(
    py: Python<'py>,
    d: &AssertionDiagnostics,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("total_assertions", d.total_assertions)?;
    dict.set_item("proven_count", d.proven_count)?;
    dict.set_item("may_fail_count", d.may_fail_count)?;
    dict.set_item("unknown_count", d.unknown_count)?;
    dict.set_item("guards_extracted", d.guards_extracted)?;
    dict.set_item("z3_calls", d.z3_calls)?;
    Ok(dict)
}

// ===========================================================================
// Feature 1: IFDS Taint Z3 Refinement
// ===========================================================================

/// Result of Z3-based IFDS taint refinement.
///
/// After IFDS taint analysis finds potential source-to-sink flows, this
/// result classifies each witness path as feasible, infeasible, or unknown
/// based on Z3 satisfiability of branch guards along the path.
///
/// Example:
///     result = project.refine_ifds_taint_z3(ifds_result, sources, sinks)
///     print(result.feasible_count, "confirmed taint flows")
///     print(result.infeasible_count, "false positives removed")
#[pyclass(name = "TaintZ3Result")]
pub struct PyTaintZ3Result {
    inner: TaintZ3Result,
}

#[pymethods]
impl PyTaintZ3Result {
    /// Number of taint flows confirmed as feasible by Z3.
    #[getter]
    fn feasible_count(&self) -> usize {
        self.inner.feasible.len()
    }

    /// Number of taint flows proven infeasible (false positives).
    #[getter]
    fn infeasible_count(&self) -> usize {
        self.inner.infeasible.len()
    }

    /// Number of taint flows where Z3 timed out or couldn't decide.
    #[getter]
    fn unknown_count(&self) -> usize {
        self.inner.unknown.len()
    }

    /// Get the feasible witness paths as a list of dictionaries.
    ///
    /// Each dictionary contains:
    ///   - ``source_inst``: Source instruction ID (hex).
    ///   - ``sink_inst``: Sink instruction ID (hex).
    ///   - ``source_value``: Source value ID (hex).
    ///   - ``sink_value``: Sink value ID (hex).
    ///   - ``block_path``: List of ``(function_id, block_id)`` hex tuples.
    ///
    /// Returns:
    ///     list[dict]: Feasible witness paths.
    fn feasible(&self, py: Python<'_>) -> PyResult<PyObject> {
        witness_paths_to_py(py, &self.inner.feasible)
    }

    /// Get the infeasible witness paths as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Infeasible witness paths (false positives).
    fn infeasible(&self, py: Python<'_>) -> PyResult<PyObject> {
        witness_paths_to_py(py, &self.inner.infeasible)
    }

    /// Get the unknown witness paths as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Unknown witness paths (Z3 timeout).
    fn unknown(&self, py: Python<'_>) -> PyResult<PyObject> {
        witness_paths_to_py(py, &self.inner.unknown)
    }

    /// Get Z3 filtering diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys ``total_items``, ``feasible_count``,
    ///         ``infeasible_count``, ``unknown_count``, ``guards_extracted``,
    ///         ``z3_calls``, ``z3_timeouts``, ``skipped_too_many_guards``.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = z3_filter_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "TaintZ3Result(feasible={}, infeasible={}, unknown={})",
            self.inner.feasible.len(),
            self.inner.infeasible.len(),
            self.inner.unknown.len(),
        )
    }
}

/// Convert a slice of `TaintWitnessPath` to a Python list of dicts.
fn witness_paths_to_py(py: Python<'_>, paths: &[TaintWitnessPath]) -> PyResult<PyObject> {
    let list = pyo3::types::PyList::empty(py);
    for path in paths {
        let dict = PyDict::new(py);
        dict.set_item("source_inst", path.source_inst.to_hex())?;
        dict.set_item("sink_inst", path.sink_inst.to_hex())?;
        dict.set_item("source_value", path.source_value.to_hex())?;
        dict.set_item("sink_value", path.sink_value.to_hex())?;
        let block_path: Vec<(String, String)> = path
            .block_path
            .iter()
            .map(|(fid, bid)| (fid.to_hex(), bid.to_hex()))
            .collect();
        dict.set_item("block_path", block_path)?;
        list.append(dict)?;
    }
    Ok(list.into())
}

// ===========================================================================
// Feature 2: ValueFlow Taint Z3 Refinement
// ===========================================================================

/// Result of Z3-based ValueFlow taint refinement.
///
/// Classifies ValueFlow taint flows (from ``query.taint_flow()``) as
/// feasible, infeasible, or unknown using Z3 path feasibility checking.
///
/// Example:
///     result = project.refine_taint_flows_z3(flows)
///     print(result.feasible_count, "confirmed flows")
#[pyclass(name = "TaintFlowZ3Result")]
pub struct PyTaintFlowZ3Result {
    inner: TaintFlowZ3Result,
}

#[pymethods]
impl PyTaintFlowZ3Result {
    /// Number of taint flows confirmed as feasible by Z3.
    #[getter]
    fn feasible_count(&self) -> usize {
        self.inner.feasible.len()
    }

    /// Number of taint flows proven infeasible (false positives).
    #[getter]
    fn infeasible_count(&self) -> usize {
        self.inner.infeasible.len()
    }

    /// Number of taint flows where Z3 timed out or couldn't decide.
    #[getter]
    fn unknown_count(&self) -> usize {
        self.inner.unknown.len()
    }

    /// Get the feasible flows as a list of dictionaries.
    ///
    /// Each dictionary contains ``source`` and ``sink`` as hex value IDs.
    ///
    /// Returns:
    ///     list[dict]: Feasible taint flows.
    fn feasible(&self, py: Python<'_>) -> PyResult<PyObject> {
        flows_to_py(py, &self.inner.feasible)
    }

    /// Get the infeasible flows as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Infeasible taint flows (false positives).
    fn infeasible(&self, py: Python<'_>) -> PyResult<PyObject> {
        flows_to_py(py, &self.inner.infeasible)
    }

    /// Get the unknown flows as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Unknown taint flows (Z3 timeout).
    fn unknown(&self, py: Python<'_>) -> PyResult<PyObject> {
        flows_to_py(py, &self.inner.unknown)
    }

    /// Get Z3 filtering diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys ``total_items``, ``feasible_count``,
    ///         ``infeasible_count``, ``unknown_count``, ``guards_extracted``,
    ///         ``z3_calls``, ``z3_timeouts``, ``skipped_too_many_guards``.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = z3_filter_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "TaintFlowZ3Result(feasible={}, infeasible={}, unknown={})",
            self.inner.feasible.len(),
            self.inner.infeasible.len(),
            self.inner.unknown.len(),
        )
    }
}

/// Convert a slice of `Flow` to a Python list of dicts.
fn flows_to_py(py: Python<'_>, flows: &[Flow]) -> PyResult<PyObject> {
    let list = pyo3::types::PyList::empty(py);
    for flow in flows {
        let dict = PyDict::new(py);
        dict.set_item("source", flow.source.to_hex())?;
        dict.set_item("sink", flow.sink.to_hex())?;
        dict.set_item("trace_length", flow.trace.len())?;
        list.append(dict)?;
    }
    Ok(list.into())
}

// ===========================================================================
// Feature 3: Typestate Z3 Refinement
// ===========================================================================

/// Result of Z3-based typestate refinement.
///
/// Classifies typestate findings (error states, resource leaks) as
/// feasible, infeasible, or unknown using Z3 dominator-based guard
/// extraction.
///
/// Example:
///     result = project.refine_typestate_z3(findings)
///     print(result.feasible_count, "confirmed typestate violations")
#[pyclass(name = "TypestateZ3Result")]
pub struct PyTypestateZ3Result {
    inner: TypestateZ3Result,
}

#[pymethods]
impl PyTypestateZ3Result {
    /// Number of typestate findings confirmed as feasible by Z3.
    #[getter]
    fn feasible_count(&self) -> usize {
        self.inner.feasible.len()
    }

    /// Number of typestate findings proven infeasible (false positives).
    #[getter]
    fn infeasible_count(&self) -> usize {
        self.inner.infeasible.len()
    }

    /// Number of typestate findings where Z3 timed out or couldn't decide.
    #[getter]
    fn unknown_count(&self) -> usize {
        self.inner.unknown.len()
    }

    /// Get the feasible typestate findings as a list of dictionaries.
    ///
    /// Each dictionary contains ``resource``, ``state``, ``inst``, ``kind``,
    /// ``spec_name``, and ``location`` (as ``(function_id, block_id)`` hex tuple).
    ///
    /// Returns:
    ///     list[dict]: Feasible typestate findings.
    fn feasible(&self, py: Python<'_>) -> PyResult<PyObject> {
        typestate_findings_to_py(py, &self.inner.feasible)
    }

    /// Get the infeasible typestate findings as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Infeasible typestate findings (false positives).
    fn infeasible(&self, py: Python<'_>) -> PyResult<PyObject> {
        typestate_findings_to_py(py, &self.inner.infeasible)
    }

    /// Get the unknown typestate findings as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Unknown typestate findings (Z3 timeout).
    fn unknown(&self, py: Python<'_>) -> PyResult<PyObject> {
        typestate_findings_to_py(py, &self.inner.unknown)
    }

    /// Get Z3 filtering diagnostics.
    ///
    /// Returns:
    ///     dict: Z3 filter diagnostics.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = z3_filter_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "TypestateZ3Result(feasible={}, infeasible={}, unknown={})",
            self.inner.feasible.len(),
            self.inner.infeasible.len(),
            self.inner.unknown.len(),
        )
    }
}

/// Convert a slice of `TypestateFinding` to a Python list of dicts.
fn typestate_findings_to_py(py: Python<'_>, findings: &[TypestateFinding]) -> PyResult<PyObject> {
    let list = pyo3::types::PyList::empty(py);
    for f in findings {
        let dict = PyDict::new(py);
        dict.set_item("resource", f.resource.to_hex())?;
        dict.set_item("state", &f.state)?;
        dict.set_item("inst", f.inst.to_hex())?;
        let kind_str = match f.kind {
            saf_analysis::ifds::TypestateFindingKind::ErrorState => "error_state",
            saf_analysis::ifds::TypestateFindingKind::NonAcceptingAtExit => "non_accepting_at_exit",
        };
        dict.set_item("kind", kind_str)?;
        dict.set_item("spec_name", &f.spec_name)?;
        if let Some((func_id, block_id)) = f.location {
            dict.set_item("location", (func_id.to_hex(), block_id.to_hex()))?;
        } else {
            dict.set_item("location", py.None())?;
        }
        list.append(dict)?;
    }
    Ok(list.into())
}

// ===========================================================================
// Feature 4: Numeric Z3 Refinement
// ===========================================================================

/// Result of Z3-based numeric checker refinement.
///
/// Classifies numeric findings (buffer overflow, integer overflow) as
/// confirmed, refuted, or uncertain using Z3 dominator-based guard
/// extraction.
///
/// Example:
///     result = project.refine_numeric_z3(findings)
///     print(result.confirmed_count, "confirmed overflows")
///     print(result.refuted_count, "false positives from widening")
#[pyclass(name = "NumericZ3Result")]
pub struct PyNumericZ3Result {
    inner: NumericZ3Result,
}

#[pymethods]
impl PyNumericZ3Result {
    /// Number of numeric findings confirmed by Z3 (overflow is feasible).
    #[getter]
    fn confirmed_count(&self) -> usize {
        self.inner.confirmed.len()
    }

    /// Number of numeric findings refuted by Z3 (false positive from widening).
    #[getter]
    fn refuted_count(&self) -> usize {
        self.inner.refuted.len()
    }

    /// Number of numeric findings where Z3 timed out.
    #[getter]
    fn uncertain_count(&self) -> usize {
        self.inner.uncertain.len()
    }

    /// Get the confirmed findings as a list of dictionaries.
    ///
    /// Each dictionary contains ``checker``, ``severity``, ``cwe``,
    /// ``inst_id``, ``description``, ``interval``, and ``function``.
    ///
    /// Returns:
    ///     list[dict]: Confirmed numeric findings.
    fn confirmed(&self, py: Python<'_>) -> PyResult<PyObject> {
        numeric_findings_to_py(py, &self.inner.confirmed)
    }

    /// Get the refuted findings as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Refuted numeric findings (false positives).
    fn refuted(&self, py: Python<'_>) -> PyResult<PyObject> {
        numeric_findings_to_py(py, &self.inner.refuted)
    }

    /// Get the uncertain findings as a list of dictionaries.
    ///
    /// Returns:
    ///     list[dict]: Uncertain numeric findings (Z3 timeout).
    fn uncertain(&self, py: Python<'_>) -> PyResult<PyObject> {
        numeric_findings_to_py(py, &self.inner.uncertain)
    }

    /// Get Z3 filtering diagnostics.
    ///
    /// Returns:
    ///     dict: Z3 filter diagnostics.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = z3_filter_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "NumericZ3Result(confirmed={}, refuted={}, uncertain={})",
            self.inner.confirmed.len(),
            self.inner.refuted.len(),
            self.inner.uncertain.len(),
        )
    }
}

/// Convert a slice of `NumericFinding` to a Python list of dicts.
fn numeric_findings_to_py(py: Python<'_>, findings: &[NumericFinding]) -> PyResult<PyObject> {
    let list = pyo3::types::PyList::empty(py);
    for f in findings {
        let dict = PyDict::new(py);
        dict.set_item("checker", f.checker.name())?;
        dict.set_item("severity", f.severity.name())?;
        dict.set_item("cwe", f.cwe)?;
        dict.set_item("inst_id", &f.inst_id)?;
        dict.set_item("description", &f.description)?;
        dict.set_item("interval", &f.interval)?;
        dict.set_item("function", &f.function)?;
        let (func_id, block_id) = f.location;
        dict.set_item("location", (func_id.to_hex(), block_id.to_hex()))?;
        list.append(dict)?;
    }
    Ok(list.into())
}

// ===========================================================================
// Feature 5: Assertion Prover
// ===========================================================================

/// Result of Z3-based assertion proving.
///
/// Classifies assertions as proven (always hold), may-fail (counterexample
/// found), or unknown (Z3 timeout).
///
/// Example:
///     result = project.prove_assertions()
///     print(result.proven_count, "assertions proven")
///     for f in result.may_fail():
///         print(f"WARN: {f.condition_desc} may fail in {f.function}")
#[pyclass(name = "AssertionResult")]
pub struct PyAssertionResult {
    inner: AssertionResult,
}

#[pymethods]
impl PyAssertionResult {
    /// Number of assertions proven to always hold.
    #[getter]
    fn proven_count(&self) -> usize {
        self.inner.proven.len()
    }

    /// Number of assertions that may fail (counterexample found).
    #[getter]
    fn may_fail_count(&self) -> usize {
        self.inner.may_fail.len()
    }

    /// Number of assertions where Z3 timed out.
    #[getter]
    fn unknown_count(&self) -> usize {
        self.inner.unknown.len()
    }

    /// Get the proven assertion findings as a list of `AssertionFinding`.
    ///
    /// Returns:
    ///     list[AssertionFinding]: Proven assertions.
    fn proven(&self) -> Vec<PyAssertionFinding> {
        self.inner
            .proven
            .iter()
            .map(|f| PyAssertionFinding { inner: f.clone() })
            .collect()
    }

    /// Get the may-fail assertion findings as a list of `AssertionFinding`.
    ///
    /// Returns:
    ///     list[AssertionFinding]: Assertions that may fail.
    fn may_fail(&self) -> Vec<PyAssertionFinding> {
        self.inner
            .may_fail
            .iter()
            .map(|f| PyAssertionFinding { inner: f.clone() })
            .collect()
    }

    /// Get the unknown assertion findings as a list of `AssertionFinding`.
    ///
    /// Returns:
    ///     list[AssertionFinding]: Assertions where Z3 timed out.
    fn unknown(&self) -> Vec<PyAssertionFinding> {
        self.inner
            .unknown
            .iter()
            .map(|f| PyAssertionFinding { inner: f.clone() })
            .collect()
    }

    /// Get assertion prover diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys ``total_assertions``, ``proven_count``,
    ///         ``may_fail_count``, ``unknown_count``, ``guards_extracted``,
    ///         ``z3_calls``.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = assertion_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "AssertionResult(proven={}, may_fail={}, unknown={})",
            self.inner.proven.len(),
            self.inner.may_fail.len(),
            self.inner.unknown.len(),
        )
    }
}

/// A finding about an individual assertion.
///
/// Contains the function, instruction, condition description, proof status,
/// and an optional counterexample for ``MayFail`` cases.
#[pyclass(name = "AssertionFinding")]
#[derive(Clone)]
pub struct PyAssertionFinding {
    inner: AssertionFinding,
}

#[pymethods]
impl PyAssertionFinding {
    /// Function name containing the assertion.
    #[getter]
    fn function(&self) -> &str {
        &self.inner.function
    }

    /// Function ID as a hex string.
    #[getter]
    fn function_id(&self) -> String {
        self.inner.function_id.to_hex()
    }

    /// Instruction ID of the assert call, as a hex string.
    #[getter]
    fn inst(&self) -> String {
        self.inner.inst.to_hex()
    }

    /// Human-readable description of the assertion condition.
    #[getter]
    fn condition_desc(&self) -> &str {
        &self.inner.condition_desc
    }

    /// Proof status: ``"proven"``, ``"may_fail"``, or ``"unknown"``.
    #[getter]
    fn status(&self) -> &str {
        match self.inner.status {
            AssertionStatus::Proven => "proven",
            AssertionStatus::MayFail => "may_fail",
            AssertionStatus::Unknown => "unknown",
        }
    }

    /// Counterexample for `MayFail` cases, or ``None``.
    ///
    /// Returns:
    ///     dict[str, int] | None: Variable name to value mapping, if available.
    fn counterexample(&self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        match &self.inner.counterexample {
            Some(ce) => {
                let dict = PyDict::new(py);
                for (name, value) in ce {
                    dict.set_item(name, *value)?;
                }
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Convert to a dictionary.
    ///
    /// Returns:
    ///     dict: All finding fields.
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("function", &self.inner.function)?;
        dict.set_item("function_id", self.inner.function_id.to_hex())?;
        dict.set_item("inst", self.inner.inst.to_hex())?;
        dict.set_item("condition_desc", &self.inner.condition_desc)?;
        dict.set_item("status", self.status())?;
        if let Some(ce) = &self.inner.counterexample {
            let ce_dict = PyDict::new(py);
            for (name, value) in ce {
                ce_dict.set_item(name, *value)?;
            }
            dict.set_item("counterexample", ce_dict)?;
        } else {
            dict.set_item("counterexample", py.None())?;
        }
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "AssertionFinding(function='{}', status='{}', condition='{}')",
            self.inner.function,
            self.status(),
            self.inner.condition_desc,
        )
    }
}

// ===========================================================================
// Feature 6: Alias Refinement
// ===========================================================================

/// Result of Z3-based alias refinement.
///
/// Refines a may-alias query by encoding dominating path constraints.
/// The result is ``"confirmed_alias"``, ``"no_alias"``, or ``"unknown"``.
///
/// Example:
///     result = project.refine_alias(p, q, block, func)
///     print(result.verdict)  # "confirmed_alias", "no_alias", or "unknown"
#[pyclass(name = "AliasRefinementResult")]
pub struct PyAliasRefinementResult {
    inner: AliasRefinementResult,
}

#[pymethods]
impl PyAliasRefinementResult {
    /// The refinement verdict: ``"confirmed_alias"``, ``"no_alias"``, or ``"unknown"``.
    #[getter]
    fn verdict(&self) -> &str {
        match self.inner.result {
            AliasRefinement::ConfirmedAlias => "confirmed_alias",
            AliasRefinement::NoAlias => "no_alias",
            AliasRefinement::Unknown => "unknown",
        }
    }

    /// Whether the alias is confirmed feasible.
    #[getter]
    fn is_alias(&self) -> bool {
        self.inner.result == AliasRefinement::ConfirmedAlias
    }

    /// Whether the alias is proven infeasible.
    #[getter]
    fn is_no_alias(&self) -> bool {
        self.inner.result == AliasRefinement::NoAlias
    }

    /// Whether the result is unknown (Z3 timeout).
    #[getter]
    fn is_unknown(&self) -> bool {
        self.inner.result == AliasRefinement::Unknown
    }

    /// Get Z3 filtering diagnostics.
    ///
    /// Returns:
    ///     dict: Z3 filter diagnostics.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = z3_filter_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!("AliasRefinementResult(verdict='{}')", self.verdict())
    }
}

// ===========================================================================
// Feature 7: Path Reachability
// ===========================================================================

/// Result of Z3-based path-reachability checking.
///
/// Determines whether a feasible execution path exists between two blocks
/// by enumerating CFG paths and checking Z3 guard feasibility.
///
/// Example:
///     result = project.check_path_reachable(from_block, to_block, func_id)
///     if result.is_reachable:
///         print("Reachable via", result.witness_path)
#[pyclass(name = "PathReachabilityResult")]
pub struct PyPathReachabilityResult {
    inner: PathReachabilityResult,
}

#[pymethods]
impl PyPathReachabilityResult {
    /// The reachability verdict: ``"reachable"``, ``"unreachable"``, or ``"unknown"``.
    #[getter]
    fn verdict(&self) -> &str {
        match &self.inner.result {
            PathReachability::Reachable(_) => "reachable",
            PathReachability::Unreachable => "unreachable",
            PathReachability::Unknown => "unknown",
        }
    }

    /// Whether a feasible path was found.
    #[getter]
    fn is_reachable(&self) -> bool {
        matches!(self.inner.result, PathReachability::Reachable(_))
    }

    /// Whether the target is provably unreachable.
    #[getter]
    fn is_unreachable(&self) -> bool {
        matches!(self.inner.result, PathReachability::Unreachable)
    }

    /// Whether the result is unknown (Z3 timeout or max paths exceeded).
    #[getter]
    fn is_unknown(&self) -> bool {
        matches!(self.inner.result, PathReachability::Unknown)
    }

    /// Get the witness path (if reachable) as a list of block ID hex strings.
    ///
    /// Returns:
    ///     list[str] | None: Block IDs of the witness path, or ``None``.
    fn witness_path(&self) -> Option<Vec<String>> {
        match &self.inner.result {
            PathReachability::Reachable(blocks) => {
                Some(blocks.iter().map(|b| b.to_hex()).collect())
            }
            _ => None,
        }
    }

    /// Number of paths checked by the solver.
    #[getter]
    fn paths_checked(&self) -> usize {
        self.inner.paths_checked
    }

    /// Get Z3 filtering diagnostics.
    ///
    /// Returns:
    ///     dict: Z3 filter diagnostics.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = z3_filter_diagnostics_to_dict(py, &self.inner.diagnostics)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "PathReachabilityResult(verdict='{}', paths_checked={})",
            self.verdict(),
            self.inner.paths_checked,
        )
    }
}

// ===========================================================================
// Public API functions (called from project.rs or standalone)
// ===========================================================================

/// Run the assertion prover.
///
/// Scans for assert-like calls and uses Z3 to prove or disprove them.
#[allow(clippy::must_use_candidate)]
pub fn run_prove_assertions(
    module: &Arc<AirModule>,
    absint_result: Option<&saf_analysis::absint::AbstractInterpResult>,
    z3_timeout_ms: u64,
    max_guards: usize,
    assert_functions: &[String],
) -> PyAssertionResult {
    let result = prove_assertions(
        module,
        absint_result,
        z3_timeout_ms,
        max_guards,
        assert_functions,
    );
    PyAssertionResult { inner: result }
}

/// Run alias refinement.
///
/// Refines a may-alias query between two pointers at a specific program point.
#[allow(
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::must_use_candidate
)]
pub fn run_refine_alias(
    module: &Arc<AirModule>,
    pta: &PtaResult,
    p: ValueId,
    q: ValueId,
    at_block: BlockId,
    func_id: FunctionId,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> PyAliasRefinementResult {
    let result = refine_alias(
        p,
        q,
        at_block,
        func_id,
        module,
        pta,
        z3_timeout_ms,
        max_guards,
    );
    PyAliasRefinementResult { inner: result }
}

/// Run path reachability checking.
///
/// Checks whether a feasible execution path exists between two blocks.
#[allow(clippy::too_many_arguments, clippy::must_use_candidate)]
pub fn run_check_path_reachable(
    module: &Arc<AirModule>,
    from_block: BlockId,
    to_block: BlockId,
    func_id: FunctionId,
    z3_timeout_ms: u64,
    max_guards: usize,
    max_paths: usize,
) -> PyPathReachabilityResult {
    let result = check_path_reachable(
        from_block,
        to_block,
        func_id,
        module,
        z3_timeout_ms,
        max_guards,
        max_paths,
    );
    PyPathReachabilityResult { inner: result }
}

// ===========================================================================
// Module registration
// ===========================================================================

/// Register all Z3 refinement pyclasses with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Feature 1: IFDS taint Z3
    m.add_class::<PyTaintZ3Result>()?;
    // Feature 2: ValueFlow taint Z3
    m.add_class::<PyTaintFlowZ3Result>()?;
    // Feature 3: Typestate Z3
    m.add_class::<PyTypestateZ3Result>()?;
    // Feature 4: Numeric Z3
    m.add_class::<PyNumericZ3Result>()?;
    // Feature 5: Assertion prover
    m.add_class::<PyAssertionResult>()?;
    m.add_class::<PyAssertionFinding>()?;
    // Feature 6: Alias refinement
    m.add_class::<PyAliasRefinementResult>()?;
    // Feature 7: Path reachability
    m.add_class::<PyPathReachabilityResult>()?;
    Ok(())
}
