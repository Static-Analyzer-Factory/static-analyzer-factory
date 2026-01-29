//! Z3-based refinement for IFDS taint analysis.
//!
//! Reconstructs witness paths from IFDS results (facts-at-points → source-to-sink
//! traces), extracts branch guards along those paths, and uses Z3 to filter
//! infeasible taint flows.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};

use crate::cfg::Cfg;
use crate::z3_utils::{
    FeasibilityResult, PathFeasibilityChecker, ValueLocationIndex, Z3FilterDiagnostics,
    extract_guards_from_blocks,
};

use super::matches_name;
use super::result::IfdsResult;
use super::taint::TaintFact;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A witness path from taint source to sink, reconstructed from IFDS results.
#[derive(Debug, Clone)]
pub struct TaintWitnessPath {
    /// The instruction that generated the taint (source call).
    pub source_inst: InstId,
    /// The instruction where taint reaches sink.
    pub sink_inst: InstId,
    /// The tainted `ValueId` at the source.
    pub source_value: ValueId,
    /// The tainted `ValueId` at the sink.
    pub sink_value: ValueId,
    /// Block-level path from source to sink (for guard extraction).
    pub block_path: Vec<(FunctionId, BlockId)>,
}

/// Result of Z3-based IFDS taint refinement.
#[derive(Debug, Clone)]
pub struct TaintZ3Result {
    /// Taint flows confirmed as feasible by Z3.
    pub feasible: Vec<TaintWitnessPath>,
    /// Taint flows proven infeasible by Z3 (false positives).
    pub infeasible: Vec<TaintWitnessPath>,
    /// Taint flows where Z3 timed out or couldn't decide.
    pub unknown: Vec<TaintWitnessPath>,
    /// Z3 filtering diagnostics.
    pub diagnostics: Z3FilterDiagnostics,
}

// ---------------------------------------------------------------------------
// Witness path reconstruction
// ---------------------------------------------------------------------------

