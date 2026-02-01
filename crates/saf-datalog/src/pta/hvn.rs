//! HVN (Hash-Value Numbering) preprocessing for Ascent PTA facts.
//!
//! Wraps the existing HVN implementation from `saf-analysis` by converting
//! between [`PtaFacts`] and [`ConstraintSet`](saf_analysis::ConstraintSet)
//! formats. This allows the Ascent-based PTA solver to benefit from the same
//! constraint reduction that the imperative solver uses.

use std::collections::BTreeSet;

use saf_analysis::{
    AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, HvnResult, LoadConstraint,
    StoreConstraint, hvn_preprocess,
};
use saf_core::ids::{LocId, ValueId};

use crate::facts::PtaFacts;

/// Result of HVN preprocessing adapted for Ascent PTA facts.
#[derive(Debug, Clone)]
pub struct DatalogHvnResult {
    /// The underlying HVN result (contains `mapping`, `num_classes`, `removed`).
    pub inner: HvnResult,
}

impl DatalogHvnResult {
    /// Number of equivalence classes found with >1 member.
    #[must_use]
    pub fn num_classes(&self) -> usize {
        self.inner.num_classes
    }

    /// Number of constraints removed by deduplication and self-constraint elimination.
    #[must_use]
    pub fn removed(&self) -> usize {
        self.inner.removed
    }

    /// Expand a points-to map by copying representative entries to all
    /// merged values. Call this after solving to recover results for
    /// values that were merged away.
    pub fn expand_results(&self, pts: &mut std::collections::BTreeMap<ValueId, BTreeSet<LocId>>) {
        for (original, rep) in &self.inner.mapping {
            if let Some(rep_pts) = pts.get(rep).cloned() {
                pts.insert(*original, rep_pts);
            }
        }
    }
}

/// Run HVN preprocessing on [`PtaFacts`].
///
/// Converts to [`ConstraintSet`], runs the existing HVN algorithm from
/// `saf-analysis`, then converts back. The facts are modified in place.
///
/// Returns a [`DatalogHvnResult`] that can be used to expand solver
/// results back to the original value space.
pub fn preprocess_hvn(facts: &mut PtaFacts) -> DatalogHvnResult {
    // Convert PtaFacts -> ConstraintSet
    let mut cs = facts_to_constraint_set(facts);

    // Run the existing HVN algorithm
    let hvn_result = hvn_preprocess(&mut cs);

    // Convert back ConstraintSet -> PtaFacts
    *facts = constraint_set_to_facts(cs);

    DatalogHvnResult { inner: hvn_result }
}

/// Convert [`PtaFacts`] (flat tuples) into a [`ConstraintSet`] (typed structs in `BTreeSet`s).
fn facts_to_constraint_set(facts: &PtaFacts) -> ConstraintSet {
    ConstraintSet {
        addr: facts
            .addr_of
            .iter()
            .map(|(ptr, loc)| AddrConstraint {
                ptr: *ptr,
                loc: *loc,
            })
            .collect(),
        copy: facts
            .copy
            .iter()
            .map(|(dst, src)| CopyConstraint {
                dst: *dst,
                src: *src,
            })
            .collect(),
        load: facts
            .load
            .iter()
            .map(|(dst, src)| LoadConstraint {
                dst: *dst,
                src_ptr: *src,
            })
            .collect(),
        store: facts
            .store
            .iter()
            .map(|(dst, src)| StoreConstraint {
                dst_ptr: *dst,
                src: *src,
            })
            .collect(),
        gep: facts
            .gep
            .iter()
            .map(|(dst, src, path)| GepConstraint {
                dst: *dst,
                src_ptr: *src,
                path: path.clone(),
                index_operands: vec![],
            })
            .collect(),
    }
}

/// Convert a [`ConstraintSet`] back into [`PtaFacts`] (flat tuples).
fn constraint_set_to_facts(cs: ConstraintSet) -> PtaFacts {
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
