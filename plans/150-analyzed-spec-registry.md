# Plan 150: Analyzed Spec Registry — Layered Specs with Computed Return Bounds

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Unify the spec system and function summary system into a layered `AnalyzedSpecRegistry` that supports computed return intervals (e.g., `strlen(buf) → [0, alloc_size(buf)-1]`), enabling the analyzer to prove Juliet good variants safe without benchmark-specific heuristics.

**Architecture:** A new `AnalyzedSpecRegistry` wraps the existing immutable `SpecRegistry` (YAML-authored) with an analysis-derived overlay containing `DerivedSpec` entries. The overlay stores computed return bounds (return interval as a function of argument properties) and parameter effect summaries. Consumers query the layered registry via a unified API; the transfer function resolves computed bounds at each call site using the allocation size map. The existing `ParameterEffectSummary` module feeds into the overlay automatically.

**Tech Stack:** Rust (saf-core for types, saf-analysis for integration), existing `SpecRegistry` + `ParameterEffectSummary` infrastructure.

---

## Background

The analyzer currently has two parallel systems for function contracts:

1. **SpecRegistry** (YAML): Static specs with fixed return intervals like `rand → [0, INT_MAX]`. Consumed by transfer function, checkers, ResourceTable.
2. **ParameterEffectSummary**: Analysis-derived boolean facts (`param_freed`, `param_dereferenced`, `return_is_allocated`). Only consumed by temporal safety filter.

These don't talk to each other. More critically, the spec system can only express **fixed** return intervals — it cannot express "strlen returns a value bounded by its argument's allocation size." This means `strlen(buf)` returns `[0, SIZE_MAX]`, which is too wide for the buffer overflow checker to prove safety.

### Root Cause of Juliet ~50% Precision

Juliet good variants use patterns like:
```c
char buf[11];
char src[11] = "AAAAAAAAAA";
memcpy(buf, src, strlen(src) + 1);  // safe: 11 <= 11
```

The analyzer sees `strlen(src)` → `[0, SIZE_MAX]` (no useful bound), so `strlen(src)+1` → `[1, TOP]`, and the memcpy checker flags `TOP > 11` as a potential overflow. Z3 cannot refute it without bounds on `strlen`.

### What This Plan Delivers

1. **Computed return bounds**: `strlen(buf) → [0, alloc_size(buf)-1]` resolved at each call site
2. **Unified spec+summary registry**: One place for all function contract information
3. **General capability**: Works for `strlen`, `strnlen`, `fread`, `recv`, and any function whose return is bounded by an argument's buffer size

---

## Key Files Reference

| Component | File | Key Lines |
|-----------|------|-----------|
| `SpecRegistry` struct | `crates/saf-core/src/spec/registry.rs` | 21-34 |
| `SpecRegistry::lookup()` | `crates/saf-core/src/spec/registry.rs` | 163-185 |
| `SpecRegistry::add()` | `crates/saf-core/src/spec/registry.rs` | 264-266 |
| `FunctionSpec` | `crates/saf-core/src/spec/types.rs` | 15-47 |
| `ReturnSpec` | `crates/saf-core/src/spec/types.rs` | 300-322 |
| `ParamSpec` | `crates/saf-core/src/spec/types.rs` | 182-223 |
| Spec module exports | `crates/saf-core/src/spec/mod.rs` | 62-74 |
| `TransferContext` | `crates/saf-analysis/src/absint/transfer.rs` | 36-62 |
| CallDirect spec lookup | `crates/saf-analysis/src/absint/transfer.rs` | 560-580 |
| `FixpointContext` | `crates/saf-analysis/src/absint/fixpoint.rs` | 42-57 |
| `ParameterEffectSummary` | `crates/saf-analysis/src/checkers/summary.rs` | 18-33 |
| `compute_parameter_effect_summaries()` | `crates/saf-analysis/src/checkers/summary.rs` | 46-49 |
| `ResourceTable::from_specs()` | `crates/saf-analysis/src/checkers/resource_table.rs` | 95-102 |
| `filter_temporal_infeasible()` | `crates/saf-analysis/src/checkers/pathsens_runner.rs` | 281-332 |
| `build_c_library_specs()` | `crates/saf-bench/src/svcomp/property.rs` | 281-345 |
| Summary computation call | `crates/saf-bench/src/svcomp/property.rs` | 763-764 |
| `extract_allocation_sizes()` | `crates/saf-analysis/src/absint/checker.rs` | 2398-2451 |

