# PTA-Absint Integration Design

## Overview

Integrate Points-To Analysis (PTA) with Abstract Interpretation (Absint) to enable:
1. Alias-aware memory tracking in abstract interpretation
2. Indirect call resolution for interprocedural analysis
3. Bidirectional refinement (interval constraints refine PTA indices)

## Problem Statement

Current abstract interpretation has these limitations:
- Memory model uses `BTreeMap<ValueId, Interval>` — only tracks exact pointer IDs
- `CallIndirect` → TOP + invalidate all memory (no function pointer resolution)
- No feedback loop between numeric intervals and pointer analysis
- PTABen tests fail due to values from function calls being TOP

## Design

### Core Memory Model Redesign

**Current model** (`state.rs`):
```rust
pub struct AbstractState {
    values: BTreeMap<ValueId, Interval>,
    memory: BTreeMap<ValueId, Interval>,
}
```

**New model**:
```rust
pub struct AbstractState<'pta> {
    values: BTreeMap<ValueId, Interval>,
    memory: BTreeMap<LocId, Interval>,
    pta: &'pta PtaIntegration<'pta>,
}
```

**Semantic changes**:

| Operation | Current | New |
|-----------|---------|-----|
| `store *p, v` | `memory[p] = v` | `for loc in pta.points_to(p): memory[loc] ⊔= v` |
| `load v, *p` | `v = memory.get(p)` | `v = ⊔{memory[loc] : loc ∈ pta.points_to(p)}` |
| `gep r, p, idx` | `r = TOP` | `r = p` (pointer, not numeric) |
| `call f(args)` | Invalidate all | Selective invalidation via mod/ref |

**Strong vs weak update**:
- `|points_to(p)| == 1`: strong update (replace value)
- `|points_to(p)| > 1`: weak update (join with existing)

### Transfer Functions with PTA

**Store operation**:
```rust
fn transfer_store(state: &mut AbstractState, ptr: ValueId, value: Interval) {
    let pts = state.pta.points_to(ptr);

    if pts.is_empty() {
        return;  // Unknown pointer - conservative no-op
    }

    let is_strong = pts.len() == 1;

    for loc in pts {
        if is_strong {
            state.memory.insert(loc, value);
        } else {
            let existing = state.memory.get(&loc).copied().unwrap_or(Interval::bottom());
            state.memory.insert(loc, existing.join(&value));
        }
    }
}
```

**Load operation**:
```rust
fn transfer_load(state: &AbstractState, ptr: ValueId, bits: u32) -> Interval {
    let pts = state.pta.points_to(ptr);

    if pts.is_empty() {
        return Interval::make_top(bits);
    }

    let mut result = Interval::bottom();
    for loc in pts {
        let val = state.memory.get(&loc).copied().unwrap_or(Interval::make_top(bits));
        result = result.join(&val);
    }
    result
}
```

### Indirect Call Resolution

Use PTA to resolve function pointer targets, then apply interprocedural analysis:

```rust
fn transfer_call_indirect(
    state: &mut AbstractState,
    dst: ValueId,
    fn_ptr: ValueId,
    args: &[ValueId],
    ctx: &AnalysisContext,
) -> Interval {
    let targets = ctx.pta.resolve_indirect_call(fn_ptr);

    if targets.is_empty() {
        state.invalidate_all_memory();
        return Interval::make_top(64);
    }

    let arg_intervals: Vec<Interval> = args.iter()
        .map(|a| state.get(*a))
        .collect();

    let mut return_interval = Interval::bottom();
    let mut modified_locs: BTreeSet<LocId> = BTreeSet::new();

    for func_id in &targets {
        let summary = ctx.get_or_compute_summary(*func_id, &arg_intervals);

        if let Some(ret) = summary.return_interval {
            return_interval = return_interval.join(&ret);
        }
        modified_locs.extend(summary.modified_locations.iter().copied());
    }

    for loc in modified_locs {
        state.memory.insert(loc, Interval::make_top(64));
    }

    return_interval
}
```

**Function summaries with mod/ref**:
```rust
pub struct FunctionSummary {
    pub return_interval: Option<Interval>,
    pub param_count: usize,
    pub modified_locations: BTreeSet<LocId>,
    pub referenced_locations: BTreeSet<LocId>,
    pub may_modify_unknown: bool,
}
```

### Bidirectional Refinement (Absint → PTA)

Use interval constraints to refine array index sensitivity:

```rust
pub struct IndexRefinement {
    pub gep_inst: InstructionId,
    pub index_value: ValueId,
    pub interval: Interval,
}

fn collect_index_refinements(
    module: &AirModule,
    absint: &AbstractInterpResult,
) -> Vec<IndexRefinement>;

impl PtaResult {
    pub fn refine_with_indices(&self, refinements: &[IndexRefinement]) -> PtaResult;
}
```

**Iterative refinement loop**:
```rust
pub fn analyze_with_refinement(
    module: &AirModule,
    config: &CombinedAnalysisConfig,
) -> CombinedResult {
    let mut pta = PtaContext::analyze(module, &config.pta);

    for _ in 0..config.max_refinement_iterations {
        let absint = analyze_with_pta(module, &pta, &config.absint);
        let refinements = collect_index_refinements(module, &absint);

        if refinements.is_empty() { break; }

        let refined_pta = pta.refine_with_indices(&refinements);
        if refined_pta.equals(&pta) { break; }

        pta = refined_pta;
    }

    CombinedResult { pta, absint }
}
```

