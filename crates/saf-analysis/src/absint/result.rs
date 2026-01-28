//! Abstract interpretation result with query API.
//!
//! Provides access to computed invariants at block entries and
//! instruction points. Used by checkers to query value ranges.

use std::collections::BTreeMap;

use saf_core::ids::{BlockId, InstId, ValueId};

use super::fixpoint::FixpointDiagnostics;
use super::interval::Interval;
use super::state::AbstractState;

/// Result of abstract interpretation analysis.
///
/// Contains abstract states at block entries and instruction points,
/// plus diagnostics about the analysis process.
#[derive(Debug, Clone)]
pub struct AbstractInterpResult {
    /// Abstract state at each block entry.
    block_states: BTreeMap<BlockId, AbstractState>,
    /// Abstract state before each instruction (pre-state).
    inst_states: BTreeMap<InstId, AbstractState>,
    /// Constant map (value → interval for constants).
    /// This allows queries to resolve constant values even when not in the state.
    constant_map: BTreeMap<ValueId, Interval>,
    /// Diagnostics from the fixpoint computation.
    diag: FixpointDiagnostics,
}

/// Serializable diagnostics for Python export.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AbstractInterpDiagnostics {
    /// Total blocks analyzed across all functions.
    pub blocks_analyzed: u64,
    /// Number of widening applications.
    pub widening_applications: u64,
    /// Number of narrowing iterations performed.
    pub narrowing_iterations_performed: u32,
    /// Whether the analysis converged.
    pub converged: bool,
    /// Number of functions analyzed.
    pub functions_analyzed: u64,
}

impl AbstractInterpResult {
    /// Create a new result.
    pub(crate) fn new(
        block_states: BTreeMap<BlockId, AbstractState>,
        inst_states: BTreeMap<InstId, AbstractState>,
        constant_map: BTreeMap<ValueId, Interval>,
        diag: FixpointDiagnostics,
    ) -> Self {
        Self {
            block_states,
            inst_states,
            constant_map,
            diag,
        }
    }

    /// Get the abstract state at a block entry.
    #[must_use]
    pub fn state_at_block(&self, block: BlockId) -> Option<&AbstractState> {
        self.block_states.get(&block)
    }

    /// Get the abstract state before an instruction.
    #[must_use]
    pub fn state_at_inst(&self, inst: InstId) -> Option<&AbstractState> {
        self.inst_states.get(&inst)
    }

    /// Get the interval for a specific value at a block entry.
    ///
    /// First checks the abstract state, then falls back to the constant map.
    #[must_use]
    pub fn interval_at_block(&self, block: BlockId, value: ValueId, bits: u8) -> Interval {
        // Check abstract state first
        if let Some(state) = self.block_states.get(&block) {
            if let Some(interval) = state.get_opt(value) {
                return interval.clone();
            }
        }
        // Fall back to constant map
        if let Some(interval) = self.constant_map.get(&value) {
            return interval.clone();
        }
        // If block exists but value not found, return top (unknown)
        // If block doesn't exist, return bottom (unreachable)
        if self.block_states.contains_key(&block) {
            Interval::make_top(bits)
        } else {
            Interval::make_bottom(bits)
        }
    }

    /// Get the interval for a specific value before an instruction.
    ///
    /// First checks the abstract state, then falls back to the constant map.
    #[must_use]
    pub fn interval_at_inst(&self, inst: InstId, value: ValueId, bits: u8) -> Interval {
        // Check abstract state first
        if let Some(state) = self.inst_states.get(&inst) {
            if let Some(interval) = state.get_opt(value) {
                return interval.clone();
            }
        }
        // Fall back to constant map
        if let Some(interval) = self.constant_map.get(&value) {
            return interval.clone();
        }
        // If inst exists but value not found, return top (unknown)
        // If inst doesn't exist, return bottom (unreachable)
        if self.inst_states.contains_key(&inst) {
            Interval::make_top(bits)
        } else {
            Interval::make_bottom(bits)
        }
    }

