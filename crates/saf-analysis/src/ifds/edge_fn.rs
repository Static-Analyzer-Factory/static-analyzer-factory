//! Built-in edge functions for the IDE framework.
//!
//! Edge functions map lattice values `L -> L` and are attached to every edge in
//! the exploded supergraph. They support composition (for path concatenation)
//! and join (for merge points).
//!
//! Using an enum rather than trait objects avoids heap allocation for the common
//! `Identity`/`AllTop` cases and enables `Ord` for deterministic `BTreeMap` keys.

use std::sync::Arc;

use super::lattice::Lattice;

/// Built-in edge functions for any `Lattice` type.
#[derive(Clone)]
pub enum BuiltinEdgeFn<V: Lattice> {
    /// `f(x) = x`. Neutral element for composition.
    Identity,
    /// `f(x) = top` for all x. Neutral element for join.
    AllTop,
    /// `f(x) = bottom` for all x. Absorbing element.
    AllBottom,
    /// `f(x) = c`. Ignores input, always returns `c`.
    Constant(V),
    /// `f(x) = g(h(x))`. Lazy composition of two edge functions.
    Composed(Box<BuiltinEdgeFn<V>>, Box<BuiltinEdgeFn<V>>),
    /// Transition table: maps input values to output values.
    /// If input not in the table, returns `default`.
    /// Used for typestate transitions where the output depends on input.
    TransitionTable {
        /// Mapping from input value to output value.
        entries: Vec<(V, V)>,
        /// Default value for inputs not in the table.
        default: V,
    },
    /// Custom edge function for analyses that need non-standard value transforms.
    /// Uses pointer identity for equality and ordering.
    Custom(Arc<dyn Fn(V) -> V + Send + Sync>),
}

impl<V: Lattice> std::fmt::Debug for BuiltinEdgeFn<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identity => write!(f, "Identity"),
            Self::AllTop => write!(f, "AllTop"),
            Self::AllBottom => write!(f, "AllBottom"),
            Self::Constant(v) => f.debug_tuple("Constant").field(v).finish(),
            Self::Composed(a, b) => f.debug_tuple("Composed").field(a).field(b).finish(),
            Self::TransitionTable { entries, default } => f
                .debug_struct("TransitionTable")
                .field("entries", entries)
                .field("default", default)
                .finish(),
            Self::Custom(_) => write!(f, "Custom(<fn>)"),
        }
    }
}

impl<V: Lattice> BuiltinEdgeFn<V> {
    /// Apply this edge function to a value.
    #[must_use]
    pub fn compute_target(&self, input: V) -> V {
        match self {
            Self::Identity => input,
            Self::AllTop => V::top(),
            Self::AllBottom => V::bottom(),
            Self::Constant(c) => c.clone(),
            Self::Composed(outer, inner) => {
                let intermediate = inner.compute_target(input);
                outer.compute_target(intermediate)
            }
            Self::TransitionTable { entries, default } => {
                for (from, to) in entries {
                    if *from == input {
                        return to.clone();
                    }
                }
                default.clone()
            }
            Self::Custom(f) => f(input),
        }
    }

    /// Compose `self` after `other`: result is `self(other(x))`.
    ///
    /// Simplifies when possible:
    /// - `Identity . f = f`
    /// - `f . Identity = f`
    /// - `AllTop . f = AllTop`
    /// - `AllBottom . f = AllBottom`
    /// - `Constant(c) . f = Constant(c)`
    #[must_use]
    pub fn compose_with(&self, other: &Self) -> Self {
        // self(other(x))
        match (self, other) {
            (Self::Identity, _) => other.clone(),
            (_, Self::Identity) | (Self::Constant(_), _) => self.clone(),
            (Self::AllTop, _) => Self::AllTop,
            (Self::AllBottom, _) => Self::AllBottom,
            (Self::Custom(_), _) | (_, Self::Custom(_)) => {
                Self::Composed(Box::new(self.clone()), Box::new(other.clone()))
            }
            _ => Self::Composed(Box::new(self.clone()), Box::new(other.clone())),
        }
    }

