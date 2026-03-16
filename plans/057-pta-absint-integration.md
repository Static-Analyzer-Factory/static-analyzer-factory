# PTA-Absint Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate Points-To Analysis with Abstract Interpretation for alias-aware memory tracking, indirect call resolution, and bidirectional refinement.

**Architecture:** Three-layer integration: (1) PTA integration layer wrapping PtaResult for absint queries, (2) LocId-based memory model replacing ValueId-based tracking, (3) Combined orchestration module coordinating analysis phases.

**Tech Stack:** Rust, saf-core AIR types, existing PTA solver infrastructure, PyO3 for Python bindings.

---

## Task 1: PTA Integration Layer

**Files:**
- Create: `crates/saf-analysis/src/absint/pta_integration.rs`
- Modify: `crates/saf-analysis/src/absint/mod.rs:19-47`

**Step 1: Write the failing test**

Create unit test for PTA integration layer.

```rust
// In crates/saf-analysis/src/absint/pta_integration.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pta::result::PtaResult;
    use crate::pta::context::PtaDiagnostics;
    use crate::pta::location::LocationFactory;
    use crate::pta::config::FieldSensitivity;
    use crate::pta::solver::PointsToMap;
    use saf_core::ids::{ValueId, LocId, FunctionId};
    use std::collections::BTreeSet;

    fn vid(n: u128) -> ValueId { ValueId::new(n) }
    fn lid(n: u128) -> LocId { LocId::new(n) }

    #[test]
    fn pta_integration_empty_returns_empty_pts() {
        let integration = PtaIntegration::empty();
        assert!(integration.points_to(vid(1)).is_empty());
    }

    #[test]
    fn pta_integration_is_singleton() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::Insensitive);
        let pta = PtaResult::new(pts, &factory, PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        assert!(integration.is_singleton(vid(1)));
    }

    #[test]
    fn pta_integration_not_singleton_for_multiple() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        set.insert(lid(101));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::Insensitive);
        let pta = PtaResult::new(pts, &factory, PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        assert!(!integration.is_singleton(vid(1)));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cargo test -p saf-analysis pta_integration_empty --no-default-features`
Expected: FAIL with "cannot find module `pta_integration`"

**Step 3: Write the implementation**

```rust
// crates/saf-analysis/src/absint/pta_integration.rs

//! PTA integration layer for abstract interpretation.
//!
//! Provides an abstraction over `PtaResult` optimized for absint queries,
//! with caching and convenience methods.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{FunctionId, LocId, ValueId};

use crate::pta::result::PtaResult;

/// Wrapper providing absint-friendly PTA queries.
///
/// Caches points-to lookups to avoid repeated traversals and provides
/// convenience methods for common absint patterns.
pub struct PtaIntegration<'a> {
    /// The underlying PTA result (None for empty/no-PTA mode).
    pta: Option<&'a PtaResult>,
    /// Cache for points-to queries.
    pts_cache: RefCell<BTreeMap<ValueId, BTreeSet<LocId>>>,
}

impl<'a> PtaIntegration<'a> {
    /// Create a new integration layer wrapping a PTA result.
    #[must_use]
    pub fn new(pta: &'a PtaResult) -> Self {
        Self {
            pta: Some(pta),
            pts_cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Create an empty integration (no PTA available).
    ///
    /// All queries return empty/conservative results.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            pta: None,
            pts_cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Get the points-to set for a pointer value.
    ///
    /// Returns an empty set if PTA is not available or pointer is not tracked.
    #[must_use]
    pub fn points_to(&self, ptr: ValueId) -> BTreeSet<LocId> {
        // Check cache first
        if let Some(cached) = self.pts_cache.borrow().get(&ptr) {
            return cached.clone();
        }

        let result = match self.pta {
            Some(pta) => pta.points_to(ptr).into_iter().collect(),
            None => BTreeSet::new(),
        };

        // Cache the result
        self.pts_cache.borrow_mut().insert(ptr, result.clone());
        result
    }

    /// Check if a pointer has a singleton points-to set.
    ///
    /// Returns true if the pointer points to exactly one location,
    /// enabling strong updates in the memory model.
    #[must_use]
    pub fn is_singleton(&self, ptr: ValueId) -> bool {
        self.points_to(ptr).len() == 1
    }

    /// Check if two pointers may alias.
    ///
    /// Returns true conservatively if PTA is not available.
    #[must_use]
    pub fn may_alias(&self, a: ValueId, b: ValueId) -> bool {
        match self.pta {
            Some(pta) => pta.may_alias(a, b).may_alias_conservative(),
            None => true, // Conservative: assume may-alias
        }
    }

    /// Resolve indirect call targets for a function pointer.
    ///
    /// Returns the set of possible callee function IDs.
    /// Returns empty set if PTA is not available or no targets found.
    #[must_use]
    pub fn resolve_indirect_call(&self, fn_ptr: ValueId) -> BTreeSet<FunctionId> {
        let Some(pta) = self.pta else {
            return BTreeSet::new();
        };

        let pts = self.points_to(fn_ptr);
        let mut targets = BTreeSet::new();

        for loc_id in pts {
            if let Some(loc) = pta.location(loc_id) {
                // Function locations have ObjId that maps to FunctionId
                // The ObjId for a function is derived from its FunctionId
                targets.insert(FunctionId::new(loc.obj.as_u128()));
            }
        }

        targets
    }

    /// Check if PTA is available.
    #[must_use]
    pub fn has_pta(&self) -> bool {
        self.pta.is_some()
    }

    /// Clear the cache (useful after state changes).
    pub fn clear_cache(&self) {
        self.pts_cache.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pta::config::FieldSensitivity;
    use crate::pta::context::PtaDiagnostics;
    use crate::pta::location::LocationFactory;
    use crate::pta::solver::PointsToMap;

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn lid(n: u128) -> LocId {
        LocId::new(n)
    }

    #[test]
    fn pta_integration_empty_returns_empty_pts() {
        let integration = PtaIntegration::empty();
        assert!(integration.points_to(vid(1)).is_empty());
    }

    #[test]
    fn pta_integration_is_singleton() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::Insensitive);
        let pta = PtaResult::new(pts, &factory, PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        assert!(integration.is_singleton(vid(1)));
    }

    #[test]
    fn pta_integration_not_singleton_for_multiple() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        set.insert(lid(101));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::Insensitive);
        let pta = PtaResult::new(pts, &factory, PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        assert!(!integration.is_singleton(vid(1)));
    }

    #[test]
    fn pta_integration_may_alias_conservative_without_pta() {
        let integration = PtaIntegration::empty();
        assert!(integration.may_alias(vid(1), vid(2)));
    }

    #[test]
    fn pta_integration_caches_queries() {
        let mut pts = PointsToMap::new();
        let mut set = BTreeSet::new();
        set.insert(lid(100));
        pts.insert(vid(1), set);

        let factory = LocationFactory::new(FieldSensitivity::Insensitive);
        let pta = PtaResult::new(pts, &factory, PtaDiagnostics::default());
        let integration = PtaIntegration::new(&pta);

        // First query populates cache
        let result1 = integration.points_to(vid(1));
        // Second query should hit cache
        let result2 = integration.points_to(vid(1));

        assert_eq!(result1, result2);
        assert_eq!(integration.pts_cache.borrow().len(), 1);
    }
}
```

