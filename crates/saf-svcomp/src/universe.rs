//! The **reachable-universe gate** shared by SAF's TRUE provers.
//!
//! A TRUE verdict is only sound if the analysis actually SAW the whole program.
//! If any code that can execute was never examined — a function called through an
//! unresolved pointer, a library routine whose effects are unknown, a block whose
//! terminator the frontend dropped, a static initializer that runs outside `main` —
//! then "I found no violation" does not mean "there is no violation". This module
//! answers exactly one question:
//!
//! > Is the direct call graph from the entry roots a COMPLETE and HONEST picture of
//! > what this program can execute?
//!
//! and answers it **fail-closed**. It owns no property semantics: no locks, no
//! ranking functions, no bounds, no aliasing. Callers layer those on top.
//!
//! # Why this exists
//!
//! The same gate was written twice — [`crate::race_true`] and [`crate::termination`]
//! — and `plans/214` needs a third for `valid-memsafety`. The two copies had drifted
//! on four axes (return type, promotion, thread roots, external policy), and one of
//! those differences was a latent soundness trap; see [`ThreadRoots::MainOnly`].
//!
//! # What callers keep
//!
//! Anything property-specific stays at the call site, deliberately:
//! * `termination` runs [`crate::promote`] first (its ranking synthesizer needs SSA)
//!   and does an extra CALL-GRAPH-NODE-level indirect check, because its SCC
//!   analysis needs a complete graph — a stronger requirement than universe
//!   completeness.
//! * `race_true` discovers thread entries with PTA + ICFG + MTA and needs the
//!   resulting `threads` map afterwards, so discovery stays there rather than being
//!   duplicated in here.

use std::collections::BTreeSet;

use saf_analysis::callgraph::CallGraph;
use saf_core::air::{AirModule, Operation};
use saf_core::ids::FunctionId;

/// Thread-spawn primitives, canonically. [`crate::race_true`] feeds the same list to
/// `MtaConfig::thread_create_funcs`, so the spawn set the thread model discovers and
/// the spawn set this gate reasons about can never drift apart.
pub const SPAWN_FUNCTIONS: &[&str] = &["pthread_create", "thrd_create"];

/// Which external (declaration-only) functions a caller is willing to admit into a
/// sound proof.
///
/// The predicate is the caller's entire soundness argument about code it cannot see,
/// and it is genuinely property-specific: `malloc` and `memcpy` are *terminating*
/// (so `termination` admits them) but neither *race-inert* nor *memory-safe* (so
/// `race_true` and a memsafety prover must not).
pub struct ExternalPolicy {
    /// For diagnostics and for the [`ThreadRoots::MainOnly`] error message.
    pub name: &'static str,
    /// `true` ⇒ this external may appear in a reachable, still-sound program.
    pub admits: fn(&str) -> bool,
}

impl ExternalPolicy {
    /// Does this policy admit any thread-spawn primitive?
    ///
    /// Derived from the predicate rather than declared separately, so it cannot
    /// disagree with the allowlist it describes.
    #[must_use]
    pub fn admits_spawn(&self) -> bool {
        SPAWN_FUNCTIONS.iter().any(|s| (self.admits)(s))
    }
}

/// Where reachability starts.
#[derive(Debug, Clone)]
pub enum ThreadRoots {
    /// `main`'s call tree only.
    ///
    /// Legal **only** when no thread can be spawned, and
    /// [`reachable_universe`] ENFORCES that rather than trusting the caller.
    ///
    /// This is the latent trap that motivated the check. `termination` uses
    /// main-only roots and is sound today — but only as a *corollary* of its
    /// external policy, which happens not to list `pthread_create`, so the (E)
    /// check fires first and no threaded program ever reaches the loop analysis.
    /// Nothing in the code said so. Adding `pthread_create` to that allowlist to
    /// win recall would have silently made every threaded program's spawned code
    /// invisible — a wrong TRUE worth −32, with no test to catch it. Pairing the
    /// root set with the policy makes that mistake a loud, immediate error.
    MainOnly,
    /// `main`'s tree ∪ the tree of every supplied thread entry.
    ///
    /// The caller discovers the entries (see the module docs for why) and is
    /// responsible for that discovery being a sound *superset* of the real ones.
    Also(BTreeSet<FunctionId>),
}

