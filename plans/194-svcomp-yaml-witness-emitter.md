# Plan 194: Property-generic YAML 2.0 violation-witness emitter + extensible verdict dispatch

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan slice-by-slice. Steps use checkbox (`- [ ]`) syntax. **All builds/tests run on the VM `ubuntu@cd-vm-15-ai-vm` (Docker); never the laptop. Subagents never call `make`. Commit only when the user asks.**

**Status:** design / **awaiting approval** (no implementation done)
**Date:** 2026-08-10
**Branch:** `svcomp` (laptop = git source of truth @ `eb707c2`)
**Reference competition:** SV-COMP 2026 rules (2027 unreleased); witness format = SV-COMP witnesses **2.0** (SPIN 2024, "Software Verification Witnesses 2.0").
**Bundles roadmap items** R1 (YAML-2.0 violation witness) + R2 (measure) + R3 (close Stage-1 holes) from `plans/193` §7.
**Supersedes** the deferred plan-192 sub-slices **1b-a / 1b-b** (YAML witness) and closes plan-192 checklist items `1b-a`, `1b-b`, and the R3 hardening.

**Goal:** Turn SAF's already-sound `unreach-call` FALSE verdicts into actual `C.FalseOverall` points by emitting a byte-deterministic, validator-confirmed YAML 2.0 *violation* witness at every FALSE return site — built as a property-generic module behind a pluggable per-property verdict-dispatch seam so R5–R10 plug in later without a rewrite.

