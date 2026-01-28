//! Constraint extraction from AIR.
//!
//! Walks AIR instructions and produces constraints for the points-to solver.

use std::collections::{BTreeMap, BTreeSet};

use rustc_hash::FxHashMap;
use saf_core::air::{
    AirModule, AirType, CastKind, Constant, FieldStep as AirFieldStep, Instruction, Operation,
};
use saf_core::ids::{FunctionId, ObjId, ValueId};

use super::constraint::{
    AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};
use super::location::{FieldPath, LocationFactory, MemoryRegion, PathStep};

/// Extract base constraints common to all extraction modes.
///
/// Generates `Addr` constraints for all globals and all functions. These are
/// always needed regardless of whether interprocedural or reachability
/// filtering is applied.
fn extract_base_constraints(
    module: &AirModule,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    extract_global_addr_constraints(module, factory, constraints);
    extract_function_addr_constraints(module, factory, constraints);
    extract_global_initializers(module, factory, constraints);
}

/// Build a map from `ValueId` to "may be a pointer" based on type information.
///
/// Returns `true` (conservative) for values with no type info, `Pointer` type,
/// or `Opaque` type. Returns `false` for values with known non-pointer types
/// (`Integer`, `Float`, `Void`, `Array`, `Struct`, `Function`).
///
/// This map is built once per extraction and used to filter out `Copy`
/// constraints for non-pointer operations (Phi, Select, Cast, Copy, Freeze),
/// reducing the constraint set the solver must process.
fn build_pointer_map(module: &AirModule) -> FxHashMap<ValueId, bool> {
    let mut map = FxHashMap::default();
    for func in &module.functions {
        for param in &func.params {
            let is_ptr = param
                .param_type
                .and_then(|tid| module.get_type(tid))
                .is_none_or(|ty| {
                    matches!(
                        ty,
                        AirType::Pointer | AirType::Reference { .. } | AirType::Opaque
                    )
                });
            map.insert(param.id, is_ptr);
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    let is_ptr = inst
                        .result_type
                        .and_then(|tid| module.get_type(tid))
                        .is_none_or(|ty| {
                            matches!(
                                ty,
                                AirType::Pointer | AirType::Reference { .. } | AirType::Opaque
                            )
                        });
                    map.insert(dst, is_ptr);
                }
            }
        }
    }
    // Globals are always pointers (their ValueId represents an address).
    for global in &module.globals {
        map.insert(global.id, true);
    }
    map
}

/// Extract instruction-level constraints from functions in the module.
///
/// When `reachable` is `Some`, only processes functions in the set.
/// When `None`, processes all non-declaration functions.
fn extract_instruction_constraints(
    module: &AirModule,
    reachable: Option<&BTreeSet<FunctionId>>,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    let pointer_map = build_pointer_map(module);
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        if let Some(reachable) = reachable {
            if !reachable.contains(&func.id) {
                continue;
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                extract_instruction(inst, &pointer_map, factory, constraints);
            }
        }
    }
}

/// Extract constraints from an AIR module.
///
/// Walks all functions and instructions, generating constraints for
/// pointer operations. The location factory is used to create abstract
/// memory locations with field sensitivity.
pub fn extract_constraints(module: &AirModule, factory: &mut LocationFactory) -> ConstraintSet {
    let mut constraints = ConstraintSet::default();
    extract_base_constraints(module, factory, &mut constraints);
    extract_instruction_constraints(module, None, factory, &mut constraints);
    extract_interprocedural_impl(module, None, &mut constraints);
    constraints
}

/// Extract constraints for a single module within a multi-module program.
///
/// Unlike [`extract_constraints`] which is module-agnostic, this wraps the
/// result in a [`ModuleConstraints`] with module metadata (ID, fingerprint,
/// function IDs) suitable for per-module caching and diffing.
///
/// The `factory` should be shared across modules for cross-module location
/// consistency.
pub fn extract_module_constraints(
    module: &AirModule,
    factory: &mut LocationFactory,
) -> super::module_constraints::ModuleConstraints {
    let constraints = extract_constraints(module, factory);
    let function_ids: BTreeSet<FunctionId> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| f.id)
        .collect();
    let fingerprint = module.id.to_hex();
    super::module_constraints::ModuleConstraints::from_constraint_set(
        module.id,
        &fingerprint,
        &constraints,
        function_ids,
    )
}

/// Generate Addr constraints for all global variables.
///
/// For each global, creates: `Addr(global.id, location(global.obj))`.
/// This ensures that global addresses are tracked by PTA even when globals
/// are referenced directly as operands (like `store ptr @global, ptr %p`)
/// without an explicit `Operation::Global` instruction.
///
/// Also handles global pointer initializers: when a global has
/// `init: Some(Constant::GlobalRef(target_id))`, generates a Store
/// constraint modeling that the global's location contains the target's address.
fn extract_global_addr_constraints(
    module: &AirModule,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    use saf_core::air::Constant;

    for global in &module.globals {
        let loc = factory.get_or_create(global.obj, FieldPath::empty());
        factory.set_region(global.obj, MemoryRegion::Global);
        constraints.addr.insert(AddrConstraint {
            ptr: global.id,
            loc,
        });

        // Handle global pointer initializers (e.g., @p = global ptr @target)
        // This models that *@p = @target (the location contains the target's address)
        if let Some(Constant::GlobalRef(target_id)) = &global.init {
            constraints.store.insert(StoreConstraint {
                dst_ptr: global.id,
                src: *target_id,
            });
        }

        // Pre-create field-indexed locations for globals with aggregate initializers.
        // Without this, PTA's solver cannot find field-indexed locations when
        // processing GEP constraints (e.g., `gep @a, 0, 1` into `int a[2]={1,2}`),
        // because the solver only searches existing locations — it doesn't create
        // new ones. We create one location per element so the solver's
        // `find_or_approximate_location` can find exact matches.
        if let Some(Constant::Aggregate { elements }) = &global.init {
            create_aggregate_field_locations(global.obj, elements, &[], factory);
        }
    }
}

