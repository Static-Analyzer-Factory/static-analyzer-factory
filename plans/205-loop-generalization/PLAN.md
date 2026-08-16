# Plan 205 — Loop v2: from pool-recall to novel-solving

**Date:** 2026-08-17  **Branch:** `svcomp`  **Status:** proposed, awaiting sign-off
**Consolidates six investigations** (all in this dir): `forensics.md`, `observability.md`,
`redesign.md`, `svcomp-compliance.md`, `reach-mechanisms.md`, `conc-term-mechanisms.md`.

## The problem (measured)
The autonomous loop raises SAF's TRAIN score but not the svcomp26 HOLDOUT: holdout confirmed
`7→7→8 / 1770` (0.45%), **unreach-call 0% on genuinely-new tasks**. SAF's score is recall on the
repeated Juliet/generated pool (memorization), not solving power on novel problems. The loop is
optimizing the wrong target and ~40% of its arms feed structurally-unscoreable families.

## Strategy — fix the incentive, then give it the right levers
Two ordered movements. Order matters: **without Movement 1, even a perfect fuzzing lever gets reverted**
(it scores ~0 on the Juliet-dominated sample and re-derives itself every arm).

- **Movement 1 — fix WHAT the loop optimizes + how we see it (infrastructure; I implement).**
- **Movement 2 — give it the RIGHT levers (mechanism-derived, SV-COMP-compliant; the loop builds them).**

---

## Movement 1 — Loop infrastructure

### 1a. Generalization gate  (`redesign.md`)  — the core fix
- `svcomp_split.py`: additively emit a held-IN **`val.jsonl`** reasoning set carved from TRAIN —
  structurally EXCLUDE generator/Juliet clusters (category-root denylist + a cluster-size backstop
  `>200 tasks/property` that auto-catches unlabeled generators; fail-safe = exclude-when-unsure). `val ⊂
  train` → worker-readable; the svcomp26 holdout stays read-forbidden. Add `cluster`/`generator` row fields.
- `svcomp_split_eval.py`: additive `--group-weight` → `confirmed_score_weighted` (cap **1 pt/cluster**) +
  per-property `confirmed_weighted`/`confirmed_clusters`. Flag-gated → existing runs byte-identical.
- `gates.py`/`verify_arm.py`: new pure **`decide_v2`** + helpers (`generalization_delta`,
  `pool_guard_delta_w`, `novel_confirm_gain`), unit-tested. New decision ladder:
  - `gen_delta_w > 0 AND pool_delta_w ≥ 0` → **KEEP** (a real reasoning-set gain; deduped pool not regressed)
  - pool-only Juliet gain → **KEEP_POOL** (banked for the honest points, **no stall-reset** → memorization levers eventually park)
  - capability first novel task solved → **ACCUMULATE_PLUS** (preserved + priority)
  - else → ACCUMULATE / REVERT as today
  - (crosscut) plus the existing no-family-regressed rule.
- `supervisor.sh`: per-arm eval on `val` (before/after) + deduped TRAIN guard; make `maybe_heldout_check`
  ACTIONABLE (park a lever whose train gains don't reproduce on the svcomp26 holdout over 2 consecutive
  checks; boost one that does); stall-reset only on generalization gains; within-family priority by earned
  `gen_credit`.
- Staged behind **`LOOP_GEN_MODE`** for A/B and reversibility.

### 1b. Observability  (`observability.md`)  — captures data the loop already throws away
- Per-arm `record.json` + append-only **`arms.jsonl`** spine (lever, scope, per-property before/after/Δ,
  decision, turns/cost/retries/max_turns from the stream-json `result` event, `git diff --numstat`).
  Emitted after the decision, BEFORE checkpoint mutates branches (so REVERTs still record the discarded
  diff = the wasted-work signal).
- **§5a task-flip capture** (highest-value): the scorer's `--per-task` dump, diffed before/after, records
  exactly which tasks went `unknown→confirmed` (new solve) vs `re-confirmed` (memorized shape) vs traded
  for an FP — the concrete memorization-vs-generalization instrument.
- Journal enrichment: per-property Δ, holdout trend + train↔holdout gap, per-lever ROI ($/confirmed-point).
- `report.sh` dashboard rebuilt around `arms.jsonl`: headline GENERALIZATION block, lever-ROI table,
  waste/re-derivation, capability milestones.

### 1c. Bug fixes  (`forensics.md`)
- **BUG-2 (capability re-derivation):** reverted capability arms currently discard ~470 lines of
  `concurrency.rs` and re-derive it every arm. Fix: capability-mode levers ACCUMULATE their source onto a
  **dedicated capability branch** (isolated from the score-gated integration branch), OR persist the
  reverted diff in `.loop-state/arm-N/` and prime the next same-lever arm's prompt with it. (Aligns with
  redesign's `ACCUMULATE_PLUS`.)
- **BUG-3 (orphaned containers):** post-arm sweep of `com.docker.compose.oneoff=True` containers older
  than the arm start in `revert_arm`/`keep_arm`; instruct workers to `docker compose run --rm` with an
  outer `timeout` + process-group kill. *(Two current orphans need a one-time manual `docker rm -f` —
  awaiting operator, see stop-loss note.)*
- **BUG-4 (accumulate-forever):** park a lever after K consecutive ACCUMULATEs with 0 KEEPs (and a plateaued
  KEEP lever) — its own budget. *(Interim: `race-confirmer` already parked.)*
- **BUG-5 (efficiency invisible):** sum turns/cost/wall/resume-count per arm into the journal (folds into 1b).
- **BUG-1 (done as stop-loss):** `conc-shim-m1` parked (mis-scoped, injected −16 FP). Real fix = the
  concurrency pipeline in Movement 2 + proper concurrency scoring.

