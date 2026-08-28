//! `no-overflow` FALSE support: parse a clang UndefinedBehaviorSanitizer
//! signed-integer-overflow report into a confirmed, located violation, and lower
//! it to a YAML-2.0 violation witness (plan 199, R6).
//!
//! This module is **pure** (no I/O, no subprocess): the `saf verify` overflow
//! strategy in `saf-cli` compiles the ORIGINAL program with
//! `-fsanitize=signed-integer-overflow`, runs it under the nondet driver +
//! multi-constant mini-fuzz, captures stderr, and hands the report text here.
//! UBSan is the sole FALSE arbiter *and* the source of the witness location.
//! Because `no-overflow` has no sub-property, the verdict is always
//! `false(no-overflow)` — no classifier is needed (simpler than memsafety's R2).
//!
//! Soundness ([`parse_ubsan_overflow`]):
//! - **SIGNED overflow only.** The confirmer compiles with
//!   `-fsanitize=signed-integer-overflow` ONLY, so the only diagnostics possible are
//!   the three signed-overflow forms below; unsigned wrap and left-shift (a separate
//!   `-fsanitize=shift` check) never trap. Every matched report is a genuine C11
//!   signed-overflow UB = a real `no-overflow` violation.
//! - **R1 (verifier-abstraction frame rejection):** a report whose faulting frame #0
//!   is a verifier abstraction (an `ldv_*` model or a `__VERIFIER_*` stub) is rejected
//!   — its internal arithmetic is not the program-under-test's semantics. This is
//!   narrower than memsafety's R1: an overflow inside a genuine program helper (even a
//!   `printLine`) IS a real `no-overflow` violation and is NOT rejected.

use crate::witness_yaml::{Action, Constraint, SourceWaypoint, WaypointKind};

/// A confirmed UBSan signed-integer-overflow violation: the program-source location
/// of the overflowing operation (the witness target). `no-overflow` has no
/// sub-property, so there is nothing else to carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverflowHit {
    /// Basename of the faulting source file (matches the task's `input_files`).
    pub file: String,
    /// 1-based line of the overflowing operation.
    pub line: u32,
    /// 1-based column, if UBSan reported one.
    pub column: Option<u32>,
}

/// The three UBSan `-fsanitize=signed-integer-overflow` diagnostic forms, each a
/// genuine signed overflow (matching only the first would drop the `INT_MIN`
/// negation and `INT_MIN / -1` division cases — real violations):
/// - `+ - *`      → `runtime error: signed integer overflow: <a> <op> <b> cannot be represented in type '<T>'`
/// - unary `-`    → `runtime error: negation of <x> cannot be represented in type '<T>'`
/// - `INT_MIN/-1` → `runtime error: division of <x> by -1 cannot be represented in type '<T>'`
const OVERFLOW_MARKERS: [&str; 3] = [
    "runtime error: signed integer overflow:",
    "runtime error: negation of ",
    "runtime error: division of ",
];

/// Parse a UBSan stderr report into a confirmed overflow hit, applying R1
/// (verifier-abstraction frame rejection). Returns `None` to abstain (no
/// signed-overflow diagnostic, an abstraction-frame fault, or no locatable line) —
/// the caller then keeps `unknown`.
#[must_use]
pub fn parse_ubsan_overflow(stderr: &str) -> Option<OverflowHit> {
    // The located diagnostic line: `<file>:<line>[:<col>]: runtime error: <signed overflow ...>`.
    let err_line = stderr.lines().find(|l| is_overflow_line(l))?;

    // Attribute the overflow to its own function. For `-fsanitize=signed-integer-overflow`
    // frame #0 IS the overflowing operation — UBSan adds no interceptor frames above the
    // fault (unlike ASan's `__asan_memcpy`/`printf_common`), so R1 inspects frame #0 only.
    // Emit `false` ONLY when that frame is a symbolized PROGRAM function: a
    // verifier-abstraction frame is a harness artifact (R1), and a frame we cannot
    // identify — unsymbolized, or no stack at all — cannot be attributed to the program,
    // so BOTH abstain (a wrong `false(no-overflow)` is −16). `print_stacktrace=1` + `-g`
    // makes frame #0 reliably symbolized in practice (plan 199 §13), so requiring it costs
    // no measured recall while closing the unattributable-frame hole.
    let func = frame0_function(stderr)?;
    if is_verifier_abstraction(func) {
        return None; // R1
    }

    let (file, line, column) = parse_error_location(err_line)?;
    Some(OverflowHit {
        file: basename(&file),
        line,
        column,
    })
}

