# Contributing to SAF

Thanks for your interest in contributing to SAF! This guide covers the development workflow, coding conventions, and how to submit changes.

## Development Environment

All builds and tests run inside Docker — you don't need LLVM or Rust installed locally.

```bash
make shell       # Open interactive dev shell
make test        # Run all tests (Rust + Python)
make fmt         # Auto-format Rust code
make lint        # Run clippy + format check
make build       # Build minimal runtime image
make clean       # Remove Docker volumes and local target/
make help        # Show all available commands
```

**Always run `make fmt` before `make lint`** — lint includes a formatting check that will fail on unformatted code.

## Project Structure

```
crates/
  saf-core/       # AIR, config, IDs (no LLVM dependency)
  saf-frontends/  # LLVM bitcode + AIR-JSON frontends
  saf-analysis/   # CFG, call graph, PTA, value-flow, checkers
  saf-cli/        # CLI binary
  saf-python/     # Python SDK (PyO3 bindings)
  saf-wasm/       # Browser/WASM build
python/           # Python SDK source
tests/            # Fixtures and benchmarks
docs/             # Documentation book
tutorials/        # Step-by-step tutorial content
```

## Coding Conventions

### Rust

- **Error handling** — use `thiserror` in libraries, `anyhow` only in binaries. No `.unwrap()` in library code; use `Result` or `expect()` with a message.
- **Determinism** — use `BTreeMap`/`BTreeSet` instead of `HashMap`/`HashSet` for deterministic iteration order.
- **Doc comments** — all public items need doc comments. Wrap type names and identifiers in backticks (e.g., `` `ValueId` ``) to satisfy the `doc_markdown` clippy lint.
- **Clippy** — pedantic lints are enabled. Prefer function-level `#[allow]` with a comment over crate-level allows.

### Python

- Type annotations on all public functions
- Docstrings on all public functions and classes
- Minimum Python 3.12
- Use `uv` for package management (not pip/poetry)

## Testing

- **TDD workflow** — write failing test, implement, refactor
- **Prefer specific assertions** — `assert!(items.iter().any(|x| x == expected))` over `assert_eq!(items.len(), 2)`
- Smoke tests go in `crates/<name>/tests/smoke.rs`
- Python tests go in `python/tests/`
- Integration fixtures in `tests/fixtures/`

**Compiling C test programs to LLVM IR** (inside Docker):

```bash
make shell
clang -S -emit-llvm -g -O0 tests/programs/c/example.c -o tests/fixtures/llvm/e2e/example.ll
```

## Submitting Changes

1. Fork the repo and create a feature branch
2. Make your changes with tests
3. Run `make fmt && make lint && make test`
4. Open a pull request with a clear description of what and why

## Questions?

Open an issue if you have questions or need guidance on where to start.
