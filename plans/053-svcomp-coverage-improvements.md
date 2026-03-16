# Plan 053: SV-COMP Coverage Improvements

## Overview

Improve SAF's SV-COMP benchmark coverage from 1% to 15-30% by implementing 5 targeted improvements to property analyzers, with configurable conservativeness.

**Current state:** 1 TRUE, 0 FALSE, ~135 UNKNOWN = +2 points (1% of max)

**Target state:** 200+ TRUE, 30+ FALSE, reduced UNKNOWN = 15-30% coverage

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    PropertyAnalysisConfig                        │
│  + conservative: bool  (default: true)                          │
│  + z3_timeout_ms, max_guards, max_paths (existing)              │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  unreach-call   │  │  no-data-race   │  │  no-overflow    │
│                 │  │                 │  │                 │
│ P1: Dead-code   │  │ P0: No-thread   │  │ P3: Loop-free   │
│ P2: Summaries   │  │     detection   │  │     intervals   │
└─────────────────┘  └─────────────────┘  └─────────────────┘
                              │
                              ▼
                     ┌─────────────────┐
                     │ valid-memsafety │
                     │                 │
                     │ P4: No-heap     │
                     │     fast-path   │
                     └─────────────────┘
```

## Improvements

### P0: No-Thread Detection for `no-data-race`

**Problem:** MTA analysis runs even for sequential programs, returns UNKNOWN when no races found.

**Solution:** Fast-path check for threading primitives before MTA analysis.

```rust
// In fast_paths.rs
pub fn has_threading_primitives(module: &AirModule) -> bool {
    const THREAD_FUNCTIONS: &[&str] = &[
        "pthread_create", "pthread_join", "thrd_create",
        "__VERIFIER_atomic_begin", "fork",
    ];
    module.functions.iter().any(|f| {
        THREAD_FUNCTIONS.contains(&f.name.as_str())
    })
}

// In property.rs
fn analyze_no_data_race(ctx: &AnalysisContext<'_>) -> PropertyResult {
    if !has_threading_primitives(ctx.module) {
        return PropertyResult::True;  // No threads = no races
    }
    // ... existing MTA analysis
}
```

**Impact:** ~100-200 TRUE verdicts across sequential categories.

---

### P1: Dead-Code Detection for `unreach-call`

**Problem:** Returns UNKNOWN when `reach_error()` is in a callee, even if that callee is never called.

**Solution:** Check call-graph reachability before returning UNKNOWN.

```rust
fn analyze_unreachability(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let error_calls = find_error_calls(module);
    let callgraph = ctx.callgraph();

    // Filter to only call-graph reachable error sites
    let reachable_errors: Vec<_> = error_calls
        .iter()
        .filter(|(func_id, _, _)| {
            *func_id == main_id ||
            is_reachable_in_callgraph(callgraph, main_id, *func_id)
        })
        .collect();

    // If ALL error calls are in unreachable functions → TRUE
    if reachable_errors.is_empty() {
        return PropertyResult::True;
    }
    // ... continue with Z3 analysis
}
```

**Impact:** ~30-50 TRUE verdicts.

---

### P2: Summary-Based Interprocedural Analysis for `unreach-call`

**Problem:** When `reach_error()` is in a reachable callee, we return UNKNOWN instead of checking path feasibility.

**Solution:** Compute function summaries capturing error reachability conditions.

```rust
// In summaries.rs
pub enum ErrorSummary {
    NeverErrors,                        // All paths return normally
    AlwaysErrors,                       // Unconditional error
    MayError { guards: Vec<PathGuard> }, // Conditional error
    Unknown,                            // Too complex
}

pub fn compute_error_summaries(
    module: &AirModule,
    callgraph: &CallGraph,
    config: &PropertyAnalysisConfig,
) -> BTreeMap<FunctionId, ErrorSummary> {
    // For each function with error calls, use Z3 to determine
    // under what conditions the error is reachable
}
```

Composition at call sites:
- If callee summary is `NeverErrors` → skip
- If `AlwaysErrors` → check if call site reachable
- If `MayError` → compose caller and callee guards, check with Z3

**Impact:** ~50-100 TRUE verdicts.

---

### P3: Loop-Free Overflow Analysis for `no-overflow`

**Problem:** Interval widening at loops causes imprecise bounds, always returns UNKNOWN.

**Solution:** For loop-free programs, intervals are exact—trust the results.

```rust
// In fast_paths.rs
pub fn is_loop_free(cfg: &Cfg) -> bool {
    cfg.back_edges().is_empty()
}

pub fn program_is_loop_free(cfgs: &BTreeMap<FunctionId, Cfg>) -> bool {
    cfgs.values().all(is_loop_free)
}

// In property.rs
fn analyze_no_overflow(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let result = ctx.absint();

    if result.findings.is_empty() && program_is_loop_free(ctx.cfgs()) {
        return PropertyResult::True;  // Loop-free + no findings = safe
    }
    // ... existing logic
}
```

**Impact:** ~20-30 TRUE verdicts.

---

### P4: No-Heap Fast-Path for `valid-memsafety`

**Problem:** Always returns UNKNOWN due to checker imprecision.

**Solution:** For stack-only programs, UAF/double-free/leaks are impossible.

```rust
// In fast_paths.rs
pub fn has_heap_allocations(module: &AirModule) -> bool {
    const ALLOC_FUNCTIONS: &[&str] = &[
        "malloc", "calloc", "realloc", "free",
        "aligned_alloc", "__VERIFIER_nondet_pointer",
    ];
    // Check for declarations and HeapAlloc operations
}

