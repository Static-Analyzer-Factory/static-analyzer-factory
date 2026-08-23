//! `no-data-race` **TRUE** prover (lever: `race-free-true-prover`).
//!
//! Emits a sound `no-data-race = true` verdict from a **static structural proof**,
//! never `false`. Mirrors the discipline of [`crate::termination`]: a sufficient,
//! decidable, SOUND-but-incomplete condition that fails **closed** (abstains) on
//! anything it cannot soundly discharge.
//!
//! # What it proves
//!
//! A program is data-race-free if, for **every** pair of memory accesses that
//! (a) may run concurrently, (b) may alias, and (c) include at least one write,
//! the two accesses are **mutually excluded** by a common lock. We discharge this
//! by one of:
//!
//! - **P1 (locking):** the pair provably always holds one common, *uniquely
//!   identified* mutex (a non-empty must-lockset intersection), OR
//! - **P3 (read-only / disjoint):** no such conflicting pair exists at all — every
//!   shared location is only ever read after the threads become concurrent (this
//!   falls out for free: a pair with no write is never a conflict).
//!
//! # Soundness contract (why this cannot emit a wrong `true`, −32)
//!
//! The single failure mode of a race-freedom prover is concluding "protected"
//! when the accesses are actually unprotected. Two imprecisions cause it, and we
//! close **both**:
//!
//! 1. **Over-approximating the must-lockset** (treating a *may*-held lock as
//!    must-held). The classic trap is a lock whose guarding pointer does not
//!    uniquely name one concrete mutex: a context-merged parameter (`munge(&mutex1)`
//!    vs `munge(&mutex2)`), an *array* of mutexes (`&m[3]` vs `&m[4]`), a *struct*
//!    of mutexes (`&m.x` vs `&m.y`), or a symbolic address (`&s->mutex`, `s ∈ {A,
//!    B}`). The frontend makes this worse: it collapses the address of an
//!    aggregate sub-object mutex onto the base object, so two *different* embedded
//!    mutexes become the same value. We therefore admit a lock into the
//!    must-lockset **only** when its pointer resolves — via a def-chain walk over
//!    casts and no-op GEPs — to a *standalone global* mutex: a global that is
//!    NEVER the base of an indexing GEP and is NOT initialized by ≥2
//!    `pthread_mutex_init` sites (either would prove it holds several distinct
//!    mutexes the frontend has conflated). A parameter, `phi`/`select`, loaded
//!    pointer, or indexed sub-object all drop the lock (fail-closed). Symmetrically,
//!    `unlock(p)` drops the lock `p` resolves to, or — when `p` is ambiguous —
//!    clears the whole set (a possibly-mismatched unlock soundly voids protection).
//!
//! 2. **Under-approximating aliasing / concurrency** (missing a conflicting pair).
//!    We use the *conservative* alias query ([`AliasResult::may_alias_conservative`],
//!    which treats untracked pointers as may-alias) and treat **every pair of
//!    distinct spawned threads as possibly-concurrent** — we never trust an MTA
//!    happens-before result to *rule out* an overlap (that would miss races like
//!    Peterson/Lamport where two joined threads race on a shared flag). An access
//!    we cannot analyze (untracked pointer) conflicts with everything and abstains.
//!
//! To keep the direct call graph *complete* (so the analysis sees every access and
//! every lock), we abstain on any reachable **indirect call**, any reachable
//! external not on a small **race-inert** allowlist, any unresolved thread entry,
//! and any threading feature we do not model (only `pthread_mutex_lock/unlock` are
//! modeled — trylock / cond / rwlock / spin / sem / barrier / atomics / OpenMP all
//! force abstain via the external allowlist). A thread that may have ≥2 concurrent
//! instances is treated as racing with itself unless it is provably single-instance.
//!
//! Verdict-only (no witness): SV-COMP's frozen validator scores every `no-data-race`
//! violation-*witness* format at 0, and a *correctness* witness is not required for
//! the `true` verdict — so a bare `true` (like [`crate::termination`]) is emitted.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::icfg::Icfg;
use saf_analysis::mta::{MtaAnalysis, MtaConfig, ThreadId};
use saf_analysis::{PtaConfig, PtaContext, PtaResult};

use saf_core::air::{AirModule, Constant, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ObjId, ValueId};

/// Thread-spawn primitives (also configured into MTA so its thread discovery and
/// our spawn gate agree).
const SPAWN_FUNCTIONS: &[&str] = &["pthread_create", "thrd_create"];
/// Thread-join primitives.
const JOIN_FUNCTIONS: &[&str] = &["pthread_join", "thrd_join"];
/// The **only** lock-acquire primitives we model.
const LOCK_FUNCTIONS: &[&str] = &["pthread_mutex_lock"];
/// The **only** lock-release primitives we model.
const UNLOCK_FUNCTIONS: &[&str] = &["pthread_mutex_unlock"];

/// Cost gate: abstain rather than run the O(n²) pairwise conflict scan on a huge
/// access set (large CIL drivers), which could time a sibling task out.
const MAX_ACCESSES: usize = 3000;

/// The exact SV-COMP verdict string for a proven `no-data-race` TRUE — a bare
/// `true` (BenchExec `RESULT_TRUE_PROP`), consistent with [`crate::termination`].
/// Writes no witness (the verify write-gate keys on a `false`-prefix).
#[must_use]
pub fn race_true_verdict() -> &'static str {
    "true"
}

/// Is `name` a **race-inert** external — one that (a) does not create a thread,
/// (b) does not acquire/release any lock we do not model, (c) does not create
/// happens-before, and (d) does not read or write caller-visible memory through a
/// pointer argument in a way the AIR-level access scan would miss?
///
/// The allowlist is deliberately *small*: anything not listed forces abstain. In
/// particular memory-touching libc (`memcpy`/`memset`/`strcpy`/`sprintf`/…) is
/// **excluded** (it could read/write a shared object invisibly) and every
/// unmodeled sync primitive (`pthread_mutex_trylock`, `pthread_cond_*`,
/// `pthread_rwlock_*`, `pthread_spin_*`, `sem_*`, `pthread_barrier_*`,
/// `pthread_once`, C11 `atomic_*` / `__atomic_*` / `__sync_*`, custom
/// `__VERIFIER_atomic_*`) is **excluded**.
///
/// `pthread_mutex_lock`/`pthread_mutex_unlock` ARE listed: their concurrency
/// effect is modeled explicitly by the lockset dataflow, so they are inert to the
/// *external-effect* gate. The modeled spawn/join/init/attr/exit helpers are also
/// listed (single-threaded init before spawn; join only adds happens-before).
#[must_use]
pub fn is_race_inert_external(name: &str) -> bool {
    if name.starts_with("__VERIFIER_nondet_") {
        return true;
    }
    RACE_INERT_EXTERNALS.contains(&name)
}

/// Allowlist backing [`is_race_inert_external`]. Keep tight — recall loss is far
/// cheaper than a wrong `true`.
const RACE_INERT_EXTERNALS: &[&str] = &[
    // Program halt / assertion (halting is not a race).
    "abort",
    "exit",
    "_exit",
    "__assert_fail",
    "reach_error",
    "__VERIFIER_error",
    "__VERIFIER_assume",
    // Allocation (fresh memory; subsequent stores are seen by the access scan).
    "malloc",
    "calloc",
    "realloc",
    "free",
    // Modeled mutex + thread lifecycle (single-threaded init / lock-modeled /
    // join adds happens-before only).
    "pthread_create",
    "thrd_create",
    "pthread_join",
    "thrd_join",
    "pthread_mutex_lock",
    "pthread_mutex_unlock",
    "pthread_mutex_init",
    "pthread_mutex_destroy",
    "pthread_mutexattr_init",
    "pthread_mutexattr_settype",
    "pthread_mutexattr_destroy",
    "pthread_exit",
    "pthread_self",
    "pthread_equal",
];

