<!-- SAF Feature Dev Workflow v0.1.0 — from saf-dev-skills -->

## SAF Feature Development Workflow

> Workflow for adding SAF features (frontends, core analysis, Python SDK, CLI). NOT checkers.
> For detailed references, see `saf-dev-skills/core/references/`.

### Scope

This workflow applies to four feature types:

| Type | Primary Crates | Typical Deliverables |
|------|---------------|---------------------|
| **Frontend** | `saf-frontends`, `saf-core` | New `Frontend` trait impl, AIR mapping, smoke test |
| **Core Analysis** | `saf-analysis`, `saf-core` | New/modified analysis pass, e2e test with C fixture, `saf_log!` instrumentation |
| **Python SDK** | `saf-python` + exposed Rust crate | `#[pyfunction]`/`#[pymethods]`, Python test, docstrings |
| **CLI** | `saf-cli` | New/modified command, integration test, help topic |

---

### Phase 1: Context Loading

1. **Read `plans/PROGRESS.md`** in full. It tracks the current epic, active plans, next steps, and blockers. This prevents duplicate work.
2. **Read relevant SRS sections** from `docs/static_analyzer_factory_srs.md`. Find requirements (FR-*, NFR-*) pertinent to your feature.
3. **Classify the feature type** using the table above.
4. **State your classification** (type, primary crates, one-sentence summary) and confirm before proceeding.

---

### Phase 2: Codebase Exploration

Read code before writing any. The goal is to understand existing structure and patterns.

**Frontend features:** Read `crates/saf-frontends/src/api.rs` (the `Frontend` trait), trace the LLVM frontend impl in `crates/saf-frontends/src/llvm/`, and study `crates/saf-core/src/air.rs` for AIR types (`AirBundle`, `AirModule`, `AirFunction`, `Operation` enum).

**Core Analysis features:** Read `crates/saf-analysis/src/pta/extract.rs` and locate all three extraction entry points (`extract_constraints()`, `extract_constraints_reachable()`, `extract_intraprocedural_constraints()`). Read `crates/saf-analysis/src/pipeline.rs` for pipeline stages. Read `crates/saf-analysis/src/pta/mod.rs` for the Andersen solver.

**Python SDK features:** Read `crates/saf-python/src/lib.rs` for module registration. Find the Rust API being exposed. List tests in `python/tests/` and read the most similar existing binding test.

**CLI features:** Trace the chain: `crates/saf-cli/src/main.rs` (arg parsing) -> `crates/saf-cli/src/commands.rs` (command definitions) -> `crates/saf-cli/src/driver.rs` (`AnalysisDriver` pipeline). Check `crates/saf-cli/src/help.rs` for help topics.

After exploration, summarize: which functions/types you will modify, which patterns to follow, any surprises, and related existing tests.

---

### Phase 3: Clarifying Questions

Work through each concern and determine whether it applies:

| Concern | Question | Why It Matters |
|---------|----------|---------------|
| **Determinism** | Does this introduce hash-based iteration or non-deterministic output? | NFR-DET-001: byte-identical outputs for identical inputs. Use `BTreeMap`/`BTreeSet`, never `HashMap`/`HashSet`. |
| **Cross-crate impact** | Does this change `saf-core` types (`AirBundle`, `Config`, `Frontend` trait)? | Changes to `saf-core` ripple to every downstream crate. |
| **Python exposure** | Should this be accessible from the Python SDK? | Python bindings need owned types, `PyResult` returns, and docstrings. Plan early. |
| **Benchmark impact** | Could this affect PTA precision or analysis performance? | Run PTABen/Juliet benchmarks before and after. |
| **AIR compatibility** | Does this need new AIR types or operations? | All analysis operates on AIR only (NFR-EXT-001). New ops go in `saf-core`'s `Operation` enum. |
| **ID system** | Does this create new identifiable objects? | All IDs are `u128`, BLAKE3-derived via `saf_core::id::make_id()`, serialized as `0x` + 32 hex chars. |
| **Constraint extraction** | Does this add a PTA constraint type? | Must update ALL THREE extraction entry points (see invariants below). |
| **No SVF reuse** | Is this inspired by SVF code? | REQ-IP-001: independent implementations only. Study algorithms, write original code. |

Present all questions at once. Do not start implementation with unresolved questions about SAF invariants.

