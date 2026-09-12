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
by CPAchecker alone. That is 12 of the 151 weighted points SAF scored under the 2026 rules (11 of the corrected 110 — see Movement 0A). SAF is clearly not
meta overall, but this is the wrong side of the line and should be replaced by
SAF-native proving.

**Soundness is non-negotiable:** `wrong_true == 0` and `false_alarms == 0` are hard
project invariants. A wrong TRUE costs −32 and passes the per-cluster dedup cap in
FULL, so a single one erases several clusters' worth of gain. Every prover must be
fail-closed: prove or abstain, never guess.

---

## 2. Where the marks actually are (measured, full 55,690-task run)

> ⚠️ **PARTLY SUPERSEDED 2026-09-12 — see §5.1(d).** "TRUE-side verdict-only
> (termination, no-data-race)" is wrong for SV-COMP 2027: `C.termination.all` now requires
> a 2.1 correctness witness and SAF emits none, so termination's **36 weighted points do
> not hold**. (`no-data-race` is fine — still "not supported" in 2027, so verdict-only.)
> The headline **151 is inflated** — MEASURED 2026-09-12: the corrected score is
> **110** (TRUE-side rule alone gives 115; the extra −5 is the `no-data-race`
> violation cell this note did not anticipate; version-aware 107).
> `no-data-race` is NOT fine: its *correctness* column is "not supported" as stated,
> but its *violation* column is "2.2", i.e. required, and the harness exempted it. `valid-memsafety` is the one block that is verdict-only in every
> suffix, including Concurrency.

Raw max 92,026 = TRUE-side 72,672 (79%) + FALSE-side 19,354 (21%).

| block | confirmed | max | rate |
|---|---|---|---|
| FALSE-side, all properties | 7,337 | 19,354 | 38% |
| TRUE-side verdict-only (termination, no-data-race) | 2,694 | 4,432 | 61% |
| TRUE-side witness-required (unreach-call, no-overflow) | 198 | 47,274 | 0.4% |
| TRUE-side valid-memsafety (verdict-only) | 0 | 20,966 | 0% |

**Score on the metric that matters — distinct (group, property) clusters solved, each
capped at 1 — is 110 / 398** (it was 151 under the 2026 rules; see Movement 0A).
Note 398 itself is understated: it omits `valid-memcleanup`, which the manifest excludes.** Always report weighted, not raw: this project has
repeatedly been burned by prizes that evaporated under dedup (raw counts are inflated
by thousands of near-duplicate Juliet files).

Note the pattern in that table: **where a TRUE verdict needs only a verdict, SAF scores
61%; where it needs a confirmed correctness witness, SAF scores ~0%.** And
valid-memsafety is verdict-only yet scores 0 because SAF has no memsafety TRUE path at
all.

---

## 3. Why the current prover proves nothing (measured — do not re-derive)

> ⚠️ **SUPERSEDED 2026-09-12 — see §5.1(a) and §5.1(b).** The histogram below was taken
> with a broken instrument (it analysed the `.c` when the task definition names the `.i`,
> for 13,419 of 36,336 TRUE tasks), and the diagnosis under it is wrong: the
> `error-reachable` wall is a ~3-line bug in `Interval::refine_eq_false`, not missing
> interprocedural context — SAF cannot prove a three-line, single-function, call-free
> program. That also invalidates the "inlining flipped 1 of 9" dead end below.
> **Do not scope work from this section; re-measure after Movement 1.**

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
  Its universe gate (`termination.rs:116-209`) is the reusable asset; §5.4 lifts it.

Added by the design session (verified in code, contradicting notes that said otherwise):

- **A summary-based interprocedural fixpoint DOES exist** —
  `absint/interprocedural.rs` (2,779 lines): `FunctionSummary` with join/widen, recursive
  SCC handling, and `refine_call_sites`, which binds caller argument intervals to callee
  parameters and re-analyses via `solve_function_with_params` — **then keeps only the
  return interval and discards every internal state**. No prover consumes it; both
  sentinels call the plain per-function `solve_abstract_interp`, which seeds every
  parameter `Interval::make_top(64)` (`fixpoint.rs:336-338`).
