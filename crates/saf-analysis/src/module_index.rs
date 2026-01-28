//! Pre-computed index maps over an `AirModule`.
//!
//! Several solvers (DDA, IFDS, IDE, flow-sensitive PTA) independently build
//! nearly identical index maps from an `AirModule` — `inst_to_func`,
//! `inst_to_block`, `value_to_inst`, etc. `ModuleIndex` computes these once
//! and is passed by shared reference, avoiding redundant O(n) walks.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};

/// Pre-computed lookup tables derived from an `AirModule`.
///
/// Build once via [`ModuleIndex::build`], then share across solvers.
#[derive(Debug, Clone)]
pub struct ModuleIndex {
    /// Map from instruction ID to the function it belongs to.
    pub inst_to_func: BTreeMap<InstId, FunctionId>,
    /// Map from instruction ID to its containing block.
    pub inst_to_block: BTreeMap<InstId, BlockId>,
    /// Map from block ID to its containing function.
    pub block_to_func: BTreeMap<BlockId, FunctionId>,
    /// Map from `ValueId` (dst of an instruction) to the instruction that produces it.
    pub value_to_inst: BTreeMap<ValueId, InstId>,
    /// Map from parameter `ValueId` to the function it belongs to.
    ///
    /// Parameters don't have a defining instruction, so they are tracked
    /// separately from `value_to_inst`.
    pub param_to_func: BTreeMap<ValueId, FunctionId>,
    /// Ordered instruction IDs within each block.
    pub block_instructions: BTreeMap<BlockId, Vec<InstId>>,
    /// Next instruction in the same block (None if last).
    pub next_inst_in_block: BTreeMap<InstId, Option<InstId>>,
    /// First instruction of each function's entry block.
    pub func_entry_inst: BTreeMap<FunctionId, InstId>,
    /// Terminator instructions of exit blocks for each function.
    pub func_exit_insts: BTreeMap<FunctionId, BTreeSet<InstId>>,
    /// Map from `FunctionId` to the `ValueId`s returned by its `Ret` instructions.
    pub return_values: BTreeMap<FunctionId, Vec<ValueId>>,
}

impl ModuleIndex {
    /// Build all index maps from an `AirModule`.
    ///
    /// Iterates the module once (with a small second pass for exit detection
    /// via CFG exits). Skips declaration-only functions.
    #[must_use]
    pub fn build(module: &AirModule) -> Self {
        let mut inst_to_func = BTreeMap::new();
        let mut inst_to_block = BTreeMap::new();
        let mut block_to_func = BTreeMap::new();
        let mut value_to_inst = BTreeMap::new();
        let mut param_to_func = BTreeMap::new();
        let mut block_instructions = BTreeMap::new();
        let mut next_inst_in_block = BTreeMap::new();
        let mut func_entry_inst = BTreeMap::new();
        let mut func_exit_insts: BTreeMap<FunctionId, BTreeSet<InstId>> = BTreeMap::new();
        let mut return_values: BTreeMap<FunctionId, Vec<ValueId>> = BTreeMap::new();

        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            // Index function parameters.
            for param in &func.params {
                param_to_func.insert(param.id, func.id);
            }

            // Entry block: use explicit entry_block or fall back to first block.
            let entry_block_id = func
                .entry_block
                .or_else(|| func.blocks.first().map(|b| b.id));

            if let Some(eb_id) = entry_block_id {
                if let Some(eb) = func.blocks.iter().find(|b| b.id == eb_id) {
                    if let Some(first_inst) = eb.instructions.first() {
                        func_entry_inst.insert(func.id, first_inst.id);
                    }
                }
            }

            let mut rets = Vec::new();

            for block in &func.blocks {
                block_to_func.insert(block.id, func.id);
                let inst_ids: Vec<InstId> = block.instructions.iter().map(|i| i.id).collect();
                block_instructions.insert(block.id, inst_ids);

                for (i, inst) in block.instructions.iter().enumerate() {
                    inst_to_func.insert(inst.id, func.id);
                    inst_to_block.insert(inst.id, block.id);

                    if let Some(dst) = inst.dst {
                        value_to_inst.insert(dst, inst.id);
                    }

                    if i + 1 < block.instructions.len() {
                        next_inst_in_block.insert(inst.id, Some(block.instructions[i + 1].id));
                    } else {
                        next_inst_in_block.insert(inst.id, None);
                    }

                    // Collect return values.
                    if matches!(inst.op, Operation::Ret) {
                        if let Some(&ret_val) = inst.operands.first() {
                            rets.push(ret_val);
                        }
                    }

                    // Exit blocks: terminators that are Ret or Unreachable.
                    if inst.is_terminator()
                        && matches!(inst.op, Operation::Ret | Operation::Unreachable)
                    {
                        func_exit_insts.entry(func.id).or_default().insert(inst.id);
                    }
                }
            }

            return_values.insert(func.id, rets);
        }

