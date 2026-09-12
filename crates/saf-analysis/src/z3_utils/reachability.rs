//! Z3-based path-reachability queries.
//!
//! Given two program points, checks if any feasible CFG path connects them
//! by enumerating paths and checking Z3 guard feasibility.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::AirModule;
use saf_core::ids::{BlockId, FunctionId, ValueId};

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
    /// Satisfying model for the reported `Reachable` path: each symbolic
    /// operand `ValueId` (e.g. a `__VERIFIER_nondet_*` result) mapped to a
    /// concrete value. Empty for `Unreachable`/`Unknown` and for guard-free
    /// reaches that never invoke Z3. Seeds slice-1c concrete replay.
    pub model: BTreeMap<ValueId, i64>,
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
                model: BTreeMap::new(),
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
                // Guard-free reach: no solver ran, so any assignment is valid.
                model: BTreeMap::new(),
            };
        }

        if branch_guards > max_guards {
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        diagnostics.z3_calls += 1;
        let (feasibility, model) = checker.check_feasibility_with_model(&pc, &index);
        match feasibility {
            FeasibilityResult::Feasible => {
                diagnostics.feasible_count += 1;
                return PathReachabilityResult {
                    result: PathReachability::Reachable(path.clone()),
                    paths_checked,
                    diagnostics,
                    model,
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
        // No feasible path reported → no witness assignment.
        model: BTreeMap::new(),
    }
}

/// Enumerate up to `max_paths` simple block paths from `from` to `to` within a
/// single function's CFG.
///
/// Public wrapper over the internal path enumerator so interprocedural callers
/// (e.g. `saf-svcomp`'s R4 chain composer) can stitch per-frame paths into a
/// cross-function `(FunctionId, BlockId)` sequence. Returns an empty vec for a
/// missing/declared function. Deterministic (BFS order).
#[must_use]
pub fn block_paths_between(
    from: BlockId,
    to: BlockId,
    func_id: FunctionId,
    module: &AirModule,
    max_paths: usize,
) -> Vec<Vec<BlockId>> {
    let Some(func) = module.function(func_id) else {
        return Vec::new();
    };
    if func.is_declaration {
        return Vec::new();
    }
    let cfg = Cfg::build(func);
    enumerate_paths(from, to, &cfg, max_paths)
}

/// The set of blocks from which `to` is reachable along CFG successor edges
/// (`to` itself included). Computed by a backward BFS over the reverse CFG.
///
/// This is the block-level backward slice of `to`: a coarse abort-prune set. Any
/// simple path `from → … → to` passes exclusively through blocks in this set (each
/// prefix's remaining suffix witnesses that the prefix's tail reaches `to`), and
/// no block *outside* it lies on any path to `to`. So restricting a path search to
/// this set never drops a `from → to` path — it only skips dead subtrees.
fn blocks_reaching(to: BlockId, cfg: &Cfg) -> BTreeSet<BlockId> {
    let mut reaching: BTreeSet<BlockId> = BTreeSet::new();
    let mut queue: VecDeque<BlockId> = VecDeque::new();
    reaching.insert(to);
    queue.push_back(to);
    while let Some(b) = queue.pop_front() {
        if let Some(preds) = cfg.predecessors.get(&b) {
            for &p in preds {
                if reaching.insert(p) {
                    queue.push_back(p);
                }
            }
        }
    }
    reaching
}

/// Enumerate simple paths from `from` to `to` in a CFG using BFS.
///
/// Returns up to `max_paths` unique paths. Each path is a sequence of `BlockId`.
///
/// The BFS frontier is pruned to blocks from which `to` is still reachable (see
/// [`blocks_reaching`]). This is a **scalability multiplier, not a semantic
/// change**: the set of simple `from → to` paths — and the BFS order in which they
/// are discovered — is identical with or without the prune, because every block on
/// such a path can reach `to` by construction. The prune only stops the queue from
/// fanning out into error-irrelevant subtrees (whose partial paths never reach `to`
/// yet can blow up combinatorially — e.g. a diamond chain off to the side), which
/// on large CFGs is the difference between the enumerator finishing and exhausting
/// its budget. Callers (BMC base/incremental, the Z3 path checker, R4 interproc
/// composition) confirm every candidate downstream, so this can only ever change
/// latency, never a verdict.
fn enumerate_paths(from: BlockId, to: BlockId, cfg: &Cfg, max_paths: usize) -> Vec<Vec<BlockId>> {
    enumerate_paths_bounded(from, to, cfg, max_paths, MAX_QUEUED_BLOCKS)
}

/// Memory budget for the BFS frontier, counted in queued `BlockId`s (~16 bytes each,
/// so ~64 MB of path data).
///
/// `max_paths` bounds the RESULT vector, not the search. On a CFG where complete
/// paths are only discovered at the very end — a long chain of diamonds, which is
/// exactly the `eca-rers2012` state-machine shape — BFS holds 2^k full path clones at
/// level k and never reaches the `max_paths` break, so the frontier grows without
/// limit. Measured before this bound existed: **19 GB of resident memory on a 22 KB
/// source**, ending in an OOM kill that produced no output at all (1,641 tasks, 2.9%
/// of a full run, died this way and were scored as unattributable errors).
const MAX_QUEUED_BLOCKS: usize = 4_000_000;

/// Announce (at most once per process) that the frontier budget curtailed a search.
///
/// Diagnostic only — it changes no verdict. It exists because the budget is the ONLY
/// way this enumerator's results can differ from the unbounded version, so "did any
/// task hit it?" is exactly the question a recall-regression audit must answer, and
/// answering it by grepping one stderr line is far cheaper than a blind full-corpus
/// A/B.
fn report_frontier_budget_reached() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        eprintln!(
            "saf: path-enumeration frontier budget ({MAX_QUEUED_BLOCKS} blocks) reached -> search curtailed"
        );
    }
}

