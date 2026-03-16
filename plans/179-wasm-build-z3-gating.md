# Plan 179: WASM Build Z3 Gating

**Goal:** Restore `make wasm` by ensuring the `saf-wasm` dependency graph compiles when `saf-analysis` is built without default features.

**Context:** `crates/saf-wasm/Cargo.toml` depends on `saf-analysis` with `default-features = false`, but `crates/saf-analysis/src/checkers/solver.rs` still compiled partial-leak helper code that directly referenced `z3`. This broke `wasm32-unknown-unknown` builds before `wasm-bindgen` ran.

## Tasks

1. Gate Z3-only partial-leak helpers in `checkers/solver.rs` behind `feature = "z3-solver"`.
2. Provide a non-Z3 fallback `detect_partial_leaks_svfg()` so non-default-feature builds remain compilable.
3. Gate Z3-dependent unit tests the same way.
4. Re-run `make wasm` and verify the package is emitted under `packages/shared/src/wasm`.

## Outcome

- `make wasm` succeeds locally after installing `wasm32-unknown-unknown`.
- The build now tolerates `saf-analysis` without `z3-solver` in the WASM dependency path.
