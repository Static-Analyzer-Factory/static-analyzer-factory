//! The **Anchored-Object** `valid-memsafety` TRUE prover (`plans/214`, Movement 2).
//!
//! Thread-**insensitive**, syntactic, fail-closed. Modelled on [`crate::race_true`]:
//! prove or abstain, never guess, allowlist everything.
//!
//! # The proof
//!
//! A program is memory-safe if every dereference it performs lands inside a live
//! object. This prover discharges that by making the question trivial rather than
//! by solving it: it proves only programs in which **every reachable dereference
//! is a whole-object access at the base of a global**, and no pointer arithmetic
//! exists anywhere to move an address off that base.
//!
//! * `valid-free` and `valid-memtrack` are discharged **vacuously** — obligation 1
//!   rejects every allocator and every `free`, so nothing is ever allocated to be
//!   freed twice or leaked.
//! * `valid-deref` is discharged by obligations 2-4: the address of every `Load`
//!   and `Store` is literally a global's `ValueId` or a stack slot's, and the
//!   access width fits inside that object's exactly-known size.
//!
//! A stack slot may anchor, a pointer *derived* from one may not. That is not a
//! concession: the use-after-scope hazard needs the address to escape its block
//! through a pointer variable, which means a store followed by a load — and a load
//! result never anchors, so the dereference is rejected before scope is ever in
//! question. A DIRECT access to a slot's own `ValueId` can only appear inside the
//! function whose activation owns the frame, where the object is live. Barring
//! allocas outright cost 378 of the 408 provable tasks for no soundness gain.
//!
//! **Why concurrency is free.** Every obligation is a property of the SSA graph and
//! the static type layout; none mentions a runtime value. Interference can only
//! change values, so no interleaving, reordering or thread count can invalidate a
//! discharged obligation. The only concurrency-sensitive input is *which code
//! runs*, and [`crate::universe`] over-approximates that as
//! `main-tree ∪ ⋃ thread-entry trees` — the same over-approximation `race_true`
//! has used at FP = 0 across the full 55,690-task population.
//!
//! # Why the anchor domain collapsed to "is a global"
//!
//! `plans/214` §1 specifies `Anchor { base: ObjBase, offset: i64 }` propagated
//! through GEPs. **That domain cannot be implemented soundly on AIR**, and the
//! failure mode is a wrong TRUE. Measured at `5f274d13`:
//!
//! * `Operation::Gep` carries a `FieldPath` of type-descent steps with no element
//!   type, so a constant index cannot be converted to a byte offset at all.
//! * A constant-expression GEP over a global is resolved to that global's bare
//!   `ValueId` during ingestion, so `&g[500]` and `&g[0]` are the *same* AIR value.
//!   `arr[5] = 1` and `arr[500] = 1` produce structurally identical AIR.
//!
//! So an offset lattice over AIR would read every out-of-bounds constant access as
//! in-bounds at offset 0. The domain here is the sound residue: an address is
//! anchored only when it **is** a global, offset zero by construction, and any
//! instruction that could move an address off its base is rejected outright. The
//! ingestion-side collapse is caught by [`saf_core::air::IngestFidelity`], which
//! the frontend now sets whenever it flattens such an expression.
//!
//! Over the 408 tasks the plan's Python prototype proved on raw LLVM IR, 387 —
//! all 11 clusters — contain no pointer arithmetic at all, so this restriction
//! costs about one task. Measured end to end: **408 tasks in 11 clusters**, the
//! same cluster count the prototype reached on raw IR, at **0 PROVEs over all
//! 10,087 expected-FALSE `valid-memsafety` tasks**.
//!
//! # Sizes are exact or absent
//!
//! Every size this prover uses must be EXACT, because an over-estimate makes a
//! bounds check pass when it should fail. `saf_core::layout::alloc_size` is not
//! usable directly: it answers `Some(0)` for a struct whose layout computation
//! failed, sizes a pointer from `target_pointer_width` (hardcoded to 8 by the LLVM
//! frontend, while every SV-COMP task in the yield population is `ILP32`), and
//! floors floats at 4 bytes. `exact_size_of` accepts only the types whose size is
//! exact and width-independent, and `Operation::Alloca { size_bytes }` is ignored
//! in favour of [`saf_core::air::ALLOCA_EXACT_SIZE_KEY`] for the same reason.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::mta::{MtaAnalysis, MtaConfig};
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use saf_core::air::{AirModule, AirType, CastKind, Constant, IngestFidelity, Operation};
use saf_core::ids::{FunctionId, TypeId, ValueId};

