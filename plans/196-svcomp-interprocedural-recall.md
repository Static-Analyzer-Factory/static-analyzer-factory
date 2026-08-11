# Plan 196: Interprocedural FALSE-candidate recall (R4) — root candidate enumeration at `main`

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan slice-by-slice. Steps use checkbox (`- [ ]`) syntax. **All builds/tests run on the VM `ubuntu@cd-vm-15-ai-vm` (Docker); never the laptop. Subagents never call `make`. Commit only when the user asks.**

**Status:** design / **awaiting approval** (no implementation done)
**Date:** 2026-08-11
**Branch:** `svcomp` (laptop = git source of truth @ `7d84138`)
**Reference competition:** SV-COMP 2026 rules (2027 unreleased); witness format = SV-COMP witnesses **2.0** (SPIN 2024).
**Builds on** plan 192 slice 1c (sound FALSE by concrete replay), plan 194 (YAML-2.0 witness emitter), plan 195 (Tier A branching + `cpa-witness2test` validator, all committed). **Implements roadmap R4** (plan 193 §7 / plan 192 §3 "Slice 1d").
**Roadmap position:** confirmed-% is now strong (**6/7 = 86%** via `cpa-witness2test`), so `C.FalseOverall` is bounded by **recall = 7/120 (~6%)**, not confirmation. R4 is the dominant recall lever.

**Goal:** Raise blind `unreach-call` recall by composing FALSE candidates **across call boundaries** — steer the nondet inputs `main` (and intermediate callers) feed into callee-guarded `reach_error` sites so native replay reaches violations the current *intraprocedural* `enumerate_false_candidates` misses — while holding every soundness redline (0 false alarms, 0 TRUE, never `true`).

**Architecture:** SAF's FALSE path proposes candidates (over-approximate Z3) and confirms them by **native concrete replay** (execution is the sole arbiter). The replay driver already pins each `__VERIFIER_nondet_*` name to a **global FIFO queue that runs the whole program from `main`** (`synthesize_driver`, `commands.rs:1218`), so R4 changes **only how the nondet sequence + block path are *computed*** — the driver, the replay gate, and the two `false` sites are untouched. We root candidate enumeration at `main`, walk bounded-depth call chains `main→…→F` to each `reach_error` site, extract each frame's own guards over a cross-frame block sequence (the existing guard extractor already accepts `(FunctionId, BlockId)` pairs), solve jointly for one model, and collect the whole-program nondet sequence in execution order. Every candidate stays replay-gated.

