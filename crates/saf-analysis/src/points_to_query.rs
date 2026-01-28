//! Unified trait for querying points-to analysis results.
//!
//! Different pointer analyses (`PtaResult`, `CsPtaResult`, `FlowSensitivePtaResult`)
//! share a common context-insensitive query interface: `points_to` and `may_alias`.
//! `PointsToQuery` captures this interface so consumers can be generic over the
//! underlying analysis.

use saf_core::ids::{LocId, ValueId};

use crate::pta::AliasResult;

/// Context-insensitive points-to query interface.
///
/// Implemented by all pointer analysis result types that support querying
/// points-to sets and alias relationships without an explicit calling context.
///
/// For context-sensitive analyses (e.g., `CsPtaResult`), the implementation
/// uses the CI summary (union across all contexts).
pub trait PointsToQuery {
    /// Get the points-to set for a value.
    ///
    /// Returns the set of abstract locations the value may point to.
    /// Returns an empty vector if the value is not tracked.
    fn points_to(&self, ptr: ValueId) -> Vec<LocId>;

    /// Check if two pointers may alias.
    ///
    /// Returns a five-valued `AliasResult`: Must, Partial, May, No, or Unknown.
    fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult;

    /// Check if a pointer's points-to set is a singleton.
    ///
    /// A pointer with a singleton points-to set points to exactly one abstract
    /// location, which is useful for strong update decisions.
    fn is_singleton(&self, ptr: ValueId) -> bool {
        self.points_to(ptr).len() == 1
    }
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

impl PointsToQuery for crate::PtaResult {
    fn points_to(&self, ptr: ValueId) -> Vec<LocId> {
        crate::PtaResult::points_to(self, ptr)
    }

    fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        crate::PtaResult::may_alias(self, p, q)
    }
}

impl PointsToQuery for crate::cspta::CsPtaResult {
    fn points_to(&self, ptr: ValueId) -> Vec<LocId> {
        crate::cspta::CsPtaResult::points_to_any(self, ptr)
    }

    fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        crate::cspta::CsPtaResult::may_alias_any(self, p, q)
    }
}

impl PointsToQuery for crate::fspta::FlowSensitivePtaResult {
    fn points_to(&self, ptr: ValueId) -> Vec<LocId> {
        crate::fspta::FlowSensitivePtaResult::points_to(self, ptr)
            .iter()
            .copied()
            .collect()
    }

    fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        // Use top-level points-to sets (not flow-sensitive at a specific node).
        use crate::svfg::SvfgNodeId;
        crate::fspta::FlowSensitivePtaResult::may_alias_at(
            self,
            p,
            q,
            SvfgNodeId::Value(ValueId::new(0)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the trait is object-safe by creating a trait object.
    #[test]
    fn trait_is_object_safe() {
        fn _accepts_dyn(_q: &dyn PointsToQuery) {}
    }

    /// Verify that generic code can be written against the trait.
    #[test]
    fn generic_over_trait() {
        fn _count_targets<Q: PointsToQuery>(q: &Q, ptr: ValueId) -> usize {
            q.points_to(ptr).len()
        }
    }
}
