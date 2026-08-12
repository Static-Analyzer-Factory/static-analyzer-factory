# Plan 198: Concurrency-aware `valid-memsafety` confirmation (R5 follow-on — the dominant lever)

**Status: SCOPING / roadmap placement — NOT yet brainstormed. Do the brainstorm → de-risk → design →
approval → TDD in a NEW session** (this file documents the decision, the evidence, and the soundness crux so
that session starts grounded). Branch `svcomp`. Laptop = git source of truth; all builds/experiments on the
VM (`ubuntu@cd-vm-15-ai-vm`); commit only when the user asks. Follows plan 197 (R5, Slices 0–2 committed
`d163e30` + `782a78f`).

## 1. The decision and why (evidence-backed)

R5 wired `saf verify` to emit sound, CPAchecker-confirmed `false(valid-deref)`/`false(valid-free)` via an
ASan concrete-replay confirmer, but **abstains on any program that spawns a thread** (`program_spawns_threads`
→ `unknown`) because a concurrency benchmark's *safe-but-ASan-traps-on-one-schedule* is a −16 risk (the
Slice-0c `28-race_*` false alarms).

**The measured reservoir (`scripts/r5_thread_reservoir.py`, VM, 2026-08-12) shows that abstain forfeits the
overwhelming majority of the memsafety reservoir:**

| bucket | count | note |
|---|---|---|
| valid-memsafety total | 20,592 | |
| buggy (expected false) | 10,093 | |
| **buggy AND thread-spawning** | **9,513 (94% of buggy)** | **R5 abstains on ALL of these — the recoverable lever** |
| buggy sequential (R5's current scope) | 550 | ~a third caught by Slices 0–2 |
| safe AND thread-spawning | 9,076 | the FP surface the de-risk must clear |
| dedicated concurrency dirs (excluded) | 804 | `pthread*/weaver/goblint/ldv-races/…` |

The sv-benchmarks Juliet integration runs almost every task's sink in a **worker thread that main joins** —
a *deterministic* execution, not a data race. So concurrency-aware confirmation is not a marginal follow-on;
it is **the dominant `valid-memsafety` recall lever** (~17× R5's current sequential scope). Do it before
resuming the main roadmap (R6 `no-overflow`).

## 2. The soundness crux (what makes this hard, and the key question)

SV-COMP `valid-memsafety = TRUE` means **no execution — under any schedule or input — violates memory
safety.** So a *correctly-labelled* safe task should never ASan-trap on any schedule. If that held
universally, we could simply drop the thread abstain and let ASan arbitrate (good-threaded = deterministic
safe sink = no trap; bad-threaded = deterministic bad sink = trap). The **−16 risk** is the residual: a
threaded *safe* task where ASan traps on the single schedule it runs — a genuine schedule-dependent
data-race memory error that the benchmark's memsafety label does not count (or a harness artifact of our
driver interacting with real pthreads). The Slice-0c `28-race_*` FPs prove this happens for *dedicated
concurrency benchmarks*; the open question is whether it happens for the **Juliet worker-thread pattern**
(main spawns one worker, joins, no concurrent shared-mutable access).

**Decisive de-risk experiment (run FIRST, no production code):** run the existing ASan confirmer with the
thread abstain REMOVED over a large sample of **thread-spawning SAFE** valid-memsafety tasks (Juliet-heavy,
excluding the `conc_dir` categories) and count ASan traps = the false-alarm surface. In parallel, measure the
**thread-spawning BUGGY** ASan-catch rate = the recall prize. Reuse `scripts/r5_verify_memsafety_eval.py`
(add a `THREADS=1` mode that keeps thread-spawners) + a temporary build with the abstain relaxed.
- If threaded-safe FP ≈ 0 → the Juliet worker pattern is deterministically safe; confirmation is (near-)sound
  and the lever is huge → design the minimal sound gate.
- If threaded-safe FP > 0 → characterise each (real race vs harness artifact vs mislabel) and design a gate
  that abstains exactly on the race-prone shape.

## 3. Approaches to evaluate in the brainstorm (soundness-first)

1. **Structural single-worker detection:** confirm only the deterministic pattern — exactly one
   `pthread_create`, a `pthread_join` before main uses any shared result, no other concurrent shared-mutable
   access — so the worker's execution is schedule-independent. (Matches the Juliet pattern; needs a light AIR
   check, not a full race analysis.)
2. **Determinism via repetition / schedule perturbation:** run the ASan replay N times (optionally under a
   perturbing scheduler / `PCT`-style seed); confirm only if it traps on EVERY run. A deterministic-thread
   bug always traps; a race is intermittent. (Cheap, but a race can be deterministic under the default
   scheduler — pair with (1) or a race check, do not use alone.)
3. **Sound race-freedom gate:** confirm a threaded task only if a (sound-for-this-purpose) check says the
   faulting access cannot race. SAF's MHP/MTA exist but are under-approximate (plan 193 §4.5) — assess
   whether a conservative "no cross-thread shared write to the faulting object" check is feasible.
4. **Serialize-the-worker replay:** run the worker inline / with a deterministic single-thread scheduler so
   ASan explores a canonical schedule, plus an argument that the violation is schedule-independent.

The bar: emit `false` only when the memory violation is **guaranteed regardless of schedule** (or provably on
a deterministic execution); otherwise abstain. Never remove the abstain blindly.

## 4. Redlines (inherit from plan 197 §1 / CLAUDE.md; concurrency-specific additions)

- Never `true`; never `false` without an ASan-confirmed, schedule-robust violation; each abstain is sound
  (recall cost only). A wrong verdict on a threaded-safe task is −16 — the whole point of the gate.
- Reuse the R5 machinery unchanged (strategy arm, `asan_confirm`, `parse_asan_report` R1/R2, mini-fuzz,
  witness lowering); this slice changes ONLY the thread gate (`program_spawns_threads`) into a
  schedule-soundness gate, plus any repetition/scheduler control in the replay.
- Byte-deterministic verdict/witness; bounded per-task time (the watchdog + `replay_timeout` × repetitions).

## 5. Acceptance (to be finalised in the design)

Blind `saf verify` recall on **thread-spawning** valid-memsafety buggy tasks ↑ from 0, holding **0 false
alarms** on thread-spawning safe tasks and **0 TRUE**, with a confirmed witness, byte-deterministic — a large
absolute recall gain over R5's sequential 550-task scope.

## 6. First steps for the new session

1. `superpowers:brainstorming` → clarify scope with the user (which approach(es), how strict the soundness
   gate, repetition budget).
2. Recon: read plan 197 §12 (the R5 confirmer + the `program_spawns_threads` gate), `crates/saf-cli/src/
   commands.rs` (`asan_confirm`, `NONDET_CONSTS`, `synthesize_asan_driver`), `crates/saf-svcomp/src/
   fast_paths.rs` (`has_threading_primitives`, MHP/MTA availability), and the Slice-0c `28-race_*` FP notes.
3. Run the §2 de-risk (threaded-safe FP surface + threaded-buggy recall) — GO/NO-GO on the abstain relaxation
   before any production code.
4. Design → user approval → TDD on the VM. Then **return to the main roadmap: R6 `no-overflow` loop-free
   FALSE** (plan 193 §7).

Context memories: [[saf-svcomp-197-memsafety-slice0-go]], [[saf-svcomp-capability-findings]],
[[saf-svcomp-196-r4-low-roi]], [[saf-svcomp-workflow]], [[saf-svcomp-vm-env]].
