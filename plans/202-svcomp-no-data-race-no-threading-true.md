# Plan 202: `no-data-race` no-threading TRUE (R8) — DE-RISKED → **DEFER** (measured 0.0% recall)

**Status: Slice-0 de-risk DONE (2026-08-13, VM `ubuntu@cd-vm-15-ai-vm`) → DEFER (user-approved 2026-08-13).
NO production code.** Branch `svcomp`. Laptop = git source of truth; all builds/experiments on the VM; commit
only when the user asks. Follows plan 201 (R7 `termination` TRUE, committed `a4d0468`). Implements roadmap **R8**
(`plans/193` §4.5/§7). This is an **R4-style honest reshape**: the de-risk measured R8's TRUE recall on the real
reservoir at **exactly 0.0%**, so — like plan 196 (R4) — the slice is **not built**; this plan records the
decisive measurements, the (sound but empty) design, and the two "if it ever ships" requirements.

Grounded by a this-session Slice-0 de-risk (§4): a full-reservoir blind `saf verify` eval through the REAL
pipeline (throwaway arm, reverted), a source-level mechanism enumeration over all 1,031 tasks, adversarial −32
probes, and a primary-source + adversarially-verified scoring/semantics research workflow. References: `plans/201`
(the R7 sound-TRUE spine + the pre-commit-review discipline — the TEMPLATE), `plans/198` (the reused
`reachable_spawns_threads` gate), `plans/196` (the R4 measure-before-build → STOP precedent), CLAUDE.md redlines
#1/#7/#8, [[saf-svcomp-201-r7-termination]], [[svcomp-termination-witness-not-required]],
[[saf-svcomp-198-reachability-gate]], [[saf-svcomp-capability-findings]], [[saf-svcomp-workflow]],
[[saf-svcomp-vm-env]].

---

## 0. The one-paragraph shape

