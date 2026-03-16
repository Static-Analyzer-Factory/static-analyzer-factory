# SAF Feature Development Workflow

This document defines the 8-phase workflow for adding features to SAF (Static Analyzer Factory). It is platform-agnostic: it describes *what* to do, not which tool to use. Any coding agent or human contributor can follow these phases to deliver well-tested, convention-compliant SAF features.

The workflow covers four feature types: **Frontend**, **Core Analysis**, **Python SDK**, and **CLI**. Checker/analyzer authoring is out of scope for this workflow.

---

## Phase 1: Context Loading

**Goal:** Understand where the project stands and classify the feature you are about to build.

**Entry criteria:** You have a feature request, bug report, or task description.

### Steps

1. **Read the progress file.** Open `plans/PROGRESS.md` and read it in full. This file tracks the current epic, all active/recent plans, next steps, and known blockers. Understanding the current state prevents duplicate work and surfaces dependencies.

2. **Read the relevant SRS sections.** Open `docs/static_analyzer_factory_srs.md` and locate the sections pertinent to your feature. The SRS defines all functional requirements (FR-*), non-functional requirements (NFR-*), and architectural constraints. If your feature touches pointer analysis, read the PTA sections. If it touches the CLI, read the CLI sections. Skim broadly if unsure.

3. **Classify the feature type.** Use the routing table below to determine which of the four feature types your work falls into. This classification drives which crates you will modify, which files you must explore, and which test strategy to use.

| Feature Type | Primary Crates | Key Files to Find | Typical Deliverables |
|---|---|---|---|
| **Frontend** | `saf-frontends`, `saf-core` | `crates/saf-frontends/src/api.rs` (the `Frontend` trait), `crates/saf-core/src/air.rs` (AIR types), existing frontend impls in `crates/saf-frontends/src/` | New `Frontend` trait impl, AIR mapping, smoke test |
| **Core Analysis** | `saf-analysis`, `saf-core` | `crates/saf-analysis/src/pta/extract.rs` (constraint extraction), `crates/saf-analysis/src/pipeline.rs`, `crates/saf-analysis/src/lib.rs`, graph builders in `crates/saf-analysis/src/` | New/modified analysis pass, e2e test with C fixture, `saf_log!` instrumentation |
| **Python SDK** | `saf-python`, plus the Rust crate being exposed | `crates/saf-python/src/lib.rs` (module registration), `crates/saf-python/src/project.rs`, `python/tests/` | New `#[pyfunction]`/`#[pymethods]`, Python test, docstrings |
| **CLI** | `saf-cli` | `crates/saf-cli/src/main.rs` (command dispatch), `crates/saf-cli/src/commands.rs` (arg structs + handlers), `crates/saf-cli/src/driver.rs` (pipeline orchestration) | New/modified command, integration test, help topic |

4. **Announce your classification.** State the feature type, the primary crates you expect to modify, and a one-sentence summary. Confirm with the contributor before proceeding. Example: "This is a **Core Analysis** feature: adding field-sensitive constraint extraction to the PTA solver. Primary crate: `saf-analysis`. Secondary: `saf-core` for new AIR types."

**Exit criteria:** You know the feature type, have read `PROGRESS.md` and relevant SRS sections, and the contributor has confirmed the classification.

---

## Phase 2: Codebase Exploration

**Goal:** Build a detailed mental model of the code you will touch before writing anything.

**Entry criteria:** Feature type classified and confirmed.

### Steps

1. **Launch parallel exploration tasks.** Depending on the feature type, run 2-3 searches simultaneously. Do not write code yet. The goal is to understand the existing structure.

2. **Follow the exploration prompts for your feature type:**

#### Frontend exploration