/// Resolve the thread entry function of a `pthread_create`/`thrd_create` call to a
/// **directly-named** function, or `None` if the entry is indirect / unresolved
/// (⇒ the caller abstains — an unmodeled thread body could race invisibly).
///
/// `pthread_create(&id, attr, start_routine, arg)` — `start_routine` is
/// argument index 2. Resolution mirrors the frontend's function-address encodings:
/// a `Constant::GlobalRef(target)` to a function, or an operand whose raw id *is*
/// a function's [`ObjId`]. PTA-based (points-to) resolution is deliberately NOT
/// used here: this gate exists precisely to reject entries that are only
/// resolvable through pointer flow.
#[must_use]
pub fn resolve_direct_thread_fn(
    module: &AirModule,
    inst: &saf_core::air::Instruction,
) -> Option<FunctionId> {
    let routine = inst.operands.get(2).copied()?;

    // Direct function address: operand's raw id matches a function's ObjId.
    let obj = ObjId::new(routine.raw());
    if let Some(f) = module
        .functions
        .iter()
        .find(|f| ObjId::new(f.id.raw()) == obj)
    {
        return Some(f.id);
    }

    // `Constant::GlobalRef(target)` to a function address.
    if let Some(Constant::GlobalRef(target)) = module.constants.get(&routine) {
        let tobj = ObjId::new(target.raw());
        if let Some(f) = module
            .functions
            .iter()
            .find(|f| ObjId::new(f.id.raw()) == tobj)
        {
            return Some(f.id);
        }
    }
    None
}

/// A statically-resolved concrete lock identity: the base object (a module
/// global) of a **standalone, non-array** global mutex reached WITHOUT any GEP
/// indexing.
///
/// This deliberately-narrow identity is forced by the frontend: it collapses the
/// address of an aggregate sub-object mutex (`&m[i]`, `&m.y`) onto the base
/// object, so two *different* mutexes embedded in one global (an array element or
/// a struct field) become indistinguishable. Admitting such a lock would let two
/// unprotected accesses look "commonly locked" — a wrong `true`. We therefore
/// accept a lock only when it names a whole standalone global mutex, where the
/// base object uniquely IS the mutex. Two locks are the same mutex iff equal.
type LockId = ObjId;

/// A single static memory access performed by some thread, with the sound
/// must-lockset (statically-resolved concrete locks provably held) at that point.
struct Access {
    thread_id: u32,
    ptr: ValueId,
    write: bool,
    /// Concrete lock identities provably must-held here.
    must_locks: BTreeSet<LockId>,
    /// A **main-thread** access that provably executes before ANY thread is
    /// spawned (no `pthread_create` can precede it). Such an access cannot race
    /// (no other thread exists yet), so it is not concurrent with any thread.
    pre_spawn: bool,
    /// This access is performed by the **main** thread (creation site `None`).
    is_main: bool,
    /// For a **main-thread** access: the set of thread **handle keys** that have
    /// provably been `pthread_join`-ed on EVERY path reaching this access (a sound
    /// under-approximation of joined-before). A thread whose handle key is in this
    /// set has terminated (join is a happens-before edge), so it cannot race this
    /// access. Empty for non-main accesses.
    joined_keys: BTreeSet<ValueId>,
}

/// Does the program provably contain **no data race**, by the sufficient
/// structural condition above? Sound and incomplete — `false` here ⇒ the strategy
/// emits `unknown`, never a verdict.
#[must_use]
pub fn program_is_race_free(module: &AirModule) -> bool {
    match race_free_classify(module) {
        Ok(()) => true,
        Err(reason) => {
            if std::env::var_os("SAF_RACE_TRUE_DEBUG").is_some() {
                eprintln!("race_true: abstain: {reason}");
            }
            false
        }
    }
}

