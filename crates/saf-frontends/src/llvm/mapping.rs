//! LLVM IR to AIR mapping.
//!
//! Converts LLVM instructions, values, and types to their AIR equivalents.
//! This is the core conversion logic shared by all LLVM version adapters.

#![cfg(any(feature = "llvm-18", feature = "llvm-22"))]

use std::collections::BTreeMap;

use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::module::Module;
use inkwell::values::{
    AnyValue, AsValueRef, BasicValueEnum, FunctionValue, GlobalValue, InstructionOpcode,
    InstructionValue, Operand, PhiValue,
};
use rustc_hash::FxHashMap;

use saf_core::air::{
    AirBlock, AirBundle, AirFunction, AirGlobal, AirModule, AirParam, BinaryOp, CastKind, Constant,
    FieldPath, FieldStep, HeapAllocKind, Instruction, Operation,
};
use saf_core::id::make_id;
use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ObjId, ValueId};

use super::debug_info::{
    LocalVarNameMap, SourceFileTracker, extract_function_span, extract_function_symbol,
    extract_local_variable_names, extract_register_name, extract_span,
};
use super::error::LlvmError;
use super::intrinsics::{IntrinsicMapping, classify_intrinsic};

/// Helper to extract `BasicValueEnum` from `Operand`.
fn operand_as_value(op: Operand<'_>) -> Option<BasicValueEnum<'_>> {
    match op {
        Operand::Value(v) => Some(v),
        Operand::Block(_) => None,
    }
}

/// Helper to extract `BasicBlock` from `Operand`.
fn operand_as_block(op: Operand<'_>) -> Option<BasicBlock<'_>> {
    match op {
        Operand::Block(bb) => Some(bb),
        Operand::Value(_) => None,
    }
}

/// Context for LLVM to AIR conversion.
pub(crate) struct MappingContext<'ctx> {
    /// The module being converted.
    pub module_id: ModuleId,
    /// Map from LLVM function names to AIR function IDs.
    pub function_ids: BTreeMap<String, FunctionId>,
    /// Map from `LLVMValueRef` pointer (as `usize`) to AIR value IDs.
    /// Using pointer identity avoids `print_to_string()` FFI calls on cache hits.
    pub ptr_cache: FxHashMap<usize, ValueId>,
    /// Counter for block IDs within a function.
    pub block_counter: usize,
    /// Counter for instruction IDs within a block.
    pub inst_counter: usize,
    /// Map from LLVM global names to AIR object IDs.
    pub global_obj_ids: BTreeMap<String, ObjId>,
    /// Map from LLVM global names to AIR value IDs.
    /// Used to ensure consistent `ValueId`s when globals are referenced as operands.
    pub global_value_ids: BTreeMap<String, ValueId>,
    /// Source file tracker.
    pub source_files: SourceFileTracker,
    /// Map from `ValueId` to constant value (for inline constants).
    pub constants: BTreeMap<ValueId, Constant>,
    /// Map from global name to initializer constant.
    /// Used to resolve constant-expression GEPs into globals (Plan 084 Phase B).
    pub global_inits: BTreeMap<String, Constant>,
    /// Current block ID (set during block conversion).
    /// Used by `decompose_constant_gep` to create synthetic instruction IDs.
    pub current_block_id: Option<BlockId>,
    /// Synthetic instructions generated during operand collection.
    /// Drained and inserted before the instruction that references their results.
    pub pending_instructions: Vec<Instruction>,
    /// Type interner for populating the AIR type table.
    pub type_interner: super::type_intern::TypeInterner,
    /// Module-level local variable name mappings from debug info.
    ///
    /// Populated once from `extract_local_variable_names()` during module conversion.
    /// Maps `function_name` -> (`register_name` -> `variable_name`).
    pub module_local_var_names: LocalVarNameMap,
    /// Current function's local variable name map (`register_name` -> `variable_name`).
    ///
    /// Set at the start of each function conversion from `module_local_var_names`.
    pub current_local_var_names: BTreeMap<String, String>,
    /// Phantom data for the context lifetime.
    _phantom: std::marker::PhantomData<&'ctx ()>,
}

impl MappingContext<'_> {
    /// Create a new mapping context.
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            function_ids: BTreeMap::new(),
            ptr_cache: FxHashMap::default(),
            block_counter: 0,
            inst_counter: 0,
            global_obj_ids: BTreeMap::new(),
            global_value_ids: BTreeMap::new(),
            source_files: SourceFileTracker::new(),
            constants: BTreeMap::new(),
            global_inits: BTreeMap::new(),
            current_block_id: None,
            pending_instructions: Vec::new(),
            type_interner: super::type_intern::TypeInterner::new(),
            module_local_var_names: BTreeMap::new(),
            current_local_var_names: BTreeMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get or create a function ID.
    pub fn get_or_create_function_id(&mut self, func: FunctionValue<'_>) -> FunctionId {
        let name = func.get_name().to_str().unwrap_or("").to_string();

        *self.function_ids.entry(name.clone()).or_insert_with(|| {
            let data = format!("{:032x}fn{}", self.module_id.raw(), name);
            FunctionId::derive(data.as_bytes())
        })
    }

    /// Get a function ID by name.
    pub fn get_function_id_by_name(&self, name: &str) -> Option<FunctionId> {
        self.function_ids.get(name).copied()
    }

    /// Get or create a function ID for an external function.
    ///
    /// This is used for external function calls (like `free`, `printf`) where
    /// the function is not defined in the module but we still want to track it.
    pub fn get_or_create_external_function_id(&mut self, name: &str) -> FunctionId {
        *self
            .function_ids
            .entry(name.to_string())
            .or_insert_with(|| {
                let data = format!("{:032x}ext_fn{}", self.module_id.raw(), name);
                FunctionId::derive(data.as_bytes())
            })
    }

    /// Create a block ID.
    pub fn create_block_id(&mut self, func_id: FunctionId) -> BlockId {
        let idx = self.block_counter;
        self.block_counter += 1;
        let data = format!("{:032x}bb{}", func_id.raw(), idx);
        BlockId::derive(data.as_bytes())
    }

    /// Reset block counter for a new function.
    pub fn reset_block_counter(&mut self) {
        self.block_counter = 0;
    }

    /// Create an instruction ID.
    pub fn create_inst_id(&mut self, block_id: BlockId) -> InstId {
        let idx = self.inst_counter;
        self.inst_counter += 1;
        let data = format!("{:032x}inst{}", block_id.raw(), idx);
        InstId::derive(data.as_bytes())
    }

    /// Reset instruction counter for a new block.
    pub fn reset_inst_counter(&mut self) {
        self.inst_counter = 0;
    }

    /// Create a value ID for an instruction result.
    // NOTE: Takes `&mut self` for API consistency with other create_* methods
    // that mutate internal state (counters, caches). Future extensions may add
    // caching or validation that requires mutable access.
    #[allow(clippy::unused_self)]
    pub fn create_inst_result_id(&mut self, inst_id: InstId) -> ValueId {
        let data = format!("{:032x}result", inst_id.raw());
        ValueId::derive(data.as_bytes())
    }

    /// Create a value ID for a function parameter.
    // NOTE: Takes `&mut self` for API consistency — see create_inst_result_id.
    #[allow(clippy::unused_self)]
    pub fn create_param_id(&mut self, func_id: FunctionId, param_index: u32) -> ValueId {
        let data = format!("{:032x}param{}", func_id.raw(), param_index);
        ValueId::derive(data.as_bytes())
    }

    /// Get or create an object ID for a global.
    pub fn get_or_create_global_obj_id(&mut self, global: GlobalValue<'_>) -> ObjId {
        let name = global.get_name().to_str().unwrap_or("").to_string();

        if let Some(id) = self.global_obj_ids.get(&name) {
            return *id;
        }

        let id = if name.is_empty() {
            let idx = self.global_obj_ids.len();
            let data = format!("{:032x}anon_global{}", self.module_id.raw(), idx);
            ObjId::derive(data.as_bytes())
        } else {
            let data = format!("{:032x}gobj{}", self.module_id.raw(), name);
            ObjId::derive(data.as_bytes())
        };

        self.global_obj_ids.insert(name, id);
        id
    }

    /// Get or create a value ID for a basic value.
    ///
    /// If the value is a constant (integer, float, null), also records it in the
    /// constants map so analyses can look up the constant value by `ValueId`.
    ///
    /// If the value is a global reference (e.g., `@global_name`), returns the
    /// same `ValueId` that was assigned when the global was defined. This ensures
    /// consistent tracking of global addresses for points-to analysis.
    ///
    /// If the value is a function reference (e.g., `@func_name`), creates a
    /// consistent `ValueId` and records a `GlobalRef` constant so MTA and PTA
    /// can resolve function pointers (e.g., for `pthread_create` targets).
    pub fn get_or_create_value_id(&mut self, value: BasicValueEnum<'_>) -> ValueId {
        // Fast path: check pointer-keyed cache first (no FFI call).
        // LLVM SSA values have stable pointer identity within a module context.
        let ptr_key = value.as_value_ref() as usize;
        if let Some(&existing_id) = self.ptr_cache.get(&ptr_key) {
            return existing_id;
        }

        // Cache miss — need string repr for ID derivation and global detection.
        let repr = value.print_to_string().to_string();

        // Decompose constant-expression GEPs into globals with aggregate initializers.
        // A constant GEP like `ptr getelementptr inbounds ([2 x i32], ptr @a, i64 0, i64 1)`
        // computes a field-indexed address into `@a`. Without decomposition,
        // `extract_global_name_from_repr` would match "@a" and return the base
        // global's ValueId, making `a[0]` and `a[1]` indistinguishable to PTA.
        //
        // Only decompose GEPs into globals that have aggregate initializers —
        // for non-aggregate globals, the simple base-address resolution is correct
        // and produces better results for alias analysis.
        if repr.contains("getelementptr") {
            if let Some((global_name, _)) = parse_constant_gep(&repr) {
                if matches!(
                    self.global_inits.get(global_name),
                    Some(Constant::Aggregate { .. })
                ) {
                    if let Some(gep_result_id) = self.decompose_constant_gep(&repr) {
                        self.ptr_cache.insert(ptr_key, gep_result_id);
                        return gep_result_id;
                    }
                }
            }
            // Non-aggregate global or decomposition failed — fall through to
            // global ref check (simple base-address resolution is appropriate).
        }

        // Check if this is a global or function reference (format: "ptr @name" or "@name")
        if let Some(ref_name) = extract_global_name_from_repr(&repr) {
            // Check global variables first
            if let Some(&existing_id) = self.global_value_ids.get(ref_name) {
                self.ptr_cache.insert(ptr_key, existing_id);
                return existing_id;
            }

            // Check if it's a function reference
            if let Some(&func_id) = self.function_ids.get(ref_name) {
                // Create a ValueId for this function address
                // Use the function's ID as the basis for consistency
                let func_addr_id = ValueId::new(func_id.raw());
                self.ptr_cache.insert(ptr_key, func_addr_id);

                // Record a GlobalRef constant so MTA can resolve this function pointer
                // The target is a synthetic ValueId representing the function's address
                self.constants
                    .entry(func_addr_id)
                    .or_insert(Constant::GlobalRef(func_addr_id));

                return func_addr_id;
            }
        }

        let value_id = ValueId::derive(repr.as_bytes());
        self.ptr_cache.insert(ptr_key, value_id);

        // Record constant value if applicable
        if let std::collections::btree_map::Entry::Vacant(e) = self.constants.entry(value_id) {
            if let Some(constant) = convert_constant_value(value) {
                e.insert(constant);
            }
        }

        value_id
    }

    /// Decompose a constant-expression GEP into a synthetic `Operation::Gep` instruction.
    ///
    /// When the LLVM frontend encounters a constant-expression GEP like
    /// `ptr getelementptr inbounds ([2 x i32], ptr @a, i64 0, i64 1)` as an
    /// operand of another instruction (e.g., a Load), this function synthesizes
    /// an explicit GEP instruction in AIR. This enables PTA to create proper
    /// field-indexed locations instead of treating the entire expression as an
    /// opaque `ValueId`.
    ///
    /// Returns the `ValueId` of the synthetic GEP's result, or `None` if the
    /// expression cannot be decomposed (non-global base, no block context, etc.).
    fn decompose_constant_gep(&mut self, repr: &str) -> Option<ValueId> {
        let (global_name, indices) = parse_constant_gep(repr)?;

        // Look up the global's ValueId as the base pointer
        let base_vid = *self.global_value_ids.get(global_name)?;

        // Need a current block to create instruction IDs
        let block_id = self.current_block_id?;

        // Drop the first index: LLVM GEP's first index is a pointer-level step
        // (advance by `sizeof(pointee)` * idx, almost always 0 for globals),
        // not a type-descent index. SAF's `FieldPath` only carries descent
        // steps — matching `resolve_constant_gep_element`'s behavior below.
        let field_indices: &[i64] = if indices.len() > 1 {
            &indices[1..]
        } else {
            &indices[..]
        };

        // Build FieldPath from parsed indices (all constant → Field steps)
        let steps: Vec<FieldStep> = field_indices
            .iter()
            .filter_map(|&idx| {
                u32::try_from(idx)
                    .ok()
                    .map(|i| FieldStep::Field { index: i })
            })
            .collect();

        if steps.len() != field_indices.len() {
            return None; // Some indices were negative or too large
        }

        let field_path = FieldPath { steps };

        // Create synthetic GEP instruction
        let inst_id = self.create_inst_id(block_id);
        let dst_id = self.create_inst_result_id(inst_id);

        // Build operands: [base_ptr, ...index_constants].
        // Use `field_indices` (not `indices`) so the operand count matches the
        // `FieldPath` step count — PTA's `convert_field_path_with_operands`
        // walks both in lockstep.
        //
        // Derive index ValueIds directly — BLAKE3 is deterministic, so the same
        // "i64 N" string always produces the same ValueId. LLVM uniques constant
        // integers, so get_or_create_value_id will produce matching IDs when it
        // encounters the same constant as an operand elsewhere.
        let mut operands = vec![base_vid];
        for &idx in field_indices {
            let idx_repr = format!("i64 {idx}");
            let idx_vid = ValueId::derive(idx_repr.as_bytes());
            self.constants
                .entry(idx_vid)
                .or_insert(Constant::int(idx, 64));
            operands.push(idx_vid);
        }

        let mut air_inst = Instruction::new(inst_id, Operation::Gep { field_path });
        air_inst.operands = operands;
        air_inst.dst = Some(dst_id);
        air_inst.result_type = Some(self.type_interner.intern_pointer());

        // Pointer cache insertion is handled by the caller (get_or_create_value_id).

        // Also store the resolved constant value (if available) so absint can
        // use it directly without going through PTA → loc_memory → load.
        if let Some(constant) = resolve_constant_gep_element(repr, &self.global_inits) {
            self.constants.entry(dst_id).or_insert(constant);
        }

        self.pending_instructions.push(air_inst);

        Some(dst_id)
    }
}

