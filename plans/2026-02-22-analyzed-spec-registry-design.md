# Analyzed Spec Registry — Design Document

**Date:** 2026-02-22
**Status:** Approved
**Plan:** 150

## Problem

SAF's static analyzer achieves ~52% precision on Juliet benchmarks, with ~50% false positive rate on buffer overflow CWEs (121-127). Root cause investigation reveals the precision ceiling is NOT caused by weak abstract domains — the constants are already precise. Instead:

1. **Missing computed return bounds:** `strlen(buf)` returns `[0, SIZE_MAX]` instead of `[0, alloc_size(buf)-1]`. The spec system only supports fixed intervals.
2. **Parallel systems:** YAML specs (SpecRegistry) and analysis summaries (ParameterEffectSummary) don't communicate. Each has information the other needs.

## Solution: Layered Registry (Approach B)

### Architecture

```
                    ┌─────────────────────────┐
                    │  AnalyzedSpecRegistry    │
                    │                          │
  YAML specs ──────►│  yaml: SpecRegistry      │──► transfer function
  (immutable)       │                          │──► checkers
                    │  derived: BTreeMap<       │──► ResourceTable
  Analysis ────────►│    String, DerivedSpec>   │──► temporal filter
  (summaries)       │                          │
                    └─────────────────────────┘
```

### New Types (saf-core)

```rust
enum BoundMode {
    AllocSizeMinusOne,    // strlen: [0, alloc_size(arg) - 1]
    AllocSize,            // fread: [0, alloc_size(arg)]
    ParamValueMinusOne,   // read(fd, buf, n): [-1, n - 1]
}

struct ComputedBound {
    param_index: u32,
    mode: BoundMode,
}

struct DerivedSpec {
    computed_return_bound: Option<ComputedBound>,
    param_freed: BTreeMap<usize, bool>,
    param_dereferenced: BTreeMap<usize, bool>,
    return_is_allocated: bool,
}

struct AnalyzedSpecRegistry {
    yaml: SpecRegistry,              // immutable after load
    derived: BTreeMap<String, DerivedSpec>,  // built during analysis
}
```

### Resolution at Call Site

When the transfer function encounters `strlen(buf)`:

1. Look up `strlen` in `AnalyzedSpecRegistry.derived` → finds `ComputedBound { param: 0, mode: AllocSizeMinusOne }`
2. Resolve: find alloc size of argument 0 (`buf`) → `[11, 11]` (from alloca)
3. Compute: return interval = `[0, 11 - 1]` = `[0, 10]`
4. If resolution fails (can't find alloc size), fall back to YAML fixed interval `[0, SIZE_MAX]`

### Merge Strategy

- YAML specs are immutable (loaded at startup)
- Derived specs are computed during analysis
- `lookup()` returns both layers; consumers pick what they need
- Computed bounds override fixed intervals when resolvable
- Summary effects (param_freed, etc.) merge additively

### Consumer Migration

All consumers that currently take `&SpecRegistry` change to `&AnalyzedSpecRegistry`:
- `TransferContext.specs` — uses computed bounds + fixed intervals
- `FixpointContext.specs` — uses noreturn detection
- `ResourceTable::from_specs()` — uses YAML layer via `.yaml()`
- `filter_temporal_infeasible()` — uses derived layer for param effects

## Design Decisions

1. **Why not inject into SpecRegistry (Approach A)?** SpecRegistry uses `(i64, i64)` for intervals — can't express computed bounds. Would require schema change that mixes authored and derived data.

2. **Why not full summary-as-analyzer (Approach C)?** Running per-function absint in summary phase is expensive and risks circular dependencies (PTA needs specs, specs need PTA). Overkill for the current need.

3. **Why function-name keyed?** Consistent with SpecRegistry's lookup-by-name pattern. FunctionId-keyed lookup can be added later if needed.

4. **Why include ldv_strlen_1 in computed bounds?** It's the SV-COMP wrapper for strlen with identical semantics. This is not benchmark-specific — it's modeling the function's actual contract.