/// Core of [`program_is_race_free`]: `Ok(())` = provably race-free, `Err(reason)` =
/// abstain (the reason string is for diagnostics only, never affects the verdict).
#[allow(clippy::too_many_lines)]
fn race_free_classify(module: &AirModule) -> Result<(), String> {
    // (T) a defined `main`.
    let Some(main_func) = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)
    else {
        return Err("no-defined-main".into());
    };
    let main_id = main_func.id;

    // Global constructors/destructors run outside main's call graph and could
    // spawn or race — abstain if any exist (defense-in-depth, as termination does).
    if module
        .globals
        .iter()
        .any(|g| g.name == "llvm.global_ctors" || g.name == "llvm.global_dtors")
    {
        return Err("global-ctors-dtors".into());
    }

    let cg = CallGraph::build(module);
    if cg.node_for_function(main_id).is_none() {
        return Err("main-not-in-callgraph".into());
    }

    // Points-to (Andersen, over-approximate) + MTA thread/concurrency model. MTA's
    // thread-entry discovery is PTA-driven, so it is a SOUND *superset* of the real
    // entry functions — we never miss a thread body (only possibly over-analyze).
    let pta = {
        let mut ctx = PtaContext::new(PtaConfig::default());
        let raw = ctx.analyze(module);
        PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics)
    };
    let icfg = Icfg::build(module, &cg);
    let mta_config = MtaConfig {
        thread_create_funcs: SPAWN_FUNCTIONS.iter().map(|s| (*s).to_string()).collect(),
        thread_join_funcs: JOIN_FUNCTIONS.iter().map(|s| (*s).to_string()).collect(),
        ..MtaConfig::default()
    };
    let mta = MtaAnalysis::with_pta(
        module,
        &cg,
        &icfg,
        mta_config,
        pta.points_to_map(),
        pta.location_factory(),
    )
    .analyze();
    let threads = &mta.thread_graph.threads;

    // The set of functions any thread may execute: reachable (direct call graph)
    // from `main` OR from any discovered thread entry. `pthread_create` is opaque,
    // so a thread body is generally *not* reachable from `main` in the call graph —
    // union the per-entry reachable sets to see every access and every lock.
    let mut reachable_fids: BTreeSet<FunctionId> = reachable_functions(&cg, main_id);
    for tctx in threads.values() {
        reachable_fids.extend(reachable_functions(&cg, tctx.entry_function));
    }

    // (E) every reachable external must be race-inert; (I) no reachable defined
    // body may contain a `CallIndirect` (an unresolved target could hide a spawn,
    // an access, a lock, or a non-modeled sync primitive); (CFG) no dropped
    // terminator. Checked over the FULL reachable set (main + every thread body).
    for func in &module.functions {
        if !reachable_fids.contains(&func.id) {
            continue;
        }
        if func.is_declaration {
            if !is_race_inert_external(&func.name) {
                return Err(format!("non-inert-external:{}", func.name));
            }
            continue;
        }
        for block in &func.blocks {
            if block.terminator().is_none() {
                return Err("dropped-terminator".into());
            }
            if block
                .instructions
                .iter()
                .any(|inst| matches!(inst.op, Operation::CallIndirect { .. }))
            {
                return Err(format!("reachable-indirect-call:{}", func.name));
            }
        }
    }

    let has_reachable_spawn = reachable_spawns(module, &reachable_fids);

    // No reachable spawn — and, since the body check above passed, no reachable
    // indirect call that could hide one — ⇒ genuinely sequential ⇒ no data race.
    if !has_reachable_spawn {
        return Ok(());
    }
    // A spawn is reachable but MTA modeled ≤1 thread — do not trust a sequential
    // conclusion, abstain.
    if threads.len() <= 1 {
        return Err("spawn-but-mta-single-thread".into());
    }
    // Completeness: every reachable spawn *site* must correspond to a discovered
    // thread context. A spawn MTA failed to model (e.g. an unresolved routine
    // pointer with empty points-to) could hide a racing thread ⇒ abstain.
    if !every_spawn_is_modeled(module, &reachable_fids, threads) {
        return Err("spawn-not-modeled".into());
    }

    // Precompute which user functions may (transitively) touch a lock/unlock — a
    // call to one voids the caller's must-lockset (it might release a held lock).
    let sync_touching = functions_touching_sync(module, &cg, &reachable_fids);
    // Lock-pointer resolution context (def map + GEP-indexed globals).
    let mut res = LockResolver::build(module);

    // Second phase: net-acquire summaries for pure-acquire wrapper functions
    // (`lock()` helpers). A function that may (transitively) unlock is excluded —
    // carrying its summary would be unsound (it might release a caller lock). The
    // summaries are computed with the phase-1 resolver (empty summaries), so a
    // wrapper that itself calls another wrapper simply gets no summary (sound).
    let unlock_reaching = functions_reaching_unlock(module, &cg, &reachable_fids);
    let mut acquire_summaries: BTreeMap<FunctionId, BTreeSet<LockId>> = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration
            || !reachable_fids.contains(&func.id)
            || unlock_reaching.contains(&func.id)
        {
            continue;
        }
        let acq = function_net_acquire(func, &res, &sync_touching, module);
        if !acq.is_empty() {
            acquire_summaries.insert(func.id, acq);
        }
    }
    res.acquire_summaries = acquire_summaries;

    // Per-function sound must-lockset dataflow.
    // Instructions in `main` that provably execute before any `pthread_create`.
    let main_prespawn = prespawn_insts(module, main_id);

    // Join happens-before: resolve each spawned thread's handle object, and the
    // set of handles provably `pthread_join`-ed on entry to each `main` block. A
    // main-thread access dominated by a join of thread `t` is NOT concurrent with
    // `t` (join is a happens-before edge — `t` has terminated). Handle resolution
    // is fail-closed: an unresolved handle, a handle shared by ≥2 thread contexts
    // (sequential reuse / handle arrays), or a recurrent thread never gets join
    // credit.
    let (thread_handles, ambiguous_handles) = build_thread_handles(module, threads, &res.defs);
    let main_joined_in = compute_main_joined_in(main_func, &res.defs, module);

    let mut accesses: Vec<Access> = Vec::new();
    for (tid, tctx) in threads {
        let is_main = tctx.creation_site.is_none();
        let thread_fns = reachable_functions(&cg, tctx.entry_function);
        for func in &module.functions {
            if func.is_declaration || !thread_fns.contains(&func.id) {
                continue;
            }
            let in_main_body = is_main && func.id == main_id;
            let locksets = compute_function_locksets(func, &res, &sync_touching, module);
            for block in &func.blocks {
                let mut held = locksets.get(&block.id).cloned().unwrap_or_default();
                // Running must-joined handle set within this main block (starts at
                // the block-entry must-join; a join call adds its handle for every
                // subsequent instruction).
                let mut joined_here: BTreeSet<ValueId> = if in_main_body {
                    main_joined_in.get(&block.id).cloned().unwrap_or_default()
                } else {
                    BTreeSet::new()
                };
                for inst in &block.instructions {
                    let pre_spawn = in_main_body && main_prespawn.contains(&inst.id);
                    // (pointer, is_write) for each memory access this instruction
                    // performs. `Store`: operand[1] is the pointer; `memcpy`/`memset`
                    // write operand[0] and `memcpy` reads operand[1].
                    let mut ops: Vec<(ValueId, bool)> = Vec::new();
                    match &inst.op {
                        Operation::Load => ops.extend(inst.operands.first().map(|p| (*p, false))),
                        Operation::Store => ops.extend(inst.operands.get(1).map(|p| (*p, true))),
                        Operation::Memcpy => {
                            ops.extend(inst.operands.first().map(|p| (*p, true)));
                            ops.extend(inst.operands.get(1).map(|p| (*p, false)));
                        }
                        Operation::Memset => ops.extend(inst.operands.first().map(|p| (*p, true))),
                        _ => {}
                    }
                    for (ptr, write) in ops {
                        accesses.push(Access {
                            thread_id: tid.0,
                            ptr,
                            write,
                            must_locks: held.clone(),
                            pre_spawn,
                            is_main,
                            joined_keys: if is_main {
                                joined_here.clone()
                            } else {
                                BTreeSet::new()
                            },
                        });
                    }
                    apply_lock_transfer(inst, &res, &sync_touching, module, &mut held);
                    // A join call establishes happens-before for every LATER access
                    // in this block (and, via `main_joined_in`, later blocks).
                    if in_main_body {
                        if let Some(key) = join_key_of(inst, &res.defs, module) {
                            joined_here.insert(key);
                        }
                    }
                    if accesses.len() > MAX_ACCESSES {
                        return Err("cost-gate-too-many-accesses".into()); // cost gate
                    }
                }
            }
        }
    }

    // Threads that may have ≥2 concurrent instances (race with themselves) — a
    // SOUND over-approximation: a thread is treated as single-instance only when
    // its spawn is directly in `main` and not inside a CFG loop.
    let recurrent = recurrent_thread_ids(module, &mta, main_id);

    // Pairwise conflict scan. Any conflicting pair without a common unique lock is
    // not provably race-free ⇒ abstain.
    for (i, a) in accesses.iter().enumerate() {
        for b in &accesses[i..] {
            let same_thread = a.thread_id == b.thread_id;
            // Concurrency is over-approximated: TWO DISTINCT threads are treated as
            // possibly-concurrent UNLESS a sound happens-before edge separates them.
            // We do NOT trust MTA HB to *rule out* an overlap. The sound exceptions:
            //  * a main-thread access that provably precedes every spawn
            //    (`pre_spawn`) — no other thread exists yet; or
            //  * a main-thread access provably dominated by a `pthread_join` of the
            //    other thread (`joined_before`) — that thread has terminated. Only
            //    for a single-instance thread with an unambiguously-resolved handle;
            //    a recurrent / array / reused handle gets no join credit.
            // A single thread conflicts with a sibling instance only when recurrent.
            let concurrent = if same_thread {
                recurrent.contains(&a.thread_id)
            } else {
                !(a.pre_spawn
                    || b.pre_spawn
                    || joined_before(
                        a,
                        b.thread_id,
                        &thread_handles,
                        &ambiguous_handles,
                        &recurrent,
                    )
                    || joined_before(
                        b,
                        a.thread_id,
                        &thread_handles,
                        &ambiguous_handles,
                        &recurrent,
                    ))
            };
            if !concurrent || (!a.write && !b.write) {
                continue;
            }
            if !pta.may_alias(a.ptr, b.ptr).may_alias_conservative() {
                continue;
            }
            // Conflicting pair — require a provably-common unique lock.
            if a.must_locks.is_disjoint(&b.must_locks) {
                return Err("conflicting-pair-no-common-lock".into());
            }
        }
    }

    Ok(())
}

/// Forward, flow-sensitive, intra-procedural **must**-lockset per basic block
/// (concrete lock identities provably held on entry to the block). Sound: `lock`
/// adds only a statically-resolved concrete lock, `unlock` removes the lock it
/// resolves to (or clears when its target is ambiguous), a sync-touching call
/// clears the set, and the block-entry meet is set **intersection**.
fn compute_function_locksets(
    func: &saf_core::air::AirFunction,
    res: &LockResolver<'_>,
    sync_touching: &BTreeSet<FunctionId>,
    module: &AirModule,
) -> BTreeMap<BlockId, BTreeSet<LockId>> {
    let cfg = Cfg::build(func);
    let entry = cfg.entry;

    // `None` = ⊤ (uninitialized / unreachable) — identity for intersection.
    let mut block_in: BTreeMap<BlockId, Option<BTreeSet<LockId>>> = BTreeMap::new();
    for block in &func.blocks {
        block_in.insert(block.id, None);
    }
    block_in.insert(entry, Some(BTreeSet::new()));

    // Bounded fixpoint (must-lattice height ≤ #blocks × #locks; sets only shrink).
    let cap = func
        .blocks
        .len()
        .saturating_mul(func.blocks.len())
        .saturating_add(4);
    let mut changed = true;
    let mut rounds = 0;
    while changed && rounds < cap {
        changed = false;
        rounds += 1;
        for block in &func.blocks {
            // Meet of predecessor OUT states.
            let meet = if block.id == entry {
                Some(BTreeSet::new())
            } else {
                let mut acc: Option<BTreeSet<LockId>> = None;
                if let Some(preds) = cfg.predecessors.get(&block.id) {
                    for p in preds {
                        let pred_out = block_out(module, func, *p, &block_in, res, sync_touching);
                        acc = match (acc, pred_out) {
                            (None, x) | (x, None) => x,
                            (Some(a), Some(b)) => Some(a.intersection(&b).copied().collect()),
                        };
                    }
                }
                acc
            };
            if block_in.get(&block.id).and_then(Clone::clone) != meet {
                block_in.insert(block.id, meet);
                changed = true;
            }
        }
    }

    func.blocks
        .iter()
        .map(|b| {
            (
                b.id,
                block_in
                    .get(&b.id)
                    .and_then(Clone::clone)
                    .unwrap_or_default(),
            )
        })
        .collect()
}