/// Extract a global/function name from a string containing an `@name` reference.
///
/// Handles formats like:
/// - `ptr @global_name` -> `"global_name"`
/// - `@global_name` -> `"global_name"`
/// - `ptr @"quoted.name"` -> `"quoted.name"`
///
/// This is the shared parsing logic used by both [`extract_global_name_from_repr`]
/// and [`extract_global_ref_name`].
fn extract_at_name(repr: &str) -> Option<&str> {
    // Find the '@' marker
    let at_pos = repr.find('@')?;
    let after_at = &repr[at_pos + 1..];

    // Handle quoted names like @"foo.bar"
    if let Some(stripped) = after_at.strip_prefix('"') {
        let end_quote = stripped.find('"')?;
        return Some(&stripped[..end_quote]);
    }

    // Unquoted name: find the end (space, comma, parenthesis, or end of string)
    let end = after_at
        .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == '(')
        .unwrap_or(after_at.len());

    if end == 0 {
        None
    } else {
        Some(&after_at[..end])
    }
}

/// Extract a global name from a value's string representation.
///
/// Handles formats like:
/// - `ptr @global_name` -> `"global_name"`
/// - `@global_name` -> `"global_name"`
/// - `ptr @"quoted.name"` -> `"quoted.name"`
fn extract_global_name_from_repr(repr: &str) -> Option<&str> {
    extract_at_name(repr)
}

/// Convert an LLVM module to an AIR bundle.
pub fn convert_module(
    module: &Module<'_>,
    module_bytes: &[u8],
    _adapter: &dyn super::adapter::LlvmAdapter,
) -> Result<AirBundle, LlvmError> {
    // Derive module ID from bitcode hash
    let module_id = ModuleId::new(make_id("llvm", module_bytes));
    let mut ctx = MappingContext::new(module_id);

    // Extract local variable names from debug info (pre-pass over module text).
    // This parses DILocalVariable metadata and dbg.declare intrinsics to build
    // a mapping from (function_name, register_name) to C/C++ variable name.
    let module_text = module.print_to_string().to_string();
    ctx.module_local_var_names = extract_local_variable_names(&module_text);

    // First pass: collect all function IDs (skip LLVM intrinsics)
    for func in module.get_functions() {
        let name = func.get_name().to_str().unwrap_or("");
        if !name.starts_with("llvm.") {
            ctx.get_or_create_function_id(func);
        }
    }

    // Collect all global IDs
    for global in module.get_globals() {
        ctx.get_or_create_global_obj_id(global);
    }

    // Convert globals
    let mut air_globals = Vec::new();
    for global in module.get_globals() {
        if let Some(air_global) = convert_global(global, &mut ctx) {
            air_globals.push(air_global);
        }
    }

    // Convert functions (skip LLVM intrinsics — they have metadata parameter
    // types that cause inkwell to panic, and are handled at call sites instead)
    let mut air_functions = Vec::new();
    for func in module.get_functions() {
        let name = func.get_name().to_str().unwrap_or("");
        if name.starts_with("llvm.") {
            continue;
        }
        let air_func = convert_function(func, &mut ctx)?;
        air_functions.push(air_func);
    }

    // Extract type hierarchy (CHA data from _ZTV*/_ZTI* globals)
    let type_hierarchy = super::cha_extract::extract_type_hierarchy(module, &ctx);

    // Build AIR module
    let mut air_module = AirModule::new(module_id);
    air_module.name = module.get_name().to_str().ok().map(String::from);
    air_module.functions = air_functions;
    air_module.globals = air_globals;
    air_module.source_files = ctx.source_files.into_files();
    air_module.type_hierarchy = type_hierarchy;
    air_module.constants = ctx.constants;
    air_module.types = ctx.type_interner.into_table();
    // TODO: Parse pointer size from LLVM module's data layout string.
    // For now, default to 8 (64-bit targets).
    air_module.target_pointer_width = 8;
    air_module.rebuild_function_index();

    Ok(AirBundle::new("llvm", air_module))
}