- **Trace the `Frontend` trait implementation.** Search for `impl Frontend` across `crates/saf-frontends/src/`. Read the LLVM frontend implementation (the primary reference impl) end-to-end. Understand the `ingest()` method's flow: how it reads input files, produces an `AirBundle`, and handles errors.
- **Map AIR types.** Read `crates/saf-core/src/air.rs` to understand the canonical IR: `AirBundle`, `AirModule`, `AirFunction`, `AirBlock`, `Operation` enum. Find every variant of `Operation` -- your frontend must produce valid AIR using these types.
- **Check supported features.** Read `supported_features()` implementations to understand the feature flags convention. Find `input_fingerprint_bytes()` to understand deterministic hashing.

#### Core Analysis exploration

- **Trace the PTA constraint extraction pipeline.** Read `crates/saf-analysis/src/pta/extract.rs` and locate all three extraction entry points:
  - `extract_constraints()` -- whole-program extraction
  - `extract_constraints_reachable()` -- reachable-function extraction for CG refinement
  - `extract_intraprocedural_constraints()` -- per-function extraction for incremental updates
  
  This is critical: if you add a new constraint generation step, you **must** update all three entry points. Missing one causes silent failures in callgraph refinement or context-sensitive PTA.
- **Map the graph builder pipeline.** Read `crates/saf-analysis/src/pipeline.rs` for the analysis pipeline stages (CFG, callgraph, PTA, def-use, MSSA, SVFG, value-flow). Read `crates/saf-analysis/src/database/mod.rs` for `ProgramDatabase` which owns all precomputed graphs and provides lazy construction. Read `crates/saf-analysis/src/pass.rs` for the `AnalysisPass` trait and pass manager.
- **Understand the PTA solver.** Read `crates/saf-analysis/src/pta/mod.rs` for the Andersen solver entry points. If your feature involves flow-sensitivity, also read `crates/saf-analysis/src/fspta/`.

#### Python SDK exploration

- **Trace PyO3 bindings.** Read `crates/saf-python/src/lib.rs` to see the module registration pattern. Each submodule (e.g., `pta.rs`, `graphs.rs`, `checkers.rs`) follows the same pattern: a `#[pyclass]` struct wrapping a Rust type, `#[pymethods]` exposing methods, and `#[pyfunction]` for module-level functions.
- **Find the Rust API you are exposing.** Read the Rust crate's public API that your Python binding will wrap. Understand the types, error conditions, and ownership model.
- **Map the Python test suite.** List files in `python/tests/` and read the test for the most similar existing binding. Tests use `pytest` and typically call `saf.Project.open(...)` to load a fixture, then exercise the new API.

#### CLI exploration

- **Trace command dispatch.** Read the chain: `crates/saf-cli/src/main.rs` (parses CLI args, matches on `Commands` enum) -> `crates/saf-cli/src/commands.rs` (defines `Cli`, `Commands` enum with arg structs, handler functions like `run()`, `export()`, `query()`) -> `crates/saf-cli/src/driver.rs` (`AnalysisDriver` orchestrates the full pipeline).
- **Map integration tests.** Search for test files in `crates/saf-cli/tests/`. Read the smoke test to understand how CLI tests invoke the binary.
- **Check help topics.** Read `crates/saf-cli/src/help.rs` to understand the built-in help system. If your feature adds a user-facing concept, you may need a new help topic.

3. **Read all key files and present findings.** After exploration, summarize what you found:
   - Which functions/types you will modify
   - Which patterns you must follow (existing code is the best style guide)
   - Any surprises or complexities discovered
   - Related tests that already exist

**Exit criteria:** You can describe the data flow path your feature touches, have identified specific functions to modify, and have listed the patterns you must follow.

---

## Phase 3: Clarifying Questions

**Goal:** Surface SAF-specific concerns that could derail the implementation if missed.

**Entry criteria:** Codebase exploration complete.

**This phase is CRITICAL. Do not skip it.** SAF has hard invariants that generic coding practices do not cover. Failing to ask these questions leads to subtle bugs, lint failures, or CI rejections.

### SAF-Specific Concern Checklist

Work through each concern and determine whether it applies to your feature:

| Concern | Question to Ask | Why It Matters |
|---|---|---|
| **Determinism** | Does this feature introduce iteration over hash-based collections? Will it produce different output on different runs? | SAF requires byte-identical outputs for identical inputs (NFR-DET-001). Use `BTreeMap`/`BTreeSet`, never `HashMap`/`HashSet`. |
| **Cross-crate impact** | Does this change affect types in `saf-core` that other crates depend on? Does it change `AirBundle`, `Config`, or the `Frontend` trait? | Changes to `saf-core` types ripple to every downstream crate. Plan for cascading updates. |
| **Python exposure** | Should this feature be accessible from the Python SDK? If so, what is the Python-friendly API shape? | Python bindings require owned types (not references), `PyResult` return types, and docstrings. Plan the Python API early. |
| **Benchmark impact** | Could this change affect PTA precision, checker results, or analysis performance? | If yes, you must run PTABen and/or Juliet benchmarks before and after to quantify impact. |
| **AIR compatibility** | Does this feature require new AIR types or operations? | All analysis operates on AIR, never frontend-specific types (NFR-EXT-001). New operations must be added to the `Operation` enum in `saf-core`. |
| **ID system** | Does this feature create new identifiable objects (locations, values, nodes)? | All IDs are `u128`, BLAKE3-derived, serialized as `0x` + 32 hex chars (FR-AIR-002). Use `saf_core::id::make_id()`. |
| **Constraint extraction** | Does this feature add a new PTA constraint type or generation step? | You must update all three extraction entry points: `extract_constraints()`, `extract_constraints_reachable()`, and `extract_intraprocedural_constraints()`. |
| **No SVF reuse** | Is this feature inspired by SVF? Are you tempted to port SVF code? | SAF requires independent implementations only (REQ-IP-001). You may study SVF's algorithms but must write original code. |

### Steps

1. **Evaluate each concern** against your feature. Mark each as "applies" or "does not apply."
2. **Formulate concrete questions.** For each concern that applies, write a specific question. Example: "This feature adds a new `PhiNode` constraint type. I need to add extraction logic to all three extraction entry points. Should the phi constraint also participate in HVN preprocessing?"
3. **Present all questions at once** and wait for answers before proceeding. Do not start implementation with unresolved questions about SAF invariants.

**Exit criteria:** All SAF-specific concerns have been evaluated, questions have been asked and answered, and you have a clear understanding of the constraints your implementation must satisfy.

---

## Phase 4: Plan & Design

**Goal:** Choose an implementation approach, document it, and get confirmation before coding.

**Entry criteria:** All clarifying questions answered.

### Steps

1. **Explore 2-3 implementation approaches.** For each approach, evaluate it against SAF's hard constraints:
   - **AIR-only rule:** Analysis must operate on AIR types, never on frontend-specific types.
   - **No SVF code reuse:** Independent implementation required.
   - **Determinism:** Must use `BTreeMap`/`BTreeSet` and produce deterministic outputs.
   - **BLAKE3 IDs:** New objects need deterministic `u128` IDs via `saf_core::id::make_id()`.
   - **Error handling:** `thiserror` for library crates, `anyhow` only in `saf-cli`. No `.unwrap()` in library code.

2. **Present approaches with tradeoffs.** For each approach, describe:
   - What changes in which files
   - Performance implications
   - Complexity and maintenance cost
   - How it interacts with existing features (PTA, checkers, pipeline)
   - Your recommendation and why

3. **Write the plan file.** Once an approach is confirmed, create a plan file:
   - Find the highest-numbered plan in `plans/` (e.g., if the latest is `plans/183-saf-feature-dev-skill.md`, your plan is `plans/184-<topic>.md`)
   - Structure the plan with: objective, approach, task list (numbered), affected files, test strategy, risks
   - Keep it concise -- 50-150 lines is typical

4. **Update PROGRESS.md.** Add your new plan to the Plans Index table with status `approved`. Update "Next Steps" if your feature is the current priority.

**Exit criteria:** A numbered plan file exists in `plans/`, `PROGRESS.md` is updated, and the contributor has confirmed the approach.

---

## Phase 5: Test First (E2E Preferred)

**Goal:** Write failing tests before writing implementation code. SAF strongly prefers end-to-end tests with real C programs.

