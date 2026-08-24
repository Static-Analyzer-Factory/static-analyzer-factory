//! Concurrency `unreach-call` FALSE confirmer via a **systematic bounded-preemption**
//! cooperative scheduler that models pthread mutexes and preempts at shared accesses.
//!
//! # Why this exists (what [`crate::conc_seq`] and [`crate::conc_replay`] cannot do)
//!
//! - [`crate::conc_seq`] (lever `conc-seq-m1`) runs each spawned thread *start-to-finish*;
//!   it only explores interleavings whose switches land on `pthread_create` / `join` /
//!   thread-exit. A bug that needs a switch *mid-body* is invisible to it.
//! - [`crate::conc_replay`] (lever `conc-replay-confirm`) does force mid-body switches, but
//!   ONLY at `__VERIFIER_atomic` boundaries, and it *abstains on any mutex* (its single-token
//!   model would deadlock on a real lock). The large mutex-guarded class of sv-benchmarks —
//!   e.g. goblint `13-privatized_*`, where `t_fun` briefly publishes `glob1=5` between two
//!   lockings of `mutex2` and `main`'s `assert(glob1==0)` fires only if it grabs `mutex2` in
//!   that window — is reached by neither engine.
//!
//! This module reaches that class by (a) **modelling mutexes inside the scheduler** so a
//! contended lock parks the thread instead of dead-locking the one OS token-holder, and
//! (b) inserting scheduling points at **shared memory accesses** (via `SanitizerCoverage`
//! load/store tracing) in addition to the lock/unlock boundaries.
//!
//! # Mechanism (fresh; independent of SVF/CSeq — REQ-IP-001)
//!
//! Inspired by CHESS's **iterative context bounding** (Musuvathi–Qadeer, *Iterative Context
//! Bounding for Systematic Testing of Multithreaded Programs*, PLDI 2007) — most concurrency
//! bugs surface with very few preemptions, so enumerate schedules with a small preemption
//! bound `c` first (`c<=1`, then `c<=2`) — but implemented from scratch:
//!
//! A tiny generated C driver is linked with the ORIGINAL program via `-Wl,--wrap=` on
//! `pthread_create` / `join` / `exit` / `mutex_lock` / `mutex_unlock` / `mutex_trylock`.
//! Threads are REAL OS threads, but a single global token (`__saf_cur`) makes execution
//! strictly serial: only the token-holder runs. Execution is DETERMINISTIC and driven by two
//! environment integers `SAF_SHIM_P1`, `SAF_SHIM_P2` — the *global yield-point indices at
//! which the running thread is forcibly preempted* (round-robin) to another runnable thread.
//! Everywhere else the running thread keeps the token (no preemption), so a run performs at
//! most `c` (∈ {1,2}) preemptions — exactly a bounded-preemption schedule. The confirmer
//! sweeps `SAF_SHIM_P{1,2}` over a small index range (`c=1` schedules first, then `c=2`).
//!
//! Mutexes are modelled in the scheduler: `mutex_lock` on a lock held by another thread parks
//! the caller (`ST_BLOCKED_LOCK`) until an `unlock` wakes it; the real program mutex object is
//! never actually locked. `mutex_lock` / `unlock` are also scheduling (yield) points; a
//! blocked/contended handoff is *forced* and does not count against the preemption bound `c`.
//!
//! # Why this is SOUND for `unreach-call` FALSE (soundness is the whole game)
//!
//! Exactly one thread holds the token at a time and switches happen only at the driver's
//! scheduling points, so every replayed run is a **real, serialized, sequentially-consistent
//! execution of one legal interleaving** of the original program, executed natively — with
//! mutual exclusion faithfully enforced (a thread cannot proceed past a lock another thread
//! holds). The SOLE confirmer is the property's own violation event (`reach_error` /
//! `__VERIFIER_error` / `__assert_fail`, confirmer-contract R1), captured by the sentinel
//! sink; `__VERIFIER_assume` is honoured as a hard path filter (R4). A schedule that drops the
//! sentinel is therefore a genuine reachable assertion violation ⇒ a sound `false(unreach-call)`;
//! a schedule that does not yields nothing. Mis-modelling can only ever *lose* a schedule
//! (deadlock → the driver exits with NO sentinel → abstain), never invent a violation. The
//! shared-access scheduling filter (skip thread-local stack addresses) only changes *which*
//! points are preemptable, so it affects recall, never soundness.
//!
//! # What we DELIBERATELY gate out (fail-closed — costs recall, never soundness)
//!
//! [`conc_shim_schedulable`] requires everything [`crate::conc_schedulable`] does (reachable
//! spawn; no nondet, TLS, condvars, barriers, rwlocks, spinlocks, semaphores, OpenMP, or
//! indirect calls) AND additionally abstains when the program:
//! - uses `__VERIFIER_atomic` — that is [`crate::conc_replay`]'s domain, and splitting an
//!   atomic section at a shared-access yield would be unsound; leave it to the engine that
//!   holds the token across atomic sections;
//! - configures a non-default mutex type (`pthread_mutexattr_settype` — recursive/errorcheck
//!   locks the `PTHREAD_MUTEX_NORMAL` model does not represent). As a defence in depth the
//!   driver also suppresses yields inside any `__VERIFIER_atomic` section it is ever handed.

