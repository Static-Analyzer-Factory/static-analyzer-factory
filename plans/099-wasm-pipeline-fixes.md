# Plan 099: WASM Pipeline Accuracy Fixes

## Context

Audit (session 098b) identified 7 gaps (D1-D7) between the WASM analysis pipeline
and the reference Rust/Python pipeline. This plan fixes D1-D4, D6-D7. D5 (spec
constraints) is deferred — WASM runs in a browser sandbox with no filesystem access,
so loading spec YAML files requires embedding infrastructure (future work).

## Bug Inventory

| # | Gap | Severity | Fix |
|---|-----|----------|-----|
| D1 | No iterative CG refinement | High | Use `cg_refinement::refine()` or iterative loop |
| D2 | PTA max_iterations 1M vs 10K | Low | Align config |
| D3 | ValueFlow mode hardcoded Precise | Medium | Parse config_json |
| D4 | Missing inst.dst PTA check | Medium | Add dst check in resolve_indirect_calls |
| D5 | No spec constraints | High | **DEFERRED** — needs WASM-compatible spec loading |
| D6 | `kind` vs `cast_kind`/`binary_op`/`heap_kind` | Bug | Fix TS field names + add GlobalRef to Rust schema |
| D7 | Missing `type_hierarchy`/`constants` in TS | Low | Add fields to TS types |

## Agent Assignment (Non-overlapping files)

### Agent 1: WASM Pipeline (D1, D2, D3, D4)
**File:** `crates/saf-wasm/src/lib.rs`

- D4: Add `inst.dst` check in `resolve_indirect_calls()`
- D3: Parse `config_json` for `vf_mode`, `pta_max_iterations`, `max_refinement_iters`
- D2: Default PTA `max_iterations` to 10,000 (match Python)
- D1: Wire in `cg_refinement::refine()` if available, else implement iterative loop

### Agent 2: TS Converter & Types (D6, D7)
**Files:** `playground/src/analysis/cst-to-air.ts`, `playground/src/types/air.ts`

- D6: Fix field names: `kind` → `cast_kind` / `binary_op` / `heap_kind`
- D6: Add `global_ref` variant to `AirConstant` type
- D7: Add `type_hierarchy` and `constants` fields to `AirModule`
- D7: Update `AirInstruction` to use specific field names

### Agent 3: Rust JSON Schema (GlobalRef)
**Files:** `crates/saf-frontends/src/air_json_schema.rs`, `crates/saf-frontends/src/air_json.rs`

- Add `GlobalRef` variant to `JsonConstant` enum
- Add conversion case in `convert_constant()`

## Deferred: D5 (Spec Constraints)

Neither WASM nor Python currently uses specs. WASM can't access the filesystem.
Options for future:
1. Embed spec YAMLs at compile time via `include_str!`
2. Accept specs as part of `config_json`
3. Build a `SpecRegistry::from_yaml_strs()` API

## Verification

1. `cd playground && npx tsc --noEmit` — TS type checks pass
2. `make fmt && make lint` — Rust clippy/fmt pass
3. `make test` — all Rust tests pass
4. Browser test with all examples — graphs render correctly
