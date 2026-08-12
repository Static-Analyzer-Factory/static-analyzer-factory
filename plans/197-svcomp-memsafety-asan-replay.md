# Plan 197: `valid-memsafety` FALSE via an ASan concrete-replay confirmer (R5)

Branch `svcomp`. Brainstormed + recon-grounded (7-agent read-only code map of the R5 surface),
user-approved design 2026-08-12. **No implementation yet — this is the approved design + TDD slicing;
the first production code waits on the Slice-0 go/no-go.** All builds/experiments on the VM
(`ubuntu@cd-vm-15-ai-vm`); laptop = git source of truth; **commit only when the user asks.**

Implements roadmap **R5** (`plans/193` §4.2/§7 row R5, §9 de-risking experiment 3). Follows R4, which was
built to a sound spine then **stopped on low ROI** (`plans/196`, [[saf-svcomp-196-r4-low-roi]]): the next
recall lever is the largest untapped reservoir — **`valid-memsafety` (20,649 tasks), `unknown` on 100%**
today because `strategy_for` has no memsafety arm.

---

## 0. The one-paragraph shape

`saf verify --property <valid-memsafety.prp> ...` currently prints `unknown` (no `strategy_for` arm).
R5 adds a `memsafety_strategy` that compiles the **original** program with `-fsanitize=address -g`, runs it
under a nondet-steering driver, and emits `false(<sub-property>)` **iff AddressSanitizer reports a
violation**. AddressSanitizer is simultaneously **(a)** the sole FALSE arbiter (redline #3 — memsafety's
own concrete confirmer, not the `reach_error` sentinel), **(b)** the witness target source line (from `-g`),
and **(c)** the sub-property classifier (ASan error class → `valid-deref`/`valid-free`). Because ASan is the
arbiter, SAF's over-approximate SVFG/interval memory checkers (measured **75% false-alarm on safe programs**,
`plans/193` §2.2) are **soundness-irrelevant** and are **not used at all in the first scored slice** — a
checker false alarm would simply fail to reproduce under ASan → `unknown`. The first slice runs the ASan
probe **unconditionally** on every memsafety task with default (zeroed) nondet inputs, catching
unconditional / default-path faults (Juliet-heavy); Z3-guided steering to input-guarded faults is a
**gated later slice**, only built if measurement shows a reservoir there.

---

## 1. Global constraints (verbatim; every task inherits these)