/// The SV-COMP verdict string for a confirmed signed-integer overflow. `no-overflow`
/// has no sub-property, so this is always `false(no-overflow)`.
#[must_use]
pub fn overflow_verdict() -> String {
    "false(no-overflow)".to_string()
}

/// One nondet-input binding used to ENRICH an overflow violation witness: on the
/// execution that reproduced the trap, the source variable `lhs` at `file:line`
/// held the concrete `value`. The witness lowerer turns each of these into a
/// `c_expression` assumption waypoint (`lhs == value`) placed BEFORE the target.
///
/// Populated by the confirmer from the winning mini-fuzz constant (which every
/// scalar-integer `__VERIFIER_nondet_*` on the reaching path returned) — it is
/// the concrete counterexample input the validator needs to replay the path to
/// the overflow, which a bare target waypoint does not carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NondetAssume {
    /// Basename of the source file the read appears in (the task's `input_files`).
    pub file: String,
    /// 1-based source line of the `lhs = __VERIFIER_nondet_*()` read.
    pub line: u32,
    /// The assigned variable (a plain C identifier).
    pub lhs: String,
    /// The concrete (type-cast) value the nondet returned on the trapping run.
    pub value: i64,
}

/// Lower a confirmed hit to a target-only YAML-2.0 violation witness: a single
/// `target` waypoint at the overflowing operation's source line.
#[must_use]
pub fn lower_overflow_hit(hit: &OverflowHit) -> Vec<SourceWaypoint> {
    lower_overflow_hit_enriched(hit, &[])
}

/// Lower a confirmed hit to an ENRICHED YAML-2.0 violation witness: one
/// `assumption` waypoint per reaching nondet read (each binding a nondet input
/// to the concrete value that reproduced the overflow), in source order,
/// followed by the `target` waypoint at the overflowing operation.
///
/// With `assumes` empty this is byte-identical to [`lower_overflow_hit`] (a
/// bare target), so a task the confirmer cannot supply inputs for is unchanged.
///
/// **Soundness:** this is output-only and verdict-neutral — the verdict is
/// already `false(no-overflow)` (UBSan is the sole arbiter). Enrichment can only
/// turn a validator's `confirmed 0 -> +1`; a malformed enrichment can at worst
/// drop an already-confirmed witness, which the confirmed-score gate reverts. The
/// assumptions are TRUE of the witnessed execution (the shim returned that exact
/// value for every scalar nondet), so they never misdirect the validator.
#[must_use]
pub fn lower_overflow_hit_enriched(
    hit: &OverflowHit,
    assumes: &[NondetAssume],
) -> Vec<SourceWaypoint> {
    let mut wps: Vec<SourceWaypoint> = assumes
        .iter()
        .map(|a| SourceWaypoint {
            kind: WaypointKind::Assumption,
            action: Action::Follow,
            file_name: a.file.clone(),
            line: a.line,
            // The read line carries no reliable column for the lhs; omit it.
            column: None,
            function: None,
            constraint: Some(Constraint {
                format: Some("c_expression".to_string()),
                value: format!("{} == {}", a.lhs, a.value),
            }),
        })
        .collect();
    wps.push(SourceWaypoint {
        kind: WaypointKind::Target,
        action: Action::Follow,
        file_name: hit.file.clone(),
        line: hit.line,
        column: hit.column,
        function: None,
        constraint: None,
    });
    wps
}

// ---------------------------------------------------------------------------
// Report-parsing helpers (private).
// ---------------------------------------------------------------------------

/// True iff `line` is a UBSan located diagnostic for one of the signed-overflow forms.
fn is_overflow_line(line: &str) -> bool {
    OVERFLOW_MARKERS.iter().any(|m| line.contains(m))
}

/// The function name of stack frame `#0` (from `print_stacktrace=1`), i.e. the text
/// after ` in ` on the `#0 0x… in <func> …` line, up to the first `(` or space.
fn frame0_function(stderr: &str) -> Option<&str> {
    for line in stderr.lines() {
        let t = line.trim_start();
        if !t.starts_with("#0 ") {
            continue;
        }
        let descr = t.split_once(" in ").map(|(_, d)| d)?;
        return Some(frame_function(descr));
    }
    None
}

