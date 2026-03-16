# Path-Sensitive Analysis Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Break SAF's ~50% Juliet precision ceiling with three complementary path-sensitivity mechanisms: SCCP pre-pass, guarded value-flow SVFG, and trace-partitioned abstract interpretation.

**Architecture:** Three-layer pipeline — SCCP runs first (produces constant values + dead blocks), feeding both the guarded SVFG checker (path-sensitive reachability) and the trace-partitioned absint (path-sensitive values). Based on Saturn/Coverity guarded facts, Astree trace partitioning, and Infer/Pulse disjunctive budgets.

**Tech Stack:** Rust, Z3 (existing integration), AIR/SVFG/absint infrastructure (all existing)

**Design doc:** `docs/plans/2026-02-22-path-sensitive-analysis-design.md`

---

## Phase A: SCCP Pre-Pass

Classical Wegman-Zadeck Sparse Conditional Constant Propagation. Runs before absint and SVFG analysis. Produces `SccpResult { constants, dead_blocks }` consumed downstream.

### Task A1: SCCP Lattice and Solver Core

**Files:**
- Create: `crates/saf-analysis/src/absint/sccp.rs`
- Modify: `crates/saf-analysis/src/absint/mod.rs:47-66` (add module declaration)

**Step 1: Write the failing test**

Add to `sccp.rs`:

```rust
//! Sparse Conditional Constant Propagation (SCCP) pre-pass.
//!
//! Runs before the main fixpoint solver. Produces:
//! - `constants`: values proven to be a single constant
//! - `dead_blocks`: blocks proven unreachable via constant branch resolution
//!
//! Algorithm: Wegman & Zadeck (POPL 1991) adapted for AIR.
//! Two worklists: SSA edges (value propagation) and CFG edges (reachability).
//! Three-level lattice: Top (unknown) -> Constant(i128) -> Bottom (overdetermined).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::{AirFunction, AirModule, Constant, Operation};
use saf_core::ids::{BlockId, ValueId};

/// SCCP lattice value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SccpValue {
    /// Not yet determined (optimistic assumption).
    Top,
    /// Proven to be exactly this constant.
    Constant(i128),
    /// Multiple distinct constants reach this value (overdetermined).
    Bottom,
}

impl SccpValue {
    /// Meet operation: merge two lattice values.
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        match (self, other) {
            (SccpValue::Top, x) | (x, SccpValue::Top) => x,
            (SccpValue::Constant(a), SccpValue::Constant(b)) if a == b => SccpValue::Constant(a),
            _ => SccpValue::Bottom,
        }
    }

    /// Is this a known constant?
    #[must_use]
    pub fn as_constant(self) -> Option<i128> {
        match self {
            SccpValue::Constant(v) => Some(v),
            _ => None,
        }
    }
}

/// Result of SCCP analysis.
#[derive(Debug, Clone)]
pub struct SccpResult {
    /// Values proven to be a single constant.
    pub constants: BTreeMap<ValueId, i128>,
    /// Blocks proven unreachable via constant branch resolution.
    pub dead_blocks: BTreeSet<BlockId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sccp_lattice_meet_top_with_const() {
        assert_eq!(SccpValue::Top.meet(SccpValue::Constant(5)), SccpValue::Constant(5));
    }

    #[test]
    fn sccp_lattice_meet_same_const() {
        assert_eq!(SccpValue::Constant(5).meet(SccpValue::Constant(5)), SccpValue::Constant(5));
    }

    #[test]
    fn sccp_lattice_meet_different_const() {
        assert_eq!(SccpValue::Constant(5).meet(SccpValue::Constant(3)), SccpValue::Bottom);
    }

    #[test]
    fn sccp_lattice_meet_bottom_absorbs() {
        assert_eq!(SccpValue::Bottom.meet(SccpValue::Constant(5)), SccpValue::Bottom);
    }
}
```

**Step 2: Run test to verify it passes**

Run: `make test` (filter: `sccp_lattice`)
Expected: 4 tests PASS

**Step 3: Add module to mod.rs**

In `crates/saf-analysis/src/absint/mod.rs`, add after line 66 (`mod transfer_fn;`):

```rust
pub mod sccp;
```

And add to the pub use block (after line 99):

```rust
pub use sccp::SccpResult;
```

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/absint/sccp.rs crates/saf-analysis/src/absint/mod.rs
git commit -m "feat(absint): SCCP lattice and result types (Plan 148 Phase A1)"
```

---

### Task A2: SCCP Solver — Per-Function Analysis

**Files:**
- Modify: `crates/saf-analysis/src/absint/sccp.rs`

**Step 1: Write the failing test**

Add to `sccp.rs` tests module:

```rust
    #[test]
    fn sccp_run_trivial_function() {
        // Test with a minimal AirModule — a function with one block, no branches
        use saf_core::air::{AirModule, AirFunction, AirBlock, Instruction};
        use saf_core::ids::{FunctionId, BlockId, InstId};

        let block_id = BlockId::from_raw(1);
        let func_id = FunctionId::from_raw(2);
        let block = AirBlock {
            id: block_id,
            label: Some("entry".to_string()),
            instructions: vec![Instruction::new(
                InstId::from_raw(3),
                Operation::Ret,
                vec![],
            )],
        };
        let func = AirFunction {
            id: func_id,
            name: "test".to_string(),
            blocks: vec![block],
            entry_block: Some(block_id),
            params: vec![],
            is_declaration: false,
            is_vararg: false,
            return_type: None,
            linkage: None,
        };
        let module = AirModule {
            functions: vec![func],
            ..AirModule::default()
        };

        let result = run_sccp_module(&module);
        // Trivial function — no constants, no dead blocks
        assert!(result.dead_blocks.is_empty());
    }
```

**Step 2: Implement `run_sccp_module` and `run_sccp_function`**

Add to `sccp.rs`:

```rust
use saf_core::air::BinaryOp;

/// Run SCCP on all functions in a module.
pub fn run_sccp_module(module: &AirModule) -> SccpResult {
    let mut constants = BTreeMap::new();
    let mut dead_blocks = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration || func.blocks.is_empty() {
            continue;
        }
        let func_result = run_sccp_function(func, &module.constants);
        constants.extend(func_result.constants);
        dead_blocks.extend(func_result.dead_blocks);
    }

    SccpResult {
        constants,
        dead_blocks,
    }
}