- **Never emit `true`.** A memsafety run with no ASan report returns `unknown_outcome()`. (Redline #1.)
- **FALSE only via a concrete ASan report.** No verdict from an over-approximate checker/absint finding
  alone (that is the −16 the 75%-FP measurement warns about). ASan is the sole arbiter. (Redlines #2, #4.)
- **Memsafety's OWN confirmer.** `replay_confirms_false` is `reach_error`-sentinel-specific and cannot
  confirm a UAF/OOB/double-free — R5 adds a *new* `asan_confirm`. (Redline #3.)
- **Exact sub-property verdict string.** SV-COMP expects `false(valid-deref)` / `false(valid-free)` /
  `false(valid-memtrack)`, **not** `false(valid-memsafety)` (`benchexec/tools/test_saf.py` pins
  `RESULT_FALSE_{DEREF,FREE,MEMTRACK}`). Ambiguous/unmapped ASan class → `unknown` (never guess a
  sub-property — a wrong sub-property is scored −16).
- **Witness is a side-output, never gates the verdict.** Written only on a `false` received before the
  watchdog deadline (existing `verify()` path, unchanged).
- **Byte-deterministic.** The emitter carries only file:line + class (never ASan's ASLR-randomized
  addresses); fixed `creation_time`, content-derived uuid; fixed `ASAN_OPTIONS`.
- **Bounded.** One ASan compile + run per task under the existing wall-clock watchdog + `replay_timeout()`.
- **All builds/experiments on the VM.** Edit locally; `rsync crates/` (`--delete`) + root files (NO
  `--delete`); TDD; subagents never call `make`. See [[saf-svcomp-vm-env]] / [[saf-svcomp-workflow]].

---

## 2. Current state (verified this session by the code map; re-confirm line numbers before editing —
they drift, e.g. the plan-193 `commands.rs:789` gate is now `:795`)

**The seam is already property-generic (built by plan 194, shifted by R4).** R5 = 3 additions, 0 seam edits:

- `crates/saf-cli/src/commands.rs`
  - `verify()` `:760` — entry; parses `.prp` (`from_prp` `:784`), **hard-gate** `strategy_for(property).is_none() -> unknown` `:795`; worker thread + `recv_timeout` watchdog `:809-825`; **witness written only when `outcome.verdict.starts_with("false")`** `:831-849`; prints verdict `:850`.
  - `strategy_for(property) -> Option<StrategyFn>` `:990-995` — **single arm** `UnreachCall => unreach_strategy`, else `None`. **← add `ValidMemsafety => memsafety_strategy` here.**
  - `run_verdict()` `:1000-1052` — shared compile→ingest→`WitnessMeta`→`VerifyCtx`→strategy. Reused unchanged.
  - `VerdictOutcome` `:959`, `unknown_outcome()` `:965`, `VerifyCtx` `:974`, `StrategyFn` `:985`, `build_witness()` `:1056`. Reused.
  - `unreach_strategy()` `:1078-1197` — the propose→confirm→witness **template** to clone.
  - `MAX_REPLAY_CANDIDATES=16` `:1202`; `SCALAR_NONDET` (12 name→C-type pairs) `:1208`; `replay_timeout()` (`$SAF_VERIFY_REPLAY_TIMEOUT`, 10s) `:1226`.
  - `synthesize_driver(candidate, sentinel)` `:1256-1332` — nondet FIFO generators + `__VERIFIER_assume`→`_exit(0)` + pointer/float defaults + **weak `reach_error`/`__VERIFIER_error` + `__assert_fail` override + `__saf_hit` sentinel** `:1313-1329`.
  - `replay_confirms_false()` `:1342-1410` — native compile `:1362-1375` (`clang -O0 -Wno-everything <clang_flag> -include stub -I srcdir input driver.c`), spawn with **stdout+stderr nulled** `:1382-1387`, poll `try_wait` w/ timeout, confirm = **sentinel file exists**.
  - `compile_to_ir()` `:903-955` — shared C→mem2reg IR (`clang -g -S -emit-llvm -O0 -Xclang -disable-O0-optnone` → `opt -passes=mem2reg`).
- `crates/saf-svcomp/src/property_kind.rs`
  - `enum Property { UnreachCall, ValidMemsafety, ValidMemcleanup, NoOverflow, NoDataRace, Termination, Coverage, Unknown }` `:18-42` — **`ValidMemsafety` is a SINGLE variant** (no sub-property split).
  - `name()` `:72-83` — `ValidMemsafety => "valid-memsafety"` (**wrong for the verdict string; decouple**).
  - `from_prp()` `:110-129` — already maps `valid-free`/`valid-deref`/`valid-memtrack` → `Some(ValidMemsafety)` (memcleanup tested first). Recognition is DONE.
  - `is_supported()` `:86-95` — already includes `ValidMemsafety` (a **red herring**: the live gate is `strategy_for`, not `is_supported`).
  - `DataModel::clang_flag()` `:151-158` — `-m32`/`-m64`.
- `crates/saf-svcomp/src/witness_yaml.rs` — `WitnessMeta` `:40-51`; `enum WaypointKind { Assumption, Branching, FunctionEnter, FunctionReturn, Target }` `:71-102`; `assemble(meta, &[SourceWaypoint])` `:151` (list must **end in exactly one `Target` with `action: follow`**; `Target` constraint MUST be omitted, enforced `:164-176`; unterminated segment errors `:186-189`). Property-generic — reused unchanged.
- `crates/saf-svcomp/src/witness_lower.rs` — `span_to_location` `:26-40` (returns **basename** `:34-38`); `reach_error_inst_in_block` `:68` and `lower_candidate` `:149-197` are **reach_error-CallDirect-specific — do NOT reuse for a faulting op**; `branching` waypoint deliberately drops `column` `:96-102`; the `Target` helper carries a column.
- **Proposers (NOT used in Slice 1; reference for Slice 2 only):** `analyze_memsafety` (`property.rs:729-1160`, DEAD/over-approx) drives the SVFG null-deref/UAF/double-free (`spec.rs` `null_deref():243` / `use_after_free():194` / `double_free():219`; `CheckerFinding` has **no `InstId`/`Span`, only `SvfgNodeId`**, and deref sinks key on the **pointer operand**, `finding.rs:34-58`, `site_classifier.rs:323-369`) + interval buffer-overflow (`absint/checker.rs` `check_buffer_overflow_with_result:498`; `NumericFinding` has `inst_id`+`location`, **no `Span`**, and points at the **GEP not the deref**, `checker.rs:91-113`).
- **BenchExec:** `benchexec/tools/saf.py` `determine_result` `:53-68` accepts any `false(<x>)` generically (**no Python change**); `test_saf.py` `:63-92` proves the expected strings are `false(valid-{free,deref,memtrack})`.
- **Toolchain (`Dockerfile:48-65`):** clang-18 (64-bit ASan runtime ships with it) + `gcc-multilib`+`libc6-dev-i386` (slice-1c, for `-m32` **linking**, NOT ASan). **No `-fsanitize=*` anywhere in the repo** (grep-clean). The 32-bit ASan runtime (`libclang_rt.asan-i386`) is **not guaranteed present** → the #1 de-risk (most memsafety tasks are ILP32). LSan/`-fsanitize=leak` is **64-bit-only** → `valid-memtrack` deferred.
- **Probe harness:** `scripts/svcomp_memsafety_probe.py` (`:1-153`; its docstring mis-names it `_probe_memsafety.py` — the underscore file does not exist). Compiles memsafety tasks, runs `saf run --checkers all --format json`, keyword-classifies (`MEM_KW` `:28-29`); measured 75% FP / 35% recall on 40 tasks.

---

## 3. Architecture decisions (user-approved 2026-08-12)

1. **Confirmer-first, staged.** Slice 1 runs the ASan probe **unconditionally** per task with a default
   (zeroed) nondet driver — no checker, no Z3. Checker-proposer + Z3 steering to input-guarded faults is a
   **gated Slice 2**, built only if measurement shows a reservoir beyond the unconditional probe (the R4
   discipline: measure prevalence before building).
2. **ASan is arbiter + witness-source + sub-property classifier.** The witness target line and the
   sub-property both come from the ASan report, so R5 needs **no** static `CheckerFinding`→`InstId`→`Span`
   locator in Slice 1 (that plumbing gap is deferred with the checkers).
3. **Sub-property scope:** `valid-deref` + `valid-free` first; **defer `valid-memtrack`** (LSan is
   64-bit-only, and leaks are the least-confirmable subset on the ILP32 majority).
4. **ILP32 is a first-class target.** Fix the dev image to provide 32-bit ASan so Slice 1 covers the ILP32
   majority; if the runtime can't be apt-installed cleanly, fall back to a `gcc -m32 -fsanitize=address`
   toolchain. (A missing runtime just link-fails → `unknown` — sound but 0 recall, hence the fix.)

---

## 4. Sub-property classification (ASan error class → verdict string)

The `memsafety_strategy` returns `format!("false({subprop})")` where `subprop` is **derived from the ASan
report class**, decoupled from `Property::name()`:

| ASan report class (stderr `ERROR: AddressSanitizer: <class>`) | sub-property |
|---|---|
| `heap-buffer-overflow`, `stack-buffer-overflow`, `global-buffer-overflow`, `dynamic-stack-buffer-overflow` | `valid-deref` |
| `heap-use-after-free`, `stack-use-after-return`, `stack-use-after-scope`, `use-after-poison` | `valid-deref` |
| `SEGV` (null / wild deref) | `valid-deref` |
| `attempting double-free` | `valid-free` |
| `attempting free on address which was not malloc`(-ed) / bad free of non-heap | **→ `unknown`** (abstain — Slice-0c rule R2: ambiguous with `valid-deref`, e.g. CWE590) |
| `detected memory leaks` (LSan) | `valid-memtrack` — **DEFERRED (not enabled in R5)** |
| any other / unclassifiable | **→ `unknown`** (abstain; never guess) |

Two Slice-0c-derived confirmer rules make the gate sound (both abstain, never guess — see §12):
**R1 (I/O-frame rejection):** reject any ASan report whose faulting frame is a libc I/O interceptor /
output helper (`printf_common`/`scanf_common`/`printLine`/`printf`/`puts`) — the fault must be in the
program's own memory access, not in printing a non-terminated buffer (the Juliet-good FP source).
**R2 (high-fidelity sub-property only):** emit `valid-deref` for buffer-overflow/underflow/overread/
underread + use-after-free/scope/return + `SEGV`, and `valid-free` only for **double-free**; abstain on
bad-free-of-non-heap and unmapped classes.

Slice 0 verifies the exact BenchExec expected-verdict matching semantics (is a *wrong* sub-property on a
memsafety-false task scored −16, or is any correct memsafety sub-property accepted when the meta-property is
violated?) against the 2026 rules + `test_saf.py`. The abstain-on-ambiguity policy is sound either way.

---

## 5. File structure (new + touched)

```
crates/saf-svcomp/src/
  memsafety.rs         (NEW) asan class->sub-property map; verdict-string helper; lower_memsafety_target()
  lib.rs               (edit) pub mod memsafety; re-exports
crates/saf-cli/src/
  commands.rs          (edit) strategy_for arm; memsafety_strategy(); synthesize_asan_driver(); asan_confirm()
Dockerfile             (edit) 32-bit ASan runtime (Slice 1a)
scripts/
  svcomp_memsafety_probe.py   (edit, Slice 0) add the ASan-gate column
  r5_asan_toolchain_probe.sh  (NEW, Slice 0) VM toy trap/link probe (no production code)
tests/programs/c/       (NEW fixtures) unconditional UAF / double-free / heap-OOB / null-deref / safe
crates/saf-cli/tests/   (NEW) verify_memsafety.rs (#[ignore] Docker e2e)
```

Open placement question resolved: memsafety lowering lives in a **new `memsafety.rs`** (keeps
`witness_lower.rs`'s `reach_error`-specific helpers untouched; promote `span_to_location` reuse via the
existing `pub`). The nondet-only driver is a **refactor of `synthesize_driver`** into a shared nondet-core +
two thin wrappers (unreach = +sentinel; memsafety = nondet-only), OR a separate `synthesize_asan_driver`
that reuses `SCALAR_NONDET`; decide in Slice 1d by whichever keeps the diff smallest and byte-stable.

---

## 6. TDD slices

### Slice 0 — de-risk spike (NO production code; the go/no-go gate) — `plans/193` §9 exp 3

- [ ] **0a Toolchain probe (VM).** `scripts/r5_asan_toolchain_probe.sh` inside `docker compose run --rm dev`:
  compile 4 toys (heap-UAF, heap-OOB, double-free, null-deref) with `clang-18 -O0 -g -fsanitize=address`
  at **both** `-m64` and `-m32`; run each; assert stderr contains `ERROR: AddressSanitizer` and the expected
  class; record exit code. **If `-m32` link-fails** (`cannot find -lclang_rt.asan-i386` or crt), identify the
  apt package / compiler-rt path that supplies the i386 runtime (or confirm a `gcc -m32 -fsanitize=address`
  fallback works). Deliverable: the exact Dockerfile change for Slice 1a.
- [ ] **0b Confirmation-signal spec.** From 0a, pin the robust confirm predicate (banner regex + exit
  convention) and the exact `ASAN_OPTIONS` — start from `exitcode=1:abort_on_error=0:detect_leaks=0`
  (deterministic exit code, no SIGABRT/coredump noise), adjust per 0a — incl. how to avoid mis-reading an
  unrelated nonzero exit / bare SIGSEGV-without-ASan as a violation.
- [ ] **0c ASan-gate measurement.** Extend `scripts/svcomp_memsafety_probe.py` with an ASan column: for each
  sampled task (stratified ILP32/LP64, safe/buggy, larger N than 40), compile the **original** with ASan +
  the zeroed-nondet stub, run, record `asan_reported` + class. Report: **(i)** of the safe-program checker
  FPs, how many ASan CLEARS (target: ~all → 0-FA evidence); **(ii)** default-nondet ASan **confirmed-FALSE
  recall floor** on buggy tasks; **(iii)** ILP32/LP64 split; **(iv)** how many buggy tasks ASan MISSES under
  zeroed nondets (the guarded-fault reservoir that would justify Slice 2).
- [ ] **0d Scoring semantics.** Confirm SV-COMP memsafety sub-property matching (wrong-subprop penalty?)
  against the 2026 rules + `benchexec/tools/test_saf.py`; document the exact expected verdict strings.
- [ ] **GATE.** GO iff: ASan cleanly separates (≈0 FA after the gate, nonzero default-nondet recall) **and**
  the (32-bit) toolchain works or has a clear fix. NO-GO → stop & report (as R4 did). Record numbers in the
  Implementation record.

### Slice 1 (GO only) — the unconditional-probe strategy

- [ ] **1a Dockerfile: ASan runtime (64-bit AND 32-bit).** Apply the **0a-confirmed** fix: the base image
  ships **no** `libclang_rt.asan*` at all, so add — near the existing clang/multilib apt block —
  `dpkg --add-architecture i386` + `apt-get install libclang-rt-18-dev libc6-dev:i386 libstdc++6:i386
  libgcc-s1:i386` (the amd64 `libclang-rt-18-dev` ships BOTH the x86_64 and i386 asan archives; do **not**
  install `libclang-rt-18-dev:i386` — it pulls `libc6-amd64:i386`, which conflicts with `libc6-dev:i386`).
  Rebuild `dev`. Add a build-time smoke (`clang-18 -m64` AND `-m32 -fsanitize=address` toy links + traps).
- [ ] **1b `memsafety.rs`: class→sub-property + verdict string.** Failing unit tests: each ASan class string
  → expected sub-property; unmapped → `None`. Implement `asan_class_to_subproperty(&str) -> Option<&str>` +
  `memsafety_verdict(subprop) -> String`. Pure, no I/O.
- [ ] **1c `memsafety.rs`: `lower_memsafety_target`.** Failing unit: given `(file, line, Option<col>)` →
  `Vec<SourceWaypoint>` = one `Target` (`action: follow`, constraint omitted, basename file_name, column if
  present) that `ViolationWitness::assemble` accepts → `entry_type: violation_sequence` + `type: target` at
  the fault line; byte-stable. (No AIR `InstId` needed — ASan supplies the line.)
- [ ] **1d `commands.rs`: ASan driver + `asan_confirm` + `memsafety_strategy`.**
  - Nondet-only driver: reuse `SCALAR_NONDET` generators + `__VERIFIER_assume`→`_exit(0)` + pointer/float
    defaults; **drop** `__saf_hit`/weak-`reach_error`/`__assert_fail`/sentinel; **do NOT** override
    `malloc`/`free`/`memcpy`. Empty nondet sequence ⇒ all generators return 0.
  - `asan_confirm(input, data_model, stub, dir, clang) -> Result<Option<AsanHit>>`: `clang <clang_flag>
    -O0 -g -fsanitize=address -fno-sanitize-recover=address -Wno-everything -include stub -I srcdir input
    driver.c -o harness`; run under `replay_timeout()` with the **0b-pinned `ASAN_OPTIONS`**, **capturing stderr**;
    parse the first `ERROR: AddressSanitizer: <class>` + the faulting stack; apply **R1** (reject if the
    faulting frame is a libc I/O interceptor / `printLine`/`printf`/`scanf`/`puts` — sound abstain) and
    **R2** (high-fidelity class→sub-property; abstain on bad-free-of-non-heap + unmapped); take the first
    program-code frame's `file:line[:col]` as the witness target; return `Some(AsanHit{subprop, file, line,
    col})` iff R1+R2 pass, else `None` (link/compile fail, clean exit, non-ASan crash, timeout, rejected →
    `None` → `unknown`). Also abstain on multithreaded tasks (concurrency out of scope). `detect_leaks=0`.
  - `memsafety_strategy(ctx) -> VerdictOutcome`: build driver → `asan_confirm` → on `Some(hit)`:
    `build_witness(ctx, Some(lower_memsafety_target(hit)))` + `VerdictOutcome{ verdict:
    memsafety_verdict(hit.subprop), witness }`; else `unknown_outcome()`.
  - e2e fixtures (`verify_memsafety.rs`, `#[ignore]` Docker): unconditional heap-UAF → `false(valid-deref)` +
    witness; double-free → `false(valid-free)`; heap-OOB → `false(valid-deref)`; null-deref →
    `false(valid-deref)`; **safe program → `unknown`, no witness**; byte-identical witness across re-runs.
