//! Nondet-aware concurrency `unreach-call` FALSE confirmer: atomic-thread
//! sequentialization **combined with byte-stream nondet-input fuzzing**.
//!
//! # Why this exists (the wall the other three concurrency engines hit)
//!
//! [`crate::conc_seq`], [`crate::conc_replay`], and [`crate::conc_shim`] all share the
//! [`crate::conc_seq::conc_schedulable`] gate, which **abstains on any reachable
//! `__VERIFIER_nondet_*`** ("keep the model deterministic-modulo-schedule"). But a
//! large fraction of the sv-benchmarks concurrency `unreach-call` FALSE tasks read a
//! nondet input and only reach `reach_error` for a *specific* input value — e.g. the
//! goblint `13-privatized_*` / `36-apron_*` family:
//!
//! ```c
//! void *t_fun(void *arg) {
//!   int t = __VERIFIER_nondet_int();
//!   pthread_mutex_lock(&mutex1);
//!   if (t == 42) glob1 = 1;
//!   t = glob1;
//!   __VERIFIER_assert(t == 0);   // reach_error iff t != 0, i.e. iff the input was 42
//!   ...
//! }
//! ```
//!
//! The bug fires under a perfectly ordinary schedule — it just needs the input `42`.
//! Because the shared gate refuses the whole program the moment it sees the nondet
//! call, none of the three engines ever runs it. This module lifts exactly that
//! restriction: it drives the `__VERIFIER_nondet_*` inputs from a fuzzed byte stream
//! (the SAME in-range byte-stream shim the sequential blind fuzzer uses,
//! [`crate::fuzz`]) while replaying the program under the non-preemptive atomic-thread
//! schedules of [`crate::conc_seq`]. A schedule × input pair that reaches
//! `reach_error` is a genuine reachable violation.
//!
//! # Mechanism (fresh; independent of SVF/CSeq — REQ-IP-001)
//!
//! Inspired by the sequentialization line (Lazy-CSeq / CSeq — Inverso, Tomasco,
//! Fischer, La Torre, Parlato) where the *inputs* of a concurrent program are treated
//! as nondeterministic choices the search assigns alongside the schedule, but
//! implemented conservatively: rather than encode the whole program into a solver, we
//! (a) generate a single native harness that links the ORIGINAL program with a tiny
//! driver providing the atomic-thread scheduler (the [`crate::conc_seq`] model) plus
//! the byte-stream nondet shim, then (b) search over `(input byte stream, schedule)`
//! pairs natively. The schedule is chosen by `SAF_SCHED`; the nondet inputs are read
//! from the `SAF_FUZZ_INPUT` byte file. The caller drives the search (dictionary-
//! seeded byte streams × the three schedules) in `saf-cli`; this crate stays
//! subprocess-free.
//!
//! # Why this is SOUND for `unreach-call` FALSE (soundness is the whole game)
//!
//! - Each atomic-thread schedule is a **legal sequentially-consistent interleaving**
//!   of the real program (one thread runs to completion, then another) — identical to
//!   [`crate::conc_seq`]'s soundness argument.
//! - Each `__VERIFIER_nondet_T()` returns `sizeof(T)` bytes of the input stream
//!   reinterpreted as `T`; every bit pattern of an integer type is an in-range value
//!   of that type (two's-complement x86, R5). So the run is a faithful native
//!   execution of ONE real (schedule, input) pair of the original program.
//! - The SOLE confirmer is the property's own violation event (`reach_error` /
//!   `__VERIFIER_error` / `__assert_fail`, R1), captured by the sentinel sink;
//!   `__VERIFIER_assume` is honoured as a hard path filter (R4). A run that drops the
//!   sentinel is a genuine reachable assertion violation ⇒ a sound
//!   `false(unreach-call)`. The input file *is* the concrete value vector, so re-running
//!   the same `(input, schedule)` reproduces it deterministically (R6). A schedule ×
//!   input that does not reach the sentinel yields nothing (abstain). We can only ever
//!   miss a bug, never invent one.
//! - Allocation-failure pruning ([`crate::fuzz::ALLOC_PRUNE_WRAP_C`]) models SV-COMP's
//!   unbounded-memory `malloc`, so a nondet-driven huge allocation never fabricates a
//!   wrong FALSE (the aws-c-common soundness sentinel).
//!
//! # What we DELIBERATELY gate out (fail-closed — costs recall, never soundness)
//!
//! [`conc_fuzz_schedulable`] requires everything [`crate::conc_seq::conc_schedulable`]
//! does EXCEPT the blanket nondet abstain — reachable spawn; no TLS, condvars,
//! barriers, rwlocks, spinlocks, semaphores, OpenMP, indirect calls, or unmodelled
//! lifecycle primitives — AND additionally:
//! - the program MUST reference a *fuzzable* `__VERIFIER_nondet_*` (else the three
//!   nondet-free engines already cover it — no added value);
//! - EVERY reachable `__VERIFIER_nondet_*` must be fuzzable ([`crate::fuzz::is_fuzzable_nondet`]);
//!   an exotic / undocumented-width nondet ⇒ abstain (R5, do not guess a width).
//!
//! Plain mutexes are fine (uncontended under atomic-thread execution, exactly as in
//! [`crate::conc_seq`]); `__VERIFIER_atomic` sections are already serialized.