        Self {
            inst_to_func,
            inst_to_block,
            block_to_func,
            value_to_inst,
            param_to_func,
            block_instructions,
            next_inst_in_block,
            func_entry_inst,
            func_exit_insts,
            return_values,
        }
    }

    /// Look up which function contains a given instruction.
    #[must_use]
    pub fn function_of_inst(&self, inst: InstId) -> Option<FunctionId> {
        self.inst_to_func.get(&inst).copied()
    }

    /// Look up which block contains a given instruction.
    #[must_use]
    pub fn block_of_inst(&self, inst: InstId) -> Option<BlockId> {
        self.inst_to_block.get(&inst).copied()
    }

    /// Look up the instruction that defines a given value.
    #[must_use]
    pub fn defining_inst(&self, value: ValueId) -> Option<InstId> {
        self.value_to_inst.get(&value).copied()
    }

    /// Look up the function that owns a given parameter `ValueId`.
    #[must_use]
    pub fn function_of_param(&self, param_value: ValueId) -> Option<FunctionId> {
        self.param_to_func.get(&param_value).copied()
    }

    /// Check whether a `ValueId` is a function parameter.
    #[must_use]
    pub fn is_param(&self, value: ValueId) -> bool {
        self.param_to_func.contains_key(&value)
    }

    /// Get the return values for a function.
    #[must_use]
    pub fn return_values_of(&self, func: FunctionId) -> &[ValueId] {
        self.return_values.get(&func).map_or(&[], Vec::as_slice)
    }
}

