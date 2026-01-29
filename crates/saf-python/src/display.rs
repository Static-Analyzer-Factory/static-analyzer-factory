//! Python bindings for `DisplayResolver` and `HumanLabel`.
//!
//! Exposes the display resolution system to Python, allowing scripts to
//! resolve opaque hex IDs into human-readable labels with names, source
//! locations, and contextual descriptions.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::PtaResult;
use saf_analysis::display::{DisplayResolver, HumanLabel};
use saf_analysis::svfg::Svfg;
use saf_core::air::AirModule;

use crate::id_parse::parse_hex;

// ---------------------------------------------------------------------------
// PyHumanLabel
// ---------------------------------------------------------------------------

/// A human-readable label for an AIR entity.
///
/// Contains a short display name, optional long name, source location,
/// entity kind, and containing function name.
#[pyclass(name = "HumanLabel")]
#[derive(Clone)]
pub struct PyHumanLabel {
    /// Entity kind (function, block, instruction, value, global, etc).
    #[pyo3(get)]
    pub kind: String,
    /// Short display name (e.g., "main", "call @printf", "%p").
    #[pyo3(get)]
    pub short_name: String,
    /// Longer descriptive name with context, if available.
    #[pyo3(get)]
    pub long_name: Option<String>,
    /// Source location as "file:line:col", if available.
    #[pyo3(get)]
    pub source_loc: Option<String>,
    /// Name of the containing function, if applicable.
    #[pyo3(get)]
    pub containing_function: Option<String>,
}

impl PyHumanLabel {
    /// Create from a Rust `HumanLabel`.
    fn from_rust(label: &HumanLabel) -> Self {
        Self {
            kind: label.kind.to_string(),
            short_name: label.short_name.clone(),
            long_name: label.long_name.clone(),
            source_loc: label
                .source_loc
                .as_ref()
                .map(std::string::ToString::to_string),
            containing_function: label.containing_function.clone(),
        }
    }
}

#[pymethods]
impl PyHumanLabel {
    /// Convert the label to a dictionary.
    ///
    /// Returns:
    ///     dict: Label data with keys: kind, short_name, long_name,
    ///         source_loc, containing_function.
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("short_name", &self.short_name)?;
        dict.set_item("long_name", &self.long_name)?;
        dict.set_item("source_loc", &self.source_loc)?;
        dict.set_item("containing_function", &self.containing_function)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        if let Some(loc) = &self.source_loc {
            format!("HumanLabel({} '{}' at {})", self.kind, self.short_name, loc)
        } else {
            format!("HumanLabel({} '{}')", self.kind, self.short_name)
        }
    }
}

// ---------------------------------------------------------------------------
// PyDisplayResolver
// ---------------------------------------------------------------------------

/// Resolves opaque hex IDs to human-readable labels.
///
/// The resolver maps AIR entity IDs (functions, blocks, instructions,
/// values, globals) and analysis-derived IDs (PTA locations, SVFG nodes)
/// to descriptive labels with names, source locations, and context.
///
/// Example:
///     resolver = proj.display_resolver()
///     label = resolver.resolve("0x00001234abcd5678...")
///     print(label.short_name, label.source_loc)
#[pyclass(name = "DisplayResolver")]
pub struct PyDisplayResolver {
    /// The AIR module (kept alive for the resolver's lifetime).
    module: Arc<AirModule>,
    /// PTA result for Tier 2 resolution (kept alive).
    pta_result: Option<Arc<PtaResult>>,
    /// SVFG for Tier 2 resolution (kept alive).
    svfg: Option<Arc<Svfg>>,
}

impl PyDisplayResolver {
    /// Create a new `PyDisplayResolver` with analysis results for Tier 2 resolution.
    #[allow(clippy::must_use_candidate)]
    pub fn new(
        module: Arc<AirModule>,
        pta_result: Option<Arc<PtaResult>>,
        svfg: Option<Arc<Svfg>>,
    ) -> Self {
        Self {
            module,
            pta_result,
            svfg,
        }
    }

    /// Build the inner Rust `DisplayResolver` on demand.
    ///
    /// We cannot store the resolver directly because it borrows from
    /// the module/pta/svfg. Instead we build it fresh each time (the
    /// internal cache makes repeated resolves cheap within a single call).
    fn build_resolver(&self) -> DisplayResolver<'_> {
        DisplayResolver::with_analysis(
            &self.module,
            self.pta_result.as_deref(),
            self.svfg.as_deref(),
        )
    }

    /// Resolve a single raw u128 ID and return the `HumanLabel`.
    pub fn resolve_raw(&self, id: u128) -> HumanLabel {
        let resolver = self.build_resolver();
        resolver.resolve(id)
    }
}

#[pymethods]
impl PyDisplayResolver {
    /// Resolve a hex ID to a human-readable label.
    ///
    /// Args:
    ///     hex_id: Hex string ID (with or without "0x" prefix).
    ///
    /// Returns:
    ///     HumanLabel: Label with kind, short_name, long_name,
    ///         source_loc, and containing_function attributes.
    fn resolve(&self, hex_id: &str) -> PyResult<PyHumanLabel> {
        let id = parse_hex(hex_id)?;
        let label = self.resolve_raw(id);
        Ok(PyHumanLabel::from_rust(&label))
    }

    /// Resolve multiple hex IDs to human-readable labels in batch.
    ///
    /// More efficient than calling `resolve()` in a loop because the
    /// internal resolver cache is shared across all lookups.
    ///
    /// Args:
    ///     hex_ids: List of hex string IDs (with or without "0x" prefix).
    ///
    /// Returns:
    ///     list[HumanLabel]: List of labels, one per input ID.
    fn resolve_batch(&self, hex_ids: Vec<String>) -> PyResult<Vec<PyHumanLabel>> {
        let resolver = self.build_resolver();
        let mut results = Vec::with_capacity(hex_ids.len());
        for hex_id in &hex_ids {
            let id = parse_hex(hex_id)?;
            let label = resolver.resolve(id);
            results.push(PyHumanLabel::from_rust(&label));
        }
        Ok(results)
    }

    fn __repr__(&self) -> String {
        let tier = if self.pta_result.is_some() || self.svfg.is_some() {
            "Tier1+Tier2"
        } else {
            "Tier1"
        };
        format!("DisplayResolver({tier})")
    }
}

/// Register display types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHumanLabel>()?;
    m.add_class::<PyDisplayResolver>()?;
    Ok(())
}
