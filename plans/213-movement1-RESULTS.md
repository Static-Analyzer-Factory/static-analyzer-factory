# Plan 213 — Movement 1 RESULTS (2026-09-14)

**Status:** fixes LANDED and measured; merge gate running. Branch `movement5/submission`
(uncommitted, per the standing never-auto-commit rule).

**Headline:** the no-overflow TRUE arm is live again as a SAF-native proof — the +12 the
tool removal cost, recovered with no SV-COMP participant anywhere in the verdict. The
unreach-call arm stays disabled, and that costs nothing.

---

## 0. Three corrections to this plan's own text

**(1) "Expected weighted effect: ~0" was right when written and is wrong now.** That
header assumed the movement both FIXED the sentinels and CUT the CPAchecker gate, so the
two cancelled. The cut already happened in `e5265281`; SAF already paid −12. Fixing the
bugs therefore *recovers* rather than cancels. Measured recoverable: **+12**, all of it
`no-overflow`.

**(2) "The 7 reduce to two classes" understates it — there are FOUR independent
mechanisms.** §1 below. The two this plan scoped are real and are fixed, but they close
only 2 of the 7. A third (SCCP cast folding) was unpredicted and closes 1 more plus every
minimized probe; a fourth is still open.

**(3) §2's claim that the constant-fold class is "already guarded in production by
`overflow_source_constant_folds`" is FALSE at HEAD.** That function was deleted in
`e5265281` together with the CPAchecker gate — `grep -rn overflow_source_constant_folds
crates/` returned nothing. It was an artifact *only while the TRUE arm was disabled*. It
has been rebuilt here, because without it three `signedintegeroverflow-regression` tasks
are real wrong TRUEs the moment the arm goes live.

---

## 1. What the 7 wrong PROVEs actually were

Re-measured at HEAD `2c951c96` over the COMPLETE expected-FALSE population
(4,324 unreach-call + 3,733 no-overflow): **exactly 7, the same 5 clusters as
2026-09-12.** The figure was current, not stale. Archived in `m1-baseline/`.

| # | mechanism | witnesses | status |
|---|---|---|---|
| a | `Interval::refine_eq_false` never ⊥ on EQUAL singletons | the 3-line `if(!(x==0))` program | **FIXED** |
| b | unsigned ICmp refined with the SIGNED refiners, no `lo>=0` guard | `implicitunsignedconversion-1` | **FIXED** |
| c | **SCCP cast folding** | `signextension-1`, + 14 minimized probes | **FIXED** |
| d | **loop-head interval UNDER-approximation** | `deep-nested`, `openssl s3_srvr_1a`, `nov1`/`nov2` | **OPEN** |

### 1c. The mechanism nobody predicted

`Interval::zext` is **correct** — it masks by the source width. The bug is in SCCP.
`SccpValue::Constant(i128)` carries **no width**, and `Operation::Cast` supplies only
`target_bits`, never the SOURCE width. So `evaluate_cast` (`sccp.rs:606`) treated `ZExt`
as a pass-through and `zext i16 -1 to i32` folded to `-1` instead of `65535`. Then
`icmp eq` folded to false, `CondBr` marked the live then-edge non-executable, and every
block behind it was collected as dead. `fixpoint.rs:388` `continue`s past a dead block
*after* writing its own entry state but *before* its successor loop — so the successor
keeps the ⊥ every block is pre-seeded with at `fixpoint.rs:328`.

That predicts the otherwise baffling **depth-1 vs depth-2** split exactly: at depth 1 the
error block IS the dead block and still received a live entry state → correct ABSTAIN *by
accident*; at depth ≥2 the intermediate block is dead, nothing propagates, and the error
block's pre-seeded ⊥ is read as a proof.

`Trunc` had the mirror defect: it masked to UNSIGNED, so `trunc i32 -1 to i16` gave
`65535`. Fixing only `ZExt` leaves that hole open — both were needed.

The invariant now documented and enforced: **the `i128` in `SccpValue::Constant` is the
SIGNED interpretation of the value at its OWN width.**

### 1d. What is still open

The interval fixpoint converges a state-machine phi to `[8466, 8496]`, *excluding* the
reachable `8512…8672`, so a live dispatch arm is refuted. It is **not** partition
dropping (`reduce_to_budget` correctly joins) and **not** narrowing (disabling the
descending phase changes nothing). It is magnitude-dependent — the identical 3-state
`while(1)` machine with states `0/1/2` analyses correctly — which points at threshold
widening. Minimal repros: `m1nov/nov1.c`…`nov4.c`.