- **`Interval` carries a bit-width but no signedness** (`{lo: i128, hi: i128, bits, bottom}`),
  which is the root of the wrong-TRUE class in §5.1(c).
- **`prove_no_signed_overflow` checks every integer `Add`/`Sub`/`Mul`**, ignoring the
  `nsw` flag the frontend already preserves (`Instruction::has_no_signed_wrap`), and
  scans **every defined function including unreachable ones**
  (`checker.rs:1156-1160`).

---

## 5. The design

Written 2026-09-12 by the design session this document seeded. Everything below is
grounded in measurements taken during that session; each is reproducible with a named
script on the VM. **Sections 2-4 above contain numbers this session overturned** — read
§5.1 before using any of them.

**The bottom line.** The first move is not a prover. SAF's reported score is measured
against the wrong rulebook (§5.1(d)), and its own sentinels are not yet sound enough to
stand without the CPAchecker gate the mandate requires us to remove (§5.1(c)). So:

| | movement | plan | size | weighted effect | blocked on |
|---|---|---|---|---|---|
| 0 | ~~re-derive the scoreboard; emit a 2.1 termination witness~~ | [`plans/212`](212-movement0-scoreboard-2027.md) | S | **DONE 2026-09-13: 151 → 110 → 137** | — |
| 1 | three domain fixes + universe gate, then cut the delegation | [`plans/213`](213-movement1-soundness-and-de-delegation.md) | S/M | ~0, but buys legitimacy and a trustworthy baseline | — (0 landed) |
| 2 | the Anchored-Object `valid-memsafety` **+ `valid-memcleanup`** prover | [`plans/214`](214-movement2-anchored-memsafety.md) | M | **+8 to +16**, plus an unmeasured memcleanup slice | M1 (`universe.rs`) |
| 3 | `no-overflow` precision, re-scoped after M1 re-baselines the funnel | [`plans/215`](215-movement3-nooverflow-precision.md) | M | +5 to +10, and now with a MEASURED low prior | M1 |
| **4** | **FALSE-side witness upgrade: `no-data-race` GraphML→2.2, Concurrency version floor** | [`plans/216`](216-movement4-false-witness-2.2.md) | **S** | **+8, measured** | **nothing** |
| **5** | **submission readiness — archive, bench-defs MR, CI** | [`plans/217`](217-movement5-submission-readiness.md) | S/M | **0 — and without it every other movement is worth 0** | **nothing** |

> ### RE-RANKED 2026-09-13, once the deadlines were verified
>
> Registration is **2026-10-08** and tool submission **2026-10-20**
> (`svcomp-2027-deadlines-verified`; `dates.php` 404s, the dates live on `index.php`).
> That is 25 and 37 days. The original 1 → 2 → 3 ordering puts a **zero-point** movement
> first and blocks both offensive ones behind it, so on that ordering nothing that scores
> AND nothing that submits happens before the gate.
>
> Movements 4 and 5 were added from Movement 0's findings and are both **unblocked**:
>
> * **5 first, or in parallel.** It is the only item with a hard external deadline that
>   nothing can move. A submitted 137 beats an unsubmitted 145.
> * **4 next.** +8 measured, on verdicts SAF already emits correctly — the work is witness
>   FORMAT, not proving. Best points-per-day on the board.
> * **1, 2, 3 after**, in that order, as post-gate work. 1 is still not optional: it
>   discharges the no-delegation mandate, and 2 and 3 both need it. But it buys ~0 points,
>   so it should not consume the pre-deadline window.
>
> One method finding worth applying to 2 and 3: Movement 0B hit the EXACT ceiling of
> CPAchecker's own producer+validator pipeline, and that ceiling was discoverable in ~45
> minutes of probing before ~750 lines of Rust. **Probe the oracle ceiling before building
> a prover for a cluster set**, and let it replace the estimate. Folded into both plans.

