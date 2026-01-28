//! Visitor pattern for traversing AIR module structures.
//!
//! Provides a trait-based visitor with default no-op implementations,
//! plus convenience `walk_module` and `walk_function` drivers.

use crate::air::{AirBlock, AirFunction, AirModule, Instruction};

/// Visitor trait for traversing AIR module structures.
///
/// Default implementations are no-ops, so visitors only need to
/// implement the methods they care about. Return `false` from
/// `visit_function` or `visit_block` to skip visiting children.
pub trait AirVisitor {
    /// Called for each function. Return `false` to skip blocks/instructions.
    fn visit_function(&mut self, func: &AirFunction) -> bool {
        let _ = func;
        true
    }

    /// Called for each block. Return `false` to skip instructions.
    fn visit_block(&mut self, func: &AirFunction, block: &AirBlock) -> bool {
        let _ = (func, block);
        true
    }

    /// Called for each instruction.
    fn visit_instruction(&mut self, func: &AirFunction, block: &AirBlock, inst: &Instruction) {
        let _ = (func, block, inst);
    }
}

/// Walk an entire module, dispatching to the visitor.
pub fn walk_module(module: &AirModule, visitor: &mut impl AirVisitor) {
    for func in &module.functions {
        if !visitor.visit_function(func) {
            continue;
        }
        for block in &func.blocks {
            if !visitor.visit_block(func, block) {
                continue;
            }
            for inst in &block.instructions {
                visitor.visit_instruction(func, block, inst);
            }
        }
    }
}

/// Walk a single function.
pub fn walk_function(func: &AirFunction, visitor: &mut impl AirVisitor) {
    if !visitor.visit_function(func) {
        return;
    }
    for block in &func.blocks {
        if !visitor.visit_block(func, block) {
            continue;
        }
        for inst in &block.instructions {
            visitor.visit_instruction(func, block, inst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
    use crate::ids::{BlockId, FunctionId, InstId, ModuleId};
    use std::collections::BTreeMap;

    /// Helper to create a minimal test module.
    fn test_module() -> AirModule {
        let inst1 = Instruction::new(InstId::new(1), Operation::Ret);
        let inst2 = Instruction::new(InstId::new(2), Operation::Ret);
        let block = AirBlock {
            id: BlockId::new(1),
            label: None,
            instructions: vec![inst1, inst2],
        };
        let func = AirFunction {
            id: FunctionId::new(1),
            name: "test_func".to_string(),
            blocks: vec![block],
            params: vec![],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        AirModule {
            id: ModuleId::new(1),
            name: Some("test_mod".to_string()),
            functions: vec![func],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: BTreeMap::new(),
            types: BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    struct InstructionCounter {
        count: usize,
    }

    impl AirVisitor for InstructionCounter {
        fn visit_instruction(&mut self, _: &AirFunction, _: &AirBlock, _: &Instruction) {
            self.count += 1;
        }
    }

    #[test]
    fn walk_module_counts_all_instructions() {
        let module = test_module();
        let mut counter = InstructionCounter { count: 0 };
        walk_module(&module, &mut counter);
        assert_eq!(counter.count, 2);
    }

    struct FunctionSkipper;

    impl AirVisitor for FunctionSkipper {
        fn visit_function(&mut self, _: &AirFunction) -> bool {
            false // Skip all functions
        }

        fn visit_instruction(&mut self, _: &AirFunction, _: &AirBlock, _: &Instruction) {
            panic!("Should not visit instructions when function is skipped");
        }
    }

    #[test]
    fn walk_module_skips_when_visit_function_returns_false() {
        let module = test_module();
        let mut skipper = FunctionSkipper;
        walk_module(&module, &mut skipper);
        // No panic means instructions were skipped correctly
    }

    struct BlockSkipper;

    impl AirVisitor for BlockSkipper {
        fn visit_block(&mut self, _: &AirFunction, _: &AirBlock) -> bool {
            false // Skip all blocks
        }

        fn visit_instruction(&mut self, _: &AirFunction, _: &AirBlock, _: &Instruction) {
            panic!("Should not visit instructions when block is skipped");
        }
    }

    #[test]
    fn walk_module_skips_when_visit_block_returns_false() {
        let module = test_module();
        let mut skipper = BlockSkipper;
        walk_module(&module, &mut skipper);
    }

    #[test]
    fn walk_function_counts_instructions() {
        let module = test_module();
        let mut counter = InstructionCounter { count: 0 };
        walk_function(&module.functions[0], &mut counter);
        assert_eq!(counter.count, 2);
    }
}
