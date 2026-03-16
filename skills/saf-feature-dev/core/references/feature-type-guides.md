# Feature Type Guides

Per-feature-type checklists: what to explore, where to hook in, what to watch out
for, and how to validate. There are four feature types in SAF.

---

## 1. Frontend

A frontend converts an external program representation into SAF's canonical AIR.

### What to explore first

- **Trait definition:** `crates/saf-frontends/src/api.rs` -- the `Frontend` trait
  - Required methods: `ingest`, `input_fingerprint_bytes`, `supported_features`, `frontend_id`
  - Default method: `ingest_multi` (caching-aware batch ingestion)
- **Reference implementation (full):** `crates/saf-frontends/src/llvm/` -- LLVM frontend
  - `mod.rs` -- `LlvmFrontend` struct + `Frontend` impl
  - `mapping.rs` -- LLVM IR to AIR instruction translation
  - `type_intern.rs` -- type interning for deterministic `TypeId` generation
  - `debug_info.rs` -- source location and debug metadata mapping
  - `intrinsics.rs` -- LLVM intrinsic classification
  - `cha_extract.rs` -- class hierarchy extraction from LLVM metadata
- **Reference implementation (simple):** `crates/saf-frontends/src/air_json.rs` + `air_json_schema.rs`
  - Simpler starting point; deserializes JSON directly into `AirBundle`
- **Error types:** `crates/saf-frontends/src/error.rs` -- `FrontendError` enum (thiserror-based)
- **AIR types:** `crates/saf-core/src/air.rs` -- `AirBundle`, `AirModule`, `AirFunction`, `Operation` enum

### Extension points

- Add a new frontend: create a module under `crates/saf-frontends/src/`, implement `Frontend` trait
- Register in `crates/saf-frontends/src/lib.rs` (pub module + re-export)
- Add a `CliFrontend` variant in `crates/saf-cli/src/commands.rs` for CLI access
- Add feature flags in `crates/saf-frontends/Cargo.toml` if the frontend has optional deps

### Watch out for

- **Never leak frontend-specific types into analysis.** Analysis operates only on AIR (`NFR-EXT-001`)
- **Instruction coverage:** Every `Operation` variant must be handled; missing variants cause silent analysis gaps
- **Deterministic IDs:** Use BLAKE3-based `make_id` from `saf_core::id` -- same input must produce same IDs
- **Fingerprinting:** `input_fingerprint_bytes` must be path-normalized and exclude debug info by default
- **Metadata preservation:** Debug info, source spans, and type information must round-trip through AIR

### Testing strategy

- Create input program -> ingest via frontend -> verify `AirBundle` structure
- Test every `Operation` variant is produced for the corresponding input construct
- Test fingerprint determinism: same input file from different paths -> same fingerprint
- Test error handling: invalid/corrupt inputs -> `FrontendError`, not panic
- Smoke test: `crates/saf-frontends/tests/smoke.rs`
- E2E tests: compile C to LLVM IR inside Docker, ingest, verify AIR

### Validation

- `make fmt && make lint` -- clippy clean
- `make test` -- all frontend tests pass
- AIR round-trip: ingest -> export as AIR-JSON -> re-ingest -> compare bundles
- No `HashMap`/`HashSet` in iteration-order-sensitive code

---

## 2. Core Analysis

Analysis passes that operate on AIR: graphs, pointer analysis, value flow, checkers.

### What to explore first

- **Graph builders:**
  - `crates/saf-analysis/src/cfg.rs` -- per-function CFG (`Cfg::build`)
  - `crates/saf-analysis/src/callgraph.rs` -- whole-program call graph
  - `crates/saf-analysis/src/defuse.rs` -- def-use graph
  - `crates/saf-analysis/src/valueflow/` -- value-flow graph (builder, node, edge, query, export)
  - `crates/saf-analysis/src/svfg/` -- sparse value-flow graph
- **Pointer analysis:**
  - `crates/saf-analysis/src/pta/` -- Andersen CI PTA (constraints, solver, export)
  - `crates/saf-analysis/src/cspta/` -- context-sensitive PTA (k-CFA)
  - `crates/saf-analysis/src/fspta/` -- flow-sensitive PTA
  - `crates/saf-analysis/src/dda/` -- demand-driven alias analysis
- **Pipeline:**
  - `crates/saf-analysis/src/pipeline.rs` -- `run_pipeline` orchestrates CG refinement + PTA + VFG
  - `crates/saf-analysis/src/pass.rs` -- `AnalysisPass` trait, `PassManager`, `AnalysisContext`
  - `crates/saf-analysis/src/passes/` -- concrete pass implementations (`PtaPass`, `DefUsePass`, `ValueFlowPass`)
