# Plan 077: Memory Leak Checker — PTABen Precision Fixes

## Status: Done (partial: 67→50 mem_leak unsound, 10→3 double_free unsound)

## Problem Statement

PTABen `mem_leak` category: **67 unsound** (76 Exact, 3 Sound, 67 Unsound, 3 Skip) out of 149 oracle checks across 92 test files.

- **38 false negatives** — Leak expected (NeverFree/PartialLeak/ContextLeak) but checker reports nothing
- **29 false positives** — Safe allocation but checker reports leak(s)

## Root Causes

1. **Dead fallback in `must_not_reach` solver** (FN cause): When BFS from source finds no exit nodes AND no sanitizers, the code block was empty (no finding generated). NeverFree tests with `ret void` or `ret 0` hit this case.

2. **Wrapper-internal `malloc` as spurious source** (FP cause): `SAFEMALLOC(n) { return malloc(n); }` creates two Allocator sources — internal HeapAlloc(malloc) and external CallDirect(SAFEMALLOC). Internal malloc generates spurious findings.

3. **Module-level counting in validation** (FP amplifier): `validate_mem_leak()` counted ALL memory-leak findings in the module, causing cross-talk between different oracle wrappers.

## Implementation

### Phase 1: Filter wrapper-internal HeapAlloc sources

**Files**: `site_classifier.rs`, `runner.rs`

- Added `site_for_return_node()` to `ClassifiedSites`
- Added `filter_wrapper_internal_sources()` in runner — removes HeapAlloc sources inside allocator wrappers
- Applied only for `MustNotReach` mode

### Phase 2: Fix dead fallback in must_not_reach solver

**File**: `solver.rs`

- Replaced empty block with finding generation when no sanitizer reachable
- Self-referential finding: `source_node == sink_node`
- Condition: `sanitizer_nodes.is_empty() || !visited.iter().any(|n| sanitizer_nodes.contains(n))`

### Phase 3: Per-allocation validation

**File**: `ptaben.rs`

- Replaced module-level `leak_finding_count` with `SvfgNodeId::Value(alloc_site)` matching
- Each oracle checked against findings for its specific allocation site

## Results

### mem_leak category

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Exact | 76 | 93 | +17 |
| Sound | 3 | 3 | 0 |
| Unsound | 67 | 50 | -17 |
| Skip | 3 | 3 | 0 |

### double_free category (bonus from Phase 4)

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Exact | 168 | 175 | +7 |
| Unsound | 10 | 3 | -7 |

### Overall PTABen

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Exact | 1886 | 1900 | +14 |
| Unsound | 333 | 321 | -12 |

### Phase 4 impact

Stripping `optnone` for mem_leak/double_free allows `mem2reg` to produce clean SSA.
Global application was tested first but regressed path_tests (+26), basic_cpp_tests (+18),
ae_recursion_tests (+13) — those analyses are calibrated for alloca patterns. Targeted
application: zero regressions in other categories.

## Remaining 50 mem_leak Unsound

- **30 Safe FPs**: SVFG still incomplete for interprocedural flows through wrapper functions
- **19 PartialLeak FNs**: Conditional free on some paths, BFS reaches exit without sanitizer on other paths
- **1 NeverFree FN**: In a mixed module where SAFEFREE creates sanitizer nodes but NFRMALLOC's BFS doesn't reach them

## Files Modified

| File | Change | LOC |
|------|--------|-----|
| `crates/saf-analysis/src/checkers/solver.rs` | Fix dead fallback + update test | ~15 |
| `crates/saf-analysis/src/checkers/runner.rs` | Filter wrapper-internal sources | ~30 |
| `crates/saf-analysis/src/checkers/site_classifier.rs` | Add `site_for_return_node()` | ~8 |
| `crates/saf-bench/src/ptaben.rs` | Per-allocation validation | ~30 |
| `scripts/compile-ptaben.sh` | Strip `optnone` for mem_leak/double_free | ~5 |

**Total**: ~88 LOC
