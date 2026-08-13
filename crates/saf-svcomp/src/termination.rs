//! `termination` (loop-free ∧ acyclic-callgraph) TRUE — SAF's first sound-TRUE
//! slice (R7, plan 201).
//!
//! Emits a sound `termination = true` verdict from a **static structural proof**,
//! never `false`. The sufficient (decidable, sound, incomplete) condition
//! [`program_structurally_terminates`] holds iff **T∧L∧A∧I∧E**:
//!
//! - **(T)** the program has a *defined* `main`;
//! - **(L)** every function reachable-from-`main` that is *defined* has a
//!   **loop-free** (DAG) CFG (no back-edge);
//! - **(A)** the reachable **call graph is acyclic** (no recursion);
//! - **(I)** there is **no reachable unresolved indirect call** — any reachable
//!   `CallIndirect` / `IndirectPlaceholder` forces abstain (it could hide a
//!   recursive cycle or an unknown non-returning target). This also keeps the
//!   direct call graph *complete* over the reachable set, so (A) is sound
//!   (CLAUDE.md redline #8), and lets R7 avoid pointer analysis entirely;
//! - **(E)** every reachable *external* callee is **known-terminating**
//!   ([`is_known_returning_external`]); any non-allowlisted reachable external
//!   forces abstain (an opaque external may loop / block / not return).
//!
//! Finite call depth (A) × finite intra-procedural paths (L) × terminating
//! externals (E) ⇒ only finite executions ⇒ the program always terminates. A
//! genuinely non-terminating program must contain a loop, infinite recursion, or
//! a looping/blocking external — each forces an abstain — so this never emits a
//! wrong `true`.

use std::collections::BTreeSet;

use saf_analysis::callgraph::{CallGraph, CallGraphNode};
use saf_analysis::cfg::Cfg;
use saf_analysis::graph_algo::{dfs, tarjan_scc};
use saf_core::air::{AirModule, Operation};
use saf_core::ids::FunctionId;

use crate::fast_paths::cfg_has_loops;

/// Is `name` a **known-terminating** external — one that provably (a) always
/// returns or halts the program, (b) contains no unbounded loop, and (c) never
/// invokes a program callback (which could re-enter user code and loop/recurse)?
///
/// Only such externals may appear on a reachable path of a program R7 proves
/// terminating; any other reachable external forces abstain. Callback-taking libc
/// (`qsort`/`bsearch`/`atexit`) and potentially-blocking I/O (`read`/`recv`/
/// `scanf`-family) are deliberately excluded. The `exit`/`abort`/`__assert_fail`
/// family does not *return* but *halts* the program, which is termination.
#[must_use]
pub fn is_known_returning_external(name: &str) -> bool {
    // All typed __VERIFIER_nondet_* generators return a value immediately.
    if name.starts_with("__VERIFIER_nondet_") {
        return true;
    }
    ALLOWLISTED_EXTERNALS.contains(&name)
}

/// The known-terminating external allowlist (see [`is_known_returning_external`]
/// for the soundness criterion each entry must satisfy).
const ALLOWLISTED_EXTERNALS: &[&str] = &[
    // SV-COMP verifier primitives + program-halting (halt = terminates).
    "__VERIFIER_assume",
    "__VERIFIER_assert",
    "__VERIFIER_error",
    "reach_error",
    "__assert_fail",
    "abort",
    "exit",
    "_exit",
    "quick_exit",
    // Allocation (bounded, always returns).
    "malloc",
    "calloc",
    "realloc",
    "free",
    "aligned_alloc",
    // Bounded, callback-free memory/string libc.
    "memcpy",
    "memmove",
    "memset",
    "memcmp",
    "strcpy",
    "strncpy",
    "strlen",
    "strcmp",
    "strncmp",
    // Output-only formatted I/O (bounded by the format string; no callback).
    "printf",
    "fprintf",
    "sprintf",
    "snprintf",
    "puts",
    "putchar",
    "fputc",
    "fputs",
    // Pure math (terminating).
    "sqrt",
];

/// The exact SV-COMP verdict string for a proven `termination` TRUE.
///
/// A bare `true` (NOT `true(termination)`): BenchExec's `RESULT_TRUE_PROP` is the
/// bare token, and `saf.py` maps it directly. A `true` writes no witness (the
/// verify write-gate keys on a `false`-prefix).
#[must_use]
pub fn termination_verdict() -> &'static str {
    "true"
}