/// Compute the OUT must-lockset of block `bid` by applying every instruction's
/// transfer to its (already-computed) IN state.
fn block_out(
    module: &AirModule,
    func: &saf_core::air::AirFunction,
    bid: BlockId,
    block_in: &BTreeMap<BlockId, Option<BTreeSet<LockId>>>,
    res: &LockResolver<'_>,
    sync_touching: &BTreeSet<FunctionId>,
) -> Option<BTreeSet<LockId>> {
    let mut state = block_in.get(&bid).and_then(Clone::clone)?;
    if let Some(block) = func.blocks.iter().find(|b| b.id == bid) {
        for inst in &block.instructions {
            apply_lock_transfer(inst, res, sync_touching, module, &mut state);
        }
    }
    Some(state)
}

/// Apply one instruction's effect to the running must-lockset.
fn apply_lock_transfer(
    inst: &saf_core::air::Instruction,
    res: &LockResolver<'_>,
    sync_touching: &BTreeSet<FunctionId>,
    module: &AirModule,
    state: &mut BTreeSet<LockId>,
) {
    let Operation::CallDirect { callee } = &inst.op else {
        return;
    };
    let Some(target) = module.function(*callee) else {
        return;
    };
    let name = target.name.as_str();
    if LOCK_FUNCTIONS.contains(&name) {
        if let Some(p) = inst.operands.first().copied() {
            // Only a statically-resolved concrete lock adds must-knowledge; an
            // ambiguous acquire (unknown index / merged pointer / runtime value)
            // adds nothing (fail-closed — that pair will look unprotected).
            if let Some(id) = lock_identity(p, res, module) {
                state.insert(id);
            }
        }
    } else if UNLOCK_FUNCTIONS.contains(&name) {
        if let Some(p) = inst.operands.first().copied() {
            match lock_identity(p, res, module) {
                // Precise release: drop exactly that lock.
                Some(id) => {
                    state.remove(&id);
                }
                // Ambiguous release (`unlock(m)`, `m` may be several mutexes): it
                // MAY release any held lock — clear the whole set (sound
                // under-approximation of what stays held).
                None => state.clear(),
            }
        }
    } else if !target.is_declaration && sync_touching.contains(callee) {
        // A call to a sync-touching user function. If it is a pure-acquire wrapper
        // (a `lock()` helper) with a net-acquire summary, ADD the locks it holds on
        // return; otherwise it may release a held lock, so void our must-knowledge.
        match res.acquire_summaries.get(callee) {
            Some(acq) => state.extend(acq.iter().copied()),
            None => state.clear(),
        }
    }
}

/// Map from a value to the instruction that defines it (has it as `dst`).
type DefMap<'a> = BTreeMap<ValueId, &'a saf_core::air::Instruction>;

/// Resolution context for lock-pointer identity: the value → defining-instruction
/// map plus the set of globals that are ever addressed via an indexing GEP.
struct LockResolver<'a> {
    defs: DefMap<'a>,
    /// Globals a lock may NOT be soundly attributed to, because they hold more
    /// than one mutex whose sub-object addresses the frontend collapses onto the
    /// base object (so distinct mutexes become indistinguishable). A global is
    /// rejected if EITHER:
    /// - it is the base of any indexing GEP (an *array* element / struct field
    ///   accessed via a preserved GEP), OR
    /// - it is the base of ≥2 distinct `pthread_mutex_init` call sites (a *struct*
    ///   of several mutexes whose field GEPs the frontend dropped entirely — the
    ///   only surviving evidence of multiplicity is the repeated init).
    ambiguous_globals: BTreeSet<ObjId>,
    /// Net **acquire** summary per user function: the concrete locks it provably
    /// holds on EVERY return path and never releases (a `lock()`-style wrapper).
    /// A call to such a function adds those locks to the caller's must-lockset.
    /// Populated in a second phase; empty during that phase's own computation.
    acquire_summaries: BTreeMap<FunctionId, BTreeSet<LockId>>,
}

impl<'a> LockResolver<'a> {
    fn build(module: &'a AirModule) -> Self {
        let mut defs: DefMap<'a> = BTreeMap::new();
        for func in &module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Some(dst) = inst.dst {
                        defs.entry(dst).or_insert(inst);
                    }
                }
            }
        }
        let mut ambiguous_globals: BTreeSet<ObjId> = BTreeSet::new();
        // Count `pthread_mutex_init` sites per base global.
        let mut init_counts: BTreeMap<ObjId, u32> = BTreeMap::new();
        for func in &module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match &inst.op {
                        // A base reached by an indexing GEP is an aggregate.
                        Operation::Gep { field_path }
                            if gep_has_indexing(field_path, inst.operands.len()) =>
                        {
                            if let Some(base) = inst.operands.first().copied() {
                                if let Some(obj) = resolve_global_base(base, &defs, module) {
                                    ambiguous_globals.insert(obj);
                                }
                            }
                        }
                        // Tally mutex-init sites per base global.
                        Operation::CallDirect { callee } => {
                            if module
                                .function(*callee)
                                .is_some_and(|f| f.name == "pthread_mutex_init")
                            {
                                if let Some(p) = inst.operands.first().copied() {
                                    if let Some(obj) = resolve_global_base(p, &defs, module) {
                                        *init_counts.entry(obj).or_insert(0) += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        for (obj, count) in init_counts {
            if count >= 2 {
                ambiguous_globals.insert(obj);
            }
        }
        Self {
            defs,
            ambiguous_globals,
            acquire_summaries: BTreeMap::new(),
        }
    }
}

/// Functions that may (transitively) call `pthread_mutex_unlock` — a call to one
/// may release a lock the caller holds, so we cannot soundly carry the caller's
/// must-lockset across it (we clear instead). Pure-acquire wrappers are exactly
/// the sync-touching functions NOT in this set.
fn functions_reaching_unlock(
    module: &AirModule,
    cg: &CallGraph,
    reachable: &BTreeSet<FunctionId>,
) -> BTreeSet<FunctionId> {
    let mut direct: BTreeSet<FunctionId> = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        'outer: for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if module
                        .function(*callee)
                        .is_some_and(|t| UNLOCK_FUNCTIONS.contains(&t.name.as_str()))
                    {
                        direct.insert(func.id);
                        break 'outer;
                    }
                }
            }
        }
    }
    let mut result = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        if reachable_functions(cg, func.id)
            .iter()
            .any(|f| direct.contains(f))
        {
            result.insert(func.id);
        }
    }
    result
}

