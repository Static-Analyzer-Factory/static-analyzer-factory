//! Python bindings for abstract interpretation.
//!
//! Provides `PyAbstractInterpResult`, `PyInterval`, `PyNumericFinding`,
//! and `PyNumericCheckResult` for Python access to the abstract
//! interpretation framework.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::id_parse::{parse_block_id, parse_inst_id, parse_value_id};
use saf_analysis::absint::{
    AbstractInterpConfig, AbstractInterpResult, Interval, NumericFinding, check_all_numeric,
    check_buffer_overflow, check_division_by_zero, check_integer_overflow, check_shift_count,
    solve_abstract_interp,
};
use saf_core::air::AirModule;

/// Python wrapper for an interval `[lo, hi]` with a given bit-width.
///
/// Represents a range of possible values at a program point.
///
/// Example:
///     interval = result.interval_at_block(block_id, value_id, 32)
///     print(interval.lo, interval.hi, interval.bits)
#[pyclass(name = "Interval")]
pub struct PyInterval {
    pub(crate) lo: i128,
    pub(crate) hi: i128,
    pub(crate) bits: u8,
    pub(crate) bottom: bool,
    pub(crate) top: bool,
}

#[pymethods]
impl PyInterval {
    /// Lower bound of the interval.
    #[getter]
    fn lo(&self) -> i128 {
        self.lo
    }

    /// Upper bound of the interval.
    #[getter]
    fn hi(&self) -> i128 {
        self.hi
    }

    /// Bit-width.
    #[getter]
    fn bits(&self) -> u8 {
        self.bits
    }

    /// Whether this interval is bottom (unreachable).
    #[getter]
    fn is_bottom(&self) -> bool {
        self.bottom
    }

    /// Whether this interval is top (unknown / full range).
    #[getter]
    fn is_top(&self) -> bool {
        self.top
    }

    fn __repr__(&self) -> String {
        if self.bottom {
            "Interval(⊥)".to_string()
        } else if self.top {
            format!("Interval(⊤, bits={})", self.bits)
        } else {
            format!(
                "Interval(lo={}, hi={}, bits={})",
                self.lo, self.hi, self.bits
            )
        }
    }
}

impl PyInterval {
    pub(crate) fn from_interval(iv: &Interval) -> Self {
        use saf_analysis::absint::AbstractDomain;
        Self {
            lo: iv.lo(),
            hi: iv.hi(),
            bits: iv.bits(),
            bottom: iv.is_bottom(),
            top: iv.is_top(),
        }
    }
}

/// Python wrapper for abstract interpretation results.
///
/// Provides access to computed invariants at block entries and
/// instruction points. Obtained via `Project.abstract_interp()`.
///
/// Example:
///     result = project.abstract_interp()
///     inv = result.invariant_at_block(block_id)
///     diag = result.diagnostics()
#[pyclass(name = "AbstractInterpResult")]
pub struct PyAbstractInterpResult {
    inner: AbstractInterpResult,
}

impl PyAbstractInterpResult {
    #[allow(clippy::must_use_candidate)]
    pub fn new(result: AbstractInterpResult) -> Self {
        Self { inner: result }
    }
}

#[pymethods]
impl PyAbstractInterpResult {
    /// Get all tracked value intervals at a block entry.
    ///
    /// Args:
    ///     block_hex: Block ID as hex string (e.g., "0x00000...").
    ///
    /// Returns:
    ///     dict[str, Interval]: Mapping from value hex ID to its interval.
    fn invariant_at_block(&self, py: Python<'_>, block_hex: &str) -> PyResult<Py<PyDict>> {
        let block_id = parse_block_id(block_hex)?;
        let invariants = self.inner.invariants_at_block(block_id);
        let dict = PyDict::new(py);
        for (vid, interval) in &invariants {
            dict.set_item(vid.to_hex(), PyInterval::from_interval(interval))?;
        }
        Ok(dict.into())
    }

    /// Get all tracked value intervals before an instruction.
    ///
    /// Args:
    ///     inst_hex: Instruction ID as hex string.
    ///
    /// Returns:
    ///     dict[str, Interval]: Mapping from value hex ID to its interval.
    fn invariant_at_inst(&self, py: Python<'_>, inst_hex: &str) -> PyResult<Py<PyDict>> {
        let inst_id = parse_inst_id(inst_hex)?;
        let invariants = self.inner.invariants_at_inst(inst_id);
        let dict = PyDict::new(py);
        for (vid, interval) in &invariants {
            dict.set_item(vid.to_hex(), PyInterval::from_interval(interval))?;
        }
        Ok(dict.into())
    }

    /// Get the interval for a specific value at a block entry.
    ///
    /// Args:
    ///     block_hex: Block ID as hex string.
    ///     value_hex: Value ID as hex string.
    ///     bits: Bit-width (default 32).
    ///
    /// Returns:
    ///     Interval: The value's interval at that block entry.
    #[pyo3(signature = (block_hex, value_hex, bits=32))]
    fn interval_at_block(
        &self,
        block_hex: &str,
        value_hex: &str,
        bits: u8,
    ) -> PyResult<PyInterval> {
        let block_id = parse_block_id(block_hex)?;
        let value_id = parse_value_id(value_hex)?;
        let iv = self.inner.interval_at_block(block_id, value_id, bits);
        Ok(PyInterval::from_interval(&iv))
    }

