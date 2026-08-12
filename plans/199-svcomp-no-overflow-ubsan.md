# Plan 199: `no-overflow` FALSE via a UBSan signed-integer-overflow confirmer (R6)

**Status: SCOPING / roadmap placement — NOT yet brainstormed. Do the brainstorm → de-risk → design →
approval → TDD in a NEW session** (this file documents the decision, the evidence, and the R5 template so
that session starts grounded). Branch `svcomp`. Laptop = git source of truth; all builds/experiments on the
VM (`ubuntu@cd-vm-15-ai-vm`); commit only when the user asks. Follows plan 198 (R5 memsafety follow-on,
committed `0ec8f0b`).

## 1. The decision and why (evidence-backed)

Implements roadmap **R6** (`plans/193` §4.3 / §7). The `no-overflow` reservoir is **9,138 tasks**, `unknown`
on **100%** today (`strategy_for` has no `NoOverflow` arm — `commands.rs`). SV-COMP `no-overflow = FALSE`
means a signed-integer overflow occurs on some execution. This is the next FALSE lever after plan 198.

**R5's `valid-memsafety` confirmer-first design generalizes almost verbatim** (plan 197 + plan 198): compile
the ORIGINAL program with a sanitizer, run it under the nondet-steering driver + multi-constant mini-fuzz,
and emit `false(no-overflow)` **iff the sanitizer reports a violation in the program's own code**. R6 swaps
the arbiter: **`-fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow`** (UBSan)
instead of `-fsanitize=address`. UBSan is simultaneously the sole FALSE **arbiter**, the witness **target
line** (from `-g`), and — since `no-overflow` has **no sub-property split** (verdict is just
`false(no-overflow)`) — no classifier is needed (simpler than memsafety's R2).

## 2. The likely reframe to test first (the plan-198 discipline)

`plans/193` framed R6 as a "loop-free slice" gated on the interval overflow checker (`absint/checker.rs`,
CWE-190) and "fix `DEFAULT_BITS` width derivation first." **But R5 proved the confirmer-first pattern makes
the over-approximate checkers — and their gating — soundness-irrelevant:** the confirmer runs the REAL
program, so a UBSan trap is a concrete overflow regardless of loops, and `DEFAULT_BITS` (a static-checker
concern) is likely irrelevant to a runtime arbiter. **Hypothesis to de-risk: the loop-free gating and the
interval checker are NOT needed** — a UBSan confirmer harvests overflow FALSEs directly, exactly as
`asan_confirm` harvests memsafety FALSEs. (Confirm before assuming; this is the plan-198-style premise check.)

## 3. Soundness redlines (inherit from R5 / CLAUDE.md)

- Never `true`; emit `false(no-overflow)` only on a concrete UBSan-confirmed signed-overflow; abstain
  (`unknown`) otherwise. The mini-fuzz stays sound (each constant is a valid concrete input; `__VERIFIER_assume`
  prunes infeasible ones). Byte-deterministic; bounded per-task time.
- **`no-overflow` counts only SIGNED integer overflow** (unsigned wrap is defined C and is NOT the property).
  Compile with `-fsanitize=signed-integer-overflow` ONLY (not the full `-fsanitize=undefined`), so UBSan does
  not trap on defined-but-flaggable behaviors the property doesn't count.
- An R1-analog **harness-frame rejection** is likely still needed (a UBSan overflow inside a libc/print/LDV
  helper is not the program's own overflow) — reuse the `parse_asan_report` R1 discipline.

## 4. Key de-risk questions (Slice 0, NO production code — the GO/NO-GO gate)

1. **Toolchain:** does the dev image ship the UBSan runtime (`libclang_rt.ubsan_standalone*`) for
   signed-integer-overflow at **both** `-m64` and `-m32`? (R5 found the ASan runtime was absent and needed a
   Dockerfile fix — probe UBSan the same way: `scripts/r6_ubsan_toolchain_probe.sh`.)
2. **Confirm signal:** pin the confirm predicate + `UBSAN_OPTIONS` (`halt_on_error=1:abort_on_error=…`) for a
   deterministic exit + a captured `runtime error: signed integer overflow: … cannot be represented in type
   'int'` at `file:line:col`.
3. **FP surface (the −16 audit):** run the UBSan probe over SAFE `no-overflow=true` tasks — does it trap on
   overflows the label doesn't count (harness code, unsigned patterns mis-compiled, intended-benign signed
   overflow)? Design the frame/rejection gate so threaded-safe FP ≈ 0 (plan-198-style).
4. **Recall floor:** default-nondet + mini-fuzz UBSan-confirmed recall on BUGGY `no-overflow` tasks; ILP32 vs
   LP64 split; the loop/multi-input miss shape.
5. **Witness:** a `no-overflow` violation witness = target-only at the overflowing op's line (UBSan supplies
   it) — reuse a `lower_memsafety_hit`-style target-only lowering. Verify **CPAchecker validates
   `no-overflow` witnesses** (the plan-194/195 validator path) — confirmed-% is the scored-FALSE metric.
6. **Scoring:** confirm `benchexec/tools/test_saf.py` expects exactly `false(no-overflow)` (no sub-property).

## 5. Code seam (R5 template — 3 additions, 0 seam edits)

- `crates/saf-svcomp/src/property_kind.rs` — `NoOverflow` variant + `from_prp` recognition already exist
  (`:29/:59/:121`). `name()` → `"no-overflow"` is the correct verdict string (no decoupling needed).
- `crates/saf-cli/src/commands.rs` — add `strategy_for` arm `NoOverflow => overflow_strategy`;
  `overflow_strategy` + `ubsan_confirm` (clone `memsafety_strategy` / `asan_confirm` with the UBSan flags +
  `UBSAN_OPTIONS` + a UBSan-report parser); **reuse** `synthesize_asan_driver` (nondet-only) + `NONDET_CONSTS`
  mini-fuzz + the witness write path unchanged.
- `crates/saf-svcomp/src/` — a small `overflow.rs` (or extend `memsafety.rs`'s pure report-parsing pattern):
  `parse_ubsan_overflow(stderr) -> Option<Hit>` with the R1-analog frame rejection + `lower_overflow_hit`.
- Dockerfile — add the UBSan runtime if Slice-0 finds it absent (mirror the R5 ASan layer).

## 6. First steps for the new session

1. `superpowers:brainstorming` → clarify scope + the §2 reframe with the user.
2. Recon: read `commands.rs` (`memsafety_strategy`, `asan_confirm`, `parse_asan_report` R1, `synthesize_asan_driver`,
   `NONDET_CONSTS`) as the template; `property_kind.rs` (`NoOverflow`); `absint/checker.rs` (CWE-190, for
   reference only — likely unused, per §2); `benchexec/tools/test_saf.py`.
3. Run the §4 de-risk (toolchain + FP surface + recall + witness-confirm) — GO/NO-GO before any production
   code (the R4/R5/plan-198 discipline).
4. Design → user approval → TDD on the VM. Then resume the roadmap at **R7 `termination` loop-free ∧
   acyclic-callgraph TRUE** (`plans/193` §7) — the first sound-TRUE slice.

Context: `plans/193` §4.3/§7 (R6), `plans/197` (R5 ASan template), `plans/198` (the confirmer-first +
premise-check discipline), [[saf-svcomp-198-reachability-gate]], [[saf-svcomp-197-memsafety-slice0-go]],
[[saf-svcomp-capability-findings]], [[saf-svcomp-workflow]], [[saf-svcomp-vm-env]].
