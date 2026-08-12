//! `valid-memsafety` FALSE support: parse an AddressSanitizer report into a
//! confirmed, located, sub-property-classified violation, and lower it to a
//! YAML-2.0 violation witness (plan 197, R5).
//!
//! This module is **pure** (no I/O, no subprocess): the `saf verify` memsafety
//! strategy in `saf-cli` compiles the ORIGINAL program with `-fsanitize=address`,
//! runs it, captures stderr, and hands the report text here. ASan is the sole
//! FALSE arbiter *and* the source of the witness location + sub-property.
//!
//! Two Slice-0c-derived soundness rules (both "abstain, never guess" — a wrong
//! sub-property or a mis-attributed fault is −16):
//! - **R1 (I/O-frame rejection):** a report whose faulting frame is a libc
//!   formatted-I/O function / interceptor (`printf`/`scanf` family) or a Juliet
//!   `print*Line` output helper is an artifact of printing a non-terminated buffer
//!   (SV-COMP `valid-memsafety` does not count libc `printf` string reads) → abstain.
//! - **R2 (high-fidelity sub-property only):** map only the unambiguous ASan
//!   classes to a sub-property (`*-buffer-overflow/underflow/overread/underread`,
//!   `*use-after-free/scope/return`, `SEGV` → `valid-deref`; `double-free` →
//!   `valid-free`); abstain on bad-free-of-non-heap and any unmapped class.

use crate::witness_yaml::{Action, SourceWaypoint, WaypointKind};

/// A confirmed AddressSanitizer memory-safety violation: the SV-COMP sub-property
/// plus the program-source location of the faulting operation (the witness target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsanHit {
    /// `valid-deref` or `valid-free` (the SV-COMP sub-property to report).
    pub subproperty: &'static str,
    /// Basename of the faulting source file (matches the task's `input_files`).
    pub file: String,
    /// 1-based line of the faulting operation.
    pub line: u32,
    /// 1-based column, if ASan reported one.
    pub column: Option<u32>,
}

/// Map an AddressSanitizer error-class phrase (the text after
/// `ERROR: AddressSanitizer: `) to the SV-COMP sub-property, or `None` to abstain
/// (R2). Only high-fidelity classes map; bad-free / leak / overlap / mismatch /
/// unknown all abstain.
#[must_use]
pub fn asan_class_to_subproperty(class_phrase: &str) -> Option<&'static str> {
    let p = class_phrase;
    // valid-free: an unambiguous double-free. Bad-free-of-non-heap is ambiguous
    // with valid-deref (SV-COMP may expect either — the CWE590 −16 case), so it
    // abstains below.
    if p.contains("double-free") {
        return Some("valid-free");
    }
    // valid-deref: an invalid dereference of any kind.
    if p.contains("buffer-overflow")
        || p.contains("buffer-underflow")
        || p.contains("buffer-overread")
        || p.contains("buffer-underread")
        || p.contains("use-after-free")
        || p.contains("use-after-scope")
        || p.contains("use-after-return")
        || p.contains("use-after-poison")
        || p.starts_with("SEGV")
    {
        return Some("valid-deref");
    }
    // Abstain on everything else: bad-free-of-non-heap, memcpy-param-overlap,
    // alloc-dealloc-mismatch, container/intra-object-overflow, leaks, unknown.
    None
}

/// Parse an AddressSanitizer stderr report into a confirmed hit, applying R1 (reject
/// I/O-interceptor / output-helper faults) and R2 (high-fidelity sub-property only).
/// Returns `None` to abstain (no report, ambiguous/unmapped class, an I/O-helper
/// fault, or no locatable program frame) — the caller then keeps `unknown`.
#[must_use]
pub fn parse_asan_report(stderr: &str) -> Option<AsanHit> {
    const MARKER: &str = "ERROR: AddressSanitizer: ";
    let start = stderr.find(MARKER)?;
    let class_line = stderr[start + MARKER.len()..].lines().next()?;
    let subproperty = asan_class_to_subproperty(class_line)?; // R2: abstain if unmapped

    // Walk the stack top-down. R1: reject if the faulting access is inside a libc
    // formatted-I/O function/interceptor or a Juliet `print*Line` output helper
    // (up to and including the first program-source frame). Otherwise the witness
    // target is the first frame carrying a program-source location.
    for line in stderr.lines() {
        let t = line.trim_start();
        if !t.starts_with('#') {
            continue;
        }
        let Some(descr) = t.split_once(" in ").map(|(_, d)| d) else {
            continue;
        };
        let func = frame_function(descr);
        if is_format_io(func) || is_print_helper(func) {
            return None; // R1
        }
        if let Some((path, line_no, col)) = parse_frame_location(descr) {
            return Some(AsanHit {
                subproperty,
                file: basename(&path),
                line: line_no,
                column: col,
            });
        }
    }
    None // no locatable program frame -> abstain
}

/// The SV-COMP verdict string for a confirmed memsafety sub-property, e.g.
/// `false(valid-deref)`. Decoupled from `Property::name()` (which is the combined
/// `valid-memsafety`) because SV-COMP scores the specific sub-property.
#[must_use]
pub fn memsafety_verdict(subproperty: &str) -> String {
    format!("false({subproperty})")
}

