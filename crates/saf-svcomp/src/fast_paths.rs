//! Fast-path checks for SV-COMP property analysis.
//!
//! These functions provide quick checks that can prove properties hold
//! without running expensive full analyses.

use std::collections::{BTreeMap, BTreeSet};

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::graph_algo::dfs;
use saf_core::air::{AirFunction, AirModule, BinaryOp, Constant, Instruction, Operation};
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

/// Does a reachable function perform a **dynamically-sized stack allocation**
/// (a variable-length array or `alloca()` whose size is not a compile-time
/// constant, i.e. `Operation::Alloca { size_bytes: None }`)?
///
/// This gates the `valid-memsafety` byte-stream ASan mini-fuzz (pass 3) OUT of such
/// programs. Under SV-COMP's semantics the abstract stack is UNBOUNDED, so an
/// `alloca(n)` / `T buf[n]` with an arbitrarily large `n` is a *safe* operation. But
/// a coverage-guided fuzzer that drives the nondet size to a large value exhausts the
/// concrete 8 MB native stack, which AddressSanitizer reports as a stack-exhaustion
/// fault (`stack-overflow`, or a stack-region `SEGV` / `dynamic-stack-buffer-overflow`
/// depending on where the guard page lands). That is an artifact of the bounded native
/// stack, NOT a violation of the program under test — so emitting `false` on it would be
/// a false alarm on a correct-TRUE task (`array-memsafety/{openbsd_cmemchr,subseq}-alloca-*`
/// are exactly this shape). Fixed-size allocas (`int x[10]` ⇒ `size_bytes: Some(_)`)
/// carry no such risk and are NOT flagged.
///
/// Fail-closed: a genuine violation in such a program is still eligible via the cheaper
/// passes (uniform / uninitialized / threshold sweeps), which drive the same nondet
/// sizes but never to a stack-exhausting magnitude; only the aggressive byte-stream
/// search — the one that can wander into the exhaustion regime — is suppressed here.
#[must_use]
pub fn reachable_has_dynamic_alloca(module: &AirModule, callgraph: &CallGraph) -> bool {
    let reachable = reachable_functions(callgraph, module);
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::Alloca { size_bytes: None }) {
                    return true;
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

/// `reach_error` / `__VERIFIER_error` / assertion-failure call names whose call
/// site is the `unreach-call` violation point. Used only to anchor a concurrency
/// violation witness's target transition to a source line (`startline`), never to
/// decide a verdict.
const ERROR_CALL_NAMES: &[&str] = &[
    "reach_error",
    "__VERIFIER_error",
    "__assert_fail",
    "__assert_rtn",
];

/// Source line (1-based) of the first `reach_error` / `__VERIFIER_error` /
/// assertion-failure call in the module, in deterministic (module → block →
/// instruction) order, if a span is recorded.
///
/// Used purely to give a concurrency GraphML violation witness a `startline`
/// anchor on its target transition so a validator's CFA walk can locate the
/// violation. Returns `None` when no such call carries a span — the witness then
/// omits the anchor (still structurally valid, just less precise). Scanning all
/// defined functions (not just `main`-reachable) is fine: the anchor is advisory.
#[must_use]
pub fn error_call_startline(module: &AirModule) -> Option<u32> {
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if ERROR_CALL_NAMES.contains(&target.name.as_str()) {
                            if let Some(span) = &inst.span {
                                return Some(span.line_start);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Source lines (1-based) of reachable-from-`main` direct thread-spawn call sites
/// ([`SPAWN_FUNCTIONS`]), in deterministic encounter order, deduplicated, capped.
///
/// Used to anchor a concurrency violation witness's `createThread` transitions to
/// the `pthread_create` source lines so a validator's CFA walk can locate them.
/// Only call sites that carry a span contribute; the list may therefore be shorter
/// than [`reachable_spawn_call_sites`] (which counts every site). The cap keeps a
/// spawn-in-a-loop task's witness small.
#[must_use]
pub fn reachable_spawn_startlines(module: &AirModule, callgraph: &CallGraph) -> Vec<u32> {
    const CAP: usize = 16;
    let reachable = reachable_functions(callgraph, module);
    let mut lines: Vec<u32> = Vec::new();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if SPAWN_FUNCTIONS.contains(&target.name.as_str()) {
                            if let Some(span) = &inst.span {
                                if !lines.contains(&span.line_start) {
                                    lines.push(span.line_start);
                                    if lines.len() >= CAP {
                                        return lines;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    lines
}

/// Resolve the start-routine function NAME of the first reachable-from-`main`
/// `pthread_create` / `thrd_create` call, if it is a **directly-named** defined
/// function.
///
/// Used only to anchor a concurrency violation witness's `enterFunction` transition
/// (the created thread's initial function). Delegates to
/// [`crate::race_true::resolve_direct_thread_fn`], which mirrors the frontend's
/// function-address encodings (a `Constant::GlobalRef` to a function, or an operand
/// whose raw id *is* a function's `ObjId`) and returns `None` on any indirect /
/// unresolved entry. On `None` the witness simply omits the anchor — a wrong
/// `enterFunction` would make the automaton unmatchable, so fail closed.
#[must_use]
pub fn spawn_start_routine_name(module: &AirModule, callgraph: &CallGraph) -> Option<String> {
    let reachable = reachable_functions(callgraph, module);
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                let Operation::CallDirect { callee } = &inst.op else {
                    continue;
                };
                let Some(target) = module.function(*callee) else {
                    continue;
                };
                if !SPAWN_FUNCTIONS.contains(&target.name.as_str()) {
                    continue;
                }
                if let Some(fid) = crate::race_true::resolve_direct_thread_fn(module, inst) {
                    if let Some(routine) = module.function(fid) {
                        if !routine.is_declaration {
                            return Some(routine.name.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Count reachable-from-`main` direct call sites to a thread-spawn primitive
/// ([`SPAWN_FUNCTIONS`]), saturating at `cap`.
///
/// A static over-count of the threads a run may create (a `pthread_create` inside a
/// loop is one call site but many runtime threads); used only to size a violation
/// witness's `createThread` edges, so a saturating count is fine. Returns at least 1
/// when a spawn is reachable.
#[must_use]
pub fn reachable_spawn_call_sites(module: &AirModule, callgraph: &CallGraph) -> usize {
    const CAP: usize = 64;
    let reachable = reachable_functions(callgraph, module);
    let mut count = 0usize;
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if SPAWN_FUNCTIONS.contains(&target.name.as_str()) {
                            count += 1;
                            if count >= CAP {
                                return CAP;
                            }
                        }
                    }
                }
            }
        }
    }
    count.max(1)
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

/// Does **any** defined function in the module contain a CFG loop (back-edge)?
///
/// A whole-module over-approximation of loop presence: unlike
/// [`module_reachable_is_loop_free`], it ignores call-graph reachability, so it
/// returns `true` whenever *any* function has a loop even if that function is only
/// reachable through an indirect call the call graph cannot resolve. This makes
/// `!module_has_any_loop` a **sound** "no loop can possibly execute" test — used by
/// the portfolio router to prune loop-only levers (SE / CBMC) without risk of
/// dropping a lever that could still fire via an unresolved indirect edge.
#[must_use]
pub fn module_has_any_loop(module: &AirModule) -> bool {
    module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .any(|f| cfg_has_loops(&Cfg::build(f)))
}

/// Is every reachable-from-`main` loop provably **ranked** (or is the sub-program
/// loop-free)?
///
/// A strict widening of [`module_reachable_is_loop_free`]: every reachable defined
/// function must be loop-free **OR** have all of its natural loops proven
/// terminating by linear ranking-function synthesis
/// ([`crate::ranking::loops_are_ranked`]). A loop-free program passes trivially, so
/// this returns `true` on a superset of the programs `module_reachable_is_loop_free`
/// accepts.
///
/// # Why this is the sound gate for `INT_MAX`-boundary injection
///
/// The overflow confirmer feeds type-boundary values (`INT_MAX`, `INT_MIN`, …) as
/// nondet inputs. The one false-alarm hazard is an injected value driving a **loop
/// counter/accumulator** to `INT_MAX` and then spuriously trapping on the next `+1`
/// — a `+1`-at-`INT_MAX` overflow that a TRUE `termination-*` task would never reach
/// under its real (constrained) inputs. `module_reachable_is_loop_free` sidesteps
/// this by requiring *no* reachable loop at all, which needlessly excludes every
/// **counted** loop (`for (i = 0; i < n; i++) …`).
///
/// `loops_are_ranked` closes the gap soundly: a ranked loop's induction variable is
/// admitted into the ranking model **only** when its per-iteration update provably
/// stays inside the type range on the loop's (over-approximate) region
/// ([`crate::ranking::next_never_overflows`]). A loop whose counter *could* reach
/// `INT_MAX` and overflow on the next step (e.g. a `<=`-guarded counter) has no
/// overflow-safe induction variable, so synthesis fails and the loop is **rejected**
/// — the gate then stays `false` and no boundary value is injected. Conversely, when
/// every reachable loop is ranked there is no counter an injected boundary value can
/// drive to a *spurious* overflow, so any `UBSan` signed-overflow trap is a genuine
/// direct violation (`UBSan` remains the sole R2 arbiter, re-triggered on the
/// original program per R6).
///
/// Recursion is treated exactly as in [`module_reachable_is_loop_free`]: a
/// call-graph cycle is not a CFG loop, so a recursive-but-CFG-loop-free function
/// passes (a nondet-driven recursion depth manifests as a stack overflow crash, not
/// a spurious integer-counter trap; a recursive accumulator overflow is genuine).
#[must_use]
pub fn module_reachable_loops_all_ranked(module: &AirModule) -> bool {
    let callgraph = CallGraph::build(module);
    let reachable = reachable_functions(&callgraph, module);
    module
        .functions
        .iter()
        .filter(|f| !f.is_declaration && reachable.contains(&f.id))
        .all(|f| {
            let cfg = Cfg::build(f);
            !cfg_has_loops(&cfg) || crate::ranking::loops_are_ranked(f, module, &cfg)
        })
}

// ---------------------------------------------------------------------------
// Spurious linear-accumulator overflow suppression (no-overflow soundness).
// ---------------------------------------------------------------------------

/// Max magnitude of a linear induction variable's per-iteration constant step for
/// its overflow to be treated as a SPURIOUS accumulator overflow.
///
/// A signed overflow of `iv ± c` from a bounded initial value needs on the order of
/// `INT_MAX / |c|` iterations to occur. With `|c| ≤ 2^20` that is `≥ ~2^11`
/// iterations, i.e. the overflow is only reachable by driving a nondeterministic
/// loop bound to an astronomically large trip count — the labeled-TRUE `termination-*`
/// idiom, not a genuine direct overflow. A step ABOVE this overflows in few
/// iterations (`sum += big`) and is never suppressed.
const SPURIOUS_IV_STEP_MAX: i128 = 1 << 20;

/// Is `v` a recognized integer constant whose magnitude is at most
/// [`SPURIOUS_IV_STEP_MAX`]? (A `BigInt` / non-int constant is never "small".)
fn is_small_int_const(module: &AirModule, v: ValueId) -> bool {
    match module.constants.get(&v) {
        Some(Constant::Int { value, .. }) => {
            i128::from(*value).unsigned_abs() <= SPURIOUS_IV_STEP_MAX.unsigned_abs()
        }
        _ => false,
    }
}

/// Transitive nondeterministic-taint per defined function: the set of SSA values that
/// can carry a `__VERIFIER_nondet_*` result. Propagated intra-procedurally through
/// data operands and phi incomings, and inter-procedurally through call arguments →
/// callee parameters (so a loop bounded by a parameter fed a nondet argument — the
/// `twisted` `f(nondet(), nondet())` idiom — is recognized as nondet-controlled).
///
/// Over-approximate by design: extra taint only ever makes the spurious-accumulator
/// check ABSTAIN from confirming, which costs recall, never soundness.
fn module_nondet_taint(module: &AirModule) -> BTreeMap<FunctionId, BTreeSet<ValueId>> {
    let mut taint: BTreeMap<FunctionId, BTreeSet<ValueId>> = BTreeMap::new();

    // Seed: the result of every direct call to a nondet function.
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let set = taint.entry(func.id).or_default();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if NONDET_FUNCTIONS.contains(&target.name.as_str()) {
                            if let Some(dst) = inst.dst {
                                set.insert(dst);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        // Intra-procedural data-flow propagation.
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            let mut set = taint.get(&func.id).cloned().unwrap_or_default();
            let before = set.len();
            for block in &func.blocks {
                for inst in &block.instructions {
                    let Some(dst) = inst.dst else { continue };
                    if set.contains(&dst) {
                        continue;
                    }
                    let flows = match &inst.op {
                        Operation::Phi { incoming } => {
                            incoming.iter().any(|(_, v)| set.contains(v))
                        }
                        // A call result is tainted only via the interprocedural pass
                        // below (or the nondet seed) — not by its arguments.
                        Operation::CallDirect { .. } | Operation::CallIndirect { .. } => false,
                        _ => inst.operands.iter().any(|v| set.contains(v)),
                    };
                    if flows {
                        set.insert(dst);
                    }
                }
            }
            if set.len() != before {
                changed = true;
                taint.insert(func.id, set);
            }
        }

        // Inter-procedural: a nondet-tainted argument taints the callee's parameter.
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            let caller_set = taint.get(&func.id).cloned().unwrap_or_default();
            for block in &func.blocks {
                for inst in &block.instructions {
                    let Operation::CallDirect { callee } = &inst.op else {
                        continue;
                    };
                    let Some(target) = module.function(*callee) else {
                        continue;
                    };
                    if target.is_declaration {
                        continue;
                    }
                    let target_id = target.id;
                    for (i, arg) in inst.operands.iter().enumerate() {
                        if !caller_set.contains(arg) {
                            continue;
                        }
                        if let Some(param) = target.params.iter().find(|p| p.index as usize == i) {
                            let callee_set = taint.entry(target_id).or_default();
                            if callee_set.insert(param.id) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    taint
}

/// The set of SSA values in `func` that are SPURIOUS slow linear induction variables:
/// a loop-header phi whose (a) preheader initial value is NOT nondet-tainted, (b)
/// back-edge value is `phi ± c` for a small constant `c` ([`SPURIOUS_IV_STEP_MAX`]),
/// and (c) loop trip count is nondet-controlled (some header phi has a nondet-tainted
/// incoming, or the header's branch condition is nondet-tainted). Values copied from
/// such a phi through casts / copies / pass-through phis (e.g. LCSSA exit phis) are
/// included, so a post-loop use of the final counter value is recognized.
fn function_spurious_iv_set(
    func: &AirFunction,
    module: &AirModule,
    tainted: &BTreeSet<ValueId>,
) -> BTreeSet<ValueId> {
    let cfg = Cfg::build(func);
    let back_edges = find_back_edges(&cfg);
    if back_edges.is_empty() {
        return BTreeSet::new();
    }

    // header block -> its latch (back-edge source) blocks.
    let mut latches: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
    for (src, header) in &back_edges {
        latches.entry(*header).or_default().insert(*src);
    }

    // dst -> defining instruction.
    let mut defs: BTreeMap<ValueId, &Instruction> = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                defs.insert(dst, inst);
            }
        }
    }

    let mut slow: BTreeSet<ValueId> = BTreeSet::new();

    for (header, latch_set) in &latches {
        let Some(hb) = func.blocks.iter().find(|b| b.id == *header) else {
            continue;
        };

        // Is this loop's trip count nondet-controlled?
        let mut nondet_trip = hb.instructions.iter().any(|inst| {
            matches!(&inst.op, Operation::Phi { incoming } if incoming.iter().any(|(_, v)| tainted.contains(v)))
        });
        if !nondet_trip {
            if let Some(term) = hb.terminator() {
                nondet_trip =
                    matches!(term.op, Operation::CondBr { .. } | Operation::Switch { .. })
                        && term.operands.iter().any(|v| tainted.contains(v));
            }
        }
        if !nondet_trip {
            continue;
        }

        for inst in &hb.instructions {
            let Operation::Phi { incoming } = &inst.op else {
                continue;
            };
            let Some(phi_dst) = inst.dst else { continue };

            let mut init_is_nondet = false;
            let mut small_step = false;
            for (pred, val) in incoming {
                if latch_set.contains(pred) {
                    if iv_back_edge_step_is_small(*val, phi_dst, &defs, module) {
                        small_step = true;
                    }
                } else if tainted.contains(val) {
                    init_is_nondet = true;
                }
            }
            if small_step && !init_is_nondet {
                slow.insert(phi_dst);
            }
        }
    }

    if slow.is_empty() {
        return slow;
    }

    // Propagate the "slow linear IV" property through the value graph:
    // - casts / copies / freezes of a slow value stay slow;
    // - a phi ALL of whose incomings are slow (e.g. an LCSSA exit copy, or a
    //   secondary phi in a multi-phi induction SCC — the `twisted` idiom) is slow;
    // - `Add`/`Sub` of slow values and small constants (with ≥1 slow operand) is a
    //   linear combination of slow IVs, hence itself slow-growing (a bounded
    //   per-iteration step), e.g. `i + 1` feeding the induction cycle.
    // A nondet/large operand blocks the last rule, so a genuine `iv + data` never
    // becomes slow.
    let mut changed = true;
    while changed {
        changed = false;
        for block in &func.blocks {
            for inst in &block.instructions {
                let Some(dst) = inst.dst else { continue };
                if slow.contains(&dst) {
                    continue;
                }
                let flows = match &inst.op {
                    Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                        inst.operands.first().is_some_and(|v| slow.contains(v))
                    }
                    Operation::Phi { incoming } => {
                        !incoming.is_empty() && incoming.iter().all(|(_, v)| slow.contains(v))
                    }
                    Operation::BinaryOp { kind }
                        if matches!(kind, BinaryOp::Add | BinaryOp::Sub)
                            && inst.operands.len() == 2 =>
                    {
                        let benign =
                            |v: ValueId| slow.contains(&v) || is_small_int_const(module, v);
                        let (o0, o1) = (inst.operands[0], inst.operands[1]);
                        benign(o0) && benign(o1) && (slow.contains(&o0) || slow.contains(&o1))
                    }
                    _ => false,
                };
                if flows {
                    slow.insert(dst);
                    changed = true;
                }
            }
        }
    }

    slow
}

/// Is `val` (a header phi's back-edge incoming) the small-constant-step update
/// `phi ± c` of `phi_dst`? Accepts `phi + c`, `c + phi`, and `phi - c` (a standard
/// unit/small decrement); rejects `c - phi` and any non-constant or large step.
fn iv_back_edge_step_is_small(
    val: ValueId,
    phi_dst: ValueId,
    defs: &BTreeMap<ValueId, &Instruction>,
    module: &AirModule,
) -> bool {
    let Some(def) = defs.get(&val) else {
        return false;
    };
    let Operation::BinaryOp { kind } = &def.op else {
        return false;
    };
    if def.operands.len() != 2 {
        return false;
    }
    let (a, b) = (def.operands[0], def.operands[1]);
    match kind {
        BinaryOp::Add => {
            (a == phi_dst && is_small_int_const(module, b))
                || (b == phi_dst && is_small_int_const(module, a))
        }
        // Only `phi - c` is a standard IV; `c - phi` is not.
        BinaryOp::Sub => a == phi_dst && is_small_int_const(module, b),
        _ => false,
    }
}

/// Does the UBSan-located overflowing operation look like a SPURIOUS linear-accumulator
/// overflow — a signed `Add`/`Sub` whose operands are all either a small integer
/// constant or a slow linear induction variable ([`function_spurious_iv_set`]), with at
/// least one such induction variable?
///
/// Such an overflow (`x = x + 1`, `return i + j`) is only reachable by driving a
/// nondeterministic loop bound to an astronomically large trip count that the real
/// program's semantics (a loop invariant / precondition) forbids — confirming it is a
/// false alarm on a labeled-TRUE `no-overflow` task (`termination-numeric/twisted`,
/// `…ESOP2008-easy2`). The check NEVER matches a `Mul`/`Div`/negation overflow (so
/// `hard2`'s `2 * d`, Juliet `data * data` stay confirmable), a direct nondet operand
/// (`data + data`), or a large/variable step (`sum += big`) — so genuine direct
/// overflows are unaffected. Returning `true` only ever turns a would-be `false` into
/// `unknown` (recall cost, never a wrong verdict).
///
/// `line`/`column` come from the UBSan report and are matched against instruction
/// spans; if no debug spans are present (nothing matches) it returns `false`, leaving
/// the confirmer's behavior unchanged.
#[must_use]
pub fn overflow_hit_is_spurious_linear_accumulator(
    module: &AirModule,
    line: u32,
    column: Option<u32>,
) -> bool {
    let taint = module_nondet_taint(module);
    let empty = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        // Candidate signed Add/Sub instructions on the faulting source line.
        let candidates: Vec<&Instruction> = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .filter(|inst| {
                matches!(&inst.op, Operation::BinaryOp { kind } if matches!(kind, BinaryOp::Add | BinaryOp::Sub))
                    && inst.span.as_ref().is_some_and(|s| s.line_start == line)
            })
            .collect();
        if candidates.is_empty() {
            continue;
        }

        // Disambiguate by column when UBSan reported one; otherwise only act on a
        // single unambiguous op (never suppress one of several ops sharing a line).
        let selected: Vec<&Instruction> = match column {
            Some(col) => {
                let exact: Vec<&Instruction> = candidates
                    .iter()
                    .copied()
                    .filter(|i| i.span.as_ref().is_some_and(|s| s.col_start == col))
                    .collect();
                if !exact.is_empty() {
                    exact
                } else if candidates.len() == 1 {
                    candidates
                } else {
                    continue;
                }
            }
            None if candidates.len() == 1 => candidates,
            None => continue,
        };

        let tainted = taint.get(&func.id).unwrap_or(&empty);
        let slow = function_spurious_iv_set(func, module, tainted);
        if slow.is_empty() {
            continue;
        }

        for inst in selected {
            if inst.operands.len() != 2 {
                continue;
            }
            let (o0, o1) = (inst.operands[0], inst.operands[1]);
            let benign = |v: ValueId| slow.contains(&v) || is_small_int_const(module, v);
            let has_iv = slow.contains(&o0) || slow.contains(&o1);
            if has_iv && benign(o0) && benign(o1) {
                return true;
            }
        }
    }

    false
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
    // Tests for module_reachable_loops_all_ranked (INT_MAX-boundary gate)
    // -----------------------------------------------------------------------

    /// A value-producing instruction with a result type (local test helper).
    fn typed_inst(
        id: &str,
        op: Operation,
        dst: ValueId,
        operands: Vec<ValueId>,
        ty: saf_core::ids::TypeId,
    ) -> Instruction {
        Instruction {
            id: make_inst_id(id),
            op,
            operands,
            dst: Some(dst),
            span: None,
            symbol: None,
            result_type: Some(ty),
            extensions: BTreeMap::new(),
        }
    }

    /// A terminator (no dst / result) (local test helper).
    fn term_inst(id: &str, op: Operation, operands: Vec<ValueId>) -> Instruction {
        Instruction {
            id: make_inst_id(id),
            op,
            operands,
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    /// Build a module whose `main` is a single ranked counter loop
    /// `while (x < 10) x += 1;` (x : i32), which `loops_are_ranked` proves
    /// terminating with an overflow-safe counter (f = 10 - x).
    fn ranked_counter_loop_module() -> AirModule {
        use saf_core::air::{AirType, BinaryOp, Constant};
        use saf_core::ids::TypeId;

        let i32t = TypeId(make_id("type", b"i32"));
        let i1t = TypeId(make_id("type", b"i1"));
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });
        let mut constants = BTreeMap::new();

        let b0 = make_block_id("rl_entry");
        let h = make_block_id("rl_header");
        let l = make_block_id("rl_latch");
        let e = make_block_id("rl_exit");

        let x = make_value_id("rl_x");
        let xn = make_value_id("rl_xn");
        let cval = make_value_id("rl_c");
        let x_init = make_value_id("rl_x_init");
        let step_v = make_value_id("rl_step");
        let bound_v = make_value_id("rl_bound");
        constants.insert(x_init, Constant::Int { value: 0, bits: 32 });
        constants.insert(step_v, Constant::Int { value: 1, bits: 32 });
        constants.insert(
            bound_v,
            Constant::Int {
                value: 10,
                bits: 32,
            },
        );

        // entry: br header
        let mut entry = AirBlock::new(b0);
        entry
            .instructions
            .push(term_inst("rl_br0", Operation::Br { target: h }, vec![]));

        // header: x = phi[entry:0, latch:xn]; c = icmp slt(x, 10); condbr c -> latch/exit
        let mut header = AirBlock::new(h);
        header.instructions.push(typed_inst(
            "rl_phi",
            Operation::Phi {
                incoming: vec![(b0, x_init), (l, xn)],
            },
            x,
            vec![],
            i32t,
        ));
        header.instructions.push(typed_inst(
            "rl_cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cval,
            vec![x, bound_v],
            i1t,
        ));
        header.instructions.push(term_inst(
            "rl_condbr",
            Operation::CondBr {
                then_target: l,
                else_target: e,
            },
            vec![cval],
        ));

        // latch: xn = add(x, 1); br header
        let mut latch = AirBlock::new(l);
        latch.instructions.push(typed_inst(
            "rl_add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            xn,
            vec![x, step_v],
            i32t,
        ));
        latch
            .instructions
            .push(term_inst("rl_br1", Operation::Br { target: h }, vec![]));

        // exit: ret
        let mut exit = AirBlock::new(e);
        exit.instructions
            .push(term_inst("rl_ret", Operation::Ret, vec![]));

        let main = AirFunction {
            id: make_func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header, latch, exit],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = make_module(vec![main]);
        module.types = types;
        module.constants = constants;
        module
    }

    #[test]
    fn all_ranked_accepts_loop_free_program() {
        // Subsumption: a loop-free program the old gate accepted still passes.
        let module = make_module(vec![make_defined_function("main")]);
        assert!(module_reachable_is_loop_free(&module));
        assert!(module_reachable_loops_all_ranked(&module));
    }

    #[test]
    fn all_ranked_accepts_a_ranked_counter_loop() {
        // The widening: a counted `while (x < 10) x++;` loop is rejected by the old
        // loop-free gate but accepted here (its counter is provably overflow-safe),
        // so the INT_MAX-boundary probes are now injected for such programs.
        let module = ranked_counter_loop_module();
        assert!(!module_reachable_is_loop_free(&module));
        assert!(module_reachable_loops_all_ranked(&module));
    }

    #[test]
    fn all_ranked_rejects_an_unranked_infinite_loop() {
        // Soundness: an unguarded `b0 -> b1 -> b0` infinite loop has no ranking
        // function, so the gate stays closed and no boundary value is injected.
        let fid = make_func_id("main");
        let b0 = make_block_id("main_b0");
        let b1 = make_block_id("main_b1");
        let mut block0 = AirBlock::new(b0);
        block0
            .instructions
            .push(term_inst("b0_term", Operation::Br { target: b1 }, vec![]));
        let mut block1 = AirBlock::new(b1);
        block1
            .instructions
            .push(term_inst("b1_term", Operation::Br { target: b0 }, vec![]));
        let main = AirFunction {
            id: fid,
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block0, block1],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let module = make_module(vec![main]);
        assert!(!module_reachable_is_loop_free(&module));
        assert!(!module_reachable_loops_all_ranked(&module));
    }

    #[test]
    fn all_ranked_ignores_loop_in_unreachable_function() {
        // A loopy function not reachable from main does not close the gate.
        let module = ranked_counter_loop_module();
        // main IS the ranked loop here; add an unreachable infinite-loop function.
        let fid = make_func_id("dead");
        let b0 = make_block_id("dead_b0");
        let mut block0 = AirBlock::new(b0);
        block0
            .instructions
            .push(term_inst("dead_term", Operation::Br { target: b0 }, vec![]));
        let dead = AirFunction {
            id: fid,
            name: "dead".to_string(),
            params: Vec::new(),
            blocks: vec![block0],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut module = module;
        module.functions.push(dead);
        assert!(module_reachable_loops_all_ranked(&module));
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

    /// The start-routine resolver returns the directly-named thread body: a
    /// `pthread_create(&t, attr, worker, arg)` whose 3rd operand's raw id IS the
    /// `worker` function's object id resolves to `"worker"`.
    #[test]
    fn spawn_start_routine_resolves_direct_named_function() {
        let worker = make_defined_function("worker");
        let pthread_create = make_declaration("pthread_create");
        // main: pthread_create(v0, v1, <worker addr>, v3).
        let bid = make_block_id("main_entry");
        let mut block = AirBlock::new(bid);
        let routine = saf_core::ids::ValueId(worker.id.raw());
        block.instructions.push(Instruction {
            id: make_inst_id("main_spawn"),
            op: Operation::CallDirect {
                callee: pthread_create.id,
            },
            operands: vec![
                make_value_id("v0"),
                make_value_id("v1"),
                routine,
                make_value_id("v3"),
            ],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        let main = AirFunction {
            id: make_func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let module = make_module(vec![main, worker, pthread_create]);
        let cg = CallGraph::build(&module);
        assert_eq!(
            spawn_start_routine_name(&module, &cg).as_deref(),
            Some("worker")
        );
    }

    /// An indirect / unresolved thread entry yields `None` (fail-safe — the witness
    /// omits the `enterFunction` anchor rather than guessing).
    #[test]
    fn spawn_start_routine_none_when_unresolved() {
        let pthread_create = make_declaration("pthread_create");
        // main spawns, but the routine operand resolves to no defined function.
        let bid = make_block_id("main_entry");
        let mut block = AirBlock::new(bid);
        block.instructions.push(Instruction {
            id: make_inst_id("main_spawn"),
            op: Operation::CallDirect {
                callee: pthread_create.id,
            },
            operands: vec![
                make_value_id("v0"),
                make_value_id("v1"),
                make_value_id("opaque_fp"),
                make_value_id("v3"),
            ],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        let main = AirFunction {
            id: make_func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let module = make_module(vec![main, pthread_create]);
        let cg = CallGraph::build(&module);
        assert_eq!(spawn_start_routine_name(&module, &cg), None);
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

    // -----------------------------------------------------------------------
    // Tests for reachable_has_dynamic_alloca (lever mem-fuzz-covguided)
    // -----------------------------------------------------------------------

    #[test]
    fn dynamic_alloca_true_when_reachable_vla() {
        // main performs a dynamically-sized stack allocation (VLA / alloca(n)):
        // Alloca { size_bytes: None }. The fuzzer must NOT run on this (native
        // stack-exhaustion false-alarm class).
        let main = make_func_with_inst("main", Operation::Alloca { size_bytes: None }, vec![]);
        let module = make_module(vec![main]);
        let cg = CallGraph::build(&module);
        assert!(reachable_has_dynamic_alloca(&module, &cg));
    }

    #[test]
    fn dynamic_alloca_false_for_fixed_size_alloca() {
        // A compile-time-constant-size alloca (`int x[10]`) cannot exhaust the stack
        // under fuzzing (its size is not nondet-driven) -> not flagged.
        let main = make_func_with_inst(
            "main",
            Operation::Alloca {
                size_bytes: Some(40),
            },
            vec![],
        );
        let module = make_module(vec![main]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_has_dynamic_alloca(&module, &cg));
    }

    #[test]
    fn dynamic_alloca_ignores_unreachable_function() {
        // A dynamic alloca in a function NOT reachable from main is irrelevant.
        let dead = make_func_with_inst("dead", Operation::Alloca { size_bytes: None }, vec![]);
        let main = make_defined_function("main");
        let module = make_module(vec![main, dead]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_has_dynamic_alloca(&module, &cg));
    }

    #[test]
    fn dynamic_alloca_false_without_any_alloca() {
        let main = make_defined_function("main");
        let module = make_module(vec![main]);
        let cg = CallGraph::build(&module);
        assert!(!reachable_has_dynamic_alloca(&module, &cg));
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

    // -----------------------------------------------------------------------
    // Tests for overflow_hit_is_spurious_linear_accumulator
    // -----------------------------------------------------------------------

    /// A value-producing instruction carrying a source span (line/col), for the
    /// span-matching in `overflow_hit_is_spurious_linear_accumulator`.
    fn typed_inst_at(
        id: &str,
        op: Operation,
        dst: ValueId,
        operands: Vec<ValueId>,
        line: u32,
        col: u32,
    ) -> Instruction {
        use saf_core::ids::TypeId;
        let mut inst = typed_inst(id, op, dst, operands, TypeId(make_id("type", b"i32")));
        inst.span = Some(saf_core::span::Span::point(
            saf_core::ids::FileId::new(1),
            0,
            line,
            col,
        ));
        inst
    }

    /// Build an `ESOP2008-easy2`-shaped module: `z = nondet(); x = 12; while (z > 0)
    /// { x = x + 1; z = z - 1; }`. The `x = x + 1` add lives at (line 15, col 9).
    /// If `use_mul` is set, the update at (15, 9) is `x * step` instead of `x + step`
    /// (a `hard2`-shaped `2 * d`), which must NOT be flagged.
    fn esop_like_module(use_mul: bool) -> AirModule {
        use saf_core::air::{AirType, BinaryOp};
        use saf_core::ids::TypeId;

        let i32t = TypeId(make_id("type", b"i32"));
        let i1t = TypeId(make_id("type", b"i1"));
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });

        let b0 = make_block_id("es_entry");
        let h = make_block_id("es_header");
        let l = make_block_id("es_latch");
        let e = make_block_id("es_exit");

        let z0 = make_value_id("es_z0");
        let zphi = make_value_id("es_zphi");
        let xphi = make_value_id("es_xphi");
        let cond = make_value_id("es_cond");
        let xnext = make_value_id("es_xnext");
        let znext = make_value_id("es_znext");
        let x_init = make_value_id("es_xinit");
        let one = make_value_id("es_one");
        let two = make_value_id("es_two");
        let zero = make_value_id("es_zero");

        let mut constants = BTreeMap::new();
        constants.insert(x_init, Constant::int(12, 32));
        constants.insert(one, Constant::int(1, 32));
        constants.insert(two, Constant::int(2, 32));
        constants.insert(zero, Constant::int(0, 32));

        let nondet = make_declaration("__VERIFIER_nondet_int");

        let mut entry = AirBlock::new(b0);
        entry.instructions.push(typed_inst(
            "es_call",
            Operation::CallDirect { callee: nondet.id },
            z0,
            vec![],
            i32t,
        ));
        entry
            .instructions
            .push(term_inst("es_br0", Operation::Br { target: h }, vec![]));

        let mut header = AirBlock::new(h);
        header.instructions.push(typed_inst(
            "es_zphi",
            Operation::Phi {
                incoming: vec![(b0, z0), (l, znext)],
            },
            zphi,
            vec![],
            i32t,
        ));
        header.instructions.push(typed_inst(
            "es_xphi",
            Operation::Phi {
                incoming: vec![(b0, x_init), (l, xnext)],
            },
            xphi,
            vec![],
            i32t,
        ));
        header.instructions.push(typed_inst(
            "es_cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            cond,
            vec![zphi, zero],
            i1t,
        ));
        header.instructions.push(term_inst(
            "es_condbr",
            Operation::CondBr {
                then_target: l,
                else_target: e,
            },
            vec![cond],
        ));

        let mut latch = AirBlock::new(l);
        let update_kind = if use_mul {
            BinaryOp::Mul
        } else {
            BinaryOp::Add
        };
        let step = if use_mul { two } else { one };
        latch.instructions.push(typed_inst_at(
            "es_xnext",
            Operation::BinaryOp { kind: update_kind },
            xnext,
            vec![xphi, step],
            15,
            9,
        ));
        latch.instructions.push(typed_inst_at(
            "es_znext",
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            znext,
            vec![zphi, one],
            16,
            9,
        ));
        latch
            .instructions
            .push(term_inst("es_br1", Operation::Br { target: h }, vec![]));

        let mut exit = AirBlock::new(e);
        exit.instructions
            .push(term_inst("es_ret", Operation::Ret, vec![]));

        let main = AirFunction {
            id: make_func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![entry, header, latch, exit],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = make_module(vec![main, nondet]);
        module.types = types;
        module.constants = constants;
        module
    }

    #[test]
    fn spurious_accumulator_add_in_nondet_loop_is_flagged() {
        // `x = x + 1` accumulator in a `while (z > 0)` loop with z = nondet(): the
        // overflow is only reachable at an astronomical trip count -> abstain.
        let module = esop_like_module(false);
        assert!(overflow_hit_is_spurious_linear_accumulator(
            &module,
            15,
            Some(9)
        ));
        // Line-only match (no UBSan column) also works (single Add on the line).
        assert!(overflow_hit_is_spurious_linear_accumulator(
            &module, 15, None
        ));
    }

    #[test]
    fn multiplicative_update_in_nondet_loop_is_not_flagged() {
        // `x = x * 2` (the `hard2` `2 * d` shape) is a Mul, never suppressed — a
        // genuine fast (log-many-iterations) overflow stays confirmable.
        let module = esop_like_module(true);
        assert!(!overflow_hit_is_spurious_linear_accumulator(
            &module,
            15,
            Some(9)
        ));
    }

    #[test]
    fn direct_nondet_addition_is_not_flagged() {
        // `y = z + z` with z = nondet() and NO loop: a genuine direct overflow, not a
        // slow accumulator -> must stay confirmable.
        use saf_core::air::BinaryOp;
        let z0 = make_value_id("dn_z0");
        let y = make_value_id("dn_y");
        let nondet = make_declaration("__VERIFIER_nondet_int");

        let bid = make_block_id("dn_entry");
        let mut block = AirBlock::new(bid);
        block.instructions.push(typed_inst_at(
            "dn_call",
            Operation::CallDirect { callee: nondet.id },
            z0,
            vec![],
            4,
            1,
        ));
        block.instructions.push(typed_inst_at(
            "dn_add",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            y,
            vec![z0, z0],
            5,
            9,
        ));
        block
            .instructions
            .push(term_inst("dn_ret", Operation::Ret, vec![]));
        let main = AirFunction {
            id: make_func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: Some(bid),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let module = make_module(vec![main, nondet]);
        assert!(!overflow_hit_is_spurious_linear_accumulator(
            &module,
            5,
            Some(9)
        ));
    }

    #[test]
    fn accumulator_in_constant_bounded_loop_is_not_flagged() {
        // `while (x < 10) x++;` — the trip count is a compile-time constant, not
        // nondet-controlled, so the counter is not a spurious accumulator (preserving
        // the counted-loop-sink confirmations). Query the `x + 1` add at its span.
        let mut module = ranked_counter_loop_module();
        // Give the `rl_add` (x + 1) a span so it is a candidate at (line 7, col 3).
        for func in &mut module.functions {
            for block in &mut func.blocks {
                for inst in &mut block.instructions {
                    if inst.dst == Some(make_value_id("rl_xn")) {
                        inst.span = Some(saf_core::span::Span::point(
                            saf_core::ids::FileId::new(1),
                            0,
                            7,
                            3,
                        ));
                    }
                }
            }
        }
        assert!(!overflow_hit_is_spurious_linear_accumulator(
            &module,
            7,
            Some(3)
        ));
    }

    #[test]
    fn interprocedural_param_bounded_accumulator_is_flagged() {
        // `twisted`-style: main calls f(nondet()); f loops `while (i < p) i++;` and
        // returns i + i. The bound p is a parameter fed a nondet argument, so the loop
        // is nondet-controlled and the `i + i` overflow is a spurious accumulator.
        use saf_core::air::{AirType, BinaryOp};
        use saf_core::ids::TypeId;

        let i32t = TypeId(make_id("type", b"i32"));
        let i1t = TypeId(make_id("type", b"i1"));
        let mut types = BTreeMap::new();
        types.insert(i32t, AirType::Integer { bits: 32 });
        types.insert(i1t, AirType::Integer { bits: 1 });

        // --- f(p) ---
        let fb0 = make_block_id("f_entry");
        let fh = make_block_id("f_header");
        let fl = make_block_id("f_latch");
        let fe = make_block_id("f_exit");
        let p = make_value_id("f_p");
        let iphi = make_value_id("f_iphi");
        let cond = make_value_id("f_cond");
        let inext = make_value_id("f_inext");
        let sum = make_value_id("f_sum");
        let i_init = make_value_id("f_iinit");
        let one = make_value_id("f_one");

        let mut fentry = AirBlock::new(fb0);
        fentry
            .instructions
            .push(term_inst("f_br0", Operation::Br { target: fh }, vec![]));
        let mut fheader = AirBlock::new(fh);
        fheader.instructions.push(typed_inst(
            "f_iphi",
            Operation::Phi {
                incoming: vec![(fb0, i_init), (fl, inext)],
            },
            iphi,
            vec![],
            i32t,
        ));
        fheader.instructions.push(typed_inst(
            "f_cmp",
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            cond,
            vec![iphi, p],
            i1t,
        ));
        fheader.instructions.push(term_inst(
            "f_condbr",
            Operation::CondBr {
                then_target: fl,
                else_target: fe,
            },
            vec![cond],
        ));
        let mut flatch = AirBlock::new(fl);
        flatch.instructions.push(typed_inst(
            "f_inext",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            inext,
            vec![iphi, one],
            i32t,
        ));
        flatch
            .instructions
            .push(term_inst("f_br1", Operation::Br { target: fh }, vec![]));
        let mut fexit = AirBlock::new(fe);
        fexit.instructions.push(typed_inst_at(
            "f_sum",
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            sum,
            vec![iphi, iphi],
            20,
            14,
        ));
        fexit
            .instructions
            .push(term_inst("f_ret", Operation::Ret, vec![]));

        let f = AirFunction {
            id: make_func_id("f"),
            name: "f".to_string(),
            params: vec![AirParam {
                id: p,
                name: Some("p".to_string()),
                index: 0,
                param_type: Some(i32t),
            }],
            blocks: vec![fentry, fheader, flatch, fexit],
            entry_block: Some(fb0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        // --- main: k = nondet(); f(k) ---
        let nondet = make_declaration("__VERIFIER_nondet_int");
        let k = make_value_id("m_k");
        let mb0 = make_block_id("m_entry");
        let mut mentry = AirBlock::new(mb0);
        mentry.instructions.push(typed_inst(
            "m_call_nd",
            Operation::CallDirect { callee: nondet.id },
            k,
            vec![],
            i32t,
        ));
        mentry.instructions.push(Instruction {
            id: make_inst_id("m_call_f"),
            op: Operation::CallDirect { callee: f.id },
            operands: vec![k],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        mentry
            .instructions
            .push(term_inst("m_ret", Operation::Ret, vec![]));
        let main = AirFunction {
            id: make_func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![mentry],
            entry_block: Some(mb0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = make_module(vec![main, f, nondet]);
        module.types = types;
        module.constants.insert(i_init, Constant::int(0, 32));
        module.constants.insert(one, Constant::int(1, 32));

        assert!(overflow_hit_is_spurious_linear_accumulator(
            &module,
            20,
            Some(14)
        ));
    }
}
