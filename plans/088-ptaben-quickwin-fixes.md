# Plan 088: PTABen Quick-Win Unsound Fixes

## Context

Plan 086 done: **2200 Exact, 117 Unsound** (baseline 2046/263, −56% unsound). Investigation (docs/debug/investigation-synthesis.md) identified quick-win fixes among remaining cases. This plan targets those.

## Dispatch Prompt (for new session)

Implement Plan 088: PTABen Quick-Win Unsound Fixes. Use a team of 3 agents working in parallel.

**Current state:** 2200 Exact, 117 Unsound PTABen cases.

### Agent 1: spec-editor (YAML-only edits)

**Phase A:** Add `returns.interval: [0, 2147483647]` to the `rand` and `random` specs in `share/saf/specs/libc/stdlib.yaml`. Reference pattern: the `strlen` spec in `share/saf/specs/libc/string.yaml` already uses `returns.interval: [0, 9223372036854775807]`. The infrastructure at `crates/saf-analysis/src/absint/interprocedural.rs:407-414` (`summary_from_spec()`) already consumes this field.

**Phase B:** Add `returns.aliases: param.0` to `strchr`, `strrchr`, `strstr`, `strpbrk`, and `memchr` specs in `share/saf/specs/libc/string.yaml`. The spec constraint system at `crates/saf-analysis/src/pta/spec_constraints.rs:94-99` already handles `returns.aliases` by generating a `CopyConstraint`. No Rust changes needed.

### Agent 2: freeze-fixer (single Rust edit)

Move `Operation::Freeze` from the no-op arm to the `Operation::Copy` arm in PTA constraint extraction at `crates/saf-analysis/src/pta/extract.rs`. The no-op list is around line ~715; the Copy/Cast arm is around line ~664. This generates `CopyConstraint { dst, src: operands[0] }` for Freeze, matching the existing `value_origin.rs:862` which already treats Freeze as Copy. All three extraction entry points share `extract_instruction()`, so a single edit suffices.

### Agent 3: strcat-checker (new Rust code, ~80-120 LOC)

Add strcat-aware overflow detection in `check_memcpy_overflow_with_pta_impl` in `crates/saf-analysis/src/absint/checker.rs`. The 4 `ExtAPI_strcat_*` ae_overflow tests follow the pattern: `strcpy(buf, short_str); strcat(buf, long_str); UNSAFE_BUFACCESS(buf, N)` where combined length exceeds allocation. Neither existing checker path models strcat's append semantics. Read the example test at `tests/benchmarks/ptaben/.compiled/ae_overflow_tests/ExtAPI_strcat_01.ll` first. Approach: for strcat/strncat calls, get dest alloc size via PTA, get source string literal length from `AirModule.globals`, trace preceding strcpy for initial content length, if combined > alloc emit finding.

### Leader responsibilities

1. Spawn all 3 agents in parallel
2. After all complete: `make fmt && make lint && make test`
3. Run PTABen benchmark in background: `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'`
4. Verify no regressions (Exact ≥ 2200) and unsound improved (< 117)
5. Save the plan to `plans/088-ptaben-quickwin-fixes.md` and update `plans/PROGRESS.md`

**Expected:** 10-14 unsound cases fixed, ~120 LOC total.

## Expected Impact

| Phase | Agent | Effort | Est. Cases Fixed |
|-------|-------|--------|-----------------|
| A+B | spec-editor | ~15 YAML | up to 10 |
| C | freeze-fixer | ~5 Rust | 0 (correctness) |
| D | strcat-checker | ~100 Rust | 4 |
| **Total** | | **~120 LOC** | **10-14** |

Projected: 117 → ~103-107 Unsound

## Verification

1. `make fmt && make lint` — clippy clean
2. `make test` — all tests pass
3. PTABen benchmark (background): `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'`
4. No regressions in Exact (≥2200), unsound improved (< 117)
