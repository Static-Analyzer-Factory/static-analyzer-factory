# Plan 198: Reachability-refined thread gate for `valid-memsafety` FALSE (R5 follow-on — the dominant lever)

**Status: DONE (Slices 0–2, 2026-08-13, VM-green, UNCOMMITTED). Design user-approved 2026-08-12.** Branch
`svcomp`. Laptop = git source of truth; all builds/experiments on the VM (`ubuntu@cd-vm-15-ai-vm`); commit
only when the user asks. Follows plan 197 (R5, Slices 0–2 committed `d163e30` + `782a78f`). Implementation
record in §13; **acceptance MET** (FP=0 / TRUE=0 / 0 wrong sub-property / CPAchecker-confirmed threaded
witness; +2 bundled soundness fixes). Next roadmap item: **R6 `no-overflow`**.

> **Filename retained for continuity** (referenced by `CLAUDE.md`, `PROGRESS.md`, and the `saf-svcomp-*`
> memories). The Slice-0 de-risk this session **falsified the "concurrency-aware" premise**: the reservoir is
> sequential-in-practice, so the mechanism is a **reachability-refined thread gate**, not a concurrency
> soundness gate. §1 records the correction in full.

---

## 1. The decision and why (evidence-backed) — CORRECTS the scoping doc's premise

R5 wired `saf verify` to emit sound, CPAchecker-confirmed `false(valid-deref)`/`false(valid-free)` via an
ASan concrete-replay confirmer, but **abstains on any program whose module contains a `pthread_create` /
`thrd_create` symbol** (`program_spawns_threads`, `commands.rs:1472`) — because a concurrency benchmark's
*safe-but-ASan-traps-on-one-schedule* is a −16 risk.

The scoping doc assumed the abstained reservoir was **real concurrency**: "the sv-benchmarks Juliet
integration runs almost every task's sink in a **worker thread that main joins**." **The Slice-0 de-risk
(this session) proves that is false.** The Juliet thread machinery is **dead scaffolding**; the sink runs in
`main`; the reservoir is **sequential-in-practice**.

**Three independent proofs (VM, 2026-08-12):**

- **Source — all 18,583 Juliet `.i` files:** `stdThreadCreate` (the *only* caller of `pthread_create`)
  appears **only** as its prototype+definition (37,166 ≈ 2×18,583 occurrences) and is **invoked 0 times**
  (`grep -c 'stdThreadCreate(&' = 0`). `pthread_create` is dead code in every Juliet task.
- **IR — SAF's exact `clang -O0 -disable-O0-optnone` → `opt -passes=mem2reg` pipeline:** on a Juliet task,
  `main` calls the sink **directly**; `pthread_create` is called **only inside `@stdThreadCreate`**; and
  `@stdThreadCreate` is called **0 times** ⇒ `pthread_create` is **unreachable from `main`**.
- **Execution — real `saf verify` (thread-abstain relaxed) + a faithful ASan probe:** every confirmed buggy
  task traps on thread **`T0` (main)**, **100 % deterministic across K=5 reps**. No fault on a worker.

**The measured reservoir (`scripts/r5_thread_reservoir.py`, VM):**

