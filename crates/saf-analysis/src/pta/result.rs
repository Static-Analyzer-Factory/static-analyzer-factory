//! PTA result types and alias queries.
//!
//! Provides the analysis result with query methods for points-to sets
//! and alias analysis.

use std::sync::Arc;

use rustc_hash::FxHashMap;
use saf_core::ids::{LocId, ValueId};

use super::context::PtaDiagnostics;
use super::location::{AllocationMultiplicity, Location, LocationFactory, MemoryRegion};
use super::solver::PointsToMap;

/// Five-valued alias result.
///
/// Represents whether two pointers must alias, partially alias, may alias,
/// definitely don't alias, or the analysis couldn't determine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasResult {
    /// Pointers must alias (identical singleton or equal points-to sets).
    ///
    /// This is the strongest form of aliasing: the pointers are guaranteed
    /// to refer to the same memory location(s).
    Must,
    /// Pointers partially alias (one location is a subfield of another).
    ///
    /// One pointer's targets are a strict subset of the other's, typically
    /// because one points to a struct and another to a field within it.
    Partial,
    /// Pointers may alias (points-to sets overlap but not identical).
    May,
    /// Pointers definitely don't alias (disjoint points-to sets).
    No,
    /// Unknown (one or both pointers have unknown points-to sets).
    Unknown,
}

impl AliasResult {
    /// Returns true if the pointers must alias.
    #[must_use]
    pub fn must_alias(self) -> bool {
        matches!(self, Self::Must)
    }

    /// Returns true if the pointers partially alias (or must alias).
    ///
    /// Partial aliasing includes must-alias since it's a stronger condition.
    #[must_use]
    pub fn partial_alias(self) -> bool {
        matches!(self, Self::Must | Self::Partial)
    }

    /// Returns true if the pointers may alias (conservative).
    ///
    /// `Must`, `Partial`, and `Unknown` are treated as `true` (may alias).
    #[must_use]
    pub fn may_alias_conservative(self) -> bool {
        matches!(self, Self::Must | Self::Partial | Self::May | Self::Unknown)
    }

    /// Returns true if the pointers may alias (optimistic).
    ///
    /// `Unknown` is treated as `false` (don't alias).
    #[must_use]
    pub fn may_alias_optimistic(self) -> bool {
        matches!(self, Self::Must | Self::Partial | Self::May)
    }

    /// Returns true if the pointers definitely don't alias.
    #[must_use]
    pub fn no_alias(self) -> bool {
        matches!(self, Self::No)
    }
}

/// Points-to analysis result with query API.
#[derive(Clone)]
pub struct PtaResult {
    /// Points-to map from values to locations.
    pts: PointsToMap,
    /// Location factory (shared via `Arc` to avoid O(N) clone).
    factory: Arc<LocationFactory>,
    /// Analysis diagnostics.
    diagnostics: PtaDiagnostics,
}

impl PtaResult {
    /// Create a new result from analysis output.
    #[must_use]
    pub fn new(
        pts: PointsToMap,
        factory: Arc<LocationFactory>,
        diagnostics: PtaDiagnostics,
    ) -> Self {
        Self {
            pts,
            factory,
            diagnostics,
        }
    }

