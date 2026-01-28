//! Z3-based refinement for ValueFlow taint analysis.
//!
//! Extracts branch guards along existing BFS taint_flow paths and uses Z3
//! to filter infeasible flows. Simpler than IFDS Z3 refinement because
//! ValueFlow already provides source→sink traces.

use saf_core::air::AirModule;

use crate::z3_utils::{
    FeasibilityResult, PathFeasibilityChecker, ValueLocationIndex, Z3FilterDiagnostics,
    extract_guards_from_blocks,
};

use super::node::NodeId;
use super::query::Flow;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of Z3-based ValueFlow taint refinement.
#[derive(Debug, Clone)]
pub struct TaintFlowZ3Result {
    /// Taint flows confirmed as feasible by Z3.
    pub feasible: Vec<Flow>,
    /// Taint flows proven infeasible by Z3 (false positives).
    pub infeasible: Vec<Flow>,
    /// Taint flows where Z3 timed out or couldn't decide.
    pub unknown: Vec<Flow>,
    /// Z3 filtering diagnostics.
    pub diagnostics: Z3FilterDiagnostics,
}

// ---------------------------------------------------------------------------
// Z3 filtering for ValueFlow taint flows
// ---------------------------------------------------------------------------

/// Filter ValueFlow taint flows using Z3 feasibility checking.
///
/// For each flow, maps trace steps' `NodeId::Value` to blocks via
/// `ValueLocationIndex`, extracts guards between consecutive block crossings,
/// and checks Z3 feasibility.
pub fn filter_taint_flows_z3(
    flows: Vec<Flow>,
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> TaintFlowZ3Result {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    let mut feasible = Vec::new();
    let mut infeasible = Vec::new();
    let mut unknown = Vec::new();
    let mut diagnostics = Z3FilterDiagnostics {
        total_items: flows.len(),
        ..Default::default()
    };

    for flow in flows {
        // Extract block sequence from the trace
        let block_seq = trace_to_block_sequence(&flow, &index);

        let pc = extract_guards_from_blocks(&block_seq, &index);
        diagnostics.guards_extracted += pc.guards.len();

        if pc.is_empty() {
            feasible.push(flow);
            diagnostics.feasible_count += 1;
            continue;
        }

        if pc.guards.len() > max_guards {
            unknown.push(flow);
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        diagnostics.z3_calls += 1;
        match checker.check_feasibility(&pc, &index) {
            FeasibilityResult::Feasible => {
                feasible.push(flow);
                diagnostics.feasible_count += 1;
            }
            FeasibilityResult::Infeasible => {
                infeasible.push(flow);
                diagnostics.infeasible_count += 1;
            }
            FeasibilityResult::Unknown => {
                unknown.push(flow);
                diagnostics.unknown_count += 1;
                diagnostics.z3_timeouts += 1;
            }
        }
    }

    TaintFlowZ3Result {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}

/// Convert a ValueFlow trace to a block sequence for guard extraction.
///
/// Maps `NodeId::Value(vid)` → `(FunctionId, BlockId)` and deduplicates
/// consecutive same-block entries.
fn trace_to_block_sequence(
    flow: &Flow,
    index: &ValueLocationIndex,
) -> Vec<(saf_core::ids::FunctionId, saf_core::ids::BlockId)> {
    let mut blocks = Vec::new();

    // Start with source node — map source ValueId to block
    if let Some(loc) = index.block_of(flow.source) {
        blocks.push(loc);
    }

    // Walk through trace steps
    for step in &flow.trace.steps {
        if let NodeId::Value { id: vid } = step.to {
            if let Some(loc) = index.block_of(vid) {
                // Deduplicate consecutive same-block entries
                if blocks.last() != Some(&loc) {
                    blocks.push(loc);
                }
            }
        }
    }

    blocks
}
