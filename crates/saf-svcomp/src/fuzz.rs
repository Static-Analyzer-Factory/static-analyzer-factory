//! Blind byte-stream fuzzing support for the `unreach-call` FALSE pipeline.
//!
//! This module is **pure** (no I/O, no subprocess): it generates the C source of
//! a deterministic *byte-stream* nondet shim, a small AFL-style mutation engine,
//! an IR-constant dictionary harvester, and a parser turning the shim's runtime
//! log back into a [`NondetCall`] sequence. The `saf verify` `unreach-call`
//! strategy in `saf-cli` drives the search: it compiles the ORIGINAL program with
//! this shim (which consumes an input byte stream and dumps the values it
//! produced), runs an AFL-style blind mutation loop natively, and — when a run
//! reaches `reach_error` — re-confirms the discovered input sequence through the
//! EXISTING deterministic sequence-replay gate before emitting a verdict.
//!
//! # Why this is sound (self-confirming, a wrong FALSE is impossible)
//!
//! The shim is not the arbiter of the verdict; it is a *candidate generator*. A
//! fuzz run that drops the sentinel yields a concrete `(func, value)` sequence,
//! which is fed to the same [`crate::FalseCandidate`] concrete-replay path the Z3
//! stages use (native execution of the original, unsliced program with the inputs
//! pinned). Only that native replay reaching `reach_error` produces
//! `false(unreach-call)`; anything else stays `unknown`. So the fuzzer can only
//! ever *propose* — it cannot manufacture a wrong FALSE.
//!
//! Contract compliance (`scripts/loop/confirmer_contract.md`):
//! - **R1** — the sentinel is dropped ONLY by the shim's `reach_error` /
//!   `__VERIFIER_error` / `__assert_fail` overrides, i.e. the property's exact
//!   violation event. No catch-all trap confirms.
//! - **R4** — `__VERIFIER_assume(c)` blocks the run (`_exit`) when `c` is false,
//!   so an assume-pruned path can never be confirmed.
//! - **R5** — each `__VERIFIER_nondet_T()` consumes exactly `sizeof(T)` bytes and
//!   returns them reinterpreted as `T`; every bit pattern of an integer type is an
//!   in-range value of that type (two's-complement x86, no trap representations),
//!   and `_Bool` is normalised to `0`/`1`. Only the scalar-integer nondet family
//!   is fuzzed — pointer/float/double return the same `0`/`NULL` the replay driver
//!   uses, so the byte-stream path and the re-confirm path stay equivalent.
//! - **R6** — the discovered value sequence re-triggers the violation
//!   deterministically on the original program (the re-confirm) before emitting.

use crate::property::NondetCall;
use crate::property_kind::DataModel;
use saf_core::air::{AirModule, Constant};
use std::collections::BTreeSet;

/// Scalar-*integer* `__VERIFIER_nondet_*` functions the byte-stream shim drives,
/// paired with their C return type. Kept identical to the replay driver's list in
/// `saf-cli` so a fuzz-discovered sequence re-confirms byte-for-byte through the
/// existing [`crate::FalseCandidate`] gate.
pub const SCALAR_NONDET: &[(&str, &str)] = &[
    ("__VERIFIER_nondet_int", "int"),
    ("__VERIFIER_nondet_uint", "unsigned int"),
    ("__VERIFIER_nondet_long", "long"),
    ("__VERIFIER_nondet_ulong", "unsigned long"),
    ("__VERIFIER_nondet_longlong", "long long"),
    ("__VERIFIER_nondet_ulonglong", "unsigned long long"),
    ("__VERIFIER_nondet_short", "short"),
    ("__VERIFIER_nondet_ushort", "unsigned short"),
    ("__VERIFIER_nondet_char", "char"),
    ("__VERIFIER_nondet_uchar", "unsigned char"),
    ("__VERIFIER_nondet_bool", "_Bool"),
    ("__VERIFIER_nondet_size_t", "size_t"),
];

/// Fixed-width integer `__VERIFIER_nondet_*` typedefs from the SV-COMP nondet API
/// whose bit width is **fixed by the standard** (data-model independent), paired with
/// a C type of exactly that width.
///
/// These are the `uN`/`sN` (stdint-style) and `unsigned`/`loff_t` generators that
/// pervade the device-driver harnesses (ldv-linux / ddv-machzwd) and the
/// firmware/entry-point batches. Unlike [`SCALAR_NONDET`], a task almost never
/// supplies a body for them — the harness only *declares* `extern u32
/// __VERIFIER_nondet_u32(void)` and expects the verifier to model it — so without a
/// definition the native harness fails to LINK and the whole fuzz/replay confirmer
/// abstains before it can drive a single input.
///
/// They are emitted as **WEAK** definitions (see [`push_bytestream_nondet_defs`] /
/// the replay driver) so that on the rare task that *does* define its own body
/// (e.g. `u32 __VERIFIER_nondet_u32(void){ return __VERIFIER_nondet_uint(); }`) the
/// task's strong symbol wins the link and behaviour is unchanged — the driven
/// definition only fills in the missing extern, never overrides a real one. Widths
/// are model-independent (`unsigned char`/`short`/`int`/`long long` are 1/2/4/8 on
/// both ILP32 and LP64), so the same `sizeof(T)`-byte consumption re-confirms
/// byte-for-byte across data models (R5).
pub const EXTENDED_NONDET: &[(&str, &str)] = &[
    ("__VERIFIER_nondet_u8", "unsigned char"),
    ("__VERIFIER_nondet_u16", "unsigned short"),
    ("__VERIFIER_nondet_u32", "unsigned int"),
    ("__VERIFIER_nondet_u64", "unsigned long long"),
    ("__VERIFIER_nondet_s8", "signed char"),
    ("__VERIFIER_nondet_s16", "short"),
    ("__VERIFIER_nondet_s32", "int"),
    ("__VERIFIER_nondet_s64", "long long"),
    ("__VERIFIER_nondet_unsigned", "unsigned int"),
    ("__VERIFIER_nondet_loff_t", "long long"),
];

/// Byte size of the zero-filled heap object a fuzzable `__VERIFIER_nondet_pointer()`
/// returns (see [`FUZZ_NONDET_POINTER_C`]). Generous enough to cover a typical harness
/// struct/array so field reads land inside the deterministic zero region.
pub const NONDET_OBJ_SIZE: usize = 4096;

/// Fuzz-shim definition of a **lazy-initialised** `__VERIFIER_nondet_pointer()`
/// (KLEE-style symbolic-object havocking).
///
/// It consumes ONE selector byte from the input stream and logs it (so the replay
/// driver reproduces the exact choice, R6): an **even** selector returns `NULL`, an
/// **odd** selector returns a fresh, zero-filled heap object of [`NONDET_OBJ_SIZE`]
/// bytes. Both the NULL and the valid-object outcomes of the nondeterministic pointer
/// are therefore explorable in a single run.
///
/// # Why this is sound for unreach-call
///
/// `__VERIFIER_nondet_pointer()` returns an *arbitrary* pointer; NULL and a valid
/// pointer to a zeroed object are both concrete members of that nondeterministic set.
/// The common firmware/entry-point harness shape is `p =
/// __VERIFIER_nondet_pointer(); if (!p) return; p->field = __VERIFIER_nondet_int();
/// ...` — with a hard `NULL` the harness bails and NOTHING downstream is explored;
/// with the valid-object branch it proceeds and the fields are driven by the ordinary
/// scalar nondet generators, whose values ARE logged and replayed. Reaching
/// `reach_error` this way is a genuine feasible execution, only ever *emitted* after
/// the deterministic native re-confirm on the ORIGINAL program (R6) — the shim never
/// decides a verdict. A genuinely-safe (TRUE) task has no nondet pointer value
/// reaching `reach_error`, so this cannot manufacture a wrong FALSE.
///
/// The object is left zero-filled (not byte-stream-filled) so it re-confirms through
/// the EXISTING scalar-sequence replay gate without capturing pointee bytes: field
/// *contents* come from replayed scalar nondet writes, not from the pointer function.
/// Reads of an unwritten field observe `0`, a valid nondet concretisation. Emitted
/// inside the driver body (it references `__saf_take`/`__saf_note`).
const FUZZ_NONDET_POINTER_C: &str = "void* __VERIFIER_nondet_pointer(void) { unsigned long long __s = __saf_take(1); __saf_note(\"__VERIFIER_nondet_pointer\", (long long)__s); return (__s & 1ULL) ? calloc(1, 4096) : (void*)0; }\n";

/// Byte-stream `__VERIFIER_nondet_float` / `_double` definitions. Each consumes
/// `sizeof(T)` little-endian bytes, reinterprets them as the float via `memcpy`
/// (no strict-aliasing UB), logs the raw bit pattern (so replay is bit-exact —
/// R6), and returns the value. Emitted inside the driver body (references
/// `__saf_take` / `__saf_note`; `string.h` is already included).
const FUZZ_NONDET_FLOAT_C: &str = concat!(
    "float __VERIFIER_nondet_float(void) { unsigned int __b = (unsigned int)__saf_take(sizeof(float)); ",
    "float __f; memcpy(&__f, &__b, sizeof(__f)); ",
    "__saf_note(\"__VERIFIER_nondet_float\", (long long)(unsigned long long)__b); return __f; }\n",
    "double __VERIFIER_nondet_double(void) { unsigned long long __b = __saf_take(sizeof(double)); ",
    "double __d; memcpy(&__d, &__b, sizeof(__d)); ",
    "__saf_note(\"__VERIFIER_nondet_double\", (long long)__b); return __d; }\n",
);

/// Classic AFL "interesting" integer values (8/16/32-bit boundary and near-boundary
/// constants). Used both as a fixed part of the mutation dictionary and as the seed
/// of the harvested dictionary so a guard like `if (nondet()==INT_MAX)` is reachable
/// even when the program carries no literal for the boundary.
pub const INTERESTING_VALUES: &[i64] = &[
    0,
    1,
    -1,
    2,
    16,
    32,
    64,
    100,
    127,
    -128,
    128,
    255,
    256,
    512,
    1000,
    1024,
    4096,
    32767,
    -32768,
    65535,
    2_147_483_647,
    -2_147_483_648,
];

/// "Interesting" floating-point values seeded into the FLOAT byte-stream fuzz
/// corpus (as both `f32` and `f64` bit-pattern tiles). A random `u32`/`u64`
/// reinterpreted as a float is almost always a huge magnitude, subnormal, NaN, or
/// infinity — so blind byte mutation essentially never lands the small in-range
/// values that guard-narrow float tasks (`assume(x > -0.8 && x < 0.8)`,
/// `if (!(x < 0.1)) reach_error()`) require. These small-magnitude and boundary
/// values give the fuzzer a productive starting point; the program's own float
/// constants (harvested by the backward slice, see
/// [`crate::slicing::Slice::guard_floats`]) are added on top per task.
pub const INTERESTING_FLOATS: &[f64] = &[
    0.0, 1.0, -1.0, 0.5, -0.5, 0.25, -0.25, 0.75, -0.75, 0.1, -0.1, 0.9, -0.9, 2.0, -2.0, 10.0,
    -10.0, 100.0, -100.0, 1e6, -1e6,
];

/// True iff `name` is a float/double nondet generator the byte-stream shim now
/// drives (consumes `sizeof(T)` bytes → reinterprets as the float → logs the
/// IEEE-754 bit pattern so the value-sequence replay reconstructs it bit-for-bit).
#[must_use]
pub fn is_float_nondet(name: &str) -> bool {
    name == "__VERIFIER_nondet_float" || name == "__VERIFIER_nondet_double"
}

/// True iff the byte-stream shim drives `name`: the standard scalar-integer family
/// ([`SCALAR_NONDET`]), the fixed-width typedef family ([`EXTENDED_NONDET`]), or the
/// lazy-init nondet pointer (whose NULL/object selector is fuzzed).
#[must_use]
pub fn is_fuzzable_nondet(name: &str) -> bool {
    SCALAR_NONDET.iter().any(|(n, _)| *n == name)
        || EXTENDED_NONDET.iter().any(|(n, _)| *n == name)
        || name == "__VERIFIER_nondet_pointer"
}

/// True iff the program references any nondet input the byte-stream shim can drive
/// (see [`is_fuzzable_nondet`]). When it does not, blind fuzzing cannot change
/// behaviour — the single deterministic path is already covered by the earlier
/// stages — so the caller skips the fuzzer.
#[must_use]
pub fn references_scalar_nondet(module: &AirModule) -> bool {
    module.functions.iter().any(|f| is_fuzzable_nondet(&f.name))
}

/// True iff the program references a float/double nondet input the byte-stream
/// shim can now drive ([`is_float_nondet`]). A mere declaration counts.
#[must_use]
pub fn references_float_nondet(module: &AirModule) -> bool {
    module.functions.iter().any(|f| is_float_nondet(&f.name))
}

/// True iff the program references ANY nondet input the byte-stream shim can drive
/// — scalar-integer / typedef / pointer ([`references_scalar_nondet`]) OR
/// float/double ([`references_float_nondet`]). This is the gate the blind fuzzer
/// keys on: without any fuzzable nondet, the single deterministic path is already
/// covered by the earlier stages, so the fuzzer is a no-op. Float/double programs
/// were previously excluded (the shim returned a fixed `0.0`), leaving the entire
/// `floats-*` / `nla-digbench`(double) class unreachable by the portfolio.
#[must_use]
pub fn references_fuzzable_nondet(module: &AirModule) -> bool {
    references_scalar_nondet(module) || references_float_nondet(module)
}