/// Lower a confirmed hit to a target-only YAML-2.0 violation witness: a single
/// `target` waypoint at the faulting operation's source line.
#[must_use]
pub fn lower_memsafety_hit(hit: &AsanHit) -> Vec<SourceWaypoint> {
    vec![SourceWaypoint {
        kind: WaypointKind::Target,
        action: Action::Follow,
        file_name: hit.file.clone(),
        line: hit.line,
        column: hit.column,
        function: None,
        constraint: None,
    }]
}

// ---------------------------------------------------------------------------
// Report-parsing helpers (private).
// ---------------------------------------------------------------------------

/// The function name from a backtrace frame descriptor (the text after ` in `),
/// i.e. everything before the first `(` (its args) or space (its location/object).
fn frame_function(descr: &str) -> &str {
    let end = descr.find(['(', ' ']).unwrap_or(descr.len());
    &descr[..end]
}

/// A libc formatted-I/O function or ASan interceptor thereof (`printf`/`scanf`
/// family, `puts`). A fault here is ASan checking a `%s` string read, not a
/// program memory bug (R1).
fn is_format_io(func: &str) -> bool {
    func.contains("printf")
        || func.contains("scanf")
        || matches!(func, "puts" | "fputs" | "fwrite" | "putchar" | "putc")
}

/// A Juliet output helper (`printLine`, `printIntLine`, …). A fault attributed to
/// one is a print-time artifact, not the property under test (R1).
fn is_print_helper(func: &str) -> bool {
    func.starts_with("print") && func.contains("Line")
}

