//! CBMC-as-oracle input synthesis for the `unreach-call` FALSE pipeline.
//!
//! This module is **pure** (no I/O, no subprocess): it provides
//! - a cheap, deterministic *structural pre-filter* ([`cbmc_precheck`]) that
//!   decides whether running the bit-precise CBMC bounded model checker on a
//!   program can plausibly help, and
//! - a parser ([`parse_cbmc_trace`]) that turns CBMC's textual counterexample
//!   trace into a [`NondetCall`] sequence in program (nondet-call) order, guided by
//!   a source-line map ([`nondet_line_map`]) of the original program.
//!
//! The subprocess orchestration (invoking the provisioned `cbmc` binary, running
//! the native replay) lives in `saf-cli`; this crate stays subprocess-free.
//!
//! # Why CBMC, and why it is still sound
//!
//! SAF's own Z3 path engine, fixed-k BMC, and forward SE model guard operands as
//! linear-integer terms and unwind loops shallowly. On bit-vector / modular
//! transition systems (the `hardware-verification-bv` btor2c cluster — thousands
//! of `SORT_n` masked-arithmetic circuits behind a `for(;;)` step loop) that model
//! either times out or over-approximates, so those tasks are enumerated but never
//! confirmed. CBMC 6.x is a bit-precise SAT-backed BMC: unwinding the step loop
//! `k` times and solving the resulting propositional formula gives the exact
//! nondet input vector that drives the program to `reach_error`.
//!
//! The same SAT backend covers a second, *loop-free* class those stages also miss:
//! constraint problems (the `xcsp` cluster) whose reach condition is a wide
//! conjunction over dozens of interacting nondet inputs. There is nothing to unwind
//! there — the difficulty is purely the joint solve — so [`cbmc_precheck`]
//! deliberately does not require a loop.
//!
//! CBMC is used ONLY as a candidate *oracle*: the concrete `__VERIFIER_nondet_*`
//! return values it reports are parsed into a [`NondetCall`] sequence and fed to
//! SAF's EXISTING native-replay confirmer, which re-runs the ORIGINAL (unsliced)
//! program with those inputs pinned. The replay is the sole arbiter — a wrong or
//! over-approximate CBMC model (or an imperfect trace parse) simply fails to
//! reproduce `reach_error` and the verdict stays `unknown`. So CBMC can never
//! manufacture a wrong FALSE (R6).
//!
//! # Extracting the input vector from the trace
//!
//! CBMC materializes a nondet read one of two ways in its `--trace`:
//! - a *compound* use (`state = __VERIFIER_nondet_X() & mask;`) introduces a
//!   `return_value___VERIFIER_nondet_X[$N]=<raw>` temp holding the RAW value, then
//!   assigns the masked result to the target; and
//! - a *simple* assignment (`input = __VERIFIER_nondet_X();`) assigns the raw value
//!   DIRECTLY to the target variable with NO `return_value` temp.
//!
//! A parser that keys only on `return_value___VERIFIER_nondet_*` therefore silently
//! drops every simple-assignment read — which is exactly the per-iteration loop
//! inputs of a transition system, the load-bearing part of the vector. So the
//! parser is guided by a source-line map: for each source line that performs a
//! scalar-integer nondet read it records the function, whether the use is compound,
//! and the target variable; then, walking the trace's `State … line N` markers, it
//! takes the RAW value from the `return_value` temp (compound) or the target-var
//! assignment (simple). The masking on the following source line is a different
//! line, so it is never mistaken for the raw value.
//!
//! Contract compliance (`scripts/loop/confirmer_contract.md`):
//! - **R1** — the replay confirms ONLY on `reach_error` / `__VERIFIER_error` /
//!   `__assert_fail` (the property's exact violation event); CBMC's own incidental
//!   checks are disabled at the call site (`--no-standard-checks`).
//! - **R4** — CBMC honours `__VERIFIER_assume` as a hard path constraint natively,
//!   and the replay driver re-applies it (`_exit` on a blocked path).
//! - **R5** — only the scalar-integer nondet family is pinned; a program using
//!   float/double/pointer nondet is pre-filtered OUT ([`cbmc_precheck`]) because
//!   those values are not model-pinnable and CBMC's float theory stalls anyway.
//! - **R6** — the discovered vector must re-trigger `reach_error` deterministically
//!   on the original program (the native replay) before any verdict is emitted.