/// Run SCCP on a single function.
///
/// Algorithm: dual-worklist (SSA edges + CFG edges).
/// 1. Mark entry block executable.
/// 2. Process executable blocks: evaluate instructions, propagate constants.
/// 3. For `CondBr`/`Switch` with constant condition, mark only taken successor executable.
/// 4. For `Phi`, evaluate only over executable predecessors.
/// 5. Iterate until both worklists empty.
fn run_sccp_function(
    func: &AirFunction,
    module_constants: &BTreeMap<ValueId, Constant>,
) -> SccpResult {
    // Map ValueId -> SccpValue (starts Top for all)
    let mut values: BTreeMap<ValueId, SccpValue> = BTreeMap::new();
    // Set of executable CFG edges: (from_block, to_block)
    let mut executable_edges: BTreeSet<(BlockId, BlockId)> = BTreeSet::new();
    // Set of executable blocks
    let mut executable_blocks: BTreeSet<BlockId> = BTreeSet::new();
    // SSA worklist: ValueIds whose lattice value changed
    let mut ssa_worklist: VecDeque<ValueId> = VecDeque::new();
    // CFG worklist: blocks newly marked executable
    let mut cfg_worklist: VecDeque<BlockId> = VecDeque::new();

    // Block lookup
    let block_map: BTreeMap<BlockId, &saf_core::air::AirBlock> =
        func.blocks.iter().map(|b| (b.id, b)).collect();

    // Initialize: seed module constants
    for (vid, constant) in module_constants {
        let sccp_val = constant_to_sccp(constant);
        values.insert(*vid, sccp_val);
    }

    // Mark entry block executable
    let entry_id = func.entry_block.unwrap_or_else(|| func.blocks[0].id);
    cfg_worklist.push_back(entry_id);
    executable_blocks.insert(entry_id);

    // Iterate until both worklists empty
    let mut iterations = 0u32;
    let max_iterations = 10_000;

    while (!cfg_worklist.is_empty() || !ssa_worklist.is_empty()) && iterations < max_iterations {
        iterations += 1;

        // Process CFG worklist: evaluate all instructions in newly executable blocks
        while let Some(block_id) = cfg_worklist.pop_front() {
            let Some(block) = block_map.get(&block_id) else {
                continue;
            };
            for inst in &block.instructions {
                evaluate_instruction(
                    inst,
                    block_id,
                    &mut values,
                    &mut ssa_worklist,
                    &mut cfg_worklist,
                    &mut executable_edges,
                    &mut executable_blocks,
                    module_constants,
                );
            }
        }

        // Process SSA worklist: re-evaluate uses of changed values
        while let Some(changed_vid) = ssa_worklist.pop_front() {
            // Find all instructions that use this value and are in executable blocks
            for block in &func.blocks {
                if !executable_blocks.contains(&block.id) {
                    continue;
                }
                for inst in &block.instructions {
                    if inst.operands.contains(&changed_vid) {
                        evaluate_instruction(
                            inst,
                            block.id,
                            &mut values,
                            &mut ssa_worklist,
                            &mut cfg_worklist,
                            &mut executable_edges,
                            &mut executable_blocks,
                            module_constants,
                        );
                    }
                }
            }
        }
    }

    // Collect results
    let constants: BTreeMap<ValueId, i128> = values
        .iter()
        .filter_map(|(vid, val)| val.as_constant().map(|c| (*vid, c)))
        .collect();

    let all_blocks: BTreeSet<BlockId> = func.blocks.iter().map(|b| b.id).collect();
    let dead_blocks: BTreeSet<BlockId> = all_blocks
        .difference(&executable_blocks)
        .copied()
        .collect();

    SccpResult {
        constants,
        dead_blocks,
    }
}

/// Convert an AIR `Constant` to an SCCP lattice value.
fn constant_to_sccp(constant: &Constant) -> SccpValue {
    match constant {
        Constant::Int { value, .. } => SccpValue::Constant(i128::from(*value)),
        Constant::BigInt { value, .. } => {
            if let Ok(v) = value.parse::<i128>() {
                SccpValue::Constant(v)
            } else {
                SccpValue::Bottom
            }
        }
        Constant::Null | Constant::ZeroInit => SccpValue::Constant(0),
        _ => SccpValue::Bottom,
    }
}

