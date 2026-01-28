//! Spec-based constraint extraction for external functions.
//!
//! Generates PTA constraints for external/library functions using function specifications.
//! This enables precise modeling of functions like malloc, strcpy, and other libc functions
//! without requiring their source code.

use std::collections::BTreeMap;

use saf_core::air::{AirModule, Instruction, Operation};
use saf_core::id::make_id;
use saf_core::ids::{FunctionId, ObjId, ValueId};
use saf_core::spec::{Pointer, Role, SpecRegistry};

use super::constraint::{AddrConstraint, ConstraintSet, CopyConstraint, StoreConstraint};
use super::location::{FieldPath, LocationFactory};

/// Extract constraints for external function calls using specs.
///
/// This is called after normal constraint extraction to handle external functions
/// that have no body. For each `CallDirect` to an external function, looks up the
/// spec and generates appropriate constraints.
///
/// # Constraints Generated
///
/// - `returns.pointer: fresh_heap` → `AddrConstraint` (models heap allocation)
/// - `returns.aliases: param.N` → `CopyConstraint` from param N to return
/// - `params[i].modifies: true` → `StoreConstraint` (models write through pointer)
pub fn extract_spec_constraints(
    module: &AirModule,
    specs: &SpecRegistry,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    // Build function name map for lookup
    let func_name_map: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    // Build set of functions that have bodies (not declarations)
    let defined_funcs: std::collections::BTreeSet<FunctionId> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| f.id)
        .collect();

    // Walk all call sites
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    // Skip if callee has a body (handled by normal interprocedural extraction)
                    if defined_funcs.contains(callee) {
                        continue;
                    }

                    // Look up callee name and spec
                    if let Some(callee_name) = func_name_map.get(callee) {
                        if let Some(spec) = specs.lookup(callee_name) {
                            extract_call_spec_constraints(inst, spec, factory, constraints);
                        }
                    }
                }
            }
        }
    }
}