use crate::fuzz::{self, SCALAR_NONDET};
use crate::property::NondetCall;
use saf_core::air::AirModule;
use std::collections::BTreeMap;

/// Default loop-unwinding bound (`--unwind k`) for the CBMC oracle.
///
/// This — not a wall-clock timer — is the DETERMINISTIC cost bound (the lever's
/// cost-gate contract). A modest depth keeps the SAT instance small enough that a
/// shallow reachable violation (the common case in the transition-system cluster)
/// is found in well under a second, while genuinely deep reaches are missed
/// consistently across machines (a recall cost, never a soundness one). The CLI
/// caller may override via `$SAF_CBMC_UNWIND`.
pub const DEFAULT_UNWIND: u32 = 16;

/// One scalar-integer `__VERIFIER_nondet_*` read recognized at a source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NondetSite {
    /// The nondet function called, e.g. `__VERIFIER_nondet_uint`.
    pub func_name: String,
    /// `true` when the call is part of a larger expression (`= f() & mask`), so the
    /// RAW value lives in a `return_value` temp; `false` for a bare
    /// `lhs = f();` where the raw value is assigned directly to `lhs`.
    pub compound: bool,
    /// The assignment's left-hand-side variable (used to pick the raw-value trace
    /// step for a simple assignment).
    pub lhs: String,
}

/// True iff a program is a plausible CBMC-oracle target.
///
/// The pre-filter is cheap and deterministic, and gates the (relatively expensive)
/// CBMC subprocess so it runs only where it can pay off — after every other
/// unreach-call stage has already abstained. Both conditions must hold:
///
/// 1. **references a scalar-integer nondet** — there is an input vector to
///    synthesize and pin; a program with no fuzzable nondet has a single
///    deterministic path the earlier stages already cover.
/// 2. **no unsupported nondet** — the program must NOT reference any
///    `__VERIFIER_nondet_*` outside the scalar-integer family (float / double /
///    pointer / `charp` / …). Those values are not pinnable by the integer replay
///    driver (so a CBMC model over them could never re-confirm), and CBMC's
///    floating-point / pointer-nondet reasoning is exactly where it stalls. This
///    is conservative (a program touching a float nondet on an irrelevant path is
///    skipped), which only costs recall.
///
/// # Why loop-freedom is NOT a condition
///
/// The filter used to additionally require a reachable loop, on the theory that
/// CBMC's only value-add over SAF's fixed-`k` BMC / forward SE is deep bit-precise
/// unwinding, so an acyclic program is already fully explored by those stages. That
/// is false for the *constraint-problem* class (the `xcsp` cluster: a straight-line
/// nondet assignment vector followed by a long chain of `if (…) goto ERROR;`
/// constraint checks). There is no loop to unwind, but the earlier stages still
/// abstain — the reach condition is a wide conjunction over dozens of interacting
/// inputs that the linear-integer models cannot invert and the blind fuzzer cannot
/// hit by chance, while CBMC's SAT backend solves it directly.
///
/// The clause was a *cost* heuristic, never a soundness one — the native replay is
/// the sole arbiter either way (R6) — and it pruned exactly the class CBMC is best
/// at. The cost it was guarding is bounded instead by CBMC running LAST in the
/// portfolio ([`crate::portfolio::plan_unreach`]), i.e. only on tasks every cheaper
/// lever has already abstained on, and by the caller's `--unwind` bound and
/// safety-valve timeout.
#[must_use]
pub fn cbmc_precheck(module: &AirModule) -> bool {
    fuzz::references_scalar_nondet(module) && !references_unsupported_nondet(module)
}

/// True iff the program references any `__VERIFIER_nondet_*` function that is NOT
/// in the scalar-integer family the replay driver can pin.
///
/// A conservative gate for [`cbmc_precheck`]: pointer / float / double / `charp` /
/// struct nondet all fall here.
#[must_use]
fn references_unsupported_nondet(module: &AirModule) -> bool {
    module.functions.iter().any(|f| {
        f.name.starts_with("__VERIFIER_nondet_")
            && !SCALAR_NONDET.iter().any(|(name, _)| *name == f.name)
    })
}