---

## 2. The unifying defect, and the gate that closes it

Both sentinels treated **"the analysis never visited this block"** as **"the program
cannot reach this block"**. Since `fixpoint.rs` pre-seeds every block with ⊥, those two
are indistinguishable.

`prove_no_signed_overflow` had `state_at_inst == None => continue`. On
`openssl-simplified/s3_srvr_1a.cil` **all four** `add`s are absent, so `Proven` was
returned having checked **zero** arithmetic instructions. Its operands come from
`__VERIFIER_nondet_int`, so a visited block would have abstained `top-or-bottom-operand`.

Fixed: `carries_overflow_obligation()` + fail-closed `ABSTAIN:unvisited-arith`.
**Measured cost: 94 → 90 sentinel PROVE (−4 tasks, −1 cluster `loop-invariants`).** That
is the honest price of closing a −32 class, and it is paid on purpose.

`prove_unreachable` has no equivalent gate, because a ⊥ error block is exactly what its
proof *looks like*; distinguishing would need the fixpoint to record which blocks it
actually visited. That is why the unreach arm stays disabled — see §4.

---

## 3. The re-enabled arm

`try_overflow_true` is live, with four SAF-native fail-closed gates, cheapest first:
OpenMP source text → reachable thread spawn → `overflow_source_constant_folds` (SAF's own
clang `-Winteger-overflow` probe, rebuilt) → the sentinel. **Nothing outside SAF decides
the verdict.**

Measured on the verify path:
* the 4 no-overflow offenders → **all `false(no-overflow)`** (correct, not merely non-true)
* 40 expected-TRUE tasks → **39 `true`, 0 false alarms**; the 1 `unknown` is
  `loop-invariants/const.yml`, the completeness gate's known cost
* witness confirmation on the offline oracle → **12/12, identical pre- and post-fix,
  0 changed.** The fixes perturb the fixpoint but not the witnesses.

---

## 4. Why `try_unreach_true` stays disabled, and why that is free

The −12 was **entirely `no-overflow`**:
```
m0-combined-rescore (137): ndr 9, nov 34, term 27, unreach 52, mem 15
saf-alone-20260913  (120): ndr 9, nov 22, term 26, unreach 48, mem 15
delta:                            nov -12, term -1, unreach -4
```
The `unreach −4` is CBMC's two clusters plus CPU starvation, not the TRUE arm. `plans/212`
says the same independently: the 3 `unreach-call` TRUEs land in clusters already at the
dedup cap. **Marginal contribution: 0.** So leaving that arm off while mechanism (d) is
open is not a compromise — it costs nothing measurable.

---

## 5. §5's re-baseline prediction was wrong

This plan predicted "after §2a the `error-reachable` histogram will move, possibly a
lot". **It did not.** Over 552 stratified expected-TRUE unreach-call tasks, pre vs post:

| | pre | post |
|---|---:|---:|
| PROVE (tasks) | 3 | 5 |
| clusters with ≥1 PROVE | 3 | 4 |
| **newly bankable (verdict-only Arrays/Heap/Floats)** | — | **0** |
| lost to precision | — | **0** |
| `ABSTAIN:error-reachable` | 412 | 417 |

So `plans/211` §5.6's "cheap bonus target" — Arrays/Heap/Floats clusters that would become
bankable with no witness after the `refine_eq_false` fix — **does not materialize**. The
one new cluster (`loop-simple`) is witness-required, facing a validator that went 0-for-5.
Scope `plans/215` on this histogram, not on a hoped-for shift.

---

## 6. A process finding worth more than one movement

**The entire `#[ignore]`d test suite has never run.** No `--run-ignored` anywhere, no
`.config/nextest.toml`; `make test` omits it. That includes
`verify_unreach_wrongprove_is_not_true`, which `submission/READINESS.md` §1 cites as the
regression guarding the TRUE arms. It was inert.

Run for the first time at HEAD, **before any Movement 1 edit**: 7 `saf-cli` failures and
9 `saf-analysis` failures (5 of them SIGSEGV on Rust fixtures). All pre-existing.
`make test-ignored` was added so they are at least visible. Movement 1 introduced **zero**
new failures — verified by running the identical suite in a clean worktree at HEAD.

Consequence: "2711 tests pass" is NOT sufficient evidence a change is safe, because it
excludes the whole end-to-end layer.