/// Convert an LLVM global to an AIR global.
fn convert_global(global: GlobalValue<'_>, ctx: &mut MappingContext<'_>) -> Option<AirGlobal> {
    let name = global.get_name().to_str().ok()?.to_string();
    let obj_id = ctx.get_or_create_global_obj_id(global);

    // Create value ID for the global's address
    let value_id = {
        let data = format!("{:032x}global{}", ctx.module_id.raw(), name);
        ValueId::derive(data.as_bytes())
    };

    // Store the mapping so operand references to this global use the same ValueId
    ctx.global_value_ids.insert(name.clone(), value_id);

    let mut air_global = AirGlobal::new(value_id, obj_id, name);
    air_global.is_constant = global.is_constant();

    // Try to convert initializer.
    // For vtable globals (_ZTV*, _ZTC*, _ZTT*), use context-aware conversion
    // that decomposes array elements to capture function pointer references.
    if let Some(init) = global.get_initializer() {
        let is_vtable_global = air_global.name.starts_with("_ZTV")
            || air_global.name.starts_with("_ZTC")
            || air_global.name.starts_with("_ZTT");
        air_global.init = convert_constant_with_context(init, ctx, is_vtable_global);

        // Store initializer for constant-expression GEP resolution (Plan 084 Phase B).
        if let Some(ref init_constant) = air_global.init {
            ctx.global_inits
                .insert(air_global.name.clone(), init_constant.clone());
        }
    }

    Some(air_global)
}

/// Extract the name of a global that a pointer constant references.
///
/// For pointers like `@global_name`, returns `Some("global_name")`.
fn extract_global_ref_name(ptr: inkwell::values::PointerValue<'_>) -> Option<String> {
    let repr = ptr.print_to_string().to_string();
    extract_at_name(&repr).map(String::from)
}

/// Convert an LLVM constant value to an AIR constant, using context for
/// resolving global and function references inside aggregate initializers.
///
/// Unlike [`convert_constant_value`] which only handles scalars, this function
/// properly converts struct aggregate initializers to `Constant::Aggregate`
/// with `GlobalRef` elements for function/global pointer fields.
fn convert_constant_with_context(
    value: BasicValueEnum<'_>,
    ctx: &MappingContext<'_>,
    decompose_arrays: bool,
) -> Option<Constant> {
    match value {
        BasicValueEnum::StructValue(sv) => {
            if sv.is_null() {
                return Some(Constant::ZeroInit);
            }
            let num = sv.count_fields();
            let mut elements = Vec::with_capacity(num as usize);
            for i in 0..num {
                if let Some(field) = sv.get_field_at_index(i) {
                    if let Some(c) = convert_constant_with_context(field, ctx, decompose_arrays) {
                        elements.push(c);
                    } else {
                        elements.push(Constant::Null);
                    }
                } else {
                    elements.push(Constant::Null);
                }
            }
            Some(Constant::Aggregate { elements })
        }
        BasicValueEnum::PointerValue(ptr) => {
            if ptr.is_null() {
                return Some(Constant::Null);
            }
            // Try to resolve as a global or function reference
            if let Some(target_name) = extract_global_ref_name(ptr) {
                // Check global variables first
                if let Some(&target_id) = ctx.global_value_ids.get(&target_name) {
                    return Some(Constant::GlobalRef(target_id));
                }
                // Check function names — create ValueId from FunctionId
                if let Some(&func_id) = ctx.function_ids.get(&target_name) {
                    return Some(Constant::GlobalRef(ValueId::new(func_id.raw())));
                }
            }
            None
        }
        BasicValueEnum::ArrayValue(av) => {
            // For vtable arrays containing function pointers, we need context-aware
            // conversion to resolve function/global references. Only do this for
            // vtable globals to avoid over-modeling non-vtable aggregates.
            let s = av.print_to_string().to_string();
            if decompose_arrays && s.contains('@') {
                // Array contains named references — parse as aggregate with
                // context to capture GlobalRef entries for function pointers.
                let ir_ptrs = super::cha_extract::parse_function_pointers_from_ir_string(&s);
                let mut elements = Vec::with_capacity(ir_ptrs.len());
                for ptr_name in &ir_ptrs {
                    if let Some(name) = ptr_name {
                        // Try function name first, then global name
                        if let Some(&func_id) = ctx.function_ids.get(name.as_str()) {
                            elements.push(Constant::GlobalRef(ValueId::new(func_id.raw())));
                        } else if let Some(&val_id) = ctx.global_value_ids.get(name.as_str()) {
                            elements.push(Constant::GlobalRef(val_id));
                        } else {
                            elements.push(Constant::Null);
                        }
                    } else {
                        elements.push(Constant::Null);
                    }
                }
                Some(Constant::Aggregate { elements })
            } else {
                // Try to decompose integer arrays into Aggregate for global
                // initializers. This enables PTA field-indexed location resolution
                // and absint constant seeding for patterns like `int a[2] = {1, 2}`.
                if let Some(aggregate) = try_parse_integer_array(&s) {
                    Some(aggregate)
                } else {
                    // Fall back to standard conversion (string constants etc.)
                    convert_constant_value(value)
                }
            }
        }
        // For scalar types, delegate to the standard conversion
        _ => convert_constant_value(value),
    }
}

/// Convert an LLVM function to an AIR function.
// INVARIANT: LLVM functions have at most hundreds of parameters in practice;
// parameter indices trivially fit in u32. LLVM integer bit widths (1-128)
// fit in u16.
#[allow(clippy::cast_possible_truncation)]
fn convert_function(
    func: FunctionValue<'_>,
    ctx: &mut MappingContext<'_>,
) -> Result<AirFunction, LlvmError> {
    let func_id = ctx.get_or_create_function_id(func);
    let name = func.get_name().to_str().unwrap_or("").to_string();

    let mut air_func = AirFunction::new(func_id, name.clone());
    // A function is a declaration if it has no basic blocks
    air_func.is_declaration = func.count_basic_blocks() == 0;

    // Extract debug info
    air_func.symbol = extract_function_symbol(func);
    air_func.span = extract_function_span(func, &mut ctx.source_files);

    // Set current function's local variable name map from the module-level pre-pass.
    ctx.current_local_var_names = ctx
        .module_local_var_names
        .get(&name)
        .cloned()
        .unwrap_or_default();

    // Clear per-function value cache. Since local SSA names (e.g., %0,
    // %this.addr) are only unique within a function, we must reset the cache
    // at function boundaries to avoid cross-function ValueId collisions.
    // The ptr_cache uses LLVMValueRef pointers which are also function-local
    // for SSA values (though globals share pointers across functions — those
    // are resolved via global_value_ids/function_ids, not the ptr_cache).
    ctx.ptr_cache.clear();

    // Convert parameters — register each parameter's LLVM value in the
    // ptr_cache so that instruction operands referencing the same
    // parameter resolve to the same ValueId.
    for (idx, param) in func.get_params().iter().enumerate() {
        let param_id = ctx.create_param_id(func_id, idx as u32);
        let ptr_key = param.as_value_ref() as usize;
        ctx.ptr_cache.insert(ptr_key, param_id);
        let mut air_param = AirParam::new(param_id, idx as u32);
        air_param.name = param.get_name().to_str().ok().map(String::from);

        // Populate param type via type interner
        air_param.param_type = Some(match param {
            BasicValueEnum::PointerValue(_) => ctx.type_interner.intern_pointer(),
            BasicValueEnum::IntValue(v) => {
                // INVARIANT: LLVM integer bit widths (1-128) fit in u16.
                let bits = v.get_type().get_bit_width() as u16;
                ctx.type_interner.intern_integer(bits)
            }
            BasicValueEnum::FloatValue(v) => {
                let type_str = v.get_type().print_to_string().to_string();
                if type_str == "float" {
                    ctx.type_interner.intern_float(32)
                } else {
                    ctx.type_interner.intern_float(64)
                }
            }
            _ => ctx.type_interner.intern_opaque(),
        });

        air_func.params.push(air_param);
    }

    // Convert basic blocks
    ctx.reset_block_counter();
    if func.count_basic_blocks() > 0 {
        // Build block ID map first
        let mut block_ids = Vec::new();
        for _ in func.get_basic_blocks() {
            block_ids.push(ctx.create_block_id(func_id));
        }
        ctx.reset_block_counter();

        for (blk_index, block) in func.get_basic_blocks().iter().enumerate() {
            let air_block = convert_block(*block, func_id, blk_index, &block_ids, ctx)?;
            air_func.add_block(air_block);
        }
    }

    Ok(air_func)
}

/// Convert an LLVM basic block to an AIR block.
fn convert_block(
    block: BasicBlock<'_>,
    func_id: FunctionId,
    blk_index: usize,
    block_ids: &[BlockId],
    ctx: &mut MappingContext<'_>,
) -> Result<AirBlock, LlvmError> {
    let block_id = block_ids[blk_index];
    ctx.current_block_id = Some(block_id);

    let mut air_block = AirBlock::new(block_id);
    air_block.label = block.get_name().to_str().ok().map(String::from);

    // Convert instructions
    ctx.reset_inst_counter();
    let mut current_inst = block.get_first_instruction();

    while let Some(inst) = current_inst {
        if let Some(air_inst) = convert_instruction(inst, block_id, block_ids, func_id, ctx)? {
            // Drain synthetic instructions (e.g., decomposed constant-expression GEPs)
            // generated during operand collection. These must appear before the
            // instruction that references their results.
            air_block.instructions.append(&mut ctx.pending_instructions);
            air_block.instructions.push(air_inst);
        }
        current_inst = inst.get_next_instruction();
    }

    Ok(air_block)
}

