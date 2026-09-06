# Plan 208 — Reason-1 / 1c: absint loop-head interval precision (the ranks-2/3 unblocker)

**Status:** IMPLEMENTED — Slices 1-4b committed 2026-09-06 (`33835f73` core absint fix, `0e1f9418` Slice 4b bench gate) on `reason1/1c-absint-loophead-precision`. Approved fix = A-struct + B-TOP. Remaining before MERGE to the loop/competition branch: Slice 4 full-scale FP=0/perf A/B (loop is STOPPED; merge is a deliberate future step).
**Branch:** `reason1/1c-absint-loophead-precision` (off `reason1/1b-correctness-witness` HEAD `35dfd81f`; **re-verify all cited line numbers on current HEAD — they drift**). Nothing pushed; `saf verify` untouched; loop STOPPED.
**Track:** TRUE-side ("Reason 1"). Sequence `1a✅ → 1b✅ → 1c (this, BLOCKS 2&3) → 2 (no-overflow-TRUE) → 3 (unreach-TRUE)`.
**Research notes (laptop-side, NOT in repo):** `investigation/reason1-1c-spike-findings.md` (root cause + the validated prototype), `investigation/reason1-true-side-roadmap.md` §4/§7. Key facts embedded below so this plan is self-contained.

---

## 0. One-line goal

Make SAF's interval abstract interpreter expose a **sound, non-trivial loop-head interval on the NAMED loop variable**, so `saf emit-correctness-witness spike_counter.c --data-model ILP32` yields a witness CPAchecker CONFIRMS — unblocking ranks 2/3 — while keeping the whole suite green and **FP=0 / wrong-TRUE=0**. No `saf verify` verdict change in 1c.

## 1. What the spike established (carry forward — do NOT rediscover)

Root cause is **decided by ground-truth measurement** (throwaway `#[ignore]` harness `crates/saf-cli/tests/absint_loophead_spike.rs` + an env-gated widen trace). Verdict = **H1: a real absint soundness bug**, NOT H2 (name/representation split — the 1b name map is perfect: "i" → the phi dst). There are **TWO compounding bugs**:

- **Bug A — precision/termination (dominant).** `AbstractState` uses "absent key ⇒ ⊤" (`state.rs:22-23`), and `AbstractState::leq` (`state.rs:~726`) only iterates `self.values`, so `empty.leq(concrete)` returns `true` vacuously (empty is treated as ≡ everything). The worklist change-gate `state_changed = !new.leq(&old) || old.is_unreachable()` (`fixpoint.rs:581-582`) therefore reads the back-edge's **refinement** of an empty (=⊤) loop-header entry as "no change" and discards it. Measured: the header is widened exactly **twice** (entry-pred once, first back-edge once) then iteration HALTS; the widened state `{i=[0,0],%4=[1,1]}` is computed but never stored → the header entry state ends **empty** → **the loop body is analysed exactly once; the named counter never ascends.**
- **Bug B — soundness (latent).** The phi transfer resolves a **reached-but-absent** incoming as **⊥**, not ⊤ (`transfer.rs:888-893`). Over the empty ⊤ entry the back-edge operand is absent ⇒ phi = `[0,0] ⊔ ⊥ = [0,0]` (an under-approximation). Independently, an overflowing accumulator (`s += i`) whose back-edge `%4 = s+i` widens to ⊤ gets **dropped** by `widen_state`'s `is_top()` guards → absent → the same ⊥ collapse. `[0,0]` for a value that grows is **unsound**.

Both stayed hidden because a too-small range only costs the FALSE-side **recall** (missed bugs — never an FP), so FP=0 held; it is fatal only for the TRUE/witness track.