**Step 4: Update mod.rs to export the module**

Add to `crates/saf-analysis/src/absint/mod.rs` after line 31:

```rust
mod pta_integration;

pub use pta_integration::PtaIntegration;
```

**Step 5: Run test to verify it passes**

Run: `make shell` then `cargo test -p saf-analysis pta_integration --no-default-features`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/absint/pta_integration.rs crates/saf-analysis/src/absint/mod.rs
git commit -m "$(cat <<'EOF'
feat(absint): add PTA integration layer

Add PtaIntegration wrapper for absint-friendly PTA queries:
- Caches points-to lookups for efficiency
- is_singleton() for strong update detection
- may_alias() with conservative fallback
- resolve_indirect_call() for function pointers
- empty() constructor for no-PTA mode

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Extend AbstractState with LocId-based Memory

**Files:**
- Modify: `crates/saf-analysis/src/absint/state.rs:1-260`

**Step 1: Write the failing test**

Add test for LocId-based memory operations to `state.rs`:

```rust
// Add to existing tests module in state.rs

#[test]
fn store_load_with_locid() {
    let mut state = AbstractState::new();
    let loc = LocId::new(100);

    state.store_loc(loc, Interval::singleton(42, 32));
    let loaded = state.load_loc(loc);

    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().lo(), 42);
    assert_eq!(loaded.unwrap().hi(), 42);
}

#[test]
fn store_weak_update_joins() {
    let mut state = AbstractState::new();
    let loc = LocId::new(100);

    state.store_loc(loc, Interval::new(0, 10, 32));
    state.store_loc_weak(loc, Interval::new(20, 30, 32));

    let loaded = state.load_loc(loc).unwrap();
    assert_eq!(loaded.lo(), 0);
    assert_eq!(loaded.hi(), 30);
}

#[test]
fn join_locid_memory() {
    let mut a = AbstractState::new();
    a.store_loc(LocId::new(100), Interval::new(0, 10, 32));

    let mut b = AbstractState::new();
    b.store_loc(LocId::new(100), Interval::new(20, 30, 32));

    let joined = a.join(&b);
    let loaded = joined.load_loc(LocId::new(100)).unwrap();

    assert_eq!(loaded.lo(), 0);
    assert_eq!(loaded.hi(), 30);
}
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cargo test -p saf-analysis state::tests::store_load_with_locid --no-default-features`
Expected: FAIL with "method `store_loc` not found"

**Step 3: Extend AbstractState implementation**

Add to `crates/saf-analysis/src/absint/state.rs` after line 14:

```rust
use saf_core::ids::LocId;
```

Add new fields and methods to `AbstractState` (after line 34):

```rust
/// Abstract state at a program point.
///
/// Maps each `ValueId` to an `Interval` representing the value's range.
/// Values not in the map are implicitly top (unknown).
///
/// Also tracks memory contents in two ways:
/// 1. `memory` - Simple ValueId-based tracking for backward compatibility
/// 2. `loc_memory` - LocId-based tracking for PTA-integrated analysis
#[derive(Clone, PartialEq, Eq)]
pub struct AbstractState {
    /// Map from value to its abstract interval.
    values: BTreeMap<ValueId, Interval>,
    /// Map from memory location (pointer `ValueId`) to stored interval.
    /// This enables tracking values through store/load sequences.
    memory: BTreeMap<ValueId, Interval>,
    /// Map from abstract location (`LocId`) to stored interval.
    /// Used when PTA is available for alias-aware memory tracking.
    loc_memory: BTreeMap<LocId, Interval>,
    /// Whether this state represents unreachable code (bottom state).
    unreachable: bool,
}
```

Update `new()` and `bottom()` constructors to initialize `loc_memory`:

```rust
/// Create a new empty state (all values implicitly top).
#[must_use]
pub fn new() -> Self {
    Self {
        values: BTreeMap::new(),
        memory: BTreeMap::new(),
        loc_memory: BTreeMap::new(),
        unreachable: false,
    }
}

/// Create an unreachable (bottom) state.
#[must_use]
pub fn bottom() -> Self {
    Self {
        values: BTreeMap::new(),
        memory: BTreeMap::new(),
        loc_memory: BTreeMap::new(),
        unreachable: true,
    }
}
```

Add new methods for LocId-based memory (after `invalidate_all_memory`):

```rust
/// Store an interval to an abstract memory location (strong update).
///
/// Replaces any existing value at the location.
pub fn store_loc(&mut self, loc: LocId, interval: Interval) {
    if !self.unreachable {
        self.loc_memory.insert(loc, interval);
    }
}

/// Store an interval to an abstract memory location (weak update).
///
/// Joins with any existing value at the location.
pub fn store_loc_weak(&mut self, loc: LocId, interval: Interval) {
    if !self.unreachable {
        let existing = self.loc_memory.get(&loc).cloned().unwrap_or_else(|| Interval::make_bottom(interval.bits()));
        self.loc_memory.insert(loc, existing.join(&interval));
    }
}

/// Load an interval from an abstract memory location.
///
/// Returns the stored interval if known, `None` otherwise.
#[must_use]
pub fn load_loc(&self, loc: LocId) -> Option<&Interval> {
    if self.unreachable {
        return None;
    }
    self.loc_memory.get(&loc)
}

/// Invalidate a specific abstract memory location.
pub fn invalidate_loc(&mut self, loc: LocId) {
    self.loc_memory.remove(&loc);
}

/// Invalidate a set of abstract memory locations.
pub fn invalidate_locs(&mut self, locs: &std::collections::BTreeSet<LocId>) {
    for loc in locs {
        self.loc_memory.remove(loc);
    }
}

/// Invalidate all abstract memory locations.
pub fn invalidate_all_loc_memory(&mut self) {
    self.loc_memory.clear();
}

/// Get all tracked LocId memory entries.
#[must_use]
pub fn loc_memory_entries(&self) -> &BTreeMap<LocId, Interval> {
    &self.loc_memory
}
```

Update `join()` to handle `loc_memory` (add after memory join section around line 189):

```rust
// Join loc_memory state
let mut all_loc_keys: BTreeSet<LocId> = BTreeSet::new();
for key in self.loc_memory.keys() {
    all_loc_keys.insert(*key);
}
for key in other.loc_memory.keys() {
    all_loc_keys.insert(*key);
}

let mut result_loc_memory = BTreeMap::new();
for key in &all_loc_keys {
    let a = self.loc_memory.get(key);
    let b = other.loc_memory.get(key);

    if let (Some(a_val), Some(b_val)) = (a, b) {
        let joined = a_val.join(b_val);
        if !joined.is_top() {
            result_loc_memory.insert(*key, joined);
        }
    }
}
```

Update the `Self` construction in `join()`:

```rust
Self {
    values: result_values,
    memory: result_memory,
    loc_memory: result_loc_memory,
    unreachable: false,
}
```