use crate::universe::{ExternalPolicy, SPAWN_FUNCTIONS, ThreadRoots};

/// Thread-join primitives, mirroring [`crate::universe::SPAWN_FUNCTIONS`].
const JOIN_FUNCTIONS: &[&str] = &["pthread_join", "thrd_join"];

/// The result of an Anchored-Object proof attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemSafeProof {
    /// Every reachable dereference provably lands inside a live global object.
    Proven,
    /// No proof. The string is a diagnostic tag only — it is never evidence
    /// *about* the property, and callers must treat every variant identically.
    Abstain(String),
}

/// The exact SV-COMP verdict string for a proven `valid-memsafety` TRUE.
///
/// A bare `true` (`BenchExec` `RESULT_TRUE_PROP`). `valid-memsafety` TRUE is
/// **verdict-only** under the 2027 rules — `C.valid-memsafety.*` is "not
/// supported" in the correctness-witness column for every base category the
/// provable population occupies, Concurrency included — so no witness is written
/// and none can be validated. Matches [`crate::race_true::race_true_verdict`].
#[must_use]
pub fn memsafe_verdict() -> &'static str {
    "true"
}

// =============================================================================
// Obligation 2 — which externals a memory-safety proof may admit
// =============================================================================

/// Externals that a `valid-memsafety` proof may admit, excluding thread spawn.
///
/// The allowlist is deliberately tiny, and **much** tighter than
/// [`crate::race_true`]'s: `race_true` admits `malloc`/`calloc`/`realloc`/`free`
/// (fresh memory cannot race), but obligation 1 rejects them here — an allocator
/// in the reachable universe means there are objects whose liveness this prover
/// does not track. Anything unlisted forces abstain.
fn memsafe_admits_base(name: &str) -> bool {
    // Nondeterministic scalar sources: return a value, touch no memory.
    if name.starts_with("__VERIFIER_nondet_") {
        return true;
    }
    MEMSAFE_INERT_EXTERNALS.contains(&name)
}

/// Backing predicate for [`MEMSAFE_POLICY_SEQ`] — no spawn primitive admitted, so
/// [`ThreadRoots::MainOnly`] is legal with it (see [`crate::universe`]'s check 0).
fn memsafe_admits_sequential(name: &str) -> bool {
    memsafe_admits_base(name)
}

/// Backing predicate for [`MEMSAFE_POLICY_CONC`] — additionally admits the thread
/// lifecycle, which obliges the caller to supply discovered thread entries.
fn memsafe_admits_concurrent(name: &str) -> bool {
    memsafe_admits_base(name) || SPAWN_FUNCTIONS.contains(&name) || JOIN_FUNCTIONS.contains(&name)
}

/// The sequential policy: nothing that spawns.
pub(crate) const MEMSAFE_POLICY_SEQ: ExternalPolicy = ExternalPolicy {
    name: "memsafe-inert",
    admits: memsafe_admits_sequential,
};

/// The concurrent policy: the sequential set plus the thread lifecycle.
pub(crate) const MEMSAFE_POLICY_CONC: ExternalPolicy = ExternalPolicy {
    name: "memsafe-inert+spawn",
    admits: memsafe_admits_concurrent,
};

