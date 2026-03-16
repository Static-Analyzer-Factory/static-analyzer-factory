# Plan 145: Juliet Precision Fixes — Root Cause Analysis & Phased Remediation

## Status: done (all 5 phases complete)
## Epic: checker-precision

## Design Principle: No Juliet-Specific Fixes

All fixes in this plan are **general SAF analysis improvements** that would benefit any
benchmark or real-world program. None use Juliet test metadata (CWE type, subproperty,
file naming convention) to improve results. In a competition setting, the analyzer is told
**which property** to check (valid-memsafety, no-overflow) but NOT which specific vulnerability
type the test contains. Our fixes must work without that knowledge.

Specifically:
- Scoping analysis to reachable code is standard practice (any dead code should be ignored)
- Adding C standard library function specs is standard (SVF, Infer, Coverity all do this)
- Improving abstract interpretation precision benefits all programs
- Fixing overly conservative guards is a bug fix, not a heuristic

## Background

Juliet benchmark (Plan 144) baseline results show poor precision across 15 CWE categories:
- **13 memsafety CWEs**: ~50% precision, 100% recall (all tests flagged FALSE)
- **2 integer CWEs**: 0% precision, 0% recall (all tests return UNKNOWN)
- **Aggregate**: 51.1% precision, 75.1% recall, F1=0.608

Agent-team investigation identified **5 root causes** (L1-L5), all core SAF analysis limitations (not Juliet-specific).

## Root Causes

### L1: Shared Harness Code Pollution
Every Juliet `.i` file embeds ~700 lines of SV-COMP harness (stdThread*, LDV wrappers).
The harness contains actual bugs (e.g., `stdThreadLockCreate` does `free(lock)` instead of
`free(my_lock)`). `analyze_memsafety()` analyzes the **entire compilation unit** including
unreachable harness functions, producing findings from harness code that fire on both good
and bad variants identically.

**Key code**: `has_heap_allocations()` in `fast_paths.rs:156-187` sees harness `malloc`
declarations, so every test takes the full analysis path. SVFG/PTA/absint all analyze
harness functions.

### L2: Composite Memsafety Verdict
`analyze_memsafety()` (`property.rs:586-850`) returns FALSE for **any** finding from **any**
checker type. A CWE476 test gets FALSE because harness triggers UAF, not because of null-deref.
CWE401 tests get FALSE even though leak detection is in `analyze_memcleanup()` (never called
for `valid-memsafety` property). Buffer overflow threshold (`property.rs:748`) of <=6 Warning
findings in aggressive mode is easily met by harness warnings.

### L3: Path-Insensitive Abstract Interpretation
Buffer overflow checker (`absint/checker.rs:148`) doesn't track branch conditions.
`if (i < bufSize) buf[i] = 'A'` still reports warning because `i` interval is
`[0, INT_MAX]` — the guard condition isn't modeled. Same warning for good and bad variants.

### L4: Integer Overflow — External Inputs are TOP
`check_integer_overflow()` (`checker.rs:289`) uses basic intraprocedural `solve_abstract_interp`
(no PTA, no specs, no interprocedural summaries). External function calls (`atoi`, `rand`,
`fscanf`) return TOP. Checker at `checker.rs:323-325` skips any operation where operand is TOP.
The spec registry already has specs for `rand` etc. but they're not wired up.

### L5: Integer Overflow — Module-Wide Loop-Free Guard
`analyze_no_overflow()` (`property.rs:880`) requires `is_loop_free && !conservative` for FALSE
verdict. `program_is_loop_free()` (`fast_paths.rs:125-127`) checks **ALL** functions in module
(including `printBytesLine`, `decodeHexChars` which have loops). So even when overflow IS
detected with Error severity in a constant-source test, the verdict falls through to UNKNOWN.

## Phased Implementation

### Phase 1: Scope Analysis to Main-Reachable Functions (L1 + L5)

**Goal**: Eliminate harness code pollution by only analyzing functions reachable from `main()`.

**Impact**: ~18,583 memsafety tests (harness findings eliminated) + ~684 integer tests (loop-free
check scoped to relevant functions).

**Changes**:

1. **Add `reachable_functions()` helper** to `crates/saf-bench/src/svcomp/fast_paths.rs`:
   - BFS/DFS from `main()` through call graph to collect reachable `FunctionId` set
   - Used by both memsafety and overflow analyzers

