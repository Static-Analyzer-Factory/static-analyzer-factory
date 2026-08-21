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

/// True iff the program references any scalar-integer `__VERIFIER_nondet_*`
/// function (declared or defined). When it does not, blind fuzzing cannot change
/// behaviour — the single deterministic path is already covered by the earlier
/// stages — so the caller skips the fuzzer.
#[must_use]
pub fn references_scalar_nondet(module: &AirModule) -> bool {
    module
        .functions
        .iter()
        .any(|f| SCALAR_NONDET.iter().any(|(name, _)| *name == f.name))
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

/// Generate the C source of the byte-stream nondet shim + error sentinel driver.
///
/// Each scalar-integer `__VERIFIER_nondet_T()` consumes `sizeof(T)` little-endian
/// bytes from the input buffer (loaded once from `$SAF_FUZZ_INPUT`; exhausted ⇒
/// `0`), reinterprets them as `T`, appends `"<name> <value>\n"` to
/// `$SAF_FUZZ_LOG`, and returns the value. `reach_error` / `__VERIFIER_error` /
/// `__assert_fail` drop `sentinel` and `_exit`; `__VERIFIER_assume(false)` exits
/// without a hit. Pointer/float/double nondet return `0`/`NULL`.
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

    for (fname, cty) in SCALAR_NONDET {
        if *fname == "__VERIFIER_nondet_bool" {
            // _Bool: an arbitrary byte in a _Bool object is UB; normalise to 0/1
            // (matching the replay driver's `(_Bool)value`).
            let _ = writeln!(
                s,
                "_Bool {fname}(void) {{ unsigned long long r = __saf_take(1); _Bool v = (_Bool)(r & 1ULL); __saf_note(\"{fname}\", (long long)v); return v; }}"
            );
            continue;
        }
        // Reinterpret the low sizeof(T) little-endian bytes as T, then log the value
        // reinterpreted back to `long long` (bit-preserving on two's-complement), so
        // the sequence-replay driver's `(T)value` cast recovers the exact T value.
        let _ = writeln!(
            s,
            "{cty} {fname}(void) {{ {cty} v; unsigned long long r = __saf_take(sizeof({cty})); memcpy(&v, &r, sizeof({cty})); __saf_note(\"{fname}\", (long long)v); return v; }}"
        );
    }

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

    // Non-integer / pointer nondet: same legal defaults the replay driver uses, so
    // the fuzz path and the re-confirm path stay behaviourally equivalent.
    s.push_str("void* __VERIFIER_nondet_pointer(void) { return (void*)0; }\n");
    s.push_str("float __VERIFIER_nondet_float(void) { return 0.0f; }\n");
    s.push_str("double __VERIFIER_nondet_double(void) { return 0.0; }\n");
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

    s
}

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

    for (fname, cty) in SCALAR_NONDET {
        if *fname == "__VERIFIER_nondet_bool" {
            let _ = writeln!(
                s,
                "_Bool {fname}(void) {{ unsigned long long r = __saf_take(1); _Bool v = (_Bool)(r & 1ULL); __saf_note(\"{fname}\", (long long)v); return v; }}"
            );
            continue;
        }
        let _ = writeln!(
            s,
            "{cty} {fname}(void) {{ {cty} v; unsigned long long r = __saf_take(sizeof({cty})); memcpy(&v, &r, sizeof({cty})); __saf_note(\"{fname}\", (long long)v); return v; }}"
        );
    }

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
        });
    }
    seq
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
}