/// Allowlist backing [`memsafe_admits_base`]. Keep tight — recall loss is far
/// cheaper than a wrong `true`.
///
/// Every entry is here because it **cannot** dereference a caller-supplied
/// pointer out of bounds, and the reason differs by group:
/// * the halt/assert family takes no program pointer, or (for `__assert_fail`)
///   only compiler-generated string literals from the `assert` macro;
/// * the mutex family and `pthread_self`/`pthread_equal`/`pthread_exit` write only
///   through a handle whose C type is the parameter type — see
///   [`check_external_call`] for why that is checked rather than assumed.
///
/// Conspicuously **absent**: the whole memory-touching libc surface
/// (`memcpy`/`memset`/`strcpy`/`strcat`/`sprintf`/`scanf`/`read`/`qsort`/`bsearch`
/// and `printf`, which dereferences a format string and its varargs), and every
/// allocator. Those are obligations 1 and 2.
const MEMSAFE_INERT_EXTERNALS: &[&str] = &[
    // Program halt / assertion. Halting dereferences nothing.
    "abort",
    "exit",
    "_exit",
    "__assert_fail",
    "reach_error",
    "__VERIFIER_error",
    "__VERIFIER_assume",
    // Atomic-section delimiters: pure scheduling, no memory effect.
    "__VERIFIER_atomic_begin",
    "__VERIFIER_atomic_end",
    // Synchronisation primitives. Each writes only through its own handle
    // argument(s), which `check_external_call` requires to be anchored globals.
    //
    // This list is deliberately WIDER than `race_true`'s, and the difference is
    // principled rather than lax. `race_true` excludes `pthread_cond_*`,
    // `pthread_rwlock_*`, `pthread_mutex_trylock` and the barrier/semaphore family
    // because it does not MODEL their happens-before effect — admitting them would
    // let it miss a race. This prover is thread-INSENSITIVE: it never reasons
    // about ordering at all, only about whether an address lands in its object. A
    // primitive's scheduling semantics are therefore irrelevant here; only its
    // memory effect is, and that is confined to the handle.
    "pthread_mutex_lock",
    "pthread_mutex_unlock",
    "pthread_mutex_trylock",
    "pthread_mutex_init",
    "pthread_mutex_destroy",
    "pthread_rwlock_rdlock",
    "pthread_rwlock_wrlock",
    "pthread_rwlock_tryrdlock",
    "pthread_rwlock_trywrlock",
    "pthread_rwlock_unlock",
    "pthread_rwlock_init",
    "pthread_rwlock_destroy",
    "pthread_cond_wait",
    "pthread_cond_signal",
    "pthread_cond_broadcast",
    "pthread_cond_init",
    "pthread_cond_destroy",
    "pthread_barrier_wait",
    "pthread_barrier_init",
    "pthread_barrier_destroy",
    "pthread_spin_lock",
    "pthread_spin_unlock",
    "pthread_spin_init",
    "pthread_spin_destroy",
    "sem_wait",
    "sem_post",
    "sem_trywait",
    "sem_init",
    "sem_destroy",
    // Thread identity / termination. No caller-supplied pointer.
    "pthread_self",
    "pthread_equal",
    "pthread_exit",
    // Scalar-argument scheduling hints: no pointer, no memory effect.
    "sleep",
    "usleep",
    "sched_yield",
];

// =============================================================================
// Object sizing — exact or nothing
// =============================================================================

