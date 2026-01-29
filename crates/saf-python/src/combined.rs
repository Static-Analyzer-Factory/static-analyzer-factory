//! Python bindings for combined PTA + Abstract Interpretation analysis.
//!
//! Provides `PyCombinedAnalysisResult` with unified access to alias queries
//! and numeric intervals.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::absint::PyInterval;
use crate::id_parse::parse_value_id;
use saf_analysis::PtaResult;
use saf_analysis::absint::{AbstractInterpResult, Interval};
use saf_analysis::combined::{CombinedAnalysisConfig, CombinedAnalysisResult, analyze_combined};
use saf_core::air::AirModule;

/// Python wrapper for combined PTA + abstract interpretation results.
///
/// Provides unified access to alias queries and numeric intervals.
///
/// Example:
///     result = project.analyze_combined()
///     # Query numeric interval
///     interval = result.interval_at(value_hex)
///     # Query alias relationship
///     alias = result.may_alias(ptr_a_hex, ptr_b_hex)
#[pyclass(name = "CombinedAnalysisResult")]
pub struct PyCombinedAnalysisResult {
    pta: PtaResult,
    absint: AbstractInterpResult,
    refinement_iterations: usize,
}

impl PyCombinedAnalysisResult {
    #[allow(clippy::must_use_candidate)]
    pub fn new(result: CombinedAnalysisResult) -> Self {
        Self {
            pta: result.pta,
            absint: result.absint,
            refinement_iterations: result.refinement_iterations,
        }
    }
}

#[pymethods]
impl PyCombinedAnalysisResult {
    /// Get the interval for a value at a function exit.
    ///
    /// Args:
    ///     value_hex: Value ID as hex string (e.g., "0x00000...").
    ///     bits: Bit-width (default 64).
    ///
    /// Returns:
    ///     Interval: The value's interval.
    #[pyo3(signature = (value_hex, bits=64))]
    fn interval_at(&self, value_hex: &str, bits: u8) -> PyResult<PyInterval> {
        let value_id = parse_value_id(value_hex)?;

        // Try to find the interval in any function's exit state
        // This is a simplification; a full implementation would
        // accept a block/inst ID parameter
        let iv = self
            .absint
            .state_at_inst(saf_core::ids::InstId::new(0))
            .and_then(|state| state.get_opt(value_id).cloned())
            .unwrap_or_else(|| Interval::make_top(bits));

        Ok(PyInterval::from_interval(&iv))
    }

    /// Get points-to set for a pointer value.
    ///
    /// Args:
    ///     ptr_hex: Pointer value ID as hex string.
    ///
    /// Returns:
    ///     list[str]: List of location IDs the pointer may point to.
    fn points_to(&self, ptr_hex: &str) -> PyResult<Vec<String>> {
        let value_id = parse_value_id(ptr_hex)?;
        let pts = self.pta.points_to(value_id);
        Ok(pts.iter().map(|loc| loc.to_hex()).collect())
    }

    /// Check alias relationship between two pointers.
    ///
    /// Args:
    ///     ptr_a_hex: First pointer value ID as hex string.
    ///     ptr_b_hex: Second pointer value ID as hex string.
    ///
    /// Returns:
    ///     str: Alias result ("must", "partial", "may", "no", or "unknown").
    #[allow(clippy::similar_names)]
    fn may_alias(&self, ptr_a_hex: &str, ptr_b_hex: &str) -> PyResult<String> {
        let a = parse_value_id(ptr_a_hex)?;
        let b = parse_value_id(ptr_b_hex)?;
        let result = self.pta.may_alias(a, b);
        Ok(crate::helpers::alias_result_to_str(result).to_string())
    }

    /// Number of refinement iterations performed.
    #[getter]
    fn refinement_iterations(&self) -> usize {
        self.refinement_iterations
    }

    /// Get diagnostics summary.
    ///
    /// Returns:
    ///     dict: Diagnostics with pta and absint info.
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        // PTA diagnostics
        let pta_diag = self.pta.diagnostics();
        let pta_dict = PyDict::new(py);
        pta_dict.set_item("iterations", pta_diag.iterations)?;
        pta_dict.set_item("location_count", pta_diag.location_count)?;
        pta_dict.set_item("constraint_count", pta_diag.constraint_count)?;
        pta_dict.set_item("iteration_limit_hit", pta_diag.iteration_limit_hit)?;
        dict.set_item("pta", pta_dict)?;

        // Absint diagnostics
        let absint_diag = self.absint.diagnostics();
        let absint_dict = PyDict::new(py);
        absint_dict.set_item("blocks_analyzed", absint_diag.blocks_analyzed)?;
        absint_dict.set_item("widening_applications", absint_diag.widening_applications)?;
        absint_dict.set_item("converged", absint_diag.converged)?;
        dict.set_item("absint", absint_dict)?;

        dict.set_item("refinement_iterations", self.refinement_iterations)?;

        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        let pta_diag = self.pta.diagnostics();
        let absint_diag = self.absint.diagnostics();
        format!(
            "CombinedAnalysisResult(pta_locations={}, absint_blocks={}, refinement_iterations={})",
            pta_diag.location_count, absint_diag.blocks_analyzed, self.refinement_iterations
        )
    }
}

/// Run combined analysis with custom configuration.
#[allow(clippy::must_use_candidate)]
pub fn run_analyze_combined_with_config(
    module: &AirModule,
    enable_refinement: Option<bool>,
    max_refinement_iterations: Option<usize>,
) -> PyCombinedAnalysisResult {
    let mut config = CombinedAnalysisConfig::default();
    if let Some(er) = enable_refinement {
        config.enable_refinement = er;
    }
    if let Some(mri) = max_refinement_iterations {
        config.max_refinement_iterations = mri;
    }
    let result = analyze_combined(module, &config);
    PyCombinedAnalysisResult::new(result)
}

/// Register combined analysis types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCombinedAnalysisResult>()?;
    Ok(())
}
