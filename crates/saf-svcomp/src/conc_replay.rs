//! Fine-grained concurrency `unreach-call` FALSE confirmer via a cooperative
//! single-token scheduler that replays *forced interleavings* of the ORIGINAL binary.
//!
//! # Why this exists (what [`crate::conc_seq`] cannot do)
//!
//! The atomic-thread sequentialization confirmer ([`crate::conc_seq`], lever
//! `conc-seq-m1`) runs each spawned thread *start-to-finish* — it only ever explores
//! interleavings whose context switches land on `pthread_create` / `pthread_join` /
//! thread-exit boundaries. A whole class of real sv-benchmarks bugs is invisible to
//! that model: they require switching threads *mid-body*, at `__VERIFIER_atomic`
//! section boundaries (`fib_unsafe`, `triangular`, `reorder`, `singleton`, …). Two
//! threads each doing `atomic{ i = i + j; }` five times only reach the buggy
//! `i,j >= bound` state under an *alternating* schedule; running one thread then the
//! other never does.
//!
//! This module explores those finer interleavings by forcing them on the REAL
//! multithreaded binary (unmodified program source), under a cooperative scheduler.
//!
//! # Mechanism (a fresh, conservative controlled-scheduling replay)
//!
//! Inspired by controlled-concurrency-testing tools (CHESS, Maple, and especially
//! **PCT** — *A Randomized Scheduler with Probabilistic Guarantees of Finding Bugs*,
//! Burckhardt–Kothari–Musuvathi–Nagarakatte, ASPLOS 2010) and by the sequentialization
//! line (Lazy-CSeq), but implemented independently (REQ-IP-001):
//!
//! A tiny generated C driver is linked with the original program via
//! `-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit`. Threads are
//! REAL OS threads, but a single global **token** (`g_cur` = the one thread allowed to
//! run) makes execution strictly serial: only the token-holder runs; everyone else
//! blocks on a condvar until it is their turn. Context switches happen ONLY at the
//! *scheduling points* the driver intercepts — `__VERIFIER_atomic_begin`, thread
//! entry, `pthread_join`, and thread completion. `__VERIFIER_atomic` sections hold the
//! token continuously (a per-thread `in_atomic` depth suppresses yields), so an atomic
//! section is never split. Two scheduling policies pick the next token-holder:
//! - `rr`  — round-robin among runnable threads, deferring `main` (id 0) until all
//!   spawned threads are done: exposes the "spawn all, let workers alternate, then
//!   read" pattern (`fib_unsafe`, `triangular`).
//! - `pct(seed)` — a PCT-style priority scheduler: each thread gets a random priority,
//!   with a bounded number of random priority-change points that preempt the running
//!   thread; the highest-priority runnable thread runs. Different seeds explore
//!   different bounded-depth interleavings (`reorder`, `singleton`).
//!
//! # Why this is SOUND for `unreach-call` FALSE (soundness is the whole game)
//!
//! Because exactly one thread holds the token at a time and switches happen only at
//! the driver's scheduling points, every replayed run is a **real, serialized,
//! sequentially-consistent execution** of one legal interleaving of the original
//! program — executed natively, no abstraction. `__VERIFIER_atomic` sections are truly
//! atomic (token held across them), so we never manufacture an interleaving the spec
//! forbids. The SOLE confirmer is the property's own violation event (`reach_error` /
//! `__VERIFIER_error` / `__assert_fail`, confirmer-contract R1), captured by the
//! sentinel sink. So a schedule that drops the sentinel is a genuine reachable
//! assertion violation ⇒ a sound `false(unreach-call)`. A schedule that does not
//! reach it yields nothing. Mis-modelling can only ever *lose* a schedule (deadlock →
//! timeout → abstain), never invent a violation.
//!
//! # What we DELIBERATELY gate out (fail-closed — costs recall, never soundness)
//!
//! [`conc_replay_schedulable`] requires everything [`crate::conc_schedulable`] does
//! (reachable spawn; no nondet, TLS, condvars, barriers, rwlocks, spinlocks, semaphores,
//! OpenMP, or indirect calls — see that function) AND, additionally:
//! - the program must actually use `__VERIFIER_atomic_begin` (otherwise there are no
//!   mid-body scheduling points, so this engine explores exactly the whole-thread
//!   schedules `conc_seq` already tried — no added coverage, skip it);
//! - the program must NOT call `pthread_mutex_lock` / `trylock` / `timedlock` (mutual
//!   exclusion is a scheduling constraint we do not model here; a real mutex would
//!   block the single OS token-holder and deadlock, but rather than pay that timeout
//!   on every schedule we abstain up front — `conc_seq` already handles the
//!   whole-thread mutex cases).

