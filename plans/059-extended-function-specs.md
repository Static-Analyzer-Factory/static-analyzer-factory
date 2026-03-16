# Plan 059: Extended Function Specs Integration

**Epic**: E34 (Function Specs Extended)
**Status**: approved
**Depends on**: Plan 058 (complete)

## Overview

Extend the function specs system (Plan 058) to fully integrate spec-defined behavior across all SAF analyses. This plan replaces hardcoded function name lists with spec lookups and adds new analysis capabilities using spec fields that were designed but not yet consumed.

## Architecture Principle

**Spec-first, fallback-second**: Each analysis will:
1. Check `SpecRegistry.lookup(name)` for the function
2. Extract the relevant field(s) from the spec
3. Only fall back to hardcoded logic if no spec exists

## Phases

### Phase A: Replace Hardcoded Lists (~150 LOC)

**Goal**: Eliminate hardcoded function name matching in favor of spec lookups.

#### Task A1: `returns.aliases` Integration (~50 LOC)

**Current state**: `nullness.rs:1037` has `returns_first_argument()` with 23 hardcoded functions.

**Files**:
- `crates/saf-analysis/src/absint/nullness.rs`

**Tasks**:
1. Create `returns_first_argument_from_spec()` that checks `spec.returns.aliases`
2. Create `returns_first_argument_with_specs()` wrapper that tries spec first, then fallback
3. Update all callers to pass `Option<&SpecRegistry>`
4. Unit test: spec lookup returns correct value, fallback works when no spec

**Spec coverage**: Already defined in `libc/string.yaml` for memcpy, strcpy, strcat, etc.

#### Task A2: `returns.nullness: not_null` Integration (~50 LOC)

**Current state**: `nullness.rs:1072` has `returns_nonnull()` with 11 hardcoded functions.

**Files**:
- `crates/saf-analysis/src/absint/nullness.rs`
- `share/saf/specs/gnu/xalloc.yaml` (new)

**Tasks**:
1. Create `returns_nonnull_from_spec()` that checks `spec.returns.nullness == NotNull`
2. Create `returns_nonnull_with_specs()` wrapper
3. Update callers to pass specs
4. Create `gnu/xalloc.yaml` with specs for xmalloc, xcalloc, xrealloc, xstrdup

**New specs**:
```yaml
# share/saf/specs/gnu/xalloc.yaml
version: "1.0"
specs:
  - name: xmalloc
    role: allocator
    returns:
      pointer: fresh_heap
      nullness: not_null  # aborts on failure
    params:
      - index: 0
        semantic: allocation_size

  - name: xcalloc
    role: allocator
    returns:
      pointer: fresh_heap
      nullness: not_null
    params:
      - index: 0
        semantic: element_count
      - index: 1
        semantic: element_size

  - name: xrealloc
    role: reallocator
    returns:
      pointer: fresh_heap
      nullness: not_null
    params:
      - index: 0
        semantic: old_pointer
      - index: 1
        semantic: new_size

  - name: xstrdup
    role: allocator
    returns:
      pointer: fresh_heap
      nullness: not_null
    params:
      - index: 0
        reads: true
        nullness: required_nonnull
```

#### Task A3: `noreturn` CFG Integration (~50 LOC)

**Current state**: `is_noreturn_from_spec()` exists in nullness.rs but isn't used in CFG building.

**Files**:
- `crates/saf-analysis/src/cfg.rs` (or appropriate location)
- `crates/saf-analysis/src/absint/nullness.rs` (move helper to shared location)

**Tasks**:
1. Create `is_noreturn_with_specs()` that checks spec first, then hardcoded fallback
2. Export from a common location (e.g., `spec_helpers.rs`)
3. Use in CFG terminator analysis to identify noreturn calls
4. Unit test: spec lookup, fallback, unknown function returns false

---

### Phase B: Integrate Existing Spec Fields (~400 LOC)

**Goal**: Wire up spec fields that exist in schema/YAML but aren't consumed by analyses.

#### Task B1: Interval Bounds for External Functions (~100 LOC)

**Current state**: External function calls return `Interval::top()` in abstract interpretation.

**Spec field**: `returns.interval: [min, max]`

**Files**:
- `crates/saf-analysis/src/absint/interprocedural.rs`
- `share/saf/specs/libc/stdlib.yaml` (update)
- `share/saf/specs/libc/string.yaml` (update)

**Tasks**:
1. Add `compute_external_summary()` that extracts interval from spec
2. In `compute_summaries()`, use spec-based summary for declarations
3. Add `returns.interval` to specs for strlen, abs, atoi, etc.
4. Integration test: call to strlen returns [0, SIZE_MAX]