## Module Structure

```
crates/saf-analysis/src/
├── absint/
│   ├── mod.rs
│   ├── domain.rs           # (unchanged)
│   ├── interval.rs         # (unchanged)
│   ├── state.rs            # MODIFIED: LocId-based memory
│   ├── transfer.rs         # MODIFIED: PTA-aware
│   ├── fixpoint.rs         # MODIFIED: pass PTA context
│   ├── interprocedural.rs  # MODIFIED: indirect calls
│   ├── nullness.rs         # MODIFIED: PTA-aware
│   ├── checker.rs          # (unchanged)
│   └── pta_integration.rs  # NEW: PTA integration layer
├── pta/
│   ├── ...
│   ├── refinement.rs       # NEW: Index refinement
│   └── mod_ref.rs          # NEW: Mod/ref analysis
└── combined/
    └── mod.rs              # NEW: Combined orchestration
```

## Public API

```rust
pub struct CombinedAnalysisConfig {
    pub pta: PtaConfig,
    pub absint: AbsIntConfig,
    pub enable_refinement: bool,
    pub max_refinement_iterations: usize,
    pub context_sensitive_indirect: bool,
}

pub struct CombinedAnalysisResult {
    pub pta: PtaResult,
    pub absint: AbstractInterpResult,
    pub summaries: BTreeMap<FunctionId, FunctionSummary>,
    pub refinement_iterations: usize,
}

pub fn analyze_combined(
    module: &AirModule,
    config: &CombinedAnalysisConfig,
) -> CombinedAnalysisResult;
```

**Python bindings**:
```python
result = saf.analyze_combined(module, enable_refinement=True)
result.interval_at(inst_id, value_id)  # -> (lo, hi)
result.points_to(value_id)              # -> [loc_str, ...]
result.may_alias(a, b)                  # -> "May" | "No" | ...
result.function_summary(func_id)        # -> PySummary
```

## Implementation Phases

| Phase | Description | Files | LOC Est. |
|-------|-------------|-------|----------|
| 1 | PTA integration layer | `absint/pta_integration.rs` | ~200 |
| 2 | State refactor (LocId memory) | `absint/state.rs` | ~150 |
| 3 | Transfer functions update | `absint/transfer.rs` | ~200 |
| 4 | Mod/ref analysis | `pta/mod_ref.rs` | ~150 |
| 5 | Indirect call resolution | `absint/interprocedural.rs` | ~250 |
| 6 | Index refinement | `pta/refinement.rs` | ~150 |
| 7 | Combined orchestration | `combined/mod.rs` | ~200 |
| 8 | Python bindings | `saf-python/src/lib.rs` | ~100 |
| 9 | Testing & PTABen validation | `tests/*.rs` | ~300 |
| **Total** | | | **~1700** |

**Implementation order**: 1 → 2 → 3 → 4 → 5 → 7 → 8 → 6 → 9

## Test Fixtures

**Alias through memory** (`pta_absint_alias.c`):
```c
void test_alias() {
    int x = 10;
    int *p = &x;
    int *q = &x;  // p and q alias
    *p = 42;
    int v = *q;   // Should get interval [42, 42]
}
```

**Indirect calls** (`indirect_call.c`):
```c
int add(int a, int b) { return a + b; }
int mul(int a, int b) { return a * b; }

int compute(int (*op)(int, int), int x, int y) {
    return op(x, y);
}

int main() {
    int r1 = compute(add, 3, 4);  // Should get [7, 7]
    int r2 = compute(mul, 3, 4);  // Should get [12, 12]
}
```

**Index refinement** (`index_refine.c`):
```c
void test() {
    int arr[10];
    int i = 5;
    int *p = &arr[i];  // Points to arr[5] after refinement
    int *q = &arr[0];  // Points to arr[0]
    // p and q should be NoAlias
}
```

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| PTA becomes mandatory | Provide `PtaIntegration::empty()` fallback |
| Performance regression | Cache points-to lookups; lazy computation |
| Weak update explosion | Threshold: `\|pts\| > 100` → TOP |
| Mod/ref over-approximation | Start conservative, refine later |
| Refinement non-convergence | Hard cap at `max_refinement_iterations` |
| Breaking API changes | Keep old API as deprecated wrapper |

## Success Metrics

- All existing 1499 Rust + 248 Python tests pass
- PTABen Exact count increases by 100+
- No regression in Unsound count

**Expected PTABen improvements**:

| Category | Current | Target |
|----------|---------|--------|
| ae_assert_tests | 68 fail | 30 fail |
| ae_recursion_tests | 32 fail | 15 fail |
| ae_overflow_tests | 164 fail | 80 fail |
| basic_c_tests | ~40 pass | ~55 pass |

## Backward Compatibility

```rust
// Existing API preserved
pub fn analyze(module: &AirModule, config: &AbsIntConfig) -> AbstractInterpResult {
    let empty_pta = PtaResult::empty();
    analyze_with_pta(module, &empty_pta, config)
}

// New PTA-aware API
pub fn analyze_with_pta(
    module: &AirModule,
    pta: &PtaResult,
    config: &AbsIntConfig
) -> AbstractInterpResult;
```
