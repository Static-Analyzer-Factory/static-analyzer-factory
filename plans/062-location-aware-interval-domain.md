# Plan 062: Location-Aware Interval Domain

## Problem Statement

The current interval domain has a fundamental disconnect from PTA's location tracking:

1. **Array element tracking fails**: `getValue(arr, idx)` returning `arr[idx]` produces TOP because array pointers become TOP
2. **Phi node handling loses precision**: Branch conditions aren't considered when joining incoming values
3. **Field-sensitive intervals don't work**: GEP operations produce TOP, disconnecting struct field access from interval tracking

### Root Cause Analysis

The common root cause is that **PTA tracks locations (LocId) while intervals track values (ValueId)**:

```
PTA Layer:    Pointer → LocId (field-sensitive, array-aware)
                         ↑
                    [DISCONNECTED]
                         ↓
Interval Layer: ValueId → Interval (scalars only)
```

When a GEP computes a field/array pointer:
- PTA creates a new `LocId` for the accessed location
- The transfer function sets the result to `TOP` (pointer values not tracked)
- Subsequent loads through this pointer can't find the stored interval

## Current Architecture

### PTA Location Tracking (Works Well)
```rust
// In solver.rs - GEP creates field-sensitive locations
for gep in &self.constraints.gep {
    let resolved_path = self.resolve_gep_path(&gep.path, &gep.index_operands);
    let field_loc = self.find_or_approximate_location(base_loc.obj, &new_path);
    // field_loc is distinct for each struct field
}
```

### Interval Transfer (The Gap)
```rust
// In transfer.rs - GEP discards precision
Operation::Gep { .. } => {
    // GEP result is a pointer → top
    state.set(dst, Interval::make_top(DEFAULT_BITS));
}
```

### Memory Tracking (Partially Connected)
```rust
// In state.rs - Two parallel systems
pub struct AbstractState {
    values: BTreeMap<ValueId, Interval>,     // Scalar values
    memory: BTreeMap<ValueId, Interval>,     // ValueId-keyed memory
    loc_memory: BTreeMap<LocId, Interval>,   // LocId-keyed memory (from PTA)
}
```

## Proposed Solution: Location-Aware Interval Tracking

### Core Idea

Bridge PTA's location tracking with interval propagation by:
1. Computing target LocIds when processing GEP
2. Storing a mapping from GEP result pointers to their target LocIds
3. Using this mapping in Load/Store to work with LocId-keyed memory

### Phase 1: GEP-to-Location Mapping

**Goal**: When a GEP computes a field/array pointer, record what location it points to.

**Changes to AbstractState**:
```rust
pub struct AbstractState {
    // Existing fields...

    /// Map from GEP result ValueId to target locations.
    /// Used to resolve field-sensitive memory operations.
    gep_targets: BTreeMap<ValueId, BTreeSet<LocId>>,
}

impl AbstractState {
    /// Register a GEP result pointing to specific locations.
    pub fn register_gep(&mut self, ptr: ValueId, targets: BTreeSet<LocId>) {
        self.gep_targets.insert(ptr, targets);
    }

    /// Resolve a pointer to its GEP targets (if any).
    pub fn resolve_gep(&self, ptr: ValueId) -> Option<&BTreeSet<LocId>> {
        self.gep_targets.get(&ptr)
    }
}
```

**Changes to transfer function (transfer.rs)**:
```rust
Operation::Gep { field_path } => {
    if let Some(&base_ptr) = inst.operands.first() {
        // Get locations the base pointer can point to
        let base_locs = pta.points_to(base_ptr);

        if !base_locs.is_empty() {
            // Compute field/array element locations
            let mut target_locs = BTreeSet::new();
            for base_loc in &base_locs {
                if let Some(field_loc) = pta.compute_gep_location(base_loc, field_path) {
                    target_locs.insert(field_loc);
                }
            }

            if !target_locs.is_empty() {
                // Register this GEP result → target locations mapping
                state.register_gep(dst, target_locs);
            }
        }
    }

    // Still set pointer value to TOP (we're not tracking pointer addresses)
    state.set(dst, Interval::make_top(DEFAULT_BITS));
}
```

### Phase 2: Location-Aware Load/Store

**Goal**: When loading/storing through a GEP result, use LocId-keyed memory.

**Changes to Load handling**:
```rust
Operation::Load => {
    if let Some(&ptr) = inst.operands.first() {
        // FIRST: Check if this pointer is a GEP result with known targets
        if let Some(targets) = state.resolve_gep(ptr) {
            let mut result = Interval::make_bottom(DEFAULT_BITS);
            let mut found_any = false;

            for loc in targets {
                if let Some(interval) = state.load_loc(*loc) {
                    result = result.join(interval);
                    found_any = true;
                }
            }

            if found_any {
                state.set(dst, result);
                return; // Successfully resolved through GEP mapping
            }
        }

        // THEN: Existing PTA-based resolution (points_to, may_alias, etc.)
        // ... existing code ...
    }
}
```

