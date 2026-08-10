# Plan 193: SAF SV-COMP 2027 — Evidence-Backed Capability & Gap Assessment + Roadmap

**Status:** research / strategic assessment — **awaiting approval (no implementation done)**
**Date:** 2026-08-10
**Branch:** `svcomp`
**Reference competition:** SV-COMP 2026 (2027 rules unreleased; 2026 is authoritative)
**Relationship to prior plans:** *Refines and, in three places, corrects* plan 191 (Part 3 gap-map +
Part 5 roadmap) with **measured** evidence and code-verified findings; *extends* plan 192 (which
implemented the unreach-call P0 slice). Where 191's roadmap ordering conflicts with the evidence
here, **193 supersedes it**.

> **How this was produced (evidence, not assertion).** (1) A blind `saf verify` sweep across all six
> C-track verification properties on the real 50,763-task `sv-benchmarks` tree (VM, Docker). (2) A
> memory-checker false-alarm/recall probe on 40 real `valid-memsafety` tasks. (3) First-hand reads of
> the load-bearing verdict-path code. (4) A 27-agent adversarial audit workflow (13 capability/engine/
> rules readers + 13 skeptics that re-checked every load-bearing claim against code + a completeness
> critic). Every capability claim below is grounded in `file:line` or a measured run. The adversarial
> pass **overturned several plan-191 assumptions** (§3).

---

## 0. Executive summary — the honest verdict

**Run blind through BenchExec today, SAF would score ≈ 0 points across the entire SV-COMP C track.**
Not because it finds nothing, but because of three compounding facts, each measured or code-verified:

1. **`saf verify` scores exactly one property — `unreach-call` — and returns `unknown` for the other
   five.** The handler hard-gates every non-`unreach-call` property to `unknown` *before any analysis*
   (`commands.rs:789`). Measured: on a blind sweep, `valid-memsafety`, `valid-memcleanup`,
   `no-overflow`, `termination`, `no-data-race` returned `unknown` on **0/N** tasks each.

2. **On `unreach-call`, measured blind recall is ~1/20 (≈5%)**, with **0 false alarms and 0 TRUE**
   (sound). SAF's FALSE engine catches only unconditional reaches (`must_reach_error`) or
   *intraprocedural* nondet-scalar-guarded reaches confirmed by native replay; the dominant real shape
   (interprocedurally-guarded `reach_error`) is missed.

3. **No witness is ever emitted.** `verify` accepts `--witness` (which BenchExec actively passes as
   `${witness}`) but **never reads or writes it** (`commands.rs:402-404` declared, referenced nowhere
   in the handler). The only witness code (`witness.rs`) is dead GraphML; there is **zero** YAML-2.0
   witness code anywhere. Under 2026 rules a FALSE without a confirmed witness scores **0**
   (correct-unconfirmed), so **even the ~5% of `unreach-call` bugs SAF does find currently score 0.**

**What SAF IS:** a *sound, under-approximating, FALSE-only, `unreach-call`-only, witness-less
bug-finder.* Its FALSE discipline (must-reach ∪ Z3-propose→native-replay-confirm) is genuinely sound —
execution is the arbiter — with two narrow residual holes (§3, §6).

**What SAF is NOT (and cannot cheaply become):** a *verifier*. It emits **no TRUE on any property** on
the competition path, by deliberate design (zero −32 exposure). Every TRUE-capable code path in the
tree is either dead code or an unsound absence-of-evidence heuristic (§5). The sound-TRUE side — the
larger scoring reservoir — is **greenfield**: it needs an inductive-invariant/k-induction engine, a
convergence-gated fixpoint, sound interprocedural PTA, and a correctness-witness emitter, none of which
are production-wired.

**The strategic thesis (unchanged from 191, now evidence-backed):** compete in **`C.FalseOverall`**
(only FALSE scores; zero −32 risk). The single highest-ROI action is **not** more analysis — it is a
**YAML-2.0 violation-witness emitter**, which flips SAF's *already-sound* `unreach-call` FALSEs from 0
to +1 across the largest reservoir (24,139 tasks). Everything else is second.

---

## 1. The reservoir (real task mix, measured on `testcomp26` tag)