---

# ADDENDUM (2026-09-14, later) — it was SEVEN mechanisms, and the checker found its own limit

§1 above said four. After re-enabling the no-overflow arm, an adversarial workflow
constructed programs that broke it, and the count rose to seven. Three more fixes landed.

## 7. The mechanisms, final tally

| # | mechanism | site | witnesses | status |
|---|---|---|---|---|
| a | `refine_eq_false` never ⊥ on equal singletons | `interval.rs` | 3-line `if(!(x==0))` | FIXED |
| b | unsigned ICmp refined with SIGNED refiners | `transfer.rs` | `implicitunsignedconversion-1` | FIXED |
| c | SCCP `evaluate_cast`: `ZExt` pass-through, `Trunc` masks unsigned | `sccp.rs` | `signextension-1`, 14 probes | FIXED |
| d | sentinels read "never visited" as "unreachable" | `checker.rs` | `openssl s3_srvr_1a` | FIXED (completeness gate) |
| **M2** | SCCP `evaluate_binary` folds in `i128` with **no normalisation to the result width** | `sccp.rs` | `t7` | **FIXED** |
| **M3** | **stale `block_refinements`** — an interval cached on one iteration, re-imposed by MEET forever | `fixpoint.rs` | `deep-nested`, `ovf1`, `ovf6`, `nov2` | **FIXED** |
| **M4** | 4-state dispatch chain under-approximates (3 and 5 states are both correct) | unlocated | `ovf2`, `p_4state_{add,mul}` | **OPEN** |

M2 is M1's sentence with `evaluate_binary` substituted for `evaluate_cast`:
`add i32 -2147483648, -2147483648` folded to `-4294967296`, a value no `i32` can hold.
Same invariant, different function — which is why fixing one did not fix the other.

M3 is the one that reached production. `collect_refinements` cached the INTERVAL a branch
predicate produced on iteration 1; `apply_refinements` re-imposed it by `meet` on every
later iteration. The true `[8467, 2147483647]` was clamped back to `[8467, 8495]`, so an
`add nsw` that really can overflow "proved" in-bounds. The `pred_count <= 1` justification
was wrong: one predecessor means no JOIN at the successor, not that the value arriving on
that edge is STABLE across iterations. A predicate is an edge invariant; the interval it
produced given one iteration's pre-state is not.

**M3 also closed `loop-simple/deep-nested`** — the last real `unreach-call` wrong PROVE,
which §1 had listed as open.

## 8. The post-fixpoint checker, and the thing it cannot see

Chasing mechanisms one at a time was not converging, so a structural check was added:
after convergence, for every edge `B -> S` out of a reachable block, verify
`refine_for_successor(B, S) ⊑ state[S]` — the defining property of a sound fixpoint
solution, which everything in the file assumed and nothing verified. It reuses the
solver's own `refine_for_successor`, so it cannot disagree with the solver for reasons of
its own. Exposed as `FixpointDiagnostics::fixpoint_verified`; both sentinels abstain
`fixpoint-unverified` when it is false.

**Measured: it costs nothing (93/99 unchanged) and it fires on no task in the 55,690-task
population. It also does not catch M4.** That is not a bug in the check — it is a real
limit worth writing down:

> A post-fixpoint check cannot detect a **self-consistent** under-approximation. If the
> analysis wrongly decides a block is unreachable, that block propagates ⊥, `⊥ ⊑ anything`
> holds, and the solution is internally consistent — just smaller than the least fixpoint.
> Catching M4 needs a **monotonicity** check during the ascending phase (no block's state
> may ever shrink), not a containment check after it.

An unconditional-edge-only version was tried first and was completely inert: the
under-approximations that matter sit on the refined dispatch edges of a loop, not on its
back-edges. The widened version is what shipped.

## 9. Where soundness actually stands

Full expected-FALSE sweep (4,324 + 3,733), sentinel level:

| property | original | final |
|---|---:|---:|
| unreach-call | 3 | **0** |
| no-overflow | 4 | **3** — all `signedintegeroverflow-regression`, **CLI-only** |

The three survivors have no arithmetic left in the IR (clang folds it), and the verify
path gates them with the rebuilt `overflow_source_constant_folds`. Confirmed end to end:
all four original offenders return `false(no-overflow)` through `saf verify`, and
`overflow_true_safe.c` returns `true`.

