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

use saf_core::air::{AirFunction, AirModule, BinaryOp, CastKind, Constant, Operation};
use saf_core::ids::{BlockId, ValueId};

// =============================================================================
// Lattice
// =============================================================================

/// Three-level SCCP lattice value.
///
/// Ordering: `Top` (unknown) > `Constant(c)` > `Bottom` (overdetermined).
/// The meet operation lowers values in this lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SccpValue {
    /// Not yet analyzed — may become any constant.
    Top,
    /// Proven to be exactly this integer constant.
    Constant(i128),
    /// Proven to have multiple possible values (overdetermined).
    Bottom,
}

impl SccpValue {
    /// Lattice meet: greatest lower bound.
    ///
    /// - `Top` meet X = X (Top is identity)
    /// - `Constant(c)` meet `Constant(c)` = `Constant(c)` (same constant is stable)
    /// - `Constant(a)` meet `Constant(b)` = `Bottom` (different constants overdetermine)
    /// - `Bottom` meet X = `Bottom` (Bottom absorbs)
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        match (self, other) {
            (Self::Top, x) | (x, Self::Top) => x,
            (Self::Bottom, _) | (_, Self::Bottom) => Self::Bottom,
            (Self::Constant(a), Self::Constant(b)) => {
                if a == b {
                    Self::Constant(a)
                } else {
                    Self::Bottom
                }
            }
        }
    }

    /// Extract the constant value, if this is a `Constant`.
    #[must_use]
    pub fn as_constant(self) -> Option<i128> {
        match self {
            Self::Constant(c) => Some(c),
            _ => None,
        }
    }
}

// =============================================================================
// Result
// =============================================================================

/// Results from an SCCP analysis pass.
#[derive(Debug, Clone, Default)]
pub struct SccpResult {
    /// Values proven to be a single integer constant.
    pub constants: BTreeMap<ValueId, i128>,
    /// Blocks proven unreachable via constant branch resolution.
    pub dead_blocks: BTreeSet<BlockId>,
}

// =============================================================================
// Module-level entry point
// =============================================================================

/// Run SCCP on all non-declaration functions in a module.
///
/// Merges per-function results into a single `SccpResult`.
/// Detects read-only globals (never stored to) and propagates their
/// initial values through `Load` instructions.
#[must_use]
pub fn run_sccp_module(module: &AirModule) -> SccpResult {
    let read_only_globals = build_read_only_globals(module);

    let mut result = SccpResult::default();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let func_result = run_sccp_function(func, &module.constants, &read_only_globals);
        result.constants.extend(func_result.constants);
        result.dead_blocks.extend(func_result.dead_blocks);
    }
    result
}

/// Build a map of read-only global addresses to their initial constant values.
///
/// A global is "read-only" if its address never appears as the pointer operand
/// of a `Store` instruction anywhere in the module. For such globals, `Load`
/// from their address always returns the initialization value.
///
/// This handles patterns like `static int staticTrue = 1` and
/// `const int GLOBAL_CONST_FIVE = 5` in the Juliet test suite.
fn build_read_only_globals(module: &AirModule) -> BTreeMap<ValueId, SccpValue> {
    // Collect all global addresses that are stored to
    let mut stored_to: BTreeSet<ValueId> = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::Store) {
                    // Store operands: [value, pointer]
                    if let Some(&ptr) = inst.operands.get(1) {
                        stored_to.insert(ptr);
                    }
                }
            }
        }
    }

    // Build map for globals that are never stored to and have an init value
    let mut read_only = BTreeMap::new();
    for global in &module.globals {
        if stored_to.contains(&global.id) {
            continue;
        }
        if let Some(ref init) = global.init {
            let val = constant_to_sccp(init);
            if !matches!(val, SccpValue::Bottom) {
                read_only.insert(global.id, val);
            }
        }
    }
    read_only
}

// =============================================================================
// Function-level solver (dual-worklist algorithm)
// =============================================================================