/// Convert an LLVM instruction to an AIR instruction.
// NOTE: This function is intentionally long because it exhaustively maps all
// LLVM instruction opcodes to their AIR `Operation` equivalents. Splitting by
// opcode category would scatter the mapping logic and make maintenance harder.
#[allow(clippy::too_many_lines)]
fn convert_instruction(
    inst: InstructionValue<'_>,
    block_id: BlockId,
    block_ids: &[BlockId],
    func_id: FunctionId,
    ctx: &mut MappingContext<'_>,
) -> Result<Option<Instruction>, LlvmError> {
    let inst_id = ctx.create_inst_id(block_id);
    let opcode = inst.get_opcode();

    // Check for intrinsic calls first.
    // We must detect llvm.* intrinsics from the instruction string BEFORE
    // calling get_operand(), because metadata operands (e.g. in llvm.dbg.declare)
    // cause inkwell to panic with "The given type is not a basic type".
    if opcode == InstructionOpcode::Call {
        let inst_str = inst.print_to_string().to_string();
        if let Some(intrinsic_name) = extract_llvm_intrinsic_from_str(&inst_str) {
            return convert_intrinsic_call(inst, inst_id, &intrinsic_name, ctx);
        }
        // Skip inline assembly — not a function pointer call.
        // Inline asm has no function name, so it would otherwise become `CallIndirect`.
        if inst_str.contains(" asm ") {
            return Ok(None);
        }
    }

    let (op, operands, has_result) = match opcode {
        InstructionOpcode::Alloca => {
            let size_bytes = extract_alloca_size_bytes(inst);
            (Operation::Alloca { size_bytes }, vec![], true)
        }

        InstructionOpcode::Load => {
            let operands = collect_operands(inst, ctx);
            (Operation::Load, operands, true)
        }

        InstructionOpcode::Store => {
            let operands = collect_operands(inst, ctx);
            (Operation::Store, operands, false)
        }

        InstructionOpcode::GetElementPtr => {
            let operands = collect_operands(inst, ctx);
            let field_path = extract_gep_field_path(inst);
            (Operation::Gep { field_path }, operands, true)
        }

        InstructionOpcode::Call => convert_call_instruction(inst, func_id, ctx)?,

        InstructionOpcode::Invoke => {
            return convert_invoke_instruction(inst, inst_id, block_id, block_ids, func_id, ctx);
        }

        InstructionOpcode::Return => {
            let operands = collect_operands(inst, ctx);
            (Operation::Ret, operands, false)
        }

        InstructionOpcode::Br => convert_branch_instruction(inst, block_ids, ctx)?,

        InstructionOpcode::Switch => convert_switch_instruction(inst, block_ids, ctx)?,

        InstructionOpcode::Phi => convert_phi_instruction(inst, block_ids, ctx)?,

        InstructionOpcode::Select => {
            let operands = collect_operands(inst, ctx);
            (Operation::Select, operands, true)
        }

        InstructionOpcode::ICmp => {
            let kind = convert_icmp_predicate(inst);
            let operands = collect_operands(inst, ctx);
            (Operation::BinaryOp { kind }, operands, true)
        }

        InstructionOpcode::FCmp => {
            let kind = convert_fcmp_predicate(inst);
            let operands = collect_operands(inst, ctx);
            (Operation::BinaryOp { kind }, operands, true)
        }

        // Binary operations
        InstructionOpcode::Add => binary_op(BinaryOp::Add, inst, ctx),
        InstructionOpcode::Sub => binary_op(BinaryOp::Sub, inst, ctx),
        InstructionOpcode::Mul => binary_op(BinaryOp::Mul, inst, ctx),
        InstructionOpcode::UDiv => binary_op(BinaryOp::UDiv, inst, ctx),
        InstructionOpcode::SDiv => binary_op(BinaryOp::SDiv, inst, ctx),
        InstructionOpcode::URem => binary_op(BinaryOp::URem, inst, ctx),
        InstructionOpcode::SRem => binary_op(BinaryOp::SRem, inst, ctx),
        InstructionOpcode::FAdd => binary_op(BinaryOp::FAdd, inst, ctx),
        InstructionOpcode::FSub => binary_op(BinaryOp::FSub, inst, ctx),
        InstructionOpcode::FMul => binary_op(BinaryOp::FMul, inst, ctx),
        InstructionOpcode::FDiv => binary_op(BinaryOp::FDiv, inst, ctx),
        InstructionOpcode::FRem => binary_op(BinaryOp::FRem, inst, ctx),
        InstructionOpcode::And => binary_op(BinaryOp::And, inst, ctx),
        InstructionOpcode::Or => binary_op(BinaryOp::Or, inst, ctx),
        InstructionOpcode::Xor => binary_op(BinaryOp::Xor, inst, ctx),
        InstructionOpcode::Shl => binary_op(BinaryOp::Shl, inst, ctx),
        InstructionOpcode::LShr => binary_op(BinaryOp::LShr, inst, ctx),
        InstructionOpcode::AShr => binary_op(BinaryOp::AShr, inst, ctx),

        // Cast operations
        InstructionOpcode::Trunc => cast_op(CastKind::Trunc, inst, ctx),
        InstructionOpcode::ZExt => cast_op(CastKind::ZExt, inst, ctx),
        InstructionOpcode::SExt => cast_op(CastKind::SExt, inst, ctx),
        InstructionOpcode::FPToUI => cast_op(CastKind::FPToUI, inst, ctx),
        InstructionOpcode::FPToSI => cast_op(CastKind::FPToSI, inst, ctx),
        InstructionOpcode::UIToFP => cast_op(CastKind::UIToFP, inst, ctx),
        InstructionOpcode::SIToFP => cast_op(CastKind::SIToFP, inst, ctx),
        InstructionOpcode::FPTrunc => cast_op(CastKind::FPTrunc, inst, ctx),
        InstructionOpcode::FPExt => cast_op(CastKind::FPExt, inst, ctx),
        InstructionOpcode::PtrToInt => cast_op(CastKind::PtrToInt, inst, ctx),
        InstructionOpcode::IntToPtr => cast_op(CastKind::IntToPtr, inst, ctx),
        InstructionOpcode::BitCast => cast_op(CastKind::Bitcast, inst, ctx),
        InstructionOpcode::AddrSpaceCast => cast_op(CastKind::AddrSpaceCast, inst, ctx),

        InstructionOpcode::Unreachable => (Operation::Unreachable, vec![], false),

        InstructionOpcode::Freeze => {
            let operands = collect_operands(inst, ctx);
            (Operation::Freeze, operands, true)
        }

        // Model `extractvalue` as a `Copy` from the aggregate operand to the result.
        //
        // Model `extractvalue` and `insertvalue` as `Copy` operations.
        //
        // `extractvalue` extracts a field from an aggregate — the result may carry
        // pointers from the aggregate. `insertvalue` produces a new aggregate with
        // a value inserted — the result may carry pointers from both the original
        // aggregate and the inserted value. Modeling both as `Copy` conservatively
        // propagates all operand points-to sets to the result.
        InstructionOpcode::ExtractValue | InstructionOpcode::InsertValue => {
            let operands = collect_operands(inst, ctx);
            (Operation::Copy, operands, true)
        }

        // Fence has no data-flow semantics — safe to skip.
        InstructionOpcode::Fence => {
            return Ok(None);
        }

        // Model `AtomicCmpXchg` as a Load + Store pair.
        //
        // LLVM semantics: `cmpxchg ptr %addr, T %expected, T %new`
        //   → atomically loads old value from `%addr`, compares with `%expected`,
        //     and stores `%new` on match.  Returns `{ T, i1 }`.
        //
        // Conservative over-approximation: always assume the swap succeeds,
        // so we emit Store(%new → *%addr) + Load(dst ← *%addr).
        // The result `ValueId` represents the loaded old value; downstream
        // `extractvalue` instructions will propagate it via `Copy`.
        //
        // Operand layout: 0 = ptr, 1 = expected, 2 = new_val
        InstructionOpcode::AtomicCmpXchg => {
            return convert_atomic_cmpxchg(inst, inst_id, block_id, ctx);
        }

        // Model `AtomicRMW` as a Load + Store pair.
        //
        // LLVM semantics: `atomicrmw op ptr %addr, T %val`
        //   → atomically loads old value from `%addr`, applies `op`, stores result.
        //     Returns the **old** value (T).
        //
        // For pointer analysis every `atomicrmw` is modeled as
        // Store(%val → *%addr) + Load(dst ← *%addr), regardless of the
        // specific RMW operation.  Non-pointer-typed atomics (add, sub, …)
        // will simply produce integer-typed Load/Store pairs that PTA
        // ignores, so there is no soundness cost.
        //
        // Operand layout: 0 = ptr, 1 = val
        InstructionOpcode::AtomicRMW => {
            return convert_atomic_rmw(inst, inst_id, block_id, ctx);
        }

        // Catch-all for unsupported instructions
        _ => {
            tracing::warn!("Unsupported LLVM instruction: {:?}", opcode);
            return Ok(None);
        }
    };

    let mut air_inst = Instruction::new(inst_id, op);
    air_inst.operands = operands;

    if has_result {
        // Check if this instruction was already forward-referenced (e.g., by a phi
        // in an earlier block that references an instruction result defined in
        // a later block). Pointer identity works because phi's get_or_create_value_id
        // uses the same LLVMValueRef for the forward-referenced instruction.
        let ptr_key = inst.as_value_ref() as usize;
        let dst_id = if let Some(&existing_id) = ctx.ptr_cache.get(&ptr_key) {
            existing_id
        } else {
            let id = ctx.create_inst_result_id(inst_id);
            ctx.ptr_cache.insert(ptr_key, id);
            id
        };
        air_inst.dst = Some(dst_id);

        // Populate result type from LLVM type.
        // For operations that always produce pointers, use the pointer type
        // directly to avoid redundant string parsing.
        let type_str = inst.get_type().print_to_string().to_string();
        match air_inst.op {
            Operation::Alloca { .. } | Operation::HeapAlloc { .. } | Operation::Gep { .. } => {
                air_inst.result_type = Some(ctx.type_interner.intern_pointer());
            }
            _ => {
                air_inst.result_type = Some(ctx.type_interner.parse_llvm_type_string(&type_str));
            }
        }
    }

    // Extract debug info
    air_inst.span = extract_span(inst, &mut ctx.source_files);

    // Populate symbol for alloca instructions from debug info.
    // The pre-pass over llvm.dbg.declare intrinsics has already built a mapping
    // from LLVM register names to C/C++ variable names. Look up the alloca's
    // register name (e.g., "%2") in that map to set the symbol.
    if matches!(air_inst.op, Operation::Alloca { .. }) && !ctx.current_local_var_names.is_empty() {
        let inst_str = inst.print_to_string().to_string();
        if let Some(reg_name) = extract_register_name(&inst_str) {
            if let Some(var_name) = ctx.current_local_var_names.get(&reg_name) {
                air_inst.symbol = Some(saf_core::span::Symbol::simple(var_name.clone()));
            }
        }
    }

    Ok(Some(air_inst))
}

