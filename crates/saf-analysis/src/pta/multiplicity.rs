//! Allocation multiplicity classification.
//!
//! Determines whether each abstract allocation site represents a unique
//! concrete object or a summary of multiple objects. Used by alias
//! analysis to decide if singleton points-to sets imply must-alias.
//!
//! Classification rules (conservative -- `Summary` is the safe default):
//! - **Globals** -> `Unique` (one instance per program)
//! - **Stack allocas** -> `Unique` if NOT in a loop-bearing function
//! - **Heap allocs** -> `Unique` if NOT in a loop-bearing function AND
//!   the enclosing function is called from at most one call site
//! - Everything else -> `Summary`

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, ObjId};

use super::location::{AllocationMultiplicity, LocationFactory, MemoryRegion};
use crate::absint::detect_loop_headers;
use crate::cfg::Cfg;

/// Classify allocation multiplicity for all objects in the location factory.
///
/// Call this after constraint extraction and before building `PtaResult`.
/// Objects not explicitly classified retain their default (`Summary`).
pub fn classify_multiplicity(module: &AirModule, factory: &mut LocationFactory) {
    // Phase 1: Count how many direct call sites target each function
    let call_site_counts = count_direct_call_sites(module);

    // Phase 2: Determine which functions contain loops
    let functions_with_loops = find_functions_with_loops(module);

    // Phase 3: Classify each allocation object
    for func in &module.functions {
        let func_has_loop = functions_with_loops.contains(&func.id);

        for block in &func.blocks {
            for inst in &block.instructions {
                let obj = ObjId::new(inst.id.raw());
                let region = factory.region_by_obj(obj);

                match region {
                    MemoryRegion::Global => {
                        // Globals are always unique -- one instance in the program
                        factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
                    }
                    MemoryRegion::Stack => {
                        // Stack alloca is unique if the function has no loops.
                        // Conservative: if function has ANY loop, all its allocas
                        // are treated as Summary (some may not be in the loop,
                        // but this is a sound over-approximation).
                        if !func_has_loop {
                            factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
                        }
                    }
                    MemoryRegion::Heap => {
                        // Heap alloc is unique if:
                        // (a) function has no loops, AND
                        // (b) function is called from at most 1 direct call site
                        let multi_caller = call_site_counts
                            .get(&func.id)
                            .is_some_and(|&count| count > 1);
                        if !func_has_loop && !multi_caller {
                            factory.set_multiplicity(obj, AllocationMultiplicity::Unique);
                        }
                    }
                    MemoryRegion::Unknown => {
                        // Unknown region stays Summary (safe default)
                    }
                }
            }
        }
    }
}