use std::fmt::Write as _;

use saf_analysis::callgraph::CallGraph;
use saf_core::air::{AirModule, Operation};

use crate::conc_seq::conc_schedulable;

/// A forced-interleaving replay plan the generated driver can execute. Each maps to a
/// `(SAF_REPLAY_POLICY, SAF_REPLAY_SEED)` environment pair the driver reads. The order
/// of [`replay_plans`] is the deterministic search order (first sentinel drop wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayPlan {
    /// Round-robin among spawned threads, deferring `main`. Deterministic (seed unused).
    RoundRobin,
    /// PCT-style randomized priority schedule with the given deterministic seed.
    Pct(u32),
}

/// Number of PCT seeds tried after the round-robin plan. Sized so the observed
/// interleaving bugs (`reorder_*` need seeds into the 40s) are reachable while keeping
/// the per-task replay budget bounded — each run of a (gated) atomic-only program that
/// does not reproduce terminates in milliseconds, not the per-run timeout.
pub const REPLAY_PCT_SEEDS: u32 = 48;

/// The forced-interleaving plans tried, in deterministic search order: round-robin
/// first (catches the "workers alternate, then main reads" class cheaply and
/// deterministically), then PCT seeds `0..REPLAY_PCT_SEEDS`.
#[must_use]
pub fn replay_plans() -> Vec<ReplayPlan> {
    let mut v = Vec::with_capacity(1 + REPLAY_PCT_SEEDS as usize);
    v.push(ReplayPlan::RoundRobin);
    for s in 0..REPLAY_PCT_SEEDS {
        v.push(ReplayPlan::Pct(s));
    }
    v
}

impl ReplayPlan {
    /// The `SAF_REPLAY_POLICY` value the driver reads (`rr` or `pct`).
    #[must_use]
    pub fn policy_env(self) -> &'static str {
        match self {
            ReplayPlan::RoundRobin => "rr",
            ReplayPlan::Pct(_) => "pct",
        }
    }

    /// The `SAF_REPLAY_SEED` value the driver reads (0 for round-robin).
    #[must_use]
    pub fn seed_env(self) -> u32 {
        match self {
            ReplayPlan::RoundRobin => 0,
            ReplayPlan::Pct(s) => s,
        }
    }

    /// A short, stable label for diagnostics / witnesses.
    #[must_use]
    pub fn label(self) -> String {
        match self {
            ReplayPlan::RoundRobin => "rr".to_string(),
            ReplayPlan::Pct(s) => format!("pct:{s}"),
        }
    }
}

/// Mutual-exclusion lock primitives we do not model in the single-token scheduler.
/// A reachable call to one of these ⇒ abstain (see the module docs).
fn is_mutex_lock_call(name: &str) -> bool {
    matches!(
        name,
        "pthread_mutex_lock" | "pthread_mutex_trylock" | "pthread_mutex_timedlock"
    )
}

/// Does any defined function directly call `__VERIFIER_atomic_begin`? (Scanned across
/// EVERY defined function, not just the callgraph-reachable set: a thread entry is
/// reached only through the `pthread_create` function pointer, an unresolved indirect
/// edge, so a reachable-only scan would miss atomic sections inside thread bodies.)
fn uses_verifier_atomic(module: &AirModule) -> bool {
    module_has_direct_call_to(module, |name| name == "__VERIFIER_atomic_begin")
}

/// Does any defined function directly call a mutual-exclusion lock primitive?
fn uses_mutex_lock(module: &AirModule) -> bool {
    module_has_direct_call_to(module, is_mutex_lock_call)
}

