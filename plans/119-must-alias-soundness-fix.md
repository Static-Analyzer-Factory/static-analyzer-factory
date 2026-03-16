# Plan 119: Must-Alias Soundness Fix — Multiplicity Tracking

**Epic:** Soundness
**Status:** approved
**Issue:** `saf-analysis-docs/soundness-issues/must-alias-singleton-pts.md`

## Problem

SAF's `may_alias()` returns `AliasResult::Must` when two pointers have identical singleton points-to sets (`pts(p) == pts(q)` and `|pts(p)| == 1`). This is **unsound** because a single abstract location can represent multiple distinct concrete runtime objects (e.g., a malloc inside a wrapper function called twice, or a heap allocation inside a loop).

### Affected locations (3 sites, identical pattern)

| File | Function | Line |
|------|----------|------|
| `crates/saf-analysis/src/pta/result.rs` | `PtaResult::may_alias()` | 159 |
| `crates/saf-analysis/src/cspta/solver.rs` | `CsptaResult::may_alias()` | 174 |
| `crates/saf-analysis/src/cspta/solver.rs` | `CsptaResult::may_alias_any()` | 200 |

## Solution: Multiplicity-Aware Must-Alias

Add an `AllocationMultiplicity` classification to each abstract object. Only return `Must` when the singleton location is provably `Unique` (represents exactly one concrete object). Otherwise return `May`.

### Classification rules

- **Unique:** globals, stack allocas in non-recursive functions outside loops, heap allocs at call sites executed at most once (single call-site, non-recursive, not in a loop)
- **Summary (default):** heap allocs in loops, allocs in functions called from multiple call sites, allocs in recursive functions, anything where k-limited context collapses

### Conservative default

`Summary` is the default. A location is `Unique` only when all conditions are proven. This ensures soundness — if we can't prove uniqueness, we assume the location may represent multiple objects.

---

## Agent Decomposition

The implementation is split into **4 independent agent tasks** plus a **leader coordination task**. Each agent works on a self-contained scope with no file-level conflicts.

---

### Agent 1: Add `AllocationMultiplicity` to `LocationFactory`

**Scope:** `crates/saf-analysis/src/pta/location.rs` only

**What to do:**

1. Add the enum after the `MemoryRegion` enum (around line 262):

```rust
/// Whether an abstract allocation site represents a unique concrete object
/// or a summary of multiple possible objects.
///
/// Used by alias analysis: `Must` alias is only sound when both pointers'
/// singleton location is `Unique`. `Summary` locations always yield `May`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AllocationMultiplicity {
    /// Provably one concrete object (e.g., a global, or a non-loop
    /// stack alloca in a non-recursive function called once).
    Unique,
    /// May represent multiple concrete objects (default — sound fallback).
    #[default]
    Summary,
}
```

2. Add a `multiplicities: BTreeMap<ObjId, AllocationMultiplicity>` field to `LocationFactory` (alongside the existing `regions` field).

3. Add these methods to `LocationFactory`:

```rust
/// Set the allocation multiplicity for a base object.
pub fn set_multiplicity(&mut self, obj: ObjId, mult: AllocationMultiplicity) {
    self.multiplicities.insert(obj, mult);
}

/// Get the multiplicity for a location.
/// Defaults to `Summary` if not explicitly classified.
#[must_use]
pub fn multiplicity(&self, loc: LocId) -> AllocationMultiplicity {
    self.locations
        .get(&loc)
        .and_then(|location| self.multiplicities.get(&location.obj))
        .copied()
        .unwrap_or_default()
}

/// Get the multiplicity map (for transferring to `PtaResult`).
#[must_use]
pub fn multiplicities(&self) -> &BTreeMap<ObjId, AllocationMultiplicity> {
    &self.multiplicities
}
```

4. Initialize `multiplicities: BTreeMap::new()` in `LocationFactory::new()`.