/// Resolve a pointer value to the **canonical id of the memory object** it names —
/// a module global's id or a stack `alloca`'s result id — forwarding through
/// address-preserving casts/copies and no-op (non-indexing) GEPs. Returns `None`
/// on ANY ambiguity (an indexing GEP into an aggregate — e.g. a `pthread_t`
/// handle *array* element — a `phi`/`select`/`load`ed pointer, or a non-object
/// base). This is the fail-closed identity used for thread-handle matching.
fn resolve_ptr_base(v: ValueId, defs: &DefMap<'_>, module: &AirModule) -> Option<ValueId> {
    let mut cur = v;
    for _ in 0..64 {
        if module.globals.iter().any(|g| g.id == cur) {
            return Some(cur);
        }
        if let Some(Constant::GlobalRef(target)) = module.constants.get(&cur) {
            return module
                .globals
                .iter()
                .find(|g| g.id == *target)
                .map(|g| g.id);
        }
        let inst = defs.get(&cur)?;
        match &inst.op {
            // A stack slot: its result id uniquely identifies the object.
            Operation::Alloca { .. } => return Some(cur),
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                cur = inst.operands.first().copied()?;
            }
            Operation::Gep { field_path } if !gep_has_indexing(field_path, inst.operands.len()) => {
                cur = inst.operands.first().copied()?;
            }
            _ => return None,
        }
    }
    None
}

/// The **handle key** joined by a `pthread_join(thread, retval)` call: `thread` is
/// a `pthread_t` *value*, in practice `load %handle`. Follow copies/casts to the
/// defining `Load` and resolve the base object of the pointer it loaded from — the
/// same object a matching `pthread_create(&handle, …)` stores into. `None` if the
/// value is not a load of a resolvable handle object (⇒ no join credit).
fn resolve_join_key(joinop: ValueId, defs: &DefMap<'_>, module: &AirModule) -> Option<ValueId> {
    let mut cur = joinop;
    for _ in 0..64 {
        let inst = defs.get(&cur)?;
        match &inst.op {
            Operation::Load => {
                return resolve_ptr_base(inst.operands.first().copied()?, defs, module);
            }
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                cur = inst.operands.first().copied()?;
            }
            _ => return None,
        }
    }
    None
}

/// If `inst` is a `pthread_join`/`thrd_join` call, the resolved handle key it joins.
fn join_key_of(
    inst: &saf_core::air::Instruction,
    defs: &DefMap<'_>,
    module: &AirModule,
) -> Option<ValueId> {
    let Operation::CallDirect { callee } = &inst.op else {
        return None;
    };
    if !module
        .function(*callee)
        .is_some_and(|f| JOIN_FUNCTIONS.contains(&f.name.as_str()))
    {
        return None;
    }
    resolve_join_key(inst.operands.first().copied()?, defs, module)
}

/// Resolve each spawned thread's `pthread_create` **handle key** (the object
/// `&handle` addresses). Returns `(tid → handle key, ambiguous keys)`. A key used
/// by ≥2 distinct thread contexts (a reused `pthread_t`, or a handle-array element
/// the frontend collapsed) is ambiguous — join credit on it could match the wrong
/// thread, so it is excluded (fail-closed).
fn build_thread_handles(
    module: &AirModule,
    threads: &BTreeMap<ThreadId, saf_analysis::mta::ThreadContext>,
    defs: &DefMap<'_>,
) -> (BTreeMap<u32, ValueId>, BTreeSet<ValueId>) {
    let mut handles: BTreeMap<u32, ValueId> = BTreeMap::new();
    let mut uses: BTreeMap<ValueId, u32> = BTreeMap::new();
    for (tid, tctx) in threads {
        let Some(site) = tctx.creation_site else {
            continue;
        };
        let Some(inst) = find_inst(module, site) else {
            continue;
        };
        let Some(hptr) = inst.operands.first().copied() else {
            continue;
        };
        if let Some(key) = resolve_ptr_base(hptr, defs, module) {
            handles.insert(tid.0, key);
            *uses.entry(key).or_insert(0) += 1;
        }
    }
    let ambiguous: BTreeSet<ValueId> = uses
        .into_iter()
        .filter_map(|(k, c)| (c >= 2).then_some(k))
        .collect();
    (handles, ambiguous)
}

/// Must-join dataflow over `main`: the handle keys provably `pthread_join`-ed on
/// EVERY path to the entry of each block (block-entry meet is set intersection).
/// Sound under-approximation of joined-before: a conditional join is dropped at the
/// merge. Within-block joins are added by the caller as it walks instructions.
fn compute_main_joined_in(
    main_func: &saf_core::air::AirFunction,
    defs: &DefMap<'_>,
    module: &AirModule,
) -> BTreeMap<BlockId, BTreeSet<ValueId>> {
    let cfg = Cfg::build(main_func);
    let entry = cfg.entry;

    // Per-block generated joins (handle keys joined somewhere in the block).
    let join_gen: BTreeMap<BlockId, BTreeSet<ValueId>> = main_func
        .blocks
        .iter()
        .map(|b| {
            let mut keys = BTreeSet::new();
            for inst in &b.instructions {
                if let Some(k) = join_key_of(inst, defs, module) {
                    keys.insert(k);
                }
            }
            (b.id, keys)
        })
        .collect();

    // `None` = ⊤ (unreachable) — identity for intersection.
    let mut block_in: BTreeMap<BlockId, Option<BTreeSet<ValueId>>> = BTreeMap::new();
    for block in &main_func.blocks {
        block_in.insert(block.id, None);
    }
    block_in.insert(entry, Some(BTreeSet::new()));

    let cap = main_func
        .blocks
        .len()
        .saturating_mul(main_func.blocks.len())
        .saturating_add(4);
    let mut changed = true;
    let mut rounds = 0;
    while changed && rounds < cap {
        changed = false;
        rounds += 1;
        for block in &main_func.blocks {
            if block.id == entry {
                continue;
            }
            let mut acc: Option<BTreeSet<ValueId>> = None;
            if let Some(preds) = cfg.predecessors.get(&block.id) {
                for p in preds {
                    // OUT[p] = IN[p] ∪ join_gen[p].
                    let pred_out = block_in.get(p).and_then(Clone::clone).map(|mut s| {
                        if let Some(g) = join_gen.get(p) {
                            s.extend(g.iter().copied());
                        }
                        s
                    });
                    acc = match (acc, pred_out) {
                        (None, x) | (x, None) => x,
                        (Some(a), Some(b)) => Some(a.intersection(&b).copied().collect()),
                    };
                }
            }
            if block_in.get(&block.id).and_then(Clone::clone) != acc {
                block_in.insert(block.id, acc);
                changed = true;
            }
        }
    }

    main_func
        .blocks
        .iter()
        .map(|b| {
            (
                b.id,
                block_in
                    .get(&b.id)
                    .and_then(Clone::clone)
                    .unwrap_or_default(),
            )
        })
        .collect()
}

/// Is `main_acc` (a main-thread access) provably separated from thread `other_tid`
/// by a `pthread_join` happens-before edge? True iff `other_tid` is single-instance
/// (not recurrent) with an unambiguously-resolved handle key that `main_acc` has
/// provably joined-before.
fn joined_before(
    main_acc: &Access,
    other_tid: u32,
    handles: &BTreeMap<u32, ValueId>,
    ambiguous: &BTreeSet<ValueId>,
    recurrent: &BTreeSet<u32>,
) -> bool {
    if !main_acc.is_main || recurrent.contains(&other_tid) {
        return false;
    }
    match handles.get(&other_tid) {
        Some(key) if !ambiguous.contains(key) => main_acc.joined_keys.contains(key),
        _ => false,
    }
}

/// Find the instruction with id `target` anywhere in the module.
fn find_inst(module: &AirModule, target: InstId) -> Option<&saf_core::air::Instruction> {
    module
        .functions
        .iter()
        .flat_map(|f| f.blocks.iter())
        .flat_map(|b| b.instructions.iter())
        .find(|inst| inst.id == target)
}

