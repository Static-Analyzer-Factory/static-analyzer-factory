//! Fast-path checks for SV-COMP property analysis.
//!
//! These functions provide quick checks that can prove properties hold
//! without running expensive full analyses.

use std::collections::{BTreeMap, BTreeSet};

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::graph_algo::dfs;
use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, ValueId};

// ---------------------------------------------------------------------------
// P0: Threading Detection
// ---------------------------------------------------------------------------

/// Threading-related function names that indicate a concurrent program.
const THREAD_FUNCTIONS: &[&str] = &[
    // POSIX threads
    "pthread_create",
    "pthread_join",
    "pthread_mutex_lock",
    "pthread_mutex_unlock",
    "pthread_cond_wait",
    "pthread_cond_signal",
    // C11 threads
    "thrd_create",
    "thrd_join",
    "mtx_lock",
    "mtx_unlock",
    // SV-COMP atomics
    "__VERIFIER_atomic_begin",
    "__VERIFIER_atomic_end",
    // Process-level concurrency (conservative)
    "fork",
];

/// Check if the program has any threading primitives.
///
/// Returns true if the program declares or uses any threading functions.
/// This is used for the `no-data-race` fast-path: programs without threading
/// primitives cannot have data races.
pub fn has_threading_primitives(module: &AirModule) -> bool {
    // Check for threading function declarations
    for func in &module.functions {
        if THREAD_FUNCTIONS.contains(&func.name.as_str()) {
            return true;
        }
    }

    // Also check for indirect calls to threading functions via function pointers
    // (conservative: if we see a CallIndirect, we can't rule out threading)
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // Check for direct calls to threading functions
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if THREAD_FUNCTIONS.contains(&target.name.as_str()) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

// ---------------------------------------------------------------------------
// P3: Loop Detection
// ---------------------------------------------------------------------------

/// Check if a CFG has any loops (back-edges).
///
/// A loop-free CFG allows exact interval analysis without widening.
/// Uses DFS to detect cycles in the control flow graph.
pub fn cfg_has_loops(cfg: &Cfg) -> bool {
    use std::collections::BTreeSet;

    fn dfs(
        block: BlockId,
        cfg: &Cfg,
        visited: &mut BTreeSet<BlockId>,
        in_stack: &mut BTreeSet<BlockId>,
    ) -> bool {
        if in_stack.contains(&block) {
            // Found a back edge (cycle)
            return true;
        }
        if visited.contains(&block) {
            return false;
        }

        visited.insert(block);
        in_stack.insert(block);

        if let Some(successors) = cfg.successors.get(&block) {
            for succ in successors {
                if dfs(*succ, cfg, visited, in_stack) {
                    return true;
                }
            }
        }

        in_stack.remove(&block);
        false
    }

    // DFS-based cycle detection
    // A back edge exists if during DFS we visit a node that's in the current path
    let mut visited = BTreeSet::new();
    let mut in_stack = BTreeSet::new();

    dfs(cfg.entry, cfg, &mut visited, &mut in_stack)
}

/// Check if all functions in the program are loop-free.
///
/// For loop-free programs, abstract interpretation produces exact intervals,
/// so overflow detection is precise.
pub fn program_is_loop_free(cfgs: &BTreeMap<FunctionId, Cfg>) -> bool {
    cfgs.values().all(|cfg| !cfg_has_loops(cfg))
}

// ---------------------------------------------------------------------------
// P4: Heap Allocation Detection
// ---------------------------------------------------------------------------

/// Heap allocation function names.
const ALLOC_FUNCTIONS: &[&str] = &[
    // Standard C
    "malloc",
    "calloc",
    "realloc",
    "free",
    "aligned_alloc",
    "posix_memalign",
    "memalign",
    // C++
    "_Znwm",  // operator new(size_t)
    "_Znam",  // operator new[](size_t)
    "_ZdlPv", // operator delete(void*)
    "_ZdaPv", // operator delete[](void*)
    // SV-COMP special (may allocate)
    "__VERIFIER_nondet_pointer",
];

/// Check if the program has any heap allocations.
///
/// Returns true if the program uses malloc/free or similar allocation functions.
/// Stack-only programs cannot have UAF, double-free, or memory leaks.
pub fn has_heap_allocations(module: &AirModule) -> bool {
    // Check for allocation function declarations
    for func in &module.functions {
        if ALLOC_FUNCTIONS.contains(&func.name.as_str()) {
            return true;
        }
    }

    // Check for HeapAlloc operations in AIR
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::HeapAlloc { .. }) {
                    return true;
                }
                // Check for direct calls to allocation functions
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if ALLOC_FUNCTIONS.contains(&target.name.as_str()) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Check if the program only uses stack allocations.
///
/// Stack-only programs cannot have UAF or double-free (only null-deref is possible).
pub fn is_stack_only(module: &AirModule) -> bool {
    !has_heap_allocations(module)
}

// ---------------------------------------------------------------------------
// Termination Detection
// ---------------------------------------------------------------------------

/// Nondeterministic function names that can produce arbitrary values.
const NONDET_FUNCTIONS: &[&str] = &[
    "__VERIFIER_nondet_int",
    "__VERIFIER_nondet_uint",
    "__VERIFIER_nondet_long",
    "__VERIFIER_nondet_ulong",
    "__VERIFIER_nondet_short",
    "__VERIFIER_nondet_ushort",
    "__VERIFIER_nondet_char",
    "__VERIFIER_nondet_uchar",
    "__VERIFIER_nondet_bool",
    "__VERIFIER_nondet_float",
    "__VERIFIER_nondet_double",
    "__VERIFIER_nondet_pointer",
    "__VERIFIER_nondet_size_t",
    "__VERIFIER_nondet_loff_t",
    "__VERIFIER_nondet_pchar",
    "__VERIFIER_nondet_u32",
    "__VERIFIER_nondet_charp",
];

/// Information about a potentially nonterminating loop.
#[derive(Debug, Clone)]
pub struct NonterminatingLoop {
    /// Function containing the loop.
    pub function: FunctionId,
    /// Block containing the loop header.
    pub loop_header: BlockId,
    /// Whether the loop condition depends on nondeterministic input.
    pub depends_on_nondet: bool,
}

/// Find loops that may not terminate.
///
/// A loop may not terminate if:
/// - Its condition depends on `__VERIFIER_nondet_*` (can be true infinitely)
/// - It has no termination condition (infinite loop)
///
/// Returns a list of potentially nonterminating loops.
pub fn find_nonterminating_loops(
    module: &AirModule,
    cfgs: &BTreeMap<FunctionId, Cfg>,
) -> Vec<NonterminatingLoop> {
    use saf_core::ids::ValueId;
    use std::collections::BTreeSet;

    let mut results = Vec::new();

    // Find all calls to nondet functions and track their return values
    let mut nondet_values: BTreeSet<ValueId> = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if NONDET_FUNCTIONS.contains(&target.name.as_str()) {
                            // The return value of this call is nondeterministic
                            if let Some(dst) = inst.dst {
                                nondet_values.insert(dst);
                            }
                        }
                    }
                }
            }
        }
    }

    // For each function, find loops and check their conditions
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        let Some(cfg) = cfgs.get(&func.id) else {
            continue;
        };

        // Find back edges (loops)
        let back_edges = find_back_edges(cfg);

        for (_, header) in back_edges {
            // Check if the loop header's branch condition depends on nondet
            let Some(header_block) = func.blocks.iter().find(|b| b.id == header) else {
                continue;
            };

            // Check the terminator instruction for branch conditions
            let depends_on_nondet = if let Some(term) = header_block.terminator() {
                match &term.op {
                    // CondBr: condition is in operands[0]
                    Operation::CondBr { .. } => term
                        .operands
                        .first()
                        .is_some_and(|v| nondet_values.contains(v)),
                    // Switch: discriminant is in operands[0]
                    Operation::Switch { .. } => term
                        .operands
                        .first()
                        .is_some_and(|v| nondet_values.contains(v)),
                    _ => false,
                }
            } else {
                false
            };

            // A while(nondet()) loop may not terminate
            if depends_on_nondet {
                results.push(NonterminatingLoop {
                    function: func.id,
                    loop_header: header,
                    depends_on_nondet: true,
                });
            }
        }
    }

    results
}