| Property | Tasks | SV-COMP category | Witness for +2 TRUE? |
|---|---:|---|---|
| **unreach-call** | 24,139 | ReachSafety | correctness witness required |
| **valid-memsafety** (free/deref/memtrack) | 20,649 | MemSafety | **none required** (free +2) |
| **no-overflow** | 9,138 | NoOverflow | correctness witness required |
| **termination** | 2,430 | Termination | **none required** (free +2) |
| **no-data-race** | 1,031 | ConcurrencySafety | **none required** (free +2) |
| **valid-memcleanup** | 93 | MemSafety | **none required** (free +2) |

(coverage-branches/-conditions/-statements/-error-call and def-behavior are Test-Comp / non-verification
and out of scope.) **MemSafety (20,649) ≈ ReachSafety in size and is SAF's *native* strength** (SVFG
memory checkers) — yet it is entirely unwired in `verify`. That mismatch is the biggest single
opportunity *and* the biggest trap (its FALSE side over-reports badly — §4.2).

Scoring recap (verified vs `rules.php` 2026): **+2** correct TRUE *with confirmed correctness witness*
(or witness-not-required categories); **+1** correct FALSE *with confirmed violation witness*; **0**
unknown/unconfirmed; **−16** wrong FALSE; **−32** wrong TRUE. `C.FalseOverall` counts *only* FALSE
results — the safe home for a bug-finder.

---

## 2. Measured results (this session, VM/Docker, blind)

**2.1 Blind `saf verify` sweep** (`scripts/svcomp_verify_eval_all.py`, stride-sampled across families):

| Property | Sampled | Result | False alarms | TRUE emitted |
|---|---|---|---|---|
| unreach-call | 40 (20T/20F) | **1 FALSE caught, 39 unknown** (recall 1/20) | **0** | **0** |
| valid-memsafety | 12 | all `unknown` (0/12) | 0 | 0 |
| valid-memcleanup | 8 | all `unknown` (0/8) | 0 | 0 |
| no-overflow | 12 | all `unknown` (0/12) | 0 | 0 |
| termination | 10 | all `unknown` (0/10) | 0 | 0 |
| no-data-race | 10 | all `unknown` (0/10) | 0 | 0 |

Soundness held perfectly (0 false alarms, 0 TRUE) — but coverage is one property at ~5% recall, and
the sole caught task (`array-examples/data_structures_set_multi_proc_ground-1.i`) is the same ILP32
task slice-1c already reported. **This confirms the code-audit's structural recall estimate with a
real measurement.**

**2.2 Memory-checker false-alarm/recall probe** (`scripts/_probe_memsafety.py`, 40 real
`valid-memsafety` tasks, running the *live* `saf run --checkers all` findings surface as a
FALSE-on-any-finding oracle):

- **False-alarm rate on *safe* programs: 15/20 (75%)** — the checkers fire on most "good" variants.
- **Recall on *buggy* programs: 7/20 (35%)** — misses ~65% of real bugs.

*Caveat (stated honestly):* this uses `--checkers all` unfiltered on a small-first (Juliet-heavy)
sample, so **75% is a crude upper bound** on the −16 exposure (the live bench path filters "Unconstrained"
TOP buffer warnings and non-memsafety checkers; a family-stride sample would differ). But the direction
is unambiguous and matches the code: **the memory-checker FALSE side over-reports badly on safe code
and cannot be wired blind without a concrete-replay/witness gate.**

---

## 3. Corrections to plan 191 (evidence-backed; the adversarial pass overturned these)

Plan 191 was strategically right but rested on three code assumptions that are **false** in the current
tree. These change the roadmap, so they are called out explicitly:

**C1 — The "sophisticated analyzer" (`analyze_property` / `analyze_memsafety` / `analyze_no_overflow` /
`analyze_termination` / `analyze_no_data_race`) is DEAD CODE on *both* surfaces.** Plan 191 §3.1 framed
reconnecting `analyze_property` as "the single highest-leverage change," and 192 partially did so for
unreach — but for **every other property** `analyze_property` has **zero live callers**. The bench
harness *imports* it (`saf-bench/svcomp/mod.rs:56`) but never calls it; it grades via
`bench_result_to_verdict` (`mod.rs:333`) over **flat `check_all` findings**. So the elaborate Z3
temporal-UAF filter, ExplicitNull-vs-malloc-null downgrade, and loop-free-exactness ladders described in
191 **produce no verdict on any surface.** *Consequence:* "just wire the existing engine into verify" is
**not** a small task — the FALSE-candidate producer for memsafety/overflow is not invoked by any live
path and is unvalidated.

