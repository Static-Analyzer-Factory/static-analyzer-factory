# Plan 211 — SAF's own prover (TRUE-side capability)

**Status: SEED, not a design.** This document exists so the design session starts from
measured ground truth instead of re-deriving it, and does not repeat the dead ends
already walked. The design itself is deliberately left open.

Author of the measurements: session of 2026-09-10..12. Branch `lever1/cbmc-loop-free`.

---

## 1. The goal, and the constraint that shapes it

SAF is a bug-finder: it refutes programs by finding a concrete input and re-running the
program to confirm the violation. That is 21% of SV-COMP's available marks, and SAF
already captures ~38% of it. The other **79% of marks require proving programs
CORRECT**, and SAF captures essentially none of it.

**HARD CONSTRAINT (user decision, 2026-09-12, emphatic): never delegate a verdict to
CPAchecker or any other verifier.** SAF must remain a legitimately independent tool so
it can earn a *ranking*. SV-COMP excludes "meta verifiers" from rankings, defined
(community-approved 2023, restated in the 2026 report) as:

> *A meta verifier is a combination of at least two existing verification components
> such that each result produced by the combination can be computed by some of its
> components alone.*

A score we cannot place with is worth nothing. Do not propose delegation in any form —
not a "portfolio arm", not an "oracle mode", not a "fallback when SAF abstains". A
bundled verifier may be used ONLY to validate a witness SAF derived itself, never to
decide.

**Existing exposure to fix, not extend:** the rank-2/rank-3 TRUE path
(`try_unreach_true` / `try_overflow_true` in `crates/saf-cli/src/commands.rs`) emits
`true` only when an in-process CPAchecker gate confirms. Those results *are* computable
by CPAchecker alone. That is 12 of the current 151 weighted points. SAF is clearly not
meta overall, but this is the wrong side of the line and should be replaced by
SAF-native proving.

**Soundness is non-negotiable:** `wrong_true == 0` and `false_alarms == 0` are hard
project invariants. A wrong TRUE costs −32 and passes the per-cluster dedup cap in
FULL, so a single one erases several clusters' worth of gain. Every prover must be
fail-closed: prove or abstain, never guess.

---

## 2. Where the marks actually are (measured, full 55,690-task run)

Raw max 92,026 = TRUE-side 72,672 (79%) + FALSE-side 19,354 (21%).

| block | confirmed | max | rate |
|---|---|---|---|
| FALSE-side, all properties | 7,337 | 19,354 | 38% |
| TRUE-side verdict-only (termination, no-data-race) | 2,694 | 4,432 | 61% |
| TRUE-side witness-required (unreach-call, no-overflow) | 198 | 47,274 | 0.4% |
| TRUE-side valid-memsafety (verdict-only) | 0 | 20,966 | 0% |

**Score on the metric that matters — distinct (group, property) clusters solved, each
capped at 1 — is 151 / 398.** Always report weighted, not raw: this project has
repeatedly been burned by prizes that evaporated under dedup (raw counts are inflated
by thousands of near-duplicate Juliet files).

Note the pattern in that table: **where a TRUE verdict needs only a verdict, SAF scores
61%; where it needs a confirmed correctness witness, SAF scores ~0%.** And
valid-memsafety is verdict-only yet scores 0 because SAF has no memsafety TRUE path at
all.

---

## 3. Why the current prover proves nothing (measured — do not re-derive)

`saf prove-unreachable` over a 120-task sample stratified across 115 clusters:

| outcome | n |
|---|---|
| PROVE | **2** |
| ABSTAIN:error-reachable | **80** |
| ABSTAIN:threads | 17 |
| (no output) | 13 |
| ABSTAIN:module-too-large | 7 |
| ABSTAIN:not-converged | 1 |

Reproduce with `scripts/measure_prove_funnel.py`. **Steer by this histogram, not by the
verdict score** — the score moves too slowly to guide design.

**The wall is `error-reachable`, and it is architectural, not domain precision.** SAF's
abstract interpreter analyses each function independently starting from a ⊤ (unknown)
entry state. **92.5% of the 18,307 unreach-call TRUE tasks** hide `reach_error` inside
the canonical SV-COMP wrapper:

```c
void reach_error(void) { __assert_fail("0", ..., "reach_error"); }
void __VERIFIER_assert(int cond) { if(!(cond)) { ERROR: {reach_error();abort();} } return; }
...
__VERIFIER_assert(property());
```

