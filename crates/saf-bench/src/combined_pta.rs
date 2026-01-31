//! Combined CS-PTA + FS-PTA result for maximum precision.
//!
//! This module provides a wrapper that runs both context-sensitive and
//! flow-sensitive pointer analysis, combining their results to get the
//! most definite alias answer for each query.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use saf_analysis::AliasResult;
use saf_analysis::cspta::CsPtaResult;
use saf_core::ids::{LocId, ValueId};

/// Flow-sensitive points-to data for alias queries.
///
/// Holds both the global top-level `pts` map (monotonically accumulated by the
/// FS-PTA solver) and per-load-destination flow-sensitive pts reconstructed
/// from `df_in` at each load node. The load-sensitive data is preferred when
/// available because it reflects the actual flow-sensitive state at the load's
/// program point, rather than the union across all iterations.
pub struct FsPts {
    /// Global top-level points-to map (fallback).
    pts: BTreeMap<ValueId, BTreeSet<LocId>>,
    /// Per-load-destination flow-sensitive points-to sets.
    /// Only contains entries where flow-sensitive info is available.
    load_sensitive_pts: BTreeMap<ValueId, BTreeSet<LocId>>,
}

impl FsPts {
    /// Create from a global points-to map (without load-sensitive data).
    #[must_use]
    pub fn new(pts: BTreeMap<ValueId, BTreeSet<LocId>>) -> Self {
        Self {
            pts,
            load_sensitive_pts: BTreeMap::new(),
        }
    }

    /// Create with both global and load-sensitive points-to maps.
    #[must_use]
    pub fn with_load_sensitive(
        pts: BTreeMap<ValueId, BTreeSet<LocId>>,
        load_sensitive_pts: BTreeMap<ValueId, BTreeSet<LocId>>,
    ) -> Self {
        Self {
            pts,
            load_sensitive_pts,
        }
    }

    /// Get the points-to set for a value, preferring flow-sensitive data.
    #[must_use]
    pub fn points_to(&self, value: ValueId) -> &BTreeSet<LocId> {
        static EMPTY: BTreeSet<LocId> = BTreeSet::new();
        // Prefer load-sensitive (flow-sensitive) pts when available
        self.load_sensitive_pts
            .get(&value)
            .or_else(|| self.pts.get(&value))
            .unwrap_or(&EMPTY)
    }

    /// Check alias relationship using the points-to sets.
    ///
    /// Prefers load-sensitive (flow-sensitive) points-to data when available
    /// for load destinations. Falls back to the global `pts` map otherwise.
    ///
    /// Note: When one pointer has an empty set, we return `Unknown` rather than
    /// `NoAlias`. An empty set means we don't have information about that pointer
    /// (e.g., it's a parameter that receives values interprocedurally, which
    /// flow-sensitive analysis doesn't model). Returning `Unknown` allows the
    /// combined result to defer to CS-PTA which has interprocedural context.
    #[must_use]
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        let ps = self.points_to(p);
        let qs = self.points_to(q);

        // Handle empty sets: if one or both are empty, we lack information.
        // Return Unknown to defer to context-sensitive analysis.
        if ps.is_empty() || qs.is_empty() {
            return AliasResult::Unknown;
        }

        // Check alias relationship
        if ps.is_disjoint(qs) {
            AliasResult::No
        } else if ps == qs && ps.len() == 1 {
            // Singleton sets pointing to same location: MustAlias
            AliasResult::Must
        } else if ps == qs {
            // Non-singleton equal sets: MayAlias
            AliasResult::May
        } else if ps.is_subset(qs) || qs.is_subset(ps) {
            // Proper subset: PartialAlias
            AliasResult::Partial
        } else {
            AliasResult::May
        }
    }
}

/// Combined result from running both CS-PTA and FS-PTA.
///
/// For alias queries, takes the most definite answer from either analysis:
/// - If either proves NoAlias → NoAlias
/// - If either proves MustAlias → MustAlias
/// - If either returns Unknown, use the other
/// - Otherwise use the more precise between Partial and May
pub struct CombinedPtaResult {
    cs_result: CsPtaResult,
    fs_pts: FsPts,
}

impl CombinedPtaResult {
    /// Create a new combined result from CS-PTA result and FS-PTA points-to map.
    #[must_use]
    pub fn new(cs_result: CsPtaResult, fs_pts: BTreeMap<ValueId, BTreeSet<LocId>>) -> Self {
        Self {
            cs_result,
            fs_pts: FsPts::new(fs_pts),
        }
    }

    /// Create with flow-sensitive load-destination points-to data.
    ///
    /// The `load_sensitive_pts` map provides per-load-destination flow-sensitive
    /// points-to sets reconstructed from FS-PTA's `df_in` at each load node.
    /// These are more precise than the global `pts` map for values that are
    /// load destinations, enabling flow-sensitive alias queries.
    #[must_use]
    pub fn with_load_sensitive(
        cs_result: CsPtaResult,
        fs_pts: BTreeMap<ValueId, BTreeSet<LocId>>,
        load_sensitive_pts: BTreeMap<ValueId, BTreeSet<LocId>>,
    ) -> Self {
        Self {
            cs_result,
            fs_pts: FsPts::with_load_sensitive(fs_pts, load_sensitive_pts),
        }
    }

