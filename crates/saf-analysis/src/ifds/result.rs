//! IFDS analysis result types.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{FunctionId, InstId};

/// Result of an IFDS analysis.
#[derive(Debug, Clone)]
pub struct IfdsResult<F: Ord + Clone> {
    /// Facts holding at each instruction (after the instruction executes).
    pub facts: BTreeMap<InstId, BTreeSet<F>>,
    /// Summary edges per function: `(entry_fact, exit_fact)` pairs.
    pub summaries: BTreeMap<FunctionId, BTreeSet<(F, F)>>,
    /// Solver diagnostics.
    pub diagnostics: IfdsDiagnostics,
}

/// Solver diagnostics for performance monitoring.
#[derive(Debug, Clone, Default)]
pub struct IfdsDiagnostics {
    /// Total worklist iterations performed.
    pub iterations: usize,
    /// Total path edges explored.
    pub path_edges_explored: usize,
    /// Total summary edges created.
    pub summary_edges_created: usize,
    /// Peak number of facts at any single program point.
    pub facts_at_peak: usize,
    /// Whether the solver hit a configured limit.
    pub reached_limit: bool,
}

impl<F: Ord + Clone> IfdsResult<F> {
    /// Get facts at a specific instruction.
    #[must_use]
    pub fn facts_at(&self, inst: InstId) -> Option<&BTreeSet<F>> {
        self.facts.get(&inst)
    }

    /// Check if a specific fact holds at a specific instruction.
    #[must_use]
    pub fn holds_at(&self, inst: InstId, fact: &F) -> bool {
        self.facts.get(&inst).is_some_and(|fs| fs.contains(fact))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_at_returns_correct_boolean() {
        let mut facts = BTreeMap::new();
        let mut set = BTreeSet::new();
        set.insert(42_u64);
        set.insert(99);
        facts.insert(InstId::new(1), set);

        let result: IfdsResult<u64> = IfdsResult {
            facts,
            summaries: BTreeMap::new(),
            diagnostics: IfdsDiagnostics::default(),
        };

        assert!(result.holds_at(InstId::new(1), &42));
        assert!(result.holds_at(InstId::new(1), &99));
        assert!(!result.holds_at(InstId::new(1), &0));
    }

    #[test]
    fn facts_at_returns_none_for_unknown() {
        let result: IfdsResult<u64> = IfdsResult {
            facts: BTreeMap::new(),
            summaries: BTreeMap::new(),
            diagnostics: IfdsDiagnostics::default(),
        };

        assert!(result.facts_at(InstId::new(999)).is_none());
    }

    #[test]
    fn holds_at_returns_false_for_unknown_instruction() {
        let result: IfdsResult<u64> = IfdsResult {
            facts: BTreeMap::new(),
            summaries: BTreeMap::new(),
            diagnostics: IfdsDiagnostics::default(),
        };

        assert!(!result.holds_at(InstId::new(999), &42));
    }
}