Analysed from a ⊤ entry, `cond` is unconstrained, so `!(cond)` is feasible and the error
block is "reachable" — **for every such task, regardless of how easy `property()` is to
prove.** The analysis never relates the callee's parameter to the caller's argument.

### Dead ends already measured — do not repeat

- **Inlining the assert wrapper**: flipped **1 of 9** applicable tasks, and **0 of 4**
  in `hardness` (the largest cluster, 6,789 tasks). So the wrapper is *a* cause but not
  *the binding* one — after inlining, intervals still cannot discharge the guard.
- **The loop-free special case** (no loop invariant needed, so a complete SMT encoding
  decides it): only **3.6%** of unreach-call TRUE tasks are loop-free, spanning 7
  unsolved clusters, and 2 of those probed came back UNKNOWN ⇒ **+2..5 weighted max**.
- **Wiring the octagon domain**: it is a complete lattice with **no transfer function
  and zero callers**. L/XL effort, +2..4 weighted.
- **Raising cost guards**: `module-too-large` is 7/120 and `not-converged` is 1/120.
  Not the constraint.
- **`plans/206`** is referenced by older notes but **exists in NO git ref**. Its "~9%
  loop-free floor" is unverifiable. Do not scope from it.

---

## 4. What SAF already has (verify before trusting — several notes are stale)

- Interval abstract interpretation, converged-fixpoint based, in `crates/saf-analysis`.
  A 2026-09-06 fix (`plans/208`) made loop-head invariants non-trivial for pure-interval
  counted loops.
- `prove_unreachable` / `prove_no_signed_overflow` sentinels in
  `crates/saf-svcomp/src/property.rs`, with fail-closed guards
  (`UNREACH_TRUE_MAX_INSTS`, block-count, convergence).
- A YAML-2.0 `invariant_set` correctness-witness emitter (`correctness_witness.rs`,
  `correctness_driver.rs`) — **proven to work end-to-end**; 98 of 99 emitted witnesses
  validate CONFIRMED against real CPAchecker. The witness pipeline is NOT the blocker.
- Z3 (check whether unbounded-Int or bitvector at each use site — it matters for
  soundness over C semantics), CEGAR machinery, trace/disjunctive partitioning (already
  wired and ON by default — status quo, not an upgrade).
- **No k-induction anywhere.** No predicate abstraction in the verdict path. No shape
  analysis / SMG.
- A sound structural termination prover (`termination.rs`) — the model to imitate for a
  verdict-only prover: prove-or-abstain, no witness, fail-closed on anything unmodelled.

---

## 5. Open questions for the design session

1. Which property to target first? `valid-memsafety` TRUE is **verdict-only** (no
   witness machinery at all) and worth 20,966 raw — but a conservative prover's UNIQUE
   dedup-weighted prize was measured at only **+1 to +3** (89.5% of that block sits in
   clusters SAF already solves). `no-overflow` TRUE is a local property over arithmetic
   rather than a reachability question, and already has 34 weighted. Rank by weighted
   yield per unit of build, not by raw marks.
2. What is the minimum analysis that discharges the assert-wrapper pattern soundly —
   function inlining at the AIR level, call-string context sensitivity, a summary-based
   interprocedural fixpoint, or something else?
3. What invariant strength is actually needed beyond intervals, measured on the tasks
   that remain after (2)? Do not assume; measure with the funnel instrument.
4. How does the prover stay fail-closed against the frontend being lossy? Plan 201's
   lesson: the AIR is incomplete (global ctors/dtors, computed goto), and a prover that
   trusts it emits wrong TRUEs.
5. Concurrency (`ABSTAIN:threads`, 17/120) — in scope or explicitly out?

---

## 6. Working rules

- All builds and evals on the VM `ubuntu@cd-vm-15-ai-vm`, repo `~/static-analyzer-factory`,
  via `docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c "..."`. **Never on
  the laptop.**
- TDD. `make lint` (clippy `-D warnings` + rustfmt) must pass. Note `cargo clippy
  --all-targets` fails on pre-existing errors in `saf-analysis` test targets — the
  repo's lint scope is `--workspace` without `--all-targets`.
- Measure before building, and prefer one targeted measurement to a broad sweep. The
  decisive facts in this document came from single scripts over the per-task dump, not
  from large agent fan-outs.
- A full-reservoir eval is ~20 h at `--jobs 8`; scope measurements to the affected
  population instead wherever the unaffected remainder is unchanged *by construction*.