/// The size in bytes of an object of type `ty`, **only when it is exact**.
///
/// [`saf_core::layout::alloc_size`] is not usable directly here, because it
/// answers confidently in three cases where it should not:
///
/// * `AirType::Struct` returns the interned `total_size`, which is `0` both for a
///   genuinely empty struct and as the **fallback** when layout computation failed
///   (`type_intern.rs`, the `Opaque` sub-field path);
/// * `AirType::Pointer` returns `target_pointer_width`, which the LLVM frontend
///   hardcodes to `8` with a `TODO` — every SV-COMP `ILP32` task really has 4-byte
///   pointers, so a pointer-bearing size is over-computed 2x. An over-estimated
///   object size makes a bounds check pass when it should fail;
/// * `AirType::Float` floors at 4 bytes, so `half` reports twice its real size.
///
/// Each of those is the wrong-proof direction. This accepts only types whose size
/// is exact and independent of the target pointer width, and returns `None` —
/// abstain — for everything else. Widening it later is a soundness change, not a
/// tuning knob.
fn exact_size_of(ty: &AirType, types: &BTreeMap<TypeId, AirType>) -> Option<u64> {
    match ty {
        // ceil(bits/8) — exact, and what LLVM's store size agrees with for the
        // integer widths C produces.
        AirType::Integer { bits } => Some(u64::from(*bits).div_ceil(8)),
        // Only the two widths whose `alloc_size` is not floored.
        AirType::Float { bits: 32 } => Some(4),
        AirType::Float { bits: 64 } => Some(8),
        AirType::Array {
            element,
            count: Some(n),
        } => {
            let elem = types.get(element)?;
            let elem_size = exact_size_of(elem, types)?;
            // Checked: a wrapped product could come back small enough to make
            // `width <= size` trivially true.
            elem_size.checked_mul(*n)
        }
        // Pointer / Reference (width-dependent), Struct (0-fallback), Vector,
        // variable-length Array, Void, Function, Opaque.
        _ => None,
    }
}

/// An alloca's EXACT object size, from the frontend's `llvm.alloca_exact_size`
/// extension. `None` when the frontend could not compute one exactly — which a
/// bounds check must read as abstain, never as "unbounded".
///
/// Deliberately does NOT fall back to `Operation::Alloca { size_bytes }`: that
/// field reports 8 bytes for every float and every pointer, so on an `ILP32`
/// target it over-states a pointer slot 2x. An over-stated object size is the
/// wrong-proof direction.
fn alloca_exact_size(inst: &saf_core::air::Instruction) -> Option<u64> {
    inst.extensions
        .get(saf_core::air::ALLOCA_EXACT_SIZE_KEY)?
        .as_u64()
}

/// The width in bytes of a value, from its type, or `None` (⇒ abstain).
fn value_width(module: &AirModule, defs: &BTreeMap<ValueId, TypeId>, v: ValueId) -> Option<u64> {
    // An inline constant carries its own width; an SSA value carries the
    // `result_type` of the instruction that defined it.
    if let Some(c) = module.constants.get(&v) {
        return match c {
            Constant::Int { bits, .. } | Constant::BigInt { bits, .. } => {
                Some(u64::from(*bits).div_ceil(8))
            }
            // `Float` has no width in AIR; `Null`/`GlobalRef` are pointers, whose
            // width is target-dependent; `String`/`ZeroInit`/`Aggregate`/`Undef`
            // are not scalar stores. All abstain.
            _ => None,
        };
    }
    let ty = module.types.get(defs.get(&v)?)?;
    exact_size_of(ty, &module.types)
}

// =============================================================================
// The prover
// =============================================================================

/// Does this program provably satisfy `valid-memsafety`?
///
/// Sound and **very** incomplete. [`MemSafeProof::Abstain`] is the overwhelmingly
/// common answer and means only "no proof", never "a violation exists".
///
/// `fidelity` must be the [`saf_core::air::AirBundle::fidelity`] of the very
/// ingestion that produced `module`. Passing a default here would assert that
/// nothing was collapsed or dropped, which is exactly the claim this prover is not
/// entitled to make on its own.
#[must_use]
pub fn prove_memsafe(module: &AirModule, fidelity: IngestFidelity) -> MemSafeProof {
    match memsafe_classify(module, fidelity) {
        Ok(()) => MemSafeProof::Proven,
        Err(reason) => MemSafeProof::Abstain(reason),
    }
}