/// The net-acquired lock set of `func`: locks provably held on EVERY return path,
/// computed by intersecting the must-lockset at each `Ret`. Only meaningful for a
/// function that never (transitively) unlocks — for those the returned locks are
/// genuinely still held by the caller after the call.
fn function_net_acquire(
    func: &saf_core::air::AirFunction,
    res: &LockResolver<'_>,
    sync_touching: &BTreeSet<FunctionId>,
    module: &AirModule,
) -> BTreeSet<LockId> {
    let locksets = compute_function_locksets(func, res, sync_touching, module);
    let mut result: Option<BTreeSet<LockId>> = None;
    for block in &func.blocks {
        let ends_in_ret = block
            .instructions
            .last()
            .is_some_and(|i| matches!(i.op, Operation::Ret));
        if !ends_in_ret {
            continue;
        }
        let mut held = locksets.get(&block.id).cloned().unwrap_or_default();
        for inst in &block.instructions {
            apply_lock_transfer(inst, res, sync_touching, module, &mut held);
        }
        result = Some(match result {
            None => held,
            Some(acc) => acc.intersection(&held).copied().collect(),
        });
    }
    result.unwrap_or_default()
}

/// Walk casts/no-op GEPs to the base global object a pointer denotes, if any.
fn resolve_global_base(p: ValueId, defs: &DefMap<'_>, module: &AirModule) -> Option<ObjId> {
    let mut cur = p;
    for _ in 0..64 {
        if let Some(g) = module.globals.iter().find(|g| g.id == cur) {
            return Some(g.obj);
        }
        if let Some(Constant::GlobalRef(t)) = module.constants.get(&cur) {
            return module.globals.iter().find(|g| g.id == *t).map(|g| g.obj);
        }
        let inst = defs.get(&cur)?;
        match &inst.op {
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                cur = inst.operands.first().copied()?;
            }
            Operation::Gep { field_path } if !gep_has_indexing(field_path, inst.operands.len()) => {
                cur = inst.operands.first().copied()?;
            }
            _ => return None,
        }
    }
    None
}

/// Does this GEP address a sub-object (any struct-field or array-index step, or an
/// extra index operand)? A pure base-only GEP (`field_path` empty and only the
/// base operand) is address-preserving and does NOT index.
fn gep_has_indexing(field_path: &saf_core::air::FieldPath, num_operands: usize) -> bool {
    !field_path.steps.is_empty() || num_operands > 1
}

/// Resolve a lock pointer `p` to its base **standalone non-array global mutex**,
/// or `None` (⇒ the lock is not counted). Returns `None` on ANY ambiguity: a GEP
/// with indexing (a struct-field / array-element sub-object address the frontend
/// collapses onto the base), a base global of *array* type (whose element
/// addresses collapse entirely onto the base), a merged/`phi`/`select` pointer, a
/// function parameter, a value loaded from memory, pointer arithmetic, or a base
/// that is not a statically-allocated global.
///
/// This is the soundness core: two locks are treated as the SAME mutex only when
/// they resolve to the SAME standalone global object — so the frontend's collapse
/// of `&m[3]`/`&m[4]` (and `&m.x`/`&m.y`) can never make two distinct
/// aggregate-embedded mutexes look common.
fn lock_identity(p: ValueId, res: &LockResolver<'_>, module: &AirModule) -> Option<LockId> {
    let mut cur = p;
    for _ in 0..64 {
        // Base case: a direct use of a global's address (⇒ the WHOLE global).
        if let Some(g) = module.globals.iter().find(|g| g.id == cur) {
            return standalone_mutex_obj(res, g.obj);
        }
        // Base case: a `GlobalRef` constant to a global's address.
        if let Some(Constant::GlobalRef(target)) = module.constants.get(&cur) {
            let g = module.globals.iter().find(|g| g.id == *target)?;
            return standalone_mutex_obj(res, g.obj);
        }
        let inst = res.defs.get(&cur)?;
        match &inst.op {
            // Any GEP with indexing addresses a sub-object whose offset the AIR does
            // not faithfully preserve — reject. A no-op GEP (base only) forwards.
            Operation::Gep { field_path } => {
                if gep_has_indexing(field_path, inst.operands.len()) {
                    return None;
                }
                cur = inst.operands.first().copied()?;
            }
            // Address-preserving forwards.
            Operation::Cast { .. } | Operation::Copy | Operation::Freeze => {
                cur = inst.operands.first().copied()?;
            }
            // Anything else (phi/select/load/binop/param/call) is ambiguous.
            _ => return None,
        }
    }
    None
}

/// The object id `obj` iff it is a valid standalone-mutex lock base — i.e. NOT in
/// [`LockResolver::ambiguous_globals`] (not GEP-indexed and not multiply-init'd).
/// Admitting an aggregate global whose distinct embedded mutexes the frontend has
/// collapsed onto one base would conflate them into a bogus common lock.
fn standalone_mutex_obj(res: &LockResolver<'_>, obj: ObjId) -> Option<LockId> {
    if res.ambiguous_globals.contains(&obj) {
        None
    } else {
        Some(obj)
    }
}

/// Functions that may (transitively) call a lock/unlock primitive — a call to one
/// invalidates the caller's must-lockset.
fn functions_touching_sync(
    module: &AirModule,
    cg: &CallGraph,
    reachable: &BTreeSet<FunctionId>,
) -> BTreeSet<FunctionId> {
    // Functions that DIRECTLY call a lock/unlock primitive.
    let mut direct: BTreeSet<FunctionId> = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        'outer: for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(t) = module.function(*callee) {
                        if LOCK_FUNCTIONS.contains(&t.name.as_str())
                            || UNLOCK_FUNCTIONS.contains(&t.name.as_str())
                        {
                            direct.insert(func.id);
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    // Propagate: F touches sync if any function reachable from F does directly.
    let mut result = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        if reachable_functions(cg, func.id)
            .iter()
            .any(|f| direct.contains(f))
        {
            result.insert(func.id);
        }
    }
    result
}

/// Threads that may have ≥2 concurrent instances (race with a sibling instance of
/// themselves). SOUND over-approximation: a thread is single-instance **only**
/// when it is the ONLY thread with its entry function AND its `pthread_create` is
/// directly in `main` and not inside a CFG loop; every other spawn shape (a shared
/// entry spawned from ≥2 sites, a loop-spawned entry, a spawn nested in a helper)
/// is treated as recurrent. This catches e.g. `create(t); create(t);` where two
/// instances of `t` race on an unprotected global.
fn recurrent_thread_ids(
    module: &AirModule,
    mta: &saf_analysis::mta::MtaResult,
    main_id: FunctionId,
) -> BTreeSet<u32> {
    let threads = &mta.thread_graph.threads;
    // Entry functions shared by ≥2 spawned thread contexts (multi-instance).
    let mut entry_counts: BTreeMap<FunctionId, u32> = BTreeMap::new();
    for tctx in threads.values() {
        if tctx.creation_site.is_some() {
            *entry_counts.entry(tctx.entry_function).or_insert(0) += 1;
        }
    }
    let main_cfg = module
        .functions
        .iter()
        .find(|f| f.id == main_id)
        .map(Cfg::build);
    let mut recurrent = BTreeSet::new();
    for (tid, tctx) in threads {
        if tctx.creation_site.is_none() {
            continue; // main thread itself
        }
        // Multiple contexts share this entry ⇒ ≥2 concurrent instances.
        if entry_counts.get(&tctx.entry_function).copied().unwrap_or(0) >= 2 {
            recurrent.insert(tid.0);
            continue;
        }
        let single = match (tctx.creation_site, &main_cfg) {
            (Some(site), Some(cfg)) => match locate_inst(module, site) {
                Some((fid, bid)) => fid == main_id && !block_in_cycle(cfg, bid),
                None => false,
            },
            _ => false,
        };
        if !single {
            recurrent.insert(tid.0);
        }
    }
    recurrent
}