/// Count the number of direct call sites targeting each function.
///
/// Walks all instructions in the module and counts `CallDirect` operations
/// per callee. Indirect calls are ignored (they conservatively leave
/// their targets as `Summary` since we can't statically determine the callee).
fn count_direct_call_sites(module: &AirModule) -> BTreeMap<FunctionId, usize> {
    let mut counts: BTreeMap<FunctionId, usize> = BTreeMap::new();
    for func in &module.functions {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    *counts.entry(*callee).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

/// Find all functions that contain at least one loop.
///
/// Builds the CFG for each function and runs back-edge detection.
/// A function "has a loop" if `detect_loop_headers` returns a non-empty set.
fn find_functions_with_loops(module: &AirModule) -> BTreeSet<FunctionId> {
    let mut result = BTreeSet::new();
    for func in &module.functions {
        if func.blocks.is_empty() {
            continue;
        }
        let cfg = Cfg::build(func);
        let headers = detect_loop_headers(&cfg);
        if !headers.is_empty() {
            result.insert(func.id);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirModule, HeapAllocKind, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    use crate::pta::config::FieldSensitivity;
    use crate::pta::location::FieldPath;

    fn make_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::None)
    }

    /// Helper: create a minimal module with one function containing one heap alloc.
    fn make_single_alloc_module() -> (AirModule, FunctionId, InstId) {
        let mut module = make_module();
        let func_id = FunctionId::derive(b"test_func");
        let block_id = BlockId::derive(b"entry");
        let inst_id = InstId::derive(b"heap_alloc");
        let dst = ValueId::derive(b"alloc_result");

        let mut block = AirBlock::new(block_id);
        block.instructions.push(
            Instruction::new(
                inst_id,
                Operation::HeapAlloc {
                    kind: HeapAllocKind::Malloc,
                },
            )
            .with_dst(dst),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));

        let mut func = AirFunction::new(func_id, "test_func");
        func.blocks.push(block);
        module.functions.push(func);

        (module, func_id, inst_id)
    }

    #[test]
    fn heap_alloc_single_caller_no_loop_is_unique() {
        let (module, _func_id, inst_id) = make_single_alloc_module();
        let mut factory = make_factory();

        // Simulate what constraint extraction does
        let obj = ObjId::new(inst_id.raw());
        let _loc = factory.get_or_create(obj, FieldPath::empty());
        factory.set_region(obj, MemoryRegion::Heap);

        classify_multiplicity(&module, &mut factory);

        assert_eq!(
            factory.multiplicity_by_obj(obj),
            AllocationMultiplicity::Unique,
            "Heap alloc in a single-caller, loop-free function should be Unique"
        );
    }

    #[test]
    fn heap_alloc_multi_caller_is_summary() {
        // Create a module where "wrapper" is called from two sites in "main"
        let mut module = make_module();

        let wrapper_id = FunctionId::derive(b"wrapper");
        let main_id = FunctionId::derive(b"main");
        let alloc_inst_id = InstId::derive(b"wrapper_alloc");

        // wrapper function with a heap alloc
        let mut alloc_block = AirBlock::new(BlockId::derive(b"wrapper_entry"));
        alloc_block.instructions.push(
            Instruction::new(
                alloc_inst_id,
                Operation::HeapAlloc {
                    kind: HeapAllocKind::Malloc,
                },
            )
            .with_dst(ValueId::derive(b"wrapper_alloc_dst")),
        );
        alloc_block.instructions.push(Instruction::new(
            InstId::derive(b"wrapper_ret"),
            Operation::Ret,
        ));

        let mut wrapper_func = AirFunction::new(wrapper_id, "wrapper");
        wrapper_func.blocks.push(alloc_block);
        module.functions.push(wrapper_func);

        // main function with two call sites targeting wrapper
        let mut main_block = AirBlock::new(BlockId::derive(b"main_entry"));
        main_block.instructions.push(
            Instruction::new(
                InstId::derive(b"call1"),
                Operation::CallDirect { callee: wrapper_id },
            )
            .with_dst(ValueId::derive(b"call1_dst")),
        );
        main_block.instructions.push(
            Instruction::new(
                InstId::derive(b"call2"),
                Operation::CallDirect { callee: wrapper_id },
            )
            .with_dst(ValueId::derive(b"call2_dst")),
        );
        main_block.instructions.push(Instruction::new(
            InstId::derive(b"main_ret"),
            Operation::Ret,
        ));

        let mut main_func = AirFunction::new(main_id, "main");
        main_func.blocks.push(main_block);
        module.functions.push(main_func);

        let mut factory = make_factory();
        let obj = ObjId::new(alloc_inst_id.raw());
        let _loc = factory.get_or_create(obj, FieldPath::empty());
        factory.set_region(obj, MemoryRegion::Heap);

        classify_multiplicity(&module, &mut factory);

        assert_eq!(
            factory.multiplicity_by_obj(obj),
            AllocationMultiplicity::Summary,
            "Heap alloc in a function called from multiple sites should be Summary"
        );
    }

    #[test]
    fn global_is_always_unique() {
        let (module, _func_id, inst_id) = make_single_alloc_module();
        let mut factory = make_factory();

        let obj = ObjId::new(inst_id.raw());
        let _loc = factory.get_or_create(obj, FieldPath::empty());
        factory.set_region(obj, MemoryRegion::Global);

        classify_multiplicity(&module, &mut factory);

        assert_eq!(
            factory.multiplicity_by_obj(obj),
            AllocationMultiplicity::Unique,
            "Global objects should always be Unique"
        );
    }

    #[test]
    fn unclassified_object_stays_summary() {
        let (module, _func_id, _inst_id) = make_single_alloc_module();
        let mut factory = make_factory();

        // Don't register any objects in the factory -- just run classification
        classify_multiplicity(&module, &mut factory);

        // A random ObjId not in the factory should default to Summary
        assert_eq!(
            factory.multiplicity_by_obj(ObjId::derive(b"nonexistent")),
            AllocationMultiplicity::Summary,
        );
    }
}
