//! Analysis-derived function specifications.
//!
//! These types represent function contract information discovered by
//! static analysis, as opposed to YAML-authored `FunctionSpec` entries.

use std::collections::BTreeMap;

/// How a return value is bounded by an argument property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundMode {
    /// Return ∈ `[0, alloc_size(param) - 1]`.
    /// Models: `strlen(buf)`, `strnlen(buf, n)` (first arg).
    AllocSizeMinusOne,
    /// Return ∈ `[0, alloc_size(param)]`.
    /// Models: `fread(buf, 1, size, fp)` where size = `alloc_size(buf)`.
    AllocSize,
    /// Return ∈ `[-1, param_value - 1]`.
    /// Models: `read(fd, buf, count)` returns `[-1, count-1]`.
    ParamValueMinusOne,
}

/// A computed return bound: return interval depends on an argument property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedBound {
    /// Which parameter's property bounds the return value.
    pub param_index: u32,
    /// How the return is bounded.
    pub mode: BoundMode,
}

/// Analysis-derived specification for a function.
///
/// Produced by the summary module and merged with YAML specs in
/// `AnalyzedSpecRegistry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedSpec {
    /// Computed return interval bound (if applicable).
    pub computed_return_bound: Option<ComputedBound>,
    /// Whether the callee frees `param[i]` (directly or transitively).
    pub param_freed: BTreeMap<usize, bool>,
    /// Whether the callee dereferences `param[i]`.
    pub param_dereferenced: BTreeMap<usize, bool>,
    /// Whether the function returns newly allocated memory.
    pub return_is_allocated: bool,
}

impl DerivedSpec {
    /// Create an empty derived spec with no discovered properties.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            computed_return_bound: None,
            param_freed: BTreeMap::new(),
            param_dereferenced: BTreeMap::new(),
            return_is_allocated: false,
        }
    }

    /// Create from a `ParameterEffectSummary`-style tuple.
    #[must_use]
    pub fn from_effects(
        param_freed: BTreeMap<usize, bool>,
        param_dereferenced: BTreeMap<usize, bool>,
        return_is_allocated: bool,
    ) -> Self {
        Self {
            computed_return_bound: None,
            param_freed,
            param_dereferenced,
            return_is_allocated,
        }
    }
}