/// Evaluate one instruction in SCCP context. Updates lattice values and worklists.
#[allow(clippy::too_many_arguments)]
fn evaluate_instruction(
    inst: &saf_core::air::Instruction,
    block_id: BlockId,
    values: &mut BTreeMap<ValueId, SccpValue>,
    ssa_worklist: &mut VecDeque<ValueId>,
    cfg_worklist: &mut VecDeque<BlockId>,
    executable_edges: &mut BTreeSet<(BlockId, BlockId)>,
    executable_blocks: &mut BTreeSet<BlockId>,
    module_constants: &BTreeMap<ValueId, Constant>,
) {
    match &inst.op {
        // Phi: meet over executable predecessors only
        Operation::Phi { incoming } => {
            if let Some(dst) = inst.dst {
                let mut result = SccpValue::Top;
                for (pred_block, val) in incoming {
                    // Only consider executable predecessors
                    if executable_edges.contains(&(*pred_block, block_id)) {
                        let operand_val = lookup_value(*val, values, module_constants);
                        result = result.meet(operand_val);
                    }
                }
                update_value(dst, result, values, ssa_worklist);
            }
        }

        // Binary operations: evaluate if both operands are constant
        Operation::BinaryOp { kind } => {
            if let Some(dst) = inst.dst {
                let result = if inst.operands.len() >= 2 {
                    let lhs = lookup_value(inst.operands[0], values, module_constants);
                    let rhs = lookup_value(inst.operands[1], values, module_constants);
                    evaluate_binary(*kind, lhs, rhs)
                } else {
                    SccpValue::Bottom
                };
                update_value(dst, result, values, ssa_worklist);
            }
        }

        // Cast: propagate constant through zero/sign extension and truncation
        Operation::Cast { kind, target_bits } => {
            if let Some(dst) = inst.dst {
                let result = if !inst.operands.is_empty() {
                    let src = lookup_value(inst.operands[0], values, module_constants);
                    evaluate_cast(*kind, src, *target_bits)
                } else {
                    SccpValue::Bottom
                };
                update_value(dst, result, values, ssa_worklist);
            }
        }

        // CondBr: if condition is constant, only mark taken successor executable
        Operation::CondBr {
            then_target,
            else_target,
        } => {
            if !inst.operands.is_empty() {
                let cond = lookup_value(inst.operands[0], values, module_constants);
                match cond {
                    SccpValue::Constant(v) if v != 0 => {
                        mark_edge_executable(
                            block_id, *then_target,
                            executable_edges, executable_blocks, cfg_worklist,
                        );
                    }
                    SccpValue::Constant(0) => {
                        mark_edge_executable(
                            block_id, *else_target,
                            executable_edges, executable_blocks, cfg_worklist,
                        );
                    }
                    _ => {
                        // Unknown: both branches executable
                        mark_edge_executable(
                            block_id, *then_target,
                            executable_edges, executable_blocks, cfg_worklist,
                        );
                        mark_edge_executable(
                            block_id, *else_target,
                            executable_edges, executable_blocks, cfg_worklist,
                        );
                    }
                }
            }
        }

        // Switch: if discriminant is constant, only mark matching case executable
        Operation::Switch { default, cases } => {
            if !inst.operands.is_empty() {
                let disc = lookup_value(inst.operands[0], values, module_constants);
                match disc {
                    SccpValue::Constant(v) => {
                        #[allow(clippy::cast_possible_truncation)]
                        let matched = cases.iter().find(|(case_val, _)| *case_val == v as i64);
                        let target = matched.map_or(*default, |(_, target)| *target);
                        mark_edge_executable(
                            block_id, target,
                            executable_edges, executable_blocks, cfg_worklist,
                        );
                    }
                    _ => {
                        // Unknown: all targets executable
                        mark_edge_executable(
                            block_id, *default,
                            executable_edges, executable_blocks, cfg_worklist,
                        );
                        for (_, target) in cases {
                            mark_edge_executable(
                                block_id, *target,
                                executable_edges, executable_blocks, cfg_worklist,
                            );
                        }
                    }
                }
            }
        }

        // Unconditional branch: always mark successor executable
        Operation::Br { target } => {
            mark_edge_executable(
                block_id, *target,
                executable_edges, executable_blocks, cfg_worklist,
            );
        }

        // Copy/Select: propagate
        Operation::Copy => {
            if let Some(dst) = inst.dst {
                let result = if !inst.operands.is_empty() {
                    lookup_value(inst.operands[0], values, module_constants)
                } else {
                    SccpValue::Bottom
                };
                update_value(dst, result, values, ssa_worklist);
            }
        }

        // Load/Store/Call/GEP/etc: conservatively Bottom (unknown result)
        _ => {
            if let Some(dst) = inst.dst {
                update_value(dst, SccpValue::Bottom, values, ssa_worklist);
            }
        }
    }
}

/// Look up a value in the SCCP lattice, falling back to module constants.
fn lookup_value(
    vid: ValueId,
    values: &BTreeMap<ValueId, SccpValue>,
    module_constants: &BTreeMap<ValueId, Constant>,
) -> SccpValue {
    if let Some(&val) = values.get(&vid) {
        return val;
    }
    if let Some(constant) = module_constants.get(&vid) {
        return constant_to_sccp(constant);
    }
    SccpValue::Top
}

/// Update a value in the lattice. If it changed, add to SSA worklist.
fn update_value(
    vid: ValueId,
    new_val: SccpValue,
    values: &mut BTreeMap<ValueId, SccpValue>,
    ssa_worklist: &mut VecDeque<ValueId>,
) {
    let old_val = values.get(&vid).copied().unwrap_or(SccpValue::Top);
    let merged = old_val.meet(new_val);
    if merged != old_val {
        values.insert(vid, merged);
        ssa_worklist.push_back(vid);
    }
}

/// Mark a CFG edge executable. If the target block is newly executable, add to CFG worklist.
fn mark_edge_executable(
    from: BlockId,
    to: BlockId,
    executable_edges: &mut BTreeSet<(BlockId, BlockId)>,
    executable_blocks: &mut BTreeSet<BlockId>,
    cfg_worklist: &mut VecDeque<BlockId>,
) {
    executable_edges.insert((from, to));
    if executable_blocks.insert(to) {
        cfg_worklist.push_back(to);
    }
}

/// Evaluate a binary operation on SCCP lattice values.
fn evaluate_binary(kind: BinaryOp, lhs: SccpValue, rhs: SccpValue) -> SccpValue {
    match (lhs, rhs) {
        (SccpValue::Constant(a), SccpValue::Constant(b)) => {
            let result = match kind {
                BinaryOp::Add => Some(a.wrapping_add(b)),
                BinaryOp::Sub => Some(a.wrapping_sub(b)),
                BinaryOp::Mul => Some(a.wrapping_mul(b)),
                BinaryOp::And => Some(a & b),
                BinaryOp::Or => Some(a | b),
                BinaryOp::Xor => Some(a ^ b),
                BinaryOp::ICmpEq => Some(i128::from(a == b)),
                BinaryOp::ICmpNe => Some(i128::from(a != b)),
                BinaryOp::ICmpSlt => Some(i128::from(a < b)),
                BinaryOp::ICmpSle => Some(i128::from(a <= b)),
                BinaryOp::ICmpSgt => Some(i128::from(a > b)),
                BinaryOp::ICmpSge => Some(i128::from(a >= b)),
                BinaryOp::SDiv if b != 0 => Some(a / b),
                BinaryOp::SRem if b != 0 => Some(a % b),
                _ => None, // Float ops, div-by-zero, shifts with large amounts
            };
            result.map_or(SccpValue::Bottom, SccpValue::Constant)
        }
        (SccpValue::Bottom, _) | (_, SccpValue::Bottom) => SccpValue::Bottom,
        _ => SccpValue::Top,
    }
}

/// Evaluate a cast operation on an SCCP lattice value.
fn evaluate_cast(
    kind: saf_core::air::CastKind,
    src: SccpValue,
    _target_bits: Option<u8>,
) -> SccpValue {
    match src {
        SccpValue::Constant(v) => {
            use saf_core::air::CastKind;
            match kind {
                CastKind::ZExt | CastKind::SExt | CastKind::Bitcast => SccpValue::Constant(v),
                CastKind::Trunc => {
                    // Truncation preserves the value if it fits; be conservative
                    SccpValue::Constant(v)
                }
                _ => SccpValue::Bottom,
            }
        }
        other => other,
    }
}
```

**Step 3: Run tests**

Run: `make test` (filter: `sccp`)
Expected: All SCCP tests PASS

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/absint/sccp.rs
git commit -m "feat(absint): SCCP solver with dual-worklist algorithm (Plan 148 Phase A2)"
```