Movements 0 and 1 are not optional and not reorderable relative to each other: 0 because
every A/B until it landed was steering on a number 41 too high (MEASURED; the estimate
here was ~36), 1 because cutting the delegation
before the soundness fixes costs roughly **−160 weighted against a score of 137**
(the figure was computed against the pre-2027 scoreboard's 151; the sign and the
conclusion are unchanged, only the denominator).

**This document is the strategy and the evidence; each movement has its own executable
plan.** §5.3-§5.6 below are the summaries those four plans expand.

Measurement scripts from this session live in `scripts/p211_*.py` and are run **from the
repo root** (they read `lever1-pertask.jsonl` by relative path):
`p211_headroom.py` (2027-priced cluster headroom), `p211_probe_unsolved.py` (funnel
restricted to the clusters where points are available), `p211_probe_soundness.py` /
`p211_sweep_soundness.py` (wrong-PROVE census, sampled / full population),
`p211_probe_confirm.py` (does the validator confirm what SAF proves?),
`p211_probe_context.py` (prove-rate × program structure),
`p211_anchor_prototype.py` / `p211_anchor_falsescan.py` (the Movement 2 spike).

### 5.1 What this session overturned

Five of this document's load-bearing facts are wrong, and two of the instruments that
produced them were broken. They come first because several were the basis for the work
this plan was about to scope.

**(a) The `error-reachable` wall is a ~3-line domain bug, NOT missing interprocedural
context.** §3 says the wall "is architectural, not domain precision — the analysis never
relates the callee's parameter to the caller's argument." Refuted by construction:

```c
extern void abort(void); void reach_error(){}
int main(){ int x = 0; if (!(x == 0)) { reach_error(); abort(); } return 0; }
```
`saf prove-unreachable --data-model ILP32` ⇒ **`ABSTAIN:error-reachable`**. One function,
no callee, no parameter, no loop, no context of any kind. Controlled sweep on the same
program: `if (x != 0)` ⇒ ABSTAIN; `if (x > 0)` ⇒ PROVE; `if (x == 1)` ⇒ PROVE.

Root cause, `crates/saf-analysis/src/absint/interval.rs` `refine_eq_false` (~line 859):
both refinement cases are guarded by `self.lo < self.hi`, so a singleton refined against
an EQUAL singleton — precisely "x is known to be c, and this edge asserts x != c" —
falls through to `self.clone()` instead of ⊥. `refine_branch_condition`
(`transfer.rs:1500-1507`) routes `(ICmpEq,false)` and `(ICmpNe,true)` there, and
`(ICmpEq,false)` is exactly what clang emits for the canonical `if (!(cond)) ERROR`.

This invalidates the "inlining the assert wrapper flipped only 1 of 9" dead end in §3:
after inlining, the guard is still `x != c`, so the same bug still blocks it. **Do not
scope an interprocedural prover on the strength of that measurement.**

**(b) The funnel instrument was measuring the wrong programs.**
`scripts/measure_prove_funnel.py` resolved each task's source as "first of `.c`/`.i` that
exists", but **13,419 of 36,336 expected-TRUE tasks ship both and the `.yml` names the
`.i`**. Where the `.c` needs a local header it did not compile at all — that was
essentially the entire `(no output)` bucket (probed 4 directly, 4/4 were clang errors:
`loop-new`/`loop-lit`/`loop-invgen` ship their own `assert.h`, which collides with SAF's
injected `-include sv-comp-stubs.h`). Fixed (`resolve_source`), and the instrument now
also takes `--property no-overflow` and prints a **per-cluster** table. Corrected
baselines, 150 tasks each, stratified over every cluster holding TRUE tasks:

| | unreach-call | no-overflow |
|---|---|---|
| PROVE | 2/150 (1.3%) | **18/150 (12.0%)** |
| clusters where PROVE dominates | 2 / 104 | **9 / 95** |
| dominant wall | `error-reachable` 105 (70%) | `top-or-bottom-operand` 71 (47%) |
| `threads` | 21 (14%) | 25 (17%) |
| `(no output)` | 5 (3%) — was 11% | 3 (2%) |