use std::fmt::Write as _;

use saf_analysis::callgraph::CallGraph;
use saf_core::air::{AirModule, Operation};

use crate::conc_seq::conc_schedulable;

/// Highest global yield-point index the confirmer forces a preemption at. Sized so the small
/// mutex-guarded sv-benchmarks bugs (whose winning schedules preempt within the first handful
/// of lock/unlock/shared-access points) are reachable while the `c=2` pair sweep stays cheap.
pub const SHIM_MAX_STEP: i64 = 24;

/// Maximum number of preemptions per schedule (CHESS context bound). We sweep `c<=1` then
/// `c<=2`; two preemptions suffice for the classic "spawn, let a worker publish, then the
/// reader grabs the lock in the gap" bugs this engine targets.
pub const SHIM_CBOUND: u32 = 2;

/// A single bounded-preemption schedule: the (up to two) global yield-point indices at which
/// the running thread is forcibly preempted. `p2 < 0` means a `c=1` (single-preemption)
/// schedule. Maps to the `SAF_SHIM_P1` / `SAF_SHIM_P2` environment integers the driver reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShimPlan {
    /// First (or only) preemption index.
    pub p1: i64,
    /// Second preemption index, or `-1` for a `c=1` schedule.
    pub p2: i64,
}

impl ShimPlan {
    /// The `SAF_SHIM_P1` value the driver reads.
    #[must_use]
    pub fn p1_env(self) -> String {
        self.p1.to_string()
    }

    /// The `SAF_SHIM_P2` value the driver reads (`-1` for a single-preemption schedule).
    #[must_use]
    pub fn p2_env(self) -> String {
        self.p2.to_string()
    }

    /// A short, stable label for diagnostics / the GraphML witness.
    #[must_use]
    pub fn label(self) -> String {
        if self.p2 < 0 {
            format!("shim:c1:{}", self.p1)
        } else {
            format!("shim:c2:{},{}", self.p1, self.p2)
        }
    }
}

/// The bounded-preemption schedules tried, in deterministic search order: every `c=1` schedule
/// (`p1 = 0..=SHIM_MAX_STEP`) first, then every `c=2` pair (`p1 < p2 <= SHIM_MAX_STEP`) ordered
/// by `p1` then `p2` — small (early) preemptions first, which is where these bugs live.
#[must_use]
pub fn shim_preemption_plans() -> Vec<ShimPlan> {
    let mut v = Vec::new();
    for p1 in 0..=SHIM_MAX_STEP {
        v.push(ShimPlan { p1, p2: -1 });
    }
    if SHIM_CBOUND >= 2 {
        for p1 in 0..=SHIM_MAX_STEP {
            for p2 in (p1 + 1)..=SHIM_MAX_STEP {
                v.push(ShimPlan { p1, p2 });
            }
        }
    }
    v
}