/// Recursively create field-indexed locations for aggregate initializer elements.
///
/// For `int a[2] = {1, 2}`, creates locations with paths `[Field{0}, Field{0}]`
/// and `[Field{0}, Field{1}]` (the first Field{0} is the pointer dereference
/// index from the GEP). For nested aggregates, recurses to create deeper paths.
///
/// `prefix` accumulates the path from outer levels of nesting.
// INVARIANT: Aggregate element indices fit in u32 (< 2^32 elements).
#[allow(clippy::cast_possible_truncation)]
fn create_aggregate_field_locations(
    obj: ObjId,
    elements: &[saf_core::air::Constant],
    prefix: &[PathStep],
    factory: &mut LocationFactory,
) {
    for (i, element) in elements.iter().enumerate() {
        let mut path_steps: Vec<PathStep> = prefix.to_vec();
        path_steps.push(PathStep::Field { index: i as u32 });

        // Create the location for this element
        factory.get_or_create(
            obj,
            FieldPath {
                steps: path_steps.clone(),
            },
        );

        // Also create with the standard GEP prefix [Field{0}, Field{i}]
        // since constant-expression GEPs typically have the pointer dereference
        // index 0 as the first step.
        if prefix.is_empty() {
            let gep_path = vec![
                PathStep::Field { index: 0 },
                PathStep::Field { index: i as u32 },
            ];
            factory.get_or_create(obj, FieldPath { steps: gep_path });
        }

        // Recurse into nested aggregates
        if let saf_core::air::Constant::Aggregate { elements: nested } = element {
            create_aggregate_field_locations(obj, nested, &path_steps, factory);
        }
    }
}

/// Generate Addr constraints for all function addresses.
///
/// For each function, creates: `Addr(func_addr_id, location(func_obj))`.
/// The `func_addr_id` is `ValueId::new(func.id.raw())` which matches the
/// `ValueId` created by the LLVM frontend when a function address is used
/// as an operand (e.g., `pthread_create(..., @foo, ...)`).
///
/// This enables PTA to track function pointers for:
/// - Thread creation (`pthread_create` targets)
/// - Callbacks and function pointer tables
/// - Indirect call resolution
fn extract_function_addr_constraints(
    module: &AirModule,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    for func in &module.functions {
        // Create a ValueId for the function's address
        // This matches the ValueId created in the frontend for function references
        let func_addr_id = ValueId::new(func.id.raw());

        // Create a location for the function
        // The ObjId is derived from the FunctionId, matching FunctionLocationMap
        let func_obj = ObjId::new(func.id.raw());
        let loc = factory.get_or_create(func_obj, FieldPath::empty());

        // Generate Addr constraint: func_addr_id -> function's location
        constraints.addr.insert(AddrConstraint {
            ptr: func_addr_id,
            loc,
        });
    }
}

/// Extract constraints from reachable functions only.
///
/// Like [`extract_constraints`] but only processes functions in the `reachable` set.
/// Functions not in the set are skipped entirely, producing a constraint set
/// that reflects only the code reachable from the given function set.
#[allow(dead_code)] // Public API when pta module is exposed directly
pub fn extract_constraints_reachable(
    module: &AirModule,
    reachable: &BTreeSet<FunctionId>,
    factory: &mut LocationFactory,
) -> ConstraintSet {
    let mut constraints = ConstraintSet::default();
    extract_base_constraints(module, factory, &mut constraints);
    extract_instruction_constraints(module, Some(reachable), factory, &mut constraints);
    extract_interprocedural_impl(module, Some(reachable), &mut constraints);
    constraints
}

/// Extract only intraprocedural constraints from an AIR module.
///
/// Like [`extract_constraints`] but excludes interprocedural arg→param
/// and return→caller copy constraints. Designed for context-sensitive
/// solvers that handle interprocedural flow with context qualification.
pub fn extract_intraprocedural_constraints(
    module: &AirModule,
    factory: &mut LocationFactory,
) -> ConstraintSet {
    let mut constraints = ConstraintSet::default();
    extract_base_constraints(module, factory, &mut constraints);
    extract_instruction_constraints(module, None, factory, &mut constraints);
    constraints
}

/// Extract constraints from global variable initializers.
///
/// Walks `module.globals` and processes `Constant::Aggregate` initializers
/// that may contain function references (vtable modeling).
/// Generates `AddrConstraint` + `StoreConstraint` pairs for function pointers
/// stored in global aggregates.
///
/// For each global with an `Aggregate` initializer, each non-zero integer
/// element whose value matches a function's `FunctionId::raw()` is treated
/// as a function pointer. The function generates constraints modeling
/// "global\[i\] stores &function".
pub fn extract_global_initializers(
    module: &AirModule,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    // Build a set of known function IDs (raw u128 → FunctionId) for lookup.
    let func_ids: BTreeSet<u128> = module.functions.iter().map(|f| f.id.raw()).collect();

    for global in &module.globals {
        if let Some(Constant::Aggregate { elements }) = &global.init {
            extract_aggregate_elements(
                elements,
                global.obj,
                &FieldPath::empty(),
                &func_ids,
                factory,
                constraints,
            );
        }
    }
}

