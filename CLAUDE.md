# SAF — Static Analyzer Factory

## Project Overview

SAF is a Rust + Python (PyO3) static analysis framework that provides:
- Whole-program pointer analysis and value-flow reasoning
- A schema-driven Python API for AI agents to author new analyzers
- A frontend-agnostic core operating on AIR (Analysis Intermediate Representation)

Full requirements: `docs/static_analyzer_factory_srs.md`

## Development Commands (Docker-only)

All builds and tests run inside Docker (Ubuntu 24.04 LTS):

```bash
make help        # Show all available commands
make test        # Run all tests (Rust + Python) in Docker
make lint        # Run clippy + rustfmt check in Docker (runs both targets below)
make lint-clippy # Run clippy only in Docker
make lint-fmt    # Run rustfmt check only in Docker
make fmt         # Auto-format all Rust code in Docker
make shell       # Open interactive dev shell in Docker
make build       # Build minimal runtime Docker image
make clean       # Remove Docker volumes and local target/
```

**IMPORTANT: Never run `cargo test` or `cargo build` locally** for crates that depend on LLVM (`saf-frontends`, `saf-analysis`, `saf-python`, `saf-cli`). LLVM 18 is only available inside the Docker container. Running `cargo test -p saf-analysis` locally will fail with "No suitable version of LLVM was found." Always use `make test` or `make shell` to run tests inside Docker. The only crate safe to build/test locally is `saf-core` (no LLVM dependency).

**IMPORTANT: Subagents must NEVER call `make` commands.** All `make` commands (`make test`, `make shell`, `make lint`, etc.) must be called by the main agent only. Subagents should focus on code reading, analysis, and editing — the main agent coordinates Docker/build operations.

**IMPORTANT: Always `make fmt` before `make lint`.** `make lint` includes a formatting check (`lint-fmt`) that will fail if code isn't formatted. Run `make fmt && make lint` in one command to avoid the wasted round-trip of lint-fail → fmt → lint-again.

**IMPORTANT: `make shell CMD='...'` is unreliable for passing commands.** The CMD variable may not be forwarded through Docker correctly, producing empty output or silently dropping the command. Always use `docker compose run --rm dev sh -c '...'` directly when you need to run a specific command inside Docker. Reserve `make shell` for interactive use only.

**IMPORTANT: PTABen benchmarks take 30-120s.** Always run them in the background (`run_in_background: true`) and check output later, rather than blocking and retrying when the timeout hits.

**IMPORTANT: Never re-run expensive Docker commands just to grep differently.** `make test`, `make lint`, etc. take minutes. If you need to inspect the output, capture it once (e.g., `make test 2>&1 | tee /tmp/test-output.txt`) and search the file afterward. Do not re-run the command with a different `grep` pattern.

**IMPORTANT: SAF uses `cargo-nextest`, not `cargo test`.** The test summary format is `Summary [...] N tests run: N passed, N skipped` — not the standard `test result: ok. N passed` from `cargo test`. Grep for `Summary` or `passed`, not `test result`.

## Architecture

### Crate Layout

```
crates/
  saf-core/       # AIR, config, IDs, deterministic utils (no internal deps)
  saf-frontends/  # Frontend trait + LLVM/AIR-JSON implementations (depends on saf-core)
  saf-analysis/   # CFG, callgraph, PTA, valueflow (depends on saf-core)
  saf-cli/        # CLI binary (depends on saf-core, saf-frontends, saf-analysis)
  saf-python/     # PyO3 bindings (depends on saf-core, saf-frontends, saf-analysis)
```

### Data Flow

```
Input (.ll / .bc / .air.json)
  → Frontend (implements api::Frontend trait)
    → AirBundle (canonical IR)
      → Graph builders (CFG, ICFG, CallGraph, DefUse)
        → PTA solver (Andersen CI)
          → ValueFlow graph
            → Queries (flows, taint_flow, points_to)
              → Export (JSON, SARIF)
```

### Key Types

- `saf_core::id` — BLAKE3-based deterministic ID generation (`make_id`, `id_to_hex`)
- `saf_core::config::Config` — Full configuration contract (SRS Section 6)
- `saf_core::air::AirBundle` — Canonical IR bundle from frontend ingestion
- `saf_frontends::api::Frontend` — Trait all frontends implement
- `saf_core::error::CoreError` — Error types using `thiserror`

