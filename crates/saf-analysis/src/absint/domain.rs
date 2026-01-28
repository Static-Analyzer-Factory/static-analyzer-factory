//! Abstract domain trait for abstract interpretation.
//!
//! Defines the core lattice operations following the IKOS/SPARTA pattern.
//! All abstract domains in SAF implement this trait.
//!
//! See also [`crate::ifds::lattice::Lattice`] for the simpler finite-lattice trait
//! used by IDE/IFDS. Types implementing `AbstractDomain + Ord` automatically
//! implement `Lattice` via a blanket impl in this module.

use crate::ifds::lattice::Lattice;

/// Core lattice operations for abstract interpretation.
///
/// An abstract domain is a complete lattice with join (⊔), meet (⊓),
/// widening (▽), and narrowing (△) operators. The partial order (⊑)
/// ensures convergence of fixpoint iteration.
///
/// # Lattice Laws
///
/// Implementations must satisfy:
/// - **Join commutativity:** `a.join(b) == b.join(a)`
/// - **Join associativity:** `a.join(b.join(c)) == a.join(b).join(c)`
/// - **Join idempotency:** `a.join(a) == a`
/// - **Meet commutativity:** `a.meet(b) == b.meet(a)`
/// - **Meet associativity:** `a.meet(b.meet(c)) == a.meet(b).meet(c)`
/// - **Meet idempotency:** `a.meet(a) == a`
/// - **Bottom absorbs join:** `bottom.join(a) == a`
/// - **Top absorbs meet:** `top.meet(a) == a`
/// - **Widening soundness:** `a ⊑ a.widen(b)` and `b ⊑ a.widen(b)`
/// - **Narrowing refinement:** `a.narrow(b) ⊑ a`
pub trait AbstractDomain: Clone + Eq + std::fmt::Debug {
    /// The bottom element (unreachable / empty set).
    #[must_use]
    fn bottom() -> Self;

    /// The top element (no information / all values).
    #[must_use]
    fn top() -> Self;

    /// Check if this is the bottom element.
    fn is_bottom(&self) -> bool;

    /// Check if this is the top element.
    fn is_top(&self) -> bool;

    /// Partial order: self ⊑ other (self is less than or equal to other).
    fn leq(&self, other: &Self) -> bool;

    /// Least upper bound (union): self ⊔ other.
    #[must_use]
    fn join(&self, other: &Self) -> Self;

    /// Greatest lower bound (intersection): self ⊓ other.
    #[must_use]
    fn meet(&self, other: &Self) -> Self;

    /// Widening: over-approximate join for convergence at loop heads.
    ///
    /// Default implementation delegates to `join`, which is correct for
    /// finite-height domains where ascending chains are bounded.
    #[must_use]
    fn widen(&self, other: &Self) -> Self {
        self.join(other)
    }

    /// Narrowing: refine after widening for improved precision.
    ///
    /// Default implementation delegates to `meet`, which is correct for
    /// any domain (narrowing result must be ⊑ self).
    #[must_use]
    fn narrow(&self, other: &Self) -> Self {
        self.meet(other)
    }
}

/// Blanket implementation: any `AbstractDomain` that also has a total order
/// automatically satisfies the [`Lattice`] trait used by IDE/IFDS.
///
/// This bridges the abstract interpretation and IFDS frameworks, allowing
/// types like custom numeric domains with natural orderings to work with
/// both the fixpoint solver and the IDE solver.
impl<T: AbstractDomain + Ord> Lattice for T {
    fn top() -> Self {
        <Self as AbstractDomain>::top()
    }

    fn bottom() -> Self {
        <Self as AbstractDomain>::bottom()
    }

    fn join(&self, other: &Self) -> Self {
        <Self as AbstractDomain>::join(self, other)
    }

    fn meet(&self, other: &Self) -> Self {
        <Self as AbstractDomain>::meet(self, other)
    }

    fn leq(&self, other: &Self) -> bool {
        <Self as AbstractDomain>::leq(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifds::lattice::Lattice;

    /// A simple sign domain that implements both `AbstractDomain` and `Ord`.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Sign {
        Bottom,
        Negative,
        Zero,
        Positive,
        Top,
    }

    impl AbstractDomain for Sign {
        fn bottom() -> Self {
            Sign::Bottom
        }

        fn top() -> Self {
            Sign::Top
        }

        fn is_bottom(&self) -> bool {
            matches!(self, Sign::Bottom)
        }

        fn is_top(&self) -> bool {
            matches!(self, Sign::Top)
        }

        fn leq(&self, other: &Self) -> bool {
            matches!(
                (self, other),
                (Sign::Bottom, _)
                    | (_, Sign::Top)
                    | (Sign::Negative, Sign::Negative)
                    | (Sign::Zero, Sign::Zero)
                    | (Sign::Positive, Sign::Positive)
            )
        }

        fn join(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            if self.is_bottom() {
                return other.clone();
            }
            if other.is_bottom() {
                return self.clone();
            }
            Sign::Top
        }

        fn meet(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            if self.is_top() {
                return other.clone();
            }
            if other.is_top() {
                return self.clone();
            }
            Sign::Bottom
        }
    }

    #[test]
    fn blanket_lattice_impl_works() {
        // `Sign` implements `AbstractDomain + Ord`, so it should also implement `Lattice`
        let a = Sign::Negative;
        let b = Sign::Positive;

        // Use via `Lattice` trait (fully qualified to avoid ambiguity)
        let joined: Sign = Lattice::join(&a, &b);
        assert_eq!(joined, Sign::Top);

        let met: Sign = Lattice::meet(&a, &b);
        assert_eq!(met, Sign::Bottom);

        assert!(Lattice::leq(&Sign::Bottom, &Sign::Top));
        assert_eq!(<Sign as Lattice>::top(), <Sign as AbstractDomain>::top());
        assert_eq!(
            <Sign as Lattice>::bottom(),
            <Sign as AbstractDomain>::bottom()
        );
    }

    #[test]
    fn blanket_impl_widen_narrow_default_to_join_meet() {
        // `AbstractDomain`'s default `widen` = `join`, `narrow` = `meet`
        let a = Sign::Negative;
        let b = Sign::Positive;
        assert_eq!(a.widen(&b), AbstractDomain::join(&a, &b));
        assert_eq!(a.narrow(&b), AbstractDomain::meet(&a, &b));
    }
}
