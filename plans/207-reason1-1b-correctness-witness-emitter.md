# Plan 207 — Reason-1 / 1b: correctness-witness emitter + offline confirmation

**Status:** in-progress (design approved 2026-09-06). **Slice 0 DONE + GREEN (2026-09-06)** — gate PASSED decisively.
**Branch:** `svcomp` (HEAD `e8a2cd76` at planning time — **re-verify all cited line numbers on current HEAD; they drift**)
**Track:** TRUE-side ("Reason 1") program, roadmap item **1b**. Sequence: `1a (spike, DONE) → 1b (this) → 2 (no-overflow-TRUE loop-free) → 3 (unreach-TRUE loop-free)`.
**Research notes (laptop-side, NOT in this repo):** `investigation/reason1-spike-1a-findings.md`, `investigation/reason1-true-side-roadmap.md`, `investigation/reason1-spike-1a-artifacts/`. Key facts are embedded below so this plan is self-contained.

---

## 0. One-line goal

Build SAF's **first correctness-witness pipeline**: recover post-mem2reg loop-variable C names, emit a YAML-2.0 `invariant_set` witness from SAF's converged interval abstract interpretation, self-validate it in-process, and confirm it **offline** with the real CPAchecker (`Verification result: TRUE`). Ship as a **library + dev-only CLI subcommand + VM script + tests** — **no `saf verify` verdict change** (that hardened live-TRUE gate is ranks 2/3). Property-generic so 2/3 plug in.

## 1. What 1a established (carry forward — do not rediscover)

- The real **CPAchecker 4.2.2** (bundled in-repo at `.svtools/CPAchecker-4.2.2-unix`, present on both VMs; Java 21 on cd-vm-15) **confirms** a hand-authored YAML-2.0 `invariant_set` witness carrying one pure-interval loop-head invariant → `Verification result: TRUE`. Reproduced 3×, including from-scratch witnesses. It rejects a wrong invariant (→ `UNKNOWN`), so the verdict depends on witness content.
- **Confirmation invocation** (authoritative):
  ```bash
  CPA=.svtools/CPAchecker-4.2.2-unix
  $CPA/bin/cpachecker \
    --config $CPA/config/correctness-witness-validation.properties \
    --witness W.yml --spec <c/properties/unreach-call.prp> --32|--64 \
    --timelimit 150s --output-path <tmp> prog.c
  # CONFIRMED  ⇔  "Verification result: TRUE"
  ```
  The correctness configs are present on the VM: `correctness-witness-validation.properties` (generic), `correctness-witness-validation--overflow.properties` (rank-2 hook), k-induction variants.
- **Finalized 2.0 format** (NOT the stale GitHub 0.1 draft): top-level `entry_type: invariant_set`; `content:` list of `{invariant:{type: loop_invariant, location:{file_name,line,column,function}, value:"<C expr>", format: c_expression}}`; `metadata.task.input_file_hashes` = SHA-256; `data_model` must match `--32/--64`.
- **Gotchas (measured):** (1) `location` must map to a CFA node = the **loop keyword** (`while`/`for`/`do`) line, 1-based, leftmost-non-ws column (ground truth used col 3 for `  while`). (2) **Never omit the column** — CPAchecker 4.2.2 crashes (`NoSuchElementException`). (3) Wrong line/column ⇒ **silent non-confirmation** (grep `Could not find node`). (4) **Function-call loop guards** (`while(__VERIFIER_nondet_uint())`) match no column → **abstain**. (5) Confirmation needs a k-inductive **and** property-sufficient invariant; SAF's converged interval is inductive by construction, sufficient only when safety follows from pure interval bounds. (6) Program-hash check on by default.
- **1b.0 empirical evidence (captured 2026-09-06 on cd-vm-15, real `saf verify` recipe):** for `const.c`, post-mem2reg IR contains
  ```
  !53 = !DILocalVariable(name: "s", scope: !50, file: !2, line: 19, ...)
  %.0 = phi i32 [ 0, %0 ], [ %.1, %14 ], !dbg !55            ; the loop-head phi for `s`
  tail call void @llvm.dbg.value(metadata i32 %.0, metadata !53, metadata !DIExpression()), !dbg !55
  ```
  The loop-head phi `%.0` **does** carry a `dbg.value` → `!53` = `DILocalVariable(name:"s")`. The name is recoverable; the phi's IR register (`%.0`) is exactly what the frontend's `extract_register_name` produces. clang-18 emits **old-style `@llvm.dbg.value(...)` intrinsic calls** here (not `#dbg_value` records); constant descriptions (`metadata i32 0`) carry no `%` operand and are skipped for free.

