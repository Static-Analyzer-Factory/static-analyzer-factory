# Plan 209 — Reason-1 / rank 2: `no-overflow` TRUE (sound verdict + confirmed witness)

**Status:** SPIKE DONE + **SIGN-OFF RECEIVED (2026-09-06)** — user approved the CPAchecker-confirmation-gate design + "implement rank 2 now". **Slices A–D IMPLEMENTED + validated** (sentinel `prove_no_signed_overflow` + constant-fold probe + in-process CPAchecker gate + TRUE-first `overflow_strategy` + `VerdictOutcome.correctness` + write-gate; e2e: `spike_counter`/`ovf_counter`/`overflow_true_safe` → `true`+confirmed witness; all 4 wrong-TRUE → `false(no-overflow)`; 2243 workspace tests + 19 overflow e2e green; clippy `-D warnings` + fmt clean). **Slice E (full-scale FP=0/wrong-TRUE=0 A/B) ✅ PASSED** — 1213 tasks: wrong-TRUE=0, false-alarm=0, +63 sound CPAchecker-confirmed TRUEs, FALSE recall preserved (213), 4 crashes proven pre-existing. Work is UNCOMMITTED on the branch (commit + merge on request).
**Branch:** `reason1/rank2-nooverflow-true` off `auto/loop-20260816` @ `1a9c0630` (has 1b + 1c). Nothing pushed; `saf verify` UNTOUCHED. The throwaway sentinel + `prove-no-overflow` dev subcommand are on the branch working tree (uncommitted, measurement-only).
**Track:** TRUE-side ("Reason 1"). Sequence `1a✅ → 1b✅ → 1c✅ → 2 (this) → 3 (unreach-TRUE)`.
**Research notes (laptop):** memory `saf-reason1-rank2-nooverflow`; `investigation/reason1-true-side-roadmap.md` §4/§5; `plans/207` (1b emitter), `plans/208` (1c absint).

---

## 0. Goal
A **sound `no-overflow` TRUE verdict** in `saf verify` (a TRUE arm in `overflow_strategy` via `strategy_for`), backed by the sound interval absint + a **hardened verdict gate** + a **CPAchecker-confirmed `invariant_set` correctness witness** (+2). **Keep FP=0 / wrong-TRUE=0.**

## 1. Spike results (measured on VM cd-vm-15, real CPAchecker-4.2.2 native)

### P1 — pool
**5367** no-overflow-TRUE tasks (LP64 3562 / ILP32 1805). **Juliet_Test 3078 (57%)** — heavily duplicated CWE190 "good" variants → raw-count huge, dedup-weight small. Large **sound-abstain** buckets: concurrency ~550 (pthread-wmm/weaver/pthread*), nonlinear ~470 (nla-digbench*), recursion ~80, floats/arrays. The distinct interval-provable *non-Juliet* set is small. Property = `CHECK( init(main()), LTL(G ! overflow) )`.

