# E0 Scaffolding Plan: Static Analyzer Factory (SAF)

## Summary

Set up a complete greenfield project scaffold for SAF — a Rust + Python (PyO3) static analysis framework. All builds and tests run inside Docker (Ubuntu 24.04 LTS).

**Decisions locked in:**
- Rust edition 2024, MSRV 1.85.0
- Python 3.12+ minimum
- Ubuntu 24.04 LTS Docker base
- Multi-crate Rust workspace
- Docker-only build/test workflow

---

## Step 0: Save plan and initialize progress tracking

Save this plan to `plans/000-scaffolding.md`.

Create `plans/PROGRESS.md` — the living status document for cross-session continuity.

Update `CLAUDE.md` to include the **Session Workflow** section.

## Step 1: Initialize git and root config files

Create in project root:

- `git init`
- `.gitignore` — Rust (`/target/`), Python (`__pycache__/`, `*.pyc`, `*.so`, `.venv/`, `dist/`, `*.egg-info/`), IDE (`.idea/`, `.vscode/`, `.DS_Store`), project (`.cache/`)
- `.editorconfig` — UTF-8, LF, 4-space indent Rust/Python, 2-space TOML/YAML/JSON
- `Cargo.lock` will be committed (workspace has a binary crate)

## Step 2: Create Rust workspace structure

### Directory layout
```
crates/
  saf-core/src/          # AIR, graph store, config, IDs, deterministic utils
  saf-core/tests/
  saf-frontends/src/     # Frontend trait + LLVM/AIR-JSON stubs
  saf-frontends/tests/
  saf-analysis/src/      # CFG, callgraph, PTA, valueflow stubs
  saf-analysis/tests/
  saf-cli/src/           # CLI binary (clap)
  saf-cli/tests/
  saf-python/src/        # PyO3 bindings (cdylib)
  saf-python/tests/
.cargo/
```

### Root `Cargo.toml` (workspace)

```toml
[workspace]
resolver = "3"
members = ["crates/saf-core", "crates/saf-frontends", "crates/saf-analysis", "crates/saf-cli", "crates/saf-python"]

[workspace.package]
edition = "2024"
rust-version = "1.85.0"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
saf-core = { path = "crates/saf-core" }
saf-frontends = { path = "crates/saf-frontends" }
saf-analysis = { path = "crates/saf-analysis" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
blake3 = "1.8"
petgraph = "0.8"
thiserror = "2.0"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
inkwell = { version = "0.8", features = ["llvm18-0"] }
pyo3 = { version = "0.23", features = ["extension-module", "abi3-py312"] }
tempfile = "3"
pretty_assertions = "1"
```

### Crate dependency graph
```
saf-core (no internal deps)
  ├── saf-frontends (depends on saf-core)
  ├── saf-analysis  (depends on saf-core)
  ├── saf-cli       (depends on saf-core, saf-frontends, saf-analysis)
  └── saf-python    (depends on saf-core, saf-frontends, saf-analysis)
```

### Key files with real implementations at scaffold time

1. **`saf-core/src/id.rs`** — BLAKE3-based `make_id(domain, data) -> u128` and `id_to_hex(id) -> String`. Trivial to implement, needed for first TDD test.

2. **`saf-core/src/config.rs`** — Full `Config` struct with all fields from SRS Section 6, with `Serialize`/`Deserialize` derives and defaults. Fully specified in SRS.

3. **`saf-frontends/src/api.rs`** — `Frontend` trait with `ingest()`, `input_fingerprint_bytes()`, `supported_features()`, `frontend_id()`. Architectural cornerstone (NFR-EXT-001/002).

4. **`saf-cli/src/main.rs`** + **`commands.rs`** — Clap CLI skeleton with 5 subcommands (index, run, query, export, schema) + `--json-errors` flag. All stubs print "not yet implemented".

5. **`saf-python/src/lib.rs`** + **`project.rs`** — PyO3 module with `version()` function and `Project` class stub.

### Stub-only files (doc comments + empty structs)

- `saf-core/src/air.rs` — `AirBundle` struct stub (FR-AIR-001)
- `saf-core/src/graph.rs` — `GraphStore` struct stub (FR-GRAPH-001)
- `saf-core/src/deterministic.rs` — placeholder for sorting/serialization utils (NFR-DET)
- `saf-core/src/error.rs` — `CoreError` enum with `NotImplemented` variant
- `saf-frontends/src/air_json.rs` — `AirJsonFrontend` stub implementing `Frontend`
- `saf-frontends/src/llvm.rs` — `LlvmFrontend` stub implementing `Frontend`
- `saf-frontends/src/error.rs` — `FrontendError` enum
- `saf-analysis/src/cfg.rs`, `callgraph.rs`, `pta.rs`, `valueflow.rs`, `error.rs` — empty stubs

### Smoke tests (one per crate)

- `saf-core/tests/smoke.rs` — ID generation is deterministic
- `saf-frontends/tests/smoke.rs` — AIR JSON frontend returns correct `frontend_id()`
- `saf-analysis/tests/smoke.rs` — crate compiles
- `saf-cli/tests/smoke.rs` — `saf --help` succeeds and contains expected text
- `saf-python/tests/smoke.rs` — crate compiles

## Step 3: Python project setup