    /// Join (pointwise least upper bound) of two edge functions.
    ///
    /// For the IDE algorithm, join of edge functions `f` and `g` is the function
    /// `h` such that `h(x) = join(f(x), g(x))` for all x. We use algebraic
    /// simplifications:
    /// - `join(AllTop, f) = f` (`AllTop` is the neutral element for join)
    /// - `join(f, AllTop) = f`
    /// - `join(f, f) = f` (idempotent when structurally equal)
    /// - `join(Constant(a), Constant(b)) = Constant(join(a,b))`
    /// - `join(TransitionTable, TransitionTable)` merges entries pointwise
    ///
    /// For combinations that cannot be precisely represented (e.g.,
    /// `Identity` + `Constant`, `Composed` + anything non-trivial), returns
    /// `AllTop` as a sound over-approximation.
    #[must_use]
    pub fn join_with(&self, other: &Self) -> Self {
        match (self, other) {
            // AllTop is the neutral element for join (it maps everything to top,
            // which is the identity for the value-level join).
            (Self::AllTop, _) => other.clone(),
            (_, Self::AllTop) => self.clone(),
            // AllBottom maps everything to bottom; bottom <= everything, so
            // join(AllBottom, f) = f.
            (Self::AllBottom, _) => other.clone(),
            (_, Self::AllBottom) => self.clone(),
            // Identity join Identity = Identity.
            (Self::Identity, Self::Identity) => Self::Identity,
            // Two constants: join their values pointwise.
            (Self::Constant(a), Self::Constant(b)) => Self::Constant(a.join(b)),
            // Two TransitionTables: merge entries pointwise with LUB.
            (
                Self::TransitionTable {
                    entries: e1,
                    default: d1,
                },
                Self::TransitionTable {
                    entries: e2,
                    default: d2,
                },
            ) => Self::join_transition_tables(e1, d1, e2, d2),
            // Custom functions: sound over-approximation.
            (Self::Custom(_), _) | (_, Self::Custom(_)) => Self::AllTop,
            // Structurally equal non-trivial functions: idempotent.
            _ if self.structural_eq(other) => self.clone(),
            // All other combinations (Identity+Constant, Identity+TransitionTable,
            // Constant+TransitionTable, anything involving Composed, etc.) cannot
            // be precisely represented. Return AllTop as a sound over-approximation:
            // for all x, top >= join(f(x), g(x)).
            _ => Self::AllTop,
        }
    }

    /// Merge two `TransitionTable` edge functions by computing the pointwise LUB.
    ///
    /// For each input key present in either table:
    /// - If the key is in both tables, the output is `join(v1, v2)`.
    /// - If the key is only in one table, the output is `join(v, other_default)`.
    /// - The merged default is `join(d1, d2)`.
    fn join_transition_tables(e1: &[(V, V)], d1: &V, e2: &[(V, V)], d2: &V) -> Self {
        let joined_default = d1.join(d2);

        // Collect all entries into a BTreeMap for deterministic merge.
        let mut merged: std::collections::BTreeMap<V, V> = std::collections::BTreeMap::new();

        // Insert entries from the first table, joining values with d2 for keys
        // not in the second table.
        for (k, v1) in e1 {
            let v2 = e2.iter().find(|(k2, _)| k2 == k).map_or(d2, |(_, v)| v);
            merged.insert(k.clone(), v1.join(v2));
        }

        // Insert entries from the second table that are not already merged,
        // joining their values with d1.
        for (k, v2) in e2 {
            merged
                .entry(k.clone())
                .and_modify(|_| { /* already merged above */ })
                .or_insert_with(|| d1.join(v2));
        }

        // Remove entries whose value equals the joined default (they add no info).
        let entries: Vec<(V, V)> = merged
            .into_iter()
            .filter(|(_, v)| *v != joined_default)
            .collect();

        Self::TransitionTable {
            entries,
            default: joined_default,
        }
    }

