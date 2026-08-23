//! Concurrency `unreach-call` FALSE finder via atomic-thread sequentialization.
//!
//! # Mechanism (a lazy, non-preemptive sequentialization)
//!
//! Inspired by the Lazy-CSeq / CSeq family of sequentialization tools (TACAS
//! sequentialization line — Inverso, Tomasco, Fischer, La Torre, Parlato), which
//! turn a concurrent C program into a sequential one whose analysis explores thread
//! interleavings, but implemented FRESH and much more conservatively:
//!
//! Instead of encoding an interleaving into a solver, we execute the ORIGINAL
//! program natively under an explicit, non-preemptive **atomic-thread schedule** —
//! each thread body runs start-to-finish without being preempted, and threads are
//! interleaved only at `pthread_create` / `pthread_join` / thread-exit boundaries.
//! A tiny generated C driver (linked via `-Wl,--wrap=pthread_create,--wrap=pthread_join,
//! --wrap=pthread_exit`) intercepts thread creation: it records each `(start, arg)`
//! pair rather than spawning an OS thread, and later runs them one-at-a-time in a
//! chosen order. The schedule is selected by the `SAF_SCHED` environment variable so
//! the SAME binary can be replayed under several deterministic schedules.
//!
//! # Why this is SOUND for `unreach-call` FALSE
//!
//! Every atomic-thread schedule the driver produces is a *legal* sequentially-
//! consistent interleaving of the real program: running one thread to completion,
//! then another, is always an allowed schedule under SC (and SC executions are a
//! subset of the behaviours allowed by every weaker memory model, so a violation we
//! observe is real under any model). Because only one thread runs at a time and
//! nothing is ever preempted mid-body, there is no hardware store-buffer reordering
//! to worry about — the run is a faithful native execution of one real schedule. The
//! sole confirmer is the property's own violation event (`reach_error` /
//! `__assert_fail`, confirmer-contract R1), captured by the sentinel sink in the
//! driver. So a schedule that reaches the sentinel is a genuine reachable assertion
//! violation → a sound `false(unreach-call)`. A schedule that does not reach it
//! yields nothing (abstain). We can only ever miss a bug, never invent one.
//!
//! # What we DELIBERATELY abstain on (fail-closed — costs recall, never soundness)
//!
//! The non-preemptive model faithfully reproduces only schedules that need no
//! mid-body interleaving, and the single-OS-thread execution does not model every
//! POSIX primitive. [`conc_schedulable`] therefore abstains whenever a reachable
//! call could break faithfulness:
//! - any `__VERIFIER_nondet_*` (we do not fuzz thread inputs here — a forced input
//!   value could drive a nondet-guarded path the task's label does not count as
//!   reachable; keep the model deterministic-modulo-schedule);
//! - thread-local storage (`pthread_key_*` / `pthread_getspecific` / `setspecific`)
//!   — running all threads on one OS thread collapses per-thread TLS (unsound);
//! - blocking coordination that needs a *concurrent* peer to make progress
//!   (`pthread_cond_*`, `pthread_barrier_*`, `pthread_rwlock_*`, `pthread_spin_*`,
//!   `sem_*`, C11 `cnd_*`) — a non-preemptive run would deadlock (→ timeout → abstain,
//!   but we gate them out to save the budget);
//! - thread-identity / lifecycle primitives whose semantics we do not model faithfully
//!   (`pthread_self`, `pthread_equal`, `pthread_cancel`, `pthread_detach`,
//!   `pthread_once`, `pthread_kill`, C11 `tss_*`);
//! - OpenMP (`omp_*` / `GOMP_*` / `__kmpc*`) — the frontend drops `#pragma omp`, so a
//!   native run would silently mis-model it (R7 relaxed-memory / OpenMP abstain);
//! - any reachable indirect call (an unresolved target could be one of the above).
//!
//! Plain mutex use (`pthread_mutex_*`) is fine: under atomic-thread execution every
//! lock is uncontended (only one thread runs at a time), so real mutexes never block.

use std::fmt::Write as _;

use saf_analysis::callgraph::CallGraph;
use saf_core::air::{AirModule, Operation};

use crate::fast_paths::reachable_spawns_threads;