### `pyproject.toml`
- Build system: maturin
- `python-source = "python"`, module-name `saf._saf`
- Manifest path: `crates/saf-python/Cargo.toml`
- Dev deps: `pytest>=8.0`, `pytest-cov>=5.0`

### Python source files
```
python/saf/__init__.py      # Re-exports from saf._saf (version, Project)
python/saf/py.typed         # PEP 561 marker (empty)
python/saf/_saf.pyi         # Type stubs for native module
python/tests/__init__.py    # Empty
python/tests/test_smoke.py  # Tests: import saf, check version()
```

## Step 4: Docker infrastructure

### `Dockerfile` (multi-stage)

| Stage | Purpose | Contents |
|-------|---------|----------|
| `base` | Shared dependencies | Ubuntu 24.04, build-essential, LLVM 18 dev, Python 3.12, Rust toolchain, maturin |
| `dev` | Interactive shell | base + pytest |
| `test` | Run all tests | COPY source, cargo build, maturin develop, cargo test + pytest |
| `runtime` | Minimal binary | Ubuntu 24.04 + libllvm18 + saf binary |

Key env vars: `LLVM_SYS_180_PREFIX=/usr/lib/llvm-18`

### `docker-compose.yml`

| Service | Target | Notes |
|---------|--------|-------|
| `dev` | dev | Volume-mounts source + cargo caches for fast iteration |
| `test` | test | Clean build from COPY'd source for reproducibility |
| `build` | runtime | Produces minimal runtime image |

### `.dockerignore`
Exclude: `target/`, `.git/`, `.venv/`, `__pycache__/`, IDE files, `plans/`

## Step 5: Development tooling

- **`rustfmt.toml`** — edition 2024, `max_width = 100`
- **`.cargo/config.toml`** — clippy pedantic lints enabled, with `module_name_repetitions` and `must_use_candidate` allowed
- **`Makefile`** — Docker wrappers: `make test`, `make lint`, `make fmt`, `make shell`, `make build`, `make clean`, `make help`

## Step 6: Test fixtures and CLAUDE.md

### Test fixtures directory
```
tests/fixtures/README.md        # Documents fixture structure
tests/fixtures/bc/.gitkeep      # LLVM bitcode fixtures (future)
tests/fixtures/air_json/.gitkeep # AIR JSON fixtures (future)
tests/fixtures/expected/.gitkeep # Expected outputs for determinism tests
```

### `CLAUDE.md`
Contents:
- Project overview + SRS reference
- Development commands (all Docker-based: `make test`, `make lint`, etc.)
- Architecture (crate layout, data flow diagram, key types)
- Coding conventions (Rust: thiserror for libs, BTreeMap for determinism, no unwrap in libs; Python: type annotations, docstrings)
- TDD workflow (failing test -> implement -> refactor)
- Key constraints (no SVF code reuse, analysis only on AIR, determinism requirements)

## Step 7: Initial git commit

Stage all files and commit with message describing E0 scaffolding.

---

## Verification

After scaffolding, run these to confirm everything works:

1. **`make shell`** then **`cargo check --workspace`** — all 5 crates compile
2. **`make test`** — all smoke tests pass (Rust + Python)
3. **`cargo run -p saf-cli -- --help`** — shows 5 subcommands
4. **`maturin develop && python3 -c "from saf import version; print(version())"`** — prints `0.1.0`
5. **`make lint`** — clippy + rustfmt pass clean
6. **`docker compose build`** — all 3 Docker targets build

---

## Files Created (complete list, ~55 files)

### Root (11)
`.gitignore`, `CLAUDE.md`, `Cargo.toml`, `pyproject.toml`, `Dockerfile`, `docker-compose.yml`, `.dockerignore`, `Makefile`, `rustfmt.toml`, `.cargo/config.toml`, `.editorconfig`

### saf-core (9)
`Cargo.toml`, `src/lib.rs`, `src/air.rs`, `src/graph.rs`, `src/config.rs`, `src/id.rs`, `src/error.rs`, `src/deterministic.rs`, `tests/smoke.rs`

### saf-frontends (7)
`Cargo.toml`, `src/lib.rs`, `src/api.rs`, `src/air_json.rs`, `src/llvm.rs`, `src/error.rs`, `tests/smoke.rs`

### saf-analysis (8)
`Cargo.toml`, `src/lib.rs`, `src/cfg.rs`, `src/callgraph.rs`, `src/pta.rs`, `src/valueflow.rs`, `src/error.rs`, `tests/smoke.rs`

### saf-cli (4)
`Cargo.toml`, `src/main.rs`, `src/commands.rs`, `tests/smoke.rs`

### saf-python (4)
`Cargo.toml`, `src/lib.rs`, `src/project.rs`, `tests/smoke.rs`

### Python (5)
`python/saf/__init__.py`, `python/saf/py.typed`, `python/saf/_saf.pyi`, `python/tests/__init__.py`, `python/tests/test_smoke.py`

### Plans (2)
`plans/000-scaffolding.md` (this plan), `plans/PROGRESS.md`

### Fixtures (4)
`tests/fixtures/README.md`, `tests/fixtures/bc/.gitkeep`, `tests/fixtures/air_json/.gitkeep`, `tests/fixtures/expected/.gitkeep`