/// Extract an `llvm.*` intrinsic name from an instruction's string representation.
///
/// Parses strings like `call void @llvm.dbg.declare(metadata ...)` to extract
/// `llvm.dbg.declare`. This avoids calling `get_operand()` on instructions with
/// metadata operands, which causes inkwell to panic with
/// "The given type is not a basic type".
fn extract_llvm_intrinsic_from_str(inst_str: &str) -> Option<String> {
    let marker = "@llvm.";
    let start = inst_str.find(marker)?;
    let name_start = start + 1; // Skip the '@'
    let rest = &inst_str[name_start..];
    let end = rest.find('(')?;
    Some(rest[..end].to_string())
}

/// Get the called function name for a call/invoke instruction.
/// Returns the function name if it's a direct call to a known function, `None` otherwise.
///
/// Only returns `Some` when the callee operand's name matches a function in
/// `known_functions` (populated from the module's defined + external functions)
/// or a recognized heap allocator. Local SSA values like `%call5` also carry
/// non-empty names but are **not** functions — without this check they would be
/// misclassified as direct calls to phantom external functions.
fn get_called_function_name(
    inst: InstructionValue<'_>,
    known_functions: &BTreeMap<String, FunctionId>,
) -> Option<String> {
    let num_operands = inst.get_num_operands();
    if num_operands == 0 {
        return None;
    }

    // The called operand is the last operand for call/invoke instructions
    let called_operand = inst.get_operand(num_operands - 1)?;
    if let Some(BasicValueEnum::PointerValue(ptr)) = operand_as_value(called_operand) {
        let name = ptr.get_name().to_str().ok()?;
        if !name.is_empty() {
            // Only treat as a direct call if the name matches a known function
            // (defined in module or previously seen as external). Local SSA values
            // like %call5 have non-empty names but are NOT functions.
            if known_functions.contains_key(name) || is_heap_alloc_function(name).is_some() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Convert an intrinsic call.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_intrinsic_call(
    inst: InstructionValue<'_>,
    inst_id: InstId,
    intrinsic_name: &str,
    ctx: &mut MappingContext<'_>,
) -> Result<Option<Instruction>, LlvmError> {
    match classify_intrinsic(intrinsic_name) {
        IntrinsicMapping::Skip => Ok(None),

        IntrinsicMapping::MapTo(intrinsic_op) => {
            let operands = collect_operands(inst, ctx);
            let mut air_inst = Instruction::new(inst_id, intrinsic_op.to_operation());
            air_inst.operands = operands;
            Ok(Some(air_inst))
        }

        IntrinsicMapping::PassThrough => {
            let operands = collect_operands(inst, ctx);
            let mut air_inst = Instruction::new(inst_id, Operation::Copy);
            air_inst.operands = operands;
            let dst_id = ctx.create_inst_result_id(inst_id);
            air_inst.dst = Some(dst_id);
            let ptr_key = inst.as_value_ref() as usize;
            ctx.ptr_cache.insert(ptr_key, dst_id);
            let type_str = inst.get_type().print_to_string().to_string();
            air_inst.result_type = Some(ctx.type_interner.parse_llvm_type_string(&type_str));
            Ok(Some(air_inst))
        }

        IntrinsicMapping::External => {
            // Unmatched intrinsics (arithmetic, float, trap, etc.) are direct
            // calls to compiler builtins — emit `CallDirect` to an external
            // function, matching SVF's `handleExtCall()` behavior. The callee
            // has no body, so PTA adds no constraints. This avoids the old
            // `CallIndirect` mapping that created phantom `IndCallSites`.
            let callee_id = ctx.get_or_create_external_function_id(intrinsic_name);
            let operands = collect_operands(inst, ctx);
            let mut air_inst =
                Instruction::new(inst_id, Operation::CallDirect { callee: callee_id });
            air_inst.operands = operands;
            let dst_id = ctx.create_inst_result_id(inst_id);
            air_inst.dst = Some(dst_id);
            let ptr_key = inst.as_value_ref() as usize;
            ctx.ptr_cache.insert(ptr_key, dst_id);
            let type_str = inst.get_type().print_to_string().to_string();
            air_inst.result_type = Some(ctx.type_interner.parse_llvm_type_string(&type_str));
            Ok(Some(air_inst))
        }
    }
}

/// Known heap allocation functions.
/// These are recognized by name and converted to `HeapAlloc` operations.
const HEAP_ALLOC_FUNCTIONS: &[(&str, &str)] = &[
    // C standard library
    ("malloc", "malloc"),
    ("calloc", "calloc"),
    ("realloc", "realloc"),
    ("aligned_alloc", "aligned_alloc"),
    ("strdup", "strdup"),
    ("strndup", "strndup"),
    // POSIX
    ("mmap", "mmap"),
    ("valloc", "valloc"),
    ("pvalloc", "pvalloc"),
    ("memalign", "memalign"),
    ("posix_memalign", "posix_memalign"),
    // C++ operators (mangled names)
    ("_Znwm", "operator_new"),
    ("_Znam", "operator_new_array"),
    ("_ZnwmRKSt9nothrow_t", "operator_new_nothrow"),
    ("_ZnamRKSt9nothrow_t", "operator_new_array_nothrow"),
    // GLib
    ("g_malloc", "g_malloc"),
    ("g_malloc0", "g_malloc0"),
    ("g_new", "g_new"),
    ("g_new0", "g_new0"),
    ("g_realloc", "g_realloc"),
    ("g_strdup", "g_strdup"),
    // GNU/common wrappers
    ("xmalloc", "xmalloc"),
    ("xcalloc", "xcalloc"),
    ("xrealloc", "xrealloc"),
    ("xstrdup", "xstrdup"),
];

/// Check if a function name is a known heap allocator.
fn is_heap_alloc_function(name: &str) -> Option<&'static str> {
    for &(func_name, kind) in HEAP_ALLOC_FUNCTIONS {
        if name == func_name {
            return Some(kind);
        }
    }
    None
}

/// Convert a call instruction.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_call_instruction(
    inst: InstructionValue<'_>,
    _func_id: FunctionId,
    ctx: &mut MappingContext<'_>,
) -> Result<(Operation, Vec<ValueId>, bool), LlvmError> {
    // Try to get the called function name for direct calls
    if let Some(callee_name) = get_called_function_name(inst, &ctx.function_ids) {
        // Check if this is a heap allocation function
        if let Some(kind) = is_heap_alloc_function(&callee_name) {
            let operands = collect_call_args(inst, ctx);
            return Ok((
                Operation::HeapAlloc {
                    kind: HeapAllocKind::from(kind),
                },
                operands,
                true,
            ));
        }

        // Check if the function is defined in the module
        if let Some(callee_id) = ctx.get_function_id_by_name(&callee_name) {
            let operands = collect_call_args(inst, ctx);
            return Ok((Operation::CallDirect { callee: callee_id }, operands, true));
        }

        // External function call - create a FunctionId for it so we can track it
        // This allows the analysis to see external calls as CallDirect
        let callee_id = ctx.get_or_create_external_function_id(&callee_name);
        let operands = collect_call_args(inst, ctx);
        return Ok((Operation::CallDirect { callee: callee_id }, operands, true));
    }

    // Indirect call
    let operands = collect_operands(inst, ctx);
    Ok((
        Operation::CallIndirect {
            expected_signature: None,
        },
        operands,
        true,
    ))
}

/// Convert an `invoke` instruction to a call + `CondBr` pair.
///
/// LLVM `invoke` has call semantics plus two successor blocks:
///   - Normal destination: where execution continues if no exception.
///   - Unwind destination: landing pad for exception handling.
///
/// We emit the call as a pending instruction (so it appears before the
/// terminator) and return a `CondBr` that targets both destinations.
/// This ensures the CFG builder picks up edges to both successors.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_invoke_instruction(
    inst: InstructionValue<'_>,
    inst_id: InstId,
    block_id: BlockId,
    block_ids: &[BlockId],
    func_id: FunctionId,
    ctx: &mut MappingContext<'_>,
) -> Result<Option<Instruction>, LlvmError> {
    // Build the call operation (same logic as convert_call_instruction)
    let (call_op, call_operands, _has_result) = convert_call_instruction(inst, func_id, ctx)?;

    let mut call_inst = Instruction::new(inst_id, call_op);
    call_inst.operands = call_operands;

    // Register the result ValueId for the call (invoke always produces a result
    // that downstream instructions can reference).
    let ptr_key = inst.as_value_ref() as usize;
    let dst_id = if let Some(&existing_id) = ctx.ptr_cache.get(&ptr_key) {
        existing_id
    } else {
        let id = ctx.create_inst_result_id(inst_id);
        ctx.ptr_cache.insert(ptr_key, id);
        id
    };
    call_inst.dst = Some(dst_id);

    // Populate result type from LLVM type.
    let type_str = inst.get_type().print_to_string().to_string();
    call_inst.result_type = Some(ctx.type_interner.parse_llvm_type_string(&type_str));

    call_inst.span = extract_span(inst, &mut ctx.source_files);

    // Push the call as a pending instruction (appears before the terminator).
    ctx.pending_instructions.push(call_inst);

    // Extract normal and unwind destination blocks from the invoke operands.
    // In LLVM, invoke block operands are accessible via get_operand() — scan
    // all operands for BasicBlock targets.
    let num_operands = inst.get_num_operands();
    let mut block_targets: Vec<BasicBlock<'_>> = Vec::new();
    for i in 0..num_operands {
        if let Some(bb) = inst.get_operand(i).and_then(operand_as_block) {
            block_targets.push(bb);
        }
    }

    // Create a CondBr to both normal and unwind destinations.
    // LLVM invoke operand order: normal dest first, then unwind dest.
    let br_inst_id = ctx.create_inst_id(block_id);
    let br_op = if block_targets.len() >= 2 {
        let normal_id = resolve_block_id(inst, block_targets[0], block_ids);
        let unwind_id = resolve_block_id(inst, block_targets[1], block_ids);
        Operation::CondBr {
            then_target: normal_id,
            else_target: unwind_id,
        }
    } else if block_targets.len() == 1 {
        // Defensive: only normal destination found (shouldn't happen for valid invoke)
        let normal_id = resolve_block_id(inst, block_targets[0], block_ids);
        Operation::Br { target: normal_id }
    } else {
        tracing::warn!("invoke instruction has no block operands — missing CFG edges");
        Operation::Unreachable
    };

    let br_inst = Instruction::new(br_inst_id, br_op);
    Ok(Some(br_inst))
}