/// Build a map from 1-based source line number to the scalar-integer nondet read on
/// that line, for every line of `source` that assigns a `__VERIFIER_nondet_*` call.
///
/// A line qualifies when it contains `<lhs> = __VERIFIER_nondet_<type>( )` with
/// `<type>` in the scalar-integer family. `compound` is set when text other than a
/// terminating `;` follows the call (so the raw value is masked / combined into a
/// temp). Lines without a scalar nondet read are absent from the map.
#[must_use]
pub fn nondet_line_map(source: &str) -> BTreeMap<u32, NondetSite> {
    let mut map = BTreeMap::new();
    for (idx, line) in source.lines().enumerate() {
        let Some(site) = parse_nondet_line(line) else {
            continue;
        };
        // 1-based line numbers (matches CBMC's `line N`).
        #[allow(clippy::cast_possible_truncation)]
        let lineno = (idx + 1) as u32;
        map.insert(lineno, site);
    }
    map
}

/// Recognize a single `<lhs> = __VERIFIER_nondet_<scalar>()` assignment on one
/// source line, returning its [`NondetSite`]. Returns `None` for any other line.
fn parse_nondet_line(line: &str) -> Option<NondetSite> {
    const CALL: &str = "__VERIFIER_nondet_";
    let call_at = line.find(CALL)?;
    // The type token follows the prefix, up to '('.
    let after = &line[call_at + CALL.len()..];
    let paren = after.find('(')?;
    let type_token = &after[..paren];
    if type_token.is_empty()
        || !type_token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }
    let func_name = format!("{CALL}{type_token}");
    if !SCALAR_NONDET.iter().any(|(name, _)| *name == func_name) {
        return None;
    }
    // There must be an assignment `=` before the call; the lhs is the token
    // immediately before it. Guard against `==` / `!=` / `<=` / `>=`.
    let before = &line[..call_at];
    let eq = before.rfind('=')?;
    let eq_prev = before.as_bytes().get(eq.wrapping_sub(1)).copied();
    let eq_next = before.as_bytes().get(eq + 1).copied();
    if matches!(eq_prev, Some(b'=' | b'!' | b'<' | b'>')) || eq_next == Some(b'=') {
        return None;
    }
    let lhs = before[..eq].split_whitespace().last()?.to_string();
    if lhs.is_empty() {
        return None;
    }
    // Compound iff anything but a terminating `;` follows the call's `()`.
    let close = after[paren..].find(')')?;
    let tail = after[paren + close + 1..].trim();
    let compound = !(tail.is_empty() || tail.starts_with(';'));
    Some(NondetSite {
        func_name,
        compound,
        lhs,
    })
}

/// Parse CBMC's textual counterexample trace into a [`NondetCall`] sequence, in
/// program (nondet-call) order, using `line_map` to recognize nondet reads.
///
/// The trace is a sequence of `State N file … line L …` markers each followed by an
/// assignment `  <var>=<value> (<binary>)`. For an assignment whose enclosing line
/// `L` is a nondet read:
/// - **compound** read → the raw value is the `return_value___VERIFIER_nondet_*`
///   step (the masked target-var step at the same line is ignored); and
/// - **simple** read → the raw value is the step assigning the site's target var.
///
/// Values are decimal (signed, or unsigned with a trailing type suffix); an
/// unsigned value above `i64::MAX` is stored as its two's-complement `i64` bit
/// pattern, which the replay driver re-widens correctly via its `(cty)` cast. The
/// per-function-name relative order is preserved, matching the FIFO the driver
/// consumes.
#[must_use]
pub fn parse_cbmc_trace(trace: &str, line_map: &BTreeMap<u32, NondetSite>) -> Vec<NondetCall> {
    let mut seq = Vec::new();
    let mut cur_line: Option<u32> = None;
    for line in trace.lines() {
        if let Some(l) = parse_state_line(line) {
            cur_line = Some(l);
            continue;
        }
        let Some(l) = cur_line else { continue };
        let Some(site) = line_map.get(&l) else {
            continue;
        };
        let Some((var, value_str)) = parse_assignment_line(line) else {
            continue;
        };
        let raw = if site.compound {
            // Only the raw return_value temp; the masked target var is skipped.
            if !var.starts_with("return_value___VERIFIER_nondet_") {
                continue;
            }
            value_str
        } else {
            // Bare `lhs = f();`: the raw value is assigned directly to lhs.
            if var != site.lhs {
                continue;
            }
            value_str
        };
        let Some(value) = parse_trace_value(raw) else {
            continue;
        };
        seq.push(NondetCall {
            func_name: site.func_name.clone(),
            value,
            call_inst: None,
        });
    }
    seq
}