**New specs additions**:
```yaml
# In libc/string.yaml
- name: strlen
  pure: true
  returns:
    interval: [0, 9223372036854775807]  # [0, SIZE_MAX]
  params:
    - index: 0
      reads: true
      nullness: required_nonnull

# In libc/stdlib.yaml
- name: abs
  pure: true
  returns:
    interval: [0, 2147483647]  # [0, INT_MAX]

- name: labs
  pure: true
  returns:
    interval: [0, 9223372036854775807]  # [0, LONG_MAX]
```

#### Task B2: Mod/Ref from Specs (~100 LOC)

**Current state**: `mod_ref.rs:84` returns `ModRefSummary::conservative()` for all externals.

**Spec fields**: `params[*].modifies`, `params[*].reads`

**Files**:
- `crates/saf-analysis/src/pta/mod_ref.rs`

**Tasks**:
1. Add `compute_all_mod_ref_with_specs()` that accepts `Option<&SpecRegistry>`
2. Create `summary_from_spec()` that builds summary from param modifies/reads
3. For declarations: use spec summary if available, else conservative
4. Update callers to pass specs
5. Unit test: strlen spec → pure summary, memcpy spec → modifies param.0

**Logic**:
- If spec has `pure: true` → empty mod/ref (pure)
- If any param has `modifies: true` → not pure, add to mod set
- If no params have `modifies` and not marked pure → conservative (unknown)

#### Task B3: Bounds Checking with `semantic` and `size_from` (~100 LOC)

**Current state**: Buffer overflow checker doesn't use spec size relationships.

**Spec fields**: `params[*].semantic`, `params[*].size_from`

**Files**:
- `crates/saf-analysis/src/absint/checker.rs`

**Tasks**:
1. Create `get_copy_info_from_spec()` → `Option<(dest_idx, size_idx)>`
2. Enhance `check_buffer_overflow_with_specs()` to use spec semantics
3. For CallDirect: extract dest/size params from spec, validate bounds
4. Integration test: strncpy with size > buffer triggers warning

**Semantic values recognized**:
- `byte_count` — size in bytes
- `element_count` — number of elements
- `max_length` — maximum length (for strn* functions)
- `allocation_size` — allocation size (for malloc-like)

#### Task B4: Pure Function Unification (~50 LOC)

**Current state**: `is_pure_function_with_specs()` only checks `pure: true`.

**Files**:
- `crates/saf-analysis/src/absint/transfer.rs`

**Tasks**:
1. Enhance to infer purity from absence of `modifies` + absence of `escapes`
2. Add check: if no params have `modifies: true` and none have `escapes: true`, infer pure
3. Exclude allocators (they have side effects even without modifies)
4. Unit test: function with only `reads: true` params inferred as pure

---

### Phase C: New Analysis Capabilities (~600 LOC)

**Goal**: Add new analysis modules using spec fields designed for future use.

#### Task C1: Escape Analysis Module (~300 LOC)

**Purpose**: Track which pointers escape function scope.

**Spec field**: `params[*].escapes`

**Files**:
- `crates/saf-analysis/src/absint/escape.rs` (new)
- `crates/saf-analysis/src/absint/mod.rs` (add module)
- `crates/saf-analysis/src/lib.rs` (re-export)
- `share/saf/specs/libc/string.yaml` (add escapes: false)

**Tasks**:
1. Define `EscapeState` enum: `NoEscape`, `ReturnEscape`, `GlobalEscape`
2. Define `EscapeResult` struct with per-value escape states
3. Implement `analyze_escape()` that:
   - Marks returned values as `ReturnEscape`
   - Marks values stored to globals/heap as `GlobalEscape`
   - For calls: checks spec `escapes` field for each argument
4. Add `escapes: false` to read-only specs (strlen, strcmp, memcmp)
5. Unit tests for each escape category
6. Integration test: local pointer passed to strlen doesn't escape

**New specs additions**:
```yaml
# In libc/string.yaml - add escapes: false to read-only params
- name: strlen
  params:
    - index: 0
      reads: true
      escapes: false
      nullness: required_nonnull

- name: strcmp
  params:
    - index: 0
      reads: true
      escapes: false
    - index: 1
      reads: true
      escapes: false

- name: memcmp
  params:
    - index: 0
      reads: true
      escapes: false
    - index: 1
      reads: true
      escapes: false
```

#### Task C2: Callback Call Graph Edges (~200 LOC)

**Purpose**: Add call graph edges for function pointer parameters.

**Spec field**: `params[*].callback`