/// Number of scalar-integer `__VERIFIER_nondet_*` CALL SITES in the module's
/// defined functions (each direct call counted once, in no particular order).
///
/// The overflow confirmer's positional pass uses this to decide whether giving
/// distinct per-call values can help: with fewer than two call sites a positional
/// value equals the uniform `SAF_NONDET_CONST` sweep already tried, so the pass would
/// add nothing. A cheap over-approximation (whole-module, not reachability-scoped) —
/// over-counting only risks a few extra fast native runs, never a missed gate.
#[must_use]
pub fn count_scalar_nondet_call_sites(module: &AirModule) -> usize {
    use saf_core::air::Operation;
    let mut count = 0usize;
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if SCALAR_NONDET.iter().any(|(name, _)| *name == target.name) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

/// A cast whose result keeps the SCALAR (integer) taint of its operand: every cast
/// kind EXCEPT `IntToPtr` (which produces a pointer — handled separately) and the
/// float-producing casts (a float can no longer index memory as an integer pointer).
/// `Trunc`/`ZExt`/`SExt`/`Bitcast`/`PtrToInt`/`AddrSpaceCast` keep an int derived from
/// the nondet bytes. Helper for [`nondet_taints_int_to_ptr_deref`].
fn cast_keeps_scalar_taint(kind: saf_core::air::CastKind) -> bool {
    use saf_core::air::CastKind;
    matches!(
        kind,
        CastKind::Trunc
            | CastKind::ZExt
            | CastKind::SExt
            | CastKind::PtrToInt
            | CastKind::Bitcast
            | CastKind::AddrSpaceCast
    )
}

/// All value inputs of an instruction: its `operands` plus (for `Phi`) the incoming
/// values, which are carried in the operation rather than in `operands`. Helper for
/// [`nondet_taints_int_to_ptr_deref`].
fn instruction_value_inputs(inst: &saf_core::air::Instruction) -> Vec<saf_core::ids::ValueId> {
    use saf_core::air::Operation;
    let mut v = inst.operands.clone();
    if let Operation::Phi { incoming } = &inst.op {
        v.extend(incoming.iter().map(|(_, val)| *val));
    }
    v
}

/// True iff a DEF-USE taint shows a scalar `__VERIFIER_nondet_*` result flowing
/// (through value-preserving casts / copies / selects / phis) into an
/// `Operation::Cast { kind: IntToPtr }` whose resulting pointer is then
/// dereferenced by a `Load` or `Store`.
///
/// # Why the blind fuzzer must ABSTAIN when this holds (soundness sentinel #1)
///
/// The byte-stream shim drives every scalar `__VERIFIER_nondet_T()` with arbitrary
/// input bytes. When such a value is cast straight to a pointer
/// (`(void*)__VERIFIER_nondet_ulong()`, as in `aws-c-common`'s
/// `aws_string_new_from_array_harness`) and then dereferenced, the fuzzer can
/// fabricate an *arbitrary invalid pointer*, dereference it, and reach
/// `reach_error` on a path the real program never takes — a spurious FALSE
/// (a full-svcomp25 false alarm). The replay driver already returns `NULL` for
/// `__VERIFIER_nondet_pointer`, but an integer→pointer cast bypasses that guard.
/// Reaching the error via such a synthesised pointer deref is not a genuine
/// property violation, so the fuzz confirmer abstains.
///
/// This is a **precise** taint (nondet ⟶ casts ⟶ `IntToPtr` ⟶ deref), NOT a blunt
/// "the program contains any `inttoptr`" gate — the latter would abstain on every
/// program that legitimately materialises a pointer from an integer and tank recall.
/// Recall on genuinely-buggy pointer harnesses is recovered later by symbolic
/// harness-havoc, not by the blind byte-stream fuzzer.
///
/// The analysis is intraprocedural and value-based: the module is `mem2reg`-promoted
/// before ingestion, so a scalar nondet result flows to the `inttoptr` directly
/// through SSA values (no intervening `alloca`/`store`/`load`). Being value-based it
/// naturally stays within a single function. Deterministic (`BTreeSet` ordering,
/// fixpoint over a fixed instruction list).
// NOTE: the fixpoint phases (seed the scalar taint, propagate it, seed + propagate
// the pointer taint, then check the deref) are one cohesive dataflow over a single
// per-function instruction list; splitting them into separate passes would duplicate
// the setup and obscure the taint pipeline.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn nondet_taints_int_to_ptr_deref(module: &AirModule) -> bool {
    use saf_core::air::{CastKind, Instruction, Operation};
    use saf_core::ids::ValueId;

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let insts: Vec<&Instruction> = func
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter())
            .collect();

        // Seed the SCALAR taint with the result value of every scalar-integer nondet
        // call site.
        let mut scalar: BTreeSet<ValueId> = BTreeSet::new();
        for inst in &insts {
            if let Operation::CallDirect { callee } = &inst.op {
                if let Some(target) = module.function(*callee) {
                    if SCALAR_NONDET.iter().any(|(name, _)| *name == target.name) {
                        if let Some(dst) = inst.dst {
                            scalar.insert(dst);
                        }
                    }
                }
            }
        }
        if scalar.is_empty() {
            continue;
        }

        // Fixpoint: propagate the scalar taint through value-preserving casts / copies
        // / freezes / selects / phis (any tainted input taints the result).
        let mut changed = true;
        while changed {
            changed = false;
            for inst in &insts {
                let Some(dst) = inst.dst else { continue };
                if scalar.contains(&dst) {
                    continue;
                }
                let tainted_in = instruction_value_inputs(inst)
                    .iter()
                    .any(|o| scalar.contains(o));
                if !tainted_in {
                    continue;
                }
                let forwards = match &inst.op {
                    Operation::Cast { kind, .. } => cast_keeps_scalar_taint(*kind),
                    Operation::Copy
                    | Operation::Freeze
                    | Operation::Select
                    | Operation::Phi { .. } => true,
                    _ => false,
                };
                if forwards {
                    scalar.insert(dst);
                    changed = true;
                }
            }
        }

        // An `IntToPtr` cast whose operand is scalar-tainted yields a TAINTED POINTER.
        let mut ptr: BTreeSet<ValueId> = BTreeSet::new();
        for inst in &insts {
            if let Operation::Cast {
                kind: CastKind::IntToPtr,
                ..
            } = &inst.op
            {
                if inst.operands.iter().any(|o| scalar.contains(o)) {
                    if let Some(dst) = inst.dst {
                        ptr.insert(dst);
                    }
                }
            }
        }
        if ptr.is_empty() {
            continue;
        }

        // Fixpoint: propagate the pointer taint through address-preserving ops —
        // `Gep` (only when the BASE, operand[0], is tainted), pointer-preserving
        // casts, copies, selects and phis.
        let mut changed = true;
        while changed {
            changed = false;
            for inst in &insts {
                let Some(dst) = inst.dst else { continue };
                if ptr.contains(&dst) {
                    continue;
                }
                let forwards = match &inst.op {
                    Operation::Gep { .. } => inst.operands.first().is_some_and(|o| ptr.contains(o)),
                    Operation::Cast {
                        kind: CastKind::Bitcast | CastKind::AddrSpaceCast,
                        ..
                    }
                    | Operation::Copy
                    | Operation::Freeze
                    | Operation::Select
                    | Operation::Phi { .. } => instruction_value_inputs(inst)
                        .iter()
                        .any(|o| ptr.contains(o)),
                    _ => false,
                };
                if forwards {
                    ptr.insert(dst);
                    changed = true;
                }
            }
        }

        // Abstain iff a tainted pointer is the ADDRESS operand of a load or store
        // (`Load` operand[0]; `Store` operand[1] — operand[0] is the stored value).
        for inst in &insts {
            let deref = match &inst.op {
                Operation::Load => inst.operands.first().is_some_and(|o| ptr.contains(o)),
                Operation::Store => inst.operands.get(1).is_some_and(|o| ptr.contains(o)),
                _ => false,
            };
            if deref {
                return true;
            }
        }
    }

    false
}

/// True iff the program references `__VERIFIER_nondet_bool` (declared or defined).
///
/// Used by the overflow confirmer to decide whether the loop-sustaining bool-decoupling
/// replay pass can change behaviour — a program with no `nondet_bool` cannot have a
/// `while (__VERIFIER_nondet_bool()) { … }` guard, so the extra pass would only waste
/// native runs. Cheap and conservative (a mere declaration counts).
#[must_use]
pub fn references_nondet_bool(module: &AirModule) -> bool {
    module
        .functions
        .iter()
        .any(|f| f.name == "__VERIFIER_nondet_bool")
}

/// Harvest a mutation dictionary: every distinct integer constant in the module
/// (the values guards compare against) plus [`INTERESTING_VALUES`]. Deterministic
/// (`BTreeSet` ordering), deduplicated, and bounded to keep the mutation loop
/// cheap.
#[must_use]
pub fn harvest_dictionary(module: &AirModule) -> Vec<i64> {
    let mut set: BTreeSet<i64> = INTERESTING_VALUES.iter().copied().collect();
    for c in module.constants.values() {
        match c {
            Constant::Int { value, .. } => {
                set.insert(*value);
            }
            Constant::BigInt { value, .. } => {
                if let Ok(v) = value.parse::<i64>() {
                    set.insert(v);
                }
            }
            _ => {}
        }
    }
    // Bound: a huge module would otherwise inflate the dictionary; the boundary
    // constants that steer guards are the load-bearing ones. Ordering is
    // deterministic, so the truncation is reproducible.
    set.into_iter().take(MAX_DICT_ENTRIES).collect()
}

/// Cap on dictionary size — keeps mutation cheap and reproducible.
pub const MAX_DICT_ENTRIES: usize = 256;

/// Fixed length of every fuzz input buffer (bytes). 256 bytes covers 32 `int`
/// reads / 64 `short` reads — deep enough for the sv-benchmarks reach tasks, while
/// a CONSTANT length keeps mutation allocation-free and the search deterministic.
pub const INPUT_LEN: usize = 256;

/// Standalone-SanitizerCoverage feedback block appended to the byte-stream driver.
///
/// Defines the callbacks the `inline-8bit-counters`, `pc-table` and `trace-cmp`
/// instrumentation kinds require (no sanitizer runtime is linked, so we own them),
/// and a `constructor`/`atexit` pair that dumps the coverage map + harvested
/// comparison operands on normal exit. The whole block is INERT when the harness is
/// compiled without `-fsanitize-coverage`: the init callbacks are simply never
/// invoked, the captured pointers stay null, and the dump writes nothing.
///
/// The CmpLog table is a fixed-size open-addressed hash (O(1) per comparison — no
/// linear scan in hot loops); collisions merely lose a candidate value, which only
/// costs recall, never soundness.
const COVERAGE_FEEDBACK_C: &str = "\
/* --- SAF SanitizerCoverage feedback (inert without -fsanitize-coverage) --- */\n\
/* CRITICAL: every callback/helper below is compiled in the SAME TU as the program\n\
 * under -fsanitize-coverage, so its OWN comparisons would be instrumented and\n\
 * re-enter the trace-cmp callbacks -> infinite recursion / stack overflow. Exclude\n\
 * them from instrumentation (matches how the real sanitizer runtime is built). */\n\
#define __SAF_NOCOV __attribute__((no_sanitize(\"coverage\")))\n\
static unsigned char* __saf_cov_start = 0;\n\
static unsigned char* __saf_cov_stop = 0;\n\
__SAF_NOCOV void __sanitizer_cov_8bit_counters_init(unsigned char* s, unsigned char* e) { __saf_cov_start = s; __saf_cov_stop = e; }\n\
static const void* __saf_pcs_beg = 0;\n\
static const void* __saf_pcs_end = 0;\n\
__SAF_NOCOV void __sanitizer_cov_pcs_init(const void* b, const void* e) { __saf_pcs_beg = b; __saf_pcs_end = e; }\n\
#define __SAF_CMP_SLOTS 512\n\
static unsigned long long __saf_cmp_tab[__SAF_CMP_SLOTS];\n\
__SAF_NOCOV static void __saf_cmp_note(unsigned long long v) {\n\
  if (v <= 1ULL) return;\n\
  unsigned long long h = (v * 0x9E3779B97F4A7C15ULL) >> 55;\n\
  __saf_cmp_tab[h & (__SAF_CMP_SLOTS - 1)] = v;\n\
}\n\
/* Redqueen-style input-to-state table: distinct (from,to,width) operand pairs so\n\
 * the fuzz loop can locate `from` in the input bytes and patch it to `to`. */\n\
#define __SAF_I2S_SLOTS 1024\n\
static unsigned long long __saf_i2s_a[__SAF_I2S_SLOTS];\n\
static unsigned long long __saf_i2s_b[__SAF_I2S_SLOTS];\n\
static unsigned char __saf_i2s_w[__SAF_I2S_SLOTS];\n\
__SAF_NOCOV static void __saf_i2s_note(unsigned long long a, unsigned long long b, unsigned char w) {\n\
  if (a == b) return;\n\
  unsigned long long h = (a * 0x9E3779B97F4A7C15ULL) ^ (b * 0xC2B2AE3D27D4EB4FULL);\n\
  unsigned idx = (unsigned)((h >> 54) & (__SAF_I2S_SLOTS - 1));\n\
  __saf_i2s_a[idx] = a; __saf_i2s_b[idx] = b; __saf_i2s_w[idx] = w;\n\
}\n\
__SAF_NOCOV static void __saf_cmp(unsigned long long a, unsigned long long b, unsigned char w) {\n\
  __saf_cmp_note(a); __saf_cmp_note(b);\n\
  __saf_i2s_note(a, b, w); __saf_i2s_note(b, a, w);\n\
}\n\
__SAF_NOCOV void __sanitizer_cov_trace_cmp1(unsigned char a, unsigned char b) { __saf_cmp(a, b, 1); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_cmp2(unsigned short a, unsigned short b) { __saf_cmp(a, b, 2); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_cmp4(unsigned int a, unsigned int b) { __saf_cmp(a, b, 4); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_cmp8(unsigned long long a, unsigned long long b) { __saf_cmp(a, b, 8); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_const_cmp1(unsigned char a, unsigned char b) { __saf_cmp(a, b, 1); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_const_cmp2(unsigned short a, unsigned short b) { __saf_cmp(a, b, 2); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_const_cmp4(unsigned int a, unsigned int b) { __saf_cmp(a, b, 4); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_const_cmp8(unsigned long long a, unsigned long long b) { __saf_cmp(a, b, 8); }\n\
__SAF_NOCOV void __sanitizer_cov_trace_switch(unsigned long long val, unsigned long long* cases) {\n\
  unsigned long long n = cases[0]; unsigned long long bw = cases[1] / 8; unsigned long long i;\n\
  unsigned char w = (bw == 1 || bw == 2 || bw == 4 || bw == 8) ? (unsigned char)bw : 8;\n\
  for (i = 0; i < n; i++) __saf_cmp(val, cases[2 + i], w);\n\
}\n\
__SAF_NOCOV static void __saf_cov_dump(void) {\n\
  const char* cp = getenv(\"SAF_FUZZ_COV\");\n\
  if (cp && __saf_cov_start && __saf_cov_stop && __saf_cov_stop > __saf_cov_start) {\n\
    FILE* f = fopen(cp, \"wb\");\n\
    if (f) { fwrite(__saf_cov_start, 1, (size_t)(__saf_cov_stop - __saf_cov_start), f); fclose(f); }\n\
  }\n\
  const char* mp = getenv(\"SAF_FUZZ_CMPLOG\");\n\
  if (mp) {\n\
    FILE* f = fopen(mp, \"w\");\n\
    if (f) { int i; for (i = 0; i < __SAF_CMP_SLOTS; i++) { if (__saf_cmp_tab[i]) fprintf(f, \"%llu\\n\", __saf_cmp_tab[i]); } fclose(f); }\n\
  }\n\
  const char* ip = getenv(\"SAF_FUZZ_I2S\");\n\
  if (ip) {\n\
    FILE* f = fopen(ip, \"w\");\n\
    if (f) { int i; for (i = 0; i < __SAF_I2S_SLOTS; i++) { if (__saf_i2s_w[i]) fprintf(f, \"%u %llu %llu\\n\", (unsigned)__saf_i2s_w[i], __saf_i2s_a[i], __saf_i2s_b[i]); } fclose(f); }\n\
  }\n\
}\n\
__attribute__((constructor)) static void __saf_cov_ctor(void) { atexit(__saf_cov_dump); }\n";