---

### Phase 4: Plan & Design

1. **Explore 2-3 approaches.** Evaluate each against SAF constraints: AIR-only, no SVF reuse, determinism (BTreeMap/BTreeSet), BLAKE3 IDs, error handling (thiserror in libraries, anyhow only in saf-cli, no `.unwrap()`).
2. **Present approaches with tradeoffs:** what changes in which files, performance, complexity, interaction with PTA/checkers/pipeline.
3. **Write a plan file** to `plans/NNN-<topic>.md` (increment from the highest existing plan number). Structure: objective, approach, task list, affected files, test strategy, risks.
4. **Update `plans/PROGRESS.md`:** add plan to Plans Index with status `approved`.

---

### Phase 5: Test First (E2E Preferred)

SAF strongly prefers e2e tests with real C programs over unit tests.

**Test preference hierarchy:**
1. E2E with C source (compile C -> LLVM IR -> analysis -> assertions)
2. E2E with handwritten `.ll` (precise IR control)
3. Integration test (public crate API)
4. Unit test (pure algorithms only)

**Recipe: C source to e2e test**

1. Write a minimal C program at `tests/programs/c/<name>.c`. One behavior per file.

2. Compile to LLVM IR inside Docker (LLVM 18 is only in the container):
   ```bash
   docker compose run --rm dev sh -c \
     'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c \
      -o tests/fixtures/llvm/e2e/<name>.ll'
   ```

3. Write a Rust e2e test in `crates/saf-analysis/tests/<name>_e2e.rs`. Load the fixture with `saf_test_utils::load_ll_fixture("<name>")`. For Python tests, create `python/tests/test_<name>.py`.

4. **Use specific assertions, not count assertions:**
   ```rust
   // GOOD: survives unrelated changes
   assert!(constraints.addr.iter().any(|a| a.ptr == expected_id));
   // FRAGILE: breaks when anything adds a new constraint
   assert_eq!(constraints.addr.len(), 2);
   ```

5. Verify tests fail with assertion errors (not build errors):
   ```bash
   make test 2>&1 | tee /tmp/test-output.txt
   ```
   SAF uses `cargo-nextest`. Search for `Summary` or `passed`, not `test result:`.

For detailed e2e examples including PTA, constraint extraction, and Python tests, see `saf-dev-skills/core/references/e2e-testing-guide.md`.

---

### Phase 6: Implementation

**Coding conventions (non-negotiable):**

- **Error handling:** `thiserror` in library crates. `anyhow` only in `saf-cli`. No `.unwrap()` in library code -- use `Result` or `.expect("reason")`.
- **Determinism:** `BTreeMap`/`BTreeSet` for all maps and sets. Never `HashMap`/`HashSet`. Exception: `IndexMap`/`FxHashMap` in documented PTA hot paths where output order is normalized afterward.
- **IDs:** `u128` via `saf_core::id::make_id()`. Serialized as `0x` + 32 hex chars.
- **Doc comments:** All public items. Wrap type names in backticks (`` `ValueId` ``), or clippy's `doc_markdown` lint will flag them.
- **Clippy:** Pedantic lints enabled. Allow lints at function level with explanatory comments, not crate level.
- **Iteration:** Use `.values()` or `.keys()` instead of `for (_, val) in map`.
- **Let-else:** Prefer `let Some(x) = expr else { return; };`.
- **Match arms:** Combine identical bodies with `|`.
- **Cast safety:** Document invariants when allowing cast lints.

**Implementation steps:**

1. **Implement in small increments.** After each unit of work, run:
   ```bash
   make fmt && make lint
   ```
   Always `fmt` before `lint` -- lint includes a formatting check.

2. **Add `saf_log!` instrumentation** at key decision points:
   ```rust
   use saf_core::saf_log;
   // Narrative + key-values (semicolon separates)
   saf_log!(pta::solve, worklist, "processing node"; val=node_id, pts_size=pts.len());
   // Narrative only
   saf_log!(pta::solve, convergence, "fixpoint reached");
   ```
   Register new modules/phases in `crates/saf-core/src/lib.rs` (`saf_log_module!` block). Tags are free-form.

