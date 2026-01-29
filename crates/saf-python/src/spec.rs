//! Python bindings for function specifications.

use pyo3::prelude::*;
use std::path::PathBuf;

use saf_core::spec::{FunctionSpec, Nullness, Pointer, Role, SpecRegistry};

fn nullness_to_string(n: Nullness) -> String {
    match n {
        Nullness::NotNull => "not_null".to_string(),
        Nullness::MaybeNull => "maybe_null".to_string(),
        Nullness::RequiredNonnull => "required_nonnull".to_string(),
        Nullness::Nullable => "nullable".to_string(),
    }
}

/// Python wrapper for SpecRegistry.
#[pyclass(name = "SpecRegistry")]
pub struct PySpecRegistry {
    inner: SpecRegistry,
}

#[pymethods]
impl PySpecRegistry {
    /// Create a new empty registry.
    #[new]
    fn new() -> Self {
        Self {
            inner: SpecRegistry::new(),
        }
    }

    /// Load specs from default discovery paths.
    ///
    /// Discovery order (later overrides earlier per-function):
    /// 1. <binary>/../share/saf/specs/*.yaml (shipped defaults)
    /// 2. ~/.saf/specs/*.yaml (user global)
    /// 3. ./saf-specs/*.yaml (project local)
    /// 4. $SAF_SPECS_PATH/*.yaml (explicit override)
    #[staticmethod]
    fn load() -> PyResult<Self> {
        let inner = SpecRegistry::load()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Load specs from specific paths.
    ///
    /// Each path can be a file or directory. Directories are scanned for *.yaml files.
    #[staticmethod]
    fn load_from(paths: Vec<String>) -> PyResult<Self> {
        let pathbufs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        let inner = SpecRegistry::load_from(&pathbufs)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Look up a function spec by name.
    ///
    /// Returns None if no spec is found or the spec is disabled.
    fn lookup(&self, name: &str) -> Option<PyFunctionSpec> {
        self.inner
            .lookup(name)
            .map(|s| PyFunctionSpec::from(s.clone()))
    }

    /// Get the number of exact-match specs.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Check if the registry is empty.
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// List all function names with specs.
    fn function_names(&self) -> Vec<String> {
        self.inner.iter().map(|s| s.name.clone()).collect()
    }

    /// Get paths that were loaded.
    fn loaded_paths(&self) -> Vec<String> {
        self.inner
            .loaded_paths()
            .iter()
            .map(|p| p.display().to_string())
            .collect()
    }

    /// Get warnings generated during loading.
    fn warnings(&self) -> Vec<String> {
        self.inner.warnings().to_vec()
    }
}

/// Python wrapper for FunctionSpec.
#[pyclass(name = "FunctionSpec")]
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct PyFunctionSpec {
    inner: FunctionSpec,
}

impl From<FunctionSpec> for PyFunctionSpec {
    fn from(spec: FunctionSpec) -> Self {
        Self { inner: spec }
    }
}

#[pymethods]
impl PyFunctionSpec {
    /// Get the function name.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// Get the role (allocator, source, sink, sanitizer, etc.) or None.
    #[getter]
    fn role(&self) -> Option<String> {
        self.inner.role.as_ref().map(|r| match r {
            Role::Allocator => "allocator".to_string(),
            Role::Reallocator => "reallocator".to_string(),
            Role::Deallocator => "deallocator".to_string(),
            Role::Source => "source".to_string(),
            Role::Sink => "sink".to_string(),
            Role::Sanitizer => "sanitizer".to_string(),
            Role::StringOperation => "string_operation".to_string(),
            Role::Io => "io".to_string(),
            Role::Custom(s) => s.clone(),
        })
    }

    /// Check if the function is pure (no side effects).
    #[getter]
    fn pure(&self) -> bool {
        self.inner.is_pure()
    }

    /// Check if the function never returns (exit, abort, etc.).
    #[getter]
    fn noreturn(&self) -> bool {
        self.inner.is_noreturn()
    }

    /// Check if the spec is disabled.
    #[getter]
    fn disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    /// Get the number of parameter specs.
    fn param_count(&self) -> usize {
        self.inner.params.len()
    }

    /// Get the return nullness (not_null, maybe_null) or None.
    #[getter]
    fn return_nullness(&self) -> Option<String> {
        self.inner
            .returns
            .as_ref()
            .and_then(|r| r.nullness)
            .map(nullness_to_string)
    }

    /// Get the return pointer type (fresh_heap, fresh_stack, unknown) or None.
    #[getter]
    fn return_pointer(&self) -> Option<String> {
        self.inner
            .returns
            .as_ref()
            .and_then(|r| r.pointer.as_ref())
            .map(|p| match p {
                Pointer::FreshHeap => "fresh_heap".to_string(),
                Pointer::FreshStack => "fresh_stack".to_string(),
                Pointer::Unknown => "unknown".to_string(),
                Pointer::StaticSingleton => "static_singleton".to_string(),
            })
    }

    /// Get the return alias (param.N) or None.
    #[getter]
    fn return_aliases(&self) -> Option<String> {
        self.inner.returns.as_ref().and_then(|r| r.aliases.clone())
    }

    /// Check if return value is tainted (for sources).
    #[getter]
    fn return_tainted(&self) -> bool {
        self.inner
            .returns
            .as_ref()
            .is_some_and(saf_core::spec::ReturnSpec::is_tainted)
    }

    /// Get parameter info by index.
    fn param(&self, index: u32) -> Option<PyParamSpec> {
        self.inner
            .param(index)
            .map(|p| PyParamSpec::from(p.clone()))
    }

    /// Check if a parameter requires non-null.
    fn param_required_nonnull(&self, index: u32) -> bool {
        self.inner
            .param(index)
            .and_then(|p| p.nullness)
            .is_some_and(|n| n == saf_core::spec::Nullness::RequiredNonnull)
    }

    /// String representation.
    fn __repr__(&self) -> String {
        let role = self.role().unwrap_or_else(|| "none".to_string());
        format!("FunctionSpec('{}', role={})", self.inner.name, role)
    }
}

/// Python wrapper for ParamSpec.
#[pyclass(name = "ParamSpec")]
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct PyParamSpec {
    inner: saf_core::spec::ParamSpec,
}

impl From<saf_core::spec::ParamSpec> for PyParamSpec {
    fn from(spec: saf_core::spec::ParamSpec) -> Self {
        Self { inner: spec }
    }
}

#[pymethods]
impl PyParamSpec {
    /// Parameter index (0-based).
    #[getter]
    fn index(&self) -> u32 {
        self.inner.index
    }

    /// Parameter name or None.
    #[getter]
    fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    /// Check if parameter modifies its pointee.
    #[getter]
    fn modifies(&self) -> bool {
        self.inner.does_modify()
    }

    /// Check if parameter reads its pointee.
    #[getter]
    fn reads(&self) -> bool {
        self.inner.does_read()
    }

    /// Check if parameter may escape.
    #[getter]
    fn escapes(&self) -> bool {
        self.inner.may_escape()
    }

    /// Check if parameter is a callback.
    #[getter]
    fn callback(&self) -> bool {
        self.inner.is_callback()
    }

    /// Get nullness requirement or None.
    #[getter]
    fn nullness(&self) -> Option<String> {
        self.inner.nullness.map(nullness_to_string)
    }

    /// Get semantic meaning or None.
    #[getter]
    fn semantic(&self) -> Option<String> {
        self.inner.semantic.clone()
    }

    /// String representation.
    fn __repr__(&self) -> String {
        format!("ParamSpec(index={})", self.inner.index)
    }
}

/// Register spec types with the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySpecRegistry>()?;
    m.add_class::<PyFunctionSpec>()?;
    m.add_class::<PyParamSpec>()?;
    Ok(())
}