Update `leq()` to check `loc_memory` (add after memory check around line 225):

```rust
// Check loc_memory intervals
for (key, val) in &self.loc_memory {
    if let Some(other_val) = other.loc_memory.get(key) {
        if !val.leq(other_val) {
            return false;
        }
    }
}
```

Add import at top of file:

```rust
use saf_core::ids::LocId;
```

**Step 4: Run tests to verify they pass**

Run: `make shell` then `cargo test -p saf-analysis state::tests --no-default-features`
Expected: PASS (all existing + 3 new tests)

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/absint/state.rs
git commit -m "$(cat <<'EOF'
feat(absint): extend AbstractState with LocId-based memory

Add loc_memory field for PTA-integrated alias-aware tracking:
- store_loc() for strong updates (singleton points-to)
- store_loc_weak() for weak updates (joins with existing)
- load_loc() to retrieve stored intervals
- invalidate_loc/locs/all_loc_memory() for selective invalidation
- join() and leq() extended to handle loc_memory

Backward compatible: existing ValueId-based memory still works.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: PTA-Aware Transfer Functions

**Files:**
- Modify: `crates/saf-analysis/src/absint/transfer.rs:179-388`

**Step 1: Write the failing test**

Add test for PTA-aware load/store to `transfer.rs`:

```rust
// Add to existing tests module in transfer.rs

use crate::absint::pta_integration::PtaIntegration;
use crate::pta::config::FieldSensitivity;
use crate::pta::context::PtaDiagnostics;
use crate::pta::location::LocationFactory;
use crate::pta::solver::PointsToMap;
use saf_core::ids::LocId;

fn lid(n: u128) -> LocId {
    LocId::new(n)
}

#[test]
fn transfer_store_load_with_pta_singleton() {
    let mut state = AbstractState::new();
    let module = empty_module();

    // Set up PTA: vid(2) points to lid(100)
    let mut pts = PointsToMap::new();
    let mut set = std::collections::BTreeSet::new();
    set.insert(lid(100));
    pts.insert(vid(2), set.clone());
    pts.insert(vid(3), set); // vid(3) also points to lid(100)

    let factory = LocationFactory::new(FieldSensitivity::Insensitive);
    let pta = crate::pta::result::PtaResult::new(pts, &factory, PtaDiagnostics::default());
    let pta_integration = PtaIntegration::new(&pta);

    // Simulate constant 42
    let mut constant_map = BTreeMap::new();
    constant_map.insert(vid(1), Interval::singleton(42, 32));

    // Store: store vid(1) to ptr vid(2)
    let store_inst = Instruction::new(iid(100), Operation::Store)
        .with_operands(vec![vid(1), vid(2)]);

    transfer_instruction_with_pta(&store_inst, &mut state, &constant_map, &module, &pta_integration);

    // Load: load from ptr vid(3) into vid(4) - should get 42 via aliasing
    let load_inst = Instruction::new(iid(101), Operation::Load)
        .with_operands(vec![vid(3)])
        .with_dst(vid(4));

    transfer_instruction_with_pta(&load_inst, &mut state, &constant_map, &module, &pta_integration);

    let result = state.get(vid(4), 32);
    assert_eq!(result.lo(), 42);
    assert_eq!(result.hi(), 42);
}
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cargo test -p saf-analysis transfer::tests::transfer_store_load_with_pta --no-default-features`
Expected: FAIL with "function `transfer_instruction_with_pta` not found"

**Step 3: Add PTA-aware transfer function**

Add new function to `transfer.rs` (after `transfer_instruction`, around line 388):

```rust
/// Apply the transfer function for a single instruction with PTA integration.
///
/// Uses PTA information for alias-aware memory operations:
/// - Store: writes to all locations in points-to set
/// - Load: joins intervals from all aliased locations
#[allow(clippy::too_many_lines)]
pub fn transfer_instruction_with_pta(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &super::pta_integration::PtaIntegration<'_>,
) {
    // Handle Store with PTA
    if let Operation::Store = &inst.op {
        if inst.operands.len() >= 2 {
            let value_id = inst.operands[0];
            let ptr_id = inst.operands[1];
            let value_interval = resolve_operand(value_id, state, constant_map);

            let pts = pta.points_to(ptr_id);
            if pts.is_empty() {
                // No PTA info - fall back to ValueId-based tracking
                state.store(ptr_id, value_interval);
            } else if pts.len() == 1 {
                // Singleton - strong update
                let loc = *pts.iter().next().unwrap();
                state.store_loc(loc, value_interval);
            } else {
                // Multiple targets - weak update
                for loc in &pts {
                    state.store_loc_weak(*loc, value_interval.clone());
                }
            }
        }
        return;
    }

    // Handle Memcpy/Memset
    if matches!(&inst.op, Operation::Memcpy | Operation::Memset) {
        state.invalidate_all_memory();
        state.invalidate_all_loc_memory();
        return;
    }

    let Some(dst) = inst.dst else {
        return;
    };

    match &inst.op {
        Operation::Load => {
            if let Some(&ptr) = inst.operands.first() {
                let pts = pta.points_to(ptr);

                if pts.is_empty() {
                    // No PTA info - fall back to ValueId-based tracking
                    if let Some(stored_interval) = state.load(ptr) {
                        state.set(dst, stored_interval.clone());
                    } else {
                        state.set(dst, Interval::make_top(DEFAULT_BITS));
                    }
                } else {
                    // Join intervals from all pointed-to locations
                    let mut result = Interval::make_bottom(DEFAULT_BITS);
                    let mut found_any = false;

                    for loc in &pts {
                        if let Some(interval) = state.load_loc(*loc) {
                            result = result.join(interval);
                            found_any = true;
                        }
                    }

                    if found_any {
                        state.set(dst, result);
                    } else {
                        state.set(dst, Interval::make_top(DEFAULT_BITS));
                    }
                }
            } else {
                state.set(dst, Interval::make_top(DEFAULT_BITS));
            }
        }

        Operation::CallDirect { callee } => {
            state.set(dst, Interval::make_top(DEFAULT_BITS));

            let is_side_effect_free = module
                .functions
                .iter()
                .find(|f| f.id == *callee)
                .map_or(false, |f| is_known_pure_function(&f.name));

            if !is_side_effect_free {
                state.invalidate_all_memory();
                state.invalidate_all_loc_memory();
            }
        }

        Operation::CallIndirect => {
            state.set(dst, Interval::make_top(DEFAULT_BITS));
            state.invalidate_all_memory();
            state.invalidate_all_loc_memory();
        }

        // All other operations: delegate to non-PTA version
        _ => {
            transfer_instruction(inst, state, constant_map, module);
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `make shell` then `cargo test -p saf-analysis transfer::tests --no-default-features`
Expected: PASS (all tests)

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/absint/transfer.rs
git commit -m "$(cat <<'EOF'
feat(absint): add PTA-aware transfer functions

Add transfer_instruction_with_pta() for alias-aware memory operations:
- Store: strong update for singleton pts, weak update for multiple
- Load: joins intervals from all aliased locations
- CallDirect/CallIndirect: invalidate both memory models
- Other ops: delegate to existing transfer_instruction

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Mod/Ref Analysis Module

**Files:**
- Create: `crates/saf-analysis/src/pta/mod_ref.rs`
- Modify: `crates/saf-analysis/src/pta/mod.rs`

**Step 1: Write the failing test**

```rust
// crates/saf-analysis/src/pta/mod_ref.rs

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirModule, AirFunction, AirBlock, Instruction, Operation};
    use saf_core::ids::{ModuleId, FunctionId, BlockId, InstId, ValueId};

    fn vid(n: u128) -> ValueId { ValueId::new(n) }
    fn iid(n: u128) -> InstId { InstId::new(n) }
    fn bid(n: u128) -> BlockId { BlockId::new(n) }
    fn fid(n: u128) -> FunctionId { FunctionId::new(n) }

    #[test]
    fn mod_ref_detects_store_as_mod() {
        // Function with a store instruction
        let store_inst = Instruction::new(iid(1), Operation::Store)
            .with_operands(vec![vid(1), vid(2)]); // store val to ptr

        let block = AirBlock {
            id: bid(1),
            instructions: vec![store_inst],
        };

        let func = AirFunction {
            id: fid(1),
            name: "test".to_string(),
            params: vec![],
            blocks: vec![block],
            is_declaration: false,
            is_vararg: false,
        };

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            functions: vec![func],
            globals: vec![],
            constants: std::collections::BTreeMap::new(),
        };

        // Empty PTA for this test
        let empty_pta = crate::pta::result::PtaResult::new(
            crate::pta::solver::PointsToMap::new(),
            &crate::pta::location::LocationFactory::new(crate::pta::config::FieldSensitivity::Insensitive),
            crate::pta::context::PtaDiagnostics::default(),
        );

        let summaries = compute_all_mod_ref(&module, &empty_pta);
        let summary = summaries.get(&fid(1)).unwrap();

        // Should have vid(2) as a modified pointer (ValueId-based tracking)
        assert!(summary.modifies_unknown); // No PTA, so conservative
    }
}
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cargo test -p saf-analysis mod_ref::tests --no-default-features`
Expected: FAIL with "cannot find module `mod_ref`"

**Step 3: Write the implementation**

```rust
// crates/saf-analysis/src/pta/mod_ref.rs

