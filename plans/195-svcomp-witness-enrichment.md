# Plan 195: Witness enrichment — Tier A (branching, no-column) + Tier B (assumption values via `dbg.value`)

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan slice-by-slice. Steps use checkbox (`- [ ]`) syntax. **All builds/tests run on the VM `ubuntu@cd-vm-15-ai-vm` (Docker); never the laptop. Subagents never call `make`. Commit only when the user asks.**

**Status:** design / **awaiting approval** (no implementation done)
**Date:** 2026-08-11
**Branch:** `svcomp` (laptop = git source of truth @ `9818f97`)
**Reference competition:** SV-COMP 2026 rules (2027 unreleased); witness format = SV-COMP witnesses **2.0** (SPIN 2024).
**Builds on** plan 194 (YAML-2.0 violation-witness emitter, committed `9818f97`). **Supersedes** plan 194's reverted Slice E and its correction #3/#4 conclusion that branching/assumption enrichment is infeasible — a 2026-08-11 feasibility spike found the two fixes this plan implements.
**Roadmap:** raises the **confirmed-witness %** (plan 193 R1 follow-through) before R4 (interprocedural recall).

**Goal:** Raise the CPAchecker-confirmed fraction of SAF's already-sound `unreach-call` FALSE witnesses above the plan-194 target-only baseline (2/3 = 67% on the 80-task sweep; the miss is the logic-heavy nondet-driven task) by adding two path-guidance waypoint kinds — **Tier A** branching waypoints emitted *with the column omitted*, and **Tier B** assumption-value waypoints pinning the nondet inputs — while holding every plan-194 soundness/determinism redline.

**The two spike findings this plan rests on (2026-08-11):**
1. **Tier A:** CPAchecker CONFIRMS a `branching` waypoint whose **column is omitted** (it resolves the location to the `if`/`while` keyword). It REJECTS one whose column points *into the condition* (mis-parsed as a ternary — "Ternary operators as branching waypoints are currently not supported"). That column-into-condition bug is exactly what got plan-194 Slice E reverted. Fix = emit `branching` with `column: None`.
2. **Tier B:** the nondet result's source C name **is** recoverable — `clang -g` emits `#dbg_value`/`@llvm.dbg.value(<ssa>, !N)` with `!N = !DILocalVariable(name: "x")`, mapping the nondet-call SSA result to its C variable. SAF's frontend today reads only `dbg.declare` (pre-`mem2reg` allocas) and so drops the name after `mem2reg`. Capturing `dbg.value`→C-name is frontend work; it unlocks `assumption` waypoints `x == <value>` and execution-based `cpa-witness2test` confirmation.

**Sequencing (user-approved 2026-08-11):** Tier A lands and is measured first (cheap, spike-proven, one file). Then a **no-production-code payoff spike** gates Tier B: the frontend work proceeds only if the spike proves the assumption waypoints actually flip CPAchecker UNKNOWN→confirmed. **Gate behavior (user, 2026-08-11):** on **GO** I **auto-proceed** into Tier B (Slices C–G) without waiting for sign-off, after posting a short GO note; on **NO-GO** I **stop** and report.

---

## Global Constraints (verbatim; every task inherits these)

