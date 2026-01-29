//! Python bindings for selector types.
//!
//! Provides `Selector` and `SelectorSet` classes for specifying taint sources,
//! sinks, and sanitizers in Python.

use pyo3::prelude::*;
use saf_analysis::selector::Selector as RustSelector;

/// A selector for identifying values in the analysis.
///
/// Selectors are created using factory functions in `saf.sources`, `saf.sinks`,
/// and `saf.sanitizers` modules. They can be combined using the `|` operator.
#[pyclass(name = "Selector")]
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct PySelector {
    pub(crate) inner: RustSelector,
}

impl PySelector {
    #[allow(clippy::must_use_candidate)]
    pub fn new(inner: RustSelector) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PySelector {
    /// Combine two selectors with OR semantics.
    fn __or__(&self, other: &PySelector) -> PySelectorSet {
        PySelectorSet {
            selectors: vec![self.inner.clone(), other.inner.clone()],
        }
    }

    fn __repr__(&self) -> String {
        format!("Selector({:?})", self.inner)
    }
}

/// A set of selectors combined with OR semantics.
///
/// Created by combining `Selector` objects with the `|` operator:
/// ```python
/// combined = sources.argv() | sources.getenv()
/// ```
#[pyclass(name = "SelectorSet")]
#[derive(Clone)]
pub struct PySelectorSet {
    pub(crate) selectors: Vec<RustSelector>,
}

impl PySelectorSet {
    /// Create a new empty selector set.
    #[allow(clippy::must_use_candidate)]
    pub fn new() -> Self {
        Self {
            selectors: Vec::new(),
        }
    }

    /// Create a selector set from a single selector.
    #[allow(clippy::must_use_candidate)]
    pub fn from_selector(sel: RustSelector) -> Self {
        Self {
            selectors: vec![sel],
        }
    }

    /// Get the inner selectors.
    #[allow(clippy::must_use_candidate)]
    pub fn inner(&self) -> &[RustSelector] {
        &self.selectors
    }
}

impl Default for PySelectorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PySelectorSet {
    /// Combine with another selector or selector set.
    fn __or__(&self, other: SelectorOrSet) -> PySelectorSet {
        let mut selectors = self.selectors.clone();
        match other {
            SelectorOrSet::Selector(s) => selectors.push(s.inner.clone()),
            SelectorOrSet::Set(set) => selectors.extend(set.selectors.clone()),
        }
        PySelectorSet { selectors }
    }

    fn __repr__(&self) -> String {
        format!("SelectorSet({} selectors)", self.selectors.len())
    }

    fn __len__(&self) -> usize {
        self.selectors.len()
    }
}

/// Union type for combining selectors and selector sets.
#[derive(FromPyObject)]
#[allow(clippy::module_name_repetitions)]
pub enum SelectorOrSet {
    Selector(PySelector),
    Set(PySelectorSet),
}

impl SelectorOrSet {
    /// Convert to a selector set.
    #[allow(clippy::must_use_candidate)]
    pub fn into_selector_set(self) -> PySelectorSet {
        match self {
            SelectorOrSet::Selector(s) => PySelectorSet::from_selector(s.inner),
            SelectorOrSet::Set(set) => set,
        }
    }
}

/// Register selector types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySelector>()?;
    m.add_class::<PySelectorSet>()?;
    Ok(())
}
