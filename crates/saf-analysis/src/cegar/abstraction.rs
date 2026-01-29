//! Abstraction state for CEGAR refinement.
//!
//! Tracks the current abstraction level and provides refinement operations
//! based on spurious counterexample analysis.

use std::collections::BTreeSet;

use saf_core::ids::ValueId;

/// A predicate for path sensitivity tracking.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Predicate {
    /// The condition being tracked.
    pub condition: PredicateCondition,
    /// Source location (for debugging).
    pub source_line: Option<u32>,
}

/// Types of conditions that can be tracked as predicates.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PredicateCondition {
    /// Value is null.
    IsNull(ValueId),
    /// Value is non-null.
    IsNonNull(ValueId),
    /// Comparison between two values.
    Compare {
        left: ValueId,
        op: CompareOp,
        right: ValueId,
    },
    /// Value equals a constant.
    ConstEq { value: ValueId, constant: i128 },
}

/// Comparison operators for predicates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompareOp {
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
}

impl CompareOp {
    /// Get the negation of this comparison.
    #[must_use]
    pub const fn negate(self) -> Self {
        match self {
            Self::Lt => Self::Ge,
            Self::Le => Self::Gt,
            Self::Gt => Self::Le,
            Self::Ge => Self::Lt,
            Self::Eq => Self::Ne,
            Self::Ne => Self::Eq,
        }
    }
}

/// Abstraction level for CEGAR.
///
/// Controls the precision of various analyses. Starting with a coarse
/// abstraction and refining based on spurious counterexamples enables
/// efficient verification.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Abstraction {
    /// Context sensitivity level for PTA (k-CFA depth).
    /// 0 = context-insensitive, 1 = 1-CFA, etc.
    pub pta_k: usize,

    /// Whether to use flow-sensitive pointer analysis.
    pub flow_sensitive: bool,

    /// Tracked predicates for path sensitivity.
    pub predicates: BTreeSet<Predicate>,

    /// Variables to track precisely (others may be widened aggressively).
    pub tracked_vars: BTreeSet<ValueId>,

    /// Whether to use strong updates in pointer analysis.
    pub strong_updates: bool,

    /// Whether to use field sensitivity.
    pub field_sensitive: bool,
}

impl Abstraction {
    /// Create initial coarse abstraction.
    ///
    /// Starts with minimal precision for fast initial analysis.
    #[must_use]
    pub fn initial() -> Self {
        Self {
            pta_k: 0, // Context-insensitive
            flow_sensitive: false,
            predicates: BTreeSet::new(),
            tracked_vars: BTreeSet::new(),
            strong_updates: false,
            field_sensitive: false,
        }
    }

    /// Create a precise abstraction (for comparison).
    #[must_use]
    pub fn precise() -> Self {
        Self {
            pta_k: 2,
            flow_sensitive: true,
            predicates: BTreeSet::new(),
            tracked_vars: BTreeSet::new(),
            strong_updates: true,
            field_sensitive: true,
        }
    }

    /// Refine the abstraction based on refinement hints.
    #[must_use]
    pub fn refine(&self, hints: &RefinementHints) -> Self {
        let mut refined = self.clone();

        // Add new predicates
        for predicate in &hints.new_predicates {
            refined.predicates.insert(predicate.clone());
        }

        // Add variables to track
        for var in &hints.track_vars {
            refined.tracked_vars.insert(*var);
        }

        // Increase context sensitivity if requested
        if hints.increase_context_sensitivity {
            refined.pta_k = refined.pta_k.saturating_add(1).min(3);
        }

        // Enable flow sensitivity if requested
        if hints.enable_flow_sensitivity {
            refined.flow_sensitive = true;
        }

        // Enable strong updates if requested
        if hints.enable_strong_updates {
            refined.strong_updates = true;
        }

        refined
    }

    /// Check if this abstraction is more precise than another.
    #[must_use]
    pub fn is_more_precise_than(&self, other: &Self) -> bool {
        self.pta_k > other.pta_k
            || (self.flow_sensitive && !other.flow_sensitive)
            || self.predicates.is_superset(&other.predicates)
            || self.tracked_vars.is_superset(&other.tracked_vars)
    }

    /// Check if maximum refinement has been reached.
    #[must_use]
    pub fn at_max_precision(&self) -> bool {
        // Consider max precision reached when:
        // - k-CFA depth is at max (3)
        // - Flow-sensitive is enabled
        // - Strong updates enabled
        self.pta_k >= 3 && self.flow_sensitive && self.strong_updates
    }
}

/// Hints for how to refine the abstraction.
#[derive(Clone, Debug, Default)]
pub struct RefinementHints {
    /// New predicates to add for path sensitivity.
    pub new_predicates: Vec<Predicate>,

    /// Variables that need precise tracking.
    pub track_vars: Vec<ValueId>,

    /// Whether to increase context sensitivity.
    pub increase_context_sensitivity: bool,

    /// Whether to enable flow sensitivity.
    pub enable_flow_sensitivity: bool,

    /// Whether to enable strong updates.
    pub enable_strong_updates: bool,
}

impl RefinementHints {
    /// Create empty refinement hints.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add a predicate to track.
    pub fn add_predicate(&mut self, predicate: Predicate) {
        self.new_predicates.push(predicate);
    }

    /// Add a variable to track precisely.
    pub fn track_variable(&mut self, var: ValueId) {
        self.track_vars.push(var);
    }

    /// Request increased context sensitivity.
    pub fn request_more_context(&mut self) {
        self.increase_context_sensitivity = true;
    }

    /// Request flow sensitivity.
    pub fn request_flow_sensitivity(&mut self) {
        self.enable_flow_sensitivity = true;
    }

    /// Check if any refinement is suggested.
    #[must_use]
    pub fn has_refinement(&self) -> bool {
        !self.new_predicates.is_empty()
            || !self.track_vars.is_empty()
            || self.increase_context_sensitivity
            || self.enable_flow_sensitivity
            || self.enable_strong_updates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_abstraction() {
        let abs = Abstraction::initial();
        assert_eq!(abs.pta_k, 0);
        assert!(!abs.flow_sensitive);
        assert!(abs.predicates.is_empty());
    }

    #[test]
    fn test_refinement() {
        let abs = Abstraction::initial();

        let mut hints = RefinementHints::empty();
        hints.increase_context_sensitivity = true;
        hints.enable_flow_sensitivity = true;

        let refined = abs.refine(&hints);
        assert_eq!(refined.pta_k, 1);
        assert!(refined.flow_sensitive);
    }

    #[test]
    fn test_max_precision() {
        let mut abs = Abstraction::initial();
        abs.pta_k = 3;
        abs.flow_sensitive = true;
        abs.strong_updates = true;

        assert!(abs.at_max_precision());
    }

    #[test]
    fn test_compare_op_negate() {
        assert_eq!(CompareOp::Lt.negate(), CompareOp::Ge);
        assert_eq!(CompareOp::Le.negate(), CompareOp::Gt);
        assert_eq!(CompareOp::Eq.negate(), CompareOp::Ne);
    }

    #[test]
    fn test_refinement_hints_has_refinement() {
        let empty = RefinementHints::empty();
        assert!(!empty.has_refinement());

        let mut with_context = RefinementHints::empty();
        with_context.increase_context_sensitivity = true;
        assert!(with_context.has_refinement());
    }
}