### 1d. Compliance contract  (`svcomp-compliance.md`)  — bake the fail-closed rules in
SV-COMP **allows fuzzing/native execution** (VeriAbs/VeriFuzz precedent); SAF's replay+witness model is the
blessed pattern. Compliance is conditional. Encode these as a shared **confirmer contract** in
`arm_prompt.md` + a `confirmer_contract.md` every FALSE-emitting lever must obey:
- **R1 match event to property** — confirm a FALSE ONLY on the property's exact violation event
  (`reach_error`/`__assert_fail` for unreach; in-scope signed-int-**operation** overflow for no-overflow;
  ASan mem-error for memsafety; TSan race for no-data-race). The benchmarks are NOT UB-free — abstain on any
  other trap. **No catch-all sanitizer→FALSE.**
- **R2 narrow the overflow oracle** — `-fsanitize=signed-integer-overflow` only; ignore conversion/shift/pointer traps.
- **R3 declared data model** — compile/run each task under its ILP32/LP64 setting (`-m32` for ILP32); abstain if unavailable. (Same 64-bit gap as R6.)
- **R4 honor `__VERIFIER_assume`** (hard path filter); **R5 in-model nondet inputs** (right width/signedness, abstain if ambiguous); **R6 deterministic reproduction** (inject the witness's exact values, re-trigger before emitting).
- **R7 concurrency** — needs an explicit forced schedule + a **GraphML 1.0** witness (YAML 2.0 scores 0 for concurrency); abstain on relaxed-memory/OpenMP.
- Packaging: **bundle SAF's own toolchain** (gcc/clang + 32-bit multilib) in the submission archive.

---

## Movement 2 — New lever menu (`levers.tsv`)

Replace the current menu with the mechanism-derived, ROI-ordered levers (specs + build-out in
`reach-mechanisms.md` §6 and `conc-term-mechanisms.md` §5). The **loop builds these** as capability arms,
now correctly measured by the generalization gate. Shared infra built once: an **AIR→SSA path encoder +
nondet-model extractor** (reused by all solve-family levers).

| rank | lever | family | new? | why |
|---|---|---|---|---|
| 1 | **`termination-recall`** → linear ranking functions (Podelski–Rybalchenko/Farkas+Motzkin+Z3) | termination | sharpen | pure recall, **no witness, no −16/−32 risk**, big pool, reuses CFG+Z3; ~16%→50%+ |
| 2 | **`fuzz-mut-nondet`** (L1) | unreach-call | **NEW** | the #1 missing lever; blind mutation over a nondet byte-cursor shim under the existing replay; **self-confirming, wrong-FALSE impossible**; directly attacks 0% novel recall |
| 3 | **`air-slicing`** (L3, sharpen) | unreach-call | sharpen | PDG backward slice from `reach_error`; **add `__VERIFIER_assume` secondary criterion** (correctness fix); multiplies every solve/fuzz lever |
| 4 | **`fuzz-covguided`** (L2) | unreach-call | **NEW** | SanitizerCoverage map + AFLFast energy + optional AFLGo direction → deep targets |
| 5 | **`bmc-fixed-k`** (L4, ex-`symbolic-oracle`) | unreach-call | build out | unwind→guarded SSA→Z3→nondet model→replay; reaches guarded/counting targets fuzzing can't; builds the shared encoder |
| 6 | **concurrency pipeline** `conc-seq-m1`→`conc-replay-confirm`→`graphml-witness` | unreach-call | rescope | Lazy-CSeq directed finder (solver assigns the schedule) → forced-interleaving replay → GraphML 1.0; makes the parked concurrency levers actually score |
| 7 | `se-interp` (L5), `bmc-incremental` (L6), `fuzz-concolic-z3` (L7), `harness-havoc` (L8), `portfolio-select` (L9) | unreach-call | NEW/sharpen | follow-ons; portfolio routes fuzz-vs-BMC-vs-SE by cheap AIR Booleans |
| 8 | data-race pipeline `race-find`→`race-schedule`→`race-confirmer` | no-data-race | rescope | lockset+MHP finder → relaxed-atomic rendezvous → TSan confirm; LAST (smaller pool) |

**Do NOT build** (prove-side, no reaching input): Craig interpolation/predicate discovery, k-induction
inductive step, IC3/PDR frame maintenance, correctness-witness generation, Büchi/trace-abstraction.

**Recommended build sequence:** `termination-recall` C1 first (highest ROI, lowest risk) ∥ `air-slicing` L3
→ `fuzz-mut-nondet` L1 → `fuzz-covguided` L2 → `bmc-fixed-k` L4 (shared encoder) → concurrency pipeline →
L5–L9 → data-race pipeline.

---

## Rollout
1. Implement Movement 1 (TDD; 10 unit + smoke as today) on `svcomp`, behind `LOOP_GEN_MODE`.
2. Swap in the Movement 2 `levers.tsv` menu.
3. **Fresh, clean campaign** (per the always-start-clean rule) with `LOOP_GEN_MODE=on`; monitor the holdout
   trajectory + task-flips on the new dashboard. Keep the current `auto/loop-20260815` gains as a baseline
   to A/B against.
4. Success metric shifts from "confirmed FALSEs on a Juliet sample" to **distinct reasoning-family
   confirmations on `val` that also reproduce on the frozen svcomp26 holdout** — i.e. the holdout finally moves.

## Preserved invariants (all six reports verified)
Soundness (FALSE only on concrete confirmation / must-reach), determinism (byte-identical), both anti-cheat
gates (immutable-freeze + holdout-not-read), never-push/never-merge-to-svcomp, REQ-IP-001 (independent
reimplementation, mechanisms not code), the −32/−16/0 abstain-beats-guess doctrine.
