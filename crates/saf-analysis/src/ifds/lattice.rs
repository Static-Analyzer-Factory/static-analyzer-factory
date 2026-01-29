//! Lattice trait for IDE value domains.
//!
//! A finite lattice used by the IDE algorithm to track values associated with
//! data-flow facts. Simpler than the `AbstractDomain` trait (E15) because IDE
//! operates on finite lattices — no widening or narrowing needed.
//!
//! See also [`crate::absint::domain::AbstractDomain`] for the abstract interpretation
//! lattice trait, which adds `widen`/`narrow` for infinite-height domains. Types
//! implementing `AbstractDomain + Ord` automatically implement `Lattice` via a
//! blanket impl in [`crate::absint::domain`].

use std::fmt::Debug;

/// A finite lattice for IDE value domains.
///
/// Implementations must satisfy the standard lattice laws:
/// - `join` is the least upper bound (LUB)
/// - `meet` is the greatest lower bound (GLB)
/// - `leq(a, b)` iff `join(a, b) == b`
/// - `top()` is the greatest element
/// - `bottom()` is the least element
pub trait Lattice: Clone + Ord + Debug {
    /// The greatest element (top of the lattice).
    #[must_use]
    fn top() -> Self;

    /// The least element (bottom of the lattice).
    #[must_use]
    fn bottom() -> Self;

    /// Least upper bound (join / merge).
    #[must_use]
    fn join(&self, other: &Self) -> Self;

    /// Greatest lower bound (meet).
    #[must_use]
    fn meet(&self, other: &Self) -> Self;

    /// Partial order: `self <= other` in the lattice ordering.
    fn leq(&self, other: &Self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple two-point lattice: Bottom < Top.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TwoPoint {
        Bottom,
        Top,
    }

    impl Lattice for TwoPoint {
        fn top() -> Self {
            TwoPoint::Top
        }

        fn bottom() -> Self {
            TwoPoint::Bottom
        }

        fn join(&self, other: &Self) -> Self {
            if *self == TwoPoint::Top || *other == TwoPoint::Top {
                TwoPoint::Top
            } else {
                TwoPoint::Bottom
            }
        }

        fn meet(&self, other: &Self) -> Self {
            if *self == TwoPoint::Bottom || *other == TwoPoint::Bottom {
                TwoPoint::Bottom
            } else {
                TwoPoint::Top
            }
        }

        fn leq(&self, other: &Self) -> bool {
            match (self, other) {
                (TwoPoint::Bottom, _) => true,
                (TwoPoint::Top, TwoPoint::Top) => true,
                (TwoPoint::Top, TwoPoint::Bottom) => false,
            }
        }
    }

    #[test]
    fn two_point_top_is_greatest() {
        assert!(TwoPoint::Bottom.leq(&TwoPoint::Top));
        assert!(TwoPoint::Top.leq(&TwoPoint::Top));
    }

    #[test]
    fn two_point_bottom_is_least() {
        assert!(TwoPoint::Bottom.leq(&TwoPoint::Bottom));
        assert!(TwoPoint::Bottom.leq(&TwoPoint::Top));
    }

    #[test]
    fn two_point_join_is_lub() {
        assert_eq!(TwoPoint::Bottom.join(&TwoPoint::Top), TwoPoint::Top);
        assert_eq!(TwoPoint::Top.join(&TwoPoint::Bottom), TwoPoint::Top);
        assert_eq!(TwoPoint::Bottom.join(&TwoPoint::Bottom), TwoPoint::Bottom);
        assert_eq!(TwoPoint::Top.join(&TwoPoint::Top), TwoPoint::Top);
    }

    #[test]
    fn two_point_meet_is_glb() {
        assert_eq!(TwoPoint::Bottom.meet(&TwoPoint::Top), TwoPoint::Bottom);
        assert_eq!(TwoPoint::Top.meet(&TwoPoint::Bottom), TwoPoint::Bottom);
        assert_eq!(TwoPoint::Top.meet(&TwoPoint::Top), TwoPoint::Top);
        assert_eq!(TwoPoint::Bottom.meet(&TwoPoint::Bottom), TwoPoint::Bottom);
    }

    #[test]
    fn leq_consistent_with_join() {
        // a <= b iff join(a,b) == b
        let vals = [TwoPoint::Bottom, TwoPoint::Top];
        for a in &vals {
            for b in &vals {
                assert_eq!(a.leq(b), a.join(b) == *b, "a={a:?}, b={b:?}");
            }
        }
    }
}