**(c) SAF's own sentinels are not yet sound, and only the CPAchecker gate hides it.**
Swept over the COMPLETE ground-truth-FALSE population (`sweep_soundness.py`):
unreach-call **3 wrong PROVEs / 4,324**; no-overflow **4 / 3,733**. Because penalties
pass the per-cluster dedup cap in full, cutting the delegation today would cost roughly
**−160 weighted against a score of 110** (computed against the pre-2027 151).** The 7 reduce to two classes:
1. *Unsound ⊥ from the interval domain* — `refine_branch_condition`
   (`transfer.rs:1510-1520`) applies **signed** refinement to **unsigned** comparisons
   with no `lo >= 0` guard, while its evaluation counterpart `Interval::icmp_ult`
   (`interval.rs` ~734) has exactly that guard. Witnesses:
   `bitvector-regression/implicitunsignedconversion-1` (`unsigned 1 < (int)-1` is TRUE in
   C; signed refinement makes the branch ⊥), `bitvector-regression/signextension-1`
   (sign- vs zero-extension of narrow types), `loop-simple/deep-nested`.
   `openssl-simplified/s3_srvr_1a.cil` is the same class by hypothesis — *not yet
   verified*, verify before scoping.
2. *IR-invisible C semantics* — clang constant-folds `(2147483647 + 1) - 23`, so no
   arithmetic instruction survives. **Production already guards this**
   (`overflow_source_constant_folds`, clang `-Winteger-overflow`, fail-closed); the
   standalone CLI does not, so those 3 are an instrument artifact.

**(d) ⚠️ THE BIGGEST ONE: SAF's 36 weighted termination points are scored under a rule
that does not hold for SV-COMP 2027.** The requirement is set per BASE CATEGORY
`C.<property>.<suffix>`, where the suffix is the benchmark family (= the `.set` file).
From the published 2027 rules page (`sv-comp.sosy-lab.org/2027/rules.php`, read
2026-09-12 — this is the authoritative reference, not the category-structure validator
lists, which over-state the requirement):

| base category | correctness witness | TRUE on verdict alone? |
|---|---|---|
| `C.unreach-call.{Arrays,Heap}` | **not supported** | **YES** |
| `C.unreach-call.Floats` | 2.0+ (demo mode) | **YES** |
| `C.unreach-call.Concurrency` | 2.1 or higher | no — **newly required in 2027** |
| `C.unreach-call.<all others>` | 2.0 or higher | no |
| `C.valid-memsafety.<any suffix>` | **not supported** | **YES** |
| `C.valid-memcleanup.all` | **not supported** | **YES** |
| `C.no-overflow.Concurrency` | 2.1 or higher | no — **newly required in 2027** |
| `C.no-overflow.<all others>` | 2.0 or higher | no |
| `C.no-data-race.all` | **not supported** | **YES** |
| **`C.termination.all`** | **2.1 or higher** | **no — NEWLY REQUIRED in 2027** |

SAF emits **no correctness witness at all** on the termination path —
`commands.rs:5529-5535` returns `correctness: None`. Under 2027 rules its **36 weighted
points score 0**. `scripts/svcomp_split_eval.py:65` `TRUE_WITNESS_NOT_REQUIRED =
{termination, valid-memsafety, valid-memcleanup, no-data-race}` encodes the 2026 rule, so
**every score this project has reported — including the loop campaign's KEEP metric — is
inflated by termination's 36.** (no-data-race is fine: still "not supported" in 2027.)

Two more things from the same page worth designing around: the correctness-witness
validator budget was **cut from 900 s to 300 s**, explicitly "to motivate verifiers to
produce witnesses whose validation is substantially easier than solving the original
verification task"; and an unconfirmed-but-correct TRUE scores **0, never −32** — so a
weak witness costs points but is not a soundness hazard.

**(e) And a hard ceiling on the witness-required side.** Of the 69 unsolved no-overflow
TRUE clusters, SAF's own sentinel already PROVEs a task in **5**
(`array-industry-pattern`, `array-lopstr16`, `array-multidimensional`, `float-benchs`,
`loop-invgen`). Emitting SAF's witness for exactly those tasks and validating:
**0 of 5 CONFIRMED** (two lint clean; cpachecker returns UNKNOWN / FALSE / nothing). So
"cut the gate and bank +5" is refuted — those verdicts would be RAW-only.