- [ ] **1e `strategy_for` arm + wiring.** Add `ValidMemsafety => Some(memsafety_strategy)` (`:990`). NO gate/
  watchdog/write-path edits. e2e through the **real** `clang→verify` pipeline at **both** ILP32 and LP64.
- [ ] **1f Reservoir-sample measurement (the R5 acceptance evidence).** Blind `saf verify` over a stratified
  memsafety sample (ILP32+LP64): report **0 FA / 0 TRUE**, **recall ↑ from 0**, and **confirmed-witness %**
  via CPAchecker (+ Witch3) on the emitted memsafety witnesses. Re-run `make test` + clippy `-D warnings` +
  fmt; confirm the plan-192/194 unreach e2e suite still green.

### Slice 2 (GATED, later) — steering for input-guarded faults

Build **only if** Slice 0c/1f show a material guarded-fault reservoir. Shape: a memsafety candidate proposer
(SVFG/absint findings, or a lighter faulting-site enumerator) → resolve the faulting instruction/block →
Z3 `check_path_reachable`-style model targeting that block → steered `nondet_sequence` → `asan_confirm`.
Requires the `CheckerFinding`/`NumericFinding`→faulting-`InstId` locator deferred from Slice 1, and
retargeting the path engine at an arbitrary instruction. Witness enrichment (branching/assumption on the
steering path) becomes possible here and is measured against confirmed-%.