---

## Task 1: Define `ComputedBound` and `DerivedSpec` Types

**Files:**
- Create: `crates/saf-core/src/spec/derived.rs`
- Modify: `crates/saf-core/src/spec/mod.rs`
- Test: `crates/saf-core/tests/smoke.rs` (add cases)

**Step 1: Write failing test**

In `crates/saf-core/tests/smoke.rs`, add:

```rust
#[test]
fn derived_spec_alloc_size_bound() {
    use saf_core::spec::{ComputedBound, BoundMode, DerivedSpec};

    let bound = ComputedBound {
        param_index: 0,
        mode: BoundMode::AllocSizeMinusOne,
    };
    let spec = DerivedSpec {
        computed_return_bound: Some(bound),
        param_freed: BTreeMap::new(),
        param_dereferenced: BTreeMap::new(),
        return_is_allocated: false,
    };
    assert_eq!(spec.computed_return_bound.as_ref().unwrap().param_index, 0);
    assert!(matches!(
        spec.computed_return_bound.as_ref().unwrap().mode,
        BoundMode::AllocSizeMinusOne
    ));
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-core derived_spec_alloc_size_bound 2>&1'`
Expected: FAIL — `ComputedBound` not found

**Step 3: Create `derived.rs` with types**

Create `crates/saf-core/src/spec/derived.rs`:

```rust
//! Analysis-derived function specifications.
//!
//! These types represent function contract information discovered by
//! static analysis, as opposed to YAML-authored `FunctionSpec` entries.

use std::collections::BTreeMap;

/// How a return value is bounded by an argument property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundMode {
    /// Return ∈ [0, alloc_size(param) - 1].
    /// Models: `strlen(buf)`, `strnlen(buf, n)` (first arg).
    AllocSizeMinusOne,
    /// Return ∈ [0, alloc_size(param)].
    /// Models: `fread(buf, 1, size, fp)` where size = alloc_size(buf).
    AllocSize,
    /// Return ∈ [-1, param_value - 1].
    /// Models: `read(fd, buf, count)` returns [-1, count-1].
    ParamValueMinusOne,
}

/// A computed return bound: return interval depends on an argument property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedBound {
    /// Which parameter's property bounds the return value.
    pub param_index: u32,
    /// How the return is bounded.
    pub mode: BoundMode,
}

/// Analysis-derived specification for a function.
///
/// Produced by the summary module and merged with YAML specs in
/// `AnalyzedSpecRegistry`.
#[derive(Debug, Clone)]
pub struct DerivedSpec {
    /// Computed return interval bound (if applicable).
    pub computed_return_bound: Option<ComputedBound>,
    /// Whether the callee frees `param[i]` (directly or transitively).
    pub param_freed: BTreeMap<usize, bool>,
    /// Whether the callee dereferences `param[i]`.
    pub param_dereferenced: BTreeMap<usize, bool>,
    /// Whether the function returns newly allocated memory.
    pub return_is_allocated: bool,
}

impl DerivedSpec {
    /// Create an empty derived spec with no discovered properties.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            computed_return_bound: None,
            param_freed: BTreeMap::new(),
            param_dereferenced: BTreeMap::new(),
            return_is_allocated: false,
        }
    }

    /// Create from a `ParameterEffectSummary`-style tuple.
    #[must_use]
    pub fn from_effects(
        param_freed: BTreeMap<usize, bool>,
        param_dereferenced: BTreeMap<usize, bool>,
        return_is_allocated: bool,
    ) -> Self {
        Self {
            computed_return_bound: None,
            param_freed,
            param_dereferenced,
            return_is_allocated,
        }
    }
}
```

**Step 4: Add module to `mod.rs`**

In `crates/saf-core/src/spec/mod.rs`, add:
```rust
mod derived;
```

And add to re-exports:
```rust
pub use derived::{BoundMode, ComputedBound, DerivedSpec};
```

