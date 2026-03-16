# Plan 137: Wire Spec Constraints into CG Refinement + Fix modifies:true Gap

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the CG refinement pipeline use existing YAML function specs to generate PTA constraints for external library functions, and fix the empty `modifies:true` handler so all spec-described pointer effects are modeled.

**Architecture:** The spec infrastructure already exists — YAML specs define pointer effects for 50+ libc functions, and `extract_spec_constraints()` generates PTA constraints from them. The CG refinement pipeline (`refine()`) never calls this. We add a `specs: Option<&SpecRegistry>` parameter to `refine()`, call `extract_spec_constraints()` after `extract_constraints()`, and fill in the empty `modifies:true` handler body with a conservative Store constraint using synthetic fresh objects.

**Tech Stack:** Rust, saf-core spec system, saf-analysis PTA constraint extraction

**Key Insight:** PTABen already uses specs via `PtaContext::analyze_with_specs()`. This fix targets the CG refinement path used by real-world analysis (CruxBC benchmarks, CLI, WASM, Python).

---

## Task 1: Fix the modifies:true Gap in spec_constraints.rs

The `modifies:true` handler at lines 117-129 has an empty body. When a param is marked `modifies: true` but no taint propagation targets it, no Store constraint is generated. This silently drops pointer writes for functions like `posix_memalign(*ptr = fresh)` and `strtol(*endptr = ptr)`.

**Files:**
- Modify: `crates/saf-analysis/src/pta/spec_constraints.rs:116-129`
- Test: `crates/saf-analysis/src/pta/spec_constraints.rs` (inline tests)

### Step 1: Write failing test

Add to the `#[cfg(test)] mod tests` block in `spec_constraints.rs`:

```rust
#[test]
fn test_modifies_without_taint_generates_store() {
    let mut module = make_module();

    // Add posix_memalign declaration
    let func_id = FunctionId::derive(b"posix_memalign");
    let mut func_decl = AirFunction::new(func_id, "posix_memalign");
    func_decl.is_declaration = true;
    module.functions.push(func_decl);

    // Add caller
    let caller_id = FunctionId::derive(b"caller");
    let mut caller = AirFunction::new(caller_id, "caller");
    let mut block = AirBlock::new(BlockId::derive(b"entry"));

    let call_inst_id = InstId::derive(b"call_posix_memalign");
    let memptr_arg = ValueId::derive(b"memptr");
    let align_arg = ValueId::derive(b"align");
    let size_arg = ValueId::derive(b"size");

    block.instructions.push(
        Instruction::new(
            call_inst_id,
            Operation::CallDirect { callee: func_id },
        )
        .with_operands(vec![memptr_arg, align_arg, size_arg]),
    );
    block
        .instructions
        .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
    caller.blocks.push(block);
    module.functions.push(caller);

    // Create spec: param 0 modifies (writes a pointer through *memptr)
    // No taint propagation — the modifies:true handler alone must generate constraints
    let mut registry = SpecRegistry::new();
    let mut spec = FunctionSpec::new("posix_memalign");
    spec.params = vec![
        ParamSpec {
            index: 0,
            modifies: Some(true),
            ..ParamSpec::new(0)
        },
        ParamSpec::new(1),
        ParamSpec::new(2),
    ];
    registry.add(spec).unwrap();

    let mut factory = make_factory();
    let mut constraints = ConstraintSet::default();

    extract_spec_constraints(&module, &registry, &mut factory, &mut constraints);

    // Should have generated:
    // 1. An Addr constraint for the synthetic value → fresh object
    // 2. A Store constraint: *memptr = synthetic_value
    assert_eq!(constraints.addr.len(), 1, "expected 1 Addr for synthetic fresh object");
    assert_eq!(constraints.store.len(), 1, "expected 1 Store for *memptr = synthetic");
    let store = constraints.store.iter().next().unwrap();
    assert_eq!(store.dst_ptr, memptr_arg, "Store dst_ptr should be the memptr argument");
}
```