**M4 is the honest caveat.** It produces zero wrong TRUEs on the 55,690-task population,
but `ovf2` and friends are constructible wrong TRUEs, and a 4-state CIL dispatch chain is
exactly the shape SV-COMP contains. It is documented, minimally reproduced
(`synprobe/ovf2.c`, `synprobe2/p_4state_{add,mul}.c`), and bounded: 3-state and 5-state
chains are both correct, so it is a narrow precision-boundary artifact, not a general
failure.

## 10. Quality gates, final

* `cargo nextest run --workspace --exclude saf-python` — **2711 passed, 0 failed**
* `cargo clippy --workspace -- -D warnings` — clean
* `cargo fmt --check` — clean
* `make test-ignored` (new) — 7 `saf-cli` failures, **byte-for-byte the pre-existing set**;
  this movement added zero new ones, verified against a clean worktree at HEAD

---

# FINAL MEASUREMENT (2026-09-15/16)

**120 → 132 dedup-weighted, +12. `false_alarms = 0`, `wrong_true = 0`.**

Four of five properties measured end-to-end on the fixed binary (33,059 of 55,690 rows);
`unreach-call` held at baseline. Scored with the harness's own
`confirmed_score` / `weighted_confirmed_summary`, so it is directly comparable to the
120 baseline.

```
BASELINE   saf-alone-20260913            120   {ndr 9, nov 22, term 26, unreach 48, mem 15}
MOVEMENT 1 (4 properties measured)       132   {ndr 9, nov 33, term 27, unreach 48, mem 15}
```

| property | rows measured | delta | what moved |
|---|---:|---:|---|
| `no-overflow` | 9,063 (100%) | **+11** | 188 tasks `Unknown -> TrueCorrect` |
| `termination` | 2,395 (100%) | **+1** | 7 witnesses `NOT_CONFIRMED -> CONFIRMED` |
| `valid-memsafety` | 20,570 (100%) | 0 | **zero transitions** |
| `no-data-race` | 1,031 (100%) | 0 | 4 losses / 5 gains — bidirectional noise |
| `unreach-call` | baseline | 0 | 41% sampled: 2 flips, both timeout-tail noise |

**The `no-overflow` +11 is +12 recovered minus the completeness gate's 1-cluster cost**
(`loop-invariants`), exactly as predicted when the gate was measured in isolation.

**The `termination` +1 was not planned.** `build_interval_invariant_witness` consumes
`solve_abstract_interp`, so the absint fixes changed the loop invariants SAF emits — and
7 of them became acceptable to the validator. The verdicts were already correct; only the
witness content improved. Worth recording as a general effect: fixing the abstract
domain improves witness CONFIRMATION, not just verdicts.

**The memsafety blast radius was real to check and empirically nil.** `analyze_memsafety`
prunes SVFG dead-PHI edges using `run_sccp_module`'s `dead_blocks` *to reduce false
positives*, and the SCCP fix shrinks that set — so less pruning could have meant more
false alarms across 20,570 tasks at -16/-32 each. Measured: **zero transitions**, FP = 0,
`RESULT: PASS (sound)`.

## What the gate cost, and what it should be next time

The first full-run attempt died after 10 h 41 m at 21,828/55,690 with `ENOSPC`: root was
at 88% when it launched and the container's `/tmp` shares that overlay. 143 GB was
reclaimed (`target/debug` 97 GB — `target/release` is only 2.2 GB — plus scratch worktrees
and isolated target dirs) and a disk guard added to `scripts/m1_memsafety_gate.sh`.

It did not need re-running. `no-overflow` had already completed 100%, and splicing those
measured rows into the baseline answers the headline question directly. Cost profile,
measured over 14,158 tasks:

| | share of CPU | mean |
|---|---:|---:|
| `unreach-call` | **75.2%** | 32.8 s |
| `no-overflow` | 19.8% | 1.6 s |
| `no-data-race` | 3.7% | 2.6 s |
| `termination` | 1.3% | 0.4 s |

**96.4% of the CPU goes to tasks that end `Unknown`** — the full 60 s budget spent to
learn nothing. Witness validation, the assumed bottleneck, is 3.5%.

So a property-scoped gate is the right default: pick the properties the change can reach
(here `no-overflow` for the verdict, `valid-memsafety` for the SCCP coupling, and
`termination` because the witness builder shares the absint), and skip `unreach-call`
unless a FALSE path or `prove_unreachable` changed. That is ~25% of the cost for the same
evidence.