**Changes to Store handling**:
```rust
Operation::Store => {
    if inst.operands.len() >= 2 {
        let value_id = inst.operands[0];
        let ptr_id = inst.operands[1];
        let value_interval = resolve_operand(value_id, state, constant_map);

        // FIRST: Check if storing through a GEP result
        if let Some(targets) = state.resolve_gep(ptr_id) {
            if targets.len() == 1 {
                // Strong update - single target
                let loc = *targets.iter().next().unwrap();
                state.store_loc(loc, value_interval.clone());
            } else {
                // Weak update - multiple targets
                for loc in targets {
                    state.store_loc_weak(*loc, value_interval.clone());
                }
            }
        }

        // Also maintain ValueId-based tracking for backwards compatibility
        state.store(ptr_id, value_interval);
    }
    return;
}
```

### Phase 3: Interprocedural GEP Resolution

**Goal**: When analyzing a callee inline, propagate GEP mappings from caller context.

**Current Issue**:
```c
int getValue(int* arr, int idx) {
    return arr[idx];  // GEP inside callee - no info about 'arr'
}

int main() {
    int local_arr[2] = {0, 1};
    getValue(local_arr, 1);  // Caller knows arr → local_arr
}
```

**Solution**: Pass caller's points-to info for arguments.

**Changes to analyze_callee_inline**:
```rust
fn analyze_callee_inline(
    callee: &AirFunction,
    arg_intervals: &[Interval],
    arg_pts: &[BTreeSet<LocId>],  // NEW: Points-to sets for pointer arguments
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
) -> Interval {
    // Initialize state with parameter bindings
    let mut state = AbstractState::new();
    for (i, param) in callee.params.iter().enumerate() {
        // Bind interval
        let interval = arg_intervals.get(i).cloned()
            .unwrap_or_else(|| Interval::make_top(64));
        state.set(param.id, interval);

        // Bind GEP targets if argument is a pointer
        if let Some(locs) = arg_pts.get(i) {
            if !locs.is_empty() {
                state.register_gep(param.id, locs.clone());
            }
        }
    }

    // ... rest of analysis ...
}
```

**At call site**:
```rust
Operation::CallDirect { callee } => {
    // Collect argument points-to sets
    let mut arg_pts: Vec<BTreeSet<LocId>> = Vec::new();
    for &arg_id in &inst.operands {
        let pts = pta.points_to(arg_id);
        arg_pts.push(pts);
    }

    // ... call analyze_callee_inline with arg_pts ...
}
```

### Phase 4: Guard-Based Phi Refinement

**Goal**: Refine Phi incoming values based on branch conditions.

**Current Issue**:
```llvm
  cond = icmp slt %x, 10
  br i1 %cond, label %then, label %else
then:
  %v1 = ...  ; Here x < 10, so %v1 might have tighter bounds
  br label %merge
else:
  %v2 = ...  ; Here x >= 10
  br label %merge
merge:
  %phi = phi [%v1, %then], [%v2, %else]  ; Currently: join(%v1, %v2)
```

**Solution**: Apply branch conditions before joining.

**Changes to Phi handling**:
```rust
Operation::Phi { incoming } => {
    let mut result = Interval::make_bottom(DEFAULT_BITS);

    for (block_id, val_id) in incoming {
        // Get the branch condition that led to this block
        if let Some(guard) = get_predecessor_guard(cfg, *block_id, current_block_id) {
            // Refine the state based on the guard
            let refined_state = refine_state_with_guard(state, guard, constant_map);
            let val = resolve_operand(*val_id, &refined_state, constant_map);
            result = result.join(&val);
        } else {
            // No guard info - use unrefined value
            let val = resolve_operand(*val_id, state, constant_map);
            result = result.join(&val);
        }
    }

    state.set(dst, result);
}
```

**Helper to extract predecessor guard**:
```rust
fn get_predecessor_guard(
    cfg: &Cfg,
    predecessor_id: BlockId,
    current_id: BlockId,
) -> Option<Guard> {
    // Find the terminator of predecessor block
    let pred_block = cfg.get_block(predecessor_id)?;
    let term = pred_block.terminator()?;

    // If CondBr, extract the condition and which branch was taken
    if let Operation::CondBr { then_target, else_target } = &term.op {
        let cond_id = term.operands.first()?;
        let take_true = *then_target == current_id;
        return Some(Guard { cond_id: *cond_id, take_true });
    }

    None
}
```