/// Does every reachable thread-spawn *site* correspond to at least one discovered
/// thread context (matched by creation-site instruction id)? A spawn MTA failed to
/// model would leave a racing thread invisible ⇒ the caller must abstain.
fn every_spawn_is_modeled(
    module: &AirModule,
    reachable: &BTreeSet<FunctionId>,
    threads: &BTreeMap<ThreadId, saf_analysis::mta::ThreadContext>,
) -> bool {
    let modeled_sites: BTreeSet<InstId> =
        threads.values().filter_map(|t| t.creation_site).collect();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(t) = module.function(*callee) {
                        if SPAWN_FUNCTIONS.contains(&t.name.as_str())
                            && !modeled_sites.contains(&inst.id)
                        {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}

/// Instructions in `main`'s own body that provably execute BEFORE any
/// `pthread_create` on every path — i.e. while the program is still single-
/// threaded. An access here cannot race (no other thread exists yet).
///
/// Sound (under-approximates pre-spawn): an instruction is pre-spawn only if no
/// spawn can reach it. We compute the set of "post-spawn" instructions — those in
/// a spawn block at/after the spawn, or in any block forward-reachable from a
/// spawn block — and return the complement over `main`'s instructions.
fn prespawn_insts(module: &AirModule, main_id: FunctionId) -> BTreeSet<InstId> {
    let Some(main_func) = module.functions.iter().find(|f| f.id == main_id) else {
        return BTreeSet::new();
    };
    let cfg = Cfg::build(main_func);

    // Blocks that directly contain a spawn call.
    let mut spawn_blocks: BTreeSet<BlockId> = BTreeSet::new();
    for block in &main_func.blocks {
        if block.instructions.iter().any(is_spawn_call_of(module)) {
            spawn_blocks.insert(block.id);
        }
    }

    // Blocks reachable AFTER a spawn: forward closure from spawn blocks'
    // successors (a spawn block itself is only partly post-spawn — split below).
    let mut post_blocks: BTreeSet<BlockId> = BTreeSet::new();
    let mut stack: Vec<BlockId> = spawn_blocks
        .iter()
        .filter_map(|b| cfg.successors.get(b))
        .flat_map(|s| s.iter().copied())
        .collect();
    while let Some(b) = stack.pop() {
        if !post_blocks.insert(b) {
            continue;
        }
        if let Some(succs) = cfg.successors.get(&b) {
            stack.extend(succs.iter().copied());
        }
    }

    let mut pre: BTreeSet<InstId> = BTreeSet::new();
    for block in &main_func.blocks {
        if post_blocks.contains(&block.id) {
            continue; // whole block is post-spawn
        }
        // Within a block, instructions up to the FIRST spawn are pre-spawn; the
        // spawn and everything after it are post-spawn.
        let mut seen_spawn = false;
        for inst in &block.instructions {
            if seen_spawn {
                break;
            }
            if is_spawn_call_of(module)(inst) {
                seen_spawn = true;
                continue;
            }
            pre.insert(inst.id);
        }
    }
    pre
}

/// Predicate: is `inst` a direct call to a thread-spawn primitive?
fn is_spawn_call_of(module: &AirModule) -> impl Fn(&saf_core::air::Instruction) -> bool + '_ {
    move |inst| {
        matches!(&inst.op, Operation::CallDirect { callee }
            if module.function(*callee).is_some_and(|f| SPAWN_FUNCTIONS.contains(&f.name.as_str())))
    }
}

/// Reachable spawn present among the reachable defined functions?
fn reachable_spawns(module: &AirModule, reachable: &BTreeSet<FunctionId>) -> bool {
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(t) = module.function(*callee) {
                        if SPAWN_FUNCTIONS.contains(&t.name.as_str()) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Locate the `(function, block)` containing instruction `target`.
fn locate_inst(module: &AirModule, target: InstId) -> Option<(FunctionId, BlockId)> {
    for func in &module.functions {
        for block in &func.blocks {
            if block.instructions.iter().any(|inst| inst.id == target) {
                return Some((func.id, block.id));
            }
        }
    }
    None
}

/// Whether `start` lies on a cycle in `cfg`.
fn block_in_cycle(cfg: &Cfg, start: BlockId) -> bool {
    let mut seen: BTreeSet<BlockId> = BTreeSet::new();
    let mut stack: Vec<BlockId> = cfg
        .successors
        .get(&start)
        .map(|s| s.iter().copied().collect())
        .unwrap_or_default();
    while let Some(b) = stack.pop() {
        if b == start {
            return true;
        }
        if !seen.insert(b) {
            continue;
        }
        if let Some(succs) = cfg.successors.get(&b) {
            stack.extend(succs.iter().copied());
        }
    }
    false
}

/// Functions reachable from `entry` via direct call-graph edges.
fn reachable_functions(cg: &CallGraph, entry: FunctionId) -> BTreeSet<FunctionId> {
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

#[cfg(test)]
mod tests {
    use super::*;

    use saf_core::air::{AirBlock, AirFunction, FieldPath, FieldStep, Instruction};
    use saf_core::id::make_id;
    use saf_core::ids::{BlockId, ModuleId};

    fn func_id(name: &str) -> FunctionId {
        FunctionId(make_id("func", name.as_bytes()))
    }
    fn block_id(name: &str) -> BlockId {
        BlockId(make_id("block", name.as_bytes()))
    }
    fn inst_id(name: &str) -> InstId {
        InstId(make_id("inst", name.as_bytes()))
    }

    fn inst(id: &str, op: Operation, operands: Vec<ValueId>) -> Instruction {
        Instruction {
            id: inst_id(id),
            op,
            operands,
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    /// Like [`inst`] but sets the result value id (`dst`) — needed to build the
    /// def-chains the handle/join resolvers walk.
    fn inst_d(id: &str, op: Operation, operands: Vec<ValueId>, dst: ValueId) -> Instruction {
        let mut i = inst(id, op, operands);
        i.dst = Some(dst);
        i
    }

    /// A declaration-only function (no body) with the given name — used so
    /// `module.function(callee)` resolves a primitive's name.
    fn declared(name: &str) -> AirFunction {
        let mut f = defined(name, Vec::new());
        f.blocks.clear();
        f.is_declaration = true;
        f
    }

    fn defined(name: &str, insts: Vec<Instruction>) -> AirFunction {
        let mut block = AirBlock::new(block_id(&format!("{name}_entry")));
        for i in insts {
            block.instructions.push(i);
        }
        block
            .instructions
            .push(inst(&format!("{name}_ret"), Operation::Ret, Vec::new()));
        AirFunction {
            id: func_id(name),
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

    fn module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: ModuleId(make_id("module", b"race_true_test")),
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

    #[test]
    fn verdict_is_bare_true() {
        assert_eq!(race_true_verdict(), "true");
    }

    #[test]
    fn inert_allowlist_and_denylist() {
        // Modeled / benign externals are inert.
        for n in [
            "pthread_mutex_lock",
            "pthread_mutex_unlock",
            "pthread_create",
            "pthread_join",
            "pthread_mutex_init",
            "malloc",
            "free",
            "abort",
            "__VERIFIER_nondet_int",
            "__VERIFIER_assume",
        ] {
            assert!(is_race_inert_external(n), "{n} should be inert");
        }
        // Unmodeled sync + memory-touching libc must force abstain.
        for n in [
            "pthread_mutex_trylock",
            "pthread_cond_wait",
            "pthread_cond_signal",
            "pthread_rwlock_wrlock",
            "pthread_spin_lock",
            "sem_wait",
            "pthread_barrier_wait",
            "pthread_once",
            "atomic_fetch_add",
            "__atomic_load",
            "__sync_fetch_and_add",
            "__VERIFIER_atomic_begin",
            "memcpy",
            "memset",
            "strcpy",
            "sprintf",
            "printf",
            "read",
        ] {
            assert!(!is_race_inert_external(n), "{n} must NOT be inert");
        }
    }

    #[test]
    fn resolve_thread_fn_direct_and_indirect() {
        let worker = defined("worker", Vec::new());
        // pthread_create(&id, attr, worker, arg): operand[2] carries the routine.
        let routine = ValueId::new(worker.id.raw());
        let dummy = ValueId::new(1);
        let create = inst(
            "create",
            Operation::CallDirect {
                callee: func_id("pthread_create"),
            },
            vec![dummy, dummy, routine, dummy],
        );
        let m = module(vec![worker.clone()]);
        assert_eq!(resolve_direct_thread_fn(&m, &create), Some(worker.id));

        // An untracked, non-function operand ⇒ unresolved ⇒ None (abstain).
        let opaque = ValueId::new(0xdead_beef);
        let create2 = inst(
            "create2",
            Operation::CallDirect {
                callee: func_id("pthread_create"),
            },
            vec![dummy, dummy, opaque, dummy],
        );
        assert_eq!(resolve_direct_thread_fn(&m, &create2), None);
    }

    #[test]
    fn sequential_program_is_race_free() {
        // A defined `main` with no reachable spawn is trivially race-free.
        let m = module(vec![defined("main", Vec::new())]);
        assert!(program_is_race_free(&m));
    }

    #[test]
    fn no_defined_main_abstains() {
        let m = module(vec![defined("helper", Vec::new())]);
        assert!(!program_is_race_free(&m));
    }

    #[test]
    fn reachable_indirect_call_abstains() {
        // main contains an indirect call ⇒ incomplete call graph ⇒ abstain, even
        // though it is otherwise sequential.
        let main = defined(
            "main",
            vec![inst(
                "ic",
                Operation::CallIndirect {
                    expected_signature: None,
                },
                vec![ValueId::new(7)],
            )],
        );
        assert!(!program_is_race_free(&module(vec![main])));
    }

    #[test]
    fn gep_indexing_predicate() {
        // Base-only GEP (no steps, single operand) is address-preserving.
        assert!(!gep_has_indexing(&FieldPath { steps: Vec::new() }, 1));
        // A field step OR an extra index operand means sub-object indexing.
        assert!(gep_has_indexing(
            &FieldPath {
                steps: vec![FieldStep::Field { index: 1 }],
            },
            1,
        ));
        assert!(gep_has_indexing(&FieldPath { steps: Vec::new() }, 2));
        assert!(gep_has_indexing(
            &FieldPath {
                steps: vec![FieldStep::Index],
            },
            2,
        ));
    }

    #[test]
    fn resolve_ptr_base_alloca_global_and_ambiguity() {
        let vt = ValueId::new(0x100); // alloca result
        let vg = ValueId::new(0x101); // no-op GEP result
        let vidx = ValueId::new(0x102); // indexing GEP result
        let vld = ValueId::new(0x103); // loaded pointer
        let alloca = inst_d("a", Operation::Alloca { size_bytes: None }, vec![], vt);
        let noop_gep = inst_d(
            "g",
            Operation::Gep {
                field_path: FieldPath { steps: Vec::new() },
            },
            vec![vt],
            vg,
        );
        let idx_gep = inst_d(
            "gi",
            Operation::Gep {
                field_path: FieldPath {
                    steps: vec![FieldStep::Index],
                },
            },
            vec![vt, ValueId::new(9)],
            vidx,
        );
        let load = inst_d("l", Operation::Load, vec![vt], vld);
        let defs: DefMap = [
            (vt, &alloca),
            (vg, &noop_gep),
            (vidx, &idx_gep),
            (vld, &load),
        ]
        .into_iter()
        .collect();
        let m = module(vec![]);
        // Alloca resolves to its own result id; a no-op GEP forwards to it.
        assert_eq!(resolve_ptr_base(vt, &defs, &m), Some(vt));
        assert_eq!(resolve_ptr_base(vg, &defs, &m), Some(vt));
        // An indexing GEP (array element) and a loaded pointer are ambiguous.
        assert_eq!(resolve_ptr_base(vidx, &defs, &m), None);
        assert_eq!(resolve_ptr_base(vld, &defs, &m), None);
    }

    #[test]
    fn join_key_resolves_load_of_handle() {
        let vt = ValueId::new(0x200); // alloca'd pthread_t
        let vv = ValueId::new(0x201); // loaded thread id value
        let alloca = inst_d("h", Operation::Alloca { size_bytes: None }, vec![], vt);
        let load = inst_d("lh", Operation::Load, vec![vt], vv);
        let join = inst(
            "j",
            Operation::CallDirect {
                callee: func_id("pthread_join"),
            },
            vec![vv, ValueId::new(0)],
        );
        // A non-join call with the same operand must resolve to nothing.
        let other = inst(
            "o",
            Operation::CallDirect {
                callee: func_id("printf"),
            },
            vec![vv],
        );
        let defs: DefMap = [(vt, &alloca), (vv, &load)].into_iter().collect();
        let m = module(vec![declared("pthread_join"), declared("printf")]);
        assert_eq!(join_key_of(&join, &defs, &m), Some(vt));
        assert_eq!(join_key_of(&other, &defs, &m), None);
    }

    #[test]
    fn joined_before_gates_recurrent_and_ambiguous() {
        let key = ValueId::new(0x300);
        let acc = Access {
            thread_id: 0,
            ptr: ValueId::new(1),
            write: true,
            must_locks: BTreeSet::new(),
            pre_spawn: false,
            is_main: true,
            joined_keys: [key].into_iter().collect(),
        };
        let handles: BTreeMap<u32, ValueId> = [(1u32, key)].into_iter().collect();
        let empty = BTreeSet::new();
        let recurrent: BTreeSet<u32> = BTreeSet::new();
        // Single-instance, unambiguous, joined handle ⇒ separated.
        assert!(joined_before(&acc, 1, &handles, &empty, &recurrent));
        // Recurrent thread ⇒ no join credit.
        let rec: BTreeSet<u32> = [1u32].into_iter().collect();
        assert!(!joined_before(&acc, 1, &handles, &empty, &rec));
        // Ambiguous handle (reused / array) ⇒ no join credit.
        let ambig: BTreeSet<ValueId> = [key].into_iter().collect();
        assert!(!joined_before(&acc, 1, &handles, &ambig, &recurrent));
        // A non-main access never gets join credit.
        let mut thread_acc = acc;
        thread_acc.is_main = false;
        assert!(!joined_before(&thread_acc, 1, &handles, &empty, &recurrent));
    }

    #[test]
    fn main_joined_in_propagates_must_join() {
        // main: entry { alloca t; load v; join(v); br mid }  mid { ret }
        // The must-join set at `mid`'s entry must contain the handle key `t`.
        let vt = ValueId::new(0x400);
        let vv = ValueId::new(0x401);
        let entry = block_id("m_entry");
        let mid = block_id("m_mid");
        let mut eb = AirBlock::new(entry);
        eb.instructions.push(inst_d(
            "a",
            Operation::Alloca { size_bytes: None },
            vec![],
            vt,
        ));
        eb.instructions
            .push(inst_d("l", Operation::Load, vec![vt], vv));
        eb.instructions.push(inst(
            "j",
            Operation::CallDirect {
                callee: func_id("pthread_join"),
            },
            vec![vv, ValueId::new(0)],
        ));
        eb.instructions
            .push(inst("br", Operation::Br { target: mid }, vec![]));
        let mut mb = AirBlock::new(mid);
        mb.instructions.push(inst("ret", Operation::Ret, vec![]));
        let main = AirFunction {
            id: func_id("main"),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![eb, mb],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let m = module(vec![main.clone(), declared("pthread_join")]);
        let defs = LockResolver::build(&m).defs;
        let joined = compute_main_joined_in(&main, &defs, &m);
        // Entry has joined nothing yet; `mid` (after the join) has the handle key.
        assert!(joined.get(&entry).unwrap().is_empty());
        assert!(joined.get(&mid).unwrap().contains(&vt));
    }

    #[test]
    fn block_in_cycle_detects_back_edge() {
        let b0 = block_id("b0");
        let b1 = block_id("b1");
        let mut successors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        successors.insert(b0, [b1].into_iter().collect());
        successors.insert(b1, [b1].into_iter().collect()); // self-loop
        let cfg = Cfg {
            function: func_id("f"),
            entry: b0,
            exits: BTreeSet::new(),
            successors,
            predecessors: BTreeMap::new(),
        };
        assert!(block_in_cycle(&cfg, b1));
        assert!(!block_in_cycle(&cfg, b0));
    }
}