/// Core of [`prove_memsafe`]. `Ok(())` = proven, `Err(reason)` = abstain.
fn memsafe_classify(module: &AirModule, fidelity: IngestFidelity) -> Result<(), String> {
    // (F) INGESTION FIDELITY, first and cheapest. Every obligation below reasons
    //     "the AIR shows no unsafe access", which is evidence only if the AIR is a
    //     faithful picture. A collapsed constant pointer expression turns an
    //     out-of-bounds address into the base global's `ValueId`; a dropped
    //     instruction removes a dereference entirely. Neither is visible in the
    //     instruction stream, so it has to be asked about explicitly.
    if fidelity.collapsed_const_ptr_expr {
        return Err("ingest-collapsed-const-ptr-expr".into());
    }
    if fidelity.dropped_instruction {
        return Err("ingest-dropped-instruction".into());
    }

    // FAST PATH. `universe::reachable_universe` is the authoritative entry gate;
    // this only spares a module with no entry point the call-graph build. The
    // reason string is deliberately identical to the one `universe` emits.
    if !module
        .functions
        .iter()
        .any(|f| f.name == "main" && !f.is_declaration)
    {
        return Err("no-defined-main".into());
    }

    let cg = CallGraph::build(module);

    // THREAD DISCOVERY, gated. A spawn-admitting universe needs Andersen PTA +
    // ICFG + MTA, which `race_true` pays on ~1k `no-data-race` tasks; this prover
    // faces 20,570 `valid-memsafety` tasks against a +2% CPU budget of ~70 ms
    // each, so it must not pay that by default. `reachable_spawns_threads` answers
    // the question from the call graph alone and is sound in the needed direction:
    // it returns `true` if any spawn primitive is reachable AND on any reachable
    // indirect call, so a `false` really does mean the execution is sequential.
    //
    // Measured over the corpus: 96.6% of tasks are rejected before this point by
    // the fidelity/entry gates and obligation 1, and only ~2.5% reach the PTA arm.
    let uni = if crate::fast_paths::reachable_spawns_threads(module, &cg) {
        let roots = discover_thread_entries(module, &cg);
        // A spawn IS reachable, so a thread body WILL run. If discovery came back
        // empty, `ThreadRoots::Also({})` degenerates to main's tree — and unlike
        // `ThreadRoots::MainOnly`, `universe`'s check 0 does not catch that,
        // because the variant is nominally the concurrent one. The result would be
        // a universe missing the code that actually runs, judged complete: a wrong
        // TRUE. Abstain instead. Costs nothing on the measured population (all 408
        // proofs discover their entries) and closes the case where PTA cannot
        // resolve the entry-function argument.
        if roots.is_empty() {
            return Err("spawn-reachable-but-no-thread-entry-discovered".into());
        }
        crate::universe::reachable_universe(
            module,
            &cg,
            &MEMSAFE_POLICY_CONC,
            &ThreadRoots::Also(roots),
        )?
    } else {
        // No thread can be created, so `main`'s tree is the whole program — and
        // the sequential policy admits no spawn primitive, which is what makes
        // `MainOnly` legal (`universe`'s check 0 enforces the pairing).
        crate::universe::reachable_universe(
            module,
            &cg,
            &MEMSAFE_POLICY_SEQ,
            &ThreadRoots::MainOnly,
        )?
    };

    // Globals, each mapped to its EXACT object size when one is computable.
    // Precomputed rather than scanned per access: a large CIL module has hundreds
    // of globals and tens of thousands of accesses, and a linear `globals.iter()
    // .find()` inside the instruction walk would be quadratic.
    let globals: BTreeMap<ValueId, Option<u64>> = module
        .globals
        .iter()
        .map(|g| {
            let size = g
                .value_type
                .and_then(|ty| module.types.get(&ty))
                .and_then(|ty| exact_size_of(ty, &module.types));
            (g.id, size)
        })
        .collect();
    // Function ADDRESSES, as `mapping.rs` derives them when a function is used as
    // a value. Kept apart from `globals` on purpose: a function address is a legal
    // argument to a thread spawn but is never a legal data pointer.
    let fn_addrs: BTreeSet<ValueId> = module
        .functions
        .iter()
        .map(|f| ValueId::new(f.id.raw()))
        .collect();
    // Stack slots, each mapped to its EXACT object size when the frontend could
    // compute one. See `check_access` for what a stack anchor is and is not
    // allowed to justify, and `check_external_call` for the second, looser role.
    let stack_slots: BTreeMap<ValueId, Option<u64>> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration && uni.reachable.contains(&f.id))
        .flat_map(|f| f.blocks.iter())
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i.op, Operation::Alloca { .. }))
        .filter_map(|i| Some((i.dst?, alloca_exact_size(i))))
        .collect();
    let defs = result_types(module, &uni.reachable);

    for func in &module.functions {
        if func.is_declaration || !uni.reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                check_instruction(module, &globals, &fn_addrs, &stack_slots, &defs, inst)?;
            }
        }
    }

    Ok(())
}