    /// Check if this is the identity function.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        matches!(self, Self::Identity)
    }

    /// Check if this is the all-top function.
    #[must_use]
    pub fn is_top(&self) -> bool {
        matches!(self, Self::AllTop)
    }

    /// Structural equality check (used in join simplification).
    #[must_use]
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identity, Self::Identity)
            | (Self::AllTop, Self::AllTop)
            | (Self::AllBottom, Self::AllBottom) => true,
            (Self::Constant(a), Self::Constant(b)) => a == b,
            (Self::Composed(a1, a2), Self::Composed(b1, b2)) => {
                a1.structural_eq(b1) && a2.structural_eq(b2)
            }
            (
                Self::TransitionTable {
                    entries: e1,
                    default: d1,
                },
                Self::TransitionTable {
                    entries: e2,
                    default: d2,
                },
            ) => e1 == e2 && d1 == d2,
            (Self::Custom(a), Self::Custom(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    /// Check if this edge function is equal to another for a given set of inputs.
    /// Used by the IDE solver to detect when a join produces no change.
    #[must_use]
    pub fn equal_to(&self, other: &Self) -> bool {
        self.structural_eq(other)
    }
}

// Implement PartialEq and Eq based on structural equality.
impl<V: Lattice> PartialEq for BuiltinEdgeFn<V> {
    fn eq(&self, other: &Self) -> bool {
        self.structural_eq(other)
    }
}

impl<V: Lattice> Eq for BuiltinEdgeFn<V> {}

// Implement PartialOrd and Ord for BTreeMap usage.
impl<V: Lattice> PartialOrd for BuiltinEdgeFn<V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<V: Lattice> Ord for BuiltinEdgeFn<V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.variant_order()
            .cmp(&other.variant_order())
            .then_with(|| match (self, other) {
                (Self::Constant(a), Self::Constant(b)) => a.cmp(b),
                (Self::Composed(a1, a2), Self::Composed(b1, b2)) => {
                    a1.cmp(b1).then_with(|| a2.cmp(b2))
                }
                (
                    Self::TransitionTable {
                        entries: e1,
                        default: d1,
                    },
                    Self::TransitionTable {
                        entries: e2,
                        default: d2,
                    },
                ) => e1.cmp(e2).then_with(|| d1.cmp(d2)),
                (Self::Custom(a), Self::Custom(b)) => Arc::as_ptr(a)
                    .cast::<()>()
                    .cmp(&Arc::as_ptr(b).cast::<()>()),
                _ => std::cmp::Ordering::Equal,
            })
    }
}