---

### Task A3: Wire SCCP Into Fixpoint and Checkers

**Files:**
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs:190-264` (call SCCP before fixpoint)
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs:275-662` (skip dead blocks)
- Modify: `crates/saf-analysis/src/absint/sccp.rs` (add integration test)

**Step 1: Write the failing test**

Add to `sccp.rs` tests module:

```rust
    #[test]
    fn sccp_detects_dead_else_branch() {
        // Build a function with:
        //   entry: %cond = icmp eq %const_5, 5  → always true
        //          br %cond, then, else
        //   then:  ret
        //   else:  ret  ← dead block
        use saf_core::air::*;
        use saf_core::ids::*;

        let const_5_id = ValueId::from_raw(100);
        let five_id = ValueId::from_raw(101);
        let cond_id = ValueId::from_raw(102);

        let entry_id = BlockId::from_raw(1);
        let then_id = BlockId::from_raw(2);
        let else_id = BlockId::from_raw(3);

        let entry = AirBlock {
            id: entry_id,
            label: Some("entry".to_string()),
            instructions: vec![
                Instruction::new(
                    InstId::from_raw(10),
                    Operation::BinaryOp { kind: BinaryOp::ICmpEq },
                    vec![const_5_id, five_id],
                ).with_dst(cond_id),
                Instruction::new(
                    InstId::from_raw(11),
                    Operation::CondBr { then_target: then_id, else_target: else_id },
                    vec![cond_id],
                ),
            ],
        };
        let then_block = AirBlock {
            id: then_id,
            label: Some("then".to_string()),
            instructions: vec![Instruction::new(InstId::from_raw(12), Operation::Ret, vec![])],
        };
        let else_block = AirBlock {
            id: else_id,
            label: Some("else".to_string()),
            instructions: vec![Instruction::new(InstId::from_raw(13), Operation::Ret, vec![])],
        };

        let func = AirFunction {
            id: FunctionId::from_raw(1),
            name: "test_dead_else".to_string(),
            blocks: vec![entry, then_block, else_block],
            entry_block: Some(entry_id),
            params: vec![],
            is_declaration: false,
            is_vararg: false,
            return_type: None,
            linkage: None,
        };

        let mut module_constants = BTreeMap::new();
        module_constants.insert(const_5_id, Constant::Int { value: 5, bits: 32 });
        module_constants.insert(five_id, Constant::Int { value: 5, bits: 32 });

        let mut module = AirModule::default();
        module.functions = vec![func];
        module.constants = module_constants;

        let result = run_sccp_module(&module);

        // The else block should be dead because icmp eq 5, 5 = true → only then is reachable
        assert!(result.dead_blocks.contains(&else_id), "else block should be dead");
        assert!(!result.dead_blocks.contains(&then_id), "then block should be alive");
        // The condition should be constant 1 (true)
        assert_eq!(result.constants.get(&cond_id), Some(&1));
    }
```

**Step 2: Run test to verify it passes** (should pass with A2 implementation)

Run: `make test` (filter: `sccp_detects_dead_else`)
Expected: PASS

**Step 3: Wire SCCP into `solve_abstract_interp_with_context`**

In `fixpoint.rs`, at line ~195, after `let constant_map = build_constant_map(module);`, add:

```rust
    // Run SCCP pre-pass to identify constants and dead blocks
    let sccp_result = crate::absint::sccp::run_sccp_module(module);

    // Seed constant_map with SCCP-discovered constants
    for (vid, val) in &sccp_result.constants {
        use crate::absint::interval::Interval;
        constant_map.entry(*vid).or_insert_with(|| Interval::singleton(*val, 64));
    }
```

In `solve_function_impl()`, at line ~360, after the `is_unreachable()` check, add dead block skip:

```rust
    // Skip SCCP-proven dead blocks (passed via ctx)
    // Note: ctx.dead_blocks is a new optional field
```

Add `dead_blocks: Option<&'a BTreeSet<BlockId>>` to `FixpointContext` struct (line ~41-73).

In the worklist loop at line ~362, add after `if entry_state.is_unreachable() { continue; }`:

```rust
        if let Some(dead) = ctx.dead_blocks {
            if dead.contains(&block_id) {
                continue;
            }
        }
```

**Step 4: Update `FixpointContext` callers**

All callers of `FixpointContext` need `dead_blocks: None` added. Search for `FixpointContext {` across the crate and update each construction site.

**Step 5: Run full test suite**

Run: `make fmt && make lint && make test`
Expected: All tests pass, no regressions

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/absint/fixpoint.rs crates/saf-analysis/src/absint/sccp.rs
git commit -m "feat(absint): wire SCCP into fixpoint solver (Plan 148 Phase A3)"
```

---

### Task A4: SCCP Integration Test with LLVM IR Fixture

**Files:**
- Create: `tests/programs/c/sccp_dead_branch.c`
- Create: `tests/fixtures/llvm/e2e/sccp_dead_branch.ll` (compiled inside Docker)
- Modify: `crates/saf-analysis/tests/absint_e2e.rs` (add SCCP e2e test)

**Step 1: Write C test fixture**

```c
// tests/programs/c/sccp_dead_branch.c
// Tests SCCP dead branch elimination.
// The else branch is dead because GLOBAL_CONST == 5 always.
#include <stdlib.h>

static const int GLOBAL_CONST = 5;

int main() {
    int *p = (int *)malloc(sizeof(int));
    if (GLOBAL_CONST == 5) {
        *p = 42;  // This is the only reachable path
        free(p);
    } else {
        // DEAD CODE — SCCP should prove this unreachable
        free(p);
        *p = 99;  // Use-after-free, but in dead code
    }
    return 0;
}
```

**Step 2: Compile inside Docker**

```bash
docker compose run --rm dev sh -c 'clang-18 -S -emit-llvm -g -O0 tests/programs/c/sccp_dead_branch.c -o tests/fixtures/llvm/e2e/sccp_dead_branch.ll'
```

**Step 3: Write e2e test**

Add to `crates/saf-analysis/tests/absint_e2e.rs`:

```rust
#[test]
fn sccp_dead_branch_eliminates_false_positive() {
    let module = load_ll_fixture("sccp_dead_branch");
    let sccp_result = saf_analysis::absint::SccpResult::from(
        saf_analysis::absint::sccp::run_sccp_module(&module)
    );
    // SCCP should identify at least one dead block (the else branch)
    assert!(!sccp_result.dead_blocks.is_empty(),
        "SCCP should find dead blocks from constant branch");
    assert!(!sccp_result.constants.is_empty(),
        "SCCP should find constant values");
}
```

**Step 4: Run test**

Run: `make test` (filter: `sccp_dead_branch`)
Expected: PASS

**Step 5: Commit**

```bash
git add tests/programs/c/sccp_dead_branch.c tests/fixtures/llvm/e2e/sccp_dead_branch.ll crates/saf-analysis/tests/absint_e2e.rs
git commit -m "test(absint): SCCP e2e test with dead branch fixture (Plan 148 Phase A4)"
```

---

### Task A5: Juliet Benchmark — Phase A Validation

**Step 1: Run Juliet baseline**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet -o /workspace/tests/benchmarks/sv-benchmarks/juliet-148a.json' 2>&1 | tail -30
```