/// Convert a branch instruction.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_branch_instruction(
    inst: InstructionValue<'_>,
    block_ids: &[BlockId],
    ctx: &mut MappingContext<'_>,
) -> Result<(Operation, Vec<ValueId>, bool), LlvmError> {
    let num_operands = inst.get_num_operands();

    if num_operands == 1 {
        // Unconditional branch - operand is the target block
        if let Some(target_block) = inst.get_operand(0).and_then(operand_as_block) {
            let target_id = resolve_block_id(inst, target_block, block_ids);
            return Ok((Operation::Br { target: target_id }, vec![], false));
        }
    } else if num_operands == 3 {
        // Conditional branch
        let condition = inst
            .get_operand(0)
            .and_then(operand_as_value)
            .map(|v| ctx.get_or_create_value_id(v));

        // Note: in LLVM IR, operand 1 is false branch, operand 2 is true branch
        let else_block = inst.get_operand(1).and_then(operand_as_block);
        let then_block = inst.get_operand(2).and_then(operand_as_block);

        if let (Some(then_bb), Some(else_bb)) = (then_block, else_block) {
            let then_target = resolve_block_id(inst, then_bb, block_ids);
            let else_target = resolve_block_id(inst, else_bb, block_ids);

            return Ok((
                Operation::CondBr {
                    then_target,
                    else_target,
                },
                condition.into_iter().collect(),
                false,
            ));
        }
    }

    // Fallback
    Ok((
        Operation::Br {
            target: BlockId::new(0),
        },
        vec![],
        false,
    ))
}

/// Find the index of a basic block within its parent function.
///
/// Compares blocks by pointer equality, not by name.
/// This is critical because unnamed blocks have empty names,
/// and comparing by name would cause all unnamed blocks to match the first one.
///
/// Returns `None` if the block is not found in the parent function.
fn find_block_index(inst: InstructionValue<'_>, target: BasicBlock<'_>) -> Option<usize> {
    let parent_block = inst.get_parent()?;
    let func = parent_block.get_parent()?;
    for (idx, block) in func.get_basic_blocks().iter().enumerate() {
        // Use pointer equality, not name comparison
        if *block == target {
            return Some(idx);
        }
    }
    None
}

/// Resolve a basic block to its `BlockId`, returning a sentinel `BlockId` on failure.
///
/// Logs a warning via `tracing::warn!` if the block index cannot be resolved.
fn resolve_block_id(
    inst: InstructionValue<'_>,
    target: BasicBlock<'_>,
    block_ids: &[BlockId],
) -> BlockId {
    find_block_index(inst, target)
        .and_then(|idx| block_ids.get(idx).copied())
        .unwrap_or_else(|| {
            tracing::warn!("could not resolve block index for branch target");
            BlockId::new(0)
        })
}

/// Convert a switch instruction.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_switch_instruction(
    inst: InstructionValue<'_>,
    block_ids: &[BlockId],
    ctx: &mut MappingContext<'_>,
) -> Result<(Operation, Vec<ValueId>, bool), LlvmError> {
    let condition = inst
        .get_operand(0)
        .and_then(operand_as_value)
        .map(|v| ctx.get_or_create_value_id(v));

    let default_block = inst.get_operand(1).and_then(operand_as_block);
    let default = default_block.map_or_else(
        || {
            tracing::warn!("switch instruction missing default block");
            BlockId::new(0)
        },
        |bb| resolve_block_id(inst, bb, block_ids),
    );

    // Collect cases (value, block pairs)
    let mut cases = Vec::new();
    let num_operands = inst.get_num_operands();

    // Cases start at operand 2, in pairs (value, block)
    let mut i = 2;
    while i + 1 < num_operands {
        let case_val = inst.get_operand(i).and_then(operand_as_value);
        let case_block = inst.get_operand(i + 1).and_then(operand_as_block);

        if let (Some(val), Some(bb)) = (case_val, case_block) {
            // Try to extract the constant value
            if val.is_int_value() {
                if let Some(int_val) = val.into_int_value().get_sign_extended_constant() {
                    let block_id = resolve_block_id(inst, bb, block_ids);
                    cases.push((int_val, block_id));
                }
            }
        }
        i += 2;
    }

    Ok((
        Operation::Switch { default, cases },
        condition.into_iter().collect(),
        false,
    ))
}

/// Convert a phi instruction.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_phi_instruction(
    inst: InstructionValue<'_>,
    block_ids: &[BlockId],
    ctx: &mut MappingContext<'_>,
) -> Result<(Operation, Vec<ValueId>, bool), LlvmError> {
    let mut incoming = Vec::new();

    // Convert InstructionValue to PhiValue for proper PHI node API access
    // PHI nodes in LLVM have a special structure that isn't accessible via get_operand()
    if let Ok(phi) = PhiValue::try_from(inst) {
        let count = phi.count_incoming();
        for i in 0..count {
            if let Some((value, bb)) = phi.get_incoming(i) {
                let value_id = ctx.get_or_create_value_id(value);
                let block_id = resolve_block_id(inst, bb, block_ids);
                incoming.push((block_id, value_id));
            }
        }
    }

    Ok((Operation::Phi { incoming }, vec![], true))
}

/// Convert an `AtomicCmpXchg` instruction to a `Load` + `Store` pair.
///
/// LLVM operands: `cmpxchg ptr %addr, T %expected, T %new_val`
///   - operand 0 = pointer address
///   - operand 1 = expected (compare) value
///   - operand 2 = new value to store
///
/// We emit a synthetic `Store`(`%new_val` -> `*%addr`) as a pending instruction,
/// then return a `Load`(`dst` <- `*%addr`) as the main instruction. The `Load`
/// result becomes the instruction's `dst`, representing the old value read from
/// memory.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_atomic_cmpxchg(
    inst: InstructionValue<'_>,
    inst_id: InstId,
    block_id: BlockId,
    ctx: &mut MappingContext<'_>,
) -> Result<Option<Instruction>, LlvmError> {
    let operands = collect_operands(inst, ctx);

    // Need at least ptr + expected + new_val
    if operands.len() < 3 {
        tracing::warn!("AtomicCmpXchg with fewer than 3 operands — skipping");
        return Ok(None);
    }

    let ptr_id = operands[0];
    let new_val_id = operands[2];

    // Emit synthetic Store(new_val → *ptr)
    let store_inst_id = ctx.create_inst_id(block_id);
    let mut store_inst = Instruction::new(store_inst_id, Operation::Store);
    store_inst.operands = vec![new_val_id, ptr_id];
    store_inst.span = extract_span(inst, &mut ctx.source_files);
    ctx.pending_instructions.push(store_inst);

    // Return Load(dst ← *ptr) as the main instruction
    let mut load_inst = Instruction::new(inst_id, Operation::Load);
    load_inst.operands = vec![ptr_id];

    // Register the result ValueId using pointer identity so that downstream
    // instructions (e.g., `extractvalue`) referencing this SSA value resolve
    // to the same `ValueId`.
    let ptr_key = inst.as_value_ref() as usize;
    let dst_id = if let Some(&existing_id) = ctx.ptr_cache.get(&ptr_key) {
        existing_id
    } else {
        let id = ctx.create_inst_result_id(inst_id);
        ctx.ptr_cache.insert(ptr_key, id);
        id
    };
    load_inst.dst = Some(dst_id);

    // `cmpxchg` returns `{ T, i1 }` but for PTA we treat the result as `T`
    // (the loaded old value). Use the LLVM type string for result_type.
    let type_str = inst.get_type().print_to_string().to_string();
    load_inst.result_type = Some(ctx.type_interner.parse_llvm_type_string(&type_str));

    load_inst.span = extract_span(inst, &mut ctx.source_files);

    Ok(Some(load_inst))
}

