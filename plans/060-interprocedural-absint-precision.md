# Plan 060: Interprocedural Abstract Interpretation Precision

**Epic**: E35 (Interprocedural Absint Precision)
**Status**: proposed
**Depends on**: Plan 057, Plan 059

## Overview

This plan addresses four root causes of unsound results in PTABen's abstract interpretation tests. The fixes target interprocedural analysis gaps that cause ~130 unsound cases across `ae_assert_tests`, `ae_nullptr_deref_tests`, `ae_overflow_tests`, and `ae_recursion_tests`.

## Problem Statement

### Current PTABen Unsound Cases (Abstract Interpretation)

| Category | Unsound | Root Cause |
|----------|---------|------------|
| ae_assert_tests | 25 | Function calls invalidate all tracked values |
| ae_nullptr_deref_tests | 25 | No interprocedural return nullness tracking |
| ae_overflow_tests | 53 | ALLOCA size not tracked; string length semantics |
| ae_recursion_tests | 26 | No recursive function summary fixpoint |
| **Total** | **129** | |

### Root Causes

1. **Memory invalidation on call**: `nullness.rs` calls `state.invalidate_all_memory()` on every non-pure function call, destroying tracked values even when the callee doesn't modify them.

2. **No interprocedural return nullness**: Calls to defined functions return `MaybeNull` unconditionally. The existing `InterproceduralResult` has summaries but nullness analysis doesn't use them.

3. **Missing ALLOCA size**: The `Operation::Alloca` variant has no size field, unlike `HeapAlloc { kind }`. Stack buffer overflow detection requires knowing allocation sizes.

4. **No recursive summary fixpoint**: `compute_function_summary()` runs once per function. Recursive functions need iterative widening until summaries stabilize.

## Phases

### Phase A: Selective Memory Invalidation via Mod/Ref (~300 LOC)

**Goal**: Replace `invalidate_all_memory()` with mod/ref-guided selective invalidation.

#### Task A1: Integrate Mod/Ref into Nullness Analysis (~100 LOC)

**Current state**: `nullness.rs:1496-1501` blindly invalidates all memory on any call.

**Files**:
- `crates/saf-analysis/src/absint/nullness.rs`

**Implementation**:
1. Add `mod_ref: Option<&BTreeMap<FunctionId, ModRefSummary>>` parameter to `analyze_nullness_with_pta_and_specs`
2. At `CallDirect`, look up callee's `ModRefSummary`
3. Only invalidate locations in `summary.modified_locs`
4. If callee has no summary (external, unknown), fall back to full invalidation

**Transfer function change**:
```rust
// Before:
state.invalidate_all_memory();
state.invalidate_all_loc_memory();

// After:
if let Some(mod_ref_map) = mod_ref {
    if let Some(summary) = mod_ref_map.get(&callee_id) {
        if summary.may_modify_unknown() {
            state.invalidate_all_loc_memory();
        } else {
            state.invalidate_locs(&summary.modified_locs);
        }
    } else {
        state.invalidate_all_loc_memory(); // Conservative fallback
    }
}
```

#### Task A2: Compute Mod/Ref in PTABen Harness (~50 LOC)

**Files**:
- `crates/saf-bench/src/ptaben.rs`

**Implementation**:
1. After PTA analysis, call `compute_all_mod_ref(module, &pta_result)`
2. Pass `mod_ref_summaries` to nullness analysis
3. Update `analyze_nullness_with_pta_and_specs` call signature

#### Task A3: Handle Indirect Calls (~50 LOC)

**Files**:
- `crates/saf-analysis/src/absint/nullness.rs`

**Implementation**:
1. For `CallIndirect`, use PTA to resolve targets: `pta.resolve_indirect_call(func_ptr)`
2. Join mod/ref from all possible callees
3. If any target modifies unknown, use full invalidation

#### Task A4: E2E Tests (~100 LOC)