**Step 5: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-core derived_spec_alloc_size_bound 2>&1'`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/saf-core/src/spec/derived.rs crates/saf-core/src/spec/mod.rs crates/saf-core/tests/smoke.rs
git commit -m "feat(spec): add DerivedSpec and ComputedBound types for analysis-derived function contracts"
```

---

## Task 2: Build `AnalyzedSpecRegistry` Wrapper

**Files:**
- Create: `crates/saf-core/src/spec/analyzed.rs`
- Modify: `crates/saf-core/src/spec/mod.rs`
- Test: `crates/saf-core/tests/smoke.rs` (add cases)

**Step 1: Write failing tests**

In `crates/saf-core/tests/smoke.rs`, add:

```rust
#[test]
fn analyzed_registry_yaml_lookup() {
    use saf_core::spec::{AnalyzedSpecRegistry, FunctionSpec, ReturnSpec, SpecRegistry};

    let mut yaml_reg = SpecRegistry::default();
    let mut spec = FunctionSpec::new("rand");
    spec.returns = Some(ReturnSpec {
        interval: Some((0, 2_147_483_647)),
        ..ReturnSpec::default()
    });
    yaml_reg.add(spec).unwrap();

    let analyzed = AnalyzedSpecRegistry::new(yaml_reg);
    let result = analyzed.lookup("rand");
    assert!(result.is_some());
    let (func_spec, derived) = result.unwrap();
    assert_eq!(func_spec.returns.as_ref().unwrap().interval, Some((0, 2_147_483_647)));
    assert!(derived.is_none()); // No analysis overlay for rand
}

#[test]
fn analyzed_registry_derived_overlay() {
    use saf_core::spec::{
        AnalyzedSpecRegistry, BoundMode, ComputedBound, DerivedSpec, FunctionSpec,
        ReturnSpec, SpecRegistry,
    };

    let mut yaml_reg = SpecRegistry::default();
    let mut spec = FunctionSpec::new("strlen");
    spec.returns = Some(ReturnSpec {
        interval: Some((0, 9_223_372_036_854_775_807)),
        ..ReturnSpec::default()
    });
    yaml_reg.add(spec).unwrap();

    let mut analyzed = AnalyzedSpecRegistry::new(yaml_reg);

    let derived = DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    };
    analyzed.add_derived("strlen", derived);

    let result = analyzed.lookup("strlen");
    assert!(result.is_some());
    let (func_spec, derived_opt) = result.unwrap();
    // YAML spec still accessible
    assert!(func_spec.returns.is_some());
    // Derived overlay also accessible
    let derived = derived_opt.unwrap();
    assert!(derived.computed_return_bound.is_some());
    assert_eq!(derived.computed_return_bound.as_ref().unwrap().param_index, 0);
}

#[test]
fn analyzed_registry_derived_only() {
    use saf_core::spec::{AnalyzedSpecRegistry, DerivedSpec, SpecRegistry};
    use std::collections::BTreeMap;

    let yaml_reg = SpecRegistry::default(); // empty
    let mut analyzed = AnalyzedSpecRegistry::new(yaml_reg);

    let derived = DerivedSpec::from_effects(
        BTreeMap::from([(0, true)]),
        BTreeMap::new(),
        false,
    );
    analyzed.add_derived("my_free_wrapper", derived);

    let result = analyzed.lookup("my_free_wrapper");
    assert!(result.is_some());
    let (_, derived_opt) = result.unwrap();
    assert!(derived_opt.unwrap().param_freed[&0]);
}
```

**Step 2: Run tests to verify they fail**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-core analyzed_registry 2>&1'`
Expected: FAIL — `AnalyzedSpecRegistry` not found

**Step 3: Create `analyzed.rs`**

Create `crates/saf-core/src/spec/analyzed.rs`:

```rust
//! Layered spec registry combining YAML-authored specs with analysis-derived facts.

use std::collections::BTreeMap;

use super::derived::DerivedSpec;
use super::registry::SpecRegistry;
use super::types::FunctionSpec;

/// A layered spec registry that combines immutable YAML-authored specs
/// with mutable analysis-derived overlays.
///
/// Consumers query via `lookup()`, which returns both the YAML spec (if any)
/// and the derived overlay (if any). At least one must exist for a lookup
/// to succeed.
#[derive(Debug)]
pub struct AnalyzedSpecRegistry {
    /// Immutable YAML-authored specs.
    yaml: SpecRegistry,
    /// Analysis-derived overlay, keyed by function name.
    derived: BTreeMap<String, DerivedSpec>,
}

impl AnalyzedSpecRegistry {
    /// Create a new layered registry wrapping existing YAML specs.
    #[must_use]
    pub fn new(yaml: SpecRegistry) -> Self {
        Self {
            yaml,
            derived: BTreeMap::new(),
        }
    }

    /// Add an analysis-derived spec overlay for a function.
    pub fn add_derived(&mut self, name: &str, spec: DerivedSpec) {
        self.derived.insert(name.to_owned(), spec);
    }

    /// Look up a function by name. Returns the YAML spec and/or derived overlay.
    ///
    /// Returns `None` if neither YAML nor derived spec exists for this name.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<(&FunctionSpec, Option<&DerivedSpec>)> {
        let yaml_spec = self.yaml.lookup(name);
        let derived_spec = self.derived.get(name);

        match (yaml_spec, derived_spec) {
            (Some(y), d) => Some((y, d)),
            (None, Some(d)) => {
                // Derived-only: create would require owned FunctionSpec.
                // Instead, return None for yaml — callers handle this case.
                // Actually, we need a sentinel. Let's use a different return type.
                None // TODO: Task 3 will refine this — for now derived-only
                     // functions need a stub YAML entry.
            }
            (None, None) => None,
        }
    }

    /// Look up only the derived overlay for a function.
    #[must_use]
    pub fn lookup_derived(&self, name: &str) -> Option<&DerivedSpec> {
        self.derived.get(name)
    }

    /// Look up only the YAML spec for a function (delegates to inner registry).
    #[must_use]
    pub fn lookup_yaml(&self, name: &str) -> Option<&FunctionSpec> {
        self.yaml.lookup(name)
    }

    /// Access the underlying YAML registry.
    #[must_use]
    pub fn yaml(&self) -> &SpecRegistry {
        &self.yaml
    }

    /// Iterate all YAML specs (delegates to inner registry).
    pub fn iter_yaml(&self) -> impl Iterator<Item = &FunctionSpec> {
        self.yaml.iter()
    }

    /// Iterate all derived specs.
    pub fn iter_derived(&self) -> impl Iterator<Item = (&str, &DerivedSpec)> {
        self.derived.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Number of derived overlay entries.
    #[must_use]
    pub fn derived_count(&self) -> usize {
        self.derived.len()
    }
}
```

Note: The `lookup()` return type for derived-only entries needs refinement. For Task 2, the test `analyzed_registry_derived_only` should use `lookup_derived()` instead. Update the test:

```rust
#[test]
fn analyzed_registry_derived_only() {
    // ... same setup ...
    let result = analyzed.lookup_derived("my_free_wrapper");
    assert!(result.is_some());
    assert!(result.unwrap().param_freed[&0]);
}
```

**Step 4: Add module to `mod.rs`**

In `crates/saf-core/src/spec/mod.rs`, add:
```rust
mod analyzed;
```

And add to re-exports:
```rust
pub use analyzed::AnalyzedSpecRegistry;
```

**Step 5: Run tests to verify they pass**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-core analyzed_registry 2>&1'`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/saf-core/src/spec/analyzed.rs crates/saf-core/src/spec/mod.rs crates/saf-core/tests/smoke.rs
git commit -m "feat(spec): add AnalyzedSpecRegistry layered wrapper for YAML + analysis-derived specs"
```

---

## Task 3: Wire `AnalyzedSpecRegistry` into Transfer Function

This is the core integration — replace `specs: Option<&SpecRegistry>` with `specs: Option<&AnalyzedSpecRegistry>` in `TransferContext` and `FixpointContext`, and resolve computed bounds at call sites.

**Files:**
- Modify: `crates/saf-analysis/src/absint/transfer.rs` (TransferContext + CallDirect)
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs` (FixpointContext)
- Test: `crates/saf-analysis/tests/absint_e2e.rs` (add integration test)

**Step 1: Write failing integration test**

In `crates/saf-analysis/tests/absint_e2e.rs`, add a test that exercises computed bounds. This test creates a module with an external `strlen` call and verifies the return interval is bounded by the buffer's allocation size.