---

## 7. Soundness & determinism invariants (mapped to slices)

| Invariant | Where enforced |
|---|---|
| Never `true` | `memsafety_strategy` returns `unknown_outcome()` when `asan_confirm` is `None` (1d) |
| FALSE only via concrete ASan report | `asan_confirm` is the sole `false` source; checkers unused in Slice 1 (1d) |
| Memsafety's own confirmer | new `asan_confirm`, distinct from the `reach_error` sentinel (1d) |
| Correct sub-property or abstain | class→subprop map; unmapped → `unknown` (1b, 1d); semantics verified (0d) |
| Witness never gates verdict; only on `false` | existing `verify()` write path `:831-849`, unchanged (1e) |
| Byte-deterministic | witness carries file:line+class only (no ASLR addrs); fixed `ASAN_OPTIONS`, `creation_time`, content-uuid (1c, 1d) |
| Bounded | one compile+run under the watchdog + `replay_timeout()` (1d) |

---

## 8. Acceptance criteria & evidence

- **Blind `valid-deref`+`valid-free` recall > 0** on the reservoir sample, with **≥1 CPAchecker/Witch3-
  confirmed** memsafety witness, holding **0 false alarms / 0 TRUE**, across **ILP32 + LP64**,
  byte-deterministic (1f).