**Entry criteria:** Plan confirmed and documented.

### Test Strategy by Feature Type

| Feature Type | Preferred Test Approach | Test Location |
|---|---|---|
| **Frontend** | C program compiled to `.ll` inside Docker, loaded via `load_ll_fixture()` | `crates/saf-frontends/tests/` or `crates/saf-analysis/tests/*_e2e.rs` |
| **Core Analysis** | C program compiled to `.ll` inside Docker, e2e test with `load_ll_fixture()` | `crates/saf-analysis/tests/*_e2e.rs` |
| **Python SDK** | Python test calling the new binding | `python/tests/test_<feature>.py` |
| **CLI** | Integration test invoking the CLI binary | `crates/saf-cli/tests/` |

### Test Preference Hierarchy

Prefer tests higher in this list. Drop to a lower level only when the higher level is impractical:

1. **E2E with C source** -- Write a C program in `tests/programs/c/<name>.c`, compile to LLVM IR inside Docker, write a Rust e2e test using `load_ll_fixture("<name>")`
2. **E2E with handwritten `.ll`** -- Write LLVM IR directly in `tests/fixtures/llvm/e2e/<name>.ll` for precise control over IR structure
3. **Integration test** -- Test the public API of a crate without going through C compilation
4. **Unit test** -- Test a single function in isolation (use only for pure algorithms)

### Steps

1. **Write the C test program** (if applicable). Create `tests/programs/c/<feature_name>.c` with a minimal program that exercises the behavior you are implementing. Include comments explaining what the test verifies.

2. **Compile to LLVM IR inside Docker.** All compilation must happen inside the Docker container because LLVM 18 is only available there:
   ```bash
   docker compose run --rm dev sh -c \
     'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll'
   ```

3. **Write the test file.** Follow existing patterns:
   - For Rust e2e tests: create or extend a file in `crates/saf-analysis/tests/`. Use `saf_test_utils::load_ll_fixture("<name>")` to load the compiled IR.
   - For Python tests: create `python/tests/test_<feature>.py`. Use `saf.Project.open(...)` to load a fixture.
   - For CLI tests: create a test in `crates/saf-cli/tests/`.

4. **Prefer specific assertions over count assertions.** Write assertions that check for specific expected values, not exact counts:
   ```rust
   // GOOD: survives when unrelated changes add legitimate constraints
   assert!(constraints.addr.iter().any(|a| a.ptr == expected_id));

   // FRAGILE: breaks when anything adds a new constraint
   assert_eq!(constraints.addr.len(), 2);
   ```

5. **Verify tests fail.** Run the test suite and confirm your new tests fail with a meaningful error (not a compilation error):
   ```bash
   make test 2>&1 | tee /tmp/test-output.txt
   ```
   Search the output for your test name to confirm failure. SAF uses `cargo-nextest`, so look for `Summary` lines, not `test result:` lines.

**Exit criteria:** Tests are written, they compile, and they fail with assertion errors (not build errors) that will pass once the feature is implemented.

---

## Phase 6: Implementation

**Goal:** Implement the feature following SAF conventions, with frequent validation.

**Entry criteria:** Failing tests exist.

### SAF Coding Conventions Checklist

Before writing code, internalize these rules:

- **Error handling:** Use `thiserror` for error types in library crates. Use `anyhow` only in `saf-cli`. Never use `.unwrap()` in library code -- return `Result` or use `.expect("reason")`.
- **Determinism:** Use `BTreeMap`/`BTreeSet` for all maps and sets. Never `HashMap`/`HashSet`.
- **IDs:** All identifiable objects get `u128` IDs via `saf_core::id::make_id()`.
- **Doc comments:** All public items must have doc comments. Wrap type names and identifiers in backticks in doc comments (e.g., `` `ValueId` ``), or clippy's `doc_markdown` lint will flag them.
- **Clippy:** Pedantic lints are enabled. When you must allow a lint, do it at function level with an explanatory comment, not at crate level.
- **Iteration:** Use `.values()` or `.keys()` instead of destructuring `for (_, val) in map`. Clippy flags this pattern.
- **Let-else:** Prefer `let Some(x) = expr else { return; };` over match expressions for the same purpose.
- **Match arms:** Combine match arms with identical bodies using `|`.
- **Cast safety:** When allowing cast lints (`cast_possible_truncation`, etc.), document the invariant that makes the cast safe.