The insight that reconciles this with the earlier "98 of 99 witnesses CONFIRMED": those
99 TRUEs were emitted *only because* the in-process CPAchecker gate had already said
TRUE, so they are selected for confirmability. **The gate makes SAF's confirmed-TRUE set
equal to CPAchecker's provable set** — the meta-verifier problem and the score ceiling
are the same fact seen from two sides. Caveat: only the bundled CPAchecker 4.2.2 with one
config was tested, over 5 clusters; SV-COMP counts a result if ANY validator confirms.

### 5.2 The targets, priced for SV-COMP 2027

Unsolved TRUE clusters (score 0 today, ≥1 ground-truth-TRUE task), after excluding
`/todo` directories that no `.set` file references (180 of 198 survive) and classifying
each cluster by the 2027 base category its tasks actually live in (`headroom_final.py`):

| property | unsolved clusters | **VERDICT-ONLY** | witness required |
|---|---:|---:|---:|
| no-overflow | 61 | 0 | 61 |
| unreach-call | 60 | **8** | 52 |
| **valid-memsafety** | **43** | **43** | 0 |
| termination | 12 | 0 | 12 |
| no-data-race | 4 | **4** | 0 |
| **TOTAL** | **180** | **55** | **125** |

The 8 verdict-only `unreach-call` clusters are the 2027 carve-out: `array-cav19`,
`array-crafted`, `array-lopstr16`, `array-multidimensional`, `array-patterns`,
`array-programs` (Arrays), `floats-cbmc-regression` (Floats), `heap-data` (Heap).

**Correction to a number this session first reported:** classifying by SV-COMP's own
`Concurrency.set` rather than by grepping sources for `pthread_create` gives **50
concurrent / 130 sequential**, not 95/103 — the grep swept in `Sequentialized.set` tasks
and ldv drivers that merely mention pthread in text but are scored in sequential
categories. Per property (conc/seq): no-overflow 16/45, unreach-call 14/46,
valid-memsafety 16/27, termination 0/12, no-data-race 4/0. Concurrency is a quarter of
the headroom, not half.

Three facts dominate the ranking, and none is about prover strength:

1. **Defending beats attacking by ~3x.** Termination's 36 weighted points are exposed by
   (d) and are recovered with an *emitter*, not a prover — and SAF's witness pipeline
   already works (98/99 confirmed on no-overflow). The largest offensive estimate any
   proposal produced was +11 to +18, and the largest *measured* one is smaller still.
2. **Only 55 of the 180 clusters are ones a validator can never block, and
   `valid-memsafety` is 43 of them.** It is verdict-only in every suffix — including
   Concurrency, uniquely — and SAF has *zero* memsafety TRUE capability today, so every
   cluster is additive. Every witness-required cluster must clear a validator that just
   went 0-for-5 on SAF's best existing proofs, in a budget cut to 300 s.
3. **`no-overflow` is the strongest prover but the weakest position.** 9x the unreach
   prove rate (per cluster as well as per task) and an incremental precision wall rather
   than a binary architectural one — but **0 of its 61 clusters are verdict-only.** It is
   the right second offensive lever, not the first.

### 5.3 Movement 0 — stop reporting a score that is not real (S, ~1 week)

> Executable plan: [`plans/212-movement0-scoreboard-2027.md`](212-movement0-scoreboard-2027.md)

Nothing else should be scoped until the scoreboard matches the 2027 rules.

1. Replace `TRUE_WITNESS_NOT_REQUIRED` in `scripts/svcomp_split_eval.py` with a
   per-BASE-CATEGORY rule derived from the 2027 table in §5.1(d) — the suffix comes from
   the task's `.set` membership, so the harness must learn it (`headroom_final.py` has
   the loader). `{valid-memsafety, valid-memcleanup, no-data-race}` stay verdict-only;
   `termination` moves to witness-required; `unreach-call` gains the
   `{Arrays, Heap, Floats}` carve-out. Re-run the authoritative eval and publish the
   corrected score. **MEASURED: 151 → 110.** Termination's 36 lost, `no-data-race`'s
   violation requirement a further 5, and the unreach carve-out returned exactly +0 —
   SAF emits 3 unreach-call TRUEs in 22,631 tasks and their clusters are already capped. Until this lands every A/B in the loop campaign — and the
   KEEP metric it steers by — is measured against the wrong target.