/// Build a map from `FunctionId` to the `ValueId`s returned by `Ret` instructions.
///
/// This is a standalone version of the same logic used by [`ModuleIndex::build`].
/// Use this when you need *only* the return-value map without building the full
/// index (e.g., inside the PTA constraint pipeline).
#[must_use]
pub fn collect_return_values(module: &AirModule) -> BTreeMap<FunctionId, Vec<ValueId>> {
    module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| {
            let rets: Vec<ValueId> = f
                .blocks
                .iter()
                .flat_map(|b| b.instructions.iter())
                .filter(|i| matches!(i.op, Operation::Ret))
                .filter_map(|i| i.operands.first().copied())
                .collect();
            (f.id, rets)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction};
    use saf_core::ids::ModuleId;

    fn make_test_module() -> AirModule {
        let alloca = Instruction::new(InstId::new(1), Operation::Alloca { size_bytes: None })
            .with_dst(ValueId::new(100));
        let store = Instruction::new(InstId::new(2), Operation::Store)
            .with_operands(vec![ValueId::new(200), ValueId::new(100)]);
        let ret =
            Instruction::new(InstId::new(3), Operation::Ret).with_operands(vec![ValueId::new(100)]);

        let block = AirBlock {
            id: BlockId::new(10),
            label: None,
            instructions: vec![alloca, store, ret],
        };

        let func = AirFunction {
            id: FunctionId::new(1000),
            name: "test_func".to_string(),
            params: vec![AirParam::new(ValueId::new(50), 0)],
            blocks: vec![block],
            is_declaration: false,
            entry_block: Some(BlockId::new(10)),
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let decl = AirFunction {
            id: FunctionId::new(2000),
            name: "extern_func".to_string(),
            params: vec![],
            blocks: vec![],
            is_declaration: true,
            entry_block: None,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        AirModule {
            id: ModuleId::new(1),
            name: Some("test".to_string()),
            functions: vec![func, decl],
            globals: vec![],
            type_hierarchy: vec![],
            source_files: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    #[test]
    fn build_populates_inst_to_func() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        assert_eq!(
            index.function_of_inst(InstId::new(1)),
            Some(FunctionId::new(1000))
        );
        assert_eq!(
            index.function_of_inst(InstId::new(2)),
            Some(FunctionId::new(1000))
        );
        assert_eq!(
            index.function_of_inst(InstId::new(3)),
            Some(FunctionId::new(1000))
        );
        // Non-existent instruction
        assert_eq!(index.function_of_inst(InstId::new(999)), None);
    }

    #[test]
    fn build_populates_inst_to_block() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        assert_eq!(index.block_of_inst(InstId::new(1)), Some(BlockId::new(10)));
    }

    #[test]
    fn build_populates_value_to_inst() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        assert_eq!(index.defining_inst(ValueId::new(100)), Some(InstId::new(1)));
        // ValueId 200 is an operand, not a dst — should not appear
        assert_eq!(index.defining_inst(ValueId::new(200)), None);
    }

    #[test]
    fn build_populates_param_to_func() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        // Parameter ValueId(50) belongs to function 1000
        assert_eq!(
            index.function_of_param(ValueId::new(50)),
            Some(FunctionId::new(1000))
        );
        assert!(index.is_param(ValueId::new(50)));
        // Instruction dst is not a parameter
        assert!(!index.is_param(ValueId::new(100)));
    }

    #[test]
    fn build_populates_next_inst() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        assert_eq!(
            index.next_inst_in_block.get(&InstId::new(1)),
            Some(&Some(InstId::new(2)))
        );
        assert_eq!(
            index.next_inst_in_block.get(&InstId::new(2)),
            Some(&Some(InstId::new(3)))
        );
        // Last instruction in block has None
        assert_eq!(index.next_inst_in_block.get(&InstId::new(3)), Some(&None));
    }

    #[test]
    fn build_populates_func_entry_inst() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        assert_eq!(
            index.func_entry_inst.get(&FunctionId::new(1000)),
            Some(&InstId::new(1))
        );
        // Declaration should not have entry
        assert_eq!(index.func_entry_inst.get(&FunctionId::new(2000)), None);
    }

    #[test]
    fn build_populates_func_exit_insts() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        let exits = index.func_exit_insts.get(&FunctionId::new(1000));
        assert!(exits.is_some());
        assert!(exits.unwrap().contains(&InstId::new(3)));
    }

    #[test]
    fn build_populates_return_values() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        let rets = index.return_values_of(FunctionId::new(1000));
        assert_eq!(rets, &[ValueId::new(100)]);
        // Declaration: empty
        assert!(index.return_values_of(FunctionId::new(2000)).is_empty());
    }

    #[test]
    fn build_skips_declarations() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        // Declaration function should not appear in any maps
        assert!(
            !index
                .inst_to_func
                .values()
                .any(|&f| f == FunctionId::new(2000))
        );
        assert!(
            index.block_instructions.is_empty()
                || !index
                    .block_to_func
                    .values()
                    .any(|&f| f == FunctionId::new(2000))
        );
    }

    #[test]
    fn build_block_instructions_ordered() {
        let module = make_test_module();
        let index = ModuleIndex::build(&module);

        let insts = index.block_instructions.get(&BlockId::new(10)).unwrap();
        assert_eq!(insts, &[InstId::new(1), InstId::new(2), InstId::new(3)]);
    }
}