/// A non-preemptive atomic-thread schedule the generated driver can replay.
///
/// Each variant maps to a `SAF_SCHED` environment value the driver reads. The order
/// of [`CONC_SCHEDULES`] is the deterministic search order (first hit wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcSchedule {
    /// Defer every spawned thread; after `main` finishes, run them in REVERSE
    /// creation order. Exposes "the first-created thread observes the effects of all
    /// later threads" bugs (e.g. an early assert that reads a value later threads
    /// bump).
    ReverseDrain,
    /// Defer every spawned thread; after `main` finishes, run them in creation order.
    ForwardDrain,
    /// Run each thread to completion immediately at its `pthread_create` (interleaves
    /// each thread between the creating thread's own steps).
    Eager,
}

/// The schedules tried, in deterministic search order.
pub const CONC_SCHEDULES: &[ConcSchedule] = &[
    ConcSchedule::ReverseDrain,
    ConcSchedule::ForwardDrain,
    ConcSchedule::Eager,
];

impl ConcSchedule {
    /// The `SAF_SCHED` environment value the generated driver reads.
    #[must_use]
    pub fn env_value(self) -> &'static str {
        match self {
            ConcSchedule::ReverseDrain => "reverse",
            ConcSchedule::ForwardDrain => "forward",
            ConcSchedule::Eager => "eager",
        }
    }

    /// A short, stable label for diagnostics / witnesses.
    #[must_use]
    pub fn label(self) -> &'static str {
        self.env_value()
    }
}

/// Reachable calls that make the non-preemptive native model unfaithful — see the
/// module docs. A reachable direct call to any of these ⇒ abstain.
fn is_disallowed_conc_call(name: &str) -> bool {
    // No forced nondet inputs in this engine (keep it deterministic-modulo-schedule).
    if name.starts_with("__VERIFIER_nondet") {
        return true;
    }
    // Blocking coordination needing a concurrent peer, or unmodelled primitives.
    if name.starts_with("pthread_cond")
        || name.starts_with("pthread_barrier")
        || name.starts_with("pthread_rwlock")
        || name.starts_with("pthread_spin")
        || name.starts_with("pthread_key")
        || name.starts_with("sem_")
        || name.starts_with("omp_")
        || name.starts_with("GOMP_")
        || name.starts_with("__kmpc")
        || name.starts_with("cnd_")
        || name.starts_with("tss_")
    {
        return true;
    }
    // Thread-local storage / identity / lifecycle we do not model faithfully.
    matches!(
        name,
        "pthread_getspecific"
            | "pthread_setspecific"
            | "pthread_self"
            | "pthread_equal"
            | "pthread_cancel"
            | "pthread_detach"
            | "pthread_once"
            | "pthread_kill"
            | "pthread_setconcurrency"
            | "thrd_sleep"
            | "thrd_yield"
            | "thrd_current"
            | "thrd_detach"
    )
}

/// Is `module` amenable to the atomic-thread sequentialization confirmer?
///
/// Requires a thread spawn reachable from `main` (else it is not concurrent — the
/// sequential stages handle it) AND that NO defined function contains a call that
/// breaks the non-preemptive native model (see [`is_disallowed_conc_call`] and the
/// module docs). An indirect call anywhere also forces abstention (its target could
/// be a disallowed primitive).
///
/// We scan EVERY defined function, not only the callgraph-reachable set: a thread
/// entry (`thr(void*)`) is reached only through the function pointer passed to
/// `pthread_create`, which the module callgraph treats as an unresolved indirect
/// edge — so a `reachable_functions` scan would miss disallowed calls *inside thread
/// bodies* (e.g. a `__VERIFIER_nondet_int()` guard, a `pthread_cond_wait`). Scanning
/// all defined functions is a sound over-approximation: any doubt ⇒ `false` (abstain).
#[must_use]
pub fn conc_schedulable(module: &AirModule, callgraph: &CallGraph) -> bool {
    // Must actually spawn a thread reachable from main.
    if !reachable_spawns_threads(module, callgraph) {
        return false;
    }

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::CallDirect { callee } => {
                        if let Some(target) = module.function(*callee) {
                            if is_disallowed_conc_call(&target.name) {
                                return false;
                            }
                        }
                    }
                    // Unresolved target could be any disallowed primitive.
                    Operation::CallIndirect { .. } => return false,
                    _ => {}
                }
            }
        }
    }
    true
}