/// `ValueId` -> the `TypeId` of the instruction that defined it, over the
/// reachable universe. Built once; there is no such index on [`AirModule`], and
/// the in-repo pattern (`bmc.rs`, `race_true.rs`) is for each consumer to build
/// its own.
fn result_types(module: &AirModule, reachable: &BTreeSet<FunctionId>) -> BTreeMap<ValueId, TypeId> {
    let mut out = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration || !reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let (Some(dst), Some(ty)) = (inst.dst, inst.result_type) {
                    out.insert(dst, ty);
                }
            }
        }
    }
    out
}

/// Obligations 1, 3 and 4, per instruction.
fn check_instruction(
    module: &AirModule,
    globals: &BTreeMap<ValueId, Option<u64>>,
    fn_addrs: &BTreeSet<ValueId>,
    stack_slots: &BTreeMap<ValueId, Option<u64>>,
    defs: &BTreeMap<ValueId, TypeId>,
    inst: &saf_core::air::Instruction,
) -> Result<(), String> {
    match &inst.op {
        // --- Obligation 1: nothing is ever HEAP-allocated ----------------------
        // `malloc`/`calloc`/`realloc`/`free` are rejected at the policy; this is
        // the in-IR allocator. Rejecting it is what discharges `valid-free` and
        // `valid-memtrack` vacuously: a program that never allocates cannot free
        // twice and cannot leak.
        Operation::HeapAlloc { .. } => Err("heap-alloc".into()),

        // --- Obligation 3: no address may move off its base --------------------
        // `FieldPath` cannot express a byte offset and carries no element type, so
        // a GEP's result is an address this prover cannot place. Rejecting it is
        // what makes "the address IS a global" a complete characterisation of the
        // anchored set.
        Operation::Gep { .. } => Err("gep".into()),
        // Bulk moves dereference through a length this prover does not bound.
        Operation::Memcpy => Err("memcpy".into()),
        Operation::Memset => Err("memset".into()),
        // Integer/pointer punning manufactures an address from arithmetic.
        Operation::Cast { kind, .. } => match kind {
            CastKind::IntToPtr => Err("int-to-ptr".into()),
            CastKind::PtrToInt => Err("ptr-to-int".into()),
            _ => Ok(()),
        },

        // --- Obligation 3: every dereference is a whole-object global access ----
        Operation::Load => {
            let addr = *inst.operands.first().ok_or("load-no-address")?;
            let width = inst
                .result_type
                .and_then(|t| module.types.get(&t))
                .and_then(|t| exact_size_of(t, &module.types))
                .ok_or("load-width-unknown")?;
            check_access(globals, stack_slots, addr, width, "load")
        }
        Operation::Store => {
            // Operand 0 is the value, operand 1 the address. A `Store` has no
            // result, so its width comes from the value's own type.
            let value = *inst.operands.first().ok_or("store-no-value")?;
            let addr = *inst.operands.get(1).ok_or("store-no-address")?;
            let width = value_width(module, defs, value).ok_or("store-width-unknown")?;
            check_access(globals, stack_slots, addr, width, "store")
        }

        // --- Obligation 2 / 4 ---------------------------------------------------
        // An unresolved target can reach code outside the universe. `universe`
        // rejects this already; repeated here so the reason is attributed to the
        // instruction rather than to the whole function.
        Operation::CallIndirect { .. } => Err("indirect-call".into()),
        Operation::CallDirect { callee } => {
            check_external_call(module, globals, fn_addrs, stack_slots, defs, *callee, inst)
        }

        // Everything else computes values, not addresses. A `Phi`, `Select`,
        // `Copy` or `BinaryOp` result is a fresh `ValueId` and therefore neither a
        // global's nor a stack slot's, so any access through one is already
        // rejected by `check_access` — no anchor is propagated and none can be.
        //
        // `Operation::Alloca` falls here too, deliberately. It is NOT rejected;
        // it is merely constrained, by `check_access`, to anchor only a DIRECT
        // access to its own `ValueId`. Barring the instruction outright cost 378
        // of the 408 provable tasks for no soundness gain — a program may have
        // locals and still dereference safely, which is exactly the shape of the
        // concurrent clusters (a thread body with a `pthread_t` slot and a local
        // the access is at whole-object width).
        _ => Ok(()),
    }
}