- Redlines §1 held (audited in 1f).
- **Extensibility proven:** memsafety added as a `strategy_for` arm + a confirmer + a lowering fn with **no
  seam rewrite** — validating the plan-194 property-generic spine generalizes to a second property with a
  different confirmer and a non-`reach_error` witness target.
- Gates: full `make test` (Rust+Python) + clippy `-D warnings` + fmt; unreach e2e suite unregressed.

---

## 9. VM workflow

```
# sync source to the VM (see [[saf-svcomp-vm-env]])
rsync -az --delete .../static-analyzer-factory/crates/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az .../Dockerfile .../scripts/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/   # no --delete
# Slice 1a rebuilds the dev image (ASan runtime):
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose build dev'
# build/test (main agent only; subagents never call make):
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make test 2>&1 | tee /tmp/t.txt'
# focused e2e (Docker, ignored):
docker compose run --rm dev sh -c 'cargo nextest run -p saf-cli --run-ignored all -E "test(verify_memsafety)"'
```

---

## 10. Risks & open questions

- **32-bit ASan runtime absent** (most likely blocker) — resolved by Slice 0a → 1a; sound fallback is
  `unknown` on link-fail, so worst case is 0 ILP32 recall, not an unsound verdict.
- **Confirmed-witness % for memsafety is unknown** (as it was for unreach in plan 194). A target-only
  memsafety witness at the faulting line may confirm poorly; if so, enrichment is a Slice-2 lever. Measured
  in 1f; this is a recall-of-*scored*-FALSEs risk, not a soundness risk.