## 2. Verified SAF integration points (stale-mirror `32dfb50`; re-verify on `e8a2cd76`)

**Frontend / name recovery**
- `crates/saf-frontends/src/llvm/debug_info.rs` — `extract_local_variable_names(module_ir:&str) -> LocalVarNameMap` (`BTreeMap<func, BTreeMap<reg, cname>>`), a **pure text parser** over the **post-mem2reg** module IR (which already contains `dbg.value`). Parses `!DILocalVariable` (`parse_di_local_variable`) + `dbg.declare` only (`parse_dbg_declare` / `parse_old_style_dbg_declare` @~329 / `parse_new_style_dbg_declare` @~364). `extract_register_name` @~397.
- `crates/saf-frontends/src/llvm/mapping.rs` — calls `extract_local_variable_names` (~556); per-function map in `current_local_var_names` (~925). Symbol attachment is **Alloca-only** (~1267–1274) via `Symbol::simple`. `convert_phi_instruction` (~1672) attaches **no** symbol.
- `crates/saf-core/src/air.rs` — `Instruction.symbol: Option<Symbol>` already exists (serde `skip_serializing_if`), **no schema change needed**.
- `crates/saf-analysis/src/display.rs` — `DisplayResolver::try_resolve_value` (~593) already reads `inst.symbol.display_name` (else `%<hex>`); `resolve_span` (~866). So once a Phi carries a `Symbol`, `resolve()` names it automatically.
- `crates/saf-frontends/src/llvm/intrinsics.rs` — `llvm.dbg.*` classified `Skip` (~69). This is **correct** and unaffected: `dbg.value` is parsed from module **text**, never as an AIR instruction.

**Absint read-out**
- `crates/saf-analysis/src/absint/result.rs` — `invariants_at_block(BlockId) -> BTreeMap<ValueId, Interval>` (~124), `invariants_at_inst` (~131), `diagnostics() -> {converged: bool, ...}` (~139).
- `crates/saf-analysis/src/absint/interval.rs` — `Interval { lo:i128, hi:i128, bits:u8, bottom:bool }`; `.lo()`, `.hi()`, `.bits()`, `.is_top()`, `.is_bottom()`; signed semantics (TOP = `[signed_min(bits), signed_max(bits)]`).
- `crates/saf-analysis/src/absint/fixpoint.rs` — `solve_abstract_interp(module, &AbstractInterpConfig::default())` (~111); `detect_loop_headers(cfg) -> IdBitSet<BlockId>` (back-edge targets, ~1331); `converged` set false on iteration cap / any non-converging function (**global** flag).
- `crates/saf-analysis/src/absint/config.rs` — `AbstractInterpConfig::default` (`max_widening_iterations=100`, threshold widening on).

**Witness emitter (violation-only today)**
- `crates/saf-svcomp/src/witness_yaml.rs` — internal serde `Entry{entry_type, metadata, content}`, `Metadata{format_version:"2.0", uuid, creation_time:"2024-01-01T00:00:00Z", producer, task}`, `Producer{name:"SAF", version}`, `Task{input_files, input_file_hashes, specification, data_model, language}`. Reusable: `compute_file_hash` (SHA-256, ~211), `deterministic_uuid` (~361). Property-agnostic API type `SourceWaypoint`; `ViolationWitness::assemble` + `to_yaml_string` (deterministic).
- `crates/saf-svcomp/src/witness_lower.rs` — `span_to_location(module, span) -> Option<(String basename, u32 line, u32 col)>` (~26). Reusable for loop-head location.
- `crates/saf-cli/src/commands.rs` — `compile_to_ir` (~904): `clang -g -S -emit-llvm -O0 -Xclang -disable-O0-optnone <-m32|-m64> -include <stub>` then `opt -S -passes=mem2reg` (in-place; pre-mem2reg IR transient). `strategy_for(property) -> Option<StrategyFn>` (~991); `VerdictOutcome{verdict:String, witness:Option<ViolationWitness>}` (~960); `verify()` writes witness **only** when `verdict.starts_with("false")` (~832). `termination_strategy` (~1781, verdict-only bare `"true"`, no witness) is the **sound-TRUE template**.

