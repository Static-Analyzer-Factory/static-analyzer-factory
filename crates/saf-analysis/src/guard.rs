//! Guard extraction from branch conditions.
//!
//! Provides types and functions for extracting branch guards from traces
//! and block sequences for Z3 feasibility checking.

use std::collections::BTreeMap;

use saf_core::air::{AirModule, BinaryOp, Constant, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ObjId, ValueId};

use crate::svfg::SvfgNodeId;

// ---------------------------------------------------------------------------
// Guard types
// ---------------------------------------------------------------------------

/// A guard condition extracted from a `CondBr` terminator.
#[derive(Debug, Clone)]
pub struct Guard {
    /// The block containing the `CondBr`.
    pub block: BlockId,
    /// The function containing the block.
    pub function: FunctionId,
    /// The `ValueId` of the ICmp/comparison result feeding `CondBr`.
    pub condition: ValueId,
    /// `true` = the trace took the then-branch, `false` = else-branch.
    pub branch_taken: bool,
}

/// Collected guards along a checker trace or dominator chain.
#[derive(Debug, Clone)]
pub struct PathCondition {
    /// Guards extracted along the trace path or dominator chain.
    pub guards: Vec<Guard>,
}

impl PathCondition {
    /// Create an empty path condition.
    #[must_use]
    pub fn empty() -> Self {
        Self { guards: Vec::new() }
    }