- **Default-nondet recall ceiling** — the unconditional probe misses input-guarded faults; Slice 0c
  quantifies the gap that would justify Slice 2 (guard against another R4-style "common shape, low payoff").
- **ASan vs the SV-COMP stub** — `-include sv-comp-stubs.h` + the driver must not shadow `malloc`/`free`
  (ASan intercepts them); verified in 1d fixtures.
- **Non-ASan SIGSEGV attribution** — require the ASan banner, not a bare nonzero exit, before claiming a
  violation (0b/1d).

---

## 11. Out of scope (deferred)

- `valid-memtrack` (LSan 64-bit-only; leaks) and `valid-memcleanup` (R10).
- Z3-guided steering to input-guarded faults + the static `CheckerFinding`/`NumericFinding`→`InstId` locator
  (Slice 2, gated).
- Witness enrichment (branching/assumption) — only meaningful once steering exists; gated on the 1f
  confirmed-% result.
- Any memsafety **TRUE** (needs shape/separation — structurally out of reach, `plans/193` §4.2c/§8).

---

## 12. Implementation record

### Slice 0a — VM ASan toolchain probe (2026-08-12, VM `ubuntu@cd-vm-15-ai-vm`, dev image `saf-dev:llvm18`) — GREEN

`scripts/r5_asan_toolchain_probe.sh` + `scripts/r5_asan_pkg_probe.sh` (measurement only, no production code),
run in throwaway `--user root` dev containers.

- **The base image ships NO ASan runtime at all** — not even 64-bit. `/usr/lib/llvm-18/lib/clang/18/lib/
  linux/` has no `libclang_rt.asan*`, so `-fsanitize=address` link-failed at both `-m64` and `-m32`
  (`cannot find libclang_rt.asan-x86_64.a` / `-i386.a`). **Corrects the code-map assumption that "64-bit
  ASan ships with clang-18"** — the Ubuntu apt `clang-18` package does not bundle the compiler-rt runtimes.
- **Confirmed minimal fix (both data models):** `dpkg --add-architecture i386` +
  `libclang-rt-18-dev libc6-dev:i386 libstdc++6:i386 libgcc-s1:i386`. The **amd64** `libclang-rt-18-dev`
  ships BOTH `libclang_rt.asan-x86_64.a` and `libclang_rt.asan-i386.a` (multilib in one package); the
  `:i386` runtime package is a **dead end** (depends on `libc6-amd64:i386`, which conflicts with the
  `libc6-dev:i386` that supplies clang's `-m32` headers/crt — requesting both aborts the whole apt
  transaction). The slice-1c `gcc-multilib`+`libc6-dev-i386` is **insufficient** for clang `-m32 -fsanitize`.
- **After the fix, all 4 classes trap deterministically at `-m64` AND `-m32`** (`exit=1` under
  `ASAN_OPTIONS=exitcode=1:abort_on_error=0:detect_leaks=0`): `heap-use-after-free`,
  `heap-buffer-overflow`, `attempting double-free`, `SEGV on unknown address 0x0` — validating the
  §4 class→sub-property map (deref/deref/free/deref) and the §2 confirm predicate
  (stderr `ERROR: AddressSanitizer: <class>`). **0a + 0b (confirm-signal spec) satisfied.**
### Slice 0c/0d — ASan-gate reservoir measurement + sub-property cross-check (2026-08-12, VM) — GO

`scripts/r5_asan_measure.py` + `scripts/r5_asan_zerodriver.c` + `scripts/r5_asan_measure_run.sh`
(measurement only). Compiles each ORIGINAL task with `clang -O0 -g -fsanitize=address` + a zero-nondet
driver (all `__VERIFIER_nondet_*` → 0, `__VERIFIER_assume` honored), runs it, classifies the ASan report
vs the `.yml` `expected_verdict` + `subproperty`. Reservoir = **20,592 valid-memsafety tasks**
(safe 10,505 / buggy 10,087), **~90% Juliet** (18,583); concurrency isolated in
`goblint*/weaver/pthread*/ldv-races/libvsync/locks` (excluded — R5 is sequential).

**Headline (N=30 stratified safe/buggy × ILP32/LP64, sequential):**
- **FALSE-ALARM on safe = 0/36** ✅ — sound gate achieved. TP=28, FN=19, TN=36, compile_fail=6,
  timeout=11, concurrency-excluded=20.
- **default-nondet (UNSTEERED) recall floor = 28/47 ≈ 60%** — well above "recall ↑ from 0"; confirms a
  large fraction of the Juliet-dominated sequential reservoir with **no checker and no Z3 steering**.
- **Works at both data models** (TP: ILP32=7, LP64=21). Class mix: buffer-overflow/underflow (heap+stack),
  use-after-scope/return, UAF, SEGV(null), double/bad-free — all map cleanly to valid-deref / valid-free.