/// Emit byte-stream-driven definitions for a nondet `table` into `s`.
///
/// Each `__VERIFIER_nondet_T()` consumes `sizeof(T)` little-endian bytes from the
/// input buffer, reinterprets them as `T`, logs `"<name> <value>\n"` (so a
/// fuzz-discovered sequence re-confirms through the replay gate, R6), and returns the
/// value. `_Bool` is normalised to `0`/`1` (an arbitrary byte in a `_Bool` object is
/// UB). When `weak`, each definition is `__attribute__((weak))` so a task that
/// supplies its own body wins the link — used for [`EXTENDED_NONDET`], whose typedef
/// generators device-driver models occasionally define themselves.
fn push_bytestream_nondet_defs(s: &mut String, table: &[(&str, &str)], weak: bool) {
    use std::fmt::Write as _;
    let attr = if weak { "__attribute__((weak)) " } else { "" };
    for (fname, cty) in table {
        if *fname == "__VERIFIER_nondet_bool" {
            let _ = writeln!(
                s,
                "{attr}_Bool {fname}(void) {{ unsigned long long r = __saf_take(1); _Bool v = (_Bool)(r & 1ULL); __saf_note(\"{fname}\", (long long)v); return v; }}"
            );
            continue;
        }
        // Reinterpret the low sizeof(T) little-endian bytes as T, then log the value
        // reinterpreted back to `long long` (bit-preserving on two's-complement), so
        // the sequence-replay driver's `(T)value` cast recovers the exact T value.
        let _ = writeln!(
            s,
            "{attr}{cty} {fname}(void) {{ {cty} v; unsigned long long r = __saf_take(sizeof({cty})); memcpy(&v, &r, sizeof({cty})); __saf_note(\"{fname}\", (long long)v); return v; }}"
        );
    }
}

/// Generate the C source of the byte-stream nondet shim + error sentinel driver.
///
/// Each scalar-integer `__VERIFIER_nondet_T()` consumes `sizeof(T)` little-endian
/// bytes from the input buffer (loaded once from `$SAF_FUZZ_INPUT`; exhausted ⇒
/// `0`), reinterprets them as `T`, appends `"<name> <value>\n"` to
/// `$SAF_FUZZ_LOG`, and returns the value. `reach_error` / `__VERIFIER_error` /
/// `__assert_fail` drop `sentinel` and `_exit`; `__VERIFIER_assume(false)` exits
/// without a hit. Float/double nondet return `0`; a nondet pointer returns a fresh
/// zero-filled object (see [`NONDET_POINTER_OBJ_C`]).
#[must_use]
pub fn synthesize_bytestream_driver(sentinel_c_literal: &str) -> String {
    use std::fmt::Write as _;

    let mut s = String::new();
    s.push_str("/* SAF blind-fuzz byte-stream driver (generated) */\n");
    let _ = writeln!(s, "#define __SAF_SENTINEL \"{sentinel_c_literal}\"");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("#include <string.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");
    let _ = writeln!(s, "static unsigned char __saf_buf[{INPUT_LEN}];");
    s.push_str("static size_t __saf_len = 0;\n");
    s.push_str("static size_t __saf_pos = 0;\n");
    s.push_str("static FILE* __saf_log = 0;\n");
    s.push_str("static int __saf_ready = 0;\n");
    s.push_str(
        "static void __saf_init(void) {\n\
         \x20 if (__saf_ready) return;\n\
         \x20 __saf_ready = 1;\n\
         \x20 const char* ip = getenv(\"SAF_FUZZ_INPUT\");\n\
         \x20 if (ip) { FILE* f = fopen(ip, \"rb\"); if (f) { __saf_len = fread(__saf_buf, 1, sizeof(__saf_buf), f); fclose(f); } }\n\
         \x20 const char* lp = getenv(\"SAF_FUZZ_LOG\");\n\
         \x20 if (lp) __saf_log = fopen(lp, \"w\");\n\
         }\n",
    );
    s.push_str(
        "static unsigned long long __saf_take(size_t n) {\n\
         \x20 __saf_init();\n\
         \x20 unsigned long long v = 0; size_t i;\n\
         \x20 for (i = 0; i < n; i++) { unsigned char b = (__saf_pos < __saf_len) ? __saf_buf[__saf_pos] : 0; __saf_pos++; v |= ((unsigned long long)b) << (8 * i); }\n\
         \x20 return v;\n\
         }\n",
    );
    s.push_str(
        "static void __saf_note(const char* name, long long val) {\n\
         \x20 if (__saf_log) { fprintf(__saf_log, \"%s %lld\\n\", name, val); fflush(__saf_log); }\n\
         }\n",
    );

    // Standard scalar-integer nondet family (strong defs), then the fixed-width
    // typedef family (weak, so a task-supplied body wins). Both are byte-stream
    // driven and logged, so a discovered sequence re-confirms through the replay gate.
    push_bytestream_nondet_defs(&mut s, SCALAR_NONDET, false);
    push_bytestream_nondet_defs(&mut s, EXTENDED_NONDET, true);

    // SanitizerCoverage feedback (INERT unless the harness is compiled with
    // `-fsanitize-coverage=...`). We define the standalone-sancov callbacks the
    // instrumentation requires — no sanitizer runtime is linked, so the compiler
    // expects us to provide them:
    //   * inline-8bit-counters -> `__sanitizer_cov_8bit_counters_init(start, stop)`
    //   * pc-table             -> `__sanitizer_cov_pcs_init(beg, end)`
    //   * trace-cmp            -> the `__sanitizer_cov_trace_cmp*` family
    // At normal exit we dump the raw counter array to `$SAF_FUZZ_COV` (the fuzz loop
    // buckets it AFL-style for new-edge seed keeping) and the harvested comparison
    // operands to `$SAF_FUZZ_CMPLOG` (a CmpLog dynamic dictionary that cracks
    // magic-value guards the static dictionary misses). This ONLY steers the search
    // — the verdict is still produced by deterministic native replay (R6), so
    // coverage feedback can never manufacture a wrong FALSE.
    s.push_str(COVERAGE_FEEDBACK_C);

    // Non-integer nondet defaults. The nondet pointer is lazy-init havocked: it logs a
    // selector so the replay driver reproduces the same NULL / zero-filled-object
    // choice (R6), letting a harness that takes a nondet struct/array pointer run past
    // the pointer instead of bailing at a NULL check.
    s.push_str(FUZZ_NONDET_POINTER_C);
    // Float/double nondet: consume `sizeof(T)` bytes from the stream, reinterpret
    // as the float, and LOG THE IEEE-754 BIT PATTERN (not the value) so the
    // value-sequence replay driver reconstructs the identical bits (R6). Logging
    // the pattern as a signed `long long` round-trips exactly through the i64 the
    // fuzz log carries.
    s.push_str(FUZZ_NONDET_FLOAT_C);
    s.push_str("void __VERIFIER_atomic_begin(void) { }\n");
    s.push_str("void __VERIFIER_atomic_end(void) { }\n");

    // R4: honour assumptions at runtime.
    s.push_str("void __VERIFIER_assume(int c) { if (!c) _exit(0); }\n");

    // R1: the sentinel is dropped ONLY by the error sinks — the property's exact
    // violation event. WEAK reach_error/__VERIFIER_error so a task-defined
    // `reach_error(){ __assert_fail(...); }` wins the link; __assert_fail is
    // overridden so that path still trips the sentinel.
    s.push_str(
        "__attribute__((noreturn)) static void __saf_hit(void) { if (__saf_log) fflush(__saf_log); FILE* f = fopen(__SAF_SENTINEL, \"w\"); if (f) { fputc('1', f); fclose(f); } _exit(0); }\n",
    );
    s.push_str("__attribute__((weak)) void reach_error(void) { __saf_hit(); }\n");
    s.push_str("__attribute__((weak)) void __VERIFIER_error(void) { __saf_hit(); }\n");
    s.push_str(
        "__attribute__((noreturn)) void __assert_fail(const char* a, const char* b, unsigned int c, const char* d) { (void)a; (void)b; (void)c; (void)d; __saf_hit(); }\n",
    );

    // Allocation-failure PRUNE (soundness sentinel: aws-c-common). See
    // `ALLOC_PRUNE_WRAP_C` — model SV-COMP's unbounded-memory `malloc` semantics so a
    // concrete OOM on a nondet-driven huge size never fabricates a wrong FALSE.
    s.push_str(ALLOC_PRUNE_WRAP_C);

    s
}

/// Linker-`--wrap` interceptors for `malloc`/`calloc`/`realloc` that PRUNE the run on a
/// concrete allocation failure — the fix for the `aws-c-common` soundness-sentinel FP.
///
/// # The unsoundness this closes
///
/// SV-COMP's memory model treats `malloc` as drawing from **unbounded** memory: an
/// allocation of any *representable* size succeeds (a program observes a NULL return
/// only where it explicitly models one, e.g. `nondet_bool() ? NULL : malloc(...)`).
/// The `aws-c-common` CBMC proof harnesses rely on this — `bounded_malloc(size)` merely
/// `assume`s `size` is within a huge bound and returns `malloc(size)`, taking success
/// for granted. Under **concrete native replay**, though, the blind fuzzer can drive
/// that size nondet to hundreds of GB, so the REAL `malloc` returns NULL, and a later
/// NULL-consistency assertion (`aws_string_new_from_array_harness`'s
/// `assert(!a == !b)`, with `a` a live string and `b` the failed allocation) trips
/// `reach_error` on a path SV-COMP deems infeasible — a wrong `false(unreach-call)`.
///
/// # Why pruning is sound
///
/// `__wrap_malloc` calls `__real_malloc`; on a NULL result for a **nonzero** request it
/// `_exit(0)`s WITHOUT dropping the sentinel — i.e. it treats the run as an infeasible
/// path (the SV-COMP model would have allocated), exactly like an
/// `__VERIFIER_assume(0)`. It never confirms anything, so it cannot manufacture a
/// FALSE; it only removes OOM-only reaches. Programs that legitimately branch on a NULL
/// return get that NULL from an explicit model (a literal `NULL`, not a *failed*
/// `malloc`) — a small concrete `malloc` there succeeds regardless — so no genuine
/// NULL-handling FALSE is lost. `calloc`/`realloc` are wrapped identically; a `malloc(0)`
/// / `realloc(p,0)` implementation-defined NULL is passed through unchanged (size 0 is
/// not an OOM).
pub const ALLOC_PRUNE_WRAP_C: &str = "\
extern void* __real_malloc(size_t);\n\
extern void* __real_calloc(size_t, size_t);\n\
extern void* __real_realloc(void*, size_t);\n\
void* __wrap_malloc(size_t n) { void* p = __real_malloc(n); if (!p && n) _exit(0); return p; }\n\
void* __wrap_calloc(size_t a, size_t b) { void* p = __real_calloc(a, b); if (!p && a && b) _exit(0); return p; }\n\
void* __wrap_realloc(void* q, size_t n) { void* p = __real_realloc(q, n); if (!p && n) _exit(0); return p; }\n";

