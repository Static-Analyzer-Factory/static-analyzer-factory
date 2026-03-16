# Plan 171: Fix `get_called_function_name()` Misclassifying Indirect Calls as Direct

## Problem

`get_called_function_name()` in `mapping.rs` determines whether a call is direct or indirect by checking if the called operand's `get_name()` returns a non-empty string. This is wrong: local SSA values also have names (e.g., LLVM auto-names call results as `%callN`). When a named local value is used as a callee, SAF treats it as a direct call to a function literally named `"call5"` or `"call6"`.

### Confirmed Impact

On libcurl.so: SAF After = 1066 vs TrueInd = 1068 (2 missing indirect calls).
The 2 calls are in `ssl_ui_reader` and `ssl_ui_writer`, where the callee is the return value of `@UI_method_get_reader` / `@UI_method_get_writer` — named `%call6` and `%call5` respectively.

## Fix

### Approach: Check `ctx.function_ids` Inside `get_called_function_name()`

Pass `known_functions: &BTreeMap<String, FunctionId>` to `get_called_function_name()`, and only return `Some(name)` if the name corresponds to a known function (defined or already registered as external) or a recognized heap allocator.

## Tasks

### Task 1: Fix `get_called_function_name()` to validate against known functions
- Change function signature to accept `known_functions: &BTreeMap<String, FunctionId>`
- Add validation: only return `Some(name)` if `known_functions.contains_key(name) || is_heap_alloc_function(name).is_some()`
- Update call site in `convert_call_instruction()`

### Task 2: Add regression test
- C source: `tests/programs/c/indirect_call_named_ssa.c`
- LLVM IR fixture: `tests/fixtures/llvm/e2e/indirect_call_named_ssa.ll` (hand-edited for named SSA callee)
- E2e test in `crates/saf-analysis/tests/cg_refinement_e2e.rs`

### Task 3: Verify on CruxBC benchmarks
- `make fmt && make lint && make test`
- CruxBC: libcurl.so `ind_call_sites` should go from 1066 → 1068
- No program should decrease

## Status: done
- Task 1: done
- Task 2: done (C source, hand-edited .ll fixture, two e2e tests)
- Task 3: done — fmt clean, lint clean, 2071 tests pass, libcurl.so IndCallSites 1066→1068 confirmed
