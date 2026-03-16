# Plan 140: Code Quality Audit — Code Smells & Design Fixes

**Epic:** maintenance
**Status:** approved
**Created:** 2026-02-20
**Source:** Parallel 6-agent codebase analysis (saf-core, saf-frontends, saf-analysis, saf-cli, saf-python, cross-crate architecture)

## Summary

Comprehensive code smell and design analysis identified 77 issues across the SAF codebase:
- 1 critical, 19 major, 57 minor
- Key themes: primitive obsession (stringly-typed configs), duplicated logic (pipeline/helpers), missing abstraction layers (no unified orchestration), dead code, and hot-path performance (linear lookups)

## Prioritized Phases

Issues are grouped into independent phases that can be tackled in any order. Each phase is self-contained and testable.

---

### Phase A: Hot-Path Performance (Critical)

**Goal:** Eliminate O(n) linear scans in `AirModule`/`AirFunction` lookups.

#### A1. Index maps for `AirModule::function()` and `AirFunction` block lookup
- **File:** `crates/saf-core/src/air.rs:939-941, 788-794`
- `AirModule::function(id)` does linear scan of `self.functions` — called 29 times across 14 files, including fixpoint inner loops
- `AirFunction::entry()` and manual `.iter().find(|b| b.id == block_id)` repeated ~30 times
- **Fix:** Add `BTreeMap<FunctionId, usize>` index to `AirModule`, `BTreeMap<BlockId, usize>` index to `AirFunction`. Populate on construction. Change `function()` and block lookups to use index.
- **Risk:** Requires all `AirModule`/`AirFunction` construction sites to populate the index. Serde deserialization needs `#[serde(skip)]` + post-deserialize rebuild.
- **Test:** Existing tests should pass unchanged. Add specific benchmark for lookup performance.

---

### Phase B: Type System — Eliminate Stringly-Typed Code

**Goal:** Replace `String` fields with enums where the set of valid values is known and finite.

#### B1. `saf-core` Config enums
- **File:** `crates/saf-core/src/config.rs:12-29, 55-58`
- Replace `frontend: String` → `Frontend { Llvm, AirJson }`
- Replace `mode: String` → `AnalysisMode { Fast, Precise }`
- Replace `field_sensitivity: String` → `FieldSensitivity { None, StructFields }`
- Replace `external_side_effects: String` → `ExternalSideEffects { None, UnknownWrite, UnknownReadwrite }`
- Use `#[serde(rename_all = "snake_case")]` for JSON compatibility.
- **Touches:** config.rs + all consumers that string-match on these fields.

#### B2. `HeapAlloc::kind` enum
- **File:** `crates/saf-core/src/air.rs:392-395`
- Replace `kind: String` → `HeapAllocKind { Malloc, New, Calloc, Realloc, Other(String) }`

#### B3. `Span`/`SourceFile` FileId newtype
- **File:** `crates/saf-core/src/span.rs:13-14, 131`
- Replace raw `u128` with a `FileId` newtype via `define_id_type!` macro
- Consistent with all other ID types in the crate

#### B4. `TaintPropagation` use `TaintLocation` enum
- **File:** `crates/saf-core/src/spec/types.rs:306-308`
- Replace `from: String` and `to: Vec<String>` with `TaintLocation` enum
- Implement custom serde for `"param.N"` format compatibility
- Eliminate parsing logic in `from_param_index()`, `to_locations()`, `alias_param_index()`

#### B5. CLI argument enums
- **File:** `crates/saf-cli/src/commands.rs:31-65`
- Replace `frontend`, `mode`, `query_type`, `format`, `target` with `#[derive(clap::ValueEnum)]` enums
- Also change `inputs: Vec<String>` → `inputs: Vec<PathBuf>`

#### B6. `BlockId::new(0)` sentinel elimination
- **File:** `crates/saf-frontends/src/llvm/mapping.rs` (6 locations)
- `find_block_index` should return `Option<usize>`
- Callers handle `None` explicitly (log warning or error) instead of sentinel

---

### Phase C: Shared Helpers — Eliminate Duplication in saf-python

**Goal:** Extract shared utility functions to eliminate ~200 lines of copy-pasted code.

#### C1. `serde_to_py_dict<T: Serialize>()` helper
- Extract from 11 files that repeat the `serde_json::to_string → json.loads → extract::<Py<PyDict>>` pattern
- Consider `pythonize` crate for direct Rust→Python without JSON intermediate
- **Files affected:** graphs.rs, pta.rs, dda.rs, absint.rs, mssa.rs, svfg.rs, fspta.rs, cspta.rs, cg_refinement.rs, query.rs, ifds.rs

