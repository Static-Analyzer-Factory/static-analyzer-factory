# Plan 191: SV-COMP 2027 Strategy & Roadmap for SAF

**Status:** research / strategy (not yet an implementation plan)
**Date:** 2026-08-09
**Branch:** `svcomp`
**Reference competition:** SV-COMP 2026 (2027 rules unreleased; 2026 is the authoritative reference — <https://sv-comp.sosy-lab.org/2026/>)
**Scope:** Answers four questions — (1) what we learn from the 2026 winners, (2) what SAF still needs to support SV-COMP, (3) a critical evaluation of the small-LLM ideas in `../ideas.md` against the competition rules, and (4) a prioritized, documented list of improvements worth trying.

> All rules/scoring/limits/winner facts below were gathered from the primary source (`sv-comp.sosy-lab.org/2026`, the BenchExec docs, the sv-witnesses spec) and cross-checked against the official TACAS 2026 report (Beyer & Strejček, *Evaluating Software Verifiers for C, Java, and SV-LIB*, DOI `10.1007/978-3-032-22749-2_23`) by an adversarial verification pass. Confidence flags and sources are in the Appendix.

---

## 0. Executive summary (the strategic thesis)

**SAF today is a *bug-finder*, not a *verifier*, and its SV-COMP code is an *offline self-grading harness*, not a *competition entry point*.** Those two facts dominate everything.

Five load-bearing conclusions:

1. **The biggest, most immediate wins are NOT LLM work — they are plumbing and soundness.** SAF cannot currently be *run* by SV-COMP's harness at all: no BenchExec entry point, no in-tool C→IR compilation, no witnesses, no per-task time budget. Before any "small-LLM advisor" matters, SAF needs a competition-shaped verifier around its analyses.

2. **SAF's current "TRUE" verdicts are unsound (absence-of-evidence), which is the single most dangerous thing in SV-COMP.** An incorrect TRUE = **−32 points** (16× the reward of a correct proof). A bug-finder that emits TRUE because "I found no bug" will be crushed. The safe posture is *UNKNOWN-not-guess*, and the natural competitive home for a bug-finder is the **`C.FalseOverall`** meta-category, where TRUE verdicts don't count at all.

3. **A correct answer scores ZERO without a *confirmed witness*.** SAF finds real bugs but emits no validator-consumable witness, so today every bug it finds would score 0 ("correct-unconfirmed"), not +1. A **YAML witness 2.0 violation-witness emitter** is the highest-ROI single feature.

4. **The winners are sound provers first, with bug-finding bolted on.** Symbiotic — the closest architectural analog to SAF (LLVM-IR + pointer analysis + slicing + symbolic execution) — reaches #3 Overall, #1 MemSafety, #1 FalseOverall. That is the walkable ladder for SAF.

5. **Small LLMs are *feasible but narrow*: strictly time-boxed, single/few-shot, hint-only, and only where a sound checker validates the output.** No network, no GPU, and a CPU-time budget *shared with the verifier and summed across cores* mean the LLM can't cannibalize the 900 s. For pure *selection/ranking* tasks a tiny classifier/GNN (PeSCo/Graves precedent) beats an LLM. The LLM's unique value is **generative** invariant/predicate synthesis, which a sound engine then checks.

**One-line roadmap:** make SAF a *runnable, sound, witness-producing* verifier that competes in `C.FalseOverall`/`C.MemSafety` first (P0–P1), grow a sound TRUE side via abstract interpretation + k-induction + correctness witnesses (P1–P2), add an in-tool strategy portfolio (P2), and only then layer learned components — an LLM invariant generator being the best-justified one (P3).

---

## Part 1 — SV-COMP 2026 reference: the constraints that govern everything

### 1.1 Scoring (verified verbatim)

| Points | Result | Condition |
|---:|---|---|
| **+2** | TRUE correct | program free of errors **and a correctness witness was confirmed** (or witness not required) |
| **+1** | FALSE correct | error found **and a violation witness was confirmed** |
| **0** | UNKNOWN | out of resources / crash / gives up / timeout |
| **−16** | FALSE incorrect | false alarm (property actually holds) |
| **−32** | TRUE incorrect | missed bug (unsound proof) |

- **Proofs are worth more than bugs (+2 vs +1); wrong answers are punished ~16× the reward** ("a single correct answer should not compensate for a wrong answer").
- **Positive points require a *confirmed* witness.** A correct answer whose witness no validator confirms is **"correct-unconfirmed" → 0 points**. (An unconfirmed *correctness* witness degrades +2→+1; an unconfirmed *violation* witness earns nothing.) In 2026, ESBMC-kind lost **2,456 ReachSafety tasks** to unconfirmed witnesses — pure lost points. This is the single largest lever separating "finds the answer" from "scores."
- **Ranking** by sum of points; ties broken by **success-runtime** (total CPU time over correct results — so *speed matters* for tie-breaks and for fitting the budget).
- **Meta-categories** are **normalized per task count** (score ÷ tasks-in-category, summed, × avg tasks/category) so small categories aren't drowned by large ones.

### 1.2 Categories (C track)

`unreach-call` (ReachSafety) · `valid-memsafety` = {valid-free, valid-deref, valid-memtrack} + `valid-memcleanup` (MemSafety) · `no-overflow` signed-int (NoOverflows) · `no-data-race` (Concurrency) · `termination` · SoftwareSystems (real code: BusyBox, coreutils, AWS-C-Common, Linux drivers, uthash, Intel-TDX).

**Meta-categories that matter for a bug-finder:**
- **`C.FalseOverall`** — *only FALSE results score* (correct/incorrect TRUE are not counted). **This is the natural home for SAF** — it can compete here without ever risking a −32.
- `C.TrueOverall` — only TRUE results score (proving ability).
- `C.Overall` — everything.

### 1.3 Resource limits & execution environment (decisive for LLMs)

- **15 GB RAM · 900 s CPU time · 4 processing units** per verification run. Hung tools killed at 15 min wall time.
- **CPU time is *summed across cores*** (BenchExec `resources.md`: "if the tool uses more than one CPU core at the same time, the CPU time is the sum of the usage times for each of the cores"). ⇒ the 900 s budget ≈ **~225 wall-seconds of 4-thread compute**. Any LLM inference *spends the same budget the verifier needs.*
- **NO network** (BenchExec container mode, no `--network-access`; DNS disabled) — rules out any remote/API model. HIGH-CONFIDENCE.
- **NO GPU** — CPU-only x86_64 Xeon (~3.4 GHz, E3-1230 v5-class, 8 units/33 GB machine, 4 units/15 GB per run). HIGH-CONFIDENCE.
- Witness validation runs separately: violation **90 s**, correctness **300 s** (cut from 15 min in 2026), 2 units / 7 GB.
- Filesystem: read-only container except cwd and `/tmp`. Tool archive must be **fully self-contained** (weights bundled, no fetch).

### 1.4 Witnesses (what SAF must emit)

- **YAML witness format 2.0/2.1 is the active format.** GraphML 1.0 is **legacy — inactive tools only**; an active tool must target YAML. (`valid-memtrack` violations are the one exception still needing 1.0.)
- **Violation witnesses are required everywhere for FALSE.** Structure: `violation_sequence` → segments → waypoints (`assumption` / `branching` / `function_enter` / `function_return` / `target`); the final segment holds the single `target` waypoint at the exact `(file,line,column)` of the violation. Fix `__VERIFIER_nondet_*` inputs and branch directions with `assumption`/`branching` waypoints so bounded validators can replay within 90 s. Metadata must include exact SHA-256 `input_file_hashes`, `specification`, `data_model`.
- **Correctness witnesses (`invariant_set` with `loop_invariant`/`location_invariant`) are required only for `unreach-call` (non Arrays/Floats/Heap) and `no-overflow`.** For **memory-safety, memcleanup, data-race, a sound TRUE scores +2 with *no witness at all*** — a free-points shortcut if SAF can prove memory safety.
- **`witnesslint` must pass** or a required witness is treated as absent. Confirm with at least one of {CPAchecker, UAutomizer, MetaVal, Theta} (violation also: Witch).

### 1.5 Participation mechanics (SV-COMP 2026)

Four artifacts (no paper needed to compete): (a) **Zenodo tool archive** (self-contained ZIP, `smoketest.sh`, LICENSE permissive for evaluation, README, `--version`, ≤2 MB stdout); (b) **fm-tools YAML** entry with a DOI, a jury member, and labels — **`ai` if it uses an LLM/other sophisticated AI**, `meta_tool` if it composes other tools; (c) **BenchExec tool-info Python module** (translates tool output → TRUE/FALSE/UNKNOWN); (d) **`category-structure.yml` merge request** declaring participation/opt-outs. Track = **"Verification"**.

### 1.6 Rules on ML / LLMs / fingerprinting (verified)

- **ML/LLMs are explicitly permitted** — you self-declare the `ai` label. No rule prohibits ML, restricts training data, or bars training on the public `sv-benchmarks` (there is a formal **train/eval split**; PeSCo has trained on it and shipped weights since 2019). *(The "≤20% of benchmark for ML training" rule some summaries cite does NOT exist in the 2026 rules — do not rely on it.)*
- **Anti-fingerprinting (verbatim):** *"The verifier should not use identifiers nor comments contained in the verification task to fingerprint and identify individual tasks."* *"forbidden from using the program name, its hash, or the current category to tune their parameters."* The **organizer may obfuscate code and rename local identifiers.**
- **Sanctioned adaptivity (verbatim):** *"It is acceptable to use occurrence of calls to extern functions from standard libraries (for example malloc, pthread_create …) to detect which feature a verification task uses and change the verifier behavior."* ⇒ feature-detection on **library-call structure** is allowed; keying on **identifiers/comments/names** is not (and is defeated by obfuscation).

---

## Part 2 — What we learn from the 2026 winners

### 2.1 Podiums (verified against report Table 13)

| Category | 🥇 | 🥈 | 🥉 |
|---|---|---|---|
| **C.Overall** | UAutomizer | CPAchecker | Symbiotic |
| **C.TrueOverall** (proving) | UAutomizer | Goblint | Mopsa |
| **C.FalseOverall** (bug-finding) | **Symbiotic** | CPAchecker | UAutomizer |
| ReachSafety | CPAchecker | ESBMC-kind | Symbiotic |
| MemSafety | **Symbiotic** | CPAchecker | UAutomizer |
| Concurrency | Deagle | Dartagnan | iekke (new) |
| NoOverflows | UParalizer (new) | UAutomizer | UTaipan |
| Termination | PROTON | UAutomizer | AProVE |
| SoftwareSystems | Mopsa | Symbiotic | CPAchecker |

Notable: **Goblint has ~zero wrong answers and is the fastest tool** (the sound-AI archetype). **CPAchecker had 0 wrong proofs** in C.Overall (soundness discipline). The C.Overall top-3 are all **multi-engine prove‖refute portfolios**. Pure algorithm-selectors (PeSCo, Graves-CPA) are **excluded from official rankings as meta-tools** — selection must live *inside* an active tool.

### 2.2 The eight highest-leverage lessons for SAF (ranked)

- **L1 — Be sound-by-construction on the TRUE side; answer UNKNOWN, never guess.** Goblint/Mopsa win by over-approximating and refusing to guess. SAF must *never* infer TRUE from "bug search found nothing"; a TRUE must come from a sound over-approximation whose fixpoint excludes the error. The `C.TrueOverall` reservoir (max 48,413) dwarfs `C.FalseOverall` (max 12,195), but a single wrong proof is −32.
- **L2 — Adopt a two-engine prove‖refute split** (every Overall medalist has one). SAF's pieces map naturally: **abstract interpretation / VFG = the prover (over-approx → TRUE)**; **symbolic/path exploration over the VFG = the refuter (under-approx → FALSE)**; take whichever fires first.
- **L3 — Witnesses are not optional; an unvalidated correct answer scores 0.** Emit **YAML 2.0** violation witnesses first (SAF's strength), then correctness witnesses from the AI fixpoint's invariants.
- **L4 — Replay/cross-check every FALSE before emitting it** (kills the −16 tax). Symbiotic *replays the error on the unsliced code*; CPAchecker uses a bit-precise counterexample checker (both ≤4 false alarms). Contrast new tool iekke: 44 false alarms.
- **L5 — Build an *in-tool* sequential strategy portfolio over the 15-min budget** (cheap sound AI → symbolic refutation → escalate precision), but keep it inside SAF (external selectors are excluded from rankings). Prefer sequential over parallel under a 4-core cap.
- **L6 — Strengthen the abstract domain toward relational + heap.** ESBMC's cautionary tale: non-relational **interval** invariants *"trebled incorrect proofs."* Mopsa wins SoftwareSystems with octagons/polyhedra + recency-heap abstraction. SAF already *has* an octagon domain — it's just unwired.
- **L7 — Exploit slicing to fit the budget — SAF's PTA/VFG makes this cheap.** Symbiotic's decisive scalability lever is property-directed slicing driven by pointer analysis. SAF has PTA + VFG already.
- **L8 — Add a k-induction / IMC rung as a cheap first prover**, seeded with SAF's AI invariants (kIkI insight), plus a **witness self-check** before submitting.

**Roadmap ordering implied by the winners:** (1) UNKNOWN-not-guess soundness → (2) confirmed violation witnesses so existing FALSE strength scores → (3) sound TRUE from AI + correctness witnesses → (4) slicing + k-induction → (5) in-tool sequential portfolio → (6) relational/heap domains. This is the Symbiotic→CPAchecker→Goblint/Mopsa capability ladder, walkable from SAF's LLVM-IR PTA+VFG+AI core.

---

## Part 3 — Where SAF stands today (honest gap map)

### 3.1 The two-pipeline problem (the #1 finding)

SAF has **two parallel, disconnected verdict pipelines**, and the CLI runs the weaker one:

- **Live (subprocess) path** — `SvCompRunner::run` → `run_task` → `saf run --bench-config` → `bench_result_to_verdict` (`crates/saf-bench/src/svcomp/mod.rs:335-455`). This is what `make test-svcomp` uses. It derives `unreach-call`/`no-overflow` verdicts from `assertion_results`, which the driver populates **only for `svf_assert(...)`** (a PTABen convention; `crates/saf-cli/src/driver.rs:1387`). Real SV-COMP tasks call `reach_error()`/`__VERIFIER_error()` — **never `svf_assert`** — so `assertion_results` is empty and the headline path returns **UNKNOWN** on essentially every real ReachSafety/Overflow task.
- **Dead (in-process) path** — `analyze_property`/`analyze_unreachability` (`crates/saf-bench/src/svcomp/property.rs:385,461`) *correctly* keys on `reach_error`/`__VERIFIER_error`, has fast-paths, temporal UAF filtering, and real aggressive/conservative logic (plans 052–056) — but has **no production caller** (only a `pub use` re-export + unit tests).

> **Highest-leverage single change:** reconnect `run_task` to `analyze_property_with_context` (in-process, shared `AnalysisContext`) instead of the `svf_assert` subprocess oracle. This is also the prerequisite for the aggressive/conservative flag to have any real effect.

### 3.2 SAF is structurally a bug-finder (good at FALSE, unsound for TRUE)

- **TRUE = absence-of-findings**, both in the live path (`mod.rs:347-455`) and via under-approximate detectors. The memsafety checkers are *bug detectors* (Juliet recall ≈68% ⇒ ~1/3 of bugs missed); plan 148's `may_reach_guarded` is explicitly under-approximate. Missing a bug ⇒ unsound TRUE ⇒ −32.
- **Sound-ish components that exist:** interval AI (`absint/{interval,transfer,fixpoint,interprocedural}.rs`, over-approximate by design **but scalar-only and with ~137 known unsound PTABen cases**), the `svf_assert` condition prover (sound direction, narrow), Z3 simple-path reachability (sound-for-UNSAT but loop-truncated), SCCP (sound, wired).
- **Sound proving machinery that exists but is *unwired / feature-gated / dead*:**
  - **Octagon relational domain** (`absint/octagon/`) — implemented, **not referenced by any solver** (orphaned).
  - **CEGAR** (`cegar/`) — behind `#[cfg(feature="analysis-cegar")]`, only test-stub implementors, **zero production callers**.
  - **Loop-invariant synthesis** (`invariants/`) — behind `#[cfg(feature="analysis-invariants")]`, **no callers**.
  - **GraphML witness emitter** (`svcomp/witness.rs`) — complete-ish but **dead** (zero non-test callers) and targets the **deprecated** format.

### 3.3 Competition packaging — mostly absent

| Gap | State | Evidence |
|---|---|---|
| **BenchExec entry point** (`saf verify --property X.prp file.c --data-model LP64` → prints `true/false/unknown`) | **ABSENT** | `commands.rs:327-346`; harness is self-grading, reads `expected_verdict` from YAML |
| **BenchExec tool-info Python module** | **ABSENT** | no `benchexec/tools/*.py` anywhere |
| **In-tool C→LLVM IR** (clang) pipeline | **ABSENT** — compilation is 100% offline (`scripts/compile-svcomp.sh`); frontend ingests only `.ll`/`.bc` | `crates/saf-frontends/src/llvm/mod.rs` |
| **`.prp` property parsing** | **ABSENT** — property inferred from *filename substring* | `task.rs:86-109` |
| **`__VERIFIER_assume` modeling** | **ABSENT** (stub decl only) — soundness + precision hazard | `sv-comp-stubs.h:16` |
| **Per-task time/memory governance & portfolio** | **ABSENT** — `timeout_secs` parsed but never enforced; only Z3 per-query timeout | `mod.rs:79,94` (unused), `runner.rs:42-50` |
| **YAML witness 2.0 emitter** | **ABSENT** (only dead GraphML) | `witness.rs` |
| **Floats in numeric domains** | **ABSENT** (→ TOP) | `transfer.rs:106-109,798-802`; `sccp.rs` |
| **Termination / provable no-data-race TRUE** | **ABSENT** beyond loop-free/single-thread fast paths | `task.rs:126-135`; `property.rs:1400-1469` |
| **Determinism assets** | **PRESENT** (byte-deterministic, BLAKE3, BTreeMap) — good for reproducibility; but witness carries a wall-clock timestamp and verdicts depend on Z3 wall-clock timeouts | `witness.rs:238-262`; `property.rs:533-552` |

### 3.4 Extensibility for a learned controller (for judging `ideas.md`)

- **~40 correctness-preserving knobs exist** (k-CFA `k`, field/flow sensitivity, PTA solver worklist/Datalog, PTS representation, DDA budget, widening thresholds, index sensitivity, SMT toggles, checker selection, aggressive/conservative) — but **all are per-run-global**. There is **no per-function / per-object precision scoping** anywhere.
- **The numeric-domain choice (interval vs octagon) is NOT tunable from any interface** — it's hardcoded by which function is called.
- **No ML/LLM/advisor hook exists** (one aspirational doc comment). Three transport channels an external controller could use *today*: **Python SDK kwargs** (richest), the **`--bench-config` JSON** contract, and the **`--serve`/`Project.request()` JSON** protocol.
- The **`cegar::Abstraction` struct** (`cegar/abstraction.rs:73-122`: `pta_k`, `flow_sensitive`, `predicates`, `tracked_vars: BTreeSet<ValueId>`, `strong_updates`, `field_sensitive`, with `initial()`/`precise()`/`refine()`) is the only unified "precision vector" and the only per-object mechanism — but it's unwired.

**Implication:** a learned controller that picks **whole-program presets** is *low plumbing* (reuse Python kwargs / bench-config JSON). Anything **per-function/per-object** is *high plumbing* (must thread a precision map through every solver — infrastructure that doesn't exist).

---

## Part 4 — Critical evaluation of the `ideas.md` small-LLM proposals

`ideas.md` is a thoughtful ranking of ~20 small-LLM roles for SAF. Its **guiding principle is exactly right and must be preserved:**

> *The LLM may choose, rank, propose, or prioritize. SAF/SMT must validate anything that affects a TRUE/FALSE verdict.*

This keeps LLM non-determinism/imperfection off the soundness path — at worst it costs *completeness* (a missed proof = 0 points), never *correctness* (a wrong verdict = −16/−32). Good.

But the document has **five blind spots** that reorder its priorities:

**C1. It is premature relative to SAF's actual bottleneck.** Every idea assumes SAF is "a verifier with knobs the LLM tunes." It isn't yet: no harness, no witnesses, no sound TRUE, and the sophisticated analyzer is dead code (Part 3). **An advisor for a verifier that can't be run and can't emit witnesses adds zero SV-COMP points.** The foundational P0/P1 work below must precede *all* LLM work. `ideas.md`'s own "Phase 1" (selection/ranking) is only meaningful once sound engines + real knobs + a scheduler exist.

**C2. The shared, cross-core CPU budget makes many patterns infeasible.** 900 s is *summed across 4 cores* (~225 wall-s of 4-thread compute) and *shared with the verifier*; a 0.5–1B Q4 model does single-digit-to-low-tens tok/s on CPU, prefill of real C is compute-heavy, and **model load recurs per task** (fresh container). Therefore:
  - ✅ Viable: **one or two forward passes per task, single/low-thread, hard cap of a few tens of CPU-seconds**, producing a hint a fast sound checker disposes.
  - ❌ Infeasible: per-alarm loops over 200 alarms as 200 generations (idea #8 as literally written), best-of-N, multi-round agentic loops, or the LLM as the decision procedure. These starve the verifier of the budget it needs.
  - A ranking task (idea #8 alarms, #19 invariants) is only viable as **one batched scoring pass**, not N generations.

**C3. For pure *selection/ranking*, a tiny classifier/GNN beats an LLM** — cheaper, faster, deterministic, and with real SV-COMP precedent (PeSCo since 2019, Graves-CPA). The LLM's *unique* value is **language/semantic generation**. So `ideas.md`'s LLM framing is over-applied to classification tasks. Re-map: strategy selection (#1), alarm ranking (#8), SMT/tactic selection (#11), difficulty/time allocation (#12) → **small classifier over CFG/DFG/library-call features**, not an LLM. Reserve the LLM for **invariant/predicate generation** (#4, #10), where evidence (Lemur, Quokka) is strongest and generation is genuinely needed.

**C4. The name-based ideas (#7 "exploit function/variable names", parts of #17) are *legitimate but bounded* — not "bonus," not a pillar.** This deserved more than a one-line demotion, so it now has a dedicated deep dive (**§4.2**). In short: identifier names *do* carry real, generalizable semantics (the ML-for-code literature is clear on this — your intuition is right), and the "names propose, a sound checker disposes" pattern is the *field-standard* design (Typilus, NullGTN, Lemur, Loopy). But for SV-COMP specifically three dampeners apply: (a) the anti-fingerprinting rule forbids using identifiers to *identify tasks* and the organizer may rename locals + obfuscate; (b) only ~15–30% of distinct C programs have meaningful names, and SAF's own `mem2reg` pipeline strips most *local* names; (c) the signal is defeasible (must degrade to a name-free fallback). So names belong as a **robustness-hedged, sound-checked hint channel** feeding proposal/ranking — never a verdict input, never a task-recognizer. The **sanctioned** structural signal (extern library calls: `malloc`, `pthread_create`) is complementary and survives obfuscation.

**C5. Determinism of CPU inference is a real engineering constraint it omits.** Greedy decoding is only bit-reproducible with pinned thread count, fixed GGML/BLAS build, temperature 0, fixed seed (float reductions are non-associative). A verdict that flips between runs is a scoring hazard. Budget for single-thread deterministic inference.

### 4.1 Verdict per idea (feasible / legal / valuable *for SAF specifically*)

| # (ideas.md) | Idea | Feasible on SV-COMP HW? | Value for SAF | Verdict |
|---|---|---|---|---|
| 1 | Strategy/config selection | ✅ (tiny model) | Med–High **once knobs+portfolio exist** | **Do it — as a small classifier, in-tool.** Not an LLM. Prereq: P2 portfolio. |
| 8 | Alarm/counterexample ranking | ✅ if batched | Med (speed → fit budget, tie-break) | **Do it — small ranker.** Helps FalseOverall throughput, not correctness. |
| 11 | SMT solver/tactic selection | ✅ | Low now (SAF has only Z3) | Defer until multiple solvers exist. |
| 12 | Difficulty / time allocation | ✅ | Med (feeds the scheduler) | **Hand-tune the scheduler first**, learn later. |
| 4, 10, 19 | Loop-invariant / numeric-predicate generation & ranking | ✅ if time-boxed | **High — converts high-value UNKNOWN→TRUE** and *produces the correctness-witness invariants SAF needs anyway* | **The best LLM bet.** LLM proposes, absint/k-induction disposes. Strongest evidence (Lemur, Quokka). Prereq: invariant-checking loop wired (P2). |
| 3 | CEGAR refinement selection | ✅ | High potential | Contingent on CEGAR being wired with real analyses (currently test-stub only). Later. |
| 2, 5 | Per-function precision / heap-abstraction selection | ⚠️ | High **but** needs per-function/object scoping that **doesn't exist** | High plumbing — defer until the precision-map infra exists. |
| 6, 18 | Function summaries / external-fn contracts | ✅ | Med (SoftwareSystems) | Later; needs summary-checking infra. |
| 7 | Exploit identifier/variable names | ✅ | **Bounded** (see §4.2) | **Reframe, don't reject.** Use as a *hint channel* for proposal/ranking (name→role prior seeding invariants; name-aware alarm ranking), never a verdict input or task-recognizer; always with a name-free fallback. Prefer sanctioned extern-call structure for feature-detection. |
| 13 | Witness assistance | ⚠️ | Low | Deterministic witness generation from a real trace beats LLM. Skip. |
| 14 | Direct verdict prediction | — | **None** (unsound) | **Reject** (ideas.md agrees). |
| 15/16/20 | Slice/inlining/widening guidance | ✅ | Med | Later refinements once the sound core + scheduler exist. |

**Net:** `ideas.md`'s design principle is sound and its *set* of ideas is reasonable, but its **priority is inverted** for SAF's situation. The right sequence is *foundations first, classical-ML selection second, one LLM invariant-generator third* — not "build the SAF-0.5B advisor first." And the single "AI" thing most worth building is an **LLM loop-invariant generator feeding a sound checker**, because it uniquely needs generation, has the best evidence, targets the biggest score reservoir (TrueOverall), and *doubles as the correctness-witness source*.

### 4.2 Deep dive: can identifier names complement SAF? (a bounded "yes")

**Motivating intuition (the user's):** LLMs learn code semantics from identifier names, so names could help SAF. **Verdict: partly true and worth exploiting — in a specific, sound-checked, robustness-hedged role — but the SV-COMP payoff is smaller than intuition suggests, and it is a complement, never a pillar.**

**(1) The signal is real and generalizable — but noisy and defeasible.** The ML-for-code literature confirms names carry genuine semantics, strongest exactly for SAF-relevant properties:
- **Type** is the most name-predictable: NL2Type top-1 84% / top-5 95% (names+comments alone beat implementation analysis); TypeWriter ~73%/92%; Typilus confidently types 70% of symbols and those type-check 95% of the time.
- **Nullability**: NullGTN reaches ~89% recall of human `@Nullable` annotations.
- **Buffer *role* (len/size/index/count/capacity)**: recoverable at the *role* level even when the exact name isn't — VarBERT's own title is *"len or index or count, anything but v1"*. (No clean peer-reviewed accuracy for "is this a non-negative length?" — LOW-CONFIDENCE; role is more predictable than identity.)
- **Ownership/aliasing role**: weakest, essentially unquantified — treat as speculative.
- **Fragility (the key caveat):** semantic-preserving *renaming* flips model predictions with success rates ~8% (robust: clone/vuln classification) to ~94% (fragile: summarization); decompiled-name recovery collapses **66%→37%** on *unseen* functions (DIRTY) — the empirical face of "generalizable prior vs memorized fingerprint." (Correction to a common myth: there is *no* reliable "70% drop from renaming" figure; renaming sometimes even *helps* — it is the *instability*, not a fixed magnitude, that disqualifies names from the soundness path.)

**(2) "Names propose, a sound oracle disposes" is the field-standard design, not a compromise.** Every mature system keeps name/NL signal on the propose/rank side and a sound checker on the decide side — exactly SAF's invariant:
- *Invariant synthesis:* Lemur (solves 107 vs 68 for ESBMC-alone), Loopy (398/469; solves cases Ultimate can't), iRank (median rank of a *verified* invariant 62→4, fewer Z3 calls), Quokka (a Decision-Soundness theorem) — all feed raw source *with identifiers* to an LLM and let CBMC/ESBMC/UAutomizer/Houdini validate.
- *Alarm ranking:* Bingo, BayeSmith (17/20 programs, 21–33% fewer inspections), Z-ranking (2–8×) — reorder-only, **zero soundness impact**.
- *Spec/type/nullness/units:* Typilus/TypeWriter (checker-gated types), NullGTN (name-augmented AST + NullAway rejects wrong guesses), SA4U (names as *probabilistic* unit hints + dimensional check), Jdoctor (NL→contracts 92% precision, dynamically checked).
- *Gap = opportunity:* no peer-reviewed work steers symbolic execution / path exploration by *identifier names specifically* — a name prior over which VFG paths/alarms to explore first would be **novel and soundness-safe by construction**.

**(3) SV-COMP-specific dampeners.**
- *Legal boundary (verbatim):* forbidden to "use identifiers nor comments … to fingerprint and identify individual tasks"; "forbidden from using the program name, its hash, or the current category to tune"; **one global parameter set**. The organizer "is allowed to … rename local identifiers (with the exception of extern functions and an allowlist of common identifiers)." Only **extern-library-call feature detection** is an explicitly sanctioned surface read. ⇒ names may *guide* but must never *decide* or *identify*, and any per-task strategy switch keyed on name is banned.
- *Coverage ceiling:* only ~**15–30% of distinct C programs** carry human-meaningful identifiers, concentrated in **SoftwareSystems** (BusyBox, coreutils, AWS-C-Common, Linux drivers, uthash) and **MemSafety/heap** families; the bulk (ECA, ProductLines, Combinations, crafted Arrays/Loops, Floats, Juliet) is synthetic/opaque. Worse, the name-rich programs are the ones the organizer is *most likely to obfuscate*.

**(4) Feasibility in SAF is good — the architecture already fits.** SAF **preserves names as an ignorable side channel** (`Instruction.symbol`, `AirParam.name`, `AirFunction.name`, `AirGlobal.name`, `StructField.name` — `crates/saf-core/src/air.rs`), while analysis identity and solver decisions run on **name-free BLAKE3 `ValueId`s** derived from position (renaming a local doesn't even change a `ValueId`). Names today feed only the `DisplayResolver` and finding-reporter — exactly the clean separation a sound hint channel needs. Four sound plug-in sites (each: name changes only proposal/selection/ranking; a name-blind validator guarantees correctness):
  - **(a) Invariant/predicate *proposal*** → `crates/saf-analysis/src/invariants/templates.rs` (add/reprioritize candidates from name→role priors), validated by `check_invariant`/absint in `invariants/{mod,checker}.rs` — a wrong name guess is *provably discarded*.
  - **(b) Alias/points-to *question selection*** → DDA/PTA query APIs (`dda/solver.rs`, `pta/result.rs`): names steer *which* alias questions to ask; the answer is name-independent (sound CI-PTA fallback on budget exhaustion).
  - **(c) Alarm/finding *ranking*** → `crates/saf-analysis/src/checkers/` (post-dedup reorder before export): **lowest-risk first target**, zero soundness impact.
  - **(d) Proposed-relation *validation*** → `z3_utils/condition_prover.rs`: a name may *propose a relation to prove*; only `Proven` (decided by absint) is accepted.
  - *Coverage caveat:* SAF attaches local-variable names only to `alloca`s, so **`mem2reg` (standard in SV-COMP pipelines) strips most local names**; function/param/global/struct-field names survive. LLVM 19+ `DbgRecord` and Rust debuginfo are not read (plan 186 follow-ups). Fix = frontend work (attach names pre-`mem2reg` / read `DbgRecord`); a missing name simply means "no hint," never unsoundness.

**(5) What to actually build (and what model).**
- **Do-first, zero-risk:** name-aware **alarm ranking** (P3.3) — but note its SV-COMP value is limited: a single confirmed FALSE scores regardless of rank, so the win is only *fitting within budget* + the success-runtime tie-break.
- **Highest-value:** name priors as an **extra input channel to the invariant/predicate *generator*** (P3.1) — a `len`→non-negative, `i < len`, `off + size ≤ cap` prior *seeds* candidates that absint/k-induction must independently confirm. This is where names + generation jointly matter and where your intuition pays off (converts UNKNOWN→TRUE, and the confirmed invariant *is* the correctness witness).
- **Cheaper than an LLM for the labeling sub-task:** name→role can be a **curated lexicon** (`len|size|count|n|idx|cap|buf|dst|src|head|next` → role) or a tiny VarCLR-style embedding classifier — deterministic, ~free on CPU, obfuscation-gateable. Reserve the LLM for generating the actual invariant *expressions*.
- **Guardrails (non-negotiable):** (i) names never influence a TRUE/FALSE verdict — only propose/rank; (ii) never fingerprint/identify a task or switch strategy on a name (rule violation); (iii) always a name-free fallback carrying 100% of the soundness burden; (iv) prefer sanctioned **extern-call structure** over variable names for any behavior change; (v) validate the whole component under a **local-identifier-scrubbing harness** so obfuscation only costs *completeness*, never soundness or a regression.

---

## Part 5 — Prioritized roadmap of improvements worth trying

Legend — **Effort:** S/M/L/XL. **Risk:** Low/Med/High. **Soundness:** 🛡️ improves/guards soundness · ⚖️ neutral · ⚠️ soundness-sensitive (must be gated by a sound check).

### P0 — Make SAF *runnable* by SV-COMP at all (no points possible without these)

| # | Improvement | Effort | Risk | Soundness | Notes / SAF pointers |
|---|---|---|---|---|---|
| P0.1 | **BenchExec verifier CLI**: `saf verify --property <.prp> --data-model {ILP32,LP64} <file.c\|.i>` printing exactly `true`/`false`/`unknown` (+ writing `witness.yml`), blind to expected verdict | M | Low | 🛡️ | New subcommand in `crates/saf-cli`; must **not** read `expected_verdict`. Parse the `.prp` `CHECK(... LTL ...)` form, not the filename. |
| P0.2 | **BenchExec tool-info module** (`benchexec/tools/saf.py`): version, cmdline, output→result mapping | S | Low | ⚖️ | Required competition artifact. |
| P0.3 | **In-tool C→LLVM IR**: invoke bundled `clang` on `.c`/`.i` with `-m32/-m64` from data model, `-include sv-comp-stubs.h`, `mem2reg` | M | Med | 🛡️ | Bundle clang + stubs in the archive (self-contained, offline). Reuse `scripts/compile-svcomp.sh` logic in-process. |
| P0.4 | **Per-task budget & scheduler skeleton**: enforce a CPU/wall cap, run analyses under a deadline, always terminate with `unknown` rather than crash/timeout | M | Med | 🛡️ | `timeout_secs` currently unused (`mod.rs:79`). Sequential phases summing < 900 s CPU. |
| P0.5 | **Reconnect the dead analyzer**: route the verifier to `analyze_property_with_context` (handles `reach_error`), retire the `svf_assert` oracle path for SV-COMP | M | Med | 🛡️ | The single highest-leverage code change (Part 3.1). |
| P0.6 | **Model `__VERIFIER_assume` / `__VERIFIER_nondet_*` / `reach_error`** in the analysis core (not just stubs) | M | Med | 🛡️ | `assume` unmodeled today = false alarms + can't prove TRUE. |
| P0.7 | **Self-contained *native* ZIP archive** (not an OCI image): `--release` build, static libLLVM or bundled `.so`s + loader, `share/saf/specs` beside the binary, `smoketest.sh` (exit 0), `LICENSE` + `THIRD-PARTY-LICENSES`, one top dir, no `.git`/`target`/sources, ≤2 MB stdout, runs from any path writing only cwd//tmp | M | Med | ⚖️ | **OCI/Docker is NOT an accepted submission** and BenchExec can't run it (§7.2). Today SAF ships a Docker image with a *debug* binary dynamically linked to `libLLVM-18.so`. |
| P0.8 | **Determinism/reproducibility pins** (see §7.1): pin/remove verdict-affecting env toggles `SAF_PTA_FIELD_MINTING` (`cg_refinement.rs:297`) & `SAF_DECOMPOSE_POINTER_ARRAYS` (`mapping.rs:131`); resolve specs binary-relative only (ignore `$HOME`/CWD/`$SAF_SPECS_PATH`, `registry.rs:226`); route the CLI `tracing` fmt layer to **stderr** (`saf-cli/src/main.rs:12`, currently stdout — pollutes the verdict); verdict-only stdout + full report to `--output` | S–M | Low | 🛡️ | Cross-machine verdict stability + clean BenchExec parse. |

### P1 — Score points where SAF is already strong (FALSE / MemSafety), safely

| # | Improvement | Effort | Risk | Soundness | Notes |
|---|---|---|---|---|---|
| P1.1 | **YAML witness 2.0 *violation* emitter** + `witnesslint` self-check + self-validate against CPAchecker/UAutomizer | M | Med | 🛡️ | **Highest single ROI.** Without it every found bug scores 0. Emit `target` waypoint at exact `(file,line,col)` + `assumption`/`branching` waypoints fixing `nondet` inputs. Replace dead GraphML in `witness.rs`. |
| P1.2 | **Replay/cross-check every FALSE** before emitting (SMT-feasibility on the VFG path or concrete replay); else downgrade to UNKNOWN | M | Med | 🛡️ | Kills the −16 tax (Symbiotic/CPAchecker discipline; L4). |
| P1.3 | **Compete in `C.FalseOverall` + `C.MemSafety` first**; **never emit TRUE unless sound** — map "no bug found" to UNKNOWN by default | S | Low | 🛡️ | Strategy/config choice. Removes all −32 exposure. |
| P1.4 | **Free memory-safety TRUEs**: where SAF's memsafety analysis is genuinely sound on simple heap/stack code, emit TRUE — **no correctness witness required** in MemSafety (+2 for free) | M | High | ⚠️ | Only for cases provably sound; gate hard. Big reward but the −32 landmine — needs P2.x soundness gate. |
| P1.5 | **"Unmodeled-construct ⇒ UNKNOWN" gate**: any float guard, unresolved indirect call, `assume` we can't honor, or truncated loop path forces UNKNOWN instead of a TRUE fallthrough | S | Low | 🛡️ | Converts silent −32s into safe 0s. |

### P2 — Grow a *sound* TRUE side (the big score reservoir) + portfolio

| # | Improvement | Effort | Risk | Soundness | Notes |
|---|---|---|---|---|---|
| P2.1 | **Sound TRUE from the AI fixpoint** for `no-overflow`/`unreach-call`: when the over-approximated fixpoint excludes the error, emit TRUE + **correctness witness** (invariants from interval/octagon at loop heads) | L | High | ⚠️ | Fix the ~137 known interval unsoundnesses first; conservative-only path. Feeds P2.2. |
| P2.2 | **Wire the octagon (relational) domain** into the fixpoint/interproc solvers; add a domain-selection knob | M | Med | 🛡️ | Octagon exists but orphaned (`absint/octagon/`). Relational facts fix ESBMC's "interval trebles wrong proofs" problem (L6). |
| P2.3 | **k-induction rung** (seeded with AI invariants) as a cheap prover before heavier analysis | L | Med | 🛡️ | kIkI/ESBMC-kind precedent (L8); combines with P2.1/P2.2. |
| P2.4 | **Property-directed slicing** (cone-of-influence via PTA+VFG) before prover & refuter | M | Med | ⚖️ | SAF already has PTA+VFG — Symbiotic's key budget lever (L7). |
| P2.5 | **In-tool sequential strategy portfolio**: cheap sound AI → symbolic refutation → escalate precision (relational domains, higher k) under per-phase **deterministic step/instruction budgets** (NOT wall-clock races) | L | Med | 🛡️ | Keep it *inside* SAF (external selectors are excluded, L5). Hand-tuned schedule first. **Determinism (§7.1):** fixed phase order; a phase that exceeds its *step* budget yields UNKNOWN deterministically; the first *sound* result in fixed order wins — never "whichever finishes first" (that flips verdicts across CPU speed). |
| P2.6 | **Float interval/relational support** (or explicit UNKNOWN on float-controlled properties) | M | Med | 🛡️ | Today floats → TOP silently (soundness hazard). |
| P2.7 | **Wire loop-invariant synthesis + correctness-witness generation** (templates + AI validation) | M | Med | 🛡️ | `invariants/` exists but feature-gated/unwired; feeds P2.1 witnesses and P3.1. |

### P3 — Learned components (classical ML first; one LLM bet)

| # | Improvement | Effort | Risk | Soundness | Notes |
|---|---|---|---|---|---|
| P3.1 | **LLM loop-invariant / numeric-predicate *generator*** (≤1B Q4, single-shot, ≤~30 CPU-s cap, offline, deterministic) → candidate C-expression invariants → **SAF absint/k-induction validates** → sound TRUE + correctness witness. **Feed identifier names as one input channel** (name→role priors, §4.2) with a name-free fallback | L | Med | ⚠️→🛡️ | The best-justified AI feature (Lemur/Quokka evidence; §C3/§4.2). Untrusted proposal, sound disposal. Declare the `ai` fm-tools label (mandatory). **Determinism (§7.1):** single-thread, greedy/temp-0, fixed seed, fixed GGML/BLAS build, no fast-math. **License (§7.4):** bundle only Apache-2.0/MIT weights (Qwen2.5-0.5B/1.5B, SmolLM2, Phi-3.5-mini, TinyLlama, OLMo, Pythia); **NOT** Llama-3.2 / Gemma-2 / Qwen2.5-3B. Doubles as the P2.1/P2.7 witness source. |
| P3.2 | **Tiny in-tool strategy selector** (GNN/gradient-boosted over CFG/DFG/library-call features; trained on `sv-benchmarks` outcomes) picking the P2.5 schedule/preset | M | Med | ⚖️ | PeSCo/Graves precedent; cheaper & more deterministic than an LLM (§C3). Features must be *structural*, never identifiers (§C4). |
| P3.3 | **Alarm/counterexample ranker** (small model, one batched scoring pass), **name-aware** (§4.2) to reach a confirmable FALSE within budget | M | Low | ⚖️ | Helps `C.FalseOverall` throughput + success-runtime tie-break (§C2). Lowest-risk name-hint site (post-dedup reorder in `checkers/`). Value capped: a confirmed FALSE scores regardless of rank — win is fitting-in-budget + tie-break. |
| P3.6 | **Name→role prior** (curated lexicon `len\|size\|count\|idx\|cap\|buf\|dst\|src\|next…` → role, or tiny VarCLR-style classifier — cheaper & more deterministic than an LLM) feeding P3.1 proposal + P3.3 ranking; extern-call feature-detection for sanctioned adaptivity | S–M | Low | ⚖️/⚠️ | §4.2. Guardrails: never a verdict input, never a task-recognizer, always a name-free fallback; validate under a local-identifier-scrubbing harness. Coverage limited by `mem2reg` stripping local names (frontend fix optional). |
| P3.4 | **Learned time-allocator / difficulty predictor** feeding the P2.5 scheduler | M | Low | ⚖️ | Refinement on the hand-tuned scheduler. |
| P3.5 | **Per-function/per-object precision map** (extend `cegar::Abstraction`, thread through solvers) — precondition for LLM-guided *selective* precision (ideas #2/#5) | XL | High | ⚖️ | High plumbing; the missing infra that gates the most fine-grained ideas. Do last. |

**Explicitly NOT worth doing** (per rules/feasibility): remote/API LLMs (no network); GPU models (none); LLM as decision procedure (unsound); direct verdict prediction (unsound, useless); multi-round agentic LLM loops or best-of-N per task (starves the budget); **using identifier names to *identify/fingerprint* a task or switch strategy on program name/hash/category** (rule violation + obfuscation-defeated) — note this is distinct from using names as a *generalizable, sound-checked proposal/ranking prior*, which **is** worth doing (§4.2, P3.1/P3.3/P3.6); **letting any name-derived signal reach a TRUE/FALSE verdict** (unsound); targeting the deprecated GraphML witness format as an active tool.

---

## Part 6 — Concrete next steps, experiments, and open questions

### 6.1 Suggested first milestone (a *runnable, scoring* SAF)

A minimal end-to-end slice that could actually enter `C.FalseOverall`/`C.MemSafety`:
**P0.1 + P0.2 + P0.3 + P0.4 + P0.5 + P1.1 + P1.2 + P1.3**, then measure on the `sv-benchmarks` training set inside Docker on the VM (`ubuntu@cd-vm-15-ai-vm`). This is the tracer-bullet: read a `.prp` + `.c`, compile, analyze under a deadline, emit `false` + a validated `witness.yml` or `unknown` (never a risky `true`), and self-score.

### 6.2 Experiments to de-risk before committing

1. **Witness confirmation rate**: emit YAML 2.0 violation witnesses for SAF's current MemSafety/ReachSafety FALSE findings; run them through CPAchecker/UAutomizer validators; measure confirmation %. (Directly predicts `C.FalseOverall` score.)
2. **Soundness audit of the interval TRUE path**: quantify the ~137 PTABen unsound cases under the new UNKNOWN-gate (P1.5); target zero unsound TRUE before enabling any TRUE emission.
3. **LLM invariant micro-benchmark** (offline, then CPU-time-boxed): on a curated set of loop-bearing `unreach-call`/`no-overflow` tasks SAF currently returns UNKNOWN on, measure how many a ≤1B Q4 model's candidate invariants let SAF's absint/k-induction prove — and the CPU-seconds cost per task. (Replicates the Lemur/Quokka setup at the small-model scale.)
4. **Budget accounting**: measure model-load + prefill + decode CPU-seconds for the chosen model on the VM at the competition thread count; confirm the LLM stays under a strict fraction (e.g. ≤10%) of 900 s.
5. **Name-hint coverage & lift** (§4.2): (a) measure what fraction of `sv-benchmarks` values/functions retain meaningful names *after* SAF's `mem2reg` pipeline (expect most local names stripped); (b) A/B the P3.1 invariant proposer with vs without name→role priors on the name-rich subset (SoftwareSystems/heap) — measure extra UNKNOWN→TRUE; (c) run everything through a **local-identifier-scrubbing harness** to confirm scrubbing only lowers completeness, never soundness or the baseline verdict (the obfuscation-robustness gate).

### 6.3 Open questions / to re-check when SV-COMP 2027 rules drop

- Confirm 2027 keeps: scoring (+2/+1/−32/−16), the 900 s/4-unit/15 GB limits, no-network/no-GPU, YAML-2.x-as-active, the `ai` label, and the `C.FalseOverall`/`C.TrueOverall` meta-categories.
- Which categories SAF should **opt into vs opt-out** of (opt-out avoids diluting a meta-score with 0s).
- Whether to enter as a **single tool** or split submissions (e.g., a FALSE-focused config and a TRUE-focused config) — allowed if "conceptually/technologically" distinct.
- The exact validator set to target for confirmation (CPAchecker + UAutomizer cover the most).

### 6.4 Corrections to carry forward (from the verification pass)

- Report DOI is **`10.1007/978-3-032-22749-2_23`** (some notes had `…22774-4_23`).
- The `−4/−8` "old scoring" appears on rules.php **only inside a stale changelog block** (alongside "Ubuntu 12.04"); the authoritative live table is **−16/−32**. The "≤20% ML training" rule does **not** exist — ignore it.
- The falsification meta-category is **`C.FalseOverall`** (not the historical "FalsificationOverall").

---

## Part 7 — Rules-compliance matrix (must fully comply)

Verified verbatim against `rules.php` / `submission.php` / the fm-tools schema / BenchExec docs, cross-checked against the TACAS 2026 report and mapped to SAF's current state (file:line). **Headline:** SAF is **license-clean and legally shippable**, but **not yet a submittable competition tool** — the compliance gaps cluster in *packaging*, *the verifier I/O contract*, and *witnesses* (which coincide with the P0/P1 roadmap), plus a short list of *determinism/env* pins and (only if an LLM is added) *model-weight licensing* + *AI declaration* + *anti-fingerprinting* items. There is one genuinely surprising blocker: **an OCI/Docker image is not an accepted submission** — the tool must run natively.

### 7.1 Determinism & reproducibility

**Rule stance (verified):** SV-COMP has **no hard determinism rule** — the string "determin*" never appears as a behavioral requirement, there is **no fixed-seed rule**, and a verdict is even *allowed* to depend on machine speed at the CPU-time timeout boundary. What **is** mandatory: **reproducibility** ("Publish your tool archive and reference it here via DOI to ensure reproducibility"; self-contained, CI-tested, `smoketest.sh`) and **re-checkability** of results via **witnesses confirmed by an independent validator**. Nondeterminism is therefore a **scoring hazard**, not a disqualifier — but a real one: a single official run (no re-run safety net), asymmetric −16/−32 penalties, and annual reproduction reports (Twente 2023, Bajczi 2025) that *do* re-run tools, flag flips, and frame significant verdict nondeterminism as "a tool bug authors are expected to avoid." BenchExec reduces environment nondeterminism (core pinning, memory limit, L3-cache isolation, Turbo-Boost warnings) but does not guarantee bit-identical re-runs. **⇒ We make verdicts a deterministic function of (program, property, resource-envelope) by design.**

SAF's **live verdict path is already same-machine deterministic** (timeout-free abstract interpretation over `BTreeMap`/`BTreeSet`/`FxHashMap` + `i128` intervals, step-bounded fixpoints, BLAKE3 IDs, no RNG, no result-affecting threading). The genuine **cross-machine** hazards (must-fix, all in **P0.8**):

| Hazard | file:line | Verdict-affecting? | Fix |
|---|---|---|---|
| `SAF_PTA_FIELD_MINTING` env toggle changes points-to precision | `cg_refinement.rs:297` | **Yes (env)** | Pin one value in the competition build; assert unset at startup |
| `SAF_DECOMPOSE_POINTER_ARRAYS` changes AIR lowering | `mapping.rs:131` | **Yes (env)** | Pin; do not read env at runtime |
| Spec-registry search path keyed on `$HOME`/CWD/`$SAF_SPECS_PATH` → different function models load per machine | `spec/registry.rs:226-254` | **Yes (env)** | Resolve specs binary-relative only; log the resolved set |
| CLI `tracing` fmt layer writes to **stdout** (hardcoded `info`) → `warn!` on edge-case IR pollutes the verdict stream | `saf-cli/src/main.rs:12-30` | Parse hazard | Route to stderr (as `saf-bench`/`saf-trace` already do) |
| Z3 **wall-clock** `timeout` ms → `Unknown`; in `assertions.rs` a timeout becomes `proved:false` → **FALSE** | `z3_utils/solver.rs:108`, `reachability.rs:119`, `assertions.rs:238` → `mod.rs:349` | **Latent** (dead path today) | **Before wiring:** use Z3 `rlimit` (deterministic) not ms `timeout`; force timeout→UNKNOWN, never FALSE |
| Portfolio "whichever fires first" race (roadmap L2/L8) | design | Would be **Yes** | **P2.5:** fixed phase order + deterministic step budgets |
| LLM CPU inference nondeterminism | design (no code yet) | Kept off soundness path | **P3.1:** single-thread, greedy, fixed seed, fixed build; untrusted-proposer only |
| Witness `SystemTime::now()` timestamp | `svcomp/witness.rs:238` | **No (cosmetic)** | Fixed/omitted timestamp → byte-stable witness |

### 7.2 Packaging & submission (four artifacts)

| Requirement (verbatim, source) | SAF state | Fix |
|---|---|---|
| **Plain public ZIP on Zenodo, exactly one top dir, no tarbomb** ("archived in a ZIP file (.zip), which contains exactly one directory"); **OCI/Docker not accepted** — "BenchExec containers do not support OCI container images … tools must be able to run on the host system" | **Absent.** Build is Docker-only; artifact is an OCI image with a **debug** binary dynamically linked to `libLLVM-18.so` (`Dockerfile:188,191`) | **P0.7:** `--release`, static/bundled libLLVM, native ZIP |
| **Self-contained** ("all necessary libraries and external tools should be contained in the archive"); stock Ubuntu pkgs via `required_ubuntu_packages` | Not self-contained (dynamic libLLVM) | Bundle libs or list Ubuntu pkgs |
| **`smoketest.sh` exit 0** (CI merge gate: "merged only if … smoketest.sh … successfully passes (exit code 0)") | Absent | Add; must pass offline/in-container |
| **≤2 MB stdout/stderr** ("does not exceed an stdout/stderr limit of at most 2 MB") | `saf run --format json`/`export` can dump large graphs to stdout (`commands.rs:806`) | Verdict-only stdout; full report to `--output` |
| **`--version`** | Present (`commands.rs:309`) | ✓ |
| **No junk** (`.git`/`target`/sources/`__MACOSX`) | N/A yet | Strip when packaging |
| **Executable from any path; RW only in cwd//tmp** | `run` is cwd-clean; specs via `current_exe()`; `incremental` writes `.saf-cache` (avoid that path) | Keep `share/saf/specs` beside the binary |
| **`LICENSE` permissive for reproduction + evaluation, no output restriction** | MIT (`LICENSE:1`) ✓ | Add `THIRD-PARTY-LICENSES`/NOTICE for bundled deps |
| **fm-tools `<id>.yml`:** `id,name,input_languages(C/LLVM-IR),project_url,spdx_license_identifier,benchexec_toolinfo_module,fmtools_format_version:"2.0",maintainers[] (valid ORCID),versions[] (doi `10.5281/zenodo.N`, benchexec_toolinfo_options w/ `${witness}`, required_ubuntu_packages),competition_participations[] (competition,track:"Verification",tool_version,jury_member),labels` | Absent | Author it; needs a Zenodo DOI + an ORCID'd maintainer |
| **BenchExec tool-info module** (`benchexec/tools/saf.py`, PR'd + tested) — assembles argv, parses stdout→TRUE/FALSE(p)/UNKNOWN, reports version | Absent | Write + test per `doc/tool-integration.md` |
| **`category-structure.yml` MR** (opt-in/opt-out declaration) | Absent | Declare categories (opt out to avoid diluting meta-scores) |
| **Qualification:** jury member who is a contributing dev (or organizer-authorized); tool publicly available; **correct labels** are a qualification gate; multiple entries only if conceptually distinct | N/A yet | Nominate jury member; set labels honestly |

### 7.3 Verifier I/O contract

| Requirement (verbatim) | SAF state | Fix |
|---|---|---|
| Per-task non-interactive run → `(ANSWER, WITNESS, TIME)`; ANSWER ∈ {TRUE, FALSE(p), UNKNOWN}; **FALSE(p)** must name the violated property; **TRUE** carries no property | Harness is batch/self-scoring; no per-task blind verdict CLI | **P0.1** verify CLI |
| **Property file is one CLI parameter** ("One parameter defines the specification file"); parse the `.prp`, don't infer from filename | Infers from filename (`task.rs:86`) | Parse `.prp` |
| **Data model as a parameter** ("if the verifier has a parameter … it needs to be defined"); ILP32/LP64 per category | Only via bitcode datalayout offline | Add `--data-model`; tie to LLVM target |
| **Preprocessing internal**; witnesses reference the **un-preprocessed `.c`** via `#line` | Compilation is offline (`compile-svcomp.sh`) | **P0.3** in-tool clang preserving `#line` |
| **One `witness.yml` (YAML 2.0/2.1)**; GraphML 1.0 legacy/inactive-only; per-category format table; **witnesslint must pass**; **≥1 validator must confirm**; small enough to validate in 90 s/300 s | Dead **GraphML** emitter, wrong format, not wired, non-deterministic timestamp | **P1.1** YAML 2.0 emitter + witnesslint + self-validate |
| Timeout/crash/OOM → UNKNOWN (0, not negative) | ✓ conceptually | Ensure graceful UNKNOWN under BenchExec SIGKILL |

### 7.4 Licensing of bundled dependencies — including model weights

**Rule (verbatim):** the archive "contains a LICENSE that allows reproduction and evaluation of the tool by anybody and **does not place any restriction on the tool output (log files, witnesses)**." The rules are **silent on model weights specifically**, but by the self-containment clause a bundled weight is part of "the tool," so its license must independently satisfy *redistributable + evaluable-by-anybody + no output restriction*. **The archive's effective license = the most-restrictive intersection of all bundled parts.**

- **SAF core + current deps: clean.** MIT (root); libz3 bundled = **MIT**; libLLVM via `llvm-sys` = **Apache-2.0-with-LLVM-exception**; roaring/ascent/pyo3/rayon = MIT/Apache. **No GPL/AGPL.** "No SVF reuse" holds (SVF is **GPLv3** — strong copyleft; never link/bundle). Add a `THIRD-PARTY-LICENSES`/NOTICE file.
- **If an LLM is added — bundle only Apache-2.0/MIT weights:** ✅ Qwen2.5-**0.5B**/**1.5B** (Apache-2.0), SmolLM2 (Apache-2.0), Phi-3/3.5-mini (MIT), TinyLlama-1.1B (Apache-2.0), OLMo (Apache-2.0), Pythia (Apache-2.0). ❌ **Do NOT bundle** Qwen2.5-**3B** (Qwen Research, non-commercial), **Llama-3.2** any size (community license + Acceptable-Use Policy — restricts field-of-use and *output*), **Gemma-2** (Gemma Terms + Prohibited-Use Policy). The "no restriction on tool output" clause is the specific trap: a witness/log that is partly a model output must not be governed by a use policy. Always re-verify the exact checkpoint/revision (GGUF re-uploads can differ).

### 7.5 Anti-fingerprinting, one-global-parameter-set, AI declaration

| Requirement (verbatim) | Obligation on SAF (esp. with name-hints/LLM, §4.2) |
|---|---|
| "There is **one global set of parameters** … forbidden from using the program name, its hash, or the current category to tune their parameters." | Single fixed config across all categories; no filename/hash/category lookup tables; document the exact parameter set in the paper |
| "should not use identifiers nor comments … to **fingerprint and identify** individual tasks" | Name-hints may only *propose/rank* generalizably; never identify a task or reach a verdict (§4.2 guardrails) — the **highest-risk clause for an AI-labeled SAF** |
| "It is acceptable to use occurrence of calls to **extern functions from standard libraries** … to detect which feature a task uses" | The only sanctioned adaptivity — key feature-detection on `malloc`/`pthread_create`, not on names |
| Organizer "may … rename local identifiers … add/change/remove comments" | Any heuristic keyed on local names must degrade to a name-free fallback |
| "**label ai** if the tool uses an LLM or other sophisticated kind of artificial intelligence"; "**meta_tool** … composed of other tools" | If an LLM ships, `ai` is **mandatory** (mislabeling risks disqualification); assess whether `meta_tool` applies to a LLVM+solver+LLM pipeline |
| Training phase vs evaluation phase | Tune only on the public training set; freeze the config for evaluation |

**Bottom line:** *No single rule blocks SAF's approach* — including the small-LLM plan — provided we: ship a **native self-contained ZIP** (not Docker) with a passing `smoketest.sh`; emit **validator-confirmable YAML witnesses**; keep verdicts a deterministic function of the fixed resource envelope (pin the env toggles, no wall-clock verdict decisions); bundle **only Apache/MIT model weights**; declare the **`ai` label**; and keep every learned/name signal on the **propose/rank** side with a sound checker deciding. Most of this work already coincides with P0/P1.

---

## Appendix — Sources & confidence

**Primary (HIGH-CONFIDENCE):**
- SV-COMP 2026 rules (scoring, limits, categories, anti-fingerprinting, `ai` label): <https://sv-comp.sosy-lab.org/2026/rules.php>
- SV-COMP 2026 benchmarks/categories: <https://sv-comp.sosy-lab.org/2026/benchmarks.php>
- SV-COMP 2026 submission (4 artifacts, tool-info, smoketest): <https://sv-comp.sosy-lab.org/2026/submission.php>
- SV-COMP 2026 verified results: <https://sv-comp.sosy-lab.org/2026/results/results-verified/>
- Report on SV-COMP 2026 (Beyer & Strejček, TACAS 2026): <https://www.sosy-lab.org/research/pub/2026-TACAS.Evaluating_Software_Verifiers_for_C_Java_and_SV-LIB_Report_on_SV-COMP_2026.pdf> · DOI <https://doi.org/10.1007/978-3-032-22749-2_23>
- Witness format 2.x spec: <https://gitlab.com/sosy-lab/benchmarking/sv-witnesses/-/blob/main/user-guide/Witness-Format.md>
- BenchExec network isolation & cross-core CPU accounting: `doc/container.md`, `doc/resources.md` in <https://github.com/sosy-lab/benchexec>
- SV-COMP FAQ (unconfirmed=1pt for correctness; runexec `--container`): <https://sv-comp.sosy-lab.org/faq.txt>
- fm-tools schema (`ai`/`meta_tool` labels, tracks): <https://fm-tools.sosy-lab.org/schema.html>

**Tool techniques:** CPAchecker <https://cpachecker.sosy-lab.org/>, strategy selection (TACAS 2024) `10.1007/978-3-031-57256-2_21`; Ultimate/trace abstraction `10.1007/978-3-642-36742-7_53`; Symbiotic (LLVM-IR + slicing + KLEE) <https://www.fi.muni.cz/~xstrejc/publications/tacas2022symbiotic.pdf>; ESBMC k-induction+intervals `10.1007/978-3-030-17502-3_15`; kIkI/2LS <https://arxiv.org/pdf/1506.05671>; Goblint `10.1007/978-3-031-30820-8_34`; Mopsa `10.1007/978-3-031-30820-8_37`.

**ML/LLM precedent & feasibility:** PeSCo (TACAS 2019) `10.1007/978-3-030-17502-3_19`; Graves-CPA (GNN selection) <https://arxiv.org/abs/2201.11711>; Lemur (LLM invariants + verifier) <https://arxiv.org/pdf/2310.04870>; Quokka <https://arxiv.org/pdf/2509.21629>; small-LLM CPU throughput ~9 tok/s for 1B-Q4 (LOW-CONFIDENCE, hardware-dependent) <https://www.techrxiv.org/doi/pdf/10.36227/techrxiv.177220011.12966071>.

**Identifier-name semantics (§4.2):** name→type — NL2Type <https://ml4code.github.io/publications/malik2019nl2type/>, TypeWriter <https://arxiv.org/pdf/1912.03768>, Typilus <https://arxiv.org/abs/2004.10657>; name→nullability — NullGTN <https://arxiv.org/html/2406.15676>; name→units — SA4U <https://arxiv.org/pdf/2210.09136>; variable-name embeddings — VarCLR <https://arxiv.org/abs/2112.02650>; decompiled name recovery + generalization collapse — DIRTY <https://edmcman.github.io/papers/usenix22.pdf>, VarBERT <https://sefcom.asu.edu/publications/varbert-oakland24.pdf>; body↔name — code2seq <https://arxiv.org/pdf/1808.01400>, VarNaming/VarMisuse <https://arxiv.org/abs/1711.00740>. Renaming robustness — ALERT <https://arxiv.org/abs/2201.08698>, Adversarial Examples for Code <https://arxiv.org/abs/1910.07517>, Du et al. FSE'23 <https://arxiv.org/html/2311.07553>, "When Names Disappear" (preprint) <https://arxiv.org/html/2510.03178>, Orvalho & Kwiatkowska (semantics-preserving mutations; the "70% drop" myth) <https://arxiv.org/abs/2505.10443>. Ranking-as-safe-insertion — Bingo <https://www.cis.upenn.edu/~mhnaik/papers/pldi18a.pdf>, BayeSmith <https://prosys.kaist.ac.kr/publications/icse22.pdf>. iRank (rank LLM invariants) <https://arxiv.org/abs/2310.09342>; Loopy <https://arxiv.org/pdf/2311.07948>. SAF name-handling audit: `crates/saf-core/src/air.rs`, `crates/saf-frontends/src/llvm/{debug_info,mapping}.rs`, `crates/saf-analysis/src/{invariants,dda,pta,checkers,z3_utils,display}/`.

**LOW/MEDIUM-CONFIDENCE flags:** exact competition CPU model (E3-1230 v5 is documented Apollon HW; rules only pin "latest Ubuntu LTS, x86_64"); precise 1B-Q4-on-Xeon tok/s; UParalizer/PROTON internal algorithms (2026 system-description PDFs not read); the exact set of empty/minimal correctness witnesses a validator will confirm.

**Compliance (Part 7):** verdict/witness/parameter contract & anti-fingerprinting & `ai`/`meta_tool` — rules.php <https://sv-comp.sosy-lab.org/2026/rules.php>; archive/LICENSE/smoketest/2 MB/fm-tools — submission.php <https://sv-comp.sosy-lab.org/2026/submission.php>; fm-tools schema (ORCID, DOI regex, `${witness}`, labels) <https://fm-tools.sosy-lab.org/schema.html>; BenchExec tool-info & OCI limitation <https://github.com/sosy-lab/benchexec/blob/main/doc/tool-integration.md>, FM-Weck FM 2024 `10.1007/978-3-031-71177-0_3`; reproduction reports (nondeterminism observed) — Twente 2023 <https://arxiv.org/pdf/2303.06477>, Bajczi TACAS 2025 `10.1007/978-3-031-90660-2_10`. Model-weight licenses — Qwen2.5 <https://qwenlm.github.io/blog/qwen2.5/>, SmolLM2/Phi/TinyLlama/OLMo/Pythia (Apache/MIT), Llama-3.2 <https://www.llama.com/llama3_2/license/>, Gemma <https://ai.google.dev/gemma/terms>; SVF = GPLv3 <https://github.com/SVF-tools/SVF>. Determinism/compliance verdicts were adversarially re-verified against the raw HTML of the primary pages.

**SAF evidence:** all file:line references in Part 3 are from a read-only audit of this repo (`crates/saf-bench/src/svcomp/`, `crates/saf-cli/src/{driver,commands,bench_types}.rs`, `crates/saf-analysis/src/{absint,cegar,invariants,z3_utils}/`, `scripts/{compile-svcomp.sh,sv-comp-stubs.h}`, `plans/051-056`).