/// [`enumerate_paths`] with an explicit frontier budget (a seam for testing the
/// bound; production callers use [`MAX_QUEUED_BLOCKS`]).
///
/// When the budget is exhausted the search stops EXTENDING paths but keeps draining
/// what is already queued, so the result degrades to a prefix of the unbounded BFS
/// order rather than collapsing to nothing. Whenever the budget is not reached the
/// output is byte-identical to the unbounded search.
///
/// Curtailing the search can only lose candidates, never invent one: every caller
/// (BMC base/incremental, the Z3 path checker, R4 interproc composition) confirms
/// each candidate by concrete native replay downstream, so a missing path costs
/// recall and can never produce a wrong verdict — the same argument the reaching
/// prune above relies on.
fn enumerate_paths_bounded(
    from: BlockId,
    to: BlockId,
    cfg: &Cfg,
    max_paths: usize,
    max_queued_blocks: usize,
) -> Vec<Vec<BlockId>> {
    if from == to {
        return vec![vec![from]];
    }

    let reaching = blocks_reaching(to, cfg);
    // If `from` cannot reach `to`, no path exists — skip the search entirely.
    if !reaching.contains(&from) {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut queue: VecDeque<Vec<BlockId>> = VecDeque::new();
    let mut queued_blocks = 1usize;
    queue.push_back(vec![from]);

    while let Some(path) = queue.pop_front() {
        if result.len() >= max_paths {
            break;
        }
        queued_blocks = queued_blocks.saturating_sub(path.len());

        let current = *path.last().expect("path is non-empty from queue");

        if let Some(succs) = cfg.successors.get(&current) {
            for &succ in succs {
                // Avoid cycles: don't revisit blocks in the current path
                if path.contains(&succ) {
                    continue;
                }
                // Coarse abort-prune: a successor from which `to` is unreachable
                // can never extend into a `from → to` path, so skip it. Never
                // drops a real path (every block on one reaches `to`).
                if !reaching.contains(&succ) {
                    continue;
                }

                let mut new_path = path.clone();
                new_path.push(succ);

                if succ == to {
                    result.push(new_path);
                    if result.len() >= max_paths {
                        break;
                    }
                } else if queued_blocks + new_path.len() <= max_queued_blocks {
                    queued_blocks += new_path.len();
                    queue.push_back(new_path);
                } else {
                    report_frontier_budget_reached();
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

    /// A reachable *guarded* error path (branch `x != 0`) must carry a Z3 model
    /// assigning the nondet operand `x` a concrete value satisfying the guard —
    /// the seed slice 1c pins the nondet input to for concrete replay.
    #[test]
    fn model_returned_for_reachable_error_path() {
        let (main_id, _assume, entry, error, _exit) = ids();
        let module = build_module(false);

        let result = check_path_reachable(entry, error, main_id, &module, 5000, 50, 1000);
        assert!(matches!(result.result, PathReachability::Reachable(_)));

        let x = ValueId::new(100);
        let v = result
            .model
            .get(&x)
            .copied()
            .expect("reachable guarded error path must carry a model value for x");
        assert!(v != 0, "model value {v} must satisfy the guard x != 0");
    }
}

#[cfg(test)]
mod enumerate_prune_tests {
    use super::{MAX_QUEUED_BLOCKS, blocks_reaching, enumerate_paths, enumerate_paths_bounded};
    use crate::cfg::Cfg;
    use saf_core::ids::{BlockId, FunctionId};
    use std::collections::{BTreeMap, BTreeSet};

    /// Build a `Cfg` directly from an adjacency list (all fields are public).
    /// `edges` is `(block, [successors])`; the first block is the entry.
    fn cfg_from(edges: &[(u128, &[u128])]) -> Cfg {
        let mut successors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        let mut predecessors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        for &(b, _) in edges {
            successors.entry(BlockId::new(b)).or_default();
            predecessors.entry(BlockId::new(b)).or_default();
        }
        for &(b, succs) in edges {
            for &s in succs {
                successors
                    .entry(BlockId::new(b))
                    .or_default()
                    .insert(BlockId::new(s));
                predecessors
                    .entry(BlockId::new(s))
                    .or_default()
                    .insert(BlockId::new(b));
            }
        }
        let exits = successors
            .iter()
            .filter(|(_, s)| s.is_empty())
            .map(|(b, _)| *b)
            .collect();
        Cfg {
            function: FunctionId::new(1),
            entry: BlockId::new(edges[0].0),
            exits,
            successors,
            predecessors,
        }
    }

    /// Reference enumerator WITHOUT the reaching prune — the original algorithm.
    /// Used to prove the prune preserves the returned path vector exactly.
    fn enumerate_unpruned(
        from: BlockId,
        to: BlockId,
        cfg: &Cfg,
        max_paths: usize,
    ) -> Vec<Vec<BlockId>> {
        use std::collections::VecDeque;
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
            let current = *path.last().unwrap();
            if let Some(succs) = cfg.successors.get(&current) {
                for &succ in succs {
                    if path.contains(&succ) {
                        continue;
                    }
                    let mut np = path.clone();
                    np.push(succ);
                    if succ == to {
                        result.push(np);
                        if result.len() >= max_paths {
                            break;
                        }
                    } else {
                        queue.push_back(np);
                    }
                }
            }
        }
        result
    }

    /// `n` diamonds in series: 2^n distinct entry->exit paths, and BFS discovers NO
    /// complete path until the final level, so what grows is the FRONTIER, not the
    /// result vector. This is the `eca-rers2012` state-machine shape that drove SAF
    /// to 19 GB of RSS on a 22 KB source until the kernel OOM-killed it.
    fn diamond_chain(n: u128) -> (Cfg, BlockId, BlockId) {
        let mut edges: Vec<(u128, Vec<u128>)> = Vec::new();
        for i in 0..n {
            let (a, l, r, z) = (3 * i, 3 * i + 1, 3 * i + 2, 3 * i + 3);
            edges.push((a, vec![l, r]));
            edges.push((l, vec![z]));
            edges.push((r, vec![z]));
        }
        edges.push((3 * n, vec![]));
        let refs: Vec<(u128, &[u128])> = edges.iter().map(|(b, s)| (*b, s.as_slice())).collect();
        (cfg_from(&refs), BlockId::new(0), BlockId::new(3 * n))
    }

    #[test]
    fn frontier_memory_is_bounded_on_an_exploding_cfg() {
        // `max_paths` bounds the RESULT vector only. Without a frontier bound the
        // queue holds 2^k full path clones at level k, so a deep chain exhausts
        // memory long before `max_paths` results ever accumulate.
        let (cfg, from, to) = diamond_chain(12); // 4096 paths
        let unbounded = enumerate_paths_bounded(from, to, &cfg, 4096, usize::MAX);
        let bounded = enumerate_paths_bounded(from, to, &cfg, 4096, 64);
        assert_eq!(unbounded.len(), 4096, "the CFG really does have 2^12 paths");
        assert!(
            bounded.len() < unbounded.len(),
            "a tight frontier budget must curtail the search (got {} of {})",
            bounded.len(),
            unbounded.len()
        );
    }

    #[test]
    fn frontier_bound_is_inert_when_it_is_not_reached() {
        // The bound must be a pure memory backstop: on any CFG whose frontier fits,
        // the returned paths are byte-identical to the unbounded search, so ordinary
        // tasks see no behaviour change at all.
        let cfg = cfg_from(&[(0, &[1, 2]), (1, &[3]), (2, &[3]), (3, &[])]);
        let (from, to) = (BlockId::new(0), BlockId::new(3));
        assert_eq!(
            enumerate_paths_bounded(from, to, &cfg, 10, usize::MAX),
            enumerate_paths_bounded(from, to, &cfg, 10, MAX_QUEUED_BLOCKS),
        );
        let (cfg, from, to) = diamond_chain(6);
        assert_eq!(
            enumerate_paths_bounded(from, to, &cfg, 64, usize::MAX),
            enumerate_paths(from, to, &cfg, 64),
        );
    }

    #[test]
    fn reaching_set_is_backward_closure() {
        // 0 -> 1 -> 3(to) ; 0 -> 2 (dead, no edge to 3)
        let cfg = cfg_from(&[(0, &[1, 2]), (1, &[3]), (2, &[]), (3, &[])]);
        let reaching = blocks_reaching(BlockId::new(3), &cfg);
        assert!(reaching.contains(&BlockId::new(3)), "target reaches itself");
        assert!(reaching.contains(&BlockId::new(1)), "1 -> 3");
        assert!(reaching.contains(&BlockId::new(0)), "0 -> 1 -> 3");
        assert!(
            !reaching.contains(&BlockId::new(2)),
            "2 cannot reach the target"
        );
    }

    #[test]
    fn prune_preserves_the_path_vector_on_a_diamond() {
        // Diamond that reconverges before the target, plus a live back path — the
        // prune must return byte-identically what the unpruned enumerator does.
        let cfg = cfg_from(&[
            (0, &[1, 2]),
            (1, &[3]),
            (2, &[3]),
            (3, &[4]),
            (4, &[]), // to = 4
        ]);
        let (from, to) = (BlockId::new(0), BlockId::new(4));
        for max in [1usize, 2, 4, 8] {
            assert_eq!(
                enumerate_paths(from, to, &cfg, max),
                enumerate_unpruned(from, to, &cfg, max),
                "pruned enumeration must equal unpruned for max={max}"
            );
        }
    }

    #[test]
    fn prune_skips_a_dead_subtree() {
        // 0 -> 1(to). 0 also -> 2, and 2..=9 form an exponential diamond chain that
        // NEVER reaches 1. The unpruned BFS fans out into 2's subtree; the pruned
        // one ignores it. Both must return exactly the single real path [0,1].
        let cfg = cfg_from(&[
            (0, &[1, 2]),
            (1, &[]), // to
            // A side diamond chain rooted at 2, none of which reach 1.
            (2, &[3, 4]),
            (3, &[5]),
            (4, &[5]),
            (5, &[6, 7]),
            (6, &[8]),
            (7, &[8]),
            (8, &[]),
        ]);
        let (from, to) = (BlockId::new(0), BlockId::new(1));
        let pruned = enumerate_paths(from, to, &cfg, 8);
        assert_eq!(pruned, vec![vec![BlockId::new(0), BlockId::new(1)]]);
        assert_eq!(
            pruned,
            enumerate_unpruned(from, to, &cfg, 8),
            "result identical to the unpruned enumerator"
        );
    }

    #[test]
    fn unreachable_target_returns_empty() {
        // from=0 can only reach 1; to=2 is disconnected.
        let cfg = cfg_from(&[(0, &[1]), (1, &[]), (2, &[])]);
        assert!(enumerate_paths(BlockId::new(0), BlockId::new(2), &cfg, 8).is_empty());
    }
}