/// A program's provably-complete executable surface.
#[derive(Debug, Clone)]
pub struct Universe {
    /// The entry function.
    pub main: FunctionId,
    /// Functions reachable from `main` alone — i.e. what the MAIN THREAD may run.
    /// Always a subset of [`Universe::reachable`], and equal to it under
    /// [`ThreadRoots::MainOnly`]. Exposed because a caller reasoning about the main
    /// thread specifically (`race_true`'s happens-before landmarks) needs it, and it
    /// is computed here anyway — recomputing it at the call site would be a second
    /// traversal and a chance for the two to disagree.
    pub main_tree: BTreeSet<FunctionId>,
    /// Every function that may execute: `main`'s tree ∪ each thread entry's tree.
    /// Every one has been checked against the policy; nothing outside it can run.
    pub reachable: BTreeSet<FunctionId>,
}

/// Functions reachable from `entry` over the DIRECT call graph.
///
/// Indirect and external call-graph nodes have no `FunctionId` and are skipped
/// here — they are caught by the reachable-body scan in [`reachable_universe`],
/// which is what makes skipping them safe.
#[must_use]
pub fn reachable_functions(cg: &CallGraph, entry: FunctionId) -> BTreeSet<FunctionId> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![entry];
    seen.insert(entry);
    while let Some(f) = stack.pop() {
        let Some(node) = cg.node_for_function(f) else {
            continue;
        };
        let Some(callees) = cg.callees_of(node) else {
            continue;
        };
        for callee in callees {
            if let Some(cf) = callee.function_id() {
                if seen.insert(cf) {
                    stack.push(cf);
                }
            }
        }
    }
    seen
}