## Coding Conventions

### Rust
- Use `thiserror` for library error types, `anyhow` only in binaries
- Use `BTreeMap`/`BTreeSet` (not `HashMap`/`HashSet`) for deterministic iteration (NFR-DET)
- No `.unwrap()` in library code — return `Result` or use `expect()` with a message
- All public items should have doc comments
- Pedantic clippy lints enabled (see `.cargo/config.toml`)
- **Clippy `doc_markdown`**: In doc comments, wrap type names, function names, and code identifiers in backticks (e.g., `` `ValueId` ``, `` `MemPhi` ``, `` `GEP` ``, `` `LiveOnEntry` ``). Without backticks, clippy treats CamelCase/PascalCase words as missing markdown links.
- **Clippy `iter_over_hash_type` / map iteration**: Use `.values()` or `.keys()` instead of destructuring `for (_, val) in map`. Clippy flags `for (_, v) in btreemap` as "you seem to want to iterate on values".
- **Clippy `manual_let_else`**: Prefer `let Some(x) = expr else { return; };` over `let x = match expr { Some(v) => v, None => return };`.
- **Clippy `match_same_arms`**: Combine match arms with identical bodies using `|` (e.g., `Operation::Cast { .. } | Operation::Gep { .. } => { ... }`).
- **PTA constraint extraction**: When adding a new constraint-generation step (e.g., `extract_function_addr_constraints`), update ALL extraction entry points: `extract_constraints()`, `extract_constraints_reachable()`, and `extract_intraprocedural_constraints()`. Missing one causes silent failures in CG refinement or CS-PTA.

### Clippy Lint Handling

**Cast safety (`cast_possible_truncation`, `cast_sign_loss`, `cast_possible_wrap`):**
When allowing cast lints, document the invariant:
```rust
// INVARIANT: LLVM limits function parameters to < 2^32
#[allow(clippy::cast_possible_truncation)]
fn param_index(i: usize) -> u32 { i as u32 }
```

**PyO3-specific allows:**
- `unnecessary_wraps`: Required for `#[pyfunction]` returning `PyResult`
- `unused_self`: Required for `#[pymethods]` on unit structs
- `needless_pass_by_value`: PyO3 requires owned types for Python conversion

**Algorithm complexity (`too_many_lines`, `too_many_arguments`):**
Add comments explaining why refactoring is not beneficial:
```rust
// NOTE: This function implements the IFDS tabulation algorithm as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
fn solve_ifds() { ... }
```

**Prefer function-level over crate-level:**
- Crate-level `#![allow]` hides issues across the entire crate
- Function-level `#[allow]` with comment documents the specific reason
- Only use crate-level for pervasive issues (doc_markdown, missing_errors_doc) or PyO3 constraints

### Python
- Type annotations on all public functions
- Docstrings on all public functions and classes
- Minimum Python 3.12
- Use `uv` for package/project management (not pip/poetry/pipenv)
  ```bash
  uv pip install -e ".[dev]"  # Install in dev mode
  uv run pytest python/tests  # Run tests
  uv add <package>            # Add dependency
  ```

### Testing
- **Prefer specific assertions over count assertions**: Use `assert!(constraints.addr.iter().any(|a| a.ptr == expected_id))` instead of `assert_eq!(constraints.addr.len(), 2)`. Exact count assertions break when unrelated improvements add legitimate new constraints.
- TDD workflow: write failing test → implement → refactor
- One smoke test per crate in `crates/<name>/tests/smoke.rs`
- Python tests in `python/tests/`
- Integration test fixtures in `tests/fixtures/`

**Test Fixture Structure:**
- **C source files**: `tests/programs/c/<name>.c`
- **Compiled LLVM IR**: `tests/fixtures/llvm/e2e/<name>.ll`
- **E2E tests**: `crates/saf-analysis/tests/*_e2e.rs` use `load_ll_fixture("<name>.ll")`

**Compiling C to LLVM IR** (must be done inside Docker):
```bash
make shell
clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll
```

**Check existing patterns before creating new files.** Don't create a file then discover the correct location — search for existing similar files first to understand the project structure.