/// The function name from a backtrace frame descriptor (text after ` in `):
/// everything before the first `(` (its args/object) or space (its location).
fn frame_function(descr: &str) -> &str {
    let end = descr.find(['(', ' ']).unwrap_or(descr.len());
    &descr[..end]
}

/// A verifier abstraction (an `ldv_*` model or a `__VERIFIER_*` stub) whose internal
/// arithmetic is not the program-under-test's semantics — a fault here is a harness
/// artifact, not a `no-overflow` violation of the program (R1).
fn is_verifier_abstraction(func: &str) -> bool {
    func.starts_with("ldv_") || func.starts_with("__VERIFIER")
}

/// Parse the location prefix from a UBSan diagnostic line
/// (`<file>:<line>[:<col>]: runtime error: …`). Handles both the `file:line:col` form
/// and the column-less `file:line` form UBSan emits when the source carries no column
/// info (mirrors `memsafety::parse_frame_location`). The file may contain '/', never ':'.
fn parse_error_location(line: &str) -> Option<(String, u32, Option<u32>)> {
    let prefix = line.split(": runtime error:").next()?;
    let mut parts = prefix.rsplitn(3, ':');
    let last = parts.next()?;
    let mid = parts.next()?;
    if let Some(file) = parts.next() {
        // <file>:<line>:<col>
        if let (Ok(col), Ok(line_no)) = (last.parse::<u32>(), mid.parse::<u32>()) {
            return Some((file.to_string(), line_no, Some(col)));
        }
        None
    } else {
        // <file>:<line> (no column)
        let line_no = last.parse::<u32>().ok()?;
        Some((mid.to_string(), line_no, None))
    }
}

/// Final path component (the file basename), matching the witness `file_name`
/// convention (`witness_lower::span_to_location`).
fn basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_kind::{DataModel, Language};
    use crate::witness_yaml::{ViolationWitness, WitnessMeta};
    use std::path::PathBuf;

    // Real UBSan reports captured on the VM in the plan-199 Slice-0 de-risk
    // (print_stacktrace=1), condensed. Location is on the `runtime error:` line AND
    // frame #0; the address in `#0 0x…` is ASLR-varying and never used.

    const ADD_OVERFLOW: &str = "\
add.c:2:52: runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'
    #0 0x555555583805 in main /tmp/x/add.c:2:52
    #1 0x7ffff7ca51c9  (/lib/x86_64-linux-gnu/libc.so.6+0x2a1c9)
";

    const NEGATION_OVERFLOW: &str = "\
neg.c:2:50: runtime error: negation of -2147483648 cannot be represented in type 'int'; cast to an unsigned type to negate this value to itself
    #0 0x555555583802 in main /tmp/x/neg.c:2:50
";

    const DIVISION_OVERFLOW: &str = "\
divmin.c:2:60: runtime error: division of -2147483648 by -1 cannot be represented in type 'int'
    #0 0x55555558381a in main /tmp/x/divmin.c:2:60
";

    // Overflow inside a genuine PROGRAM helper `h` — a real no-overflow violation,
    // NOT rejected (R1 only rejects verifier abstractions).
    const PROGRAM_HELPER_OVERFLOW: &str = "\
helper.c:2:50: runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'
    #0 0x5555555837fa in h /tmp/x/helper.c:2:50
    #1 0x555555583850 in main /tmp/x/helper.c:3:10
";

    // Real Juliet 64-bit report (frame #0 is the program's badSink). The parser must
    // still LOCATE it (SAF emits the sound false regardless of the validator's 64-bit
    // confirmation gap — plan 199 §11).
    const INT64_OVERFLOW: &str = "\
CWE190_x.i:1562:35: runtime error: signed integer overflow: 9223372036854775807 * 2 cannot be represented in type 'int64_t' (aka 'long')
    #0 0x555555583f69 in CWE190_Integer_Overflow__int64_t_max_multiply_05_bad /workspace/x/CWE190_x.i:1562:35
