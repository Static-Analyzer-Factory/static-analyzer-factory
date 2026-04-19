# Plan 186 — LLVM 22 support + multi-version Docker images

## Status
approved — 2026-04-18

## Motivation

Users want to analyze code written against modern C/C++ language standards (C23,
C++26) that only recent clang versions can parse. The current Docker image ships
only `clang-18` / `llvm-18-dev`, so users whose source requires newer compilers
must compile externally and hope the resulting IR is LLVM-18-compatible.

Two conflated concerns to separate up-front:
- **Compiler version** (clang's C/C++ frontend parsing capability)
- **Analyzer LLVM library version** (what IR SAF can ingest)

LLVM libraries have **forward-IR-incompatibility**: an LLVM-18-linked parser
cannot read IR emitted by clang-22. Therefore supporting "modern clang" means
shipping an LLVM-22-linked SAF variant, not just a newer clang binary.

## Non-goals

- Multiple LLVMs in a single Docker image (bloat doubles per extra LLVM).
- Runtime LLVM-version switching (adds link-time complexity for no real user win).
- Publishing per-LLVM Python wheels initially (only default wheel on PyPI).
- Supporting LLVM 19/20/21 — only 18 and 22 for now.
- Backward support for LLVM 17 (deprecated; scaffold preserved for future re-add).

## Design

### Image strategy

Single parameterized Dockerfile. Two published tags:

- `saf:llvm18` + `saf:latest` (alias) — default, stable
- `saf:llvm22` — opt-in

```dockerfile
ARG LLVM_VERSION=18
RUN apt-get install -y --no-install-recommends \
      llvm-${LLVM_VERSION}-dev libclang-${LLVM_VERSION}-dev \
      clang-${LLVM_VERSION} libpolly-${LLVM_VERSION}-dev ...
ENV LLVM_SYS_${LLVM_VERSION}1_PREFIX=/usr/lib/llvm-${LLVM_VERSION}
```

Each image contains exactly one LLVM — no per-image bloat. Ubuntu 24.04's
universe may not carry `llvm-22-dev`; the Dockerfile falls back to the official
LLVM APT repository (`apt.llvm.org`) when `LLVM_VERSION` is not in the base
distro.

Docker Compose adds a second service (`dev-llvm22`); volumes are LLVM-suffixed
(`saf-target-llvm18`, `saf-target-llvm22`) so builds don't clobber each other's
cargo caches.

Makefile gains `LLVM_VERSION ?= 18`. Convenience targets:
`make shell`, `make shell-llvm22`, `make test`, `make test-llvm22`, etc.

### Cargo / feature-flag wiring

Workspace:
```toml
inkwell = { version = "0.9", default-features = false }
```

`saf-frontends/Cargo.toml`:
```toml
[features]
default = ["llvm-18"]
llvm-18 = ["dep:inkwell", "inkwell/llvm18-1"]
llvm-22 = ["dep:inkwell", "inkwell/llvm22-1"]
```

Each downstream crate (`saf-analysis`, `saf-cli`, `saf-python`, `saf-trace`,
`saf-bench`, `saf-datalog`, `saf-test-utils`) exposes passthrough features:
```toml
[features]
default = ["llvm-18"]
llvm-18 = ["saf-frontends/llvm-18"]
llvm-22 = ["saf-frontends/llvm-22"]

[dependencies]
saf-frontends = { workspace = true, default-features = false }
```

Docker build selects via `--no-default-features --features llvm-22`. Mutual
exclusion enforced at compile time with `compile_error!`.

Code changes:
- `crates/saf-frontends/src/llvm/llvm17.rs` — delete
- `crates/saf-frontends/src/llvm/llvm22.rs` — new, 3-line macro invocation
- `mod.rs`, `adapter.rs` — replace `llvm-17` cfg gates with `llvm-22`
- `create_adapter()` — add `llvm-22` arm

### `mapping.rs` + `debug_info.rs` audit

LLVM 19–22 added IR constructs the current code doesn't handle. Audit scope:

**Attributes**: `captures(...)`, `dead_on_unwind`, `nofpclass`,
`initializes(...)`, `writable` — mostly pass-through safe, but `captures` /
`initializes` carry aliasing info worth preserving long-term.

**Intrinsics**: `@llvm.ptrauth.*`, `@llvm.experimental.convergence.*`, updated
`@llvm.assume` operand bundles, new `@llvm.lifetime` signatures. `intrinsics.rs`
classifier gets new arms; unknowns degrade to `IntrinsicOp::Unknown` (warning,
not error).

**Opcodes**: GEP `inrange`, `ptradd` (LLVM 20+), new atomic orderings.

**Debug info (the big one)**: LLVM 19 replaced `llvm.dbg.declare` /
`llvm.dbg.value` intrinsic calls with non-instruction `DbgRecord`s attached to
instructions. Current `debug_info.rs` scans for intrinsic calls; needs a
cfg-gated branch to use `Instruction::get_debug_records()` on llvm-22. This
also addresses the existing Known Issue (`#dbg_declare` crashes Rust debuginfo).

**Strategy**: run every `.ll` fixture + PTABen + Juliet recompiled with clang-22
through the llvm-22 build; triage each parse/mapping failure. Add a
`strict_mode=false` config switch so unknowns warn rather than abort.

### Testing strategy

- Existing fixtures stay clang-18-compiled (LLVM 22 parses them — backward compat).
- Add `tests/programs/c/llvm22-only/` for fixtures requiring clang-22; tests
  gated by `cfg(feature="llvm-22")`. Not needed at initial bringup.
- Benchmarks run on `llvm18` only — not a multi-version regression target.

### CI

GitHub Actions `matrix.llvm: [18, 22]` for test/lint. Release workflow pushes
both image tags; `saf:latest` aliased to `saf:llvm18` via registry manifest.

## Implementation phases

### Phase 1 — Cargo scaffolding (no LLVM switching yet)
1. Bump `inkwell` to 0.9 in workspace `Cargo.toml`.
2. `saf-frontends/Cargo.toml`: add `llvm-22` feature; drop `llvm-17`.
3. Create `llvm22.rs`; delete `llvm17.rs`; update `mod.rs` and `adapter.rs` cfg.
4. Add `compile_error!` for simultaneous `llvm-18` + `llvm-22`.
5. Downstream crates: add passthrough `llvm-18`/`llvm-22` features, drop
   hardcoded `features = ["llvm-18"]`, use `default-features = false`.
6. Verify `cargo build --workspace` still succeeds under default features in
   Docker (LLVM 18 still).

### Phase 2 — Dockerfile parameterization
1. Add `ARG LLVM_VERSION=18` in base stage.
2. Parameterize apt-get install, env vars, linker paths.
3. Add `apt.llvm.org` fallback for versions not in distro.
4. Docker Compose: new `dev-llvm22` service; suffixed volumes.
5. Makefile: `LLVM_VERSION ?= 18` + `*-llvm22` convenience targets.
6. Verify `LLVM_VERSION=18 make test` passes (baseline unchanged).
7. Verify `LLVM_VERSION=22 make build` produces a working image with
   `--no-default-features --features llvm-22`.

### Phase 3 — IR compatibility audit (llvm-22 build)
1. Inside the llvm-22 image, run the full test suite; collect parse/mapping
   failures.
2. Triage: attribute/intrinsic/opcode/dbg — categorize.
3. Fix `intrinsics.rs` classifier for new intrinsics (cfg-gated where needed).
4. Fix `mapping.rs` for new opcodes/attributes.
5. Fix `debug_info.rs` for DbgRecord format.
6. Re-run `.ll` fixtures compiled with clang-22 (a small set first).
7. Re-run PTABen + Juliet against llvm-22 image — expect identical verdicts
   (if not, investigate regressions).

### Phase 4 — CI wiring
1. GitHub Actions: add `llvm-version: [18, 22]` matrix to test + lint jobs.
2. Release workflow: build + push both image tags; update `saf:latest` alias.
3. Update `README.md` install snippets showing both tags.

### Phase 5 — docs + cleanup
1. `CLAUDE.md`: update LLVM version references; document multi-version policy.
2. `docs/book/src/getting-started/llvm-versions.md` (new) — support policy,
   how to pick a tag, forward-incompatibility caveat.
3. Update `plans/PROGRESS.md`.

## Open questions (not blocking)

- Do we eventually backport new analysis features to llvm-17/earlier branches?
  For now, no — the scaffolding supports future re-adding but there's no
  current demand.

## Files touched (estimate)

- `Cargo.toml` (workspace)
- `crates/saf-frontends/Cargo.toml`
- `crates/saf-frontends/src/llvm/{mod,adapter,llvm22,intrinsics,mapping,debug_info}.rs`
- `crates/saf-frontends/src/llvm/llvm17.rs` (delete)
- 7 downstream `Cargo.toml` files
- `Dockerfile`, `docker-compose.yml`, `Makefile`
- `.github/workflows/*.yml`
- `CLAUDE.md`, `README.md`
- `plans/PROGRESS.md`
- `docs/book/src/getting-started/llvm-versions.md` (new)