/// Run SCCP on a single function.
///
/// Implements the dual-worklist Wegman-Zadeck algorithm:
/// 1. CFG worklist tracks newly executable blocks.
/// 2. SSA worklist tracks values whose lattice state changed.
///
/// The `module_constants` map provides constant values for `ValueId`s that
/// correspond to module-level constants (e.g., literal integers in operands).
/// The `read_only_globals` map provides constant values for globals that are
/// never stored to — `Load` from these addresses returns the init value.
#[must_use]
pub fn run_sccp_function(
    func: &AirFunction,
    module_constants: &BTreeMap<ValueId, Constant>,
    read_only_globals: &BTreeMap<ValueId, SccpValue>,
) -> SccpResult {
    // Value lattice: all values start at Top (unknown).
    let mut values: BTreeMap<ValueId, SccpValue> = BTreeMap::new();

    // Executable edges (from_block, to_block) — tracks which CFG edges have been taken.
    let mut executable_edges: BTreeSet<(BlockId, BlockId)> = BTreeSet::new();

    // Executable blocks — blocks reached by at least one executable edge.
    let mut executable_blocks: BTreeSet<BlockId> = BTreeSet::new();

    // Worklists.
    let mut cfg_worklist: VecDeque<BlockId> = VecDeque::new();
    let mut ssa_worklist: VecDeque<ValueId> = VecDeque::new();

    // Seed module constants into the value lattice.
    for (vid, constant) in module_constants {
        let sccp_val = constant_to_sccp(constant);
        values.insert(*vid, sccp_val);
    }

    // Mark entry block executable.
    let entry_id = match func.entry_block {
        Some(id) => id,
        None => {
            if let Some(first) = func.blocks.first() {
                first.id
            } else {
                return SccpResult::default();
            }
        }
    };
    executable_blocks.insert(entry_id);
    cfg_worklist.push_back(entry_id);

    // Main solver loop.
    let mut iterations = 0;
    let max_iterations = 10_000;

    while (!cfg_worklist.is_empty() || !ssa_worklist.is_empty()) && iterations < max_iterations {
        iterations += 1;

        // Process CFG worklist: evaluate all instructions in newly executable blocks.
        while let Some(block_id) = cfg_worklist.pop_front() {
            if let Some(block) = func.block(block_id) {
                for inst in &block.instructions {
                    evaluate_instruction(
                        inst,
                        block_id,
                        &mut values,
                        module_constants,
                        read_only_globals,
                        &mut ssa_worklist,
                        &mut executable_edges,
                        &mut executable_blocks,
                        &mut cfg_worklist,
                    );
                }
            }
        }

        // Process SSA worklist: re-evaluate instructions that use changed values.
        while let Some(changed_vid) = ssa_worklist.pop_front() {
            // Find all instructions that use this value and are in executable blocks.
            for block in &func.blocks {
                if !executable_blocks.contains(&block.id) {
                    continue;
                }
                for inst in &block.instructions {
                    let uses_value = inst.operands.contains(&changed_vid)
                        || matches!(&inst.op, Operation::Phi { incoming } if incoming.iter().any(|(_, v)| *v == changed_vid));
                    if uses_value {
                        evaluate_instruction(
                            inst,
                            block.id,
                            &mut values,
                            module_constants,
                            read_only_globals,
                            &mut ssa_worklist,
                            &mut executable_edges,
                            &mut executable_blocks,
                            &mut cfg_worklist,
                        );
                    }
                }
            }
        }
    }

    // Collect results.
    let mut result = SccpResult::default();

    // Gather proven constants.
    for (vid, val) in &values {
        if let Some(c) = val.as_constant() {
            result.constants.insert(*vid, c);
        }
    }

    // Gather dead blocks (exist in function but never became executable).
    for block in &func.blocks {
        if !executable_blocks.contains(&block.id) {
            result.dead_blocks.insert(block.id);
        }
    }

    result
}

// =============================================================================
// Helpers
// =============================================================================

/// Convert an AIR `Constant` to an `SccpValue`.
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
        // Float, String, Undef, Aggregate, GlobalRef — not integer-foldable.
        _ => SccpValue::Bottom,
    }
}

/// Look up the SCCP value for a `ValueId`.
///
/// Checks the local lattice first, then falls back to module constants.
/// Returns `Top` if the value has never been seen (optimistic assumption).
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

/// Meet-update a value in the lattice. If the value changes (lowers), add it
/// to the SSA worklist for re-propagation.
fn update_value(
    vid: ValueId,
    new_val: SccpValue,
    values: &mut BTreeMap<ValueId, SccpValue>,
    ssa_worklist: &mut VecDeque<ValueId>,
) {
    let old = values.get(&vid).copied().unwrap_or(SccpValue::Top);
    let met = old.meet(new_val);
    if met != old {
        values.insert(vid, met);
        ssa_worklist.push_back(vid);
    }
}