/// Convert an `AtomicRMW` instruction to a `Load` + `Store` pair.
///
/// LLVM operands: `atomicrmw op ptr %addr, T %val`
///   - operand 0 = pointer address
///   - operand 1 = value
///
/// We emit a synthetic `Store`(`%val` -> `*%addr`) as a pending instruction,
/// then return a `Load`(`dst` <- `*%addr`) as the main instruction. The `Load`
/// result represents the old value that was at the memory location before the
/// RMW.
#[allow(clippy::unnecessary_wraps)] // Keep consistent Result return type with other converters
fn convert_atomic_rmw(
    inst: InstructionValue<'_>,
    inst_id: InstId,
    block_id: BlockId,
    ctx: &mut MappingContext<'_>,
) -> Result<Option<Instruction>, LlvmError> {
    let operands = collect_operands(inst, ctx);

    // Need at least ptr + val
    if operands.len() < 2 {
        tracing::warn!("AtomicRMW with fewer than 2 operands — skipping");
        return Ok(None);
    }

    let ptr_id = operands[0];
    let val_id = operands[1];

    // Emit synthetic Store(val → *ptr)
    let store_inst_id = ctx.create_inst_id(block_id);
    let mut store_inst = Instruction::new(store_inst_id, Operation::Store);
    store_inst.operands = vec![val_id, ptr_id];
    store_inst.span = extract_span(inst, &mut ctx.source_files);
    ctx.pending_instructions.push(store_inst);

    // Return Load(dst ← *ptr) as the main instruction
    let mut load_inst = Instruction::new(inst_id, Operation::Load);
    load_inst.operands = vec![ptr_id];

    // Register the result ValueId
    let ptr_key = inst.as_value_ref() as usize;
    let dst_id = if let Some(&existing_id) = ctx.ptr_cache.get(&ptr_key) {
        existing_id
    } else {
        let id = ctx.create_inst_result_id(inst_id);
        ctx.ptr_cache.insert(ptr_key, id);
        id
    };
    load_inst.dst = Some(dst_id);

    let type_str = inst.get_type().print_to_string().to_string();
    load_inst.result_type = Some(ctx.type_interner.parse_llvm_type_string(&type_str));

    load_inst.span = extract_span(inst, &mut ctx.source_files);

    Ok(Some(load_inst))
}

/// Helper for binary operations.
fn binary_op(
    kind: BinaryOp,
    inst: InstructionValue<'_>,
    ctx: &mut MappingContext<'_>,
) -> (Operation, Vec<ValueId>, bool) {
    let operands = collect_operands(inst, ctx);
    (Operation::BinaryOp { kind }, operands, true)
}

/// Helper for cast operations.
///
/// Extracts the target type bit-width from the LLVM instruction string
/// representation (e.g., `trunc i64 %x to i8` → `target_bits = Some(8)`).
// INVARIANT: LLVM integer bit widths (1-128) fit in u8.
#[allow(clippy::cast_possible_truncation)]
fn cast_op(
    kind: CastKind,
    inst: InstructionValue<'_>,
    ctx: &mut MappingContext<'_>,
) -> (Operation, Vec<ValueId>, bool) {
    let operands = collect_operands(inst, ctx);
    let target_bits = extract_cast_target_bits(inst);
    (Operation::Cast { kind, target_bits }, operands, true)
}

/// Extract the target type bit-width from a cast instruction.
///
/// Parses the LLVM IR text (e.g., `%5 = trunc i64 %4 to i8`) to find the
/// target integer type after "to". Returns `None` for non-integer targets
/// (float, pointer, etc.).
// INVARIANT: LLVM integer bit widths (1-128) fit in u8.
#[allow(clippy::cast_possible_truncation)]
fn extract_cast_target_bits(inst: InstructionValue<'_>) -> Option<u8> {
    let s = inst.print_to_string().to_string();
    // Find " to i<N>" pattern
    let to_idx = s.rfind(" to ")?;
    let after_to = s[to_idx + 4..].trim();
    if let Some(rest) = after_to.strip_prefix('i') {
        // Parse the integer bit width (stop at non-digit or end)
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse::<u32>().ok().map(|w| w as u8)
    } else {
        None
    }
}

/// Collect operands from an instruction.
fn collect_operands(inst: InstructionValue<'_>, ctx: &mut MappingContext<'_>) -> Vec<ValueId> {
    let mut operands = Vec::new();
    let num = inst.get_num_operands();

    for i in 0..num {
        if let Some(value) = inst.get_operand(i).and_then(operand_as_value) {
            operands.push(ctx.get_or_create_value_id(value));
        }
    }

    operands
}

/// Collect call arguments (excludes the callee operand).
fn collect_call_args(inst: InstructionValue<'_>, ctx: &mut MappingContext<'_>) -> Vec<ValueId> {
    let mut operands = Vec::new();
    let num = inst.get_num_operands();

    // Last operand is the callee, skip it
    for i in 0..num.saturating_sub(1) {
        if let Some(value) = inst.get_operand(i).and_then(operand_as_value) {
            operands.push(ctx.get_or_create_value_id(value));
        }
    }

    operands
}

/// Extract the field path from a GEP instruction.
// INVARIANT: GEP indices from LLVM are i64; struct field indices fit in u32
// (compilers limit struct fields far below 2^32). Sign loss is safe because
// negative struct indices are invalid LLVM IR.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn extract_gep_field_path(inst: InstructionValue<'_>) -> FieldPath {
    let mut steps = Vec::new();
    let num_operands = inst.get_num_operands();

    // Skip first operand (base pointer) and iterate indices
    for i in 1..num_operands {
        if let Some(value) = inst.get_operand(i).and_then(operand_as_value) {
            if value.is_int_value() {
                if let Some(int_val) = value.into_int_value().get_sign_extended_constant() {
                    // Constant index - it's a field index
                    steps.push(FieldStep::Field {
                        index: int_val as u32,
                    });
                } else {
                    // Dynamic index
                    steps.push(FieldStep::Index);
                }
            } else {
                steps.push(FieldStep::Index);
            }
        }
    }

    FieldPath { steps }
}

/// Extract the total byte size of an LLVM `alloca` instruction.
///
/// Handles these patterns from the LLVM IR string representation:
/// - `alloca i8, i64 50`       → 50 bytes (element i8=1 byte, count=50)
/// - `alloca [100 x i8]`       → 100 bytes (array of 100 i8)
/// - `alloca [100 x i64]`      → 800 bytes (array of 100 i64=8 bytes)
/// - `alloca i32`              → 4 bytes
/// - `alloca ptr`              → 8 bytes (on 64-bit targets)
/// - `alloca [10 x i32]`      → 40 bytes
///
/// Returns `None` for variable-size allocas or unrecognized patterns.
// INVARIANT: Alloca sizes from LLVM IR fit in u64 for any reasonable program.
#[allow(clippy::cast_possible_truncation)]
fn extract_alloca_size_bytes(inst: InstructionValue<'_>) -> Option<u64> {
    let s = inst.print_to_string().to_string();

    // Find the alloca type portion: everything after "alloca " and before the first ","
    // or before ", align" / end of string.
    // Format: "%N = alloca TYPE[, iNN COUNT][, align N][, !dbg ...]"
    let alloca_pos = s.find("alloca ")?;
    let after_alloca = &s[alloca_pos + 7..]; // skip "alloca "

    // Split off alignment, debug metadata, etc.
    // The type+count portion ends at ", align" or end of string
    let type_and_count = if let Some(align_pos) = after_alloca.find(", align") {
        after_alloca[..align_pos].trim()
    } else {
        after_alloca.trim()
    };

    // Check if there's a dynamic count: "TYPE, iNN COUNT"
    // The count is the last ", iNN VALUE" portion
    let (type_str, count) = if let Some(comma_pos) = type_and_count.rfind(", i") {
        let type_part = type_and_count[..comma_pos].trim();
        let count_part = type_and_count[comma_pos + 2..].trim(); // skip ", "
        // Parse "i64 50" → 50
        let count_val = count_part
            .split_whitespace()
            .nth(1)
            .and_then(|v| v.parse::<u64>().ok());
        if let Some(c) = count_val {
            (type_part, c)
        } else {
            // Could not parse count — dynamic VLA
            return None;
        }
    } else {
        (type_and_count, 1u64)
    };

    // Compute the element type size in bytes
    let elem_size = compute_llvm_type_size(type_str)?;
    Some(elem_size * count)
}

/// Compute the size in bytes of an LLVM type from its string representation.
///
/// Handles: `i8`, `i16`, `i32`, `i64`, `i128`, `ptr`, `float`, `double`,
/// `[N x TYPE]` (arrays), and `{ TYPE, TYPE, ... }` (structs, approximate).
fn compute_llvm_type_size(type_str: &str) -> Option<u64> {
    let t = type_str.trim();

    // Integer types: i1, i8, i16, i32, i64, i128
    if let Some(bits_str) = t.strip_prefix('i') {
        let bits: u64 = bits_str.parse().ok()?;
        return Some(bits.div_ceil(8)); // round up to bytes
    }

    // Pointer type (opaque pointer in LLVM 15+)
    if t == "ptr" {
        return Some(8); // 64-bit target
    }

    // Float types
    if t == "float" {
        return Some(4);
    }
    if t == "double" {
        return Some(8);
    }

    // Array type: [N x TYPE]
    if t.starts_with('[') {
        // Parse "[N x TYPE]"
        let inner = t.strip_prefix('[')?.strip_suffix(']')?;
        let x_pos = inner.find(" x ")?;
        let n: u64 = inner[..x_pos].trim().parse().ok()?;
        let elem_type = inner[x_pos + 3..].trim();
        let elem_size = compute_llvm_type_size(elem_type)?;
        return Some(n * elem_size);
    }

    // Struct types: { i32, ptr, ... } — approximate by summing fields
    if t.starts_with('{') || t.starts_with('%') {
        // For named struct types or complex struct types, fall back to None
        // since we can't easily compute padding/alignment.
        return None;
    }

    None
}

