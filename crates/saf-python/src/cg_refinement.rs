//! Python bindings for call graph refinement (CHA + PTA iterative loop).
//!
//! Exposes `RefinementResult` and `ClassHierarchy` to Python.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::cg_refinement::{EntryPointStrategy, RefinementConfig, RefinementResult};
use saf_analysis::cha::ClassHierarchy;
use saf_analysis::{FieldSensitivity, PtaConfig};
use saf_core::air::AirModule;
use saf_core::config::PtaSolver;

/// Python wrapper for [`RefinementResult`].
#[pyclass(name = "RefinementResult")]
pub struct PyRefinementResult {
    inner: RefinementResult,
    module: Arc<AirModule>,
}

impl PyRefinementResult {
    #[allow(clippy::must_use_candidate)]
    pub fn new(inner: RefinementResult, module: Arc<AirModule>) -> Self {
        Self { inner, module }
    }
}

#[pymethods]
impl PyRefinementResult {
    /// Number of PTA iterations executed.
    #[getter]
    fn iterations(&self) -> usize {
        self.inner.iterations
    }

    /// Export the refined call graph as a dictionary.
    ///
    /// Returns:
    ///     dict: Serialized call graph with "nodes" and "edges".
    fn call_graph_export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let export = self.inner.call_graph.export(&self.module);
        crate::helpers::serde_to_py_dict(py, &export)
    }

    /// Get the resolved indirect call sites.
    ///
    /// Returns:
    ///     dict: Maps call-site instruction ID (hex) to list of target
    ///           function IDs (hex).
    fn resolved_sites(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (site, targets) in &self.inner.resolved_sites {
            let key = site.to_hex();
            let value: Vec<String> = targets.iter().map(|fid| fid.to_hex()).collect();
            dict.set_item(key, value)?;
        }
        Ok(dict.into())
    }

    /// Export the PTA result as a dictionary (if PTA ran).
    ///
    /// Returns:
    ///     dict or None: Serialized PTA result.
    fn pta_export(&self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        let Some(ref pta_result) = self.inner.pta_result else {
            return Ok(None);
        };
        let config = PtaConfig::default();
        let export = pta_result.export(&config);
        Ok(Some(crate::helpers::serde_to_py_dict(py, &export)?))
    }

    /// Get the class hierarchy (if one was built).
    ///
    /// Returns:
    ///     ClassHierarchy or None.
    fn class_hierarchy(&self) -> Option<PyCha> {
        self.inner
            .cha
            .as_ref()
            .map(|cha| PyCha { inner: cha.clone() })
    }

    fn __repr__(&self) -> String {
        format!(
            "RefinementResult(iterations={}, resolved_sites={})",
            self.inner.iterations,
            self.inner.resolved_sites.len()
        )
    }
}

/// Python wrapper for [`ClassHierarchy`].
#[pyclass(name = "ClassHierarchy")]
#[derive(Clone)]
pub struct PyCha {
    inner: ClassHierarchy,
}

#[pymethods]
impl PyCha {
    /// Get transitive subclasses of a class.
    ///
    /// Args:
    ///     class_name: The class name.
    ///
    /// Returns:
    ///     list[str]: Sorted list of subclass names.
    fn subclasses_of(&self, class_name: &str) -> Vec<String> {
        self.inner.subclasses_of(class_name).into_iter().collect()
    }

    /// Resolve a virtual call on a receiver class at a vtable slot.
    ///
    /// Args:
    ///     receiver: The receiver class name.
    ///     slot: The vtable slot index.
    ///
    /// Returns:
    ///     list[str]: Function IDs (hex) of possible targets.
    fn resolve_virtual(&self, receiver: &str, slot: usize) -> Vec<String> {
        self.inner
            .resolve_virtual(receiver, slot)
            .iter()
            .map(|fid| fid.to_hex())
            .collect()
    }

    /// Export the class hierarchy as a dictionary.
    ///
    /// Returns:
    ///     dict: Hierarchy data with "classes", "inheritance", "vtables".
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let value = self.inner.export();
        crate::helpers::serde_to_py_dict(py, &value)
    }

    fn __repr__(&self) -> String {
        "ClassHierarchy(...)".to_string()
    }
}

/// Run call graph refinement from Python.
///
/// Called by `Project.refine_call_graph()`.
#[allow(clippy::must_use_candidate)]
pub fn run_refinement(
    module: &Arc<AirModule>,
    entry_points: &str,
    max_iterations: usize,
) -> PyRefinementResult {
    let entry_strategy = if entry_points == "all" {
        EntryPointStrategy::AllDefined
    } else {
        EntryPointStrategy::Named(
            entry_points
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        )
    };

    let config = RefinementConfig {
        max_iterations,
        entry_points: entry_strategy,
        pta_config: PtaConfig::default(),
        field_sensitivity: FieldSensitivity::default(),
        pta_solver: PtaSolver::default(),
    };

    let result = saf_analysis::cg_refinement::refine(module, &config, None);
    PyRefinementResult::new(result, Arc::clone(module))
}

/// Register CG refinement types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRefinementResult>()?;
    m.add_class::<PyCha>()?;
    Ok(())
}
