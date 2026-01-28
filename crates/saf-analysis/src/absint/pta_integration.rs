//! PTA integration layer for abstract interpretation.
//!
//! Provides an abstraction over `PtaResult` optimized for absint queries,
//! with caching and convenience methods.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{FunctionId, LocId, ValueId};

use crate::PtaResult;

/// Wrapper providing absint-friendly PTA queries.
///
/// Caches points-to lookups to avoid repeated traversals and provides
/// convenience methods for common absint patterns.
pub struct PtaIntegration<'a> {
    /// The underlying PTA result (None for empty/no-PTA mode).
    pta: Option<&'a PtaResult>,
    /// Cache for points-to queries.
    pts_cache: RefCell<BTreeMap<ValueId, BTreeSet<LocId>>>,
}

impl<'a> PtaIntegration<'a> {
    /// Create a new integration layer wrapping a PTA result.
    #[must_use]
    pub fn new(pta: &'a PtaResult) -> Self {
        Self {
            pta: Some(pta),
            pts_cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Create an empty integration (no PTA available).
    ///
    /// All queries return empty/conservative results.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            pta: None,
            pts_cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Get the points-to set for a pointer value.
    ///
    /// Returns an empty set if PTA is not available or pointer is not tracked.
    #[must_use]
    pub fn points_to(&self, ptr: ValueId) -> BTreeSet<LocId> {
        // Check cache first
        if let Some(cached) = self.pts_cache.borrow().get(&ptr) {
            return cached.clone();
        }

        let result = match self.pta {
            Some(pta_ref) => pta_ref.points_to(ptr).into_iter().collect(),
            None => BTreeSet::new(),
        };

        // Cache the result
        self.pts_cache.borrow_mut().insert(ptr, result.clone());
        result
    }

    /// Check if a pointer has a singleton points-to set.
    ///
    /// Returns true if the pointer points to exactly one location,
    /// enabling strong updates in the memory model.
    #[must_use]
    pub fn is_singleton(&self, ptr: ValueId) -> bool {
        self.points_to(ptr).len() == 1
    }

    /// Check if two pointers may alias.
    ///
    /// Returns true conservatively if PTA is not available.
    #[must_use]
    pub fn may_alias(&self, a: ValueId, b: ValueId) -> bool {
        match self.pta {
            Some(pta) => pta.may_alias(a, b).may_alias_conservative(),
            None => true, // Conservative: assume may-alias
        }
    }

    /// Resolve indirect call targets for a function pointer.
    ///
    /// Returns the set of possible callee function IDs.
    /// Returns empty set if PTA is not available or no targets found.
    #[must_use]
    pub fn resolve_indirect_call(&self, fn_ptr: ValueId) -> BTreeSet<FunctionId> {
        let Some(pta_ref) = self.pta else {
            return BTreeSet::new();
        };

        let pt_set = self.points_to(fn_ptr);
        let mut targets = BTreeSet::new();

        for loc_id in pt_set {
            if let Some(loc) = pta_ref.location(loc_id) {
                // Function locations have ObjId that maps to FunctionId
                // The ObjId for a function is derived from its FunctionId
                targets.insert(FunctionId::new(loc.obj.raw()));
            }
        }

        targets
    }

    /// Check if PTA is available.
    #[must_use]
    pub fn has_pta(&self) -> bool {
        self.pta.is_some()
    }

    /// Get a reference to the underlying `PtaResult` (if available).
    #[must_use]
    pub fn pta_ref(&self) -> Option<&'a PtaResult> {
        self.pta
    }

    /// Clear the cache (useful after state changes).
    pub fn clear_cache(&self) {
        self.pts_cache.borrow_mut().clear();
    }

    /// Check if two locations share the same base object (ObjId).
    ///
    /// This is more permissive than may_alias, catching cases where
    /// different GEP paths to the same object produce different LocIds.
    #[must_use]
    pub fn locations_share_object(&self, loc1: LocId, loc2: LocId) -> bool {
        let Some(pta) = self.pta else {
            return false;
        };

        let l1 = pta.location(loc1);
        let l2 = pta.location(loc2);

        match (l1, l2) {
            (Some(l1), Some(l2)) => l1.obj == l2.obj,
            _ => false,
        }
    }

    /// Get the ObjId for a location (if available).
    ///
    /// Returns the base object ID for field-sensitive location matching.
    #[must_use]
    pub fn object_of_location(&self, loc: LocId) -> Option<saf_core::ids::ObjId> {
        let pta = self.pta?;
        pta.location(loc).map(|l| l.obj)
    }

    /// Find all locations that share the same base object as `loc`.
    ///
    /// Used by memset/memcpy to propagate intervals to all element-level
    /// sub-locations of an allocation.
    #[must_use]
    pub fn locations_of_same_object(&self, loc: LocId) -> BTreeSet<LocId> {
        let mut result = BTreeSet::new();
        let Some(pta) = self.pta else {
            return result;
        };
        let Some(base) = pta.location(loc) else {
            return result;
        };
        let obj = base.obj;
        for (lid, location) in pta.locations() {
            if location.obj == obj {
                result.insert(*lid);
            }
        }
        result
    }

    /// Find location with matching ObjId and constant index.
    ///
    /// When we know a GEP result should point to a specific array element
    /// (because the index interval is a singleton), this method helps find
    /// the precise location by searching all known locations.
    ///
    /// Returns the location if found, or None if not found.
    #[must_use]
    pub fn find_location_with_index(&self, base_loc: LocId, constant_index: i64) -> Option<LocId> {
        let pta = self.pta?;
        let base_location = pta.location(base_loc)?;
        let base_obj = base_location.obj;
        let base_depth = base_location.path.steps.len();

        // Search all locations for one with matching ObjId, path depth, and constant index
        // This is a linear search but locations are typically bounded
        for (loc_id, location) in pta.locations() {
            if location.obj != base_obj {
                continue;
            }

            // Require same path depth to prevent cross-depth matches.
            // Without this, a GEP accessing `a[c].b` (4-step path) could match
            // `a[1]` (2-step path) because both have last step Field{1}.
            if location.path.steps.len() != base_depth {
                continue;
            }

            // Check if this location has the matching constant index
            // We look for Index(Constant(n)) OR Field { index: n } where n == constant_index
            // Field steps are used for small arrays that are modeled as structs with numbered fields
            if let Some(last_step) = location.path.steps.last() {
                let matches = match last_step {
                    crate::pta::PathStep::Index(crate::pta::IndexExpr::Constant(idx)) => {
                        *idx == constant_index
                    }
                    crate::pta::PathStep::Field { index } => {
                        #[allow(clippy::cast_possible_wrap)]
                        let field_as_i64 = i64::from(*index);
                        field_as_i64 == constant_index
                    }
                    crate::pta::PathStep::Index(_) => false,
                };

                if matches {
                    return Some(*loc_id);
                }
            }
        }

        None
    }

    /// Find a child element location under a base location with a specific index.
    ///
    /// When PTA approximated a variable-index GEP to its parent location
    /// (e.g., path `[Field{0}]` instead of `[Field{0}, Field{1}]`), this
    /// method finds the precise child location by searching for locations
    /// with the parent path as prefix plus a matching index step.
    #[must_use]
    pub fn find_child_location_with_index(
        &self,
        base_loc: LocId,
        constant_index: i64,
    ) -> Option<LocId> {
        let pta = self.pta?;
        let base_location = pta.location(base_loc)?;
        let base_obj = base_location.obj;
        let base_depth = base_location.path.steps.len();

        // Search for any descendant location with matching last step.
        // PTA may model arrays with structural wrappers (e.g., LLVM [N x T]
        // becomes a struct with N fields), so elements can be at depth+2
        // or deeper rather than just depth+1.
        for (loc_id, location) in pta.locations() {
            if location.obj != base_obj {
                continue;
            }
            // Must be deeper than base
            if location.path.steps.len() <= base_depth {
                continue;
            }
            // Prefix must match (if base has a non-empty path)
            if base_depth > 0 && location.path.steps[..base_depth] != base_location.path.steps[..] {
                continue;
            }
            // Last step must match the index
            if let Some(last_step) = location.path.steps.last() {
                let matches = match last_step {
                    crate::pta::PathStep::Index(crate::pta::IndexExpr::Constant(idx)) => {
                        *idx == constant_index
                    }
                    crate::pta::PathStep::Field { index } => {
                        #[allow(clippy::cast_possible_wrap)]
                        let field_as_i64 = i64::from(*index);
                        field_as_i64 == constant_index
                    }
                    crate::pta::PathStep::Index(_) => false,
                };
                if matches {
                    return Some(*loc_id);
                }
            }
        }
        None
    }

    /// Refine GEP targets by finding child element locations.
    ///
    /// When PTA collapsed a variable-index GEP to a parent location,
    /// and the absint knows the concrete index, find the element location.
    #[must_use]
    pub fn refine_gep_targets_by_child_index(
        &self,
        targets: &std::collections::BTreeSet<LocId>,
        index_interval: &crate::absint::Interval,
    ) -> std::collections::BTreeSet<LocId> {
        let Some(constant_index) = index_interval.as_singleton() else {
            return targets.clone();
        };

        let mut refined = std::collections::BTreeSet::new();
        for &base_loc in targets {
            if let Some(child_loc) = self.find_child_location_with_index(base_loc, constant_index) {
                refined.insert(child_loc);
            } else {
                refined.insert(base_loc);
            }
        }
        refined
    }

    /// Check whether any target location has unresolved `Index(Unknown)` steps.
    ///
    /// Returns `true` if refinement is potentially useful (at least one location
    /// has an `Index(Unknown)` step in its path). Returns `false` when all
    /// locations have fully-resolved paths (only `Field` or `Index(Constant)` steps),
    /// meaning PTA already produced the correct specific location.
    #[must_use]
    pub fn targets_have_unresolved_index(
        &self,
        targets: &std::collections::BTreeSet<LocId>,
    ) -> bool {
        let Some(pta) = self.pta else {
            return false;
        };
        for &loc in targets {
            if let Some(location) = pta.location(loc) {
                for step in &location.path.steps {
                    if matches!(
                        step,
                        crate::pta::PathStep::Index(crate::pta::IndexExpr::Unknown)
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Refine GEP targets using the base pointer's resolved GEP targets.
    ///
    /// When a chained GEP has `Index(Unknown)` in its target path, and the base
    /// pointer's GEP targets were already resolved (e.g., from an earlier GEP
    /// that selected a specific array element), substitute the resolved prefix
    /// to find the precise location.
    ///
    /// Example: GEP 1 resolves `a[c]` → `[Field{0}, Field{0}]` (a[0]).
    /// GEP 2 accesses `.b` → PTA gives `[Field{0}, Index(Unknown), Field{0}, Field{1}]`.
    /// This method substitutes the Unknown prefix with the base's resolved path,
    /// constructing `[Field{0}, Field{0}, Field{0}, Field{1}]` and finding `a[0].b`.
    #[must_use]
    pub fn refine_gep_by_base_targets(
        &self,
        targets: &std::collections::BTreeSet<LocId>,
        base_targets: &std::collections::BTreeSet<LocId>,
    ) -> std::collections::BTreeSet<LocId> {
        let Some(pta) = self.pta else {
            return targets.clone();
        };

        let mut refined = std::collections::BTreeSet::new();

        for &target_loc in targets {
            let Some(target_location) = pta.location(target_loc) else {
                refined.insert(target_loc);
                continue;
            };
            let target_path = &target_location.path.steps;

            // Find the first Index(Unknown) step
            let unknown_pos = target_path.iter().position(|s| {
                matches!(
                    s,
                    crate::pta::PathStep::Index(crate::pta::IndexExpr::Unknown)
                )
            });
            let Some(unknown_pos) = unknown_pos else {
                refined.insert(target_loc);
                continue;
            };

            // The suffix is everything after the Unknown step
            let suffix = &target_path[unknown_pos + 1..];

            let mut found_refinement = false;
            for &base_loc in base_targets {
                let Some(base_location) = pta.location(base_loc) else {
                    continue;
                };
                let base_path = &base_location.path.steps;

                // Base path should extend exactly to the Unknown step position
                // (the base GEP resolved the array index, giving unknown_pos+1 steps)
                if base_path.len() != unknown_pos + 1 {
                    continue;
                }

                // Verify the prefix before Unknown matches
                if target_path[..unknown_pos] != base_path[..unknown_pos] {
                    continue;
                }

                // Construct the resolved path: base_path + suffix
                let mut resolved_path = base_path.clone();
                resolved_path.extend_from_slice(suffix);

                // Search for a location with this resolved path and same ObjId
                for (loc_id, location) in pta.locations() {
                    if location.obj == target_location.obj && location.path.steps == resolved_path {
                        refined.insert(*loc_id);
                        found_refinement = true;
                        break;
                    }
                }
            }

            if !found_refinement {
                refined.insert(target_loc);
            }
        }

        refined
    }

    /// Refine GEP targets using known index interval.
    ///
    /// When the index operand has a singleton interval (e.g., [1, 1]),
    /// we can refine the GEP target to the specific array element location.
    ///
    /// Returns a refined set of locations, or the original if no refinement possible.
    #[must_use]
    pub fn refine_gep_targets_with_index(
        &self,
        targets: &std::collections::BTreeSet<LocId>,
        index_interval: &crate::absint::Interval,
    ) -> std::collections::BTreeSet<LocId> {
        // Only refine if the index is a singleton (known constant)
        let Some(constant_index) = index_interval.as_singleton() else {
            return targets.clone();
        };

        let mut refined = std::collections::BTreeSet::new();

        for &base_loc in targets {
            // Try to find the location with matching constant index
            if let Some(precise_loc) = self.find_location_with_index(base_loc, constant_index) {
                refined.insert(precise_loc);
            } else {
                // Couldn't find a precise location, keep the original
                refined.insert(base_loc);
            }
        }

        refined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::{FieldSensitivity, LocationFactory, PointsToMap, PtaDiagnostics};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn lid(n: u128) -> LocId {
        LocId::new(n)
    }

    #[test]
    fn pta_integration_empty_returns_empty_pts() {
        let integration = PtaIntegration::empty();
        assert!(integration.points_to(vid(1)).is_empty());
    }

    #[test]
    fn pta_integration_is_singleton() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(pts, Arc::new(factory), PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        assert!(integration.is_singleton(vid(1)));
    }

    #[test]
    fn pta_integration_not_singleton_for_multiple() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        set.insert(lid(101));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(pts, Arc::new(factory), PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        assert!(!integration.is_singleton(vid(1)));
    }

    #[test]
    fn pta_integration_may_alias_conservative_without_pta() {
        let integration = PtaIntegration::empty();
        assert!(integration.may_alias(vid(1), vid(2)));
    }

    #[test]
    fn pta_integration_caches_queries() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(pts, Arc::new(factory), PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        // First query populates cache
        let result1 = integration.points_to(vid(1));
        // Second query should hit cache
        let result2 = integration.points_to(vid(1));

        assert_eq!(result1, result2);
        assert_eq!(integration.pts_cache.borrow().len(), 1);
    }

    #[test]
    fn pta_integration_has_pta() {
        let empty = PtaIntegration::empty();
        assert!(!empty.has_pta());

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(
            PointsToMap::new(),
            Arc::new(factory),
            PtaDiagnostics::default(),
        );
        let with_pta = PtaIntegration::new(&pta);
        assert!(with_pta.has_pta());
    }

    #[test]
    fn pta_integration_clear_cache() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(pts, Arc::new(factory), PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        // Populate cache
        let _ = integration.points_to(vid(1));
        assert_eq!(integration.pts_cache.borrow().len(), 1);

        // Clear cache
        integration.clear_cache();
        assert!(integration.pts_cache.borrow().is_empty());
    }
}