### Steps

1. **Implement in small increments.** Follow your plan's task list. After each meaningful unit of work:
   - Run formatting and linting:
     ```bash
     make fmt && make lint
     ```
     Always run `fmt` before `lint` -- the lint target includes a formatting check that will fail on unformatted code.
   - Fix any issues immediately. Do not accumulate lint debt.

2. **Instrument with `saf_log!` calls.** Add structured logging at key decision points in your implementation:
   ```rust
   use saf_core::saf_log;

   // Narrative + key-values
   saf_log!(pta::solve, worklist, "processing node"; val=node_id, pts_size=pts.len());

   // Narrative only
   saf_log!(pta::solve, convergence, "fixpoint reached");
   ```
   Register new modules/phases in `crates/saf-core/src/lib.rs` if needed (in the `saf_log_module!` block). Tags are free-form and do not need registration.

3. **Use `SAF_LOG` to debug instead of guessing.** When behavior is unexpected, enable logging before adding print statements or guessing at fixes:
   ```bash
   docker compose run --rm -e SAF_LOG="pta::solve[worklist,pts]" dev sh -c \
     'cargo test -p saf-analysis --test pta_integration -- your_test_name'
   ```
   Common debug workflows:
   - Wrong PTA result: `SAF_LOG=pta::solve[pts,worklist]`
   - Missing callgraph edge: `SAF_LOG=callgraph[edge],pta::solve[pts]`
   - False positive/negative: `SAF_LOG=checker[reasoning,path,result]`

4. **Build inside Docker only.** Never run `cargo build` or `cargo test` locally for crates that depend on LLVM (`saf-frontends`, `saf-analysis`, `saf-python`, `saf-cli`). LLVM 18 is only inside the container. The only crate safe to build locally is `saf-core`.
   ```bash
   # Run all tests
   make test

   # Run a specific test inside Docker
   docker compose run --rm dev sh -c \
     'cargo nextest run -p saf-analysis --test your_e2e_test'
   ```

5. **Commit after each meaningful unit.** Do not accumulate a massive uncommitted diff. Each commit should represent a coherent piece of progress (a new type, a complete function, a passing test).

6. **Handle PyO3-specific patterns** (if implementing Python bindings):
   - `#[pyfunction]` requires `PyResult` return types (allow `unnecessary_wraps`)
   - `#[pymethods]` requires `&self` even for static-like methods (allow `unused_self`)
   - PyO3 needs owned types, not references (allow `needless_pass_by_value`)
   - Add type annotations and docstrings to all public Python functions

**Exit criteria:** Implementation is complete, `make fmt && make lint` passes, and you are ready for validation.

---

## Phase 7: Validation

**Goal:** Verify correctness, performance, and convention compliance through comprehensive testing and review.

**Entry criteria:** Implementation complete, lint clean.

### Steps

1. **Run the full test suite.** Capture output for inspection:
   ```bash
   make test 2>&1 | tee /tmp/test-output.txt
   ```
   Search the captured output for the summary line. SAF uses `cargo-nextest`, which prints `Summary [...] N tests run: N passed, N skipped` -- **not** the standard `test result: ok` from `cargo test`. Search for `Summary` or `passed` in the output.

   **Important:** Never re-run expensive Docker commands just to search differently. Capture once, search the file multiple times.

2. **Run formatting and lint checks:**
   ```bash
   make fmt && make lint
   ```