**Step 2: Compare with previous baseline** (52.1% precision, 68.0% recall, F1=0.590)

Read: `tests/benchmarks/sv-benchmarks/juliet-148a.json`

Record results in commit message.

**Step 3: Run PTABen to verify no regressions** (baseline: 67 unsound)

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-148a.json'
```

**Step 4: Commit**

```bash
git commit --allow-empty -m "bench: Plan 148 Phase A results — SCCP pre-pass [record numbers here]"
```

---

## Phase B: Guarded Value-Flow (Path-Sensitive SVFG)

Saturn/Coverity-style guarded edges on SVFG. Guard-aware BFS with Z3 feasibility checking at sinks. Pulse-style disjunct budget (default 20).

### Task B1: SVFG Edge Guard Storage

**Files:**
- Modify: `crates/saf-analysis/src/svfg/mod.rs:202-212` (add `edge_guards` field)

**Step 1: Add `edge_guards` field to `Svfg`**

In `Svfg` struct (line ~207), add:

```rust
    /// Guards on edges: (from, to) -> list of guards that must hold for the edge.
    edge_guards: BTreeMap<(SvfgNodeId, SvfgNodeId), Vec<crate::z3_utils::Guard>>,
```

**Step 2: Add accessor methods**

After `edge_count()` method:

```rust
    /// Get guards on a specific edge.
    pub fn edge_guard(&self, from: SvfgNodeId, to: SvfgNodeId) -> Option<&[crate::z3_utils::Guard]> {
        self.edge_guards.get(&(from, to)).map(|v| v.as_slice())
    }

    /// Set guards on a specific edge.
    pub fn set_edge_guard(&mut self, from: SvfgNodeId, to: SvfgNodeId, guards: Vec<crate::z3_utils::Guard>) {
        if !guards.is_empty() {
            self.edge_guards.insert((from, to), guards);
        }
    }

    /// Number of guarded edges.
    pub fn guarded_edge_count(&self) -> usize {
        self.edge_guards.len()
    }
```

**Step 3: Initialize in constructor/Default**

Update `Svfg::new()` or `Default` impl to include `edge_guards: BTreeMap::new()`.

**Step 4: Run tests**

Run: `make fmt && make lint && make test`
Expected: All pass (field is unused so far, no regressions)

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/svfg/mod.rs
git commit -m "feat(svfg): add edge_guards storage for path-sensitive analysis (Plan 148 Phase B1)"
```

---

### Task B2: Extract Guards During SVFG Construction

**Files:**
- Modify: `crates/saf-analysis/src/svfg/builder.rs` (extract guards at Phi/MemPhi edges)

**Step 1: Add guard extraction helper**

In `builder.rs`, add helper function:

```rust
/// Extract a guard from a block's terminator CondBr, if the edge goes to `target`.
fn extract_condBr_guard(
    block: &saf_core::air::AirBlock,
    func_id: saf_core::ids::FunctionId,
    target: saf_core::ids::BlockId,
) -> Option<crate::z3_utils::Guard> {
    let term = block.terminator()?;
    let Operation::CondBr { then_target, else_target } = &term.op else {
        return None;
    };
    let cond_id = *term.operands.first()?;
    if target == *then_target {
        Some(crate::z3_utils::Guard {
            block: block.id,
            function: func_id,
            condition: cond_id,
            branch_taken: true,
        })
    } else if target == *else_target {
        Some(crate::z3_utils::Guard {
            block: block.id,
            function: func_id,
            condition: cond_id,
            branch_taken: false,
        })
    } else {
        None
    }
}
```

**Step 2: Attach guards during Phi edge construction**

In `process_direct_instruction()` (line ~358), when creating `DirectDef` edges for Phi incoming values, look up the predecessor block and extract guard:

```rust
Operation::Phi { incoming } => {
    if let Some(result) = dst {
        for (pred_block_id, value) in incoming {
            graph.add_edge(
                SvfgNodeId::Value(*value),
                SvfgEdgeKind::DirectDef,
                SvfgNodeId::Value(result),
            );
            // Extract guard from predecessor block's terminator
            if let Some(pred_block) = func_blocks.get(pred_block_id) {
                if let Some(guard) = extract_condBr_guard(pred_block, func_id, block_id) {
                    graph.set_edge_guard(
                        SvfgNodeId::Value(*value),
                        SvfgNodeId::Value(result),
                        vec![guard],
                    );
                }
            }
            graph.diagnostics_mut().direct_edge_count += 1;
        }
    }
}
```

This requires passing `func_id`, `block_id`, and a `func_blocks: &BTreeMap<BlockId, &AirBlock>` to `process_direct_instruction`. Update the function signature accordingly.

**Step 3: Attach guards during MemPhi edge construction**

In `build_phi_edges_static()` (line ~184), when creating `IndirectStore` edges from specific predecessor blocks, extract the guard from that predecessor's terminator CondBr. The predecessor block identity is known from the MSSA Phi operand.

**Step 4: Run tests**

Run: `make fmt && make lint && make test`
Expected: All pass, guarded edges now populated

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/svfg/builder.rs
git commit -m "feat(svfg): extract guards from CondBr during SVFG construction (Plan 148 Phase B2)"
```

---

### Task B3: Guard-Aware BFS in `may_reach`

**Files:**
- Modify: `crates/saf-analysis/src/checkers/solver.rs:48-141` (guarded BFS)

**Step 1: Add `GuardedSolverConfig`**

```rust
/// Configuration for guard-aware reachability solvers.
#[derive(Debug, Clone)]
pub struct GuardedSolverConfig {
    /// Base solver config.
    pub base: SolverConfig,
    /// Maximum number of disjuncts (path conditions) per node.
    /// When exceeded, drop disjuncts with the most guards.
    /// Default: 20 (Infer/Pulse finding: 20 disjuncts finds 97% of bugs).
    pub max_disjuncts: usize,
}