2. Emit a 2.1 correctness witness on the termination path, to win the 36 back.
   `ranking.rs` already synthesises linear ranking-function coefficients via Farkas and
   **discards them** (`loops_are_ranked` returns `bool`); return them instead and render
   each as a `\at(L, AnyPrev) > L` transition invariant at the loop's controlling
   expression, per the 2.1 `terminating-program-example`. Parameterise the hardcoded
   `format_version: '2.0'` in `correctness_witness.rs`. Validate with witnesslint plus a
   real 2.1-capable validator — **not** the bundled CPAchecker 4.2.2, which rejects 2.1
   outright.

**Kill criterion:** if the corrected score does *not* fall by roughly 36, the reading of
the 2027 rules is wrong — stop and re-derive before doing anything else. Verify the
competition deadlines independently: a field agent reported registration 2026-10-08 and
submission 2026-10-20, but `sv-comp.sosy-lab.org/2027/dates.php` returns 404, so that is
**unverified**. If it is right, this movement is the only one that fits before the gate.

### 5.4 Movement 1 — make SAF's own verdict trustworthy, then cut the delegation (S/M, ~2 weeks)

> Executable plan: [`plans/213-movement1-soundness-and-de-delegation.md`](213-movement1-soundness-and-de-delegation.md)

This is the mandated exposure-closing work from §1 and the prerequisite for everything
offensive. **The three fixes must land together**, because (i) increases the number of ⊥
blocks and therefore the exposure to (ii).

1. `Interval::refine_eq_false`: singleton == singleton ⇒ `make_bottom`. ~3 lines. TDD the
   4-program sweep in §5.1(a) as RED tests.
2. `refine_branch_condition` unsigned arms (`transfer.rs:1510-1520`): guard with
   `lhs.lo() >= 0 && rhs.lo() >= 0`, no refinement otherwise — mirroring
   `Interval::icmp_ult`. TDD the three §5.1(c) witnesses as RED tests.
3. Restrict `prove_no_signed_overflow`'s scan to functions **reachable from `main`**
   (`checker.rs:1156-1160` universally quantifies over every defined function, including
   dead ones) and add the completeness gate — a parameterised lift of
   `termination.rs:116-209`: defined `main`; reject `llvm.global_ctors`/`global_dtors`;
   callgraph DFS from main; reject any reachable indirect-call placeholder; reject any
   reachable block whose terminator is `None` (the frontend dropped an `indirectbr` and
   the CFG is a lie); reject any reachable external not on a per-property inert
   allowlist. This is the §4 "AIR is lossy" defence, and it is a **whitelist**, which is
   the only safe discipline.
4. Re-run both full sweeps. **Only when both report 0 PROVEs on ground-truth-FALSE** may
   the CPAchecker gate be cut: delete `cpachecker_confirms_no_overflow` /
   `cpachecker_confirms_unreach` and their call sites in `try_overflow_true` (~5324) and
   `try_unreach_true` (~5437). **KEEP `overflow_source_constant_folds`** — it is SAF's
   own clang-based gate, not delegation, and it covers class 2 of §5.1(c).
5. Re-baseline the funnel. The post-fix `error-reachable` histogram is the *only* valid
   input for scoping any unreach-call work; today's is not.

**Expected score effect: ~0.** This movement buys legitimacy and a trustworthy baseline,
not points. Say so plainly when reporting it.

### 5.5 Movement 2 — `valid-memsafety` TRUE, SAF's first validator-free prover (M, ~3 weeks)

> Executable plan: [`plans/214-movement2-anchored-memsafety.md`](214-movement2-anchored-memsafety.md)