/// Generate the C source of a byte-stream nondet shim for the **`valid-memsafety`
/// ASan confirmer** (lever `mem-fuzz-covguided`).
///
/// This is the byte-stream analogue of `synthesize_asan_driver` in `saf-cli`: the
/// uniform / positional-split ASan passes give every scalar nondet call the SAME
/// value (or a single leading/trailing split), which cannot reach a memory violation
/// gated behind two or more *independently-valued* nondet inputs (e.g. `n = nondet()`
/// picks an array size, then a LATER `idx = nondet()` must be a different in-range-then-
/// OOB value). Here each `__VERIFIER_nondet_T()` instead consumes `sizeof(T)`
/// little-endian bytes from the AFL-mutated input buffer, so a single run can supply a
/// distinct value to every call — exactly what a coverage-guided greybox search over
/// the SAME ASan harness needs.
///
/// # Why this is sound (ASan is the sole arbiter — a wrong FALSE is impossible)
///
/// Unlike [`synthesize_bytestream_driver`], this shim installs **no** `reach_error` /
/// `__VERIFIER_error` / `__assert_fail` sentinel sinks: it never itself decides a
/// verdict. AddressSanitizer intercepts `malloc`/`free`/loads/stores, and a genuine
/// memory-safety fault is intrinsic to the ORIGINAL program under whatever concrete
/// nondet values the byte stream supplies. The `saf-cli` driver captures ASan's stderr
/// and hands it to the UNMODIFIED [`crate::parse_asan_report`] R1/R2 gate; only a
/// high-fidelity, program-frame ASan mem-error yields `false(<sub-property>)`. The
/// fuzzer can therefore only ever *propose* a concrete input — it cannot manufacture a
/// wrong FALSE, and a safe program faults for none of them.
///
/// Contract compliance (`scripts/loop/confirmer_contract.md`):
/// - **R1** — confirmation is ONLY an ASan mem-error in the program's own code (the
///   parse gate rejects harness/interceptor frames); no catch-all trap confirms.
/// - **R4** — `__VERIFIER_assume(c)` hard-`_exit`s the run when `c` is false, so an
///   assume-pruned path can never fault-confirm.
/// - **R5** — each scalar nondet returns `sizeof(T)` reinterpreted bytes, an in-range
///   value of `T` (two's-complement, no trap reps); `_Bool` is normalised to `0`/`1`.
/// - **R6** — the byte-stream runs the ORIGINAL (unsliced) program, so the confirming
///   run *is* the reproduction; `saf-cli` additionally re-runs the same input to require
///   the identical ASan hit before emitting.
///
/// `rand`/`srand` are linker-`--wrap`ped (see the `-Wl,--wrap=rand/srand` compile
/// flags): `__wrap_rand` draws from the SAME byte stream (so Juliet `*_rand_*` variants
/// are driven deterministically, not by a wall-clock seed), and `__wrap_srand` is a
/// no-op. Pointer/float/double nondet return `0`/`NULL`, matching the uniform driver.
///
/// The runtime log (`$SAF_FUZZ_LOG`, `"<name> <value>\n"` per consumed nondet call)
/// is a lightweight, instrumentation-free execution-depth proxy: an input that drives
/// the program to consume MORE nondet inputs reached deeper into the (multi-nondet)
/// state machine, so the fuzz loop keeps it as a corpus seed — the greybox feedback
/// that steers the search toward the deep violation states without needing a linked
/// sanitizer-coverage runtime (which would clash with ASan's).
#[must_use]
pub fn synthesize_bytestream_asan_shim() -> String {
    use std::fmt::Write as _;

    let mut s = String::new();
    s.push_str("/* SAF byte-stream ASan mini-fuzz shim (generated) */\n");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("#include <string.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");
    let _ = writeln!(s, "static unsigned char __saf_buf[{INPUT_LEN}];");
    s.push_str("static size_t __saf_len = 0;\n");
    s.push_str("static size_t __saf_pos = 0;\n");
    s.push_str("static FILE* __saf_log = 0;\n");
    s.push_str("static int __saf_ready = 0;\n");
    s.push_str(
        "static void __saf_init(void) {\n\
         \x20 if (__saf_ready) return;\n\
         \x20 __saf_ready = 1;\n\
         \x20 const char* ip = getenv(\"SAF_FUZZ_INPUT\");\n\
         \x20 if (ip) { FILE* f = fopen(ip, \"rb\"); if (f) { __saf_len = fread(__saf_buf, 1, sizeof(__saf_buf), f); fclose(f); } }\n\
         \x20 const char* lp = getenv(\"SAF_FUZZ_LOG\");\n\
         \x20 if (lp) __saf_log = fopen(lp, \"w\");\n\
         }\n",
    );
    s.push_str(
        "static unsigned long long __saf_take(size_t n) {\n\
         \x20 __saf_init();\n\
         \x20 unsigned long long v = 0; size_t i;\n\
         \x20 for (i = 0; i < n; i++) { unsigned char b = (__saf_pos < __saf_len) ? __saf_buf[__saf_pos] : 0; __saf_pos++; v |= ((unsigned long long)b) << (8 * i); }\n\
         \x20 return v;\n\
         }\n",
    );
    s.push_str(
        "static void __saf_note(const char* name, long long val) {\n\
         \x20 if (__saf_log) { fprintf(__saf_log, \"%s %lld\\n\", name, val); fflush(__saf_log); }\n\
         }\n",
    );

    // Standard scalar-integer family (strong) plus the fixed-width typedef family
    // (weak). Driving the extended typedefs lets a memsafety harness that only
    // *declares* e.g. `u32 __VERIFIER_nondet_u32(void)` link and be fuzzed.
    push_bytestream_nondet_defs(&mut s, SCALAR_NONDET, false);
    push_bytestream_nondet_defs(&mut s, EXTENDED_NONDET, true);

    // SanitizerCoverage feedback (INERT unless the harness is compiled with
    // `-fsanitize-coverage=...`), identical to the unreach byte-stream driver. It
    // dumps the 8-bit edge map (`$SAF_FUZZ_COV`), the harvested comparison operands
    // (`$SAF_FUZZ_CMPLOG`) and the Redqueen input-to-state pairs (`$SAF_FUZZ_I2S`) on
    // normal exit. This ONLY steers the search — ASan is still the sole FALSE arbiter
    // (R1), so coverage feedback can never manufacture a wrong FALSE. Coexists with
    // `-fsanitize=address`: the `__SAF_NOCOV` callbacks are our own symbols (ASan's
    // runtime does not define them), so there is no duplicate-symbol clash.
    s.push_str(COVERAGE_FEEDBACK_C);

    // Non-integer / pointer nondet: the same legal defaults the uniform ASan driver uses.
    s.push_str("void* __VERIFIER_nondet_pointer(void) { return (void*)0; }\n");
    s.push_str("float __VERIFIER_nondet_float(void) { return 0.0f; }\n");
    s.push_str("double __VERIFIER_nondet_double(void) { return 0.0; }\n");
    s.push_str("void __VERIFIER_atomic_begin(void) { }\n");
    s.push_str("void __VERIFIER_atomic_end(void) { }\n");

    // R4: honour assumptions at runtime (hard reject).
    s.push_str("void __VERIFIER_assume(int c) { if (!c) _exit(0); }\n");

    // Determinism for Juliet `*_rand_*` variants: `rand()` draws from the byte stream
    // (masked non-negative, as libc's `rand` returns `[0, RAND_MAX]`), `srand()` is a
    // no-op. Requires `-Wl,--wrap=rand -Wl,--wrap=srand` at link (as the uniform driver).
    s.push_str(
        "int __wrap_rand(void) { unsigned long long r = __saf_take(sizeof(int)); return (int)(r & 0x7fffffffULL); }\n",
    );
    s.push_str("void __wrap_srand(unsigned s) { (void)s; }\n");

    // NOTE: deliberately NO reach_error/__VERIFIER_error/__assert_fail sentinel sinks —
    // ASan is the sole FALSE arbiter for valid-memsafety (an assertion failure is NOT a
    // memory-safety violation). The stub / program's own definitions link normally.

    s
}

/// Parse the shim's runtime log (`"<name> <value>\n"` per consumed nondet call)
/// into a [`NondetCall`] sequence in execution order. Malformed lines are skipped.
/// The result feeds a [`crate::FalseCandidate`] whose deterministic sequence-replay
/// re-confirms the violation (R6).
#[must_use]
pub fn parse_fuzz_log(log: &str) -> Vec<NondetCall> {
    let mut seq = Vec::new();
    for line in log.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // "<name> <value>" — name has no spaces; value is a base-10 i64.
        let Some((name, val)) = line.rsplit_once(' ') else {
            continue;
        };
        if !name.starts_with("__VERIFIER_nondet_") {
            continue;
        }
        let Ok(value) = val.trim().parse::<i64>() else {
            continue;
        };
        seq.push(NondetCall {
            func_name: name.to_string(),
            value,
            call_inst: None,
        });
    }
    seq
}

// ---------------------------------------------------------------------------
// Nondet-sequence -> byte-stream input conversion.
// ---------------------------------------------------------------------------

/// Byte width the byte-stream shim consumes for a scalar-integer
/// `__VERIFIER_nondet_T()` call — i.e. `sizeof(T)` under the task's data model
/// (long / unsigned long / size_t are pointer-width). `0` for names the shim does
/// not drive from the byte stream (pointer/float/double return a fixed default and
/// consume nothing, so they never appear in the fuzz log).
#[must_use]
pub fn nondet_width(name: &str, dm: DataModel) -> usize {
    let long_bytes = match dm {
        DataModel::ILP32 => 4,
        DataModel::LP64 => 8,
    };
    match name {
        // 1-byte scalars, plus the lazy-init nondet pointer's NULL/object selector
        // byte (see FUZZ_NONDET_POINTER_C) so a sequence-derived seed stays aligned.
        "__VERIFIER_nondet_char"
        | "__VERIFIER_nondet_uchar"
        | "__VERIFIER_nondet_bool"
        | "__VERIFIER_nondet_u8"
        | "__VERIFIER_nondet_s8"
        | "__VERIFIER_nondet_pointer" => 1,
        "__VERIFIER_nondet_short"
        | "__VERIFIER_nondet_ushort"
        | "__VERIFIER_nondet_u16"
        | "__VERIFIER_nondet_s16" => 2,
        "__VERIFIER_nondet_int"
        | "__VERIFIER_nondet_uint"
        | "__VERIFIER_nondet_u32"
        | "__VERIFIER_nondet_s32"
        | "__VERIFIER_nondet_unsigned"
        // Float/double consume sizeof(T) bytes; the logged NondetCall value is the
        // IEEE-754 bit pattern, so laying its low `width` little-endian bytes
        // reproduces the exact float bytes the shim consumed.
        | "__VERIFIER_nondet_float" => 4,
        "__VERIFIER_nondet_long" | "__VERIFIER_nondet_ulong" | "__VERIFIER_nondet_size_t" => {
            long_bytes
        }
        "__VERIFIER_nondet_longlong"
        | "__VERIFIER_nondet_ulonglong"
        | "__VERIFIER_nondet_u64"
        | "__VERIFIER_nondet_s64"
        | "__VERIFIER_nondet_loff_t"
        | "__VERIFIER_nondet_double" => 8,
        _ => 0,
    }
}

/// Lay a concrete nondet-value sequence into a byte-stream fuzz input the shim
/// reproduces verbatim: each call's value is written as its `sizeof(T)` low
/// little-endian bytes at the running offset (exactly how the shim's `__saf_take`
/// consumes them). The buffer is [`INPUT_LEN`] bytes, zero-padded; a value whose
/// name the shim does not drive (`width == 0`) or that would overrun the buffer is
/// skipped. Deterministic. Used to feed a concolically-solved flip sequence back into
/// the greybox fuzz loop as a seed — it steers the search only; native replay of the
/// original program remains the sole FALSE arbiter.
#[must_use]
pub fn nondet_seq_to_input(seq: &[NondetCall], dm: DataModel) -> Vec<u8> {
    let mut buf = vec![0u8; INPUT_LEN];
    let mut off = 0usize;
    for call in seq {
        let w = nondet_width(&call.func_name, dm);
        if w == 0 {
            continue;
        }
        if off + w > buf.len() {
            break;
        }
        #[allow(clippy::cast_sign_loss)]
        let bits = call.value as u64;
        for i in 0..w {
            #[allow(clippy::cast_possible_truncation)]
            let b = (bits >> (8 * i)) as u8;
            buf[off + i] = b;
        }
        off += w;
    }
    buf
}

// ---------------------------------------------------------------------------
// Deterministic AFL-style mutation engine.
// ---------------------------------------------------------------------------