#### C2. `build_cfgs()` helper
- Extract from 5 files: mssa.rs, svfg.rs, fspta.rs, dda.rs, checkers.rs
- `fn build_cfgs(module: &AirModule) -> BTreeMap<FunctionId, Cfg>`

#### C3. `alias_result_to_str()` helper
- Extract from 4 files: pta.rs, dda.rs, cspta.rs, combined.rs
- Return `&'static str` instead of allocating `String`

#### C4. Remove duplicate ID parsers
- `mssa.rs` and `fspta.rs` define their own `parse_inst_id`/`parse_func_id`/`parse_loc_id` — use `crate::id_parse` instead

#### C5. `extract_checker_names()` helper
- Duplicated in `project.rs` at lines 454-462 and 590-598

---

### Phase D: Shared Analysis Infrastructure — Eliminate Duplication in saf-analysis

**Goal:** Extract shared data structures for cross-analysis reuse.

#### D1. `ModuleIndex` struct
- **Files affected:** dda/solver.rs, fspta/solver.rs, ifds/solver.rs, ifds/ide_solver.rs
- Pre-compute `inst_to_func`, `value_to_inst`, `inst_to_block` maps once
- Pass to all analysis phases that need them

#### D2. Shared `collect_return_values()`
- Currently duplicated in pta/extract.rs (2x) and cg_refinement.rs
- Extract to a module-level utility

#### D3. `PointsToQuery` trait
- Unify `PtaResult`, `CsPtaResult`, and DDA result behind a shared trait
- `fn points_to(&self, ptr: ValueId) -> Vec<LocId>`
- `fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult`
- Deduplicate `is_unique` implementation

---

### Phase E: Architecture — Unified Pipeline & Config

**Goal:** Create a shared analysis orchestration layer.

#### E1. Unified `AnalysisPipeline` builder in saf-analysis
- Replaces duplicated pipeline in saf-python/project.rs, saf-wasm/lib.rs, saf-bench
- Takes `AirModule` + `PipelineConfig`, returns result bundle
- Standardizes: CG refinement, PTA config, ValueFlow mode
- Entry-point crates become thin wrappers for serialization/FFI

#### E2. Resolve dual `PtaConfig` / dead `Config`
- **Option A (recommended):** Thread `saf_core::config::Config` into analysis, map fields to analysis-specific configs
- **Option B:** Remove analysis-related fields from `saf_core::config::Config`, acknowledge each pass owns its config
- Either way, eliminate the current state where `Config.pta` is silently ignored

#### E3. Consolidate abstract interpretation API
- Replace 10 `solve_*_with_*` entry points with builder/context pattern
- Single `solve_abstract_interp_with_context(module, config, &FixpointContext { ... })`
- Keep old functions as deprecated shims during transition

#### E4. Tiered `saf-analysis` public API
- Keep key types at crate root: `PtaResult`, `PtaConfig`, `AliasResult`, `PtaContext`, etc.
- Move constraint types behind `saf_analysis::pta::constraints::*`
- Move internal functions like `extract_constraints_reachable` to `pub(crate)`

---

### Phase F: Correctness Fixes

**Goal:** Fix silent data loss and error handling issues.

#### F1. Non-string arrays/structs should NOT become `ZeroInit`
- **File:** `crates/saf-frontends/src/llvm/mapping.rs:1490-1508`
- `convert_constant_value` maps all non-null, non-string arrays and structs to `Constant::ZeroInit`
- **Fix:** Return `None` for arrays/structs that can't be decomposed. `ZeroInit` only for actual `zeroinitializer`.

#### F2. `LlvmError` to `FrontendError` conversion
- **File:** `crates/saf-frontends/src/llvm/error.rs:63-67`
- Map `LlvmError::FileRead` to `FrontendError::Io`, not `FrontendError::Parse`

#### F3. Double `extract_global_initializers` call
- **File:** `crates/saf-analysis/src/pta/context.rs:84-89`
- `analyze_with_specs` calls `extract_constraints` (which includes `extract_global_initializers`) then calls `extract_global_initializers` again
- **Fix:** Remove the redundant call

#### F4. Replace `UnsafeCell` with safe interior mutability
- **File:** `crates/saf-analysis/src/pta/solver.rs:591-593`
- Use `Cell<SolverStats>` or `RefCell<SolverStats>` instead of `UnsafeCell`
- Or refactor `find_rep` to take `&mut self`

---

### Phase G: Dead Code Cleanup