- Misses are the expected categories: memory **leaks** (valid-memtrack, out of scope + LSan off),
  `fscanf`/`socket`/`console` **input-dependent** (Slice-2 steering or out of reach), a few guarded/loops.

**Two soundness findings that REFINE the Slice-1 confirmer (both "abstain, never guess"):**
1. **Un-gated, the probe was NOT 0-FA (7/72):** 4 concurrency tasks (excluded) + 3 Juliet-*good* tasks
   whose ASan trap is a `printf("%s")` **read-past a non-terminated buffer inside the `printLine` output
   helper** (frame #0 = ASan's `printf_common` interceptor). SV-COMP `valid-memsafety` does **not** count
   libc `printf` string reads → these are artifacts. **Confirmer rule R1:** reject any ASan report whose
   faulting frame is a libc **I/O interceptor / output helper** (`printf_common`/`scanf_common`/`printLine`/
   `printf`/`puts`) — the fault must be in the program's own memory access. (`ASAN_OPTIONS=check_printf=0`
   also removes the artifact but is too blunt — see finding 2 — so prefer frame-based rejection.)
2. **Sub-property fidelity = 11/12 agree; 1 −16 risk.** `CWE590_Deref_Out_Of_Scope...` expects
   `valid-deref`, but its intended out-of-scope deref manifests *through* `printLine`/`printf`; with
   `check_printf=0` that intended fault was suppressed, the program ran on, and ASan caught a *secondary*
   bad-free → mapper said `valid-free` (**wrong sub-property = −16**). Frame-based rejection (R1) fixes this
   (ASan aborts at the FIRST error = the printf read, which R1 rejects → sound abstain, never a
   mis-classified secondary verdict). **Confirmer rule R2:** map ASan class → sub-property only for the
   high-fidelity classes (`*-buffer-overflow/underflow/overread/underread`, `*use-after-free/scope/return`,
   `SEGV` → `valid-deref`; **double-free** → `valid-free`); **abstain (unknown) on ambiguous
   bad-free-of-non-heap and any unmapped class**. Blind, we cannot cross-check the expected sub-property, so
   abstain-on-ambiguity is the sound lever (recall cost only, never −16).

**GO decision:** the toolchain is solved (0a), the gate is sound (0 FA with R1), unsteered recall is ~60%
on the sequential reservoir, and the −16 sub-property risk is eliminable with R1+R2. **Proceed to Slice 1**
with the confirmer implementing R1 (I/O-frame rejection) + R2 (high-fidelity sub-property map, abstain
otherwise) + concurrency abstain + `detect_leaks=0`. Slice 1's e2e/measurement re-validates 0 FA and the
confirmed sub-property fidelity on the real `saf verify` path.

### Slice 1 — the `memsafety_strategy` Rust build (2026-08-12, VM, TDD) — DONE (uncommitted)

- **1a Dockerfile** — added the 0a-confirmed ASan package layer (`libclang-rt-18-dev libc6-dev:i386
  libstdc++6:i386 libgcc-s1:i386`, `dpkg --add-architecture i386`) late in `base`; rebuilt `saf-dev:llvm18`;
  ASan now works out-of-the-box at `-m64` AND `-m32`.
- **1b/1c `crates/saf-svcomp/src/memsafety.rs`** (pure, 10 unit tests, RED→GREEN): `parse_asan_report`
  (R1 I/O-frame rejection via `frame_function`/`is_format_io`/`is_print_helper`; R2 `asan_class_to_subproperty`;
  first program-source frame → target location), `memsafety_verdict`, `lower_memsafety_hit` (target-only
  waypoint). Fixtures = the real ASan reports captured in 0c. Re-exported from `lib.rs`.
- **1d/1e `crates/saf-cli/src/commands.rs`**: `strategy_for` arm `ValidMemsafety => memsafety_strategy`;
  `memsafety_strategy` (propose-free confirmer-first); `synthesize_asan_driver` (nondet-only, all-zero,
  `__VERIFIER_assume`-honouring, no sentinel, no malloc/free override); `asan_confirm` (compiles the ORIGINAL
  with `-fsanitize=address -g` + the driver, runs under `replay_timeout()` with
  `ASAN_OPTIONS=exitcode=1:abort_on_error=0:detect_leaks=0` capturing stderr to a FILE, parses via
  `parse_asan_report`; abstains on threads via `has_threading_primitives`; `Ok(None)` on any
  inconclusive → `unknown`). 8 e2e fixtures + tests (`tests/programs/c/svcomp/memsafety_*.c` +
  `valid-memsafety.prp`): heap-OOB→`false(valid-deref)` at LP64 AND ILP32, double-free→`false(valid-free)`,
  null-deref→`false(valid-deref)`, safe→`unknown`/no witness, witness-written, byte-stable.
- **Gates (all VM-green):** `make test` **2250 nextest** (+ pytest), **31/31 verify e2e** (unreach
  unregressed + 8 memsafety), clippy `--workspace -D warnings` + fmt clean.