**C2 — The LIVE self-grading path is *cruder and less sound* than 191 credited.**
`bench_result_to_verdict` for `ValidMemsafety`/`ValidMemcleanup` emits FALSE on **any** substring match
(`c.contains("free")`, `mod.rs:376`) of a raw over-approximate checker finding, with **no Z3 gate, no
temporal filter, no replay**, and TRUE on **finding-absence** (unsound). My 75%-FP probe (§2.2) measures
exactly this surface. Additionally: `assertion_results` — the sole driver of the bench `UnreachCall`
*and* `NoOverflow` verdicts (`mod.rs:346,417`) — is populated **only** from `svf_assert` oracles
(`driver.rs:1384`), which real SV-COMP tasks never call, so the bench returns `Unknown` on essentially
all real reach-call/overflow tasks. **The bench harness gives no real SV-COMP signal for
unreach/overflow; its self-graded numbers for those are measured on `svf_assert` micro-benchmarks, not
competition inputs.**

**C3 — 191's own P0.8 audit had inverted/updated facts (192 already caught some):** CLI tracing is on
stderr (not stdout); Z3 wall-clock timeout never yields FALSE. **New in 193:** `verify` **writes no
witness at all** (the `--witness` arg is declared and silently dropped — worse than "targets the wrong
format"); and `must_reach_error` has **two narrow but real unsound-FALSE residuals** that 191 did not
identify (§6, redlines).

---

## 4. Per-property capability & gap matrix (the core assessment)

For each property: **(a)** what `saf verify` produces today + is it sound; **(b)** the credible
near-term path to *scored* coverage; **(c)** what is structurally out of reach and why. All grounded in
code/measurement.

### 4.1 `unreach-call` (24,139) — the only wired property

- **(a) Today:** emits `false(unreach-call)` or `unknown`, **never `true`** (`commands.rs:933-1007`;
  no TRUE branch). FALSE comes from **(1)** `must_reach_error` — a genuine under-approximation that
  follows only forced single-successor CFG chains + unconditionally-executed *direct* calls, bailing at
  any branch/loop/recursion/noreturn/assume (`property.rs:1679-1785`) — or **(2)** `enumerate_false_candidates`
  (over-approximate Z3, unsound alone: models guard operands as fresh unconstrained ints, no
  arithmetic/cast/pointer-identity, intraprocedural) **confirmed by native concrete replay**
  (`replay_confirms_false`, `commands.rs:1152`) — execution is the arbiter, so false alarms that don't
  reproduce stay `unknown`. **Measured: 1/20 recall, 0 false alarms, 0 TRUE.** Sound. **Scores 0** in
  competition (no witness).
  - **Two residual −16 holes in Stage-1** (must-reach, which has *no* replay backstop): matches only
    `Operation::CallDirect` (`property.rs:1745`), so an **indirect** call to `exit`/`abort`/`assume`
    before `reach_error` is walked past; and opaque **external declarations are modeled optimistically
    as returning** (`property.rs:1715-1717`, comment admits "Residual"). A diverging-but-unlisted
    external or indirect noreturn before `reach_error` → spurious FALSE. Narrow, but real, and unguarded
    by replay.
- **(b) Near-term scored coverage:** ① **YAML-2.0 violation witness** (the trace already exists as
  `FalseCandidate.block_path`/`nondet_sequence`) → flips sound FALSEs from 0 to +1. ② **Close the two
  Stage-1 holes** (treat `CallIndirect` and non-allowlisted externals as `Indeterminate`, or route
  Stage-1 through replay). ③ **Interprocedural candidate composition** (slice 1d): reuse
  `compute_error_summaries` + `z3_utils/interprocedural` to propose callee error sites reachable from
  `main`, still replay-gated — the structural recall fix for the dominant missed shape.
- **(c) Out of reach:** **sound TRUE at scale.** The only TRUE-capable path (`analyze_unreachability`)
  is over-approximate on a fresh-var Z3 model that *cannot prove infeasibility*, with no loop-invariant/
  k-induction/interprocedural-fixpoint engine. Proving the (large) safe bucket needs infrastructure SAF
  lacks.

### 4.2 `valid-memsafety` (20,649) — native strength, unwired, over-reporting FALSE

- **(a) Today:** `saf verify` → **`unknown` on 100%** (gated out, `commands.rs:789`). The memory
  checkers (SVFG null-deref/UAF/double-free + interval buffer-overflow) exist but drive **no verdict
  surface**: `analyze_memsafety` is dead (C1); the live `bench_result_to_verdict` path (self-graded, not
  blind) maps raw findings→FALSE by substring, TRUE by absence. **Measured on the live findings surface:
  75% false-alarm on safe / 35% recall on buggy** (§2.2). Under-approximate detectors → sound-*ish* for
  FALSE *only with a confirmation gate*; the TRUE side is unsound absence-of-evidence. `valid-memtrack`
  (leak) isn't even in `analyze_memsafety`'s spec set.
- **(b) Near-term scored coverage:** wire **FALSE-only** into `verify` with a **new ASan-based replay
  confirmer** — emit `false(valid-memsafety)` only when an over-approximate SVFG/interval candidate is
  *reproduced* by a concrete run trapping the UAF/double-free/OOB/explicit-null; map all TRUE/Unknown →
  `unknown`. This inherits unreach-call's soundness discipline and mirrors §2.2's lesson (no gate ⇒ −16
  bleed). Requires a YAML-2.0 violation witness lowering SVFG nodes → source lines. **This is genuinely
  new infrastructure** (`replay_confirms_false` is `reach_error`-sentinel-specific — not a parameter
  tweak) and a multi-month effort, but the reservoir (20,649) justifies it as the #2 property.
- **(c) Out of reach:** **sound memory-safety TRUE** (the free +2). Proving absence of all
  OOB/UAF/double-free/leak on all paths needs an over-approximate whole-program heap/shape/separation
  proof + correctness invariants. SAF's stack is under-approximate may-analyses on Andersen CI PTA;
  "found nothing" is not a proof (−32). No shape/separation domain, no invariant→witness pipeline.

### 4.3 `no-overflow` (9,138) — the most tractable sound-TRUE reservoir (later)

- **(a) Today:** `saf verify` → **`unknown` on 100%**. The interval integer-overflow checker
  (`absint/checker.rs`, CWE-190) runs only in `saf run` (findings-only, written to a field no verdict
  reads) and in the dead `analyze_no_overflow`. The interval **domain's core lattice ops are provably
  sound** (join/meet/widen/narrow correct, saturating arithmetic, **float→TOP is conservative
  imprecision, not unsoundness**) — a real building block. But nothing is wired to a verdict.
- **(b) Near-term scored coverage:** **sound FALSE on the loop-free slice** — reuse
  `check_integer_overflow_with_specs` Error-severity findings (whole result interval outside signed
  range) in loop-free functions with concrete non-TOP operands, **confirmed by
  `-fsanitize=signed-integer-overflow -fsanitize-trap` native replay** (reuses the unreach harness,
  no new domain work). Prerequisite: fix `DEFAULT_BITS=64` to derive signed width from the AIR int type
  (`transfer.rs:131,821`). Harvests the CWE-190/loop-free subset.
- **(c) Out of reach:** overflow **inside loops** (interval widens → can't prove tight bounds),
  **relational** overflow (box loses correlation; octagon exists but is **orphaned** — zero callers),
  **float-derived** overflow (float→TOP), and **correctness witnesses** for looping programs (needs
  invariants the domain can't emit). Sound TRUE also needs the fixpoint's `converged` flag (currently
  never gates a verdict) + PTA convergence gating (§5).

### 4.4 `termination` (2,430) — the *only* cheap sound-TRUE, tiny slice

- **(a) Today:** `saf verify` → **`unknown` on 100%** (also gated by `is_supported()=false`). No
  termination analysis exists: no ranking-function synthesis, no lexicographic measure, no
  lasso/recurrent-set engine (grep-confirmed). A latent **unsound** aggressive branch
  (`property.rs:1451`) would call looping-but-non-nondet programs — including `while(1)` — TRUE; it is
  gated off but must be **deleted, not gated**.
- **(b) Near-term scored coverage:** wire **loop-free ⇒ TRUE** (the one sound `+2`-no-witness win) —
  `program_is_loop_free` already exists — **but require an ACYCLIC call graph too** (a loop-free but
  self/mutually-recursive program may not halt; the naive proposal misses this ⇒ −32). Small effort,
  small but nonzero, and the only place SAF can soundly say TRUE.
- **(c) Out of reach:** everything else. Proving TRUE on data-dependent loops (ranking functions) or
  FALSE (non-termination / recurrent sets) needs a genuine termination prover SAF does not have and
  cannot cheaply grow.

### 4.5 `no-data-race` (1,031) — no-threading TRUE only

- **(a) Today:** `saf verify` → **`unknown` on 100%**. **No surface ever emits a race FALSE.** MTA/MHP
  exists but: MHP *under*-approximates (only parent-child concurrency; siblings not concurrent,
  `types.rs:184`), the lockset engine is dead code, per-thread access attribution is broken
  (drops thread-entry callees), join-narrowing is a no-op, and thread discovery in the verdict path
  runs *without* PTA. The single-thread⇒TRUE path is sound *in practice* (pthread-declaration scan) but
  unsound in general (an indirectly-called `pthread_create` is missed ⇒ spurious TRUE ⇒ −32).
- **(b) Near-term scored coverage:** **no-threading ⇒ TRUE** (`+2`, no witness) via
  `has_threading_primitives` — **but first fix its `CallIndirect` blind spot** (the comment claims
  indirect handling; the loop matches only `CallDirect`, `fast_paths.rs:58-69`) or sequential-TRUE is
  unsound.
- **(c) Out of reach:** race **detection** (sound FALSE — needs correct per-thread access sets, sibling
  MHP, a wired must-lockset, and a concrete schedule witness) and race **proving** on multithreaded code
  (sound TRUE — needs over-approximate MHP over all interleavings + all indirect forks + join
  narrowing). None of the plumbing is connected.

### 4.6 `valid-memcleanup` (93) — lowest priority

- **(a) Today:** `unknown` on 100%. The `memory-leak` checker is a **may**-analysis ("reaches *some*
  free ⇒ safe"), structurally wrong for the all-paths/at-exit "all heap freed" property; both directions
  unsound if wired.
- **(b) Near-term:** at most a narrow sound-FALSE-only abstainer (zero-value-flow alloc + no reachable
  deallocator) → a handful of points. TRUE must stay permanently off.
- **(c) Out of reach:** sound TRUE (needs must-free-on-all-paths + at-exit heap-liveness). 93 tasks ⇒
  not worth engineering before everything else.

---

## 5. Per-engine soundness posture (cross-cutting)

| Engine | Role | Sound? | Wired to a verdict? |
|---|---|---|---|
| `must_reach_error` | unreach FALSE (Stage-1) | **Sound** under-approx, **modulo** indirect-call + optimistic-external residuals; **no replay backstop** | ✅ `verify` |
| Z3 path (`check_path_reachable`) + `enumerate_false_candidates` | unreach FALSE proposer | **Unsound alone** (fresh-var operands); only a candidate generator | ✅ `verify` **behind replay** |
| slice-1c native replay | unreach FALSE confirmer | **Sound** (execution is arbiter); scalar-int nondet only | ✅ `verify` |
| interval absint | overflow/buffer; TRUE-side foundation | **Domain ops sound** (widen/join correct, float→TOP conservative); but `converged` flag **never gates a verdict**; scalar-only | ❌ dead / findings-only |
| octagon (relational) | precision upgrade | sound DBM | ❌ **orphaned, zero callers** |
| SVFG/MSSA mem checkers | memsafety FALSE | over-approximate (Andersen CI PTA) → **over-reports (75% FP measured)**; TRUE=absence=unsound | ❌ dead / findings-only |
| MTA/MHP + lockset | race | MHP **under**-approx; lockset **dead**; attribution broken | ❌ dead / findings-only |
| Andersen PTA | foundation for all memory reasoning | over-approx (sound) **when converged**; **base-layer truncation is UNGATED** → non-fixpoint `NoAlias` is unsound (the FS-PTA layer gates, the base layer does not) | underlies checkers |
| `witness.rs` | witness | GraphML (dead, wrong format, wall-clock timestamp) | ❌ **zero callers** |
| YAML-2.0 witness | required for any FALSE to score | **does not exist** | — |
| IFDS/IDE taint | — | n/a | irrelevant (no SV-COMP taint property) |

---

## 6. Soundness redlines (zero tolerance for −16/−32; must hold in every future change)

1. **Never emit TRUE from blind `verify` with the current engines.** Every TRUE-capable path is
   finding-absence heuristic or rests on ungated PTA truncation / non-fixpoint absint. The current hard
   "never prints `true`" gate is the primary soundness guarantee — **preserve it** until a
   convergence-gated fixpoint + sound interprocedural PTA + correctness-witness pipeline exist.
2. **Never emit FALSE without (a) an unconditional must-reach proof OR (b) concrete replay
   confirmation.** The Z3 enumerator is unsound alone; keep its candidates replay-gated. No FALSE
   straight from Z3 SAT.
3. **Close or replay-gate the Stage-1 `must_reach` holes before scaling FALSE:** treat any
   `CallIndirect` and any non-allowlisted external as `Indeterminate` (`property.rs:1715,1745`).
   Stage-1 has no replay backstop, so these are the *only* live unsound-FALSE paths in the wired CLI.
4. **Any NEW property FALSE (memsafety/overflow/memcleanup) needs its OWN concrete confirmer.**
   `replay_confirms_false` is `reach_error`-sentinel-specific; it cannot confirm a UAF/OOB/leak. Emitting
   on over-approximate findings alone = −16 (the 75%-FP result).
5. **Never ship `bench_result_to_verdict` as a competition surface.** Substring matching +
   finding-absence-TRUE + no gate ⇒ both −16 and −32 exposure. It is a self-grading measurement tool.
6. **Termination TRUE requires loop-free CFGs AND an acyclic call graph**, and the aggressive
   `!conservative ⇒ TRUE` branch must be **deleted**.
7. **Sequential no-data-race TRUE requires hardening `has_threading_primitives`** to treat
   `CallIndirect`/unlisted spawn wrappers conservatively.
8. **Before any sound-TRUE work, gate on absint fixpoint convergence AND base-Andersen PTA
   convergence** (fail-closed to `Unknown` on truncation). These contracts are currently
   documentation-level, not enforced returns.

---

## 7. Prioritized roadmap (ROI-ordered; effort + soundness-gated)

Legend: **Effort** S/M/L/XL. Reservoir in parens. All FALSE work targets `C.FalseOverall` (zero −32).

| # | Action | Effort | Why here / evidence |
|---|---|---|---|
| **R1** | **YAML-2.0 violation-witness emitter**, wired into `unreach_verdict`'s two FALSE returns; replace dead GraphML; SHA-256 `input_file_hashes` + spec + data_model metadata; drop wall-clock timestamp; `witnesslint` + validator (CPAchecker/UAutomizer) round-trip | **M** | **Highest ROI by far.** The unreach FALSE engine already works and is sound, but **every FALSE scores 0 without a confirmed witness** (measured: no witness written). Converts existing FALSEs 0→+1 across the **largest reservoir (24,139)** at near-zero new analysis. *Non-trivial part:* mapping AIR BlockId/Z3 model → source `file:line` waypoints. |
| **R2** | **Measure real blind recall** over the full `unreach-call` reservoir + **count −16 triggers** from the Stage-1 holes | **S** | Everything is code-reasoned + a 40-task sample. One full run tells us whether must-reach+replay yields ~500 or ~5,000 sound FALSEs and whether the indirect-call/optimistic-external holes actually fire. Cheap, decisive, ungates evidence-based prioritization. |
| **R3** | **Close the two Stage-1 `must_reach` −16 holes** (indirect-call → Indeterminate; non-allowlisted external → Indeterminate, or replay-gate all Stage-1 FALSE) | **S** | Purely defensive; removes the only unsound-FALSE paths in the wired CLI. Do **before** scaling witnesses — a witnessed wrong-FALSE is still −16. |
| **R4** | **Interprocedural FALSE-candidate composition** (slice 1d): propose callee `reach_error` sites reachable from `main`, composing caller↔callee arg constraints; still replay-gated | **M-L** | The **structural recall fix** — most real `reach_error` sites are in callees, invisible to today's intraprocedural enumerator. Second-largest ROI after witnesses; soundness free (replay-gated). Reuses `compute_error_summaries` + `z3_utils/interprocedural`. |
| **R5** | **`valid-memsafety` FALSE-only into `verify`** with a **new ASan-based replay confirmer**; emit only on replay-confirmed UAF/double-free/explicit-null/Error-overflow; all TRUE/Unknown→unknown; + violation witness | **L-XL** | **Second-largest reservoir (20,649)** and SAF's native strength, but **genuinely new infra** (dead producer, unsound live path, 75%-FP without a gate). Realistically multi-month. High reward, high effort. |
| **R6** | **`no-overflow` sound-FALSE loop-free slice** via `-fsanitize=signed-integer-overflow` replay; fix `DEFAULT_BITS` width derivation first | **M** | Reuses the interval checker + replay harness; harvests the CWE-190/loop-free subset (9,138 reservoir, smaller effective slice). |
| **R7** | **`termination` loop-free ∧ acyclic-callgraph ⇒ TRUE**; delete the unsound aggressive branch | **S** | The **only cheap sound-TRUE** in the codebase (`+2`, no witness). Small slice of 2,430, but it is the sole place SAF can soundly say TRUE. |
| **R8** | **`no-data-race` no-threading ⇒ TRUE**; harden `has_threading_primitives` `CallIndirect` first | **S** | `+2` no witness on the sequential subset of 1,031; multithreaded majority out of reach. |
| **R9** | **Native self-contained ZIP packaging** (release build, bundled/loader libLLVM, `share/saf/{stubs,specs}`, `smoketest.sh` exit 0, ≤2 MB stdout) + fm-tools YAML (`ai` label if any LLM) | **M** | **OCI/Docker is not an accepted submission.** Required before *any* real entry, but no code-scoring value until R1 lands — sequence after the FALSE engine scores. |
| **R10** | **`valid-memcleanup`** narrow sound-FALSE abstainer | **S** | 93 tasks. Do last, if ever. |

**Deliberately NOT on the near-term path (structurally out of reach for a *sound* 2027 entry):** sound
TRUE for unreach-call/no-overflow at scale (needs k-induction/IC3 + invariant→witness + convergence-
gated PTA/absint — greenfield); sound memory-safety/memcleanup TRUE (needs shape/separation); race
detection or race-freedom proving; general termination/non-termination; the Test-Comp Coverage track
(no test-goal generator).

---

## 8. What SAF realistically CAN and CANNOT solve for SV-COMP 2027

**CAN (sound, achievable with R1–R8, in ROI order):**
- **`unreach-call` FALSE** — already sound; **scores once R1 (witness) lands**; recall grows with R4
  (interprocedural). Realistic target: the intra/inter-procedural nondet-scalar-guarded + unconditional
  bug subset of 24,139, with a confirmed witness. This is SAF's real competitive contribution to
  `C.FalseOverall`.
- **`valid-memsafety` FALSE** (R5) — a meaningful fraction of 20,649 *if* a replay/ASan confirmer is
  built to tame the 75%-FP problem. Highest-reward stretch goal.
- **`no-overflow` FALSE** on the loop-free slice (R6).
- **A small sound-TRUE footprint**: `termination` loop-free (R7) and `no-data-race` no-threading (R8),
  both `+2` with no witness.

**CANNOT (structurally, without new engines SAF does not have):**
- **Any sound TRUE at scale** for reach-call/overflow (no inductive-invariant/k-induction engine; no
  correctness-witness emitter; ungated PTA/absint convergence). This forfeits `C.TrueOverall` and
  `C.Overall`.
- **Sound memory-safety / memcleanup TRUE** (no shape/separation/heap-liveness).
- **Data-race detection or race-freedom proving** on multithreaded code (MHP under-approx, lockset dead).
- **General termination/non-termination** (no ranking-function/recurrent-set machinery).
- **The Coverage/Test-Comp track** (no test-goal generator).

**Bottom line:** SAF can be a **credible, sound `C.FalseOverall` entrant** — realistically competitive
on `unreach-call` (and later `valid-memsafety`/`no-overflow`) FALSE — **once it can emit a confirmed
witness.** It **cannot** be an `Overall`/`TrueOverall` contender for 2027 without building the
sound-prover side essentially from scratch. Chasing TRUE prematurely is the −32 trap; the disciplined
path is *witnessed sound FALSE first.*

---

## 9. De-risking experiments (run before committing to R4/R5)

1. **Witness confirmation rate:** emit YAML-2.0 violation witnesses for the ~5% of `unreach-call` FALSEs
   SAF finds; run through CPAchecker/UAutomizer; measure confirmation %. Directly predicts
   `C.FalseOverall` score. (Validates R1 before scaling.)
2. **Full-reservoir blind recall + −16 audit** (R2): the numbers this whole plan is reasoned from.
3. **Memsafety replay-confirmer feasibility:** on the 75%-FP sample, measure how many false alarms an
   ASan-instrumented replay filters out, and the recall retained — decides whether R5's ROI justifies
   the multi-month build.
4. **PTA convergence-truncation frequency:** how often the base-Andersen iteration limit is hit on the
   memsafety reservoir (the ungated-truncation soundness blocker for any future TRUE).

---

## 10. Rules / packaging compliance snapshot (verified vs 2026 primary sources)

- **Scoring / meta-categories / anti-fingerprinting / `ai` label:** as in plan 191 §1, re-confirmed.
  `C.FalseOverall` (only FALSE scores) is the target; MemSafety/memcleanup/termination/data-race TRUEs
  are `+2` **with no witness required** (a free-points surface SAF forfeits today by returning `unknown`).
- **Witness:** YAML 2.0 is the active format; GraphML is legacy. `witnesslint` + ≥1 validator must
  confirm. **SAF emits none** → 0-point ceiling until R1.
- **Submission artifact:** a **native self-contained ZIP** — **OCI/Docker is not accepted.** SAF today
  ships a Docker image with a *debug* binary dynamically linked to `libLLVM` → **R9 required.**
- **Determinism:** verify pins the two env toggles (`commands.rs:765-771`); residual cross-machine
  hazards are the ungated base-PTA truncation and replay timeout/candidate-cap (bounded, `unknown`-only).
- **`benchexec/tools/saf.py` + `smoketest.sh`:** present and correct (tests green); `saf.py` passes
  `--witness ${witness}` which SAF currently drops (fixed by R1).

---

## 11. Ask — and the approved decisions (2026-08-10)

This was an **assessment + roadmap for approval — no implementation was done.** **User-approved
2026-08-10:** (a) the CAN/CANNOT framing (§8) — the CANNOT/greenfield items are pursued **later**, not
this cycle; (b) the ROI ordering (§7), with **R1 (witness) as the immediate next implementation target**
ahead of any new analysis; (c) the soundness redlines (§6) as **non-negotiable invariants** (mirrored
into repo `CLAUDE.md` → "SV-COMP Competition Work" so every future session honors them).

**Approved design principle — EXTENSIBILITY IS A HARD REQUIREMENT.** Because R5–R10 (memsafety /
overflow / termination / race / memcleanup) are approved-but-deferred, the R1 work must NOT hardcode the
`unreach-call` path. Concretely: build the **YAML-2.0 witness emitter as a property-generic module**
(violation-witness for any property; correctness-witness variant reserved), and refactor the `verify`
verdict dispatch (`commands.rs` `unreach_verdict`) into a **property→verdict-fn table** with a pluggable
**per-property "propose → concrete-confirm → witness" pipeline** (the confirmer is property-specific —
reach-error sentinel for unreach, ASan trap for memsafety, `-fsanitize` trap for overflow — but the
orchestration, watchdog, determinism pins, and witness emission are shared). This lets each deferred
property plug in later without a rewrite.

On approval the natural first implementation plan is **plan 194: property-generic YAML-2.0
violation-witness emitter + extensible verdict dispatch, applied first to the unreach-call FALSE path**
(bundles R1 + R2 + R3), turning SAF's existing sound work into actual `C.FalseOverall` points while
laying the extensible spine R5–R10 build on.