/// Find back edges in a CFG using DFS.
/// Returns pairs of (source, target) where the edge goes back to an ancestor.
fn find_back_edges(cfg: &Cfg) -> Vec<(BlockId, BlockId)> {
    use std::collections::BTreeSet;

    fn dfs(
        block: BlockId,
        cfg: &Cfg,
        visited: &mut BTreeSet<BlockId>,
        in_stack: &mut BTreeSet<BlockId>,
        back_edges: &mut Vec<(BlockId, BlockId)>,
    ) {
        if visited.contains(&block) {
            return;
        }

        visited.insert(block);
        in_stack.insert(block);

        if let Some(successors) = cfg.successors.get(&block) {
            for succ in successors {
                if in_stack.contains(succ) {
                    // Found a back edge
                    back_edges.push((block, *succ));
                } else if !visited.contains(succ) {
                    dfs(*succ, cfg, visited, in_stack, back_edges);
                }
            }
        }

        in_stack.remove(&block);
    }

    let mut back_edges = Vec::new();
    let mut visited = BTreeSet::new();
    let mut in_stack = BTreeSet::new();

    dfs(cfg.entry, cfg, &mut visited, &mut in_stack, &mut back_edges);
    back_edges
}

/// Check if the program has any nondeterministic loops.
///
/// Programs with while(__VERIFIER_nondet_*()) loops may not terminate.
pub fn has_nondet_loops(module: &AirModule, cfgs: &BTreeMap<FunctionId, Cfg>) -> bool {
    !find_nonterminating_loops(module, cfgs).is_empty()
}

// ---------------------------------------------------------------------------
// Memory Leak Detection Helpers
// ---------------------------------------------------------------------------

/// Allocation function names (only allocators, not free).
const ALLOCATOR_FUNCTIONS: &[&str] = &[
    "malloc",
    "calloc",
    "realloc",
    "aligned_alloc",
    "posix_memalign",
    "memalign",
    "_Znwm", // operator new(size_t)
    "_Znam", // operator new[](size_t)
];

/// Deallocation function names.
const DEALLOCATOR_FUNCTIONS: &[&str] = &[
    "free", "_ZdlPv", // operator delete(void*)
    "_ZdaPv", // operator delete[](void*)
];

/// Exit function names that terminate the program.
const EXIT_FUNCTIONS: &[&str] = &["exit", "_exit", "abort", "__assert_fail", "quick_exit"];

/// Find all allocation sites in the program.
pub fn find_allocation_sites(
    module: &AirModule,
) -> Vec<(
    FunctionId,
    BlockId,
    saf_core::ids::InstId,
    saf_core::ids::ValueId,
)> {
    let mut sites = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // HeapAlloc in AIR
                if matches!(inst.op, Operation::HeapAlloc { .. }) {
                    if let Some(dst) = inst.dst {
                        sites.push((func.id, block.id, inst.id, dst));
                    }
                    continue;
                }

                // Direct calls to allocators
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if ALLOCATOR_FUNCTIONS.contains(&target.name.as_str()) {
                            if let Some(dst) = inst.dst {
                                sites.push((func.id, block.id, inst.id, dst));
                            }
                        }
                    }
                }
            }
        }
    }

    sites
}