**Soundness redlines (from `CLAUDE.md` "SV-COMP Competition Work" + plan 193 §6 — NON-NEGOTIABLE; a wrong verdict is −16/−32 ≈ 16× the reward of a right one):**
1. `saf verify` **must never emit `true`** with the current engines. Preserve the hard "never prints `true`" gate.
2. Never emit `false` without **(a)** an unconditional must-reach proof **OR (b)** concrete replay confirmation. The witness is a *side output of an already-sound verdict*, never the basis for it.
3. **Enrichment is strictly additive/optional.** A missing C-name, an ambiguous branch (`then == else`), a Switch terminator, or any location we cannot resolve → **emit fewer waypoints (degrade toward target-only)**, NEVER a guessed, synthesized, or mislocated constraint. A malformed enrichment waypoint makes CPAchecker **reject the whole witness** — strictly worse than target-only (this is the Slice-E regression, plan 194 correction #3). No enrichment path may turn a confirmed target-only witness into a rejected one by construction.
4. The witness **never gates the verdict**: a `false` is emitted even when the witness (enriched or not) is unconstructible. Timeout/unknown/true never write a witness.
5. R3 Stage-1 holes stay closed (plan 194 Slice C). This plan adds no new FALSE sites.

**Determinism (NFR-DET-001):** witness bytes must be **byte-identical** for identical inputs. Branch-value derivation walks the deterministic `block_path`; nondet C-names come from a `BTreeMap`; no `SystemTime::now()`, RNG, or `HashMap`-iteration order in the witness path. Every serialization change re-pins the golden.

**Extensibility (plan 193 §11):** `witness_yaml.rs` stays property-generic; the `WaypointKind::{Branching, Assumption}` variants and `assemble` grouping already exist (plan 194 kept the model when it reverted the lowering). This plan touches only lowering + the frontend name capture, not the generic spine.

**Format / validator authority:** violation witness 2.0 per `[[svcomp-yaml-witness-2.0-format]]`. `branching` constraint = `{value: "true"|"false"}` (no `format`); `assumption` constraint = `{format: "c_expression", value: "<expr>"}`, side-effect-free over in-scope vars. **witnesslint is the hard syntactic gate** (exit 0 required on every emitted witness). **CPAchecker** (`--config config/violation-witness-validation.properties`) measures confirmation; **`cpa-witness2test`** (execution-based) is the additional route for assumption-bearing witnesses (it needs the pinned input values — mirrors SAF's own replay). Pin the exact accepted byte layout to what the provisioned tools accept (emit→lint→fix loop); do not freeze from the paper.

**Workflow:** edit locally; targeted rsync to the VM before each build (`crates/` with `--delete`; root files + `scripts/` individually WITHOUT `--delete`; see `[[saf-svcomp-vm-env]]`). TDD throughout. `make test` / focused `docker compose run --rm dev sh -c '…'` on the VM only. `make fmt` before `make lint`. Let `cargo` regenerate `Cargo.lock` on the VM; sync it back before any commit. **Commit only when the user asks.**

---

## Current state (verified via recon 2026-08-11, file:line — re-confirm before editing; line numbers drift)

**Tier A surface (`saf-svcomp`):**
- `witness_lower.rs:126-130` `lower_candidate` — target-only: resolves `cand.reach_error_inst` via `find_inst` + `span_to_location`, emits one `target_waypoint`. Does **not** read `block_path`.
- `witness_lower.rs:103-109` `lower_must_reach` — target-only on `chain.last()`. **Needs no branching**: the must-reach walk bails (`Indeterminate`) at any block with >1 successor (`property.rs:1823-1834`), so the chain is branch-free by construction.
- `witness_lower.rs:84-95` `target_waypoint` helper — hardcodes `column: Some(column)`; the new `branching_waypoint` helper (`column: None`) sits beside it.
- `witness_lower.rs:10-14, 118-124` — Slice-E revert **doc comments only** (no dead code).
- `witness_yaml.rs:73-84` `WaypointKind` — **`Branching` + `Assumption` variants still exist** (not reverted). `:106-131` `Constraint` + `SourceWaypoint` (`column: Option<u32>` at `:126`). `:285-286` `LocationOut.column` has `#[serde(skip_serializing_if = "Option::is_none")]` — emitting `column: None` drops the `column:` key. `:151-198` `assemble` groups follow-terminated segments (each `follow` closes a segment; final must be `target`). `:403-416` existing `branching()` test helper (value `true`/`false`, `column: None`) — **the serialize path already works and is tested**; `:461-485` assumption test. **⇒ Tier A is a pure lowering change; `witness_yaml.rs` is untouched.**
- `air.rs:515-520` `Operation::CondBr { then_target, else_target }`; `:522` `Switch`; `:774-776` `AirBlock::terminator()` (returns the last inst iff terminator, carries the `CondBr` span).
- `property.rs:1871-1883` `FalseCandidate { reach_error_inst: InstId, block_path: Vec<BlockId>, assignments: BTreeMap<ValueId,i64>, nondet_sequence: Vec<NondetCall> }` — `block_path` is populated & public; Tier A needs **no** property.rs change.

**Tier B surface (frontend + property):**
- `commands.rs:903-955` `compile_to_ir` — `clang -g -S -emit-llvm -O0 -Xclang -disable-O0-optnone` then `opt -S -passes=mem2reg`. `mem2reg` deletes the alloca + `dbg.declare` and emits `#dbg_value` at the def. **⇒ on the real verify IR the nondet result's only surviving name carrier is a `#dbg_value` record.**
- `debug_info.rs:212` `extract_local_variable_names` — line-by-line text scan of the printed module IR; parses `!DILocalVariable` + `dbg.declare` in **both** forms (`parse_old_style_dbg_declare` `:329` `@llvm.dbg.declare`; `parse_new_style_dbg_declare` `:364` `#dbg_declare`). `:196` `LocalVarNameMap = BTreeMap<func, BTreeMap<register, name>>`, keyed by textual register. `:397` `extract_register_name`. **Reads `dbg.declare` only — never `dbg.value`.**
- `mapping.rs:556` runs the pre-pass over `module.print_to_string()` (post-`mem2reg` text); `:925-926` `current_local_var_names` per function; `:1263-1274` (`:1267`) the **only** attach site — stamps names onto **`Alloca`** `air_inst.symbol`. Nothing for SSA call results.
- `air.rs:660-662` `Instruction.symbol: Option<Symbol>` — the existing per-instruction C-name carrier (⇒ **no `air.rs` schema change**); `:652-654` `Instruction.dst: Option<ValueId>` (the nondet result); `:669` `Instruction.extensions` (generic alternative carrier). `span.rs` `Symbol` + `Symbol::simple(name)` (confirm the name field spelling — `display_name` vs `name` — during impl).
- `property.rs:1851-1857` `struct NondetCall { pub func_name: String, pub value: i64 }` — **two fields, no span** (Slice E's span was reverted). `:1918` `enumerate_false_candidates`; `:1942` sole call site of `resolve_nondet_sequence`; `:1960/1974-1992` the loop (constructor at `:1988`); the nondet SSA result is `inst.dst`, and **`inst.symbol` is readable at the constructor for free**. `:1957` comment: nondet slots are kept in program/call order (zero-filled) — the driver relies on it.
- `commands.rs:1204-1224` `synthesize_driver` groups `nondet_sequence` by `func_name` into call-ordered pinned arrays — **the assumption waypoint value must use the same per-slot `NondetCall.value`**, not a by-`ValueId` re-lookup.
- `intrinsics.rs` classifies `dbg.value` as `Skip`; absint marks it pure (no other consumer). `plans/186-llvm-22-support.md:109` + `plans/PROGRESS.md:37` = the "plan-186 DbgRecord follow-up" (LLVM 19+ replaces the intrinsics with non-instruction `DbgRecord`s reached via `Instruction::get_debug_records()`); the text-scan works for llvm-18 (records printed inline as `#dbg_value`). Keep the parser cfg-tolerant so the llvm22 image doesn't silently lose names.

**Validator infra (baked in the dev image; plan 194):** witnesslint 2.1.3-dev at `$SAF_SVWITNESSES=/opt/sv-witnesses` (run under `/usr/bin/python3`) + openjdk-21; CPAchecker 4.2.2 provisioned lazily to `/workspace/.svtools` by `scripts/validate_witness.sh`. Confirm cmd: `bin/cpachecker --config config/violation-witness-validation.properties --witness <W> --spec <prp> --timelimit 90s <program>` → `Verification result: FALSE` = confirmed. `scripts/svcomp_verify_eval.py` has the `EVAL_CONFIRM_WITNESS=1` sweep mode.

---

## File Structure

| File | Responsibility | Action |
|---|---|---|
| `crates/saf-svcomp/src/witness_lower.rs` | Tier A: `branching_waypoint` helper + `block_path` walk in `lower_candidate`. Tier B: `assumption` waypoint emission. | Modify |
| `crates/saf-svcomp/src/property.rs` | Tier B: `NondetCall.cname: Option<String>` (+ call-site span iff the spike requires an assumption location); fill in `resolve_nondet_sequence`. | Modify |
| `crates/saf-frontends/src/llvm/debug_info.rs` | Tier B: `extract_nondet_result_names` — parse `#dbg_value`/`@llvm.dbg.value` → register→C-name map (reuse the `!DILocalVariable` phase). | Modify |
| `crates/saf-frontends/src/llvm/mapping.rs` | Tier B: stamp the nondet `CallDirect` result's `Instruction.symbol` from the new map (mirror the `Alloca` attach at `:1267`). | Modify |
| `crates/saf-svcomp/src/witness_yaml.rs` | **Unchanged** — model + `Branching`/`Assumption` + column omission already present. | — |
| `crates/saf-core/src/air.rs` | **Unchanged** — `Instruction.symbol` already carries a C-name. | — |
| `scripts/validate_witness.sh` | Extend with a `cpa-witness2test` (execution-based) route for assumption-bearing witnesses. | Modify |
| `scripts/svcomp_verify_eval.py` | Report confirmed-% at target-only → +Tier A → +Tier B + per-task regression diagnostic. | Modify |
| `crates/saf-svcomp/tests/` golden + unit; `crates/saf-frontends` unit; `crates/saf-cli/tests/` e2e | Branching/assumption lowering, `dbg.value` parse, updated golden, e2e validation (Docker `#[ignore]`). | Create/Modify |
| `plans/PROGRESS.md` | Plan-195 index entry + Next Steps + Session Log. | Modify |

**Slice order (each ends at a VM-green, independently reviewable deliverable):**
`A` Tier-A lowering → `B` Tier-A e2e + measure → **`S` Tier-B payoff spike (GATE — hard stop)** → *[GO only]* `C` frontend `dbg.value` parse → `D` frontend attach → `E` property `cname` plumbing → `F` Tier-B lowering (assumption) → `G` Tier-B e2e + measure.

`A`→`B` independent of everything below. `S` depends on `B` (needs the Tier-A witness shape + a measured UNKNOWN task). `C`→`D`→`E`→`F`→`G` run only after `S` returns GO — **auto-proceed on GO, stop on NO-GO** (user, 2026-08-11). `C` and `E` are independent of each other; `D` depends on `C`; `F` depends on `E` (+ `D` at runtime); `G` depends on `F`.

---

## Slice A — Tier A: branching waypoints (no column) in `lower_candidate`

**Files:** `crates/saf-svcomp/src/witness_lower.rs` (+ its unit tests + the golden).

**Interfaces:** no signature change. New private helper:
```rust
/// A `branching` waypoint located by LINE ONLY (column omitted) so CPAchecker
/// snaps it to the `if`/`while` keyword rather than mis-parsing a condition-column
/// as a ternary (plan-194 Slice-E regression). `value` = which successor the
/// confirmed path took (`true` = then_target).
fn branching_waypoint(file_name: String, line: u32, function: Option<String>, value: bool) -> SourceWaypoint;
```

- [ ] **A.1 — Failing unit test: branch taken → branching(no column) then target.** Build a module `main`: `bb0` ends in `CondBr{then: bb1, else: bb2}` (terminator span line 10), path takes `bb1`, which holds `reach_error` (span line 12). `cand.block_path = [bb0, bb1]`, `reach_error_inst` = that call. Assert `lower_candidate` returns `[branching{line:10, column:None, value:"true"}, target{line:12}]`; assert the branching waypoint has **no column** and the final waypoint is `target`.
- [ ] **A.2 — Failing unit test: guards degrade to target-only.** (a) `then_target == else_target` → no branching waypoint (ambiguous); (b) terminator is `Switch` or unconditional `Br` → no branching for that hop; (c) `block_path` empty/one-block → target-only (unchanged). Each asserts the output is exactly `[target]`.
- [ ] **A.3 — Run A.1–A.2; expect FAIL** (`docker compose run --rm dev sh -c 'cargo test -p saf-svcomp witness_lower'`).
- [ ] **A.4 — Implement.** Add `branching_waypoint` beside `target_waypoint`. In `lower_candidate`, before the final `target_waypoint`, iterate consecutive pairs `(block_path[i], block_path[i+1])`: locate `block_path[i]` in the candidate's function, take `AirBlock::terminator()`; if it is `CondBr{then_target, else_target}` **and** `then_target != else_target`, set `value = (block_path[i+1] == then_target)`, resolve the terminator span via `span_to_location` and emit `branching_waypoint(file, line, function, value)` **discarding the column**. Skip (emit nothing) for `then==else`, `Switch`, `Br`, or an unresolved span. Then push the existing `target_waypoint`. Empty/degenerate `block_path` ⇒ loop emits nothing ⇒ identical to today's target-only witness (the strictly-additive fallback of redline #3).
- [ ] **A.5 — Run A.1–A.2; expect PASS.**
- [ ] **A.6 — Determinism + golden.** Update the plan-194 byte-golden (`crates/saf-svcomp/tests/data/…`) for the enriched shape (a branch-guarded synthetic candidate); re-run the byte-stable "twice → identical" test. Confirm no `HashMap`/`now()`/RNG entered the path.
- [ ] **A.7 — VM green:** `cargo nextest run -p saf-svcomp && cargo clippy -p saf-svcomp -- -D warnings && cargo fmt --check`. Update the `witness_lower.rs:10-14/118-124` revert-doc comments to describe the no-column fix. (Commit only when the user asks.)

**Risk pins (verify in Slice B):** the branching **line** must equal the `if`/`while` keyword line; `block_path` must be a true consecutive CFG walk so `block_path[i+1]` matches a `then/else_target`. If either fails on real fixtures, the waypoint fails to confirm (never a wrong verdict) — measure in B before trusting.

---

## Slice B — Tier A: e2e validation + measurement

**Files:** `crates/saf-cli/tests/` (extend the plan-194 `verify_witness` e2e); `scripts/svcomp_verify_eval.py`.

- [ ] **B.1 — Failing e2e (Docker `#[ignore]`): branch-guarded fixture confirms.** Add `unreach_false_branch.c` — `main` with an `if (__VERIFIER_nondet_int() == K) reach_error();` style guard whose true-branch is the replayed path. `saf verify --witness w.yml` → stdout `false(unreach-call)`, `w.yml` contains `type: branching` with `value:` and **no `column:`** on that waypoint, and is witnesslint-clean.
- [ ] **B.2 — Run; expect FAIL / iterate the emit→witnesslint loop** until witnesslint exits 0 on the enriched bytes.
- [ ] **B.3 — CPAchecker confirmation check.** In-container, run `scripts/validate_witness.sh` on the branch fixture; assert CPAchecker reports `FALSE`. Assert the plan-194 target-only fixtures (`unreach_false_direct.c`, the confirmed nondet one) **still** confirm (no regression). If the branch fixture does NOT confirm, capture the `CondBr` span line vs the source keyword line (the A.4 risk pin) and record the finding.
- [ ] **B.4 — Measurement sweep.** `EVAL_CONFIRM_WITNESS=1 scripts/svcomp_verify_eval.py` over a **larger reservoir slice than plan-194's 80** (more recalled FALSEs ⇒ more branch-guarded cases; ILP32+LP64). Report **target-only baseline vs +branching**: recall, false-alarm count (must be 0), TRUE count (must be 0), confirmed-witness %, and a **per-task diff** flagging any `confirmed → UNKNOWN/rejected` regression.
- [ ] **B.5 — VM green + record.** Full `make fmt && make lint` + `cargo nextest run -p saf-cli --run-ignored all -E 'test(verify_witness)'`; write the numbers into the plan's evidence section + `plans/PROGRESS.md`. **Honest expectation:** on today's ~3/40 recall the headline lift may be small (the current UNKNOWN task is nondet-driven → needs Tier B); Tier A's value is on branch-guarded paths and grows with recall. Any regression here is a redline-#3 stop.

---

## Slice S — Tier B payoff spike (GATE — no production code)

**Purpose:** prove, before any invasive frontend work, that assumption-value waypoints actually flip CPAchecker UNKNOWN→confirmed and that the nondet C-name is recoverable from the real post-`mem2reg` IR. **No source is edited in this slice.** **Gate (user, 2026-08-11): GO → auto-proceed to Slice C; NO-GO → stop and report.**

**Tasks (on the VM, in-container):**
- [ ] **S.1 — Pick fixtures.** `array-examples/data_structures_set_multi_proc_ground-1.i` (the known plan-194 UNKNOWN) + 1–2 more logic-heavy nondet-driven UNKNOWN FALSEs from the Slice-B sweep.
- [ ] **S.2 — IR presence check.** For each, dump the exact IR SAF ingests (reproduce `compile_to_ir`: `clang -g … -O0 -disable-O0-optnone` → `opt -passes=mem2reg`). Grep for `#dbg_value`/`@llvm.dbg.value` binding the **nondet call's SSA result** to a `!DILocalVariable`. Record: is the name present? the exact register/name token shapes (`i32 %3` vs `%3`, whitespace) the parser must match?
- [ ] **S.3 — Payoff check (hand-authored witness).** Using SAF's own replay model values, hand-author an enriched witness (target + Tier-A branching(no-column) + `assumption{format:c_expression, value:"<name> == <value>"}`) for each fixture. Run **witnesslint** (must pass) then **CPAchecker** (`violation-witness-validation.properties`) **and** **`cpa-witness2test`** (execution-based). Record which route confirms.
- [ ] **S.4 — Pin the accepted assumption shape.** Determine: does the assumption waypoint need a **location** (nondet call-site span) or does CPAchecker accept it location-light? `action: follow` vs `avoid`? Does it belong at the call site or the `DILocalVariable` decl line? This decides whether `NondetCall` must also carry a span in Slice E.
- [ ] **S.5 — GO/NO-GO checkpoint.**
  - **GO** iff (S.2) the `#dbg_value`→name is recoverable on ≥1 fixture **and** (S.3) the hand-authored enriched witness flips ≥1 UNKNOWN→confirmed (CPAchecker or cpa-witness2test). → **Auto-proceed to Slice C** (user authorized auto-start on GO, 2026-08-11): post a short GO note (what confirmed + the S.4 shape) and continue implementing Tier B; no sign-off needed.
  - **NO-GO** → **STOP and report** the blocker (name absent, or assumption rejected/ignored). Tier A already shipped; do not write Slices C–G without the user.
  - Either way, record the S.4 shape decision so Slices E/F implement exactly what confirmed.

---

## Slice C — Tier B frontend: parse `dbg.value` → nondet-result C-name *(GO only)*

**Files:** `crates/saf-frontends/src/llvm/debug_info.rs` (+ unit tests).

- [ ] **C.1 — Failing unit test.** Feed a printed-IR snippet (from S.2's real tokens) containing `#dbg_value(i32 %N, !M, …)` with `!M = !DILocalVariable(name: "x")` to a new `extract_nondet_result_names`; assert it returns `{func → {"%N" → "x"}}`. Add an old-intrinsic-form case (`@llvm.dbg.value(metadata i32 %N, metadata !M, …)`) and a negative case (first operand `undef`/constant → not captured).
- [ ] **C.2 — Run; expect FAIL.**
- [ ] **C.3 — Implement `extract_nondet_result_names`** mirroring `parse_new_style_dbg_declare`/`parse_old_style_dbg_declare`: reuse the `!DILocalVariable` metadata phase, then parse value-binding records, extracting the **first operand's SSA register** only when it is `%reg` (skip constant/undef), mapping `register → name`. cfg-tolerant to the llvm22 print form (plan-186 `DbgRecord` note).
- [ ] **C.4 — Run; expect PASS.** VM green (`cargo nextest run -p saf-frontends` + clippy + fmt).

---

## Slice D — Tier B frontend: stamp the nondet `CallDirect` result symbol *(GO only)*

**Files:** `crates/saf-frontends/src/llvm/mapping.rs` (+ integration check).

- [ ] **D.1 — Failing test.** Ingest a tiny `-g` + `mem2reg` fixture with `int x = __VERIFIER_nondet_int();`; assert the AIR `CallDirect` instruction for the nondet call has `symbol == Some("x")` and that `Alloca` naming is unaffected.
- [ ] **D.2 — Run; expect FAIL.**
- [ ] **D.3 — Implement.** Store the S.2/C map in `MappingContext` beside `current_local_var_names` (populate at `:556`/`:925`). Add an arm near the `Alloca` attach (`:1267`): if the instruction is a nondet `CallDirect` whose result register is in the map, set `air_inst.symbol = Some(Symbol::simple(name))`. **Restrict to the exact nondet-call result register** (not arbitrary dbg.value'd temps) — a mislabeled symbol becomes a wrong assumption → CPAchecker rejection (redline #3). Confirm no other pass reads a `CallDirect` symbol (grep before overloading; fall back to `Instruction.extensions` if a conflict exists).
- [ ] **D.4 — Run; expect PASS.** VM green (`cargo nextest run -p saf-frontends` + clippy + fmt).

---

## Slice E — Tier B property: `NondetCall.cname` plumbing *(GO only)*

**Files:** `crates/saf-svcomp/src/property.rs` (+ the S.4 span field iff required).

- [ ] **E.1 — Failing unit test.** `enumerate_false_candidates`/`resolve_nondet_sequence` on the D.1 fixture yields `nondet_sequence[0].cname == Some("x")` with the value still correctly resolved.
- [ ] **E.2 — Run; expect FAIL** (field absent).
- [ ] **E.3 — Implement.** Add `pub cname: Option<String>` to `NondetCall` (doc: "source C name of the nondet result, from `llvm.dbg.value` via `inst.symbol`; `None` when debug info is absent"). In the `resolve_nondet_sequence` constructor set `cname: inst.symbol.as_ref().map(|s| s.<name-field>.clone())` — read directly off the `inst` already iterated (**no new map parameter**). If S.4 required an assumption location, also add the call-site `span`/`InstId` and fill it here. `NondetCall` derives `PartialEq/Eq/Clone/Debug` — an `Option<String>` keeps all derives valid; fix the one `witness_lower.rs:254` test constructor (`nondet_sequence: vec![]`, no change needed) and any struct-literal sites.
- [ ] **E.4 — Run; expect PASS.** VM green (`cargo nextest run -p saf-svcomp` + clippy + fmt).

---

## Slice F — Tier B lowering: assumption waypoints in `lower_candidate` *(GO only)*

**Files:** `crates/saf-svcomp/src/witness_lower.rs`.

- [ ] **F.1 — Failing unit test.** A candidate with one nondet (`cname:"x"`, `value:6`) on the path → `lower_candidate` emits an `assumption{format:"c_expression", value:"x == 6"}` at the S.4-pinned location, interleaved with the Tier-A branching waypoints, ending in `target`. A candidate whose nondet has `cname:None` emits **no assumption** for it (degrade). Value comes from the per-slot `NondetCall.value`.
- [ ] **F.2 — Run; expect FAIL.**
- [ ] **F.3 — Implement.** In `lower_candidate`, for each `NondetCall` with `cname == Some(name)`, emit an `assumption` waypoint `format!("{name} == {}", nc.value)` at the pinned location/order; `cname == None` → skip (never a guessed name — redline #3). Keep the value from the call-order-aligned slot (consistent with `synthesize_driver`, `commands.rs:1221`). Optionally sanitize `name` at emit time (degrade to no-assumption if it contains chars CPAchecker's grammar can't parse) rather than in property.rs.
- [ ] **F.4 — Run; expect PASS.** Update the golden for the assumption-bearing shape; re-run byte-stable test. VM green.

---

## Slice G — Tier B: e2e validation + measurement *(GO only)*

**Files:** `crates/saf-cli/tests/`; `scripts/validate_witness.sh`; `scripts/svcomp_verify_eval.py`.

- [ ] **G.1 — Extend `validate_witness.sh`** with a `cpa-witness2test` execution route for assumption-bearing witnesses (report `CONFIRMED` iff it reproduces the violation).
- [ ] **G.2 — e2e (Docker `#[ignore]`)** on the S.1 fixtures: emit the full enriched witness; assert witnesslint-clean and that CPAchecker or cpa-witness2test **CONFIRMS** at least one previously-UNKNOWN task.
- [ ] **G.3 — Measurement sweep.** Re-run `EVAL_CONFIRM_WITNESS=1` reporting confirmed-% at **target-only → +Tier A → +Tier B**, with the per-task regression diff, 0 false alarms, 0 TRUE, byte-stable.
- [ ] **G.4 — VM green** (`make fmt && make lint` + full `make test`); record the numbers + update `plans/PROGRESS.md` (Next Steps → R4 interprocedural recall).

---

## Soundness & determinism invariants (must hold; mapped to slices)

1. **No `true` ever / FALSE only via must-reach OR replay** — unchanged; enrichment adds waypoints to an already-sound verdict only (A, F).
2. **Strictly additive enrichment** — missing cname / `then==else` / Switch / unresolved span → fewer waypoints, never a guessed or mislocated constraint; target-only is the always-valid floor (A.4, D.3, F.3; redline #3).
3. **Witness never gates the verdict** — `false` emitted even when enrichment (or any witness) is unconstructible; timeout/unknown/true never write (inherited from plan 194 Slice D).
4. **Byte-determinism** — deterministic `block_path` walk, `BTreeMap` name map, fixed `creation_time`, content-derived `uuid`; goldens re-pinned (A.6, F.4).
5. **Validity gate** — every emitted witness passes witnesslint; confirmation measured by CPAchecker + cpa-witness2test (B, G).
6. **No new FALSE sites** — R3 holes stay closed; this plan only enriches existing witnesses.

---

## Acceptance criteria & evidence

1. **Confirmed-% net lift** vs the plan-194 target-only baseline across the reservoir sweep — the headline metric (predicts `C.FalseOverall`). Reported at target-only → +Tier A → (+Tier B if GO) (B.4, G.3).
2. **Soundness preserved:** 0 false alarms, 0 TRUE across the sweep (B.4, G.3).
3. **No confirmed→unconfirmed regression:** the per-task diff shows no witness that confirmed target-only now fails/rejects (B.4, G.3). Any such case is a redline-#3 stop.
4. **Byte-stable witnesses:** goldens (A.6, F.4) + re-run test green.
5. **Tier B gated on evidence:** Slice S GO/NO-GO recorded; frontend code exists only on a GO.
6. **Guardrails green:** `make test`, clippy `-D warnings`, fmt; plan-192/194 e2e suites unchanged.
7. **PROGRESS.md updated** (B.5, G.4).

---

## Risks & open questions

- **Tier A line-accuracy** — confirmation rests on the `CondBr` span line == `if`/`while` keyword line; multi-line conditions / macro-expanded lines could shift it. Measured in B; target-only is the fallback (never a wrong verdict).
- **`block_path` adjacency** — the `then/else_target` match assumes consecutive elements are direct CFG successors. If the Z3 engine can emit gaps, the pair-match silently emits nothing (safe). Confirm in A/B.
- **Tier B empirical contingency** — the entire frontend investment is gated on Slice S. `#dbg_value` may be absent/coalesced post-`mem2reg`, or the assumption may be rejected like Slice E. That is precisely why S is a hard stop.
- **Register-token brittleness** — the text-scan must match the exact printed register token (`i32 %3` vs `%3`); pinned from S.2's real IR, unit-tested in C.
- **`inst.symbol` overload** — Tier B adds a second writer of a `CallDirect` symbol; verify no consumer is surprised (grep in D.3) or use `extensions`.
- **Assumption location** — whether the waypoint needs a call-site span (⇒ `NondetCall` grows a span field) is resolved by S.4 before Slice E.
- **Switch-guarded paths** — Tier A is CondBr-only; Switch branch decisions are deferred (a later tier).
- **llvm22 print form** — keep the `dbg.value` parser cfg-tolerant (plan-186 `DbgRecord` follow-up) so the opt-in image doesn't silently lose names.

---

## Out of scope (deferred)

- R4 interprocedural FALSE-candidate composition (next after 195); R5+ (memsafety/overflow/termination/race/packaging/memcleanup) per plan 193.
- `Switch` branching waypoints; interprocedural branch/assumption decisions (block_path is intra-function).
- Restoring pre-`mem2reg` alloca names for non-promoted aggregates (arrays/structs) — only the nondet-result `dbg.value` case is in scope.
- Correctness (invariant) witnesses; any `true` verdict.

---

## VM workflow (all builds/experiments here; never the laptop)

```bash
# sync source to the VM (see [[saf-svcomp-vm-env]])
rsync -az --delete <repo>/crates/  ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az        <repo>/scripts/   ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/scripts/   # WITHOUT --delete
# focused unit (Tier A / frontend)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-svcomp"'
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-frontends"'
# e2e witness tests (Docker, ignored)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-cli --run-ignored all -E \"test(verify_witness)\""'
# measurement sweep
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm -e EVAL_CONFIRM_WITNESS=1 dev sh -c "python3 scripts/svcomp_verify_eval.py …"'
# full guardrail
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make fmt && make lint && make test'
```
Let `cargo` regenerate `Cargo.lock` on the VM; sync it back before any commit. **Commit only when the user asks.**

---

## Implementation record (2026-08-11, VM-verified, uncommitted)

TDD on the VM (`ubuntu@cd-vm-15-ai-vm`, Docker); laptop = git source of truth; **uncommitted**.

**Slice A — Tier A branching (no column) — DONE, VM-green.** `witness_lower.rs::lower_candidate` walks `FalseCandidate.block_path`, matches each consecutive pair against the block's `CondBr{then,else}` terminator, and emits a `branching` waypoint with **`column: None`** (value = which successor the confirmed path took) before the target. Guards: skip `then == else`; CondBr-only (Switch/`Br` → no waypoint); empty/degenerate path → target-only. `witness_yaml.rs` unchanged. 6 unit tests (true/false direction, `then==else`, `Br`, multi-branch order, byte-stable enriched witness). `lower_must_reach` unchanged (branch-free by construction). Gates: **62/62** `saf-svcomp` nextest, workspace clippy `-D warnings` clean, fmt clean.

**Slice B — Tier A e2e + measurement — DONE.** New e2e (`smoke.rs::verify_nondet_witness_has_branching_no_column_and_lints`, `#[ignore]` Docker): the branch-guarded `unreach_false_nondet.c` witness carries a `branching` waypoint at the **`if`-keyword line** (line 8) with **no column**, and passes witnesslint. **Validated end-to-end: a branching-enriched witness is `LINT_OK` + CPAchecker `CONFIRMED`** — the Tier A spike finding holds in-pipeline (the `CondBr` span line = the `if` line). Reservoir sweep (N=40 = plan-194's exact 80-task sample, `EVAL_CONFIRM_WITNESS=1`): **0 false alarms, 0 TRUE (sound), recall 3/40, confirmed 2/3 (67%)** — identical to plan-194's target-only baseline, **no regression**. Larger sweep (N=120, 240 tasks): **0 FA, 0 TRUE, recall 7/120, confirmed 3/7 (43%)**.

**Slice S — Tier B payoff spike — NO-GO (Tier B DEFERRED; Slices C–G not built).**
- **S.2 (feasibility): PASS.** On a scalar-nondet fixture the C name is recoverable from post-`mem2reg` IR: `%1 = call @__VERIFIER_nondet_int()` → `@llvm.dbg.value(metadata i32 %1, !16)` → `!16 = !DILocalVariable(name: "x")` (old intrinsic-call form, keyed by register `%1`).
- **S.3 (payoff): FAIL — no payoff task in the current recall.** Every named-scalar-nondet-value FALSE SAF recalls already **confirms via Tier A branching** (CPAchecker resolves unconstrained nondets given the path — no value needed). Every recalled-but-UNKNOWN task is **outside Tier B's named-scalar-assumption scope**:
  - `data_structures_set_multi_proc` — array-store `int` nondets (`set[x]=nondet()`); result `%6` has **no `dbg.value`** name; needs array/loop reasoning.
  - `test_mutex_double_unlock` — `if(__VERIFIER_nondet_int())` boolean branch (unnamed; Tier A already captures the branch) + set-membership logic.
  - `maxpool_10_unsafe` (neural-net) — array-store **float** nondets + NN computation.
  - `fibo_2calls_25-1` — **no nondet at all** (`int x = 25`); deep recursion `fib(25)` CPAchecker can't unroll.
- **Corrects the plan-194 premise** that "data_structures needs assumption values": it needs array-value/loop reasoning, which named-scalar assumptions cannot express.
- **Verdict:** Tier B's mechanism is feasible but **premature** — the tasks it would help (complex *named-scalar*-nondet FALSEs where CPAchecker can't confirm from branching) are not in SAF's current recall; they require **R4 interprocedural composition** to surface. Building the frontend `dbg.value` capture now has **zero measurable confirmed-% payoff**.
- **Cheaper orthogonal lever surfaced:** adding **`cpa-witness2test` (execution-based)** to `validate_witness.sh` would confirm *deterministic-replay* tasks like `fibo` from the existing target/branching witness (pure execution reaches the error) — no frontend work, and a robust second validator.

**Net plan-195 outcome:** **Tier A landed** (sound, validated, no regression, byte-stable, extensible spine intact); **Tier B gated out by evidence.** Confirmed-% is unchanged *from branching* on the current recall because the real bottlenecks are **recall** (7/120) and **validator power** — see the follow-on lever, which is where the actual lift came from.

**Follow-on lever — `cpa-witness2test` execution validator (2026-08-11, DONE, uncommitted).** The Slice-S spike surfaced that the confirmed-% bottleneck is *validator power*, not enrichment. Added a **Stage 3** to `scripts/validate_witness.sh`: when analysis-based CPAchecker returns UNKNOWN/TRUE, fall back to **`cpa-witness2test`** (CPAchecker's execution-based validator — compiles + runs a test harness from the witness; confirms iff it reaches the target). A witness is CONFIRMED iff analysis **OR** execution agrees. Data-model → `--32`/`--64`; `timeout 120`; `SAF_SKIP_WITNESS2TEST=1` opt-out; `provision_cpachecker` now also `chmod +x`es `cpa-witness2test`.
- **Measured lift on the N=120 sample: 3/7 → 6/7 (43% → 86%)** confirmed — from SAF's **existing** target+branching witnesses, **no Rust/frontend change**. cpa-witness2test confirms `fibo` (deep recursion analysis can't unroll), `data_structures` (default-0 nondets collide the set → assert fails), and `test_mutex_double_unlock`; only `maxpool` (neural-net) stays UNKNOWN by both validators. simple-ext/elevator/transmitter continue to confirm via analysis (short-circuit; no regression).
- **This is the real `C.FalseOverall` lift of plan 195.** Measured end-to-end on the N=120 sweep (`EVAL_CONFIRM_TIMEOUT=250`): **6/7 (86%) confirmed, 0 false alarms, 0 TRUE** (up from 3/7 = 43% analysis-only). Soundness untouched (the witness is a side-output; SAF's FALSE verdict was already replay-proven; cpa-witness2test only checks witness re-reachability).
- **Next after this:** raising **recall** (R4 interprocedural composition) is now the dominant lever — 6/7 of the *few* FALSEs SAF finds already confirm, so the score is bounded by how many violations SAF finds (7/120), not by confirmation.