/// Reconstruct witness paths from IFDS taint results.
///
/// Scans for sink call instructions where tainted facts hold, then traces
/// backward to find the source generation point. Returns block-level paths
/// suitable for guard extraction.
// NOTE: This function builds lookup maps, scans for sinks, traces backward
// to sources, and constructs witness paths as a single cohesive pipeline.
// Splitting would obscure the data flow.
#[allow(clippy::too_many_lines)]
pub fn reconstruct_taint_paths(
    result: &IfdsResult<TaintFact>,
    module: &AirModule,
    source_functions: &BTreeSet<String>,
    sink_functions: &BTreeSet<String>,
) -> Vec<TaintWitnessPath> {
    // Build helper maps
    let mut inst_to_block: BTreeMap<InstId, (FunctionId, BlockId)> = BTreeMap::new();
    let mut source_insts: BTreeMap<InstId, ValueId> = BTreeMap::new(); // source call → generated value
    let mut sink_insts: Vec<(InstId, FunctionId, BlockId)> = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                inst_to_block.insert(inst.id, (func.id, block.id));

                // Identify source calls (generate taint on dst)
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(callee_func) = module.function(*callee) {
                        if source_functions
                            .iter()
                            .any(|s| matches_name(&callee_func.name, s))
                        {
                            if let Some(dst) = inst.dst {
                                source_insts.insert(inst.id, dst);
                            }
                        }
                        if sink_functions
                            .iter()
                            .any(|s| matches_name(&callee_func.name, s))
                        {
                            sink_insts.push((inst.id, func.id, block.id));
                        }
                    }
                }
            }
        }
    }

    let mut paths = Vec::new();

    // For each sink call with tainted arguments, find the source
    for (sink_inst_id, sink_func_id, sink_block_id) in &sink_insts {
        let Some(facts) = result.facts_at(*sink_inst_id) else {
            continue;
        };

        // Check which tainted values reach the sink
        // Get the sink instruction to check its operands
        let sink_inst = module
            .functions
            .iter()
            .flat_map(|f| f.blocks.iter())
            .flat_map(|b| b.instructions.iter())
            .find(|i| i.id == *sink_inst_id);

        let Some(sink_inst) = sink_inst else { continue };

        for fact in facts {
            let TaintFact::Tainted(tainted_vid) = fact else {
                continue;
            };

            // Check if the tainted value is used as an operand of the sink
            if !sink_inst.operands.contains(tainted_vid) {
                continue;
            }

            // Find which source generated this taint (or a value it derives from)
            for (src_inst_id, src_value) in &source_insts {
                let Some((src_func_id, src_block_id)) = inst_to_block.get(src_inst_id) else {
                    continue;
                };

                // Check if the source value's taint reaches the sink
                if !result.holds_at(*sink_inst_id, &TaintFact::Tainted(*src_value))
                    && !result.holds_at(*sink_inst_id, &TaintFact::Tainted(*tainted_vid))
                {
                    continue;
                }

                // Build block-level path from source to sink using BFS on CFG.
                if src_func_id == sink_func_id {
                    // Intraprocedural: BFS within the shared function's CFG.
                    if let Some(func) = module.function(*src_func_id) {
                        let cfg = Cfg::build(func);
                        if let Some(block_path) =
                            bfs_block_path(*src_block_id, *sink_block_id, &cfg)
                        {
                            let path: Vec<(FunctionId, BlockId)> =
                                block_path.into_iter().map(|b| (*src_func_id, b)).collect();

                            paths.push(TaintWitnessPath {
                                source_inst: *src_inst_id,
                                sink_inst: *sink_inst_id,
                                source_value: *src_value,
                                sink_value: *tainted_vid,
                                block_path: path,
                            });
                            break; // Found a source for this sink, move on
                        }
                    }
                } else {
                    // Interprocedural: source and sink are in different functions.
                    // We cannot do a single-CFG BFS across function boundaries,
                    // so we construct a witness path by concatenating the
                    // source-side suffix (source block to function exit) with
                    // the sink-side prefix (function entry to sink block).
                    // The Z3 feasibility checker works on the value-flow graph
                    // which already encodes interprocedural edges, so the guard
                    // extraction will still be sound — it simply extracts guards
                    // from whatever blocks are in the path.
                    let mut path: Vec<(FunctionId, BlockId)> = Vec::new();

                    // Source side: try to get blocks from source block to an exit.
                    if let Some(src_func) = module.function(*src_func_id) {
                        let src_cfg = Cfg::build(src_func);
                        // Find path from source block to any exit block.
                        let mut found_src_suffix = false;
                        for &exit_block in &src_cfg.exits {
                            if let Some(suffix) =
                                bfs_block_path(*src_block_id, exit_block, &src_cfg)
                            {
                                path.extend(suffix.into_iter().map(|b| (*src_func_id, b)));
                                found_src_suffix = true;
                                break;
                            }
                        }
                        if !found_src_suffix {
                            // Fallback: just include the source block.
                            path.push((*src_func_id, *src_block_id));
                        }
                    }

                    // Sink side: try to get blocks from entry to sink block.
                    if let Some(sink_func) = module.function(*sink_func_id) {
                        let sink_cfg = Cfg::build(sink_func);
                        if let Some(prefix) =
                            bfs_block_path(sink_cfg.entry, *sink_block_id, &sink_cfg)
                        {
                            path.extend(prefix.into_iter().map(|b| (*sink_func_id, b)));
                        } else {
                            // Fallback: just include the sink block.
                            path.push((*sink_func_id, *sink_block_id));
                        }
                    }

                    paths.push(TaintWitnessPath {
                        source_inst: *src_inst_id,
                        sink_inst: *sink_inst_id,
                        source_value: *src_value,
                        sink_value: *tainted_vid,
                        block_path: path,
                    });
                    break; // Found a source for this sink, move on
                }
            }
        }
    }

    paths
}