/// Extract the 1-based source line from a `State N file … line L thread T` marker.
fn parse_state_line(line: &str) -> Option<u32> {
    let line = line.trim_start();
    if !line.starts_with("State ") {
        return None;
    }
    let idx = line.find(" line ")?;
    let rest = &line[idx + " line ".len()..];
    let num: String = rest.chars().take_while(char::is_ascii_digit).collect();
    num.parse().ok()
}

/// Parse an assignment step line `  <var>=<value> (<binary>)`, returning
/// `(var, value_token)`. `None` for any non-assignment line.
fn parse_assignment_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let eq = line.find('=')?;
    let var = &line[..eq];
    if var.is_empty()
        || !var
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        return None;
    }
    let rhs = &line[eq + 1..];
    // The value is the token before the ` (binary)` annotation (or the whole rhs).
    let value = rhs.split_whitespace().next()?;
    Some((var, value))
}

/// Parse one CBMC trace value token (e.g. `103`, `-7`, `4000000000u`, `42ll`) into
/// an `i64`. Trailing C integer type suffixes (`u`/`l`/`U`/`L`) are stripped; an
/// unsigned magnitude above `i64::MAX` is reinterpreted as its two's-complement
/// `i64` bit pattern (the replay driver's `(cty)` cast restores the value).
fn parse_trace_value(tok: &str) -> Option<i64> {
    let digits = tok.trim_end_matches(['u', 'U', 'l', 'L']);
    if digits.is_empty() {
        return None;
    }
    if let Ok(v) = digits.parse::<i64>() {
        return Some(v);
    }
    // Unsigned values above i64::MAX: keep the bit pattern.
    #[allow(clippy::cast_possible_wrap)]
    if let Ok(v) = digits.parse::<u64>() {
        return Some(v as i64);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- nondet_line_map / parse_nondet_line ---

    #[test]
    fn recognizes_simple_scalar_assignment() {
        let s = parse_nondet_line("  input_44 = __VERIFIER_nondet_ushort();").unwrap();
        assert_eq!(s.func_name, "__VERIFIER_nondet_ushort");
        assert!(!s.compound);
        assert_eq!(s.lhs, "input_44");
    }

    #[test]
    fn recognizes_compound_masked_assignment() {
        let s = parse_nondet_line("  SORT_3 state_6 = __VERIFIER_nondet_ushort() & mask_SORT_3;")
            .unwrap();
        assert_eq!(s.func_name, "__VERIFIER_nondet_ushort");
        assert!(s.compound);
        assert_eq!(s.lhs, "state_6");
    }

    #[test]
    fn recognizes_declaration_with_init() {
        let s = parse_nondet_line("  int x = __VERIFIER_nondet_int();").unwrap();
        assert_eq!(s.func_name, "__VERIFIER_nondet_int");
        assert!(!s.compound);
        assert_eq!(s.lhs, "x");
    }

    #[test]
    fn ignores_non_scalar_and_equality_and_bare_call() {
        assert!(parse_nondet_line("  double d = __VERIFIER_nondet_double();").is_none());
        assert!(parse_nondet_line("  if (x == __VERIFIER_nondet_int()) {").is_none());
        // A bare call with no assignment target.
        assert!(parse_nondet_line("  __VERIFIER_nondet_int();").is_none());
        assert!(parse_nondet_line("  int y = 3;").is_none());
    }

    #[test]
    fn line_map_is_one_based_and_selective() {
        let src = "int main(){\n  int a = __VERIFIER_nondet_int();\n  int b = 5;\n}\n";
        let m = nondet_line_map(src);
        assert_eq!(m.len(), 1);
        assert!(m.contains_key(&2));
        assert_eq!(m[&2].lhs, "a");
    }

    // --- parse_cbmc_trace ---

    fn map_from(pairs: &[(u32, &str, bool, &str)]) -> BTreeMap<u32, NondetSite> {
        pairs
            .iter()
            .map(|(l, f, c, lhs)| {
                (
                    *l,
                    NondetSite {
                        func_name: (*f).to_string(),
                        compound: *c,
                        lhs: (*lhs).to_string(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn extracts_simple_loop_reads_by_target_var() {
        // A simple assignment on line 100 read three times (loop unwound) with
        // distinct values — the case a return_value-only parser drops.
        let trace = "\
State 5 file p.c function main line 100 thread 0
----------------------------------------------------
  input_44=17 (00000000 00010001)

State 40 file p.c function main line 100 thread 0
----------------------------------------------------
  input_44=34 (00000000 00100010)

State 80 file p.c function main line 100 thread 0
----------------------------------------------------
  input_44=3 (00000000 00000011)
";
        let map = map_from(&[(100, "__VERIFIER_nondet_ushort", false, "input_44")]);
        let seq = parse_cbmc_trace(trace, &map);
        let vals: Vec<i64> = seq.iter().map(|n| n.value).collect();
        assert_eq!(vals, vec![17, 34, 3]);
        assert!(
            seq.iter()
                .all(|n| n.func_name == "__VERIFIER_nondet_ushort")
        );
    }

    #[test]
    fn compound_read_uses_return_value_not_masked_target() {
        // Line 64 is compound: return_value has the RAW value (7), state_6 the
        // masked one (3). Only the raw value must be captured.
        let trace = "\
State 12 file p.c function main line 64 thread 0
----------------------------------------------------
  return_value___VERIFIER_nondet_ushort=7 (00000000 00000111)

State 13 file p.c function main line 64 thread 0
----------------------------------------------------
  state_6=3 (00000000 00000011)
";
        let map = map_from(&[(64, "__VERIFIER_nondet_ushort", true, "state_6")]);
        let seq = parse_cbmc_trace(trace, &map);
        assert_eq!(seq.len(), 1);
        assert_eq!(seq[0].value, 7);
    }

    #[test]
    fn ignores_assignments_on_non_nondet_lines() {
        // The masking on line 101 assigns input_44 again but line 101 is not a
        // nondet line, so it is not captured.
        let trace = "\
State 5 file p.c function main line 100 thread 0
----------------------------------------------------
  input_44=17 (00000000 00010001)

State 6 file p.c function main line 101 thread 0
----------------------------------------------------
  input_44=1 (00000000 00000001)
";
        let map = map_from(&[(100, "__VERIFIER_nondet_ushort", false, "input_44")]);
        let seq = parse_cbmc_trace(trace, &map);
        assert_eq!(seq.len(), 1);
        assert_eq!(seq[0].value, 17);
    }

    #[test]
    fn parses_negative_and_unsigned_suffixed_values() {
        let trace = "\
State 1 file p.c function main line 5 thread 0
----------------------------------------------------
  x=-7 (11111111 11111111 11111111 11111001)

State 2 file p.c function main line 6 thread 0
----------------------------------------------------
  u=4000000000u (11101110 01101011 00101000 00000000)
";
        let map = map_from(&[
            (5, "__VERIFIER_nondet_int", false, "x"),
            (6, "__VERIFIER_nondet_uint", false, "u"),
        ]);
        let seq = parse_cbmc_trace(trace, &map);
        assert_eq!(seq.len(), 2);
        assert_eq!(seq[0].value, -7);
        assert_eq!(seq[1].value, 4_000_000_000);
    }

    #[test]
    fn empty_trace_or_no_map_yields_no_calls() {
        let map = map_from(&[(5, "__VERIFIER_nondet_int", false, "x")]);
        assert!(parse_cbmc_trace("", &map).is_empty());
        assert!(parse_cbmc_trace("VERIFICATION SUCCESSFUL\n", &map).is_empty());
        // A trace with a state but an empty map captures nothing.
        let trace = "State 1 file p.c function main line 5 thread 0\n  x=9 (...)\n";
        assert!(parse_cbmc_trace(trace, &BTreeMap::new()).is_empty());
    }

    #[test]
    fn parse_trace_value_strips_suffixes() {
        assert_eq!(parse_trace_value("42"), Some(42));
        assert_eq!(parse_trace_value("-1"), Some(-1));
        assert_eq!(parse_trace_value("7u"), Some(7));
        assert_eq!(parse_trace_value("7ll"), Some(7));
        // (u64)-7 stored as its two's-complement i64.
        assert_eq!(parse_trace_value("18446744073709551609"), Some(-7));
        assert_eq!(parse_trace_value(""), None);
        assert_eq!(parse_trace_value("abc"), None);
    }

    // --- references_unsupported_nondet (the pre-filter's novel gate) ---

    fn module_with_decls(names: &[&str]) -> AirModule {
        use saf_core::air::AirFunction;
        use saf_core::id::make_id;
        use saf_core::ids::{FunctionId, ModuleId};
        let functions = names
            .iter()
            .map(|n| AirFunction {
                id: FunctionId(make_id("func", n.as_bytes())),
                name: (*n).to_string(),
                params: Vec::new(),
                blocks: Vec::new(),
                entry_block: None,
                is_declaration: true,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            })
            .collect();
        AirModule {
            id: ModuleId(make_id("module", b"test")),
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
    fn scalar_only_nondet_is_supported() {
        let m = module_with_decls(&["main", "__VERIFIER_nondet_uint", "__VERIFIER_nondet_uchar"]);
        assert!(!references_unsupported_nondet(&m));
    }

    #[test]
    fn float_and_pointer_nondet_are_unsupported() {
        let m = module_with_decls(&["main", "__VERIFIER_nondet_int", "__VERIFIER_nondet_float"]);
        assert!(references_unsupported_nondet(&m));
        let m = module_with_decls(&["main", "__VERIFIER_nondet_pointer"]);
        assert!(references_unsupported_nondet(&m));
    }

    #[test]
    fn no_nondet_is_not_unsupported() {
        let m = module_with_decls(&["main", "foo"]);
        assert!(!references_unsupported_nondet(&m));
    }

    // --- cbmc_precheck (the whole structural pre-filter) ---

    /// The module of [`module_with_decls`] with its `main` replaced by a DEFINED
    /// two-block `main` carrying a `b0 -> b1 -> b0` CFG back-edge, so
    /// `module_reachable_is_loop_free` is `false`.
    fn with_looping_main(mut module: AirModule) -> AirModule {
        use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
        use saf_core::id::make_id;
        use saf_core::ids::{BlockId, FunctionId, InstId};
        let (b0, b1) = (
            BlockId(make_id("block", b"b0")),
            BlockId(make_id("block", b"b1")),
        );
        let mut block0 = AirBlock::new(b0);
        block0.instructions.push(Instruction::new(
            InstId(make_id("inst", b"b0_term")),
            Operation::Br { target: b1 },
        ));
        let mut block1 = AirBlock::new(b1);
        block1.instructions.push(Instruction::new(
            InstId(make_id("inst", b"b1_term")),
            Operation::Br { target: b0 },
        ));
        module.functions.retain(|f| f.name != "main");
        module.functions.push(AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block0, block1],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        module
    }

    #[test]
    fn precheck_accepts_loop_free_scalar_nondet() {
        // A loop-free constraint program (the `xcsp` shape: a nondet assignment
        // vector then a straight-line chain of constraint checks). CBMC's value here
        // is the bit-precise SAT solve, not loop unwinding, so it must NOT be pruned.
        let m = module_with_decls(&["main", "__VERIFIER_nondet_int", "reach_error"]);
        assert!(crate::fast_paths::module_reachable_is_loop_free(&m));
        assert!(cbmc_precheck(&m));
    }

    #[test]
    fn precheck_accepts_looping_scalar_nondet() {
        let m = with_looping_main(module_with_decls(&["main", "__VERIFIER_nondet_uint"]));
        assert!(!crate::fast_paths::module_reachable_is_loop_free(&m));
        assert!(cbmc_precheck(&m));
    }

    #[test]
    fn precheck_rejects_program_without_scalar_nondet() {
        // Nothing to synthesize/pin: the deterministic path is already covered.
        assert!(!cbmc_precheck(&module_with_decls(&["main", "foo"])));
        assert!(!cbmc_precheck(&with_looping_main(module_with_decls(&[
            "main", "foo"
        ]))));
    }

    #[test]
    fn precheck_rejects_float_only_nondet() {
        // The loop-free x float-only frontier: the blind fuzzer drives these, but the
        // integer replay driver cannot pin a float, so CBMC must stay out.
        let names = &["main", "__VERIFIER_nondet_float"];
        assert!(!cbmc_precheck(&module_with_decls(names)));
        assert!(!cbmc_precheck(&with_looping_main(module_with_decls(names))));
    }

    #[test]
    fn precheck_rejects_unsupported_nondet_regardless_of_loops() {
        // A float/pointer nondet is not pinnable by the integer replay driver, so a
        // CBMC model over it could never re-confirm — with or without loops.
        let names = &["main", "__VERIFIER_nondet_int", "__VERIFIER_nondet_float"];
        assert!(!cbmc_precheck(&module_with_decls(names)));
        assert!(!cbmc_precheck(&with_looping_main(module_with_decls(names))));
    }
}