- **Checkers:** `crates/saf-analysis/src/checkers/` -- bug-finding checkers (pathsens, resource, summary)
- **PropertyGraph export:** `crates/saf-analysis/src/export.rs` -- unified `PropertyGraph` format

### Extension points

- **New graph type:** Add a builder module, implement `to_pg()` method returning `PropertyGraph`
- **New analysis pass:** Implement `AnalysisPass` trait from `crates/saf-analysis/src/pass.rs`, register in `passes/mod.rs`
- **New PTA constraint kind:** Add to `crates/saf-analysis/src/pta/constraint.rs`, update extraction
- **New checker:** Add module under `crates/saf-analysis/src/checkers/`, register in `checkers/mod.rs`
- **MSSA/SVFG extensions:** `crates/saf-analysis/src/mssa/` and `crates/saf-analysis/src/svfg/`

### Watch out for

- **PTA triple-update rule:** When adding constraint extraction logic, update ALL THREE functions in `crates/saf-analysis/src/pta/extract.rs`:
  1. `extract_constraints()` -- whole-program
  2. `extract_constraints_reachable()` -- reachability-filtered
  3. `extract_intraprocedural_constraints()` -- single-function
  Missing one causes silent failures in CG refinement or CS-PTA
- **Determinism:** Use `BTreeMap`/`BTreeSet` for all iteration-order-sensitive collections. `IndexMap` is permitted only in PTA hot paths where insertion order matters for performance
- **PropertyGraph format:** All graph exports share the same schema (`schema_version`, `graph_type`, `metadata`, `nodes[]`, `edges[]`). Follow the existing `to_pg()` pattern
- **SVFG instruction-level vs node-level mismatch:** Checker specs express instruction-level properties but SVFG operates on SSA value nodes. Known failure modes: `BTreeSet<SvfgNodeId>` deduplicates same-SSA call sites; `ResourceRole` is flat namespace; solver's `target != source` guard blocks zero-length flows
- **Field sensitivity config:** Respect `FieldSensitivity` settings -- struct fields vs array index vs flat

### Testing strategy

- **Unit tests:** Per-module tests for graph construction, constraint extraction, solver convergence
- **E2E tests:** `crates/saf-analysis/tests/*_e2e.rs` using `load_ll_fixture("<name>.ll")`
- **Test fixtures:** C source in `tests/programs/c/`, compiled IR in `tests/fixtures/llvm/e2e/`
- **Compile fixture inside Docker:** `clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll`
- **Benchmarks (before AND after changes):**
  - PTABen: `cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o results.json`
  - Juliet: `make test-juliet` or `make test-juliet CWE=CWE476`
- **Prefer specific assertions** over count assertions (e.g., `assert!(constraints.addr.iter().any(...))` not `assert_eq!(len, 2)`)

### Validation

- `make fmt && make lint` -- clippy clean
- `make test` -- all analysis tests pass
- PTABen results: no regressions in exact/unsound counts
- Juliet results: no regressions in precision/recall/F1 for affected CWEs
- No `HashMap`/`HashSet` outside documented hot paths

---

## 3. Python SDK

PyO3 bindings exposing SAF's analysis to Python scripts and AI agents.

### What to explore first

- **Binding root:** `crates/saf-python/src/lib.rs` -- module registration, top-level `#[pyfunction]`s
- **Per-domain bindings:**
  - `pta.rs` -- points-to analysis
  - `graphs.rs` -- `PyGraphStore` with PropertyGraph export, DOT, HTML
  - `checkers.rs` -- bug checker bindings
  - `query.rs` -- query API
  - `svfg.rs` -- SVFG bindings
  - `ifds.rs` / `ide.rs` -- IFDS/IDE solver bindings
  - `absint.rs` -- abstract interpretation
  - `selector.rs` -- value selectors (function_param, function_return, etc.)
  - `project.rs` -- high-level project API
- **Error mapping:** `crates/saf-python/src/exceptions.rs` -- `SafError` hierarchy (`FrontendError`, `AnalysisError`, `QueryError`, `ConfigError`) with `.code` and `.details` attributes
- **Python tests:** `python/tests/` -- test files like `test_smoke.py`, `test_checkers.py`, `test_svfg.py`

### Extension points

- **New `#[pyfunction]`:** Add function, register in the `#[pymodule]` block in `lib.rs`
- **New `#[pyclass]`:** Add struct with `#[pymethods]`, register class in `lib.rs`
- **New submodule:** Create `.rs` file, add `mod` in `lib.rs`, expose via `m.add_function()` or `m.add_class()`
- **Expose a new graph type:** Add case to `PyGraphStore::build_pg()` in `graphs.rs`

### Watch out for