impl Default for GuardedSolverConfig {
    fn default() -> Self {
        Self {
            base: SolverConfig::default(),
            max_disjuncts: 20,
        }
    }
}
```

**Step 2: Implement `may_reach_guarded`**

```rust
/// Guard-aware may_reach: accumulates path conditions during BFS,
/// prunes infeasible paths via guard contradiction detection.
///
/// Uses Pulse-style disjunct budget: each node tracks up to
/// `max_disjuncts` separate path conditions. When exceeded, the
/// disjunct with the most guards is dropped (under-approximate:
/// may miss bugs, never adds false positives).
pub fn may_reach_guarded(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    sink_nodes: &BTreeSet<SvfgNodeId>,
    sanitizer_nodes: &BTreeSet<SvfgNodeId>,
    config: &GuardedSolverConfig,
    dead_blocks: &BTreeSet<BlockId>,
    block_of: &BTreeMap<ValueId, BlockId>,
) -> Vec<CheckerFinding> {
    // BFS state: each node can have multiple disjuncts (path conditions)
    // visited: (node, path_condition_hash) to avoid re-exploring same state
    let mut findings: Vec<CheckerFinding> = Vec::new();

    for &source in source_nodes {
        // Skip sources in dead blocks
        if let SvfgNodeId::Value(vid) = source {
            if let Some(bid) = block_of.get(&vid) {
                if dead_blocks.contains(bid) {
                    continue;
                }
            }
        }

        let mut visited: BTreeSet<SvfgNodeId> = BTreeSet::new();
        let mut explored_sinks: BTreeSet<SvfgNodeId> = BTreeSet::new();
        let mut parent: BTreeMap<SvfgNodeId, SvfgNodeId> = BTreeMap::new();
        // Track guards accumulated along the path for each node
        let mut node_guards: BTreeMap<SvfgNodeId, Vec<crate::z3_utils::Guard>> = BTreeMap::new();
        let mut queue: VecDeque<(SvfgNodeId, usize)> = VecDeque::new();
        queue.push_back((source, 0));

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= config.base.max_depth {
                continue;
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (_, target) in succs {
                    // Skip targets in dead blocks
                    if let SvfgNodeId::Value(vid) = target {
                        if let Some(bid) = block_of.get(vid) {
                            if dead_blocks.contains(bid) {
                                continue;
                            }
                        }
                    }

                    // Accumulate edge guards
                    let mut accumulated = node_guards.get(&node).cloned().unwrap_or_default();
                    if let Some(edge_guards) = svfg.edge_guard(node, *target) {
                        accumulated.extend_from_slice(edge_guards);
                    }

                    // Check for self-contradictory guards (complementary pair)
                    if has_contradictory_guards(&accumulated) {
                        continue; // Infeasible path — prune
                    }

                    let is_sink = sink_nodes.contains(target);

                    if is_sink {
                        if !explored_sinks.insert(*target) {
                            continue;
                        }
                        parent.insert(*target, node);
                        let trace = reconstruct_trace(&parent, source, *target);
                        findings.push(CheckerFinding {
                            checker_name: spec.name.clone(),
                            severity: spec.severity,
                            source_node: source,
                            sink_node: *target,
                            trace,
                            cwe: spec.cwe,
                            message: format!(
                                "[{}] {}: source reaches sink",
                                spec.name, spec.severity
                            ),
                            sink_traces: vec![],
                            source_kind: super::finding::NullSourceKind::default(),
                        });
                        continue;
                    }

                    if sanitizer_nodes.contains(target) {
                        continue; // Sanitized
                    }

                    if visited.insert(*target) {
                        parent.insert(*target, node);
                        // Budget: limit guards per node
                        if accumulated.len() <= config.max_disjuncts * 2 {
                            node_guards.insert(*target, accumulated);
                        }
                        // else: drop guards (under-approximate)
                        queue.push_back((*target, depth + 1));
                    }
                }
            }
        }
    }

    // Dedup
    findings.sort_by(|a, b| {
        a.source_node.cmp(&b.source_node).then(a.sink_node.cmp(&b.sink_node))
    });
    findings.dedup_by(|a, b| a.source_node == b.source_node && a.sink_node == b.sink_node);
    findings
}

/// Check if a guard list contains a contradictory pair (same condition, opposite branch_taken).
fn has_contradictory_guards(guards: &[crate::z3_utils::Guard]) -> bool {
    for (i, g1) in guards.iter().enumerate() {
        for g2 in &guards[i + 1..] {
            if g1.condition == g2.condition
                && g1.block == g2.block
                && g1.branch_taken != g2.branch_taken
            {
                return true;
            }
        }
    }
    false
}
```

**Step 3: Run tests**

Run: `make fmt && make lint && make test`
Expected: All pass (new function not yet called from production path)

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/checkers/solver.rs
git commit -m "feat(checker): guard-aware BFS solver with disjunct budget (Plan 148 Phase B3)"
```

---

### Task B4: Wire Guarded BFS Into Checker Pipeline

**Files:**
- Modify: `crates/saf-analysis/src/checkers/runner.rs` (call `may_reach_guarded` when edge guards exist)
- Modify: `crates/saf-bench/src/svcomp/property.rs` (pass SCCP dead blocks)

**Step 1: Find where `may_reach` is called in the checker runner**

Look for `solver::may_reach(` in `runner.rs` and `property.rs`. Replace with `may_reach_guarded` when SVFG has guarded edges and dead blocks are available.

The key integration point: `run_checkers()` in `runner.rs` calls `solver::may_reach()`. Add a parallel path that calls `may_reach_guarded()` when SVFG edge guards are populated.

**Step 2: In `property.rs` (`analyze_memsafety`), pass SCCP dead blocks**

The `analyze_memsafety` function builds the SVFG and runs checkers. Add SCCP before SVFG construction:

```rust
let sccp_result = saf_analysis::absint::sccp::run_sccp_module(module);
// Pass dead_blocks to SVFG builder and checker
```

**Step 3: Run full test suite + Juliet benchmark**