- **1f blind `saf verify` reservoir eval** (`scripts/r5_verify_memsafety_eval.py`, N=20 stratified
  safe/buggy × ILP32/LP64, sequential): **FP=0 (0 false alarms) / TRUE=0 — the −16 audit passes on the REAL
  pipeline**; recall **6/40 = 15%** (honest: the ILP32 stratum is real-world `memsafety-cve` tasks whose
  bugs need crafted input — Slice-2/out-of-reach — while the confirmed TPs are small array/scope tasks that
  fault unconditionally); **sub-property match 6/6 (0 −16)**; **6/6 false verdicts wrote a witness**.
- **Confirmed witness (the "does it score" proof):** on `scopes5.c` (memsafety-ext3, ILP32) `saf verify` →
  `false(valid-deref)` + a target-only YAML-2.0 witness → **witnesslint `LINT_OK` + CPAchecker `CONFIRMED`**.

**ACCEPTANCE MET:** blind `valid-deref`+`valid-free` recall ↑ from 0 with a **CPAchecker-confirmed** witness,
holding **0 false alarms / 0 TRUE** across ILP32+LP64, byte-deterministic; memsafety added as a `strategy_for`
arm + confirmer + lowering fn with **no seam rewrite** (the plan-194 spine generalized). Redlines §1 held.
Committed `d163e30`.

### Slice 2 — de-risk → the recall levers (2026-08-12, VM, TDD) — DONE

Started as the planned Z3 input-steering slice; the de-risk (`scripts/r5_slice2_derisk.py` + `r5_asan_constdriver.c`) redirected it to three sound, higher-ROI fixes, and surfaced that the honest recall story is NOT what Slice 1f's 15% implied.

- **Slice-2 de-risk (mini-fuzz):** compile each buggy task once with a const-driver, run under a spread of
  constants ({0,1,2,42,255,256,1024,65535,2³¹−1,−1}). On the ALL-buggy sample: 0 steerable (the R4 pattern —
  masked because caught@0 was dominated by threaded Juliet variants). On the **sequential-only** subset:
  **8/60 trap under a nonzero constant but not 0** (`@42:stack-buffer-overflow`, `@2³¹−1:SEGV`, `@1:…`) — a
  real scalar-guarded/scalar-sized reservoir. **So heavy Z3 steering is NOT needed; a multi-constant
  mini-fuzz recovers it.** Scalar-nondet Z3 steering (the plan's original Slice 2) would only add the
  specific-value guards (`==31337`) a fuzz misses — deferred as low marginal ROI.
- **Fix A — threading guard was over-broad (bug).** `has_threading_primitives` flags `__VERIFIER_atomic_*`,
  mutex ops, and `fork` — none of which spawn threads — so it false-abstained on huge numbers of sequential
  tasks. Replaced with `program_spawns_threads` (only `pthread_create`/`thrd_create`; LLVM keeps only
  referenced symbols so a name match = an actual call). Regression fixture `memsafety_false_atomic_nothread.c`.
  (Genuinely-threaded Juliet flow variants — a `pthread_create` sink — are still soundly abstained: a
  concurrency benchmark's safe-but-ASan-traps schedule is a −16 risk, per the Slice-0c race FPs.)
- **Fix B — `check_printf=0`** added to the replay `ASAN_OPTIONS`: stops ASan aborting at a benign
  `printf("%s")` artifact before a task's real fault (sound — SV-COMP doesn't count libc printf reads;
  suppressing reports can't add a false alarm; R2 still abstains on any ambiguous secondary fault).
- **Fix C — multi-constant mini-fuzz confirmer:** `synthesize_asan_driver` now returns `$SAF_NONDET_CONST`
  for every scalar nondet; `asan_confirm` runs the one binary under `NONDET_CONSTS` (0 first) and confirms on
  the FIRST trap. Sound: each constant is a valid concrete input; `__VERIFIER_assume` prunes infeasible ones.
  Regression fixture `memsafety_false_guarded.c` (`if (nondet()==42) OOB`).
- **Measured (blind `saf verify`, sequential subset, N=40):** recall **8/37 → 12/37 (22%→32%)** with the
  mini-fuzz, holding **FP=0 / TRUE=0 / sub-property 12/12 / 12-12 witnesses** — the fuzz never traps a safe
  program (soundness confirmed). Gates: 2250 nextest, **33/33 verify e2e**, clippy `-D warnings` + fmt clean.
- **Honest recall correction:** Slice 1f's 15% (and the 2% representative number) were sampling/guard
  artifacts. The real picture: on the **sequential** reservoir (R5's scope) the confirmer catches ~a third
  of buggy tasks soundly; the misses are leaks (valid-memtrack, out of scope), real-world CVE needing crafted
  non-scalar input, loops/arrays (big-engine), and native-compile failures — NOT a steering gap.

**Deferred (future):** concurrency-aware confirmation to soundly recover deterministic-thread Juliet variants
(distinguish a single-worker deterministic sink from a race — the largest remaining reservoir, but a subtle
soundness design); a full CPAchecker confirmation-% sweep; per-nondet Z3 steering for specific-value guards.