### Step 2: Run test to verify it fails

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis test_modifies_without_taint_generates_store'
```

Expected: FAIL — the current empty body generates no constraints.

### Step 3: Implement the modifies:true handler

In `spec_constraints.rs`, add `ValueId` to the imports (line 10) and `saf_core::id::make_id` import:

```rust
use saf_core::ids::{FunctionId, ObjId, ValueId};
use saf_core::id::make_id;
```

Replace the empty `modifies:true` handler (lines 116-129) with:

```rust
    // Handle parameter modifications (modifies: true without taint coverage).
    // When a param is modified but no taint propagation targets it,
    // conservatively model: the callee writes an unknown pointer through the param.
    // This covers cases like posix_memalign(*ptr = fresh) and strtol(*endptr = ptr).
    for param_spec in &spec.params {
        if param_spec.does_modify() {
            if let Some(&arg_ptr) = inst.operands.get(param_spec.index as usize) {
                // Check if taint propagation already covers this param as a destination.
                // If so, the taint handler already generated a Store — skip to avoid duplicates.
                let covered_by_taint = spec.taint.as_ref().is_some_and(|t| {
                    t.propagates.iter().any(|p| {
                        p.to_locations().iter().any(|loc| {
                            matches!(loc, saf_core::spec::TaintLocation::Param(idx) if *idx == param_spec.index)
                        })
                    })
                });
                if covered_by_taint {
                    continue;
                }

                // Create a synthetic value that the callee "writes" through this pointer.
                // Use inst ID + param index as seed for deterministic, unique IDs.
                let seed = [
                    inst.id.raw().to_le_bytes(),
                    (param_spec.index as u128).to_le_bytes(),
                ]
                .concat();
                let synthetic_val =
                    ValueId::new(make_id("stub_modifies_val", &seed));
                let synthetic_obj =
                    ObjId::new(make_id("stub_modifies_obj", &seed));
                let loc = factory.get_or_create(synthetic_obj, FieldPath::empty());

                // Addr: synthetic_val → fresh object
                constraints
                    .addr
                    .insert(AddrConstraint { ptr: synthetic_val, loc });
                // Store: *arg_ptr = synthetic_val
                constraints
                    .store
                    .insert(StoreConstraint { dst_ptr: arg_ptr, src: synthetic_val });
            }
        }
    }
```

### Step 4: Run test to verify it passes

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis test_modifies_without_taint_generates_store'
```

Expected: PASS

### Step 5: Run all spec_constraints tests

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis spec_constraints'
```

Expected: All 4 existing tests + new test pass. Verify the `test_taint_propagation_generates_store_constraint` test still passes (the `strcpy` case with taint should NOT double-generate Store constraints).

### Step 6: Commit

```bash
git add crates/saf-analysis/src/pta/spec_constraints.rs
git commit -m "fix(pta): generate Store constraints for modifies:true params without taint coverage"
```

---

## Task 2: Add specs Parameter to refine() and Wire extract_spec_constraints

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs:107` (function signature)
- Modify: `crates/saf-analysis/src/cg_refinement.rs:149-157` (add spec extraction after constraint extraction)

### Step 1: Update refine() signature and add spec extraction

In `crates/saf-analysis/src/cg_refinement.rs`:

Add import at top (around line 31-34):

```rust
use saf_core::spec::SpecRegistry;
use crate::pta::spec_constraints::extract_spec_constraints;
```

Change function signature (line 107):

```rust
pub fn refine(
    module: &AirModule,
    config: &RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> RefinementResult {
```

After `extract_constraints` (line 150), before `constraint_counts` (line 151), add spec extraction:

```rust
    let mut factory = LocationFactory::new(config.field_sensitivity.clone());
    let mut constraints = extract_constraints(module, &mut factory);

    // Extract additional constraints from function specs for external library calls
    if let Some(specs) = specs {
        extract_spec_constraints(module, specs, &mut factory, &mut constraints);
    }

    let constraint_counts = [
```

Note: `constraints` must change from `let` to `let mut` on line 150.

### Step 2: Update all callers — internal tests in cg_refinement.rs

There are ~6 calls to `refine()` in the `#[cfg(test)]` section of `cg_refinement.rs`. Update each to pass `None`:

```rust
let result = refine(&module, &config, None);
```

And the two determinism test calls:
```rust
let r1 = refine(&module, &config, None);
let r2 = refine(&module, &config, None);
```

