//! GEP (field-sensitive) constraint pre-resolution for Ascent PTA.
//!
//! The Ascent solver handles addr, copy, store, and load rules natively.
//! GEP constraints model struct field access (`dst = &src_ptr->field`) and
//! require `LocationFactory` access to resolve field paths into concrete
//! `LocId` values. Since `LocationFactory` cannot be accessed inside Ascent
//! rules, we pre-resolve GEP constraints before solving:
//!
//! 1. For each GEP fact `(dst, src_ptr, path)`, look up what `src_ptr` already
//!    points to via the `addr_of` facts.
//! 2. For each base location, compute the field-qualified location using
//!    `LocationFactory::get_or_create` (with merge support for
//!    pointer-arithmetic GEPs).
//! 3. Emit a new `addr_of(dst, field_loc)` fact for each resolved field
//!    location.
//! 4. If `src_ptr` has no known `addr_of` targets, fall back to a copy edge
//!    `dst = src_ptr` so the points-to set propagates conservatively.
//!
//! This is a static, single-pass pre-resolution. A more complete approach
//! would iterate until fixpoint (GEPs can depend on other GEPs), but the
//! existing imperative solver also pre-creates field locations in a single
//! pass via `precompute_indexed_locations`.

use std::collections::{BTreeMap, BTreeSet};

use saf_analysis::{FieldPath, LocationFactory, merge_gep_with_base_path};
use saf_core::ids::{LocId, ValueId};

use crate::facts::PtaFacts;
use crate::pta::solver::PointsToMap;

/// Result of partial GEP resolution attempt.
///
/// Unlike [`resolve_gep_facts_with_pts`] which adds copy fallback edges for
/// unresolved GEPs, [`try_resolve_gep_facts`] returns unresolved GEPs in this
/// struct so callers can iterate until fixpoint.
pub struct GepResolutionResult {
    /// Number of GEP facts successfully resolved to `addr_of` entries.
    pub resolved_count: usize,
    /// GEP facts that could not be resolved because `src_ptr` was not
    /// found in the points-to map. These can be retried after re-solving.
    pub unresolved: Vec<(ValueId, ValueId, FieldPath)>,
}