**Files**:
- `crates/saf-analysis/tests/nullness_modref_e2e.rs` (new)
- `tests/fixtures/llvm/e2e/modref_nullness.ll` (new)

**Test cases**:
1. Function that only reads memory → nullness preserved
2. Function that modifies specific location → only that location invalidated
3. Function with unknown side effects → full invalidation

---

### Phase B: Interprocedural Return Nullness (~250 LOC)

**Goal**: Track nullness of return values through function calls.

#### Task B1: Extend Function Summaries with Return Nullness (~50 LOC)

**Files**:
- `crates/saf-analysis/src/absint/interprocedural.rs`

**Implementation**:
1. Add `return_nullness: Option<Nullness>` to `FunctionSummary`
2. In `compute_function_summary()`, analyze Ret instructions to determine return nullness
3. Algorithm:
   ```rust
   let return_nullness = func.blocks.iter()
       .flat_map(|b| &b.instructions)
       .filter_map(|inst| match &inst.op {
           Operation::Ret => inst.operands.first().map(|v| state.get(*v)),
           _ => None
       })
       .reduce(|a, b| a.join(b));
   ```

#### Task B2: Wire Summaries into Nullness Transfer (~100 LOC)

**Files**:
- `crates/saf-analysis/src/absint/nullness.rs`

**Implementation**:
1. Add `summaries: Option<&BTreeMap<FunctionId, FunctionSummary>>` parameter
2. In `CallDirect` transfer, lookup callee summary for return nullness:
   ```rust
   if let Some(summary) = summaries.and_then(|s| s.get(&callee_id)) {
       if let Some(ret_nullness) = summary.return_nullness() {
           state.set(dst, *ret_nullness);
           return; // Don't fall back to MaybeNull
       }
   }
   state.set(dst, Nullness::MaybeNull);
   ```

#### Task B3: Compute Summaries in PTABen (~50 LOC)

**Files**:
- `crates/saf-bench/src/ptaben.rs`

**Implementation**:
1. Call `solve_interprocedural` (or new variant) before nullness analysis
2. Extract function summaries from result
3. Pass to `analyze_nullness_with_pta_and_specs`

#### Task B4: E2E Tests (~50 LOC)

**Files**:
- `crates/saf-analysis/tests/nullness_interprocedural_e2e.rs` (new)

**Test cases**:
1. `getNullPointer()` returns NULL → caller sees Null
2. `allocateBuffer()` returns malloc → caller sees MaybeNull (malloc can fail)
3. `xmalloc()` wrapper → caller sees NotNull (via spec)

---

### Phase C: ALLOCA Size Tracking (~400 LOC)

**Goal**: Track stack allocation sizes for buffer overflow detection.

#### Task C1: Extend AIR Operation::Alloca (~50 LOC)

**Files**:
- `crates/saf-core/src/air.rs`

**Implementation**:
```rust
// Before:
Alloca,

// After:
Alloca {
    /// Size in bytes (None if dynamic/unknown).
    /// For fixed-size allocas like `int x[10]`, this is 40.
    size_bytes: Option<u64>,
    /// The type being allocated (for reference).
    element_type: Option<String>,
},
```

**Note**: This is a breaking change to the AIR schema. All pattern matches on `Operation::Alloca` must be updated.

#### Task C2: Extract ALLOCA Size in LLVM Frontend (~150 LOC)

**Files**:
- `crates/saf-frontends/src/llvm/instruction.rs`

**Implementation**:
1. For LLVM `alloca` instruction, extract:
   - `getAllocatedType()` → element type
   - `getArraySize()` → number of elements (often ConstantInt 1)
   - Compute `size_bytes = type_size * array_size`
2. Handle dynamic allocas (VLA): set `size_bytes: None`
3. Use `DataLayout::getTypeAllocSize()` for accurate sizing