/// Find all deallocation sites in the program.
pub fn find_deallocation_sites(
    module: &AirModule,
) -> Vec<(FunctionId, BlockId, saf_core::ids::InstId)> {
    let mut sites = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if DEALLOCATOR_FUNCTIONS.contains(&target.name.as_str()) {
                            sites.push((func.id, block.id, inst.id));
                        }
                    }
                }
            }
        }
    }

    sites
}

/// Check if a function call is to an exit function.
pub fn is_exit_call(module: &AirModule, callee: FunctionId) -> bool {
    if let Some(target) = module.function(callee) {
        EXIT_FUNCTIONS.contains(&target.name.as_str())
    } else {
        false
    }
}

/// Find all exit points in the program (return from main or exit calls).
pub fn find_exit_points(module: &AirModule) -> Vec<(FunctionId, BlockId)> {
    let mut exits = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            // Return from main
            if func.name == "main" {
                if let Some(term) = block.terminator() {
                    if matches!(term.op, Operation::Ret) {
                        exits.push((func.id, block.id));
                    }
                }
            }

            // Exit calls
            for inst in &block.instructions {
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if is_exit_call(module, *callee) {
                        exits.push((func.id, block.id));
                    }
                }
            }
        }
    }

    exits
}

// ---------------------------------------------------------------------------
// Reachable-Function Helpers
// ---------------------------------------------------------------------------

/// Compute the set of functions reachable from `main` via the call graph.
///
/// Performs DFS from the `main` function through the call graph and returns
/// the `FunctionId`s of all reachable functions (including `main` itself).
/// Returns an empty set if no `main` function is found in the module.
pub fn reachable_functions(callgraph: &CallGraph, module: &AirModule) -> BTreeSet<FunctionId> {
    // Find main() — must be a defined function (not a declaration)
    let main_func = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration);

    let Some(main_func) = main_func else {
        return BTreeSet::new();
    };

    let main_id = main_func.id;

    let Some(main_node) = callgraph.node_for_function(main_id) else {
        // main exists in module but not in callgraph — return just main
        let mut set = BTreeSet::new();
        set.insert(main_id);
        return set;
    };

    let reachable_nodes = dfs(main_node, callgraph);

    let mut result: BTreeSet<FunctionId> = reachable_nodes
        .iter()
        .filter_map(saf_analysis::callgraph::CallGraphNode::function_id)
        .collect();

    // Always include main
    result.insert(main_id);
    result
}

/// Check if any reachable function uses heap allocations.
///
/// Same logic as [`has_heap_allocations`] but only checks functions whose
/// `FunctionId` is in the `reachable` set.
pub fn reachable_has_heap_allocations(
    module: &AirModule,
    reachable: &BTreeSet<FunctionId>,
) -> bool {
    // Check for allocation function declarations that are reachable
    for func in &module.functions {
        if reachable.contains(&func.id) && ALLOC_FUNCTIONS.contains(&func.name.as_str()) {
            return true;
        }
    }

    // Check for HeapAlloc operations and direct calls to allocators in reachable functions
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::HeapAlloc { .. }) {
                    return true;
                }
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if ALLOC_FUNCTIONS.contains(&target.name.as_str()) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Thread-spawn primitive names — an actual thread *creation* (not mutex/atomic/
/// join/`fork`). Any real POSIX/C11 thread spawn bottoms out in one of these, so
/// call-graph reachability of one of these from `main` also captures spawns made
/// through wrapper functions.
const SPAWN_FUNCTIONS: &[&str] = &["pthread_create", "thrd_create"];

/// Does a thread spawn MAY-happen on a path reachable from `main`? (Plan 198.)
///
/// A sound over-approximation used to gate the `valid-memsafety` ASan confirmer:
/// returns `true` (⇒ the confirmer abstains) iff a [`SPAWN_FUNCTIONS`] primitive is
/// reachable from `main` via a direct call, **or** a reachable function contains an
/// indirect call while a spawn primitive is linked into the module (an unresolved
/// indirect target could be a spawn, and the module-level call graph may
/// under-approximate indirect edges — so this never *misses* a spawn). Returns
/// `false` only when the execution is provably sequential, in which case ASan's
/// single run is schedule-independent and a trap is a real violation on every
/// schedule.
///
/// Unlike [`has_threading_primitives`] (symbol presence — also flags mutex/atomic/
/// `fork`), this fires only on an actually-*reachable* spawn, so it does **not**
/// abstain on the sv-benchmarks Juliet reservoir whose `pthread_create` is dead
/// scaffolding (`stdThreadCreate` is never called; the sink runs directly in `main`).
#[must_use]
pub fn reachable_spawns_threads(module: &AirModule, callgraph: &CallGraph) -> bool {
    // No spawn primitive linked at all ⇒ no thread can be created ⇒ sequential.
    if !module
        .functions
        .iter()
        .any(|f| SPAWN_FUNCTIONS.contains(&f.name.as_str()))
    {
        return false;
    }

    let reachable = reachable_functions(callgraph, module);
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    // A reachable direct call to a spawn primitive.
                    Operation::CallDirect { callee } => {
                        if let Some(target) = module.function(*callee) {
                            if SPAWN_FUNCTIONS.contains(&target.name.as_str()) {
                                return true;
                            }
                        }
                    }
                    // A reachable indirect call could target the linked spawn
                    // primitive — abstain conservatively (never miss a spawn).
                    Operation::CallIndirect { .. } => return true,
                    _ => {}
                }
            }
        }
    }
    false
}