/// BFS to find shortest block-level path in a CFG.
fn bfs_block_path(from: BlockId, to: BlockId, cfg: &Cfg) -> Option<Vec<BlockId>> {
    if from == to {
        return Some(vec![from]);
    }

    let mut visited = BTreeSet::new();
    let mut parent: BTreeMap<BlockId, BlockId> = BTreeMap::new();
    let mut queue = VecDeque::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some(current) = queue.pop_front() {
        if let Some(succs) = cfg.successors.get(&current) {
            for &succ in succs {
                if !visited.contains(&succ) {
                    visited.insert(succ);
                    parent.insert(succ, current);
                    if succ == to {
                        // Reconstruct path
                        let mut path = vec![to];
                        let mut cur = to;
                        while let Some(&p) = parent.get(&cur) {
                            path.push(p);
                            cur = p;
                        }
                        path.reverse();
                        return Some(path);
                    }
                    queue.push_back(succ);
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Z3 filtering
// ---------------------------------------------------------------------------

/// Filter taint witness paths using Z3 feasibility checking.
///
/// For each witness path, extracts branch guards along the block path
/// and checks if they are satisfiable. Infeasible paths are false positives.
pub fn filter_taint_paths_z3(
    paths: Vec<TaintWitnessPath>,
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> TaintZ3Result {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    let mut feasible = Vec::new();
    let mut infeasible = Vec::new();
    let mut unknown = Vec::new();
    let mut diagnostics = Z3FilterDiagnostics {
        total_items: paths.len(),
        ..Default::default()
    };

    for path in paths {
        let pc = extract_guards_from_blocks(&path.block_path, &index);
        diagnostics.guards_extracted += pc.guards.len();

        if pc.is_empty() {
            feasible.push(path);
            diagnostics.feasible_count += 1;
            continue;
        }

        if pc.guards.len() > max_guards {
            unknown.push(path);
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        diagnostics.z3_calls += 1;
        match checker.check_feasibility(&pc, &index) {
            FeasibilityResult::Feasible => {
                feasible.push(path);
                diagnostics.feasible_count += 1;
            }
            FeasibilityResult::Infeasible => {
                infeasible.push(path);
                diagnostics.infeasible_count += 1;
            }
            FeasibilityResult::Unknown => {
                unknown.push(path);
                diagnostics.unknown_count += 1;
                diagnostics.z3_timeouts += 1;
            }
        }
    }

    TaintZ3Result {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_finds_shortest_path() {
        let entry = BlockId::new(0);
        let b1 = BlockId::new(1);
        let b2 = BlockId::new(2);

        let mut successors = BTreeMap::new();
        successors.insert(entry, [b1].into_iter().collect());
        successors.insert(b1, [b2].into_iter().collect());
        successors.insert(b2, BTreeSet::new());

        let cfg = Cfg {
            function: FunctionId::new(1),
            entry,
            exits: [b2].into_iter().collect(),
            successors,
            predecessors: BTreeMap::new(),
        };

        let path = bfs_block_path(entry, b2, &cfg);
        assert_eq!(path, Some(vec![entry, b1, b2]));
    }

    #[test]
    fn bfs_same_block() {
        let entry = BlockId::new(0);
        let cfg = Cfg {
            function: FunctionId::new(1),
            entry,
            exits: [entry].into_iter().collect(),
            successors: [(entry, BTreeSet::new())].into_iter().collect(),
            predecessors: BTreeMap::new(),
        };

        let path = bfs_block_path(entry, entry, &cfg);
        assert_eq!(path, Some(vec![entry]));
    }

    #[test]
    fn bfs_unreachable() {
        let entry = BlockId::new(0);
        let b1 = BlockId::new(1);

        let mut successors = BTreeMap::new();
        successors.insert(entry, BTreeSet::new());
        successors.insert(b1, BTreeSet::new());

        let cfg = Cfg {
            function: FunctionId::new(1),
            entry,
            exits: [entry, b1].into_iter().collect(),
            successors,
            predecessors: BTreeMap::new(),
        };

        let path = bfs_block_path(entry, b1, &cfg);
        assert_eq!(path, None);
    }

    #[test]
    fn matches_name_glob() {
        assert!(matches_name("getenv", "getenv"));
        assert!(matches_name("getenv", "get*"));
        assert!(matches_name("getenv", "*env"));
        assert!(matches_name("anything", "*"));
        assert!(!matches_name("getenv", "setenv"));
    }
}