The offensive target: **43 clusters, every one verdict-only**, zero existing capability
so every cluster is additive, and — uniquely among the five properties — verdict-only
*including* the Concurrency suffix, so its 16 concurrent clusters (`pthread-wmm` 283
TRUE tasks, `weaver` 174, `goblint-regression` 116, `pthread-ext`, `pthread`, …) are in
scope with no witness work at all.

**Build the Anchored-Object prover** — thread-*insensitive*, syntactic, fail-closed,
modelled on `termination.rs` exactly as §4 recommends: prove or abstain, never guess.

*Domain.* `Anchor = { base: Global(ValueId) | Stack(alloca ValueId), offset: i64 }` —
a flat lattice with explicit ⊥ ("not anchored"); joining two different bases gives ⊥.
**A `Load` result never anchors**, so a pointer that came out of memory — dangling,
aliased, or interference-modified — can never be the subject of a discharged obligation.

*Obligations,* over the Movement-1 reachable universe: reject any reachable
`malloc`/`calloc`/`realloc`/`free`/`alloca`, and any reachable libc call that could
dereference a caller-supplied pointer (`memcpy`, `memset`, `strcpy`, `sprintf`, `qsort`,
… are **not** inert — `race_true.rs:145-149` already encodes this rule); then require
every reachable `Load`/`Store`/`GEP` address to be `base + constant` with
`offset + width <= size_of(base)` from the AIR's static type layout. Abstain on anything
else. No new abstract domain, no SMT, no interference reasoning.

*Why concurrency is free here.* Every obligation is a syntactic property of the SSA
pointer graph and static layout; none mentions a runtime value. Interference can only
change values, so no interleaving, reordering or thread count can invalidate a
discharged obligation. The only concurrency-sensitive input is *which code runs*, and
the universe over-approximates that as `main-tree ∪ ⋃ thread-entry trees` — the same
over-approximation `race_true.rs:551-556` already uses, which has held FP=0 across the
full 55,690-task run. `valid-free` and `valid-memtrack` are discharged *vacuously*
(nothing is ever allocated or freed).

*Reuse, don't rewrite.* The universe gate exists twice already —
`race_true.rs:491-596` and `termination.rs:114-209`. Movement 1 step 3 extracts it once
into `crates/saf-svcomp/src/universe.rs`, parameterised by an `ExternalPolicy`; this
movement supplies the memsafety policy. The only genuinely new code is a small
`layout.rs` (`size_of` over `AirType`, which already carries `byte_offset`/`byte_size`/
`total_size`) and the anchor propagation.

### 5.6 The de-risking spike, and what would kill this

**The spike is ~70% already run, in Python, with zero Rust written.** The decision
procedure was prototyped over LLVM IR during this design session and its artifacts are on
the VM at `~/static-analyzer-factory/{anchor2.py,falsescan2.py}`. Measured so far:

- **Yield: ≥1 provable task in 10 of the 16 concurrent clusters** — `pthread-theta`
  13/13, `pthread-atomic` 8/8, `pthread-wmm` 20/25, `pthread-ext` 17/25, `weaver` 14/24,
  `pthread` 11/25, `goblint-regression` 9/25, `ldv-races` 4/11, `pthread-C-DAC` 2/4,
  `pthread-deagle` 2/4 — and 3 of 27 sequential. Zero in `libvsync`/`pthread-divine`
  (`inttoptr`) and the heap-using clusters, as designed.
- **Soundness: 1 escape in 1,180 adversarial expected-FALSE tasks**, and that escape was
  exactly the one obligation the prototype had not yet implemented (the non-inert libc
  call gate). That is the good failure mode — a missing gate, not a broken argument.
- A necessary IR recipe fell out of it and is itself a deliverable: `clang-18 -O0
  -Xclang -disable-O0-optnone` then `opt-18 -passes=sroa,mem2reg,instcombine`. Without
  mem2reg the `-O0` pointer spill defeats anchoring outright.

**Remaining spike work (1 engineer-day, still no Rust):** add the layout/bounds
obligation (so `memset(p,0,81)` into an 80-byte object is rejected) and the non-inert
libc gate, then re-run both sweeps — the yield run over all 43 unsolved clusters, and the
soundness run over **all 10,014** expected-FALSE `valid-memsafety` tasks, not a sample.

