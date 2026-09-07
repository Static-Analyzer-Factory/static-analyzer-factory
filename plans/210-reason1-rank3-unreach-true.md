# Plan 210 — Reason-1 / rank 3: `unreach-call` TRUE (sound verdict + confirmed witness)

**Status:** SPIKE DONE + **SIGN-OFF RECEIVED (2026-09-07, "ship it now")** + **IMPLEMENTED (Slices A–D, TDD-green) + MERGE-GATE (Slice E) PASSED.** UNCOMMITTED on the branch; commit + merge to `auto/loop-20260816` on user request. `prove_unreachable`/`UnreachProof` (+ size/inst cost guards) live in `saf-svcomp/property.rs`; `try_unreach_true` + `cpachecker_confirms_unreach` + TRUE-first `unreach_strategy` in `saf-cli/commands.rs`; the `saf prove-unreachable` dev subcommand stays (measurement tool). 2696 workspace tests green (6 new `prove_unreachable` unit tests + 2 new e2e), `cargo fmt --all --check` + `cargo clippy --workspace -- -D warnings` clean.
**Branch:** `reason1/rank3-unreach-true` off `auto/loop-20260816` @ `61386591` (has rank 2 + 1c + 1b). Nothing pushed.
**Track:** TRUE-side ("Reason 1"). Sequence `1a✅ → 1b✅ → 1c✅ → 2 (no-overflow-TRUE)✅ → 3 (this)`.
**Research notes (laptop):** memory `saf-reason1-true-side-investigation`; `investigation/reason1-true-side-roadmap.md` §3/§4/§5; `plans/207` (1b emitter), `plans/208` (1c absint), `plans/209` (rank-2 — the template cloned almost verbatim).

---

## 0. Goal
A **sound `unreach-call` TRUE verdict** in `saf verify` (a TRUE arm in `unreach_strategy` via `strategy_for`), backed by the sound converged interval absint + a **hardened error-block-⊥ read-out** + a **CPAchecker-confirmed `invariant_set` correctness witness** (+2). **Keep FP=0 / wrong-TRUE=0.** Same "SAF proposes, CPAchecker disposes" design as rank 2 — the absint alone is NOT wrong-TRUE=0-safe (the spike measured 3 wrong-proves), so the in-process CPAchecker confirmation gate is MANDATORY.

## 1. Spike results (VM cd-vm-15, real CPAchecker-4.2.2 native, `.p3spike/`)