**LLVM API calls**:
```cpp
AllocaInst *AI = cast<AllocaInst>(I);
Type *AllocTy = AI->getAllocatedType();
uint64_t TypeSize = DL.getTypeAllocSize(AllocTy);
if (auto *CI = dyn_cast<ConstantInt>(AI->getArraySize())) {
    size_bytes = TypeSize * CI->getZExtValue();
}
```

#### Task C3: Use ALLOCA Size in Buffer Overflow Checker (~100 LOC)

**Files**:
- `crates/saf-analysis/src/absint/checker.rs`

**Implementation**:
1. In `check_buffer_overflow`, track alloca sizes alongside heap sizes
2. Create `AllocationInfo` struct with source (heap/stack) and size
3. When analyzing GEP, check index against both heap and stack allocation sizes

#### Task C4: Update All Operation::Alloca Pattern Matches (~50 LOC)

**Files** (grep for `Operation::Alloca`):
- `crates/saf-analysis/src/absint/nullness.rs`
- `crates/saf-analysis/src/absint/transfer.rs`
- `crates/saf-analysis/src/pta/constraint.rs`
- Various other files

**Implementation**:
Update all matches to handle new fields:
```rust
// Before:
Operation::Alloca => { ... }

// After:
Operation::Alloca { size_bytes, .. } => { ... }
```

#### Task C5: E2E Tests (~50 LOC)

**Files**:
- `crates/saf-analysis/tests/alloca_overflow_e2e.rs` (new)
- `tests/fixtures/llvm/e2e/stack_overflow.ll` (new)

**Test cases**:
1. Fixed-size stack array with overflow → detected
2. Fixed-size stack array without overflow → no warning
3. VLA (dynamic alloca) → conservative handling

---

### Phase D: Recursive Function Summary Fixpoint (~500 LOC)

**Goal**: Compute accurate summaries for recursive functions via iterative widening.

#### Task D1: Detect Recursive Functions via SCC (~100 LOC)

**Files**:
- `crates/saf-analysis/src/absint/interprocedural.rs`

**Implementation**:
1. Build call graph from module
2. Compute SCCs using Tarjan's algorithm (or reuse existing CG SCC)
3. Functions in non-trivial SCCs (size > 1 or self-recursive) need iterative analysis

```rust
pub fn find_recursive_sccs(cg: &CallGraph) -> Vec<BTreeSet<FunctionId>> {
    // Tarjan's SCC algorithm
    // Return SCCs with size > 1 OR single nodes with self-edges
}
```

#### Task D2: Iterative Summary Computation (~200 LOC)

**Files**:
- `crates/saf-analysis/src/absint/interprocedural.rs`

**Implementation**:
1. For recursive SCCs, iterate until summaries stabilize:
   ```rust
   fn compute_recursive_summaries(
       scc: &BTreeSet<FunctionId>,
       module: &AirModule,
       config: &AbstractInterpConfig,
   ) -> BTreeMap<FunctionId, FunctionSummary> {
       let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

       // Initialize with bottom summaries
       for &func_id in scc {
           summaries.insert(func_id, FunctionSummary::bottom());
       }

       // Iterate until fixpoint
       for iteration in 0..config.max_recursive_iterations {
           let mut changed = false;

           for &func_id in scc {
               let func = get_function(module, func_id);
               let new_summary = analyze_with_summaries(func, &summaries, config);

               // Widening after threshold iterations
               let widened = if iteration >= config.widening_threshold {
                   summaries[&func_id].widen(&new_summary)
               } else {
                   summaries[&func_id].join(&new_summary)
               };

               if widened != summaries[&func_id] {
                   summaries.insert(func_id, widened);
                   changed = true;
               }
           }

           if !changed {
               break;
           }
       }

       summaries
   }
   ```

#### Task D3: Summary Widening (~100 LOC)

**Files**:
- `crates/saf-analysis/src/absint/interprocedural.rs`