2. **Filter SVFG checker scope in `analyze_memsafety()`** (`property.rs:586`):
   - After building SVFG, filter findings to only those originating from reachable functions
   - Or: build SVFG only from reachable functions (more efficient but larger change)

3. **Filter buffer overflow checker scope** (`property.rs:706-821`):
   - Run `check_buffer_overflow()` only on reachable functions
   - Or: post-filter findings by function origin

4. **Scope `program_is_loop_free()` in `analyze_no_overflow()`** (`property.rs:862`):
   - Change to `reachable_is_loop_free()` — only check functions reachable from main
   - This unblocks Error-severity integer overflow findings in constant-source tests

5. **Scope `has_heap_allocations()` to reachable functions** (`fast_paths.rs:156`):
   - Only check if reachable functions use heap
   - Stack-only test programs can then use the fast path (null-deref only)

**Validation**: Run `make test-juliet CWE=CWE476` — expect significant reduction in FP
(good variants should no longer trigger harness UAF findings). Run `make test-juliet CWE=CWE190`
— expect ~684 `int_max`/`int_min` tests to now get FALSE verdicts.

### Phase 2: Wire Spec-Aware Absint for Integer Overflow (L4)

**Goal**: Give integer overflow checker non-TOP intervals for external function calls.

**Impact**: ~5,548 CWE190/CWE191 tests with external input sources.

**Changes**:

1. **Change `AnalysisContext::absint()` to use spec-aware solver** (`property.rs:239-243`):
   - Replace `check_integer_overflow(module, &ai_config)` with a version that uses
     `solve_abstract_interp_with_specs` or `solve_interprocedural`
   - The spec registry at `transfer.rs:497-516` already has interval specs for `rand`,
     `abs`, `labs` etc.

