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
    // `printf` dereferences its format and any `%s`/`%n` argument; it is admitted
    // here and its format is checked per call (see `check_printf_format`).
    name == "printf" || MEMSAFE_INERT_EXTERNALS.contains(&name)
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
///
/// `escaping_locals` names, as `(function, C variable)`, the block-scoped
/// locals whose address escaped (read from the IR before `mem2reg`). Any one
/// of them makes the prover abstain: `mem2reg` can promote such a local after
/// the pointer that held it, which forwards a value through the dead object
/// and erases the dangling dereference (`{ int y; p = &y; } *p = 1;` becomes
/// no access at all).
///
/// The target's pointer width — which fixes the sizes of the `pthread_*` handle
/// objects an admitted external writes (see [`handle_accesses`]) — is read from
/// `module.target_pointer_width` (the module's own data layout). Prefer
/// [`prove_memsafe_for_data_model`], which also checks that width against the
/// task's declared data model.
#[must_use]
pub fn prove_memsafe(
    module: &AirModule,
    fidelity: IngestFidelity,
    escaping_locals: &BTreeSet<(String, String)>,
) -> MemSafeProof {
    match memsafe_classify(
        module,
        fidelity,
        escaping_locals,
        module.target_pointer_width,
    ) {
        Ok(()) => MemSafeProof::Proven,
        Err(reason) => MemSafeProof::Abstain(reason),
    }
}

/// [`prove_memsafe`] for a task of the given `data_model`: abstains unless the
/// module's data layout has that model's pointer width (4 for `ILP32`, 8 for
/// `LP64`). A module compiled for — or defaulted to — another width would size
/// every handle object for the wrong ABI, and an over-sized handle check is the
/// wrong-proof direction.
#[must_use]
pub fn prove_memsafe_for_data_model(
    module: &AirModule,
    fidelity: IngestFidelity,
    escaping_locals: &BTreeSet<(String, String)>,
    data_model: crate::DataModel,
) -> MemSafeProof {
    let width = match data_model {
        crate::DataModel::ILP32 => 4,
        crate::DataModel::LP64 => 8,
    };
    if module.target_pointer_width != width {
        return MemSafeProof::Abstain("data-model-pointer-width-mismatch".into());
    }
    prove_memsafe(module, fidelity, escaping_locals)
}

/// Core of [`prove_memsafe`]. `Ok(())` = proven, `Err(reason)` = abstain.
// NOTE: one linear sequence of gates and obligations; splitting it would only
// scatter the order in which they must run.
#[allow(clippy::too_many_lines)]
fn memsafe_classify(
    module: &AirModule,
    fidelity: IngestFidelity,
    escaping_locals: &BTreeSet<(String, String)>,
    ptr_width: u32,
) -> Result<(), String> {
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
    // (S) SCOPE. A block-scoped local whose address escaped can be used after
    //     its block ends, and the promoted IR this prover reads may no longer
    //     show that access at all (see `prove_memsafe`).
    if let Some((f, v)) = escaping_locals.iter().next() {
        return Err(format!("escaping-block-local:{f}:{v}"));
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
        let roots = discover_thread_entries(module, &cg)?;
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
    let fn_addrs: BTreeMap<ValueId, FunctionId> = module
        .functions
        .iter()
        .map(|f| (ValueId::new(f.id.raw()), f.id))
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
    // Object sizes for the HANDLE check of an admitted external (see
    // `check_external_call`): the data-layout allocation size, which — unlike the
    // dereference bound above — may count pointers, floats and structs, because it
    // comes from the module's own data layout rather than a width-blind type walk.
    // `sizeof(T)` IS the allocation size, so a handle object of at least the
    // callee's handle size holds the whole write.
    let handle_sizes: BTreeMap<ValueId, Option<u64>> = module
        .globals
        .iter()
        .map(|g| {
            let size = g
                .layout
                .as_ref()
                .map(saf_core::air::Layout::size)
                .or_else(|| {
                    g.value_type
                        .and_then(|ty| module.types.get(&ty))
                        .and_then(|ty| exact_size_of(ty, &module.types))
                });
            (g.id, size)
        })
        .chain(
            module
                .functions
                .iter()
                .filter(|f| !f.is_declaration && uni.reachable.contains(&f.id))
                .flat_map(|f| f.blocks.iter())
                .flat_map(|b| b.instructions.iter())
                .filter(|i| matches!(i.op, Operation::Alloca { .. }))
                .filter_map(|i| {
                    let size = i
                        .extensions
                        .get(saf_core::air::ALLOCA_LAYOUT_SIZE_KEY)
                        .and_then(serde_json::Value::as_u64)
                        .or_else(|| alloca_exact_size(i));
                    Some((i.dst?, size))
                }),
        )
        .collect();
    let defs = result_types(module, &uni.reachable);
    let scan = Scan {
        module,
        globals: &globals,
        fn_addrs: &fn_addrs,
        stack_slots: &stack_slots,
        handle_sizes: &handle_sizes,
        defs: &defs,
        reachable: &uni.reachable,
        ptr_width,
    };

    for func in &module.functions {
        if func.is_declaration || !uni.reachable.contains(&func.id) {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                check_instruction(&scan, inst)?;
            }
        }
    }

    Ok(())
}