**PTABen Benchmark Testing:**
- The PTABen submodule contains compiled LLVM IR files (`.ll`):
  - `tests/benchmarks/ptaben/.compiled/` — canonical compiled tests (`.ll` text IR)
  - `tests/benchmarks/ptaben/test_cases_bc/` — duplicate/legacy tests (DO NOT USE)
- Compile with `make compile-ptaben` (runs `scripts/compile-ptaben.sh` in Docker). The script outputs `.ll` (text IR) files. The upstream `generate_bc.sh` misleadingly names output `.bc` despite producing text IR — our script fixes this.
- The benchmark discovery accepts both `.ll` and `.bc` extensions for backwards compatibility.
- Running on the wrong path inflates test counts (e.g., 200 tests vs 90 actual)
- **Always use `--compiled-dir tests/benchmarks/ptaben/.compiled`** when running the harness
- Commands for different use cases:
  ```bash
  # Quick check (inside make shell)
  cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled

  # Filter to specific category
  cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_nullptr_deref_tests/*"

  # JSON output to file (PREFERRED for debugging — avoids Docker stdout pollution and truncation)
  cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o tests/benchmarks/ptaben/results.json
  # Also works with filter:
  cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_assert_tests/*" -o tests/benchmarks/ptaben/results.json
  ```
- **Always use `-o <path>` (not `--json` to stdout)** when investigating PTABen results from Docker. The `-o` flag writes clean JSON directly to a file inside the container (the project root is mounted at `/workspace`), bypassing all Docker stdout pollution issues. Human-readable output is still printed to stderr for visibility. The `--json` flag to stdout still works but requires `|| true` and `2>/dev/null` workarounds when run via Docker.
- **Reading JSON results after `-o`**: The output file is written inside the Docker container at `/workspace/...` which maps to the project root on the host. Read the JSON with the `Read` tool or `python3 -c "import json; ..."` on the host:
  ```bash
  # Run benchmark with -o inside Docker (use docker compose, NOT make shell CMD)
  docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
  # Then read the file on the host (same path minus /workspace/ prefix)
  # Use Read tool on: tests/benchmarks/ptaben/results.json
  # Or parse with python3:
  python3 -c "
  import json
  with open('tests/benchmarks/ptaben/results.json') as f:
      data = json.load(f)
  for cat in data['by_category']:
      print(f'{cat[\"category\"]}: Exact={cat[\"exact\"]}, Unsound={cat[\"unsound\"]}, Skip={cat[\"skip\"]}')
  "
  ```
- **Do NOT use `make test-ptaben-json`** — it sends JSON to Docker stdout which gets polluted with container logs. Always use `-o <path>` inside Docker instead.
- **Lesson learned:** When reporting benchmark results, always verify you're using the correct `--compiled-dir` path. Inflated counts from duplicate directories cause incorrect improvement claims.

**Juliet Benchmark Testing (NIST CWE Test Suite):**
- NIST Juliet C/C++ v1.3 integrated via SV-COMP YAML task definitions
- 15 supported CWE categories (~24,815 tests) across `valid-memsafety` and `no-overflow` properties
- Source: `tests/benchmarks/sv-benchmarks/c/Juliet_Test/` (YAML + preprocessed `.i` files)
- Compiled output: `tests/benchmarks/sv-benchmarks/.compiled-juliet/<CWE>/<testname>.ll`
- Always runs in aggressive mode (`conservative=false`) for definitive verdicts needed for precision/recall/F1
- Commands:
  ```bash
  make compile-juliet              # Compile all 15 CWEs (LLVM 18 + mem2reg)
  make test-juliet                 # Run full benchmark suite
  make test-juliet CWE=CWE476     # Run single CWE category
  make test-juliet-json            # JSON output to file
  make juliet-categories           # List all 15 supported CWEs
  make clean-juliet                # Remove compiled artifacts
  ```
- Scoring: per-CWE precision/recall/F1 plus SV-COMP scoring. Classification: expected_verdict=false (bad variant) with FALSE verdict = TP; expected_verdict=true (good variant) with FALSE verdict = FP.

### Shell Commands
- **Always quote arguments containing shell metacharacters** (`>`, `<`, `|`, `&`, `*`, `?`)
  - `pip install "pytest>=8.0"` — correct
  - `pip install pytest>=8.0` — WRONG: `>` is shell redirection, silently creates a junk file