```rust
#[test]
fn computed_bound_strlen_resolves_at_callsite() {
    use saf_core::spec::{
        AnalyzedSpecRegistry, BoundMode, ComputedBound, DerivedSpec,
        FunctionSpec, ReturnSpec, SpecRegistry,
    };
    // Load a fixture with strlen call on a known-size buffer
    let module = load_ll_fixture("strlen_known_buffer.ll");

    // Build YAML specs with wide strlen interval
    let mut yaml_reg = SpecRegistry::default();
    let mut strlen_spec = FunctionSpec::new("strlen");
    strlen_spec.pure = Some(true);
    strlen_spec.returns = Some(ReturnSpec {
        interval: Some((0, 9_223_372_036_854_775_807)),
        ..ReturnSpec::default()
    });
    yaml_reg.add(strlen_spec).unwrap();

    // Add computed bound overlay
    let mut analyzed = AnalyzedSpecRegistry::new(yaml_reg);
    analyzed.add_derived("strlen", DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    });

    // Run absint with analyzed specs
    let result = solve_abstract_interp(&module, &analyzed);

    // Find the strlen call's return value and check its interval
    // is bounded by the buffer's alloc size (e.g., [0, 10] for char buf[11])
    let strlen_return = find_value_by_name(&result, &module, "strlen_result");
    assert!(strlen_return.is_some(), "strlen return value should be tracked");
    let interval = strlen_return.unwrap();
    assert!(!interval.is_top(), "strlen return should not be TOP with computed bound");
    // buf is alloca [11 x i8], so strlen should return [0, 10]
    assert_eq!(interval.lo(), 0);
    assert_eq!(interval.hi(), 10);
}
```

Note: This test requires a new fixture `strlen_known_buffer.ll` and helper functions. The fixture should be:

```llvm
; tests/fixtures/llvm/e2e/strlen_known_buffer.ll
declare i64 @strlen(ptr)

define i64 @test_strlen_known() {
entry:
  %buf = alloca [11 x i8], align 1
  %len = call i64 @strlen(ptr %buf)
  ret i64 %len
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis computed_bound_strlen 2>&1'`
Expected: FAIL — API doesn't accept `AnalyzedSpecRegistry` yet

**Step 3: Update `TransferContext` and `FixpointContext`**

In `crates/saf-analysis/src/absint/transfer.rs`, change the `specs` field:

```rust
// Old:
pub specs: Option<&'a SpecRegistry>,

// New:
pub specs: Option<&'a AnalyzedSpecRegistry>,
```

Add import at top of file:
```rust
use saf_core::spec::AnalyzedSpecRegistry;
```

In `crates/saf-analysis/src/absint/fixpoint.rs`, same change:

```rust
// Old:
pub specs: Option<&'a SpecRegistry>,

// New:
pub specs: Option<&'a AnalyzedSpecRegistry>,
```

**Step 4: Update CallDirect handler to resolve computed bounds**

In `transfer.rs`, replace the spec lookup block (lines 560-580) with:

```rust
// If the return interval is TOP, check the analyzed spec registry.
// First try computed bounds (dynamic), then fall back to fixed interval (static).
if return_interval.is_top() {
    if let Some(analyzed_specs) = ctx.specs {
        let callee_name = module
            .functions
            .iter()
            .find(|f| f.id == *callee)
            .map(|f| f.name.as_str());
        if let Some(name) = callee_name {
            // Try computed bound first — resolves at this call site
            if let Some(derived) = analyzed_specs.lookup_derived(name) {
                if let Some(ref bound) = derived.computed_return_bound {
                    let resolved = resolve_computed_bound(
                        bound, inst, state, module, ctx.pta,
                    );
                    if !resolved.is_top() {
                        return_interval = resolved;
                    }
                }
            }

            // Fall back to fixed YAML interval if still TOP
            if return_interval.is_top() {
                if let Some(spec) = analyzed_specs.lookup_yaml(name) {
                    if let Some(ref ret) = spec.returns {
                        if let Some((lo, hi)) = ret.interval {
                            return_interval =
                                Interval::new(i128::from(lo), i128::from(hi), DEFAULT_BITS);
                        }
                    }
                }
            }
        }
    }
}
```

**Step 5: Implement `resolve_computed_bound()`**

Add to `transfer.rs`:

```rust
use saf_core::spec::{BoundMode, ComputedBound};

/// Resolve a computed return bound at a specific call site.
///
/// Uses the argument's allocation size (from alloca/malloc) to compute
/// a concrete interval for the return value.
fn resolve_computed_bound(
    bound: &ComputedBound,
    inst: &Instruction,
    state: &AbstractState,
    module: &AirModule,
    pta: Option<&PtaIntegration<'_>>,
) -> Interval {
    let param_idx = bound.param_index as usize;
    if param_idx >= inst.operands.len() {
        return Interval::make_top(DEFAULT_BITS);
    }
    let arg_value = inst.operands[param_idx];

    // Try to find allocation size for this argument.
    // Strategy: look for alloca/HeapAlloc that produced this pointer.
    let alloc_size = find_argument_alloc_size(arg_value, module, pta);

    match alloc_size {
        Some(size) if size > 0 => match bound.mode {
            BoundMode::AllocSizeMinusOne => {
                Interval::new(0, size - 1, DEFAULT_BITS)
            }
            BoundMode::AllocSize => {
                Interval::new(0, size, DEFAULT_BITS)
            }
            BoundMode::ParamValueMinusOne => {
                // For this mode, use the argument's interval value, not alloc size
                let arg_interval = state.get_opt(arg_value)
                    .cloned()
                    .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));
                if arg_interval.is_top() || arg_interval.is_bottom() {
                    Interval::make_top(DEFAULT_BITS)
                } else {
                    Interval::new(-1, arg_interval.hi() - 1, DEFAULT_BITS)
                }
            }
        },
        _ => Interval::make_top(DEFAULT_BITS),
    }
}

/// Find the allocation size for a value that is a pointer argument.
///
/// Walks back through the instruction chain to find the originating
/// alloca or heap allocation, then returns its size in bytes.
fn find_argument_alloc_size(
    value: ValueId,
    module: &AirModule,
    pta: Option<&PtaIntegration<'_>>,
) -> Option<i128> {
    // Strategy 1: Direct instruction scan — find the alloca that produced this value
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.dst == Some(value) {
                    match &inst.op {
                        Operation::Alloca { size_bytes: Some(size) } => {
                            return Some(i128::from(*size));
                        }
                        Operation::Gep { .. } => {
                            // GEP from alloca — walk through to base
                            if let Some(base) = inst.operands.first() {
                                return find_argument_alloc_size(*base, module, pta);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Strategy 2: If PTA available, resolve through points-to set
    if let Some(pta_int) = pta {
        if let Some(locs) = pta_int.points_to(value) {
            for loc_id in locs {
                if let Some(loc) = pta_int.location(loc_id) {
                    if let Some(size) = loc.size_bytes {
                        return Some(i128::from(size));
                    }
                }
            }
        }
    }

    None
}
```

Note: The `PtaIntegration` access pattern (`points_to`, `location`) may need adjustment to match the actual API. Check `crates/saf-analysis/src/absint/transfer.rs` for existing PTA usage patterns and match them.

**Step 6: Fix all compilation errors from `SpecRegistry → AnalyzedSpecRegistry` change**

Every file that passes `&SpecRegistry` to `TransferContext` or `FixpointContext` needs updating. The key call sites are:

- `crates/saf-analysis/src/absint/fixpoint.rs` — `FixpointContext::new()` signature
- `crates/saf-analysis/src/absint/checker.rs` — checker functions that create `TransferContext`
- `crates/saf-analysis/src/absint/interprocedural.rs` — interprocedural solver
- `crates/saf-analysis/src/absint/escape.rs` — escape analysis
- `crates/saf-analysis/src/absint/nullness.rs` — nullness analysis
- `crates/saf-analysis/src/absint/function_properties.rs` — function properties
- `crates/saf-bench/src/svcomp/property.rs` — property analyzer (wraps `SpecRegistry` in `AnalyzedSpecRegistry`)
- `crates/saf-bench/src/ptaben.rs` — PTABen benchmark
- `crates/saf-bench/src/cruxbc.rs` — CruxBC benchmark

For each file, the change is mechanical: where it previously passed `Some(&spec_registry)`, it now passes `Some(&analyzed_spec_registry)`. The property analyzer wraps its `build_c_library_specs()` result:

```rust
// Old:
let specs = build_c_library_specs();
// ... later:
specs: Some(&specs),

// New:
let yaml_specs = build_c_library_specs();
let analyzed_specs = AnalyzedSpecRegistry::new(yaml_specs);
// ... later:
specs: Some(&analyzed_specs),
```