/// Is there a direct call, in any defined function, to a callee whose name satisfies
/// `pred`?
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

/// Is `module` amenable to the forced-interleaving replay confirmer?
///
/// Requires everything [`conc_schedulable`] does (reachable spawn; only faithfully
/// modelled primitives) AND, additionally, that the program uses `__VERIFIER_atomic`
/// (so there are mid-body scheduling points beyond what [`crate::conc_seq`] explores)
/// and does NOT use a mutual-exclusion lock (which the single-token model does not
/// represent). See the module docs for the soundness rationale. Any doubt ⇒ abstain.
#[must_use]
pub fn conc_replay_schedulable(module: &AirModule, callgraph: &CallGraph) -> bool {
    conc_schedulable(module, callgraph) && uses_verifier_atomic(module) && !uses_mutex_lock(module)
}

/// Generate the C source of the cooperative single-token replay driver.
///
/// Linked with the ORIGINAL program via
/// `-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit`, this driver
/// forces one interleaving selected by `SAF_REPLAY_POLICY` (`rr` | `pct`) and
/// `SAF_REPLAY_SEED`. It:
/// - makes execution strictly serial via a global token (`g_cur`), so every run is a
///   real sequentially-consistent interleaving;
/// - switches threads only at `__VERIFIER_atomic_begin`, thread entry, `pthread_join`,
///   and thread completion, holding the token across whole `__VERIFIER_atomic`
///   sections (so they stay atomic — soundness);
/// - honours `__VERIFIER_assume` as a hard path filter (R4);
/// - drops the sentinel ONLY on the property's violation event — `reach_error`,
///   `__VERIFIER_error`, or `__assert_fail` (R1) — which doubles as the witness.
///
/// `sentinel_c_literal` is embedded as a C string literal (the path the driver writes
/// `1` to on a confirmed violation); callers must pass an already-escaped literal.
// NOTE: this is one cohesive code generator emitting a single self-contained C
// translation unit; splitting it across helpers would only scatter the driver source.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn synthesize_conc_replay_driver(sentinel_c_literal: &str) -> String {
    let mut s = String::new();
    s.push_str("/* SAF cooperative single-token concurrency replay driver (generated) */\n");
    s.push_str("#define _GNU_SOURCE\n");
    let _ = writeln!(s, "#define __SAF_SENTINEL \"{sentinel_c_literal}\"");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("#include <pthread.h>\n");
    // Bounded thread table (sv-benchmarks spawn counts fit comfortably; overflow just
    // refuses to record extra threads — a partial schedule, never a wrong verdict).
    s.push_str("#define __SAF_MAXT 4096\n");
    s.push_str("enum { ST_RUN=0, ST_PARKED=1, ST_BLOCKED_JOIN=2, ST_DONE=3, ST_STARTING=4 };\n");
    // PCT preemption budget (bug depth) and step horizon for random change points.
    s.push_str("#define __SAF_PCT_D 4\n");
    s.push_str("#define __SAF_PCT_BOUND 256\n");

    s.push_str("extern int __real_pthread_create(pthread_t*, const pthread_attr_t*, void*(*)(void*), void*);\n");
    s.push_str("extern void __real_pthread_exit(void*) __attribute__((noreturn));\n");

    s.push_str("static pthread_mutex_t __saf_m = PTHREAD_MUTEX_INITIALIZER;\n");
    s.push_str("static pthread_cond_t  __saf_c = PTHREAD_COND_INITIALIZER;\n");
    s.push_str("static int __saf_cur = 0;\n");
    s.push_str("static int __saf_nthreads = 1;\n");
    s.push_str("static int __saf_state[__SAF_MAXT];\n");
    s.push_str("static int __saf_in_atomic[__SAF_MAXT];\n");
    s.push_str("static int __saf_join_target[__SAF_MAXT];\n");
    s.push_str("static int __saf_prio[__SAF_MAXT];\n");
    s.push_str("static long __saf_last_run[__SAF_MAXT];\n");
    s.push_str("static long __saf_rr = 0;\n");
    s.push_str("static int __saf_policy = 0; /* 0=pct, 1=rr */\n");
    s.push_str("static void* (*__saf_routine[__SAF_MAXT])(void*);\n");
    s.push_str("static void* __saf_arg[__SAF_MAXT];\n");
    s.push_str("static unsigned long __saf_handle[__SAF_MAXT];\n");
    s.push_str("static unsigned long __saf_next_handle = 1;\n");
    s.push_str("static unsigned long __saf_rng_state = 0;\n");
    s.push_str("static long __saf_step = 0;\n");
    s.push_str("static long __saf_pc_step[__SAF_PCT_D];\n");
    s.push_str("static int  __saf_pc_used[__SAF_PCT_D];\n");
    s.push_str("static int  __saf_pct_next_low = 0;\n");
    s.push_str("static _Thread_local int __saf_tid = 0;\n");

    s.push_str(
        "static unsigned long __saf_rng(void){\n\
         \x20 __saf_rng_state = __saf_rng_state*6364136223846793005UL + 1442695040888963407UL;\n\
         \x20 return __saf_rng_state >> 33;\n\
         }\n",
    );
    s.push_str(
        "static int __saf_prio_new(void){ return __SAF_PCT_D + (int)(__saf_rng() % 1000000); }\n",
    );

    s.push_str(
        "__attribute__((constructor)) static void __saf_init(void){\n\
         \x20 const char* s = getenv(\"SAF_REPLAY_SEED\");\n\
         \x20 __saf_rng_state = s ? strtoul(s,0,10) : 0;\n\
         \x20 const char* pol = getenv(\"SAF_REPLAY_POLICY\");\n\
         \x20 __saf_policy = (pol && pol[0]=='r') ? 1 : 0;\n\
         \x20 __saf_state[0]=ST_RUN; __saf_cur=0; __saf_nthreads=1; __saf_tid=0;\n\
         \x20 __saf_prio[0]=__saf_prio_new();\n\
         \x20 for(int i=0;i<__SAF_PCT_D;i++){ __saf_pc_step[i]=(long)(__saf_rng()%__SAF_PCT_BOUND); __saf_pc_used[i]=0; }\n\
         }\n",
    );

    // Pick the next runnable (PARKED) thread per policy; -1 if none.
    s.push_str(
        "static int __saf_pick(void){\n\
         \x20 int best=-1;\n\
         \x20 if(__saf_policy==1){\n\
         \x20   long bestkey=0;\n\
         \x20   for(int i=0;i<__saf_nthreads;i++){\n\
         \x20     if(__saf_state[i]!=ST_PARKED) continue;\n\
         \x20     long key = __saf_last_run[i] + (i==0 ? 1000000000L : 0);\n\
         \x20     if(best<0 || key<bestkey){ best=i; bestkey=key; }\n\
         \x20   }\n\
         \x20 } else {\n\
         \x20   for(int i=0;i<__saf_nthreads;i++)\n\
         \x20     if(__saf_state[i]==ST_PARKED && (best<0 || __saf_prio[i]>__saf_prio[best])) best=i;\n\
         \x20 }\n\
         \x20 if(best>=0) __saf_last_run[best]=++__saf_rr;\n\
         \x20 return best;\n\
         }\n",
    );

    // Yield the token: caller holds __saf_m and has set __saf_state[me]. Pick the next
    // runnable thread and wait until it is me's turn again. Abstain (exit, NO sentinel)
    // on deadlock — a lost schedule can never be a wrong verdict.
    s.push_str(
        "static void __saf_switch(int me){\n\
         \x20 __saf_step++;\n\
         \x20 for(int i=0;i<__SAF_PCT_D;i++)\n\
         \x20   if(!__saf_pc_used[i] && __saf_step==__saf_pc_step[i]){ __saf_pc_used[i]=1; __saf_prio[me]=__saf_pct_next_low++; break; }\n\
         \x20 for(;;){\n\
         \x20   int best=__saf_pick();\n\
         \x20   if(best>=0){ __saf_cur=best; break; }\n\
         \x20   __saf_cur=-1; pthread_cond_broadcast(&__saf_c); pthread_mutex_unlock(&__saf_m); _exit(0);\n\
         \x20 }\n\
         \x20 pthread_cond_broadcast(&__saf_c);\n\
         \x20 while(__saf_cur!=me) pthread_cond_wait(&__saf_c,&__saf_m);\n\
         }\n",
    );

    // Terminal handoff for a finished thread: pick the next runnable and return without
    // waiting. If nothing is runnable and threads remain non-DONE it is a deadlock (abstain).
    s.push_str(
        "static void __saf_handoff_final(int me){\n\
         \x20 (void)me;\n\
         \x20 int best=__saf_pick();\n\
         \x20 if(best>=0){ __saf_cur=best; pthread_cond_broadcast(&__saf_c); return; }\n\
         \x20 int alive=0; for(int i=0;i<__saf_nthreads;i++) if(__saf_state[i]!=ST_DONE) alive++;\n\
         \x20 if(alive==0){ __saf_cur=-1; pthread_cond_broadcast(&__saf_c); return; }\n\
         \x20 __saf_cur=-1; pthread_cond_broadcast(&__saf_c); pthread_mutex_unlock(&__saf_m); _exit(0);\n\
         }\n",
    );

    // A scheduling point: yield unless inside an atomic section (token held across it).
    s.push_str(
        "static void __saf_sched_point(void){\n\
         \x20 int me=__saf_tid;\n\
         \x20 pthread_mutex_lock(&__saf_m);\n\
         \x20 if(__saf_in_atomic[me]>0){ pthread_mutex_unlock(&__saf_m); return; }\n\
         \x20 __saf_state[me]=ST_PARKED;\n\
         \x20 __saf_switch(me);\n\
         \x20 __saf_state[me]=ST_RUN;\n\
         \x20 pthread_mutex_unlock(&__saf_m);\n\
         }\n",
    );

    // Caller holds __saf_m: mark id DONE, wake its joiners, hand off to next runnable.
    s.push_str(
        "static void __saf_complete(int id){\n\
         \x20 if(__saf_state[id]==ST_DONE) return;\n\
         \x20 __saf_state[id]=ST_DONE;\n\
         \x20 for(int i=0;i<__saf_nthreads;i++)\n\
         \x20   if(__saf_state[i]==ST_BLOCKED_JOIN && __saf_join_target[i]==id) __saf_state[i]=ST_PARKED;\n\
         \x20 __saf_handoff_final(id);\n\
         }\n",
    );

    s.push_str(
        "static void* __saf_trampoline(void* p){\n\
         \x20 int id=(int)(long)p;\n\
         \x20 __saf_tid=id;\n\
         \x20 pthread_mutex_lock(&__saf_m);\n\
         \x20 __saf_state[id]=ST_PARKED;\n\
         \x20 pthread_cond_broadcast(&__saf_c);\n\
         \x20 while(__saf_cur!=id) pthread_cond_wait(&__saf_c,&__saf_m);\n\
         \x20 __saf_state[id]=ST_RUN;\n\
         \x20 pthread_mutex_unlock(&__saf_m);\n\
         \x20 void* r=__saf_routine[id](__saf_arg[id]); (void)r;\n\
         \x20 pthread_mutex_lock(&__saf_m);\n\
         \x20 __saf_complete(id);\n\
         \x20 pthread_mutex_unlock(&__saf_m);\n\
         \x20 return NULL;\n\
         }\n",
    );

    s.push_str(
        "int __wrap_pthread_create(pthread_t* th, const pthread_attr_t* attr,\n\
         \x20                       void* (*start)(void*), void* arg){\n\
         \x20 (void)attr;\n\
         \x20 pthread_mutex_lock(&__saf_m);\n\
         \x20 if(__saf_nthreads>=__SAF_MAXT){ pthread_mutex_unlock(&__saf_m); return 11; }\n\
         \x20 int id=__saf_nthreads++;\n\
         \x20 __saf_state[id]=ST_STARTING; __saf_routine[id]=start; __saf_arg[id]=arg;\n\
         \x20 __saf_handle[id]=__saf_next_handle++; __saf_prio[id]=__saf_prio_new();\n\
         \x20 if(th) *(unsigned long*)th=__saf_handle[id];\n\
         \x20 pthread_t os;\n\
         \x20 __real_pthread_create(&os,NULL,__saf_trampoline,(void*)(long)id);\n\
         \x20 while(__saf_state[id]==ST_STARTING) pthread_cond_wait(&__saf_c,&__saf_m);\n\
         \x20 pthread_mutex_unlock(&__saf_m);\n\
         \x20 return 0;\n\
         }\n",
    );

    s.push_str(
        "int __wrap_pthread_join(pthread_t th, void** ret){\n\
         \x20 (void)ret;\n\
         \x20 int me=__saf_tid;\n\
         \x20 unsigned long h=(unsigned long)th;\n\
         \x20 pthread_mutex_lock(&__saf_m);\n\
         \x20 int tgt=-1;\n\
         \x20 for(int i=0;i<__saf_nthreads;i++) if(__saf_handle[i]==h){ tgt=i; break; }\n\
         \x20 if(tgt<0){ pthread_mutex_unlock(&__saf_m); return 0; }\n\
         \x20 while(__saf_state[tgt]!=ST_DONE){\n\
         \x20   __saf_state[me]=ST_BLOCKED_JOIN; __saf_join_target[me]=tgt;\n\
         \x20   __saf_switch(me);\n\
         \x20 }\n\
         \x20 __saf_state[me]=ST_RUN;\n\
         \x20 pthread_mutex_unlock(&__saf_m);\n\
         \x20 return 0;\n\
         }\n",
    );

    // A thread-body pthread_exit runs the completion handoff then really terminates the
    // OS thread; a main-level pthread_exit likewise hands off (POSIX keeps the process
    // alive for the remaining threads).
    s.push_str(
        "void __wrap_pthread_exit(void* ret){\n\
         \x20 int id=__saf_tid;\n\
         \x20 pthread_mutex_lock(&__saf_m);\n\
         \x20 __saf_complete(id);\n\
         \x20 pthread_mutex_unlock(&__saf_m);\n\
         \x20 __real_pthread_exit(ret);\n\
         }\n",
    );

    // Atomic sections: begin is a scheduling point (switch may happen at the boundary),
    // then the token is held until end (yields suppressed while in_atomic>0).
    s.push_str(
        "void __VERIFIER_atomic_begin(void){\n\
         \x20 int me=__saf_tid;\n\
         \x20 __saf_sched_point();\n\
         \x20 pthread_mutex_lock(&__saf_m); __saf_in_atomic[me]++; pthread_mutex_unlock(&__saf_m);\n\
         }\n",
    );
    s.push_str(
        "void __VERIFIER_atomic_end(void){\n\
         \x20 int me=__saf_tid;\n\
         \x20 pthread_mutex_lock(&__saf_m); if(__saf_in_atomic[me]>0) __saf_in_atomic[me]--; pthread_mutex_unlock(&__saf_m);\n\
         }\n",
    );
    // R4: assumptions are hard path filters.
    s.push_str("void __VERIFIER_assume(int c){ if(!c) _exit(0); }\n");

    // R1: the sentinel drops ONLY on the property's violation event.
    s.push_str(
        "__attribute__((noreturn)) static void __saf_hit(void){\n\
         \x20 FILE* f=fopen(__SAF_SENTINEL,\"w\"); if(f){ fputc('1',f); fclose(f); } _exit(0);\n\
         }\n",
    );
    s.push_str("__attribute__((weak)) void reach_error(void){ __saf_hit(); }\n");
    s.push_str("__attribute__((weak)) void __VERIFIER_error(void){ __saf_hit(); }\n");
    s.push_str(
        "__attribute__((noreturn)) void __assert_fail(const char*a,const char*b,unsigned c,const char*d){\n\
         \x20 (void)a;(void)b;(void)c;(void)d; __saf_hit();\n\
         }\n",
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

    /// main -> pthread_create + __VERIFIER_atomic_begin (in a thread body) ⇒ replayable.
    #[test]
    fn replayable_on_spawn_with_atomic() {
        let create = decl(1, "pthread_create");
        let join = decl(2, "pthread_join");
        let ab = decl(3, "__VERIFIER_atomic_begin");
        let main = func(4, "main", &[create.id, join.id]);
        // The atomic section lives in a thread body (reached via the create fn ptr).
        let thr = func(5, "thr", &[ab.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(join);
        m.functions.push(ab);
        m.functions.push(main);
        m.functions.push(thr);
        let cg = CallGraph::build(&m);
        assert!(conc_replay_schedulable(&m, &cg));
    }

    /// A spawn with NO `__VERIFIER_atomic` ⇒ abstain (no mid-body scheduling points,
    /// so `conc_seq`'s whole-thread schedules already cover it — no added value).
    #[test]
    fn abstains_without_atomic() {
        let create = decl(1, "pthread_create");
        let main = func(2, "main", &[create.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_replay_schedulable(&m, &cg));
    }

    /// A spawn + atomic BUT with a mutex lock ⇒ abstain (mutual exclusion unmodelled).
    #[test]
    fn abstains_on_mutex_lock() {
        let create = decl(1, "pthread_create");
        let ab = decl(2, "__VERIFIER_atomic_begin");
        let lock = decl(3, "pthread_mutex_lock");
        let main = func(4, "main", &[create.id]);
        let thr = func(5, "thr", &[ab.id, lock.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(ab);
        m.functions.push(lock);
        m.functions.push(main);
        m.functions.push(thr);
        let cg = CallGraph::build(&m);
        assert!(!conc_replay_schedulable(&m, &cg));
    }

    /// A reachable nondet call ⇒ abstain (inherited from `conc_schedulable`).
    #[test]
    fn abstains_on_nondet() {
        let create = decl(1, "pthread_create");
        let ab = decl(2, "__VERIFIER_atomic_begin");
        let nondet = decl(3, "__VERIFIER_nondet_int");
        let main = func(4, "main", &[create.id, ab.id, nondet.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(create);
        m.functions.push(ab);
        m.functions.push(nondet);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_replay_schedulable(&m, &cg));
    }

    /// No reachable spawn ⇒ not our job (inherited from `conc_schedulable`).
    #[test]
    fn abstains_without_spawn() {
        let ab = decl(1, "__VERIFIER_atomic_begin");
        let main = func(2, "main", &[ab.id]);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(ab);
        m.functions.push(main);
        let cg = CallGraph::build(&m);
        assert!(!conc_replay_schedulable(&m, &cg));
    }

    #[test]
    fn replay_plans_are_rr_then_pct_and_unique() {
        let plans = replay_plans();
        assert_eq!(plans[0], ReplayPlan::RoundRobin);
        assert_eq!(plans.len(), 1 + REPLAY_PCT_SEEDS as usize);
        // policy/seed pairs are unique.
        let mut pairs: Vec<(String, u32)> =
            plans.iter().map(|p| (p.label(), p.seed_env())).collect();
        pairs.sort();
        pairs.dedup();
        assert_eq!(pairs.len(), plans.len());
        // round-robin reports rr with seed 0; pct reports its seed.
        assert_eq!(ReplayPlan::RoundRobin.policy_env(), "rr");
        assert_eq!(ReplayPlan::Pct(7).policy_env(), "pct");
        assert_eq!(ReplayPlan::Pct(7).seed_env(), 7);
    }

    #[test]
    fn driver_has_wrappers_scheduler_sentinel_and_assume() {
        let d = synthesize_conc_replay_driver("/tmp/sent");
        assert!(d.contains("__wrap_pthread_create"));
        assert!(d.contains("__wrap_pthread_join"));
        assert!(d.contains("__wrap_pthread_exit"));
        assert!(d.contains("__real_pthread_create"));
        assert!(d.contains("__real_pthread_exit"));
        // scheduler internals
        assert!(d.contains("__saf_switch"));
        assert!(d.contains("__saf_pick"));
        assert!(d.contains("in_atomic")); // atomic-section token hold
        assert!(d.contains("SAF_REPLAY_POLICY"));
        assert!(d.contains("SAF_REPLAY_SEED"));
        // property events + assume
        assert!(d.contains("__assert_fail"));
        assert!(d.contains("reach_error"));
        assert!(d.contains("__VERIFIER_atomic_begin"));
        assert!(d.contains("__VERIFIER_assume"));
        assert!(d.contains("\"/tmp/sent\""));
    }
}
