# Plan 215 — Movement 3: `no-overflow` precision (+5..10 weighted)

**Status:** DESIGNED but **DELIBERATELY UNDER-SCOPED.** Blocked on [`plans/213`](213-movement1-soundness-and-de-delegation.md) §5 re-baselining the funnel — today's histogram was taken with `refine_eq_false` broken and is not a valid input.
**Branch:** cut `movement3/nooverflow-precision` off whichever of Movement 1/2 lands last.
**Track:** TRUE-side. Movement 3 of 4.
**Design source:** `plans/211` §5.1(b), §5.2, §5.7. Memory: `saf-prover-funnel-corrected-baseline`, `saf-validator-ceiling-true-side`.

---

## 0. Goal, and why this is third and not first

`no-overflow` is SAF's **strongest prover** and its **weakest position**.

Strongest: 18/150 PROVE on the corrected funnel versus unreach-call's 2/150 — 9x, per
cluster as well as per task — and its wall is *incremental precision*
(`top-or-bottom-operand` 47%, `may-overflow` 13%) rather than a binary architectural one.
On the FALSE side it is visibly healthy: it correctly says `may-overflow` on 3,218 of
3,733 expected-FALSE tasks (86%), so it genuinely detects overflow rather than abstaining
for unrelated reasons.

Weakest: **0 of its 61 unsolved clusters are verdict-only.** Every point must clear an
external correctness-witness validator, in a budget cut to 300 s — and that validator
confirmed **0 of 5** of the clusters SAF already proves
(`saf-validator-ceiling-true-side`). Movement 2's 43 clusters have no such gate.

> ### FOLDED IN FROM MOVEMENT 0 — the validator ceiling is now MEASURED, and it is binding
>
> `saf-validator-ceiling-true-side`'s 0-of-5 was a worrying signal. Movement 0B turned the
> same question into a method and a hard number on the termination population, and the
> result transfers directly to this movement's premise.
>
> **The method.** CPAchecker 4.2.2 can PRODUCE the witness format it validates
> (`config/lassoRankerAnalysis.properties` for termination 2.1; find the analogous exporter
> for `no-overflow` correctness). Run it in prove mode with witness export over the target
> clusters, feed each witness straight back to its own validator, and record TRUE/UNKNOWN.
> That yields a per-cluster CEILING before any prover work: **if the oracle cannot
> produce-and-reconfirm a cluster, SAF cannot score it either.**
>
> **The result on termination.** The oracle reconfirmed 10 of 19 loop-bearing clusters.
> SAF, after Movement 0B, confirms exactly those 10 — the identical set — and without the
> `ignoreOverflowsForUnsignedVariables` knob the oracle needed for 4 of them and which
> CPAchecker itself flags as potentially unsound. So SAF was already at the tool ceiling,
> and the 9 remaining clusters were never an emitter gap.
>
> **Two hard limits found, both plausibly live here too.** (a) `product-lines` — 178 tasks,
> the single largest cluster — is unprovable by the oracle at ANY budget, including
> SV-COMP's real 300 s, because LassoRanker cannot build a ranking relation over CIL string
> literals. (b) **Nested loops crash the validator outright**:
> `IllegalArgumentException: Not supported interface` in
> `TransitionInvariantUtils.makeStatesEquivalent`, 0 of 15 fixtures, not fixable from the
> witness side.
>
> **Make this blocking, alongside §1's re-baseline.** Run the oracle probe over the 61
> unsolved `no-overflow` clusters BEFORE scoping any slice, and replace the `+5..10`
> estimate with what it measures. Given 0-of-5 on the clusters SAF already proves, the
> honest prior is that this movement's ceiling is low and it should be re-ranked BELOW
> Movement 4 (`plans/216`, +8 on the FALSE side, unblocked) and Movement 2.

So: real prover work, sequenced behind the work that cannot be blocked from outside.

## 1. Re-baseline FIRST (blocking)

`plans/213` §2a changes `Interval::refine_eq_false`, which fires on the guard shape that
dominates SV-COMP. The `top-or-bottom-operand` and `may-overflow` populations will move.
**Do not scope any slice below until these are re-run:**
```
scripts/measure_prove_funnel.py --per-task lever1-pertask.jsonl --property no-overflow -n 150
scripts/p211_probe_unsolved.py no-overflow 4 7
scripts/p211_probe_context.py  no-overflow 200 8
```
The pre-Movement-1 per-cluster picture over the 69 then-unsolved clusters was:
`top-or-bottom-operand` 32 (all sequential), `threads` 15 (all concurrent),
`may-overflow` 6, PROVE-but-unscored 5, `not-converged` 4, `indirect-call` 4,
`(no output)` 2, `unmodeled-arith` 1. Treat it as a shape, not as numbers.

## 2. Candidate slices, ranked by (measured yield) / (build cost)

Each is independently mergeable. Re-rank after §1.

### 2a. `nsw` gating — S, possibly the largest single win