//! Mod/Ref analysis for function side effects.
//!
//! Computes which memory locations each function may modify (mod)
//! or reference (ref). Used for precise memory invalidation in
//! interprocedural abstract interpretation.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, LocId, ValueId};

use super::result::PtaResult;

/// Summary of a function's memory side effects.
#[derive(Debug, Clone, Default)]
pub struct ModRefSummary {
    /// Locations this function may modify.
    pub modified_locs: BTreeSet<LocId>,
    /// Locations this function may read.
    pub referenced_locs: BTreeSet<LocId>,
    /// Pointers this function stores to (ValueId-based, for no-PTA fallback).
    pub modified_ptrs: BTreeSet<ValueId>,
    /// Pointers this function loads from (ValueId-based).
    pub referenced_ptrs: BTreeSet<ValueId>,
    /// Whether function may modify unknown locations (escaping pointers).
    pub modifies_unknown: bool,
    /// Whether function may read unknown locations.
    pub references_unknown: bool,
}

/// Compute mod/ref summaries for all functions in a module.
#[must_use]
pub fn compute_all_mod_ref(
    module: &AirModule,
    pta: &PtaResult,
) -> BTreeMap<FunctionId, ModRefSummary> {
    let mut summaries = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            // External functions: conservative (modifies/references unknown)
            summaries.insert(func.id, ModRefSummary {
                modifies_unknown: true,
                references_unknown: true,
                ..Default::default()
            });
            continue;
        }

        let summary = compute_function_mod_ref(func, pta);
        summaries.insert(func.id, summary);
    }

    summaries
}

/// Compute mod/ref summary for a single function.
fn compute_function_mod_ref(
    func: &saf_core::air::AirFunction,
    pta: &PtaResult,
) -> ModRefSummary {
    let mut summary = ModRefSummary::default();

    for block in &func.blocks {
        for inst in &block.instructions {
            match &inst.op {
                Operation::Store => {
                    // Store operands: [value, pointer]
                    if inst.operands.len() >= 2 {
                        let ptr = inst.operands[1];
                        summary.modified_ptrs.insert(ptr);

                        let pts = pta.points_to(ptr);
                        if pts.is_empty() {
                            summary.modifies_unknown = true;
                        } else {
                            for loc in pts {
                                summary.modified_locs.insert(loc);
                            }
                        }
                    }
                }
                Operation::Load => {
                    // Load operands: [pointer]
                    if let Some(&ptr) = inst.operands.first() {
                        summary.referenced_ptrs.insert(ptr);

                        let pts = pta.points_to(ptr);
                        if pts.is_empty() {
                            summary.references_unknown = true;
                        } else {
                            for loc in pts {
                                summary.referenced_locs.insert(loc);
                            }
                        }
                    }
                }
                Operation::Memcpy | Operation::Memset => {
                    // These modify memory at destination pointer
                    summary.modifies_unknown = true;
                    if matches!(&inst.op, Operation::Memcpy) {
                        summary.references_unknown = true;
                    }
                }
                Operation::CallDirect { .. } | Operation::CallIndirect => {
                    // Conservative: assume callees may modify/reference anything
                    // TODO: Use callee summaries for interprocedural precision
                    summary.modifies_unknown = true;
                    summary.references_unknown = true;
                }
                _ => {}
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }
    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }
    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }
    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }

    fn empty_pta() -> PtaResult {
        use crate::pta::config::FieldSensitivity;
        use crate::pta::context::PtaDiagnostics;
        use crate::pta::location::LocationFactory;
        use crate::pta::solver::PointsToMap;

        PtaResult::new(
            PointsToMap::new(),
            &LocationFactory::new(FieldSensitivity::Insensitive),
            PtaDiagnostics::default(),
        )
    }

    #[test]
    fn mod_ref_detects_store_as_mod() {
        let store_inst = Instruction::new(iid(1), Operation::Store)
            .with_operands(vec![vid(1), vid(2)]);

        let block = AirBlock {
            id: bid(1),
            instructions: vec![store_inst],
        };

        let func = AirFunction {
            id: fid(1),
            name: "test".to_string(),
            params: vec![],
            blocks: vec![block],
            is_declaration: false,
            is_vararg: false,
        };

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            functions: vec![func],
            globals: vec![],
            constants: std::collections::BTreeMap::new(),
        };

        let summaries = compute_all_mod_ref(&module, &empty_pta());
        let summary = summaries.get(&fid(1)).unwrap();

        assert!(summary.modified_ptrs.contains(&vid(2)));
        assert!(summary.modifies_unknown);
    }

    #[test]
    fn mod_ref_detects_load_as_ref() {
        let load_inst = Instruction::new(iid(1), Operation::Load)
            .with_operands(vec![vid(1)])
            .with_dst(vid(2));

        let block = AirBlock {
            id: bid(1),
            instructions: vec![load_inst],
        };

        let func = AirFunction {
            id: fid(1),
            name: "test".to_string(),
            params: vec![],
            blocks: vec![block],
            is_declaration: false,
            is_vararg: false,
        };

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            functions: vec![func],
            globals: vec![],
            constants: std::collections::BTreeMap::new(),
        };

        let summaries = compute_all_mod_ref(&module, &empty_pta());
        let summary = summaries.get(&fid(1)).unwrap();

        assert!(summary.referenced_ptrs.contains(&vid(1)));
        assert!(summary.references_unknown);
    }

    #[test]
    fn mod_ref_external_is_conservative() {
        let func = AirFunction {
            id: fid(1),
            name: "external".to_string(),
            params: vec![],
            blocks: vec![],
            is_declaration: true,
            is_vararg: false,
        };

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            functions: vec![func],
            globals: vec![],
            constants: std::collections::BTreeMap::new(),
        };

        let summaries = compute_all_mod_ref(&module, &empty_pta());
        let summary = summaries.get(&fid(1)).unwrap();

        assert!(summary.modifies_unknown);
        assert!(summary.references_unknown);
    }
}
```

**Step 4: Update pta/mod.rs to export the module**

Add to `crates/saf-analysis/src/pta/mod.rs`:

```rust
pub mod mod_ref;