### Writing Code That Consumes APIs
- **Never write code against an assumed API shape.** Before writing a script that parses or processes an API's return value, inspect the real output first with a quick `print(type(x), x)` probe in Docker. This applies to Python bindings, graph exports, PTA results, and any other structured data.
- **SAF graph exports use a unified PropertyGraph format** — all graph types share the same structure:
  ```python
  {"schema_version": "0.1.0", "graph_type": "<type>", "metadata": {},
   "nodes": [{"id": "0x...", "labels": [...], "properties": {...}}, ...],
   "edges": [{"src": "0x...", "dst": "0x...", "edge_type": "...", "properties": {}}, ...]}
  ```
  - `callgraph`: nodes have `labels: ["Function"]`, `properties.name`, `properties.kind`; edges have `edge_type: "CALLS"`
  - `cfg`: nodes have `labels: ["Block", "Entry"?]`, `properties.function`, `properties.name`; edges have `edge_type: "FLOWS_TO"`
  - `defuse`: nodes have `labels: ["Value"]` or `["Instruction"]`; edges have `edge_type: "DEFINES"` or `"USED_BY"`
  - `valueflow`: nodes have `properties.kind` (Value/Location/UnknownMem); edges have `edge_type: "Direct"|"Store"|"Load"|...`; `metadata` has `node_count`/`edge_count`
  - `pta.export()`: `{"points_to": [{"value":..., "locations":[...]}, ...], ...}` (unchanged, not PropertyGraph)

### Tutorials and Examples
- **Tutorial scripts should fail loudly.** Do not silently catch and skip errors — tutorials are teaching tools, not production services. If a compilation or analysis step fails, the user needs to see it.
- **Always test tutorials end-to-end in Docker** after creating or modifying them. Run `python3 detect.py` inside `make shell` (the venv and SAF SDK are auto-initialized by the entrypoint).

### Docker Environment
- **The dev container auto-initializes** on first run via `docker/dev-entrypoint.sh`:
  creates a venv, installs pytest, and runs `maturin develop --release` if `saf`
  is not yet importable. Subsequent runs skip the build. Just use `make shell`.

## Frontend & Docs Maintenance

### Site Structure
- Landing page: `site/` → deployed to `/` on GitHub Pages
- Tutorials: `tutorials/` → deployed to `/tutorials/`
- Playground: `playground/` → deployed to `/playground/`
- Documentation: `docs/book/` → deployed to `/docs/`
- Tutorial content: `tutorials/public/content/` (steps.json + pre-computed graphs)
- CI workflow: `.github/workflows/playground.yml` builds all four apps

### When to Update
| Event | Required Update |
|-------|----------------|
| New analysis feature | Update relevant docs concept page |
| New graph type / export format | Update PropertyGraph docs + add embeddable example |
| Playground spec files changed | Sync `playground/public/specs/` with `saf-analysis` |
| New tutorial needed | Add to `tutorials/public/content/<id>/` + update registry |
| Tutorial content outdated | Update `steps.json` + regenerate graphs if needed |
| Python SDK API changed | Update Pyodide bridge in `playground/src/analysis/pyodide-bridge.ts` |
| WASM capability changed | Update `docs/book/src/getting-started/browser-vs-full.md` |

## SAF Debug Logging (`SAF_LOG`)

SAF has a structured debug logging system controlled by the `SAF_LOG` env var.
Disabled by default — no output unless explicitly enabled. Zero recompilation needed.

**Enabling (inside Docker):**
```bash
docker compose run --rm -e SAF_LOG="pta::solve[worklist,pts]" dev sh -c 'cargo run ...'
docker compose run --rm -e SAF_LOG="pta::solve[worklist],checker[reasoning]" dev sh -c '...'
docker compose run --rm -e SAF_LOG=all dev sh -c '...'
docker compose run --rm -e SAF_LOG="all,-pta::solve[worklist]" dev sh -c '...'
```

**File output** (avoids mixing with other stderr):
```bash
docker compose run --rm -e SAF_LOG="pta" -e SAF_LOG_FILE=/tmp/saf.log dev sh -c '...'
```

**Output DSL format:**
```
[module::phase][tag] narrative | key=value key=value
```

**Value types in output:**
- `0x1a2b` — SAF ID
- `{0x1a,0x2b}` — set
- `[0x1a,0x2b]` — ordered list
- `bb1->bb3->bb5` — path
- `main->foo` — pair/edge
- `+{0x3c}` / `-{0x3c}` — set delta
- `12/50` — ratio
- Bare integers, strings, bools