/// Extract constraints for a single call based on its spec.
#[allow(clippy::too_many_lines)]
fn extract_call_spec_constraints(
    inst: &Instruction,
    spec: &saf_core::spec::FunctionSpec,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    let call_result = inst.dst;

    // Handle return value
    if let Some(ret_spec) = &spec.returns {
        if let Some(dst) = call_result {
            // Fresh heap allocation
            if ret_spec.pointer == Some(Pointer::FreshHeap) {
                // Use instruction ID as allocation site
                let obj = ObjId::new(inst.id.raw());
                let loc = factory.get_or_create(obj, FieldPath::empty());
                constraints.addr.insert(AddrConstraint { ptr: dst, loc });
            }

            // Static singleton — all calls to the same function share one location
            if ret_spec.pointer == Some(Pointer::StaticSingleton) {
                // Use callee function name hash as the allocation site so all
                // call sites to the same function produce the same abstract object.
                let callee_name = &spec.name;
                let name_hash = saf_core::id::make_id("static_singleton", callee_name.as_bytes());
                let obj = ObjId::new(name_hash);
                let loc = factory.get_or_create(obj, FieldPath::empty());
                constraints.addr.insert(AddrConstraint { ptr: dst, loc });
            }

            // Return aliases a parameter
            if let Some(param_idx) = ret_spec.alias_param_index() {
                if let Some(&arg) = inst.operands.get(param_idx as usize) {
                    constraints.copy.insert(CopyConstraint { dst, src: arg });
                }
            }
        }
    }

    // Handle allocator role (generates fresh heap even without explicit returns.pointer)
    if spec.role == Some(Role::Allocator) || spec.role == Some(Role::Reallocator) {
        if let Some(dst) = call_result {
            // Check if we already added an Addr constraint from returns.pointer
            let has_addr = constraints.addr.iter().any(|a| a.ptr == dst);
            if !has_addr {
                let obj = ObjId::new(inst.id.raw());
                let loc = factory.get_or_create(obj, FieldPath::empty());
                constraints.addr.insert(AddrConstraint { ptr: dst, loc });
            }
        }
    }

    // Handle parameter modifications (modifies: true without taint coverage).
    // When a param is modified but no taint propagation targets it,
    // conservatively model: the callee writes an unknown pointer through the param.
    // This covers cases like `posix_memalign(*ptr = fresh)` and `strtol(*endptr = ptr)`.
    for param_spec in &spec.params {
        if param_spec.does_modify() {
            if let Some(&arg_ptr) = inst.operands.get(param_spec.index as usize) {
                // Check if taint propagation already covers this param as a destination.
                // If so, the taint handler already generated a Store — skip to avoid duplicates.
                let covered_by_taint = spec.taint.as_ref().is_some_and(|t| {
                    t.propagates.iter().any(|p| {
                        p.to.iter().any(|loc| {
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
                    (u128::from(param_spec.index)).to_le_bytes(),
                ]
                .concat();
                let synthetic_val = ValueId::new(make_id("stub_modifies_val", &seed));
                let synthetic_obj = ObjId::new(make_id("stub_modifies_obj", &seed));
                let loc = factory.get_or_create(synthetic_obj, FieldPath::empty());

                // Addr: `synthetic_val` points to fresh object
                constraints.addr.insert(AddrConstraint {
                    ptr: synthetic_val,
                    loc,
                });
                // Store: `*arg_ptr = synthetic_val`
                constraints.store.insert(StoreConstraint {
                    dst_ptr: arg_ptr,
                    src: synthetic_val,
                });
            }
        }
    }

    // Handle taint propagation as copy constraints
    // When taint propagates from param.i to param.j, it implies data flow
    if let Some(taint_spec) = &spec.taint {
        for prop in &taint_spec.propagates {
            // Get source value
            let src = match prop.from {
                saf_core::spec::TaintLocation::Return => call_result,
                saf_core::spec::TaintLocation::Param(i) => inst.operands.get(i as usize).copied(),
                saf_core::spec::TaintLocation::Unknown => None,
            };

            if let Some(src_val) = src {
                // Apply to each destination
                for to_loc in &prop.to {
                    match to_loc {
                        saf_core::spec::TaintLocation::Return => {
                            if let Some(dst) = call_result {
                                // Return value receives taint from source
                                // This is a data flow, model as copy for PTA
                                if src_val != dst {
                                    constraints
                                        .copy
                                        .insert(CopyConstraint { dst, src: src_val });
                                }
                            }
                        }
                        saf_core::spec::TaintLocation::Param(idx) => {
                            // Destination parameter receives data from source.
                            // If the param is modified (modifies: true), we should
                            // store the source value through the param pointer.
                            if let Some(&dst_ptr) = inst.operands.get(*idx as usize) {
                                // Check if this param is marked as modifies
                                let modifies = spec
                                    .param(*idx)
                                    .is_some_and(saf_core::spec::ParamSpec::does_modify);
                                if modifies {
                                    // Model *dst_ptr = src
                                    constraints.store.insert(StoreConstraint {
                                        dst_ptr,
                                        src: src_val,
                                    });
                                }
                            }
                        }
                        saf_core::spec::TaintLocation::Unknown => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction};
    use saf_core::ids::{BlockId, InstId, ModuleId, ValueId};
    use saf_core::spec::{
        FunctionSpec, ParamSpec, ReturnSpec, TaintLocation, TaintPropagation, TaintSpec,
    };

    use crate::pta::config::FieldSensitivity;

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    fn make_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    #[test]
    fn test_allocator_spec_generates_addr_constraint() {
        let mut module = make_module();

        // Add malloc declaration
        let malloc_id = FunctionId::derive(b"malloc");
        let mut malloc_func = AirFunction::new(malloc_id, "malloc");
        malloc_func.is_declaration = true;
        module.functions.push(malloc_func);

        // Add caller that calls malloc
        let caller_id = FunctionId::derive(b"caller");
        let mut caller = AirFunction::new(caller_id, "caller");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        let call_inst_id = InstId::derive(b"call_malloc");
        let result_id = ValueId::derive(b"malloc_result");
        let size_arg = ValueId::derive(b"size");

        block.instructions.push(
            Instruction::new(call_inst_id, Operation::CallDirect { callee: malloc_id })
                .with_operands(vec![size_arg])
                .with_dst(result_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        caller.blocks.push(block);
        module.functions.push(caller);

        // Create spec registry with malloc spec
        let mut registry = SpecRegistry::new();
        let mut malloc_spec = FunctionSpec::new("malloc");
        malloc_spec.role = Some(Role::Allocator);
        malloc_spec.returns = Some(ReturnSpec {
            pointer: Some(Pointer::FreshHeap),
            ..ReturnSpec::default()
        });
        registry.add(malloc_spec).unwrap();

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_spec_constraints(&module, &registry, &mut factory, &mut constraints);

        // Should have generated an Addr constraint for the malloc result
        assert_eq!(constraints.addr.len(), 1);
        let addr = constraints.addr.iter().next().unwrap();
        assert_eq!(addr.ptr, result_id);
    }

    #[test]
    fn test_alias_spec_generates_copy_constraint() {
        let mut module = make_module();

        // Add strcpy declaration
        let strcpy_id = FunctionId::derive(b"strcpy");
        let mut strcpy_func = AirFunction::new(strcpy_id, "strcpy");
        strcpy_func.is_declaration = true;
        module.functions.push(strcpy_func);

        // Add caller
        let caller_id = FunctionId::derive(b"caller");
        let mut caller = AirFunction::new(caller_id, "caller");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        let call_inst_id = InstId::derive(b"call_strcpy");
        let result_id = ValueId::derive(b"strcpy_result");
        let dst_arg = ValueId::derive(b"dst");
        let src_arg = ValueId::derive(b"src");

        block.instructions.push(
            Instruction::new(call_inst_id, Operation::CallDirect { callee: strcpy_id })
                .with_operands(vec![dst_arg, src_arg])
                .with_dst(result_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        caller.blocks.push(block);
        module.functions.push(caller);

        // Create spec registry with strcpy spec (returns param.0)
        let mut registry = SpecRegistry::new();
        let mut strcpy_spec = FunctionSpec::new("strcpy");
        strcpy_spec.returns = Some(ReturnSpec {
            aliases: Some("param.0".to_string()),
            ..ReturnSpec::default()
        });
        registry.add(strcpy_spec).unwrap();

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_spec_constraints(&module, &registry, &mut factory, &mut constraints);

        // Should have generated a Copy constraint: result = dst
        assert_eq!(constraints.copy.len(), 1);
        let copy = constraints.copy.iter().next().unwrap();
        assert_eq!(copy.dst, result_id);
        assert_eq!(copy.src, dst_arg);
    }

    #[test]
    fn test_taint_propagation_generates_store_constraint() {
        let mut module = make_module();

        // Add strcpy declaration
        let strcpy_id = FunctionId::derive(b"strcpy");
        let mut strcpy_func = AirFunction::new(strcpy_id, "strcpy");
        strcpy_func.is_declaration = true;
        module.functions.push(strcpy_func);

        // Add caller
        let caller_id = FunctionId::derive(b"caller");
        let mut caller = AirFunction::new(caller_id, "caller");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        let call_inst_id = InstId::derive(b"call_strcpy");
        let result_id = ValueId::derive(b"strcpy_result");
        let dst_arg = ValueId::derive(b"dst");
        let src_arg = ValueId::derive(b"src");

        block.instructions.push(
            Instruction::new(call_inst_id, Operation::CallDirect { callee: strcpy_id })
                .with_operands(vec![dst_arg, src_arg])
                .with_dst(result_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        caller.blocks.push(block);
        module.functions.push(caller);

        // Create spec with taint propagation: param.1 → [param.0]
        let mut registry = SpecRegistry::new();
        let mut strcpy_spec = FunctionSpec::new("strcpy");
        strcpy_spec.params = vec![
            ParamSpec {
                index: 0,
                modifies: Some(true),
                ..ParamSpec::new(0)
            },
            ParamSpec::new(1),
        ];
        strcpy_spec.taint = Some(TaintSpec {
            propagates: vec![TaintPropagation {
                from: TaintLocation::Param(1),
                to: vec![TaintLocation::Param(0)],
            }],
        });
        registry.add(strcpy_spec).unwrap();

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_spec_constraints(&module, &registry, &mut factory, &mut constraints);

        // Should have generated a Store constraint: *dst = src
        assert_eq!(constraints.store.len(), 1);
        let store = constraints.store.iter().next().unwrap();
        assert_eq!(store.dst_ptr, dst_arg);
        assert_eq!(store.src, src_arg);
    }

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
            Instruction::new(call_inst_id, Operation::CallDirect { callee: func_id })
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
        // 1. An Addr constraint for the synthetic value -> fresh object
        // 2. A Store constraint: *memptr = synthetic_value
        assert_eq!(
            constraints.addr.len(),
            1,
            "expected 1 Addr for synthetic fresh object"
        );
        assert_eq!(
            constraints.store.len(),
            1,
            "expected 1 Store for *memptr = synthetic"
        );
        let store = constraints.store.iter().next().unwrap();
        assert_eq!(
            store.dst_ptr, memptr_arg,
            "Store dst_ptr should be the memptr argument"
        );
    }

    #[test]
    fn test_no_spec_no_constraints() {
        let mut module = make_module();

        // Add unknown external function
        let unknown_id = FunctionId::derive(b"unknown_func");
        let mut unknown_func = AirFunction::new(unknown_id, "unknown_func");
        unknown_func.is_declaration = true;
        module.functions.push(unknown_func);

        // Add caller
        let caller_id = FunctionId::derive(b"caller");
        let mut caller = AirFunction::new(caller_id, "caller");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        block.instructions.push(
            Instruction::new(
                InstId::derive(b"call_unknown"),
                Operation::CallDirect { callee: unknown_id },
            )
            .with_dst(ValueId::derive(b"result")),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        caller.blocks.push(block);
        module.functions.push(caller);

        // Empty registry (no specs)
        let registry = SpecRegistry::new();

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_spec_constraints(&module, &registry, &mut factory, &mut constraints);

        // No constraints generated for unknown function
        assert!(constraints.is_empty());
    }
}