/// Convert LLVM icmp predicate to AIR `BinaryOp`.
fn convert_icmp_predicate(inst: InstructionValue<'_>) -> BinaryOp {
    let predicate = inst.get_icmp_predicate().unwrap_or(IntPredicate::EQ);
    match predicate {
        IntPredicate::EQ => BinaryOp::ICmpEq,
        IntPredicate::NE => BinaryOp::ICmpNe,
        IntPredicate::UGT => BinaryOp::ICmpUgt,
        IntPredicate::UGE => BinaryOp::ICmpUge,
        IntPredicate::ULT => BinaryOp::ICmpUlt,
        IntPredicate::ULE => BinaryOp::ICmpUle,
        IntPredicate::SGT => BinaryOp::ICmpSgt,
        IntPredicate::SGE => BinaryOp::ICmpSge,
        IntPredicate::SLT => BinaryOp::ICmpSlt,
        IntPredicate::SLE => BinaryOp::ICmpSle,
    }
}

/// Convert LLVM fcmp predicate to AIR `BinaryOp`.
fn convert_fcmp_predicate(inst: InstructionValue<'_>) -> BinaryOp {
    let predicate = inst.get_fcmp_predicate().unwrap_or(FloatPredicate::OEQ);
    match predicate {
        FloatPredicate::ONE | FloatPredicate::UNE => BinaryOp::FCmpOne,
        FloatPredicate::OGT | FloatPredicate::UGT => BinaryOp::FCmpOgt,
        FloatPredicate::OGE | FloatPredicate::UGE => BinaryOp::FCmpOge,
        FloatPredicate::OLT | FloatPredicate::ULT => BinaryOp::FCmpOlt,
        FloatPredicate::OLE | FloatPredicate::ULE => BinaryOp::FCmpOle,
        // OEQ, UEQ, and all other predicates map to FCmpOeq
        _ => BinaryOp::FCmpOeq,
    }
}

/// Convert a constant LLVM value to an AIR constant.
// INVARIANT: LLVM integer bit widths (1-128) fit in u8.
#[allow(clippy::cast_possible_truncation)]
fn convert_constant_value(value: BasicValueEnum<'_>) -> Option<Constant> {
    match value {
        BasicValueEnum::IntValue(v) => {
            let bit_width = v.get_type().get_bit_width() as u8;
            if let Some(val) = v.get_sign_extended_constant() {
                if bit_width <= 64 {
                    Some(Constant::int(val, bit_width))
                } else {
                    Some(Constant::big_int(i128::from(val), bit_width))
                }
            } else {
                None
            }
        }
        BasicValueEnum::FloatValue(v) => {
            let bit_width = match v.get_type().print_to_string().to_str() {
                Ok("float") => 32,
                // "double" and all other types default to 64
                _ => 64,
            };
            v.get_constant()
                .map(|(val, _)| Constant::float(val, bit_width))
        }
        BasicValueEnum::PointerValue(v) => {
            if v.is_null() {
                Some(Constant::Null)
            } else {
                None
            }
        }
        BasicValueEnum::ArrayValue(v) => {
            // Guard: check for null/zeroinitializer BEFORE calling
            // `is_const_string()`.  inkwell's `is_const_string()` incorrectly
            // returns true for `ConstantAggregateZero` (zeroinitializer) arrays,
            // and the subsequent `get_string_constant()` call dereferences
            // invalid memory (SIGSEGV) because there is no backing string data.
            if v.is_null() {
                return Some(Constant::ZeroInit);
            }
            // Try to extract string constant
            if v.is_const_string() {
                #[allow(deprecated)]
                v.get_string_constant()
                    .and_then(|s| s.to_str().ok())
                    .map(Constant::string)
            } else {
                None
            }
        }
        BasicValueEnum::StructValue(v) => {
            if v.is_null() {
                Some(Constant::ZeroInit)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Try to parse an LLVM constant integer array from its string representation.
///
/// Handles patterns like `[2 x i32] [i32 1, i32 2]` and returns
/// `Constant::Aggregate { elements: [Int{1,32}, Int{2,32}] }`.
/// Returns `None` for non-integer arrays, strings, or unparseable formats.
fn try_parse_integer_array(repr: &str) -> Option<Constant> {
    // LLVM prints constant arrays as: [N x iM] [iM v1, iM v2, ...]
    // Find "] [" which separates the type bracket from the value bracket.
    let sep_pos = repr.find("] [")?;
    let value_start = sep_pos + 3; // skip "] ["
    let value_end = repr.rfind(']')?;

    if value_start >= value_end {
        return None;
    }

    let contents = &repr[value_start..value_end];

    let mut elements = Vec::new();
    for part in contents.split(',') {
        let trimmed = part.trim();
        // Parse "iN VALUE" pattern (e.g., "i32 1", "i8 -5")
        let rest = trimmed.strip_prefix('i')?;
        let space_pos = rest.find(' ')?;
        let bits: u8 = rest[..space_pos].parse().ok()?;
        let value: i64 = rest[space_pos + 1..].trim().parse().ok()?;
        elements.push(Constant::Int { value, bits });
    }

    if elements.is_empty() {
        return None;
    }

    Some(Constant::Aggregate { elements })
}

/// Parse a constant-expression GEP repr to extract the global name and indices.
///
/// Given a string like `ptr getelementptr inbounds ([2 x i32], ptr @a, i64 0, i64 1)`,
/// returns `("a", [0, 1])`.
fn parse_constant_gep(repr: &str) -> Option<(&str, Vec<i64>)> {
    if !repr.contains("getelementptr") {
        return None;
    }

    // Extract the global name: find @name in the repr
    let at_pos = repr.find('@')?;
    let after_at = &repr[at_pos + 1..];
    let global_name = if let Some(stripped) = after_at.strip_prefix('"') {
        let end_quote = stripped.find('"')?;
        &stripped[..end_quote]
    } else {
        let end = after_at.find([',', ')', ' ']).unwrap_or(after_at.len());
        &after_at[..end]
    };

    // Extract constant integer indices: find "iN <number>" patterns after each
    // comma following the global name. LLVM 18 may prefix an index with
    // `inrange` (e.g., "inrange i32 0"); LLVM 20+ moved `inrange(N,M)` to the
    // GEP level instead, so it doesn't appear here. Skip any leading attribute
    // word (alphabetic) before matching the "iN N" type+value pair.
    let mut indices: Vec<i64> = Vec::new();
    let mut search_from = at_pos;
    while let Some(comma_pos) = repr[search_from..].find(',') {
        let after_comma = &repr[search_from + comma_pos + 1..];
        let mut tok = after_comma.trim_start();

        // Strip leading attribute words (e.g., "inrange ") until we see "iN ".
        while let Some(first) = tok.chars().next() {
            if first == 'i' {
                // Candidate "iN N" — check that what follows 'i' is digits + space.
                let after_i = &tok[1..];
                let digits_end = after_i
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(after_i.len());
                if digits_end > 0 && after_i[digits_end..].starts_with(' ') {
                    break; // matches "iN " — parse below
                }
            }
            // Skip one whitespace-delimited token and continue.
            match tok.find(char::is_whitespace) {
                Some(next) => tok = tok[next..].trim_start(),
                None => {
                    tok = "";
                    break;
                }
            }
        }

        if let Some(rest) = tok.strip_prefix('i') {
            if let Some(space_pos) = rest.find(' ') {
                let num_str = rest[space_pos + 1..]
                    .trim_start()
                    .split([',', ')', ' '])
                    .next()
                    .unwrap_or("");
                if let Ok(idx) = num_str.parse::<i64>() {
                    indices.push(idx);
                }
            }
        }
        search_from += comma_pos + 1;
    }

    if indices.is_empty() {
        return None;
    }

    Some((global_name, indices))
}

/// Resolve a constant-expression GEP into a global aggregate initializer.
///
/// Given a string repr like `ptr getelementptr inbounds ([2 x i32], ptr @a, i64 0, i64 1)`,
/// extracts the global name (`a`) and indices (`[0, 1]`), looks up the global's
/// initializer, and returns the indexed scalar element as a `Constant`.
///
/// This enables the absint to resolve loads from constant-expression GEPs
/// into globals with aggregate initializers (e.g., `int a[2] = {1, 2}`).
fn resolve_constant_gep_element(
    repr: &str,
    global_inits: &BTreeMap<String, Constant>,
) -> Option<Constant> {
    let (global_name, indices) = parse_constant_gep(repr)?;
    let init = global_inits.get(global_name)?;

    // Walk the aggregate initializer using the indices.
    // Skip the first index (it's the pointer dereference index, usually 0).
    let element_indices = if indices.len() > 1 {
        &indices[1..]
    } else {
        &indices[..]
    };

    let mut current = init;
    for &idx in element_indices {
        let u = usize::try_from(idx).ok()?;
        match current {
            Constant::Aggregate { elements } => {
                current = elements.get(u)?;
            }
            _ => return None,
        }
    }

    // Only return scalar constants
    match current {
        Constant::Int { .. }
        | Constant::BigInt { .. }
        | Constant::Null
        | Constant::Undef
        | Constant::ZeroInit => Some(current.clone()),
        _ => None,
    }
}