**Validator plumbing**
- `scripts/validate_witness.sh` — violation-only 3 stages: witnesslint `--expectViolationWitness` → CPAchecker `violation-witness-validation.properties` → `cpa-witness2test`. Provisions `$SAF_CPACHECKER` lazily; runs in the dev image. Never fails the script.

## 3. Scope

**In scope (1b):** 1b.0 name recovery + measurement gate; `invariant_set` emitter; in-process self-validator; loop-head selection + soundness gates; dev-only CLI subcommand; offline correctness-confirmation script; end-to-end demo + measurement report.

**Out of scope (→ ranks 2/3):** any change to `strategy_for`/`verify`/the witness write-gate; emitting a `true` verdict for any competition property; k-induction / BV-Z3; relational domains; SMG. 1b emits **no verdict** ⇒ **zero wrong-TRUE risk by construction**; the soundness gates are built now so 2/3 inherit them, and are de-risked empirically by the offline confirmation rate.

## 4. Slice plan (hard gate between Slice 0 and Slice 1)

### Slice 0 — 1b.0: `dbg.value` name recovery + measurement gate  *(blocking)*
**Files:** `saf-frontends/src/llvm/debug_info.rs`, `saf-frontends/src/llvm/mapping.rs`.

TDD (write tests first):
1. `debug_info.rs::parse_dbg_value` returning `Option<(reg, meta_id)>`:
   - old-style: `... @llvm.dbg.value(metadata <ty> %reg, metadata !N, metadata !DIExpression()) ...` → `("%reg","!N")`;
   - new-style: `#dbg_value(<ty> %reg, !N, !DIExpression(), !M)` → `("%reg","!N")`;
   - constant/`undef`/`poison` first operand (no `%`) → `None` (skipped);
   - fixture from the real `const.c` line above; also `%.0`, `%.1`, `%p.addr`, `%7` register shapes.
2. Extend `extract_local_variable_names` Phase-2: after the `parse_dbg_declare` arm, add a `parse_dbg_value` arm feeding the same `result[func][reg]=cname` map. Test: a module with only `dbg.value` (post-mem2reg shape) yields the promoted-reg→name map; declare-only still works (regression).
3. `mapping.rs` symbol attachment: extend the Alloca-only block (~1267) to also attach a `Symbol` to **Phi** (and any promoted value whose register resolves in `current_local_var_names`). Prefer inkwell `get_name()` for the register if cleaner than re-`print_to_string`; keep behavior identical for Alloca. Test at the AIR level (or e2e fixture) that a promoted loop phi ends up with `symbol = Some("s")`.
4. **Measurement harness** (throwaway, mirror 1a Q2; keep under `investigation/` on the laptop or a `tmp-spike/`): over a sample of `tests/benchmarks/sv-benchmarks/c/loops/*` + `c/loop-invariants/*` (target a few hundred programs), for each detected loop-head, count induction ValueIds whose `DisplayResolver::resolve` yields a **C identifier** (not `%<hex>`). Report the rate.

**GATE:** proceed to Slice 1 only if a **solid majority (~≥70%)** of loop-head variables resolve to C names. Else STOP and reassess (pre-mem2reg `dbg.declare` capture: save the pre-`opt` IR in `compile_to_ir`, build alloca→name, carry onto promoted values). Record the measured rate in PROGRESS + this plan.

### Slice 1 — `invariant_set` emitter  `saf-svcomp`
**Files:** new `crates/saf-svcomp/src/correctness_witness.rs`; small refactor in `witness_yaml.rs` to share `Metadata`/`Producer`/`Task`/`compute_file_hash`/`deterministic_uuid`; `crates/saf-svcomp/src/lib.rs` exports.