/// Does the program provably **always terminate** by the sufficient structural
/// condition T∧L∧A∧I∧E (module docs)? Sound and incomplete — abstains (`false`
/// here ⇒ the strategy emits `unknown`, never a verdict) on any loop, recursion,
/// reachable indirect call, or non-allowlisted reachable external.
#[must_use]
pub fn program_structurally_terminates(module: &AirModule) -> bool {
    // (T) a defined `main`.
    let Some(main_func) = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)
    else {
        return false;
    };
    let main_id = main_func.id;

    // Code that runs OUTSIDE main's call graph — global constructors
    // (`llvm.global_ctors`, run before main) and destructors (`llvm.global_dtors`,
    // run after main returns) — can loop forever. R7 only checks
    // reachable-from-main, so abstain if any static initializer/finalizer exists
    // (else e.g. `__attribute__((constructor)) void c(){for(;;);}` would be a −32).
    if module
        .globals
        .iter()
        .any(|g| g.name == "llvm.global_ctors" || g.name == "llvm.global_dtors")
    {
        return false;
    }

    let cg = CallGraph::build(module);

    // Node-level reachability from `main` (so `External` and `IndirectPlaceholder`
    // nodes on reachable paths are visible to the (I)/(E) checks below).
    let reachable_nodes: BTreeSet<CallGraphNode> = match cg.node_for_function(main_id) {
        Some(main_node) => {
            let mut set: BTreeSet<CallGraphNode> = dfs(main_node, &cg).into_iter().collect();
            set.insert(main_node.clone());
            set
        }
        // `main` is defined but absent from the call graph — `CallGraph::build`
        // always adds a node per function, so this is unreachable in practice;
        // abstain rather than risk an optimistic `true` that skips the (I)/(E)
        // body checks (−32 defense-in-depth).
        None => return false,
    };

    // (I) a reachable unresolved indirect call ⇒ abstain. Keeps the direct call
    // graph complete over the reachable set (so the (A) acyclicity check is sound,
    // CLAUDE.md redline #8) and avoids pointer analysis entirely; an indirect
    // target could also hide recursion or a non-returning function.
    if reachable_nodes.iter().any(CallGraphNode::is_indirect) {
        return false;
    }

    let reachable_fids: BTreeSet<FunctionId> = reachable_nodes
        .iter()
        .filter_map(CallGraphNode::function_id)
        .chain(std::iter::once(main_id))
        .collect();

    // (E) every reachable external declaration must be known-terminating; plus a
    // second-form (I) check that no reachable defined body contains a `CallIndirect`.
    for func in &module.functions {
        if !reachable_fids.contains(&func.id) {
            continue;
        }
        if func.is_declaration {
            if !is_known_returning_external(&func.name) {
                return false;
            }
            continue;
        }
        for block in &func.blocks {
            // (CFG completeness) a reachable defined block with no recognized
            // terminator means the frontend DROPPED an unsupported terminator
            // (e.g. `indirectbr`/`callbr` → `mapping.rs` catch-all `Ok(None)`),
            // leaving the block edge-less. Loop detection would then miss a
            // computed-goto/asm-goto back-edge ⇒ a −32. Abstain instead.
            if block.terminator().is_none() {
                return false;
            }
            if block
                .instructions
                .iter()
                .any(|inst| matches!(inst.op, Operation::CallIndirect { .. }))
            {
                return false;
            }
        }
    }

    // (A) the reachable call graph is acyclic (no recursion).
    if !reachable_callgraph_is_acyclic(&cg, &reachable_nodes) {
        return false;
    }

    // (L) every reachable defined function has a loop-free CFG.
    !module.functions.iter().any(|f| {
        reachable_fids.contains(&f.id) && !f.is_declaration && cfg_has_loops(&Cfg::build(f))
    })
}

