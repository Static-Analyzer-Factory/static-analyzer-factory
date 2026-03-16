# Plan 034: Optimize Docker Image Structure

## Goal

Restructure Docker images for faster daily development, correct Python test freshness, and disk-space efficiency.

## Changes

### Dockerfile
- Split monolithic `base` stage into `system-deps` (apt packages, ~1.5 GB) + `toolchain` (Rust + maturin, ~800 MB)
- Parameterized Rust version via `ARG RUST_VERSION=1.85.0`
- `dev` and `test` stages now inherit from `toolchain` instead of `base`
- Bumping Rust/maturin only rebuilds `toolchain` onward, reusing the LLVM apt layer

### docker/dev-entrypoint.sh
- **Stale binary fix:** Compares `.so` mtime against `crates/**/*.rs` via `find -newer`
- **pip overhead fix:** Uses `command -v pytest` (~0ms) instead of `pip install` (~1-3s)
- **Known .so path:** Checks `/workspace/python/saf/_saf.abi3.so` directly

### docker-compose.yml
- Added `venv-cache:/workspace/.venv` named volume to `dev` and `test` services
- Declared `venv-cache` in volumes section
- Faster I/O on macOS, stops `.venv/` from appearing in host source tree

### Makefile
- `test` drops `--build` — no Docker rebuild check on every test run
- `test-rust` — Rust-only iteration, skips Python entirely
- `test-python` — always runs `maturin develop` before pytest
- `rebuild` — explicit target for Dockerfile changes
- `test-clean` keeps `--build` for hermetic builds

### CLAUDE.md
- Updated Development Commands table with new targets
- Updated Docker Environment description for new entrypoint behavior