TDD:
1. Serde model: `InvariantSetWitness` → serializes to exactly the finalized 2.0 shape in §1 (golden-string test vs a hand-checked expected YAML; assert byte-identical on re-serialize = determinism). UUID seed = `file_hash || specification || producer_version || ordered-invariant-content`.
2. Property-agnostic API type `SourceInvariant { file_name, line, column, function, value }` and `InvariantSetWitness::assemble(meta, Vec<SourceInvariant>) -> Result<Self>` + `to_yaml_string()`.
3. `interval_to_c_expr(name, iv) -> Option<String>`: `[lo,hi]` → `"lo <= x && x <= hi"`; drop type-trivial half (`lo==signed_min(bits)` / `hi==signed_max(bits)`); `is_top`/`is_bottom` → `None`; singleton `[v,v]` → `"x == v"`. Unit tests for each case incl. unsigned-vs-signed edge, `[0,0]`.

### Slice 2 — selection + soundness gates + dev command  `saf-svcomp` + `saf-cli`
**Files:** `correctness_witness.rs` (driver + self-validator), new `source_has_openmp` helper, `crates/saf-cli/src/commands.rs` (new dev subcommand only — **not** `strategy_for`/`verify`).

TDD:
1. `build_interval_invariant_witness(module, source_path, meta) -> Option<InvariantSetWitness>`:
   - run `solve_abstract_interp(default)`; **abstain (return None) unless `diagnostics().converged==true`**;
   - **module-level abstain if source text contains `#pragma omp`** (`source_has_openmp`);
   - for each `detect_loop_headers` block H: `invariants_at_block(H)`; for each `(vid, iv)`: name via `DisplayResolver::resolve`; keep only **named C-identifier + non-top + renderable** vars; conjoin into one `value` per header;
   - location = H `!dbg` line + **leftmost-non-ws column** of that source line (read source); function from H's containing function;
   - **abstain per-loop on function-call guards** (header branch-condition def chain includes `Call`/`CallIndirect`) and on any unnamed/anonymous var.
2. In-process **self-validator**: for each rendered `SourceInvariant`, re-check the expression is entailed by its source `Interval` and the variable is a real C name → `Confirmed`/`Unconfirmed`/`Refuted`; drop any non-`Confirmed` loop. (Honest framing: rendering-integrity + gate check, not an independent proof; CPAchecker is the oracle.) Unit tests incl. a deliberately mis-rendered bound → `Refuted` → dropped.
3. Dev CLI subcommand `saf svcomp emit-correctness-witness <prog.c> [--data-model ilp32|lp64] [-o w.yml]`: compile→ingest→`build_interval_invariant_witness`→write YAML (+ print self-validation summary). No verdict, no competition path.

### Slice 3 — offline confirmation + end-to-end milestone  `scripts/`
**Files:** new `scripts/validate_correctness_witness.sh` (sibling of `validate_witness.sh`).

1. Stage 1 witnesslint `--expectCorrectnessWitness --expectedWitnessVersion 2.0` (confirm exact flag against the vendored witnesslint); Stage 2 CPAchecker `--config $CPA/config/correctness-witness-validation.properties --witness W.yml --spec PRP --32|--64 --timelimit 150s --output-path <tmp> prog.c`; **CONFIRMED ⇔ "Verification result: TRUE"**; always emit explicit column; grep `Could not find node`. Prints `CONFIRMED`/`NOT_CONFIRMED`/`CPACHECKER_ABSENT`; never fails the script. No witness2test stage.
2. **End-to-end milestone (definition of done):** SAF's own absint → `emit-correctness-witness` → CPAchecker `TRUE` on `spike_counter.c` (`0<=i<=1000000`) and `const.c` (`s==0`), reproduced. Regression test asserts a well-formed, self-validated witness (CPAchecker step runs on the VM, not in `cargo test`).
3. **Measurement:** run the emitter + offline confirmation over the Slice-0 loops sample; report name-recovery %, emit %, and CPAchecker-confirmation % (feeds the ranks-2/3 ceiling estimate).

## 5. Soundness redlines (built now; 1b emits no verdict, so these protect ranks 2/3)

- **Converged gate:** never build an invariant unless `diagnostics().converged==true` (fail-closed).
- **OpenMP abstain:** module-level abstain if source text has `#pragma omp` (frontend drops it → AIR silently loses parallelism).
- **Bit-precise intervals only** (the `Interval` domain); never the unbounded-Int Z3 layer; clamp to machine type bounds; never lift acyclic-path unreachability to TRUE.
- **Function-call loop guards:** abstain (unmatchable column).
- **Post-solving, non-speculative read-out** (converged solution only, never widening iterates / CEGAR internals).
- **Self-validate then emit** (0 unconfirmed / 0 refuted per emitted loop).
- **Confirmable subset bounded** to pure-interval-provable safety → modest ceiling by design; don't chase relational programs unsoundly.