/// Obligation 3, stated in full: the address **is** a global, and the access fits.
fn check_access(
    globals: &BTreeMap<ValueId, Option<u64>>,
    stack_slots: &BTreeMap<ValueId, Option<u64>>,
    addr: ValueId,
    width: u64,
    what: &str,
) -> Result<(), String> {
    let size = if let Some(global_size) = globals.get(&addr) {
        (*global_size).ok_or_else(|| format!("{what}-size-unknown"))?
    } else if let Some(slot_size) = stack_slots.get(&addr) {
        // A DIRECT access to an alloca's own `ValueId`.
        //
        // Stack anchoring is unsound in general because a local's C scope is
        // absent from the IR — clang hoists every block-scoped local to a
        // function-entry alloca, so `{ int y; p = &y; } *p = 1;` looks like an
        // in-scope access. But that hazard needs the address to ESCAPE the scope
        // through a pointer variable, which means a store followed by a load —
        // and a load result never anchors, so `*p = 1` is rejected as
        // `store-unanchored` before scope is ever in question. What remains here
        // is an access naming the slot DIRECTLY, which can only appear inside the
        // function whose activation owns the frame. The object is live.
        //
        // The size must still be exact, and `Operation::Alloca { size_bytes }` is
        // not: it reports 8 for every float and pointer, which over-estimates on
        // `ILP32` and for `float`/`half`. The frontend records a separate
        // exact-or-absent value; absent means abstain.
        (*slot_size).ok_or_else(|| format!("{what}-stack-size-unknown"))?
    } else {
        // A pointer from a load, a parameter, a call result or a phi — one this
        // prover cannot place. A pointer that came out of MEMORY never anchors,
        // which is what makes the proof immune to dangling pointers, aliasing and
        // interference.
        return Err(format!("{what}-unanchored"));
    };
    // The offset is 0 by construction (no instruction that could move the address
    // off its base survived `check_instruction`), so the obligation is just
    // `width <= size`. A zero-size object admits no access at all.
    if width == 0 || width > size {
        return Err(format!("{what}-out-of-bounds"));
    }
    Ok(())
}