/// Compute the reachable universe, or explain why no sound proof is possible.
///
/// `Err(reason)` is a diagnostic tag only — callers must treat ANY error as
/// "abstain", never as evidence about the property.
///
/// # Errors
///
/// * `main-only-roots-with-spawn-admitting-policy:<policy>` — see [`ThreadRoots::MainOnly`].
/// * `no-defined-main` — no `main`, or `main` is only a declaration.
/// * `global-ctors-dtors` — code runs outside `main`'s call graph.
/// * `non-inert-external:<name>` — a reachable external the policy does not admit.
/// * `dropped-terminator:<func>` — a reachable block with no terminator: the frontend
///   dropped an `indirectbr`/`callbr` and the CFG is a lie.
/// * `reachable-indirect-call:<func>` — an unresolved call target.
pub fn reachable_universe(
    module: &AirModule,
    cg: &CallGraph,
    policy: &ExternalPolicy,
    roots: &ThreadRoots,
) -> Result<Universe, String> {
    // (0) The root set and the policy must agree about threads. Checked FIRST and
    //     statically — it is a property of the CALLER's configuration, not of this
    //     program, so it should fail on every input rather than only on the
    //     threaded ones that happen to expose it. See `ThreadRoots::MainOnly`.
    if matches!(roots, ThreadRoots::MainOnly) && policy.admits_spawn() {
        return Err(format!(
            "main-only-roots-with-spawn-admitting-policy:{}",
            policy.name
        ));
    }

    // (T) A defined `main`. A declaration-only `main` means the entry point's body
    //     was never ingested, so there is no program to reason about.
    let main_func = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)
        .ok_or_else(|| "no-defined-main".to_string())?;
    let main_id = main_func.id;

    // (C) Static initializers and finalizers run OUTSIDE main's call graph —
    //     `__attribute__((constructor))` before it, destructors after it — so a
    //     universe rooted at `main` would silently exclude them.
    if module
        .globals
        .iter()
        .any(|g| g.name == "llvm.global_ctors" || g.name == "llvm.global_dtors")
    {
        return Err("global-ctors-dtors".into());
    }

    // (G) `main` must have a call-graph node, or reachability from it is vacuous
    //     and every downstream conclusion would be drawn over an empty universe.
    //     `CallGraph::build` adds a node per function, so this is unreachable in
    //     practice — which is exactly why it must be an explicit error rather than
    //     an optimistic empty set.
    if cg.node_for_function(main_id).is_none() {
        return Err("main-not-in-callgraph".into());
    }

    // (R) Everything that may execute: main's tree, plus each thread entry's tree.
    //     `pthread_create` is opaque, so a thread body is generally NOT reachable
    //     from `main` in the call graph and must be unioned in explicitly.
    let main_tree = reachable_functions(cg, main_id);
    let mut reachable = main_tree.clone();
    if let ThreadRoots::Also(entries) = roots {
        for entry in entries {
            reachable.extend(reachable_functions(cg, *entry));
        }
    }

    // (E/I/CFG) Judge every reachable function — and ONLY the reachable ones.
    // Gating on unreachable code would cost recall for no soundness gain.
    for func in &module.functions {
        if !reachable.contains(&func.id) {
            continue;
        }
        if func.is_declaration {
            // (E) An external the policy does not admit could do anything the
            //     caller's proof assumes away.
            if !(policy.admits)(&func.name) {
                return Err(format!("non-inert-external:{}", func.name));
            }
            continue;
        }
        for block in &func.blocks {
            // (CFG) No terminator means the frontend DROPPED an unsupported one
            //       (`indirectbr`/`callbr`), leaving the block edge-less. The CFG
            //       is then a lie and every path-based conclusion drawn from it is
            //       unfounded.
            if block.terminator().is_none() {
                return Err(format!("dropped-terminator:{}", func.name));
            }
            // (I) An unresolved call target can reach code outside the universe —
            //     which is exactly what this gate exists to rule out.
            if block
                .instructions
                .iter()
                .any(|inst| matches!(inst.op, Operation::CallIndirect { .. }))
            {
                return Err(format!("reachable-indirect-call:{}", func.name));
            }
        }
    }

    Ok(Universe {
        main: main_id,
        main_tree,
        reachable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

    use saf_core::air::{AirBlock, AirFunction, AirGlobal, Instruction};
    use saf_core::id::make_id;
    use saf_core::ids::{BlockId, InstId, ModuleId, ObjId, ValueId};

    fn fid(name: &str) -> FunctionId {
        FunctionId(make_id("func", name.as_bytes()))
    }
    fn bid(name: &str) -> BlockId {
        BlockId(make_id("block", name.as_bytes()))
    }
    fn iid(name: &str) -> InstId {
        InstId(make_id("inst", name.as_bytes()))
    }

    fn inst(id: &str, op: Operation) -> Instruction {
        Instruction {
            id: iid(id),
            op,
            operands: Vec::new(),
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn func(name: &str, blocks: Vec<AirBlock>, is_declaration: bool) -> AirFunction {
        AirFunction {
            id: fid(name),
            name: name.to_string(),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// A defined function whose one block returns.
    fn defined(name: &str) -> AirFunction {
        let mut b = AirBlock::new(bid(&format!("{name}_entry")));
        b.instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret));
        func(name, vec![b], false)
    }

    /// A defined function whose one block calls each callee, then returns.
    fn calling(name: &str, callees: &[FunctionId]) -> AirFunction {
        let mut b = AirBlock::new(bid(&format!("{name}_entry")));
        for (i, callee) in callees.iter().enumerate() {
            b.instructions.push(inst(
                &format!("{name}_call_{i}"),
                Operation::CallDirect { callee: *callee },
            ));
        }
        b.instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret));
        func(name, vec![b], false)
    }

    /// A defined function whose block has NO terminator (frontend dropped one).
    fn unterminated(name: &str) -> AirFunction {
        let mut b = AirBlock::new(bid(&format!("{name}_entry")));
        b.instructions
            .push(inst(&format!("{name}_load"), Operation::Load));
        func(name, vec![b], false)
    }

    /// A defined function containing one indirect call.
    fn indirect(name: &str) -> AirFunction {
        let mut b = AirBlock::new(bid(&format!("{name}_icall_blk")));
        b.instructions.push(inst(
            &format!("{name}_icall"),
            Operation::CallIndirect {
                expected_signature: None,
            },
        ));
        b.instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret));
        func(name, vec![b], false)
    }

    fn declaration(name: &str) -> AirFunction {
        func(name, Vec::new(), true)
    }

    fn module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: ModuleId(make_id("module", b"test")),
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

    fn with_global(mut m: AirModule, name: &str) -> AirModule {
        m.globals.push(AirGlobal::new(
            ValueId(make_id("value", name.as_bytes())),
            ObjId(make_id("obj", name.as_bytes())),
            name,
        ));
        m
    }

    /// Admits nothing — the strictest possible policy.
    const STRICT: ExternalPolicy = ExternalPolicy {
        name: "strict",
        admits: |_| false,
    };
    /// Admits only `puts`. Notably does NOT admit any spawn primitive.
    const PUTS_ONLY: ExternalPolicy = ExternalPolicy {
        name: "puts-only",
        admits: |n| n == "puts",
    };
    /// Admits `pthread_create` — a spawn-admitting policy.
    const SPAWNS: ExternalPolicy = ExternalPolicy {
        name: "spawns",
        admits: |n| n == "pthread_create",
    };

    fn run(m: &AirModule, pol: &ExternalPolicy, roots: &ThreadRoots) -> Result<Universe, String> {
        let cg = CallGraph::build(m);
        reachable_universe(m, &cg, pol, roots)
    }

    // --- entry conditions ---------------------------------------------------

    #[test]
    fn rejects_missing_main() {
        let m = module(vec![defined("helper")]);
        assert!(run(&m, &STRICT, &ThreadRoots::MainOnly).is_err());
    }

    #[test]
    fn rejects_declaration_only_main() {
        let m = module(vec![declaration("main")]);
        assert!(run(&m, &STRICT, &ThreadRoots::MainOnly).is_err());
    }

    #[test]
    fn rejects_global_ctors() {
        // A `__attribute__((constructor))` body runs BEFORE main and is not in its
        // call graph, so the universe would silently exclude it.
        let m = with_global(module(vec![defined("main")]), "llvm.global_ctors");
        let e = run(&m, &STRICT, &ThreadRoots::MainOnly).unwrap_err();
        assert_eq!(e, "global-ctors-dtors");
    }

    #[test]
    fn rejects_global_dtors() {
        let m = with_global(module(vec![defined("main")]), "llvm.global_dtors");
        assert_eq!(
            run(&m, &STRICT, &ThreadRoots::MainOnly).unwrap_err(),
            "global-ctors-dtors"
        );
    }

    // --- THE LATENT TRAP ----------------------------------------------------

    /// RED. A main-only root set is sound ONLY if no thread can be spawned. Pairing
    /// it with a spawn-admitting policy must be an immediate, loud error — not a
    /// silently incomplete universe.
    #[test]
    fn rejects_main_only_roots_with_spawn_admitting_policy() {
        let m = module(vec![defined("main")]);
        let e = run(&m, &SPAWNS, &ThreadRoots::MainOnly).unwrap_err();
        assert!(
            e.starts_with("main-only-roots-with-spawn-admitting-policy"),
            "got: {e}"
        );
    }

    /// GUARD: the same policy with thread roots supplied is fine.
    #[test]
    fn accepts_spawn_admitting_policy_when_thread_roots_given() {
        let m = module(vec![defined("main")]);
        assert!(run(&m, &SPAWNS, &ThreadRoots::Also(BTreeSet::new())).is_ok());
    }

    /// GUARD: a non-spawn-admitting policy with main-only roots is the normal case.
    #[test]
    fn accepts_main_only_roots_with_non_spawning_policy() {
        let m = module(vec![defined("main")]);
        assert!(run(&m, &PUTS_ONLY, &ThreadRoots::MainOnly).is_ok());
    }

    // --- the reachable-body scan --------------------------------------------

    #[test]
    fn rejects_reachable_non_admitted_external() {
        let m = module(vec![
            calling("main", &[fid("getchar")]),
            declaration("getchar"),
        ]);
        let e = run(&m, &PUTS_ONLY, &ThreadRoots::MainOnly).unwrap_err();
        assert_eq!(e, "non-inert-external:getchar");
    }

    #[test]
    fn accepts_reachable_admitted_external() {
        let m = module(vec![calling("main", &[fid("puts")]), declaration("puts")]);
        assert!(run(&m, &PUTS_ONLY, &ThreadRoots::MainOnly).is_ok());
    }

    #[test]
    fn rejects_dropped_terminator_in_reachable_block() {
        let m = module(vec![calling("main", &[fid("bad")]), unterminated("bad")]);
        let e = run(&m, &STRICT, &ThreadRoots::MainOnly).unwrap_err();
        assert!(e.starts_with("dropped-terminator"), "got: {e}");
    }

    #[test]
    fn rejects_reachable_indirect_call() {
        let m = module(vec![calling("main", &[fid("ic")]), indirect("ic")]);
        let e = run(&m, &STRICT, &ThreadRoots::MainOnly).unwrap_err();
        assert!(e.starts_with("reachable-indirect-call"), "got: {e}");
    }

    // --- only REACHABLE code is judged --------------------------------------

    /// An unreachable function may contain anything: it cannot execute, so gating on
    /// it would cost recall for no soundness gain. This is the whole point of
    /// computing a universe rather than scanning the module.
    #[test]
    fn ignores_unreachable_indirect_call_and_external() {
        let m = module(vec![
            defined("main"),
            indirect("dead_ic"),
            unterminated("dead_unterm"),
            declaration("dead_extern"),
        ]);
        assert!(run(&m, &STRICT, &ThreadRoots::MainOnly).is_ok());
    }

    #[test]
    fn reachable_set_is_main_tree() {
        let m = module(vec![
            calling("main", &[fid("a")]),
            calling("a", &[fid("b")]),
            defined("b"),
            defined("unreached"),
        ]);
        let u = run(&m, &STRICT, &ThreadRoots::MainOnly).unwrap();
        assert!(u.reachable.contains(&fid("main")));
        assert!(u.reachable.contains(&fid("a")));
        assert!(u.reachable.contains(&fid("b")));
        assert!(!u.reachable.contains(&fid("unreached")));
    }

    /// A thread body is generally NOT reachable from `main` in the call graph —
    /// `pthread_create` is opaque — so its tree must be unioned in explicitly, and
    /// its contents judged by the same policy.
    #[test]
    fn thread_entry_tree_is_included_and_judged() {
        let m = module(vec![
            defined("main"),
            calling("worker", &[fid("ic")]),
            indirect("ic"),
        ]);
        let roots = ThreadRoots::Also([fid("worker")].into_iter().collect());
        let e = run(&m, &STRICT, &roots).unwrap_err();
        assert!(
            e.starts_with("reachable-indirect-call"),
            "a thread body must be scanned like main's tree; got: {e}"
        );
    }

    // --- the policy/spawn coupling is derived, not declared -------------------

    /// An empty universe must never look like a clean one.
    #[test]
    fn rejects_main_absent_from_callgraph() {
        // Build a graph from a DIFFERENT module so `main`'s node is genuinely absent.
        let m = module(vec![defined("main")]);
        let other = module(vec![defined("unrelated")]);
        let cg = CallGraph::build(&other);
        let e = reachable_universe(&m, &cg, &STRICT, &ThreadRoots::MainOnly).unwrap_err();
        assert_eq!(e, "main-not-in-callgraph");
    }

    /// `main_tree` is the main thread's surface; `reachable` adds the thread bodies.
    /// Conflating them would make a main-thread-only conclusion apply to code the
    /// main thread never runs.
    #[test]
    fn main_tree_excludes_thread_bodies_that_reachable_includes() {
        let m = module(vec![
            defined("main"),
            calling("worker", &[fid("helper")]),
            defined("helper"),
        ]);
        let roots = ThreadRoots::Also([fid("worker")].into_iter().collect());
        let u = run(&m, &STRICT, &roots).unwrap();
        assert!(u.main_tree.contains(&fid("main")));
        assert!(!u.main_tree.contains(&fid("worker")));
        assert!(!u.main_tree.contains(&fid("helper")));
        assert!(u.reachable.contains(&fid("worker")));
        assert!(u.reachable.contains(&fid("helper")));
    }

    #[test]
    fn admits_spawn_is_derived_from_the_predicate() {
        assert!(!PUTS_ONLY.admits_spawn());
        assert!(SPAWNS.admits_spawn());
        assert!(!STRICT.admits_spawn());
    }
}
