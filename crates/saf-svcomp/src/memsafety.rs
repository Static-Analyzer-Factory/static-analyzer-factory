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
//! - **R1 (harness-frame rejection):** a report whose faulting frame is a libc
//!   formatted-I/O function / interceptor (`printf`/`scanf` family), a Juliet
//!   `print*Line` output helper (an artifact of printing a non-terminated buffer —
//!   SV-COMP `valid-memsafety` does not count libc `printf` string reads), or an
//!   sv-benchmarks LDV allocator MODEL (`ldv_reference_realloc` &c., whose native
//!   execution over-reads by design of the abstraction) is a harness artifact, not a
//!   memory-safety violation of the program under test → abstain.
//! - **R2 (high-fidelity sub-property only):** map only the unambiguous ASan
//!   classes to a sub-property (`*-buffer-overflow/underflow/overread/underread`,
//!   `*use-after-free/scope/return`, `SEGV` → `valid-deref`; `double-free` and
//!   `free on address which was not malloc()-ed` → `valid-free`); abstain on any
//!   unmapped class (`alloc-dealloc-mismatch`, `memcpy-param-overlap`, leaks, …).

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
/// (R2). Only high-fidelity classes map; leak / overlap / mismatch / unknown abstain.
#[must_use]
pub fn asan_class_to_subproperty(class_phrase: &str) -> Option<&'static str> {
    let p = class_phrase;
    // valid-free: freeing a pointer that is not the base of a currently-allocated
    // block. SV-COMP's `valid-free` is violated exactly when `free`/`realloc` is
    // applied to such a pointer, so both ASan free-classes map here:
    //   - `attempting double-free` (a block freed twice), and
    //   - `attempting free on address which was not malloc()-ed` (an offset-into-heap
    //     pointer as in `p = malloc(n); free(p + 1)`, or a non-heap/stack/global
    //     address). ASan attributes the fault to the `free` operation itself, so this
    //     is unambiguously a free violation — never a dereference. A `SEGV` reached
    //     THROUGH a deallocator stays abstained in `parse_asan_report` (the class
    //     `SEGV` alone cannot tell a wild deref from a bad free).
    if p.contains("double-free") || p.contains("not malloc()-ed") {
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
    // A `SEGV` can be a wild DEREF (valid-deref) OR a crash inside a deallocator when
    // the program frees a wild pointer (valid-free). The class alone cannot tell them
    // apart, so a `SEGV` whose stack passes through a deallocator is treated as an
    // ambiguous bad-free and abstained — never guess a sub-property (a wrong one is
    // −16). Non-SEGV free violations (`double-free`) map via R2 and are unaffected.
    let is_segv = class_line.starts_with("SEGV");

    // Walk the stack top-down. R1: reject if the faulting access is inside a libc
    // formatted-I/O function/interceptor, a Juliet `print*Line` output helper, or an
    // LDV allocator model (up to and including the first program-source frame).
    // Otherwise the witness target is the first frame carrying a program location.
    for line in stderr.lines() {
        let t = line.trim_start();
        if !t.starts_with('#') {
            continue;
        }
        let Some(descr) = t.split_once(" in ").map(|(_, d)| d) else {
            continue;
        };
        let func = frame_function(descr);
        if is_format_io(func) || is_print_helper(func) || is_harness_memory_model(func) {
            return None; // R1
        }
        if is_segv && is_deallocator(func) {
            return None; // ambiguous bad-free SEGV -> abstain (never guess -16)
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

/// An sv-benchmarks LDV MODEL whose fault under concrete ASan replay is an ARTIFACT of
/// the abstraction, not a bug in the program under test. Two families qualify:
///
/// 1. **Allocator models** (`ldv_reference_realloc`, `ldv_malloc`, `ldv_free`, …): e.g.
///    `ldv_reference_realloc` does `res = malloc(NEW); memcpy(res, old, NEW)` — an OOB
///    read of the smaller old buffer.
/// 2. **String-LENGTH models** (`ldv_strlen` / `ldv_strnlen` / `ldv_wcslen`, incl. the
///    numbered wrappers `ldv_strlen_3`): these walk to a NUL, and overread a heap buffer
///    ONLY because the companion `ldv_strcpy`/`ldv_strdup` models copy `ldv_strlen(src)`
///    bytes WITHOUT the terminating NUL (the CWE761 `char_fixed_string` good artifact).
///
/// CRITICALLY, this must NOT match the COPY/INDEX models (`ldv_memcpy`, `ldv_memmove`,
/// `ldv_strcpy`, `ldv_memset`): a REAL CWE121/122/124/126/127 buffer overflow/underread
/// faults *inside those* with a bad size the PROGRAM supplied, so it is a genuine
/// violation — rejecting them abstained on ~1000 real bugs (measured). Only the
/// length-walk artifact and the allocator models are rejected here.
fn is_harness_memory_model(func: &str) -> bool {
    if !func.starts_with("ldv_") {
        return false;
    }
    func.contains("alloc")
        || func.contains("free")
        || func.contains("strlen")
        || func.contains("wcslen")
}

/// A deallocation function or its ASan interceptor (`free`, `cfree`, the allocator's
/// `Deallocate`, `operator delete`). Used to disambiguate a `SEGV`: a crash reached
/// THROUGH one of these is a bad-free (`valid-free`), not a wild deref, so the
/// deref-class mapping is unsafe and the report abstains.
fn is_deallocator(func: &str) -> bool {
    func == "free"
        || func == "cfree"
        || func == "__libc_free"
        || func.contains("Deallocate")
        || func.contains("operator delete")
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
    fn ldv_realloc_model_over_read_is_rejected_r1() {
        // Plan-198 FP (real report, CWE401_..._realloc_good): the sv-benchmarks LDV
        // realloc MODEL does `res = malloc(NEW_size); memcpy(res, old, NEW_size)`, an
        // OOB read of the smaller old buffer under native execution. The fault is in
        // the harness memory model (`ldv_reference_realloc`), NOT the program under
        // test, so it must abstain (never emit `false` on a safe task).
        const LDV_REALLOC_MODEL_OOB: &str = "\
==23==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x5140000001d0 at pc 0x555 bp 0x7ff sp 0x7ff
READ of size 519600 at 0x5140000001d0 thread T0
    #0 0x555 in __asan_memcpy (/tmp/h+0xc5105)
    #1 0x555 in ldv_reference_realloc /workspace/x/CWE401_realloc_13_good.i:1581:5
    #2 0x555 in ldv_realloc /workspace/x/CWE401_realloc_13_good.i:1326:9
";
        assert_eq!(parse_asan_report(LDV_REALLOC_MODEL_OOB), None);
    }

    #[test]
    fn ldv_strcpy_model_overread_inside_ldv_strlen_is_rejected_r1() {
        // Real report (VM, 2026-08-14): CWE761_..._char_fixed_string_01_good
        // (expected_verdict TRUE). goodB2G does malloc(100); ldv_strcpy(data,"Fixed
        // String"); ldv_strlen(data). The sv-benchmarks `ldv_strcpy` MODEL copies
        // ldv_strlen(src) bytes WITHOUT the terminating NUL, so the subsequent
        // ldv_strlen walks off the 100-byte buffer → heap-buffer-overflow READ INSIDE
        // the `ldv_strlen` model. A model artifact on a SAFE program, not a program bug
        // → must abstain (this was 38 valid-deref false alarms at full-pool scale).
        const LDV_STRLEN_OVERREAD: &str = "\
==22==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x50b0000000a4 at pc 0x555 bp 0x7ff sp 0x7ff
READ of size 1 at 0x50b0000000a4 thread T0
    #0 0x555 in ldv_strlen /workspace/x/CWE761_char_fixed_string_01_good.i:1098:12
    #1 0x555 in ldv_strlen_3 /workspace/x/CWE761_char_fixed_string_01_good.i:964:9
    #2 0x555 in goodB2G /workspace/x/CWE761_char_fixed_string_01_good.i:888:13
";
        assert_eq!(parse_asan_report(LDV_STRLEN_OVERREAD), None);
    }

    #[test]
    fn buffer_overflow_faulting_in_ldv_memcpy_is_a_real_bug_and_is_caught() {
        // A real CWE121/122/124/126/127 buffer overflow/underread copies with a bad size
        // the PROGRAM supplied, faulting INSIDE the faithful `ldv_memcpy` model. This is a
        // genuine violation and must be CAUGHT — the earlier broad `ldv_*mem*` rejection
        // abstained on ~1000 of these (measured full-pool). The narrowed rule rejects only
        // the length-walk artifact (`ldv_strlen`), never the copy/index models. #0 is
        // ASan's memcpy interceptor (a `+offset` object token, skipped); the located frame
        // is `ldv_memcpy`.
        const LDV_MEMCPY_OOB: &str = "\
==7==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x502 at pc 0x555 bp 0x7ff sp 0x7ff
WRITE of size 40 at 0x502 thread T0
    #0 0x555 in __asan_memcpy (/tmp/h+0xc5105)
    #1 0x555 in ldv_memcpy /workspace/x/CWE122_heap_bad.i:1162:3
    #2 0x555 in badSink /workspace/x/CWE122_heap_bad.i:900:5
";
        let hit = parse_asan_report(LDV_MEMCPY_OOB).expect("a real ldv_memcpy OOB is a bug");
        assert_eq!(hit.subproperty, "valid-deref");
        assert_eq!(hit.line, 1162);
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

    // A SEGV whose fault is inside a deallocator (`free`) is a bad-free of a wild
    // pointer, not a wild DEREF — the class `SEGV` does not reliably map to the
    // sub-property here (it is really valid-free, or a downstream effect of an
    // earlier deref), so abstain rather than guess `valid-deref` (a wrong
    // sub-property is −16). Real report: ldv-memsafety/memleaks_test3-1
    // (expected_verdict false, subproperty valid-free).
    const SEGV_IN_FREE_BADFREE: &str = "\
==23==ERROR: AddressSanitizer: SEGV on unknown address 0xfffffffffffffff1 (pc 0x555 bp 0x000 sp 0x7ff T0)
    #0 0x555 in __asan::Allocator::Deallocate(void*, unsigned long, unsigned long, __sanitizer::BufferedStackTrace*, __asan::AllocType) (/tmp/h+0x2d756)
    #1 0x555 in free (/tmp/h+0xc5f6f)
    #2 0x555 in entry_point /workspace/x/memleaks_test3-1.i:763:8
";

    #[test]
    fn segv_inside_free_is_ambiguous_bad_free_abstains() {
        // Must NOT emit valid-deref on a free-fault SEGV (the -16 sub-property case).
        assert_eq!(parse_asan_report(SEGV_IN_FREE_BADFREE), None);
    }

    #[test]
    fn free_of_non_malloced_address_is_valid_free() {
        // ASan attributes the fault to the `free` operation itself (`attempting free on
        // address which was not malloc()-ed`), so it is unambiguously a `valid-free`
        // violation — the located program frame is the offending `free` call site.
        // Real reasoning task: memsafety/960521-1-1 (`p = malloc(n); free(p + 1)`),
        // expected_verdict false, subproperty valid-free.
        let hit = parse_asan_report(BAD_FREE_NON_HEAP).expect("a hit");
        assert_eq!(hit.subproperty, "valid-free");
        assert_eq!(hit.file, "badfree.c");
        assert_eq!(hit.line, 10);
    }

    // A real report from the pass-2 (`-ftrivial-auto-var-init=pattern`) replay on
    // `array-memsafety/cstrcat_unsafe.c`: `main` declares `char *s1;` and passes the
    // uninitialized pointer to `cstrcat`, which dereferences it. Pattern-init makes
    // the indeterminate pointer a fixed wild address, so the deref SEGVs
    // deterministically. Frame #0 is the program's own `cstrcat`, so this must map to
    // `valid-deref` at the faulting line (NOT abstain — it is a genuine violation of a
    // program that reads an indeterminate pointer). Locks in the pass-2 output shape.
    const UNINIT_PTR_SEGV: &str = "\
AddressSanitizer:DEADLYSIGNAL
==23==ERROR: AddressSanitizer: SEGV on unknown address 0xffffffff (pc 0x56648147 bp 0xffffdac8 sp 0xffffda90 T0)
==23==The signal is caused by a READ memory access.
    #0 0x56648147 in cstrcat /workspace/x/cstrcat_unsafe.c:5:13
    #1 0x5664825b in main /workspace/x/cstrcat_unsafe.c:17:3
    #2 0xf7c4ecb8  (/lib/i386-linux-gnu/libc.so.6+0x24cb8)
";

    #[test]
    fn uninitialized_pointer_segv_in_program_is_valid_deref() {
        let hit = parse_asan_report(UNINIT_PTR_SEGV).expect("a hit");
        assert_eq!(hit.subproperty, "valid-deref");
        assert_eq!(hit.file, "cstrcat_unsafe.c");
        assert_eq!(hit.line, 5);
        assert_eq!(hit.column, Some(13));
    }

    #[test]
    fn stack_overflow_class_is_not_mapped() {
        // Real-stack exhaustion (a large-but-in-bounds VLA/`alloca`, or deep recursion)
        // is NOT a `valid-memsafety` violation under SV-COMP's unbounded-abstract-stack
        // model — e.g. `array-memsafety/count_down-alloca-1.i` clamps `length < 2^31/4`
        // yet still overflows an 8 MB native stack, but is expected TRUE. So the
        // `stack-overflow` class must stay unmapped (abstain), never `valid-deref`.
        assert_eq!(
            asan_class_to_subproperty("stack-overflow on address 0x1"),
            None
        );
        const SO: &str = "\
==30==ERROR: AddressSanitizer: stack-overflow on address 0xff0bb534 (pc 0x1 bp 0x2 sp 0x3 T0)
    #0 0x1 in main /workspace/x/count_down-alloca-1.i:17:3
";
        assert_eq!(parse_asan_report(SO), None);
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
        assert_eq!(
            asan_class_to_subproperty("attempting free on address which was not malloc()-ed"),
            Some("valid-free")
        );
        // abstain:
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