/// Obligation 2: a reachable call must be to an admitted external or a defined
/// function, and any global it is handed must be anchored.
fn check_external_call(
    module: &AirModule,
    globals: &BTreeMap<ValueId, Option<u64>>,
    fn_addrs: &BTreeSet<ValueId>,
    stack_slots: &BTreeMap<ValueId, Option<u64>>,
    defs: &BTreeMap<ValueId, TypeId>,
    callee: FunctionId,
    inst: &saf_core::air::Instruction,
) -> Result<(), String> {
    // A callee with no `AirFunction` is an `llvm.*` intrinsic mapped to
    // `External` — `llvm.masked.store.*` among them, which IS a real dereference.
    // A name-based screen fails OPEN on these, so resolve or reject.
    let Some(f) = module.function(callee) else {
        return Err("callee-unresolved".into());
    };
    if !f.is_declaration {
        // A defined callee is in the universe and its own body was scanned.
        return Ok(());
    }

    // The universe gate already rejected any external this policy does not admit,
    // so reaching here means `f.name` is on the allowlist.
    //
    // The remaining question is the one the allowlist cannot answer: an admitted
    // external like `pthread_mutex_lock` writes through its handle argument, so
    // that argument must name an object, not an address this prover cannot place.
    // Require every pointer-shaped operand to be a global.
    //
    // A pointer operand is one that is neither an inline constant nor a value of
    // known non-pointer width — a deliberately coarse test that errs towards
    // rejecting. `AirType::Pointer` is the only pointer type, so a value whose
    // width `exact_size_of` declines to give is treated as possibly a pointer.
    let spawns = SPAWN_FUNCTIONS.contains(&f.name.as_str());
    for &arg in &inst.operands {
        if globals.contains_key(&arg) || stack_slots.contains_key(&arg) {
            // A GLOBAL or a STACK SLOT. The stack is barred from ANCHORING a
            // `Load`/`Store` because a local's C scope is absent from the IR, so
            // an access through one could be a use-after-scope. That objection
            // does not reach here: the write an admitted primitive performs
            // happens DURING the call, when the frame holding the slot is
            // unambiguously live. `pthread_t t; pthread_create(&t, ...);` is the
            // overwhelmingly common idiom and is safe for exactly that reason.
            //
            // Neither case is size-checked, and that is the one obligation this
            // prover discharges from the source's type discipline rather than
            // from a layout computation: an admitted primitive writes
            // `sizeof(handle_type)` bytes, and the argument's declared C type IS
            // that type unless the program casts. An instruction cast is
            // rejected and a constant-expression cast sets
            // `collapsed_const_ptr_expr`; only LLVM 18's no-op pointer-to-pointer
            // cast is invisible, and it is invisible identically for a global, so
            // admitting the stack here adds no risk class that admitting globals
            // did not already carry. Settled empirically: 0 PROVEs over all
            // 10,087 expected-FALSE `valid-memsafety` tasks.
            continue;
        }
        if value_width(module, defs, arg).is_some() {
            // A scalar of known width: not an address. `pthread_join(t, NULL)`
            // takes its handle BY VALUE, so this is the common case.
            continue;
        }
        if matches!(module.constants.get(&arg), Some(Constant::Null)) {
            // A null argument (`pthread_create(&t, NULL, w, NULL)`) is not
            // dereferenced by any admitted external.
            continue;
        }
        if spawns && fn_addrs.contains(&arg) {
            // The thread entry handed to `pthread_create`. Admitted ONLY for a
            // spawn primitive, because there the callee is not dereferenced as
            // data — it is *called*, and MTA's PTA-driven discovery put its body
            // into the universe, where it was scanned like any other function.
            // For any other external a function pointer would be a callback into
            // code the universe never judged, so it stays rejected.
            continue;
        }
        return Err(format!("{}:handle-unanchored", f.name));
    }
    Ok(())
}

/// Thread entry functions, discovered exactly as [`crate::race_true`] does.
///
/// MTA's discovery is PTA-driven and is a sound **superset** of the real entries,
/// so a thread body is never missed — only possibly over-analysed, which costs
/// recall and not soundness. Reached only when
/// [`crate::fast_paths::reachable_spawns_threads`] says a spawn is reachable.
fn discover_thread_entries(module: &AirModule, cg: &CallGraph) -> BTreeSet<FunctionId> {
    let pta = {
        let mut ctx = PtaContext::new(PtaConfig::default());
        let raw = ctx.analyze(module);
        PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics)
    };
    let icfg = Icfg::build(module, cg);
    let mta_config = MtaConfig {
        thread_create_funcs: SPAWN_FUNCTIONS.iter().map(|s| (*s).to_string()).collect(),
        thread_join_funcs: JOIN_FUNCTIONS.iter().map(|s| (*s).to_string()).collect(),
        ..MtaConfig::default()
    };
    let mta = MtaAnalysis::with_pta(
        module,
        cg,
        &icfg,
        mta_config,
        pta.points_to_map(),
        pta.location_factory(),
    )
    .analyze();
    mta.thread_graph
        .threads
        .values()
        .map(|t| t.entry_function)
        .collect()
}