    /// Get the interval for a specific value before an instruction.
    ///
    /// Args:
    ///     inst_hex: Instruction ID as hex string.
    ///     value_hex: Value ID as hex string.
    ///     bits: Bit-width (default 32).
    ///
    /// Returns:
    ///     Interval: The value's interval at that instruction.
    #[pyo3(signature = (inst_hex, value_hex, bits=32))]
    fn interval_at_inst(&self, inst_hex: &str, value_hex: &str, bits: u8) -> PyResult<PyInterval> {
        let inst_id = parse_inst_id(inst_hex)?;
        let value_id = parse_value_id(value_hex)?;
        let iv = self.inner.interval_at_inst(inst_id, value_id, bits);
        Ok(PyInterval::from_interval(&iv))
    }

    /// Get analysis diagnostics.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys "blocks_analyzed", "widening_applications",
    ///           "narrowing_iterations_performed", "converged", "functions_analyzed".
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let diag = self.inner.diagnostics();
        let dict = PyDict::new(py);
        dict.set_item("blocks_analyzed", diag.blocks_analyzed)?;
        dict.set_item("widening_applications", diag.widening_applications)?;
        dict.set_item(
            "narrowing_iterations_performed",
            diag.narrowing_iterations_performed,
        )?;
        dict.set_item("converged", diag.converged)?;
        dict.set_item("functions_analyzed", diag.functions_analyzed)?;
        Ok(dict.into())
    }

    /// Number of blocks with computed states.
    #[getter]
    fn block_count(&self) -> usize {
        self.inner.block_count()
    }

    /// Number of instruction states.
    #[getter]
    fn inst_count(&self) -> usize {
        self.inner.inst_count()
    }

    /// Export the result to a dictionary.
    ///
    /// Returns:
    ///     dict: Full export with schema, block_count, inst_count,
    ///           diagnostics, and block_invariants.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let export = self.inner.export();
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        let diag = self.inner.diagnostics();
        format!(
            "AbstractInterpResult(blocks={}, insts={}, converged={})",
            self.inner.block_count(),
            self.inner.inst_count(),
            diag.converged,
        )
    }
}

/// Python wrapper for a numeric finding.
///
/// Represents a finding from numeric checkers (buffer overflow, integer overflow,
/// division by zero, shift count).
///
/// Example:
///     findings = project.check_numeric("buffer_overflow")
///     for f in findings:
///         print(f.checker, f.severity, f.description)
#[pyclass(name = "NumericFinding")]
pub struct PyNumericFinding {
    inner: NumericFinding,
}

#[pymethods]
impl PyNumericFinding {
    /// Checker name (e.g., "buffer_overflow", "integer_overflow").
    #[getter]
    fn checker(&self) -> &str {
        self.inner.checker.name()
    }

    /// Severity level ("safe", "warning", or "error").
    #[getter]
    fn severity(&self) -> &str {
        self.inner.severity.name()
    }

    /// CWE identifier.
    #[getter]
    fn cwe(&self) -> u32 {
        self.inner.cwe
    }

    /// Instruction ID (hex string) where the finding was detected.
    #[getter]
    fn inst_id(&self) -> &str {
        &self.inner.inst_id
    }

    /// Human-readable description of the finding.
    #[getter]
    fn description(&self) -> &str {
        &self.inner.description
    }

    /// Computed interval at the finding location (as string).
    #[getter]
    fn interval(&self) -> &str {
        &self.inner.interval
    }

    /// Function name where the finding occurs.
    #[getter]
    fn function(&self) -> &str {
        &self.inner.function
    }

    fn __repr__(&self) -> String {
        format!(
            "NumericFinding(checker='{}', severity='{}', function='{}', cwe={})",
            self.inner.checker.name(),
            self.inner.severity.name(),
            self.inner.function,
            self.inner.cwe,
        )
    }
}

/// Run abstract interpretation with custom config.
#[allow(clippy::must_use_candidate)]
pub fn run_abstract_interp_with_config(
    module: &AirModule,
    max_widening: Option<u32>,
    narrowing_iterations: Option<u32>,
    use_thresholds: Option<bool>,
) -> PyAbstractInterpResult {
    let mut config = AbstractInterpConfig::default();
    if let Some(mw) = max_widening {
        config.max_widening_iterations = mw;
    }
    if let Some(ni) = narrowing_iterations {
        config.narrowing_iterations = ni;
    }
    if let Some(ut) = use_thresholds {
        config.use_threshold_widening = ut;
    }
    let result = solve_abstract_interp(module, &config);
    PyAbstractInterpResult::new(result)
}

/// Run a named numeric checker.
pub fn run_check_numeric(module: &AirModule, name: &str) -> PyResult<Vec<PyNumericFinding>> {
    let config = AbstractInterpConfig::default();

    let check_result = match name {
        "buffer_overflow" => check_buffer_overflow(module, &config),
        "integer_overflow" => check_integer_overflow(module, &config),
        "division_by_zero" => check_division_by_zero(module, &config),
        "shift_count" => check_shift_count(module, &config),
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown numeric checker '{other}'. Available: 'buffer_overflow', 'integer_overflow', 'division_by_zero', 'shift_count'."
            )));
        }
    };

    Ok(check_result
        .findings
        .into_iter()
        .map(|f| PyNumericFinding { inner: f })
        .collect())
}

/// Run all numeric checkers.
#[allow(clippy::must_use_candidate)]
pub fn run_check_all_numeric(module: &AirModule) -> Vec<PyNumericFinding> {
    let config = AbstractInterpConfig::default();
    let check_result = check_all_numeric(module, &config);
    check_result
        .findings
        .into_iter()
        .map(|f| PyNumericFinding { inner: f })
        .collect()
}

/// Register abstract interpretation types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAbstractInterpResult>()?;
    m.add_class::<PyInterval>()?;
    m.add_class::<PyNumericFinding>()?;
    Ok(())
}
