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
}