## 6. Testing / determinism / ops

- **TDD** (red→green→refactor) per slice; `cargo-nextest`; one smoke test per touched crate stays green.
- **Determinism:** fixed `creation_time`, content-derived UUID, `BTreeMap` ordering; golden YAML re-serialize test.
- **All builds/tests via `docker compose run --rm dev sh -c '...'` on the VM** (`saf-dev:llvm18`); CPAchecker native on the VM; **never the laptop**. Subagents must not call `make`.
- `make fmt && make lint` before any commit; keep pedantic clippy clean.
- **Re-verify every cited line number on HEAD `e8a2cd76`** before editing.

## 7. Risks

- **Name-recovery hit-rate** (the Slice-0 gate) is the one real unknown; `const.c` confirmed positive, but `dbg.value` is point-dependent / can be "optimized out". Broad sample measures it before the emitter is built.
- **Column heuristic** (leftmost-non-ws) can miss on do-while / multi-line / macro-expanded headers → non-confirmation (harmless: scores 0, no wrong-TRUE). Offline confirmation is the true gate.
- **witnesslint correctness flag** name may differ from the assumed `--expectCorrectnessWitness`; confirm against the vendored linter in Slice 3.
- **Confirmation rate** for interval invariants unknown for this codebase (plan-195 saw ~40–86% for violations); measured in Slice 3, not assumed.

## 8a. Slice 0 result (2026-09-06) — GATE PASSED

- `parse_dbg_value` (old- + new-style) + shared `extract_reg_and_meta` helper added to `debug_info.rs`; `extract_local_variable_names` now ingests `dbg.value`; `parse_dbg_declare` refactored onto the shared helper (−~64 LOC dup). mapping.rs symbol-attachment extended from Alloca-only to `Alloca | Phi`.
- Tests: 5 new unit tests (parse_dbg_value old/new/constant-skip/named + extract-from-dbg.value) — `debug_info` module 23/23 green; new e2e `dbg_value_naming_e2e` (mem2reg'd fixture `dbgvalue_promoted_phi`) green. Full regression: **saf-frontends + saf-analysis 1815/1815 pass**, clippy `-D warnings` clean, fmt clean.
- **Measurement gate:** over 138 ingested mem2reg'd `-g` programs from `c/loops` + `c/loop-invariants` + `c/loop-simple` + `c/loop-acceleration`, **loop-head phis named = 375/375 = 100.0%** (loop-header blocks with a named phi 253/253; programs all-named 120/120). Explanation: at `-O0 + mem2reg` clang emits `dbg.value` for every promoted local, so there are no anonymous loop-carried phis (the "optimized-out dbg.value" risk is `-O2+`, not the `saf verify` regime). Throwaway harness deleted post-measurement; the `dbg_value_naming_e2e` test is the permanent regression.

## 8. Acceptance criteria

- [x] Slice 0: `parse_dbg_value` + Phi symbol attachment landed, tested; loop-head name-recovery rate measured = **100%** (≫ ~70% gate). **PROCEED to Slice 1.**
- [x] Slice 1 (2026-09-06): `invariant_set` emitter (`saf-svcomp/src/correctness_witness.rs`) — `interval_to_c_expr` + `SourceInvariant`/`InvariantSetWitness::assemble`/`to_yaml_string`, reusing `witness_yaml`'s `build_metadata_seeded`/`LocationOut`. 9 new tests incl. golden layout + determinism; saf-svcomp 438/438; clippy/fmt clean.
- [ ] Slice 2: driver honors every §5 redline; dev subcommand produces a self-validated witness; `saf verify`/`strategy_for`/write-gate **unchanged** (diff-verified).
- [ ] Slice 3: `validate_correctness_witness.sh` returns **CONFIRMED** for the two 1a programs from SAF's own emitted witnesses; measurement report produced.
- [ ] FP=0 / wrong-TRUE=0 preserved (no verdict emitted by 1b).