| bucket | count | note |
|---|---|---|
| valid-memsafety total | 20,592 | |
| buggy (expected false) | 10,093 | |
| **buggy AND "thread-spawning" (symbol present)** | **9,513 (94 % of buggy)** | **R5 abstains on ALL — the lever** |
| safe AND "thread-spawning" | 9,076 | the FP surface the de-risk cleared |
| sequential (R5's committed scope) | 1,199 (buggy 550) | ~a third caught by Slices 0–2 |
| dedicated concurrency dirs (excluded) | 804 | `pthread*/weaver/goblint/ldv-races/…` |

**Composition of the "thread-spawning" reservoir (`scripts/r5_thread_residual.py`):** Juliet **18,583**
(dead scaffolding, sequential) + **only 6 non-Juliet** — all `ldv-linux-3.14-races/` (real race harnesses,
all `safe`, all "(no-output)" on the real pipeline). So ~100 % of the recoverable lever is Juliet
sequential-in-practice.

**Conclusion:** the dominant `valid-memsafety` lever is real (~17× R5's sequential scope) but the abstain is
forfeiting **sequential** tasks over a **dead symbol**. The sound fix is to make the thread gate fire on an
**actually-reachable spawn**, not on symbol presence. The "single-worker vs race" soundness crux does not
arise for this reservoir (there is no thread); it is preserved conservatively for the genuinely-threaded
residual.

## 2. The soundness argument (the −16 avoidance)

SV-COMP `valid-memsafety = TRUE` means no execution violates memory safety **under any schedule or input**.
The −16 risk R5 guards against is a threaded *safe* task whose ASan trap is a genuine schedule-dependent race
that the memsafety label does not count. **If no thread-spawn primitive is reachable from `main`, there is
only one schedule** — the program is sequential — so ASan's single run is schedule-independent and its trap is
a real violation on *every* execution ⇒ confirming is sound. The gate therefore emits `false` only when it
has **proven the execution sequential**, and abstains otherwise. It may only *over*-approximate reachability
(err toward abstaining) — it must never miss a reachable spawn, which is why reachable indirect calls are
treated conservatively (§3).

## 3. The gate (the only logic change)

A pure, unit-tested `reachable_spawns_threads(module, callgraph) -> bool` in
`crates/saf-svcomp/src/fast_paths.rs` (mirrors the existing `reachable_has_heap_allocations` at `:553`):

```
SPAWN_FUNCTIONS = ["pthread_create", "thrd_create"]     // actual spawns only (NOT mutex/atomic/join/fork)

reachable = reachable_functions(callgraph, module)      // DFS from main, fast_paths.rs:517
spawn_present = any module function named in SPAWN_FUNCTIONS
for each reachable, defined function f:
    for each instruction:
        CallDirect{callee} where callee.name in SPAWN_FUNCTIONS  -> return true   // a real reachable spawn
        CallIndirect{..} and spawn_present                       -> return true   // conservative: unresolved
                                                                                   // target could be a spawn
return false                                             // proven sequential
```

- **Home:** `saf-svcomp/src/fast_paths.rs` (pure, RED→GREEN unit tests), re-used from `saf-cli`.
- **Call site:** in `asan_confirm` (`commands.rs:1537`), replace
  `if program_spawns_threads(module) { … return Ok(None) }` with
  `if saf_svcomp::fast_paths::reachable_spawns_threads(module, &CallGraph::build(module)) { … return Ok(None) }`,
  and **delete the temporary `SAF_MEMSAFETY_ALLOW_THREADS` toggle** added for the Slice-0 de-risk.
  (`CallGraph::build(&module)` — `saf_analysis::callgraph`; saf-cli already depends on saf-analysis.)
- **Why conservative on indirect calls:** the module-level callgraph may under-approximate indirect edges
  (PTA truncation — see CLAUDE.md redline #7/#8). Independently treating any reachable `CallIndirect`
  (while a spawn primitive is linked) as a possible spawn guarantees we never *miss* a spawn, so soundness
  does not depend on the callgraph's indirect-edge completeness.
- **No-main / empty reachable set:** a program with no `main` cannot run (the ASan harness link-fails →
  `Ok(None)`), so the gate value is moot there; `reachable_functions` returns `∅` ⇒ gate returns `false`
  ⇒ the confirmer proceeds and resolves to `unknown` via link failure. Sound.

`program_spawns_threads` (the symbol-presence check) is deleted (its only caller becomes the reachability
gate).

## 4. What is reused UNCHANGED (this slice touches only the thread gate)

`asan_confirm`'s compile (`-fsanitize=address -g` + `synthesize_asan_driver`) and `NONDET_CONSTS` mini-fuzz;
`parse_asan_report` (R1 I/O-frame reject + R2 high-fidelity sub-property map); `memsafety_verdict`;
`lower_memsafety_hit`; the witness write path; the `strategy_for` `ValidMemsafety` arm; `ASAN_OPTS`. Zero
changes to the watchdog / `run_verdict` / witness emitter.

## 5. Current state (verified this session; re-confirm line numbers before editing — they drift)

- `crates/saf-cli/src/commands.rs`
  - `asan_confirm` `:1523` — the confirmer; thread abstain at `:1537` (currently `program_spawns_threads`
    `&& SAF_MEMSAFETY_ALLOW_THREADS unset`, the de-risk toggle). **← swap in `reachable_spawns_threads`;
    drop the toggle.**
  - `program_spawns_threads` `:1472` — symbol-presence check. **← delete.**
  - `memsafety_strategy` `:1433`, `synthesize_asan_driver` `:1484`, `NONDET_CONSTS` `:1204`, `ASAN_OPTS`
    `:1210`, `replay_timeout` `:1239`, `SCALAR_NONDET` `:1221` — unchanged.
- `crates/saf-svcomp/src/fast_paths.rs`
  - `reachable_functions(callgraph, module) -> BTreeSet<FunctionId>` `:517`, `reachable_has_heap_allocations`
    `:553` (the pattern to mirror), `THREAD_FUNCTIONS` `:19` (superset; do NOT reuse — includes mutex/atomic/
    fork; the gate uses the narrower `SPAWN_FUNCTIONS`). **← add `reachable_spawns_threads`.**
- `crates/saf-core/src/air.rs` — `Operation::CallDirect { callee }` `:552`, `Operation::CallIndirect { .. }`
  `:559`. `AirModule::function(FunctionId)` accessor exists (used throughout fast_paths).
- `saf_analysis::callgraph::CallGraph::build(&module)` — the callgraph constructor.

## 6. File structure (touched)

```
crates/saf-svcomp/src/fast_paths.rs   (edit) + reachable_spawns_threads() + SPAWN_FUNCTIONS + unit tests
crates/saf-cli/src/commands.rs        (edit) asan_confirm gate swap; delete program_spawns_threads + toggle
tests/programs/c/svcomp/              (NEW fixtures) dead-scaffolding sequential + genuinely-threaded worker
crates/saf-cli/tests/                 (edit) verify_memsafety.rs — 2 e2e (dead-scaffold->false; threaded->unknown)
scripts/r5_verify_memsafety_eval.py   (already has THREADS=1 mode, added Slice 0)
```

## 7. TDD slices

### Slice 0 — de-risk spike (NO production code; the go/no-go gate) — DONE (this session) → GO

See §12. Deliverables: the reframe (dead-scaffolding proof ×3), FP=0 on ~232 safe threaded tasks, ~53 %
buggy recall, IR-level reachability proof, determinism/thread-id characterization. **GO.**

### Slice 1 — the reachability gate (GO)

- [ ] **1a `reachable_spawns_threads` (RED→GREEN, pure).** Unit tests in `fast_paths.rs`:
  - dead-scaffolding module (a defined `pthread_create` caller that is unreachable from `main`) → `false`;
  - `main` directly calls `pthread_create` → `true`;
  - `main` reachably calls a helper that calls `thrd_create` → `true`;
  - a reachable `CallIndirect` while `pthread_create` is present in the module → `true` (conservative);
  - no spawn symbol in the module → `false`;
  - no `main` → `false` (moot; documented).
  Implement per §3 (reuse `reachable_functions`).
- [ ] **1b Wire into `asan_confirm`.** Replace the `program_spawns_threads` + `SAF_MEMSAFETY_ALLOW_THREADS`
  gate with `reachable_spawns_threads(module, &CallGraph::build(module))`; delete `program_spawns_threads`
  and the toggle. No other edits to the confirmer.
- [ ] **1c e2e (`verify_memsafety.rs`, `#[ignore]` Docker).** Two new fixtures under
  `tests/programs/c/svcomp/`:
  - `memsafety_false_dead_pthread.c` — links a never-called `stdThreadCreate`-style wrapper (so
    `pthread_create` is present but unreachable) and a `main` that unconditionally faults → `false(valid-deref)`
    + witness (proves the gate flips dead-scaffolding tasks from abstain to confirm);
  - `memsafety_safe_threaded_worker.c` — `main` spawns a worker that performs a **safe** op and joins →
    `unknown` (proves a genuine reachable spawn still abstains).
  Plus a regression: the existing `memsafety_false_atomic_nothread.c` (atomics, no spawn) still → `false`.
- [ ] **1d Verify the residual abstains.** Confirm `reachable_spawns_threads` returns `true` (abstain) on a
  real `ldv-linux-3.14-races` task whose IR reachably spawns (or, if it doesn't reachably spawn, document why
  it is still safe — currently "(no-output)"). Belt-and-suspenders benchmark hygiene: add
  `ldv-linux-3.14-races` to the eval/probe `CONC_DIRS` (measurement scripts only; the gate stays
  directory-agnostic).
- [ ] **Gates:** `make test` (Rust nextest + pytest), **clippy `--workspace -D warnings`**, `make fmt`;
  the plan-192/194 unreach e2e suite and the R5 sequential-memsafety e2e must be unregressed.

### Slice 2 — full-reservoir eval (the acceptance evidence)

- [ ] Blind `saf verify` over a stratified sample of the **full** valid-memsafety reservoir (sequential +
  thread-spawning, ILP32 + LP64) via `scripts/r5_verify_memsafety_eval.py` (no `SEQ_ONLY`/`THREADS` filter):
  report **FP=0 / TRUE=0**, the absolute confirmed-FALSE count vs R5's sequential-only baseline, sub-property
  fidelity, and **≥1 CPAchecker-confirmed** witness on a (formerly-abstained) threaded task.
- [ ] **Measure the conservative-indirect recall cost:** count Juliet buggy tasks that now abstain because a
  `CallIndirect` is reachable from `main` (the price of §3's conservative rule). Report it; it bounds Slice 3.

### Slice 3 — address-taken-closure refinement (GATED, later)

Build **only if** Slice 2 shows conservative indirect handling costs material recall. Replace "any reachable
`CallIndirect` ⇒ spawn" with "a reachable `CallIndirect` whose **address-taken-function closure** can reach a
spawn primitive ⇒ spawn" — still sound, still **no PTA** (uses the set of address-taken functions + callgraph
reachability). Measure-before-build (the R4 discipline). Full PTA-based indirect resolution stays out of scope
(the user deferred it).

## 8. Soundness & determinism invariants (mapped to slices)

| Invariant | Where enforced |
|---|---|
| Never `true` | `memsafety_strategy` returns `unknown_outcome()` when `asan_confirm` is `None` (unchanged) |
| FALSE only via a concrete ASan report | `asan_confirm` is the sole `false` source (unchanged) |
| **Confirm only a schedule-independent (sequential) violation** | `reachable_spawns_threads` == `false` gate (1a/1b) |
| **Never miss a reachable spawn** | conservative reachable-`CallIndirect` rule; over-approx callgraph (1a) |
| Abstain on every genuine/uncertain spawn | gate returns `true` ⇒ `Ok(None)` ⇒ `unknown` (1b); residual check (1d) |
| Correct sub-property or abstain | R1/R2 in `parse_asan_report` (unchanged) |
| Byte-deterministic verdict + witness | unchanged (witness carries file:line+class only; fixed `ASAN_OPTS`) |
| Bounded per-task time | one extra `CallGraph::build` + the unchanged one-compile/mini-fuzz replay (1b) |

## 9. Acceptance criteria & evidence

- **Large absolute `valid-deref`+`valid-free` recall gain** over R5's sequential-only baseline (the
  ~9,513-buggy thread-spawn reservoir recovered), holding **0 false alarms / 0 TRUE** across ILP32 + LP64,
  **≥1 CPAchecker-confirmed** witness on a formerly-abstained threaded task, byte-deterministic (Slice 2).
- Redlines §11 held (audited in Slice 2).
- **Extensibility preserved:** the change is one pure fast-path function + a one-line gate swap; no seam,
  confirmer, or witness rewrite.
- Gates: full `make test` + clippy `-D warnings` + fmt; unreach + R5 sequential-memsafety e2e unregressed.

## 10. VM workflow

```
rsync -az --delete .../static-analyzer-factory/crates/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az .../scripts/ .../tests/programs/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/   # no --delete
# no image rebuild needed — the R5 ASan runtime is already in saf-dev:llvm18
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make fmt && make test 2>&1 | tee /tmp/t.txt'
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo nextest run -p saf-cli --run-ignored all -E "test(verify_memsafety)"'
```

## 11. Redlines (inherited verbatim from R5 / CLAUDE.md; concurrency-specific reading)

- Never `true`; never `false` without a concrete ASan-confirmed violation on a **proven-sequential**
  execution; each abstain is sound (recall cost only). A wrong verdict on a threaded-safe task is −16 — the
  reachability gate is the whole guard.
- Reuse ALL R5 machinery unchanged; this slice changes ONLY the thread gate (symbol-presence →
  reachability) plus deleting the de-risk toggle.
- Byte-deterministic verdict/witness; bounded per-task time.
- Commit only when the user asks.

## 12. Out of scope (deferred)

- `valid-memtrack` (LSan 64-bit-only) and `valid-memcleanup` (R10).
- Z3 / PTA-guided steering for input-dependent faults (`fscanf`/socket/`rand`) — the residual misses, shared
  with the sequential reservoir; already deferred by R5 Slice 2.
- Full PTA-based indirect-call resolution (Slice 3 uses a sound address-taken-closure instead, and only if
  measured to pay off).
- Any memsafety **TRUE**; genuinely-concurrent memory-safety reasoning (the dedicated concurrency dirs stay
  abstained/excluded).
- After plan 198: resume the main roadmap at **R6 `no-overflow` loop-free FALSE** (plan 193 §7).

---

## 13. Implementation record

### Slice 0 — de-risk spike (2026-08-12, VM `ubuntu@cd-vm-15-ai-vm`, dev image `saf-dev:llvm18`) — GO

**Instruments (measurement only, no production code except a temporary env toggle):**
- `SAF_MEMSAFETY_ALLOW_THREADS=1` env toggle added to `asan_confirm` (`commands.rs:1537`) relaxing the thread
  abstain (default keeps it). **Temporary — Slice 1 replaces it with the reachability gate.**
- `scripts/r5_verify_memsafety_eval.py` — added a `THREADS=1` mode (keep only thread-spawners; the inverse of
  `SEQ_ONLY`; still excludes `CONC_DIRS`) + a mode-aware header.
- `scripts/r5_thread_derisk.py` (NEW) — a faithful `asan_confirm` mirror (same driver/flags/stub/`ASAN_OPTS`/
  `NONDET_CONSTS`, 0-first, first-trap-wins) that surfaces what the real binary discards: ASan class,
  faulting frame (R1 status), **thread id (`T0`/`Tn`)**, and **determinism across K reps**.
- `scripts/r5_thread_residual.py` (NEW) — thread-spawn reservoir composition (Juliet vs non-Juliet).

**The reframe (§1): the "thread-spawning" reservoir is sequential-in-practice.** Proven by source (0 live
`stdThreadCreate` calls across 18,583 Juliet files), IR (`pthread_create` unreachable from `main`; sink
called directly), and execution (all confirmations on `T0`, K/K deterministic). R5 abstains on a **dead**
`pthread_create` symbol.

**Numbers:**
- **ASan mechanism validated:** a pthread worker-thread heap-OOB toy traps deterministically (`exit=1`),
  reports `thread T1` + the worker frame (so a *real* worker fault would be R1-passable / R2 → `valid-deref`).
  ASan works at `-m64` and `-m32` out of the box (R5's runtime layer).
- **Pass 1 — real `saf verify`, `THREADS=1`, N=150 (306 tasks; thread-abstain relaxed):** **FP=0 / TRUE=0**
  (0 false alarms over **153 safe** thread-spawn tasks), **TP=80 / FN=70 → 53 % recall** on buggy,
  **sub-property 10/10 agree**, **80/80 witnesses**. FN = input-dependent (`fscanf`/socket/`rand`/`fopen`) +
  CWE761 bad-free (R2 abstains) — the same misses as sequential.
- **Pass 2 — `r5_thread_derisk.py`, N=60, K=5:** **SAFE confirmations = 0/66**; **BUGGY TP=33/60, ALL on
  `T0`, ALL K/K deterministic** (the reported "3 Tn" was a SEGV-line parse artifact — the thread is `T0)` in
  parens; no real worker fault). Classes: heap/stack buffer-overflow/underflow, use-after-scope, SEGV(null),
  double-free — all map cleanly.
- **Smoke — N=26:** FP=0, TP=7/10 — first confirmation of the toggle + pipeline.
- **Reservoir (`r5_thread_reservoir.py`):** thread_spawn 18,589 (buggy 9,513 / safe 9,076); sequential 1,199
  (buggy 550); conc_dir 804.
- **Residual (`r5_thread_residual.py`):** thread-spawn excl conc_dir = Juliet 18,583 (safe 9,070 / buggy
  9,513) + **6 non-Juliet, all `ldv-linux-3.14-races/` (safe, "(no-output)")**. The eval's `CONC_DIRS` is
  missing `ldv-linux-3.14-races` (hygiene note; the gate is directory-agnostic).

**GO decision:** the −16 surface is empty on the reservoir (it is sequential), the recall prize is ~53 % of a
9,513-task buggy pool (~17× R5's sequential scope), and the sound gate is a single reachability check
(validated at the IR level: `pthread_create` unreachable from `main` in Juliet). Proceed to Slice 1.

### Slice 1 — the reachability gate (2026-08-12, VM, TDD) — DONE (uncommitted)

- **1a `reachable_spawns_threads` (`crates/saf-svcomp/src/fast_paths.rs`, pure).** RED→GREEN: 7 unit tests
  (dead-scaffolding→false, direct pthread_create→true, helper thrd_create→true, reachable-`CallIndirect`+
  spawn-linked→true, indirect-without-spawn→false, no-spawn→false, no-main→false) — verified RED (7×
  `cannot find function`) then GREEN (7 passed). Implementation mirrors `reachable_has_heap_allocations`:
  `SPAWN_FUNCTIONS = {pthread_create, thrd_create}`; short-circuits `false` if no spawn primitive is linked;
  else abstains (`true`) iff a reachable function directly calls a spawn primitive OR contains a
  `CallIndirect` (conservative — never miss a spawn).
- **1b Wired into `asan_confirm` (`commands.rs`).** Replaced the `program_spawns_threads` symbol-presence
  check (and deleted that fn + the temporary `SAF_MEMSAFETY_ALLOW_THREADS` de-risk toggle) with
  `reachable_spawns_threads(module, &CallGraph::build(module))`. Only the thread gate changed; the confirmer,
  R1/R2, mini-fuzz, and witness are untouched.
- **1c e2e (`crates/saf-cli/tests/smoke.rs`, `#[ignore]` Docker) + fixtures
  (`tests/programs/c/svcomp/`).** `memsafety_false_dead_pthread.c` (unreachable `pthread_create` + an
  unconditional heap-OOB in `main`) verified RED under the old gate (`unknown`) → GREEN after the swap
  (`false(valid-deref)`); `memsafety_safe_threaded_worker.c` (a genuine reachable spawn on a safe program)
  stays `unknown` (the residual soundness guard). **`verify_memsafety` suite 12/12**; the R5 sequential
  fixtures + `atomic_nothread` regression unregressed.
- **1d Residual + hygiene.** An `ldv-linux-3.14-races` task (the only genuinely-threaded non-Juliet residual)
  **segfaults during frontend ingest** (the pre-existing LDV `.cil.i` crash, plans 192/196) — it dies before
  the memsafety strategy runs, emits no verdict, and is therefore not a false alarm (sound). Added
  `ldv-linux-3.14-races` to the measurement scripts' `CONC_DIRS` (measurement-only; the gate is
  directory-agnostic).
- **Gates (all VM-green):** `make fmt` clean, `make lint` clippy `--workspace -D warnings` clean, `make test`
  **2257 nextest passed** (+7 unit) + **94 pytest**, **35/35 verify e2e** (unreach unregressed + memsafety +
  witnesslint). The 8 build warnings in the log are pre-existing, in untouched crates (`saf-analysis`,
  `saf-datalog`).

### Slice 2 — full-reservoir acceptance eval + an FP fix (2026-08-12/13, VM) — IN PROGRESS

**S2a first run surfaced a soundness bug (FP=1) that the Slice-0 de-risk missed.** A blind THREADS=1
`saf verify` eval (production gate, no toggle, N=120) reported **FP=1**:
`CWE401_Memory_Leak__malloc_realloc_int_13_good.i` (a *safe* task) → `false(valid-deref)`. The de-risk's
FP=0 held over ~232 sampled safe tasks; the larger eval hit a shape the sampling missed. A −16 is a redline,
so this blocked "acceptance" until root-caused and fixed (systematic-debugging discipline).

**Root cause (confirmed with the exact ASan report):** the sv-benchmarks **LDV realloc MODEL**
`ldv_reference_realloc` does `res = malloc(NEW_size); memcpy(res, old, NEW_size)` — an OOB **read** of the
smaller old buffer (400 B → 520 000 B copy). It is gated on `if (ldv_undef_int() != 0)`, so R5's Slice-2
multi-constant mini-fuzz (driving the undef to `1`) enters that branch and ASan traps **inside the harness
memory model** (`file.i:1581`, frame `#1 ldv_reference_realloc`, below `#0 __asan_memcpy`), not the program's
own access. Under the model's intended abstract semantics the task is safe (correctly labelled). This is an
**R5-confirmer blind spot** (native execution of a verifier memory model), which the thread gate merely
*exposed* by recovering the threaded CWE401 tasks R5 used to abstain on.

**Fix — a principled R1 extension (`crates/saf-svcomp/src/memsafety.rs`, TDD):** `parse_asan_report` now also
rejects a fault whose located frame is an **LDV allocator model** (`is_harness_memory_model`: `ldv_*` +
`alloc`/`free`), exactly as it already rejects libc I/O interceptors and `printLine` helpers (R1). Sound
(abstain, never a wrong verdict) and **zero recall cost** — the FP-surface characterization
(`r5_thread_derisk.py`, N=150) showed all 80 real-bug TPs fault at program sink frames (`CWE121_…`,
`badSink`, …), **never** inside an `ldv_` model. RED→GREEN: `ldv_realloc_model_over_read_is_rejected_r1`
(verified failing → passing); the existing `memcpy_overflow_locates_the_program_frame` test still passes
(real in-program `memcpy` bugs unaffected). The FP task now → `unknown` on the real pipeline.

**S2a re-verification (production gate + fix, THREADS=1, N=200):** **FP=0 / TRUE=0** over **200 safe
threaded tasks**, **recall 107/200 = 54 %** (unchanged from the pre-fix ~53 %), **sub-property 16/16 agree**,
**107/107 witnesses**. Combined with the de-risk, ~430 safe threaded tasks with zero false alarms.
Gate re-run after the fix: `make fmt`/`make lint` clean, **`make test` 2258 nextest + 94 pytest**.

**S2b surfaced a SECOND, PRE-EXISTING R5 −16 (NOT a thread-gate issue).** A full-reservoir blind eval (no
filter, N=100) held **FP=0 / TRUE=0** but reported a **wrong sub-property (1/42)**:
`ldv-memsafety/memleaks_test3-1.i` (a **sequential** task — 0 `pthread_create`, so the thread gate never
touches it; R5 has always processed it) → `false(valid-deref)`, expected `false(valid-free)`. **Root cause:**
the task does `free()` of a **wild pointer** (`0xff…f1`); ASan reports a **`SEGV`** crashing inside
`free`/`__asan::…::Deallocate`, and R2 maps `SEGV → valid-deref`, but the violation is a bad-free
(`valid-free`). A wrong sub-property is −16. This is a **pre-existing R5 confirmer (R2) bug surfaced by the
plan-198 full-reservoir eval**, unrelated to the thread gate; bundled here for soundness.

**Fix — SEGV/deallocator disambiguation (`memsafety.rs`, TDD):** `parse_asan_report` now abstains when the
class is `SEGV` **and** the fault stack passes through a deallocator (`is_deallocator`: `free`/`cfree`/
`Deallocate`/`operator delete`) — a bad-free SEGV is ambiguous (could also be a downstream effect of an
earlier deref), so never guess `valid-deref` (abstain, recall cost only). RED→GREEN:
`segv_inside_free_is_ambiguous_bad_free_abstains`; the regressions `segv_null_deref_is_valid_deref`
(wild-deref SEGV still → `valid-deref`) and `double_free_is_valid_free` (non-SEGV free class unaffected) stay
green. Net: a −16 becomes a sound abstain.

- [x] **S2b re-verification (full reservoir, N=100): FP=0 / TRUE=0 / sub-property 40/40 (0 wrong)**, recall
  85/200 = 42 %, 85/85 witnesses.
- [x] **Final gate: `make fmt`/`make lint` clean, `make test` 2259 nextest + 94 pytest** (2250 base + 7 gate
  unit tests + 2 confirmer-fix tests), clippy `--workspace -D warnings` clean.
- [x] **S2c CPAchecker-confirmed witness on a formerly-abstained THREADED task:** the threaded (dead
  pthread scaffolding) `CWE590_Use_Stack_Memory_Out_Of_Scope__deref_int_declare_11_bad.i` → `false(valid-deref)`
  + a target witness at `:665` → **`witnesslint LINT_OK` + CPAchecker `CONFIRMED (cpachecker-analysis)`**.
  The recovered threaded FALSE actually SCORES.

**Recall lift (the dominant-lever payoff):** R5 abstained on all ~9,513 buggy thread-spawn tasks (recall 0
there); the gate recovers them at ~54 % confirmed (THREADS=1 eval), soundly. Absolute scored memsafety
FALSEs go from R5's sequential-only scope (~550 buggy × ~a third) to that plus ~9,513 × ~54 % — roughly a
**30× increase** in confirmed, witnessed `valid-deref`/`valid-free` verdicts, holding 0 FA / 0 TRUE / 0 wrong
sub-property.

**Slice 3 (address-taken-closure) NOT NEEDED — measured.** The production reachability gate's THREADS=1
recall (S2a 107/200 = 54 %) matches the de-risk toggle's (Pass 1 80/150 = 53 %), so the conservative
"reachable `CallIndirect` ⇒ abstain" rule costs **≈0 recall** on this reservoir (Juliet mostly has no
reachable-from-`main` indirect call). No refinement warranted (R4 measure-before-build discipline).

### Slice 2 — ACCEPTANCE MET (2026-08-13)

Blind `valid-deref`+`valid-free` recall recovered across the ~9,513-buggy thread-spawn reservoir R5 abstained
on (~54 % confirmed), holding **0 false alarms / 0 TRUE / 0 wrong sub-property** across the full reservoir
(sequential + threaded, ILP32 + LP64), with a **CPAchecker-confirmed** threaded witness, byte-deterministic.
The change is one pure fast-path fn + a one-line gate swap; **no seam / confirmer / witness rewrite**. Two
soundness fixes were bundled (both principled R1/R2 extensions in `memsafety.rs`, TDD): the thread-gate-
exposed **LDV-model FP** and the pre-existing **SEGV-in-free −16**. Gates: **2259 nextest + 94 pytest**,
clippy `-D warnings` + fmt clean, 35/35 verify e2e (unreach unregressed). **Uncommitted — commit when the
user asks.** Next roadmap item: **R6 `no-overflow` loop-free FALSE** (plan 193 §7).