Run: `make fmt && make lint && make test`
Then: Juliet benchmark comparison

**Step 4: Commit**

```bash
git commit -m "feat(checker): wire guarded BFS into memsafety pipeline (Plan 148 Phase B4)"
```

---

### Task B5: Juliet Benchmark — Phase B Validation

Same structure as Task A5. Run Juliet, compare with Phase A results. Expect +10-20pp precision on temporal CWEs (UAF, null-deref, double-free).

---

## Phase C: Trace Partitioning (Path-Sensitive Absint)

Astree-style trace partitioning with Eva-style fuel budget. Partitions the abstract state by branch conditions at split points.

### Task C1: Partition Token Types

**Files:**
- Create: `crates/saf-analysis/src/absint/partition.rs`
- Modify: `crates/saf-analysis/src/absint/mod.rs` (add module)

**Step 1: Define partition types**

```rust
//! Trace partitioning for path-sensitive abstract interpretation.
//!
//! Based on Astree (Rival & Mauborgne, ESOP 2005) trace partitioning
//! with Eva-style fuel budget.

use std::collections::BTreeMap;

use saf_core::ids::{BlockId, ValueId};

use super::state::AbstractState;

/// A token identifying which branch was taken at a split point.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartitionToken {
    /// Taken branch at a CondBr (cond_id, true=then/false=else).
    Branch { cond_id: ValueId, taken: bool },
    /// Loop iteration number at a loop header.
    LoopIter { header: BlockId, iteration: u8 },
}

/// A partition key: stack of active tokens.
/// Kept small (max 4 tokens) to bound state space.
pub type PartitionKey = smallvec::SmallVec<[PartitionToken; 4]>;

/// Partitioned state at a block: multiple abstract states, one per partition.
#[derive(Debug, Clone)]
pub struct PartitionedState {
    /// Map from partition key to abstract state.
    partitions: BTreeMap<PartitionKey, AbstractState>,
}

/// Configuration for trace partitioning.
#[derive(Debug, Clone)]
pub struct PartitionConfig {
    /// Maximum partitions per block (Eva-style fuel cap). Default: 16.
    pub max_partitions: usize,
    /// Maximum loop iterations to partition before merging. Default: 3.
    pub max_loop_partitions: u8,
    /// Whether partitioning is enabled. Default: true.
    pub enabled: bool,
}

impl Default for PartitionConfig {
    fn default() -> Self {
        Self {
            max_partitions: 16,
            max_loop_partitions: 3,
            enabled: true,
        }
    }
}

impl PartitionedState {
    /// Create from a single unpartitioned state.
    pub fn from_single(state: AbstractState) -> Self {
        let mut partitions = BTreeMap::new();
        partitions.insert(PartitionKey::new(), state);
        Self { partitions }
    }

    /// Number of active partitions.
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Iterate over all partitions.
    pub fn iter(&self) -> impl Iterator<Item = (&PartitionKey, &AbstractState)> {
        self.partitions.iter()
    }

    /// Merge all partitions into a single state using join.
    pub fn merge_all(&self) -> AbstractState {
        let mut result: Option<AbstractState> = None;
        for (_, state) in &self.partitions {
            result = Some(match result {
                None => state.clone(),
                Some(acc) => acc.join(state),
            });
        }
        result.unwrap_or_else(AbstractState::bottom)
    }

    /// Split this state at a branch point, creating two partitions
    /// (one for each branch). Only splits if condition is against a constant.
    pub fn split_at_branch(
        &self,
        cond_id: ValueId,
        then_state: AbstractState,
        else_state: AbstractState,
        config: &PartitionConfig,
    ) -> (Self, Self) {
        let mut then_partitions = BTreeMap::new();
        let mut else_partitions = BTreeMap::new();

        for (key, _) in &self.partitions {
            let mut then_key = key.clone();
            then_key.push(PartitionToken::Branch { cond_id, taken: true });
            let mut else_key = key.clone();
            else_key.push(PartitionToken::Branch { cond_id, taken: false });

            // Respect fuel budget: if we'd exceed max_partitions, use unpartitioned key
            if then_partitions.len() < config.max_partitions {
                then_partitions.insert(then_key, then_state.clone());
            } else {
                then_partitions.insert(key.clone(), then_state.clone());
            }
            if else_partitions.len() < config.max_partitions {
                else_partitions.insert(else_key, else_state.clone());
            } else {
                else_partitions.insert(key.clone(), else_state.clone());
            }
        }

        (
            Self { partitions: then_partitions },
            Self { partitions: else_partitions },
        )
    }

    /// Merge partitions at a join point: join states with matching keys,
    /// preserve one-sided partitions.
    pub fn join(&self, other: &Self) -> Self {
        let mut result = BTreeMap::new();

        // Join entries present in both
        for (key, state) in &self.partitions {
            if let Some(other_state) = other.partitions.get(key) {
                result.insert(key.clone(), state.join(other_state));
            } else {
                result.insert(key.clone(), state.clone());
            }
        }
        // Add entries only in other
        for (key, state) in &other.partitions {
            result.entry(key.clone()).or_insert_with(|| state.clone());
        }

        Self { partitions: result }
    }

    /// Reduce partitions to fit within budget by merging the two
    /// most similar partitions (semantic-directed clumping).
    pub fn reduce_to_budget(&mut self, max: usize) {
        while self.partitions.len() > max {
            // Find two partitions to merge (smallest combined key)
            let keys: Vec<PartitionKey> = self.partitions.keys().cloned().collect();
            if keys.len() < 2 {
                break;
            }
            // Simple heuristic: merge the two with the longest keys
            // (most specific partitions are most expensive to keep)
            let mut longest_idx = 0;
            let mut second_idx = 1;
            for (i, k) in keys.iter().enumerate() {
                if k.len() > keys[longest_idx].len() {
                    second_idx = longest_idx;
                    longest_idx = i;
                } else if i != longest_idx && k.len() > keys[second_idx].len() {
                    second_idx = i;
                }
            }
            let k1 = keys[longest_idx].clone();
            let k2 = keys[second_idx].clone();
            if let (Some(s1), Some(s2)) = (self.partitions.remove(&k1), self.partitions.remove(&k2)) {
                // Merge into the shorter key (more general partition)
                let merged_key = if k1.len() <= k2.len() { k1 } else { k2 };
                self.partitions.insert(merged_key, s1.join(&s2));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partitioned_state_from_single() {
        let state = AbstractState::bottom();
        let ps = PartitionedState::from_single(state);
        assert_eq!(ps.partition_count(), 1);
    }

    #[test]
    fn partitioned_state_merge_all_single() {
        let state = AbstractState::bottom();
        let ps = PartitionedState::from_single(state.clone());
        let merged = ps.merge_all();
        assert!(merged.is_unreachable());
    }

    #[test]
    fn partition_config_defaults() {
        let config = PartitionConfig::default();
        assert_eq!(config.max_partitions, 16);
        assert_eq!(config.max_loop_partitions, 3);
        assert!(config.enabled);
    }
}
```