/// Recursively extract function pointer constraints from aggregate elements.
///
/// For each element at field index `i`:
/// - If it is a non-zero `Constant::Int` matching a known function ID,
///   generate `AddrConstraint` + `StoreConstraint` modeling the store.
/// - If it is a nested `Constant::Aggregate`, recurse with an extended field path.
///
/// Create a deterministic synthetic `ValueId` from a tag, global object, and element index.
///
/// Used in aggregate initializer processing to create unique `ValueId`s for
/// function pointers, field pointers, and global reference pointers.
fn make_synthetic_value(kind: &str, global_obj: ObjId, index: usize) -> ValueId {
    ValueId::new(saf_core::id::make_id(
        kind,
        &[
            global_obj.raw().to_le_bytes(),
            (index as u128).to_le_bytes(),
        ]
        .concat(),
    ))
}

fn extract_aggregate_elements(
    elements: &[Constant],
    global_obj: ObjId,
    base_path: &FieldPath,
    func_ids: &BTreeSet<u128>,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    for (i, element) in elements.iter().enumerate() {
        // INVARIANT: Array indices from enumerate() fit in u32 — aggregate
        // elements (structs/arrays) have practical limits far below 2^32 fields.
        #[allow(clippy::cast_possible_truncation)]
        let field_index = i as u32;
        let field_path = base_path.extend(&FieldPath::field(field_index));

        // GEP-prefixed path: GEPs in LLVM IR have a pointer dereference index
        // (usually 0) before the struct field index, producing paths like
        // `[Field{0}, Field{i}]`. We must also Store at this path so that
        // Loads through GEP chains can find the value.
        let gep_path = if base_path.steps.is_empty() {
            Some(FieldPath {
                steps: vec![
                    PathStep::Field { index: 0 },
                    PathStep::Field { index: field_index },
                ],
            })
        } else {
            None
        };

        match element {
            Constant::Int { value, .. } if *value != 0 => {
                // Check if this integer value matches any function's raw ID.
                // In LLVM IR, vtable entries hold function addresses as integers.
                // INVARIANT: Reinterpret i128 as u128 for ID comparison — the bit
                // pattern is what matters, not the signed interpretation.
                #[allow(clippy::cast_sign_loss)]
                let raw = *value as u128;
                if func_ids.contains(&raw) {
                    let func_id = FunctionId::new(raw);
                    let func_obj = ObjId::new(func_id.raw());
                    let func_loc = factory.get_or_create(func_obj, FieldPath::empty());

                    let synthetic_value = make_synthetic_value("synth_fptr", global_obj, i);
                    constraints.addr.insert(AddrConstraint {
                        ptr: synthetic_value,
                        loc: func_loc,
                    });

                    let field_loc = factory.get_or_create(global_obj, field_path.clone());
                    let field_ptr = make_synthetic_value("synth_field_ptr", global_obj, i);
                    constraints.addr.insert(AddrConstraint {
                        ptr: field_ptr,
                        loc: field_loc,
                    });

                    constraints.store.insert(StoreConstraint {
                        dst_ptr: field_ptr,
                        src: synthetic_value,
                    });

                    // Also store at GEP-prefixed path
                    if let Some(gep) = &gep_path {
                        let gep_loc = factory.get_or_create(global_obj, gep.clone());
                        let gep_ptr = make_synthetic_value("synth_gep_field_ptr", global_obj, i);
                        constraints.addr.insert(AddrConstraint {
                            ptr: gep_ptr,
                            loc: gep_loc,
                        });
                        constraints.store.insert(StoreConstraint {
                            dst_ptr: gep_ptr,
                            src: synthetic_value,
                        });
                    }
                }
            }
            Constant::GlobalRef(target_id) => {
                let field_loc = factory.get_or_create(global_obj, field_path.clone());
                let field_ptr = make_synthetic_value("synth_globalref_ptr", global_obj, i);
                constraints.addr.insert(AddrConstraint {
                    ptr: field_ptr,
                    loc: field_loc,
                });

                constraints.store.insert(StoreConstraint {
                    dst_ptr: field_ptr,
                    src: *target_id,
                });

                // Also store at GEP-prefixed path
                if let Some(gep) = &gep_path {
                    let gep_loc = factory.get_or_create(global_obj, gep.clone());
                    let gep_ptr = make_synthetic_value("synth_gep_globalref_ptr", global_obj, i);
                    constraints.addr.insert(AddrConstraint {
                        ptr: gep_ptr,
                        loc: gep_loc,
                    });
                    constraints.store.insert(StoreConstraint {
                        dst_ptr: gep_ptr,
                        src: *target_id,
                    });
                }
            }
            Constant::Aggregate { elements: nested } => {
                extract_aggregate_elements(
                    nested,
                    global_obj,
                    &field_path,
                    func_ids,
                    factory,
                    constraints,
                );
            }
            _ => {
                // Skip null, zero, float, string, undef, zeroinit, and zero ints
            }
        }
    }
}

/// Extract interprocedural constraints (arg→param, return→caller).
///
/// When `reachable` is `Some`, only processes functions in the set.
/// When `None`, processes all non-declaration functions.
fn extract_interprocedural_impl(
    module: &AirModule,
    reachable: Option<&BTreeSet<FunctionId>>,
    constraints: &mut ConstraintSet,
) {
    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> =
        module.functions.iter().map(|f| (f.id, f)).collect();

    let return_values = crate::module_index::collect_return_values(module);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        if let Some(reachable) = reachable {
            if !reachable.contains(&func.id) {
                continue;
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    extract_call_constraints(inst, *callee, &func_map, &return_values, constraints);
                }
            }
        }
    }
}