**The fix direction is PROVEN end-to-end** (one-line prototype of each, measured on the committed fixtures, Docker warm cache):
- Fix A (structural-inequality change-gate) → `spike_counter` `i = [0,1000000]` (was `[0,0]`), `converged=true`, `widening=59`; `dbgvalue_promoted_phi` `i = [0,100]`.
- **MILESTONE (definition of done) PASSED:** `saf emit-correctness-witness tests/programs/c/spike_counter.c --data-model ILP32` emits `value: 0 <= i && i <= 1000000` (line 26, col 5, fn main); native CPAchecker-4.2.2 via `scripts/validate_correctness_witness.sh` returns **`CONFIRMED (cpachecker-correctness)`** (raw verdict).
- Fix A **alone** leaves the accumulator `s` unsound (`[0,0]`). Adding Fix B (reached-but-absent phi incoming ⇒ ⊤) makes `s = [-2147483648, 2147483647]` (sound ⊤, which `interval_to_c_expr` drops → abstain) **while `i` stays tight `[0,1000000]`** — i.e. Fix B does not regress the bounded-counter precision on these fixtures.

## 2. The fix (recommended: A-struct + B-⊤; both validated)

**Fix A — change-gate detects refinement, not only growth** (`fixpoint.rs:581-582`, ascending phase):
```rust
let state_changed = new_partitioned.merge_all() != old_partitioned.merge_all()
    || old_partitioned.is_unreachable();
```
Rationale: non-header blocks use a monotone `join` (new ⊒ old) so structural inequality ≡ the old `!leq` there (no behaviour change); only loop headers (which use `widen`) differ, and widening still guarantees a sound post-fixpoint + termination (finite threshold chain → ⊤; the iteration cap at `fixpoint.rs:366` is the backstop → `converged=false` ⇒ abstain, never hang/wrong).
- **Perf note (TDD must measure):** `merge_all()` allocates two states per successor per visit, and the header now iterates ~|thresholds| (≈60–70) times per loop instead of 2. Per-loop widens are bounded by the fixed threshold-set size (NOT by the loop bound N), so it is O(70)/loop, not O(N) — but ~35× more absint iterations overall. Mitigations if too slow: derive/impl a cheap `PartitionedState` structural-eq (avoid `merge_all` clones), or restrict the refinement-detecting gate to loop-header successors only.
- **Alternative A-leq (more principled, higher blast radius — evaluate only if A-struct shows perf/termination problems):** fix the genuinely-buggy `AbstractState::leq` to also iterate `other`'s keys (so `⊤ ⋢ concrete`) AND use a lattice-inequality gate `!new.leq(old) || !old.leq(new)`. This also repairs the same latent `leq` bug in interproc convergence (`interprocedural.rs:~1975`) and narrowing (`fixpoint.rs:~1239`) — but changes `leq` semantics used in 4+ sites, so needs broader validation.

**Fix B — phi reads a reached-but-absent incoming as ⊤, not ⊥** (`transfer.rs:892`, the `unwrap_or_else`):
```rust
.unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));  // was make_bottom
```
Rationale: unreached predecessors are already excluded by the `reached_blocks` filter above, so a *reached* pred with an absent value genuinely means ⊤ under the "absent = ⊤" convention (e.g. it widened to ⊤ and was dropped). This aligns the phi with the rest of the codebase (the audit found `interval_at_inst` returns ⊤ for reachable-but-absent at `result.rs:113-119`, `compute_function_summary` already uses `make_top` for an absent Ret operand, and the condition-prover **explicitly requires** "a reachable-but-imprecise loop var must surface as TOP/wide, never bottom"). **Verify B does not broadly regress precision** (transient absence during iteration → premature-⊤ poisoning is the theoretical risk; it did NOT manifest on the two fixtures — measure on a broader loop sample + the full suite). If it does regress, fall back to **B-keep-⊤** (stop dropping `is_top()` entries in `widen_state`/`join`, pairing with A-leq's lattice gate so explicit-⊤ vs absent don't over-iterate).

## 3. Soundness audit result (plan-207 §5 — done: workflow, 8 agents, adversarially verified)

Question: does making loop-head intervals **wider** (sound over-approx) risk an FP/wrong-TRUE in any consumer? Every adversarial verifier returned **refuted=false**.