5. Add unit tests in the existing `#[cfg(test)] mod tests` block of `location.rs`:
   - Test that default multiplicity is `Summary`
   - Test `set_multiplicity` / `multiplicity` round-trip
   - Test that field locations inherit their parent object's multiplicity

**Files touched:** `crates/saf-analysis/src/pta/location.rs`
**No dependencies on other agents.**

---

### Agent 2: Multiplicity Classification Pass

**Scope:** New file `crates/saf-analysis/src/pta/multiplicity.rs` + mod registration in `crates/saf-analysis/src/pta/mod.rs`

**What to do:**

1. Create `crates/saf-analysis/src/pta/multiplicity.rs` with a single public function:

```rust
//! Allocation multiplicity classification.
//!
//! Determines whether each abstract allocation site represents a unique
//! concrete object or a summary of multiple objects. Used by alias
//! analysis to decide if singleton points-to sets imply must-alias.

use std::collections::BTreeSet;

use saf_core::air::AirModule;
use saf_core::ids::{FunctionId, InstId, ObjId};

use super::constraint::ConstraintSet;
use super::location::{AllocationMultiplicity, LocationFactory, MemoryRegion};
use crate::cfg::{Cfg, build_cfg};
use crate::absint::fixpoint::detect_loop_headers;

/// Classify allocation multiplicity for all objects in the factory.
///
/// Call this after constraint extraction and before building `PtaResult`.
/// Uses conservative heuristics:
/// - Globals are always `Unique`
/// - Stack allocas are `Unique` if not in a loop
/// - Heap allocs are `Unique` if (a) not in a loop AND (b) the enclosing
///   function has at most one call site in the module
/// - Everything else is `Summary` (the default)
///
/// Recursive functions are not handled yet — all their allocations stay
/// `Summary` by default since we don't classify them as `Unique`.
pub fn classify_multiplicity(
    module: &AirModule,
    factory: &mut LocationFactory,
    constraints: &ConstraintSet,
) {
    // Phase 1: Build per-function call-site count
    let call_site_counts = count_call_sites(module);

    // Phase 2: Build per-function loop block sets
    let loop_blocks = find_loop_blocks(module);

    // Phase 3: Classify each object
    for func in &module.functions {
        for block in &func.blocks {
            for inst in &block.instructions {
                let obj = ObjId::new(inst.id.raw());
                let region = factory.region_by_obj(obj);

                match region {
                    MemoryRegion::Global => {
                        // Globals are unique — one instance in the program
                        factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
                    }
                    MemoryRegion::Stack => {
                        // Stack alloca is unique if not inside a loop
                        if !is_in_loop_block(block.id, &loop_blocks, &func.id) {
                            factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
                        }
                        // Otherwise stays Summary (default)
                    }
                    MemoryRegion::Heap => {
                        // Heap alloc is unique if:
                        // (a) not in a loop block, AND
                        // (b) enclosing function is called from <= 1 site
                        let in_loop = is_in_loop_block(block.id, &loop_blocks, &func.id);
                        let multi_caller = call_site_counts
                            .get(&func.id)
                            .map_or(false, |&count| count > 1);
                        if !in_loop && !multi_caller {
                            factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
                        }
                    }
                    MemoryRegion::Unknown => {
                        // Unknown region stays Summary (default)
                    }
                }
            }
        }
    }
}
```

2. Implement helper functions in the same file:

- `count_call_sites(module: &AirModule) -> BTreeMap<FunctionId, usize>`: Walk all instructions, count how many `Call`/`IndirectCall` sites target each function.
- `find_loop_blocks(module: &AirModule) -> BTreeMap<FunctionId, BTreeSet<BlockId>>`: For each function, build CFG, run `detect_loop_headers()`, then expand loop headers to include all blocks dominated by the header that have a back-edge (or simplify: just use the header set — any alloc in a loop header block is in a loop). NOTE: A simpler initial approach is to check if the instruction's block is a loop header OR has a predecessor that is a loop header. The simplest sound approach: use `detect_loop_headers` to find headers, then conservatively mark ALL blocks in a function as "in a loop" if the function has ANY loop header. This over-approximates but is sound and simple. A more precise version can come later.
- `is_in_loop_block(block: BlockId, loop_blocks: &BTreeMap<FunctionId, BTreeSet<BlockId>>, func: &FunctionId) -> bool`