### Step 3: Update e2e test callers

In `crates/saf-analysis/tests/cg_refinement_e2e.rs`, update all ~9 calls:

```rust
let result = refine(&module, &config, None);
```

And:
```rust
let r1 = refine(&module, &config, None);
let r2 = refine(&module, &config, None);
```

### Step 4: Update saf-bench callers

In `crates/saf-bench/src/cruxbc.rs:321`:

```rust
let specs = SpecRegistry::load().unwrap_or_else(|_| SpecRegistry::new());
let mut refinement_result = refine(module, &refinement_config, Some(&specs));
```

In `crates/saf-bench/src/ptaben.rs:428`:

```rust
let specs = SpecRegistry::load().unwrap_or_else(|_| SpecRegistry::new());
let refinement_result = refine(module, &refinement_config, Some(&specs));
```

Make sure `SpecRegistry` is imported in both files (`use saf_core::spec::SpecRegistry;`). Check if it's already imported from the existing `analyze_with_specs` usage.

### Step 5: Update Python bindings

In `crates/saf-python/src/cg_refinement.rs:183`:

```rust
let result = saf_analysis::cg_refinement::refine(module, &config, None);
```

(Python bindings don't load specs for CG refinement currently — can be enhanced later.)

### Step 6: Update WASM bindings

In `crates/saf-wasm/src/lib.rs:152`:

```rust
} = saf_analysis::cg_refinement::refine(module, &refinement_config, None);
```

(WASM has its own spec loading pipeline — can be wired separately.)

### Step 7: Compile check

```bash
docker compose run --rm dev sh -c 'cargo check --workspace'
```

Expected: Compiles without errors.

### Step 8: Run all tests

```bash
make fmt && make test
```

Expected: All tests pass. No regressions.

### Step 9: Commit

```bash
git add crates/saf-analysis/src/cg_refinement.rs \
    crates/saf-analysis/tests/cg_refinement_e2e.rs \
    crates/saf-bench/src/cruxbc.rs \
    crates/saf-bench/src/ptaben.rs \
    crates/saf-python/src/cg_refinement.rs \
    crates/saf-wasm/src/lib.rs
git commit -m "feat(pta): wire spec constraints into CG refinement pipeline

Adds specs parameter to refine() so the main analysis pipeline
uses YAML function specs (strcpy, strdup, getenv, fopen, etc.)
to generate PTA constraints for external library calls.
Previously, specs were only used via PtaContext::analyze_with_specs()."
```

---

## Task 3: Measure Impact

### Step 1: Run PTABen benchmarks (background)

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
```

Expected: 69 Unsound (unchanged — PTABen uses `analyze_with_specs`, not `refine()`).

### Step 2: Run CruxBC bash benchmark

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --programs bash --dir tests/benchmarks/cruxbc'
```

Compare constraint counts and PTA metrics with previous baseline (Plan 136):
- Previous: Addr=11,573, Copy=41,293, Load=11,785, Store=3,571, GEP=20,736
- Expected: Copy/Load/Store should increase (spec constraints for externals add new Copy/Store edges)

### Step 3: Document results in PROGRESS.md

Update `plans/PROGRESS.md` with Plan 137 results.

---

## Summary of Changes

| File | Change |
|------|--------|
| `crates/saf-analysis/src/pta/spec_constraints.rs` | Fill in `modifies:true` handler with synthetic Addr+Store |
| `crates/saf-analysis/src/cg_refinement.rs` | Add `specs` param to `refine()`, call `extract_spec_constraints` |
| `crates/saf-analysis/tests/cg_refinement_e2e.rs` | Pass `None` for specs in all 9 test calls |
| `crates/saf-bench/src/cruxbc.rs` | Load specs and pass to `refine()` |
| `crates/saf-bench/src/ptaben.rs` | Load specs and pass to `refine()` |
| `crates/saf-python/src/cg_refinement.rs` | Pass `None` for specs |
| `crates/saf-wasm/src/lib.rs` | Pass `None` for specs |

**Estimated impact:** More constraints generated for external library calls in CG refinement, improving analysis completeness for real-world programs. No PTABen regression expected.