**Goal:** Remove unused code that clutters the API surface and confuses maintainers.

| Item | Location |
|------|----------|
| `CoreError` enum (never used) | core/error.rs |
| `AnalysisError` enum (never used in production) | analysis/error.rs |
| `deterministic` module (empty stub, publicly exported) | core/deterministic.rs |
| Legacy `Solver` wrapper (entirely `#[allow(dead_code)]`) | analysis/pta/solver.rs:1506-1528 |
| `LlvmAdapter::is_intrinsic_call`/`get_intrinsic_name` (never called) | frontends/llvm/adapter.rs:46-49 |
| Unused `LlvmError` variants (`Llvm`, `Unsupported`, `Invalid`) | frontends/llvm/error.rs:20-31 |
| `is_pointer` fields in JSON schema (removed from core) | frontends/air_json_schema.rs:166, 269 |
| Unused saf-cli deps (saf-frontends, saf-analysis, serde_json, tracing) | cli/Cargo.toml |
| 15 `#[allow(unused_imports)]` masking dead API in pta/mod.rs | analysis/pta/mod.rs |

---

### Phase H: Minor Improvements (Low Priority)

These are real but low-impact issues. Fix opportunistically.

#### Code Organization
- Split `mapping.rs` (1641 lines) — extract IR string parsers to `ir_parser.rs`, constants to `constants.rs`
- Split `specs()` god function in CLI into `specs_list`, `specs_validate`, `specs_lookup`
- Group `Project` pymethods by domain (checkers, PTA, Z3, etc.) into separate files
- Move imports inside `specs()` function to module level

#### Consistency
- `PatternError` should use `thiserror` like all other error types
- `saf-wasm` should use edition 2024 + workspace metadata inheritance
- `saf-bench` should use `thiserror` for library trait, `anyhow` only in binary
- Direct `blake3` usage in frontends should use `saf-core::id` abstraction
- Document `FxHashMap`/`FxHashSet` usage (31 occurrences) with `// DETERMINISM:` comments

#### Minor Duplication
- Merge logic in spec types — consider derive macro or helper
- `GenericSolver::new` should delegate to `new_with_template`
- `TypeHierarchyEntry` construction repeated 3x in cha_extract.rs
- `HEAP_ALLOC_FUNCTIONS` linear scan could be `phf::Map` or `LazyLock<HashMap>`
- `Nullness` to string conversion duplicated in spec.rs
- `_pub` wrapper in cha_extract.rs should be `pub(crate)` visibility
- `Result` wrapping on 5 infallible functions with `#[allow(unnecessary_wraps)]`

#### API Polish
- `Span::new()` — use `Position`/`ByteRange` sub-structs to prevent parameter mixup
- `AirBundle::schema_version` — add validation on deserialization
- `supported_features()` — define `Feature` enum or `const` keys
- `FunctionSpec::param()` linear search → `BTreeMap<u32, ParamSpec>`
- `LocationFactory` — consider BLAKE3-derived IDs instead of sequential counter
- `PtaConfig` — group into `PtaCoreConfig`, `Z3IndexConfig`, `PathSensitiveConfig`
- `ConstraintSet` — consider `Vec + sort + dedup` instead of `BTreeSet`

#### saf-python Specific
- Group 6 `Arc<Mutex<usize>>` DDA diagnostics into single struct
- Replace env-var debug flags (14 instances) with `tracing` instrumentation
- `checker_diagnostics()` wastefully re-runs all checkers — cache result
- Stale "LLVM not available" note in schema.rs
- `ResourceTable` recreated per call; accept optional parameter

---

## Recommended Execution Order

1. **Phase G** (dead code) — Quick wins, zero risk, reduces noise
2. **Phase F** (correctness) — Fixes real bugs (F1 `ZeroInit`, F3 double extraction)
3. **Phase A** (hot-path perf) — Critical for analysis performance
4. **Phase C** (python helpers) — High ROI, ~200 lines of duplication removed
5. **Phase B** (type safety) — Prevents future bugs from stringly-typed configs
6. **Phase D** (analysis infra) — Reduces cross-solver duplication
7. **Phase E** (architecture) — Largest effort, biggest long-term payoff
8. **Phase H** (minor) — Opportunistic cleanup

## Notes
- Each phase is independently testable — no phase depends on another
- Phase E (architecture) is the largest and most impactful but also highest-risk
- Phase G + F can be done in a single session (< 1 hour)
- Phase C + D can be done in a single session (1-2 hours)
- Phase B spans multiple crates and should be done carefully with `make test` validation