3. Add `pub mod multiplicity;` to `crates/saf-analysis/src/pta/mod.rs`.

4. Also add a `region_by_obj` method to `LocationFactory` (or instruct Agent 1 to add it — coordinate via the leader). This simply does: `self.regions.get(&obj).copied().unwrap_or_default()`. If Agent 1 doesn't add this, Agent 2 should add it in `location.rs` (this is the one allowed cross-agent touch point, and it's additive-only).

5. Add unit tests:
   - Test that globals are classified as `Unique`
   - Test that heap allocs in single-caller, non-loop functions are `Unique`
   - Test that heap allocs in multi-caller functions are `Summary`

**Files touched:** `crates/saf-analysis/src/pta/multiplicity.rs` (new), `crates/saf-analysis/src/pta/mod.rs` (one line)
**Coordinate with Agent 1:** needs `AllocationMultiplicity` enum and `set_multiplicity()` / `multiplicity()` / `region_by_obj()` on `LocationFactory`.

---

### Agent 3: Update Must-Alias Checks in PTA and CSPTA

**Scope:** `crates/saf-analysis/src/pta/result.rs` + `crates/saf-analysis/src/cspta/solver.rs`

**What to do:**

1. **`PtaResult` struct** (`result.rs` line ~80): Add a `multiplicities: BTreeMap<ObjId, AllocationMultiplicity>` field.

2. **`PtaResult::new()`** (`result.rs` line ~94): Extract multiplicities from factory:
   ```rust
   let multiplicities = factory.multiplicities().clone();
   ```

3. Add a private helper:
   ```rust
   /// Check if a location is provably unique (one concrete object).
   fn is_unique(&self, loc: LocId) -> bool {
       self.locations
           .get(&loc)
           .and_then(|location| self.multiplicities.get(&location.obj))
           .copied()
           .unwrap_or_default()
           == AllocationMultiplicity::Unique
   }
   ```

4. **Update `may_alias()`** (`result.rs` line 159): Change the singleton check:
   ```rust
   if p_set == q_set && p_set.len() == 1 {
       let loc = *p_set.iter().next().unwrap();
       if self.is_unique(loc) {
           AliasResult::Must
       } else {
           // Summary location — singleton set doesn't guarantee must-alias
           AliasResult::May
       }
   }
   ```

5. **Update `CsptaResult::may_alias()`** (`cspta/solver.rs` line 174): Same pattern. `CsptaResult` needs access to multiplicities. Add a `multiplicities: BTreeMap<ObjId, AllocationMultiplicity>` field to `CsptaResult`, populated during construction. Add the same `is_unique()` helper. Guard the singleton check:
   ```rust
   } else if ps == qs && ps.len() == 1 {
       let loc = *ps.iter().next().unwrap();
       if self.is_unique(loc) {
           crate::pta::AliasResult::Must
       } else {
           crate::pta::AliasResult::May
       }
   ```

6. **Update `CsptaResult::may_alias_any()`** (`cspta/solver.rs` line 200): Same guard.

7. **Update tests** in `result.rs`:
   - Update `may_alias_identical_sets_must` to set the location's object as `Unique` so it still returns `Must`.
   - Add new test `may_alias_identical_singleton_summary_returns_may`: Create a singleton set with a `Summary` object, assert result is `May`.
   - Add new test `may_alias_identical_singleton_unique_returns_must`: Explicit `Unique` classification, assert `Must`.