use std::fmt::Write as _;

use saf_analysis::callgraph::CallGraph;
use saf_core::air::{AirModule, Operation};

use crate::fast_paths::reachable_spawns_threads;
use crate::fuzz::{self, is_fuzzable_nondet, references_fuzzable_nondet};

/// Reachable calls that make the non-preemptive native model unfaithful — the same
/// set [`crate::conc_seq`] gates on, MINUS the `__VERIFIER_nondet_*` blanket (this
/// engine drives nondet from a fuzzed byte stream instead of abstaining). A reachable
/// direct call to any of these ⇒ abstain.
fn is_disallowed_conc_fuzz_call(name: &str) -> bool {
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

/// Is `module` amenable to the nondet-aware atomic-thread sequentialization confirmer?
///
/// Requires a thread spawn reachable from `main`, a reference to a *fuzzable* nondet
/// input (else the nondet-free engines already cover it), that NO defined function
/// contains a call breaking the non-preemptive native model (see
/// [`is_disallowed_conc_fuzz_call`]) or an indirect call, and that EVERY reachable
/// `__VERIFIER_nondet_*` is fuzzable (an exotic width ⇒ abstain, R5). We scan EVERY
/// defined function (not only the callgraph-reachable set) because a thread entry is
/// reached only through the `pthread_create` function pointer — an unresolved indirect
/// edge — so a reachable-only scan would miss primitives inside thread bodies. Any
/// doubt ⇒ `false` (abstain).
#[must_use]
pub fn conc_fuzz_schedulable(module: &AirModule, callgraph: &CallGraph) -> bool {
    if !reachable_spawns_threads(module, callgraph) {
        return false;
    }
    // Nothing to fuzz (and no added value over the nondet-free engines) without a
    // fuzzable nondet input.
    if !references_fuzzable_nondet(module) {
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
                            let name = target.name.as_str();
                            // An exotic (non-fuzzable) nondet ⇒ abstain: we cannot pin
                            // its width/signedness soundly (R5).
                            if name.starts_with("__VERIFIER_nondet") && !is_fuzzable_nondet(name) {
                                return false;
                            }
                            if is_disallowed_conc_fuzz_call(name) {
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

/// Generate the C source of the nondet-aware atomic-thread sequentialization driver.
///
/// Linked with the ORIGINAL program via
/// `-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit`, this driver
/// combines:
/// - the atomic-thread scheduler of [`crate::conc_seq`] (records each
///   `pthread_create(start, arg)` instead of spawning; drains deferred threads in the
///   `SAF_SCHED`-selected order after `main`; models `pthread_join` by running the
///   joined thread; a thread-body `pthread_exit` is a non-local return to the
///   scheduler via `setjmp`/`longjmp`);
/// - the byte-stream nondet shim of [`crate::fuzz`] ([`crate::fuzz::bytestream_nondet_shim_c`]):
///   each `__VERIFIER_nondet_T()` consumes `sizeof(T)` bytes from `$SAF_FUZZ_INPUT`
///   (in-range, R5) and logs the value to `$SAF_FUZZ_LOG`;
/// - allocation-failure pruning ([`crate::fuzz::ALLOC_PRUNE_WRAP_C`], R6 soundness);
/// - `__VERIFIER_assume` as a hard path filter (R4);
/// - the sentinel dropped ONLY on the property's violation event — `reach_error`,
///   `__VERIFIER_error`, or `__assert_fail` (R1) — which doubles as the witness.
///
/// `sentinel_c_literal` is embedded as a C string literal (the path the driver writes
/// `1` to on a confirmed violation); callers must pass an already-escaped literal.
// NOTE: this emits one self-contained C translation unit (scheduler + nondet shim +
// sinks); splitting it across helpers would only scatter the driver source.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn synthesize_conc_fuzz_driver(sentinel_c_literal: &str) -> String {
    let mut s = String::new();
    s.push_str("/* SAF nondet-aware atomic-thread sequentialization driver (generated) */\n");
    s.push_str("#define _GNU_SOURCE\n");
    let _ = writeln!(s, "#define __SAF_SENTINEL \"{sentinel_c_literal}\"");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("#include <string.h>\n");
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
    s.push_str("static jmp_buf* __saf_cur_jmp = 0;\n");

    // --- byte-stream nondet input core (shared with the sequential blind fuzzer) ---
    s.push_str(&fuzz::bytestream_nondet_shim_c());

    // --- atomic-thread scheduler (the conc_seq model) ---
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

    // R1: the sentinel drops ONLY on the property's violation event. Flush the nondet
    // log first so a discovered value sequence is complete for the witness / re-confirm.
    s.push_str(
        "__attribute__((noreturn)) static void __saf_hit(void) { if (__saf_log) fflush(__saf_log); FILE* f = fopen(__SAF_SENTINEL, \"w\"); if (f) { fputc('1', f); fclose(f); } _exit(0); }\n",
    );
    s.push_str("__attribute__((weak)) void reach_error(void) { __saf_hit(); }\n");
    s.push_str("__attribute__((weak)) void __VERIFIER_error(void) { __saf_hit(); }\n");
    s.push_str(
        "__attribute__((noreturn)) void __assert_fail(const char* a, const char* b, unsigned int c, const char* d) { (void)a; (void)b; (void)c; (void)d; __saf_hit(); }\n",
    );

    // Allocation-failure PRUNE (soundness sentinel: aws-c-common).
    s.push_str(fuzz::ALLOC_PRUNE_WRAP_C);

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

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

    /// main -> pthread_create; a thread body reads a fuzzable nondet + a plain mutex ⇒
    /// conc-fuzz-schedulable (the class the three nondet-FREE engines all gate out).
    #[test]
    fn schedulable_on_spawn_with_nondet_and_mutex() {
        let create = decl(1, "pthread_create");
        let join = decl(2, "pthread_join");
        let lock = decl(3, "pthread_mutex_lock");
        let nondet = decl(4, "__VERIFIER_nondet_int");
        let main = func(5, "main", &[create.id, join.id]);
        let thr = func(6, "thr", &[lock.id, nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, join, lock, nondet, main, thr] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(conc_fuzz_schedulable(&m, &cg));
    }

    /// A spawn with NO fuzzable nondet ⇒ abstain (the nondet-free engines cover it,
    /// no added value here).
    #[test]
    fn abstains_without_nondet() {
        let create = decl(1, "pthread_create");
        let main = func(2, "main", &[create.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_fuzz_schedulable(&m, &cg));
    }

    /// A reachable nondet inside a THREAD BODY makes the program schedulable HERE
    /// (unlike the nondet-free engines, which abstain on it). The scan must reach into
    /// thread bodies (found via the create fn-pointer, an unresolved indirect edge).
    #[test]
    fn nondet_in_thread_body_is_schedulable_here() {
        let create = decl(1, "pthread_create");
        let nondet = decl(2, "__VERIFIER_nondet_int");
        let main = func(3, "main", &[create.id]);
        let thr = func(4, "thr", &[nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, nondet, main, thr] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(conc_fuzz_schedulable(&m, &cg));
    }

    /// A condvar ⇒ abstain even with nondet present (would deadlock non-preemptively).
    #[test]
    fn abstains_on_condvar() {
        let create = decl(1, "pthread_create");
        let nondet = decl(2, "__VERIFIER_nondet_int");
        let cw = decl(3, "pthread_cond_wait");
        let main = func(4, "main", &[create.id, nondet.id, cw.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, nondet, cw, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_fuzz_schedulable(&m, &cg));
    }

    /// TLS ⇒ abstain even with nondet present.
    #[test]
    fn abstains_on_tls() {
        let create = decl(1, "pthread_create");
        let nondet = decl(2, "__VERIFIER_nondet_int");
        let tls = decl(3, "pthread_getspecific");
        let main = func(4, "main", &[create.id, nondet.id, tls.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, nondet, tls, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_fuzz_schedulable(&m, &cg));
    }

    /// An exotic / non-fuzzable `__VERIFIER_nondet_*` ⇒ abstain (cannot pin its width
    /// soundly, R5) — but note it also fails `references_fuzzable_nondet`, so the gate
    /// is doubly closed.
    #[test]
    fn abstains_on_exotic_nondet_only() {
        let create = decl(1, "pthread_create");
        let exotic = decl(2, "__VERIFIER_nondet_charp");
        let main = func(3, "main", &[create.id, exotic.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, exotic, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_fuzz_schedulable(&m, &cg));
    }

    /// A fuzzable nondet alongside an exotic one ⇒ abstain (the exotic one is
    /// un-pinnable even though a fuzzable input exists).
    #[test]
    fn abstains_when_any_nondet_is_exotic() {
        let create = decl(1, "pthread_create");
        let good = decl(2, "__VERIFIER_nondet_int");
        let exotic = decl(3, "__VERIFIER_nondet_charp");
        let main = func(4, "main", &[create.id]);
        let thr = func(5, "thr", &[good.id, exotic.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, good, exotic, main, thr] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_fuzz_schedulable(&m, &cg));
    }

    /// No reachable spawn ⇒ not our job (the sequential fuzzer handles it).
    #[test]
    fn abstains_without_spawn() {
        let nondet = decl(1, "__VERIFIER_nondet_int");
        let main = func(2, "main", &[nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [nondet, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_fuzz_schedulable(&m, &cg));
    }

    #[test]
    fn driver_has_scheduler_nondet_shim_sentinel_and_assume() {
        let d = synthesize_conc_fuzz_driver("/tmp/sent");
        // pthread scheduler wrappers.
        assert!(d.contains("__wrap_pthread_create"));
        assert!(d.contains("__wrap_pthread_join"));
        assert!(d.contains("__wrap_pthread_exit"));
        assert!(d.contains("SAF_SCHED"));
        // byte-stream nondet shim.
        assert!(d.contains("__saf_take"));
        assert!(d.contains("SAF_FUZZ_INPUT"));
        assert!(d.contains("int __VERIFIER_nondet_int(void)"));
        // property events + assume + sentinel path.
        assert!(d.contains("__assert_fail"));
        assert!(d.contains("reach_error"));
        assert!(d.contains("__VERIFIER_assume"));
        assert!(d.contains("\"/tmp/sent\""));
        // allocation-failure prune (soundness).
        assert!(d.contains("__wrap_malloc"));
    }

    /// Determinism (NFR-DET): identical inputs ⇒ byte-identical driver.
    #[test]
    fn driver_is_deterministic() {
        assert_eq!(
            synthesize_conc_fuzz_driver("/tmp/s"),
            synthesize_conc_fuzz_driver("/tmp/s")
        );
    }
}