`saf verify --property <no-data-race.prp> ...` prints `unknown` on 100% (`strategy_for` has no `NoDataRace` arm).
R8 would add a `no_data_race_strategy` that emits a bare **`true`** (witness-not-required, like R7) **iff** the
program is **provably sequential** — no thread spawn reachable from `main` — because a single-threaded program
has no *concurrent* accesses and therefore vacuously satisfies `G ! data-race`. The machinery already exists: it
is a ~6-line arm mirroring `termination_strategy`, reusing plan-198's `fast_paths::reachable_spawns_threads`
(abstain on a reachable spawn / a reachable `CallIndirect` while a spawn symbol is linked). The soundness is real
(§3) and the −32 audit is clean (§4b). **But the de-risk measured the payoff at 0.0%: the entire no-data-race
reservoir is dedicated concurrency benchmarks in which every race-free (`true`) task is genuinely, reachably
threaded — race-free by *synchronization*, never by being sequential — so R8's gate soundly abstains on all
791/791 of them.** With ~0 recall and the score landing outside SAF's stated `C.FalseOverall` home (§4c), R8 is
**deferred** (the R4 discipline: measure the prevalence, report honestly, don't build a spine that scores nothing).

---

## 1. The decision and why (Slice-0-evidence-backed) — DEFER

R8 is the roadmap's **second** sound-TRUE slice and was expected to be *smaller* than R7 because the machinery is
already built (plan 198's `reachable_spawns_threads` already computes "provably sequential"). The de-risk
confirmed the machinery is ready and sound — and then measured its payoff at **nil**:

> **Blind `saf verify` (real pipeline, throwaway R8 arm) over the FULL no-data-race reservoir (795 expected-true +
> 236 expected-false, each at its declared data model): TRUE recall = 0/791 = 0.0%; the −32 audit is clean
> (WRONG_TRUE = 0 over all 236 racy tasks, FALSE_EMITTED = 0, determinism 15/15).**

The reason is structural and was independently corroborated by an exhaustive adversarial reservoir audit: the
`no-data-race` category (ConcurrencySafety) is **100% dedicated concurrency benchmarks** — every one of the 1,031
tasks (all 795 `true` + 236 `false`) links and *reachably* calls `pthread_create`. The race-free (`true`) tasks
are race-free because they synchronize correctly (mutexes / atomics / `__VERIFIER_atomic`), **not** because they
are sequential — so the sound "provably-sequential ⇒ TRUE" gate correctly abstains on every single one. Control
probes prove the mechanism itself works (a genuinely-sequential program → `true`; dead-scaffolding unreachable
`pthread_create` → `true`) — the reservoir simply contains nothing of that shape.

This mirrors **plan 196 (R4)**: the mechanism is sound and the spine is cheap, but a full-reservoir prevalence
measurement shows the payoff is ~nil, so the honest engineering call is to **stop before building**. It also
mirrors the strategic caveat that made R7 a documented-but-accepted trade-off: **a `no-data-race` TRUE scores in
ConcurrencySafety / C.Overall, NOT in `C.FalseOverall`** (SAF's stated home) — §4c. With 0 recall *and* an
out-of-home score, there is no case for building R8 now.

**User decision (2026-08-13): DEFER.** No R8 TDD. Optionally a tiny redline-#7 hygiene commit (§5) neutralizing
the dead unsound `analyze_no_data_race` branch — worth doing regardless of R8, but not urgent (it is dead code).

---

## 2. What R8 *would* be (recon — the design is ready, should it ever be revisited)

The build is a near-verbatim clone of R7's sound-TRUE spine; recorded here so a future revisit (e.g. if SV-COMP
adds sequential `no-data-race` tasks) needs no re-recon:

- **Property recognition already exists** (no edit): `property_kind.rs` — `Property::NoDataRace` variant (`:31`),
  `from_prp` matches `data-race` → `NoDataRace` (`:122`), `name()` → `"no-data-race"` (`:78`), `is_supported()`
  includes it (`:93`), with passing `parses_no_data_race` unit test.
- **The gate already exists** (no edit): `fast_paths::reachable_spawns_threads(module, callgraph)` (`:611`, plan
  198) — returns `true` (⇒ abstain) iff a `SPAWN_FUNCTIONS` primitive (`pthread_create`/`thrd_create`) is
  reachable from `main` via a direct call, OR a reachable function contains a `CallIndirect` while a spawn symbol
  is linked; returns `false` only when provably sequential.
- **The strategy** (the only new code): `crates/saf-cli/src/commands.rs` — a `NoDataRace => Some(no_data_race_strategy)`
  arm on `strategy_for` (`:991`) + a ~6-line `no_data_race_strategy(ctx)` mirroring `termination_strategy`
  (`:1695`): build the callgraph, `if reachable_spawns_threads(module, &cg) { unknown_outcome() } else {
  VerdictOutcome { verdict: "true".into(), witness: None } }`. No compile-of-original, no confirmer, no witness
  (witness-not-required, §4c). This is what the throwaway de-risk arm implemented.
- **Redline-#6/#7 hygiene:** neutralize the dead `analyze_no_data_race` (`property.rs:1304`) — its
  `if !has_threading_primitives(module) return True` is an unsound symbol-presence TRUE branch (misses an
  indirectly-called `pthread_create`); mirror R7's neutralization of the dead `analyze_termination`. §5.

---

## 3. Soundness (the design is sound — this is not why R8 is deferred)

SV-COMP 2026 `no-data-race` = `CHECK( init(main()), LTL(G ! data-race) )`, with the English definition (primary
source, §4c): *"If there exist two or more concurrent accesses to the same memory location and at least one is a
write access, then all accesses must be atomic."* A **single-threaded** program has exactly one thread of
execution, so **no two accesses are ever concurrent** — the antecedent is never satisfied, `G ! data-race` holds
vacuously, and `no-data-race = TRUE`. Therefore **"no thread reachable from `main` ⇒ TRUE" is sound**, *provided
the spawn/concurrency gate is COMPLETE* (redline #7): it must recognize-or-abstain on every thread-creation
mechanism, and abstain on any reachable indirect call (redline #8, satisfied by `reachable_spawns_threads` —
which abstains on `CallIndirect` and thus needs no PTA). Never emit `false` (race *detection* is a separate
future confirmer, and a `no-data-race` FALSE additionally needs a confirmed violation witness — §4c). The verdict
is a byte-deterministic constant `"true"`, no witness, one static pass — bounded.

---

## 4. De-risk record (Slice 0, this session, 2026-08-13, VM `ubuntu@cd-vm-15-ai-vm`) — DEFER

### 4a — Prevalence (de-risk unknown #1, the go/no-go) — **0/791 = 0.0% recall → DEFER**
Reservoir: **1,031 no-data-race tasks (795 expected-true, 236 expected-false)**, entirely concurrency families
(`pthread-wmm` 283, `goblint-regression` 205, `weaver` 175, `pthread-ext` 76, `pthread-race-challenges` 63,
`pthread` 62, … `ldv-races`, `libvsync`, `pthread-divine`, `pthread-theta`). A **source-level mechanism scan**
(`scripts/r8_ndr_mechanisms.py`) over all 1,031 sources found **every task contains a `pthread_create` token**
and **0 tasks are "sequential-looking"** (no spawn token at all). A **full-reservoir blind `saf verify` eval**
(`scripts/r8_verify_ndr_eval.py`, a THROWAWAY `no_data_race_strategy` arm wired on the VM reusing
`reachable_spawns_threads`, since reverted) confirmed at the IR/reachability level: **recall = 0/791 = 0.0%**
(4 no-output frontend crashes → sound abstain), i.e. `reachable_spawns_threads` finds a reachable spawn in every
race-free task. **Control probes** (`scripts/r8probes/`) prove the mechanism is not broken: `ndr_seq_norace.c`
(genuinely sequential) → `true`; `ndr_dead_pthread_seq.c` (unreachable `pthread_create` + sequential `main`) →
`true`. The recall-0 is the reservoir's nature — no `no-data-race=true` task is provably sequential.

### 4b — Spawn-completeness (de-risk unknown #2, the −32 gate) — clean on the reservoir; one documented gap
The −32 audit (a wrong `true` on a racy task) was **WRONG_TRUE = 0 over all 236 racy tasks** on the real
pipeline — `reachable_spawns_threads` correctly abstained on every genuinely-racy program (its `pthread_create`
is reachable from `main`; no frontend-drop hid a spawn). The reservoir is **pthread-ONLY**: the source scan found
**0** OpenMP (`GOMP_parallel`/`__kmpc_fork_call`/`#pragma omp`), **0** `clone`, **0** `thrd_create`; the 29
`fork` hits all co-occur with `pthread_create`. So `SPAWN_FUNCTIONS` is complete for today's scored set — this was
independently corroborated by an exhaustive adversarial audit of all 19 Concurrency.set directories (the two
non-pthread textual hits were false positives: a `clone(` in a man-page comment and a `thread_create` in a `.yml`
filename). **BUT — the R7 "do not trust the reservoir as complete" lesson bit:** an adversarial probe
`ndr_omp_race.c` (a genuinely-racy OpenMP program, no pthread) → **`true`** (a −32). Cause: the gate whitelists
only `pthread_create`/`thrd_create`, and SAF's compile (no `-fopenmp`) *drops* the pragma (verified: no
`GOMP_`/`__kmpc_` symbols in SAF's IR; a real `-fopenmp` toolchain lowers `#pragma omp parallel for` to
`__kmpc_fork_call`). This is currently **unrealized** (SV-COMP's un-preprocessed-include rule permits only
C-standard headers + `pthread.h`; no OpenMP task exists), so present-day −32 risk ≈ 0 — but it is a hard
**completeness requirement if R8 ever ships** (§4d).

### 4c — Scoring / semantics (de-risk unknown #3) — RESOLVED (primary source + adversarial)
A research workflow (2 primary-source researchers + 3 adversarial verifiers + synthesizer; 2026 & 2025
`rules.php`/`benchmarks.php`) established, HIGH confidence:
- **`no-data-race` TRUE is witness-not-required (verdict-only +2).** The 2026 witness-format table lists
  `C.no-data-race`: correctness = *"not supported"*, violation = `1.0`; the exemption rule ("no witnesses for the
  TRUE results are required … where the correctness witnesses are not supported") applies. Same posture as R7 —
  R8 emits a bare `true`, no witness pipeline. (A `no-data-race` **FALSE**, by contrast, *does* need a confirmed
  1.0 violation witness — relevant only to a future race-detection confirmer, not R8.)
- **Wrong TRUE = −32** (missed bug / unsound analysis); the correctness-witness exemption does **not** soften it.
- **Scoring home:** `no-data-race` lives in **ConcurrencySafety / C.Concurrency** (part of C.Overall).
  `C.FalseOverall` *includes the category* but counts **only its FALSE results** — *"the results 'correct TRUE'
  and 'incorrect TRUE' are not counted."* So R8's `true` (+2) would score in **ConcurrencySafety / C.Overall, NOT
  in `C.FalseOverall`** (SAF's stated home). This is the same caveat as R7, and — combined with 0 recall —
  reinforces DEFER. (Note: this refines the [[svcomp-termination-witness-not-required]] wording: `C.FalseOverall`
  *contains* the Concurrency category but ignores its TRUE results; it *excludes* Termination entirely because
  Termination is a liveness, not safety, property.)

### 4d — The "if it ever ships" completeness requirement (the −32 design, for the record)
A shipped R8 must be **fail-closed**, not a pthread-only allowlist: emit sequential-`true` only when the whole
reachable program is provably free of ANY thread-creation, and **abstain** (`unknown`) on any of —
`GOMP_parallel`/`__kmpc_fork_call`/`__kmpc_fork_teams`/surviving `#pragma omp` (OpenMP; adversarially confirmed a
real race mechanism), `fork`/`vfork`/`clone` (+ shared memory `mmap MAP_SHARED`/`shmget`), `signal`/`sigaction`
handlers, and any reachable `CallIndirect` (already handled). Because SAF compiles without `-fopenmp`, OpenMP must
be caught at the **source level** (`#pragma omp` / `omp.h`), since its lowering is invisible in SAF's own IR.
This extends the shared `reachable_spawns_threads` (recall-only cost to plan 198/199, which have no OpenMP tasks)
or a companion `reachable_creates_concurrency` gate. Not built (R8 is deferred), but documented so a revisit is
sound-by-construction.

### 4e — De-risk artifacts (uncommitted, on the laptop `scripts/` + VM; measurement-only)
`scripts/r8_ndr_mechanisms.py` (source mechanism enumeration), `scripts/r8_verify_ndr_eval.py` (blind eval, clone
of `r7_verify_termination_eval.py`), `scripts/r8_derisk_patch.py` (the throwaway VM-only arm patch, reverted),
`scripts/r8probes/*.c` (the 5 adversarial/control probes). The throwaway VM `commands.rs` arm was reverted (clean
`crates/` re-rsync from the laptop); the laptop production tree was never edited.

---

## 5. Optional redline-#7 hygiene (independent of the DEFER)

The dead `analyze_no_data_race` (`property.rs:1304`, part of the DEAD `analyze_property` family — zero live
callers on either the `verify` or bench surface) still carries an **unsound TRUE branch**:
`if !has_threading_primitives(module) return PropertyResult::True` — symbol-presence, so it misses an
indirectly-called `pthread_create` ⇒ a latent spurious TRUE if the dead engine were ever wired (redline #7). This
is the same latent-unsound-branch posture R7 cleaned up for `analyze_termination` (redline #6). Neutralizing it to
always-`Unknown` (and dropping any orphaned helper) is a small, R8-independent hygiene fix. Deferred with R8;
worth a tiny standalone commit if/when convenient. It is **not urgent** (the branch is dead code).

---

## 6. Out of scope / what's next

- **Race DETECTION (a `no-data-race` FALSE):** out of reach — needs correct per-thread access sets, sibling MHP,
  a wired must-lockset, and a confirmed 1.0 violation witness (`plans/193` §4.5(c)). None of the plumbing is
  connected; MHP under-approximates and the lockset engine is dead.
- **Race PROVING on multithreaded code (a `no-data-race` TRUE beyond sequential):** out of reach — needs
  over-approximate MHP over all interleavings + all indirect forks + join narrowing.
- **Building R8:** deferred (this plan). Revisit only if SV-COMP adds provably-sequential `no-data-race` tasks
  (measure with `scripts/r8_verify_ndr_eval.py`) or if the C.FalseOverall-vs-C.Overall strategy changes.
- **Next (branch-focus decision, user 2026-08-13):** **the `svcomp` branch focuses on IMPROVING SV-COMP RESULTS;
  native-ZIP packaging (R9) + `valid-memcleanup` (R10) are the LAST steps.** SAF's general perf/memory issue
  (`plans/190` CruxBC parity) was **already addressed by PR #7** (`5c875b7`, merged 2026-08-05, in `svcomp` HEAD),
  so there is no separate performance push pending (an earlier same-day "performance first" note was corrected;
  `plans/190`'s "in-progress" status is stale). The branch improves SV-COMP results (recall / coverage /
  witness-confirmation across the wired properties — e.g. the measure-before-build recall levers on unreach-call
  and memsafety, the gated R5/R6 Slice-2 levers, and witness confirmation-% enrichment) and packages (ZIP) last.
  See PROGRESS.md Next Steps (top bullet) + [[saf-svcomp-branch-sequencing]].

---

## 7. Redlines held (even though nothing was built)

Never `true` except on a proven sequential path (the throwaway de-risk arm emitted `true` only when
`reachable_spawns_threads` proved sequentiality; 0/236 wrong on racy tasks). Never `false`. Byte-deterministic
(15/15). Bounded (one static pass). The throwaway measurement arm was reverted; **no production code landed**. The
completeness requirement (§4d) and the redline-#7 hygiene (§5) are documented for any future revisit.