**Extracting values:** grep for `key=` patterns, e.g.:
```bash
grep '\[pta::solve\]\[pts\]' /tmp/saf.log | grep 'val=0x1a2b'
grep '\[checker.*\]\[reasoning\]' /tmp/saf.log
```

**Adding logging to new code:**
```rust
use saf_core::saf_log;

// Full form: narrative + key-values (semicolon separates)
saf_log!(pta::solve, worklist, "pts grew"; val=node_id, delta=&added);

// Narrative only
saf_log!(pta::solve, convergence, "fixpoint reached");

// Keys only
saf_log!(pta::solve, stats; iter=12, worklist=342);
```
Module and phase must be registered in `saf-core/src/lib.rs` (`saf_log_module!` invocation).
Tags are free-form — no registration needed.

**Common debug workflows:**
- Wrong PTA result → `SAF_LOG=pta::solve[pts,worklist]`
- Missing callgraph edge → `SAF_LOG=callgraph[edge],pta::solve[pts]`
- False positive/negative → `SAF_LOG=checker[reasoning,path,result]`
- Slow analysis → `SAF_LOG=pta::solve[stats,convergence]`
- Interprocedural bug → `SAF_LOG=absint::interproc`

## Key Constraints

- **No SVF code reuse** — independent implementations only (REQ-IP-001)
- **Analysis operates only on AIR** — never on frontend-specific types (NFR-EXT-001)
- **Determinism required** — byte-identical outputs for identical inputs (NFR-DET-001)
- **All IDs are u128** — BLAKE3-derived, serialized as `0x` + 32 hex chars (FR-AIR-002)
- **SVFG checkers: instruction-level vs node-level mismatch** — Checker specs express instruction-level properties (call sites, resource types) but the SVFG solver operates on SSA value nodes. Three known failure modes: (1) `BTreeSet<SvfgNodeId>` deduplicates same-SSA call sites (e.g., `free(p); free(p);` → 1 node) — use call-site counting when multiplicity matters; (2) `ResourceRole` is a flat namespace — roles shared across resource categories (memory/file/lock) cause checker cross-contamination; (3) solver's `target != source` guard blocks zero-length flows where SSA collapses source and sink to the same node (e.g., alloca returned directly). When adding new checkers or modifying the solver, verify behavior at this boundary.

## Development Skills

SAF ships coding-agent skills under `skills/` for guided feature development.

### saf-feature-dev (Claude Code)

Install as a Claude Code plugin:
```bash
claude plugin add /path/to/static-analyzer-factory/skills/saf-feature-dev/claude-code
```

The skill activates automatically when developing SAF features (frontends, core analysis, Python SDK, CLI). It provides an 8-phase workflow with SAF-specific guidance: e2e testing recipes, `SAF_LOG` instrumentation, benchmark validation, and determinism checks.

To sync references after editing core content: `skills/saf-feature-dev/build.sh`

### saf-checker-dev (Claude Code)

Install as a Claude Code plugin:
```bash
claude plugin add /path/to/static-analyzer-factory/skills/saf-checker-dev/claude-code
```

The skill activates when creating bug-finding checkers, taint rules, resource leak detectors, or typestate specifications. Spec-first workflow with 3 tiers (declarative, typestate, custom patterns) and iterative test-debug-refine loop.

To sync references after editing core content: `skills/saf-checker-dev/build.sh`

## Session Workflow (MANDATORY)

### On session start
1. Read `plans/PROGRESS.md` to understand current development state
2. Read the plan file for the current/next task listed in "Next Steps"

### After brainstorming a new plan
1. Save the plan to `plans/NNN-<topic>.md` (increment NNN from the last plan)
2. Update `plans/PROGRESS.md`:
   - Add the new plan to the Plans Index with status "approved"
   - Update "Next Steps" if appropriate
   - Append to Session Log

### After ANY implementation work (complete OR partial)
1. Update `plans/PROGRESS.md`:
   - If plan fully implemented: set plan status to "done"
   - If partial (bugs, blockers, session ending): set plan status to "in-progress" and
     add Notes describing what's done, what remains, and any blockers
   - Update "Next Steps" for the next session
   - Append to Session Log with summary of work done