/// Parse a `path:line[:col]` source-location token out of a frame descriptor.
/// Skips binary-offset tokens like `(/tmp/harness+0x1a2b)`.
fn parse_frame_location(descr: &str) -> Option<(String, u32, Option<u32>)> {
    for raw in descr.split_whitespace() {
        let tok = raw.trim_matches(|c| c == '(' || c == ')');
        if !tok.contains('/') || tok.contains('+') {
            continue;
        }
        let mut parts = tok.rsplitn(3, ':');
        let last = parts.next()?;
        let Some(mid) = parts.next() else { continue };
        match parts.next() {
            // path:line:col
            Some(file) => {
                if let (Ok(col), Ok(line)) = (last.parse::<u32>(), mid.parse::<u32>()) {
                    return Some((file.to_string(), line, Some(col)));
                }
            }
            // path:line
            None => {
                if let Ok(line) = last.parse::<u32>() {
                    return Some((mid.to_string(), line, None));
                }
            }
        }
    }
    None
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

    // Real ASan reports captured on the VM in Slice 0c (condensed).
    const HEAP_OOB_PROGRAM: &str = "\
=================================================================
==123==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x502 at pc 0x555 bp 0x7ff sp 0x7ff
WRITE of size 4 at 0x502 thread T0
    #0 0x555 in main /workspace/tests/programs/c/oob.c:5:12
    #1 0x7ff in __libc_start_main (/lib/x86_64-linux-gnu/libc.so.6+0x2a1c9)
";

    // The Slice-0c Juliet-good false alarm: fault is in ASan's printf interceptor.
    const PRINTF_ARTIFACT: &str = "\
==302==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x503 at pc 0x555 bp 0x7ff sp 0x7ff
READ of size 21 at 0x503 thread T0
    #0 0x555 in printf_common(void*, char const*, __va_list_tag*) asan_interceptors.cpp.o
    #1 0x555 in printf (/tmp/inv+0x5306d)
    #2 0x555 in printLine /workspace/x/file.i:204:32
    #3 0x555 in goodB2G /workspace/x/file.i:873:7
";

    const DOUBLE_FREE: &str = "\
==368==ERROR: AddressSanitizer: attempting double-free on 0x502 in thread T0:
    #0 0x555 in free (/tmp/inv+0xc7243)
    #1 0x555 in bad /workspace/x/dfree.c:33:5
";

    const SEGV_NULL: &str = "\
==379==ERROR: AddressSanitizer: SEGV on unknown address 0x000000000000 (pc 0x555 bp 0x7ff sp 0x7ff T0)
    #0 0x555 in main /workspace/x/nullderef.c:1:29
";

    const BAD_FREE_NON_HEAP: &str = "\
==400==ERROR: AddressSanitizer: attempting free on address which was not malloc()-ed: 0x7ff in thread T0
    #0 0x555 in free (/tmp/inv+0x1)
    #1 0x555 in badfn /workspace/x/badfree.c:10:3
";

    // memcpy overflow: frame #0 is ASan's memcpy interceptor (NOT format-I/O),
    // the located program frame is #1.
    const MEMCPY_OOB: &str = "\
==500==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x502 at pc 0x555 bp 0x7ff sp 0x7ff
WRITE of size 10 at 0x502 thread T0
    #0 0x555 in __asan_memcpy (/tmp/inv+0x999)
    #1 0x555 in copyfn /workspace/x/memcpy.c:12:5
    #2 0x555 in main /workspace/x/memcpy.c:20:3
";

    #[test]
    fn heap_overflow_in_program_is_valid_deref_at_the_faulting_line() {
        let hit = parse_asan_report(HEAP_OOB_PROGRAM).expect("a hit");
        assert_eq!(hit.subproperty, "valid-deref");
        assert_eq!(hit.file, "oob.c");
        assert_eq!(hit.line, 5);
        assert_eq!(hit.column, Some(12));
    }

    #[test]
    fn printf_interceptor_read_is_rejected_r1() {
        // The Slice-0c FP source: SV-COMP does not count libc printf string reads.
        assert_eq!(parse_asan_report(PRINTF_ARTIFACT), None);
    }

    #[test]
    fn double_free_is_valid_free() {
        let hit = parse_asan_report(DOUBLE_FREE).expect("a hit");
        assert_eq!(hit.subproperty, "valid-free");
        assert_eq!(hit.file, "dfree.c");
        assert_eq!(hit.line, 33);
    }

    #[test]
    fn segv_null_deref_is_valid_deref() {
        let hit = parse_asan_report(SEGV_NULL).expect("a hit");
        assert_eq!(hit.subproperty, "valid-deref");
        assert_eq!(hit.file, "nullderef.c");
        assert_eq!(hit.line, 1);
    }

    #[test]
    fn bad_free_of_non_heap_abstains_r2() {
        // Ambiguous with valid-deref (the CWE590 −16 case) -> abstain.
        assert_eq!(parse_asan_report(BAD_FREE_NON_HEAP), None);
    }

    #[test]
    fn no_asan_banner_is_none() {
        assert_eq!(parse_asan_report("just some program output\nexit\n"), None);
        assert_eq!(parse_asan_report(""), None);
    }

    #[test]
    fn memcpy_overflow_locates_the_program_frame_not_the_interceptor() {
        let hit = parse_asan_report(MEMCPY_OOB).expect("a hit");
        assert_eq!(hit.subproperty, "valid-deref");
        assert_eq!(hit.file, "memcpy.c");
        assert_eq!(hit.line, 12);
        assert_eq!(hit.column, Some(5));
    }

    #[test]
    fn class_mapping_high_fidelity_only() {
        assert_eq!(
            asan_class_to_subproperty("heap-buffer-overflow on ..."),
            Some("valid-deref")
        );
        assert_eq!(
            asan_class_to_subproperty("stack-buffer-underflow ..."),
            Some("valid-deref")
        );
        assert_eq!(
            asan_class_to_subproperty("dynamic-stack-buffer-overflow ..."),
            Some("valid-deref")
        );
        assert_eq!(
            asan_class_to_subproperty("stack-use-after-scope ..."),
            Some("valid-deref")
        );
        assert_eq!(
            asan_class_to_subproperty("heap-use-after-free ..."),
            Some("valid-deref")
        );
        assert_eq!(
            asan_class_to_subproperty("SEGV on unknown address 0x0"),
            Some("valid-deref")
        );
        assert_eq!(
            asan_class_to_subproperty("attempting double-free on 0x5"),
            Some("valid-free")
        );
        // abstain:
        assert_eq!(
            asan_class_to_subproperty("attempting free on address which was not malloc()-ed"),
            None
        );
        assert_eq!(asan_class_to_subproperty("memcpy-param-overlap ..."), None);
        assert_eq!(
            asan_class_to_subproperty("alloc-dealloc-mismatch ..."),
            None
        );
        assert_eq!(asan_class_to_subproperty("detected memory leaks"), None);
        assert_eq!(asan_class_to_subproperty("something-unknown"), None);
    }

    #[test]
    fn verdict_string_is_the_subproperty() {
        assert_eq!(memsafety_verdict("valid-deref"), "false(valid-deref)");
        assert_eq!(memsafety_verdict("valid-free"), "false(valid-free)");
    }

    #[test]
    fn lowering_produces_a_target_only_witness_at_the_fault_line() {
        let hit = AsanHit {
            subproperty: "valid-deref",
            file: "oob.c".to_string(),
            line: 5,
            column: Some(12),
        };
        let wps = lower_memsafety_hit(&hit);
        assert_eq!(wps.len(), 1);
        assert!(matches!(wps[0].kind, WaypointKind::Target));
        assert!(matches!(wps[0].action, Action::Follow));
        assert!(wps[0].constraint.is_none());

        let meta = WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "SPEC".to_string(),
            data_model: DataModel::LP64,
            language: Language::C,
            input_file: PathBuf::from("/nonexistent/oob.c"),
        };
        let yaml = ViolationWitness::assemble(&meta, &wps)
            .expect("assemble")
            .to_yaml_string()
            .expect("serialize");
        assert!(yaml.contains("entry_type: violation_sequence"), "{yaml}");
        assert!(yaml.contains("type: target"), "{yaml}");
        assert!(yaml.contains("line: 5"), "{yaml}");
        assert!(yaml.contains("file_name: oob.c"), "{yaml}");
    }
}
