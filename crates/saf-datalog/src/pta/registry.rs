//! Location ID registry for dense bitvector indexing.
//!
//! Maps sparse `LocId` (u128 BLAKE3 hashes) to dense `u32` indices
//! for use in `FixedBitSet`-backed points-to sets. The registry is
//! built once from all `LocId`s in `PtaFacts` and shared via `Arc`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use saf_core::ids::LocId;

use crate::facts::PtaFacts;

thread_local! {
    static PTS_REGISTRY: RefCell<Option<Arc<LocIdRegistry>>> = const { RefCell::new(None) };
}

/// Run a closure with the given `LocIdRegistry` available via [`current_registry`].
///
/// Sets the thread-local registry before the closure and clears it after.
/// This is the entry point for making the registry available inside Ascent's
/// generated rule code, where `AscentPtsSet::singleton(loc)` needs to look
/// up the dense index for a `LocId`.
///
/// # Panics
///
/// Panics if the thread-local is already set (nested calls are not supported).
pub fn with_registry<R>(registry: Arc<LocIdRegistry>, f: impl FnOnce() -> R) -> R {
    PTS_REGISTRY.with(|r| {
        let prev = r.borrow().clone();
        assert!(
            prev.is_none(),
            "nested with_registry calls are not supported"
        );
        *r.borrow_mut() = Some(registry);
    });
    let result = f();
    PTS_REGISTRY.with(|r| {
        *r.borrow_mut() = None;
    });
    result
}

/// Get the current thread-local `LocIdRegistry`.
///
/// Returns a clone of the `Arc` — cheap (atomic refcount bump).
///
/// # Panics
///
/// Panics if called outside a [`with_registry`] scope.
#[must_use]
pub fn current_registry() -> Arc<LocIdRegistry> {
    PTS_REGISTRY.with(|r| {
        r.borrow()
            .as_ref()
            .expect("PTS_REGISTRY not set — call with_registry() first")
            .clone()
    })
}

/// Maps sparse `LocId` values to dense sequential `u32` indices.
///
/// Built once per analysis phase from the `LocId`s appearing in `PtaFacts`.
/// Shared across all `AscentPtsSet` instances via `Arc<LocIdRegistry>`.
#[derive(Debug, Clone)]
pub struct LocIdRegistry {
    to_index: BTreeMap<LocId, u32>,
    to_loc: Vec<LocId>,
}

impl LocIdRegistry {
    /// Build a registry from all `LocId`s appearing in the given facts.
    ///
    /// Collects unique `LocId`s from `addr_of` facts (the only source of
    /// location IDs), assigns them dense sequential indices starting at 0,
    /// and builds both forward and reverse mappings.
    #[must_use]
    pub fn from_facts(facts: &PtaFacts) -> Self {
        let to_loc: Vec<LocId> = facts
            .addr_of
            .iter()
            .map(|(_, loc)| *loc)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        // BTreeSet already sorts, so to_loc is in deterministic order.

        let to_index: BTreeMap<LocId, u32> = to_loc
            .iter()
            .enumerate()
            .map(|(i, loc)| {
                // INVARIANT: PTA programs have < 2^32 abstract locations.
                #[allow(clippy::cast_possible_truncation)]
                let idx = i as u32;
                (*loc, idx)
            })
            .collect();

        Self { to_index, to_loc }
    }

    /// Look up the dense index for a `LocId`.
    ///
    /// # Panics
    ///
    /// Panics if `loc` was not in the facts used to build this registry.
    #[must_use]
    pub fn index_of(&self, loc: LocId) -> u32 {
        self.to_index[&loc]
    }

    /// Try to look up the dense index for a `LocId`.
    ///
    /// Returns `None` if the `LocId` is not in the registry.
    #[must_use]
    pub fn try_index_of(&self, loc: LocId) -> Option<&u32> {
        self.to_index.get(&loc)
    }

    /// Look up the `LocId` for a dense index.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[must_use]
    pub fn loc_at(&self, idx: u32) -> LocId {
        self.to_loc[idx as usize]
    }

