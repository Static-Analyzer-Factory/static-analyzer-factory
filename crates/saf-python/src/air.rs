//! Python bindings for AIR module access.
//!
//! Provides read-only access to the analyzed AIR module.

use std::sync::Arc;

use pyo3::prelude::*;

use saf_core::air::{AirFunction, AirGlobal, AirModule};

/// Python wrapper for AIR module.
#[pyclass(name = "AirModule")]
pub struct PyAirModule {
    pub(crate) inner: Arc<AirModule>,
}

impl PyAirModule {
    #[allow(clippy::must_use_candidate)]
    pub fn new(inner: Arc<AirModule>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyAirModule {
    /// Get the module name.
    #[getter]
    fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    /// Get the module ID as hex string.
    #[getter]
    fn id(&self) -> String {
        self.inner.id.to_hex()
    }

    /// Get the number of functions.
    #[getter]
    fn function_count(&self) -> usize {
        self.inner.functions.len()
    }

    /// Get the number of globals.
    #[getter]
    fn global_count(&self) -> usize {
        self.inner.globals.len()
    }

    /// Get all function names.
    fn function_names(&self) -> Vec<String> {
        self.inner
            .functions
            .iter()
            .map(|f| f.name.clone())
            .collect()
    }

    /// Get all global names.
    fn global_names(&self) -> Vec<String> {
        self.inner.globals.iter().map(|g| g.name.clone()).collect()
    }

    /// Get a function by name.
    fn get_function(&self, name: &str) -> Option<PyAirFunction> {
        self.inner
            .functions
            .iter()
            .find(|f| f.name == name)
            .map(|f| PyAirFunction::new(f.clone()))
    }

    /// Get a global by name.
    fn get_global(&self, name: &str) -> Option<PyAirGlobal> {
        self.inner
            .globals
            .iter()
            .find(|g| g.name == name)
            .map(|g| PyAirGlobal::new(g.clone()))
    }

    /// Get all functions.
    fn functions(&self) -> Vec<PyAirFunction> {
        self.inner
            .functions
            .iter()
            .map(|f| PyAirFunction::new(f.clone()))
            .collect()
    }

    /// Get all globals.
    fn globals(&self) -> Vec<PyAirGlobal> {
        self.inner
            .globals
            .iter()
            .map(|g| PyAirGlobal::new(g.clone()))
            .collect()
    }

    fn __repr__(&self) -> String {
        let name = self.inner.name.as_deref().unwrap_or("<unnamed>");
        format!(
            "AirModule(name='{}', functions={}, globals={})",
            name,
            self.inner.functions.len(),
            self.inner.globals.len()
        )
    }
}

/// Python wrapper for AIR function.
#[pyclass(name = "AirFunction")]
#[derive(Clone)]
pub struct PyAirFunction {
    inner: AirFunction,
}

impl PyAirFunction {
    #[allow(clippy::must_use_candidate)]
    pub fn new(inner: AirFunction) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyAirFunction {
    /// Get the function name.
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get the function ID as hex string.
    #[getter]
    fn id(&self) -> String {
        self.inner.id.to_hex()
    }

    /// Whether this is a declaration (no body).
    #[getter]
    fn is_declaration(&self) -> bool {
        self.inner.is_declaration
    }

    /// Get the number of parameters.
    #[getter]
    fn param_count(&self) -> usize {
        self.inner.params.len()
    }

    /// Get the number of basic blocks.
    #[getter]
    fn block_count(&self) -> usize {
        self.inner.blocks.len()
    }

    /// Get parameter IDs.
    fn param_ids(&self) -> Vec<String> {
        self.inner.params.iter().map(|p| p.id.to_hex()).collect()
    }

    /// Get parameter names.
    fn param_names(&self) -> Vec<Option<String>> {
        self.inner.params.iter().map(|p| p.name.clone()).collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "AirFunction(name='{}', params={}, blocks={})",
            self.inner.name,
            self.inner.params.len(),
            self.inner.blocks.len()
        )
    }
}

/// Python wrapper for AIR global.
#[pyclass(name = "AirGlobal")]
#[derive(Clone)]
pub struct PyAirGlobal {
    inner: AirGlobal,
}

impl PyAirGlobal {
    #[allow(clippy::must_use_candidate)]
    pub fn new(inner: AirGlobal) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyAirGlobal {
    /// Get the global name.
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get the global ID as hex string.
    #[getter]
    fn id(&self) -> String {
        self.inner.id.to_hex()
    }

    /// Whether this global is a constant.
    #[getter]
    fn is_constant(&self) -> bool {
        self.inner.is_constant
    }

    fn __repr__(&self) -> String {
        format!(
            "AirGlobal(name='{}', constant={})",
            self.inner.name, self.inner.is_constant
        )
    }
}

/// Register AIR types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAirModule>()?;
    m.add_class::<PyAirFunction>()?;
    m.add_class::<PyAirGlobal>()?;
    Ok(())
}