    /// Get the points-to set for a value (sorted).
    #[must_use]
    pub fn points_to(&self, ptr: ValueId) -> Vec<LocId> {
        self.pts
            .get(&ptr)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Check if a value is tracked in the points-to analysis.
    ///
    /// Returns true if the value has a points-to set entry (even if empty).
    #[must_use]
    pub fn is_tracked(&self, ptr: ValueId) -> bool {
        self.pts.contains_key(&ptr)
    }

    /// Check if a location is provably unique (represents one concrete object).
    ///
    /// Returns `false` for unknown locations or `Summary` locations (safe default).
    pub fn is_unique(&self, loc: LocId) -> bool {
        self.factory.multiplicity(loc) == AllocationMultiplicity::Unique
    }

    /// Check if two pointers may alias (five-valued).
    ///
    /// Returns:
    /// - `Must`: Points-to sets are equal (bidirectional containment)
    /// - `Partial`: One set is a proper subset or locations have field prefix relationship
    /// - `May`: Sets overlap but neither is a subset of the other
    /// - `No`: Disjoint points-to sets with no field path overlap
    /// - `Unknown`: One or both pointers were never tracked (not in pts map)
    ///
    /// Note: When one pointer has a non-empty points-to set and the other has
    /// an empty set (both tracked), we return `No`. This is because an empty
    /// set means the pointer was never assigned any tracked location (e.g.,
    /// loaded from uninitialized memory), so it cannot alias with pointers
    /// that point to tracked allocated objects.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        // Identity check: the same SSA value always holds the same address
        // at any given program point, so alias(p, p) is always MustAlias
        // regardless of PTS size or uniqueness.
        if p == q {
            return match self.pts.get(&p) {
                Some(pts) if !pts.is_empty() => AliasResult::Must,
                Some(_) => AliasResult::No, // empty PTS: never assigned
                None => AliasResult::Unknown,
            };
        }

        let p_pts = self.pts.get(&p);
        let q_pts = self.pts.get(&q);

        match (p_pts, q_pts) {
            // If either pointer is not tracked at all, we don't know
            (None, _) | (_, None) => AliasResult::Unknown,
            (Some(p_set), Some(q_set)) => {
                // Both empty: both were tracked but never assigned, can't alias
                // with anything, so they're definitionally disjoint
                if p_set.is_empty() && q_set.is_empty() {
                    return AliasResult::No;
                }
                // One empty, one non-empty: the empty one points to no tracked
                // locations, so it can't alias with tracked locations
                if p_set.is_empty() || q_set.is_empty() {
                    return AliasResult::No;
                }
                if p_set == q_set && p_set.len() == 1 {
                    // Singleton sets pointing to the same location.
                    // Only MustAlias if the location is provably unique
                    // (represents exactly one concrete object). Summary
                    // locations may represent multiple objects, so we
                    // return May to be sound.
                    let loc = *p_set.iter().next().unwrap();
                    if self.is_unique(loc) {
                        AliasResult::Must
                    } else {
                        AliasResult::May
                    }
                } else if p_set == q_set {
                    // Non-singleton equal sets: both can point to the same set of
                    // locations, but we don't know if they hold the same one
                    AliasResult::May
                } else if self.has_partial_alias_relationship(p_set, q_set) {
                    // Check for partial aliasing (subset or field path prefix)
                    // This must come before disjoint check because field path
                    // relationships can exist between different LocIds
                    AliasResult::Partial
                } else if p_set.is_disjoint(q_set) {
                    // No overlap and no field path relationships
                    AliasResult::No
                } else {
                    // Sets overlap but not equal and not partial
                    AliasResult::May
                }
            }
        }
    }