- **Crate-level clippy allows** already set in `lib.rs` for PyO3-inherent issues:
  - `unnecessary_wraps` -- `#[pyfunction]` returning `PyResult` even for infallible ops
  - `needless_pass_by_value` -- PyO3 requires owned types for Python conversion
  - `unused_self` -- `#[pymethods]` on unit structs
- **Error conversion:** SAF `CoreError`/`AnalysisError` must map to Python exceptions via the `SafError` hierarchy. Do not let Rust panics escape to Python
- **Probe before consuming:** Never write Python code against an assumed API shape. Run `print(type(x), x)` inside Docker first to verify actual return types
- **PropertyGraph is the shared format:** All graph exports (`callgraph`, `cfg`, `defuse`, `valueflow`) return the same `PropertyGraph` structure. PTA export uses a different format (`{"points_to": [...]}`)
- **Owned types:** PyO3 requires owned `String`, `Vec`, etc. -- not references -- for Python conversion

### Testing strategy

- Tests live in `python/tests/` with real analysis pipelines, not mocks
- Run inside Docker: `make test` runs both Rust and Python tests
- Test against compiled LLVM IR fixtures (same as Rust E2E tests)
- Test error cases: invalid inputs should raise typed `SafError` subclasses
- Verify `conftest.py` for shared fixtures

### Validation

- `make fmt && make lint` -- clippy clean (PyO3 allows are crate-level, no per-function action needed)
- `make test` -- Python tests in `python/tests/` all pass
- New bindings have docstrings and type annotations
- Error paths return structured exceptions, not raw strings

---

## 4. CLI

The `saf` command-line binary for running analyses, exporting results, and querying.

### What to explore first

- **Entry point:** `crates/saf-cli/src/main.rs` -- parses args via clap, dispatches to command handlers
- **Command definitions:** `crates/saf-cli/src/commands.rs` -- clap `Parser`/`Args`/`Subcommand` structs
  - Commands: `index`, `run`, `query`, `export`, `schema`, `specs`, `incremental`, `help`
  - CLI-local enums: `CliFrontend`, `CliAnalysisMode`, `CliPtaVariant`, `CliPtaSolver`, `CliPtsRepr`, `CliFieldSensitivity`
  - Each enum wraps a `saf-core` type with `ValueEnum` support, keeping clap out of core
- **Analysis driver:** `crates/saf-cli/src/driver.rs` -- `AnalysisDriver` orchestrates the full pipeline (frontend ingestion -> PTA -> VFG -> checkers -> export)
- **Help system:** `crates/saf-cli/src/help.rs` -- topic guides (`saf help run`, `saf help checkers`, `saf help pta`, etc.)
- **Output formats:** JSON, SARIF, PropertyGraph, DOT, HTML

### Extension points

- **New subcommand:** Add variant to `Commands` enum in `commands.rs`, add `Args` struct, add match arm in `main.rs`
- **New CLI flag on `run`:** Add field to `RunArgs` in `commands.rs`, wire into `driver.rs`
- **New help topic:** Add match arm in `help.rs` `print_help()`, implement `print_<topic>_guide()`
- **New output format:** Add to export logic in `driver.rs` or `commands.rs`
- **New frontend option:** Add variant to `CliFrontend` enum, implement `From<CliFrontend> for saf_core::config::Frontend`

### Watch out for

- **CLI enums are thin wrappers:** Each `Cli*` enum wraps a `saf_core` or `driver` type. Always implement `From` for conversion and `Display` via `to_possible_value()`
- **Clap arg naming:** Use kebab-case for long flags (`--field-sensitivity`, not `--field_sensitivity`). Clap converts `snake_case` struct fields automatically but explicit `#[arg(name = "...")]` may be needed for multi-word values
- **Output format consistency:** All structured output should use `PropertyGraph` format for graphs or SARIF for findings. Do not invent new JSON shapes
- **Help text:** Keep `print_overview()` in `help.rs` synchronized when adding/removing commands
- **Error handling:** Use `anyhow` (binary crate). Return `anyhow::Result` from command handlers

### Testing strategy

- Test with fixture inputs: compile C to IR, run `saf run`, verify output
- Test output format correctness: parse JSON/SARIF output programmatically
- Test help output: verify expected topics are listed
- Test error cases: invalid inputs, missing files, incompatible flag combinations
- Smoke test: `crates/saf-cli/tests/smoke.rs`
- CLI integration tests use the binary as a subprocess

### Validation

- `make fmt && make lint` -- clippy clean
- `make test` -- all CLI tests pass
- New flags appear in `saf help` and `saf run --help`
- Output matches documented formats (PropertyGraph schema, SARIF spec)
- No panics on bad input -- all errors reported as structured messages