**User decisions (2026-08-11):**
1. **First increment = Slice 0 + Slice 1** — the frame-aware nondet walk (refactor) + interprocedural candidates rooted at `main` with **no arg→param binding** ("Shape 1": the steering nondet and its guard live in the same frame). Measure recall, then decide Shape 2.
2. **Shape 2 (arg→param joint model) is spike-gated** (Slice S, like plan 195's Slice S): confirm callee param-guards reference `params[i].id` post-`mem2reg` **and** categorize the blind-sample interprocedural misses before building it. **GO → auto-proceed to Slice 2** (post a GO note); **NO-GO → stop & report.**

---

## Global Constraints (verbatim; every slice inherits these)

**Soundness redlines (from `CLAUDE.md` "SV-COMP Competition Work" + plan 193 §6 — NON-NEGOTIABLE; a wrong verdict is −16/−32 ≈ 16× the reward of a right one):**
1. `saf verify` **must never emit `true`.** This plan adds no TRUE path; it only adds FALSE *candidates*, which are still replay-gated.
2. **Never emit `false` without (a) an unconditional must-reach proof OR (b) concrete native-replay confirmation.** Every interprocedural candidate is a *proposal* fed to the SAME `replay_confirms_false` gate — **no FALSE from a Z3 SAT alone.** A spurious joint-SAT, a wrong nondet order, or an unbound argument **fails *closed* to `unknown`** (recall loss, score 0), never to a wrong FALSE (−16).
3. **No new `false` construction site.** The two `false(unreach-call)` sites stay exactly where they are: the unconditional `must_reach_error` proof (`commands.rs:1089`) and the one nested inside `replay_confirms_false → Ok(true)` (`commands.rs:1122`). This plan routes every interprocedural candidate through the second gate.
4. **Do NOT touch Stage-1 `must_reach_error`.** Stage-1 has *no* replay backstop; its two holes are already closed (plan 194 R3, `property.rs:1758-1762`/`:1792-1794`). R4 adds recall **only on the replay-gated Stage-2/3 side.** Putting any interprocedural reachability into `must_reach_error` would be a new unguarded FALSE site — forbidden.
5. **Bounded, watchdog-safe.** The replay driver consumes a **fixed finite** per-name array (exhausted reads return `0`), so the number of dynamic nondet calls on any confirmed path must be statically bounded. Cap call-depth, chains-per-site, and paths-per-frame; keep `MAX_REPLAY_CANDIDATES = 16` (`commands.rs:1150`). The wall-clock watchdog may only downgrade to `unknown`, never `true`/`false`.

**Determinism (NFR-DET-001):** byte-identical verdicts + witnesses for identical inputs. The call-chain enumerator, cross-frame path assembly, and nondet walk must use `BTreeMap`/`BTreeSet`/sorted iteration only — no `HashMap` iteration, `SystemTime::now()`, or RNG on the verdict path. Z3 stays pinned (`rlimit` + `random_seed`, plan 192).

**Extensibility (plan 193 §11):** the interprocedural enumerator is additive; it does not change the property-generic witness spine or the verdict-dispatch seam. Later properties still plug in via a strategy arm + a lowering fn.

**Workflow:** edit locally; targeted rsync to the VM before each build (`crates/` with `--delete`; root files + `scripts/` individually WITHOUT `--delete`; see `[[saf-svcomp-vm-env]]`). TDD throughout. `make test` / focused `docker compose run --rm dev sh -c '…'` on the VM only. `make fmt` before `make lint`. Let `cargo` regenerate `Cargo.lock` on the VM; sync it back before any commit. **Commit only when the user asks.**

---

## Current state (verified via recon 2026-08-11, file:line — re-confirm before editing; line numbers drift)

**The intraprocedural boundary (the recall gap):**
- `enumerate_false_candidates` (`property.rs:1917-1953`) finds every `reach_error`/`__VERIFIER_error` call site across the module (`find_calls_to`, `:1922-1925`), but for each site `(func_id, block_id, inst_id)` it checks reachability **within that function only**: `get_entry_block(module, func_id)` (`:1929`) → `check_path_reachable(entry, block_id, func_id, …)` (`:1932-1940`). So when `reach_error` is in a callee `F`, the query runs over `F`'s CFG; `main`'s steering nondet is never seen.
- `check_path_reachable` (`reachability.rs:56-162`) is the intraprocedural boundary: single `func_id`, one `Cfg::build(f)` (`:70`), paths enumerated within that CFG (`enumerate_paths`, `:85` — **private**), and every block mapped to the same func: `block_seq: Vec<(FunctionId, BlockId)> = path.iter().map(|&b| (func_id, b))` (`:92`).
- `resolve_nondet_sequence` (`property.rs:1960-1995`) collects scalar-int nondets **only along `block_path` within `func_id`** (`:1970-1974`), each valued from the model or `0` (`:1984-1987`), one slot per call.

**The replay driver already runs whole-program (so R4 needs no driver change):**
- `synthesize_driver` (`commands.rs:1204-1280`) emits, per scalar-nondet name, a fixed array `__saf_arr_<suffix>` and a **per-name counter** `__saf_i_<suffix>` (`:1218-1248`). At runtime `__VERIFIER_nondet_int()` returns `arr[i++]` while `i < n`, else `0`. The program runs from its **real `main`** (`replay_confirms_false` compiles `input` + the driver, `:1290-1358`), so **any function** that draws a nondet consumes from that name's global queue **in dynamic execution order**. ⇒ a whole-program-ordered `nondet_sequence` replays correctly with **zero driver change** (verified).
- Exactly TWO `false(unreach-call)` sites, both in `unreach_strategy` (`commands.rs:1078-1145`): Stage-1 `must_reach_error` (`:1089`) and the one nested in the `Ok(true)` arm of `replay_confirms_false` (`:1122`). Every other arm falls through to `unknown_outcome()`. ⇒ replay is the sole arbiter (verified, adversarial).

**The interprocedural infra the roadmap named is a dead end (verified, REFUTED):**
- `saf-svcomp/src/summaries.rs::compute_error_summaries` is called **only** from `analyze_unreachability` (`property.rs:566`) ← `analyze_property` — the dead-on-verify engine family plan 193 flagged (the live path is `must_reach_error` + `enumerate_false_candidates`, never `analyze_property`). It is also **value-blind** (a `May/Never`-error lattice; no model, no nondet values). Not reusable for value-steering.
- `saf-analysis/src/z3_utils/interprocedural.rs` (`CallerGuardContext`, `augment_with_caller_guards`) is live (used by the path-sensitive checker, `checkers/pathsens_runner.rs:150`) but: (a) its `get_dominating_block_path` is an admitted stub (`:148-170`, "just returns entry and target"), (b) `common_caller_guards` returns **empty for multi-caller functions** (`:104-107`), and (c) it **conjoins caller guard *conditions* without binding caller-arg→callee-param** (guards on disjoint `ValueId`s). It cannot connect "caller passes `x`" to "callee guards on `p`". Not reusable as-is for arg→param composition.

**Reusable primitives (verified, live):**
- `extract_guards_from_blocks(block_seq: &[(FunctionId, BlockId)], index) -> PathCondition` (`guard.rs`, pub) — **already cross-frame-shaped**: it consumes `(FunctionId, BlockId)` pairs. Feeding it a stitched `main→…→F` sequence extracts each frame's own guards with no change.
- `extract_assume_guards(block_seq, module, index)` (`guard.rs`, pub) — folds `__VERIFIER_assume` along the same cross-frame sequence.
- `PathFeasibilityChecker::new(z3_timeout_ms).check_feasibility_with_model(&pc, &index) -> (FeasibilityResult, BTreeMap<ValueId,i64>)` (`solver.rs`, pub) — the model-returning solver. Disjoint-frame guards reference disjoint `ValueId`s, so this already produces a joint model over all frames' nondets (**no solver change for Shape 1**).
- `ValueLocationIndex::build(module)` (`guard.rs`, pub) — whole-module operand index.
- `saf_analysis::callgraph::CallGraph::build(module)` — call graph with caller/callee queries (used for chain enumeration).
- `saf_analysis::…::graph_algo::tarjan_scc` (`graph_algo.rs:137`) — recursion-SCC detection for bounding.
- `check_joint_feasibility` with a shared `var_cache` (`solver.rs:243`) + `translate_operand` (`solver.rs:331-348`, where fresh unconstrained Z3 ints are minted) — the seam Shape 2 (Slice 2) injects `var(actual_i) == var(param_i)` equalities into. **Slice 1 does not touch this.**

**FALSE-path data structures (`property.rs`):**
- `NondetCall { pub func_name: String, pub value: i64 }` (`:1850-1857`), derives `Debug, Clone, PartialEq, Eq`.
- `FalseCandidate { pub reach_error_inst: InstId, pub block_path: Vec<BlockId>, pub assignments: BTreeMap<ValueId,i64>, pub nondet_sequence: Vec<NondetCall> }` (`:1870-1883`), derives `Debug, Clone`.

---

## The design in one paragraph

For each `reach_error` site in function `F`, enumerate bounded-depth call chains `main→…→F`; for each chain, build a **cross-frame block sequence** `Vec<(FunctionId, BlockId)>` = [`main`: entry→callsite] ++ … ++ [`F`: entry→`reach_error` block]; extract that sequence's guards + assume guards with the **existing** extractors; solve once with `check_feasibility_with_model` for a **joint model** over all frames' nondet `ValueId`s; on `Feasible`, walk the cross-frame sequence to collect the **whole-program nondet sequence in execution order** (Slice 0); emit a `FalseCandidate` (`nondet_sequence` = whole-program, `block_path` = `F`'s local sub-path for the witness target, `reach_error_inst` = the error inst). Feed it to the **unchanged** `replay_confirms_false` gate. **Shape 1** (steering nondet guarded in its own frame, e.g. `x=nondet(); if(x==K) buggy();`) is cracked with **no arg binding** — because the real program does the arg→param binding at runtime; we only need `main`'s nondet value, and `main`'s guard `x==K` is intraprocedural in `main`. **Shape 2** (guard on the callee *parameter*, e.g. `f(x){ if(n==K) reach_error(); }`) needs arg→param equalities and is deferred behind Slice S.

---

## File Structure

| File | Responsibility | Action |
|---|---|---|
| `crates/saf-svcomp/src/property.rs` | Slice 0: `resolve_nondet_sequence_interproc` (frame-aware nondet walk) + make `resolve_nondet_sequence` a thin wrapper. Slice 1: `enumerate_false_candidates_interproc` + the bounded call-chain enumerator + cross-frame path assembly. | Modify |
| `crates/saf-analysis/src/z3_utils/reachability.rs` | Slice 1: expose a `pub` block-path enumerator (`block_paths_between`) wrapping the private `enumerate_paths`, so the saf-svcomp chain builder can enumerate intra-frame paths to a call site. | Modify |
| `crates/saf-svcomp/src/lib.rs` | Slice 1: re-export `enumerate_false_candidates_interproc`. | Modify |
| `crates/saf-cli/src/commands.rs` | Slice 1: in `unreach_strategy`, after the existing intraprocedural candidates fail to confirm, enumerate + replay interprocedural candidates through the SAME `replay_confirms_false` gate. No driver/replay change. | Modify |
| `crates/saf-analysis/src/z3_utils/solver.rs` | **Slice 2 (GO only):** `check_feasibility_with_model` variant (or extra param) accepting `(formal, actual)` `ValueId` equalities injected via the shared `var_cache`. Slice 0/1: **unchanged.** | Modify (gated) |
| `crates/saf-cli/tests/` (`verify_*` e2e, Docker `#[ignore]`) | Slice 1e/2e: interprocedural fixtures confirm `false`; measurement. | Create/Modify |
| `crates/saf-svcomp/tests/` + inline `#[cfg(test)]` | Unit tests for the nondet walk, chain enumerator, arg binding. | Create/Modify |
| `scripts/svcomp_verify_eval.py` | Slice 1e: report recall lift over the 7/120 baseline + **miss-shape categorization** (interproc-confirmed / main-guard / param-guard / pointer-float-heap out-of-scope). | Modify |
| `plans/PROGRESS.md` | Plan-196 index entry + Next Steps + Session Log. | Modify |

**Slice order (each ends at a VM-green, independently reviewable deliverable):**
`0` frame-aware nondet walk (refactor, recall-neutral) → `1` interprocedural candidates rooted at `main` (Shape 1) → `1e` e2e + measurement → **`S` Shape-2 payoff spike (GATE — hard stop)** → *[GO only]* `2a` arg→param equalities in the solver → `2b` wire arg-binding into the enumerator → `2e` e2e + measurement.

`0` is independent and lands first. `1` depends on `0`. `1e` depends on `1`. `S` depends on `1e` (needs the interproc candidate shape + a measured miss set). `2a`→`2b`→`2e` run **only** after `S` returns GO (auto-proceed on GO, stop on NO-GO — user, 2026-08-11).

---

## Slice 0 — frame-aware whole-program nondet walk (refactor; recall-neutral)

**Files:** `crates/saf-svcomp/src/property.rs` (+ inline unit tests).

**Interfaces:**
- Produces: `fn resolve_nondet_sequence_interproc(module: &AirModule, path: &[(FunctionId, BlockId)], assignments: &BTreeMap<ValueId, i64>) -> Vec<NondetCall>` — walks a cross-frame `(func, block)` sequence **in execution order**, collecting scalar-int nondets with one slot each (model value or `0`).
- `resolve_nondet_sequence(module, func_id, block_path, assignments)` becomes a thin wrapper that maps `block_path` to `(func_id, b)` pairs and delegates. **No behavior change for existing single-frame callers** (recall-neutral).

- [ ] **0.1 — Failing unit test: cross-frame order.** Synthetic 2-function module: `main` block `M` reads `__VERIFIER_nondet_int` (result `%a`), then a `CallDirect` to `f`; `f` block `Fb` reads `__VERIFIER_nondet_int` (result `%b`). `assignments = {%a: 7, %b: 9}`. Assert `resolve_nondet_sequence_interproc(module, &[(main,M),(f,Fb)], &assignments)` == `[NondetCall{int,7}, NondetCall{int,9}]` in exactly that order.

```rust
#[test]
fn interproc_nondet_walk_preserves_execution_order() {
    // main: %a = nondet_int(); call f();   f: %b = nondet_int();
    let (m, main_id, f_id, m_blk, f_blk, a, b) = build_two_frame_nondet_module();
    let assigns = BTreeMap::from([(a, 7_i64), (b, 9_i64)]);
    let seq = resolve_nondet_sequence_interproc(&m, &[(main_id, m_blk), (f_id, f_blk)], &assigns);
    assert_eq!(
        seq,
        vec![
            NondetCall { func_name: "__VERIFIER_nondet_int".into(), value: 7 },
            NondetCall { func_name: "__VERIFIER_nondet_int".into(), value: 9 },
        ]
    );
}
```

- [ ] **0.2 — Failing unit test: per-name subsequence + zero-fill.** Add a `__VERIFIER_nondet_uint` read (`%c`, unmodeled) in `f` between two `int` reads; assert the `int` slots keep call order and the unmodeled `uint` slot is `0` (mirrors `nondet_sequence_zero_fills_unconstrained_in_call_order` at `property.rs:2516`).
- [ ] **0.3 — Failing unit test: wrapper equivalence.** For a single-frame path, assert `resolve_nondet_sequence(m, f_id, &blocks, &a)` == `resolve_nondet_sequence_interproc(m, &blocks.map(|b|(f_id,b)), &a)` (proves the refactor is behavior-preserving).
- [ ] **0.4 — Run 0.1–0.3; expect FAIL** (`docker compose run --rm dev sh -c 'cargo test -p saf-svcomp resolve_nondet'`).
- [ ] **0.5 — Implement.** Move the body of `resolve_nondet_sequence` into `resolve_nondet_sequence_interproc(module, path, assignments)`: iterate `path` in order; for each `(func_id, block_id)` resolve the function + block; iterate the block's instructions in order; for each `CallDirect` whose callee name passes `is_scalar_integer_nondet`, push `NondetCall { func_name, value: inst.dst.and_then(|d| assignments.get(&d).copied()).unwrap_or(0) }`. Reimplement `resolve_nondet_sequence` as: `resolve_nondet_sequence_interproc(module, &block_path.iter().map(|b| (func_id, *b)).collect::<Vec<_>>(), assignments)`.
- [ ] **0.6 — Run 0.1–0.3; expect PASS.**
- [ ] **0.7 — VM green:** `cargo nextest run -p saf-svcomp && cargo clippy -p saf-svcomp -- -D warnings && cargo fmt --check`. (Commit only when the user asks.)

**Why this is soundness-neutral:** it changes only how the sequence is *computed*, not any reachability or verdict logic; the single-frame path produces the identical sequence (0.3). No candidate is enumerated differently yet — recall is unchanged.

---

## Slice 1 — interprocedural candidates rooted at `main` (Shape 1; no arg binding)

**Files:** `crates/saf-svcomp/src/property.rs`, `crates/saf-analysis/src/z3_utils/reachability.rs`, `crates/saf-svcomp/src/lib.rs`, `crates/saf-cli/src/commands.rs`.

**Interfaces:**
- Consumes: `resolve_nondet_sequence_interproc` (Slice 0); `extract_guards_from_blocks`, `extract_assume_guards`, `ValueLocationIndex`, `PathFeasibilityChecker::check_feasibility_with_model` (existing pub primitives); `CallGraph::build`.
- Produces:
  - `pub fn block_paths_between(from: BlockId, to: BlockId, func_id: FunctionId, module: &AirModule, max_paths: usize) -> Vec<Vec<BlockId>>` (`reachability.rs`) — thin pub wrapper over the private `enumerate_paths` (+ `Cfg::build`), returning simple block paths within one function.
  - `pub fn enumerate_false_candidates_interproc(module: &AirModule, config: &PropertyAnalysisConfig) -> Vec<FalseCandidate>` (`property.rs`) — builds the `CallGraph` internally; for each `reach_error` site, enumerates bounded call chains `main→…→F`, assembles cross-frame block sequences, joint-solves, and emits replay-gated candidates whose `nondet_sequence` spans the whole program. Bounds via `config` (add `max_call_depth: usize` default `3`, `max_chains_per_site: usize` default `4`).
- Config: extend `PropertyAnalysisConfig` with `pub max_call_depth: usize` and `pub max_chains_per_site: usize` (defaults in its `Default`).

- [ ] **1.1 — Failing unit test: Shape-1 candidate spans two frames with a whole-program nondet sequence.** Module: `main` reads `%x = __VERIFIER_nondet_int()`, `CondBr(%x == 42)` to a block that `CallDirect`s `buggy`, else returns; `buggy` unconditionally `CallDirect`s `reach_error`. Assert `enumerate_false_candidates_interproc` yields ≥1 candidate whose `nondet_sequence` contains `NondetCall{int, 42}` (the joint model pins `main`'s nondet) and whose `reach_error_inst` is `buggy`'s error call.

```rust
#[test]
fn interproc_candidate_pins_main_nondet_for_callee_error() {
    // main: %x = nondet_int(); if (%x == 42) buggy(); else return;
    // buggy: reach_error();
    let m = build_main_guard_calls_buggy_module(/* K = */ 42);
    let cands = enumerate_false_candidates_interproc(&m, &PropertyAnalysisConfig {
        conservative: false, ..Default::default()
    });
    assert!(cands.iter().any(|c| c.nondet_sequence.iter()
        .any(|n| n.func_name == "__VERIFIER_nondet_int" && n.value == 42)),
        "joint model must pin main's steering nondet to 42; got {cands:#?}");
}
```

- [ ] **1.2 — Failing unit test: recursion / depth bounding.** A self-recursive `main→g→g→…` module must not hang or explode: assert `enumerate_false_candidates_interproc` returns within the bound (no panic; chain count ≤ `max_chains_per_site`), using the `visiting: BTreeSet<FunctionId>` guard + `max_call_depth`.
- [ ] **1.3 — Failing unit test: determinism.** Two calls on the same module return identical `Vec<FalseCandidate>` (block paths + nondet sequences byte-equal) — guards the sorted-iteration requirement.
- [ ] **1.4 — Run 1.1–1.3; expect FAIL.**
- [ ] **1.5 — Implement `block_paths_between`** in `reachability.rs`: `let cfg = Cfg::build(module.function(func_id)?); enumerate_paths(from, to, &cfg, max_paths)` (make `enumerate_paths` `pub(crate)` or inline the wrapper; keep deterministic BFS).
- [ ] **1.6 — Implement the chain enumerator** (`property.rs`, private): `fn error_call_chains(module, callgraph, error_func: FunctionId, max_depth, max_chains) -> Vec<Vec<FunctionId>>` — bounded reverse DFS from `error_func` up to `main` via `callgraph` callers, `visiting: BTreeSet<FunctionId>` to break cycles, sorted caller iteration, truncated at `max_depth` / `max_chains`. Each result is `[main, …, error_func]`.
- [ ] **1.7 — Implement `enumerate_false_candidates_interproc`.** Build `index = ValueLocationIndex::build(module)`, `callgraph = CallGraph::build(module)`, `checker = PathFeasibilityChecker::new(config.z3_timeout_ms)`. For each `(func_id, block_id, inst_id)` in the `reach_error` sites (`find_calls_to`): for each `chain` from `error_call_chains(...)`: assemble the cross-frame `block_seq: Vec<(FunctionId, BlockId)>` by, for each adjacent `(caller, callee)` in the chain, taking one `block_paths_between(caller_entry, callsite_block_of(caller, callee), caller, module, config.max_paths)` (bounded; pick the first, iterate up to a small cap) and, in the final frame `F`, `block_paths_between(F_entry, block_id, F, ...)`; concatenate. Then `let mut pc = extract_guards_from_blocks(&block_seq, &index); pc.guards.extend(extract_assume_guards(&block_seq, module, &index));` and `let (feas, model) = checker.check_feasibility_with_model(&pc, &index);`. On `FeasibilityResult::Feasible`, push `FalseCandidate { reach_error_inst: inst_id, block_path: /* F-local tail of block_seq's block ids */, assignments: model.clone(), nondet_sequence: resolve_nondet_sequence_interproc(module, &block_seq, &model) }`. Skip `Infeasible`/`Unknown`. **Do NOT root at `F` when `F == main`** (that is the existing intraprocedural case; leave it to `enumerate_false_candidates`).
  - Conservative guard (redline #5): if a frame's traced sub-path contains a `CallDirect` to a *different* defined function that itself reads scalar nondets and returns (a nondet-reading sibling call whose reads would interleave the FIFO), **skip that chain** (its whole-program order can't be computed exactly) → fewer candidates, never a desynced one. Since replay gates anyway, a skipped chain is a recall loss, not a wrong verdict.
- [ ] **1.8 — Run 1.1–1.3; expect PASS.** VM green (`cargo nextest run -p saf-svcomp` + clippy + fmt).
- [ ] **1.9 — Wire into the verdict** (`commands.rs`, `unreach_strategy`). After the existing intraprocedural replay loop (`:1103-1130`) fails to return, add a second loop over `enumerate_false_candidates_interproc(ctx.module, &config)` (same `MAX_REPLAY_CANDIDATES` cap), each candidate routed through the **unchanged** `replay_confirms_false`; on `Ok(true)`, build the witness (`lower_candidate`; for interproc candidates it degrades to target-only, which is sound and still confirms via the `cpa-witness2test` route) and return `false(unreach-call)`. On no confirmation, fall through to `unknown_outcome()`.

```rust
// after the intraprocedural loop, before the `if candidates.is_empty()` diag:
let interproc = saf_svcomp::enumerate_false_candidates_interproc(ctx.module, &config);
for (idx, candidate) in interproc.iter().take(MAX_REPLAY_CANDIDATES).enumerate() {
    match replay_confirms_false(ctx.input, ctx.data_model, ctx.stub, ctx.tempdir, ctx.clang,
                                MAX_REPLAY_CANDIDATES + idx, candidate) {
        Ok(true) => {
            let witness = build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, candidate));
            return VerdictOutcome { verdict: format!("false({})", Property::UnreachCall.name()), witness };
        }
        Ok(false) => {}
        Err(e) => eprintln!("saf verify: interproc replay of candidate {idx} errored: {e:#} -> continue"),
    }
}
```

- [ ] **1.10 — VM green:** `cargo nextest run -p saf-svcomp -p saf-analysis -p saf-cli` + `cargo clippy --workspace -- -D warnings` + `cargo fmt --check`. Confirm no regression in the existing verify unit suite.

---

## Slice 1e — e2e validation + measurement (VM)

**Files:** `crates/saf-cli/tests/` (extend the `verify` e2e); `scripts/svcomp_verify_eval.py`; `crates/saf-svcomp/tests/programs/` fixtures.

- [ ] **1e.1 — Failing e2e (Docker `#[ignore]`): Shape-1 fixture confirms `false`.** Add `unreach_false_interproc_nondet.c`:

```c
extern int __VERIFIER_nondet_int(void);
extern void reach_error(void);
void buggy(void) { reach_error(); }
int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x == 42) buggy();   // steering nondet + guard in main; error in callee
    return 0;
}
```

`saf verify --property unreach-call.prp --data-model ILP32 unreach_false_interproc_nondet.c` → stdout `false(unreach-call)` (today: `unknown`), and the emitted witness is witnesslint-clean. Add a **negative** fixture where the guard is unsatisfiable (`if (x == 42 && x == 7)`) → stays `unknown` (soundness).
- [ ] **1e.2 — Run; expect FAIL, then iterate** until the Shape-1 fixture confirms and the negative fixture stays `unknown`.
- [ ] **1e.3 — Witness check.** Run `scripts/validate_witness.sh` on the fixture; assert CONFIRMED (analysis or `cpa-witness2test`). Assert the plan-194/195 fixtures still confirm (no regression).
- [ ] **1e.4 — Measurement sweep + miss categorization.** Extend `svcomp_verify_eval.py` to classify each expected-false miss as: `interproc-confirmed` (new), `main-guard` (Shape 1 residual), `param-guard` (Shape 2 — needs Slice 2), `pointer/float/heap/loop` (out of scope), or `frontend-ingest` (pre-existing segfault, NOT R4). Run `EVAL_CONFIRM_WITNESS=1 EVAL_CONFIRM_TIMEOUT=250` over the plan-195 N=120 sample (240 tasks, ILP32+LP64). Report **recall vs the 7/120 baseline**, false-alarm count (**must be 0**), TRUE count (**must be 0**), confirmed-witness %, and the miss-shape histogram.
- [ ] **1e.5 — VM green + record.** `make fmt && make lint` + `cargo nextest run -p saf-cli --run-ignored all -E 'test(verify)'`; write the numbers + the miss histogram into this plan's evidence section + `plans/PROGRESS.md`. **The miss histogram sizes Slice S's payoff** (how many `param-guard` misses Shape 2 would unlock).

**Honest expectation:** Slice 1 cracks the `main-guard` shape (steering nondet + guard in the same frame). If the reservoir's interprocedural misses are dominated by `param-guard`, the Slice-1 lift may be modest and the histogram will say so — that is the signal that decides Slice S.

---

## Slice S — Shape-2 payoff spike (GATE — no production code)

**Purpose:** before any solver/arg-binding work, prove (1) callee param-guards reference `params[i].id` on the real post-`mem2reg` IR (not a renamed SSA value), and (2) there is a real payoff (`param-guard` misses exist in the reservoir). **No source is edited.** **Gate (user, 2026-08-11): GO → auto-proceed to Slice 2a; NO-GO → stop & report.**

- [ ] **S.1 — AIR reference check.** On the VM, ingest a minimal Shape-2 fixture (`x=nondet_int(); f(x); f(int n){ if(n==42) reach_error(); }`) via the real `compile_to_ir` (`clang -g -O0 -disable-O0-optnone` → `opt -passes=mem2reg`) and dump the AIR (`saf index` / debug print). Confirm: does `f`'s `CondBr` guard operand equal `f.params[0].id`, or a renamed SSA value? Record the exact relationship (this is the one assumption Slice 2 rests on; absint/SVFG bind positionally but that is evidence, not proof for the guard-operand case).
- [ ] **S.2 — Payoff check.** From the Slice-1e miss histogram, confirm ≥1 real reservoir task is a `param-guard` miss (scalar-int arg). Hand-assemble the arg→param equality on the S.1 fixture (conceptually) and confirm that pinning `main`'s nondet through the callee guard would reproduce under replay.
- [ ] **S.3 — GO/NO-GO checkpoint.**
  - **GO** iff (S.1) callee guards reference `params[i].id` (directly, or via a mechanically-recoverable rename map) **and** (S.2) ≥1 `param-guard` payoff task exists. → **Auto-proceed to Slice 2a** (post a short GO note: the S.1 relationship + the payoff count); no sign-off needed.
  - **NO-GO** → **STOP and report** the blocker (renamed SSA with no cheap map, or no `param-guard` payoff in the reservoir). Slice 0/1 already shipped the recall win; do not build Slice 2 without the user.

---

## Slice 2a — arg→param equalities in the joint solver *(GO only)*

**Files:** `crates/saf-analysis/src/z3_utils/solver.rs` (+ unit tests).

**Interfaces (pin the exact shape from S.1):**
- Produces: `check_feasibility_with_model_bound(&self, pc: &PathCondition, bindings: &[(ValueId, ValueId)], index) -> (FeasibilityResult, BTreeMap<ValueId,i64>)` — like `check_feasibility_with_model` but asserts `var(formal_i) == var(actual_i)` for each `(formal, actual)` pair via the shared `var_cache` (`solver.rs:243`/`:331-348`), so a callee param-guard constrains the caller's actual argument.

- [ ] **2a.1 — Failing unit test.** A `PathCondition` with a guard `n == 42` (on formal `n`) plus a binding `(n, x)` where `x` is a nondet operand; assert the returned model has `x == 42`. Add a negative test (guard `n == 42` with binding `(n, x)` and a second guard `x == 7`) → `Infeasible`.
- [ ] **2a.2 — Run; expect FAIL.**
- [ ] **2a.3 — Implement** by injecting the equality asserts into the existing joint-feasibility path, reusing the shared `var_cache` so `formal` and `actual` map to the same Z3 var (or an explicit `Int == Int` assert). Keep the fresh-var over-approximation for everything else (still replay-gated).
- [ ] **2a.4 — Run; expect PASS.** VM green (`cargo nextest run -p saf-analysis` + clippy + fmt).

---

## Slice 2b — wire arg-binding into the interprocedural enumerator *(GO only)*

**Files:** `crates/saf-svcomp/src/property.rs`.

- [ ] **2b.1 — Failing unit test.** Shape-2 module (`main: %x=nondet_int(); f(%x);  f(n): if(n==42) reach_error();`): assert `enumerate_false_candidates_interproc` yields a candidate whose `nondet_sequence` pins `__VERIFIER_nondet_int == 42` (the joint model binds `%x==n` and solves `42`).
- [ ] **2b.2 — Run; expect FAIL.**
- [ ] **2b.3 — Implement.** In the chain assembly, at each `(caller, callee)` call site, collect `(callee.params[i].id, actual_operand_i)` pairs from the `CallDirect` operands (positional, per S.1), accumulate across the chain, and call `check_feasibility_with_model_bound(&pc, &bindings, &index)` instead of the unbound solver. `resolve_nondet_sequence_interproc` is unchanged (it reads the model). Skip non-scalar-int actuals (pointer/float args stay unpinned → unknown, sound).
- [ ] **2b.4 — Run; expect PASS.** VM green.

---

## Slice 2e — Shape-2 e2e + measurement *(GO only)*

**Files:** `crates/saf-cli/tests/`; `scripts/svcomp_verify_eval.py`.

- [ ] **2e.1 — e2e (Docker `#[ignore]`).** `unreach_false_interproc_argguard.c` (`f(x)` with `if(n==42) reach_error()`) → `false(unreach-call)`; a variant with an unsatisfiable arg guard stays `unknown`.
- [ ] **2e.2 — Measurement sweep.** Re-run the N=120 sample; report recall at **baseline → +Slice 1 → +Slice 2**, the miss histogram, 0 false alarms, 0 TRUE, byte-stable.
- [ ] **2e.3 — VM green + record** (`make fmt && make lint && make test`); update `plans/PROGRESS.md` (Next Steps → R5 or the next lever).

---

## Soundness & determinism invariants (must hold; mapped to slices)

1. **No `true` ever / FALSE only via must-reach OR replay** — R4 adds only replay-gated candidates (Slice 1/2); Stage-1 `must_reach_error` untouched (redlines #1, #2, #4).
2. **No new `false` site** — interproc candidates flow through the existing `replay_confirms_false → Ok(true)` gate (`commands.rs:1122` pattern); every other arm → `unknown` (redline #3; 1.9, 2e).
3. **Fail-closed** — a spurious joint-SAT / wrong nondet order / unbound arg → replay does not reproduce → `unknown` (recall loss, never −16). Verified by the negative fixtures (1e.1, 2e.1) and unit negatives (2a.1, 2b).
4. **Bounded** — `max_call_depth`, `max_chains_per_site`, `max_paths`, `MAX_REPLAY_CANDIDATES`, recursion `visiting`-set + `tarjan_scc`; the number of dynamic nondet calls on any confirmed path is statically bounded (redline #5; 1.2). Unboundable → skip chain → `unknown`.
5. **Determinism** — sorted iteration only; `BTreeMap`/`BTreeSet`; no `HashMap`/`now()`/RNG; Z3 `rlimit`+`random_seed` pinned. Verified (1.3); goldens re-pinned if witness bytes change.
6. **Driver unchanged** — `synthesize_driver` + `replay_confirms_false` are not edited; the per-name FIFO already replays whole-program nondets (verified). Any PR touching them in Slice 1 is a review stop.

---

## Acceptance criteria & evidence

1. **Blind recall ↑ over the 7/120 baseline** on the N=120 sample (the headline; 1e.4, 2e.2), holding **0 false alarms, 0 TRUE**.
2. **Soundness preserved** — 0 −16/0 −32 across the sweep; negative fixtures stay `unknown`.
3. **Miss-shape histogram** recorded (1e.4) — sizes Slice S and documents the residual (param-guard / pointer-float / heap-loop / frontend-ingest).
4. **Byte-stable** verdicts + witnesses (re-run identical).
5. **Slice S gated on evidence** — GO/NO-GO recorded; Slice 2 code exists only on a GO.
6. **Guardrails green** — `make test`, clippy `-D warnings`, fmt; plan-192/194/195 e2e suites unchanged.
7. **PROGRESS.md updated** (1e.5, 2e.3).

---

## Risks & open questions

- **Path/chain explosion** — the chain × intra-frame-path product can blow up; bound aggressively (`max_call_depth=3`, `max_chains_per_site=4`, `max_paths`), and the watchdog only downgrades to `unknown`. Measure worst-case CPU on the sweep.
- **`mem2reg` param reference (blocks Slice 2 only)** — the one Slice-2 assumption; Slice S is the hard gate for it. Slice 0/1 are unaffected.
- **Nondet-reading sibling calls** — a returning callee that reads nondets interleaves the FIFO; Slice 1 conservatively **skips** such chains (recall loss, never a desynced FALSE). A later slice could model returns.
- **Multi-caller `F`** — the enumerator picks specific chains (bounded `max_chains_per_site`); single-chain-per-candidate precision (like `CallerGuardContext`'s single-caller-only) is acceptable for R4; multi-caller disjunction is deferred.
- **`CallIndirect`** — current sound behavior bails on indirect calls everywhere; R4 **keeps bailing** (no PTA/CHA chain resolution) — a later lever.
- **Witness for interproc candidates** — `lower_candidate` walks an `F`-local `block_path` → target-only witness for interproc candidates (sound; confirms via `cpa-witness2test`). Cross-frame branching/assumption waypoints are a later enrichment (plan 195 Tier B territory).
- **Frontend-ingest segfaults** (e.g. the `tdg…` shape) are a **separate** recall limiter (frontend robustness), **not R4** — categorized separately in the histogram, not conflated.

---

## Out of scope (deferred)

- Shape 2 arg-binding **unless** Slice S returns GO.
- Bounded concrete search (plan 192 approach B) — largely redundant with today's replay for constants (verified); revisit only if a measured shape needs it.
- Interprocedural branching/assumption **witness** enrichment (target-only is sound + confirms).
- Return-value flow (callee `Ret` → caller `dst`) constraints; pointer/float/heap/loop reachability; recursion-cycle steering; `CallIndirect` chains.
- R5+ (memsafety/overflow/termination/race/packaging/memcleanup) per plan 193.

---

## VM workflow (all builds/experiments here; never the laptop)

```bash
# sync source to the VM (see [[saf-svcomp-vm-env]])
rsync -az --delete <repo>/crates/  ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az        <repo>/scripts/   ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/scripts/   # WITHOUT --delete
# focused unit (Slice 0/1)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-svcomp -p saf-analysis"'
# e2e verify tests (Docker, ignored)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-cli --run-ignored all -E \"test(verify)\""'
# measurement sweep + miss histogram
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm -e EVAL_CONFIRM_WITNESS=1 -e EVAL_CONFIRM_TIMEOUT=250 dev sh -c "python3 scripts/svcomp_verify_eval.py …"'
# full guardrail
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make fmt && make lint && make test'
```
Let `cargo` regenerate `Cargo.lock` on the VM; sync it back before any commit. **Commit only when the user asks.**

---

## Implementation record (2026-08-11, VM-verified, uncommitted)

TDD on the VM (`ubuntu@cd-vm-15-ai-vm`, Docker); laptop = git source of truth; **uncommitted** (commit when the user asks).

**Slice 0 — frame-aware whole-program nondet walk — DONE, VM-green.** Extracted the nondet-collection body of `resolve_nondet_sequence` into `resolve_nondet_sequence_interproc(module, path: &[(FunctionId, BlockId)], assignments)` which walks a cross-frame `(func, block)` sequence in execution order (a caller's pre-call reads precede the callee's), one slot per scalar-int nondet call (unmodeled → `0`), each tagged with the real callee name. `resolve_nondet_sequence` is now a thin wrapper mapping to single-frame `(func_id, b)` pairs — behavior-preserving. 3 unit tests. Recall-neutral, soundness-neutral. Gates: **65/65** saf-svcomp nextest, clippy `--workspace -D warnings` clean, fmt clean.

**Slice 1 — interprocedural candidates rooted at `main` (Shape 1) — DONE, VM-green.** New (`saf-svcomp/src/property.rs` unless noted): `block_paths_between` (`reachability.rs`, **pub**, wraps the private `enumerate_paths`); `build_direct_call_sites` (`callee → {(caller, call_block)}`, deterministic); `error_call_chains`/`collect_chains` (bounded reverse DFS `main → … → F`, `visiting` cycle-break, `max_call_depth`=3 + `max_chains_per_site`=4 new config fields); `assemble_interproc_path` (stitches each caller frame's entry→call-site path + the error frame's entry→reach_error path into a cross-frame `Vec<(FunctionId, BlockId)>`); `enumerate_false_candidates_interproc` (**pub**, re-exported) — for each `reach_error` NOT in `main`, per bounded chain: existing `extract_guards_from_blocks` (already takes `(FunctionId, BlockId)` pairs) + `extract_assume_guards` → joint `check_feasibility_with_model` (disjoint-frame ValueIds ⇒ one model over all frames' nondets; **no arg→param binding — Shape 1**) → `FalseCandidate` with the whole-program `nondet_sequence` (Slice 0) + error-frame `block_path`. **Wiring** (`commands.rs::unreach_strategy`): Stage-4 loop replays interproc candidates through the **unchanged** `replay_confirms_false` gate (index offset past the intraproc batch). **No change** to `synthesize_driver`/`replay_confirms_false`/the two `false` sites. 3 unit tests (pins main's nondet to 42 for a callee error; recursion bounded; determinism). Gates: **1817/1817** nextest (saf-svcomp+saf-analysis+saf-cli), clippy `--workspace -D warnings` clean, fmt clean.

**Slice 1e — e2e (Docker) — DONE.** `unreach_false_interproc_nondet.c` (`x=nondet(); if(x==42) buggy(); buggy(){reach_error();}`) → **`false(unreach-call)`** — for this shape `false` can ONLY come from R4 (must-reach bails at the guard; the intraprocedural enumeration roots at `buggy` with an empty nondet sequence → replay leaves `x=0` → never calls `buggy`). `unreach_false_interproc_unsat.c` (`if(x>5 && x<3)`) → **`unknown`** (soundness negative). Pre-existing `verify_false_interproc` (unconditional, must-reach) still passes — no regression. **End-to-end proof R4 recovers a previously-missed FALSE while staying sound.**

**Slice 1e measurement — DONE (honest finding: no reservoir recall lift).** N=120 blind sweep (`EVAL_N_PER_CLASS=120`, 240 tasks, ILP32+LP64): **recall 7/120, 0 false alarms, 0 TRUE** — *identical* to plan-195's 7/120 baseline (same 7 tasks: `data_structures`, `test_mutex`, `simple-ext`, `maxpool`, `elevator`, `fibo`, `transmitter`). **R4 Slice 1 added zero reservoir recall.** The N=120 misses are dominated by loops/arrays/concurrency/heap/weak-memory/huge-driver/deep-recursion (`pals_*`, `safe*_rmo/power`, `drivers--*.cil`, `rangesum`, `cs_stack`, `CostasArray`, `tdh_*`) — none are interprocedural scalar-nondet steering.

**Root-cause diagnostic (26 interprocedural-heavy expected-false tasks: recursive/recursive-simple/bitvector/ldv-regression):** 18/26 caught, but **R4 fired 0 meaningful interproc candidates** — every catch is the EXISTING intraproc-enumerate + real-program replay (`reach_error` in `main` guarded by a callee's return value, e.g. `if(fibo(x)==k) reach_error()`; or constant-driven recursion — the driver runs the real program so constants/deterministic returns reproduce). **R4's exact shape — `reach_error` in a CALLEE, steered by `main`'s scalar nondet through a guard — is rare in sv-benchmarks.**

**Slice S (Shape-2 spike) — done; findings:** (1) **mem2reg param-ref: PASS** — for the `f(int n){ if(n==42) reach_error(); }` shape, post-`mem2reg` IR is `%2 = icmp eq i32 %0, 42` guarding directly on the parameter `%0`, so arg→param binding is mechanically feasible (no rename layer). (2) **R4 fires on real Shape-2 IR** — the crafted `unreach_false_interproc_argguard.c` yields the diagnostic "2 candidate(s) enumerated (1 intraproc + 1 **interproc**); none reproduced" → Slice 2 (arg binding) WOULD catch it. (3) **BUT no reservoir payoff task found** — Shape-2 (nondet→callee-param-guard) appears as rare as Shape-1 in the sampled reservoir. **Per the plan's gate (GO requires ≥1 reservoir payoff task), this is a NO-GO for Slice 2 on ROI grounds** pending a broader prevalence scan or a user decision.

**Prevalence scan (user-requested; the decisive measurement, 2026-08-12).** Strided sample of **532** tasks across the **5,104** expected-false unreach reservoir (≤300 KB; a per-task `eprintln!` reports when R4's interproc enumerator *fires*, i.e. reaches the Stage-4 loop because must-reach + the intraprocedural candidates did NOT already confirm):
- **R4 fired on 165/532 (31%)** — the interprocedural `main → callee reach_error` shape is **common, NOT rare** (the earlier 146-task samples were unlucky: recursive-simple has `reach_error` in `main`; the N=120 stride clustered on already-caught/out-of-scope tasks).
- **R4 caught 0/165.** Slice-1 (no arg binding) converts **none** of its fires.
- **Fired-but-missed categories:** `tdh`+`tdg` 66 (TDX memory-havoc), `pals`+`safe`+`mix` 42 (concurrency / weak-memory / `pthread-wmm`), `aws` 11 (memory-safety harnesses), `Problem` 12 (eca-rers — reach_error in `calculate_output(int input)` gated on `input` **plus** loop-driven global state `a29/a25/a23/...` → multi-iteration, not single-path arg-binding), plus Linux LDV driver `.cil.out.i` ~18 (memory/loop/huge/stateful-infinite), array-copy/list/openssl a handful. **≈95% of the fires need capabilities R4 fundamentally lacks** (loops/multi-input, memory/heap, concurrency/weak-memory, arrays) — they fire only because a guarded callee `reach_error` path structurally exists.
- **Genuine Slice-2 (scalar single-path Shape-2) surface: ~5-6 tasks / 532 (<1.2%), and uncertain** (the coreutils remainder likely needs string/array reasoning).

**Net R4 outcome (honest, evidence-backed):** Slices 0+1 are a **sound, correct, e2e-proven interprocedural spine** (0 FA / 0 TRUE, no regression, fires on 31% of real tasks) but deliver **0 reservoir recall** — and **Slice 2 (arg→param binding) would add at most a handful of tasks (<1.2%)**, because the reservoir's missed unreach FALSEs need **loops/multi-input, memory/heap, concurrency, and arrays** — exactly the big-engine capabilities plan 193 says SAF lacks (R5+), NOT scalar interprocedural composition. **Recommendation: STOP R4 at the Slice-0/1 spine; do not build Slice 2.** This strongly corroborates plan 195's finding and refines plan 195's premise that "R4 is the dominant next lever" — the *shape* is common but the *bug-reasoning* required is out of scope. The sound spine + the R4-fire diagnostic remain as foundation if future loop/arg/memory engines land.