/// A tiny deterministic PRNG (xorshift64). Fixed-seeded by the caller so the whole
/// fuzz search — and therefore the verdict — is reproducible (SAF determinism
/// invariant). Never uses wall-clock/`rand`.
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    /// Create a PRNG from `seed` (forced non-zero — xorshift fixed-points at 0).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed | 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// A value in `0..n` (`0` when `n == 0`).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            usize::try_from(self.next_u64() % n as u64).unwrap_or(0)
        }
    }

    /// A value in `0..n` biased toward the HIGH end (the max of two uniform draws —
    /// a triangular distribution). Used to pick a mutation base that favours the
    /// most-recently-added corpus entries, which are the coverage frontier
    /// (AFLFast-style energy: spend more on the newest, deepest inputs).
    pub fn below_biased_high(&mut self, n: usize) -> usize {
        let a = self.below(n);
        let b = self.below(n);
        a.max(b)
    }
}

/// Number of recency-biased candidates sampled per exploitation tournament in
/// [`select_base`]. Small so exploitation stays cheap and never fully abandons the
/// coverage frontier (a larger pool would collapse onto a single deep seed).
const SELECT_TOURNAMENT: usize = 3;

/// AFLGo-style annealed *directed power schedule*: choose which corpus entry to mutate
/// next, given a per-entry "closeness to the target" score and a simulated-annealing
/// temperature.
///
/// `progress[i]` is a source-level distance proxy for corpus entry `i` — how many
/// backward-slice guard constants that entry's run satisfied ([`cmplog_progress`]);
/// higher = deeper toward `reach_error`. `temp` in `[0.0, 1.0]` is the annealing
/// temperature, cooled from `1.0` (pure exploration) at the campaign start to `0.0`
/// (pure exploitation) at the end.
///
/// With probability `temp` the pick is the unchanged recency/frontier bias
/// ([`XorShift64::below_biased_high`] — favour the newest corpus entries, i.e. the
/// coverage frontier). With probability `1 - temp` it runs a small tournament and
/// returns the highest-`progress` entry (ties broken toward the most recent), so as
/// the campaign cools, mutation energy concentrates on the seeds closest to the
/// target — the AFLGo directed-scheduling idea adapted to a static-slice distance.
///
/// Purely a search heuristic — native replay (R6) stays the sole FALSE arbiter — and
/// deterministic in `rng` (integer-quantised annealing draw, no floats compared).
#[must_use]
pub fn select_base(rng: &mut XorShift64, progress: &[usize], temp: f64) -> usize {
    let n = progress.len();
    if n == 0 {
        return 0;
    }
    // Recency-biased explorer pick (the unchanged baseline behaviour).
    let explore = rng.below_biased_high(n);
    // Integer-quantised annealing draw (1024 buckets) — no float comparison, so the
    // schedule is byte-identical across platforms.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let temp_q = (temp.clamp(0.0, 1.0) * 1024.0) as u64;
    if (rng.next_u64() & 1023) < temp_q {
        return explore;
    }
    // Exploitation: tournament of a few recency-biased candidates; keep the one that
    // progressed furthest toward the target (tie -> most recent / highest index).
    let mut best = explore;
    for _ in 0..SELECT_TOURNAMENT {
        let c = rng.below_biased_high(n);
        if progress[c] > progress[best] || (progress[c] == progress[best] && c > best) {
            best = c;
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Edge-coverage feedback (SanitizerCoverage inline-8bit-counters).
// ---------------------------------------------------------------------------

/// AFL-style hit-count bucketing: fold a raw 8-bit edge counter into a single
/// one-hot *bucket bit* so near-identical hit counts (e.g. 8 vs 9) collapse to one
/// coverage class while 1 vs 2 vs 3 stay distinct. `0` hits map to `0` (no bit).
#[must_use]
pub fn classify_counter(c: u8) -> u8 {
    match c {
        0 => 0,
        1 => 1,
        2 => 1 << 1,
        3 => 1 << 2,
        4..=7 => 1 << 3,
        8..=15 => 1 << 4,
        16..=31 => 1 << 5,
        32..=127 => 1 << 6,
        _ => 1 << 7,
    }
}

/// Accumulated edge-coverage map: the running OR of every bucket bit seen across all
/// fuzz runs so far. A run reveals *new coverage* iff it sets a bucket bit not
/// previously accumulated — the classic greybox "keep the input that reached
/// somewhere new" signal, far stronger than the depth (log-line-count) heuristic it
/// replaces. Purely a search heuristic: it never influences the verdict.
#[derive(Debug, Default, Clone)]
pub struct CoverageMap {
    seen: Vec<u8>,
}

impl CoverageMap {
    /// A fresh, empty map.
    #[must_use]
    pub fn new() -> Self {
        Self { seen: Vec::new() }
    }

    /// Fold one raw counter snapshot (`raw[i]` = hit count of edge `i`) into the
    /// accumulated map. Returns `true` iff it contributed at least one previously
    /// unseen bucket bit. Grows the map to `raw.len()` on first sight of a larger
    /// snapshot (module load order is stable, so this is deterministic).
    pub fn fold(&mut self, raw: &[u8]) -> bool {
        if raw.len() > self.seen.len() {
            self.seen.resize(raw.len(), 0);
        }
        let mut novel = false;
        for (i, &c) in raw.iter().enumerate() {
            let bucket = classify_counter(c);
            let acc = &mut self.seen[i];
            if bucket & !*acc != 0 {
                *acc |= bucket;
                novel = true;
            }
        }
        novel
    }

    /// True iff no coverage has been accumulated yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seen.iter().all(|&b| b == 0)
    }
}

/// Merge a CmpLog dump — one unsigned-decimal comparison operand per line, as the
/// instrumented harness wrote them — into the live mutation `dict`. Each value's
/// 64-bit pattern is reinterpreted as `i64` (so a large unsigned magic constant maps
/// to the same little-endian window bytes the guard compares against). The dictionary
/// is re-deduplicated, kept in deterministic sorted order, and capped at `cap`.
/// Returns the number of NEW distinct values added.
///
/// This is the dynamic-dictionary half of CmpLog: comparison operands that never
/// appear as IR literals (computed magic values, multi-byte tokens of a state
/// machine) become steering constants for subsequent mutations, cracking guards the
/// static harvest cannot.
pub fn merge_cmplog(dict: &mut Vec<i64>, cmplog: &str, cap: usize) -> usize {
    let mut set: BTreeSet<i64> = dict.iter().copied().collect();
    let before = set.len();
    for line in cmplog.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        #[allow(clippy::cast_possible_wrap)]
        if let Ok(u) = t.parse::<u64>() {
            set.insert(u as i64);
        } else if let Ok(v) = t.parse::<i64>() {
            set.insert(v);
        }
    }
    let added = set.len().saturating_sub(before);
    let mut v: Vec<i64> = set.into_iter().collect();
    if v.len() > cap {
        v.truncate(cap);
    }
    *dict = v;
    added
}

/// Count how many DISTINCT `targets` constants (the backward slice's on-path guard
/// constants) appeared as a comparison operand in this run's CmpLog `dump`.
///
/// A cheap source-level *distance-to-target* proxy for the AFLGo directed power
/// schedule ([`select_base`]): a run that compared its values against more of the
/// guard constants lying on the path to `reach_error` has progressed further toward
/// it. Operands are parsed exactly as [`merge_cmplog`] does (unsigned decimal,
/// reinterpreted as `i64`, with a signed-decimal fallback). Purely a search signal —
/// never the verdict — and deterministic (a pure function of its inputs).
#[must_use]
pub fn cmplog_progress(dump: &str, targets: &BTreeSet<i64>) -> usize {
    if targets.is_empty() {
        return 0;
    }
    let mut hit: BTreeSet<i64> = BTreeSet::new();
    for line in dump.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        #[allow(clippy::cast_possible_wrap)]
        let v = if let Ok(u) = t.parse::<u64>() {
            u as i64
        } else if let Ok(v) = t.parse::<i64>() {
            v
        } else {
            continue;
        };
        if targets.contains(&v) {
            hit.insert(v);
        }
    }
    hit.len()
}

/// Produce a mutated copy of `input` (length preserved) by applying a random stack
/// of AFL-style havoc operations, drawing steering values from `dict`. Purely a
/// function of `(rng state, input, dict)` — deterministic.
///
/// The operations: single-bit flip, random/interesting byte set, small arithmetic
/// delta, and — the load-bearing one for guarded reaches — overwriting an aligned
/// window with a little-endian dictionary value (an IR constant a guard compares
/// against). Length is fixed at [`INPUT_LEN`] so mutation is allocation-free and
/// the byte-stream offsets stay stable.
#[must_use]
pub fn mutate(rng: &mut XorShift64, input: &[u8], dict: &[i64]) -> Vec<u8> {
    let mut out = input.to_vec();
    if out.is_empty() {
        return out;
    }
    let rounds = 1 + rng.below(16);
    for _ in 0..rounds {
        match rng.below(6) {
            0 => {
                // Flip one bit.
                let pos = rng.below(out.len());
                let bit = rng.below(8);
                out[pos] ^= 1u8 << bit;
            }
            1 => {
                // Set a byte to an interesting 8-bit value.
                let pos = rng.below(out.len());
                let iv = INTERESTING_VALUES[rng.below(INTERESTING_VALUES.len())];
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let b = iv as u8;
                out[pos] = b;
            }
            2 => {
                // Add a small signed delta to a byte (AFL arith).
                let pos = rng.below(out.len());
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_possible_wrap,
                    clippy::cast_sign_loss
                )]
                let delta = (rng.below(71) as i32 - 35) as u8;
                out[pos] = out[pos].wrapping_add(delta);
            }
            3 => {
                // Set a byte to a fully random value.
                let pos = rng.below(out.len());
                #[allow(clippy::cast_possible_truncation)]
                let b = rng.next_u64() as u8;
                out[pos] = b;
            }
            _ => {
                // Overwrite an aligned little-endian window with a dictionary value.
                if dict.is_empty() {
                    continue;
                }
                let val = dict[rng.below(dict.len())];
                // Width in {1,2,4,8} — matches the scalar nondet widths.
                let width = 1usize << rng.below(4);
                if width > out.len() {
                    continue;
                }
                let pos = rng.below(out.len() - width + 1);
                #[allow(clippy::cast_sign_loss)]
                let bits = val as u64;
                for i in 0..width {
                    #[allow(clippy::cast_possible_truncation)]
                    let b = (bits >> (8 * i)) as u8;
                    out[pos + i] = b;
                }
            }
        }
    }
    out
}

/// Build the deterministic seed corpus: an all-zero buffer (the common
/// unconditional / small-value path) plus one buffer per dictionary value tiled
/// across the whole length (so a single-guard reach `if (nondet()==K)` is hit
/// immediately for every harvested `K`). All of length [`INPUT_LEN`].
#[must_use]
pub fn seed_corpus(dict: &[i64]) -> Vec<Vec<u8>> {
    let mut corpus = vec![vec![0u8; INPUT_LEN]];
    for &val in dict {
        let mut buf = vec![0u8; INPUT_LEN];
        #[allow(clippy::cast_sign_loss)]
        let bits = val as u64;
        for (i, slot) in buf.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let b = (bits >> (8 * (i % 8))) as u8;
            *slot = b;
        }
        corpus.push(buf);
    }
    corpus
}

/// Cap on the number of distinct float values tiled into the seed corpus (each
/// yields one `f32`-width and one `f64`-width seed). Keeps the verbatim-seed phase
/// bounded within the fuzz budget.
const MAX_FLOAT_SEEDS: usize = 48;

/// Build the FLOAT seed corpus: for each interesting / slice-harvested float value,
/// two buffers that tile the value's IEEE-754 bytes across the whole input — one at
/// `f32` (4-byte) stride, one at `f64` (8-byte) stride — so a program reading a
/// `__VERIFIER_nondet_float()` or `_double()` at any aligned offset observes that
/// value immediately, without the blind fuzzer having to stumble onto an in-range
/// float by mutating raw bytes (which it essentially never does).
///
/// `extra` is the backward slice's `guard_floats` (the program's own float boundary
/// constants); each is added verbatim AND nudged toward zero (`* 0.9`) so a STRICT
/// boundary guard such as `x > -0.8` — which a tiled `-0.8` would fail — is still
/// covered from the inside. Deterministic; deduplicated on `(width, bit-pattern)`.
/// These only steer the search; native replay of the ORIGINAL program stays the sole
/// FALSE arbiter (R6), so an ill-fitting seed simply fails to reach and is discarded.
#[must_use]
pub fn float_seed_corpus(extra: &[f64]) -> Vec<Vec<u8>> {
    // Ordered value list: interesting values first, then slice guards (+ inward
    // nudges), deduplicated on the f64 bit pattern.
    let mut values: Vec<f64> = Vec::new();
    let mut seen_val: BTreeSet<u64> = BTreeSet::new();
    let push_val = |v: f64, values: &mut Vec<f64>, seen: &mut BTreeSet<u64>| {
        if v.is_finite() && seen.insert(v.to_bits()) && values.len() < MAX_FLOAT_SEEDS {
            values.push(v);
        }
    };
    for &v in INTERESTING_FLOATS {
        push_val(v, &mut values, &mut seen_val);
    }
    for &g in extra {
        push_val(g, &mut values, &mut seen_val);
        push_val(g * 0.9, &mut values, &mut seen_val);
    }

    let mut corpus: Vec<Vec<u8>> = Vec::new();
    let mut seen_buf: BTreeSet<Vec<u8>> = BTreeSet::new();
    let tile = |bytes: &[u8], corpus: &mut Vec<Vec<u8>>, seen: &mut BTreeSet<Vec<u8>>| {
        let w = bytes.len();
        let mut buf = vec![0u8; INPUT_LEN];
        for (i, slot) in buf.iter_mut().enumerate() {
            *slot = bytes[i % w];
        }
        if seen.insert(buf.clone()) {
            corpus.push(buf);
        }
    };
    for &v in &values {
        #[allow(clippy::cast_possible_truncation)]
        let as_f32 = v as f32;
        tile(&as_f32.to_le_bytes(), &mut corpus, &mut seen_buf);
        tile(&v.to_le_bytes(), &mut corpus, &mut seen_buf);
    }
    corpus
}