/// Generate the C source of the atomic-thread sequentialization driver.
///
/// Linked with the ORIGINAL program via
/// `-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit`, this driver:
/// - records each `pthread_create(start, arg)` instead of spawning (or, in `eager`
///   mode, runs it immediately);
/// - drains deferred threads in the `SAF_SCHED`-selected order after `main` returns
///   (registered via `atexit`), or on a `main`-level `pthread_exit`;
/// - models `pthread_join` by running the joined (still-pending) thread to completion;
/// - models a thread-body `pthread_exit` as a non-local return to the scheduler
///   (`setjmp`/`longjmp`), so the rest of the schedule still runs;
/// - honours `__VERIFIER_assume` as a hard path filter (R4);
/// - drops the sentinel ONLY on the property's violation event — `reach_error`,
///   `__VERIFIER_error`, or `__assert_fail` (R1) — which doubles as the witness.
///
/// `sentinel_c_literal` is embedded as a C string literal (the path the driver writes
/// `1` to on a confirmed violation); callers must pass an already-escaped literal.
#[must_use]
pub fn synthesize_conc_driver(sentinel_c_literal: &str) -> String {
    let mut s = String::new();
    s.push_str("/* SAF atomic-thread sequentialization driver (generated) */\n");
    s.push_str("#define _GNU_SOURCE\n");
    let _ = writeln!(s, "#define __SAF_SENTINEL \"{sentinel_c_literal}\"");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("#include <setjmp.h>\n");
    s.push_str("#include <pthread.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");
    s.push_str("typedef void* (*__saf_routine_t)(void*);\n");
    // Bounded thread table (sv-benchmarks spawn counts fit comfortably; overflow just
    // stops recording extra threads — a partial schedule, never a wrong verdict).
    s.push_str("#define __SAF_MAXT 100000\n");
    s.push_str("static __saf_routine_t __saf_routines[__SAF_MAXT];\n");
    s.push_str("static void* __saf_args[__SAF_MAXT];\n");
    s.push_str("static unsigned long __saf_handles[__SAF_MAXT];\n");
    s.push_str("static int __saf_n = 0;\n");
    s.push_str("static int __saf_drained = 0;\n");
    s.push_str("static unsigned long __saf_next_handle = 1;\n");
    // Per-thread setjmp target so a thread-body pthread_exit returns to the scheduler.
    s.push_str("static jmp_buf* __saf_cur_jmp = 0;\n");

    // Run one recorded thread to completion (clearing its slot first so a re-entrant
    // join cannot run it twice). A local jmp_buf makes nested joins safe.
    s.push_str(
        "static void __saf_run_one(int i) {\n\
         \x20 if (i < 0 || i >= __saf_n || !__saf_routines[i]) return;\n\
         \x20 __saf_routine_t r = __saf_routines[i]; void* a = __saf_args[i];\n\
         \x20 __saf_routines[i] = 0;\n\
         \x20 jmp_buf buf; jmp_buf* prev = __saf_cur_jmp; __saf_cur_jmp = &buf;\n\
         \x20 if (setjmp(buf) == 0) { r(a); }\n\
         \x20 __saf_cur_jmp = prev;\n\
         }\n",
    );

    // Drain deferred threads in the SAF_SCHED order (reverse | forward). Eager mode
    // never defers, so the table is empty here.
    s.push_str(
        "static void __saf_drain(void) {\n\
         \x20 if (__saf_drained) return;\n\
         \x20 __saf_drained = 1;\n\
         \x20 const char* o = getenv(\"SAF_SCHED\");\n\
         \x20 if (o && o[0] == 'r') { for (int i = __saf_n - 1; i >= 0; i--) __saf_run_one(i); }\n\
         \x20 else { for (int i = 0; i < __saf_n; i++) __saf_run_one(i); }\n\
         }\n",
    );

    s.push_str(
        "int __wrap_pthread_create(pthread_t* th, const pthread_attr_t* attr,\n\
         \x20                       __saf_routine_t start, void* arg) {\n\
         \x20 (void)attr;\n\
         \x20 const char* o = getenv(\"SAF_SCHED\");\n\
         \x20 unsigned long h = __saf_next_handle++;\n\
         \x20 if (th) *(unsigned long*)th = h;\n\
         \x20 if (o && o[0] == 'e') { /* eager: run to completion now */\n\
         \x20   jmp_buf buf; jmp_buf* prev = __saf_cur_jmp; __saf_cur_jmp = &buf;\n\
         \x20   if (setjmp(buf) == 0) { start(arg); }\n\
         \x20   __saf_cur_jmp = prev;\n\
         \x20   return 0;\n\
         \x20 }\n\
         \x20 if (__saf_n < __SAF_MAXT) {\n\
         \x20   __saf_routines[__saf_n] = start; __saf_args[__saf_n] = arg;\n\
         \x20   __saf_handles[__saf_n] = h; __saf_n++;\n\
         \x20   if (__saf_n == 1) atexit(__saf_drain);\n\
         \x20 }\n\
         \x20 return 0;\n\
         }\n",
    );

    s.push_str(
        "int __wrap_pthread_join(pthread_t th, void** ret) {\n\
         \x20 (void)ret;\n\
         \x20 unsigned long h = (unsigned long)th;\n\
         \x20 for (int i = 0; i < __saf_n; i++)\n\
         \x20   if (__saf_routines[i] && __saf_handles[i] == h) { __saf_run_one(i); return 0; }\n\
         \x20 return 0;\n\
         }\n",
    );

    // Thread-body pthread_exit -> non-local return to the scheduler; a main-level
    // pthread_exit drains the deferred threads then exits (mirrors POSIX: the process
    // stays alive for the other threads).
    s.push_str(
        "void __wrap_pthread_exit(void* ret) {\n\
         \x20 (void)ret;\n\
         \x20 if (__saf_cur_jmp) longjmp(*__saf_cur_jmp, 1);\n\
         \x20 __saf_drain();\n\
         \x20 _exit(0);\n\
         }\n",
    );

    // R4: assumptions are hard path filters.
    s.push_str("void __VERIFIER_assume(int c) { if (!c) _exit(0); }\n");
    // Atomic sections are already serialized under atomic-thread execution.
    s.push_str("void __VERIFIER_atomic_begin(void) { }\n");
    s.push_str("void __VERIFIER_atomic_end(void) { }\n");

    // R1: the sentinel drops ONLY on the property's violation event.
    s.push_str(
        "__attribute__((noreturn)) static void __saf_hit(void) { FILE* f = fopen(__SAF_SENTINEL, \"w\"); if (f) { fputc('1', f); fclose(f); } _exit(0); }\n",
    );
    s.push_str("__attribute__((weak)) void reach_error(void) { __saf_hit(); }\n");
    s.push_str("__attribute__((weak)) void __VERIFIER_error(void) { __saf_hit(); }\n");
    s.push_str(
        "__attribute__((noreturn)) void __assert_fail(const char* a, const char* b, unsigned int c, const char* d) { (void)a; (void)b; (void)c; (void)d; __saf_hit(); }\n",
    );

    s
}