3. **Debug with `SAF_LOG`, not guesswork:**
   ```bash
   docker compose run --rm -e SAF_LOG="pta::solve[worklist,pts]" dev sh -c \
     'cargo nextest run -p saf-analysis --test your_test'
   ```
   Common filters: wrong PTA = `pta::solve[pts,worklist]`, missing CG edge = `callgraph[edge],pta::solve[pts]`, false positive = `checker[reasoning,path,result]`.

4. **Build inside Docker only.** Never run `cargo build`/`cargo test` locally for LLVM-dependent crates (`saf-frontends`, `saf-analysis`, `saf-python`, `saf-cli`). Only `saf-core` can build locally.

5. **PyO3 patterns** (Python bindings only): allow `unnecessary_wraps` for `PyResult`, `unused_self` for `#[pymethods]`, `needless_pass_by_value` for owned types. Add type annotations and docstrings.

For detailed `saf_log!` usage (value types, filter grammar, file output), see `saf-dev-skills/core/references/saf-log-guide.md`. For per-feature-type checklists, see `saf-dev-skills/core/references/feature-type-guides.md`.

---

### Phase 7: Validation

1. **Run full test suite** (capture output once, search the file, do not re-run):
   ```bash
   make test 2>&1 | tee /tmp/test-output.txt
   ```

2. **Run formatting and lint:**
   ```bash
   make fmt && make lint
   ```

3. **Run benchmarks** if your change affects core analysis:
   - **PTABen** (30-120s, run in background):
     ```bash
     docker compose run --rm dev sh -c \
       'cargo run --release -p saf-bench -- ptaben \
         --compiled-dir tests/benchmarks/ptaben/.compiled \
         -o /workspace/tests/benchmarks/ptaben/results.json'
     ```
     Then read `tests/benchmarks/ptaben/results.json` on the host. Always use `-o <path>`, not `--json` to stdout.
   - **Juliet** (if touching checkers): `make test-juliet`

4. **Code review checklist:**

   | Issue | What to look for |
   |-------|-----------------|
   | Determinism violation | `HashMap`, `HashSet`, `FxHashMap` where output order matters |
   | Missing constraint entry points | New extraction step not in all 3 functions in `pta/extract.rs` |
   | `.unwrap()` in library code | Should be `.expect("reason")` or `Result` |
   | Missing doc comments | All new public items need `///` docs |
   | Unguarded casts | `as u32` etc. needs `#[allow]` with `INVARIANT:` comment |
   | CamelCase in doc comments | Type names like `ValueId` must be in backticks |
   | PyO3 compliance | Type annotations, docstrings, `PyResult` wrapping |

---

### Phase 8: Wrap-up

1. **Update `plans/PROGRESS.md`:**
   - Fully implemented: set plan status to `done`, add `Notes:` summary
   - Partially implemented: set to `in-progress`, describe what remains and blockers
   - Update "Next Steps" for the next session
   - Append to Session Log with dated summary

2. **Write a summary** covering: what was built, key decisions, files changed (absolute paths), test coverage, remaining work, benchmark impact (before/after if applicable).

3. **Verify no loose ends:** no uncommitted changes, no debug `eprintln!`, no `TODO` without plan entries.

---

### Quick Reference: Commands

| Action | Command |
|--------|---------|
| All tests | `make test` |
| Format + lint | `make fmt && make lint` |
| Docker command | `docker compose run --rm dev sh -c '...'` |
| PTABen | See Phase 7 |
| Debug logging | `docker compose run --rm -e SAF_LOG="..." dev sh -c '...'` |

### SAF Invariants (Non-Negotiable)

1. **Determinism (NFR-DET-001):** Identical inputs produce byte-identical outputs. `BTreeMap`/`BTreeSet` only (exception: `IndexMap`/`FxHashMap` in documented PTA hot paths).
2. **AIR-only analysis (NFR-EXT-001):** Analysis operates only on AIR types, never frontend-specific representations.
3. **No SVF reuse (REQ-IP-001):** Independent implementations only.
4. **BLAKE3 IDs:** All IDs are `u128`, BLAKE3-derived, `0x` + 32 hex chars.
5. **PTA triple-update:** New constraint types must appear in all three extraction entry points in `pta/extract.rs`.
6. **Docker-only builds:** LLVM 18 is only inside the container. Only `saf-core` can build locally.
7. **`cargo-nextest`:** Grep for `Summary` or `passed`, not `test result: ok`.