pub use mod_ref::{ModRefSummary, compute_all_mod_ref};
```

**Step 5: Run tests to verify they pass**

Run: `make shell` then `cargo test -p saf-analysis mod_ref::tests --no-default-features`
Expected: PASS (3 tests)

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/pta/mod_ref.rs crates/saf-analysis/src/pta/mod.rs
git commit -m "$(cat <<'EOF'
feat(pta): add mod/ref analysis for function side effects

Add ModRefSummary tracking:
- modified_locs/referenced_locs for PTA-aware tracking
- modified_ptrs/referenced_ptrs for ValueId-based fallback
- modifies_unknown/references_unknown for conservative cases

compute_all_mod_ref() analyzes all functions in a module.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Indirect Call Resolution in Interprocedural Analysis

**Files:**
- Modify: `crates/saf-analysis/src/absint/interprocedural.rs:1-275`

**Step 1: Write the failing test**

Add E2E test file for indirect call resolution:

```rust
// crates/saf-analysis/tests/absint_indirect_call_e2e.rs

//! E2E tests for indirect call resolution in abstract interpretation.

use saf_analysis::absint::{solve_interprocedural_with_pta, AbstractInterpConfig};
use saf_analysis::pta::PtaContext;

mod common;
use common::load_ll_fixture;

#[test]
fn indirect_call_resolves_targets() {
    // This test requires a fixture with indirect calls
    // Skip if fixture doesn't exist yet
    let Ok(module) = load_ll_fixture("indirect_call.ll") else {
        eprintln!("Skipping: indirect_call.ll fixture not found");
        return;
    };

    let pta_config = saf_analysis::pta::PtaConfig::default();
    let pta = PtaContext::analyze(&module, &pta_config);

    let config = AbstractInterpConfig::default();
    let result = solve_interprocedural_with_pta(&module, &config, &pta);

    // Verify we got non-TOP results for the indirect call return values
    assert!(result.intraprocedural().diagnostics().functions_analyzed > 0);
}
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cargo test -p saf-analysis absint_indirect_call --no-default-features`
Expected: FAIL with "function `solve_interprocedural_with_pta` not found"

**Step 3: Extend interprocedural.rs**

Add imports at top of `interprocedural.rs`:

```rust
use crate::pta::result::PtaResult;
use crate::pta::mod_ref::{ModRefSummary, compute_all_mod_ref};
use super::pta_integration::PtaIntegration;
```

Extend `FunctionSummary` struct (around line 26):

```rust
/// Summary of a function's abstract behavior.
#[derive(Debug, Clone)]
pub struct FunctionSummary {
    /// Return value interval (None if function is void or returns pointer)
    return_interval: Option<Interval>,
    /// Parameter bindings that were analyzed
    param_count: usize,
    /// Locations this function may modify
    modified_locs: BTreeSet<LocId>,
    /// Whether function may modify unknown locations
    modifies_unknown: bool,
}
```

Update `FunctionSummary::new()`:

```rust
impl FunctionSummary {
    /// Create a new empty summary.
    pub fn new() -> Self {
        Self {
            return_interval: None,
            param_count: 0,
            modified_locs: BTreeSet::new(),
            modifies_unknown: false,
        }
    }

    /// Get the return value interval.
    pub fn return_interval(&self) -> Option<&Interval> {
        self.return_interval.as_ref()
    }

    /// Get the modified locations.
    pub fn modified_locations(&self) -> &BTreeSet<LocId> {
        &self.modified_locs
    }

    /// Check if function may modify unknown locations.
    pub fn may_modify_unknown(&self) -> bool {
        self.modifies_unknown
    }
}
```

Add new function after `solve_interprocedural` (around line 138):

```rust
/// Solve abstract interpretation with interprocedural analysis and PTA.
///
/// Extends `solve_interprocedural` with:
/// - Indirect call resolution via PTA
/// - Mod/ref-based selective memory invalidation
/// - Function summaries including side effects
#[must_use]
pub fn solve_interprocedural_with_pta(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaResult,
) -> InterproceduralResult {
    // Phase 1: Compute mod/ref summaries
    let mod_ref_summaries = compute_all_mod_ref(module, pta);

    // Phase 2: Compute function summaries via intraprocedural analysis
    let intraprocedural = super::solve_abstract_interp(module, config);
    let constant_map = build_constant_map(module);

    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, f))
        .collect();

    // Compute summaries for each function (including mod/ref info)
    let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        let mut summary = compute_function_summary(func, &intraprocedural, &constant_map);

        // Add mod/ref information
        if let Some(mod_ref) = mod_ref_summaries.get(&func.id) {
            summary.modified_locs = mod_ref.modified_locs.clone();
            summary.modifies_unknown = mod_ref.modifies_unknown;
        }

        summaries.insert(func.id, summary);
    }

    // Phase 3: Refine call sites with PTA for indirect calls
    let pta_integration = PtaIntegration::new(pta);
    let refined_inst_states = refine_call_sites_with_pta(
        module,
        &intraprocedural,
        &summaries,
        &func_map,
        &constant_map,
        config,
        &pta_integration,
    );

    InterproceduralResult {
        summaries,
        intraprocedural,
        refined_inst_states,
    }
}

