# Plan 061: Interprocedural Memory Propagation

## Problem Statement

Interprocedural analysis currently doesn't improve PTABen results because:

1. **-O0 Memory Pattern**: Call results go through memory before use:
   ```llvm
   %10 = call i32 @a()       ; call returns 10
   store i32 %10, ptr %4     ; store to alloca
   %12 = load i32, ptr %4    ; load produces new ValueId
   %13 = icmp sge i32 %12, 5 ; comparison uses %12, not %10
   ```

2. **Post-Processing Limitation**: Current interprocedural refinement stores intervals at call sites only. Later instructions that load from memory get TOP intervals because the refined value went through store/load.

3. **Recursive Functions**: SCC summary computation extracts from a fixed intraprocedural result (computed with TOP parameters) instead of re-analyzing with updated summaries.

## Root Cause Analysis

### Current Flow
```
1. solve_interprocedural_with_pta_and_specs()
   ├─ solve_abstract_interp_with_pta()     # Intraprocedural: calls → TOP
   ├─ compute_all_summaries()              # Extract summaries from TOP result
   └─ refine_call_sites()                  # Post-process: add refined states at call sites
```

### Why It Fails
- `refine_call_sites()` stores `{%10: [10,10]}` at the call instruction
- The store/load sequence creates `%12` which is a different ValueId
- When querying `interval_at_inst(assertion, %12)`, we find nothing
- Memory tracking in intraprocedural didn't know about `[10,10]` because calls returned TOP

### Correct Flow (Proposed)
```
1. First Pass: Compute initial summaries
   ├─ solve_abstract_interp_with_pta()     # Intraprocedural: calls → TOP
   └─ compute_all_summaries()              # Extract initial summaries

2. Second Pass: Re-analyze with summaries
   └─ solve_abstract_interp_with_summaries() # NEW: calls → summary intervals
       └─ transfer_instruction_with_summaries() # Use summary return values
           └─ Store/Load propagates correct intervals through memory
```

## Solution Design

### Phase 1: Summary-Aware Transfer Function (~150 LOC)

Create `transfer_instruction_with_pta_and_summaries()` that handles `CallDirect`:

```rust
// In transfer.rs
pub fn transfer_instruction_with_pta_and_summaries(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,  // NEW
) {
    match &inst.op {
        Operation::CallDirect { callee } => {
            if let Some(dst) = inst.dst {
                // Use summary return interval instead of TOP
                let return_interval = summaries
                    .get(callee)
                    .and_then(|s| s.return_interval.as_ref())
                    .cloned()
                    .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));

                state.set(dst, return_interval);
            }

            // Memory invalidation based on mod/ref (existing logic)
            // ...
        }
        // Other operations: delegate to existing transfer
        _ => transfer_instruction_with_pta(inst, state, constant_map, module, pta),
    }
}
```

### Phase 2: Summary-Aware Fixpoint Solver (~100 LOC)

Create `solve_abstract_interp_with_pta_and_summaries()`:

```rust
// In fixpoint.rs
pub fn solve_abstract_interp_with_pta_and_summaries(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> AbstractInterpResult {
    // Same as solve_abstract_interp_with_pta but uses
    // transfer_instruction_with_pta_and_summaries
}
```

### Phase 3: Two-Pass Interprocedural Analysis (~50 LOC)

Modify `solve_interprocedural_with_pta_and_specs()`:

```rust
pub fn solve_interprocedural_with_pta_and_specs(...) -> InterproceduralResult {
    // Pass 1: Initial analysis with TOP call returns
    let initial_result = solve_abstract_interp_with_pta(module, config, pta);

    // Compute summaries from Pass 1
    let summaries = compute_all_summaries_with_pta_and_specs(
        module, &initial_result, pta, specs,
    );

    // Pass 2: Re-analyze with summaries (calls now use summary intervals)
    let refined_result = solve_abstract_interp_with_pta_and_summaries(
        module, config, pta, &summaries,
    );

    // Optionally: refine call sites for context-sensitive precision
    let refined_states = refine_call_sites_with_pta(...);

    InterproceduralResult {
        summaries,
        intraprocedural: refined_result,  // Now has correct intervals through memory
        refined_inst_states: refined_states,
    }
}
```

### Phase 4: Recursive Function Fixpoint (~200 LOC)

Fix `compute_recursive_scc_summaries()` to re-analyze functions:

```rust
fn compute_recursive_scc_summaries(
    scc: &[FunctionId],
    module: &AirModule,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    external_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    pta: Option<&PtaIntegration<'_>>,
) -> BTreeMap<FunctionId, FunctionSummary> {
    let func_map: BTreeMap<_, _> = module.functions.iter()
        .map(|f| (f.id, f))
        .collect();

    // Initialize with bottom summaries
    let mut scc_summaries: BTreeMap<FunctionId, FunctionSummary> = scc.iter()
        .map(|id| (*id, FunctionSummary::new()))
        .collect();

    // Fixpoint iteration
    for iteration in 0..MAX_SCC_ITERATIONS {
        let mut changed = false;

        // Combine external + current SCC summaries
        let mut all_summaries = external_summaries.clone();
        all_summaries.extend(scc_summaries.clone());

        for &func_id in scc {
            let func = func_map.get(&func_id).unwrap();

            // RE-ANALYZE the function with current summaries
            // This is the key fix: we run analysis again, not extract from fixed result
            let func_result = if let Some(pta) = pta {
                solve_function_with_pta_and_summaries(
                    func, config, constant_map, module, pta, &all_summaries,
                )
            } else {
                solve_function_with_summaries(
                    func, config, constant_map, module, &all_summaries,
                )
            };

            // Extract new summary from fresh analysis
            let new_summary = extract_summary_from_result(func, &func_result, constant_map);

            // Check if summary changed (with widening for convergence)
            let old_summary = scc_summaries.get(&func_id).unwrap();
            let widened = widen_summary(old_summary, &new_summary);

            if widened != *old_summary {
                scc_summaries.insert(func_id, widened);
                changed = true;
            }
        }

        if !changed {
            break; // Fixpoint reached
        }
    }

    scc_summaries
}

fn solve_function_with_pta_and_summaries(
    func: &AirFunction,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> FunctionAnalysisResult {
    // Similar to solve_function but uses transfer_instruction_with_pta_and_summaries
}
```

### Phase 5: Summary Widening for Convergence (~50 LOC)

Add widening for recursive summary fixpoint:

```rust
fn widen_summary(old: &FunctionSummary, new: &FunctionSummary) -> FunctionSummary {
    FunctionSummary {
        return_interval: match (&old.return_interval, &new.return_interval) {
            (Some(old_int), Some(new_int)) => Some(old_int.widen(new_int)),
            (None, Some(new_int)) => Some(new_int.clone()),
            (Some(old_int), None) => Some(old_int.clone()),
            (None, None) => None,
        },
        return_nullness: old.return_nullness.join(&new.return_nullness),
        // ... other fields
    }
}
```

## Testing Strategy

### Unit Tests
1. `summary_aware_transfer_uses_return_interval` - CallDirect uses summary
2. `two_pass_propagates_through_memory` - store/load preserves interval
3. `recursive_scc_reanalyzes_functions` - fixpoint converges

### E2E Tests
1. Simple non-recursive call: `int get10() { return 10; } svf_assert(get10() >= 5)`
2. Call through memory: `int x = get10(); svf_assert(x >= 5)` (-O0)
3. Recursive function: `addition(-10, -5)` returns -15, assert <= -10

### PTABen Validation
Run full PTABen suite and compare:
- Before: 689 Exact, 685 Unsound
- Target: Significant reduction in ae_assert_tests Unsound

## Implementation Order

1. **Phase 1**: Summary-aware transfer function (foundation)
2. **Phase 2**: Summary-aware fixpoint solver (uses Phase 1)
3. **Phase 3**: Two-pass interprocedural (uses Phase 2)
4. **Phase 4**: Recursive SCC fixpoint (uses Phase 1, 2)
5. **Phase 5**: Summary widening (needed by Phase 4)

## Estimated Effort

| Phase | LOC | Complexity |
|-------|-----|------------|
| Phase 1 | ~150 | Medium |
| Phase 2 | ~100 | Low |
| Phase 3 | ~50 | Low |
| Phase 4 | ~200 | High |
| Phase 5 | ~50 | Low |
| Tests | ~200 | Medium |
| **Total** | **~750** | |

## Risks and Mitigations

### Risk 1: Performance Regression
Two-pass analysis doubles analysis time.
- **Mitigation**: Only run second pass for functions with calls to functions that have non-TOP summaries.

### Risk 2: Recursive Non-Convergence
SCC fixpoint may not converge for complex recursive patterns.
- **Mitigation**: Widening + iteration limit (already in place).

### Risk 3: Memory Overhead
Storing summaries for all functions.
- **Mitigation**: Summaries are small (one interval + nullness per function).

## Success Criteria

1. PTABen ae_assert_tests Unsound reduced by 50%+
2. Recursive tests like `recursive_addition` pass
3. No performance regression > 2x on full PTABen suite
4. All existing tests pass

## Dependencies

- Existing `FunctionSummary` infrastructure (Plan 060)
- `IntervalQuery` trait (just added)
- PTA integration for memory tracking