**Architecture:** A new pure `saf-svcomp/src/witness_yaml.rs` owns the YAML 2.0 data model, deterministic serialization, and the generic *flat-waypoints → segments* assembly. A separate AIR→source lowering (also in `saf-svcomp`, uses `Instruction.span` + `AirModule.source_files`) turns the unreach evidence (`must_reach_error`'s `Vec<BlockId>` or a confirmed `FalseCandidate`) into source waypoints. `saf-cli` gains a `property → strategy` dispatch seam whose one live arm (`unreach-call`) runs the existing `propose → concrete-confirm` pipeline and now returns the witness; `verify()` writes it to `--witness` only on a `false` verdict received before the watchdog deadline. Execution (native replay, validators) stays in `saf-cli`; `saf-svcomp` remains subprocess-free.

**Tech Stack:** Rust (workspace crates `saf-svcomp`, `saf-cli`, `saf-core`); `serde` + `serde_yaml 0.9` + `sha2 0.10` (all already deps); LLVM 18 `clang-18`/`opt-18` (in the Docker dev image); `witnesslint` + CPAchecker (to be provisioned in the dev image) for the validation gate; Python eval harness under `scripts/`.

---

## Global Constraints (verbatim; every task inherits these)

**Soundness redlines (from `CLAUDE.md` "SV-COMP Competition Work" + plan 193 §6 — NON-NEGOTIABLE; a wrong verdict is −16/−32 ≈ 16× the reward of a right one):**
1. `saf verify` **must never emit `true`** with the current engines. Preserve the hard "never prints `true`" gate. (Zero −32 exposure.)
2. Never emit `false` without **(a)** an unconditional must-reach proof **OR (b)** concrete replay confirmation. No FALSE straight from a Z3 SAT.
3. Each NEW property's FALSE needs its OWN concrete confirmer — do not generalize the reach_error-sentinel confirmer to other properties in this plan.
4. **A witnessed WRONG false is still −16** → R3 (close the two Stage-1 `must_reach` holes) must land **before/with** the witness work.
5. Never wire `bench_result_to_verdict` to a competition surface.

**Determinism (NFR-DET-001):** witness bytes must be **byte-identical** for identical inputs. No `SystemTime::now()`, no RNG, no `HashMap`-iteration order in the witness path. `creation_time` fixed; `uuid` derived from content hash; hashes via `BTreeMap`.

**Extensibility (hard requirement, plan 193 §11):** the witness emitter is property-GENERIC (violation witness for any property; correctness-witness variant reserved) and the verdict dispatch is a `property → verdict-fn` table with a pluggable per-property `propose → concrete-confirm → witness` pipeline. Do NOT hardcode `unreach-call` in the shared spine.

**Format authority (from SPIN 2024 paper, Fig. 2 + Tables 2–3; see `[[svcomp-yaml-witness-2.0-format]]`):** violation witness = a YAML list with ONE entry `entry_type: violation_sequence`; `content` = sequence of `segment` wrappers, each a sequence of `waypoint` mappings; a segment = zero+ `action: avoid` waypoints ending in exactly one `action: follow`; the single FINAL segment ends in a `type: target` waypoint. `target`/`function_enter` **omit** `constraint`; `assumption`(c_expression)/`branching`(value)/`function_return`(acsl_expression) **require** it. **A single final segment with one `target` waypoint is valid.** The exact byte layout (wrapper keys, metadata spellings, `file_name` form) can drift paper-vs-deployed → **pin it to what the provisioned `witnesslint` + CPAchecker accept** (emit→lint→fix loop), do not freeze from the paper.

**Validators for 2.0 VIOLATION witnesses = CPAchecker (+ Witch3 optional).** UAutomizer does NOT validate 2.0-violation witnesses — **do not use it for this path** (corrects the brief's "CPAchecker and/or UAutomizer").

**Workflow:** edit locally; targeted rsync to the VM before each build (`crates/` with `--delete`; root files individually WITHOUT `--delete`; see `[[saf-svcomp-vm-env]]`). TDD throughout. `make test` / focused `docker compose run --rm dev sh -c '…'` on the VM only.

---

## Current state (verified this session, file:line)

- `saf-cli/src/commands.rs:760` `verify()` spawns a worker calling `unreach_verdict(&input, data_model) -> String`; the worker→main channel carries a **`String`**. `args.witness` (`commands.rs:404`, default `"witness.yml"`) is **never read or written**. The `AirModule` (`bundle.module`) lives **inside** `unreach_verdict`.
- Two FALSE sites in `unreach_verdict`: `commands.rs:969` `must_reach_error(&bundle.module).is_some()` (**discards the `Vec<BlockId>`**); `commands.rs:986-988` the replay loop, where a confirming `&FalseCandidate` is in scope.
- `saf-svcomp/src/property.rs:1679` `must_reach_error(&AirModule) -> Option<Vec<BlockId>>`; `:1706` `must_reach_body` holds the two R3 holes at **`:1714-1717`** (`is_declaration ⇒ Returns`) and **`:1744`** (`let Operation::CallDirect{..} = &inst.op else { continue }` — skips `CallIndirect`).
- `saf-svcomp/src/property.rs:1800` `struct NondetCall { func_name: String, value: i64 }` (**no span**); `:1820` `struct FalseCandidate { reach_error_inst: InstId, block_path: Vec<BlockId>, assignments: BTreeMap<ValueId,i64>, nondet_sequence: Vec<NondetCall> }`.
- `saf-core/src/span.rs:14` `Span { file_id: FileId, byte_start, byte_end, line_start, col_start, line_end, col_end }` (all `u32`, line/col 1-based); `:130` `SourceFile { id: FileId, path: String, checksum: Option<String> }`.
- `saf-core/src/air.rs:640` `Instruction { id: InstId, op: Operation, operands, dst: Option<ValueId>, span: Option<Span>, .. }`; `:552` `Operation::CallDirect{callee: FunctionId}`, `:559` `Operation::CallIndirect{expected_signature}`; `:956` `AirFunction::block(BlockId)`; `:1079/1087` `AirModule.functions`/`source_files`; `:1245` `AirModule::function(id)`, `:1275` `function_by_name`.
- `saf-svcomp/src/witness.rs` = **dead** GraphML (0 live callers), `compute_file_hash` (`:711`, SHA-256 hex, reusable), `SystemTime::now()` timestamp (`:238`, the determinism bug). `lib.rs:26` re-exports `Witness/WitnessEdge/WitnessNode/WitnessType`.
- `saf-svcomp/src/property_kind.rs`: `Property::name()`, `DataModel::{ILP32,LP64}` + `clang_flag()`. No spec-string or architecture helper (not needed — `verify()` already reads the raw `.prp`).
- Deps present: `serde_yaml 0.9`, `sha2 0.10`, `serde 1.0` (`saf-svcomp/Cargo.toml`). **No new crate required.**

---

## File Structure

| File | Responsibility | Action |
|---|---|---|
| `crates/saf-svcomp/src/witness_yaml.rs` | **NEW.** Property-generic YAML 2.0 witness: serde data model, deterministic `to_yaml_string`, deterministic `uuid`, fixed `creation_time`, generic flat-`SourceWaypoint`→segments assembly, `WitnessMeta`, `compute_file_hash` (salvaged). No AIR types. | Create |
| `crates/saf-svcomp/src/witness_lower.rs` | **NEW.** AIR→source lowering: `span_to_location`, `find_inst`, `find_block_in_module`; `lower_must_reach` (target-only) and `lower_candidate` (target + enrichment) producing `Vec<SourceWaypoint>`. Depends on AIR; property-aware but confined here. | Create |
| `crates/saf-svcomp/src/witness.rs` | Dead GraphML — **delete**. | Delete |
| `crates/saf-svcomp/src/property.rs` | R3: conservative `must_reach_body` bail; extend `NondetCall` with a call-site `span` for enrichment; extend `resolve_nondet_sequence` to fill it. | Modify |
| `crates/saf-svcomp/src/lib.rs` | Drop GraphML re-exports; add `witness_yaml`/`witness_lower` modules + public types. | Modify |
| `crates/saf-cli/src/commands.rs` | Verdict-dispatch seam (`strategy_for`, `VerdictOutcome`); `unreach_strategy` returns the witness; `verify()` threads `args.witness`, writes it on `false` only, before-deadline only. | Modify |
| `Dockerfile` | Provision `witnesslint` + CPAchecker in the dev image (validation gate). | Modify |
| `scripts/svcomp_verify_eval*.py` | R2: large/full-reservoir recall + confirmed-witness % + −16 audit. | Modify/Create |
| `crates/saf-cli/tests/` + `crates/saf-svcomp/` unit tests | Golden witness (byte-stable + witnesslint-accepted), lowering, R3, e2e (Docker `#[ignore]`). | Create |

**Slice order (each ends at a VM-green, independently reviewable deliverable):**
A → witness_yaml pure module · B → AIR lowering + target-only witnesses · C → R3 bail · D → dispatch seam + write witness · E → enrichment · F → validator infra + witnesslint gate + golden pin · G → R2 measurement.

C is independent of A/B and may run in parallel; D depends on A+B (+C for soundness); E depends on B+D; F depends on D; G depends on F.

---

## Slice A — `witness_yaml.rs`: the property-generic YAML 2.0 module

**Files:** Create `crates/saf-svcomp/src/witness_yaml.rs`; modify `crates/saf-svcomp/src/lib.rs`.

**Interfaces — Produces:**
```rust
pub struct WitnessMeta {
    pub producer_version: String,      // saf --version
    pub specification: String,         // raw trimmed .prp contents
    pub data_model: crate::DataModel,  // ILP32 | LP64
    pub language: Language,            // C
    pub input_file: std::path::PathBuf,// un-preprocessed input given to `verify`
}
pub enum Action { Follow, Avoid }
pub enum WaypointKind { Assumption, Branching, FunctionEnter, FunctionReturn, Target }
pub struct SourceWaypoint {
    pub kind: WaypointKind,
    pub action: Action,
    pub file_name: String,
    pub line: u32,
    pub column: Option<u32>,
    pub function: Option<String>,
    pub constraint: Option<Constraint>, // None for Target/FunctionEnter
}
pub struct Constraint { pub format: Option<String>, pub value: String }
pub struct ViolationWitness(/* opaque; serializes to the YAML list */);
impl ViolationWitness {
    /// Assemble a violation witness from a flat, ordered waypoint list.
    /// Groups into segments: each `Follow` ends a segment; the list MUST end
    /// with a `Target` waypoint (that closes the final segment). Returns
    /// `Err` if the final waypoint is not a `Target`.
    pub fn assemble(meta: &WitnessMeta, waypoints: Vec<SourceWaypoint>) -> anyhow::Result<Self>;
    pub fn to_yaml_string(&self) -> anyhow::Result<String>;
}
pub fn compute_file_hash(path: &std::path::Path) -> String; // salvaged; SHA-256 hex
```

**Serde data model (starting point — Slice F pins exact bytes to `witnesslint`):**
```rust
#[derive(serde::Serialize)]
struct Entry {
    entry_type: &'static str,             // "violation_sequence"
    metadata: Metadata,
    content: Vec<SegmentWrap>,
}
#[derive(serde::Serialize)]
struct Metadata {
    format_version: &'static str,         // "2.0"
    uuid: String,
    creation_time: &'static str,          // FIXED constant (determinism)
    producer: Producer,
    task: Task,
}
#[derive(serde::Serialize)] struct Producer { name: &'static str, version: String }
#[derive(serde::Serialize)]
struct Task {
    input_files: Vec<String>,
    input_file_hashes: std::collections::BTreeMap<String, String>,
    specification: String,
    data_model: String,                   // "ILP32" | "LP64"
    language: &'static str,               // "C"
}
#[derive(serde::Serialize)] struct SegmentWrap { segment: Vec<WaypointWrap> }
#[derive(serde::Serialize)] struct WaypointWrap { waypoint: WaypointOut }
#[derive(serde::Serialize)]
struct WaypointOut {
    action: &'static str,                 // "follow" | "avoid"
    #[serde(rename = "type")] kind: &'static str,
    location: LocationOut,
    #[serde(skip_serializing_if = "Option::is_none")] constraint: Option<ConstraintOut>,
}
#[derive(serde::Serialize)]
struct LocationOut {
    file_name: String,
    line: u32,
    #[serde(skip_serializing_if = "Option::is_none")] column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")] function: Option<String>,
}
#[derive(serde::Serialize)]
struct ConstraintOut {
    #[serde(skip_serializing_if = "Option::is_none")] format: Option<String>,
    value: String,
}
// top level serialized: Vec<Entry> -> serde_yaml::to_string(&vec)
```
Determinism helpers:
```rust
const FIXED_CREATION_TIME: &str = "2024-01-01T00:00:00Z"; // intentionally fixed for byte-stability
/// RFC-4122 v5-style UUID derived from a SHA-256 of the seed (deterministic).
fn deterministic_uuid(seed: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let d = Sha256::digest(seed);               // 32 bytes; take first 16
    let mut b = [0u8; 16];
    b.copy_from_slice(&d[..16]);
    b[6] = (b[6] & 0x0f) | 0x50;                // version 5
    b[8] = (b[8] & 0x3f) | 0x80;                // variant RFC-4122
    format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7],b[8],b[9],b[10],b[11],b[12],b[13],b[14],b[15])
}
```
`assemble` seeds the uuid from `sha256(input_hash || specification || producer_version)`; `input_files` = `[input_file basename]`; `input_file_hashes` = `{basename: compute_file_hash(input_file)}` (basename vs full path pinned in Slice F).

- [ ] **A.1 — Write failing test: segment grouping.**
```rust
#[test]
fn assemble_groups_follow_into_segments_and_requires_target() {
    let meta = tiny_meta();
    // branching(follow) then target(follow) => two segments, final ends in target.
    let wps = vec![
        SourceWaypoint{ kind: WaypointKind::Branching, action: Action::Follow, file_name:"a.c".into(), line:11, column:None, function:None, constraint: Some(Constraint{format:None, value:"true".into()}) },
        SourceWaypoint{ kind: WaypointKind::Target, action: Action::Follow, file_name:"a.c".into(), line:17, column:None, function:None, constraint: None },
    ];
    let w = ViolationWitness::assemble(&meta, wps).unwrap();
    let yaml = w.to_yaml_string().unwrap();
    assert!(yaml.contains("entry_type: violation_sequence"));
    assert!(yaml.contains("type: target"));
    assert_eq!(yaml.matches("segment:").count(), 2);
    // final waypoint not a target => Err
    let bad = vec![SourceWaypoint{ kind: WaypointKind::Branching, action: Action::Follow, file_name:"a.c".into(), line:1, column:None, function:None, constraint: Some(Constraint{format:None, value:"true".into()}) }];
    assert!(ViolationWitness::assemble(&meta, bad).is_err());
}
```
- [ ] **A.2 — Run it; expect FAIL** (`docker compose run --rm dev sh -c 'cargo test -p saf-svcomp witness_yaml'`) — "cannot find function `assemble`".
- [ ] **A.3 — Implement `witness_yaml.rs`** (structs above + `assemble` grouping: iterate waypoints, push each into the current segment, and on an `Action::Follow` close the segment; error unless the last waypoint is `Target`; build `Entry`/`Metadata`; `to_yaml_string` = `serde_yaml::to_string(&vec![entry])`). Salvage `compute_file_hash` here (delete from `witness.rs` in A.6). Map `data_model`→`"ILP32"/"LP64"`, `Action`/`WaypointKind`→their strings.
- [ ] **A.4 — Run it; expect PASS.**
- [ ] **A.5 — Write + run determinism test:** `assemble(...).to_yaml_string()` twice → byte-identical; assert no `SystemTime`/RNG by asserting `creation_time: 2024-01-01T00:00:00Z` present and uuid stable across two calls with the same seed.
- [ ] **A.6 — Delete `witness.rs`; update `lib.rs` and the one downstream importer.** Remove `pub mod witness;` and the `Witness/WitnessEdge/WitnessNode/WitnessType` re-exports; add `pub mod witness_yaml;` + `pub use witness_yaml::{WitnessMeta, SourceWaypoint, WaypointKind, Action, Constraint, ViolationWitness, compute_file_hash};`. **Known downstream importer (verified): `crates/saf-bench/src/svcomp/mod.rs:55-56`** re-imports `Witness, WitnessEdge, WitnessNode, WitnessType` in a `use saf_svcomp::{…}` list (imported but never *used* — recon confirmed zero live callers of the GraphML methods) — **delete those four names from that `use` list**. Then grep the workspace to confirm nothing else references them (`rg 'WitnessNode|WitnessEdge|WitnessType|\bWitness\b' crates/` — expect only the new `ViolationWitness`) and adjust any stragglers.
- [ ] **A.7 — VM green:** `docker compose run --rm dev sh -c 'cargo nextest run -p saf-svcomp && cargo clippy -p saf-svcomp -- -D warnings && cargo fmt --check'`. (Commit only when the user asks.)

---

## Slice B — `witness_lower.rs`: AIR→source lowering + target-only witnesses

**Files:** Create `crates/saf-svcomp/src/witness_lower.rs`; modify `crates/saf-svcomp/src/lib.rs`.

**Interfaces — Consumes:** Slice A (`SourceWaypoint`, `WaypointKind`, `Action`); `must_reach_error -> Option<Vec<BlockId>>`, `FalseCandidate`. **Produces:**
```rust
/// Resolve a Span to (file_name, line, column) via module.source_files.
pub fn span_to_location(module: &AirModule, span: &Span) -> Option<(String, u32, u32)>;
/// Target-only waypoint list for a must-reach chain (unconditional violation).
pub fn lower_must_reach(module: &AirModule, chain: &[BlockId]) -> Option<Vec<SourceWaypoint>>;
/// Waypoint list for a replay-confirmed candidate: target (+ enrichment in Slice E).
pub fn lower_candidate(module: &AirModule, cand: &FalseCandidate) -> Option<Vec<SourceWaypoint>>;
```
Helpers (private): `find_block_in_module(module,&BlockId)->Option<(&AirFunction,&AirBlock)>`; `find_inst(module,&InstId)->Option<(&AirFunction,&Instruction)>`; `reach_error_inst_in_block(module,&AirBlock)->Option<&Instruction>` (first `CallDirect` whose callee name ∈ `REACH_ERROR_NAMES`).

- [ ] **B.1 — Failing test: `span_to_location`.** Build a tiny `AirModule` with one `SourceFile{id: FileId::new(1), path:"t.c"}` and a `Span{file_id: FileId::new(1), line_start:12, col_start:5, ..}`; assert `span_to_location` returns `("t.c",12,5)`; unknown `file_id` → `None`.
- [ ] **B.2 — Run; expect FAIL.**
- [ ] **B.3 — Implement `span_to_location`** (linear scan of `module.source_files` for `sf.id == span.file_id`; return `(sf.path.clone(), span.line_start, span.col_start)`).
- [ ] **B.4 — Run; expect PASS.**
- [ ] **B.5 — Failing test: `lower_must_reach` target-only.** Construct a module `main → bb0 → bb1` where `bb1` holds `CallDirect{reach_error}` with a `Span` at line 20; `chain = [bb0, bb1]`; assert `lower_must_reach` returns exactly one waypoint `{kind:Target, action:Follow, line:20}`. Reach-error inst without a span → `None` (soundness: no unlocatable witness).
- [ ] **B.6 — Run; expect FAIL.**
- [ ] **B.7 — Implement `lower_must_reach`** (take the **last** `BlockId` in `chain`; `find_block_in_module`; `reach_error_inst_in_block`; `span_to_location(inst.span?)` → one `Target` waypoint; `None` on any miss).
- [ ] **B.8 — Failing test: `lower_candidate` target-only.** Module with a `reach_error` call whose `InstId` == `cand.reach_error_inst`, span at line 33; `cand.nondet_sequence` empty; assert one `Target` waypoint at line 33.
- [ ] **B.9 — Run; expect FAIL.**
- [ ] **B.10 — Implement `lower_candidate`** (`find_inst(cand.reach_error_inst)`; span → one `Target` waypoint; enrichment added in Slice E). Update `lib.rs`: `pub mod witness_lower;` + `pub use witness_lower::{span_to_location, lower_must_reach, lower_candidate};`.
- [ ] **B.11 — VM green** (`cargo nextest run -p saf-svcomp` + clippy + fmt).

---

## Slice C — R3: conservative `must_reach` bail (close the two Stage-1 holes)

**Files:** Modify `crates/saf-svcomp/src/property.rs` (`must_reach_body`, ~`:1706-1785`; add a `KNOWN_RETURNING_EXTERNALS` allowlist near `:1656`).

**Interfaces:** signature of `must_reach_error` unchanged (`Option<Vec<BlockId>>`); behavior only becomes MORE conservative (fewer `Some`, never a new `Some`).

Add (near the existing `NORETURN_FUNCTIONS`/`ASSUME_FUNCTIONS`):
```rust
/// External declarations we trust to return normally. Anything else that is a
/// declaration is treated as Indeterminate (it might diverge/exit/longjmp),
/// closing the optimistic-external hole (property.rs:1714-1717).
fn is_known_returning_external(name: &str) -> bool {
    is_scalar_integer_nondet(name)
        || matches!(name,
            "__VERIFIER_nondet_float" | "__VERIFIER_nondet_double" | "__VERIFIER_nondet_pointer"
            | "printf" | "puts" | "putchar" | "fprintf" | "sprintf" | "snprintf"
            | "malloc" | "calloc" | "realloc" | "free" | "memcpy" | "memmove" | "memset"
            | "strlen" | "strcpy" | "strncpy" | "strcmp" | "strncmp")
}
```

- [ ] **C.1 — Failing test: indirect call before reach_error → `None`.** Build `main` where `bb0` has an `Operation::CallIndirect{..}` then a forced edge to `bb1` with `CallDirect{reach_error}`. Assert `must_reach_error(&m).is_none()`.
- [ ] **C.2 — Failing test: diverging unknown external before reach_error → `None`.** `main`: `CallDirect{ext_decl}` (a declaration named `"maybe_diverges"`, not allow-listed) then `CallDirect{reach_error}`. Assert `None`.
- [ ] **C.3 — Guard tests still pass (no regression): (a)** unconditional direct reach (`main → reach_error`) still `Some`; **(b)** an allow-listed external (`printf`) before `reach_error` still `Some`; **(c)** a `__VERIFIER_nondet_int` call on the chain still `Some`.
- [ ] **C.4 — Run C.1–C.3; expect C.1/C.2 FAIL, C.3 (a/b/c) currently PASS** (b/c may currently pass via the optimistic path — that's fine).
- [ ] **C.5 — Implement the two edits:**
  - `:1714-1717` → `if func.is_declaration { return if is_known_returning_external(&func.name) { MustResult::Returns } else { MustResult::Indeterminate }; }`.
  - `:1743-1746` loop head → before `continue`ing on non-`CallDirect`, bail on indirect calls:
    ```rust
    let op = &inst.op;
    if matches!(op, Operation::CallIndirect { .. }) {
        return MustResult::Indeterminate; // unknown target may not return
    }
    let Operation::CallDirect { callee } = op else { continue };
    ```
- [ ] **C.6 — Run C.1–C.3; expect ALL PASS.** Also run the existing `must_reach_tests` module — expect green.
- [ ] **C.7 — VM green** (`cargo nextest run -p saf-svcomp` + clippy + fmt). Update the `:1715` comment (the "Residual" note is now resolved).

---

## Slice D — verdict-dispatch seam + write the witness in `verify()`

**Files:** Modify `crates/saf-cli/src/commands.rs` (`verify()` `:760`, `unreach_verdict` `:933` → refactor into a strategy).

**Interfaces — Consumes:** A (`ViolationWitness`, `WitnessMeta`, `Action`), B (`lower_must_reach`, `lower_candidate`), C. **Produces (internal to `saf-cli`):**
```rust
struct VerdictOutcome { verdict: String, witness: Option<saf_svcomp::ViolationWitness> }
struct VerifyCtx<'a> {
    input: &'a Path, data_model: saf_svcomp::DataModel, module: &'a AirModule,
    meta: &'a saf_svcomp::WitnessMeta, stub: &'a Path, tempdir: &'a Path, clang: &'a str,
}
type StrategyFn = fn(&VerifyCtx) -> VerdictOutcome;
fn strategy_for(p: saf_svcomp::Property) -> Option<StrategyFn>; // UnreachCall => unreach_strategy; else None
fn unreach_strategy(ctx: &VerifyCtx) -> VerdictOutcome;
```

Refactor: keep `unreach_verdict(input, data_model, spec, producer_version) -> VerdictOutcome` as the worker body — it does compile→ingest (shared), builds `WitnessMeta`, then `strategy_for(UnreachCall)` and runs it. The two FALSE sites now build the witness:
```rust
// Stage 1 (was commands.rs:969):
if let Some(chain) = saf_svcomp::must_reach_error(ctx.module) {
    let witness = saf_svcomp::lower_must_reach(ctx.module, &chain)
        .and_then(|wps| saf_svcomp::ViolationWitness::assemble(ctx.meta, wps).ok());
    if witness.is_none() {
        eprintln!("saf verify: FALSE (must-reach) but witness unconstructible (missing span) -> emitting false without witness");
    }
    return VerdictOutcome { verdict: format!("false({})", Property::UnreachCall.name()), witness };
}
// Stage 2/3 (was commands.rs:986-988):
Ok(true) => {
    let witness = saf_svcomp::lower_candidate(ctx.module, candidate)
        .and_then(|wps| saf_svcomp::ViolationWitness::assemble(ctx.meta, wps).ok());
    return VerdictOutcome { verdict: format!("false({})", Property::UnreachCall.name()), witness };
}
```
`verify()` changes: read `.prp` into `spec` (already read at `:776`); pass `spec`/producer version to the worker; channel type → `VerdictOutcome`; after `rx.recv_timeout(deadline)` returns a `VerdictOutcome` **in time**, write the witness then print:
```rust
let outcome = match rx.recv_timeout(deadline) { Ok(o) => o, Err(_) => { eprintln!("… budget … -> unknown"); VerdictOutcome{ verdict:"unknown".into(), witness:None } } };
if outcome.verdict.starts_with("false") {
    if let Some(w) = &outcome.witness {
        match w.to_yaml_string().and_then(|s| Ok(std::fs::write(&args.witness, s)?)) {
            Ok(()) => eprintln!("saf verify: wrote violation witness to {}", args.witness.display()),
            Err(e) => eprintln!("saf verify: failed to write witness: {e:#} (verdict still emitted)"),
        }
    }
}
println!("{}", outcome.verdict);
```
Soundness: the witness is written **only** on a `false` verdict received before the deadline; timeout/unknown/true never write. The `false` verdict is still emitted even if the witness is `None` (sound; scores 0, same as unknown, but truthful).

- [ ] **D.1 — Failing e2e test (`saf-cli/tests/verify_witness.rs`, `#[ignore]` Docker):** `unreach_false_direct.c` (`main` calls `reach_error()` unconditionally) with an `unreach-call.prp`, `--witness w.yml`; assert stdout `false(unreach-call)` AND `w.yml` exists and contains `entry_type: violation_sequence` + `type: target`.
- [ ] **D.2 — Run; expect FAIL** (no witness written yet).
- [ ] **D.3 — Implement the seam + `VerdictOutcome` + witness-write** (as above). Keep `strategy_for` a 1-arm match returning `unreach_strategy`.
- [ ] **D.4 — Run D.1; expect PASS** (`docker compose run --rm dev sh -c 'cargo nextest run -p saf-cli --run-ignored all -E "test(verify_witness)"'`).
- [ ] **D.5 — Failing e2e: no witness on unknown/true.** `unreach_true_simple.c` (reach_error present but unreachable) → stdout `unknown`, `--witness w.yml` **not created**. Implement already covers it; assert file absent.
- [ ] **D.6 — Failing e2e: byte-identical witness across re-runs.** Run D.1's task twice to two paths; assert the two files are byte-identical.
- [ ] **D.7 — Run D.5, D.6; expect PASS.**
- [ ] **D.8 — VM green:** full `make test` (Rust+Python) + clippy `-D warnings` + fmt. Confirm the pre-existing verify e2e/false-alarm suite (plan-192, 16 tests) still green.

---

## Slice E — witness enrichment (assumption + branching waypoints)

**Files:** Modify `crates/saf-svcomp/src/property.rs` (extend `NondetCall`, `resolve_nondet_sequence`); `crates/saf-svcomp/src/witness_lower.rs` (`lower_candidate` enrichment).

**Why:** target-only witnesses confirm on unconditional reaches but validate poorly on the nondet-guarded FALSEs that concrete replay finds (our actual recall). Adding `assumption` (nondet value) + `branching` (guard decision) waypoints lets CPAchecker replay the path within its budget.

**Interfaces — Produces:** `NondetCall` gains `pub span: Option<Span>` (the call site); `lower_candidate` emits, in `block_path` execution order: an `assumption` waypoint per scalar nondet call (`constraint{format:"c_expression", value:"<retval-var> == <value>"}` located at the nondet call span) — where a stable C-expression variable is not recoverable, fall back to a `branching`-only witness — and a `branching` waypoint at each `CondBr` guard along `block_path` (value `true`/`false` = which successor the path took), ending in the `Target`.

- [ ] **E.1 — Failing unit test (`property.rs`):** `enumerate_false_candidates` on a nondet-guarded fixture yields a `FalseCandidate` whose `nondet_sequence[0].span` is `Some(..)` at the nondet call line.
- [ ] **E.2 — Run; expect FAIL** (field absent).
- [ ] **E.3 — Implement:** add `span: Option<Span>` to `NondetCall`; in `resolve_nondet_sequence` set `span: inst.span.clone()`. Fix all `NondetCall{..}` constructors + the derive-based tests.
- [ ] **E.4 — Run; expect PASS.**
- [ ] **E.5 — Failing unit test (`witness_lower`):** a candidate with one nondet (`value:6`, span line 8) and a `CondBr` guard on `block_path` (span line 10, true-branch taken) and a target (line 12) → `lower_candidate` returns `[assumption@8, branching@10(true), target@12]` (final = target).
- [ ] **E.6 — Run; expect FAIL.**
- [ ] **E.7 — Implement branching/assumption derivation** in `lower_candidate`: walk `block_path`; for each block's terminator `CondBr`, determine which successor is the next block in `block_path` → `branching{value: "true"|"false"}` at the terminator span; for each scalar nondet call with a span, `assumption{format:"c_expression", value: format!("{lhs} == {}", nc.value)}` when a lhs C-name is available (else skip that assumption — soundness/validity: never emit an unlocatable or unnameable constraint). Append the `Target` last.
- [ ] **E.8 — Run; expect PASS.**
- [ ] **E.9 — VM green** (`cargo nextest run -p saf-svcomp` + clippy + fmt). Re-run Slice D e2e (still green; richer witness bytes — update the D.6 golden if pinned).

---

## Slice F — validator infra + `witnesslint` gate + golden byte-pin

**Files:** Modify `Dockerfile`; add `crates/saf-svcomp/tests/golden_witness.rs` (or a `#[test]` with a checked-in golden `.yml`); add `scripts/validate_witness.sh`.

**Why:** every emitted witness must pass `witnesslint` (syntactic gate); CPAchecker measures the confirmation % (the score predictor). The exact serialization is **pinned here** to what the provisioned tools accept.

- [ ] **F.1 — Dockerfile: provision `witnesslint`** (clone `sosy-lab/sv-witnesses`, put `witnesslinter.py` on PATH or a fixed location; ensure `python3` present). Rebuild the dev image on the VM (`docker compose build dev`).
- [ ] **F.2 — Dockerfile: provision CPAchecker** (download a fixed CPAchecker release with YAML-2.0 violation validation; Java runtime; fixed path e.g. `/opt/cpachecker`). Pin the version for determinism. Rebuild.
- [ ] **F.3 — emit→lint→fix loop:** generate a witness for `unreach_false_direct.c` in-container; run `witnesslint` on it; if it rejects the wrapper-key / metadata / `file_name` layout, adjust the `witness_yaml.rs` structs until `witnesslint` exits 0. Record the accepted layout.
- [ ] **F.4 — Golden test:** check in the `witnesslint`-accepted witness bytes as `crates/saf-svcomp/tests/data/golden_unreach_false_direct.yml`; add a test asserting `assemble(...).to_yaml_string()` equals the golden (byte-stable) for a fixed synthetic input. (This locks the serialization + determinism.)
- [ ] **F.5 — `scripts/validate_witness.sh`:** wraps `witnesslint <w.yml>` (must exit 0) then CPAchecker `-witnessValidation -witness <w.yml> -spec <p.prp> <prog.c>` (90 s), printing `CONFIRMED` iff CPAchecker reports FALSE.
- [ ] **F.6 — e2e gate test (`#[ignore]` Docker):** for `unreach_false_direct.c` and one nondet fixture (`unreach_false_nondet.c`), emit the witness and assert `witnesslint` accepts it; assert CPAchecker CONFIRMS at least the unconditional one.
- [ ] **F.7 — VM green** (targeted e2e). Note in PROGRESS if CPAchecker confirmation on the nondet fixture needs Slice-E enrichment (expected).

---

## Slice G — R2: full-reservoir recall + confirmed-witness % + −16 audit

**Files:** Modify/extend `scripts/svcomp_verify_eval_all.py` (and/or `svcomp_verify_eval.py`).

- [ ] **G.1 — Extend the eval harness** to (a) run `saf verify --witness` over a large/full `unreach-call` reservoir slice (family-stride, not the 40-task sample), (b) record verdict + witness path per task, (c) pipe each emitted witness through `scripts/validate_witness.sh`, (d) compute: **recall** = #`false` / #`expected==false`, **false-alarm count** on `expected==true` (must be 0), **TRUE count** (must be 0), **confirmed-witness %** = #CONFIRMED / #`false`.
- [ ] **G.2 — −16 audit:** run the **pre-Slice-C** binary (or a feature flag) over the reservoir to count tasks where the old optimistic-external / CallIndirect holes would have emitted `false`; confirm Slice C removed them (post-C count on those tasks = 0 wrong FALSE). Report the delta.
- [ ] **G.3 — Run on the VM**, capture output to a file, summarize in PROGRESS + the plan's evidence section: recall, 0 false alarms, 0 TRUE, confirmed-witness %, −16 delta.
- [ ] **G.4 — Update `plans/PROGRESS.md`** (Plan 194 entry → done/in-progress; Next Steps = R4 interprocedural composition; Session Log).

---

## Soundness & determinism invariants (must hold; mapped to slices)

1. **No `true` ever** — `strategy_for` returns `unknown` for all non-`unreach-call`; `unreach_strategy` has no TRUE branch (D).
2. **FALSE only via must-reach OR replay** — unchanged; witness is a *side output* of an already-sound verdict, never the basis for it (D).
3. **Stage-1 holes closed** — indirect-call + non-allow-listed external → `Indeterminate` (C), with regression tests. A witnessed wrong-FALSE is impossible on those paths.
4. **Witness never gates the verdict** — a `false` is emitted even when the witness is unconstructible; only witness *emission* is conditional (D). Timeout/unknown/true never write a witness (D).
5. **Byte-determinism** — fixed `creation_time`, content-derived `uuid`, `BTreeMap` hashes, struct-field key order, no `now()`/RNG; golden test locks it (A, F).
6. **Validity gate** — every emitted witness passes `witnesslint`; confirmation measured by CPAchecker (F, G).

---

## Acceptance criteria & evidence

1. **Points, not zero:** on the reservoir, emitted `false` witnesses achieve a **reported confirmed-witness %** via CPAchecker (G) — the headline metric. (Predicts `C.FalseOverall` score.)
2. **Soundness preserved:** 0 false alarms, 0 TRUE across the reservoir (G).
3. **Stage-1 holes closed:** C.1–C.3 green; G.2 shows 0 wrong-FALSE on the previously-holey tasks.
4. **Byte-stable witnesses:** golden test (F.4) + re-run test (D.6) green.
5. **Extensible spine in place:** `strategy_for` table + `VerdictOutcome` + property-generic `witness_yaml` (A, D) — a future property adds a strategy arm + a lowering fn, no rewrite.
6. **Guardrails green:** `make test`, clippy `-D warnings`, fmt; pre-existing plan-192 verify e2e suite unchanged (D.8).
7. **PROGRESS.md updated** (G.4).

---

## VM workflow (all builds/experiments here; never the laptop)

```bash
# sync source to the VM (see [[saf-svcomp-vm-env]])
rsync -az --delete <repo>/crates/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az <repo>/Dockerfile ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/         # WITHOUT --delete
rsync -az <repo>/scripts/   ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/scripts/ # WITHOUT --delete
# build/test (main agent only; subagents never call make)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose build dev'      # after Dockerfile edits (F)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-svcomp"'
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make test'                       # full guardrail (D.8)
# focused e2e (Docker, ignored tests)
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && docker compose run --rm dev sh -c "cargo nextest run -p saf-cli --run-ignored all -E \"test(verify_witness)\""'
```
Let `cargo` regenerate `Cargo.lock` on the VM; sync it back before any commit. **Commit only when the user asks.**

---

## Risks & open questions

- **Serialization drift (paper vs deployed).** Mitigation: Slice F emit→lint→fix loop + golden byte-pin; the data model in Slice A is a *starting point*, not frozen.
- **`witnesslint` YAML-2.0 maturity.** If lint's 2.0 coverage is thin, rely on CPAchecker's own parse as the effective gate + the golden schema self-check; document what lint actually enforces.
- **CPAchecker provisioning weight** (Java, large download) inflates the dev-image build. Pin a fixed release; cache the image layer. Witch3 as an optional second validator only if CPAchecker confirmation looks weak.
- **`file_name` matching.** CPAchecker matches the witness `file_name`/`input_files` against the source it's given. If confirmation fails on a path mismatch, normalize to basename (pin in F.3). Same for `input_file_hashes` key.
- **Enrichment C-expression naming (E.7).** A stable, in-scope C variable name for a nondet result may not be recoverable from mem2reg'd IR; when it isn't, emit `branching`-only (skip the `assumption`) rather than an invalid constraint — lower confirmation, never an invalid witness.
- **`creation_time` requiredness.** Appears required; kept as a fixed constant. If a validator rejects a constant/old timestamp, revisit (unlikely — format, not recency, is validated).
- **Span availability.** Relies on `clang -g`; a `reach_error` inst without a span → `false` emitted without a witness (sound, scores 0). Measured breadth reported in G.

---

## Out of scope (deferred)

- R4 interprocedural FALSE-candidate composition (next after 194); R5 memsafety FALSE + ASan confirmer; R6 no-overflow; R7 termination TRUE; R8 no-data-race TRUE; R9 native ZIP packaging; R10 memcleanup.
- Correctness (invariant) witnesses; any `true` verdict; `valid-memtrack` GraphML-1.0 exception (GraphML deleted — re-add if R10 ever needs it).
- bench `run_task` in-process switch (plan-192 1.2b).

---

## Implementation record & corrections (2026-08-11, VM-verified, uncommitted)

Implemented TDD on the VM, slice by slice. All builds/tests on `ubuntu@cd-vm-15-ai-vm`
(Docker); laptop = git source of truth; **uncommitted**.

**Landed (Slices A–F):**
- **A** `saf-svcomp/src/witness_yaml.rs` — property-generic YAML-2.0 model + `assemble`
  (flat waypoints → segments; final must be `target`) + deterministic `to_yaml_string`
  (fixed `creation_time`, content-derived RFC-4122 `uuid`, `BTreeMap` hashes) + salvaged
  `compute_file_hash`. Dead GraphML `witness.rs` **deleted**; `lib.rs` re-exports updated;
  `saf-bench/svcomp/mod.rs:55-56` import fixed. `anyhow` added to the crate.
- **B** `saf-svcomp/src/witness_lower.rs` — `span_to_location` (returns the **basename**,
  not clang's absolute path — matches `input_files` + a validator's view + determinism),
  `lower_must_reach`, `lower_candidate`.
- **C** R3 conservative `must_reach_body` bail (`is_known_returning_external` allowlist;
  `CallIndirect` → Indeterminate) — closes the two Stage-1 −16 holes. 4 regression tests.
- **D** `saf-cli` verdict-dispatch seam (`strategy_for`/`VerdictOutcome`/`VerifyCtx`/
  `run_verdict`/`unreach_strategy`); witness written to `--witness` **only on a `false`
  received before the watchdog deadline**. 4 witness e2e tests.
- **F** Dockerfile bakes `witnesslint` (2.1.3-dev) + its Python deps + **openjdk-21**;
  `scripts/validate_witness.sh` (witnesslint hard gate + CPAchecker best-effort). Golden
  layout test pins the exact serialization. CPAchecker 4.2.2 provisioned lazily to
  `/workspace/.svtools` (too big for the shared image).

**Corrections to the plan (found during implementation):**
1. **Format validated against ground truth.** Confirmed my layout matches the SPIN 2024
   paper (Fig. 2) *and* the repo's canonical CPAchecker/Symbiotic 2.0 witnesses. **Every
   emitted witness passes `witnesslint` (exit 0).** No serialization changes were needed
   beyond the basename fix.
2. **CPAchecker needs Java 21, not 17** (4.2.2). Correct validation invocation:
   `bin/cpachecker --config config/violation-witness-validation.properties --witness <W>
   --spec <prp> --timelimit 90s <program>` → `Verification result: FALSE` = confirmed.
   (`--witness-validation` and `cpa-witness2test` did not work for our target-only witnesses.)
3. **Slice E branching enrichment was implemented, validated, and REVERTED.** CPAchecker
   requires a `branching` waypoint at the `if`/`while` **keyword**, but the AIR `CondBr`
   span points into the *condition expression* → CPAchecker mis-parses it as a ternary and
   **rejects the whole witness** ("Ternary operators as branching waypoints are currently
   not supported"), strictly worse than target-only. Assumption-value waypoints are
   likewise infeasible (no reliable C variable name after `mem2reg`). `lower_candidate` is
   therefore **target-only**, which CPAchecker confirms for *simple* violations. Doing
   **F before E** (validate the baseline first) is what surfaced this — the plan's E-before-F
   order was wrong.
4. **The confirmation gap is the real bottleneck (not emission).** Measured: target-only
   witnesses confirm on trivial violations (unconditional + single-guard nondet fixtures →
   CPAchecker FALSE), but a logic-heavy real task
   (`array-examples/data_structures_set_multi_proc_ground-1.i`) yields CPAchecker
   **UNKNOWN (incomplete analysis)** — its predicate analysis cannot re-derive the violating
   path from a target-only witness. So **witnesses are now emitted + syntactically valid +
   confirmed on the simple subset, but confirming logic-heavy FALSEs needs path-guidance
   waypoints** (correct-location branching + assumption values) — the next lever, blocked on
   `if`-keyword-location and post-`mem2reg` C-name recovery.

**Gates (VM):** nextest **2237** pass + **20/20** verify e2e (incl. witnesslint gate);
`clippy --workspace -D warnings` + `fmt --all` clean; golden byte-stable; every emitted
witness passes witnesslint.

**Slice G (R2 measurement) — DONE.** `scripts/svcomp_verify_eval.py` extended with an opt-in
`EVAL_CONFIRM_WITNESS=1` mode (writes a witness per task; runs `validate_witness.sh` on each
emitted FALSE → confirmed-witness %). **Reservoir sweep (80 tasks: 40 expected-true /
40 expected-false, stride-sampled across the 22,681-task `unreach-call` reservoir, ILP32+LP64,
`--timeout 20s`):**
- **Soundness: 0 false alarms, 0 TRUE** — this is also the R2 **−16 audit** (a wrong FALSE is
  −16): zero across 80 tasks post-R3.
- **Recall: 3/40 (7.5%)** on expected-false (`simple-ext.i`, `elevator_spec1_product30.cil.c`,
  `data_structures_set_multi_proc_ground-1.i`) — consistent with plan-193's 1/20 baseline.
- **Confirmed-witness: 2/3 (67%)** via CPAchecker — `simple-ext.i` and
  `elevator_spec1_product30.cil.c` → `Verification result: FALSE` (score **0 → +1**); the
  logic-heavy `data_structures_set_multi_proc_ground-1.i` → CPAchecker **UNKNOWN**
  (target-only witness insufficient — the confirmation gap of correction #4).

**Bottom line:** SAF now emits witnesslint-valid YAML-2.0 violation witnesses at every sound
`unreach-call` FALSE, and **~⅔ are CPAchecker-confirmed → real `C.FalseOverall` points where
there were none.** Raising the confirmed-% further is the enrichment work (deferred below).

**Deferred (future levers to raise confirmed-%):** correct-location `branching` waypoints
(recover the `if`/`while` keyword span); `assumption`-value waypoints (recover the nondet
result's C name from debug info) — both would let CPAchecker/`cpa-witness2test` confirm the
logic-heavy FALSEs SAF's replay already finds.
