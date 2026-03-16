# Design: Fix basic_cpp_tests Unsound Cases

## Problem

PTABen basic_cpp_tests has 36 unsound cases after CLI wrapping (Plan 171). Before CLI wrapping, the in-process pipeline had ~30 unsound. The regression is 6 cases caused by `pick_best_alias` logic; the remaining 30 are genuine PTA precision issues.

## Investigation Findings

Debug output per-analysis results for all 36 failures:

| Pattern | Count | CI | CS | FS | PS | Expected | Root Cause |
|---------|-------|----|----|----|----|----------|-----------|
| FS correct, PS overrides | 6 | May | Unknown | **MustAlias** | NoAlias | MustAlias | pick_best_alias bug |
| All PartialAlias | 19 | Partial | Unknown | Partial | Partial | Must/No | CI dispatch conflation |
| FS MustAlias wrong for NoAlias | 5 | May | Unknown | MustAlias | MustAlias | NoAlias | CHA over-resolution |
| Everything Unknown except PS | 4 | Unknown | Unknown | Unknown | NoAlias | MustAlias | Values untracked |
| Dynamic cast opaque | 2 | Unknown | Unknown | Unknown/Must | N/A | Must/No | __dynamic_cast opaque |

CS-PTA returns Unknown for ALL basic_cpp_tests — expected behavior (oracle function parameters not in ci_summary).

## Root Causes

### 1. pick_best_alias regression (6 cases)
Old code: combine CS+FS first, only try PS as fallback when Unsound.
New code: `pick_best_alias` blindly picks highest precision (NoAlias=5 > MustAlias=4).
FS says MustAlias (correct), PS says NoAlias (wrong), pick_best picks NoAlias.

### 2. Missing library function summaries (4 cases)
`getenv`, `__errno_location`, `gmtime`, `localtime` are opaque externals.
Each call site gets a fresh abstract allocation. Two calls to the same
deterministic function should return the same static pointer.

### 3. `__dynamic_cast` opaque (2 cases)
Returns fresh allocation instead of aliasing input pointer.

### 4. CHA over-resolution (10+ cases)
CHA resolves vtable slot k to ALL classes with a function at that slot.
Only B is allocated but A::f is added as callee → parameter pollution.
Tests: single-inheritance-3/4, abstract, member-variable, diamond patterns.

### 5. Diamond/virtual inheritance (14 cases)
Dynamic vbase offsets, constructor vtable pollution, context-insensitive
parameter merging. Requires deep CS analysis — deferred.

## Fix Design

### Tier 1: Validation Logic Fix (6 cases)
In `ptaben.rs:validate_expectation_from_bench_result`, replace blind `entry.best`
usage with old combine+fallback strategy:
1. Combine CS+FS using `combine_alias_results` (Unknown defers)
2. Fall back to CI if combined is Unknown
3. Check if result is correct (not Unsound)
4. Only try PS as fallback when Unsound; PS can only help

### Tier 2: Library Function Summaries (4 cases)
Add specs to `SpecRegistry::load()` for singleton-returning functions:
- `getenv` → returns pointer to static env string
- `__errno_location` → returns pointer to static errno
- `gmtime` → returns pointer to static struct tm
- `localtime` → returns pointer to static struct tm
Implementation: new `ReturnBehavior::StaticSingleton` in spec system.

### Tier 3: `__dynamic_cast` Summary (2 cases)
Add spec for `__dynamic_cast`: return value aliases first argument (or null).
Implementation: `ReturnBehavior::AliasesArg(0)` in spec system.

### Tier 4: VTable-Aware CG Filtering (10+ cases)
Improve CG refinement to filter indirect call targets by vtable reachability:
- When resolving `CallIndirect` through vtable GEP pattern, check which
  vtable globals flow to the object's vtable pointer field via PTA
- Only add callees whose function pointer is present in reachable vtables
- This prevents CHA from adding A::f when only B's vtable is reachable
Implementation: New `vtable_filter_targets()` in cg_refinement.rs.

## Expected Outcomes
- Tier 1: 36 → 30 unsound (6 fixed)
- Tier 2: 30 → 26 unsound (4 fixed)
- Tier 3: 26 → 24 unsound (2 fixed)
- Tier 4: 24 → ~14 unsound (10+ fixed)
- Total: 36 → ~14 unsound