- **PASS:** ≥7 of the 16 concurrent clusters yield a clean task with the bounds
  obligation on, **and** 0 of the 10,014 expected-FALSE tasks survive.
- **KILL (a):** fewer than 7 ⇒ the binding constraint is the universe gate (non-inert
  externals, indirect calls, dropped terminators), not the anchoring, so the estimate
  falls under +10 and sequential `no-overflow` dominates instead.
- **KILL (b), absolute:** *any* expected-FALSE survivor that needs a second, third and
  fourth ad-hoc gate. That is evidence the syntactic argument does not actually close,
  and one uncaught wrong proof is −32 uncapped.

**What would kill the whole plan:** if Movement 0's corrected score does not fall as
predicted, the 2027 rule reading is wrong and §5.2's entire ranking inverts. Test it as
TWO exact assertions, not one fuzzy one — the "roughly 50" written here was stale and
contradicted §5.3, and would have killed the plan on a passing result:
(a) switching only the TRUE-side rule must give exactly `151 → 115` (−36);
(b) the full table must give exactly `151 → 110` (−41).
**Both MEASURED and PASSED 2026-09-12** — see `plans/212`.

**The single measurement most worth buying next:** run a *second* validator family
(uautomizer-v2 / goblint-v2, not the bundled CPAchecker) against the 5 clusters of
§5.1(e). If they confirm, the witness-required ceiling is much softer than measured and
`no-overflow` should be promoted above `valid-memsafety`.

**A cheap bonus target created by the 2027 carve-out.** `C.unreach-call.{Arrays,Heap,
Floats}` are verdict-only, and two of those 8 clusters — `array-lopstr16` and
`array-multidimensional` — are clusters where the *no-overflow* sentinel already PROVEs.
After Movement 1's `refine_eq_false` fix, re-run `probe_unsolved.py --property
unreach-call` restricted to those 8: any that PROVE are bankable immediately, with no
witness and no validator.

### 5.7 Explicitly not doing, and why

> Movement 3 (`no-overflow` precision) has its own plan:
> [`plans/215-movement3-nooverflow-precision.md`](215-movement3-nooverflow-precision.md)

- **k-induction / 2LS-style kIkI over `bmc_incremental.rs`.** The encoding is not
  proof-grade: `ssa_encode.rs:20` uses a uniform `BV_WIDTH = 64` and self-documents that
  widths "differ only on overflow, which the native replay filters out" — a prover has no
  replay; `ssa_encode.rs:68-83` pushes `divisor != 0` as a path *guard that prunes the
  path* rather than modelling the case; floats return `None` and the caller substitutes a
  fresh havoc. Each is fine for FALSE-with-replay and fatal for a proof.
- **Trace abstraction (Ultimate-style).** Z3 removed its interpolation API in 4.8.0, and
  SAF's AIR is SSA with `ValueId`s and Phi nodes, so every instruction is a unique letter
  and an interpolant automaton degenerates to the trivial one-trace automaton. The
  Taipan variant (abstract interpretation over the path program) needs no interpolation
  and is the right door if this is ever revisited.
- **Thread-modular abstract interpretation (Miné/Goblint interference fixpoint).** The
  correct L-sized answer for the concurrent clusters that need *values* rather than
  syntax, and the natural Movement 4; `race_true.rs` already supplies thread discovery,
  the reachable-set union over thread entries and a lock-protection map. Not now, for
  two reasons: Movement 2 reaches 10 of the 16 concurrent memsafety clusters *without*
  any interference reasoning, and 2027 requires a 2.1 correctness witness for concurrent
  `unreach-call`/`no-overflow` that SAF cannot yet emit and cannot locally validate (the
  bundled CPAchecker 4.2.2 rejects format 2.1 outright and dies on `pthread_create`).
- **Sequential `unreach-call`.** Re-measure after Movement 1 before scoping. Its current
  1.3% prove rate was taken with `refine_eq_false` broken and is not a valid input.
- **Anything that makes an external tool decide a verdict** — §1, permanently.

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