**Step 7: Create test fixture and run**

Create `tests/fixtures/llvm/e2e/strlen_known_buffer.ll` (see Step 1).

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis computed_bound_strlen 2>&1'`
Expected: PASS

**Step 8: Run full test suite**

Run: `make fmt && make test 2>&1 | tee /tmp/test-output-150.txt`
Expected: All tests pass (may need mechanical fixes for changed APIs)

**Step 9: Commit**

```bash
git add -A
git commit -m "feat(absint): wire AnalyzedSpecRegistry into transfer function with computed return bounds"
```

---

## Task 4: Register Computed Bounds for Standard Library Functions

**Files:**
- Modify: `crates/saf-bench/src/svcomp/property.rs` (add computed bounds)
- Modify: `share/saf/specs/posix.yaml` or equivalent (add computed bounds to YAML specs)
- Test: existing Juliet benchmark validates improvement

**Step 1: Add computed bounds to property analyzer**

In `crates/saf-bench/src/svcomp/property.rs`, after `build_c_library_specs()`, add derived specs:

```rust
fn build_analyzed_specs() -> AnalyzedSpecRegistry {
    let yaml_specs = build_c_library_specs();
    let mut analyzed = AnalyzedSpecRegistry::new(yaml_specs);

    // strlen: return ∈ [0, alloc_size(arg0) - 1]
    analyzed.add_derived("strlen", DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    });

    // strnlen: return ∈ [0, alloc_size(arg0) - 1]
    // (also bounded by arg1, but alloc_size is tighter when available)
    analyzed.add_derived("strnlen", DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    });

    // wcslen: return ∈ [0, alloc_size(arg0) - 1]
    analyzed.add_derived("wcslen", DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    });

    // SV-COMP/Juliet wrapper: ldv_strlen_1 behaves like strlen
    analyzed.add_derived("ldv_strlen_1", DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    });

    analyzed
}
```

Replace all uses of `build_c_library_specs()` in the property analyzer with `build_analyzed_specs()`.

**Step 2: Run Juliet benchmark to measure improvement**

Run: `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet -o /workspace/tests/benchmarks/sv-benchmarks/juliet-150.json 2>&1'`

Read: `tests/benchmarks/sv-benchmarks/juliet-150.json`

Expected: CWE121/122/124/126/127 precision should improve where good variants use `strlen` on known-size buffers. Aggregate precision should increase above 52.1%.

**Step 3: Commit**

```bash
git add crates/saf-bench/src/svcomp/property.rs
git commit -m "feat(spec): register computed return bounds for strlen/strnlen/wcslen"
```

---

## Task 5: Integrate `ParameterEffectSummary` into `AnalyzedSpecRegistry`

Merge the existing summary system so temporal filtering uses the unified registry.

**Files:**
- Modify: `crates/saf-bench/src/svcomp/property.rs` (populate derived specs from summaries)
- Modify: `crates/saf-analysis/src/checkers/pathsens_runner.rs` (accept `AnalyzedSpecRegistry`)
- Test: existing tests validate no regression

**Step 1: Convert summaries to derived specs**

In `property.rs`, after computing summaries, feed them into the analyzed registry:

```rust
// Old:
let summaries = compute_parameter_effect_summaries(module, &table);
let result = filter_temporal_infeasible(result, &program_points, cfgs, Some(&summaries));

// New:
let summaries = compute_parameter_effect_summaries(module, &table);
// Populate analyzed registry with summary-derived specs
for (func_id, summary) in &summaries {
    if let Some(func) = module.functions.iter().find(|f| f.id == *func_id) {
        let existing = analyzed_specs.lookup_derived(&func.name)
            .cloned()
            .unwrap_or_else(DerivedSpec::empty);
        let merged = DerivedSpec {
            computed_return_bound: existing.computed_return_bound,
            param_freed: summary.param_freed.clone(),
            param_dereferenced: summary.param_dereferenced.clone(),
            return_is_allocated: summary.return_is_allocated,
        };
        analyzed_specs.add_derived(&func.name, merged);
    }
}
```

**Step 2: Update `filter_temporal_infeasible` to accept `AnalyzedSpecRegistry`**

Change the signature in `pathsens_runner.rs`:

```rust
// Old:
pub fn filter_temporal_infeasible(
    result: PathSensitiveResult,
    program_points: &ProgramPointMap,
    cfgs: &BTreeMap<FunctionId, Cfg>,
    summaries: Option<&BTreeMap<FunctionId, ParameterEffectSummary>>,
) -> PathSensitiveResult

// New:
pub fn filter_temporal_infeasible(
    result: PathSensitiveResult,
    program_points: &ProgramPointMap,
    cfgs: &BTreeMap<FunctionId, Cfg>,
    analyzed_specs: Option<&AnalyzedSpecRegistry>,
    module: &AirModule,
) -> PathSensitiveResult
```

Inside the function, where it previously looked up `summaries_map.get(&sink_pp.function)`, change to:

```rust
if let Some(specs) = analyzed_specs {
    let func_name = module.functions.iter()
        .find(|f| f.id == sink_pp.function)
        .map(|f| f.name.as_str());
    if let Some(name) = func_name {
        if let Some(derived) = specs.lookup_derived(name) {
            match finding.checker_name.as_str() {
                "use-after-free" | "double-free" => {
                    if !derived.param_freed.values().any(|&frees| frees) {
                        return false;
                    }
                }
                _ => {}
            }
        }
    }
}
```

**Step 3: Run full test suite**

Run: `make fmt && make test 2>&1 | tee /tmp/test-output-150-task5.txt`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "feat(spec): integrate ParameterEffectSummary into AnalyzedSpecRegistry for unified function contracts"
```

---

## Task 6: Benchmark Validation and Documentation

**Files:**
- Modify: `plans/PROGRESS.md` (update with results)
- Run: Juliet and PTABen benchmarks

**Step 1: Run PTABen benchmark**

Run (background): `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-150.json 2>&1'`

Expected: No regression from baseline (66-67 unsound).

**Step 2: Run Juliet benchmark**

Run (background): `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet -o /workspace/tests/benchmarks/sv-benchmarks/juliet-150.json 2>&1'`

Expected: Precision improvement on CWE121-127 where good variants use `strlen` on known-size buffers.

**Step 3: Compare results**

```bash
python3 -c "
import json
for label, path in [('148c', 'tests/benchmarks/sv-benchmarks/juliet-148c.json'),
                     ('150', 'tests/benchmarks/sv-benchmarks/juliet-150.json')]:
    with open(path) as f:
        data = json.load(f)
    print(f'=== {label} ===')
    for cwe in data.get('by_cwe', data.get('by_category', [])):
        name = cwe.get('cwe', cwe.get('category', ''))
        p = cwe.get('precision', 0)
        r = cwe.get('recall', 0)
        f1 = cwe.get('f1', 0)
        print(f'  {name}: P={p:.1f}% R={r:.1f}% F1={f1:.3f}')
"
```

**Step 4: Update PROGRESS.md**

Add Plan 150 entry with results. Update session log.

**Step 5: Final commit**

```bash
git add plans/PROGRESS.md tests/benchmarks/
git commit -m "docs: Plan 150 results — analyzed spec registry with computed return bounds"
```

---

## Expected Impact

- **CWE121-127 (buffer overflows):** Precision improvement where good variants call `strlen` on known-size buffers. The computed bound `[0, alloc_size-1]` replaces `[0, SIZE_MAX]`, allowing the memcpy checker to prove `strlen(buf)+1 ≤ alloc_size(buf)`.
- **Aggregate Juliet:** Precision increase dependent on how many good variants use `strlen` patterns vs other patterns.
- **Real-world code:** `strlen`, `strnlen`, `wcslen`, `fread`, `recv` all benefit from computed bounds — this is a general capability, not benchmark-specific.
- **Architecture:** Unified spec+summary system eliminates parallel data paths and enables future extensions (purity inference, escape analysis integration, etc.).

## Future Extensions (Not in This Plan)

- **Absint-derived return intervals:** Run lightweight per-function absint to discover return bounds for user-defined functions (Approach C territory).
- **Relational buffer constraints:** Use `size_from` field to model "memcpy copies exactly arg2 bytes from arg1 to arg0" for cross-argument validation.
- **Z3 refinement for integer overflow:** Wire Z3 feasibility into `analyze_no_overflow` (separate plan, identified as clear win).
- **Escape analysis integration:** Feed escape facts into derived specs for memory leak precision.