    /// Number of registered locations (= bitvector capacity).
    #[must_use]
    pub fn len(&self) -> usize {
        self.to_loc.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to_loc.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ValueId;

    fn make_loc(name: &[u8]) -> LocId {
        LocId::derive(name)
    }

    fn make_val(name: &[u8]) -> ValueId {
        ValueId::derive(name)
    }

    #[test]
    fn empty_facts_produce_empty_registry() {
        let facts = PtaFacts::default();
        let reg = LocIdRegistry::from_facts(&facts);
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());
    }

    #[test]
    fn round_trip_index_to_loc() {
        let loc_a = make_loc(b"a");
        let loc_b = make_loc(b"b");
        let val = make_val(b"ptr");

        let facts = PtaFacts {
            addr_of: vec![(val, loc_a), (val, loc_b)],
            ..Default::default()
        };
        let reg = LocIdRegistry::from_facts(&facts);

        assert_eq!(reg.len(), 2);

        // Round-trip: loc -> index -> loc
        let idx_a = reg.index_of(loc_a);
        let idx_b = reg.index_of(loc_b);
        assert_ne!(idx_a, idx_b);
        assert_eq!(reg.loc_at(idx_a), loc_a);
        assert_eq!(reg.loc_at(idx_b), loc_b);
    }

    #[test]
    fn duplicate_locs_are_deduplicated() {
        let loc = make_loc(b"same");
        let v1 = make_val(b"p1");
        let v2 = make_val(b"p2");

        let facts = PtaFacts {
            addr_of: vec![(v1, loc), (v2, loc)],
            ..Default::default()
        };
        let reg = LocIdRegistry::from_facts(&facts);

        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn indices_are_sequential_from_zero() {
        let locs: Vec<LocId> = (0..5)
            .map(|i| make_loc(format!("loc{i}").as_bytes()))
            .collect();
        let val = make_val(b"ptr");

        let facts = PtaFacts {
            addr_of: locs.iter().map(|l| (val, *l)).collect(),
            ..Default::default()
        };
        let reg = LocIdRegistry::from_facts(&facts);

        assert_eq!(reg.len(), 5);

        let mut indices: Vec<u32> = locs.iter().map(|l| reg.index_of(*l)).collect();
        indices.sort_unstable();
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn index_of_unknown_loc_panics() {
        let facts = PtaFacts::default();
        let reg = LocIdRegistry::from_facts(&facts);
        reg.index_of(make_loc(b"unknown"));
    }

    #[test]
    #[should_panic]
    fn loc_at_out_of_bounds_panics() {
        let facts = PtaFacts::default();
        let reg = LocIdRegistry::from_facts(&facts);
        reg.loc_at(0);
    }

    // Thread-local tests

    #[test]
    fn with_registry_provides_access() {
        let loc = make_loc(b"loc");
        let val = make_val(b"ptr");
        let facts = PtaFacts {
            addr_of: vec![(val, loc)],
            ..Default::default()
        };
        let reg = Arc::new(LocIdRegistry::from_facts(&facts));

        with_registry(reg.clone(), || {
            let current = current_registry();
            assert_eq!(current.len(), 1);
            assert_eq!(current.index_of(loc), 0);
        });
    }

    #[test]
    #[should_panic(expected = "PTS_REGISTRY not set")]
    fn current_registry_panics_without_setup() {
        let _ = current_registry();
    }

    #[test]
    fn with_registry_cleans_up_after_closure() {
        let facts = PtaFacts {
            addr_of: vec![(make_val(b"p"), make_loc(b"l"))],
            ..Default::default()
        };
        let reg = Arc::new(LocIdRegistry::from_facts(&facts));

        with_registry(reg, || {
            // Registry available here
            assert_eq!(current_registry().len(), 1);
        });

        // After with_registry, thread-local should be cleared
        // (panics if we try to access)
        let result = std::panic::catch_unwind(|| current_registry());
        assert!(result.is_err());
    }
}