/// Everything the per-instruction obligations read, built once per proof.
struct Scan<'a> {
    module: &'a AirModule,
    /// Globals → EXACT width-independent size (the dereference bound).
    globals: &'a BTreeMap<ValueId, Option<u64>>,
    /// Function addresses (as `mapping.rs` derives them) → the function.
    fn_addrs: &'a BTreeMap<ValueId, FunctionId>,
    /// Reachable stack slots → EXACT size (the dereference bound).
    stack_slots: &'a BTreeMap<ValueId, Option<u64>>,
    /// Globals and reachable stack slots → data-layout allocation size (the
    /// handle bound for an admitted external).
    handle_sizes: &'a BTreeMap<ValueId, Option<u64>>,
    /// Result type of every reachable instruction.
    defs: &'a BTreeMap<ValueId, TypeId>,
    /// The reachable universe.
    reachable: &'a BTreeSet<FunctionId>,
    /// Target pointer width in bytes (fixes the `pthread_*` handle sizes).
    ptr_width: u32,
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
fn check_instruction(scan: &Scan<'_>, inst: &saf_core::air::Instruction) -> Result<(), String> {
    let module = scan.module;
    let (globals, stack_slots, defs) = (scan.globals, scan.stack_slots, scan.defs);
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
        Operation::CallDirect { callee } => check_external_call(scan, *callee, inst),

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

/// The handle objects an admitted external reads or writes through its pointer
/// arguments, as `(operand index, bytes accessed)`, for a target with
/// `ptr_width`-byte pointers (glibc on x86: `ILP32` for 4, `LP64` for 8). `None`
/// for any other width — no handle size is then known, and every handle-taking
/// call abstains.
///
/// The sizes are the RUNTIME library's, not the program's: `pthread_join` stores a
/// `void *` through its out-parameter whatever the program declared the slot as,
/// and `pthread_mutex_lock` touches `sizeof(pthread_mutex_t)` bytes of whatever
/// object it is handed. A pointer cast that narrows the object is invisible under
/// opaque pointers, so the object's own size is checked against these.
///
/// An admitted external absent from this table dereferences no program pointer
/// (`pthread_self`, `exit`, `sleep`, …), or only reads the compiler-emitted,
/// NUL-terminated string literals of the `assert` macro (`__assert_fail`). A null
/// argument is accepted only where the runtime allows one (an attr, a join
/// retval); a null required handle abstains (see [`check_external_call`]).
fn handle_accesses(name: &str, ptr_width: u32) -> Option<Vec<(usize, u64)>> {
    let lp64 = match ptr_width {
        4 => false,
        8 => true,
        _ => return None,
    };
    // `sizeof` per ABI.
    let abi = |ilp32: u64, lp: u64| if lp64 { lp } else { ilp32 };
    // `pthread_t`, `thrd_t` (`unsigned long`) and `void *`.
    let word = abi(4, 8);
    let mutex = abi(24, 40);
    let attr = abi(36, 56);
    let rwlock = abi(32, 56);
    let barrier = abi(20, 32);
    let sem = abi(16, 32);
    // `pthread_cond_t` is 48 bytes on both; every `*attr_t` but the rwlock one
    // is 4, that one 8; `pthread_spinlock_t` is a `volatile int`.
    let cond = 48;
    Some(match name {
        // pthread_create(pthread_t *, const pthread_attr_t *, start, arg)
        "pthread_create" => vec![(0, word), (1, attr)],
        // thrd_create(thrd_t *, start, arg)
        "thrd_create" => vec![(0, word)],
        // pthread_join(pthread_t, void **retval) — stores a `void *`.
        "pthread_join" => vec![(1, word)],
        // thrd_join(thrd_t, int *res)
        "thrd_join" => vec![(1, 4)],
        "pthread_mutex_lock"
        | "pthread_mutex_unlock"
        | "pthread_mutex_trylock"
        | "pthread_mutex_destroy" => vec![(0, mutex)],
        "pthread_mutex_init" => vec![(0, mutex), (1, 4)],
        "pthread_rwlock_rdlock"
        | "pthread_rwlock_wrlock"
        | "pthread_rwlock_tryrdlock"
        | "pthread_rwlock_trywrlock"
        | "pthread_rwlock_unlock"
        | "pthread_rwlock_destroy" => vec![(0, rwlock)],
        "pthread_rwlock_init" => vec![(0, rwlock), (1, 8)],
        "pthread_cond_wait" => vec![(0, cond), (1, mutex)],
        "pthread_cond_signal" | "pthread_cond_broadcast" | "pthread_cond_destroy" => {
            vec![(0, cond)]
        }
        "pthread_cond_init" => vec![(0, cond), (1, 4)],
        "pthread_barrier_wait" | "pthread_barrier_destroy" => vec![(0, barrier)],
        // The count is a scalar.
        "pthread_barrier_init" => vec![(0, barrier), (1, 4)],
        "pthread_spin_lock"
        | "pthread_spin_unlock"
        | "pthread_spin_init"
        | "pthread_spin_destroy" => vec![(0, 4)],
        "sem_wait" | "sem_post" | "sem_trywait" | "sem_init" | "sem_destroy" => vec![(0, sem)],
        _ => Vec::new(),
    })
}

/// Obligation 2: a reachable call must be to an admitted external or a defined
/// function, any object it is handed must be anchored, and any object it reads
/// or writes through must be at least as large as the access.
fn check_external_call(
    scan: &Scan<'_>,
    callee: FunctionId,
    inst: &saf_core::air::Instruction,
) -> Result<(), String> {
    let module = scan.module;
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
    // Require every pointer-shaped operand to be a global or a stack slot, and
    // every operand the callee dereferences to name one at least as large as the
    // runtime's access.
    //
    // A pointer operand is one that is neither an inline constant nor a value of
    // known non-pointer width — a deliberately coarse test that errs towards
    // rejecting. `AirType::Pointer` is the only pointer type, so a value whose
    // width `exact_size_of` declines to give is treated as possibly a pointer.
    if f.name == "printf" {
        // The format is a constant global literal and no conversion dereferences
        // an argument, so no other operand is ever read through: nothing more to
        // anchor (a `double` constant has no AIR width and would otherwise be
        // taken for a possible pointer).
        return check_printf_format(module, inst);
    }
    let accesses = handle_accesses(&f.name, scan.ptr_width)
        .ok_or_else(|| format!("{}:pointer-width-unknown", f.name))?;
    let spawns = SPAWN_FUNCTIONS.contains(&f.name.as_str());
    for (idx, &arg) in inst.operands.iter().enumerate() {
        if matches!(module.constants.get(&arg), Some(Constant::Null)) {
            // A null OPTIONAL argument (`pthread_create(&t, NULL, w, NULL)`, an
            // `*_init` attr, a `pthread_join` retval) is not dereferenced. A null
            // REQUIRED handle (`pthread_mutex_lock(NULL)`, `sem_wait(NULL)`, the
            // `pthread_create` thread slot, `pthread_cond_wait`'s mutex) is a
            // dereference of NULL inside the runtime: abstain.
            let required = idx == 0 || f.name == "pthread_cond_wait";
            if required && accesses.iter().any(|(i, _)| *i == idx) {
                return Err(format!("{}:null-handle", f.name));
            }
            continue;
        }
        if let Some(&(_, need)) = accesses.iter().find(|(i, _)| *i == idx) {
            // A HANDLE the callee reads or writes `need` bytes of. It must be a
            // global or a stack slot (offset 0 by construction: no GEP survives)
            // whose data-layout size covers the access. `pthread_join(t, &st)`
            // with `char st` stores a whole `void *` into one byte; with opaque
            // pointers the `(void **)&st` cast is invisible, so only the object's
            // own size can tell. Unknown size abstains.
            //
            // The stack is barred from ANCHORING a `Load`/`Store` because a
            // local's C scope is absent from the IR; that objection does not reach
            // here — the callee's access happens DURING the call, when the frame
            // holding the slot is unambiguously live.
            let Some(size) = scan.handle_sizes.get(&arg) else {
                return Err(format!("{}:handle-unanchored", f.name));
            };
            match size {
                Some(size) if *size >= need => continue,
                Some(_) => return Err(format!("{}:handle-too-small", f.name)),
                None => return Err(format!("{}:handle-size-unknown", f.name)),
            }
        }
        if scan.globals.contains_key(&arg) || scan.stack_slots.contains_key(&arg) {
            // A GLOBAL or a STACK SLOT the callee does NOT dereference (a
            // `pthread_create` `arg`, a `pthread_exit` value): it is only passed
            // on, and any later dereference of it is through a parameter or a
            // loaded pointer, which never anchors.
            continue;
        }
        if value_width(module, scan.defs, arg).is_some() {
            // A scalar of known width: not an address. `pthread_join(t, NULL)`
            // takes its handle BY VALUE, so this is the common case.
            continue;
        }
        if spawns {
            if let Some(entry) = scan.fn_addrs.get(&arg) {
                // The thread entry handed to `pthread_create`. Admitted ONLY for a
                // spawn primitive, because there the callee is not dereferenced as
                // data — it is *called* — and ONLY when its body is in the
                // universe that was scanned. MTA reads `pthread_create`'s routine
                // from operand 2, so a `thrd_create` routine (operand 1) or any
                // entry discovery missed would otherwise run unjudged.
                if scan.reachable.contains(entry) {
                    continue;
                }
                return Err(format!("{}:thread-entry-not-in-universe", f.name));
            }
        }
        return Err(format!("{}:handle-unanchored", f.name));
    }
    Ok(())
}

/// Admit one `printf` call only if it reads nothing but its own format.
///
/// The format (operand 0) must be a CONSTANT global whose initializer is a string
/// literal — a mutable global could be rewritten before the call — and every
/// conversion in it must take a scalar argument: `d i o u x X c f F e E g G a A`,
/// `p` (prints the pointer value, never dereferences it) and `%%`, with any flags,
/// width, precision (`*` included: an `int` argument) and length modifier. `%s`
/// and `%n` (and every other conversion) dereference a caller-supplied pointer, so
/// they abstain, as does a format that is not a constant string literal or that
/// ends inside a conversion. The literal itself is NUL-terminated by
/// construction, so reading it stays in bounds.
fn check_printf_format(
    module: &AirModule,
    inst: &saf_core::air::Instruction,
) -> Result<(), String> {
    let fmt = *inst.operands.first().ok_or("printf-format:missing")?;
    let global = module
        .globals
        .iter()
        .find(|g| g.id == fmt)
        .ok_or("printf-format:not-a-global")?;
    if !global.is_constant {
        return Err("printf-format:mutable".into());
    }
    let Some(Constant::String { value }) = &global.init else {
        return Err("printf-format:not-a-string-literal".into());
    };
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            continue;
        }
        // Flags, width, precision and length modifiers, then the conversion.
        let conv = loop {
            match chars.next() {
                Some(
                    '-'
                    | '+'
                    | ' '
                    | '#'
                    | '0'..='9'
                    | '.'
                    | '*'
                    | 'h'
                    | 'l'
                    | 'j'
                    | 'z'
                    | 't'
                    | 'L'
                    | 'q',
                ) => {}
                other => break other,
            }
        };
        match conv {
            Some(
                'd' | 'i' | 'o' | 'u' | 'x' | 'X' | 'c' | 'f' | 'F' | 'e' | 'E' | 'g' | 'G' | 'a'
                | 'A' | 'p' | '%',
            ) => {}
            Some(other) => return Err(format!("printf-format:conversion-{other}")),
            None => return Err("printf-format:truncated".into()),
        }
    }
    Ok(())
}

