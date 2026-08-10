//! Z3-based path-reachability queries.
//!
//! Given two program points, checks if any feasible CFG path connects them
//! by enumerating paths and checking Z3 guard feasibility.

use std::collections::VecDeque;

use saf_core::air::AirModule;
use saf_core::ids::{BlockId, FunctionId};

use crate::cfg::Cfg;

use super::solver::{FeasibilityResult, PathFeasibilityChecker, Z3FilterDiagnostics};
use crate::guard::{ValueLocationIndex, extract_assume_guards, extract_guards_from_blocks};

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

        let mut pc = extract_guards_from_blocks(&block_seq, &index);
        // The `max_guards` complexity bound applies to branch guards (path
        // conditions); assume constraints are folded in on top and are exempt.
        let branch_guards = pc.guards.len();

        // Fold in `__VERIFIER_assume(cond)` constraints along this path so
        // infeasible (assume-blocked) error paths are pruned. This must run
        // before the is_empty() short-circuit below, otherwise a guard-free path
        // carrying only an assume would be reported Reachable without Z3.
        pc.guards
            .extend(extract_assume_guards(&block_seq, module, &index));
        diagnostics.guards_extracted += pc.guards.len();

        if pc.is_empty() {
            diagnostics.feasible_count += 1;
            return PathReachabilityResult {
                result: PathReachability::Reachable(path.clone()),
                paths_checked,
                diagnostics,
            };
        }

        if branch_guards > max_guards {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod assume_tests {
    use super::*;
    use saf_core::air::{
        AirBlock, AirFunction, BinaryOp, CastKind, Constant, Instruction, Operation,
    };
    use saf_core::ids::{InstId, ModuleId, ValueId};
    use std::collections::BTreeMap;

    // Value / block / function ids shared by the fixtures below.
    fn ids() -> (FunctionId, FunctionId, BlockId, BlockId, BlockId) {
        (
            FunctionId::new(1), // main
            FunctionId::new(2), // __VERIFIER_assume (declaration)
            BlockId::new(10),   // entry
            BlockId::new(11),   // error
            BlockId::new(12),   // exit
        )
    }

    fn decl_assume(assume_id: FunctionId) -> AirFunction {
        AirFunction {
            id: assume_id,
            name: "__VERIFIER_assume".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn ret_block(id: BlockId, inst: u128) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions: vec![Instruction::new(InstId::new(inst), Operation::Ret)],
        }
    }

    /// A `main` whose entry block branches to `error` on `x != 0`, optionally
    /// preceded by `__VERIFIER_assume(x == 0)`. Returns the module.
    fn build_module(with_assume: bool) -> AirModule {
        let (main_id, assume_id, entry, error, exit) = ids();

        let x = ValueId::new(100);
        let zero = ValueId::new(101);
        let cond_eq = ValueId::new(102);
        let conv = ValueId::new(103);
        let cond_ne = ValueId::new(104);

        let mut insts = Vec::new();
        if with_assume {
            // %cond_eq = icmp eq i32 %x, 0
            insts.push(
                Instruction::new(
                    InstId::new(1000),
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpEq,
                    },
                )
                .with_operands(vec![x, zero])
                .with_dst(cond_eq),
            );
            // %conv = zext i1 %cond_eq to i32   (this is what `assume(x == 0)` lowers to)
            insts.push(
                Instruction::new(
                    InstId::new(1001),
                    Operation::Cast {
                        kind: CastKind::ZExt,
                        target_bits: Some(32),
                    },
                )
                .with_operands(vec![cond_eq])
                .with_dst(conv),
            );
            // call void @__VERIFIER_assume(i32 %conv)
            insts.push(
                Instruction::new(
                    InstId::new(1002),
                    Operation::CallDirect { callee: assume_id },
                )
                .with_operands(vec![conv]),
            );
        }
        // %cond_ne = icmp ne i32 %x, 0
        insts.push(
            Instruction::new(
                InstId::new(1003),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpNe,
                },
            )
            .with_operands(vec![x, zero])
            .with_dst(cond_ne),
        );
        // condbr %cond_ne, error, exit
        insts.push(
            Instruction::new(
                InstId::new(1004),
                Operation::CondBr {
                    then_target: error,
                    else_target: exit,
                },
            )
            .with_operands(vec![cond_ne]),
        );

        let entry_block = AirBlock {
            id: entry,
            label: Some("entry".to_string()),
            instructions: insts,
        };

        let main = AirFunction {
            id: main_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![entry_block, ret_block(error, 1005), ret_block(exit, 1006)],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module
            .constants
            .insert(zero, Constant::Int { value: 0, bits: 32 });
        module.functions.push(main);
        module.functions.push(decl_assume(assume_id));
        module
    }

    /// Sanity: without any assume, the `x != 0` error block IS reachable.
    #[test]
    fn error_reachable_without_assume() {
        let (main_id, _assume, entry, error, _exit) = ids();
        let module = build_module(false);

        let result = check_path_reachable(entry, error, main_id, &module, 5000, 50, 1000);
        assert!(
            matches!(result.result, PathReachability::Reachable(_)),
            "x != 0 error path should be reachable when unconstrained; got {:?}",
            result.result
        );
    }

    /// `__VERIFIER_assume(x == 0)` must prune the `x != 0` error path: the
    /// conjunction `x == 0 ∧ x != 0` is UNSAT, so the error is Unreachable.
    #[test]
    fn assume_prunes_infeasible_error_path() {
        let (main_id, _assume, entry, error, _exit) = ids();
        let module = build_module(true);

        let result = check_path_reachable(entry, error, main_id, &module, 5000, 50, 1000);
        assert!(
            matches!(result.result, PathReachability::Unreachable),
            "assume(x == 0) must make the x != 0 error path Unreachable; got {:?}",
            result.result
        );
    }
}