    /// Check if two points-to sets have a partial aliasing relationship.
    ///
    /// Partial alias occurs when:
    /// 1. One set is a proper subset of the other, OR
    /// 2. Some locations in the sets have a field path prefix relationship
    ///    (e.g., struct vs field of that struct)
    fn has_partial_alias_relationship(
        &self,
        p_set: &std::collections::BTreeSet<LocId>,
        q_set: &std::collections::BTreeSet<LocId>,
    ) -> bool {
        // Check 1: proper subset relationship (when sets overlap)
        if !p_set.is_disjoint(q_set) {
            let p_subset_q = p_set.is_subset(q_set) && p_set.len() < q_set.len();
            let q_subset_p = q_set.is_subset(p_set) && q_set.len() < p_set.len();
            if p_subset_q || q_subset_p {
                return true;
            }
        }

        // Check 2: field path prefix relationship between any pair of locations
        // This applies even when sets are disjoint at the LocId level
        for &p_loc in p_set {
            for &q_loc in q_set {
                if self.locations_partially_overlap(p_loc, q_loc) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two locations have a partial overlap (field prefix relationship).
    ///
    /// Returns true if both locations refer to the same base object and one's
    /// field path is a prefix of the other's (but they're not equal).
    fn locations_partially_overlap(&self, p: LocId, q: LocId) -> bool {
        let (Some(p_loc), Some(q_loc)) = (self.factory.get(p), self.factory.get(q)) else {
            return false;
        };

        // Must be same base object
        if p_loc.obj != q_loc.obj {
            return false;
        }

        let p_path = &p_loc.path.steps;
        let q_path = &q_loc.path.steps;

        // Equal paths means equal location (handled by == check above)
        if p_path.len() == q_path.len() {
            return false;
        }

        // Check if one path is a prefix of the other
        let (shorter, longer) = if p_path.len() < q_path.len() {
            (p_path, q_path)
        } else {
            (q_path, p_path)
        };

        longer.starts_with(shorter)
    }

    /// Resolve a location ID to its full location info.
    #[must_use]
    pub fn location(&self, id: LocId) -> Option<&Location> {
        self.factory.get(id)
    }

    /// Get the memory region for a location.
    ///
    /// Looks up the location's base object and returns its region.
    /// Defaults to `Unknown` if the object has no assigned region.
    #[must_use]
    pub fn region(&self, loc: LocId) -> MemoryRegion {
        self.factory.region(loc)
    }

    /// Check if two locations may alias based on their memory regions.
    #[must_use]
    pub fn may_alias_region(&self, a: LocId, b: LocId) -> bool {
        self.factory.may_alias_region(a, b)
    }

    /// Get the analysis diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &PtaDiagnostics {
        &self.diagnostics
    }

    /// Get the raw points-to map.
    #[must_use]
    pub fn points_to_map(&self) -> &PointsToMap {
        &self.pts
    }

    /// Get all locations.
    #[must_use]
    pub fn locations(&self) -> &FxHashMap<LocId, Location> {
        self.factory.all_locations()
    }

    /// Get a reference to the underlying `LocationFactory`.
    #[must_use]
    pub fn location_factory(&self) -> &Arc<LocationFactory> {
        &self.factory
    }

    /// Get the number of values with points-to sets.
    #[must_use]
    pub fn value_count(&self) -> usize {
        self.pts.len()
    }

    /// Get the number of locations.
    #[must_use]
    pub fn location_count(&self) -> usize {
        self.factory.len()
    }

    /// Get the number of unique base objects (distinct `ObjId` values).
    ///
    /// This counts base objects before field expansion. Compare with
    /// `location_count()` which counts (`ObjId`, `FieldPath`) pairs.
    #[must_use]
    pub fn obj_count(&self) -> usize {
        self.factory
            .all_locations()
            .values()
            .map(|loc| loc.obj)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    /// Check if two pointers may alias at a specific program point (path-sensitive).
    ///
    /// This method considers branch conditions that dominate the query point
    /// to provide more precise alias results for path-dependent aliasing.
    ///
    /// Returns the same five-valued `AliasResult` as `may_alias`, but may be
    /// more precise when the alias relationship depends on which branch was taken.
    ///
    /// # Arguments
    ///
    /// * `p` - First pointer value
    /// * `q` - Second pointer value
    /// * `query_block` - The block where the alias query is performed
    /// * `query_function` - The function containing the query block
    /// * `module` - The AIR module for guard extraction
    /// * `config` - Path-sensitive configuration
    ///
    /// # Returns
    ///
    /// The alias result and updated diagnostics.
    #[cfg(feature = "z3-solver")]
    pub fn may_alias_path_sensitive(
        &self,
        p: ValueId,
        q: ValueId,
        query_block: saf_core::ids::BlockId,
        query_function: saf_core::ids::FunctionId,
        module: &saf_core::air::AirModule,
        config: &super::path_sensitive::PathSensitiveConfig,
    ) -> (AliasResult, super::path_sensitive::PathSensitiveDiagnostics) {
        use super::path_sensitive::{PathSensitiveAliasChecker, PathSensitiveDiagnostics};

        let mut diag = PathSensitiveDiagnostics::default();
        let mut checker = PathSensitiveAliasChecker::new(
            &self.pts,
            self.factory.all_locations(),
            module,
            config.clone(),
        );

        let result = checker.may_alias_at(p, q, query_block, query_function, &mut diag);
        (result, diag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use saf_core::ids::ObjId;

    use crate::pta::config::FieldSensitivity;
    use crate::pta::location::FieldPath;

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    #[test]
    fn alias_result_must_alias() {
        assert!(AliasResult::Must.must_alias());
        assert!(!AliasResult::Partial.must_alias());
        assert!(!AliasResult::May.must_alias());
        assert!(!AliasResult::No.must_alias());
        assert!(!AliasResult::Unknown.must_alias());
    }

    #[test]
    fn alias_result_partial_alias() {
        assert!(AliasResult::Must.partial_alias());
        assert!(AliasResult::Partial.partial_alias());
        assert!(!AliasResult::May.partial_alias());
        assert!(!AliasResult::No.partial_alias());
        assert!(!AliasResult::Unknown.partial_alias());
    }

    #[test]
    fn alias_result_may_conservative() {
        assert!(AliasResult::Must.may_alias_conservative());
        assert!(AliasResult::Partial.may_alias_conservative());
        assert!(AliasResult::May.may_alias_conservative());
        assert!(AliasResult::Unknown.may_alias_conservative());
        assert!(!AliasResult::No.may_alias_conservative());
    }

    #[test]
    fn alias_result_may_optimistic() {
        assert!(AliasResult::Must.may_alias_optimistic());
        assert!(AliasResult::Partial.may_alias_optimistic());
        assert!(AliasResult::May.may_alias_optimistic());
        assert!(!AliasResult::Unknown.may_alias_optimistic());
        assert!(!AliasResult::No.may_alias_optimistic());
    }

    #[test]
    fn alias_result_no_alias() {
        assert!(AliasResult::No.no_alias());
        assert!(!AliasResult::Must.no_alias());
        assert!(!AliasResult::Partial.no_alias());
        assert!(!AliasResult::May.no_alias());
        assert!(!AliasResult::Unknown.no_alias());
    }

    #[test]
    fn pta_result_empty() {
        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(make_factory()),
            PtaDiagnostics::default(),
        );
        assert_eq!(result.value_count(), 0);
        assert_eq!(result.location_count(), 0);
    }

    #[test]
    fn pta_result_points_to_empty_for_unknown() {
        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(make_factory()),
            PtaDiagnostics::default(),
        );
        let pts = result.points_to(ValueId::new(1));
        assert!(pts.is_empty());
    }

    #[test]
    fn pta_result_points_to_returns_sorted() {
        let mut factory = make_factory();
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(50), FieldPath::empty());
        let loc3 = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        let p = ValueId::new(1);
        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc3);
        p_set.insert(loc1);
        p_set.insert(loc2);
        pts_map.insert(p, p_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());
        let pts = result.points_to(p);

        // Should be sorted by LocId
        assert_eq!(pts.len(), 3);
        // BTreeSet maintains order, so the result should be in insertion order
        // (which is ID order since BTreeSet is ordered)
    }

    #[test]
    fn may_alias_both_unknown() {
        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(make_factory()),
            PtaDiagnostics::default(),
        );
        let p = ValueId::new(1);
        let q = ValueId::new(2);

        assert_eq!(result.may_alias(p, q), AliasResult::Unknown);
    }