// ---------------------------------------------------------------------------
// GraphML 1.0 violation witness (R7: concurrency FALSE needs GraphML, not YAML)
// ---------------------------------------------------------------------------

/// Fixed, deterministic creation timestamp (determinism forbids the wall clock).
const CONC_WITNESS_CREATION_TIME: &str = "1970-01-01T00:00:00Z";

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Build a GraphML 1.0 violation witness for a confirmed concurrency `unreach-call`
/// FALSE (confirmer-contract R7 — a YAML-2.0 witness scores 0 for concurrency).
///
/// Emits the mandatory graph metadata plus, for the winning schedule, a `main` entry
/// node, one `createThread` edge per spawned thread (bounded to `thread_count`, capped
/// so a 10^4-thread task stays small), and a `threadId`-annotated edge into the
/// `violation` node. The `schedule` label records which non-preemptive order reached
/// the violation. Fully deterministic given its inputs. Targeted at CPAchecker /
/// Dartagnan / `ConcurrentWitness2Test`.
#[must_use]
pub fn conc_graphml_witness(
    spec: &str,
    programfile: &str,
    programhash: &str,
    architecture: &str,
    thread_count: usize,
    schedule: ConcSchedule,
) -> String {
    // Cap the number of modelled threads in the witness so a 10^4-spawn task does not
    // produce a pathologically large witness; one createThread edge is enough to make
    // the witness structurally valid.
    let modelled = thread_count.clamp(1, 16);

    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n");
    s.push_str(
        "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" \
xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n",
    );
    for (id, ty) in [
        ("witness-type", "string"),
        ("sourcecodelang", "string"),
        ("producer", "string"),
        ("specification", "string"),
        ("programfile", "string"),
        ("programhash", "string"),
        ("architecture", "string"),
        ("creationtime", "string"),
    ] {
        let _ = writeln!(
            s,
            "  <key attr.name=\"{id}\" attr.type=\"{ty}\" for=\"graph\" id=\"{id}\"/>"
        );
    }
    s.push_str(
        "  <key attr.name=\"entry\" attr.type=\"boolean\" for=\"node\" id=\"entry\">\
<default>false</default></key>\n",
    );
    s.push_str(
        "  <key attr.name=\"violation\" attr.type=\"boolean\" for=\"node\" id=\"violation\">\
<default>false</default></key>\n",
    );
    s.push_str(
        "  <key attr.name=\"threadId\" attr.type=\"string\" for=\"edge\" id=\"threadId\"/>\n",
    );
    s.push_str(
        "  <key attr.name=\"createThread\" attr.type=\"string\" for=\"edge\" id=\"createThread\"/>\n",
    );

    s.push_str("  <graph edgedefault=\"directed\">\n");
    for (k, v) in [
        ("witness-type", "violation_witness"),
        ("sourcecodelang", "C"),
        ("producer", "SAF"),
        ("specification", spec),
        ("programfile", programfile),
        ("programhash", programhash),
        ("architecture", architecture),
        ("creationtime", CONC_WITNESS_CREATION_TIME),
    ] {
        let _ = writeln!(s, "    <data key=\"{k}\">{}</data>", xml_escape(v));
    }
    // Entry node (thread 0 = main).
    s.push_str("    <node id=\"N0\"><data key=\"entry\">true</data></node>\n");
    // One node + createThread edge per modelled spawned thread.
    for t in 1..=modelled {
        let _ = writeln!(s, "    <node id=\"N{t}\"/>");
        let _ = writeln!(
            s,
            "    <edge source=\"N0\" target=\"N{t}\"><data key=\"createThread\">{}</data></edge>",
            t - 1
        );
    }
    // Violation node reached by the last modelled thread under `schedule`.
    let vnode = modelled + 1;
    let _ = writeln!(
        s,
        "    <node id=\"N{vnode}\"><data key=\"violation\">true</data></node>"
    );
    let _ = writeln!(
        s,
        "    <edge source=\"N{modelled}\" target=\"N{vnode}\"><data key=\"threadId\">{}</data></edge>",
        xml_escape(schedule.label())
    );
    s.push_str("  </graph>\n");
    s.push_str("</graphml>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId};

    /// A defined function whose body is a single block calling each `calls` id.
    fn func(id: u128, name: &str, calls: &[FunctionId]) -> AirFunction {
        let mut block = AirBlock::new(BlockId::new(1000 + id));
        for (i, callee) in calls.iter().enumerate() {
            block.instructions.push(Instruction {
                id: InstId::new(10_000 + id * 100 + i as u128),
                op: Operation::CallDirect { callee: *callee },
                operands: vec![],
                dst: None,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            });
        }
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: vec![],
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// An extern declaration (no body) — a call target.
    fn decl(id: u128, name: &str) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// main -> pthread_create(...) with only whitelisted primitives ⇒ schedulable.
    #[test]
    fn schedulable_on_plain_spawn_and_mutex() {
        let create = decl(1, "pthread_create");
        let join = decl(2, "pthread_join");
        let lock = decl(3, "pthread_mutex_lock");
        let main = func(4, "main", &[create.id, join.id, lock.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(join);
        m.functions.push(lock);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(conc_schedulable(&m, &cg));
    }

    /// A reachable nondet call ⇒ abstain (keeps the engine deterministic-modulo-schedule).
    #[test]
    fn abstains_on_reachable_nondet() {
        let create = decl(1, "pthread_create");
        let nondet = decl(2, "__VERIFIER_nondet_int");
        let main = func(3, "main", &[create.id, nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(nondet);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_schedulable(&m, &cg));
    }

    /// A disallowed call inside a THREAD BODY (not reachable via the module callgraph,
    /// since the entry is only taken as a `pthread_create` function pointer) must still
    /// force abstention — the scan covers every defined function, not just the
    /// callgraph-reachable set.
    #[test]
    fn abstains_on_nondet_in_thread_body() {
        let create = decl(1, "pthread_create");
        let nondet = decl(2, "__VERIFIER_nondet_int");
        // `main` only spawns; the nondet call lives in `thr`, reached via a function
        // pointer the callgraph does not resolve.
        let main = func(3, "main", &[create.id]);
        let thr = func(4, "thr", &[nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(nondet);
        m.functions.push(main);
        m.functions.push(thr);
        let cg = CallGraph::build(&m);
        assert!(!conc_schedulable(&m, &cg));
    }

    /// TLS ⇒ abstain (single-OS-thread execution collapses per-thread storage).
    #[test]
    fn abstains_on_tls() {
        let create = decl(1, "pthread_create");
        let tls = decl(2, "pthread_getspecific");
        let main = func(3, "main", &[create.id, tls.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(tls);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_schedulable(&m, &cg));
    }

    /// A blocking condvar ⇒ abstain (would deadlock under non-preemptive execution).
    #[test]
    fn abstains_on_condvar() {
        let create = decl(1, "pthread_create");
        let cw = decl(2, "pthread_cond_wait");
        let main = func(3, "main", &[create.id, cw.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(cw);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_schedulable(&m, &cg));
    }

    /// No reachable spawn ⇒ not our job (sequential stages handle it).
    #[test]
    fn abstains_when_no_spawn() {
        let main = func(1, "main", &[]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_schedulable(&m, &cg));
    }

    #[test]
    fn driver_has_wrappers_sentinel_and_assume() {
        let d = synthesize_conc_driver("/tmp/sent");
        assert!(d.contains("__wrap_pthread_create"));
        assert!(d.contains("__wrap_pthread_join"));
        assert!(d.contains("__wrap_pthread_exit"));
        assert!(d.contains("__assert_fail"));
        assert!(d.contains("reach_error"));
        assert!(d.contains("__VERIFIER_assume"));
        assert!(d.contains("\"/tmp/sent\""));
        // Both drain directions present.
        assert!(d.contains("__saf_n - 1"));
    }

    #[test]
    fn schedules_have_distinct_env_values() {
        let vals: BTreeSet<_> = CONC_SCHEDULES.iter().map(|s| s.env_value()).collect();
        assert_eq!(vals.len(), CONC_SCHEDULES.len());
    }

    #[test]
    fn graphml_witness_is_well_formed() {
        let w = conc_graphml_witness(
            "CHECK( init(main()), LTL(G ! call(reach_error())) )",
            "p.i",
            "hash",
            "32bit",
            3,
            ConcSchedule::ReverseDrain,
        );
        assert!(w.starts_with("<?xml"));
        assert!(w.contains("violation_witness"));
        assert!(w.contains("createThread"));
        assert!(w.contains("<data key=\"violation\">true</data>"));
        assert!(w.contains("reverse"));
        assert!(w.trim_end().ends_with("</graphml>"));
    }

    #[test]
    fn graphml_witness_escapes_and_caps_threads() {
        let w = conc_graphml_witness(
            "a & b < c",
            "p.i",
            "h",
            "32bit",
            10_000,
            ConcSchedule::Eager,
        );
        assert!(w.contains("a &amp; b &lt; c"));
        // capped at 16 modelled threads.
        assert!(w.contains("N16"));
        assert!(!w.contains("N9999"));
    }
}