2. **Add specs for common Juliet input functions** (if not already present):
   - `atoi` → `[INT_MIN, INT_MAX]` (full range but not TOP — checker won't skip)
   - `fscanf` → model the scanned variable as `[INT_MIN, INT_MAX]`
   - `recv`/`connect`/`listen` → model returned data as `[INT_MIN, INT_MAX]`
   - These are general C library specs, not Juliet-specific

3. **Adjust checker TOP-skip logic** (`checker.rs:323-325`):
   - Distinguish between "truly unknown" (TOP) and "full range" (`[INT_MIN, INT_MAX]`)
   - Full range + addition → definite overflow if result exceeds range
   - This may require a separate `is_external_input` flag or using a sentinel interval

**Validation**: Run `make test-juliet CWE=CWE190` — expect detection of overflows in
external-input tests (at least Warning severity). Check PTABen for no regressions.

### Phase 3: Function-Scoped Loop-Free Check (L5 refinement)

**Goal**: Allow definite integer overflow findings to produce FALSE even when unrelated functions
have loops.

**Impact**: Reinforces Phase 1 fix; catches any remaining cases.

**Changes**:

1. **Change `analyze_no_overflow()` to use per-finding loop-free check** (`property.rs:880`):
   - For each Error-severity finding, check if the function containing the finding is loop-free
   - If the finding function is loop-free AND aggressive mode → FALSE
   - Don't require the entire module to be loop-free

2. **Alternative**: For constant-operand overflows (both operands are concrete intervals,
   not widened), emit FALSE regardless of loop status since no widening was involved.

**Validation**: Run `make test-juliet CWE=CWE190` with `int_max` filter — expect FALSE verdicts.

### Phase 4: Path-Condition Narrowing in Abstract Interpreter (L3)

**Goal**: Track branch conditions to narrow intervals inside then/else blocks.

**Impact**: ~13,000 buffer overflow false positives across CWE121/122/124/126/127.

**Changes**:

1. **Add guard tracking to abstract interpreter state**:
   - At conditional branches (`CondBr`), extract the condition (e.g., `icmp slt %i, %n`)
   - On the true-branch, narrow the interval: `i` → `[lo, n-1]`
   - On the false-branch: `i` → `[n, hi]`

2. **Propagate narrowed intervals through GEP checks**:
   - Buffer overflow checker sees narrowed `i ∈ [0, bufSize-1]` inside bounds-checked code
   - No warning generated for guarded accesses

3. **Handle common patterns**:
   - `if (p != NULL) *p` → p is non-null in then-block (helps null-deref too)
   - `if (i >= 0 && i < size) arr[i]` → i in `[0, size-1]`

**Note**: This is a significant analysis improvement with broad impact beyond Juliet.
Consider as a separate epic if scope grows.

**Validation**: Run `make test-juliet CWE=CWE121` — expect good variants with bounds checks
to no longer produce buffer overflow warnings.

### Phase 5: Confidence-Based Verdict Thresholds (L2 refinement)

**Goal**: Reduce false positives from the composite memsafety checker by requiring higher
confidence before emitting FALSE, without using any test-specific metadata.

**Impact**: Reduces cross-checker false positives across all memsafety programs.

**Note**: This phase does NOT use Juliet metadata (CWE type, subproperty, file naming).
It improves the general-purpose verdict logic that applies to any `valid-memsafety` check.

**Changes**:

1. **Tighten buffer overflow aggressive threshold** (`property.rs:748`):
   - Current threshold: <=6 Warning findings → FALSE
   - This fires on programs where harness warnings are the only findings
   - Require at least 1 Error-severity finding before promoting Warnings to FALSE
   - Or: require Warning findings to be in reachable functions (reinforces Phase 1)

2. **Add finding deduplication by root cause**:
   - Multiple findings from the same allocation/free site should count as one
   - Currently the same harness function can generate multiple findings that individually
     look significant but all stem from one root cause
   - Deduplicate by (function, checker_name, source_node) before counting

3. **Require cross-checker corroboration for aggressive FALSE**:
   - Instead of any single finding triggering FALSE, require either:
     - An Error-severity finding (high confidence), OR
     - Multiple independent findings from different checker types
   - This prevents a single low-confidence warning from flipping the verdict

4. **Improve UNKNOWN-to-TRUE promotion for clean programs**:
   - When checkers run successfully with zero findings and zero unknowns, return TRUE
   - Currently, having any infeasible findings (Z3-proven safe) still returns UNKNOWN
     (`property.rs:825-834`). This is overly conservative — if Z3 proved all findings
     infeasible, the program is actually verified safe for the checked properties

**Validation**: Run full `make test-juliet` and `make test-ptaben` — expect improved
precision without regression in PTABen scores. Run SV-COMP benchmarks for no regression.

## Expected Results by Phase

| Phase | CWE190/191 Recall | Memsafety Precision | Key Metric |
|-------|-------------------|---------------------|------------|
| Baseline | 0% | ~50% | F1=0.608 |
| Phase 1 | ~11% (684 const tests) | 60-80% (harness FPs removed) | Major FP reduction |
| Phase 2 | 30-60% (ext input tests) | unchanged | Integer detection enabled |
| Phase 3 | 40-70% (refinement) | unchanged | Reinforces Phase 1 |
| Phase 4 | unchanged | 80-95% (bounds-checked code) | Fundamental precision |
| Phase 5 | unchanged | 85-95% (tighter thresholds) | Verdict quality |

## Dependencies

- Phase 1 is independent (highest priority, biggest impact)
- Phase 2 is independent (can run in parallel with Phase 1)
- Phase 3 depends on Phase 1 (scoped loop-free check)
- Phase 4 is independent but large (can be a separate epic)
- Phase 5 depends on Phase 1 (building on scoped findings)

## Files Modified (by phase)

**Phase 1**:
- `crates/saf-bench/src/svcomp/fast_paths.rs` — add `reachable_functions()`, scope `has_heap_allocations`, `program_is_loop_free`
- `crates/saf-bench/src/svcomp/property.rs` — scope `analyze_memsafety` and `analyze_no_overflow` to reachable functions

**Phase 2**:
- `crates/saf-bench/src/svcomp/property.rs` — change `AnalysisContext::absint()` to use spec-aware solver
- `crates/saf-analysis/src/absint/checker.rs` — adjust TOP-skip logic
- `crates/saf-analysis/src/absint/specs/` — add C library specs (if needed)

**Phase 3**:
- `crates/saf-bench/src/svcomp/property.rs` — per-finding loop-free check in `analyze_no_overflow`

**Phase 4**:
- `crates/saf-analysis/src/absint/transfer.rs` — guard extraction at CondBr
- `crates/saf-analysis/src/absint/state.rs` — interval narrowing
- `crates/saf-analysis/src/absint/checker.rs` — use narrowed intervals

**Phase 5**:
- `crates/saf-bench/src/svcomp/property.rs` — tighten threshold logic, add deduplication, improve UNKNOWN→TRUE promotion