    /// Get all tracked value intervals at a block entry.
    #[must_use]
    pub fn invariants_at_block(&self, block: BlockId) -> BTreeMap<ValueId, Interval> {
        self.block_states
            .get(&block)
            .map_or_else(BTreeMap::new, |s| s.entries().clone())
    }

    /// Get all tracked value intervals before an instruction.
    #[must_use]
    pub fn invariants_at_inst(&self, inst: InstId) -> BTreeMap<ValueId, Interval> {
        self.inst_states
            .get(&inst)
            .map_or_else(BTreeMap::new, |s| s.entries().clone())
    }

    /// Get diagnostics about the analysis.
    #[must_use]
    pub fn diagnostics(&self) -> AbstractInterpDiagnostics {
        AbstractInterpDiagnostics {
            blocks_analyzed: self.diag.blocks_analyzed,
            widening_applications: self.diag.widening_applications,
            narrowing_iterations_performed: self.diag.narrowing_iterations_performed,
            converged: self.diag.converged,
            functions_analyzed: self.diag.functions_analyzed,
        }
    }

    /// Get the total number of block states.
    #[must_use]
    pub fn block_count(&self) -> usize {
        self.block_states.len()
    }

    /// Get the total number of instruction states.
    #[must_use]
    pub fn inst_count(&self) -> usize {
        self.inst_states.len()
    }

    /// Access the constant map.
    ///
    /// Maps constant values (literals) to their intervals.
    #[must_use]
    pub fn constant_map(&self) -> &BTreeMap<ValueId, Interval> {
        &self.constant_map
    }

    /// Access the raw block states map.
    #[must_use]
    pub fn block_states(&self) -> &BTreeMap<BlockId, AbstractState> {
        &self.block_states
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::absint::domain::AbstractDomain;

    #[test]
    fn empty_result() {
        let result = AbstractInterpResult::new(
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            FixpointDiagnostics::default(),
        );
        assert_eq!(result.block_count(), 0);
        assert_eq!(result.inst_count(), 0);
        assert!(result.diagnostics().converged);
    }

    #[test]
    fn query_missing_block_returns_bottom() {
        let result = AbstractInterpResult::new(
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            FixpointDiagnostics::default(),
        );
        let val = result.interval_at_block(BlockId::new(999), ValueId::new(1), 32);
        assert!(val.is_bottom());
    }

    #[test]
    fn query_existing_block() {
        let mut block_states = BTreeMap::new();
        let mut state = AbstractState::new();
        state.set(ValueId::new(1), Interval::new(0, 10, 32));
        block_states.insert(BlockId::new(1), state);

        let result = AbstractInterpResult::new(
            block_states,
            BTreeMap::new(),
            BTreeMap::new(),
            FixpointDiagnostics::default(),
        );

        let val = result.interval_at_block(BlockId::new(1), ValueId::new(1), 32);
        assert_eq!(val.lo(), 0);
        assert_eq!(val.hi(), 10);
    }

    #[test]
    fn query_constant_fallback() {
        // Test that constants are found even when not in the abstract state
        let mut block_states = BTreeMap::new();
        let state = AbstractState::new();
        block_states.insert(BlockId::new(1), state);

        let mut constant_map = BTreeMap::new();
        constant_map.insert(ValueId::new(42), Interval::singleton(100, 32));

        let result = AbstractInterpResult::new(
            block_states,
            BTreeMap::new(),
            constant_map,
            FixpointDiagnostics::default(),
        );

        // Value 42 is a constant, should return [100, 100]
        let val = result.interval_at_block(BlockId::new(1), ValueId::new(42), 32);
        assert_eq!(val.lo(), 100);
        assert_eq!(val.hi(), 100);

        // Value 99 is not in state or constants, should return top (unknown)
        let unknown_val = result.interval_at_block(BlockId::new(1), ValueId::new(99), 32);
        assert!(unknown_val.is_top());
    }
}