/// Propagate `addr_of` locations through copy edges until fixpoint.
///
/// For each copy edge `(dst, src)`, adds all locations known for `src` to `dst`.
/// Repeats until no new locations are discovered. This ensures that GEP
/// resolution can find locations for pointers that receive their targets
/// through copy chains rather than direct `addr_of` facts.
fn propagate_addr_through_copies(
    addr_map: &mut BTreeMap<ValueId, BTreeSet<LocId>>,
    copies: &[(ValueId, ValueId)],
) {
    if copies.is_empty() {
        return;
    }

    // Iterate until fixpoint
    loop {
        let mut changed = false;
        // For each copy edge src -> dst, propagate src's locations to dst
        for &(dst, src) in copies {
            let src_locs = addr_map.get(&src).cloned().unwrap_or_default();
            if !src_locs.is_empty() {
                let dst_entry = addr_map.entry(dst).or_default();
                let before = dst_entry.len();
                dst_entry.extend(src_locs);
                if dst_entry.len() > before {
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
}

/// Pre-resolve GEP constraints by computing field-qualified locations.
///
/// For each GEP fact `(dst, src_ptr, path)`:
/// - Builds a map from `ValueId` to `BTreeSet<LocId>` from existing `addr_of` facts
/// - For each base location that `src_ptr` points to, computes the field location
///   via pointer-arithmetic merge or path extension
/// - Adds `addr_of(dst, field_loc)` for each resolved field location
/// - Falls back to `copy(dst, src_ptr)` when `src_ptr` has no known locations
///
/// After resolution, all GEP facts are cleared from the `PtaFacts` -- the solver
/// only needs to process `addr_of`, `copy`, `load`, and `store`.
pub fn resolve_gep_facts(facts: &mut PtaFacts, factory: &mut LocationFactory) {
    // Build a map: ValueId -> Set<LocId> from addr_of facts
    let mut addr_map: BTreeMap<ValueId, BTreeSet<LocId>> = BTreeMap::new();
    for &(ptr, loc) in &facts.addr_of {
        addr_map.entry(ptr).or_default().insert(loc);
    }

    // Propagate addr_of through copy edges until fixpoint.
    // This ensures GEP resolution sees locations from copy chains,
    // e.g., `q = p; r = &q->field` — q inherits p's locations.
    propagate_addr_through_copies(&mut addr_map, &facts.copy);

    let mut new_addr_of: Vec<(ValueId, LocId)> = Vec::new();
    let mut new_copy: Vec<(ValueId, ValueId)> = Vec::new();

    for (dst, src_ptr, path) in &facts.gep {
        let mut resolved_any = false;

        if let Some(src_locs) = addr_map.get(src_ptr) {
            for &base_loc_id in src_locs {
                // Clone location data before mutably borrowing factory
                let loc_snapshot = factory.get(base_loc_id).map(|loc| {
                    (
                        loc.obj,
                        loc.path.clone(),
                        merge_gep_with_base_path(loc, path),
                    )
                });
                if let Some((obj, base_path, merged)) = loc_snapshot {
                    let field_loc_id = if let Some(merged_path) = merged {
                        factory.get_or_create(obj, merged_path)
                    } else {
                        let new_path = base_path.extend(path);
                        factory.get_or_create(obj, new_path)
                    };

                    new_addr_of.push((*dst, field_loc_id));
                    resolved_any = true;
                }
            }
        }

        if !resolved_any {
            // src_ptr has no known addr_of targets; fall back to copy
            // so the points-to set propagates conservatively
            new_copy.push((*dst, *src_ptr));
        }
    }

    facts.addr_of.extend(new_addr_of);
    facts.copy.extend(new_copy);
    facts.gep.clear();
}

/// Resolve GEP facts using a pre-computed points-to map.
///
/// Like [`resolve_gep_facts`], but uses a full [`PointsToMap`] (from a
/// preliminary Ascent solve) to look up what each `src_ptr` points to,
/// rather than only using `addr_of` facts. This captures locations
/// reachable via `store`/`load` chains that single-pass resolution misses.
pub fn resolve_gep_facts_with_pts(
    facts: &mut PtaFacts,
    factory: &mut LocationFactory,
    pts: &PointsToMap,
) {
    // Build addr_map from the full points-to map
    let addr_map: BTreeMap<ValueId, BTreeSet<LocId>> =
        pts.iter().map(|(v, locs)| (*v, locs.clone())).collect();

    let mut new_addr_of: Vec<(ValueId, LocId)> = Vec::new();
    let mut new_copy: Vec<(ValueId, ValueId)> = Vec::new();

    for (dst, src_ptr, path) in &facts.gep {
        let mut resolved_any = false;

        if let Some(src_locs) = addr_map.get(src_ptr) {
            for &base_loc_id in src_locs {
                let loc_snapshot = factory.get(base_loc_id).map(|loc| {
                    (
                        loc.obj,
                        loc.path.clone(),
                        merge_gep_with_base_path(loc, path),
                    )
                });
                if let Some((obj, base_path, merged)) = loc_snapshot {
                    let field_loc_id = if let Some(merged_path) = merged {
                        factory.get_or_create(obj, merged_path)
                    } else {
                        let new_path = base_path.extend(path);
                        factory.get_or_create(obj, new_path)
                    };

                    new_addr_of.push((*dst, field_loc_id));
                    resolved_any = true;
                }
            }
        }

        if !resolved_any {
            new_copy.push((*dst, *src_ptr));
        }
    }

    facts.addr_of.extend(new_addr_of);
    facts.copy.extend(new_copy);
    facts.gep.clear();
}

/// Resolve GEP facts using a points-to map, without copy fallbacks.
///
/// Like [`resolve_gep_facts_with_pts`], but instead of adding conservative
/// copy edges for unresolved GEPs, returns them in [`GepResolutionResult`]
/// so callers can iterate: re-solve with newly resolved facts, then retry
/// unresolved GEPs with the updated points-to map.
///
/// This enables iterative GEP resolution that handles GEP-to-GEP
/// dependencies and virtual dispatch chains where `src_ptr` receives its
/// points-to set only after callgraph refinement.
pub fn try_resolve_gep_facts(
    facts: &mut PtaFacts,
    factory: &mut LocationFactory,
    pts: &PointsToMap,
) -> GepResolutionResult {
    let addr_map: BTreeMap<ValueId, BTreeSet<LocId>> =
        pts.iter().map(|(v, locs)| (*v, locs.clone())).collect();

    let mut new_addr_of: Vec<(ValueId, LocId)> = Vec::new();
    let mut unresolved: Vec<(ValueId, ValueId, FieldPath)> = Vec::new();
    let mut resolved_count = 0;

    for (dst, src_ptr, path) in &facts.gep {
        let mut resolved_any = false;

        if let Some(src_locs) = addr_map.get(src_ptr) {
            for &base_loc_id in src_locs {
                let loc_snapshot = factory.get(base_loc_id).map(|loc| {
                    (
                        loc.obj,
                        loc.path.clone(),
                        merge_gep_with_base_path(loc, path),
                    )
                });
                if let Some((obj, base_path, merged)) = loc_snapshot {
                    let field_loc_id = if let Some(merged_path) = merged {
                        factory.get_or_create(obj, merged_path)
                    } else {
                        let new_path = base_path.extend(path);
                        factory.get_or_create(obj, new_path)
                    };

                    new_addr_of.push((*dst, field_loc_id));
                    resolved_any = true;
                }
            }
        }

        if resolved_any {
            resolved_count += 1;
        } else {
            unresolved.push((*dst, *src_ptr, path.clone()));
        }
    }

    facts.addr_of.extend(new_addr_of);
    facts.gep.clear();

    GepResolutionResult {
        resolved_count,
        unresolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_analysis::{FieldPath, FieldSensitivity};
    use saf_core::ids::ObjId;

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 4 })
    }

    #[test]
    fn resolve_gep_with_known_base() {
        let mut factory = make_factory();
        let obj = ObjId::derive(b"alloc");
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        let ptr = ValueId::derive(b"ptr");
        let gep_dst = ValueId::derive(b"gep_dst");
        let path = FieldPath::field(1);

        let mut facts = PtaFacts {
            addr_of: vec![(ptr, base_loc)],
            copy: vec![],
            load: vec![],
            store: vec![],
            gep: vec![(gep_dst, ptr, path)],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        // GEP should be resolved: gep cleared, addr_of extended
        assert!(facts.gep.is_empty(), "GEP facts should be cleared");
        assert_eq!(
            facts.addr_of.len(),
            2,
            "Should have original + resolved addr_of"
        );
        assert!(facts.copy.is_empty(), "No copy fallback needed");

        // The resolved addr_of should point to a field location
        let (resolved_val, resolved_loc) = facts.addr_of[1];
        assert_eq!(resolved_val, gep_dst);
        assert_ne!(resolved_loc, base_loc, "Field loc should differ from base");

        // Verify the field location in the factory
        let loc = factory.get(resolved_loc).expect("field loc should exist");
        assert_eq!(loc.obj, obj);
        assert_eq!(loc.path, FieldPath::field(1));
    }

    #[test]
    fn resolve_gep_unknown_base_falls_back_to_copy() {
        let mut factory = make_factory();

        let unknown_ptr = ValueId::derive(b"unknown");
        let gep_dst = ValueId::derive(b"gep_dst");
        let path = FieldPath::field(0);

        let mut facts = PtaFacts {
            addr_of: vec![],
            copy: vec![],
            load: vec![],
            store: vec![],
            gep: vec![(gep_dst, unknown_ptr, path)],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        assert!(facts.gep.is_empty());
        assert!(facts.addr_of.is_empty(), "No addr_of since base is unknown");
        assert_eq!(facts.copy.len(), 1, "Should fall back to copy");
        assert_eq!(facts.copy[0], (gep_dst, unknown_ptr));
    }

    #[test]
    fn resolve_gep_multiple_base_locs() {
        let mut factory = make_factory();
        let obj1 = ObjId::derive(b"alloc1");
        let obj2 = ObjId::derive(b"alloc2");
        let loc1 = factory.get_or_create(obj1, FieldPath::empty());
        let loc2 = factory.get_or_create(obj2, FieldPath::empty());

        let ptr = ValueId::derive(b"ptr");
        let gep_dst = ValueId::derive(b"gep_dst");
        let path = FieldPath::field(0);

        let mut facts = PtaFacts {
            addr_of: vec![(ptr, loc1), (ptr, loc2)],
            copy: vec![],
            load: vec![],
            store: vec![],
            gep: vec![(gep_dst, ptr, path)],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        assert!(facts.gep.is_empty());
        // Original 2 addr_of + 2 resolved (one per base loc)
        assert_eq!(facts.addr_of.len(), 4);
        assert!(facts.copy.is_empty());
    }

    #[test]
    fn resolve_gep_preserves_existing_facts() {
        let mut factory = make_factory();
        let obj = ObjId::derive(b"alloc");
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        let ptr = ValueId::derive(b"ptr");
        let other_val = ValueId::derive(b"other");
        let gep_dst = ValueId::derive(b"gep_dst");

        let mut facts = PtaFacts {
            addr_of: vec![(ptr, base_loc)],
            copy: vec![(other_val, ptr)],
            load: vec![(other_val, ptr)],
            store: vec![(ptr, other_val)],
            gep: vec![(gep_dst, ptr, FieldPath::field(2))],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        // Existing facts should be untouched
        assert_eq!(facts.copy.len(), 1);
        assert_eq!(facts.load.len(), 1);
        assert_eq!(facts.store.len(), 1);
        // addr_of extended
        assert_eq!(facts.addr_of.len(), 2);
    }

    #[test]
    fn resolve_gep_empty_path_returns_same_base() {
        let mut factory = make_factory();
        let obj = ObjId::derive(b"alloc");
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        let ptr = ValueId::derive(b"ptr");
        let gep_dst = ValueId::derive(b"gep_dst");

        let mut facts = PtaFacts {
            addr_of: vec![(ptr, base_loc)],
            copy: vec![],
            load: vec![],
            store: vec![],
            gep: vec![(gep_dst, ptr, FieldPath::empty())],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        assert!(facts.gep.is_empty());
        // Resolved addr_of should point to the same base loc (empty path extends to empty)
        let (_, resolved_loc) = facts.addr_of[1];
        assert_eq!(resolved_loc, base_loc);
    }

    #[test]
    fn resolve_gep_through_copy_chain() {
        let mut factory = make_factory();
        let obj = ObjId::derive(b"alloc");
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        // p = &obj; q = p; r = &q->field
        let p = ValueId::derive(b"p");
        let q = ValueId::derive(b"q");
        let gep_dst = ValueId::derive(b"gep_dst");

        let mut facts = PtaFacts {
            addr_of: vec![(p, base_loc)],
            copy: vec![(q, p)],
            load: vec![],
            store: vec![],
            gep: vec![(gep_dst, q, FieldPath::field(1))],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        // GEP should be resolved via copy propagation, NOT fallen back to copy
        assert!(facts.gep.is_empty());
        // Should have original addr_of(p, base) + resolved addr_of(gep_dst, field)
        assert_eq!(
            facts.addr_of.len(),
            2,
            "Should resolve GEP through copy chain"
        );
        // Copy edge should remain (original q=p), no additional copy fallback
        assert_eq!(
            facts.copy.len(),
            1,
            "No extra copy fallback should be added"
        );

        let (resolved_val, resolved_loc) = facts.addr_of[1];
        assert_eq!(resolved_val, gep_dst);
        assert_ne!(resolved_loc, base_loc, "Field loc should differ from base");
    }

    #[test]
    fn resolve_gep_through_transitive_copy() {
        let mut factory = make_factory();
        let obj = ObjId::derive(b"alloc");
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        // p = &obj; q = p; r = q; s = &r->field
        let p = ValueId::derive(b"p");
        let q = ValueId::derive(b"q");
        let r = ValueId::derive(b"r");
        let gep_dst = ValueId::derive(b"gep_dst");

        let mut facts = PtaFacts {
            addr_of: vec![(p, base_loc)],
            copy: vec![(q, p), (r, q)],
            load: vec![],
            store: vec![],
            gep: vec![(gep_dst, r, FieldPath::field(0))],
        };

        resolve_gep_facts(&mut facts, &mut factory);

        assert!(facts.gep.is_empty());
        assert_eq!(
            facts.addr_of.len(),
            2,
            "Should resolve through transitive copy chain"
        );
        assert_eq!(
            facts.copy.len(),
            2,
            "Original copies preserved, no fallback"
        );
    }

    #[test]
    fn try_resolve_returns_unresolved_without_copy_fallback() {
        let mut factory = make_factory();
        let obj = ObjId::derive(b"alloc");
        let base_loc = factory.get_or_create(obj, FieldPath::empty());

        let known_ptr = ValueId::derive(b"known");
        let unknown_ptr = ValueId::derive(b"unknown");
        let gep_dst1 = ValueId::derive(b"gep_dst1");
        let gep_dst2 = ValueId::derive(b"gep_dst2");

        let mut pts = PointsToMap::new();
        pts.insert(known_ptr, [base_loc].into_iter().collect());

        let mut facts = PtaFacts {
            addr_of: vec![(known_ptr, base_loc)],
            copy: vec![],
            load: vec![],
            store: vec![],
            gep: vec![
                (gep_dst1, known_ptr, FieldPath::field(0)),
                (gep_dst2, unknown_ptr, FieldPath::field(1)),
            ],
        };

        let result = try_resolve_gep_facts(&mut facts, &mut factory, &pts);

        assert_eq!(result.resolved_count, 1);
        assert_eq!(result.unresolved.len(), 1);
        assert_eq!(result.unresolved[0].0, gep_dst2);
        assert_eq!(result.unresolved[0].1, unknown_ptr);

        // No copy fallback should be added
        assert!(
            facts.copy.is_empty(),
            "try_resolve should NOT add copy fallbacks"
        );
        // But resolved addr_of should be added
        assert_eq!(facts.addr_of.len(), 2); // original + resolved
        assert!(facts.gep.is_empty()); // GEPs cleared
    }
}