";

    #[test]
    fn add_overflow_is_located_at_the_faulting_op() {
        let hit = parse_ubsan_overflow(ADD_OVERFLOW).expect("a hit");
        assert_eq!(hit.file, "add.c");
        assert_eq!(hit.line, 2);
        assert_eq!(hit.column, Some(52));
    }

    #[test]
    fn negation_overflow_is_located() {
        let hit = parse_ubsan_overflow(NEGATION_OVERFLOW).expect("a hit");
        assert_eq!(hit.file, "neg.c");
        assert_eq!(hit.line, 2);
        assert_eq!(hit.column, Some(50));
    }

    #[test]
    fn division_overflow_is_located() {
        let hit = parse_ubsan_overflow(DIVISION_OVERFLOW).expect("a hit");
        assert_eq!(hit.file, "divmin.c");
        assert_eq!(hit.line, 2);
        assert_eq!(hit.column, Some(60));
    }

    #[test]
    fn overflow_in_a_program_helper_is_accepted() {
        // Frame #0 is `h`, the program's own function -> a real violation, located there.
        let hit = parse_ubsan_overflow(PROGRAM_HELPER_OVERFLOW).expect("a hit");
        assert_eq!(hit.file, "helper.c");
        assert_eq!(hit.line, 2);
    }

    #[test]
    fn int64_overflow_is_still_located() {
        let hit = parse_ubsan_overflow(INT64_OVERFLOW).expect("a hit");
        assert_eq!(hit.file, "CWE190_x.i");
        assert_eq!(hit.line, 1562);
        assert_eq!(hit.column, Some(35));
    }

    #[test]
    fn ldv_model_frame_is_rejected_r1() {
        // A signed-overflow fault attributed to an LDV model is a harness artifact.
        let report = "\
x.i:99:7: runtime error: signed integer overflow: 2000000000 + 2000000000 cannot be represented in type 'int'
    #0 0x555555583f69 in ldv_malloc /workspace/x/x.i:99:7
    #1 0x555555583f00 in main /workspace/x/x.i:200:3
";
        assert_eq!(parse_ubsan_overflow(report), None);
    }

    #[test]
    fn verifier_stub_frame_is_rejected_r1() {
        let report = "\
x.i:50:9: runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'
    #0 0x555555583f69 in __VERIFIER_nondet_int /workspace/x/x.i:50:9
";
        assert_eq!(parse_ubsan_overflow(report), None);
    }

    #[test]
    fn no_overflow_diagnostic_is_none() {
        assert_eq!(parse_ubsan_overflow("just program output\ndone\n"), None);
        assert_eq!(parse_ubsan_overflow(""), None);
        // An unsigned/shift diagnostic never appears under -fsanitize=signed-integer-overflow;
        // even if some other UBSan line leaked, it is not one of our markers -> None.
        assert_eq!(
            parse_ubsan_overflow("x.c:1:1: runtime error: left shift of 1 by 31 places"),
            None
        );
    }

    #[test]
    fn frameless_report_abstains_cannot_attribute() {
        // No `#0` frame at all (symbolization absent / print_stacktrace off): the overflow
        // cannot be attributed to a program vs a verifier-abstraction function, so R1
        // cannot run -> abstain (never risk a wrong false(no-overflow); −16). In the real
        // pipeline UBSAN_OPTIONS pins print_stacktrace=1, so a symbolized #0 is always
        // present and this path does not cost recall.
        let report = "add.c:2:52: runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'\n";
        assert_eq!(parse_ubsan_overflow(report), None);
    }

    #[test]
    fn unsymbolized_frame0_abstains() {
        // Frame #0 present but UN-symbolized (no ` in <func>`): we cannot tell whether the
        // fault is in the program or a verifier abstraction, so abstain rather than accept
        // (closing the −16 hole a raw accept-on-missing-frame would open).
        let report = "\
x.i:99:7: runtime error: signed integer overflow: 2000000000 + 2000000000 cannot be represented in type 'int'
    #0 0x555555583f69 (/tmp/harness+0x1a2b3c)
    #1 0x555555583f00 in main /workspace/x/x.i:200:3
";
        assert_eq!(parse_ubsan_overflow(report), None);
    }

    #[test]
    fn line_only_report_is_located_with_no_column() {
        // UBSan can emit a column-less located line (`file:line: runtime error: ...`) when
        // the source carries no column info; the confirmer must still locate it (column
        // None) rather than drop the confirmed overflow to unknown.
        let report = "\
add.c:2: runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'
    #0 0x555555583805 in main /tmp/x/add.c:2
";
        let hit = parse_ubsan_overflow(report).expect("a hit");
        assert_eq!(hit.file, "add.c");
        assert_eq!(hit.line, 2);
        assert_eq!(hit.column, None);
    }

    #[test]
    fn verdict_string_is_false_no_overflow() {
        assert_eq!(overflow_verdict(), "false(no-overflow)");
    }

    #[test]
    fn enriched_lowering_prepends_assumption_waypoints_before_the_target() {
        let hit = OverflowHit {
            file: "t.c".to_string(),
            line: 20,
            column: Some(9),
        };
        let assumes = vec![
            NondetAssume {
                file: "t.c".to_string(),
                line: 5,
                lhs: "x".to_string(),
                value: 2147483647,
            },
            NondetAssume {
                file: "t.c".to_string(),
                line: 6,
                lhs: "y".to_string(),
                value: -3,
            },
        ];
        let wps = lower_overflow_hit_enriched(&hit, &assumes);
        // Two assumptions then the target.
        assert_eq!(wps.len(), 3);
        assert!(matches!(wps[0].kind, WaypointKind::Assumption));
        assert_eq!(wps[0].line, 5);
        assert_eq!(
            wps[0].constraint.as_ref().map(|c| c.value.as_str()),
            Some("x == 2147483647")
        );
        assert_eq!(
            wps[0].constraint.as_ref().and_then(|c| c.format.as_deref()),
            Some("c_expression")
        );
        assert!(matches!(wps[1].kind, WaypointKind::Assumption));
        assert_eq!(
            wps[1].constraint.as_ref().map(|c| c.value.as_str()),
            Some("y == -3")
        );
        // Target last, at the fault, no constraint (format requirement).
        assert!(matches!(wps[2].kind, WaypointKind::Target));
        assert_eq!(wps[2].line, 20);
        assert!(wps[2].constraint.is_none());

        // The enriched witness assembles into a well-formed YAML-2.0 document.
        let meta = WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "CHECK( init(main()), LTL(G ! overflow) )".to_string(),
            data_model: DataModel::LP64,
            language: Language::C,
            input_file: PathBuf::from("/nonexistent/t.c"),
        };
        let yaml = ViolationWitness::assemble(&meta, &wps)
            .expect("assemble")
            .to_yaml_string()
            .expect("serialize");
        assert!(yaml.contains("type: assumption"), "{yaml}");
        assert!(yaml.contains("type: target"), "{yaml}");
        assert!(yaml.contains("value: x == 2147483647"), "{yaml}");
        assert!(yaml.contains("format: c_expression"), "{yaml}");
    }

    #[test]
    fn enriched_lowering_with_no_assumes_equals_target_only() {
        let hit = OverflowHit {
            file: "t.c".to_string(),
            line: 42,
            column: Some(7),
        };
        assert_eq!(
            lower_overflow_hit_enriched(&hit, &[]),
            lower_overflow_hit(&hit)
        );
    }

    #[test]
    fn lowering_produces_a_target_only_witness_at_the_fault_line() {
        let hit = OverflowHit {
            file: "add.c".to_string(),
            line: 42,
            column: Some(7),
        };
        let wps = lower_overflow_hit(&hit);
        assert_eq!(wps.len(), 1);
        assert!(matches!(wps[0].kind, WaypointKind::Target));
        assert!(matches!(wps[0].action, Action::Follow));
        assert!(wps[0].constraint.is_none());

        let meta = WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "CHECK( init(main()), LTL(G ! overflow) )".to_string(),
            data_model: DataModel::LP64,
            language: Language::C,
            input_file: PathBuf::from("/nonexistent/add.c"),
        };
        let yaml = ViolationWitness::assemble(&meta, &wps)
            .expect("assemble")
            .to_yaml_string()
            .expect("serialize");
        assert!(yaml.contains("entry_type: violation_sequence"), "{yaml}");
        assert!(yaml.contains("type: target"), "{yaml}");
        assert!(yaml.contains("line: 42"), "{yaml}");
        assert!(yaml.contains("file_name: add.c"), "{yaml}");
    }
}
