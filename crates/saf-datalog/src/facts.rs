//! Unified fact extraction for Ascent-based analyses.
//!
//! Provides a single [`extract_facts`] entry point that converts SAF's
//! [`ConstraintSet`](saf_analysis::ConstraintSet) into flat tuples suitable
//! for Ascent input relations. This replaces the three separate extraction
//! functions in `saf-analysis` with a scope-driven API.

use std::collections::BTreeSet;

use saf_analysis::{
    ConstraintSet, FieldPath, FieldSensitivity, LocationFactory, extract_constraints,
    extract_intraprocedural_constraints,
};
use saf_core::air::AirModule;
use saf_core::ids::{FunctionId, LocId, ValueId};

/// Scope for fact extraction — selects which extraction strategy to use.
#[derive(Debug, Clone)]
pub enum AnalysisScope {
    /// Extract from all functions (whole-program analysis).
    WholeProgram,
    /// Extract from reachable functions only.
    ///
    /// NOTE: Currently delegates to `WholeProgram` because
    /// `extract_constraints_reachable` is not in `saf-analysis`'s public API.
    /// TODO: expose `extract_constraints_reachable` and use it here.
    Reachable(BTreeSet<FunctionId>),
    /// Extract without interprocedural constraints (arg/param, return/caller).
    ///
    /// Designed for context-sensitive solvers that handle interprocedural
    /// flow with context qualification.
    Intraprocedural,
}

/// Flat fact tuples for Ascent input relations.
///
/// Each variant maps directly to an Ascent relation:
/// - `addr_of` → `addr_of(ptr, loc)` — pointer `ptr` points to location `loc`
/// - `copy` → `copy(dst, src)` — `dst` receives the points-to set of `src`
/// - `load` → `load(dst, src_ptr)` — `dst = *src_ptr`
/// - `store` → `store(dst_ptr, src)` — `*dst_ptr = src`
/// - `gep` → `gep(dst, src_ptr, path)` — `dst = &src_ptr->field`
#[derive(Debug, Clone, Default)]
pub struct PtaFacts {
    /// Address-of facts: `(pointer, location)`.
    pub addr_of: Vec<(ValueId, LocId)>,
    /// Copy facts: `(destination, source)`.
    pub copy: Vec<(ValueId, ValueId)>,
    /// Load facts: `(destination, source_pointer)`.
    pub load: Vec<(ValueId, ValueId)>,
    /// Store facts: `(destination_pointer, source)`.
    pub store: Vec<(ValueId, ValueId)>,
    /// GEP (field access) facts: `(destination, source_pointer, field_path)`.
    pub gep: Vec<(ValueId, ValueId, FieldPath)>,
}

impl PtaFacts {
    /// Total number of facts across all categories.
    #[must_use]
    pub fn total(&self) -> usize {
        self.addr_of.len() + self.copy.len() + self.load.len() + self.store.len() + self.gep.len()
    }

    /// Check whether no facts were extracted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

/// Single entry point for PTA fact extraction.
///
/// Extracts constraints from `module` using the given `scope`, then
/// converts the [`ConstraintSet`] into flat [`PtaFacts`] tuples.
///
/// # Arguments
///
/// * `module` — The AIR module to extract from.
/// * `factory` — Location factory for abstract memory locations.
/// * `scope` — Controls which functions and constraints are extracted.
pub fn extract_facts(
    module: &AirModule,
    factory: &mut LocationFactory,
    scope: AnalysisScope,
) -> PtaFacts {
    let constraints = match scope {
        AnalysisScope::WholeProgram => extract_constraints(module, factory),
        AnalysisScope::Reachable(_reachable) => {
            // Delegate to whole-program until the reachable variant
            // is exposed in saf-analysis's public API.
            extract_constraints(module, factory)
        }
        AnalysisScope::Intraprocedural => extract_intraprocedural_constraints(module, factory),
    };
    constraint_set_to_facts(constraints)
}

/// Convert a [`ConstraintSet`] into flat [`PtaFacts`] tuples.
pub fn constraint_set_to_facts(cs: ConstraintSet) -> PtaFacts {
    PtaFacts {
        addr_of: cs.addr.into_iter().map(|a| (a.ptr, a.loc)).collect(),
        copy: cs.copy.into_iter().map(|c| (c.dst, c.src)).collect(),
        load: cs.load.into_iter().map(|l| (l.dst, l.src_ptr)).collect(),
        store: cs.store.into_iter().map(|s| (s.dst_ptr, s.src)).collect(),
        gep: cs
            .gep
            .into_iter()
            .map(|g| (g.dst, g.src_ptr, g.path))
            .collect(),
    }
}

/// Convenience wrapper that creates a default [`LocationFactory`] and
/// extracts whole-program facts.
///
/// Useful for quick prototyping and tests where fine-grained control
/// over field sensitivity is not needed.
pub fn extract_facts_default(module: &AirModule) -> (PtaFacts, LocationFactory) {
    let mut factory = LocationFactory::new(FieldSensitivity::default());
    let facts = extract_facts(module, &mut factory, AnalysisScope::WholeProgram);
    (facts, factory)
}