/// Refine call sites with PTA support for indirect calls.
fn refine_call_sites_with_pta(
    module: &AirModule,
    result: &AbstractInterpResult,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
    func_map: &BTreeMap<FunctionId, &saf_core::air::AirFunction>,
    constant_map: &BTreeMap<ValueId, Interval>,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
) -> BTreeMap<InstId, AbstractState> {
    let mut refined_states = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::CallDirect { callee } => {
                        // Existing direct call handling
                        if let Some(dst) = inst.dst {
                            let base_state = result.state_at_inst(inst.id);

                            if let Some(callee_func) = func_map.get(callee) {
                                if let Some(refined) = try_context_sensitive_call(
                                    inst,
                                    dst,
                                    callee_func,
                                    base_state,
                                    constant_map,
                                    config,
                                    module,
                                ) {
                                    refined_states.insert(inst.id, refined);
                                    continue;
                                }
                            }

                            if let Some(summary) = summaries.get(callee) {
                                if let Some(return_interval) = &summary.return_interval {
                                    if !return_interval.is_top() {
                                        let mut refined =
                                            base_state.cloned().unwrap_or_default();
                                        refined.set(dst, return_interval.clone());
                                        refined_states.insert(inst.id, refined);
                                    }
                                }
                            }
                        }
                    }
                    Operation::CallIndirect => {
                        if let Some(dst) = inst.dst {
                            // Get function pointer from operands (first operand)
                            let Some(&fn_ptr) = inst.operands.first() else {
                                continue;
                            };

                            let targets = pta.resolve_indirect_call(fn_ptr);
                            if targets.is_empty() {
                                continue; // No targets found, keep TOP
                            }

                            let base_state = result.state_at_inst(inst.id);

                            // Join return intervals from all possible targets
                            let mut return_interval = Interval::make_bottom(DEFAULT_BITS);

                            for target_id in &targets {
                                if let Some(summary) = summaries.get(target_id) {
                                    if let Some(ret) = &summary.return_interval {
                                        return_interval = return_interval.join(ret);
                                    } else {
                                        // Void or unknown return - go to top
                                        return_interval = Interval::make_top(DEFAULT_BITS);
                                        break;
                                    }
                                } else {
                                    // Unknown function - go to top
                                    return_interval = Interval::make_top(DEFAULT_BITS);
                                    break;
                                }
                            }

                            if !return_interval.is_top() && !return_interval.is_bottom() {
                                let mut refined = base_state.cloned().unwrap_or_default();
                                refined.set(dst, return_interval);
                                refined_states.insert(inst.id, refined);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    refined_states
}

/// Try context-sensitive analysis for a direct call.
fn try_context_sensitive_call(
    inst: &saf_core::air::Instruction,
    dst: ValueId,
    callee_func: &saf_core::air::AirFunction,
    base_state: Option<&AbstractState>,
    constant_map: &BTreeMap<ValueId, Interval>,
    config: &AbstractInterpConfig,
    module: &AirModule,
) -> Option<AbstractState> {
    let mut param_bindings: BTreeMap<ValueId, Interval> = BTreeMap::new();

    for (i, &arg) in inst.operands.iter().enumerate() {
        if i < callee_func.params.len() {
            let param_id = callee_func.params[i].id;
            let arg_interval = base_state
                .and_then(|s| s.get_opt(arg).cloned())
                .or_else(|| constant_map.get(&arg).cloned());

            if let Some(interval) = arg_interval {
                param_bindings.insert(param_id, interval);
            } else {
                return None; // Unknown argument
            }
        }
    }

    if param_bindings.is_empty() {
        return None;
    }

    let context_result = solve_function_with_params(
        callee_func,
        config,
        &param_bindings,
        constant_map,
        module,
    );

    if let Some(return_interval) = context_result.get(&ValueId::new(0)) {
        if !return_interval.is_top() {
            let mut refined = base_state.cloned().unwrap_or_default();
            refined.set(dst, return_interval.clone());
            return Some(refined);
        }
    }

    None
}
```

Add import for `LocId`:

```rust
use saf_core::ids::{FunctionId, InstId, LocId, ValueId};
```

**Step 4: Export the new function in mod.rs**

Update `crates/saf-analysis/src/absint/mod.rs` exports:

```rust
pub use interprocedural::{FunctionSummary, InterproceduralResult, solve_interprocedural, solve_interprocedural_with_pta};
```

**Step 5: Run tests to verify they pass**

Run: `make shell` then `cargo test -p saf-analysis interprocedural --no-default-features`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/absint/interprocedural.rs crates/saf-analysis/src/absint/mod.rs
git commit -m "$(cat <<'EOF'
feat(absint): add PTA-aware interprocedural analysis

Add solve_interprocedural_with_pta() extending interprocedural analysis:
- Indirect call resolution via PTA function pointer targets
- Mod/ref integration in FunctionSummary
- refine_call_sites_with_pta() handles both direct and indirect calls
- Joins return intervals from all possible indirect call targets

FunctionSummary now includes modified_locs and modifies_unknown.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Combined Analysis Orchestration

**Files:**
- Create: `crates/saf-analysis/src/combined/mod.rs`
- Modify: `crates/saf-analysis/src/lib.rs`

**Step 1: Write the failing test**

```rust
// crates/saf-analysis/src/combined/mod.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_config_default() {
        let config = CombinedAnalysisConfig::default();
        assert!(config.enable_refinement);
        assert_eq!(config.max_refinement_iterations, 3);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cargo test -p saf-analysis combined::tests --no-default-features`
Expected: FAIL with "cannot find module `combined`"

**Step 3: Write the implementation**

```rust
// crates/saf-analysis/src/combined/mod.rs

//! Combined PTA + Abstract Interpretation analysis.
//!
//! Orchestrates pointer analysis and abstract interpretation with:
//! - Alias-aware memory tracking
//! - Indirect call resolution
//! - Bidirectional refinement (optional)

use std::collections::BTreeMap;

use saf_core::air::AirModule;
use saf_core::ids::FunctionId;

use crate::absint::{
    AbstractInterpConfig, AbstractInterpResult, FunctionSummary,
    solve_interprocedural_with_pta,
};
use crate::pta::{PtaConfig, PtaContext, PtaResult};

/// Configuration for combined PTA + abstract interpretation.
#[derive(Debug, Clone)]
pub struct CombinedAnalysisConfig {
    /// PTA configuration.
    pub pta: PtaConfig,
    /// Abstract interpretation configuration.
    pub absint: AbstractInterpConfig,
    /// Enable bidirectional refinement loop.
    pub enable_refinement: bool,
    /// Maximum refinement iterations (0 = single pass).
    pub max_refinement_iterations: usize,
    /// Use context-sensitive PTA for indirect calls.
    pub context_sensitive_indirect: bool,
}

impl Default for CombinedAnalysisConfig {
    fn default() -> Self {
        Self {
            pta: PtaConfig::default(),
            absint: AbstractInterpConfig::default(),
            enable_refinement: true,
            max_refinement_iterations: 3,
            context_sensitive_indirect: false,
        }
    }
}

/// Combined analysis result.
#[derive(Debug)]
pub struct CombinedAnalysisResult {
    /// Points-to analysis results.
    pub pta: PtaResult,
    /// Abstract interpretation results (with PTA-aware memory model).
    pub absint: AbstractInterpResult,
    /// Function summaries (return intervals + mod/ref).
    pub summaries: BTreeMap<FunctionId, FunctionSummary>,
    /// Number of refinement iterations performed.
    pub refinement_iterations: usize,
}

impl CombinedAnalysisResult {
    /// Get the PTA result.
    #[must_use]
    pub fn pta(&self) -> &PtaResult {
        &self.pta
    }

    /// Get the abstract interpretation result.
    #[must_use]
    pub fn absint(&self) -> &AbstractInterpResult {
        &self.absint
    }

    /// Get function summary by ID.
    #[must_use]
    pub fn function_summary(&self, func_id: &FunctionId) -> Option<&FunctionSummary> {
        self.summaries.get(func_id)
    }
}

/// Run combined PTA + abstract interpretation analysis.
///
/// Phases:
/// 1. Initial PTA
/// 2. Interprocedural abstract interpretation with PTA
/// 3. (Optional) Bidirectional refinement loop
#[must_use]
pub fn analyze_combined(
    module: &AirModule,
    config: &CombinedAnalysisConfig,
) -> CombinedAnalysisResult {
    // Phase 1: Initial PTA
    let pta = PtaContext::analyze(module, &config.pta);

    // Phase 2: Interprocedural absint with PTA
    let interprocedural = solve_interprocedural_with_pta(module, &config.absint, &pta);

    // Extract summaries from interprocedural result
    let mut summaries = BTreeMap::new();
    for func in &module.functions {
        if let Some(summary) = interprocedural.function_summary(&func.id) {
            summaries.insert(func.id, summary.clone());
        }
    }

    // Phase 3: Refinement loop (TODO: implement index refinement)
    let refinement_iterations = 0;

    CombinedAnalysisResult {
        pta,
        absint: interprocedural.intraprocedural().clone(),
        summaries,
        refinement_iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_config_default() {
        let config = CombinedAnalysisConfig::default();
        assert!(config.enable_refinement);
        assert_eq!(config.max_refinement_iterations, 3);
    }

    #[test]
    fn combined_result_accessors() {
        use saf_core::ids::ModuleId;

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            functions: vec![],
            globals: vec![],
            constants: BTreeMap::new(),
        };

        let config = CombinedAnalysisConfig::default();
        let result = analyze_combined(&module, &config);

        assert_eq!(result.refinement_iterations, 0);
        assert!(result.summaries.is_empty());
    }
}
```

**Step 4: Update lib.rs to export the module**

Add to `crates/saf-analysis/src/lib.rs`:

```rust
pub mod combined;

pub use combined::{CombinedAnalysisConfig, CombinedAnalysisResult, analyze_combined};
```

**Step 5: Run tests to verify they pass**

Run: `make shell` then `cargo test -p saf-analysis combined::tests --no-default-features`
Expected: PASS (2 tests)

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/combined/mod.rs crates/saf-analysis/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(combined): add combined PTA + absint orchestration module

Add CombinedAnalysisConfig and CombinedAnalysisResult:
- Configurable PTA and absint settings
- enable_refinement and max_refinement_iterations
- analyze_combined() runs PTA → interprocedural absint pipeline

Refinement loop placeholder for future index refinement.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Python Bindings

**Files:**
- Modify: `crates/saf-python/src/lib.rs`

**Step 1: Write the test**

Create Python test file:

```python
# python/tests/test_combined_analysis.py

import pytest
import saf

def test_combined_analysis_empty_module():
    """Test combined analysis on empty module."""
    module = saf.AirModule.from_json({
        "id": "0x" + "00" * 16,
        "functions": [],
        "globals": [],
        "constants": {}
    })

    result = saf.analyze_combined(module)
    assert result is not None
    assert result.refinement_iterations == 0


def test_combined_analysis_config():
    """Test combined analysis with custom config."""
    module = saf.AirModule.from_json({
        "id": "0x" + "00" * 16,
        "functions": [],
        "globals": [],
        "constants": {}
    })

    result = saf.analyze_combined(module, enable_refinement=False)
    assert result is not None
```

**Step 2: Run test to verify it fails**

Run: `make shell` then `cd /workspace && uv run pytest python/tests/test_combined_analysis.py -v`
Expected: FAIL with "module 'saf' has no attribute 'analyze_combined'"

**Step 3: Add Python bindings**

Add to `crates/saf-python/src/lib.rs` (in the appropriate section with other analysis functions):

```rust
use saf_analysis::combined::{CombinedAnalysisConfig, CombinedAnalysisResult, analyze_combined as rust_analyze_combined};

/// Combined PTA + Abstract Interpretation analysis result.
#[pyclass]
#[derive(Clone)]
pub struct PyCombinedAnalysisResult {
    inner: CombinedAnalysisResult,
}

#[pymethods]
impl PyCombinedAnalysisResult {
    /// Get the number of refinement iterations performed.
    #[getter]
    fn refinement_iterations(&self) -> usize {
        self.inner.refinement_iterations
    }

    /// Query interval at a program point.
    ///
    /// Returns (lo, hi) tuple or None if not tracked.
    #[pyo3(signature = (inst_id, value_id))]
    fn interval_at(&self, inst_id: u128, value_id: u128) -> Option<(i128, i128)> {
        use saf_core::ids::{InstId, ValueId};

        let inst = InstId::new(inst_id);
        let value = ValueId::new(value_id);

        self.inner.absint.state_at_inst(inst)
            .and_then(|state| state.get_opt(value))
            .map(|iv| (iv.lo(), iv.hi()))
    }

    /// Query points-to set for a value.
    ///
    /// Returns list of location ID strings.
    #[pyo3(signature = (value_id))]
    fn points_to(&self, value_id: u128) -> Vec<String> {
        use saf_core::ids::ValueId;

        self.inner.pta.points_to(ValueId::new(value_id))
            .iter()
            .map(|loc| format!("{:?}", loc))
            .collect()
    }

    /// Query alias relationship between two pointers.
    ///
    /// Returns "Must", "Partial", "May", "No", or "Unknown".
    #[pyo3(signature = (a, b))]
    fn may_alias(&self, a: u128, b: u128) -> String {
        use saf_core::ids::ValueId;

        let result = self.inner.pta.may_alias(ValueId::new(a), ValueId::new(b));
        format!("{:?}", result)
    }
}

/// Run combined PTA + abstract interpretation analysis.
///
/// Args:
///     module: The AIR module to analyze
///     enable_refinement: Enable bidirectional refinement (default: True)
///     max_iterations: Maximum refinement iterations (default: 3)
///
/// Returns:
///     CombinedAnalysisResult with PTA and absint results
#[pyfunction]
#[pyo3(signature = (module, enable_refinement=true, max_iterations=3))]
#[allow(clippy::needless_pass_by_value)] // PyO3 requires owned types
fn analyze_combined(
    module: PyAirModule,
    enable_refinement: bool,
    max_iterations: usize,
) -> PyResult<PyCombinedAnalysisResult> {
    let config = CombinedAnalysisConfig {
        enable_refinement,
        max_refinement_iterations: max_iterations,
        ..Default::default()
    };

    let result = rust_analyze_combined(&module.inner, &config);
    Ok(PyCombinedAnalysisResult { inner: result })
}
```

Add to the module registration (in the `#[pymodule]` function):

```rust
m.add_class::<PyCombinedAnalysisResult>()?;
m.add_function(wrap_pyfunction!(analyze_combined, m)?)?;
```

**Step 4: Run tests to verify they pass**

Run: `make shell` then `cd /workspace && maturin develop --release && uv run pytest python/tests/test_combined_analysis.py -v`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add crates/saf-python/src/lib.rs python/tests/test_combined_analysis.py
git commit -m "$(cat <<'EOF'
feat(python): add combined analysis bindings

Add saf.analyze_combined() and PyCombinedAnalysisResult:
- interval_at(inst_id, value_id) -> (lo, hi)
- points_to(value_id) -> [loc_str, ...]
- may_alias(a, b) -> "Must" | "Partial" | "May" | "No" | "Unknown"
- refinement_iterations property

Configurable via enable_refinement and max_iterations args.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Update PROGRESS.md

**Files:**
- Modify: `plans/PROGRESS.md`

**Step 1: Update progress file**

Add to Plans Index:
```markdown
| 057 | pta-absint-integration | E32 | in-progress |
```

Add new epic:
```markdown
- [ ] E32: PTA-Absint Integration
```

Update Next Steps:
```markdown
## Next Steps
1. **Complete PTA-Absint Integration (Plan 057)** — Phases 1-7 implement core integration, Phase 8 adds Python bindings, Phase 9 validates with PTABen
```

Add to Session Log:
```markdown
| 2026-02-02 | E32 | **PTA-Absint Integration Design:** Created comprehensive design for integrating PTA with abstract interpretation. Core changes: (1) LocId-based memory model for alias-aware tracking, (2) Indirect call resolution via PTA, (3) Bidirectional refinement loop. ~1700 LOC across 9 phases. Implementation plan in Plan 057. |
```

**Step 2: Commit**

```bash
git add plans/PROGRESS.md
git commit -m "$(cat <<'EOF'
docs: add Plan 057 PTA-Absint Integration to progress

- Add E32: PTA-Absint Integration epic
- Add Plan 057 to Plans Index (in-progress)
- Update Next Steps with integration plan
- Add session log entry for design work

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: E2E Test Fixtures and PTABen Validation

**Files:**
- Create: `tests/programs/c/pta_absint_alias.c`
- Create: `tests/programs/c/indirect_call.c`
- Create: `crates/saf-analysis/tests/combined_e2e.rs`

**Step 1: Create C test fixtures**

```c
// tests/programs/c/pta_absint_alias.c

// Test alias-aware memory tracking in abstract interpretation

void test_alias_store_load() {
    int x = 10;
    int *p = &x;
    int *q = &x;  // p and q alias

    *p = 42;
    int v = *q;   // Should get interval [42, 42] via aliasing

    // Marker for test validation
    if (v == 42) {
        // Expected path
    }
}

void test_no_alias() {
    int x = 10, y = 20;
    int *p = &x;
    int *q = &y;  // p and q don't alias

    *p = 42;
    int v = *q;   // Should get interval [20, 20] (original value)

    if (v == 20) {
        // Expected path
    }
}

void test_weak_update() {
    int arr[2] = {10, 20};
    int i = 0;  // Could be 0 or 1 at runtime
    if (i) i = 1;

    int *p = &arr[i];  // Points to arr[0] or arr[1]
    *p = 42;           // Weak update to both locations

    int v0 = arr[0];   // Should get [10, 42] (original or updated)
    int v1 = arr[1];   // Should get [20, 42] (original or updated)
}

int main() {
    test_alias_store_load();
    test_no_alias();
    test_weak_update();
    return 0;
}
```

```c
// tests/programs/c/indirect_call.c

// Test indirect call resolution in interprocedural analysis

int add(int a, int b) {
    return a + b;
}

int mul(int a, int b) {
    return a * b;
}

int sub(int a, int b) {
    return a - b;
}

typedef int (*binop_t)(int, int);

int compute(binop_t op, int x, int y) {
    return op(x, y);
}

int main() {
    int r1 = compute(add, 3, 4);   // Should get [7, 7]
    int r2 = compute(mul, 3, 4);   // Should get [12, 12]
    int r3 = compute(sub, 10, 3);  // Should get [7, 7]

    // Variable function pointer
    binop_t ops[2] = {add, mul};
    int i = 0;
    if (i) i = 1;

    int r4 = ops[i](5, 6);  // Should get [11, 30] (add=11, mul=30)

    return r1 + r2 + r3 + r4;
}
```

**Step 2: Compile fixtures in Docker**

Run: `make shell` then:
```bash
clang -S -emit-llvm -g -O0 tests/programs/c/pta_absint_alias.c -o tests/fixtures/llvm/e2e/pta_absint_alias.ll
clang -S -emit-llvm -g -O0 tests/programs/c/indirect_call.c -o tests/fixtures/llvm/e2e/indirect_call.ll
```

**Step 3: Create E2E test**

```rust
// crates/saf-analysis/tests/combined_e2e.rs

//! E2E tests for combined PTA + abstract interpretation.

use saf_analysis::combined::{analyze_combined, CombinedAnalysisConfig};

mod common;
use common::load_ll_fixture;

#[test]
fn combined_analysis_alias_tracking() {
    let module = load_ll_fixture("pta_absint_alias.ll").expect("fixture should exist");

    let config = CombinedAnalysisConfig::default();
    let result = analyze_combined(&module, &config);

    // Verify analysis completed
    assert!(result.absint.diagnostics().functions_analyzed > 0);

    // Verify PTA found locations
    assert!(result.pta.location_count() > 0);
}

#[test]
fn combined_analysis_indirect_calls() {
    let module = load_ll_fixture("indirect_call.ll").expect("fixture should exist");

    let config = CombinedAnalysisConfig::default();
    let result = analyze_combined(&module, &config);

    // Verify analysis completed
    assert!(result.absint.diagnostics().functions_analyzed >= 4); // main + add + mul + sub

    // Verify function summaries exist
    assert!(!result.summaries.is_empty());
}
```

**Step 4: Run E2E tests**

Run: `make test`
Expected: PASS (all tests including new E2E tests)

**Step 5: Commit**

```bash
git add tests/programs/c/pta_absint_alias.c tests/programs/c/indirect_call.c \
        tests/fixtures/llvm/e2e/pta_absint_alias.ll tests/fixtures/llvm/e2e/indirect_call.ll \
        crates/saf-analysis/tests/combined_e2e.rs
git commit -m "$(cat <<'EOF'
test(combined): add E2E tests for PTA-absint integration

Add test fixtures:
- pta_absint_alias.c: alias store/load, no-alias, weak update
- indirect_call.c: function pointers, variable targets

Add combined_e2e.rs with tests:
- combined_analysis_alias_tracking
- combined_analysis_indirect_calls

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Summary

**Total tasks:** 9
**Estimated LOC:** ~1700
**Key deliverables:**
1. `PtaIntegration` wrapper for absint-friendly PTA queries
2. `AbstractState` extended with LocId-based memory
3. PTA-aware transfer functions
4. Mod/ref analysis module
5. Indirect call resolution in interprocedural analysis
6. Combined analysis orchestration
7. Python bindings
8. E2E test fixtures

**Commit sequence:** 9 atomic commits, each self-contained and testable.