    /// Query alias relationship using combined analysis.
    ///
    /// Uses the CI summary from CS-PTA and top-level pts from FS-PTA,
    /// combining the results to return the most definite answer.
    #[must_use]
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        let cs_alias = self.cs_result.may_alias_any(p, q);
        let fs_alias = self.fs_pts.may_alias(p, q);
        combine_alias_results(cs_alias, fs_alias)
    }

    /// Get the CS-PTA result.
    #[must_use]
    pub fn cs_result(&self) -> &CsPtaResult {
        &self.cs_result
    }

    /// Get the FS-PTA points-to data.
    #[must_use]
    pub fn fs_pts(&self) -> &FsPts {
        &self.fs_pts
    }
}

/// Combine two alias results, taking the most PRECISE answer.
///
/// This uses precision-oriented semantics where definite answers (No, Must)
/// are trusted over conservative answers (May, Partial). The assumption is
/// that both analyses are sound, so a more precise answer from either can
/// be safely used.
///
/// Precision order (from most to least definite):
/// - `No` and `Must` are definite answers (strongest claims)
/// - `Partial` is more precise than `May`
/// - `Unknown` defers to the other result
///
/// | CS-PTA  | FS-PTA  | Combined |
/// |---------|---------|----------|
/// | No      | *       | No       |
/// | *       | No      | No       |
/// | Must    | *       | Must     |
/// | *       | Must    | Must     |
/// | Unknown | X       | X        |
/// | X       | Unknown | X        |
/// | Partial | Partial | Partial  |
/// | Partial | May     | Partial  |
/// | May     | May     | May      |
///
/// NOTE: This prioritizes precision over soundness. If one analysis is buggy
/// and reports NoAlias incorrectly, the combined result will be wrong.
/// For maximum soundness, use positive-wins semantics instead.
#[must_use]
pub fn combine_alias_results(a: AliasResult, b: AliasResult) -> AliasResult {
    // NoAlias is a definite negative claim - trust it if either analysis proves it
    if matches!(a, AliasResult::No) || matches!(b, AliasResult::No) {
        return AliasResult::No;
    }

    // MustAlias is a definite positive claim - trust it if either analysis proves it
    if matches!(a, AliasResult::Must) || matches!(b, AliasResult::Must) {
        return AliasResult::Must;
    }

    // Unknown defers to the other result
    if matches!(a, AliasResult::Unknown) {
        return b;
    }
    if matches!(b, AliasResult::Unknown) {
        return a;
    }

    // At this point, both are Partial or May
    // Partial is more precise than May
    if matches!(a, AliasResult::Partial) || matches!(b, AliasResult::Partial) {
        return AliasResult::Partial;
    }

    // Both are May
    AliasResult::May
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_no_alias_wins() {
        // NoAlias is definite - wins over conservative answers
        assert_eq!(
            combine_alias_results(AliasResult::No, AliasResult::May),
            AliasResult::No
        );
        assert_eq!(
            combine_alias_results(AliasResult::May, AliasResult::No),
            AliasResult::No
        );
        // NoAlias beats MustAlias (both are definite, No checked first)
        assert_eq!(
            combine_alias_results(AliasResult::No, AliasResult::Must),
            AliasResult::No
        );
    }

    #[test]
    fn test_combine_must_alias_wins() {
        // MustAlias wins over May/Partial (when No not present)
        assert_eq!(
            combine_alias_results(AliasResult::Must, AliasResult::May),
            AliasResult::Must
        );
        assert_eq!(
            combine_alias_results(AliasResult::May, AliasResult::Must),
            AliasResult::Must
        );
        assert_eq!(
            combine_alias_results(AliasResult::Must, AliasResult::Partial),
            AliasResult::Must
        );
    }

    #[test]
    fn test_combine_unknown_defers() {
        assert_eq!(
            combine_alias_results(AliasResult::Unknown, AliasResult::May),
            AliasResult::May
        );
        assert_eq!(
            combine_alias_results(AliasResult::Partial, AliasResult::Unknown),
            AliasResult::Partial
        );
        assert_eq!(
            combine_alias_results(AliasResult::Unknown, AliasResult::Unknown),
            AliasResult::Unknown
        );
    }

    #[test]
    fn test_combine_partial_over_may() {
        assert_eq!(
            combine_alias_results(AliasResult::Partial, AliasResult::May),
            AliasResult::Partial
        );
        assert_eq!(
            combine_alias_results(AliasResult::May, AliasResult::Partial),
            AliasResult::Partial
        );
        assert_eq!(
            combine_alias_results(AliasResult::May, AliasResult::May),
            AliasResult::May
        );
    }
}