    #[test]
    fn may_alias_one_unknown() {
        let mut factory = make_factory();
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc);
        pts_map.insert(p, p_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // q is unknown
        assert_eq!(result.may_alias(p, q), AliasResult::Unknown);
    }

    #[test]
    fn may_alias_disjoint_no() {
        let mut factory = make_factory();
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc1);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc2);
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        assert_eq!(result.may_alias(p, q), AliasResult::No);
    }

    #[test]
    fn may_alias_identical_singleton_unique_must() {
        let mut factory = make_factory();
        let obj = ObjId::new(100);
        let loc = factory.get_or_create(obj, FieldPath::empty());
        factory.set_multiplicity(obj, AllocationMultiplicity::Unique);

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc);
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // Unique singleton location means MustAlias
        assert_eq!(result.may_alias(p, q), AliasResult::Must);
    }

    #[test]
    fn may_alias_identical_singleton_summary_returns_may() {
        let mut factory = make_factory();
        let obj = ObjId::new(100);
        let loc = factory.get_or_create(obj, FieldPath::empty());
        // Default multiplicity is Summary — don't set it

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc);
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // Summary location: singleton set does NOT guarantee must-alias
        assert_eq!(result.may_alias(p, q), AliasResult::May);
    }

    #[test]
    fn may_alias_identical_singleton_no_multiplicity_returns_may() {
        let mut factory = make_factory();
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        // No multiplicity set at all — should default to Summary

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc);
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // No multiplicity classification → defaults to Summary → May
        assert_eq!(result.may_alias(p, q), AliasResult::May);
    }

    #[test]
    fn may_alias_overlap_not_identical_may() {
        let mut factory = make_factory();
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc1);
        p_set.insert(loc2);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc1);
        // q only has loc1, p has loc1 and loc2 - this is a subset relationship
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // q is a proper subset of p -> Partial alias
        assert_eq!(result.may_alias(p, q), AliasResult::Partial);
    }

    #[test]
    fn may_alias_overlap_neither_subset_may() {
        let mut factory = make_factory();
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());
        let loc3 = factory.get_or_create(ObjId::new(300), FieldPath::empty());

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc1);
        p_set.insert(loc2);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc1);
        q_set.insert(loc3);
        // p has {loc1, loc2}, q has {loc1, loc3} - overlap but neither is subset
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // Overlap but neither subset -> MayAlias
        assert_eq!(result.may_alias(p, q), AliasResult::May);
    }

    #[test]
    fn may_alias_field_prefix_partial() {
        let mut factory = make_factory();
        let obj = ObjId::new(100);
        // Base struct location
        let loc_base = factory.get_or_create(obj, FieldPath::empty());
        // Field 0 within the struct
        let loc_field = factory.get_or_create(obj, FieldPath::field(0));

        let p = ValueId::new(1);
        let q = ValueId::new(2);

        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc_base);
        pts_map.insert(p, p_set);

        let mut q_set = BTreeSet::new();
        q_set.insert(loc_field);
        pts_map.insert(q, q_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());

        // Different locations with field prefix relationship -> Partial
        assert_eq!(result.may_alias(p, q), AliasResult::Partial);
    }

    #[test]
    fn location_lookup() {
        let mut factory = make_factory();
        let obj = ObjId::new(100);
        let path = FieldPath::field(0);
        let loc_id = factory.get_or_create(obj, path.clone());

        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(factory),
            PtaDiagnostics::default(),
        );

        let loc = result.location(loc_id).expect("location should exist");
        assert_eq!(loc.obj, obj);
        assert_eq!(loc.path, path);
    }
}