**Files touched:** `crates/saf-analysis/src/pta/result.rs`, `crates/saf-analysis/src/cspta/solver.rs`
**Coordinate with Agent 1:** needs `AllocationMultiplicity` type and `multiplicities()` accessor on `LocationFactory`.

---

### Agent 4: Wire Classification into Analysis Pipeline + E2E Test

**Scope:** `crates/saf-analysis/src/pta/context.rs`, `crates/saf-analysis/src/cspta/solver.rs` (construction path only — no overlap with Agent 3's alias-check changes), E2E test file

**What to do:**

1. **CI PTA pipeline** (`pta/context.rs`, `analyze_with_specs` around line 84): After `extract_constraints()` and before solving, call:
   ```rust
   // Classify allocation multiplicity for must-alias soundness
   super::multiplicity::classify_multiplicity(module, &mut self.factory, &constraints);
   ```

2. **CSPTA pipeline**: Find where `CsptaResult` is constructed in `cspta/solver.rs`. Pass `factory.multiplicities().clone()` to the result struct. (This is in the construction/return path, not the `may_alias` methods that Agent 3 touches.)

3. **E2E test**: Add a test in `crates/saf-analysis/tests/` (e.g., `must_alias_soundness_e2e.rs` or add to `pta_integration.rs`) that:
   - Creates an AIR-JSON fixture with a wrapper function pattern (function `wrapper` calls heap alloc, `main` calls `wrapper` twice)
   - Runs PTA
   - Asserts the alias result for the two pointers is `May`, not `Must`
   - Also tests that a global variable singleton still returns `Must`

   Alternatively, compile the existing `saf-analysis-docs/soundness-issues/tests/must_alias_unsound.c` to LLVM IR and use `load_ll_fixture` if that's simpler.

4. Copy `saf-analysis-docs/soundness-issues/tests/must_alias_unsound.ll` to `tests/fixtures/llvm/e2e/must_alias_unsound.ll` for the E2E test fixture.

**Files touched:** `crates/saf-analysis/src/pta/context.rs`, `crates/saf-analysis/src/cspta/solver.rs` (construction path), E2E test file, test fixture
**Depends on:** Agents 1, 2, 3 (all must complete first)

---

### Leader: Coordination and Verification

The leader agent orchestrates the work:

1. **Dispatch Agents 1, 2, 3 in parallel** (they touch non-overlapping files)
2. **After 1+2+3 complete, dispatch Agent 4** (wiring + E2E test)
3. **Run `make fmt && make lint`** to fix formatting
4. **Run `make test`** to verify all unit + integration tests pass
5. **Run PTABen benchmarks** to check for regressions:
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
   ```
6. **Update `plans/PROGRESS.md`** with results

### Leader Context Budget

The leader should NOT read full source files. It only needs to:
- Read this plan
- Dispatch agents with the task descriptions above
- Read agent outputs (success/failure)
- Run build/test/lint commands
- Update PROGRESS.md

---

## Dependency Graph

```
Agent 1 (LocationFactory)  ──┐
Agent 2 (multiplicity.rs)  ──┤──► Agent 4 (wiring + E2E) ──► Leader verification
Agent 3 (alias checks)     ──┘
```

Agents 1, 2, 3 are independent and can run in parallel.
Agent 4 depends on all three completing.

## Risk Assessment

- **Low risk:** This is a conservative change. The default is `Summary`, which downgrades `Must` to `May`. No existing `May` or `No` results change.
- **PTABen impact:** Tests that assert `MUSTALIAS` for wrapper-allocated pointers will now get `May` instead. These are currently passing due to the bug — they should have been failing. If PTABen expects `MUSTALIAS`, those specific oracle assertions will regress. This is correct behavior (the oracles were wrong to expect `Must` for summary locations).
- **Performance:** Negligible — one extra BTreeMap lookup per must-alias check, and a one-time O(n) classification pass over instructions.