/// Check if all reachable functions are loop-free.
///
/// Same as [`program_is_loop_free`] but only checks CFGs whose `FunctionId`
/// is in the `reachable` set.
pub fn reachable_is_loop_free(
    cfgs: &BTreeMap<FunctionId, Cfg>,
    reachable: &BTreeSet<FunctionId>,
) -> bool {
    cfgs.iter()
        .filter(|(fid, _)| reachable.contains(fid))
        .all(|(_, cfg)| !cfg_has_loops(cfg))
}

/// Convenience wrapper: is the sub-program reachable from `main` loop-free?
///
/// Builds the call graph, the `main`-reachable function set, and the per-function
/// CFGs internally, then defers to [`reachable_is_loop_free`]. A `true` result means
/// no reachable function contains a CFG back-edge (recursion, which is a call-graph
/// cycle rather than a CFG loop, is NOT considered a loop here — each recursive
/// function's own CFG may still be acyclic).
///
/// The overflow confirmer uses this to decide whether it is safe to feed `INT_MAX`
/// / `INT_MIN` boundary values as nondet inputs: with no reachable loop there is no
/// counter/accumulator a boundary input could drive to a *spurious* `+1`-at-`INT_MAX`
/// overflow (the loop-driven false alarm the fixed `2^30` probe is capped to avoid),
/// so any `UBSan` signed-overflow trap under a boundary input is a genuine direct
/// overflow of the program.
#[must_use]
pub fn module_reachable_is_loop_free(module: &AirModule) -> bool {
    let callgraph = CallGraph::build(module);
    let reachable = reachable_functions(&callgraph, module);
    // An empty reachable set (no `main`) is vacuously loop-free, but the caller's
    // confirmer never runs without a `main`, so the value is immaterial there.
    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();
    reachable_is_loop_free(&cfgs, &reachable)
}

/// Check if a specific function's CFG is loop-free.
///
/// Returns true if the function has no loops (back-edges) in its CFG,
/// meaning abstract interpretation produces exact intervals without widening.
/// Returns true if the function has no CFG entry (e.g., declaration).
pub fn function_is_loop_free(cfgs: &BTreeMap<FunctionId, Cfg>, func_id: FunctionId) -> bool {
    cfgs.get(&func_id).is_none_or(|cfg| !cfg_has_loops(cfg))
}

/// Build a mapping from every `ValueId` (instruction destinations and function
/// parameters) to the `FunctionId` that contains it.
///
/// This is useful for scoping value-flow queries to only reachable functions.
pub fn build_value_function_map(module: &AirModule) -> BTreeMap<ValueId, FunctionId> {
    let mut map = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        // Map parameter ValueIds
        for param in &func.params {
            map.insert(param.id, func.id);
        }

        // Map instruction destination ValueIds
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    map.insert(dst, func.id);
                }
            }
        }
    }

    map
}

// ---------------------------------------------------------------------------
// Branch-steering constant harvesting (overflow / memsafety mini-fuzz reachability)
// ---------------------------------------------------------------------------

/// Magnitude cap for a harvested branch-steering constant: strictly below `2^30`.
///
/// A program literal at or above this (e.g. a near-`INT_MAX` loop bound) fed as a
/// nondet input could drive a loop counter/accumulator to `INT_MAX` and then spuriously
/// trap on the next `+1` — the `termination-*` loop-counter false alarm that the fixed
/// `2^30` probe already avoids. Keeping harvested literals strictly below `2^30`
/// preserves the identical soundness margin (the existing large-positive probe is `2^30`
/// exactly, so nothing steered in exceeds it).
const BRANCH_STEERING_MAGNITUDE_CAP: i64 = 1 << 30;

/// Cap on how many distinct program-literal branch-steering constants to harvest,
/// bounding the extra native replay runs. Deterministic `BTreeSet` iteration makes the
/// first-N selection reproducible (`NFR-DET`).
const MAX_BRANCH_STEERING_CONSTS: usize = 16;

/// True iff `kind` is an integer comparison (`ICmp*`) — the guards whose constant
/// operand is a branch threshold worth steering the mini-fuzz toward. Float compares
/// and arithmetic are excluded (their literals are not integer input thresholds).
fn is_integer_compare(kind: saf_core::air::BinaryOp) -> bool {
    use saf_core::air::BinaryOp::{
        ICmpEq, ICmpNe, ICmpSge, ICmpSgt, ICmpSle, ICmpSlt, ICmpUge, ICmpUgt, ICmpUle, ICmpUlt,
    };
    matches!(
        kind,
        ICmpEq
            | ICmpNe
            | ICmpUgt
            | ICmpUge
            | ICmpUlt
            | ICmpUle
            | ICmpSgt
            | ICmpSge
            | ICmpSlt
            | ICmpSle
    )
}

/// Insert `v` into `set` iff its magnitude is under [`BRANCH_STEERING_MAGNITUDE_CAP`].
fn push_capped(set: &mut BTreeSet<i64>, v: i64) {
    // `unsigned_abs` cannot overflow (unlike `abs` at `i64::MIN`); the cap is well
    // within `u64` range, so the comparison is exact.
    if v.unsigned_abs() < BRANCH_STEERING_MAGNITUDE_CAP as u64 {
        set.insert(v);
    }
}