3. **Run benchmarks if applicable.** If your change affects core analysis (PTA, checkers, constraint extraction, graph builders):

   **PTABen** (pointer analysis benchmarks):
   ```bash
   docker compose run --rm dev sh -c \
     'cargo run --release -p saf-bench -- ptaben \
       --compiled-dir tests/benchmarks/ptaben/.compiled \
       -o /workspace/tests/benchmarks/ptaben/results.json'
   ```
   Then read `tests/benchmarks/ptaben/results.json` on the host to inspect results. Parse it to check for regressions:
   ```bash
   python3 -c "
   import json
   with open('tests/benchmarks/ptaben/results.json') as f:
       data = json.load(f)
   for cat in data['by_category']:
       print(f'{cat[\"category\"]}: Exact={cat[\"exact\"]}, Unsound={cat[\"unsound\"]}')
   "
   ```

   **Juliet** (if touching checkers):
   ```bash
   make test-juliet
   ```

   PTABen benchmarks take 30-120 seconds. Run them in the background and check output when complete. Always use `-o <path>` to write results to a file rather than capturing JSON from Docker stdout, which gets polluted with container logs.

4. **Perform code review for SAF-specific issues.** Systematically check for:

   | Issue | What to Look For |
   |---|---|
   | **Determinism violation** | Any `HashMap`, `HashSet`, `FxHashMap`, or `FxHashSet` used where output order matters. Any iteration that could produce non-deterministic results. |
   | **Missing constraint entry points** | If you added a constraint extraction step, verify it appears in all three functions: `extract_constraints()`, `extract_constraints_reachable()`, and `extract_intraprocedural_constraints()` in `crates/saf-analysis/src/pta/extract.rs`. |
   | **Unwrap in library code** | Search for `.unwrap()` in any crate except `saf-cli`. Each instance should be `.expect("reason")` or converted to `Result`. |
   | **Missing doc comments** | All new public items (functions, types, fields, modules) must have `///` doc comments. |
   | **Unguarded casts** | Any `as u32`, `as usize`, etc. should have a clippy allow with an `INVARIANT:` comment explaining why the cast is safe. |
   | **CamelCase in doc comments** | Type names like `ValueId`, `MemPhi`, `GEP` in doc comments must be wrapped in backticks, or clippy's `doc_markdown` lint will fail. |
   | **Hash iteration in output** | If any `BTreeMap`/`BTreeSet` was accidentally replaced with a hash variant during refactoring. |
   | **PyO3 compliance** | If Python bindings were added: type annotations, docstrings, correct `PyResult` wrapping. |

5. **Present findings.** Summarize:
   - Test results (total passed, any failures)
   - Lint status
   - Benchmark results (before/after if applicable, with specific category numbers)
   - Any code review issues found and fixed

**Exit criteria:** All tests pass, lint is clean, benchmarks show no regressions (or regressions are understood and accepted), and code review issues are resolved.

---

## Phase 8: Wrap-up

**Goal:** Document what was built, update project tracking, and leave the codebase ready for the next contributor.

**Entry criteria:** Validation complete, all tests pass.

### Steps

1. **Update `plans/PROGRESS.md`.** This is mandatory after any implementation work, whether complete or partial:
   - **If the plan is fully implemented:** Set the plan status to `done` in the Plans Index table. Add a `Notes:` field summarizing what was built, key files changed, and test counts.
   - **If the plan is partially implemented:** Set the plan status to `in-progress`. Add `Notes:` describing what is done, what remains, and any blockers.
   - **Update "Next Steps"** to reflect what should happen next.
   - **Append to the Session Log** with a dated summary of work done this session.

2. **Write a summary.** Provide a concise report covering:
   - **What was built:** One paragraph describing the feature.
   - **Key decisions:** Any design choices made and why.
   - **Files changed:** List the primary files created or modified (absolute paths).
   - **Test coverage:** What tests were added and what they verify.
   - **Remaining work:** Anything deferred or known to be incomplete.
   - **Benchmark impact** (if applicable): Before/after numbers with specific categories. Example: "PTABen basic_cpp_tests: 25 unsound -> 20 unsound (-5). No regressions in other categories."