### Phase 5: Propagate Caller Memory into Callee

**Goal**: When inlining callee analysis, make caller's memory state visible.

**Current Issue**:
```c
int main() {
    int arr[2] = {0, 1};
    // Memory: arr[0] → [0], arr[1] → [1]
    getValue(arr, 1);  // Callee doesn't see this memory!
}

int getValue(int* arr, int idx) {
    return arr[idx];  // Loads from empty memory state
}
```

**Solution**: Clone relevant memory entries into callee state.

**Changes to analyze_callee_inline**:
```rust
fn analyze_callee_inline(
    callee: &AirFunction,
    arg_intervals: &[Interval],
    arg_pts: &[BTreeSet<LocId>],
    caller_loc_memory: &BTreeMap<LocId, Interval>,  // NEW: Caller's memory
    // ...
) -> Interval {
    let mut state = AbstractState::new();

    // Bind parameters
    for (i, param) in callee.params.iter().enumerate() {
        // ... existing interval/GEP binding ...

        // Propagate memory reachable through this argument
        if let Some(locs) = arg_pts.get(i) {
            for loc in locs {
                // Copy caller's memory for locations reachable from argument
                propagate_reachable_memory(loc, caller_loc_memory, &mut state);
            }
        }
    }

    // ... analysis ...
}

fn propagate_reachable_memory(
    root: LocId,
    source: &BTreeMap<LocId, Interval>,
    target: &mut AbstractState,
) {
    // Copy interval for root
    if let Some(interval) = source.get(&root) {
        target.store_loc(root, interval.clone());
    }

    // Also copy intervals for nested locations (array elements, struct fields)
    for (loc, interval) in source {
        if loc.is_subfield_of(root) {
            target.store_loc(*loc, interval.clone());
        }
    }
}
```

## Implementation Order

1. **Phase 1** (Foundation): GEP-to-Location Mapping
   - Add `gep_targets` to AbstractState
   - Modify GEP handler to compute and record target locations
   - ~50 LOC

2. **Phase 2** (Core Fix): Location-Aware Load/Store
   - Modify Load to check GEP targets first
   - Modify Store to use strong/weak updates on GEP targets
   - ~100 LOC

3. **Phase 3** (Interprocedural): Pass PTS to Inline Analysis
   - Collect argument points-to sets at call sites
   - Pass to analyze_callee_inline
   - ~50 LOC

4. **Phase 5** (Critical for Array Tests): Propagate Caller Memory
   - Clone relevant loc_memory entries into callee state
   - Enables array element tracking through function calls
   - ~80 LOC

5. **Phase 4** (Phi Improvement): Guard-Based Refinement
   - Extract branch conditions from predecessor blocks
   - Refine state before evaluating Phi incoming values
   - ~100 LOC

**Total estimated**: ~380 LOC

## Expected Results

### Array Element Tracking
```c
int getValue(int* arr, int idx) { return arr[idx]; }
int main() {
    int arr[2] = {0, 1};
    int v = getValue(arr, 1);
    svf_assert(v == 1);  // NOW PROVABLE
}
```
- Phase 3 passes `arr → {arr_loc}` to callee
- Phase 5 copies `arr[0]→[0], arr[1]→[1]` to callee state
- Phase 2 resolves `arr[1]` load to interval `[1]`
- Return propagates `[1]` back to caller

### Field-Sensitive Tracking
```c
struct { int x; int y; } a;
int *p = &a.y;
a.y = 10;
svf_assert(*p == 10);  // NOW PROVABLE
```
- Phase 1 records `p → {a.y_loc}` from GEP
- Phase 2 stores `[10]` at `a.y_loc`
- Phase 2 loads from `a.y_loc` through `p`

### Phi Node Refinement
```c
int c = (x < 10) ? x : 10;  // Phi with two paths
svf_assert(c <= 10);  // NOW PROVABLE
```
- Phase 4 refines `x` to `[-∞, 9]` on true branch
- Phase 4 refines constant `10` to `[10, 10]` on false branch
- Join produces `[-∞, 10]`, proving `c <= 10`

## Testing

After each phase, run:
```bash
make test  # All tests pass
cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter 'ae_assert_tests/*'
```

Target: Reduce ae_assert_tests Unsound from 47 to <20.

## Risks and Mitigations

1. **Performance**: More GEP processing per instruction
   - Mitigation: Cache PTA results, only process non-trivial GEPs

2. **Memory Usage**: gep_targets map grows
   - Mitigation: Clear per-function, bounded by instruction count

3. **Soundness**: GEP approximation may lose precision
   - Mitigation: Fall back to TOP when uncertain (current behavior)

4. **Complexity**: Multi-phase implementation
   - Mitigation: Each phase is independent and testable