/// Thread entry functions, discovered exactly as [`crate::race_true`] does.
///
/// MTA's discovery is PTA-driven and is a sound **superset** of the real entries,
/// so a thread body is never missed — only possibly over-analysed, which costs
/// recall and not soundness. Reached only when
/// [`crate::fast_paths::reachable_spawns_threads`] says a spawn is reachable.
///
/// That superset claim needs a CONVERGED points-to solve and a discovery walk
/// that was not cut at its context-depth cap; either truncation can drop a real
/// entry, so each abstains (`Err`).
fn discover_thread_entries(
    module: &AirModule,
    cg: &CallGraph,
) -> Result<BTreeSet<FunctionId>, String> {
    let pta = {
        let mut ctx = PtaContext::new(PtaConfig::default());
        let raw = ctx.analyze(module);
        PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics)
    };
    if pta.diagnostics().iteration_limit_hit {
        return Err("pta-iteration-limit-hit".into());
    }
    let icfg = Icfg::build(module, cg);
    let mta_config = MtaConfig {
        thread_create_funcs: SPAWN_FUNCTIONS.iter().map(|s| (*s).to_string()).collect(),
        thread_join_funcs: JOIN_FUNCTIONS.iter().map(|s| (*s).to_string()).collect(),
        // Discovery is bounded by its (thread, function, call site) visited set,
        // not by this cap; the cap only truncates, and a truncated discovery now
        // abstains (`context_depth_limit_hit`). The default 10 is shallower than
        // many drivers' call depth, so raise it to keep those provable.
        max_context_depth: crate::race_true::MTA_MAX_CONTEXT_DEPTH,
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
    if mta.thread_graph.context_depth_limit_hit {
        return Err("mta-context-depth-limit-hit".into());
    }
    Ok(mta
        .thread_graph
        .threads
        .values()
        .map(|t| t.entry_function)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    use saf_core::air::{AirFunction, Instruction};
    use saf_core::id::make_id;
    use saf_core::ids::{InstId, ModuleId};

    fn fid(name: &str) -> FunctionId {
        FunctionId(make_id("func", name.as_bytes()))
    }

    fn declared(name: &str) -> AirFunction {
        let mut f = AirFunction::new(fid(name), name);
        f.is_declaration = true;
        f
    }

    fn call(callee: &str, operands: Vec<ValueId>) -> Instruction {
        Instruction::new(
            InstId(make_id("inst", callee.as_bytes())),
            Operation::CallDirect {
                callee: fid(callee),
            },
        )
        .with_operands(operands)
    }

    /// Runs `check_external_call` for `inst` against one stack slot `slot` whose
    /// data-layout size is `slot_size`, with `entry` the only reachable function.
    fn check(
        inst: &Instruction,
        slot: ValueId,
        slot_size: u64,
        ptr_width: u32,
    ) -> Result<(), String> {
        let mut module = AirModule::new(ModuleId(make_id("module", b"memsafe_test")));
        for name in [
            "pthread_join",
            "pthread_create",
            "pthread_mutex_lock",
            "pthread_mutex_init",
            "entry",
            "other",
        ] {
            let mut f = declared(name);
            f.is_declaration = !matches!(name, "entry" | "other");
            module.functions.push(f);
        }
        module
            .constants
            .insert(ValueId::new(0x10), Constant::Int { value: 1, bits: 64 });
        module.constants.insert(ValueId::new(0x11), Constant::Null);
        let globals = BTreeMap::new();
        let fn_addrs: BTreeMap<ValueId, FunctionId> = ["entry", "other"]
            .iter()
            .map(|n| (ValueId::new(fid(n).raw()), fid(n)))
            .collect();
        let stack_slots: BTreeMap<ValueId, Option<u64>> = [(slot, Some(slot_size))].into();
        let handle_sizes = stack_slots.clone();
        let defs = BTreeMap::new();
        let reachable: BTreeSet<FunctionId> = [fid("entry")].into();
        let scan = Scan {
            module: &module,
            globals: &globals,
            fn_addrs: &fn_addrs,
            stack_slots: &stack_slots,
            handle_sizes: &handle_sizes,
            defs: &defs,
            reachable: &reachable,
            ptr_width,
        };
        let Operation::CallDirect { callee } = inst.op else {
            return Err("not a call".into());
        };
        check_external_call(&scan, callee, inst)
    }

    /// m1: `char st; pthread_join(t, (void **)&st)` — the join stores a whole
    /// `void *` into a one-byte slot.
    #[test]
    fn join_retval_into_a_too_small_slot_abstains() {
        let st = ValueId::new(0x20);
        let join = call("pthread_join", vec![ValueId::new(0x10), st]);
        assert_eq!(
            check(&join, st, 1, 4),
            Err("pthread_join:handle-too-small".to_string())
        );
        assert_eq!(
            check(&join, st, 4, 8),
            Err("pthread_join:handle-too-small".to_string()),
            "a 4-byte slot is too small on LP64"
        );
        assert_eq!(check(&join, st, 4, 4), Ok(()));
        assert_eq!(check(&join, st, 8, 8), Ok(()));
        // `pthread_join(t, NULL)` dereferences nothing.
        let join_null = call("pthread_join", vec![ValueId::new(0x10), ValueId::new(0x11)]);
        assert_eq!(check(&join_null, st, 1, 4), Ok(()));
        // An unknown pointer width abstains.
        assert!(check(&join, st, 8, 2).is_err());
    }

    /// `pthread_mutex_lock(NULL)` / `pthread_create(NULL, …)` dereference NULL
    /// inside the runtime (`valid-deref`); an optional null attr does not.
    #[test]
    fn null_required_handle_abstains() {
        let null = ValueId::new(0x11);
        let lock = call("pthread_mutex_lock", vec![null]);
        assert_eq!(
            check(&lock, ValueId::new(0x30), 40, 8),
            Err("pthread_mutex_lock:null-handle".to_string())
        );
        let m = ValueId::new(0x30);
        let init = call("pthread_mutex_init", vec![m, null]);
        assert_eq!(check(&init, m, 40, 8), Ok(()));
    }

    #[test]
    fn mutex_handle_must_cover_the_runtime_mutex() {
        let m = ValueId::new(0x30);
        let lock = call("pthread_mutex_lock", vec![m]);
        assert_eq!(check(&lock, m, 24, 4), Ok(()));
        assert!(check(&lock, m, 24, 8).is_err(), "24 bytes < LP64 mutex");
        assert!(check(&lock, m, 4, 4).is_err());
        // An address that is neither a global nor a slot is unanchored.
        let lock_unknown = call("pthread_mutex_lock", vec![ValueId::new(0x31)]);
        assert_eq!(
            check(&lock_unknown, m, 40, 8),
            Err("pthread_mutex_lock:handle-unanchored".to_string())
        );
    }

    /// M2: the thread entry handed to `pthread_create` must be in the scanned
    /// universe.
    #[test]
    fn spawn_entry_must_be_reachable() {
        let t = ValueId::new(0x40);
        let null = ValueId::new(0x11);
        let ok = call(
            "pthread_create",
            vec![t, null, ValueId::new(fid("entry").raw()), null],
        );
        assert_eq!(check(&ok, t, 4, 4), Ok(()));
        let unscanned = call(
            "pthread_create",
            vec![t, null, ValueId::new(fid("other").raw()), null],
        );
        assert_eq!(
            check(&unscanned, t, 4, 4),
            Err("pthread_create:thread-entry-not-in-universe".to_string())
        );
        // The handle itself is size-checked too.
        assert_eq!(
            check(&ok, t, 4, 8),
            Err("pthread_create:handle-too-small".to_string())
        );
    }

    #[test]
    fn data_model_width_mismatch_abstains() {
        let mut module = AirModule::new(ModuleId(make_id("module", b"memsafe_dm")));
        module.target_pointer_width = 8;
        let proof = prove_memsafe_for_data_model(
            &module,
            IngestFidelity::default(),
            &BTreeSet::new(),
            crate::DataModel::ILP32,
        );
        assert_eq!(
            proof,
            MemSafeProof::Abstain("data-model-pointer-width-mismatch".into())
        );
    }
}
