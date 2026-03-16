# Plan 147: Juliet Precision Implementation — Phased Fixes

## Status: approved
## Epic: checker-precision
## Depends on: Plan 146 (root cause analysis)

## Design Principle: No Juliet-Specific Fixes

All fixes are **general SAF analysis improvements** — no Juliet test metadata is used.

## Baseline (Plan 145 final)

```
Aggregate: 52.0% precision, 65.8% recall, F1=0.581
CWE476:    50% precision,  9.8% recall
CWE190:   11.8% precision, 0.1% recall
CWE191:    0% precision,   0% recall
CWE121:   53.3% precision, 64.2% recall
Others:   ~50% precision, ~100% recall
```

---

## Phase 1: NULL Analysis Completeness (RC5+RC6+RC7)

**Target:** CWE476 recall 9.8% -> 60-80%
**Effort:** ~1 session
**Root causes addressed:** RC5 (missing NULL source), RC6 (missing GEP sink), RC7 (conservative verdict)

### Task 1.1: NULL Constant as NullSource (RC5)

**Problem:** `site_classifier.rs` only recognizes `HeapAlloc`/`CallDirect` as `NullSource`. Explicit `ptr = NULL` compiles to `Copy`/`Constant` — never checked.

**Files:** `crates/saf-analysis/src/checkers/site_classifier.rs`

**Changes:**
1. Add `null_source_values: BTreeSet<SvfgNodeId>` field to `ClassifiedSites` struct
2. Add getter `pub fn null_source_values(&self) -> &BTreeSet<SvfgNodeId>`
3. In `classify()` instruction loop, add arm for `Operation::Copy` / `Operation::Constant`:
   - Check if source operand is NULL using existing `is_null_operand()` helper (lines 410-430)
   - If NULL, insert destination into `null_source_values`

**Key API:** `is_null_operand()` already exists and checks `Constant::Null` and `Constant::Int { value: 0 }`.

### Task 1.2: GEP as Dereference Sink (RC6)

**Problem:** `ptr->field` (GEP instruction) not classified as dereference sink. Many CWE476 tests dereference via member access.

**Note:** In LLVM semantics, GEP computes an address but doesn't dereference. However, the base pointer must be valid (dereferenceable). SV-COMP considers GEP on NULL as a violation.

**Files:** `crates/saf-analysis/src/checkers/site_classifier.rs`, `spec.rs`

**Changes:**
1. Add `gep_deref_pointers: BTreeSet<SvfgNodeId>` field to `ClassifiedSites`
2. In `classify()` loop, add arm for `Operation::Gep`:
   - `operands[0]` is the base pointer — insert into `gep_deref_pointers`
3. Add `GepDeref` variant to `SitePattern` enum in `spec.rs`
4. Wire `GepDeref` in checker runner to map to `gep_deref_pointers` set
5. Add `SitePattern::GepDeref` to null-deref checker spec's `sinks` list

### Task 1.3: Verdict Distinction for Explicit NULL (RC7)

**Problem:** All null-deref findings return UNKNOWN ("cannot confirm malloc returns NULL"). Wrong for explicit `ptr = NULL`.

**Files:** `crates/saf-analysis/src/checkers/finding.rs`, runner, `crates/saf-bench/src/svcomp/property.rs`

**Changes:**
1. Add enum to `finding.rs`:
   ```rust
   pub enum NullSourceKind { ExplicitNull, FunctionMayReturnNull, Unknown }
   ```