| Dimension | fp_risk | wrong_true_risk | Finding |
|---|---|---|---|
| numeric-checkers | low | **none** | All abstain on TOP; report only on bounded intervals ⇒ wider = *more raw candidates*, never a prove-safe over-claim. Competition FALSE verdicts gated behind a native ASan/UBSan trap on the **original** program ⇒ can't fabricate a wrong `false`/`true`. |
| condition-prover | low | **none** | Proof predicates are **anti-monotone in width** ⇒ widening only DROPS `Proven`, never fabricates one. Spurious-False channel is `svf_assert` (PTABen-only, absent from real SV-COMP tasks). |
| interproc-nullness | **none** | **none** | Nullness fully decoupled from intervals; return-summaries feed only the TOP-abstaining Z3 prover. `compute_function_summary` already defaults absent→`make_top`. |
| competition-path | **none** | **none** | **No live `saf verify` strategy reads loop-head intervals for a verdict.** All FALSE via native replay; TRUE structural. Only live reader = the dev witness emitter (offline-CPAchecker-gated). |

**Only regression surface = the internal bench/research harness** (`saf-bench::bench_result_to_verdict` ValidMemsafety, `svcomp/mod.rs:380-398`) which promotes *any* non-'Unconstrained' buffer finding (Warning OR Error) straight to `False` with no confirmer — precision, NOT a shipped SV-COMP verdict. **Mitigation (bench-only):** restrict its direct-FALSE to Error severity (mirroring `analyze_memsafety`'s Warning→Z3 gate), or route bench buffer-Warnings through `asan_confirm`.

## 4. Soundness redlines (wrong-TRUE MUST stay 0)

- The loop-head interval must be a sound **over-approximation** after the fix; a reachable-but-imprecise value surfaces as **⊤/wide, never ⊥** (Fix B is exactly this).
- Full `saf-frontends`+`saf-analysis` suite stays green (was **1815/1815** on the 1b branch); this domain is cross-cutting. Run the WHOLE suite, not just new tests.
- Honor the convergence gate: any invariant/verdict rests only on `diagnostics().converged==true` (already enforced in the 1b driver).
- **No `saf verify` / `strategy_for` / write-gate change in 1c** (that is ranks 2/3). 1c = absint fix + the 1b emitter's demonstration only.

## 5. TDD implementation slices

- **Slice 1 — Bug B (soundness floor), test-first.** Add absint unit/e2e tests: (a) a counted loop's named counter reads a sound superset of its true range (never a strict subset like `[0,0]`); (b) an overflowing accumulator reads a sound over-approx (⊤/wide), never `[0,0]`. Apply the `transfer.rs:892` ⊤ change. Green.
- **Slice 2 — Bug A (precision), test-first.** Test: `spike_counter`/`dbgvalue_promoted_phi` loop-head named var reads the tight sound bound (`i=[0,1000000]`, `[0,100]`), `converged=true`. Apply the `fixpoint.rs:581` structural gate. Green. Run the FULL `saf-analysis`+`saf-frontends` suite; fix/relax only tests that legitimately pinned the buggy `[0,0]` (audit flagged `absint_e2e.rs:~273 memcpy_overflow_no_findings_in_good_cases` as the one at-risk fixture) — each such change must be justified as "was pinning an under-approximation."
- **Slice 3 — end-to-end + measurement.** `saf emit-correctness-witness spike_counter.c --data-model ILP32` → `validate_correctness_witness.sh` returns CONFIRMED (real CPAchecker). Then re-run the 1b confirmation over a `c/loops` / `loop-invariants` sample and report the honest confirmation rate (feeds the ranks-2/3 ceiling estimate).
- **Slice 4 — FP=0 / perf guard.** Standalone A/B on cd-vm-14 or -15: full-pool (or large sample) `saf verify` before/after → confirm CONF unchanged or up, **FP=0 / wrong-TRUE=0 preserved**, and record absint runtime delta (perf regression budget). Apply the bench-harness mitigation (Slice-4b, saf-bench Error-only gate) if the bench eval shows new FALSEs on safe programs.

## 6. Acceptance criteria

- [ ] Bug B: no named/anonymous loop-carried value ever reads a strict subset of its true range (tested); overflowing accumulators read sound ⊤/wide.
- [ ] Bug A: `spike_counter` `i=[0,1000000]`, `dbgvalue_promoted_phi` `i=[0,100]`, `converged=true`.
- [ ] MILESTONE: `emit-correctness-witness spike_counter.c` witness is **CONFIRMED** by real CPAchecker (raw verdict shown).
- [ ] Full `saf-frontends`+`saf-analysis` suite green (≥1815); every changed test justified.
- [ ] FP=0 / wrong-TRUE=0 preserved at scale; absint runtime delta recorded + acceptable.
- [ ] `saf verify` / `strategy_for` / write-gate unchanged (diff-verified).

## 7. Risks & rollback

- **Perf (top risk):** ~35× more absint iterations per loop. Mitigate via cheap structural-eq / header-only gate; hard cap already fails safe (abstain). Measure in Slice 4; if unacceptable, optimize before merge.
- **Fix-B precision poisoning:** transient absent→⊤. Measure on a broad loop sample; fall back to B-keep-⊤ + A-leq if needed.
- **Bench-eval precision regression:** the one audited FP surface; bench-only mitigation ready.
- **Rollback:** both fixes are ≤ a few lines in two files; revert restores exact prior behaviour. Nothing wired to `saf verify`.

## 8. Open questions for sign-off

1. Recommended path = **A-struct + B-⊤** (validated end-to-end, minimal blast radius). OK, or prefer the more-principled **A-leq** (fix the root `leq` bug) despite wider blast radius?
2. Apply the bench-harness Error-only mitigation in this plan (Slice 4b), or track it separately as a bench-precision follow-up?


## 9. Results (implemented 2026-09-06)

- **Fix landed:** Bug A = structural change-gate (`fixpoint.rs:586`); Bug B = phi reached-but-absent -> TOP (`transfer.rs:892`). Approved combo A-struct + B-TOP.
- **Regression test** `crates/saf-analysis/tests/absint_loophead_precision.rs`: `spike_counter` i=[0,1000000], `dbgvalue` i=[0,100] (tight); accumulator `s` sound TOP (never [0,0]). Full `saf-analysis`+`saf-frontends` suite green (**1835 passed**). `clippy --workspace -D warnings` + fmt clean.
- **MILESTONE CONFIRMED:** `saf emit-correctness-witness spike_counter.c --data-model ILP32` -> `0 <= i && i <= 1000000` -> real CPAchecker `CONFIRMED`.
- **Slice 4b landed** (`0e1f9418`): severity threaded through `BenchBufferFinding`; bench memsafety scores direct-FALSE on Error only; 2 unit tests lock Warning->True / Error->False.
- **Confirmation-rate reality (honest):** on a 5-program `c/loops`/`loop-invariants` sample (`linear-inequality-inv-a`, `count_up_down-1/2`, `const`, `even`) SAF ABSTAINS on all — they need RELATIONAL (`x+y==N`) or MODULAR (`i%2`) invariants the interval domain cannot express (sound abstain, no wrong-TRUE). The fix unblocks the PURE-INTERVAL counted-loop subset (spike_counter); that subset is a minority of loop-invariant benchmarks -> ranks-2/3 prize is MODEST, as the roadmap predicted. Octagon/relational or modular domains would be needed to grow it (separate work).
- **Remaining (Slice 4, merge gate):** full-scale FP=0/perf A/B on the competition pool. Top residual risk = absint perf (header now iterates ~|thresholds| per loop); small-program emits ran fast, but large loop-heavy programs need measurement before merging to the loop branch.

- **Slice 4 MERGE GATE PASSED (A/B, 2026-09-06):** standalone `svcomp_split_eval.py` on a fixed 871-task `val` sample (every 10th), pre-fix `35dfd81f` vs HEAD, RAW scoring. BEFORE: FP=0 wrong_true=0 RAW=246 CONF=154. AFTER: FP=0 wrong_true=0 RAW=247 CONF=155. **Soundness preserved (both RESULT: PASS sound).** Exactly ONE verdict change and it is a GAIN (`pthread-race-challenges/...race-4`: Unknown -> correct-FALSE); zero regressions, zero new FPs. **Perf: per-task total 10814s -> 10826s = +0.1%** (mean 12.92 -> 12.93 s/task) over 837 common tasks -> no measurable regression (the interval absint is a small fraction of saf-verify cost). => 1c is SAFE TO MERGE into the loop/competition branch (a deliberate step, on request; loop STOPPED).