/// Harvest integer comparison / switch literals from the program to steer the
/// mini-fuzz toward guard-gated violations (cpa-witness2test-style input steering).
///
/// A guard such as `if (x == 500) INT_MAX + x;` is only reached when the nondet input
/// equals the program's own literal `500` — a value the fixed constant spread will
/// never guess. This pass collects the integer literal operand of every `ICmp*`
/// comparison and every `Switch` case value, so those exact guard thresholds enter the
/// sweep as candidate inputs. It widens the *reachability* that feeds the overflow
/// confirmer without touching how a violation is confirmed.
///
/// Soundness: each harvested value is a concrete integer the verifier may legally
/// choose for the nondet input; `__VERIFIER_assume` still prunes any that are
/// infeasible, and the confirmer only emits `false` on a re-triggered concrete trap on
/// the ORIGINAL program. Literals of magnitude `≥ 2^30` are dropped (see
/// [`BRANCH_STEERING_MAGNITUDE_CAP`]) so no steered value can widen the
/// loop-counter-to-`INT_MAX` false-alarm surface. The result is deterministic
/// (`BTreeSet` order) and capped at [`MAX_BRANCH_STEERING_CONSTS`].
#[must_use]
pub fn branch_steering_constants(module: &AirModule) -> Vec<i64> {
    use saf_core::air::Constant;

    let mut consts: BTreeSet<i64> = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::BinaryOp { kind } if is_integer_compare(*kind) => {
                        for v in &inst.operands {
                            if let Some(Constant::Int { value, .. }) = module.constants.get(v) {
                                push_capped(&mut consts, *value);
                            }
                        }
                    }
                    Operation::Switch { cases, .. } => {
                        for (case_val, _) in cases {
                            push_capped(&mut consts, *case_val);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    consts
        .into_iter()
        .take(MAX_BRANCH_STEERING_CONSTS)
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_function_list() {
        assert!(THREAD_FUNCTIONS.contains(&"pthread_create"));
        assert!(THREAD_FUNCTIONS.contains(&"thrd_create"));
        assert!(THREAD_FUNCTIONS.contains(&"fork"));
    }

    #[test]
    fn test_alloc_function_list() {
        assert!(ALLOC_FUNCTIONS.contains(&"malloc"));
        assert!(ALLOC_FUNCTIONS.contains(&"free"));
        assert!(ALLOC_FUNCTIONS.contains(&"_Znwm"));
    }

    #[test]
    fn test_nondet_function_list() {
        assert!(NONDET_FUNCTIONS.contains(&"__VERIFIER_nondet_int"));
        assert!(NONDET_FUNCTIONS.contains(&"__VERIFIER_nondet_bool"));
    }

    #[test]
    fn test_exit_function_list() {
        assert!(EXIT_FUNCTIONS.contains(&"exit"));
        assert!(EXIT_FUNCTIONS.contains(&"abort"));
    }

    // -----------------------------------------------------------------------
    // Helpers for building test fixtures
    // -----------------------------------------------------------------------

    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction};
    use saf_core::id::make_id;

    fn make_func_id(name: &str) -> FunctionId {
        FunctionId(make_id("func", name.as_bytes()))
    }

    fn make_value_id(name: &str) -> ValueId {
        ValueId(make_id("value", name.as_bytes()))
    }

    fn make_block_id(name: &str) -> BlockId {
        BlockId(make_id("block", name.as_bytes()))
    }

    fn make_inst_id(name: &str) -> saf_core::ids::InstId {
        saf_core::ids::InstId(make_id("inst", name.as_bytes()))
    }

    fn make_module_id(name: &str) -> saf_core::ids::ModuleId {
        saf_core::ids::ModuleId(make_id("module", name.as_bytes()))
    }

    /// Build a minimal defined function with a single empty block.
    fn make_defined_function(name: &str) -> AirFunction {
        let fid = make_func_id(name);
        let bid = make_block_id(&format!("{name}_entry"));
        AirFunction {
            id: fid,
            name: name.to_string(),
            params: Vec::new(),
            blocks: vec![AirBlock::new(bid)],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// Build a minimal declaration function (no body).
    fn make_declaration(name: &str) -> AirFunction {
        let fid = make_func_id(name);
        AirFunction {
            id: fid,
            name: name.to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// Build a minimal `AirModule` from a list of functions.
    fn make_module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: make_module_id("test"),
            name: Some("test".to_string()),
            functions,
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: BTreeMap::new(),
            types: BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    /// Build a defined function that calls another function (via `CallDirect`).
    fn make_calling_function(name: &str, callees: &[&AirFunction]) -> AirFunction {
        let fid = make_func_id(name);
        let bid = make_block_id(&format!("{name}_entry"));
        let mut block = AirBlock::new(bid);

        for (i, callee) in callees.iter().enumerate() {
            let inst = Instruction {
                id: make_inst_id(&format!("{name}_call_{i}")),
                op: Operation::CallDirect { callee: callee.id },
                operands: Vec::new(),
                dst: None,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            };
            block.instructions.push(inst);
        }

        AirFunction {
            id: fid,
            name: name.to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// Build a defined function whose body contains a single indirect call.
    fn make_function_with_indirect_call(name: &str) -> AirFunction {
        let fid = make_func_id(name);
        let bid = make_block_id(&format!("{name}_entry"));
        let mut block = AirBlock::new(bid);
        block.instructions.push(Instruction {
            id: make_inst_id(&format!("{name}_icall")),
            op: Operation::CallIndirect {
                expected_signature: None,
            },
            operands: Vec::new(),
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        AirFunction {
            id: fid,
            name: name.to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Tests for reachable_functions
    // -----------------------------------------------------------------------

    #[test]
    fn test_reachable_functions_no_main() {
        let module = make_module(vec![make_defined_function("foo")]);
        let cg = CallGraph::build(&module);
        let result = reachable_functions(&cg, &module);
        assert!(result.is_empty());
    }

    #[test]
    fn test_reachable_functions_main_only() {
        let main_fn = make_defined_function("main");
        let main_id = main_fn.id;
        let module = make_module(vec![main_fn]);
        let cg = CallGraph::build(&module);

        let result = reachable_functions(&cg, &module);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&main_id));
    }

    #[test]
    fn test_reachable_functions_with_callees() {
        let bar_fn = make_defined_function("bar");
        let foo_fn = make_calling_function("foo", &[&bar_fn]);
        let main_fn = make_calling_function("main", &[&foo_fn]);
        let unreachable_fn = make_defined_function("unreachable");

        let main_id = main_fn.id;
        let foo_id = foo_fn.id;
        let bar_id = bar_fn.id;
        let unreachable_id = unreachable_fn.id;

        // main -> foo -> bar, unreachable is disconnected
        let module = make_module(vec![main_fn, foo_fn, bar_fn, unreachable_fn]);
        let cg = CallGraph::build(&module);

        let result = reachable_functions(&cg, &module);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&main_id));
        assert!(result.contains(&foo_id));
        assert!(result.contains(&bar_id));
        assert!(!result.contains(&unreachable_id));
    }

    #[test]
    fn test_reachable_functions_declaration_main_ignored() {
        // A declaration named "main" should not count
        let module = make_module(vec![make_declaration("main")]);
        let cg = CallGraph::build(&module);

        let result = reachable_functions(&cg, &module);
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // Tests for reachable_has_heap_allocations
    // -----------------------------------------------------------------------

    #[test]
    fn test_reachable_has_heap_allocations_empty() {
        let module = make_module(vec![make_defined_function("main")]);
        let reachable: BTreeSet<FunctionId> = [make_func_id("main")].into_iter().collect();
        assert!(!reachable_has_heap_allocations(&module, &reachable));
    }

    #[test]
    fn test_reachable_has_heap_allocations_malloc_declaration_reachable() {
        let main_fn = make_defined_function("main");
        let malloc_fn = make_declaration("malloc");
        let reachable: BTreeSet<FunctionId> = [main_fn.id, malloc_fn.id].into_iter().collect();
        let module = make_module(vec![main_fn, malloc_fn]);
        assert!(reachable_has_heap_allocations(&module, &reachable));
    }

    #[test]
    fn test_reachable_has_heap_allocations_malloc_not_reachable() {
        let main_fn = make_defined_function("main");
        let malloc_fn = make_declaration("malloc");
        // Only main is reachable, malloc is not
        let reachable: BTreeSet<FunctionId> = [main_fn.id].into_iter().collect();
        let module = make_module(vec![main_fn, malloc_fn]);
        assert!(!reachable_has_heap_allocations(&module, &reachable));
    }

    #[test]
    fn test_reachable_has_heap_allocations_heap_alloc_op() {
        let mut main_fn = make_defined_function("main");
        let inst = Instruction {
            id: make_inst_id("heap_alloc"),
            op: Operation::HeapAlloc {
                kind: saf_core::air::HeapAllocKind::Malloc,
            },
            operands: Vec::new(),
            dst: Some(make_value_id("ptr")),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
        main_fn.blocks[0].instructions.push(inst);

        let reachable: BTreeSet<FunctionId> = [main_fn.id].into_iter().collect();
        let module = make_module(vec![main_fn]);
        assert!(reachable_has_heap_allocations(&module, &reachable));
    }

    // -----------------------------------------------------------------------
    // Tests for reachable_is_loop_free
    // -----------------------------------------------------------------------

    #[test]
    fn test_reachable_is_loop_free_empty() {
        let cfgs = BTreeMap::new();
        let reachable = BTreeSet::new();
        assert!(reachable_is_loop_free(&cfgs, &reachable));
    }

    #[test]
    fn test_reachable_is_loop_free_linear_cfg() {
        let fid = make_func_id("main");
        let b0 = make_block_id("b0");
        let b1 = make_block_id("b1");

        let mut successors = BTreeMap::new();
        successors.insert(b0, [b1].into_iter().collect());

        let cfg = Cfg {
            function: fid,
            entry: b0,
            exits: [b1].into_iter().collect(),
            successors,
            predecessors: BTreeMap::new(),
        };

        let cfgs: BTreeMap<FunctionId, Cfg> = [(fid, cfg)].into_iter().collect();
        let reachable: BTreeSet<FunctionId> = [fid].into_iter().collect();
        assert!(reachable_is_loop_free(&cfgs, &reachable));
    }

    #[test]
    fn test_reachable_is_loop_free_with_loop_reachable() {
        let fid = make_func_id("loopy");
        let b0 = make_block_id("b0");
        let b1 = make_block_id("b1");

        let mut successors = BTreeMap::new();
        successors.insert(b0, [b1].into_iter().collect());
        successors.insert(b1, [b0].into_iter().collect()); // back edge

        let cfg = Cfg {
            function: fid,
            entry: b0,
            exits: BTreeSet::new(), // no exits — it's a loop
            successors,
            predecessors: BTreeMap::new(),
        };

        let cfgs: BTreeMap<FunctionId, Cfg> = [(fid, cfg)].into_iter().collect();
        let reachable: BTreeSet<FunctionId> = [fid].into_iter().collect();
        assert!(!reachable_is_loop_free(&cfgs, &reachable));
    }

    #[test]
    fn test_reachable_is_loop_free_loop_unreachable() {
        let main_id = make_func_id("main");
        let loopy_id = make_func_id("loopy");

        let b0_main = make_block_id("main_b0");
        let b0_loop = make_block_id("loop_b0");
        let b1_loop = make_block_id("loop_b1");

        // main has a linear CFG
        let main_cfg = Cfg {
            function: main_id,
            entry: b0_main,
            exits: [b0_main].into_iter().collect(),
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
        };

        // loopy has a back edge
        let mut loop_succs = BTreeMap::new();
        loop_succs.insert(b0_loop, [b1_loop].into_iter().collect());
        loop_succs.insert(b1_loop, [b0_loop].into_iter().collect());
        let loopy_cfg = Cfg {
            function: loopy_id,
            entry: b0_loop,
            exits: BTreeSet::new(),
            successors: loop_succs,
            predecessors: BTreeMap::new(),
        };

        let cfgs: BTreeMap<FunctionId, Cfg> = [(main_id, main_cfg), (loopy_id, loopy_cfg)]
            .into_iter()
            .collect();

        // Only main is reachable, so the loopy function is excluded
        let reachable: BTreeSet<FunctionId> = [main_id].into_iter().collect();
        assert!(reachable_is_loop_free(&cfgs, &reachable));
    }

    // -----------------------------------------------------------------------
    // Tests for module_reachable_is_loop_free (module-level convenience)
    // -----------------------------------------------------------------------

    #[test]
    fn test_module_reachable_is_loop_free_linear_main() {
        // A single-block `main` (no back-edge) is loop-free.
        let module = make_module(vec![make_defined_function("main")]);
        assert!(module_reachable_is_loop_free(&module));
    }

    #[test]
    fn test_module_reachable_is_loop_free_detects_back_edge() {
        // `main` with `b0 -> b1 -> b0` has a reachable CFG loop.
        let fid = make_func_id("main");
        let b0 = make_block_id("main_b0");
        let b1 = make_block_id("main_b1");
        let mut block0 = AirBlock::new(b0);
        block0.instructions.push(Instruction {
            id: make_inst_id("b0_term"),
            op: Operation::Br { target: b1 },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        let mut block1 = AirBlock::new(b1);
        block1.instructions.push(Instruction {
            id: make_inst_id("b1_term"),
            op: Operation::Br { target: b0 },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        let main = AirFunction {
            id: fid,
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block0, block1],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let module = make_module(vec![main]);
        assert!(!module_reachable_is_loop_free(&module));
    }

    // -----------------------------------------------------------------------
    // Tests for build_value_function_map
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_value_function_map_empty_module() {
        let module = make_module(vec![]);
        let map = build_value_function_map(&module);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_value_function_map_skips_declarations() {
        let module = make_module(vec![make_declaration("malloc")]);
        let map = build_value_function_map(&module);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_value_function_map_params_and_instructions() {
        let mut func = make_defined_function("main");
        let param_vid = make_value_id("param0");
        let inst_vid = make_value_id("inst0");

        func.params.push(AirParam {
            id: param_vid,
            name: Some("argc".to_string()),
            index: 0,
            param_type: None,
        });

        let inst = Instruction {
            id: make_inst_id("nop"),
            op: Operation::Load,
            operands: Vec::new(),
            dst: Some(inst_vid),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
        func.blocks[0].instructions.push(inst);

        let fid = func.id;
        let module = make_module(vec![func]);
        let map = build_value_function_map(&module);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&param_vid), Some(&fid));
        assert_eq!(map.get(&inst_vid), Some(&fid));
    }

    #[test]
    fn test_build_value_function_map_no_dst_instruction() {
        let mut func = make_defined_function("main");
        // Instruction with no dst should be skipped
        let inst = Instruction {
            id: make_inst_id("void_call"),
            op: Operation::Load,
            operands: Vec::new(),
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };
        func.blocks[0].instructions.push(inst);

        let module = make_module(vec![func]);
        let map = build_value_function_map(&module);
        assert!(map.is_empty());
    }

    // -----------------------------------------------------------------------
    // Tests for function_is_loop_free
    // -----------------------------------------------------------------------

    #[test]
    fn test_function_is_loop_free() {
        // Test with loop-free function
        let fid = make_func_id("linear");
        let b0 = make_block_id("b0");
        let b1 = make_block_id("b1");
        let mut successors = BTreeMap::new();
        successors.insert(b0, [b1].into_iter().collect());
        let cfg = Cfg {
            function: fid,
            entry: b0,
            exits: [b1].into_iter().collect(),
            successors,
            predecessors: BTreeMap::new(),
        };
        let cfgs: BTreeMap<FunctionId, Cfg> = [(fid, cfg)].into_iter().collect();
        assert!(function_is_loop_free(&cfgs, fid));

        // Test with loopy function
        let loopy_id = make_func_id("loopy");
        let lb0 = make_block_id("lb0");
        let lb1 = make_block_id("lb1");
        let mut loop_succs = BTreeMap::new();
        loop_succs.insert(lb0, [lb1].into_iter().collect());
        loop_succs.insert(lb1, [lb0].into_iter().collect());
        let loopy_cfg = Cfg {
            function: loopy_id,
            entry: lb0,
            exits: BTreeSet::new(),
            successors: loop_succs,
            predecessors: BTreeMap::new(),
        };
        let cfgs2: BTreeMap<FunctionId, Cfg> = [(loopy_id, loopy_cfg)].into_iter().collect();
        assert!(!function_is_loop_free(&cfgs2, loopy_id));

        // Test with unknown function (not in cfgs)
        let unknown_id = make_func_id("unknown");
        assert!(function_is_loop_free(&cfgs2, unknown_id));
    }

    // -----------------------------------------------------------------------
    // Tests for reachable_spawns_threads (plan 198)
    // -----------------------------------------------------------------------

    #[test]
    fn spawns_threads_false_on_dead_pthread_scaffolding() {
        // pthread_create is present and called, but ONLY by stdThreadCreate, which
        // nothing reachable from main calls (the sv-benchmarks Juliet pattern:
        // main runs the sink directly; the thread wrapper is dead scaffolding).
        let pthread_create = make_declaration("pthread_create");
        let std_thread = make_calling_function("stdThreadCreate", &[&pthread_create]);
        let sink = make_defined_function("sink");
        let main = make_calling_function("main", &[&sink]);
        let module = make_module(vec![main, sink, std_thread, pthread_create]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_spawns_threads(&module, &cg));
    }

    #[test]
    fn spawns_threads_true_on_direct_reachable_pthread_create() {
        let pthread_create = make_declaration("pthread_create");
        let main = make_calling_function("main", &[&pthread_create]);
        let module = make_module(vec![main, pthread_create]);
        let cg = CallGraph::build(&module);
        assert!(reachable_spawns_threads(&module, &cg));
    }

    #[test]
    fn spawns_threads_true_via_reachable_helper_thrd_create() {
        let thrd_create = make_declaration("thrd_create");
        let spawn_it = make_calling_function("spawn_it", &[&thrd_create]);
        let main = make_calling_function("main", &[&spawn_it]);
        let module = make_module(vec![main, spawn_it, thrd_create]);
        let cg = CallGraph::build(&module);
        assert!(reachable_spawns_threads(&module, &cg));
    }

    #[test]
    fn spawns_threads_true_on_reachable_indirect_call_with_spawn_linked() {
        // A reachable indirect call could target the linked pthread_create and the
        // callgraph may miss the indirect edge -> abstain conservatively.
        let pthread_create = make_declaration("pthread_create");
        let main = make_function_with_indirect_call("main");
        let module = make_module(vec![main, pthread_create]);
        let cg = CallGraph::build(&module);
        assert!(reachable_spawns_threads(&module, &cg));
    }

    #[test]
    fn spawns_threads_false_on_indirect_call_without_spawn_symbol() {
        // No spawn primitive linked at all -> an indirect call cannot create a thread.
        let main = make_function_with_indirect_call("main");
        let module = make_module(vec![main]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_spawns_threads(&module, &cg));
    }

    #[test]
    fn spawns_threads_false_without_spawn_symbol() {
        let sink = make_defined_function("sink");
        let main = make_calling_function("main", &[&sink]);
        let module = make_module(vec![main, sink]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_spawns_threads(&module, &cg));
    }

    #[test]
    fn spawns_threads_false_without_main() {
        // No main -> empty reachable set; the program cannot run (link-fail ->
        // unknown), so the gate returns false and the confirmer resolves to unknown.
        let pthread_create = make_declaration("pthread_create");
        let foo = make_calling_function("foo", &[&pthread_create]);
        let module = make_module(vec![foo, pthread_create]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_spawns_threads(&module, &cg));
    }

    // --- branch_steering_constants ---------------------------------------

    /// Build a one-block function whose single instruction is `op` with `operands`.
    fn make_func_with_inst(name: &str, op: Operation, operands: Vec<ValueId>) -> AirFunction {
        let bid = make_block_id(&format!("{name}_entry"));
        let mut block = AirBlock::new(bid);
        block.instructions.push(Instruction {
            id: make_inst_id(&format!("{name}_i0")),
            op,
            operands,
            dst: Some(make_value_id(&format!("{name}_r0"))),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        AirFunction {
            id: make_func_id(name),
            name: name.to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    #[test]
    fn branch_steering_harvests_icmp_literal() {
        use saf_core::air::{BinaryOp, Constant};
        let c = make_value_id("c500");
        let x = make_value_id("x");
        let main = make_func_with_inst(
            "main",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![x, c],
        );
        let mut module = make_module(vec![main]);
        module.constants.insert(c, Constant::int(500, 32));
        assert_eq!(branch_steering_constants(&module), vec![500]);
    }

    #[test]
    fn branch_steering_harvests_switch_cases() {
        let d = make_value_id("disc");
        let target = make_block_id("case_target");
        let main = make_func_with_inst(
            "main",
            Operation::Switch {
                default: make_block_id("default"),
                cases: vec![(7, target), (42, target), (7, target)],
            },
            vec![d],
        );
        let module = make_module(vec![main]);
        // Deduplicated + sorted (BTreeSet), determinism holds.
        assert_eq!(branch_steering_constants(&module), vec![7, 42]);
    }

    #[test]
    fn branch_steering_drops_out_of_range_and_keeps_in_range_negative() {
        use saf_core::air::{BinaryOp, Constant};
        let big = make_value_id("big");
        let neg = make_value_id("neg");
        let x = make_value_id("x");
        let cmp_big = make_func_with_inst(
            "f_big",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![x, big],
        );
        let cmp_neg = make_func_with_inst(
            "f_neg",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            vec![x, neg],
        );
        let mut module = make_module(vec![cmp_big, cmp_neg]);
        // 2^30 is at the cap (excluded, `< cap` is strict); i64::MIN excluded; -5 kept.
        module.constants.insert(big, Constant::int(1 << 30, 32));
        module.constants.insert(neg, Constant::int(-5, 32));
        assert_eq!(branch_steering_constants(&module), vec![-5]);
    }

    #[test]
    fn branch_steering_ignores_float_compare_and_declarations() {
        use saf_core::air::{BinaryOp, Constant};
        let c = make_value_id("cf");
        let x = make_value_id("xf");
        // A float compare's literal is not an integer input threshold -> ignored.
        let ffn = make_func_with_inst(
            "ffn",
            Operation::BinaryOp {
                kind: BinaryOp::FCmpOeq,
            },
            vec![x, c],
        );
        let decl = make_declaration("some_decl");
        let mut module = make_module(vec![ffn, decl]);
        module.constants.insert(c, Constant::int(9, 32));
        assert!(branch_steering_constants(&module).is_empty());
    }

    #[test]
    fn branch_steering_handles_i64_min_without_panic() {
        // `abs(i64::MIN)` would overflow; `unsigned_abs` must be used. The value is
        // out of range so it is dropped, and the call must not panic.
        let d = make_value_id("disc");
        let target = make_block_id("t");
        let main = make_func_with_inst(
            "main",
            Operation::Switch {
                default: make_block_id("default"),
                cases: vec![(i64::MIN, target), (3, target)],
            },
            vec![d],
        );
        let module = make_module(vec![main]);
        assert_eq!(branch_steering_constants(&module), vec![3]);
    }
}