3. **Verify no loose ends.** Before declaring done, confirm:
   - No uncommitted changes remain that should be committed
   - No temporary debug code or `eprintln!` statements were left behind
   - No `TODO` comments were added without corresponding plan entries
   - The plan file accurately reflects the final state of the implementation

**Exit criteria:** `PROGRESS.md` is updated, summary is written, all changes are committed, and the project is in a clean state for the next session.

---

## Quick Reference: Commands

All builds and tests run inside Docker. Never run `cargo build` or `cargo test` locally for LLVM-dependent crates.

| Action | Command |
|---|---|
| Run all tests | `make test` |
| Format code | `make fmt` |
| Lint code | `make lint` (always run `make fmt` first) |
| Format + lint | `make fmt && make lint` |
| Open dev shell | `make shell` |
| Run specific command in Docker | `docker compose run --rm dev sh -c '...'` |
| Compile C to LLVM IR | `docker compose run --rm dev sh -c 'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll'` |
| Run PTABen benchmarks | `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'` |
| Run Juliet benchmarks | `make test-juliet` |
| Enable debug logging | `docker compose run --rm -e SAF_LOG="module::phase[tag]" dev sh -c '...'` |
| Log to file | `docker compose run --rm -e SAF_LOG="pta" -e SAF_LOG_FILE=/tmp/saf.log dev sh -c '...'` |

## Quick Reference: Key Paths

| What | Path |
|---|---|
| Progress tracking | `plans/PROGRESS.md` |
| SRS document | `docs/static_analyzer_factory_srs.md` |
| Plan files | `plans/NNN-<topic>.md` |
| Core AIR types | `crates/saf-core/src/air.rs` |
| ID system | `crates/saf-core/src/id.rs` |
| Configuration | `crates/saf-core/src/config.rs` |
| Error types | `crates/saf-core/src/error.rs` |
| Log module registry | `crates/saf-core/src/lib.rs` (the `saf_log_module!` block) |
| Frontend trait | `crates/saf-frontends/src/api.rs` |
| PTA constraint extraction | `crates/saf-analysis/src/pta/extract.rs` |
| Analysis pipeline | `crates/saf-analysis/src/pipeline.rs` |
| Program database | `crates/saf-analysis/src/database/` |
| CLI entry point | `crates/saf-cli/src/main.rs` |
| CLI commands | `crates/saf-cli/src/commands.rs` |
| CLI driver | `crates/saf-cli/src/driver.rs` |
| Python module registration | `crates/saf-python/src/lib.rs` |
| Python tests | `python/tests/` |
| C test programs | `tests/programs/c/` |
| Compiled LLVM IR fixtures | `tests/fixtures/llvm/e2e/` |
| E2E Rust tests | `crates/saf-analysis/tests/*_e2e.rs` |
| PTABen compiled benchmarks | `tests/benchmarks/ptaben/.compiled/` |
| Spec files (function models) | `crates/saf-core/src/spec/` |

## Quick Reference: SAF Invariants

These are non-negotiable. Violating any of them will cause CI failures or silent analysis bugs.

1. **Determinism (NFR-DET-001):** Identical inputs must produce byte-identical outputs. Use `BTreeMap`/`BTreeSet`. Never `HashMap`/`HashSet`.
2. **AIR-only analysis (NFR-EXT-001):** Analysis operates only on AIR types. Never on LLVM IR, C AST, or other frontend-specific representations.
3. **No SVF reuse (REQ-IP-001):** Independent implementations only. Study algorithms, write original code.
4. **BLAKE3 IDs (FR-AIR-002):** All IDs are `u128`, derived via BLAKE3, serialized as `0x` + 32 hex chars.
5. **No `.unwrap()` in libraries:** Return `Result` or use `.expect("reason")` with an explanation.
6. **All three extraction entry points:** New PTA constraint types must appear in `extract_constraints()`, `extract_constraints_reachable()`, and `extract_intraprocedural_constraints()`.
7. **Docker-only builds:** LLVM 18 is only inside the container. Only `saf-core` can build locally.
8. **`cargo-nextest` output format:** Test summaries use `Summary [...] N tests run` -- not `test result: ok`.