impl<V: Lattice> BuiltinEdgeFn<V> {
    /// Discriminant order for `Ord` implementation.
    fn variant_order(&self) -> u8 {
        match self {
            Self::Identity => 0,
            Self::AllTop => 1,
            Self::AllBottom => 2,
            Self::Constant(_) => 3,
            Self::Composed(_, _) => 4,
            Self::TransitionTable { .. } => 5,
            Self::Custom(_) => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test lattice: flat lattice with Top > {A, B, C} > Bottom
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum FlatLat {
        Bottom,
        A,
        B,
        C,
        Top,
    }

    impl Lattice for FlatLat {
        fn top() -> Self {
            FlatLat::Top
        }

        fn bottom() -> Self {
            FlatLat::Bottom
        }

        fn join(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            match (self, other) {
                (FlatLat::Bottom, x) | (x, FlatLat::Bottom) => x.clone(),
                (FlatLat::Top, _) | (_, FlatLat::Top) => FlatLat::Top,
                _ => FlatLat::Top, // Incomparable elements join to Top
            }
        }

        fn meet(&self, other: &Self) -> Self {
            if self == other {
                return self.clone();
            }
            match (self, other) {
                (FlatLat::Top, x) | (x, FlatLat::Top) => x.clone(),
                (FlatLat::Bottom, _) | (_, FlatLat::Bottom) => FlatLat::Bottom,
                _ => FlatLat::Bottom, // Incomparable elements meet to Bottom
            }
        }

        fn leq(&self, other: &Self) -> bool {
            self.join(other) == *other
        }
    }

    type EF = BuiltinEdgeFn<FlatLat>;

    // ── compute_target tests ────────────────────────────────────────

    #[test]
    fn identity_returns_input() {
        assert_eq!(EF::Identity.compute_target(FlatLat::A), FlatLat::A);
        assert_eq!(
            EF::Identity.compute_target(FlatLat::Bottom),
            FlatLat::Bottom
        );
    }

    #[test]
    fn all_top_always_returns_top() {
        assert_eq!(EF::AllTop.compute_target(FlatLat::A), FlatLat::Top);
        assert_eq!(EF::AllTop.compute_target(FlatLat::Bottom), FlatLat::Top);
    }

    #[test]
    fn all_bottom_always_returns_bottom() {
        assert_eq!(EF::AllBottom.compute_target(FlatLat::A), FlatLat::Bottom);
        assert_eq!(EF::AllBottom.compute_target(FlatLat::Top), FlatLat::Bottom);
    }

    #[test]
    fn constant_ignores_input() {
        let f = EF::Constant(FlatLat::B);
        assert_eq!(f.compute_target(FlatLat::A), FlatLat::B);
        assert_eq!(f.compute_target(FlatLat::Top), FlatLat::B);
    }

    #[test]
    fn composed_applies_outer_of_inner() {
        // outer(inner(x)): Constant(B) after Identity = Constant(B)
        let f = EF::Composed(Box::new(EF::Constant(FlatLat::B)), Box::new(EF::Identity));
        assert_eq!(f.compute_target(FlatLat::A), FlatLat::B);
    }

    // ── compose_with tests ──────────────────────────────────────────

    #[test]
    fn compose_identity_left_neutral() {
        let f = EF::Constant(FlatLat::A);
        let composed = EF::Identity.compose_with(&f);
        assert_eq!(composed.compute_target(FlatLat::Top), FlatLat::A);
    }

    #[test]
    fn compose_identity_right_neutral() {
        let f = EF::Constant(FlatLat::A);
        let composed = f.compose_with(&EF::Identity);
        assert_eq!(composed.compute_target(FlatLat::Top), FlatLat::A);
    }

    #[test]
    fn compose_all_top_absorbs() {
        let f = EF::Constant(FlatLat::A);
        let composed = EF::AllTop.compose_with(&f);
        assert_eq!(composed.compute_target(FlatLat::B), FlatLat::Top);
    }

    #[test]
    fn compose_constant_absorbs_inner() {
        let f = EF::Identity;
        let composed = EF::Constant(FlatLat::C).compose_with(&f);
        assert_eq!(composed.compute_target(FlatLat::A), FlatLat::C);
    }

    // ── join_with tests ─────────────────────────────────────────────

    #[test]
    fn join_all_top_is_neutral() {
        let f = EF::Constant(FlatLat::A);
        let joined = EF::AllTop.join_with(&f);
        assert_eq!(joined.compute_target(FlatLat::Bottom), FlatLat::A);
    }

    #[test]
    fn join_identity_with_identity() {
        let joined = EF::Identity.join_with(&EF::Identity);
        assert!(joined.is_identity());
    }

    #[test]
    fn join_two_constants() {
        let f1 = EF::Constant(FlatLat::A);
        let f2 = EF::Constant(FlatLat::B);
        let joined = f1.join_with(&f2);
        // join(A, B) in flat lattice = Top
        assert_eq!(joined.compute_target(FlatLat::Bottom), FlatLat::Top);
    }

    #[test]
    fn join_same_constant_is_same() {
        let f1 = EF::Constant(FlatLat::A);
        let f2 = EF::Constant(FlatLat::A);
        let joined = f1.join_with(&f2);
        assert_eq!(joined.compute_target(FlatLat::Bottom), FlatLat::A);
    }

    #[test]
    fn join_all_bottom_with_f_returns_f() {
        let f = EF::Constant(FlatLat::B);
        let joined = EF::AllBottom.join_with(&f);
        assert_eq!(joined.compute_target(FlatLat::Top), FlatLat::B);
    }

    #[test]
    fn join_f_with_all_bottom_returns_f() {
        let f = EF::Constant(FlatLat::B);
        let joined = f.join_with(&EF::AllBottom);
        assert_eq!(joined.compute_target(FlatLat::Top), FlatLat::B);
    }

    // ── join_with: non-trivial cases (C4 fix) ──────────────────────

    #[test]
    fn join_two_transition_tables_disjoint_keys() {
        // Table1: A -> B, default C
        // Table2: B -> A, default C
        // Merged: A -> join(B, C)=Top, B -> join(C, A)=Top, default join(C, C)=C
        let t1 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::B)],
            default: FlatLat::C,
        };
        let t2 = EF::TransitionTable {
            entries: vec![(FlatLat::B, FlatLat::A)],
            default: FlatLat::C,
        };
        let joined = t1.join_with(&t2);
        // Both A->Top and B->Top differ from default C, so they should be in entries.
        assert_eq!(joined.compute_target(FlatLat::A), FlatLat::Top);
        assert_eq!(joined.compute_target(FlatLat::B), FlatLat::Top);
        // Unlisted key should get the joined default = C.
        assert_eq!(joined.compute_target(FlatLat::C), FlatLat::C);
    }

    #[test]
    fn join_two_transition_tables_shared_key() {
        // Table1: A -> A, default Bottom
        // Table2: A -> B, default Bottom
        // Merged: A -> join(A, B) = Top, default join(Bottom, Bottom) = Bottom
        let t1 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::A)],
            default: FlatLat::Bottom,
        };
        let t2 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::B)],
            default: FlatLat::Bottom,
        };
        let joined = t1.join_with(&t2);
        assert_eq!(joined.compute_target(FlatLat::A), FlatLat::Top);
        assert_eq!(joined.compute_target(FlatLat::B), FlatLat::Bottom);
    }

    #[test]
    fn join_two_transition_tables_same_entries() {
        // Identical tables should produce an equal table.
        let t1 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::B)],
            default: FlatLat::C,
        };
        let t2 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::B)],
            default: FlatLat::C,
        };
        let joined = t1.join_with(&t2);
        assert_eq!(joined.compute_target(FlatLat::A), FlatLat::B);
        assert_eq!(joined.compute_target(FlatLat::C), FlatLat::C);
    }

    #[test]
    fn join_identity_with_constant_returns_all_top() {
        // Identity(x)=x, Constant(A)=A. Their pointwise LUB is not representable
        // as a single variant, so we conservatively return AllTop.
        let joined = EF::Identity.join_with(&EF::Constant(FlatLat::A));
        assert!(joined.is_top(), "Identity join Constant should be AllTop");
    }

    #[test]
    fn join_constant_with_identity_returns_all_top() {
        let joined = EF::Constant(FlatLat::A).join_with(&EF::Identity);
        assert!(joined.is_top(), "Constant join Identity should be AllTop");
    }

    #[test]
    fn join_identity_with_transition_table_returns_all_top() {
        let t = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::B)],
            default: FlatLat::C,
        };
        let joined = EF::Identity.join_with(&t);
        assert!(
            joined.is_top(),
            "Identity join TransitionTable should be AllTop"
        );
    }

    #[test]
    fn join_composed_with_constant_returns_all_top() {
        let composed = EF::Composed(Box::new(EF::Constant(FlatLat::B)), Box::new(EF::Identity));
        let joined = composed.join_with(&EF::Constant(FlatLat::A));
        assert!(joined.is_top(), "Composed join Constant should be AllTop");
    }

    #[test]
    fn join_two_composed_returns_all_top() {
        let c1 = EF::Composed(Box::new(EF::Constant(FlatLat::A)), Box::new(EF::Identity));
        let c2 = EF::Composed(Box::new(EF::Constant(FlatLat::B)), Box::new(EF::Identity));
        let joined = c1.join_with(&c2);
        assert!(joined.is_top(), "different Composed join should be AllTop");
    }

    #[test]
    fn join_structurally_equal_composed_is_idempotent() {
        let c1 = EF::Composed(Box::new(EF::Constant(FlatLat::A)), Box::new(EF::Identity));
        let c2 = EF::Composed(Box::new(EF::Constant(FlatLat::A)), Box::new(EF::Identity));
        let joined = c1.join_with(&c2);
        // Structurally equal => idempotent, not AllTop.
        assert_eq!(joined.compute_target(FlatLat::B), FlatLat::A);
    }

    #[test]
    fn join_transition_table_entries_pruned_when_equal_to_default() {
        // Table1: A -> A, default A
        // Table2: A -> A, default A
        // Merged default = join(A, A) = A, entry A -> join(A, A) = A => pruned
        let t1 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::A)],
            default: FlatLat::A,
        };
        let t2 = EF::TransitionTable {
            entries: vec![(FlatLat::A, FlatLat::A)],
            default: FlatLat::A,
        };
        let joined = t1.join_with(&t2);
        // All inputs should return A (the default).
        assert_eq!(joined.compute_target(FlatLat::A), FlatLat::A);
        assert_eq!(joined.compute_target(FlatLat::B), FlatLat::A);
        // The result should be a TransitionTable with no entries (all pruned).
        if let EF::TransitionTable { entries, default } = &joined {
            assert!(entries.is_empty(), "entries should be pruned");
            assert_eq!(*default, FlatLat::A);
        }
        // Note: structural equality check uses the idempotent path,
        // so joined should equal t1 modulo pruning.
    }

    // ── equality and ordering tests ──────────────────────────────────

    #[test]
    fn structural_equality() {
        assert_eq!(EF::Identity, EF::Identity);
        assert_eq!(EF::AllTop, EF::AllTop);
        assert_eq!(EF::Constant(FlatLat::A), EF::Constant(FlatLat::A));
        assert_ne!(EF::Constant(FlatLat::A), EF::Constant(FlatLat::B));
        assert_ne!(EF::Identity, EF::AllTop);
    }

    #[test]
    fn ordering_is_deterministic() {
        let mut fns = vec![
            EF::Constant(FlatLat::B),
            EF::Identity,
            EF::AllTop,
            EF::Constant(FlatLat::A),
            EF::AllBottom,
        ];
        let fns2 = fns.clone();
        fns.sort();
        let mut fns3 = fns2;
        fns3.sort();
        assert_eq!(fns, fns3, "sorting should be deterministic");
    }

    // ── is_identity / is_top helpers ─────────────────────────────────

    #[test]
    fn is_identity_check() {
        assert!(EF::Identity.is_identity());
        assert!(!EF::AllTop.is_identity());
        assert!(!EF::Constant(FlatLat::A).is_identity());
    }

    #[test]
    fn is_top_check() {
        assert!(EF::AllTop.is_top());
        assert!(!EF::Identity.is_top());
        assert!(!EF::Constant(FlatLat::A).is_top());
    }

    #[test]
    fn custom_edge_fn_applies() {
        let f: BuiltinEdgeFn<FlatLat> = BuiltinEdgeFn::Custom(std::sync::Arc::new(|_| FlatLat::C));
        assert_eq!(f.compute_target(FlatLat::A), FlatLat::C);
        assert_eq!(f.compute_target(FlatLat::Top), FlatLat::C);
    }

    #[test]
    fn custom_edge_fn_structural_eq_by_pointer() {
        let arc_fn: std::sync::Arc<dyn Fn(FlatLat) -> FlatLat + Send + Sync> =
            std::sync::Arc::new(|_| FlatLat::C);
        let f1: BuiltinEdgeFn<FlatLat> = BuiltinEdgeFn::Custom(arc_fn.clone());
        let f2: BuiltinEdgeFn<FlatLat> = BuiltinEdgeFn::Custom(arc_fn);
        assert_eq!(f1, f2); // Same Arc pointer

        let f3: BuiltinEdgeFn<FlatLat> = BuiltinEdgeFn::Custom(std::sync::Arc::new(|_| FlatLat::C));
        assert_ne!(f1, f3); // Different Arc pointer
    }
}
