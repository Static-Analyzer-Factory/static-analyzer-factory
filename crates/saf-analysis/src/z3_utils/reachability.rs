//! Z3-based path-reachability queries.
//!
//! Given two program points, checks if any feasible CFG path connects them
//! by enumerating paths and checking Z3 guard feasibility.

use std::collections::VecDeque;

use saf_core::air::AirModule;
use saf_core::ids::{BlockId, FunctionId};

use crate::cfg::Cfg;

use super::solver::{FeasibilityResult, PathFeasibilityChecker, Z3FilterDiagnostics};
use crate::guard::{ValueLocationIndex, extract_guards_from_blocks};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of a path-reachability query.
#[derive(Debug, Clone)]
pub enum PathReachability {
    /// SAT — a feasible path exists (with witness block path).
    Reachable(Vec<BlockId>),
    /// All enumerated paths are UNSAT — unreachable.
    Unreachable,
    /// Timeout or max paths exceeded.
    Unknown,
}

/// Result of a Z3-based path-reachability query.
#[derive(Debug, Clone)]
pub struct PathReachabilityResult {
    /// The reachability verdict.
    pub result: PathReachability,
    /// Number of paths checked.
    pub paths_checked: usize,
    /// Z3 filtering diagnostics.
    pub diagnostics: Z3FilterDiagnostics,
}

// ---------------------------------------------------------------------------
// Path-reachability checker
// ---------------------------------------------------------------------------

/// Check if a feasible path exists between two blocks in the same function.
///
/// Enumerates CFG paths via BFS (up to `max_paths`), extracts branch
/// guards along each path, and checks Z3 feasibility. Returns the first
/// feasible path found as a witness.
pub fn check_path_reachable(
    from_block: BlockId,
    to_block: BlockId,
    func_id: FunctionId,
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
    max_paths: usize,
) -> PathReachabilityResult {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    let func = module.function(func_id);
    let cfg = match func {
        Some(f) if !f.is_declaration => Cfg::build(f),
        _ => {
            return PathReachabilityResult {
                result: PathReachability::Unknown,
                paths_checked: 0,
                diagnostics: Z3FilterDiagnostics::default(),
            };
        }
    };

    let mut diagnostics = Z3FilterDiagnostics::default();
    let mut paths_checked = 0;

    // Enumerate paths using BFS with path tracking
    let paths = enumerate_paths(from_block, to_block, &cfg, max_paths);

    diagnostics.total_items = paths.len();

    for path in &paths {
        paths_checked += 1;

        let block_seq: Vec<(FunctionId, BlockId)> = path.iter().map(|&b| (func_id, b)).collect();

        let pc = extract_guards_from_blocks(&block_seq, &index);
        diagnostics.guards_extracted += pc.guards.len();

        if pc.is_empty() {
            diagnostics.feasible_count += 1;
            return PathReachabilityResult {
                result: PathReachability::Reachable(path.clone()),
                paths_checked,
                diagnostics,
            };
        }

        if pc.guards.len() > max_guards {
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        diagnostics.z3_calls += 1;
        match checker.check_feasibility(&pc, &index) {
            FeasibilityResult::Feasible => {
                diagnostics.feasible_count += 1;
                return PathReachabilityResult {
                    result: PathReachability::Reachable(path.clone()),
                    paths_checked,
                    diagnostics,
                };
            }
            FeasibilityResult::Infeasible => {
                diagnostics.infeasible_count += 1;
            }
            FeasibilityResult::Unknown => {
                diagnostics.unknown_count += 1;
                diagnostics.z3_timeouts += 1;
            }
        }
    }

    // No feasible path found
    // Only return Unreachable if we actually checked some paths and Z3 proved all infeasible.
    // If no paths were enumerated (complex CFG, loops) or any were unknown, return Unknown.
    let result = if paths_checked == 0 || diagnostics.unknown_count > 0 {
        PathReachability::Unknown
    } else {
        PathReachability::Unreachable
    };

    PathReachabilityResult {
        result,
        paths_checked,
        diagnostics,
    }
}

/// Enumerate simple paths from `from` to `to` in a CFG using BFS.
///
/// Returns up to `max_paths` unique paths. Each path is a sequence of `BlockId`.
fn enumerate_paths(from: BlockId, to: BlockId, cfg: &Cfg, max_paths: usize) -> Vec<Vec<BlockId>> {
    if from == to {
        return vec![vec![from]];
    }

    let mut result = Vec::new();
    let mut queue: VecDeque<Vec<BlockId>> = VecDeque::new();
    queue.push_back(vec![from]);

    while let Some(path) = queue.pop_front() {
        if result.len() >= max_paths {
            break;
        }

        let current = *path.last().expect("path is non-empty from queue");

        if let Some(succs) = cfg.successors.get(&current) {
            for &succ in succs {
                // Avoid cycles: don't revisit blocks in the current path
                if path.contains(&succ) {
                    continue;
                }

                let mut new_path = path.clone();
                new_path.push(succ);

                if succ == to {
                    result.push(new_path);
                    if result.len() >= max_paths {
                        break;
                    }
                } else {
                    queue.push_back(new_path);
                }
            }
        }
    }

    result
}
