# Plan 075: ae_recursion_tests Unsound Fixes

## Problem

All 33 ae_recursion_tests are Unsound (0 Exact). Every test shows LHS interval as TOP (`[-2^63, 2^63-1]`), meaning the abstract interpretation fails to compute any useful intervals for recursive function return values.

## Root Cause Analysis

### Core Issue: `-O0` Alloca Pattern Defeats Recursive Summary Fixpoint

At `-O0`, LLVM generates store/load patterns for ALL parameter access:
```llvm
define i32 @sum(i32 %0, i32 %1) {
  %n_alloca = alloca i32
  store i32 %0, ptr %n_alloca          ; param → alloca
  %n = load i32, ptr %n_alloca          ; alloca → register
  %cmp = icmp sle i32 %n, 0            ; branch condition
  ...
  %rec = call i32 @sum(i32 sub(%n,1), i32 add(%m,1))  ; recursive call
  store i32 %rec, ptr %ret_alloca      ; return via alloca
  ...
  %ret = load i32, ptr %ret_alloca
  ret i32 %ret
}
```

### Why Current Analysis Fails

`compute_recursive_scc_summaries_with_pta` initializes parameters to TOP in `solve_single_function_with_pta_and_summaries`. At `-O0`:
1. TOP params → stored to allocas → loaded back as TOP
2. Branch conditions on TOP → both branches reachable
3. Base case arithmetic on TOP → return TOP
4. Recursive case with TOP summary → return TOP
5. Summary converges immediately at TOP (no improvement possible)

Phase 3 (second pass) tries `analyze_callee_inline` for `main→sum(0,3)`, but inline hits the recursive call which uses the TOP summary, producing TOP.

### Test Categories & Their Specific Issues

| Category | Count | Caller Args | Key Challenge |
|----------|-------|-------------|---------------|
| recursive_sum | 3 | Concrete (0,3), (-10,3), (-100,3) | Simple tail-recursive, param binding solves |
| recursive_addition | 5 | Concrete (10,5), (0,5), etc. | Same as sum |
| recursive_mc91 | 9 | Concrete (40-120) | Nested `mc91(mc91(p+11))`, inner call arg depends on fixpoint |
| recursive_id | 10 | TOP (scanf) + some with guard | Has clamping (`if ret > N return N`), fixpoint must discover bound |
| recursive_afterrec | 5 | Concrete (4, 10, 1000) | Void fn, check global `g=3` side effect, not return value |
| demo | 1 | Concrete (0) | Missing return on recursive path (source bug) |

## Solution: Caller-Argument-Bound Recursive Summary Computation

### Key Insight

Instead of computing a universal summary from TOP parameters, bind parameters to known caller argument intervals. Then iterate the fixpoint with progressively refined parameter + return intervals.

For PTABen tests, `main()` always passes concrete arguments, so parameter binding is immediately precise.

For `recursive_id` tests (scanf → TOP args), the fixpoint still benefits because the **clamping logic** inside the function bounds the return value regardless of input.

## Implementation Plan

### Phase A: Add param_intervals to solve_single_function (~15 LOC)

**File**: `crates/saf-analysis/src/absint/interprocedural.rs`

Modify `solve_single_function_with_pta_and_summaries` to accept optional parameter intervals:

```rust
fn solve_single_function_with_pta_and_summaries(
    func: &AirFunction,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
    param_intervals: Option<&[Interval]>,  // NEW
) -> SingleFunctionResult
```

In the entry state initialization (line ~1278-1282):
```rust
let mut entry_state = AbstractState::new();
for (i, param) in func.params.iter().enumerate() {
    let interval = param_intervals
        .and_then(|pis| pis.get(i))
        .cloned()
        .unwrap_or_else(|| Interval::make_top(64));
    entry_state.set(param.id, interval);
}
```

Update all call sites to pass `None` (backward compatible).

### Phase B: Collect external caller arguments (~40 LOC)

**File**: `crates/saf-analysis/src/absint/interprocedural.rs`

New function:
```rust
fn collect_external_call_args(
    module: &AirModule,
    scc: &BTreeSet<FunctionId>,
    constant_map: &BTreeMap<ValueId, Interval>,
    existing_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
) -> BTreeMap<FunctionId, Vec<Interval>>
```

