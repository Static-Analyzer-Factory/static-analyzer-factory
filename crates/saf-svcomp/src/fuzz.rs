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
const MAX_DICT_ENTRIES: usize = 256;

/// Fixed length of every fuzz input buffer (bytes). 256 bytes covers 32 `int`
/// reads / 64 `short` reads — deep enough for the sv-benchmarks reach tasks, while
/// a CONSTANT length keeps mutation allocation-free and the search deterministic.
pub const INPUT_LEN: usize = 256;

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