### P3 — GO/NO-GO = **GO** (CPAchecker validates no-overflow correctness witnesses)
CPAchecker config `correctness-witness-validation--overflow.properties` (purpose: "checks correctness witnesses for overflows"; sets its own `specification/overflow.spc`). Measured: a counted-loop witness (`0<=i&&i<=1000000`) → `Verification result: TRUE`; an **empty** invariant_set → **also TRUE** (CPAchecker's own predicate/k-induction re-proves ⇒ witness confirmation is CHEAP for the tractable subset). **Negative controls pass:** `int b=a+1` (unconstrained) → `FALSE. Property violation (integer overflow line 4)`; unbounded counter → `UNKNOWN`; constant-fold `INT_MAX+1` (`AdditionIntMax`) → `FALSE (integer overflow line 332)`. It does **NOT** rubber-stamp. Ops notes: `scripts/validate_correctness_witness.sh` currently hard-codes the GENERIC (unreach) config → for no-overflow it must use the `--overflow` config + `--spec no-overflow.prp`; witnesslint is not at `/opt/sv-witnesses` (Stage-1 → LINT_WARN; CPAchecker is the authoritative gate); witnesses MUST be block-style YAML (inline-flow broke the parser).

### P2 — sentinel prove-rate
`prove_no_signed_overflow` (added to `absint/checker.rs`, re-exported at `saf_analysis::absint`): fail-closed inversion of `check_integer_overflow_with_result` — proves every reachable signed Add/Sub/Mul in-bounds against **its own result-type width** (`inst.result_type → AirType::Integer{bits}`, NOT `Interval::bits()` — the `DEFAULT_BITS=64` trap; unit-tested), abstains on any TOP/absent operand, non-converged fixpoint, `shl`/`sdiv`/`srem`, indirect call, or missing width. Dev subcommand `saf prove-no-overflow` adds the OpenMP-source + reachable-thread gates. 6/6 unit tests green (incl. the width-trap soundness test).

Over a **1213-task stratified sample** (835 TRUE / 378 FALSE, 15–20/category): **prove-rate 115/835 = 13.8%**, concentrated in `loop-simple` (6/6), `loop-acceleration` (7/8), `loop-new` (4/4), `loop-lit` (8/15), simple arrays, and `float-benchs` (15/15, vacuous — no integer arithmetic). **Juliet ≈ 0** (good variants abstain — their overflow guards are relational, beyond the interval domain). ⇒ **pool-weighted prove-rate ≪ 13.8%**; dedup-weighted prize is single-digit-to-low-tens, exactly as the roadmap predicted. Abstain reasons (TRUE tasks): top/absent operand 309, threads 185, may-overflow 154, shl/sdiv/srem 27, not-converged 23, indirect-call 15.

### ⭐ P2 wrong-TRUE audit — **4 wrong-TRUE found (the pivotal soundness finding)**
The sentinel proved **4 FALSE (overflowing) tasks** — a soundness violation. Root-caused into **two classes**:
1. **Compile-time constant overflow (3):** `AdditionIntMax` (`int x=(2147483647+1)-23`), `Multiplication-2` (`65536*32768`), `NoConversion` (`long x=2147483647+1`). clang **constant-folds** the overflowing constant expression (emits `-Winteger-overflow`, which SAF's `-Wno-everything` compile suppresses) → **no `add`/`mul` instruction in the IR** → the sentinel sees no arithmetic → wrongly PROVEs. Confirmed by inspecting the IR (`main` = just `ret`).
2. **Deeper absint miss (1):** `openssl-simplified/s3_srvr_1a.cil.c` — real IR arithmetic (4 `add`, 3 `nsw`), no clang warning, yet the sentinel PROVEs a FALSE task ⇒ an absint under-approximation or reachability unsoundness (not constant-folding).

**Conclusion: the interval absint + sentinel ALONE is NOT wrong-TRUE=0-safe** — ≥2 unsoundness classes, and more may lurk at scale. A pure-absint sound-TRUE would risk −16/−32 (and SAF's soundness brand).

### ⭐ Design validation
CPAchecker's overflow config **REJECTS** the wrong-TRUE (`AdditionIntMax` + empty witness → `FALSE (integer overflow line 332)`), and by the negative controls it rejects real overflows generally. ⇒ **Gating the TRUE verdict on in-process CPAchecker confirmation makes wrong-TRUE=0 robust regardless of absint unsoundness.**

### Confirmable yield (CPAchecker-confirm-rate on the 115 PROVE-TRUE + reject-check on the 4 wrong-TRUE)
Empty-witness CPAchecker overflow-validation, native, correct data model:
- **⭐ All 4 wrong-TRUE → `Verification result: FALSE`** (0/4 confirmed TRUE) ⇒ the in-process CPAchecker gate achieves **wrong-TRUE = 0** on the exact tasks the absint got wrong.
- **PROVE-TRUE confirm-rate: 98/115 = 85.2%** (11 UNKNOWN + 6 NONE = CPAchecker can't decide, mostly `float-benchs` and struct-heavy `array-industry-pattern` → score 0, still sound). SAF's loop invariants (vs the empty witness measured here) would only recover a few loopy UNKNOWNs; the float UNKNOWNs are a CPAchecker limit SAF can't help.
- Net (sample): sound-confirmed-TRUE ≈ 98/835 = **11.7%** of TRUE tasks; **pool-weighted ≪ that** (Juliet ≈ 0), **dedup-weighted single-digit-to-low-tens** — the roadmap's prediction holds.

## 2. Design (proposed — the crux; needs sign-off)

**TRUE arm = sentinel (cheap sound-leaning FILTER) → emit witness → in-process CPAchecker confirmation as the VERDICT gate → emit `true` iff CONFIRMED; else fall through to the existing UBSan-FALSE / unknown path.**

Rationale: SAF's identity is wrong-TRUE=0 on the competition path; the spike proved the absint alone can't guarantee it. CPAchecker (the SV-COMP reference overflow analysis) as the final gate makes wrong-TRUE=0 robust — "SAF proposes, CPAchecker disposes." This is the roadmap's *"verify post-solver + self-validate then emit"* guardrail in its strongest form. The sentinel's job is (a) cheaply decide *which* tasks to attempt (avoid CPAchecker on the ~86%+ it can't prove, preserving the FALSE path's budget), and (b) supply loop invariants that help CPAchecker close loopy cases.

**Plumbing (verified in the code-map):**
- `overflow_strategy` (`commands.rs:4941`): make it TRUE-first (mirror `no_data_race_strategy:5046`): read source (`std::fs::read_to_string(ctx.input)`); OpenMP + reachable-thread gates; `prove_no_signed_overflow(ctx.module)`; if PROVE → `build_interval_invariant_witness(ctx.module, &source, &meta)` (or an empty invariant_set for loop-free) → **run CPAchecker overflow-validation on that witness in-process** → if raw `Verification result: TRUE` → `VerdictOutcome{ verdict:"true", correctness:Some(witness) }`; else `unknown_outcome()`. Then the existing `ubsan_confirm` FALSE path unchanged.
- `VerdictOutcome` (`commands.rs:1192`): add `correctness: Option<saf_svcomp::InvariantSetWitness>`.
- Write-gate (`commands.rs:855`): add a `verdict.starts_with("true")` branch → serialize `correctness` via `to_yaml_string()` → write to `args.witness`.
- Verdict token: bare `"true"` (mirror `termination_verdict`). `meta.specification` on the verify path is already the raw no-overflow `.prp` (no hard-coded spec, unlike the dev `emit-correctness-witness`).
- Dead `analyze_no_overflow`/`analyze_no_data_race` stay disconnected.

Cost: CPAchecker runs only on the sentinel-PROVEd subset, once per task, within the 850s budget; fail-closed to `unknown` on timeout/absence/parse-fail.

## 3. Soundness guardrails (wrong-TRUE MUST stay 0)
- **Sentinel:** converged gate; width from `result_type` (never `Interval::bits()`); abstain on TOP/absent/`shl`/`sdiv`/`srem`/indirect-call/malformed/no-width.
- **Module-level OpenMP abstain** (`source_has_openmp`) + **reachable-thread abstain** (`reachable_spawns_threads` + `CallGraph::build`).
- **Constant-fold guard (recommended, belt-and-suspenders):** a `-Winteger-overflow` probe compile (WITHOUT `-Wno-everything`) → abstain on any integer-overflow warning. Catches the constant-fold class *before* the CPAchecker gate (cheaper).
- **⭐ In-process CPAchecker confirmation is the FINAL verdict gate:** emit `true` ONLY on raw `Verification result: TRUE`; FALSE/UNKNOWN/absent/timeout/parse-fail ⇒ `unknown`. Use the `--overflow` config + `--spec no-overflow.prp` + matching `--32/--64` + `witness.checkProgramHash=false`; block-style witness.
- **TRUE ⟂ FALSE authority:** never TRUE from UBSan/fuzz absence; keep the UBSan FALSE path.
- **Watchdog:** timeout/panic ⇒ `unknown` only.

## 4. Slices (TDD)
- **A ✅** — `VerdictOutcome.correctness: Option<InvariantSetWitness>` (commands.rs) + write-gate `"true"` branch. Done.
- **B ✅** — `prove_no_signed_overflow` (absint/checker.rs, re-exported at `saf_analysis::absint`; 6 unit tests incl. the width-trap) + `overflow_source_constant_folds` `-Winteger-overflow` probe. Done.
- **C ✅** — `cpachecker_confirms_no_overflow` + `resolve_cpachecker` (commands.rs): in-process `--overflow` config validation, fail-closed. Java 21 + CPAchecker confirmed present IN the dev container (no Dockerfile change needed). Done.
- **D ✅** — TRUE-first `overflow_strategy` via `try_overflow_true` (OpenMP/thread/constant-fold gates → sentinel → witness (invariants or empty) → CPAchecker confirm → `true`+correctness witness, else fall through to UBSan FALSE). e2e green: `spike_counter`/`ovf_counter`/`overflow_true_safe` → `true` + confirmed `invariant_set` witness; all 4 measured wrong-TRUE → `false(no-overflow)`; new `overflow_false_constfold.c` regression fixture. Done.
- **E (MERGE GATE) ✅ PASSED (2026-09-07)** — real `saf verify` (release, TRUE arm + CPAchecker gate + UBSan FALSE path) over the full 1213-task no-overflow corpus (835 true / 378 false). **Result matrix:** `true→true` **63** (NEW sound, CPAchecker-confirmed no-overflow TRUEs, +2 each), `true→unknown` 768 (sound abstain), `false→false(no-overflow)` **213** (FALSE recall, unchanged from baseline — the UBSan path is untouched), `false→unknown` 165, `true→TIMEOUT_rc139` 4. **⭐ wrong-TRUE = 0 · false-alarm = 0.** The 4 `rc139` are a native stack overflow on huge `ldv-linux-3.14-races` kernel CIL files — **confirmed PRE-EXISTING** (the baseline binary at `1a9c0630`, `try_overflow_true` absent, segfaults identically) ⇒ non-regression, scores 0 either way. Note the 63/115 vs the spike's 98/115 empty-witness estimate: real `saf verify` emits SAF's actual witness under `--timeout 240` + `-P 6` CPAchecker contention → some confirms time out; still net +63 sound TRUEs on the sample (pool-weighted modest, Juliet≈0, as predicted). **Merge to `auto/loop-20260816` only on user request.**

## 6. Results summary (2026-09-07)
Rank 2 is **implemented, validated, and merge-gate-clean** on `reason1/rank2-nooverflow-true` (UNCOMMITTED; commit on request). Soundness held end-to-end: FP=0 / wrong-TRUE=0 across 1213 tasks; the 4 measured spike wrong-TRUE all became correct `false(no-overflow)` (TRUE arm abstains, UBSan catches). Net gain on the sample: **+63 sound, CPAchecker-confirmed `no-overflow` TRUEs** (+2 each), FALSE recall preserved. Honest ceiling: single-digit-to-low-tens dedup-weighted (Juliet contributes ≈0), matching the roadmap. Same machinery (sentinel read-out + 1b witness + CPAchecker gate) now proven and reusable for **rank 3 (unreach-call TRUE)**.

## 5. Open questions for sign-off
1. **Approve the in-process-CPAchecker-confirmation gate** (vs relying on absint soundness)? Heavier per proven task, but makes wrong-TRUE=0 robust against the two measured unsoundness classes. **[RECOMMEND: yes]**
2. **Add the `-Winteger-overflow` constant-fold probe** (cheap defense-in-depth), or rely solely on the CPAchecker gate? **[RECOMMEND: add it]**
3. **Loop-free witness:** emit an empty `invariant_set` (relax `assemble`'s non-empty check) or a trivial function-entry invariant? **[RECOMMEND: empty — P3 confirmed it validates]**
4. **Scope/worth:** given the modest dedup-weighted prize + the CPAchecker-gate cost, ship rank 2 now, or measure confirmed-yield-at-scale first / prioritize rank 3 (unreach-TRUE, biggest single dedup slice, same machinery)? **[decide with the yield number]**