For each non-SCC function that calls an SCC function:
1. Run a quick single-function analysis with summaries (for the calling function)
2. Extract argument intervals at the call site from the analysis state
3. Join all call site argument intervals per parameter position
4. Return map: `FunctionId → [joined_param0, joined_param1, ...]`

**Optimization**: For simple callers like `main()` that just load constants and call, we can extract directly from the constant map without a full analysis.

Simpler approach — scan call instructions in non-SCC functions:
- For each `CallDirect` to SCC function, collect operand intervals from constants
- Join per-parameter across all call sites
- This covers the PTABen case where `main()` passes constants

### Phase C: Argument-bound recursive fixpoint (~50 LOC)

**File**: `crates/saf-analysis/src/absint/interprocedural.rs`

Modify `compute_recursive_scc_summaries_with_pta`:

1. Accept `external_call_args: &BTreeMap<FunctionId, Vec<Interval>>` parameter
2. In the fixpoint loop, compute **parameter intervals** for each function:
   - Start with external caller args (from Phase B)
   - On each iteration, also collect recursive call argument intervals from the analysis
   - Join external + recursive args per parameter position
3. Pass these to `solve_single_function_with_pta_and_summaries` via the new `param_intervals` parameter

New helper:
```rust
fn extract_recursive_call_args_from_result(
    func: &AirFunction,
    scc: &BTreeSet<FunctionId>,
    result: &SingleFunctionResult,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> BTreeMap<FunctionId, Vec<Vec<Interval>>>
```

Scans the function's instructions for `CallDirect` to SCC members, extracts argument intervals from the `inst_states` at each call site.

### Phase D: Wire into pipeline (~10 LOC)

**File**: `crates/saf-analysis/src/absint/interprocedural.rs`

In `solve_with_pta_impl`, before calling `compute_recursive_scc_summaries_with_pta`:
1. Call `collect_external_call_args()` for the current SCC
2. Pass result to the modified function

### Phase E: Global side-effect propagation for afterrec tests (~30 LOC)

The `afterrec` tests assert `g == 3` after calling `f(n)`. The function `f` is void and stores to global `@g` in the base case.

In Phase 3 (second pass), `main()` is analyzed with summaries. `f()` is called and returns void, but we need the global store `store i32 3, ptr @g` to be reflected in `main`'s abstract state.

The current approach: `analyze_callee_inline` for `f()` in main's context. It processes `f`'s instructions including the `store i32 3, ptr @g`. If PTA resolves `@g` correctly, this store updates `main`'s `loc_memory` state. Then `load ptr @g` in `main` retrieves `[3,3]`.

**Potential issue**: `f()` has 2 blocks + 10 instructions — should qualify for inline analysis (≤3 blocks, ≤20 instructions). But `f()` recursively calls itself. `analyze_callee_inline` will hit `CallDirect @f` and try `compute_call_return_with_summaries`, which falls back to the summary (void, no return interval). The inline analysis processes the base case store `g=3` and the recursive call (void, ignored). The global store should propagate.

**However**, there's a subtlety: the inline analysis processes blocks linearly without branching refinement. If both branches are taken (because `n` from caller is concrete but the check `n < 3` compares after alloca load), the base case branch sets `g=3` regardless.

Likely works already once the other phases are implemented. Verify and add specific handling if needed.

## Files to Modify

- `crates/saf-analysis/src/absint/interprocedural.rs` — All phases
  - `solve_single_function_with_pta_and_summaries` — add `param_intervals` parameter (Phase A)
  - `compute_recursive_scc_summaries_with_pta` — accept & use external args (Phase C)
  - `solve_with_pta_impl` — wire caller arg collection (Phase D)
  - New: `collect_external_call_args()` (Phase B)
  - New: `extract_recursive_call_args_from_result()` (Phase C)

## Expected Outcomes

- **High confidence (≥20 tests)**: recursive_sum (3), recursive_addition (5), recursive_afterrec (5), recursive_mc91 with concrete args that don't recurse deeply (mc91_4=exact, others with fixpoint ≥9)
- **Medium confidence**: recursive_id tests where clamping is tight (0-4)
- **Lower confidence**: recursive_id tests with mutual recursion (5-9), demo (source has undefined behavior)
- **Target**: 33 → 5-10 Unsound

## Verification

```bash
# Inside Docker (make shell):
cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_recursion_tests/*"

# Full PTABen regression check:
cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled

# Unit tests and lint:
cargo nextest run --workspace --exclude saf-python
```