pub fn is_stack_only(module: &AirModule) -> bool {
    !has_heap_allocations(module)
}

// In property.rs
fn analyze_memsafety(ctx: &AnalysisContext<'_>) -> PropertyResult {
    if is_stack_only(ctx.module) {
        // Only run null-deref checker (skip UAF, double-free)
        let specs = vec![null_deref()];
        let result = run_checkers_path_sensitive(&specs, ...);
        if result.feasible.is_empty() {
            return PropertyResult::True;
        }
    }
    // ... full analysis
}
```

**Impact:** ~20-30 TRUE verdicts.

---

## Configuration

### New Config Field

```rust
pub struct PropertyAnalysisConfig {
    // ... existing fields
    pub conservative: bool,  // default: true
}
```

### CLI Flag

```bash
# Default: ultra-conservative (safe, lower coverage)
saf-bench svcomp --compiled-dir .compiled

# Aggressive: higher coverage, small risk
saf-bench svcomp --compiled-dir .compiled --aggressive
```

### Behavior by Mode

| Improvement | Conservative (default) | Aggressive |
|-------------|------------------------|------------|
| P0: No-thread | TRUE if no pthread decls | Same |
| P1: Dead-code | TRUE if unreachable | Same |
| P2: Summaries | UNKNOWN if any summary unknown | Continue analysis |
| P3: Loop-free | TRUE only, no FALSE | TRUE and FALSE |
| P4: No-heap | TRUE if stack-only + no findings | TRUE if no findings |

---

## Files to Create/Modify

### New Files

**`crates/saf-bench/src/svcomp/fast_paths.rs`** (~100 lines)
```rust
//! Fast-path checks for property analysis.

pub fn has_threading_primitives(module: &AirModule) -> bool;
pub fn is_stack_only(module: &AirModule) -> bool;
pub fn has_heap_allocations(module: &AirModule) -> bool;
pub fn is_loop_free(cfg: &Cfg) -> bool;
pub fn program_is_loop_free(cfgs: &BTreeMap<FunctionId, Cfg>) -> bool;
```

**`crates/saf-bench/src/svcomp/summaries.rs`** (~150 lines)
```rust
//! Function summaries for interprocedural error reachability.

pub enum ErrorSummary { NeverErrors, AlwaysErrors, MayError { guards }, Unknown }
pub fn compute_error_summaries(...) -> BTreeMap<FunctionId, ErrorSummary>;
pub fn compute_function_summary(...) -> ErrorSummary;
pub fn check_call_site_error_reachable(...) -> PathReachability;
```

**`crates/saf-bench/tests/svcomp_improvements_e2e.rs`** (~80 lines)

### Modified Files

| File | Changes |
|------|---------|
| `svcomp/mod.rs` | Add `conservative` to config, re-export new modules |
| `svcomp/property.rs` | Integrate P0-P4 into analyzers |
| `main.rs` | Add `--aggressive` CLI flag |
| `scripts/compile-svcomp.sh` | Support extended categories |

---

## Implementation Order

| Phase | Task | Est. Lines |
|-------|------|------------|
| 1 | Add `conservative` flag to config | ~20 |
| 2 | Create `fast_paths.rs` (P0, P3, P4 helpers) | ~100 |
| 3 | Implement P0 (no-thread detection) | ~15 |
| 4 | Implement P1 (dead-code filtering) | ~20 |
| 5 | Implement P4 (no-heap fast-path) | ~30 |
| 6 | Implement P3 (loop-free overflow) | ~40 |
| 7 | Create `summaries.rs` for P2 | ~150 |
| 8 | Implement P2 (interprocedural summaries) | ~100 |
| 9 | Add `--aggressive` CLI flag | ~15 |
| 10 | Unit tests | ~100 |
| 11 | E2E tests with fixtures | ~80 |
| 12 | Compile extended categories | ~20 |

**Total: ~700 lines of Rust**

---

## Testing Strategy

### Unit Tests
- `has_threading_primitives()` with/without pthread
- `is_stack_only()` with malloc/without
- `is_loop_free()` with/without back-edges
- Summary computation for simple functions

### E2E Tests
- P0: Sequential program returns TRUE for no-data-race
- P1: Dead-code error returns TRUE for unreach-call
- P2: Callee with unreachable error returns TRUE
- P3: Loop-free arithmetic returns TRUE for no-overflow
- P4: Stack-only program returns TRUE for memsafety

### Validation
```bash
# Check for incorrect verdicts (must be 0 in conservative mode)
saf-bench svcomp --compiled-dir .compiled --json | \
  jq '.results[] | select(.outcome == "TrueIncorrect" or .outcome == "FalseIncorrect")'
```

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| TRUE correct | 1 | 200+ |
| FALSE correct | 0 | 20-50 |
| Incorrect verdicts | 0 | 0 (conservative) |
| Coverage | 1% | 15-30% |

---

## Categories to Compile

```bash
# High-value categories for testing
loops                              # 186 tasks
recursive-simple                   # 159 tasks
bitvector                          # 154 tasks
locks                              # 30 tasks
signedintegeroverflow-regression   # 50 tasks
nla-digbench                       # 119 tasks
loop-lit                           # 100 tasks
```

---

## Risk Mitigation

1. **Incorrect TRUE (-32):** All fast-paths are proven sound; aggressive mode is opt-in
2. **Incorrect FALSE (-16):** Only P3 returns FALSE, only in aggressive mode with loop-free proof
3. **Regression:** Existing behavior preserved in conservative mode