/// Generate argument→parameter and return→result copy constraints for a call.
fn extract_call_constraints(
    inst: &Instruction,
    callee: FunctionId,
    func_map: &BTreeMap<FunctionId, &saf_core::air::AirFunction>,
    return_values: &BTreeMap<FunctionId, Vec<ValueId>>,
    constraints: &mut ConstraintSet,
) {
    let Some(callee_func) = func_map.get(&callee) else {
        return;
    };

    // arg → param copy constraints
    for param in &callee_func.params {
        if let Some(&arg) = inst.operands.get(param.index as usize) {
            constraints.copy.insert(CopyConstraint {
                dst: param.id,
                src: arg,
            });
        }
    }

    // return → call result copy constraints
    if let Some(dst) = inst.dst {
        if let Some(ret_vals) = return_values.get(&callee) {
            for &ret_val in ret_vals {
                constraints
                    .copy
                    .insert(CopyConstraint { dst, src: ret_val });
            }
        }
    }
}

/// Extract interprocedural constraints for resolved indirect call sites.
///
/// For each `CallIndirect` instruction whose `InstId` appears in `resolved_sites`,
/// generates arg→param and return→caller copy constraints for each resolved target.
/// This enables CI-PTA to track data flow through indirect calls that have been
/// resolved by CG refinement (e.g., function pointers loaded from global structs).
pub fn extract_resolved_indirect_constraints(
    module: &AirModule,
    resolved_sites: &BTreeMap<saf_core::ids::InstId, Vec<FunctionId>>,
    constraints: &mut ConstraintSet,
) {
    if resolved_sites.is_empty() {
        return;
    }

    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> =
        module.functions.iter().map(|f| (f.id, f)).collect();

    let return_values = crate::module_index::collect_return_values(module);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::CallIndirect { .. }) {
                    if let Some(targets) = resolved_sites.get(&inst.id) {
                        // For indirect calls, the last operand is the function pointer;
                        // all preceding operands are arguments.
                        let args = if inst.operands.is_empty() {
                            &[] as &[ValueId]
                        } else {
                            &inst.operands[..inst.operands.len() - 1]
                        };

                        for &target in targets {
                            if let Some(callee_func) = func_map.get(&target) {
                                // arg → param
                                for param in &callee_func.params {
                                    if let Some(&arg) = args.get(param.index as usize) {
                                        constraints.copy.insert(CopyConstraint {
                                            dst: param.id,
                                            src: arg,
                                        });
                                    }
                                }
                                // return → call result
                                if let Some(dst) = inst.dst {
                                    if let Some(ret_vals) = return_values.get(&target) {
                                        for &ret_val in ret_vals {
                                            constraints
                                                .copy
                                                .insert(CopyConstraint { dst, src: ret_val });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Check if a value may be a pointer based on the pre-built pointer map.
///
/// Returns `true` (conservative) if the value is not in the map.
fn may_be_pointer(vid: ValueId, pointer_map: &FxHashMap<ValueId, bool>) -> bool {
    pointer_map.get(&vid).copied().unwrap_or(true)
}

/// Extract constraints from a single instruction.
///
/// The `pointer_map` is used to skip `Copy` constraints for non-pointer
/// operations (Phi, Select, Cast, Copy, Freeze), reducing solver work.
// NOTE: This function handles all AIR instruction types in a single match.
// Splitting would fragment the constraint extraction logic.
#[allow(clippy::too_many_lines)]
fn extract_instruction(
    inst: &Instruction,
    pointer_map: &FxHashMap<ValueId, bool>,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    // Handle operations that have no dst (void-returning)
    match &inst.op {
        Operation::Store => {
            if inst.operands.len() >= 2 {
                constraints.store.insert(StoreConstraint {
                    dst_ptr: inst.operands[1],
                    src: inst.operands[0],
                });
            }
            return;
        }
        Operation::Memcpy => {
            // Model memcpy(dst, src, n) as: tmp = *src; *dst = tmp
            // Load+Store correctly models the indirection — only one-directional
            // flow from src's pointees into dst's pointees. The previous Copy(dst, src)
            // caused bidirectional aliasing, inflating points-to sets.
            // operands[0] = dst pointer, operands[1] = src pointer
            if inst.operands.len() >= 2 {
                let tmp = ValueId::new(saf_core::id::make_id(
                    "memcpy_tmp",
                    &inst.id.raw().to_le_bytes(),
                ));
                constraints.load.insert(LoadConstraint {
                    dst: tmp,
                    src_ptr: inst.operands[1],
                });
                constraints.store.insert(StoreConstraint {
                    dst_ptr: inst.operands[0],
                    src: tmp,
                });
            }
            return;
        }
        _ => {}
    }

    // All other pointer-relevant ops need a dst
    let Some(dst) = inst.dst else {
        return;
    };

    match &inst.op {
        // Allocation operations create Addr constraints
        Operation::Alloca { .. } => {
            // Use instruction ID as the object ID for stack allocation
            let obj = ObjId::new(inst.id.raw());
            let loc = factory.get_or_create(obj, FieldPath::empty());
            factory.set_region(obj, MemoryRegion::Stack);
            constraints.addr.insert(AddrConstraint { ptr: dst, loc });
        }

        Operation::Global { obj } => {
            let loc = factory.get_or_create(*obj, FieldPath::empty());
            constraints.addr.insert(AddrConstraint { ptr: dst, loc });
        }

        Operation::HeapAlloc { .. } => {
            // Use instruction ID as the object ID for heap allocation
            let obj = ObjId::new(inst.id.raw());
            let loc = factory.get_or_create(obj, FieldPath::empty());
            factory.set_region(obj, MemoryRegion::Heap);
            constraints.addr.insert(AddrConstraint { ptr: dst, loc });
        }

        // Memory operations — NOT filtered (structural, required for soundness)
        Operation::Load => {
            if !inst.operands.is_empty() {
                constraints.load.insert(LoadConstraint {
                    dst,
                    src_ptr: inst.operands[0],
                });
            }
        }

        Operation::Gep { field_path } => {
            if !inst.operands.is_empty() {
                let (path, index_operands) = convert_field_path_with_operands(field_path, inst);
                constraints.gep.insert(GepConstraint {
                    dst,
                    src_ptr: inst.operands[0],
                    path,
                    index_operands,
                });
            }
        }

        // Copy operations — filtered by pointer type when type info is available.
        // Skip Copy constraints whose destination is known to be a non-pointer
        // (e.g., integer, float). This is safe because non-pointer values can
        // never participate in points-to relationships.
        Operation::Copy | Operation::Freeze => {
            if !inst.operands.is_empty() && may_be_pointer(dst, pointer_map) {
                constraints.copy.insert(CopyConstraint {
                    dst,
                    src: inst.operands[0],
                });
            }
        }

        Operation::Cast { kind, .. } => {
            // Only pointer-relevant casts propagate points-to info;
            // integer/float casts (Trunc, ZExt, SExt, FPTo*, etc.) cannot produce pointers.
            if matches!(
                kind,
                CastKind::Bitcast
                    | CastKind::IntToPtr
                    | CastKind::PtrToInt
                    | CastKind::AddrSpaceCast
            ) && !inst.operands.is_empty()
                && may_be_pointer(dst, pointer_map)
            {
                constraints.copy.insert(CopyConstraint {
                    dst,
                    src: inst.operands[0],
                });
            }
        }

        Operation::Phi { incoming } => {
            // Skip phi nodes for non-pointer values
            if may_be_pointer(dst, pointer_map) {
                for (_block, value) in incoming {
                    constraints.copy.insert(CopyConstraint { dst, src: *value });
                }
            }
        }

        Operation::Select => {
            // operands: [cond, true_val, false_val]
            if inst.operands.len() >= 3 && may_be_pointer(dst, pointer_map) {
                constraints.copy.insert(CopyConstraint {
                    dst,
                    src: inst.operands[1], // true value
                });
                constraints.copy.insert(CopyConstraint {
                    dst,
                    src: inst.operands[2], // false value
                });
            }
        }

        // Operations that don't generate pointer constraints
        Operation::Store
        | Operation::Memcpy
        | Operation::CallDirect { .. }
        | Operation::CallIndirect { .. }
        | Operation::Br { .. }
        | Operation::CondBr { .. }
        | Operation::Switch { .. }
        | Operation::Ret
        | Operation::Unreachable
        | Operation::BinaryOp { .. }
        | Operation::Memset => {}
    }
}

/// Convert AIR field path to PTA field path, extracting index operands.
///
/// Returns the field path and a list of ValueIds for each `Index` step.
/// The operands follow LLVM GEP convention: `operands[0]` is the base pointer,
/// subsequent operands are indices. Each `FieldStep::Index` in the AIR path
/// corresponds to one of these index operands (in order).
///
/// Currently converts all array indices to `Unknown` (collapsed).
/// Index sensitivity is applied later during constraint solving using
/// the index_operands and the module's constants table.
fn convert_field_path_with_operands(
    air_path: &saf_core::air::FieldPath,
    inst: &Instruction,
) -> (FieldPath, Vec<ValueId>) {
    use super::location::IndexExpr;

    let mut index_operands = Vec::new();
    let mut operand_idx = 1; // Skip operands[0] which is the base pointer

    let steps = air_path
        .steps
        .iter()
        .map(|step| match step {
            AirFieldStep::Index => {
                // Capture the index operand if available
                if let Some(&operand) = inst.operands.get(operand_idx) {
                    index_operands.push(operand);
                }
                operand_idx += 1;
                PathStep::Index(IndexExpr::Unknown)
            }
            AirFieldStep::Field { index } => {
                // Field steps also consume an operand in LLVM GEP
                operand_idx += 1;
                PathStep::Field { index: *index }
            }
        })
        .collect();

    (FieldPath { steps }, index_operands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{
        AirBlock, AirFunction, AirGlobal, AirModule, AirParam, FieldPath as AirFieldPath,
        HeapAllocKind, Instruction, Operation,
    };
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ObjId, ValueId};

    use crate::pta::config::FieldSensitivity;
    use crate::pta::location::LocationFactory;

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    fn make_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    #[test]
    fn extract_empty_module() {
        let module = make_module();
        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);
        assert!(constraints.is_empty());
    }

    #[test]
    fn extract_alloca_creates_addr_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let inst_id = InstId::derive(b"alloca");
        let dst_id = ValueId::derive(b"alloca_result");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Alloca { size_bytes: None }).with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // 1 from alloca + 1 from function address
        assert_eq!(constraints.addr.len(), 2);
        let addr = constraints.addr.iter().find(|a| a.ptr == dst_id).unwrap();
        assert_eq!(addr.ptr, dst_id);
    }

    #[test]
    fn extract_global_creates_addr_constraint() {
        let mut module = make_module();

        let global_obj = ObjId::derive(b"global_obj");
        let global_value = ValueId::derive(b"global_value");
        module
            .globals
            .push(AirGlobal::new(global_value, global_obj, "my_global"));

        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");
        let inst_id = InstId::derive(b"global_ref");
        let dst_id = ValueId::derive(b"global_result");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Global { obj: global_obj }).with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // Three Addr constraints:
        // 1. From extract_global_addr_constraints: global_value → global_obj
        // 2. From extract_function_addr_constraints: func_addr → func_obj
        // 3. From Operation::Global instruction: dst_id → global_obj
        assert_eq!(constraints.addr.len(), 3);
    }

    #[test]
    fn extract_load_creates_load_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let ptr_id = ValueId::derive(b"ptr");
        let dst_id = ValueId::derive(b"loaded");
        let inst_id = InstId::derive(b"load");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Load)
                .with_operands(vec![ptr_id])
                .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        assert_eq!(constraints.load.len(), 1);
        let load = constraints.load.iter().next().unwrap();
        assert_eq!(load.dst, dst_id);
        assert_eq!(load.src_ptr, ptr_id);
    }

    #[test]
    fn extract_store_creates_store_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let val_id = ValueId::derive(b"value");
        let ptr_id = ValueId::derive(b"ptr");
        let inst_id = InstId::derive(b"store");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block
            .instructions
            .push(Instruction::new(inst_id, Operation::Store).with_operands(vec![val_id, ptr_id]));
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        assert_eq!(constraints.store.len(), 1);
        let store = constraints.store.iter().next().unwrap();
        assert_eq!(store.dst_ptr, ptr_id);
        assert_eq!(store.src, val_id);
    }

    #[test]
    fn extract_copy_creates_copy_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let src_id = ValueId::derive(b"src");
        let dst_id = ValueId::derive(b"dst");
        let inst_id = InstId::derive(b"copy");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Copy)
                .with_operands(vec![src_id])
                .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        assert_eq!(constraints.copy.len(), 1);
        let copy = constraints.copy.iter().next().unwrap();
        assert_eq!(copy.dst, dst_id);
        assert_eq!(copy.src, src_id);
    }

    #[test]
    fn extract_gep_creates_gep_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let base_ptr = ValueId::derive(b"base");
        let dst_id = ValueId::derive(b"gep_result");
        let inst_id = InstId::derive(b"gep");

        let field_path = AirFieldPath::field(0);

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Gep { field_path })
                .with_operands(vec![base_ptr])
                .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        assert_eq!(constraints.gep.len(), 1);
        let gep = constraints.gep.iter().next().unwrap();
        assert_eq!(gep.dst, dst_id);
        assert_eq!(gep.src_ptr, base_ptr);
    }

    #[test]
    fn extract_phi_creates_copy_constraints() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let dst_id = ValueId::derive(b"phi_result");
        let val1 = ValueId::derive(b"val1");
        let val2 = ValueId::derive(b"val2");
        let block1 = BlockId::derive(b"block1");
        let block2 = BlockId::derive(b"block2");
        let inst_id = InstId::derive(b"phi");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                inst_id,
                Operation::Phi {
                    incoming: vec![(block1, val1), (block2, val2)],
                },
            )
            .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // Should create 2 copy constraints (one per incoming value)
        assert_eq!(constraints.copy.len(), 2);
    }

    #[test]
    fn extract_select_creates_copy_constraints() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let cond = ValueId::derive(b"cond");
        let true_val = ValueId::derive(b"true_val");
        let false_val = ValueId::derive(b"false_val");
        let dst_id = ValueId::derive(b"select_result");
        let inst_id = InstId::derive(b"select");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Select)
                .with_operands(vec![cond, true_val, false_val])
                .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // Should create 2 copy constraints (one per branch)
        assert_eq!(constraints.copy.len(), 2);
    }

    #[test]
    fn extract_heap_alloc_creates_addr_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let inst_id = InstId::derive(b"malloc");
        let dst_id = ValueId::derive(b"malloc_result");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                inst_id,
                Operation::HeapAlloc {
                    kind: HeapAllocKind::Malloc,
                },
            )
            .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // 1 from HeapAlloc + 1 from function address
        assert_eq!(constraints.addr.len(), 2);
    }

    #[test]
    fn extract_cast_creates_copy_constraint() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let src_id = ValueId::derive(b"src");
        let dst_id = ValueId::derive(b"cast_result");
        let inst_id = InstId::derive(b"cast");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                inst_id,
                Operation::Cast {
                    kind: saf_core::air::CastKind::Bitcast,
                    target_bits: None,
                },
            )
            .with_operands(vec![src_id])
            .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        assert_eq!(constraints.copy.len(), 1);
    }

    #[test]
    fn extract_memcpy_creates_load_store_constraints() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test_fn"), "test_fn");

        let dst_ptr = ValueId::derive(b"dst_ptr");
        let src_ptr = ValueId::derive(b"src_ptr");
        let inst_id = InstId::derive(b"memcpy");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Memcpy).with_operands(vec![dst_ptr, src_ptr]),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // memcpy(dst, src) should generate Load(tmp, src) + Store(dst, tmp)
        assert_eq!(constraints.load.len(), 1, "one load for memcpy");
        assert_eq!(constraints.store.len(), 1, "one store for memcpy");
        assert!(
            constraints.copy.is_empty(),
            "no copy constraints from memcpy"
        );

        let load = constraints.load.iter().next().unwrap();
        let store = constraints.store.iter().next().unwrap();

        // Load reads from src pointer
        assert_eq!(load.src_ptr, src_ptr);
        // Store writes to dst pointer
        assert_eq!(store.dst_ptr, dst_ptr);
        // The synthetic tmp connects load dst to store src
        assert_eq!(load.dst, store.src);
    }

    // =========================================================================
    // Tests for extract_global_initializers
    // =========================================================================

    #[test]
    fn extract_global_initializers_empty_globals_does_nothing() {
        let module = make_module();
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_global_initializers(&module, &mut factory, &mut constraints);

        assert!(constraints.is_empty());
    }

    #[test]
    fn extract_global_initializers_with_function_pointer_aggregate() {
        let mut module = make_module();

        // Create a function whose raw ID we'll embed in the global initializer
        let func_id = FunctionId::new(42);
        let mut func = AirFunction::new(func_id, "target_func");
        let block = AirBlock::new(BlockId::derive(b"entry"));
        func.blocks.push(block);
        module.functions.push(func);

        // Create a global with an Aggregate initializer containing the function ID
        let global_obj = ObjId::derive(b"vtable");
        let global_value = ValueId::derive(b"vtable_ptr");
        let mut global = AirGlobal::new(global_value, global_obj, "vtable");
        global.init = Some(saf_core::air::Constant::Aggregate {
            elements: vec![
                saf_core::air::Constant::Int { value: 0, bits: 64 }, // null slot
                saf_core::air::Constant::Int {
                    value: 42,
                    bits: 64,
                }, // function pointer
            ],
        });
        module.globals.push(global);

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_global_initializers(&module, &mut factory, &mut constraints);

        // Should have generated constraints for the function pointer at index 1
        // AddrConstraint for synthetic_value → function location
        // AddrConstraint for field_ptr → field location (simple path)
        // AddrConstraint for gep_field_ptr → field location (GEP-prefixed path)
        // StoreConstraint for *field_ptr = synthetic_value (simple path)
        // StoreConstraint for *gep_field_ptr = synthetic_value (GEP-prefixed path)
        assert_eq!(constraints.addr.len(), 3);
        assert_eq!(constraints.store.len(), 2);
    }

    #[test]
    fn extract_global_initializers_skips_non_matching_ints() {
        let mut module = make_module();

        // Create a function with a known ID
        let func_id = FunctionId::new(100);
        let mut func = AirFunction::new(func_id, "known_func");
        let block = AirBlock::new(BlockId::derive(b"entry"));
        func.blocks.push(block);
        module.functions.push(func);

        // Global with integer values that don't match any function
        let global_obj = ObjId::derive(b"data");
        let global_value = ValueId::derive(b"data_ptr");
        let mut global = AirGlobal::new(global_value, global_obj, "data");
        global.init = Some(saf_core::air::Constant::Aggregate {
            elements: vec![
                saf_core::air::Constant::Int {
                    value: 999,
                    bits: 64,
                }, // not a known function
                saf_core::air::Constant::Int { value: 0, bits: 64 }, // zero, always skipped
            ],
        });
        module.globals.push(global);

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        extract_global_initializers(&module, &mut factory, &mut constraints);

        // No matching function IDs, so no constraints generated
        assert!(constraints.is_empty());
    }

    // =========================================================================
    // Tests for extract_constraints_reachable
    // =========================================================================

    #[test]
    fn extract_constraints_reachable_empty_reachable_set() {
        let mut module = make_module();

        // Add a function with an alloca instruction
        let func_id = FunctionId::derive(b"unreachable_fn");
        let mut func = AirFunction::new(func_id, "unreachable_fn");
        let inst_id = InstId::derive(b"alloca_unreachable");
        let dst_id = ValueId::derive(b"alloca_result_unreachable");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(inst_id, Operation::Alloca { size_bytes: None }).with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let mut factory = make_factory();
        let reachable = BTreeSet::new();
        let constraints = extract_constraints_reachable(&module, &reachable, &mut factory);

        // Empty reachable set means no functions' instructions are processed.
        // However, function address constraints are generated for ALL functions
        // (reachable or not) to support function pointer resolution.
        // 1 function → 1 function addr constraint
        assert_eq!(
            constraints.addr.len(),
            1,
            "function addr constraint for unreachable_fn"
        );
        assert!(constraints.store.is_empty(), "no store constraints");
        assert!(constraints.load.is_empty(), "no load constraints");
    }

    #[test]
    fn extract_constraints_reachable_extracts_only_reachable() {
        let mut module = make_module();

        // Reachable function with an alloca
        let reachable_id = FunctionId::derive(b"reachable_fn");
        let mut reachable_func = AirFunction::new(reachable_id, "reachable_fn");
        let inst_id1 = InstId::derive(b"alloca_reachable");
        let dst_id1 = ValueId::derive(b"alloca_result_reachable");
        let mut block1 = AirBlock::new(BlockId::derive(b"entry_reachable"));
        block1.instructions.push(
            Instruction::new(inst_id1, Operation::Alloca { size_bytes: None }).with_dst(dst_id1),
        );
        block1
            .instructions
            .push(Instruction::new(InstId::derive(b"ret1"), Operation::Ret));
        reachable_func.blocks.push(block1);
        module.functions.push(reachable_func);

        // Unreachable function with an alloca
        let unreachable_id = FunctionId::derive(b"unreachable_fn2");
        let mut unreachable_func = AirFunction::new(unreachable_id, "unreachable_fn2");
        let inst_id2 = InstId::derive(b"alloca_unreachable2");
        let dst_id2 = ValueId::derive(b"alloca_result_unreachable2");
        let mut block2 = AirBlock::new(BlockId::derive(b"entry_unreachable"));
        block2.instructions.push(
            Instruction::new(inst_id2, Operation::Alloca { size_bytes: None }).with_dst(dst_id2),
        );
        block2
            .instructions
            .push(Instruction::new(InstId::derive(b"ret2"), Operation::Ret));
        unreachable_func.blocks.push(block2);
        module.functions.push(unreachable_func);

        let mut factory = make_factory();
        let mut reachable = BTreeSet::new();
        reachable.insert(reachable_id);

        let constraints = extract_constraints_reachable(&module, &reachable, &mut factory);

        // The reachable function's alloca + 2 function addr constraints (both functions)
        // Function addr constraints are generated for ALL functions to support fn ptr resolution.
        assert_eq!(constraints.addr.len(), 3, "1 alloca + 2 function addrs");
        assert!(
            constraints.addr.iter().any(|a| a.ptr == dst_id1),
            "reachable alloca constraint should be present"
        );
    }

    // =========================================================================
    // Tests for interprocedural parameter passing
    // =========================================================================

    /// Build a two-function module: caller calls callee(arg) → result.
    fn make_interproc_module() -> (AirModule, ValueId, ValueId, ValueId, ValueId) {
        let mut module = make_module();

        // Callee: fn callee(param0) { alloca; ret param0 }
        let callee_id = FunctionId::derive(b"callee");
        let param0_id = ValueId::derive(b"param0");
        let ret_val = param0_id; // returns the parameter

        let mut callee = AirFunction::new(callee_id, "callee");
        callee.params.push(AirParam::new(param0_id, 0));

        let alloca_inst = InstId::derive(b"callee_alloca");
        let alloca_dst = ValueId::derive(b"callee_alloca_result");
        let mut callee_block = AirBlock::new(BlockId::derive(b"callee_entry"));
        callee_block.instructions.push(
            Instruction::new(alloca_inst, Operation::Alloca { size_bytes: None })
                .with_dst(alloca_dst),
        );
        callee_block.instructions.push(
            Instruction::new(InstId::derive(b"callee_ret"), Operation::Ret)
                .with_operands(vec![ret_val]),
        );
        callee.blocks.push(callee_block);
        module.functions.push(callee);

        // Caller: fn caller() { arg = alloca; result = call callee(arg); ret }
        let caller_id = FunctionId::derive(b"caller");
        let arg_id = ValueId::derive(b"arg");
        let call_result_id = ValueId::derive(b"call_result");

        let mut caller = AirFunction::new(caller_id, "caller");
        let arg_alloca = InstId::derive(b"arg_alloca");
        let call_inst = InstId::derive(b"call_inst");

        let mut caller_block = AirBlock::new(BlockId::derive(b"caller_entry"));
        caller_block.instructions.push(
            Instruction::new(arg_alloca, Operation::Alloca { size_bytes: None }).with_dst(arg_id),
        );
        caller_block.instructions.push(
            Instruction::new(call_inst, Operation::CallDirect { callee: callee_id })
                .with_operands(vec![arg_id])
                .with_dst(call_result_id),
        );
        caller_block.instructions.push(Instruction::new(
            InstId::derive(b"caller_ret"),
            Operation::Ret,
        ));
        caller.blocks.push(caller_block);
        module.functions.push(caller);

        (module, arg_id, param0_id, call_result_id, ret_val)
    }

    #[test]
    fn interproc_arg_to_param_copy_constraint() {
        let (module, arg_id, param0_id, _, _) = make_interproc_module();
        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // Should have a copy constraint: param0 = arg (arg → param)
        let has_arg_param = constraints
            .copy
            .iter()
            .any(|c| c.dst == param0_id && c.src == arg_id);
        assert!(
            has_arg_param,
            "Expected copy constraint arg → param0, got: {:?}",
            constraints.copy
        );
    }

    #[test]
    fn interproc_return_to_caller_copy_constraint() {
        let (module, _, _, call_result_id, ret_val) = make_interproc_module();
        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // Should have a copy constraint: call_result = ret_val (return → caller)
        let has_ret_caller = constraints
            .copy
            .iter()
            .any(|c| c.dst == call_result_id && c.src == ret_val);
        assert!(
            has_ret_caller,
            "Expected copy constraint ret_val → call_result, got: {:?}",
            constraints.copy
        );
    }

    #[test]
    fn interproc_declaration_callee_no_constraints() {
        let mut module = make_module();

        // Declaration-only callee (no body)
        let callee_id = FunctionId::derive(b"extern_fn");
        let mut callee = AirFunction::new(callee_id, "extern_fn");
        callee.is_declaration = true;
        callee
            .params
            .push(AirParam::new(ValueId::derive(b"extern_param"), 0));
        module.functions.push(callee);

        // Caller that calls the declaration
        let mut caller = AirFunction::new(FunctionId::derive(b"caller2"), "caller2");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"call_extern"),
                Operation::CallDirect { callee: callee_id },
            )
            .with_operands(vec![ValueId::derive(b"some_arg")])
            .with_dst(ValueId::derive(b"extern_result")),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        caller.blocks.push(block);
        module.functions.push(caller);

        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);

        // Arg→param constraint is still generated (declaration has params)
        let has_param_copy = constraints.copy.iter().any(|c| {
            c.dst == ValueId::derive(b"extern_param") && c.src == ValueId::derive(b"some_arg")
        });
        assert!(
            has_param_copy,
            "Should still generate arg→param for declarations"
        );

        // No return→caller constraint (declaration has no Ret instructions)
        let has_ret_copy = constraints
            .copy
            .iter()
            .any(|c| c.dst == ValueId::derive(b"extern_result"));
        assert!(
            !has_ret_copy,
            "Should not generate return→caller for declarations (no body)"
        );
    }

    #[test]
    fn interproc_pointer_flow_through_call() {
        // Verify that PTA can track pointer flow through a function call
        let (module, _, _, _, _) = make_interproc_module();
        let mut factory = make_factory();
        let constraints = extract_constraints(&module, &mut factory);
        let result = crate::pta::solver::solve(&constraints, &factory, 1_000_000);

        // arg_id should point to an alloca location
        let arg_id = ValueId::derive(b"arg");
        let arg_pts = result.get(&arg_id);
        assert!(
            arg_pts.map_or(false, |s| !s.is_empty()),
            "arg should have a points-to set from its alloca"
        );

        // param0 should also point to that same location (via arg→param copy)
        let param0_id = ValueId::derive(b"param0");
        let param_pts = result.get(&param0_id);
        assert!(
            param_pts.map_or(false, |s| !s.is_empty()),
            "param0 should inherit arg's points-to set"
        );

        // call_result should also point there (via return→caller copy)
        let call_result_id = ValueId::derive(b"call_result");
        let result_pts = result.get(&call_result_id);
        assert!(
            result_pts.map_or(false, |s| !s.is_empty()),
            "call_result should inherit return value's points-to set"
        );

        // All three should point to the same location
        assert_eq!(
            arg_pts.unwrap(),
            param_pts.unwrap(),
            "arg and param should point to same locations"
        );
        assert_eq!(
            arg_pts.unwrap(),
            result_pts.unwrap(),
            "arg and call_result should point to same locations"
        );
    }
}
