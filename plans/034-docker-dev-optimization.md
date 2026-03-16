# Plan 034: Optimize Docker Development Setup

**Save to:** `plans/034-docker-dev-optimization.md`

## Goal
Speed up `make test` and daily development workflow by: using `mold` linker, fixing stale Python extension caching, tuning build profiles, adding `cargo-nextest`, and improving Makefile granularity.

## Design Decisions (User-Approved)
- **Mold**: Docker-only via env vars in Dockerfile (not `.cargo/config.toml`)
- **Stale detection**: Source fingerprint (hash Rust sources + Cargo files)
- **Test runner**: Switch to `cargo nextest`
- **Maturin mode**: Drop `--release` for dev/test (debug mode)
- **Docker --build**: Remove from `make test`, add explicit `make rebuild`

---

## Changes

### 1. Dockerfile — Install mold + nextest, configure mold linker

**File:** `Dockerfile`

**a) Add `mold` to the apt-get install list** in the `base` stage (add after `libzstd-dev`):
```
mold \
```

**b) After the Rust install block, install cargo-nextest:**
```dockerfile
RUN cargo install cargo-nextest --locked
```

**c) Configure mold as linker via env vars** (after the Rust install and `ENV PATH` line):
```dockerfile
# Use mold linker for faster linking (Docker-only)
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-18"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="clang-18"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

**Why env vars work**: The repo's `.cargo/config.toml` sets clippy flags under `[target.'cfg(all())']`. The env vars set `[target.x86_64-unknown-linux-gnu]` flags. Cargo joins both `target.<triple>` and `target.<cfg>` rustflags when both match the current target, so both clippy flags and mold flags apply. This is documented behavior in the Cargo book.

**d) Update the `test` stage** to use nextest and debug maturin:
```dockerfile
# Change:  RUN maturin develop --release
# To:      RUN maturin develop

# Change CMD:
CMD ["sh", "-c", "cargo nextest run --workspace --exclude saf-python && pytest python/tests/ -v"]
```

### 2. Workspace Cargo.toml — Build Profile Tuning

**File:** `Cargo.toml` (workspace root)

Add at the end:

```toml
# --- Build profiles for faster development ---

# Reduce debug info for faster linking (still get file:line in backtraces)
[profile.dev]
debug = "line-tables-only"

# Optimize third-party deps even in dev mode (compiled once, run faster)
[profile.dev.package."*"]
opt-level = 2
```

**Rationale:**
- `opt-level = 2` for deps: z3, inkwell, petgraph, serde are compiled once with optimizations. Tests with Z3 constraints run significantly faster. Only recompiled when dep versions change.
- `debug = "line-tables-only"`: Reduces DWARF debug info dramatically. Still get file:line in backtraces. Cuts link time and binary size.

### 3. dev-entrypoint.sh — Source Fingerprint for Stale Detection

**File:** `docker/dev-entrypoint.sh`

Full replacement with fingerprint-based rebuild detection:
- Fingerprint = sha256 of all `.rs` and `Cargo.toml` files under `crates/`
- Stored in `.venv/.saf-build-fingerprint` (persists via Docker volume)
- Falls back to import check if fingerprint matches but module missing
- Uses `maturin develop` (debug, no `--release`)
- Checks for pytest before running pip

### 4. docker-compose.yml — Update test command

**File:** `docker-compose.yml`

Change the `test` service command:
- `cargo test` → `cargo nextest run`
- `maturin develop --release` → `maturin develop`

### 5. Makefile — Remove `--build`, Add Granular Targets

**File:** `Makefile`

Full replacement:
- `make test` no longer has `--build` flag
- New: `make test-rust` — Rust only, no Python rebuild
- New: `make test-python` — Python only
- New: `make test-crate CRATE=saf-core` — single crate
- New: `make rebuild` — explicit Docker image rebuild

---

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `Dockerfile` | Install `mold` + `cargo-nextest`; set mold env vars; update test stage |
| `Cargo.toml` | Add `[profile.dev]` and `[profile.dev.package."*"]` |
| `docker/dev-entrypoint.sh` | Source fingerprint stale detection; debug maturin; pytest check |
| `docker-compose.yml` | Update test command for nextest + debug maturin |
| `Makefile` | Remove `--build`; add `test-rust`, `test-python`, `test-crate`, `rebuild` |

## Verification

1. **Mold linker working:**
   ```bash
   make rebuild && make shell
   cargo build -p saf-core -v 2>&1 | grep mold
   # Should see: -fuse-ld=mold in the linker invocation
   ```

2. **Clippy flags still active with mold:**
   ```bash
   make shell
   cargo clippy --workspace -- -D warnings
   # Should still enforce pedantic lints
   ```

3. **Stale extension detection:**
   ```bash
   make shell           # First run: builds extension, creates fingerprint
   exit
   # Edit any .rs file in crates/saf-python/src/
   make shell           # Should see "Rust sources changed — rebuilding..."
   ```

4. **Nextest works:**
   ```bash
   make test-rust
   # Should see nextest parallel test output
   ```

5. **Full test suite:**
   ```bash
   make test
   # cargo nextest run → maturin develop → pytest
   ```

6. **Granular targets:**
   ```bash
   make test-crate CRATE=saf-core
   make test-python
   ```

7. **Build profiles active:**
   ```bash
   make shell
   cargo build -p saf-core -v 2>&1 | grep 'opt-level'
   # Dependencies should show opt-level=2
   ```

8. **Full test suite passes:**
   ```bash
   make test
   # All Rust + Python tests must pass
   ```
