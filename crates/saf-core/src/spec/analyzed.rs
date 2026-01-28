//! Layered spec registry combining YAML-authored specs with analysis-derived facts.

use std::collections::BTreeMap;

use super::derived::DerivedSpec;
use super::registry::SpecRegistry;
use super::types::FunctionSpec;

/// Result of looking up a function in the analyzed registry.
#[derive(Debug)]
pub enum LookupResult<'a> {
    /// YAML spec exists, with optional derived overlay.
    Yaml(&'a FunctionSpec, Option<&'a DerivedSpec>),
    /// Only analysis-derived spec exists (no YAML entry).
    DerivedOnly(&'a DerivedSpec),
}

impl<'a> LookupResult<'a> {
    /// Get the YAML spec if present.
    #[must_use]
    pub fn yaml(&self) -> Option<&'a FunctionSpec> {
        match self {
            Self::Yaml(spec, _) => Some(spec),
            Self::DerivedOnly(_) => None,
        }
    }

    /// Get the derived spec if present.
    #[must_use]
    pub fn derived(&self) -> Option<&'a DerivedSpec> {
        match self {
            Self::Yaml(_, d) => *d,
            Self::DerivedOnly(d) => Some(d),
        }
    }
}

/// A layered spec registry that combines immutable YAML-authored specs
/// with mutable analysis-derived overlays.
///
/// Consumers query via `lookup()`, which returns both the YAML spec (if any)
/// and the derived overlay (if any).
#[derive(Debug)]
pub struct AnalyzedSpecRegistry {
    /// Immutable YAML-authored specs.
    yaml: SpecRegistry,
    /// Analysis-derived overlay, keyed by function name.
    derived: BTreeMap<String, DerivedSpec>,
}

impl AnalyzedSpecRegistry {
    /// Create a new layered registry wrapping existing YAML specs.
    #[must_use]
    pub fn new(yaml: SpecRegistry) -> Self {
        Self {
            yaml,
            derived: BTreeMap::new(),
        }
    }

    /// Add an analysis-derived spec overlay for a function.
    pub fn add_derived(&mut self, name: &str, spec: DerivedSpec) {
        self.derived.insert(name.to_owned(), spec);
    }

    /// Look up a function by name. Returns the YAML spec and/or derived overlay.
    ///
    /// Returns `None` if neither YAML nor derived spec exists for this name.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<LookupResult<'_>> {
        let yaml_spec = self.yaml.lookup(name);
        let derived_spec = self.derived.get(name);

        match (yaml_spec, derived_spec) {
            (Some(y), d) => Some(LookupResult::Yaml(y, d)),
            (None, Some(d)) => Some(LookupResult::DerivedOnly(d)),
            (None, None) => None,
        }
    }

    /// Look up only the derived overlay for a function.
    #[must_use]
    pub fn lookup_derived(&self, name: &str) -> Option<&DerivedSpec> {
        self.derived.get(name)
    }

    /// Look up only the YAML spec for a function (delegates to inner registry).
    #[must_use]
    pub fn lookup_yaml(&self, name: &str) -> Option<&FunctionSpec> {
        self.yaml.lookup(name)
    }

    /// Access the underlying YAML registry.
    #[must_use]
    pub fn yaml(&self) -> &SpecRegistry {
        &self.yaml
    }

    /// Iterate all YAML specs (delegates to inner registry).
    pub fn iter_yaml(&self) -> impl Iterator<Item = &FunctionSpec> {
        self.yaml.iter()
    }

    /// Iterate all derived specs.
    pub fn iter_derived(&self) -> impl Iterator<Item = (&str, &DerivedSpec)> {
        self.derived.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Number of derived overlay entries.
    #[must_use]
    pub fn derived_count(&self) -> usize {
        self.derived.len()
    }
}

impl Default for AnalyzedSpecRegistry {
    fn default() -> Self {
        Self::new(SpecRegistry::default())
    }
}