`prove_no_signed_overflow_with_result` (`checker.rs:1165-1192`) checks **every** integer
`Add`/`Sub`/`Mul` against **signed** bounds, ignoring the `nsw` flag the frontend already
preserves (`Instruction::has_no_signed_wrap`, set at
`crates/saf-frontends/src/llvm/mapping.rs:1260-1276`). The SV-COMP property is *signed*
integer overflow only; clang emits `nsw` exactly on signed C arithmetic and leaves
unsigned ops unflagged, where wrapping is defined and **not a violation**.

So the prover currently abstains on unsigned arithmetic it never needed to check.
`bitvector/sum02-2.c` is the worked example: every operation is `unsigned long long`, yet
the sentinel returns `ABSTAIN:top-or-bottom-operand`. Precedent exists in-tree —
`ranking.rs:1064` already uses `has_no_signed_wrap()` as a signedness signal.

**⚠️ Soundness caveat, and it is the whole risk.** Filtering by `nsw` is sound only if no
pass dropped the flag between clang and the analysis. `compile_to_ir` runs `clang -O0
-Xclang -disable-O0-optnone` then an `opt -passes=...` pipeline, and `opt` passes may drop
`nsw` (conservative for a compiler, **anti**-conservative for us). Detection is also
textual (`mapping.rs:1860`, `tok == "nsw"`). Before trusting this:
1. Enumerate the `opt` passes actually run and confirm each preserves `nsw`, or pin the
   prover's ingestion to a pipeline that does.
2. Gate the merge on `p211_sweep_soundness.py no-overflow` = 0 over all 3,733
   expected-FALSE tasks. If `nsw` filtering ever lets a real signed overflow through, that
   sweep is where it shows up.

### 2b. `unmodeled-arith` — S, small and certain

`Shl` / `SDiv` / `SRem` currently abstain unconditionally (`checker.rs:1187-1189`).
Model them: `Shl` overflows iff `lhs << rhs` leaves the signed range; `SDiv`/`SRem`
overflow only on `INT_MIN / -1`. Worth 1 cluster on the pre-fix measurement and ~2-3% of
tasks; cheap and risk-free relative to 2a.

### 2c. `top-or-bottom-operand` — M, the main body of work

The dominant wall, and **not primarily a context problem**: on main-only programs (zero
own function definitions) it still fired on 36 of 73 probed tasks, so it is not mostly
callee parameters being ⊤. The ⊤ sources seen in source inspection were, roughly:
- unsigned arithmetic checked against signed bounds → **2a removes this entirely**;
- values from `__VERIFIER_nondet_*` that the program then *guards*, where the interval
  domain fails to learn the bound — e.g. `loop-zilu/benchmark09_conjunctive.c` guards
  `x == y && y >= 0`, which needs equality propagation between two variables;
- loads from arrays / memory with no content invariant — e.g.
  `loops/sum_array-2-2.c` needs `A[i] ∈ [-LIMIT, LIMIT]`. This is a genuine M/L
  memory-abstraction job; consider it a separate plan, not a slice.

### 2d. `indirect-call` / `not-converged` — S each, bounded

4 and 4 clusters pre-fix. `indirect-call` becomes a `universe.rs` policy question once
`plans/213` lands (resolve via PTA when the points-to set is a singleton, else abstain).
`not-converged` correlated strongly with arrays and floats — check it is not just the
widening giving up, before spending anything.

## 3. Explicitly out of scope

- **The `ABSTAIN:threads` block** (15 of the then-69 clusters). Concurrent `no-overflow`
  TRUE needs a **2.1** correctness witness in 2027, which SAF cannot yet emit and cannot
  locally validate — the bundled CPAchecker 4.2.2 rejects 2.1 outright and dies on
  `pthread_create`. Revisit only after `plans/212` §3 provisions a 2.1-capable validator.
- **Array/memory content invariants** (§2c third bullet) — a separate plan.
- **Any new abstract domain.** The dormant octagon is a complete lattice with no transfer
  function and zero callers; `plans/211` §3 prices wiring it at L/XL for +2..4.

## 4. Merge gate

- [ ] `p211_sweep_soundness.py no-overflow` = 0 PROVEs over all 3,733 expected-FALSE.
      **Non-negotiable for 2a specifically** — that slice removes checks.
- [ ] Full 55,690-task run: `false_alarms == 0`, `wrong_true == 0`, weighted up.
- [ ] Each slice merged separately so its delta is attributable.
- [ ] `make lint` clean, TDD-green.

## 5. The measurement that would kill this movement

If, after `plans/212`, a 2.1-capable non-CPAchecker validator still confirms ~0 of SAF's
`no-overflow` proofs, then the prover is not the binding constraint and no amount of §2
work converts into score. Re-run `scripts/p211_probe_confirm.py` against that validator
before funding §2c. If it confirms well, the ceiling is softer than
`saf-validator-ceiling-true-side` measured and this movement should be promoted.