/// Is there a direct call, in any defined function, to a callee whose name satisfies `pred`?
/// (Scanned across EVERY defined function, not just the callgraph-reachable set: a thread entry
/// is reached only through the `pthread_create` function pointer — an unresolved indirect edge —
/// so a reachable-only scan would miss primitives used inside thread bodies.)
fn module_has_direct_call_to(module: &AirModule, pred: impl Fn(&str) -> bool) -> bool {
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if pred(&target.name) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Does the program use `__VERIFIER_atomic` (mid-body atomic sections)?
fn uses_verifier_atomic(module: &AirModule) -> bool {
    module_has_direct_call_to(module, |name| name == "__VERIFIER_atomic_begin")
}

/// Does the program configure a non-default mutex type (recursive / errorcheck)? The
/// `PTHREAD_MUTEX_NORMAL` model (self-relock deadlocks) cannot represent those.
fn uses_mutex_attr(module: &AirModule) -> bool {
    module_has_direct_call_to(module, |name| name == "pthread_mutexattr_settype")
}

/// Is `module` amenable to the bounded-preemption mutex-aware replay confirmer?
///
/// Requires everything [`conc_schedulable`] does (reachable spawn; only faithfully modelled
/// primitives — mutexes ARE allowed here, unlike [`crate::conc_replay`]) AND additionally that
/// the program uses NO `__VERIFIER_atomic` (left to [`crate::conc_replay`], which holds the
/// token across atomic sections) and configures NO non-default mutex type. See the module docs
/// for the soundness rationale. Any doubt ⇒ abstain.
#[must_use]
pub fn conc_shim_schedulable(module: &AirModule, callgraph: &CallGraph) -> bool {
    conc_schedulable(module, callgraph) && !uses_verifier_atomic(module) && !uses_mutex_attr(module)
}

/// Generate the C source of the systematic bounded-preemption replay driver.
///
/// Linked with the ORIGINAL program via `-Wl,--wrap=pthread_create,--wrap=pthread_join,
/// --wrap=pthread_exit,--wrap=pthread_mutex_lock,--wrap=pthread_mutex_unlock,
/// --wrap=pthread_mutex_trylock`. The driver:
/// - makes execution strictly serial via a global token, so every run is a real SC interleaving;
/// - preempts the running thread ONLY at the global yield-point indices `SAF_SHIM_P1` /
///   `SAF_SHIM_P2` (round-robin), so a run performs at most two preemptions;
/// - models pthread mutexes in the scheduler (a contended lock parks the caller until an unlock
///   wakes it — the real program mutex is never locked);
/// - inserts extra scheduling points at shared memory accesses via the `SanitizerCoverage`
///   load/store callbacks (defined here as no-ops when the program is built without that
///   instrumentation), skipping thread-local stack addresses;
/// - honours `__VERIFIER_assume` as a hard path filter (R4);
/// - drops the sentinel ONLY on the property's violation event — `reach_error`,
///   `__VERIFIER_error`, or `__assert_fail` (R1) — which doubles as the witness.
///
/// The driver reads the program mutex API through `__real_pthread_mutex_lock/unlock` for its OWN
/// internal bookkeeping mutex, so the `--wrap` on the program's mutex calls never recurses into
/// the scheduler's locking. Every driver function is `no_sanitize("coverage")` so the load/store
/// instrumentation never fires inside the scheduler itself.
///
/// `sentinel_c_literal` is embedded as a C string literal (the path the driver writes `1` to on
/// a confirmed violation); callers must pass an already-escaped literal.
// NOTE: this is one cohesive code generator emitting a single self-contained C translation unit;
// splitting it across helpers would only scatter the driver source.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn synthesize_conc_shim_driver(sentinel_c_literal: &str) -> String {
    let mut s = String::new();
    s.push_str("/* SAF systematic bounded-preemption concurrency replay driver (generated) */\n");
    s.push_str("#define _GNU_SOURCE\n");
    let _ = writeln!(s, "#define __SAF_SENTINEL \"{sentinel_c_literal}\"");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("#include <stdint.h>\n");
    s.push_str("#include <pthread.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");
    // Every driver function is exempt from coverage instrumentation, so the shared-access
    // load/store callbacks never fire on the scheduler's own memory (no reentrancy).
    s.push_str("#define NS __attribute__((no_sanitize(\"coverage\")))\n");
    s.push_str("#define __SAF_MAXT 4096\n");
    s.push_str("#define __SAF_MAXM 4096\n");
    s.push_str(
        "enum { ST_RUN=0, ST_PARKED=1, ST_BLOCKED_JOIN=2, ST_DONE=3, ST_STARTING=4, ST_BLOCKED_LOCK=5 };\n",
    );

    // Real pthread entry points (the driver's OWN bookkeeping must bypass the --wrap redirects).
    s.push_str(
        "extern int __real_pthread_create(pthread_t*, const pthread_attr_t*, void*(*)(void*), void*);\n",
    );
    s.push_str("extern void __real_pthread_exit(void*) __attribute__((noreturn));\n");
    s.push_str("extern int __real_pthread_mutex_lock(pthread_mutex_t*);\n");
    s.push_str("extern int __real_pthread_mutex_unlock(pthread_mutex_t*);\n");

    s.push_str("static pthread_mutex_t __saf_m = PTHREAD_MUTEX_INITIALIZER;\n");
    s.push_str("static pthread_cond_t  __saf_c = PTHREAD_COND_INITIALIZER;\n");
    s.push_str("static int __saf_cur=0, __saf_nthreads=1;\n");
    s.push_str("static int __saf_state[__SAF_MAXT];\n");
    s.push_str("static int __saf_in_atomic[__SAF_MAXT];\n");
    s.push_str("static int __saf_blocked_mtx[__SAF_MAXT];\n");
    s.push_str("static int __saf_join_target[__SAF_MAXT];\n");
    s.push_str("static void*(*__saf_routine[__SAF_MAXT])(void*);\n");
    s.push_str("static void* __saf_arg[__SAF_MAXT];\n");
    s.push_str("static unsigned long __saf_handle[__SAF_MAXT];\n");
    s.push_str("static unsigned long __saf_next_handle=1;\n");
    s.push_str("static uintptr_t __saf_stack_hi[__SAF_MAXT];\n");
    s.push_str("static void* __saf_mtx_addr[__SAF_MAXM];\n");
    s.push_str("static int   __saf_mtx_owner[__SAF_MAXM];\n");
    s.push_str("static int   __saf_mtx_n=0;\n");
    s.push_str("static long  __saf_step=0;\n");
    s.push_str("static long  __saf_p1=-1, __saf_p2=-1;\n");
    s.push_str("static _Thread_local int __saf_tid=0;\n");

    s.push_str(
        "NS __attribute__((constructor)) static void __saf_init(void){\n\
         \x20 const char* a=getenv(\"SAF_SHIM_P1\"); __saf_p1 = a?strtol(a,0,10):-1;\n\
         \x20 const char* b=getenv(\"SAF_SHIM_P2\"); __saf_p2 = b?strtol(b,0,10):-1;\n\
         \x20 __saf_state[0]=ST_RUN; __saf_cur=0; __saf_nthreads=1; __saf_tid=0;\n\
         \x20 char probe; __saf_stack_hi[0]=(uintptr_t)&probe;\n\
         }\n",
    );

    // First ST_PARKED thread strictly after `me` in round-robin order (for a preemption); -1 none.
    s.push_str(
        "NS static int __saf_next_parked(int me){\n\
         \x20 for(int off=1; off<=__saf_nthreads; off++){ int i=(me+off)%__saf_nthreads;\n\
         \x20   if(i!=me && __saf_state[i]==ST_PARKED) return i; }\n\
         \x20 return -1;\n\
         }\n",
    );
    // Any ST_PARKED thread (round-robin from `me`); -1 if none (for a forced blocking handoff).
    s.push_str(
        "NS static int __saf_any_parked(int me){\n\
         \x20 for(int off=1; off<=__saf_nthreads; off++){ int i=(me+off)%__saf_nthreads;\n\
         \x20   if(__saf_state[i]==ST_PARKED) return i; }\n\
         \x20 return -1;\n\
         }\n",
    );
    // Caller holds __saf_m: give the token to `next`, wait until it is `me`'s turn again.
    s.push_str(
        "NS static void __saf_run(int next, int me){\n\
         \x20 __saf_cur=next; pthread_cond_broadcast(&__saf_c);\n\
         \x20 while(__saf_cur!=me) pthread_cond_wait(&__saf_c,&__saf_m);\n\
         }\n",
    );
    // Caller holds __saf_m and `me` just became non-runnable (blocked/complete): hand the token
    // to a runnable thread. No runnable thread while others are non-DONE ⇒ deadlock ⇒ abstain
    // (exit WITHOUT a sentinel — a lost schedule can never be a wrong verdict).
    s.push_str(
        "NS static void __saf_block(int me){\n\
         \x20 int nxt=__saf_any_parked(me);\n\
         \x20 if(nxt<0){ int alive=0; for(int i=0;i<__saf_nthreads;i++) if(__saf_state[i]!=ST_DONE) alive++;\n\
         \x20   if(alive<=1){ return; }\n\
         \x20   __saf_cur=-1; pthread_cond_broadcast(&__saf_c); __real_pthread_mutex_unlock(&__saf_m); _exit(0); }\n\
         \x20 __saf_run(nxt, me);\n\
         }\n",
    );
    // A preemptable yield point (lock/unlock boundary, shared access). Default: keep running `me`.
    // At the swept indices p1/p2: force a round-robin preemption to another runnable thread.
    // Yields inside a __VERIFIER_atomic section are suppressed (defence in depth — such programs
    // are gated out, but if ever handed one the section stays indivisible).
    s.push_str(
        "NS static void __saf_yield(void){\n\
         \x20 int me=__saf_tid;\n\
         \x20 if(__saf_in_atomic[me]>0) return;\n\
         \x20 __real_pthread_mutex_lock(&__saf_m);\n\
         \x20 long k=__saf_step++;\n\
         \x20 if(k==__saf_p1 || k==__saf_p2){\n\
         \x20   int nxt=__saf_next_parked(me);\n\
         \x20   if(nxt>=0){ __saf_state[me]=ST_PARKED; __saf_run(nxt, me); __saf_state[me]=ST_RUN; }\n\
         \x20 }\n\
         \x20 __real_pthread_mutex_unlock(&__saf_m);\n\
         }\n",
    );
    s.push_str(
        "NS static void __saf_complete(int id){\n\
         \x20 if(__saf_state[id]==ST_DONE) return; __saf_state[id]=ST_DONE;\n\
         \x20 for(int i=0;i<__saf_nthreads;i++) if(__saf_state[i]==ST_BLOCKED_JOIN && __saf_join_target[i]==id) __saf_state[i]=ST_PARKED;\n\
         \x20 __saf_block(id);\n\
         }\n",
    );
    s.push_str(
        "NS static void* __saf_trampoline(void* p){\n\
         \x20 int id=(int)(long)p; __saf_tid=id; char probe; __saf_stack_hi[id]=(uintptr_t)&probe;\n\
         \x20 __real_pthread_mutex_lock(&__saf_m); __saf_state[id]=ST_PARKED; pthread_cond_broadcast(&__saf_c);\n\
         \x20 while(__saf_cur!=id) pthread_cond_wait(&__saf_c,&__saf_m); __saf_state[id]=ST_RUN; __real_pthread_mutex_unlock(&__saf_m);\n\
         \x20 void* r=__saf_routine[id](__saf_arg[id]); (void)r;\n\
         \x20 __real_pthread_mutex_lock(&__saf_m); __saf_complete(id); __real_pthread_mutex_unlock(&__saf_m); return NULL;\n\
         }\n",
    );
    s.push_str(
        "NS int __wrap_pthread_create(pthread_t* th, const pthread_attr_t* a, void*(*start)(void*), void* arg){\n\
         \x20 (void)a; __real_pthread_mutex_lock(&__saf_m);\n\
         \x20 if(__saf_nthreads>=__SAF_MAXT){__real_pthread_mutex_unlock(&__saf_m);return 11;}\n\
         \x20 int id=__saf_nthreads++; __saf_state[id]=ST_STARTING; __saf_routine[id]=start; __saf_arg[id]=arg;\n\
         \x20 __saf_handle[id]=__saf_next_handle++;\n\
         \x20 if(th)*(unsigned long*)th=__saf_handle[id];\n\
         \x20 pthread_t os; __real_pthread_create(&os,NULL,__saf_trampoline,(void*)(long)id);\n\
         \x20 while(__saf_state[id]==ST_STARTING) pthread_cond_wait(&__saf_c,&__saf_m);\n\
         \x20 __real_pthread_mutex_unlock(&__saf_m); return 0;\n\
         }\n",
    );
    s.push_str(
        "NS int __wrap_pthread_join(pthread_t th, void** ret){ (void)ret;\n\
         \x20 int me=__saf_tid; unsigned long h=(unsigned long)th; __real_pthread_mutex_lock(&__saf_m);\n\
         \x20 int tgt=-1; for(int i=0;i<__saf_nthreads;i++) if(__saf_handle[i]==h){tgt=i;break;}\n\
         \x20 if(tgt<0){__real_pthread_mutex_unlock(&__saf_m);return 0;}\n\
         \x20 while(__saf_state[tgt]!=ST_DONE){ __saf_state[me]=ST_BLOCKED_JOIN; __saf_join_target[me]=tgt; __saf_block(me); }\n\
         \x20 __saf_state[me]=ST_RUN; __real_pthread_mutex_unlock(&__saf_m); return 0;\n\
         }\n",
    );
    s.push_str(
        "NS void __wrap_pthread_exit(void* ret){ int id=__saf_tid; __real_pthread_mutex_lock(&__saf_m); __saf_complete(id); __real_pthread_mutex_unlock(&__saf_m); __real_pthread_exit(ret); }\n",
    );
    // Program mutex shadow table: find-or-add a slot for a mutex address (owner -1 = free).
    s.push_str(
        "NS static int __saf_mtx_slot(void* m){ for(int i=0;i<__saf_mtx_n;i++) if(__saf_mtx_addr[i]==m) return i;\n\
         \x20 if(__saf_mtx_n<__SAF_MAXM){ int i=__saf_mtx_n++; __saf_mtx_addr[i]=m; __saf_mtx_owner[i]=-1; return i; } return __saf_mtx_n-1; }\n",
    );
    // lock: yield boundary, then acquire — parking (forced, uncounted) while held by anyone.
    s.push_str(
        "NS int __wrap_pthread_mutex_lock(pthread_mutex_t* m){\n\
         \x20 __saf_yield(); int me=__saf_tid; __real_pthread_mutex_lock(&__saf_m);\n\
         \x20 int slot=__saf_mtx_slot(m);\n\
         \x20 while(__saf_mtx_owner[slot]>=0){ __saf_state[me]=ST_BLOCKED_LOCK; __saf_blocked_mtx[me]=slot; __saf_block(me); }\n\
         \x20 __saf_mtx_owner[slot]=me; __saf_state[me]=ST_RUN; __real_pthread_mutex_unlock(&__saf_m); return 0;\n\
         }\n",
    );
    // unlock: release + wake this-mutex waiters, then a yield boundary (so a waiter can run).
    s.push_str(
        "NS int __wrap_pthread_mutex_unlock(pthread_mutex_t* m){\n\
         \x20 int me=__saf_tid; __real_pthread_mutex_lock(&__saf_m); int slot=__saf_mtx_slot(m);\n\
         \x20 if(__saf_mtx_owner[slot]==me){ __saf_mtx_owner[slot]=-1;\n\
         \x20   for(int i=0;i<__saf_nthreads;i++) if(__saf_state[i]==ST_BLOCKED_LOCK && __saf_blocked_mtx[i]==slot) __saf_state[i]=ST_PARKED; }\n\
         \x20 __real_pthread_mutex_unlock(&__saf_m); __saf_yield(); return 0;\n\
         }\n",
    );
    s.push_str(
        "NS int __wrap_pthread_mutex_trylock(pthread_mutex_t* m){\n\
         \x20 __saf_yield(); int me=__saf_tid; __real_pthread_mutex_lock(&__saf_m); int slot=__saf_mtx_slot(m); int r;\n\
         \x20 if(__saf_mtx_owner[slot]>=0) r=16; else { __saf_mtx_owner[slot]=me; r=0; }\n\
         \x20 __real_pthread_mutex_unlock(&__saf_m); return r;\n\
         }\n",
    );
    // Shared-access scheduling point: yield unless the address is on the running thread's own
    // stack (thread-local ⇒ cannot race ⇒ preempting there only wastes schedules). The window is
    // a sound over-approximation of the stack; the filter only affects recall, never soundness.
    s.push_str(
        "NS static void __saf_shared(void* addr){\n\
         \x20 int me=__saf_tid; uintptr_t a=(uintptr_t)addr, hi=__saf_stack_hi[me];\n\
         \x20 if(hi && a<=hi && a+(8UL<<20)>=hi) return;\n\
         \x20 __saf_yield();\n\
         }\n",
    );
    for n in [1, 2, 4, 8, 16] {
        let _ = writeln!(
            s,
            "NS void __sanitizer_cov_load{n}(void*a){{__saf_shared(a);}}"
        );
        let _ = writeln!(
            s,
            "NS void __sanitizer_cov_store{n}(void*a){{__saf_shared(a);}}"
        );
    }
    s.push_str("NS void __sanitizer_cov_trace_pc_guard(unsigned*g){(void)g;}\n");
    s.push_str(
        "NS void __sanitizer_cov_trace_pc_guard_init(unsigned*a,unsigned*b){(void)a;(void)b;}\n",
    );
    // R4: assumptions are hard path filters.
    s.push_str("NS void __VERIFIER_assume(int c){ if(!c) _exit(0); }\n");
    // Defence in depth: keep any __VERIFIER_atomic section indivisible (such programs are gated
    // out, so these are normally dead — but if handed one, suppress yields across the section).
    s.push_str(
        "NS void __VERIFIER_atomic_begin(void){ __saf_yield(); __saf_in_atomic[__saf_tid]++; }\n",
    );
    s.push_str("NS void __VERIFIER_atomic_end(void){ int me=__saf_tid; if(__saf_in_atomic[me]>0) __saf_in_atomic[me]--; }\n");
    // R1: the sentinel drops ONLY on the property's violation event.
    s.push_str(
        "NS __attribute__((noreturn)) static void __saf_hit(void){ FILE* f=fopen(__SAF_SENTINEL,\"w\"); if(f){fputc('1',f);fclose(f);} _exit(0); }\n",
    );
    s.push_str("NS __attribute__((weak)) void reach_error(void){ __saf_hit(); }\n");
    s.push_str("NS __attribute__((weak)) void __VERIFIER_error(void){ __saf_hit(); }\n");
    s.push_str(
        "NS __attribute__((noreturn)) void __assert_fail(const char*a,const char*b,unsigned c,const char*d){(void)a;(void)b;(void)c;(void)d;__saf_hit();}\n",
    );

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

    /// main -> pthread_create + a mutex in the thread body ⇒ shim-schedulable (the class
    /// `conc_replay` refuses because of the mutex, and `conc_seq`'s whole-thread schedules miss).
    #[test]
    fn schedulable_on_spawn_with_mutex_no_atomic() {
        let create = decl(1, "pthread_create");
        let join = decl(2, "pthread_join");
        let lock = decl(3, "pthread_mutex_lock");
        let unlock = decl(4, "pthread_mutex_unlock");
        let main = func(5, "main", &[create.id, join.id]);
        let thr = func(6, "thr", &[lock.id, unlock.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, join, lock, unlock, main, thr] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(conc_shim_schedulable(&m, &cg));
    }

    /// A spawn that uses `__VERIFIER_atomic` ⇒ abstain (that is `conc_replay`'s domain; splitting
    /// an atomic section at a shared-access yield would be unsound).
    #[test]
    fn abstains_on_verifier_atomic() {
        let create = decl(1, "pthread_create");
        let ab = decl(2, "__VERIFIER_atomic_begin");
        let main = func(3, "main", &[create.id]);
        let thr = func(4, "thr", &[ab.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, ab, main, thr] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_shim_schedulable(&m, &cg));
    }

    /// A non-default mutex type (recursive/errorcheck) ⇒ abstain (the NORMAL model cannot
    /// represent it).
    #[test]
    fn abstains_on_mutexattr_settype() {
        let create = decl(1, "pthread_create");
        let settype = decl(2, "pthread_mutexattr_settype");
        let main = func(3, "main", &[create.id, settype.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, settype, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_shim_schedulable(&m, &cg));
    }

    /// A reachable nondet call ⇒ abstain (inherited from `conc_schedulable`).
    #[test]
    fn abstains_on_nondet() {
        let create = decl(1, "pthread_create");
        let nondet = decl(2, "__VERIFIER_nondet_int");
        let main = func(3, "main", &[create.id, nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [create, nondet, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_shim_schedulable(&m, &cg));
    }

    /// No reachable spawn ⇒ not our job (inherited from `conc_schedulable`).
    #[test]
    fn abstains_without_spawn() {
        let lock = decl(1, "pthread_mutex_lock");
        let main = func(2, "main", &[lock.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        for f in [lock, main] {
            m.functions.push(f);
        }
        let cg = CallGraph::build(&m);
        assert!(!conc_shim_schedulable(&m, &cg));
    }

    #[test]
    fn plans_sweep_c1_then_c2_and_are_unique() {
        let plans = shim_preemption_plans();
        // Every c=1 schedule comes first, in ascending p1.
        let c1: Vec<_> = plans.iter().take_while(|p| p.p2 < 0).collect();
        assert_eq!(c1.len(), (SHIM_MAX_STEP + 1) as usize);
        for (i, p) in c1.iter().enumerate() {
            assert_eq!(p.p1, i as i64);
            assert_eq!(p.p2, -1);
        }
        // The rest are c=2 pairs with p1 < p2.
        for p in plans.iter().skip(c1.len()) {
            assert!(p.p2 > p.p1 && p.p1 >= 0);
        }
        // All (p1,p2) pairs are unique.
        let mut pairs: Vec<(i64, i64)> = plans.iter().map(|p| (p.p1, p.p2)).collect();
        pairs.sort_unstable();
        let n = pairs.len();
        pairs.dedup();
        assert_eq!(pairs.len(), n);
    }

    #[test]
    fn plan_labels_and_env_are_stable() {
        let a = ShimPlan { p1: 0, p2: -1 };
        assert_eq!(a.label(), "shim:c1:0");
        assert_eq!(a.p1_env(), "0");
        assert_eq!(a.p2_env(), "-1");
        let b = ShimPlan { p1: 0, p2: 4 };
        assert_eq!(b.label(), "shim:c2:0,4");
        assert_eq!(b.p2_env(), "4");
    }

    #[test]
    fn driver_has_wrappers_mutex_model_sentinel_and_assume() {
        let d = synthesize_conc_shim_driver("/tmp/sent");
        // pthread wrappers (incl. the mutex wrappers `conc_replay` lacks).
        assert!(d.contains("__wrap_pthread_create"));
        assert!(d.contains("__wrap_pthread_join"));
        assert!(d.contains("__wrap_pthread_exit"));
        assert!(d.contains("__wrap_pthread_mutex_lock"));
        assert!(d.contains("__wrap_pthread_mutex_unlock"));
        assert!(d.contains("__wrap_pthread_mutex_trylock"));
        // Internal bookkeeping must use the REAL mutex ops (no --wrap recursion).
        assert!(d.contains("__real_pthread_mutex_lock(&__saf_m)"));
        assert!(!d.contains(" pthread_mutex_lock(&__saf_m)"));
        // Mutex shadow model + bounded-preemption scheduler.
        assert!(d.contains("__saf_mtx_owner"));
        assert!(d.contains("ST_BLOCKED_LOCK"));
        assert!(d.contains("SAF_SHIM_P1"));
        assert!(d.contains("SAF_SHIM_P2"));
        // Shared-access scheduling points (SanitizerCoverage load/store callbacks).
        assert!(d.contains("__sanitizer_cov_load4"));
        assert!(d.contains("__sanitizer_cov_store8"));
        // Coverage exemption so the callbacks never fire inside the driver.
        assert!(d.contains("no_sanitize(\"coverage\")"));
        // Property events + assume.
        assert!(d.contains("__assert_fail"));
        assert!(d.contains("reach_error"));
        assert!(d.contains("__VERIFIER_assume"));
        assert!(d.contains("\"/tmp/sent\""));
    }
}