**Files**:
- `crates/saf-analysis/src/callgraph.rs`
- `share/saf/specs/libc/stdlib.yaml` (add callback)
- `share/saf/specs/posix/pthread.yaml` (new)

**Tasks**:
1. Add `CallGraph::add_callback_edges()` method
2. For each CallDirect, check spec for callback params
3. Use PTA to resolve callback argument to function targets
4. Add edges from caller to resolved callback targets
5. Create `posix/pthread.yaml` with pthread_create callback spec
6. Add callback specs to qsort, bsearch, atexit
7. Integration test: qsort call adds edge to comparator function

**New specs**:
```yaml
# In libc/stdlib.yaml
- name: qsort
  params:
    - index: 0
      modifies: true
    - index: 3
      callback: true

- name: bsearch
  pure: true
  params:
    - index: 4
      callback: true

- name: atexit
  params:
    - index: 0
      callback: true

# share/saf/specs/posix/pthread.yaml (new file)
version: "1.0"
specs:
  - name: pthread_create
    params:
      - index: 2
        callback: true
      - index: 3
        escapes: true

  - name: pthread_exit
    noreturn: true

  - name: pthread_join
    params:
      - index: 1
        modifies: true
```

#### Task C3: Noreturn Successor Pruning (~100 LOC)

**Purpose**: Mark successors of noreturn calls as unreachable.

**Files**:
- `crates/saf-analysis/src/icfg.rs` (or reachability module)
- `crates/saf-analysis/src/absint/fixpoint.rs` (skip unreachable blocks)

**Tasks**:
1. Create `block_ends_noreturn()` that scans for noreturn calls
2. In ICFG/reachability: mark successor blocks as unreachable
3. In fixpoint: skip unreachable blocks (don't process)
4. In path-sensitive analysis: prune paths through noreturn
5. Integration test: code after exit() not analyzed, no spurious warnings

---

## Testing Strategy

### Unit Tests
- Each task: test spec lookup, fallback behavior, edge cases
- Escape analysis: test each lattice value transition
- Callback edges: test edge creation with/without PTA

### Integration Tests
- Create `tests/fixtures/llvm/e2e/spec_integration.ll`:
  - `strcpy` (returns first arg)
  - `strlen` (pure, interval)
  - `exit` (noreturn)
  - `qsort` with callback
- Compile from `tests/programs/c/spec_integration.c`

### PTABen Validation
- Run after each phase, ensure no regressions
- Track improvements from escape/mod-ref precision

## Rollout Order

```
Phase A: A1 → A2 → A3
Phase B: B2 → B1 → B4 → B3
Phase C: C3 → C1 → C2
```

**Rationale**:
- A1 before A2: `returns.aliases` more commonly used
- B2 before B1: Mod/ref enables better interprocedural intervals
- C3 before C1: Noreturn simpler, immediately useful
- C2 last: Callbacks depend on PTA quality

## Files Summary

**Modified (9 files)**:
- `crates/saf-analysis/src/absint/nullness.rs`
- `crates/saf-analysis/src/absint/transfer.rs`
- `crates/saf-analysis/src/absint/interprocedural.rs`
- `crates/saf-analysis/src/absint/fixpoint.rs`
- `crates/saf-analysis/src/pta/mod_ref.rs`
- `crates/saf-analysis/src/absint/checker.rs`
- `crates/saf-analysis/src/callgraph.rs`
- `crates/saf-analysis/src/icfg.rs`
- `crates/saf-analysis/src/lib.rs`

**Created (3 files)**:
- `crates/saf-analysis/src/absint/escape.rs`
- `share/saf/specs/gnu/xalloc.yaml`
- `share/saf/specs/posix/pthread.yaml`

**Specs Updated (2 files)**:
- `share/saf/specs/libc/stdlib.yaml`
- `share/saf/specs/libc/string.yaml`

## Estimated LOC

| Phase | Task | LOC |
|-------|------|-----|
| A | A1: returns.aliases | 50 |
| A | A2: returns.nullness | 50 |
| A | A3: noreturn CFG | 50 |
| B | B1: returns.interval | 100 |
| B | B2: mod/ref from specs | 100 |
| B | B3: bounds checking | 100 |
| B | B4: pure unification | 50 |
| C | C1: escape analysis | 300 |
| C | C2: callback edges | 200 |
| C | C3: noreturn pruning | 100 |
| | **Total** | **~1100** |

## Success Criteria

1. All 1573 Rust + 248 Python tests pass after each task
2. PTABen: no decrease in Exact/Sound counts
3. 80%+ of hardcoded function names have equivalent specs
4. New escape analysis produces meaningful results on test programs
5. Callback edges appear in call graph for qsort/pthread_create tests