// ---------------------------------------------------------------------------
// Redqueen-style input-to-state (I2S) replacement.
// ---------------------------------------------------------------------------

/// A comparison operand pair observed by the instrumented harness: at some point the
/// program compared a `width`-byte value equal to `from`. If `from` appears verbatim
/// in the fuzz input, overwriting it with `to` is very likely to flip the guard —
/// the mechanism that cracks magic-value and multi-stage guards in a handful of execs
/// (Redqueen / `AFL++` CmpLog "input-to-state"), without blind brute force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2sPair {
    /// Operand byte width (1, 2, 4 or 8).
    pub width: u8,
    /// The value seen in the comparison (candidate to locate in the input).
    pub from: u64,
    /// The value to write in its place (the other operand — what the guard wants).
    pub to: u64,
}

/// Parse an I2S dump (`"<width> <from> <to>"` per line, unsigned decimals) into
/// pairs. Malformed lines and non-`{1,2,4,8}` widths are skipped; identity pairs are
/// dropped. Deterministic (input-order preserving).
#[must_use]
pub fn parse_i2s(dump: &str) -> Vec<I2sPair> {
    let mut out = Vec::new();
    for line in dump.lines() {
        let mut it = line.split_whitespace();
        let (Some(w), Some(a), Some(b)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let (Ok(width), Ok(from), Ok(to)) = (w.parse::<u8>(), a.parse::<u64>(), b.parse::<u64>())
        else {
            continue;
        };
        if !matches!(width, 1 | 2 | 4 | 8) || from == to {
            continue;
        }
        out.push(I2sPair { width, from, to });
    }
    out
}

/// Read a `width`-byte little-endian window of `buf` at `off` as a `u64`.
fn read_le(buf: &[u8], off: usize, width: usize) -> u64 {
    let mut v = 0u64;
    for i in 0..width {
        v |= u64::from(buf[off + i]) << (8 * i);
    }
    v
}

/// Generate input-to-state candidate inputs from `input` and the observed `pairs`:
/// for every pair, the little-endian windows of `input` whose bytes equal `from` are
/// rewritten to `to`, yielding one candidate per match site. To follow the
/// left-to-right byte-stream fill frontier — and to stop a high-multiplicity noise
/// pair (e.g. `0 -> 1`) from crowding out the load-bearing `0 -> magic` pair — at most
/// `per_pair` EARLIEST match sites are taken per pair; the total is capped at `max`.
/// Candidates are deduplicated, never equal the original, and preserve length.
/// Deterministic: pairs scanned in order, offsets ascending.
#[must_use]
pub fn i2s_candidates(
    input: &[u8],
    pairs: &[I2sPair],
    max: usize,
    per_pair: usize,
) -> Vec<Vec<u8>> {
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut out: Vec<Vec<u8>> = Vec::new();
    for p in pairs {
        let w = p.width as usize;
        if w == 0 || w > input.len() {
            continue;
        }
        let mask = if w == 8 {
            u64::MAX
        } else {
            (1u64 << (8 * w)) - 1
        };
        let from = p.from & mask;
        let to = p.to & mask;
        let mut taken = 0usize;
        for off in 0..=(input.len() - w) {
            if read_le(input, off, w) != from {
                continue;
            }
            let mut cand = input.to_vec();
            for i in 0..w {
                #[allow(clippy::cast_possible_truncation)]
                let b = (to >> (8 * i)) as u8;
                cand[off + i] = b;
            }
            if cand.as_slice() != input && seen.insert(cand.clone()) {
                out.push(cand);
                if out.len() >= max {
                    return out;
                }
                taken += 1;
                if taken >= per_pair {
                    break;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::AirModule;
    use saf_core::ids::{ModuleId, ValueId};

    #[test]
    fn driver_defines_the_scalar_nondet_family_and_sinks() {
        let src = synthesize_bytestream_driver("/tmp/s.sentinel");
        for (fname, _) in SCALAR_NONDET {
            assert!(src.contains(fname), "missing {fname}");
        }
        // R1 sinks + R4 assume present.
        assert!(src.contains("reach_error"));
        assert!(src.contains("__VERIFIER_error"));
        assert!(src.contains("__assert_fail"));
        assert!(src.contains("__VERIFIER_assume"));
        // The sentinel literal is embedded.
        assert!(src.contains("/tmp/s.sentinel"));
        // Bool is normalised, not memcpy'd.
        assert!(src.contains("(_Bool)(r & 1ULL)"));
    }

    #[test]
    fn asan_shim_defines_nondet_family_no_sentinel_and_rand_wraps() {
        let src = synthesize_bytestream_asan_shim();
        // Byte-stream scalar nondet family present.
        for (fname, _) in SCALAR_NONDET {
            assert!(src.contains(fname), "missing {fname}");
        }
        // R4 assume hard-reject present.
        assert!(src.contains("__VERIFIER_assume"));
        assert!(src.contains("_exit(0)"));
        // Bool is normalised, not memcpy'd.
        assert!(src.contains("(_Bool)(r & 1ULL)"));
        // Deterministic rand: byte-stream draw + no-op srand (linker --wrap).
        assert!(src.contains("__wrap_rand"));
        assert!(src.contains("__wrap_srand"));
        // CRITICAL soundness: ASan is the sole arbiter — NO error/assert sentinel sinks
        // (an assertion failure is not a memory-safety violation).
        assert!(
            !src.contains("reach_error"),
            "must not define a reach_error sink"
        );
        assert!(
            !src.contains("__VERIFIER_error"),
            "must not define a __VERIFIER_error sink"
        );
        assert!(
            !src.contains("__assert_fail"),
            "must not override __assert_fail"
        );
        // Reads the AFL-mutated input buffer.
        assert!(src.contains("SAF_FUZZ_INPUT"));
    }

    #[test]
    fn driver_drives_extended_typedef_family_as_weak_defs() {
        let src = synthesize_bytestream_driver("/tmp/s.sentinel");
        // Every fixed-width typedef generator is defined and driven from the stream.
        for (fname, _) in EXTENDED_NONDET {
            assert!(src.contains(fname), "missing extended nondet {fname}");
            // Driven, not a constant stub: it consumes bytes and logs the value.
            let driven = format!("{fname}(void) {{ ");
            assert!(src.contains(&driven), "not defined as a function: {fname}");
        }
        // The typedef defs are WEAK so a task-supplied body wins the link.
        assert!(
            src.contains("__attribute__((weak)) unsigned int __VERIFIER_nondet_u32(void)"),
            "u32 must be a weak driven def"
        );
        // The standard scalar family stays STRONG (unchanged link behaviour).
        assert!(
            src.contains("int __VERIFIER_nondet_int(void)")
                && !src.contains("__attribute__((weak)) int __VERIFIER_nondet_int(void)"),
            "scalar int must remain a strong def"
        );
    }

    #[test]
    fn asan_shim_drives_extended_typedef_family() {
        let src = synthesize_bytestream_asan_shim();
        for (fname, _) in EXTENDED_NONDET {
            assert!(
                src.contains(fname),
                "asan shim missing extended nondet {fname}"
            );
        }
        assert!(src.contains("__attribute__((weak)) unsigned int __VERIFIER_nondet_u32(void)"));
    }

    #[test]
    fn nondet_pointer_is_a_lazy_init_selector_object() {
        // The unreach fuzz driver's nondet pointer logs a selector and returns a fresh
        // zero-filled object on the odd branch — no longer a hard NULL.
        let src = synthesize_bytestream_driver("/tmp/s.sentinel");
        assert!(
            src.contains(FUZZ_NONDET_POINTER_C.trim_end()),
            "fuzz driver must lazy-init the nondet pointer: {src}"
        );
        assert!(
            !src.contains("__VERIFIER_nondet_pointer(void) { return (void*)0; }"),
            "nondet pointer must no longer be a hard NULL in the unreach driver"
        );
        // Both nondet outcomes are explorable (selector parity) and the object is
        // zero-filled (calloc) so unwritten fields read a deterministic 0.
        assert!(FUZZ_NONDET_POINTER_C.contains("__saf_take(1)"));
        assert!(FUZZ_NONDET_POINTER_C.contains("calloc(1, 4096)"));
        assert!(FUZZ_NONDET_POINTER_C.contains("(void*)0"));
        assert_eq!(NONDET_OBJ_SIZE, 4096);
        // The selector is logged so the replay driver reproduces the same choice (R6).
        assert!(FUZZ_NONDET_POINTER_C.contains("__VERIFIER_nondet_pointer"));
        // The selector byte is accounted for in seed conversion.
        assert_eq!(
            nondet_width("__VERIFIER_nondet_pointer", DataModel::LP64),
            1
        );
        assert_eq!(
            nondet_width("__VERIFIER_nondet_pointer", DataModel::ILP32),
            1
        );
    }

    #[test]
    fn extended_nondet_widths_are_model_independent() {
        // The fixed-width typedefs consume the same number of bytes under both models
        // (unlike long/size_t), so a discovered sequence re-confirms across ILP32/LP64.
        for dm in [DataModel::ILP32, DataModel::LP64] {
            assert_eq!(nondet_width("__VERIFIER_nondet_u8", dm), 1);
            assert_eq!(nondet_width("__VERIFIER_nondet_s8", dm), 1);
            assert_eq!(nondet_width("__VERIFIER_nondet_u16", dm), 2);
            assert_eq!(nondet_width("__VERIFIER_nondet_u32", dm), 4);
            assert_eq!(nondet_width("__VERIFIER_nondet_unsigned", dm), 4);
            assert_eq!(nondet_width("__VERIFIER_nondet_u64", dm), 8);
            assert_eq!(nondet_width("__VERIFIER_nondet_loff_t", dm), 8);
        }
    }

    #[test]
    fn extended_nondet_seq_round_trips_a_u32() {
        // A logged u32 sequence lays exactly 4 little-endian bytes per call, so the
        // byte-stream shim reproduces the same value the fuzz run recorded (R6).
        let seq = vec![
            NondetCall {
                func_name: "__VERIFIER_nondet_u32".to_string(),
                value: 0x0102_0304,
                call_inst: None,
            },
            NondetCall {
                func_name: "__VERIFIER_nondet_u8".to_string(),
                value: 0xAB,
                call_inst: None,
            },
        ];
        let bytes = nondet_seq_to_input(&seq, DataModel::LP64);
        assert_eq!(&bytes[0..4], &[0x04, 0x03, 0x02, 0x01], "u32 little-endian");
        assert_eq!(bytes[4], 0xAB, "u8 byte");
    }

    #[test]
    fn dictionary_includes_interesting_and_module_constants() {
        let mut m = AirModule::new(ModuleId::new(1));
        m.constants
            .insert(ValueId::new(1), Constant::int(424_242, 32));
        m.constants.insert(ValueId::new(2), Constant::int(7, 32));
        let dict = harvest_dictionary(&m);
        assert!(dict.contains(&424_242), "module constant harvested");
        assert!(dict.contains(&7));
        assert!(dict.contains(&2_147_483_647), "interesting value seeded");
        assert!(dict.contains(&0));
        // Deterministic ordering (sorted BTreeSet).
        let mut sorted = dict.clone();
        sorted.sort_unstable();
        assert_eq!(dict, sorted);
    }

    #[test]
    fn parse_log_round_trips_a_sequence() {
        let log = "__VERIFIER_nondet_int 42\n__VERIFIER_nondet_uint 7\n\
                   garbage line\n__VERIFIER_nondet_char -1\n";
        let seq = parse_fuzz_log(log);
        assert_eq!(seq.len(), 3);
        assert_eq!(seq[0].func_name, "__VERIFIER_nondet_int");
        assert_eq!(seq[0].value, 42);
        assert_eq!(seq[1].func_name, "__VERIFIER_nondet_uint");
        assert_eq!(seq[1].value, 7);
        assert_eq!(seq[2].value, -1);
    }

    #[test]
    fn parse_log_rejects_non_nondet_and_nonnumeric() {
        assert!(parse_fuzz_log("printf hello\n").is_empty());
        assert!(parse_fuzz_log("__VERIFIER_nondet_int notanumber\n").is_empty());
        assert!(parse_fuzz_log("").is_empty());
    }

    #[test]
    fn float_nondet_detection_and_gating() {
        use saf_core::air::AirFunction;
        use saf_core::ids::FunctionId;
        use std::collections::BTreeMap;
        let decl = |id: u128, name: &str| AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        assert!(is_float_nondet("__VERIFIER_nondet_float"));
        assert!(is_float_nondet("__VERIFIER_nondet_double"));
        assert!(!is_float_nondet("__VERIFIER_nondet_int"));

        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(decl(1, "__VERIFIER_nondet_double"));
        assert!(references_float_nondet(&m));
        assert!(!references_scalar_nondet(&m), "double is not a scalar-int");
        assert!(
            references_fuzzable_nondet(&m),
            "float/double is now fuzzable"
        );
    }

    #[test]
    fn float_seed_corpus_tiles_interesting_and_slice_floats() {
        // Interesting floats alone produce seeds even with no slice guards.
        let base = float_seed_corpus(&[]);
        assert!(!base.is_empty());
        assert!(base.iter().all(|b| b.len() == INPUT_LEN));

        // -0.75f tiled at f32 stride: the 4-byte pattern repeats from offset 0, so a
        // `__VERIFIER_nondet_float()` read at offset 0 reconstructs exactly -0.75.
        let want = (-0.75f32).to_le_bytes();
        assert!(
            base.iter().any(|b| b[0..4] == want && b[4..8] == want),
            "an f32 -0.75 tile must be present"
        );

        // A slice-harvested boundary (e.g. -0.8) is added AND nudged inward (*0.9).
        let with_guard = float_seed_corpus(&[-0.8]);
        let nudged = (-0.8f64 * 0.9) as f32;
        assert!(
            with_guard.iter().any(|b| b[0..4] == nudged.to_le_bytes()),
            "the inward-nudged guard float must be tiled"
        );
        // Deterministic.
        assert_eq!(float_seed_corpus(&[-0.8]), with_guard);
    }

    #[test]
    fn bytestream_driver_drives_float_and_double() {
        let src = synthesize_bytestream_driver("/tmp/x");
        assert!(
            src.contains("float __VERIFIER_nondet_float(void)") && src.contains("memcpy(&__f"),
            "float shim must consume bytes + memcpy"
        );
        assert!(
            src.contains("double __VERIFIER_nondet_double(void)") && src.contains("memcpy(&__d"),
            "double shim must consume bytes + memcpy"
        );
        // No longer the fixed-0 stub.
        assert!(!src.contains("return 0.0f; }"));
    }

    #[test]
    fn cmplog_progress_counts_distinct_on_path_targets() {
        let targets: BTreeSet<i64> = [500i64, 42, -7].into_iter().collect();
        // Two of the three targets appear (one twice -> deduped); 999 is off-path.
        let dump = "500\n999\n42\n500\n";
        assert_eq!(cmplog_progress(dump, &targets), 2);
        // A large unsigned magic value reinterpreted as i64 still matches a negative
        // target with the same little-endian bit pattern.
        let neg = (-7i64) as u64;
        let dump2 = format!("{neg}\n");
        assert_eq!(cmplog_progress(&dump2, &targets), 1);
        // Empty target set (empty slice) -> always 0, so the schedule degrades to the
        // recency bias with no regression.
        assert_eq!(cmplog_progress(dump, &BTreeSet::new()), 0);
        assert_eq!(cmplog_progress("", &targets), 0);
    }

    #[test]
    fn select_base_hot_is_pure_recency_bias() {
        // At temperature 1.0 the pick is exactly the recency-biased explorer draw.
        let progress = vec![0usize; 6];
        let mut r1 = XorShift64::new(99);
        let mut r2 = XorShift64::new(99);
        let want = r2.below_biased_high(progress.len());
        assert_eq!(select_base(&mut r1, &progress, 1.0), want);
    }

    #[test]
    fn select_base_cold_never_regresses_below_the_explorer() {
        // At temperature 0.0 exploitation runs a tournament seeded from the explorer
        // pick and only ever upgrades to a >= progress entry — it can never choose a
        // shallower seed than the explorer would have.
        let progress = vec![0usize, 1, 0, 9, 0, 2];
        for seed in 1..200u64 {
            let mut r1 = XorShift64::new(seed);
            let mut r2 = r1.clone();
            let explore = r2.below_biased_high(progress.len());
            let idx = select_base(&mut r1, &progress, 0.0);
            assert!(idx < progress.len(), "valid index");
            assert!(
                progress[idx] >= progress[explore],
                "exploitation must not regress below the explorer pick"
            );
        }
    }

    #[test]
    fn select_base_is_deterministic_and_handles_empty() {
        assert_eq!(select_base(&mut XorShift64::new(1), &[], 0.5), 0);
        let progress = vec![0usize, 3, 1, 4];
        let mut a = XorShift64::new(2024);
        let mut b = XorShift64::new(2024);
        for _ in 0..500 {
            assert_eq!(
                select_base(&mut a, &progress, 0.4),
                select_base(&mut b, &progress, 0.4),
                "same seed -> identical directed pick"
            );
        }
    }

    #[test]
    fn mutation_is_deterministic_for_a_fixed_seed() {
        let dict = vec![42i64, 2_147_483_647, -1];
        let seed_input = vec![0u8; INPUT_LEN];
        let mut a = XorShift64::new(1234);
        let mut b = XorShift64::new(1234);
        for _ in 0..1000 {
            let ma = mutate(&mut a, &seed_input, &dict);
            let mb = mutate(&mut b, &seed_input, &dict);
            assert_eq!(ma, mb, "same seed -> identical mutation stream");
            assert_eq!(ma.len(), INPUT_LEN, "length preserved");
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let dict = vec![42i64];
        let input = vec![0u8; INPUT_LEN];
        let mut a = XorShift64::new(1);
        let mut b = XorShift64::new(2);
        // Over many draws the two streams must differ at least once.
        let mut differed = false;
        for _ in 0..100 {
            if mutate(&mut a, &input, &dict) != mutate(&mut b, &input, &dict) {
                differed = true;
                break;
            }
        }
        assert!(differed);
    }

    #[test]
    fn seed_corpus_tiles_each_dict_value_and_a_zero_buffer() {
        let dict = vec![42i64, 255];
        let corpus = seed_corpus(&dict);
        assert_eq!(corpus.len(), 1 + dict.len());
        assert!(corpus.iter().all(|b| b.len() == INPUT_LEN));
        // First buffer is all-zero.
        assert!(corpus[0].iter().all(|&b| b == 0));
        // The 42-tiled buffer reads back as 42 in its low 4 bytes (little-endian).
        let buf42 = &corpus[1];
        let v = u32::from_le_bytes([buf42[0], buf42[1], buf42[2], buf42[3]]);
        assert_eq!(v, 42);
    }

    #[test]
    fn references_scalar_nondet_detects_declared_nondet() {
        use saf_core::air::AirFunction;
        use saf_core::ids::FunctionId;
        use std::collections::BTreeMap;
        let mut m = AirModule::new(ModuleId::new(1));
        assert!(!references_scalar_nondet(&m));
        m.functions.push(AirFunction {
            id: FunctionId::new(9),
            name: "__VERIFIER_nondet_int".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        assert!(references_scalar_nondet(&m));
    }

    #[test]
    fn is_fuzzable_nondet_covers_scalar_extended_and_pointer() {
        // Standard scalar family.
        assert!(is_fuzzable_nondet("__VERIFIER_nondet_int"));
        // Fixed-width typedef family (the newly driven names).
        assert!(is_fuzzable_nondet("__VERIFIER_nondet_u32"));
        assert!(is_fuzzable_nondet("__VERIFIER_nondet_loff_t"));
        assert!(is_fuzzable_nondet("__VERIFIER_nondet_unsigned"));
        // Lazy-init nondet pointer (selector is fuzzed).
        assert!(is_fuzzable_nondet("__VERIFIER_nondet_pointer"));
        // Not a driven input.
        assert!(!is_fuzzable_nondet("__VERIFIER_nondet_float"));
        assert!(!is_fuzzable_nondet("printf"));
    }

    #[test]
    fn references_scalar_nondet_triggers_on_extended_only_module() {
        // A harness that declares ONLY a fixed-width typedef generator (no standard
        // scalar family) must still engage the fuzzer.
        use saf_core::air::AirFunction;
        use saf_core::ids::FunctionId;
        use std::collections::BTreeMap;
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(AirFunction {
            id: FunctionId::new(9),
            name: "__VERIFIER_nondet_u32".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        assert!(references_scalar_nondet(&m));
    }

    #[test]
    fn count_scalar_nondet_call_sites_counts_direct_calls() {
        use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
        use saf_core::ids::{BlockId, FunctionId, InstId};
        use std::collections::BTreeMap;

        // A `__VERIFIER_nondet_int` declaration + a `main` that calls it twice.
        let nondet = AirFunction {
            id: FunctionId::new(1),
            name: "__VERIFIER_nondet_int".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut block = AirBlock::new(BlockId::new(10));
        for i in 0..2u128 {
            block.instructions.push(Instruction {
                id: InstId::new(100 + i),
                op: Operation::CallDirect {
                    callee: FunctionId::new(1),
                },
                operands: vec![],
                dst: None,
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            });
        }
        let main = AirFunction {
            id: FunctionId::new(2),
            name: "main".to_string(),
            params: vec![],
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = AirModule::new(ModuleId::new(1));
        // No call sites yet (empty module).
        assert_eq!(count_scalar_nondet_call_sites(&m), 0);
        m.functions.push(nondet);
        m.functions.push(main);
        // Two direct calls to a scalar nondet -> gate (>=2) is met.
        assert_eq!(count_scalar_nondet_call_sites(&m), 2);
    }

    #[test]
    fn driver_defines_sancov_callbacks_and_dumper() {
        let src = synthesize_bytestream_driver("/tmp/s.sentinel");
        // All instrumentation-required callbacks are defined (else the coverage
        // build would fail to link).
        for cb in [
            "__sanitizer_cov_8bit_counters_init",
            "__sanitizer_cov_pcs_init",
            "__sanitizer_cov_trace_cmp1",
            "__sanitizer_cov_trace_cmp2",
            "__sanitizer_cov_trace_cmp4",
            "__sanitizer_cov_trace_cmp8",
            "__sanitizer_cov_trace_const_cmp1",
            "__sanitizer_cov_trace_const_cmp8",
            "__sanitizer_cov_trace_switch",
        ] {
            assert!(src.contains(cb), "missing sancov callback {cb}");
        }
        // The dump reads the two feedback channels.
        assert!(src.contains("SAF_FUZZ_COV"));
        assert!(src.contains("SAF_FUZZ_CMPLOG"));
        assert!(src.contains("atexit"));
    }

    #[test]
    fn classify_counter_buckets_hit_counts() {
        assert_eq!(classify_counter(0), 0);
        assert_eq!(classify_counter(1), 1);
        assert_eq!(classify_counter(2), 1 << 1);
        assert_eq!(classify_counter(3), 1 << 2);
        // Same bucket for 4..=7.
        assert_eq!(classify_counter(4), classify_counter(7));
        assert_ne!(classify_counter(3), classify_counter(4));
        assert_eq!(classify_counter(200), 1 << 7);
    }

    #[test]
    fn coverage_map_reports_only_new_edges() {
        let mut cov = CoverageMap::new();
        assert!(cov.is_empty());
        // First non-zero snapshot is entirely novel.
        assert!(cov.fold(&[0, 1, 0, 5]));
        assert!(!cov.is_empty());
        // Identical snapshot -> nothing new.
        assert!(!cov.fold(&[0, 1, 0, 5]));
        // Same edges but a DIFFERENT hit-count bucket on edge 1 -> novel.
        assert!(cov.fold(&[0, 2, 0, 5]));
        // A brand-new edge index (grows the map) -> novel.
        assert!(cov.fold(&[0, 0, 0, 0, 9]));
        // An all-zero snapshot never counts as new.
        assert!(!cov.fold(&[0, 0, 0, 0, 0]));
    }

    #[test]
    fn merge_cmplog_extends_dictionary_deterministically() {
        let mut dict = vec![0i64, 1, 42];
        let added = merge_cmplog(&mut dict, "3735928559\n42\n1000\n", 256);
        // 3735928559 == 0xDEADBEEF and 1000 are new; 42 already present.
        assert_eq!(added, 2);
        assert!(dict.contains(&3_735_928_559));
        assert!(dict.contains(&1000));
        // Sorted, deduped (deterministic).
        let mut sorted = dict.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(dict, sorted);
        // Idempotent: re-merging the same log adds nothing.
        assert_eq!(merge_cmplog(&mut dict, "42\n1000\n", 256), 0);
    }

    #[test]
    fn merge_cmplog_respects_cap() {
        let mut dict: Vec<i64> = Vec::new();
        let log: String = (0..1000).map(|n| format!("{n}\n")).collect();
        merge_cmplog(&mut dict, &log, 100);
        assert_eq!(dict.len(), 100);
    }

    #[test]
    fn driver_defines_i2s_channel() {
        let src = synthesize_bytestream_driver("/tmp/s.sentinel");
        assert!(src.contains("SAF_FUZZ_I2S"));
        assert!(src.contains("__saf_i2s_note"));
    }

    #[test]
    fn parse_i2s_skips_malformed_and_identity() {
        let pairs = parse_i2s("4 0 305419896\n2 7 7\n8 1 2\nbad line\n3 5 6\n");
        // width 4 (0->0x12345678) and width 8 (1->2) kept; identity (7->7) and
        // invalid width 3 dropped.
        assert_eq!(pairs.len(), 2);
        assert_eq!(
            pairs[0],
            I2sPair {
                width: 4,
                from: 0,
                to: 305_419_896
            }
        );
        assert_eq!(
            pairs[1],
            I2sPair {
                width: 8,
                from: 1,
                to: 2
            }
        );
    }

    #[test]
    fn i2s_patches_the_matching_window() {
        // Input has 0 in bytes 0..4; a (from=0, to=0x12345678, w=4) pair should
        // rewrite offset 0 (and every other all-zero 4-window) to the magic value.
        let input = vec![0u8; 8];
        let pairs = vec![I2sPair {
            width: 4,
            from: 0,
            to: 0x1234_5678,
        }];
        let cands = i2s_candidates(&input, &pairs, 16, 8);
        assert!(!cands.is_empty());
        // The EARLIEST match (offset 0) is produced first — the fill frontier.
        assert_eq!(
            u32::from_le_bytes([cands[0][0], cands[0][1], cands[0][2], cands[0][3]]),
            0x1234_5678
        );
        // Length preserved; none equals the original.
        assert!(cands.iter().all(|c| c.len() == input.len() && c != &input));
    }

    #[test]
    fn i2s_per_pair_cap_leaves_room_for_later_pairs() {
        // A high-multiplicity noise pair (0 -> 1) precedes the load-bearing pair
        // (0 -> magic). With per_pair small, the noise pair cannot exhaust the budget
        // before the magic pair is reached.
        let input = vec![0u8; 64];
        let pairs = vec![
            I2sPair {
                width: 4,
                from: 0,
                to: 1,
            },
            I2sPair {
                width: 4,
                from: 0,
                to: 0xCAFE_BABE,
            },
        ];
        let cands = i2s_candidates(&input, &pairs, 64, 2);
        assert!(
            cands
                .iter()
                .any(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]) == 0xCAFE_BABE),
            "magic pair must still get a candidate"
        );
    }

    #[test]
    fn i2s_finds_a_nonzero_value_and_respects_cap() {
        // 42 (LE, width 4) sits at offset 2.
        let mut input = vec![0u8; 12];
        input[2..6].copy_from_slice(&42u32.to_le_bytes());
        let pairs = vec![I2sPair {
            width: 4,
            from: 42,
            to: 99,
        }];
        let cands = i2s_candidates(&input, &pairs, 16, 8);
        // The offset-2 window becomes 99.
        assert!(
            cands
                .iter()
                .any(|c| { u32::from_le_bytes([c[2], c[3], c[4], c[5]]) == 99 })
        );
        // Cap is honoured.
        let capped = i2s_candidates(&vec![0u8; 64], &pairs, 3, 8);
        assert!(capped.len() <= 3);
    }

    #[test]
    fn biased_high_favours_the_frontier() {
        let mut rng = XorShift64::new(99);
        let n = 10;
        let mut high = 0usize;
        let mut plain = 0usize;
        for _ in 0..2000 {
            if rng.below_biased_high(n) >= n / 2 {
                high += 1;
            }
            if rng.below(n) >= n / 2 {
                plain += 1;
            }
        }
        // The biased draw lands in the top half markedly more often than uniform.
        assert!(high > plain, "biased={high} uniform={plain}");
    }

    #[test]
    fn nondet_width_tracks_data_model() {
        assert_eq!(nondet_width("__VERIFIER_nondet_int", DataModel::LP64), 4);
        assert_eq!(nondet_width("__VERIFIER_nondet_char", DataModel::LP64), 1);
        assert_eq!(nondet_width("__VERIFIER_nondet_short", DataModel::LP64), 2);
        // long / size_t are pointer-width: 8 under LP64, 4 under ILP32.
        assert_eq!(nondet_width("__VERIFIER_nondet_long", DataModel::LP64), 8);
        assert_eq!(nondet_width("__VERIFIER_nondet_long", DataModel::ILP32), 4);
        assert_eq!(
            nondet_width("__VERIFIER_nondet_size_t", DataModel::ILP32),
            4
        );
        assert_eq!(
            nondet_width("__VERIFIER_nondet_longlong", DataModel::ILP32),
            8
        );
        // Float/double are now byte-stream driven (sizeof(T)).
        assert_eq!(nondet_width("__VERIFIER_nondet_float", DataModel::LP64), 4);
        assert_eq!(nondet_width("__VERIFIER_nondet_double", DataModel::LP64), 8);
        // A name the shim does not drive consumes nothing.
        assert_eq!(nondet_width("some_unknown_fn", DataModel::LP64), 0);
    }

    #[test]
    fn nondet_seq_to_input_lays_le_bytes_the_shim_reads() {
        let seq = vec![
            NondetCall {
                func_name: "__VERIFIER_nondet_int".to_string(),
                value: 42,
                call_inst: None,
            },
            NondetCall {
                func_name: "__VERIFIER_nondet_short".to_string(),
                value: -1,
                call_inst: None,
            },
        ];
        let buf = nondet_seq_to_input(&seq, DataModel::LP64);
        assert_eq!(buf.len(), INPUT_LEN);
        // First int at offset 0..4 reads back as 42.
        assert_eq!(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]), 42);
        // The short at offset 4..6 reads back as 0xffff (-1 truncated to 16 bits).
        assert_eq!(u16::from_le_bytes([buf[4], buf[5]]), 0xffff);
        // Everything after is zero-padded.
        assert!(buf[6..].iter().all(|&b| b == 0));
    }

    #[test]
    fn nondet_seq_to_input_is_deterministic_and_skips_undriven() {
        let seq = vec![
            NondetCall {
                func_name: "some_unknown_fn".to_string(), // width 0 -> skipped
                value: 5,
                call_inst: None,
            },
            NondetCall {
                func_name: "__VERIFIER_nondet_int".to_string(),
                value: 7,
                call_inst: None,
            },
        ];
        let a = nondet_seq_to_input(&seq, DataModel::LP64);
        let b = nondet_seq_to_input(&seq, DataModel::LP64);
        assert_eq!(a, b);
        // The float consumed no bytes, so the int lands at offset 0.
        assert_eq!(u32::from_le_bytes([a[0], a[1], a[2], a[3]]), 7);
    }

    #[test]
    fn references_nondet_bool_detects_declared_bool() {
        use saf_core::air::AirFunction;
        use saf_core::ids::FunctionId;
        use std::collections::BTreeMap;
        let mut m = AirModule::new(ModuleId::new(1));
        assert!(!references_nondet_bool(&m));
        // A non-bool nondet must NOT trigger the bool pass.
        m.functions.push(AirFunction {
            id: FunctionId::new(7),
            name: "__VERIFIER_nondet_int".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        assert!(!references_nondet_bool(&m));
        m.functions.push(AirFunction {
            id: FunctionId::new(9),
            name: "__VERIFIER_nondet_bool".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        assert!(references_nondet_bool(&m));
    }

    // --- nondet_taints_int_to_ptr_deref (soundness sentinel #1) ---------------

    /// Build a module: a `__VERIFIER_nondet_ulong` declaration (id 1) + a `main`
    /// (id 2) whose single block contains `body`. Instruction ValueIds/InstIds are
    /// the caller's responsibility.
    fn module_with_body(body: Vec<saf_core::air::Instruction>) -> AirModule {
        use saf_core::air::{AirBlock, AirFunction};
        use saf_core::ids::{BlockId, FunctionId};
        use std::collections::BTreeMap;

        let nondet = AirFunction {
            id: FunctionId::new(1),
            name: "__VERIFIER_nondet_ulong".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut block = AirBlock::new(BlockId::new(10));
        block.instructions = body;
        let main = AirFunction {
            id: FunctionId::new(2),
            name: "main".to_string(),
            params: vec![],
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(nondet);
        m.functions.push(main);
        m
    }

    fn inst(
        id: u128,
        op: saf_core::air::Operation,
        operands: Vec<ValueId>,
        dst: Option<ValueId>,
    ) -> saf_core::air::Instruction {
        use saf_core::ids::InstId;
        use std::collections::BTreeMap;
        saf_core::air::Instruction {
            id: InstId::new(id),
            op,
            operands,
            dst,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn taint_flags_nondet_ulong_cast_to_ptr_then_loaded() {
        use saf_core::air::{CastKind, Operation};
        use saf_core::ids::FunctionId;
        // %v = call nondet_ulong(); %p = inttoptr %v; %x = load %p
        let v = ValueId::new(100);
        let p = ValueId::new(101);
        let x = ValueId::new(102);
        let body = vec![
            inst(
                1,
                Operation::CallDirect {
                    callee: FunctionId::new(1),
                },
                vec![],
                Some(v),
            ),
            inst(
                2,
                Operation::Cast {
                    kind: CastKind::IntToPtr,
                    target_bits: None,
                },
                vec![v],
                Some(p),
            ),
            inst(3, Operation::Load, vec![p], Some(x)),
        ];
        assert!(nondet_taints_int_to_ptr_deref(&module_with_body(body)));
    }

    #[test]
    fn taint_flags_through_intermediate_int_cast_and_gep() {
        use saf_core::air::{CastKind, FieldPath, Operation};
        use saf_core::ids::FunctionId;
        // %v = call nondet_ulong(); %w = zext %v; %p = inttoptr %w;
        // %q = gep %p; store _, %q
        let v = ValueId::new(100);
        let w = ValueId::new(101);
        let p = ValueId::new(102);
        let q = ValueId::new(103);
        let val = ValueId::new(104);
        let body = vec![
            inst(
                1,
                Operation::CallDirect {
                    callee: FunctionId::new(1),
                },
                vec![],
                Some(v),
            ),
            inst(
                2,
                Operation::Cast {
                    kind: CastKind::ZExt,
                    target_bits: Some(64),
                },
                vec![v],
                Some(w),
            ),
            inst(
                3,
                Operation::Cast {
                    kind: CastKind::IntToPtr,
                    target_bits: None,
                },
                vec![w],
                Some(p),
            ),
            inst(
                4,
                Operation::Gep {
                    field_path: FieldPath::default(),
                },
                vec![p],
                Some(q),
            ),
            // store value=val to pointer=q (operand[1] is the address)
            inst(5, Operation::Store, vec![val, q], None),
        ];
        assert!(nondet_taints_int_to_ptr_deref(&module_with_body(body)));
    }

    #[test]
    fn taint_does_not_flag_inttoptr_from_non_nondet_source() {
        use saf_core::air::{CastKind, Operation};
        // A plain `inttoptr` of a constant/param value (NOT nondet-derived) that is
        // dereferenced must NOT trip the gate — this is the precision requirement
        // (no blunt any-inttoptr abstain).
        let c = ValueId::new(200); // some non-nondet value (never defined by a nondet call)
        let p = ValueId::new(201);
        let x = ValueId::new(202);
        let body = vec![
            inst(
                1,
                Operation::Cast {
                    kind: CastKind::IntToPtr,
                    target_bits: None,
                },
                vec![c],
                Some(p),
            ),
            inst(2, Operation::Load, vec![p], Some(x)),
        ];
        assert!(!nondet_taints_int_to_ptr_deref(&module_with_body(body)));
    }

    #[test]
    fn taint_does_not_flag_nondet_ptr_that_is_never_dereferenced() {
        use saf_core::air::{CastKind, Operation};
        use saf_core::ids::FunctionId;
        // %v = call nondet_ulong(); %p = inttoptr %v; ret %p  (no load/store)
        let v = ValueId::new(100);
        let p = ValueId::new(101);
        let body = vec![
            inst(
                1,
                Operation::CallDirect {
                    callee: FunctionId::new(1),
                },
                vec![],
                Some(v),
            ),
            inst(
                2,
                Operation::Cast {
                    kind: CastKind::IntToPtr,
                    target_bits: None,
                },
                vec![v],
                Some(p),
            ),
            inst(3, Operation::Ret, vec![p], None),
        ];
        assert!(!nondet_taints_int_to_ptr_deref(&module_with_body(body)));
    }

    #[test]
    fn taint_does_not_flag_scalar_nondet_used_only_as_integer() {
        use saf_core::air::{BinaryOp, Operation};
        use saf_core::ids::FunctionId;
        // A scalar nondet used purely arithmetically (never cast to a pointer):
        // the common fuzzable guard — must stay confirmable (no abstain).
        let v = ValueId::new(100);
        let k = ValueId::new(101);
        let cmp = ValueId::new(102);
        let body = vec![
            inst(
                1,
                Operation::CallDirect {
                    callee: FunctionId::new(1),
                },
                vec![],
                Some(v),
            ),
            inst(
                2,
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
                vec![v, k],
                Some(cmp),
            ),
        ];
        assert!(!nondet_taints_int_to_ptr_deref(&module_with_body(body)));
    }

    #[test]
    fn taint_ignores_nondet_only_as_store_value_not_address() {
        use saf_core::air::{CastKind, Operation};
        use saf_core::ids::FunctionId;
        // %v = call nondet_ulong(); %p = inttoptr %v; store %p -> %dst
        // Here the tainted pointer is the STORED VALUE (operand[0]), not the address
        // (operand[1]); no deref of the fabricated pointer occurs, so no abstain.
        let v = ValueId::new(100);
        let p = ValueId::new(101);
        let dst = ValueId::new(102); // a legitimate address (not tainted)
        let body = vec![
            inst(
                1,
                Operation::CallDirect {
                    callee: FunctionId::new(1),
                },
                vec![],
                Some(v),
            ),
            inst(
                2,
                Operation::Cast {
                    kind: CastKind::IntToPtr,
                    target_bits: None,
                },
                vec![v],
                Some(p),
            ),
            inst(3, Operation::Store, vec![p, dst], None),
        ];
        assert!(!nondet_taints_int_to_ptr_deref(&module_with_body(body)));
    }
}