/// Mark a CFG edge executable. If the target block is newly reached, add it
/// to the CFG worklist.
fn mark_edge_executable(
    from: BlockId,
    to: BlockId,
    edges: &mut BTreeSet<(BlockId, BlockId)>,
    blocks: &mut BTreeSet<BlockId>,
    worklist: &mut VecDeque<BlockId>,
) {
    edges.insert((from, to));
    if blocks.insert(to) {
        // Block was not previously executable — schedule it.
        worklist.push_back(to);
    }
}

/// Evaluate a single instruction, updating the lattice and worklists.
// NOTE: This function implements the SCCP instruction evaluation for all
// AIR operations as a single cohesive unit. Splitting would obscure the algorithm.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn evaluate_instruction(
    inst: &saf_core::air::Instruction,
    block_id: BlockId,
    values: &mut BTreeMap<ValueId, SccpValue>,
    module_constants: &BTreeMap<ValueId, Constant>,
    read_only_globals: &BTreeMap<ValueId, SccpValue>,
    ssa_worklist: &mut VecDeque<ValueId>,
    executable_edges: &mut BTreeSet<(BlockId, BlockId)>,
    executable_blocks: &mut BTreeSet<BlockId>,
    cfg_worklist: &mut VecDeque<BlockId>,
) {
    match &inst.op {
        // -----------------------------------------------------------------
        // Phi: meet over executable predecessors
        // -----------------------------------------------------------------
        Operation::Phi { incoming } => {
            if let Some(dst) = inst.dst {
                let mut result = SccpValue::Top;
                for (pred_block, val_id) in incoming {
                    // Only consider values from executable predecessors.
                    if executable_edges.contains(&(*pred_block, block_id)) {
                        let val = lookup_value(*val_id, values, module_constants);
                        result = result.meet(val);
                    }
                }
                update_value(dst, result, values, ssa_worklist);
            }
        }

        // -----------------------------------------------------------------
        // Binary operations
        // -----------------------------------------------------------------
        Operation::BinaryOp { kind } => {
            if let Some(dst) = inst.dst {
                if inst.operands.len() >= 2 {
                    let lhs = lookup_value(inst.operands[0], values, module_constants);
                    let rhs = lookup_value(inst.operands[1], values, module_constants);
                    let result = evaluate_binary(*kind, lhs, rhs);
                    update_value(dst, result, values, ssa_worklist);
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Cast operations
        // -----------------------------------------------------------------
        Operation::Cast { kind, target_bits } => {
            if let Some(dst) = inst.dst {
                if let Some(&src_vid) = inst.operands.first() {
                    let src = lookup_value(src_vid, values, module_constants);
                    let result = evaluate_cast(*kind, src, *target_bits);
                    update_value(dst, result, values, ssa_worklist);
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Conditional branch
        // -----------------------------------------------------------------
        Operation::CondBr {
            then_target,
            else_target,
        } => {
            let cond = inst
                .operands
                .first()
                .map(|vid| lookup_value(*vid, values, module_constants));

            if let Some(SccpValue::Constant(c)) = cond {
                if c != 0 {
                    mark_edge_executable(
                        block_id,
                        *then_target,
                        executable_edges,
                        executable_blocks,
                        cfg_worklist,
                    );
                } else {
                    mark_edge_executable(
                        block_id,
                        *else_target,
                        executable_edges,
                        executable_blocks,
                        cfg_worklist,
                    );
                }
            } else {
                // Top, Bottom, or malformed — conservatively mark both.
                mark_edge_executable(
                    block_id,
                    *then_target,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
                mark_edge_executable(
                    block_id,
                    *else_target,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
            }
        }

        // -----------------------------------------------------------------
        // Switch
        // -----------------------------------------------------------------
        Operation::Switch { default, cases } => {
            let disc = inst
                .operands
                .first()
                .map(|vid| lookup_value(*vid, values, module_constants));

            if let Some(SccpValue::Constant(c)) = disc {
                // Find the matching case, or fall through to default.
                let target = cases
                    .iter()
                    .find(|(val, _)| i128::from(*val) == c)
                    .map_or(*default, |(_, tgt)| *tgt);
                mark_edge_executable(
                    block_id,
                    target,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
            } else {
                // Top, Bottom, or malformed — mark all targets.
                mark_edge_executable(
                    block_id,
                    *default,
                    executable_edges,
                    executable_blocks,
                    cfg_worklist,
                );
                for (_, tgt) in cases {
                    mark_edge_executable(
                        block_id,
                        *tgt,
                        executable_edges,
                        executable_blocks,
                        cfg_worklist,
                    );
                }
            }
        }

        // -----------------------------------------------------------------
        // Unconditional branch
        // -----------------------------------------------------------------
        Operation::Br { target } => {
            mark_edge_executable(
                block_id,
                *target,
                executable_edges,
                executable_blocks,
                cfg_worklist,
            );
        }

        // -----------------------------------------------------------------
        // Copy (identity)
        // -----------------------------------------------------------------
        Operation::Copy => {
            if let Some(dst) = inst.dst {
                if let Some(&src_vid) = inst.operands.first() {
                    let src = lookup_value(src_vid, values, module_constants);
                    update_value(dst, src, values, ssa_worklist);
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Load: check if loading from a read-only global with known init
        // -----------------------------------------------------------------
        Operation::Load => {
            if let Some(dst) = inst.dst {
                if let Some(&ptr) = inst.operands.first() {
                    if let Some(&global_val) = read_only_globals.get(&ptr) {
                        // Loading from a read-only global — use its init value
                        update_value(dst, global_val, values, ssa_worklist);
                    } else {
                        update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                    }
                } else {
                    update_value(dst, SccpValue::Bottom, values, ssa_worklist);
                }
            }
        }

        // -----------------------------------------------------------------
        // Default: Store, Call, Alloca, Gep, etc. — overdetermined
        // -----------------------------------------------------------------
        _ => {
            if let Some(dst) = inst.dst {
                update_value(dst, SccpValue::Bottom, values, ssa_worklist);
            }
        }
    }
}

/// Evaluate a binary operation when both operands may be constant.
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
                BinaryOp::SDiv => {
                    if b != 0 {
                        Some(a.wrapping_div(b))
                    } else {
                        None
                    }
                }
                BinaryOp::SRem => {
                    if b != 0 {
                        Some(a.wrapping_rem(b))
                    } else {
                        None
                    }
                }
                // Unsigned comparisons, float ops, shifts — not folded here.
                _ => None,
            };
            result.map_or(SccpValue::Bottom, SccpValue::Constant)
        }
        // If either operand is Bottom, the result is Bottom.
        (SccpValue::Bottom, _) | (_, SccpValue::Bottom) => SccpValue::Bottom,
        // If either operand is still Top, we don't know yet — stay Top.
        _ => SccpValue::Top,
    }
}

/// Evaluate a cast operation when the source may be constant.
///
/// For `ZExt`, `SExt`, and `Bitcast`, the integer value passes through
/// (SCCP uses `i128` which can represent all AIR integer widths).
/// For `Trunc`, we apply a bit mask to correctly model narrowing casts
/// (e.g., truncating `i128` value 256 to `i8` yields 0, not 256).
fn evaluate_cast(kind: CastKind, src: SccpValue, target_bits: Option<u8>) -> SccpValue {
    match kind {
        CastKind::ZExt | CastKind::SExt | CastKind::Bitcast => src,
        CastKind::Trunc => match (src, target_bits) {
            (SccpValue::Constant(v), Some(bits)) if bits < 128 => {
                SccpValue::Constant(v & ((1i128 << bits) - 1))
            }
            // No target_bits info or non-constant — pass through unchanged.
            _ => src,
        },
        // Float-to-int, ptr-to-int, etc. — not folded.
        _ => SccpValue::Bottom,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Lattice tests ----

    #[test]
    fn sccp_lattice_meet_top_with_const() {
        assert_eq!(
            SccpValue::Top.meet(SccpValue::Constant(5)),
            SccpValue::Constant(5)
        );
    }

    #[test]
    fn sccp_lattice_meet_same_const() {
        assert_eq!(
            SccpValue::Constant(5).meet(SccpValue::Constant(5)),
            SccpValue::Constant(5)
        );
    }

    #[test]
    fn sccp_lattice_meet_different_const() {
        assert_eq!(
            SccpValue::Constant(5).meet(SccpValue::Constant(3)),
            SccpValue::Bottom
        );
    }

    #[test]
    fn sccp_lattice_meet_bottom_absorbs() {
        assert_eq!(
            SccpValue::Bottom.meet(SccpValue::Constant(5)),
            SccpValue::Bottom
        );
    }

    // ---- Solver tests ----

    #[test]
    fn sccp_run_trivial_function() {
        use saf_core::air::{AirBlock, Instruction};
        use saf_core::ids::{FunctionId, InstId, ModuleId};

        let block_id = BlockId::new(1);
        let func_id = FunctionId::new(2);
        let mut block = AirBlock::new(block_id);
        block.instructions = vec![Instruction::new(InstId::new(3), Operation::Ret)];
        let mut func = AirFunction::new(func_id, "test");
        func.blocks = vec![block];
        func.entry_block = Some(block_id);
        func.rebuild_block_index();

        let module = AirModule::new(ModuleId::new(0));
        // Module has no functions added, test with standalone function
        let result = run_sccp_function(&func, &module.constants, &BTreeMap::new());
        assert!(result.dead_blocks.is_empty());
    }

    #[test]
    fn sccp_detects_dead_else_branch() {
        use saf_core::air::*;
        use saf_core::ids::*;

        let const_5_id = ValueId::new(100);
        let five_id = ValueId::new(101);
        let cond_id = ValueId::new(102);

        let entry_id = BlockId::new(1);
        let then_id = BlockId::new(2);
        let else_id = BlockId::new(3);

        let mut entry = AirBlock::with_label(entry_id, "entry");
        entry.instructions = vec![
            Instruction::new(
                InstId::new(10),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
            )
            .with_operands(vec![const_5_id, five_id])
            .with_dst(cond_id),
            Instruction::new(
                InstId::new(11),
                Operation::CondBr {
                    then_target: then_id,
                    else_target: else_id,
                },
            )
            .with_operands(vec![cond_id]),
        ];

        let mut then_block = AirBlock::with_label(then_id, "then");
        then_block.instructions = vec![Instruction::new(InstId::new(12), Operation::Ret)];

        let mut else_block = AirBlock::with_label(else_id, "else");
        else_block.instructions = vec![Instruction::new(InstId::new(13), Operation::Ret)];

        let mut func = AirFunction::new(FunctionId::new(1), "test_dead_else");
        func.blocks = vec![entry, then_block, else_block];
        func.entry_block = Some(entry_id);
        func.rebuild_block_index();

        let mut module_constants = BTreeMap::new();
        module_constants.insert(const_5_id, Constant::Int { value: 5, bits: 32 });
        module_constants.insert(five_id, Constant::Int { value: 5, bits: 32 });

        let result = run_sccp_function(&func, &module_constants, &BTreeMap::new());

        assert!(
            result.dead_blocks.contains(&else_id),
            "else block should be dead"
        );
        assert!(
            !result.dead_blocks.contains(&then_id),
            "then block should be alive"
        );
        assert_eq!(result.constants.get(&cond_id), Some(&1));
    }

    // ---- Cast evaluation tests ----

    #[test]
    fn evaluate_cast_trunc_masks_bits() {
        // 256 truncated to i8 should become 0 (256 & 0xFF = 0)
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(256), Some(8)),
            SccpValue::Constant(0)
        );
    }

    #[test]
    fn evaluate_cast_trunc_preserves_in_range_value() {
        // 255 truncated to i8 stays 255 (255 & 0xFF = 255)
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(255), Some(8)),
            SccpValue::Constant(255)
        );
    }

    #[test]
    fn evaluate_cast_trunc_no_target_bits_passes_through() {
        // Without target_bits info, value passes through unchanged
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Constant(256), None),
            SccpValue::Constant(256)
        );
    }

    #[test]
    fn evaluate_cast_trunc_non_constant_passes_through() {
        // Non-constant values pass through unchanged
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Top, Some(8)),
            SccpValue::Top
        );
        assert_eq!(
            evaluate_cast(CastKind::Trunc, SccpValue::Bottom, Some(8)),
            SccpValue::Bottom
        );
    }

    #[test]
    fn evaluate_cast_zext_sext_bitcast_pass_through() {
        // Extension and bitcast always pass through
        assert_eq!(
            evaluate_cast(CastKind::ZExt, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
        assert_eq!(
            evaluate_cast(CastKind::SExt, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
        assert_eq!(
            evaluate_cast(CastKind::Bitcast, SccpValue::Constant(42), Some(64)),
            SccpValue::Constant(42)
        );
    }
}