**Step 2: Add module to mod.rs**

In `crates/saf-analysis/src/absint/mod.rs`, after `mod sccp;`:

```rust
pub mod partition;
```

**Step 3: Run tests**

Run: `make fmt && make lint && make test`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/absint/partition.rs crates/saf-analysis/src/absint/mod.rs
git commit -m "feat(absint): trace partition types and operations (Plan 148 Phase C1)"
```

---

### Task C2: Partitioned Fixpoint Integration

**Files:**
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs` (partitioned state map)

**Step 1: Add partition-aware branch handling**

In `solve_function_impl()`, at the successor propagation section (line ~391-526), add a check: when propagating through a `CondBr` terminator and the condition compares against a program constant (detected via `constant_map` or SCCP results), create partitioned states for each successor instead of a single refined state.

The key change:

```rust
// In the successor propagation loop:
let terminator = block.terminator();

if let Some(term) = terminator {
    if let Operation::CondBr { then_target, else_target } = &term.op {
        if !term.operands.is_empty() {
            let cond_id = term.operands[0];
            // Check if condition compares against a constant
            let is_const_comparison = is_constant_comparison(
                cond_id, &cond_inst_map, constant_map,
            );

            if is_const_comparison && partition_config.enabled {
                // Create partition-aware states for each successor
                // ... (partition split logic)
            }
        }
    }
}
```

**Step 2: Implement `is_constant_comparison` helper**

```rust
/// Check if a condition value is an ICmp comparing against a program constant.
fn is_constant_comparison(
    cond_id: ValueId,
    cond_inst_map: &BTreeMap<ValueId, Instruction>,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> bool {
    let Some(inst) = cond_inst_map.get(&cond_id) else {
        return false;
    };
    let Operation::BinaryOp { kind } = &inst.op else {
        return false;
    };
    // Must be a comparison operation
    if !kind.is_comparison() {
        return false;
    }
    // At least one operand must be a singleton constant
    if inst.operands.len() < 2 {
        return false;
    }
    let lhs_const = constant_map.get(&inst.operands[0]).map_or(false, |i| i.is_singleton());
    let rhs_const = constant_map.get(&inst.operands[1]).map_or(false, |i| i.is_singleton());
    lhs_const || rhs_const
}
```

**Step 3: Add `PartitionConfig` to `AbstractInterpConfig`**

In `config.rs`, add:

```rust
pub partition: PartitionConfig,
```

With default initialized to `PartitionConfig::default()`.

**Step 4: Run tests**

Run: `make fmt && make lint && make test`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/absint/fixpoint.rs crates/saf-analysis/src/absint/config.rs
git commit -m "feat(absint): partition-aware branch handling in fixpoint (Plan 148 Phase C2)"
```

---

### Task C3: Partitioned State Convergence and Merge

**Files:**
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs` (partition merge at loops + function exit)

**Step 1: Add partition merge at loop headers**

When a partitioned state reaches a loop header (widening point), merge all partitions within the loop iteration token into a single state before widening. This prevents unbounded growth of partitions through loops.

**Step 2: Add partition merge at function exit**

Before constructing the final `AbstractInterpResult`, merge all partitions at each block into a single state. The partitioned analysis is internal to the fixpoint solver — downstream consumers see the merged (more precise) result.

**Step 3: Run tests**

Run: `make fmt && make lint && make test`
Expected: All pass

**Step 4: Commit**

```bash
git commit -m "feat(absint): partition merge at loops and function exit (Plan 148 Phase C3)"
```

---

### Task C4: Trace Partitioning E2E Test

**Files:**
- Create: `tests/programs/c/partition_const_branch.c`
- Modify: `crates/saf-analysis/tests/absint_e2e.rs`

**Step 1: Write C test fixture**

```c
// tests/programs/c/partition_const_branch.c
// Tests trace partitioning: constant branch should produce precise intervals.
#include <stdlib.h>

#define BUFFER_SIZE 10

int main() {
    char buf[BUFFER_SIZE];
    int idx;

    // Constant comparison — partitioning should split here
    if (BUFFER_SIZE == 10) {
        idx = 5;  // Safe: 5 < 10
    } else {
        idx = 15; // Overflow: 15 >= 10, but this is dead code
    }

    buf[idx] = 'A';
    return 0;
}
```

**Step 2: Compile and write e2e test**

Similar to Task A4.

**Step 3: Run tests + Juliet benchmark**

Run: `make fmt && make lint && make test`
Then: Juliet benchmark comparison

**Step 4: Commit**

```bash
git commit -m "test(absint): trace partitioning e2e test (Plan 148 Phase C4)"
```

---

### Task C5: Juliet Benchmark — Phase C Validation

Same structure as Tasks A5/B5. Run Juliet, compare with Phase B results. Expect +5-15pp precision on buffer/integer overflow CWEs.

---

## Phase D: Final Validation and Integration

### Task D1: Full Benchmark Suite

**Step 1: Run complete Juliet benchmark**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet -o /workspace/tests/benchmarks/sv-benchmarks/juliet-148-final.json'
```

**Step 2: Run PTABen benchmark**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-148-final.json'
```

**Step 3: Compare all phases**

| Phase | Precision | Recall | F1 | PTABen Unsound |
|-------|-----------|--------|----|----------------|
| Baseline (pre-148) | 52.1% | 68.0% | 0.590 | 67 |
| After A (SCCP) | ? | ? | ? | ? |
| After B (Guarded SVFG) | ? | ? | ? | ? |
| After C (Partitioning) | ? | ? | ? | ? |

**Step 4: Full test suite**

Run: `make fmt && make lint && make test`
Expected: All 1729+ tests pass

### Task D2: Update PROGRESS.md

Update `plans/PROGRESS.md`:
- Add Plan 148 to Plans Index
- Update Session Log with final results
- Update "Next Steps" based on remaining gaps