/// Is the reachable call graph acyclic (no recursion)?
///
/// `reachable` is the DFS closure from `main`'s node, so it is closed under the
/// call graph's successor relation and [`tarjan_scc`] sees only intra-reachable
/// edges. The graph is cyclic iff some SCC has more than one node (mutual
/// recursion) or a node calls itself (self-recursion).
fn reachable_callgraph_is_acyclic(cg: &CallGraph, reachable: &BTreeSet<CallGraphNode>) -> bool {
    for scc in tarjan_scc(reachable, cg) {
        if scc.len() > 1 {
            return false;
        }
        if let Some(node) = scc.iter().next() {
            if cg
                .callees_of(node)
                .is_some_and(|callees| callees.contains(node))
            {
                return false;
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

    use saf_core::air::{AirBlock, AirFunction, AirGlobal, Instruction, Operation};
    use saf_core::id::make_id;
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ObjId, ValueId};

    fn make_value_id(name: &str) -> ValueId {
        ValueId(make_id("value", name.as_bytes()))
    }

    // --- id + builder helpers (mirror the fast_paths test idiom) -------------

    fn make_func_id(name: &str) -> FunctionId {
        FunctionId(make_id("func", name.as_bytes()))
    }
    fn make_block_id(name: &str) -> BlockId {
        BlockId(make_id("block", name.as_bytes()))
    }
    fn make_inst_id(name: &str) -> InstId {
        InstId(make_id("inst", name.as_bytes()))
    }
    fn make_module_id(name: &str) -> ModuleId {
        ModuleId(make_id("module", name.as_bytes()))
    }

    fn defined(name: &str) -> AirFunction {
        // Real compiled blocks always end in a terminator; the (CFG-completeness)
        // gate abstains on any block that lacks one, so builders must terminate.
        let mut block = AirBlock::new(make_block_id(&format!("{name}_entry")));
        block
            .instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret));
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

    /// A defined function whose single block has an instruction but NO recognized
    /// terminator (models a frontend-dropped terminator such as `indirectbr`).
    fn unterminated(name: &str) -> AirFunction {
        let mut block = AirBlock::new(make_block_id(&format!("{name}_entry")));
        block
            .instructions
            .push(inst(&format!("{name}_load"), Operation::Load));
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

    fn declaration(name: &str) -> AirFunction {
        AirFunction {
            id: make_func_id(name),
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

    fn inst(id: &str, op: Operation) -> Instruction {
        Instruction {
            id: make_inst_id(id),
            op,
            operands: Vec::new(),
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    /// A defined function whose single block calls each `callee` id directly.
    fn calling(name: &str, callees: &[FunctionId]) -> AirFunction {
        let mut block = AirBlock::new(make_block_id(&format!("{name}_entry")));
        for (i, callee) in callees.iter().enumerate() {
            block.instructions.push(inst(
                &format!("{name}_call_{i}"),
                Operation::CallDirect { callee: *callee },
            ));
        }
        block
            .instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret));
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

    /// A defined function whose single block contains one indirect call.
    fn indirect(name: &str) -> AirFunction {
        let mut block = AirBlock::new(make_block_id(&format!("{name}_entry")));
        block.instructions.push(inst(
            &format!("{name}_icall"),
            Operation::CallIndirect {
                expected_signature: None,
            },
        ));
        block
            .instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret));
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

    /// A defined function with two blocks and a back-edge (a loop).
    fn looping(name: &str) -> AirFunction {
        let b0 = make_block_id(&format!("{name}_b0"));
        let b1 = make_block_id(&format!("{name}_b1"));
        let mut block0 = AirBlock::new(b0);
        block0
            .instructions
            .push(inst(&format!("{name}_br0"), Operation::Br { target: b1 }));
        let mut block1 = AirBlock::new(b1);
        block1
            .instructions
            .push(inst(&format!("{name}_br1"), Operation::Br { target: b0 })); // back-edge
        AirFunction {
            id: make_func_id(name),
            name: name.to_string(),
            params: Vec::new(),
            blocks: vec![block0, block1],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn module(functions: Vec<AirFunction>) -> AirModule {
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

    // --- is_known_returning_external ----------------------------------------

    #[test]
    fn allowlist_accepts_verifier_and_pure_libc() {
        for n in [
            "__VERIFIER_nondet_int",
            "__VERIFIER_nondet_pointer",
            "__VERIFIER_nondet_bool",
            "__VERIFIER_assume",
            "abort",
            "exit",
            "reach_error",
            "malloc",
            "free",
            "printf",
            "snprintf",
            "memcmp",
            "strcmp",
            "strncmp",
            "sqrt",
        ] {
            assert!(is_known_returning_external(n), "{n} should be allowlisted");
        }
    }

    #[test]
    fn allowlist_rejects_blocking_callback_and_opaque() {
        for n in [
            "read",
            "recv",
            "poll",
            "scanf",
            "fscanf",
            "gets",
            "getchar",
            "qsort",
            "bsearch",
            "atexit",
            "my_helper",
            "__startrek_foo",
            "unknown_0x1",
        ] {
            assert!(
                !is_known_returning_external(n),
                "{n} must NOT be allowlisted"
            );
        }
    }

    // --- termination_verdict -------------------------------------------------

    #[test]
    fn verdict_is_bare_true() {
        assert_eq!(termination_verdict(), "true");
    }

    // --- program_structurally_terminates: TRUE cases ------------------------

    #[test]
    fn straight_line_main_terminates() {
        assert!(program_structurally_terminates(&module(vec![defined(
            "main"
        )])));
    }

    #[test]
    fn main_calling_allowlisted_external_terminates() {
        let m = module(vec![
            calling(
                "main",
                &[
                    make_func_id("printf"),
                    make_func_id("__VERIFIER_nondet_int"),
                ],
            ),
            declaration("printf"),
            declaration("__VERIFIER_nondet_int"),
        ]);
        assert!(program_structurally_terminates(&m));
    }

    #[test]
    fn loop_free_defined_chain_terminates() {
        // main -> foo -> bar, all loop-free, no externals.
        let m = module(vec![
            calling("main", &[make_func_id("foo")]),
            calling("foo", &[make_func_id("bar")]),
            defined("bar"),
        ]);
        assert!(program_structurally_terminates(&m));
    }

    #[test]
    fn loop_in_unreachable_function_is_ignored() {
        // main is loop-free and calls nothing; a disconnected function loops.
        let m = module(vec![defined("main"), looping("dead")]);
        assert!(program_structurally_terminates(&m));
    }

    // --- program_structurally_terminates: ABSTAIN cases ---------------------

    #[test]
    fn reachable_loop_abstains() {
        let m = module(vec![
            calling("main", &[make_func_id("loopy")]),
            looping("loopy"),
        ]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn main_itself_looping_abstains() {
        let m = module(vec![looping("main")]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn self_recursion_abstains() {
        // main -> rec -> rec (self-loop in the call graph).
        let m = module(vec![
            calling("main", &[make_func_id("rec")]),
            calling("rec", &[make_func_id("rec")]),
        ]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn mutual_recursion_abstains() {
        // main -> a -> b -> a (an SCC of size 2).
        let m = module(vec![
            calling("main", &[make_func_id("a")]),
            calling("a", &[make_func_id("b")]),
            calling("b", &[make_func_id("a")]),
        ]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn reachable_direct_indirect_call_abstains() {
        let m = module(vec![indirect("main")]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn reachable_indirect_call_in_callee_abstains() {
        let m = module(vec![
            calling("main", &[make_func_id("helper")]),
            indirect("helper"),
        ]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn non_allowlisted_external_abstains() {
        let m = module(vec![
            calling("main", &[make_func_id("read")]),
            declaration("read"),
        ]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn no_main_abstains() {
        let m = module(vec![defined("foo")]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn declaration_main_abstains() {
        let m = module(vec![declaration("main")]);
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn unterminated_block_abstains() {
        // A reachable defined block with no recognized terminator means the
        // frontend dropped an unsupported terminator (e.g. `indirectbr`) — the CFG
        // (hence loop detection) is incomplete ⇒ abstain (a computed-goto loop
        // would otherwise be invisible = −32).
        assert!(!program_structurally_terminates(&module(vec![
            unterminated("main")
        ])));
    }

    #[test]
    fn global_constructors_abstain() {
        // A `llvm.global_ctors` entry runs BEFORE main (outside main's call graph)
        // and could loop forever ⇒ abstain.
        let mut m = module(vec![defined("main")]);
        m.globals.push(AirGlobal::new(
            make_value_id("gc"),
            ObjId(make_id("obj", b"gc")),
            "llvm.global_ctors",
        ));
        assert!(!program_structurally_terminates(&m));
    }

    #[test]
    fn global_destructors_abstain() {
        // A `llvm.global_dtors` entry runs AFTER main returns ⇒ abstain.
        let mut m = module(vec![defined("main")]);
        m.globals.push(AirGlobal::new(
            make_value_id("gd"),
            ObjId(make_id("obj", b"gd")),
            "llvm.global_dtors",
        ));
        assert!(!program_structurally_terminates(&m));
    }
}