### P1 — pool
**18,621** unreach-call-TRUE tasks (ILP32/LP64 mixed) + **4,805** unreach-call-FALSE (SAF's core recall). The TRUE pool is **dominated by families the interval domain cannot prove**: `hardness` 6789, `regression` 4193, `ldv-linux-*` ~1700, `eca-rers2012` 734, `hardware-verification-bv` 724, `neural-networks` 546, `nla-digbench` 280, plus concurrency/float/recursion sound-abstain buckets. The interval-provable subset (interval bounds alone imply the error guard infeasible: loop-free guarded programs, counted loops with post-loop asserts) is **tiny** — exactly the roadmap/§prompt prediction ("interval-provable unreachability is narrow; likely WORSE than no-overflow").

### P2 — sentinel prove-rate + wrong-TRUE audit
Throwaway `prove_unreachable(module) -> UnreachProof` (added to `saf-svcomp/property.rs`, exported at `saf_svcomp`): converged-gate → enumerate every `reach_error`/`__VERIFIER_error` site via `find_calls_to(REACH_ERROR_NAMES)` → require, over the CONVERGED solution, that EVERY error site's block is **present AND `is_unreachable()`** (the SOUND rule — never infer from absence), with a function-analyzed guard (entry present + non-⊥) and fail-closed on no-error-site. Dev subcommand `saf prove-unreachable` adds the OpenMP-source + reachable-thread gates.

Stratified corpus: **3561 tasks** (2379 TRUE / 1182 FALSE), capped 40 TRUE / 25 FALSE per category across 128 categories. Sentinel timeout 20s.

- **PROVE = 8 total: 5 TRUE-proves + 3 FALSE-proves.**
- **TRUE prove-rate 5/2379 = 0.21%** (rank-2 was 13.8%). Proves: `signextension2-1.c`, `ldv-regression/volatile_alias`, `heap-data/quick_sort_split`, `list-simple/dll2n_update_all_reverse`, `busybox/basename-2`.
- **⭐ 3 FALSE-proves = sentinel WRONG-TRUEs** (`bitvector-regression/signextension-1`, `bitvector-regression/implicitunsignedconversion-1`, `loop-simple/deep-nested`) — the absint over-narrows sign-extension / unsigned-conversion / deep-nested guards, spuriously marking the error block ⊥. **Confirms the absint + sentinel ALONE is NOT wrong-TRUE=0-safe** (the exact rank-2 lesson; the CPAchecker gate is required).
- Abstain reasons: `error-reachable` 2599 (dominant), `not-converged` 385, `threads` 339 (sound), `no-error-site` 3 (fail-closed); + 170 absint-timeout(20s), 41 native-stack-overflow(rc139), 16 compile-fail(rc1).

### ⭐ P3a — CPAchecker confirmation (the decisive gate test)
`build_interval_invariant_witness` returned **None for ALL 8 proves** (loop-free / non-renderable invariants) ⇒ `try_unreach_true` emits an **EMPTY** `invariant_set` witness ⇒ CPAchecker self-proves. Empty-witness CPAchecker (GENERIC `correctness-witness-validation.properties` + `unreach-call.prp` + matching `--32/--64` + `checkProgramHash=false`):
- **5 TRUE-proves → 2 CONFIRMED (`Verification result: TRUE`)** (`signextension2-1`, `volatile_alias`), **3 UNKNOWN** (`dll2n`, `basename-2`, `quick_sort_split`) → sound abstain, score 0.
- **⭐ 3 FALSE-proves → ALL REJECTED** (2× `FALSE`, 1× `UNKNOWN`) → **wrong-TRUE = 0.** The CPAchecker gate catches every absint wrong-prove.

### P3b — broader FALSE audit (empty-witness CPAchecker never self-proves a reachable error)
**119 FALSE tasks, empty-witness CPAchecker → 0 TRUE** (98 UNKNOWN, 14 FALSE, 7 NONE). Combined with the 3 rejected wrong-proves (P3a) that is **0 / 122 FALSE tasks ever confirmed TRUE** by the gate — strong corroboration that the CPAchecker gate keeps wrong-TRUE=0 even against absint unsoundness at scale.

### ⭐ Confirmable-yield estimate (honest)
Sample: **2 sound CPAchecker-confirmed unreach-call TRUEs / 2379 TRUE = 0.084%.** Pool-weighted over 18,621 TRUE ≈ **~16 raw** confirmed-TRUEs; the confirmed tasks are scattered singletons (`signextension2`, `volatile_alias`) ⇒ **dedup-weighted low-single-digits.** **Smaller than rank-2 (+63).** SAF's loop invariants (1b/1c) are **not load-bearing** here — CPAchecker self-proves the tiny loop-free/guarded programs from an empty witness; SAF's value is (a) the cheap FILTER selecting which ~0.2% of tasks to hand CPAchecker, and (b) the FP=0 gate.

### ⭐ Rank-3-specific cost finding (new vs rank-2): absint blow-up on big programs
170 corpus tasks hit the 20s absint timeout; those files are **min 5825 / median 87,997 / max 1.49M lines** (eca-rers/ldv/hardware-bv), while all 5 proves are **≤1602 lines**. The unreach pool contains these giant reactive/driver families (the no-overflow pool did not), so rank-3 needs a **cost guard** rank-2 did not: skip the TRUE attempt (before the absint) on large modules. A module-block-count (or source-line) pre-gate cleanly separates the two populations (keeps all proves, skips all timeouts). Without it, running the absint first on SAF's most-used property risks eating the 850s budget on huge FALSE tasks → a FALSE-recall regression.

## 2. Design (proposed — needs sign-off; clones rank-2 `try_overflow_true` almost verbatim)

**TRUE arm = cheap gates → size guard → sound error-block-⊥ sentinel (FILTER) → emit witness (empty, or 1b invariants when renderable) → in-process CPAchecker confirmation as the VERDICT gate → emit `true` iff CONFIRMED; else fall through to the EXISTING FALSE path (must_reach → concurrency confirmers → replay/BMC/interproc), unchanged.**

**Plumbing (verified in the code-map @ `61386591`):**
- `unreach_strategy` (`commands.rs:1563`): make it **TRUE-first** — call `try_unreach_true(ctx)` at the top; on `Some` return it, else run the existing FALSE pipeline verbatim.
- `try_unreach_true(ctx)` (new, clone of `try_overflow_true` `commands.rs:5150`):
  1. `source = read_to_string(ctx.input)`; **OpenMP abstain** (`source_has_openmp`).
  2. **Reachable-thread abstain** (`CallGraph::build` + `fast_paths::reachable_spawns_threads`).
  3. **Size guard** (rank-3-specific): abstain if the module exceeds the measured block/inst threshold (skips the absint blow-up on eca/ldv/hardware-bv).
  4. **Sentinel**: `saf_svcomp::prove_unreachable(ctx.module) == Proven` (the sound error-block-⊥ read-out over the converged solution; fail-closed).
  5. Witness: `build_interval_invariant_witness(module, &source, meta).unwrap_or_else(|| InvariantSetWitness::empty(meta))` (empty is the common case here; the invariant path is retained for the rare renderable loop, reusing 1b).
  6. **FINAL GATE**: `cpachecker_confirms_unreach(ctx, &witness_path)` — clone of `cpachecker_confirms_no_overflow` (`commands.rs:5095`) but with the **GENERIC** `correctness-witness-validation.properties` config (NOT `--overflow`) and `--spec` = `ctx.meta.specification` (already the raw unreach-call `.prp` on the verify path). `resolve_cpachecker` reused as-is.
  7. On confirm: `VerdictOutcome{ verdict:"true", witness:None, graphml:None, correctness:Some(witness) }`; else `None`.
- `VerdictOutcome.correctness` + the write-gate `true` branch (`commands.rs:912`) **already exist** (rank 2) — the correctness witness is written with zero new plumbing.
- Dead `analyze_unreachability` (`property.rs:490`) stays DISCONNECTED; the new verdict plugs into `strategy_for`/`unreach_strategy`, never `property.rs`.

**Why TRUE-first is cost-safe here:** the size guard (step 3) skips the absint on the giant programs; on the small provable/FALSE set the absint is <1s. CPAchecker runs ONLY on the ~0.2% sentinel-proved tasks. The merge gate (Slice E) confirms no FALSE-recall/latency regression. (Alternative considered: run `must_reach_error` first, or FALSE-first; TRUE-first + size guard matches rank-2's clean shape and the guard removes the only cost risk.)

## 3. Soundness guardrails (wrong-TRUE MUST stay 0)
- **Sentinel read-out over the CONVERGED solution only** (`diagnostics().converged`); the SOUND rule = error site's block PRESENT and `is_unreachable()` (⊥). **NEVER infer unreachability from a `None`/absent key** (absence = "not analyzed", not "dead"). Function-analyzed guard: the error's function entry present + non-⊥. Per-function analysis starts from ⊤ entry ⇒ over-approximates every call context ⇒ a ⊥ error block is unreachable whole-program.
- **Fail-closed** on: non-converged fixpoint, no located error site, OpenMP source, reachable thread spawn, oversized module (cost guard). (Indirect calls / recursion over-approximate in the absint and cannot manufacture a false ⊥ — not required for soundness; the CPAchecker gate is final anyway.)
- **⭐ In-process CPAchecker confirmation is the FINAL verdict gate** (GENERIC correctness config + unreach-call `.prp` + matching `--32/--64` + `checkProgramHash=false`; block-style witness): emit `true` ONLY on raw `Verification result: TRUE`; FALSE/UNKNOWN/absent/timeout/parse-fail ⇒ fall through (never `true`). REQUIRED — the spike proved the absint sentinel alone is not wrong-TRUE=0-safe (3 wrong-proves, all caught here).
- **TRUE ⟂ FALSE authority:** never infer TRUE from the FALSE path finding nothing; the FALSE pipeline is unchanged and remains the FALSE authority.
- **Watchdog:** the existing 850s worker-thread watchdog (`commands.rs:860`) maps timeout/panic ⇒ `unknown`.

## 4. Slices (TDD)
- **A** — `prove_unreachable(module) -> UnreachProof` promoted from the throwaway to a tested core (unit tests: a proven loop-free-guard fixture PROVES; a reachable-error fixture ABSTAINS `error-reachable`; **a wrong-TRUE-class fixture** — e.g. an unconditionally-reachable error — ABSTAINS, never PROVES; non-converged ⇒ abstain; no-error-site ⇒ abstain; declaration-only error fn ⇒ abstain `func-not-analyzed`). Decide final home (`saf-svcomp` vs a generic `prove_blocks_unreachable` in `saf-analysis` + error-site wrapper in `saf-svcomp`).
- **B** — the size guard (block/inst-count threshold), unit-tested against a synthetic large module + the measured proves.
- **C** — `cpachecker_confirms_unreach` + reuse `resolve_cpachecker` (GENERIC config + unreach-call spec, fail-closed).
- **D** — TRUE-first `unreach_strategy` via `try_unreach_true`; e2e green: a provable fixture (`signextension2-1`-style) → `true` + CPAchecker-confirmed empty witness; the 3 measured wrong-TRUE fixtures → NOT `true` (abstain → FALSE path); FALSE recall fixtures unchanged. `make fmt && make lint` clean; full nextest suite green; `saf verify` diff-reviewed.
- **E (MERGE GATE) ✅ PASSED (2026-09-07)** — real `saf verify` A/B (release): rank-3 (`saf-rank3`) over the full **3561-task** corpus vs baseline (`saf-baseline` = `61386591`, `try_unreach_true` absent) over the **1182 FALSE** tasks. **Result:**
  - **⭐ wrong-TRUE (expected-false→true) = 0** (over all 3561; + spike 0/122 + the wrong-prove e2e). The paramount invariant holds.
  - **⭐ +2 sound CPAchecker-confirmed TRUE** (`bitvector-regression/signextension2-1`, `ldv-regression/volatile_alias`) — exactly the spike's 2; both empty-witness self-proofs (+2 each).
  - **false-alarm = 0 introduced by rank-3.** The report's 40 "expected-true→false" were a **corpus-scan mislabel** (`validation-crafted`: my `scan_pool.py` emitted spurious `expected=true` rows for `for.c`/`if.c`/… whose only `.yml` declares `expected_verdict: false`); **baseline ≡ rank-3** on them (both correctly `false`), and rank-3's TRUE arm cannot emit `false` by construction.
  - **FALSE recall preserved** — over 859 common FALSE tasks: rank-3 solved **466** vs baseline **465** (net **+1**). The 5 "regressions" (vs 6 gains) were **timing noise** in SAF's non-deterministic FALSE portfolio under parallel contention: an **isolated re-run of all 5 gave `baseline=false` AND `rank3=false` (identical)**.
  - **Operational finding (fixed): a shared-container OOM at -P 8.** A single `saf verify` on a `hardware-verification-bv` btor2c model reached **16 GB RSS** (dense bit-level state → the interval absint), and 8 in parallel exceeded the 52 GB container cgroup → OOM cascade at task 1149. **This is a merge-gate harness artifact, not a competition regression** (BenchExec memory-isolates each task; an absint OOM there scores that already-0 non-provable task 0, same as baseline). **Hardened:** added an **instruction-count cost guard** (`defined_inst_count > 20_000`, alongside `defined_block_count > 5000`) so the absint skips these dense models before the solve (block-count alone missed them — btor2c is few-blocks/huge-instruction); resumed at -P 4 with a memory-heavy-category skip. `prove-unreachable` on the btor2c model then abstains cleanly (`error-reachable`), and the 2 confirmed TRUEs still confirm.
  - Crashes: 218 `rc137` = the -P8 OOM-cascade artifacts (the guarded -P4 resume had 11 timeouts); 8 `rc139` = native stack overflow on huge CIL (pre-existing, same class rank-2 saw). None are wrong verdicts (no verdict emitted ⇒ score 0, as on baseline).

  **Merge to `auto/loop-20260816` only on user request.**

## 5. Open questions for sign-off
1. **GO / scope:** the honest dedup-weighted prize is **low-single-digits** (smaller than rank-2's +63), but the machinery is ~100% reused, wrong-TRUE=0 is proven, it composes with the FALSE path with no regression (Slice E), and it completes the witness-required TRUE roadmap. Ship rank 3 now, or defer as the smallest TRUE rank? **[RECOMMEND: ship — low cost, sound, completes the roadmap; set expectations at "modest".]**
2. **Approve the CPAchecker-confirmation gate** as the final verdict authority (vs absint soundness)? **[RECOMMEND: yes — spike proved the sentinel wrong-proves 3 FALSE tasks; CPAchecker caught all 3.]**
3. **Approve the rank-3 size guard** (skip the TRUE attempt on oversized modules before the absint, to protect the unreach FALSE path's budget)? **[RECOMMEND: yes — clean separation, sound (only affects TRUE recall on giant programs, which are unprovable anyway).]**
4. **Ordering:** TRUE-first + size guard (recommended, matches rank-2), or gate the TRUE attempt behind `must_reach_error`/FALSE-first for extra cost safety? **[RECOMMEND: TRUE-first + size guard; the merge gate is the empirical check.]**