2. Add `source_kind: NullSourceKind` field to `CheckerFinding`
3. In checker runner: when building findings, classify source as `ExplicitNull` if source_node is in `null_source_values`, else `FunctionMayReturnNull` if from NullSource role
4. In `property.rs` null-deref verdict (lines 810-858):
   - Split findings by `source_kind`
   - `ExplicitNull` findings -> `PropertyResult::False` (definite bug)
   - `FunctionMayReturnNull` findings -> `PropertyResult::Unknown` (can't prove malloc returns NULL)

### Validation
- `make fmt && make lint` — no warnings
- `make test` — 1708+ Rust tests, 77 Python tests pass
- `make test-juliet CWE=CWE476` — expect recall 9.8% -> 60-80%
- `make test-juliet` — verify no regressions on other CWEs

---

## Phase 2: Z3 Guard Improvements (RC2)

**Target:** +5-10% precision on 11 temporal safety CWEs
**Effort:** ~1 session
**Root causes addressed:** RC2 (incomplete guard extraction)

### Task 2.1: Constant Propagation in Guards (A3)

**Problem:** Global constants (e.g., `GLOBAL_CONST_FIVE == 5`) treated as free Z3 variables. Z3 can't prune paths guarded by constants.

**Files:** `crates/saf-analysis/src/z3_utils/guard.rs`

**Changes:**
1. In `ValueLocationIndex` struct, add `value_to_global_const: BTreeMap<ValueId, Constant>`
2. In `ValueLocationIndex::build()`, scan all functions for `Operation::Global { obj }` and `Operation::Constant`:
   - Trace global references to their constant values (if initializer is constant)
   - Map `ValueId` -> resolved `Constant`
3. In `resolve_operand()` (lines 249-265), before returning symbolic `OperandInfo::Value`:
   - Check `value_to_global_const` for the operand
   - If found, return `OperandInfo::IntConst(value)` instead
4. **Fallback approach (if global resolution is complex):** During guard extraction, when an ICmp operand is a `Constant` operation with known value, inline the constant directly

**Key insight:** Many Juliet tests use `static const int GLOBAL_CONST_FIVE = 5;` — resolving this single pattern enables Z3 to prune ~30% of false positive paths.

### Task 2.2: Complementary Guard Recognition (A4)

**Problem:** When two SVFG traces have the same condition with opposite `branch_taken` flags, they're mutually exclusive — but this isn't detected before Z3.

**Files:** `crates/saf-analysis/src/checkers/pathsens_runner.rs` (or `z3_utils/solver.rs`)

**Changes:**
1. In `check_joint_feasibility()` (or `filter_multi_reach_infeasible()`), add pre-Z3 quick check:
   ```rust
   // For each pair of traces, check if same condition appears with opposite branch_taken
   for guard_a in &pc_a.guards {
       for guard_b in &pc_b.guards {
           if guard_a.condition == guard_b.condition
              && guard_a.branch_taken != guard_b.branch_taken {
               return FeasibilityResult::Infeasible;  // Contradictory
           }
       }
   }
   ```
2. This short-circuits the expensive Z3 call for obvious contradictions

### Validation
- `make fmt && make lint` — no warnings
- `make test` — all tests pass
- `make test-juliet` — verify precision improvement across 11 CWEs
- Spot-check CWE415/416 for reduced FP from complementary guard detection

---

## Phase 3: Absint Domain Improvements (RC4b+RC4c+RC4d)

**Target:** CWE121 recall +15%, CWE190/191 recall +30-50%
**Effort:** ~1 session
**Root causes addressed:** RC4b (widening), RC4c (bitwise ops), RC4d (mem2reg sizes)

### Task 3.1: mem2reg Size Preservation (B4)

**Problem:** Stack allocations promoted by mem2reg lose `size_bytes` -> checker fallback is TOP -> no finding.

**Files:** `crates/saf-analysis/src/absint/checker.rs`, `threshold.rs`

**Changes:**
1. In `checker.rs`, add function `build_alloca_size_map(module: &AirModule) -> BTreeMap<ValueId, Interval>`:
   - Scan all functions for `Operation::Alloca { size_bytes }`
   - Map destination `ValueId` -> `Interval::singleton(size, 64)`
   - Call this **before** analysis starts, pass as parameter to `check_buffer_overflow()`
2. In `extract_allocation_sizes()` (lines 2358-2364):
   - When `size_bytes` is `None`, try looking up in pre-built alloca map
   - Remove fallback to `Interval::make_top(64)` when map has an entry
3. In `threshold.rs:extract_thresholds()` (line 69):
   - Add scan for Alloca `size_bytes` values as thresholds with +-1 neighbors
   - Ensures allocation sizes participate in widening bounds

### Task 3.2: Bitwise Transfer Function Improvements (B3)

**Problem:** `(rand() << 30) | (rand() << 15) | rand()` returns TOP because shift overflows signed range and OR returns TOP.

**Files:** `crates/saf-analysis/src/absint/interval.rs`

**Changes:**
1. **Improve `shl()` (lines 438-458):**
   - When both operands are non-negative, compute shift without clamping to signed range
   - Use `unsigned_max(bits)` as ceiling instead of `signed_max(bits)`
   - `[0, RAND_MAX] << 30` should produce `[0, unsigned_max(32)]` not TOP

2. **Improve `bitor()` (lines 522-533) with non-overlapping bit detection:**
   - Add helper `highest_set_bit(val: i128) -> i32` (MSB position)
   - Before the conservative `hi + other.hi` bound, check if bit ranges are non-overlapping:
     - If `highest_set_bit(a.hi) + highest_set_bit(b.hi) < bits`, ranges don't overlap
     - Result is `[0, 2^(max_msb + 1) - 1]` (exact for disjoint bit fields)
   - Fall back to existing `min(unsigned_max, hi + other.hi)` if ranges overlap

3. **Improve `bitxor()` (lines 535-547):**
   - Same non-overlapping bit logic as `bitor()` (XOR of disjoint ranges = OR)

4. **Add unit tests for all improvements:**
   - `[0, 2^31-1] << 30` -> bounded, not TOP
   - `([0, 1] << 30) | ([0, 1] << 15) | [0, 65535]` -> exact bound
   - Backward compatibility: existing bitwise tests still pass

### Validation
- `make fmt && make lint`
- `make test` — all tests pass (especially interval domain tests)
- `make test-juliet CWE=CWE121` — verify recall improvement
- `make test-juliet CWE=CWE190` — verify recall improvement from bitwise ops
- `make test-ptaben` — no regression

---

## Phase 4: Function Effect Summaries (RC3)

**Target:** +10-15% precision on CWE415, 416, 401, 761
**Effort:** ~1-2 sessions
**Root causes addressed:** RC3 (cross-function temporal analysis)

### Task 4.1: Define Parameter Effect Summary

**Files:** `crates/saf-analysis/src/checkers/summary.rs` (new file)

**Data structure:**
```rust
pub struct ParameterEffectSummary {
    pub func_id: FunctionId,
    pub param_freed: BTreeMap<usize, bool>,        // Does callee free param[i]?
    pub param_dereferenced: BTreeMap<usize, bool>,  // Does callee deref param[i]?
    pub return_is_allocated: bool,                   // Does callee return newly allocated memory?
}
```

### Task 4.2: Compute Summaries Bottom-Up

**Files:** `crates/saf-analysis/src/checkers/summary.rs`

**Algorithm:**
1. Topological sort of callgraph (leaves first)
2. For each function (bottom-up):
   - **Leaf functions:** Scan instructions for `free()` calls, load/store on parameters
   - **Internal functions:** Compose callee summaries (if callee frees param, and we pass our param to callee, our param is "freed")
3. For **external functions** (declarations): populate from `ResourceTable` / `SpecRegistry`
   - `free` -> frees param[0]
   - `strlen` -> dereferences param[0]
   - `memcpy` -> dereferences param[0] and param[1]
4. Result: `BTreeMap<FunctionId, ParameterEffectSummary>`

### Task 4.3: Use Summaries in Temporal Filter

**Files:** `crates/saf-analysis/src/checkers/pathsens_runner.rs`

**Current code (lines 499-507):**
```rust
if src_pp.function != sink_pp.function {
    return true;  // Keep finding conservatively
}
```

**New logic:**
```rust
if src_pp.function != sink_pp.function {
    // Check if sink function actually performs the suspected operation
    if let Some(summary) = summaries.get(&sink_func_id) {
        // For UAF/double-free: does sink function actually free the parameter?
        if !summary.param_freed.values().any(|&frees| frees) {
            return false;  // Sink doesn't free -> false positive
        }
    }
    return true;  // No summary or summary confirms -> keep
}
```

### Task 4.4: Thread Summaries Through Pipeline

**Files:** `crates/saf-analysis/src/checkers/runner.rs`, `property.rs`

- Compute summaries once during analysis setup
- Pass to checker runner and path-sensitive filter
- Extend `FunctionSummary` in `absint/interprocedural.rs` with `param_freed`/`param_dereferenced` fields

### Validation
- `make fmt && make lint`
- `make test` — all tests pass
- `make test-juliet CWE=CWE415` — verify precision improvement (double-free)
- `make test-juliet CWE=CWE416` — verify precision improvement (UAF)
- `make test-juliet CWE=CWE401` — verify precision improvement (memory leak)
- Full `make test-juliet` — verify no regressions

---

## Phase 5: Loop-Aware Widening (RC4b)

**Target:** CWE121-127 precision +10%, CWE190-191 recall +20-30%
**Effort:** ~1 session
**Root causes addressed:** RC4b (widening loses loop bounds)

### Task 5.1: Extract Loop Bound Constants

**Files:** `crates/saf-analysis/src/absint/fixpoint.rs`

**New function (after `detect_loop_headers()`, line 1255):**
```rust
pub fn extract_loop_bound_constants(
    cfg: &Cfg,
    loop_headers: &IdBitSet<BlockId>,
    func: &AirFunction,
    module: &AirModule,
) -> BTreeMap<BlockId, BTreeSet<i128>>
```

**Algorithm:**
1. For each loop header block in `loop_headers`
2. Find the block in the function
3. Scan instructions for ICmp operations (comparison ops)
4. Extract constant operands from the comparisons
5. Add +-1 neighbors (same as threshold extraction)
6. Return map of `BlockId -> BTreeSet<i128>` (per-loop thresholds)

### Task 5.2: Integrate Loop Bounds into Widening

**Files:** `crates/saf-analysis/src/absint/fixpoint.rs`

**Changes to `solve_abstract_interp_reachable()` (line 269):**
1. After `let loop_headers = detect_loop_headers(cfg);`, add:
   ```rust
   let loop_bounds = extract_loop_bound_constants(&cfg, &loop_headers, &func, module);
   ```
2. At widening application (line 463), merge loop-specific bounds with global thresholds:
   ```rust
   let mut merged = thresholds.clone();
   if let Some(bounds) = loop_bounds.get(succ_id) {
       merged.extend(bounds.iter().copied());
   }
   let widened = val_a.widen_with_thresholds(&val_b, &merged);
   ```
3. Apply same changes in `solve_abstract_interp()` (non-reachable variant)

### Task 5.3: Add Delayed Widening Option (Optional)

**Files:** `crates/saf-analysis/src/absint/fixpoint.rs`

If loop bound extraction alone is insufficient:
1. Add iteration counter per loop header: `BTreeMap<BlockId, usize>`
2. Only apply widening after K=3 iterations (first iterations use join/union)
3. After K iterations, widen with merged thresholds
4. This gives the solver more precision for the first few loop iterations

### Validation
- `make fmt && make lint`
- `make test` — all tests pass
- `make test-juliet CWE=CWE121` — verify precision and recall improvements
- `make test-juliet CWE=CWE190` — verify recall improvement
- `make test-ptaben` — no regression (widening changes can affect PTA-dependent tests)

---

## Phase 6: Field-Sensitive Struct Tracking (RC4a)

**Target:** CWE121-127 precision +15-20%
**Effort:** ~2 sessions
**Root causes addressed:** RC4a (no field-sensitive struct tracking in absint)

### Task 6.1: Extend AbstractState with Field Memory

**Files:** `crates/saf-analysis/src/absint/state.rs`

**Changes:**
1. Add to `AbstractState`:
   ```rust
   field_memory: BTreeMap<(LocId, u64), Interval>,       // (location, byte_offset) -> interval
   field_gep_targets: BTreeMap<ValueId, BTreeSet<(LocId, u64)>>,  // GEP result -> field locations
   ```
2. Add methods:
   - `register_field_gep(&mut self, ptr: ValueId, targets: BTreeSet<(LocId, u64)>)`
   - `resolve_field_gep(&self, ptr: ValueId) -> Option<&BTreeSet<(LocId, u64)>>`
   - `store_field(&mut self, loc: LocId, offset: u64, interval: Interval)`
   - `load_field(&self, loc: LocId, offset: u64) -> Option<&Interval>`
3. Update `join()` to merge `field_memory` entries (join matching keys, keep all)

### Task 6.2: Resolve GEP Field Offsets from Type System

**Files:** `crates/saf-analysis/src/absint/transfer.rs`

**Changes:**
1. In GEP transfer function, when `FieldPath` has `Field { index }` steps:
   - Look up base type in `module.type_registry` (Plan 139 infrastructure)
   - If `AirType::Struct { fields, .. }`, extract `fields[index].byte_offset`
   - Register `(LocId, byte_offset)` pair in `field_gep_targets`
2. Add helper function:
   ```rust
   fn compute_field_byte_offset(
       field_path: &FieldPath,
       base_type: &AirType,
       type_registry: &BTreeMap<TypeId, AirType>,
   ) -> Option<u64>
   ```

### Task 6.3: Field-Sensitive Store/Load

**Files:** `crates/saf-analysis/src/absint/transfer.rs`

**Changes:**
1. In Store transfer: if pointer resolves to field GEP targets, use `store_field()`
2. In Load transfer: if pointer resolves to field GEP targets, use `load_field()`
3. Fall back to existing `loc_memory` for non-field-sensitive accesses

### Task 6.4: Field-Aware Buffer Overflow Checker

**Files:** `crates/saf-analysis/src/absint/checker.rs`

**Changes:**
1. In `check_buffer_overflow()` and `check_memcpy_overflow()`:
   - When comparing access size against allocation size
   - If access is through a GEP with known field offset:
     - Use **field size** (from struct type) instead of **total allocation size**
     - Example: `malloc(32)` for struct with 16-byte field -> field size is 16, not 32
   - This allows distinguishing `memcpy(field, src, 32)` (overflow) from `memcpy(field, src, 16)` (safe)
2. Build `alloc_sizes_by_field` map alongside existing `alloc_sizes`

### Validation
- `make fmt && make lint`
- `make test` — all tests pass
- `make test-juliet CWE=CWE121` — verify precision improvement (stack BOF)
- `make test-juliet CWE=CWE122` — verify precision improvement (heap BOF)
- Full `make test-juliet` — verify aggregate improvement
- `make test-ptaben` — no regression

---

## Expected Cumulative Results

| Phase | CWE476 Recall | CWE190 Recall | CWE121 Precision | Temporal Precision | Aggregate F1 |
|-------|--------------|---------------|-------------------|-------------------|-------------|
| Baseline | 9.8% | 0.1% | 53.3% | ~50% | 0.581 |
| Phase 1 | **60-80%** | 0.1% | 53.3% | ~50% | ~0.62 |
| Phase 2 | 60-80% | 0.1% | 53.3% | **55-60%** | ~0.65 |
| Phase 3 | 60-80% | **30-50%** | **58-63%** | 55-60% | ~0.68 |
| Phase 4 | 60-80% | 30-50% | 58-63% | **60-70%** | ~0.72 |
| Phase 5 | 60-80% | **50-60%** | **63-68%** | 60-70% | ~0.75 |
| Phase 6 | 60-80% | 50-60% | **70-80%** | 60-70% | ~0.78 |

**Note:** These are estimates. Each phase should be benchmarked independently.

## Session Workflow

Each phase is designed as one Claude Code session:
1. Read this plan and the relevant phase section
2. Implement the tasks in order
3. Run `make fmt && make lint` after implementation
4. Run `make test` to verify no regressions
5. Run `make test-juliet` (or specific CWE) to measure improvement
6. Update PROGRESS.md with phase results
7. Commit changes