    /// Whether this path condition has any guards to check.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Condition info (for Z3 translation)
// ---------------------------------------------------------------------------

/// Information about a comparison instruction, used for Z3 translation.
#[derive(Debug, Clone)]
pub struct ConditionInfo {
    /// The comparison kind.
    pub cmp_kind: BinaryOp,
    /// Left operand.
    pub lhs: OperandInfo,
    /// Right operand.
    pub rhs: OperandInfo,
}

/// An operand in a condition — either a known constant or an opaque value.
#[derive(Debug, Clone)]
pub enum OperandInfo {
    /// A known integer constant.
    IntConst(i64),
    /// A null pointer (treated as integer 0).
    Null,
    /// An opaque value — will become a fresh Z3 variable.
    Value(ValueId),
}

// ---------------------------------------------------------------------------
// ValueId → (FunctionId, BlockId) lookup
// ---------------------------------------------------------------------------

/// Pre-built index mapping `ValueId` → `(FunctionId, BlockId)`.
///
/// This allows us to find which block a given SVFG node (value) belongs to,
/// so we can check if consecutive trace nodes cross a block boundary with a
/// `CondBr` terminator.
#[derive(Debug)]
pub struct ValueLocationIndex {
    /// `ValueId` → `(FunctionId, BlockId)` for instruction results.
    value_to_location: BTreeMap<ValueId, (FunctionId, BlockId)>,
    /// `InstId` → `(FunctionId, BlockId)` for all instructions (including those without dst).
    inst_to_location: BTreeMap<InstId, (FunctionId, BlockId)>,
    /// `ValueId` → condition info (for ICmp instructions).
    value_to_condition: BTreeMap<ValueId, ConditionInfo>,
    /// `BlockId` → terminator info.
    block_terminators: BTreeMap<BlockId, TerminatorInfo>,
    /// `ValueId` → resolved integer constant (for constant globals).
    value_to_global_const: BTreeMap<ValueId, i64>,
}

/// Terminator information for a block.
#[derive(Debug, Clone)]
pub enum TerminatorInfo {
    /// Conditional branch.
    CondBr {
        /// The condition `ValueId` (operands[0] of the `CondBr` instruction).
        condition: ValueId,
        /// Block reached when condition is true.
        then_target: BlockId,
        /// Block reached when condition is false.
        else_target: BlockId,
    },
    /// Non-conditional terminator (Br, Ret, Switch, Unreachable).
    Other,
}

impl ValueLocationIndex {
    /// Build the index from an `AirModule`.
    #[must_use]
    pub fn build(module: &AirModule) -> Self {
        let mut value_to_location = BTreeMap::new();
        let mut inst_to_location = BTreeMap::new();
        let mut value_to_condition = BTreeMap::new();
        let mut block_terminators = BTreeMap::new();

        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            for block in &func.blocks {
                for inst in &block.instructions {
                    // Map instruction id to (function, block)
                    inst_to_location.insert(inst.id, (func.id, block.id));

                    // Map instruction result to (function, block)
                    if let Some(dst) = inst.dst {
                        value_to_location.insert(dst, (func.id, block.id));
                    }

                    // Map comparison instructions to condition info
                    if let Operation::BinaryOp { kind } = &inst.op {
                        if is_icmp(*kind) {
                            if let Some(dst) = inst.dst {
                                if inst.operands.len() >= 2 {
                                    let lhs = resolve_operand(inst.operands[0], module);
                                    let rhs = resolve_operand(inst.operands[1], module);
                                    value_to_condition.insert(
                                        dst,
                                        ConditionInfo {
                                            cmp_kind: *kind,
                                            lhs,
                                            rhs,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                // Record terminator
                if let Some(term) = block.terminator() {
                    let info = match &term.op {
                        Operation::CondBr {
                            then_target,
                            else_target,
                        } => {
                            if let Some(&cond_val) = term.operands.first() {
                                TerminatorInfo::CondBr {
                                    condition: cond_val,
                                    then_target: *then_target,
                                    else_target: *else_target,
                                }
                            } else {
                                TerminatorInfo::Other
                            }
                        }
                        _ => TerminatorInfo::Other,
                    };
                    block_terminators.insert(block.id, info);
                }
            }
        }

        // --- Phase 2: resolve global constants into condition operands ---
        let value_to_global_const = resolve_global_constants(module, &mut value_to_condition);

        Self {
            value_to_location,
            inst_to_location,
            value_to_condition,
            block_terminators,
            value_to_global_const,
        }
    }

    /// Look up the block a `ValueId` belongs to.
    #[must_use]
    pub fn block_of(&self, vid: ValueId) -> Option<(FunctionId, BlockId)> {
        self.value_to_location.get(&vid).copied()
    }

    /// Look up the block an `InstId` belongs to.
    #[must_use]
    pub fn block_of_inst(&self, iid: InstId) -> Option<(FunctionId, BlockId)> {
        self.inst_to_location.get(&iid).copied()
    }

    /// Look up the terminator info for a block.
    #[must_use]
    pub fn terminator_of(&self, block: BlockId) -> Option<&TerminatorInfo> {
        self.block_terminators.get(&block)
    }

    /// Look up condition info for a comparison result.
    #[must_use]
    pub fn condition_info(&self, vid: ValueId) -> Option<&ConditionInfo> {
        self.value_to_condition.get(&vid)
    }

    /// Look up a resolved global constant for a `ValueId`.
    #[must_use]
    pub fn global_const(&self, vid: ValueId) -> Option<i64> {
        self.value_to_global_const.get(&vid).copied()
    }

    /// Build an index from condition info only (for testing Z3 translation).
    #[cfg(test)]
    pub(crate) fn from_conditions(
        conditions: Vec<(ValueId, BinaryOp, OperandInfo, OperandInfo)>,
    ) -> Self {
        let mut value_to_condition = BTreeMap::new();
        for (vid, kind, lhs, rhs) in conditions {
            value_to_condition.insert(
                vid,
                ConditionInfo {
                    cmp_kind: kind,
                    lhs,
                    rhs,
                },
            );
        }
        Self {
            value_to_location: BTreeMap::new(),
            inst_to_location: BTreeMap::new(),
            value_to_condition,
            block_terminators: BTreeMap::new(),
            value_to_global_const: BTreeMap::new(),
        }
    }
}

/// Check if a `BinaryOp` is an integer comparison.
pub fn is_icmp(kind: BinaryOp) -> bool {
    matches!(
        kind,
        BinaryOp::ICmpEq
            | BinaryOp::ICmpNe
            | BinaryOp::ICmpUgt
            | BinaryOp::ICmpUge
            | BinaryOp::ICmpUlt
            | BinaryOp::ICmpUle
            | BinaryOp::ICmpSgt
            | BinaryOp::ICmpSge
            | BinaryOp::ICmpSlt
            | BinaryOp::ICmpSle
    )
}

/// Resolve an operand `ValueId` to an `OperandInfo`.
///
/// Checks if the operand is a constant in the module's constant table
/// and returns the appropriate `OperandInfo` variant. This enables Z3
/// to reason about concrete values instead of treating them as fresh
/// symbolic variables.
pub fn resolve_operand(vid: ValueId, module: &AirModule) -> OperandInfo {
    // Check if this ValueId corresponds to a constant
    if let Some(constant) = module.constants.get(&vid) {
        match constant {
            Constant::Int { value, .. } => return OperandInfo::IntConst(*value),
            Constant::Null => return OperandInfo::Null,
            // For other constant types (Float, BigInt, Array, etc.),
            // fall through to symbolic treatment for now
            _ => {}
        }
    }

    // Fall back to symbolic value
    OperandInfo::Value(vid)
}

// ---------------------------------------------------------------------------
// Global constant propagation
// ---------------------------------------------------------------------------

/// Resolve global constants to concrete values and patch condition operands.
///
/// Scans the module for `Operation::Global` instructions that reference constant
/// globals (where `is_constant == true` and `init` is `Constant::Int`), then
/// propagates through `Operation::Load` instructions that load from those globals.
/// Finally, any `ConditionInfo` operands that reference resolved globals are
/// replaced with `OperandInfo::IntConst`.
///
/// Returns the `ValueId` to constant map for later lookup.
fn resolve_global_constants(
    module: &AirModule,
    value_to_condition: &mut BTreeMap<ValueId, ConditionInfo>,
) -> BTreeMap<ValueId, i64> {
    // Step 1: Build ObjId -> constant value map from globals
    let mut obj_to_const: BTreeMap<ObjId, i64> = BTreeMap::new();
    for global in &module.globals {
        if global.is_constant {
            if let Some(Constant::Int { value, .. }) = &global.init {
                obj_to_const.insert(global.obj, *value);
            }
        }
    }

    // Early exit if there are no constant globals
    if obj_to_const.is_empty() {
        return BTreeMap::new();
    }

    // Step 2: Build ValueId -> ObjId map for Global instruction results
    let mut global_addr_vid: BTreeMap<ValueId, ObjId> = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Global { obj } = &inst.op {
                    if let Some(dst) = inst.dst {
                        global_addr_vid.insert(dst, *obj);
                    }
                }
            }
        }
    }

    // Step 3: Build ValueId -> constant for loads of constant globals
    let mut value_to_global_const: BTreeMap<ValueId, i64> = BTreeMap::new();

    // Direct global address references resolve to their constant value
    for (&vid, &obj) in &global_addr_vid {
        if let Some(&const_val) = obj_to_const.get(&obj) {
            value_to_global_const.insert(vid, const_val);
        }
    }

    // Load instructions whose pointer operand is a constant global address
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.op == Operation::Load {
                    if let (Some(dst), Some(&ptr_vid)) = (inst.dst, inst.operands.first()) {
                        if let Some(&obj) = global_addr_vid.get(&ptr_vid) {
                            if let Some(&const_val) = obj_to_const.get(&obj) {
                                value_to_global_const.insert(dst, const_val);
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 4: Patch condition operands to resolve global constants
    for cond in value_to_condition.values_mut() {
        if let OperandInfo::Value(vid) = &cond.lhs {
            if let Some(&const_val) = value_to_global_const.get(vid) {
                cond.lhs = OperandInfo::IntConst(const_val);
            }
        }
        if let OperandInfo::Value(vid) = &cond.rhs {
            if let Some(&const_val) = value_to_global_const.get(vid) {
                cond.rhs = OperandInfo::IntConst(const_val);
            }
        }
    }

    value_to_global_const
}

// ---------------------------------------------------------------------------
// Guard extraction (trace-based)
// ---------------------------------------------------------------------------

/// Extract guards along a checker trace (SVFG node sequence).
///
/// For each pair of Value nodes in the trace (bridging across any MemPhi nodes),
/// if they are in different blocks and the source block has a `CondBr` terminator,
/// we record which branch was taken.
///
/// This implementation bridges guards across MemPhi nodes: when the trace contains
/// `[Value(a), MemPhi(m), Value(b)]`, we still extract the guard between `a`'s block
/// and `b`'s block, even though MemPhi doesn't have a direct block location.
pub fn extract_guards(trace: &[SvfgNodeId], index: &ValueLocationIndex) -> PathCondition {
    let mut guards = Vec::new();

    // Track the last Value node's location for bridging across MemPhi nodes
    let mut last_value_loc: Option<(FunctionId, BlockId, ValueId)> = None;

    for node in trace {
        // Only Value nodes have block locations
        let SvfgNodeId::Value(vid) = node else {
            // Skip MemPhi nodes — keep last_value_loc unchanged to bridge guards
            continue;
        };

        // Look up block location for this Value node
        let Some((func_id, block_id)) = index.block_of(*vid) else {
            continue;
        };

        // If we have a previous Value node, check for guards between them
        if let Some((prev_func, prev_block, _prev_vid)) = last_value_loc {
            // Only extract guard if we cross a block boundary within the same function
            if prev_block != block_id && prev_func == func_id {
                // Check if prev_block has a CondBr terminator
                if let Some(TerminatorInfo::CondBr {
                    condition,
                    then_target,
                    else_target,
                }) = index.terminator_of(prev_block)
                {
                    let branch_taken = if block_id == *then_target {
                        true
                    } else if block_id == *else_target {
                        false
                    } else {
                        // The next block isn't a direct successor of CondBr;
                        // this can happen with interprocedural edges or CFG
                        // structures we don't handle. Skip this guard.
                        last_value_loc = Some((func_id, block_id, *vid));
                        continue;
                    };

                    guards.push(Guard {
                        block: prev_block,
                        function: prev_func,
                        condition: *condition,
                        branch_taken,
                    });
                }
            }
        }

        // Update last Value location
        last_value_loc = Some((func_id, block_id, *vid));
    }

    PathCondition { guards }
}

/// Extract guards along a block sequence (for ValueFlow or IFDS trace).
///
/// Given a sequence of `(FunctionId, BlockId)` pairs, extracts guards at
/// each block boundary where a `CondBr` terminator controls the transition.
pub fn extract_guards_from_blocks(
    blocks: &[(FunctionId, BlockId)],
    index: &ValueLocationIndex,
) -> PathCondition {
    let mut guards = Vec::new();

    for window in blocks.windows(2) {
        let (func_i, block_i) = window[0];
        let (_func_j, block_j) = window[1];

        if block_i == block_j {
            continue;
        }

        if let Some(TerminatorInfo::CondBr {
            condition,
            then_target,
            else_target,
        }) = index.terminator_of(block_i)
        {
            let branch_taken = if block_j == *then_target {
                true
            } else if block_j == *else_target {
                false
            } else {
                continue;
            };

            guards.push(Guard {
                block: block_i,
                function: func_i,
                condition: *condition,
                branch_taken,
            });
        }
    }

    PathCondition { guards }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    /// Build a module with: main() has two blocks.
    /// Block 0: icmp %cond = %x == 0; condBr %cond → block1, block2
    /// Block 1: (then branch) — has an instruction producing val_then
    /// Block 2: (else branch) — has an instruction producing val_else
    fn make_condbr_module() -> (AirModule, ValueId, ValueId, ValueId) {
        let func_id = FunctionId::new(1);
        let block0 = BlockId::new(10);
        let block1 = BlockId::new(11);
        let block2 = BlockId::new(12);

        let x_val = ValueId::new(100);
        let zero_val = ValueId::new(101);
        let cond_val = ValueId::new(102);
        let val_then = ValueId::new(103);
        let val_else = ValueId::new(104);

        // Block 0: icmp + condBr
        let icmp = Instruction::new(
            InstId::new(1000),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![x_val, zero_val])
        .with_dst(cond_val);

        let condbr = Instruction::new(
            InstId::new(1001),
            Operation::CondBr {
                then_target: block1,
                else_target: block2,
            },
        )
        .with_operands(vec![cond_val]);

        let b0 = AirBlock {
            id: block0,
            label: Some("entry".to_string()),
            instructions: vec![icmp, condbr],
        };

        // Block 1: then branch
        let then_inst = Instruction::new(InstId::new(1002), Operation::Load)
            .with_operands(vec![x_val])
            .with_dst(val_then);
        let then_ret = Instruction::new(InstId::new(1003), Operation::Ret);
        let b1 = AirBlock {
            id: block1,
            label: Some("then".to_string()),
            instructions: vec![then_inst, then_ret],
        };

        // Block 2: else branch
        let else_inst = Instruction::new(InstId::new(1004), Operation::Load)
            .with_operands(vec![x_val])
            .with_dst(val_else);
        let else_ret = Instruction::new(InstId::new(1005), Operation::Ret);
        let b2 = AirBlock {
            id: block2,
            label: Some("else".to_string()),
            instructions: vec![else_inst, else_ret],
        };

        let func = AirFunction {
            id: func_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![b0, b1, b2],
            entry_block: Some(block0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(func);

        (module, cond_val, val_then, val_else)
    }

    #[test]
    fn extract_guard_from_condbr_then_branch() {
        let (module, cond_val, val_then, _val_else) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        let trace = vec![SvfgNodeId::Value(cond_val), SvfgNodeId::Value(val_then)];

        let pc = extract_guards(&trace, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(pc.guards[0].branch_taken);
        assert_eq!(pc.guards[0].condition, cond_val);
    }

    #[test]
    fn extract_guard_from_condbr_else_branch() {
        let (module, cond_val, _val_then, val_else) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        let trace = vec![SvfgNodeId::Value(cond_val), SvfgNodeId::Value(val_else)];

        let pc = extract_guards(&trace, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(!pc.guards[0].branch_taken);
        assert_eq!(pc.guards[0].condition, cond_val);
    }

    #[test]
    fn no_guard_within_same_block() {
        let (module, _cond_val, _val_then, _val_else) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        let trace = vec![SvfgNodeId::Value(ValueId::new(102))];
        let pc = extract_guards(&trace, &index);
        assert!(pc.is_empty());
    }

    #[test]
    fn bridge_across_memphi_nodes() {
        use crate::mssa::MemAccessId;

        let (module, cond_val, val_then, _) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        // Trace with MemPhi in between should still extract the guard
        // (MemPhi bridging: we track Value nodes and extract guards between them)
        let trace = vec![
            SvfgNodeId::Value(cond_val),
            SvfgNodeId::MemPhi(MemAccessId::new(999)),
            SvfgNodeId::Value(val_then),
        ];

        let pc = extract_guards(&trace, &index);
        // Should now extract the guard (bridging across MemPhi)
        assert_eq!(pc.guards.len(), 1);
        assert!(pc.guards[0].branch_taken); // then branch
        assert_eq!(pc.guards[0].condition, cond_val);
    }

    #[test]
    fn multiple_memphi_bridge() {
        use crate::mssa::MemAccessId;

        let (module, cond_val, val_then, _) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        // Multiple MemPhi nodes should still be bridged
        let trace = vec![
            SvfgNodeId::Value(cond_val),
            SvfgNodeId::MemPhi(MemAccessId::new(998)),
            SvfgNodeId::MemPhi(MemAccessId::new(999)),
            SvfgNodeId::Value(val_then),
        ];

        let pc = extract_guards(&trace, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(pc.guards[0].branch_taken);
    }

    #[test]
    fn empty_trace_produces_empty_guards() {
        let module = AirModule::new(ModuleId::new(1));
        let index = ValueLocationIndex::build(&module);
        let pc = extract_guards(&[], &index);
        assert!(pc.is_empty());
    }

    #[test]
    fn extract_guards_from_blocks_basic() {
        let (module, cond_val, _val_then, _val_else) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        let func_id = FunctionId::new(1);
        let block0 = BlockId::new(10);
        let block1 = BlockId::new(11);
        let block2 = BlockId::new(12);

        // Block sequence: entry → then
        let blocks = vec![(func_id, block0), (func_id, block1)];
        let pc = extract_guards_from_blocks(&blocks, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(pc.guards[0].branch_taken);
        assert_eq!(pc.guards[0].condition, cond_val);

        // Block sequence: entry → else
        let blocks = vec![(func_id, block0), (func_id, block2)];
        let pc = extract_guards_from_blocks(&blocks, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(!pc.guards[0].branch_taken);
    }

    // -----------------------------------------------------------------------
    // Global constant propagation tests
    // -----------------------------------------------------------------------

    use saf_core::air::AirGlobal;

    /// Build a module with a constant global and an ICmp that compares
    /// a load of that global against another value.
    ///
    /// Layout:
    ///   @CONST_FIVE = constant i32 5
    ///   main():
    ///     %addr = global @CONST_FIVE       (dst = global_addr_vid)
    ///     %val  = load %addr               (dst = load_vid)
    ///     %cmp  = icmp eq %x, %val         (dst = cmp_vid)
    fn make_global_const_module() -> (AirModule, ValueId, ValueId, ValueId, ValueId) {
        let func_id = FunctionId::new(1);
        let block0 = BlockId::new(10);
        let obj = ObjId::new(500);

        let x_val = ValueId::new(100);
        let global_addr_vid = ValueId::new(200);
        let load_vid = ValueId::new(201);
        let cmp_vid = ValueId::new(202);

        // Instruction: %addr = global @CONST_FIVE
        let global_inst = Instruction::new(InstId::new(3000), Operation::Global { obj })
            .with_dst(global_addr_vid);

        // Instruction: %val = load %addr
        let load_inst = Instruction::new(InstId::new(3001), Operation::Load)
            .with_operands(vec![global_addr_vid])
            .with_dst(load_vid);

        // Instruction: %cmp = icmp eq %x, %val
        let icmp_inst = Instruction::new(
            InstId::new(3002),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![x_val, load_vid])
        .with_dst(cmp_vid);

        let ret_inst = Instruction::new(InstId::new(3003), Operation::Ret);

        let b0 = AirBlock {
            id: block0,
            label: Some("entry".to_string()),
            instructions: vec![global_inst, load_inst, icmp_inst, ret_inst],
        };

        let func = AirFunction {
            id: func_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![b0],
            entry_block: Some(block0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(func);

        // Add the constant global
        let mut global = AirGlobal::new(ValueId::new(999), obj, "CONST_FIVE");
        global.is_constant = true;
        global.init = Some(Constant::Int { value: 5, bits: 32 });
        module.globals.push(global);

        (module, x_val, load_vid, cmp_vid, global_addr_vid)
    }

    #[test]
    fn global_const_resolves_load_in_condition() {
        let (module, _x_val, _load_vid, cmp_vid, _global_addr_vid) = make_global_const_module();
        let index = ValueLocationIndex::build(&module);

        // The ICmp should have its RHS resolved to IntConst(5) instead of Value
        let cond = index
            .condition_info(cmp_vid)
            .expect("condition should exist");
        assert!(
            matches!(cond.rhs, OperandInfo::IntConst(5)),
            "expected IntConst(5), got {:?}",
            cond.rhs,
        );
        // LHS should remain a symbolic value (it's just %x, not a global)
        assert!(
            matches!(cond.lhs, OperandInfo::Value(_)),
            "expected Value(_), got {:?}",
            cond.lhs,
        );
    }

    #[test]
    fn global_const_accessor_works() {
        let (module, _x_val, load_vid, _cmp_vid, global_addr_vid) = make_global_const_module();
        let index = ValueLocationIndex::build(&module);

        // Both the global address and the load result should resolve
        assert_eq!(index.global_const(global_addr_vid), Some(5));
        assert_eq!(index.global_const(load_vid), Some(5));

        // An unrelated ValueId should return None
        assert_eq!(index.global_const(ValueId::new(12345)), None);
    }

    #[test]
    fn non_constant_global_not_resolved() {
        let func_id = FunctionId::new(1);
        let block0 = BlockId::new(10);
        let obj = ObjId::new(600);

        let x_val = ValueId::new(100);
        let global_addr_vid = ValueId::new(300);
        let load_vid = ValueId::new(301);
        let cmp_vid = ValueId::new(302);

        let global_inst = Instruction::new(InstId::new(4000), Operation::Global { obj })
            .with_dst(global_addr_vid);

        let load_inst = Instruction::new(InstId::new(4001), Operation::Load)
            .with_operands(vec![global_addr_vid])
            .with_dst(load_vid);

        let icmp_inst = Instruction::new(
            InstId::new(4002),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![x_val, load_vid])
        .with_dst(cmp_vid);

        let ret_inst = Instruction::new(InstId::new(4003), Operation::Ret);

        let b0 = AirBlock {
            id: block0,
            label: Some("entry".to_string()),
            instructions: vec![global_inst, load_inst, icmp_inst, ret_inst],
        };

        let func = AirFunction {
            id: func_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![b0],
            entry_block: Some(block0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(func);

        // Global is NOT constant (is_constant = false)
        let mut global = AirGlobal::new(ValueId::new(998), obj, "MUTABLE_VAR");
        global.is_constant = false;
        global.init = Some(Constant::Int {
            value: 42,
            bits: 32,
        });
        module.globals.push(global);

        let index = ValueLocationIndex::build(&module);

        // RHS should remain a symbolic Value — not resolved
        let cond = index
            .condition_info(cmp_vid)
            .expect("condition should exist");
        assert!(
            matches!(cond.rhs, OperandInfo::Value(_)),
            "mutable global should not be resolved, got {:?}",
            cond.rhs,
        );
        assert_eq!(index.global_const(load_vid), None);
    }

    #[test]
    fn global_const_no_globals_is_noop() {
        // Module with no globals — should not panic or change conditions
        let (module, _cond_val, _val_then, _val_else) = make_condbr_module();
        let index = ValueLocationIndex::build(&module);

        // The existing condition should be unchanged (both operands are Values)
        let cond = index
            .condition_info(ValueId::new(102))
            .expect("condition should exist");
        assert!(matches!(cond.lhs, OperandInfo::Value(_)));
        assert!(matches!(cond.rhs, OperandInfo::Value(_)));
    }

    #[test]
    fn global_const_both_operands_resolved() {
        // Test where both LHS and RHS of an ICmp are loads of constant globals
        let func_id = FunctionId::new(1);
        let block0 = BlockId::new(10);
        let obj_a = ObjId::new(700);
        let obj_b = ObjId::new(701);

        let addr_a = ValueId::new(400);
        let load_a = ValueId::new(401);
        let addr_b = ValueId::new(402);
        let load_b = ValueId::new(403);
        let cmp_vid = ValueId::new(404);

        let global_a =
            Instruction::new(InstId::new(5000), Operation::Global { obj: obj_a }).with_dst(addr_a);

        let load_a_inst = Instruction::new(InstId::new(5001), Operation::Load)
            .with_operands(vec![addr_a])
            .with_dst(load_a);

        let global_b =
            Instruction::new(InstId::new(5002), Operation::Global { obj: obj_b }).with_dst(addr_b);

        let load_b_inst = Instruction::new(InstId::new(5003), Operation::Load)
            .with_operands(vec![addr_b])
            .with_dst(load_b);

        let icmp_inst = Instruction::new(
            InstId::new(5004),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![load_a, load_b])
        .with_dst(cmp_vid);

        let ret_inst = Instruction::new(InstId::new(5005), Operation::Ret);

        let b0 = AirBlock {
            id: block0,
            label: Some("entry".to_string()),
            instructions: vec![
                global_a,
                load_a_inst,
                global_b,
                load_b_inst,
                icmp_inst,
                ret_inst,
            ],
        };

        let func = AirFunction {
            id: func_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![b0],
            entry_block: Some(block0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(func);

        let mut ga = AirGlobal::new(ValueId::new(990), obj_a, "CONST_A");
        ga.is_constant = true;
        ga.init = Some(Constant::Int {
            value: 10,
            bits: 32,
        });
        module.globals.push(ga);

        let mut gb = AirGlobal::new(ValueId::new(991), obj_b, "CONST_B");
        gb.is_constant = true;
        gb.init = Some(Constant::Int {
            value: 20,
            bits: 32,
        });
        module.globals.push(gb);

        let index = ValueLocationIndex::build(&module);

        let cond = index
            .condition_info(cmp_vid)
            .expect("condition should exist");
        assert!(
            matches!(cond.lhs, OperandInfo::IntConst(10)),
            "expected IntConst(10), got {:?}",
            cond.lhs,
        );
        assert!(
            matches!(cond.rhs, OperandInfo::IntConst(20)),
            "expected IntConst(20), got {:?}",
            cond.rhs,
        );
    }
}