**Implementation**:
1. Add `widen` method to `FunctionSummary`:
   ```rust
   impl FunctionSummary {
       pub fn widen(&self, other: &Self) -> Self {
           FunctionSummary {
               return_interval: match (&self.return_interval, &other.return_interval) {
                   (Some(a), Some(b)) => Some(a.widen(b)),
                   (None, x) | (x, None) => x.clone(),
               },
               return_nullness: match (&self.return_nullness, &other.return_nullness) {
                   (Some(a), Some(b)) => Some(a.join(*b)),
                   _ => Some(Nullness::MaybeNull),
               },
               modified_locs: self.modified_locs.union(&other.modified_locs).copied().collect(),
               modifies_unknown: self.modifies_unknown || other.modifies_unknown,
           }
       }
   }
   ```

#### Task D4: Update solve_interprocedural (~50 LOC)

**Files**:
- `crates/saf-analysis/src/absint/interprocedural.rs`

**Implementation**:
1. Identify recursive SCCs before main analysis
2. Process non-recursive functions first (bottom-up by reverse topological order)
3. Process recursive SCCs with iterative fixpoint
4. Then proceed with call site refinement

#### Task D5: E2E Tests (~50 LOC)

**Files**:
- `crates/saf-analysis/tests/recursive_summary_e2e.rs` (new)

**Test cases**:
1. `id(x)` function that bounds return to [0, 2] → `svf_assert(id(x) <= 2)` passes
2. McCarthy 91 function → return value bounded
3. Simple recursive factorial → return interval widens correctly

---

## Configuration Additions

Add to `AbstractInterpConfig`:
```rust
pub struct AbstractInterpConfig {
    // ... existing fields ...

    /// Maximum iterations for recursive function summary fixpoint.
    pub max_recursive_iterations: usize,  // default: 10

    /// Number of iterations before applying widening in recursive summaries.
    pub widening_threshold: usize,  // default: 3
}
```

---

## Expected Improvements

| Category | Current Unsound | Expected After | Improvement |
|----------|-----------------|----------------|-------------|
| ae_assert_tests | 25 | ~5 | 80% reduction |
| ae_nullptr_deref_tests | 25 | ~5 | 80% reduction |
| ae_overflow_tests | 53 | ~30 | 43% reduction |
| ae_recursion_tests | 26 | ~10 | 62% reduction |
| **Total** | **129** | **~50** | **~61% reduction** |

**Notes**:
- ae_overflow_tests: Remaining issues will be string length semantics (strcat/strcpy with unknown source length)
- ae_recursion_tests: Some tests require complex invariants beyond interval widening
- ae_assert_tests: Some tests require array content tracking (not addressed in this plan)

---

## Implementation Order

1. **Phase B** (Return Nullness) - Most straightforward, builds on existing infrastructure
2. **Phase A** (Mod/Ref) - Requires threading mod/ref through nullness analysis
3. **Phase D** (Recursive Summaries) - Complex but high impact
4. **Phase C** (ALLOCA Size) - Breaking AIR change, should be done carefully

---

## Estimated LOC

| Phase | Task | LOC |
|-------|------|-----|
| A | Mod/Ref Integration | 300 |
| B | Return Nullness | 250 |
| C | ALLOCA Size | 400 |
| D | Recursive Summaries | 500 |
| **Total** | | **~1450** |

---

## Risks and Mitigations

1. **AIR Breaking Change (Phase C)**: The `Alloca` variant change requires updating all consumers.
   - *Mitigation*: Use `#[serde(default)]` for optional size_bytes to maintain backwards compatibility with existing .air.json files.

2. **Recursive Summary Non-Termination**: Widening might not stabilize for some functions.
   - *Mitigation*: Hard iteration limit with fallback to Top summary.

3. **Performance**: Mod/ref computation adds overhead.
   - *Mitigation*: Make mod/ref analysis optional, only enable for precision-critical paths.

---

## Test Verification

After implementation, run:
```bash
make test-ptaben | grep -E "ae_assert|ae_nullptr|ae_overflow|ae_recursion"
```

Expected: Unsound counts significantly reduced as detailed in Expected Improvements table.
