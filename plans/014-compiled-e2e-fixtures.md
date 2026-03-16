# Plan 014: Replace Hand-Crafted AIR-JSON Fixtures with Real Compiled LLVM IR

## Goal

Replace all 23 hand-crafted `.air.json` E2E fixtures with real `.ll` files compiled from the existing C/C++/Rust source programs. The E2E tests will exercise the full pipeline: source → clang/rustc → .ll → LLVM frontend → AIR → analysis → findings.

## Motivation

The current `.air.json` fixtures are hand-written approximations of what real compiler output looks like. They pass tests but don't validate the actual compilation pipeline. Real compiled `.ll` files will:
1. Exercise the LLVM frontend against real compiler output (clang-18, clang++-18, rustc)
2. Validate that SAF can detect vulnerabilities in genuinely compiled programs
3. Remove the maintenance burden of keeping hand-crafted AIR in sync with evolving schemas

## Decisions

- **Replace entirely**: Delete hand-crafted `.air.json` fixtures; E2E tests use compiled `.ll` files only
- **Pre-compile and commit**: Run `compile.sh` once, commit `.ll` files to repo
- **Fix-as-you-go**: Fix each program's bugs before moving to the next
- **Batch by category**: Split work into 5 batches for context-safe checkpoints

## Changes

### 1. `.gitignore` Update
Add exception for fixture `.ll` files:
```
# Generated LLVM IR (tutorials compile these at runtime)
*.ll
!tests/fixtures/**/*.ll
```

### 2. Compile All Programs
Run `compile.sh` in Docker to produce `.ll` files in `tests/fixtures/llvm/e2e/`.

### 3. Rust E2E Test Updates
Update `load_e2e_fixture()` helper (or create `load_e2e_ll_fixture()`) to:
- Load `.ll` file from `tests/fixtures/llvm/e2e/`
- Ingest via `LlvmFrontend` to produce `AirBundle`
- Return the first module from the bundle

### 4. Python E2E Test Updates
Update `conftest.py` to point at `.ll` files and use `Project.open("...ll")`.

### 5. Cleanup
Delete all 23 `.air.json` files from `tests/fixtures/e2e/`.

## Batch Schedule

Each batch is a self-contained checkpoint — safe to stop between batches.

| Batch | Programs | Count | Category |
|-------|----------|-------|----------|
| 1 | command_injection, format_string, sql_injection, path_traversal, taint_sanitized, taint_unsafe | 6 | Taint flow |
| 2 | use_after_free, double_free, null_deref, buffer_overflow, callback_fn_ptr, cross_module_taint | 6 | Memory safety |
| 3 | integer_overflow, info_leak, uninitialized, uninitialized_heap | 4 | Integer/info |
| 4 | vtable_dispatch, raii_resource, dangling_ptr_cpp, dangling_ptr_rs, trait_dispatch | 5 | OOP patterns |
| 5 | library_wrapper, callback_chain + delete old .air.json + Python test updates | 2+cleanup | Multi-module + cleanup |

## Expected Issues

The LLVM frontend has been tested against tutorial programs but not all 23. Likely issues:
- Unhandled LLVM instructions (e.g., `invoke`, `phi`, `switch`, `select`)
- Missing intrinsic patterns
- SSA value resolution failures for complex operand patterns
- Test assertion adjustments (real IR has more instructions than hand-crafted)
