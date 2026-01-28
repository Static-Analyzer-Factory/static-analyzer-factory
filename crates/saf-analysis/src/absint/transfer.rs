//! Transfer functions for abstract interpretation.
//!
//! Interprets AIR instructions on abstract states, mapping each operation
//! to interval arithmetic. Branch conditions refine operand intervals on
//! true/false edges.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, AirType, BinaryOp, CastKind, FieldStep, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, LocId, ObjId, TypeId, ValueId};
use saf_core::saf_log;
use saf_core::spec::{AnalyzedSpecRegistry, BoundMode, ComputedBound};

use crate::pta::ptsset::IdBitSet;

use super::domain::AbstractDomain;
use super::function_properties::{is_pure_function_with_specs, is_set_value_function};
use super::interval::Interval;
use super::pta_integration::PtaIntegration;
use super::state::AbstractState;

/// Default bit-width for values without explicit type information.
const DEFAULT_BITS: u8 = 64;

thread_local! {
    /// Guard against infinite recursion in inline analysis.
    /// Tracks which functions are currently being analyzed inline.
    static INLINE_STACK: RefCell<BTreeSet<FunctionId>> = const { RefCell::new(BTreeSet::new()) };
}

/// Context for interval transfer with optional precision enhancements.
///
/// Provides a unified interface for all transfer function variants by bundling
/// optional PTA and function summary information.
#[derive(Default)]
pub struct TransferContext<'a> {
    /// Optional PTA for alias-aware memory operations.
    pub pta: Option<&'a PtaIntegration<'a>>,
    /// Optional return interval summaries for interprocedural propagation.
    pub return_intervals: Option<&'a BTreeMap<FunctionId, Interval>>,
    /// Optional set of reached block IDs for phi predecessor filtering.
    ///
    /// When set, the phi handler skips incoming values from predecessor blocks
    /// that are not in this set. This is critical for SSA-promoted IR (after
    /// `mem2reg`) where phi nodes reference values from unreachable branches.
    pub reached_blocks: Option<&'a IdBitSet<BlockId>>,
    /// Optional memory side-effect summaries for interprocedural propagation.
    ///
    /// Maps function ID → (param index → interval stored to `*param[i]`).
    /// Used when inline analysis is skipped to apply callee's memory effects.
    pub memory_summaries: Option<&'a BTreeMap<FunctionId, BTreeMap<usize, Interval>>>,
    /// Optional global variable store summaries for interprocedural propagation.
    ///
    /// Maps function ID → (`LocId` → interval stored to the global).
    /// Applied at call sites to propagate global modifications across calls.
    pub global_summaries: Option<&'a BTreeMap<FunctionId, BTreeMap<LocId, Interval>>>,
    /// Optional analyzed spec registry for looking up external function return intervals
    /// and computed bounds.
    pub specs: Option<&'a AnalyzedSpecRegistry>,
    /// Optional map from allocation `ObjId` to struct `TypeId`.
    /// Used to compute field byte offsets for field-sensitive GEP tracking.
    pub obj_type_map: Option<&'a BTreeMap<ObjId, TypeId>>,
}

impl<'a> TransferContext<'a> {
    /// Create a new transfer context with optional enhancements.
    #[must_use]
    pub fn new(
        pta: Option<&'a PtaIntegration<'a>>,
        return_intervals: Option<&'a BTreeMap<FunctionId, Interval>>,
    ) -> Self {
        Self {
            pta,
            return_intervals,
            reached_blocks: None,
            memory_summaries: None,
            global_summaries: None,
            specs: None,
            obj_type_map: None,
        }
    }
}

/// Build a value-to-constant map for a module (for constant operands).
///
/// Uses the `AirModule.constants` map populated by the frontend to create
/// intervals for constant values, and registers function parameters as top.
pub fn build_constant_map(module: &AirModule) -> BTreeMap<ValueId, Interval> {
    use saf_core::air::Constant;

    let mut constants = BTreeMap::new();

    // Convert AIR constants to intervals
    for (value_id, constant) in &module.constants {
        let interval = match constant {
            Constant::Int { value, bits } => Interval::singleton(i128::from(*value), *bits),
            Constant::BigInt { value, bits } => {
                // Parse the string as i128
                if let Ok(v) = value.parse::<i128>() {
                    Interval::singleton(v, *bits)
                } else {
                    Interval::make_top(*bits)
                }
            }
            Constant::Float { .. } => {
                // Floats are not handled by interval domain — storing them
                // as truncated integers would produce incorrect results for
                // float comparisons (FCmpOeq). Float→integer conversion is
                // handled at FPToSI/FPToUI cast sites in the transfer function.
                continue;
            }
            Constant::Null | Constant::Undef | Constant::ZeroInit => {
                // Null/undef/zero treated as 0 (pointer value)
                Interval::singleton(0, DEFAULT_BITS)
            }
            Constant::String { .. } | Constant::Aggregate { .. } | Constant::GlobalRef(_) => {
                // Strings, aggregates, and global refs are not scalar values
                continue;
            }
        };
        constants.insert(*value_id, interval);
    }

    // Register function parameters as top
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for param in &func.params {
            constants.insert(param.id, Interval::make_top(DEFAULT_BITS));
        }
    }

    constants
}

/// Resolve an operand `ValueId` to its interval in the current state.
///
/// Checks the abstract state first, then falls back to the constant map,
/// then returns top.
pub fn resolve_operand(
    operand: ValueId,
    state: &AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> Interval {
    // First check abstract state
    if let Some(val) = state.get_opt(operand) {
        return val.clone();
    }
    // Then check constant map
    if let Some(val) = constant_map.get(&operand) {
        return val.clone();
    }
    // Default to top (unknown)
    Interval::make_top(DEFAULT_BITS)
}

/// Apply the transfer function for a single instruction with optional context.
///
/// This is the unified transfer function that handles all variants:
/// - No context: basic `ValueId`-based memory tracking
/// - With PTA: alias-aware memory operations using points-to information
/// - With PTA + summaries: additionally uses return interval summaries and GEP tracking
///
/// # Arguments
///
/// * `inst` - The instruction to transfer
/// * `state` - The abstract state (modified in place)
/// * `constant_map` - Map of constant values
/// * `module` - The AIR module (for function lookups)
/// * `ctx` - Transfer context with optional PTA and return summaries
///
/// # Panics
///
/// Panics if a singleton GEP target set is empty after a `len() == 1` check (should
/// never happen), or if `ctx.return_intervals` is `Some` but `ctx.pta` is `None`
/// (invariant: summaries implies PTA).
// NOTE: Transfer function covers all instruction types with context-dependent
// behavior (Store, Memcpy, Load, Call, GEP, etc.). Splitting by instruction
// type would fragment the shared PTA/summary dispatch logic.
#[allow(clippy::too_many_lines)]
pub fn transfer_instruction_with_context(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    ctx: &TransferContext<'_>,
) {
    // ==========================================================================
    // Store handling - varies based on context
    // ==========================================================================
    if let Operation::Store = &inst.op {
        if inst.operands.len() >= 2 {
            let value_id = inst.operands[0];
            let ptr_id = inst.operands[1];
            let value_interval = resolve_operand(value_id, state, constant_map);

            // Debug: trace stores (summaries variant only)
            if ctx.return_intervals.is_some() {
                saf_log!(absint::transfer, constraint, "store trace"; inst=inst.id, ptr=ptr_id, interval=format!("[{}, {}]", value_interval.lo(), value_interval.hi()), mem_before=state.memory_entries().len());
            }

            // Always track by pointer ValueId for may_alias fallback
            state.store(ptr_id, value_interval.clone());

            // Field-sensitive store: check field GEP targets first
            if let Some(field_targets) = state.resolve_field_gep(ptr_id).cloned() {
                if field_targets.len() == 1 {
                    let (loc, offset) = *field_targets.iter().next().expect("checked len == 1");
                    state.store_field(loc, offset, value_interval.clone());
                } else {
                    for &(loc, offset) in &field_targets {
                        state.store_field_weak(loc, offset, value_interval.clone());
                    }
                }
            }

            if let Some(pta) = ctx.pta {
                // PTA-aware store handling
                if ctx.return_intervals.is_some() {
                    // With summaries: check GEP targets first, then fall back to PTA
                    if let Some(gep_targets) = state.resolve_gep(ptr_id).cloned() {
                        saf_log!(absint::transfer, constraint, "store via GEP"; ptr=ptr_id, gep_targets=format!("{:?}", gep_targets), interval=format!("{:?}", value_interval));

                        if gep_targets.len() == 1 {
                            let loc = *gep_targets.iter().next().expect("checked len == 1");
                            state.store_loc(loc, value_interval);
                        } else {
                            for loc in &gep_targets {
                                state.store_loc_weak(*loc, value_interval.clone());
                            }
                        }
                    } else {
                        // Fall back to PTA-based tracking
                        transfer_store_with_pta(ptr_id, value_interval, state, pta);
                    }
                } else {
                    // PTA only (no summaries): use PTA directly
                    transfer_store_with_pta(ptr_id, value_interval, state, pta);
                }
            }
            // Without PTA: already tracked by ValueId above
        }
        return;
    }

    // ==========================================================================
    // Memcpy/Memset handling
    // ==========================================================================
    if matches!(&inst.op, Operation::Memcpy | Operation::Memset) {
        if let Some(pta) = ctx.pta {
            match &inst.op {
                Operation::Memset => {
                    // memset(dst, val, len): store val to all tracked locs of dst,
                    // including all element-level sub-locations sharing the same object
                    if inst.operands.len() >= 2 {
                        let dst_ptr = inst.operands[0];
                        let val_interval = resolve_operand(inst.operands[1], state, constant_map);
                        let dst_locs = if let Some(gep) = state.resolve_gep(dst_ptr).cloned() {
                            gep
                        } else {
                            pta.points_to(dst_ptr)
                        };
                        // Store to direct targets
                        for loc in &dst_locs {
                            state.store_loc(*loc, val_interval.clone());
                        }
                        // Also store to all sub-locations of the same object
                        // so GEP-indexed loads find the memset value
                        for loc in &dst_locs {
                            for sub_loc in pta.locations_of_same_object(*loc) {
                                state.store_loc(sub_loc, val_interval.clone());
                            }
                        }
                    }
                }
                Operation::Memcpy => {
                    // memcpy(dst, src, len): copy intervals from src locs to dst locs,
                    // including all element-level sub-locations
                    if inst.operands.len() >= 2 {
                        let dst_ptr = inst.operands[0];
                        let src_ptr = inst.operands[1];
                        let src_locs = if let Some(gep) = state.resolve_gep(src_ptr).cloned() {
                            gep
                        } else {
                            pta.points_to(src_ptr)
                        };
                        let dst_locs = if let Some(gep) = state.resolve_gep(dst_ptr).cloned() {
                            gep
                        } else {
                            pta.points_to(dst_ptr)
                        };
                        // Collect source intervals (including sub-locations)
                        let mut src_interval = Interval::make_bottom(DEFAULT_BITS);
                        for loc in &src_locs {
                            if let Some(iv) = state.load_loc(*loc) {
                                src_interval = src_interval.join(iv);
                            }
                            for sub_loc in pta.locations_of_same_object(*loc) {
                                if let Some(iv) = state.load_loc(sub_loc) {
                                    src_interval = src_interval.join(iv);
                                }
                            }
                        }
                        // Write joined source interval to all dst locs and sub-locs
                        if !src_interval.is_bottom() {
                            for loc in &dst_locs {
                                state.store_loc(*loc, src_interval.clone());
                                for sub_loc in pta.locations_of_same_object(*loc) {
                                    state.store_loc(sub_loc, src_interval.clone());
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        // Always invalidate ValueId-based memory (conservative for non-PTA path)
        state.invalidate_all_memory();
        return;
    }

    // For instructions that produce values, we need a destination
    let Some(dst) = inst.dst else {
        return;
    };

    match &inst.op {
        // =====================================================================
        // Load handling - varies based on context
        // =====================================================================
        Operation::Load => {
            if let Some(&ptr) = inst.operands.first() {
                if let Some(pta) = ctx.pta {
                    // PTA-aware load
                    let result = if ctx.return_intervals.is_some() {
                        // With summaries: check GEP targets first
                        transfer_load_with_pta_and_gep(ptr, dst, state, pta)
                    } else {
                        // PTA only
                        transfer_load_with_pta(ptr, dst, state, pta)
                    };
                    state.set(dst, result);
                } else {
                    // No PTA: basic ValueId-based load
                    if let Some(stored_interval) = state.load(ptr) {
                        state.set(dst, stored_interval.clone());
                    } else {
                        state.set(dst, Interval::make_top(DEFAULT_BITS));
                    }
                }
            } else {
                state.set(dst, Interval::make_top(DEFAULT_BITS));
            }
        }

        // =====================================================================
        // GEP handling - varies based on context
        // =====================================================================
        Operation::Gep { .. } => {
            if let Some(pta) = ctx.pta {
                if ctx.return_intervals.is_some() {
                    // With summaries: register GEP targets for field-sensitive tracking
                    let mut target_locs = pta.points_to(dst);

                    // Try to refine GEP targets when PTA produced locations with
                    // unresolved Index steps (dynamic array indices).
                    // Also filter multi-target sets using known operand values.
                    if !target_locs.is_empty() && inst.operands.len() >= 2 {
                        let needs_refinement = pta.targets_have_unresolved_index(&target_locs);

                        if needs_refinement {
                            // Strategy 1: Use base pointer's resolved GEP targets.
                            // When this GEP chains off a previous GEP (e.g., accessing
                            // `.b` after selecting `a[0]`), the base GEP already resolved
                            // the array index. Substitute the resolved prefix path to find
                            // the precise location (e.g., a[0].b instead of a[?].b).
                            let base_ptr = inst.operands[0];
                            if let Some(base_gep_targets) = state.resolve_gep(base_ptr) {
                                let refined =
                                    pta.refine_gep_by_base_targets(&target_locs, base_gep_targets);
                                if refined != target_locs {
                                    saf_log!(absint::transfer, context, "GEP base-resolved"; from=format!("{:?}", target_locs), to=format!("{:?}", refined));
                                    target_locs = refined;
                                }
                            }

                            // Strategy 2: Use the last GEP operand as array index.
                            // For single-level GEPs (no base GEP chain), the last operand
                            // is the array index itself. Only apply if Strategy 1 didn't
                            // fully resolve (still has Unknown steps).
                            if pta.targets_have_unresolved_index(&target_locs) {
                                let index_operand = inst.operands[inst.operands.len() - 1];
                                let index_interval =
                                    resolve_operand(index_operand, state, constant_map);

                                saf_log!(absint::transfer, context, "GEP refinement check"; index_operand=index_operand, index_interval=format!("{:?}", index_interval));

                                if let Some(idx) = index_interval.as_singleton() {
                                    let refined = pta.refine_gep_targets_with_index(
                                        &target_locs,
                                        &index_interval,
                                    );
                                    if refined == target_locs {
                                        saf_log!(absint::transfer, context, "GEP no refinement found"; index=idx, targets=format!("{:?}", target_locs));
                                    } else {
                                        saf_log!(absint::transfer, context, "GEP refined targets"; from=format!("{:?}", target_locs), to=format!("{:?}", refined), index=idx);
                                        target_locs = refined;
                                    }
                                }
                            }
                        }

                        // Strategy 3: PTA collapsed variable-index GEP to parent.
                        // When PTA approximated a variable-index GEP to its parent
                        // location (no Index(Unknown) in path), but the GEP's AIR
                        // field_path has Index steps, try to find the child element
                        // location using the known index value.
                        if !needs_refinement {
                            if let Operation::Gep { field_path, .. } = &inst.op {
                                let has_air_index = field_path
                                    .steps
                                    .iter()
                                    .any(|s| matches!(s, saf_core::air::FieldStep::Index));
                                if has_air_index {
                                    let index_operand = inst.operands[inst.operands.len() - 1];
                                    let index_interval =
                                        resolve_operand(index_operand, state, constant_map);
                                    let refined = pta.refine_gep_targets_by_child_index(
                                        &target_locs,
                                        &index_interval,
                                    );
                                    if refined != target_locs {
                                        saf_log!(absint::transfer, context, "GEP child-refined"; from=format!("{:?}", target_locs), to=format!("{:?}", refined));
                                        target_locs = refined;
                                    }
                                }
                            }
                        }
                    }

                    if !target_locs.is_empty() {
                        saf_log!(absint::transfer, context, "GEP targets"; dst=dst, target_locs=format!("{:?}", target_locs));

                        // Field-sensitive GEP tracking: compute byte offsets
                        if let Operation::Gep { field_path, .. } = &inst.op {
                            if let Some(obj_type_map) = ctx.obj_type_map {
                                let has_field_step = field_path
                                    .steps
                                    .iter()
                                    .any(|s| matches!(s, FieldStep::Field { .. }));
                                if has_field_step {
                                    if let Some(pta) = ctx.pta {
                                        if let Some(pta_ref) = pta.pta_ref() {
                                            let mut field_targets = BTreeSet::new();
                                            for &loc_id in &target_locs {
                                                if let Some(loc) = pta_ref.location(loc_id) {
                                                    if let Some(type_id) =
                                                        obj_type_map.get(&loc.obj)
                                                    {
                                                        if let Some(base_type) =
                                                            module.types.get(type_id)
                                                        {
                                                            if let Some(offset) =
                                                                compute_field_byte_offset(
                                                                    field_path,
                                                                    base_type,
                                                                    &module.types,
                                                                )
                                                            {
                                                                field_targets
                                                                    .insert((loc_id, offset));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if !field_targets.is_empty() {
                                                saf_log!(absint::transfer, context, "GEP field targets"; dst=dst, field_targets=format!("{:?}", field_targets));
                                                state.register_field_gep(dst, field_targets);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        state.register_gep(dst, target_locs);
                    }
                }
                // PTA without summaries: no GEP tracking needed
            }
            // GEP result is always a pointer -> top
            state.set(dst, Interval::make_top(DEFAULT_BITS));
        }

        // =====================================================================
        // CallDirect handling - varies based on context
        // =====================================================================
        Operation::CallDirect { callee } => {
            let (mut return_interval, inline_succeeded) =
                if let Some(return_intervals) = ctx.return_intervals {
                    // With summaries: try inline analysis or use summary
                    compute_call_return_with_summaries(
                        inst,
                        *callee,
                        state,
                        constant_map,
                        module,
                        ctx.pta.expect("summaries implies PTA is available"),
                        return_intervals,
                    )
                } else {
                    // Without summaries: return TOP
                    (Interval::make_top(DEFAULT_BITS), false)
                };

            // If the return interval is TOP, check the analyzed spec registry.
            // First try computed bounds (dynamic), then fall back to fixed interval (static).
            if return_interval.is_top() {
                if let Some(analyzed_specs) = ctx.specs {
                    let callee_name = module
                        .functions
                        .iter()
                        .find(|f| f.id == *callee)
                        .map(|f| f.name.as_str());
                    if let Some(name) = callee_name {
                        // Try computed bound first — resolves at this call site
                        if let Some(derived) = analyzed_specs.lookup_derived(name) {
                            if let Some(ref bound) = derived.computed_return_bound {
                                let resolved = resolve_computed_bound(bound, inst, state, module);
                                if !resolved.is_top() {
                                    return_interval = resolved;
                                }
                            }
                        }

                        // Fall back to fixed YAML interval if still TOP
                        if return_interval.is_top() {
                            if let Some(spec) = analyzed_specs.lookup_yaml(name) {
                                if let Some(ref ret) = spec.returns {
                                    if let Some((lo, hi)) = ret.interval {
                                        return_interval = Interval::new(
                                            i128::from(lo),
                                            i128::from(hi),
                                            DEFAULT_BITS,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            state.set(dst, return_interval.clone());

            // If the return interval is bottom (e.g. recursive SCC summary not
            // yet computed), the call cannot return yet.  Mark the rest of the
            // block unreachable so successor joins don't pollute the phi with
            // a TOP fallback for the undefined value.
            if return_interval.is_bottom() {
                *state = super::state::AbstractState::bottom();
                return;
            }

            // Handle set_value(val, lb, ub): constrains the variable that val
            // was loaded from to [lb, ub].
            let callee_func = module.functions.iter().find(|f| f.id == *callee);
            if callee_func.is_some_and(|f| is_set_value_function(&f.name))
                && inst.operands.len() >= 3
            {
                let val_id = inst.operands[0];
                let lb_interval = resolve_operand(inst.operands[1], state, constant_map);
                let ub_interval = resolve_operand(inst.operands[2], state, constant_map);
                if lb_interval.is_singleton() && ub_interval.is_singleton() {
                    let constrained =
                        Interval::new(lb_interval.lo(), ub_interval.lo(), DEFAULT_BITS);

                    // SSA-direct path: if PTA knows what val_id points to,
                    // store the constraint directly to those locations.
                    let mut stored = false;
                    if let Some(pta) = ctx.pta {
                        let pts = pta.points_to(val_id);
                        if !pts.is_empty() {
                            for loc in &pts {
                                state.store_loc(*loc, constrained.clone());
                            }
                            stored = true;
                        }
                    }

                    // Fallback for address-taken locals: trace Load→alloca
                    if !stored {
                        'find_load: for func in &module.functions {
                            for block in &func.blocks {
                                for block_inst in &block.instructions {
                                    if block_inst.dst == Some(val_id) {
                                        if let Operation::Load = &block_inst.op {
                                            if let Some(&src_ptr) = block_inst.operands.first() {
                                                if let Some(pta) = ctx.pta {
                                                    let pts = pta.points_to(src_ptr);
                                                    for loc in &pts {
                                                        state.store_loc(*loc, constrained.clone());
                                                    }
                                                }
                                                state.store(src_ptr, constrained.clone());
                                                stored = true;
                                            }
                                        }
                                        break 'find_load;
                                    }
                                }
                            }
                        }
                    }

                    // Final fallback: store to the ValueId itself
                    if !stored {
                        state.store(val_id, constrained);
                    }
                }
                return; // set_value doesn't invalidate other memory
            }

            // Memory invalidation — skip when inline analysis succeeded,
            // since it already computed precise callee effects on loc_memory.
            let is_side_effect_free =
                callee_func.is_some_and(|f| is_pure_function_with_specs(&f.name, None));

            if !is_side_effect_free && !inline_succeeded {
                state.invalidate_all_memory();
                if let Some(pta) = ctx.pta {
                    // For known defined functions with PTA: only invalidate
                    // locations reachable through pointer arguments, preserving
                    // caller-local allocas that aren't passed to the callee.
                    let is_defined_callee = callee_func.is_some_and(|f| !f.is_declaration);
                    if is_defined_callee {
                        let mut reachable_locs = std::collections::BTreeSet::new();
                        for &arg_id in &inst.operands {
                            // Check if arg has GEP targets
                            if let Some(gep_targets) = state.resolve_gep(arg_id) {
                                reachable_locs.extend(gep_targets.iter().copied());
                            }
                            // Check PTA points-to set
                            let pts = pta.points_to(arg_id);
                            reachable_locs.extend(pts);
                        }
                        if reachable_locs.is_empty() {
                            // No pointer args → callee can't modify any loc
                            // (it only creates its own allocas)
                        } else {
                            state.invalidate_locs(&reachable_locs);
                        }
                    } else {
                        // External/unknown function: set loc_memory entries
                        // reachable through pointer arguments to TOP. This handles
                        // functions like `scanf` that write through out-params.
                        // Using explicit TOP (instead of removal) ensures the
                        // location is tracked for subsequent branch refinement.
                        let mut reachable_locs = std::collections::BTreeSet::new();
                        for &arg_id in &inst.operands {
                            if let Some(gep_targets) = state.resolve_gep(arg_id) {
                                reachable_locs.extend(gep_targets.iter().copied());
                            }
                            let pts = pta.points_to(arg_id);
                            reachable_locs.extend(pts);
                        }
                        for loc in &reachable_locs {
                            state.store_loc(*loc, Interval::make_top(DEFAULT_BITS));
                        }
                    }
                }

                // Apply memory side-effects from summaries (Plan 084 Phase C).
                // After invalidation, restore precise intervals for locations
                // that the callee stores to via its parameters.
                apply_memory_effects(inst, *callee, state, ctx);
            }

            // Apply global store effects (Plan 086 Phase H4).
            // Always apply global variable summaries regardless of inline success,
            // because inline analysis only propagates loc_memory for locations
            // reachable through pointer arguments — not globals.
            if !is_side_effect_free {
                apply_global_effects(*callee, state, ctx);
            }
        }

        // =====================================================================
        // CallIndirect handling
        // =====================================================================
        Operation::CallIndirect { .. } => {
            state.set(dst, Interval::make_top(DEFAULT_BITS));
            state.invalidate_all_memory();

            if let Some(pta) = ctx.pta {
                // Try to resolve indirect call targets via PTA
                let fn_ptr = inst.operands.last().copied();
                let targets = fn_ptr
                    .map(|fp| pta.resolve_indirect_call(fp))
                    .unwrap_or_default();

                if targets.is_empty() {
                    state.invalidate_all_loc_memory();
                } else {
                    // Arguments are all operands except the last (fn pointer)
                    let arg_count = inst.operands.len().saturating_sub(1);

                    // Selective loc invalidation (like CallDirect)
                    let mut reachable_locs = std::collections::BTreeSet::new();
                    for &arg_id in &inst.operands[..arg_count] {
                        if let Some(gep_targets) = state.resolve_gep(arg_id) {
                            reachable_locs.extend(gep_targets.iter().copied());
                        }
                        reachable_locs.extend(pta.points_to(arg_id));
                    }
                    if !reachable_locs.is_empty() {
                        state.invalidate_locs(&reachable_locs);
                    }

                    // Apply return intervals from resolved targets
                    if let Some(return_intervals) = ctx.return_intervals {
                        let mut return_val = Interval::make_bottom(DEFAULT_BITS);
                        let mut has_return = false;
                        for target_id in &targets {
                            if let Some(ri) = return_intervals.get(target_id) {
                                return_val = return_val.join(ri);
                                has_return = true;
                            }
                        }
                        if has_return {
                            state.set(dst, return_val);
                        }
                    }

                    // Apply memory side-effects from resolved targets
                    if let Some(memory_summaries) = ctx.memory_summaries {
                        for target_id in &targets {
                            if let Some(effects) = memory_summaries.get(target_id) {
                                for (&param_idx, effect_interval) in effects {
                                    if param_idx < arg_count {
                                        let arg_id = inst.operands[param_idx];
                                        apply_effect_to_arg(arg_id, effect_interval, state, pta);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // =====================================================================
        // Operations that are identical across all variants
        // =====================================================================
        Operation::Copy => {
            if let Some(&operand) = inst.operands.first() {
                let val = resolve_operand(operand, state, constant_map);
                state.set(dst, val);
            }
        }

        Operation::BinaryOp { kind } => {
            if inst.operands.len() >= 2 {
                let lhs = resolve_operand(inst.operands[0], state, constant_map);
                let rhs = resolve_operand(inst.operands[1], state, constant_map);

                let result = match kind {
                    BinaryOp::Add => lhs.add(&rhs),
                    BinaryOp::Sub => lhs.sub(&rhs),
                    BinaryOp::Mul => lhs.mul(&rhs),
                    BinaryOp::SDiv => lhs.sdiv(&rhs),
                    BinaryOp::UDiv => lhs.udiv(&rhs),
                    BinaryOp::SRem => lhs.srem(&rhs),
                    BinaryOp::URem => lhs.urem(&rhs),
                    BinaryOp::Shl => lhs.shl(&rhs),
                    BinaryOp::LShr => lhs.lshr(&rhs),
                    BinaryOp::AShr => lhs.ashr(&rhs),
                    BinaryOp::And => lhs.bitand(&rhs),
                    BinaryOp::Or => lhs.bitor(&rhs),
                    BinaryOp::Xor => lhs.bitxor(&rhs),

                    // Comparisons produce i1 (boolean) intervals
                    BinaryOp::ICmpSlt => lhs.icmp_slt(&rhs),
                    BinaryOp::ICmpSle => lhs.icmp_sle(&rhs),
                    BinaryOp::ICmpSgt => lhs.icmp_sgt(&rhs),
                    BinaryOp::ICmpSge => lhs.icmp_sge(&rhs),
                    BinaryOp::ICmpEq => lhs.icmp_eq(&rhs),
                    BinaryOp::ICmpNe => lhs.icmp_ne(&rhs),
                    BinaryOp::ICmpUlt => lhs.icmp_ult(&rhs),
                    BinaryOp::ICmpUle => lhs.icmp_ule(&rhs),
                    BinaryOp::ICmpUgt => lhs.icmp_ugt(&rhs),
                    BinaryOp::ICmpUge => lhs.icmp_uge(&rhs),

                    // Float operations -> top (we don't track floats)
                    BinaryOp::FAdd
                    | BinaryOp::FSub
                    | BinaryOp::FMul
                    | BinaryOp::FDiv
                    | BinaryOp::FRem
                    | BinaryOp::FCmpOeq
                    | BinaryOp::FCmpOne
                    | BinaryOp::FCmpOgt
                    | BinaryOp::FCmpOge
                    | BinaryOp::FCmpOlt
                    | BinaryOp::FCmpOle => Interval::make_top(DEFAULT_BITS),
                };
                state.set(dst, result);
            }
        }

        Operation::Cast { kind, target_bits } => {
            if let Some(&operand) = inst.operands.first() {
                let val = resolve_operand(operand, state, constant_map);

                let result = match kind {
                    CastKind::ZExt => {
                        let bits = target_bits.unwrap_or(DEFAULT_BITS);
                        val.zext(bits)
                    }
                    CastKind::SExt => {
                        let bits = target_bits.unwrap_or(DEFAULT_BITS);
                        val.sext(bits)
                    }
                    CastKind::Trunc => {
                        let bits = target_bits.unwrap_or(32);
                        // Preserve boolean values through `trunc ... to i1`.
                        // When the source interval is within `[0, 1]` and
                        // the target is 1 bit, the standard `trunc()` would
                        // lose precision. Keeping the interval as-is makes
                        // `br i1 %cond` provable after `%cond = trunc i8 %v to i1`.
                        if bits == 1 && val.lo() >= 0 && val.hi() <= 1 {
                            Interval::new(val.lo(), val.hi(), 1)
                        } else {
                            val.trunc(bits)
                        }
                    }
                    CastKind::FPToSI | CastKind::FPToUI => {
                        let bits = target_bits.unwrap_or(DEFAULT_BITS);
                        // Check for constant float source in the module (direct or via SSA chain)
                        if let Some(saf_core::air::Constant::Float { value, .. }) =
                            module.constants.get(&operand)
                        {
                            // INVARIANT: f64 values within i128 range after truncation.
                            #[allow(clippy::cast_possible_truncation)]
                            let int_val = *value as i128;
                            Interval::singleton(int_val, bits)
                        } else if let Some(float_val) = find_float_constant_ssa(operand, module) {
                            // Found float constant through SSA phi/copy chain
                            #[allow(clippy::cast_possible_truncation)]
                            let int_val = float_val as i128;
                            Interval::singleton(int_val, bits)
                        } else if val.is_singleton() {
                            Interval::singleton(val.lo(), bits)
                        } else {
                            Interval::make_top(bits)
                        }
                    }
                    // Pointer/float casts -> top
                    _ => Interval::make_top(target_bits.unwrap_or(DEFAULT_BITS)),
                };
                state.set(dst, result);
            }
        }

        Operation::Phi { incoming } => {
            let mut result = Interval::make_bottom(DEFAULT_BITS);
            for (block_id, val_id) in incoming {
                // Skip incoming values from unreachable predecessor blocks.
                // After mem2reg, phi nodes may reference values (params, undef)
                // that are in the state from earlier blocks even though the
                // predecessor never actually flows to this phi.
                if let Some(reached) = ctx.reached_blocks {
                    if !reached.contains(*block_id) {
                        continue;
                    }
                }
                // During fixpoint iteration, resolve incoming values using
                // bottom (not TOP) for values not yet in the state.  This
                // prevents premature overapproximation when a predecessor
                // block hasn't been processed yet (e.g. recursive call path
                // where the callee summary is still bottom).  The worklist
                // will re-process this phi once the predecessor contributes
                // a concrete value.
                let val = state
                    .get_opt(*val_id)
                    .cloned()
                    .or_else(|| constant_map.get(val_id).cloned())
                    .unwrap_or_else(|| Interval::make_bottom(DEFAULT_BITS));
                result = result.join(&val);
            }
            state.set(dst, result);
        }

        Operation::Select => {
            if inst.operands.len() >= 3 {
                let true_val = resolve_operand(inst.operands[1], state, constant_map);
                let false_val = resolve_operand(inst.operands[2], state, constant_map);
                state.set(dst, true_val.join(&false_val));
            }
        }

        Operation::Alloca { .. } | Operation::HeapAlloc { .. } | Operation::Global { .. } => {
            // Stack, heap, and global addresses are always non-null.
            // Represent as `[1, i64::MAX]` so that `ICmpNe(ptr, 0)` is
            // provable for `svf_assert(ptr != NULL)` patterns.
            state.set(dst, Interval::new(1, i128::from(i64::MAX), DEFAULT_BITS));
        }

        Operation::Freeze => {
            state.set(dst, Interval::make_top(DEFAULT_BITS));
        }

        // Store and Memcpy/Memset are handled at the beginning of this function
        Operation::Store | Operation::Memcpy | Operation::Memset => {
            unreachable!("Store/Memcpy/Memset should be handled early");
        }

        // Non-producing operations (terminators, etc.)
        Operation::Br { .. }
        | Operation::CondBr { .. }
        | Operation::Switch { .. }
        | Operation::Ret
        | Operation::Unreachable => {}
    }
}

// =============================================================================
// Helper functions for PTA-aware transfer operations
// =============================================================================

/// Handle store with PTA information (strong/weak update based on points-to set size).
// `pta` (points-to analysis) and `pts` (points-to set) are distinct concepts
#[allow(clippy::similar_names)]
fn transfer_store_with_pta(
    ptr_id: ValueId,
    value_interval: Interval,
    state: &mut AbstractState,
    pta: &PtaIntegration<'_>,
) {
    let pts = pta.points_to(ptr_id);
    saf_log!(absint::transfer, constraint, "store with PTA"; ptr=ptr_id, pts_len=pts.len(), interval=format!("{:?}", value_interval));

    if pts.is_empty() {
        // No PTA info - already tracked by ValueId
    } else if pts.len() == 1 {
        // Singleton - strong update
        let loc = *pts.iter().next().expect("checked len == 1");
        saf_log!(absint::transfer, constraint, "store strong update"; loc=loc);
        state.store_loc(loc, value_interval);
    } else {
        // Multiple targets - weak update
        saf_log!(absint::transfer, constraint, "store weak update"; locs=pts.len());
        for loc in &pts {
            state.store_loc_weak(*loc, value_interval.clone());
        }
    }
}

/// Handle load with PTA information (cascading fallbacks for alias resolution).
// `ptr` (pointer operand), `pta` (analysis), and `pts` (points-to set) are distinct concepts
// NOTE: Load resolution requires cascading fallback logic (PTA locs -> ValueId memory
// -> GEP targets -> default). Splitting would fragment the fallback chain.
#[allow(clippy::similar_names, clippy::too_many_lines)]
fn transfer_load_with_pta(
    ptr: ValueId,
    dst: ValueId,
    state: &AbstractState,
    pta: &PtaIntegration<'_>,
) -> Interval {
    let pts = pta.points_to(ptr);

    saf_log!(absint::transfer, pts, "load with PTA"; ptr=ptr, dst=dst, pts_len=pts.len());

    let mut result = Interval::make_bottom(DEFAULT_BITS);
    let mut found_any = false;

    // Check all PTA locations
    for loc in &pts {
        if let Some(interval) = state.load_loc(*loc) {
            saf_log!(absint::transfer, pts, "load loc hit"; loc=*loc, interval=format!("{:?}", interval));
            result = result.join(interval);
            found_any = true;
        } else {
            saf_log!(absint::transfer, pts, "load loc empty"; loc=*loc);
        }
    }

    // When PTA doesn't track this pointer (empty pts), check exact pointer
    // match FIRST. This handles local allocas where the same ValueId is used
    // for all stores/loads to the same alloca. The may_alias fallback below
    // would conservatively join ALL stores (since Unknown aliases with
    // everything), losing precision.
    if !found_any && pts.is_empty() {
        if let Some(stored_interval) = state.load(ptr) {
            saf_log!(absint::transfer, pts, "load exact ptr match (no PTA)"; interval=format!("{:?}", stored_interval));
            return stored_interval.clone();
        }
    }

    // Fallback: check may_alias with stored pointers
    if !found_any {
        let mem_entries = state.memory_entries();
        saf_log!(absint::transfer, pts, "load may_alias fallback"; stored_ptrs=mem_entries.len());
        for (stored_ptr, stored_interval) in mem_entries {
            let aliases = pta.may_alias(ptr, *stored_ptr);
            saf_log!(absint::transfer, pts, "load may_alias check"; ptr=ptr, stored_ptr=*stored_ptr, aliases=aliases);
            if aliases {
                saf_log!(absint::transfer, pts, "load may_alias hit"; interval=format!("{:?}", stored_interval));
                result = result.join(stored_interval);
                found_any = true;
            }
        }
    }

    // Fallback 2: check if any stored location has same ObjId as load location
    if !found_any && pta.has_pta() {
        let load_pts = &pts;
        for (stored_ptr, stored_interval) in state.memory_entries() {
            let store_pts = pta.points_to(*stored_ptr);
            for &load_loc in load_pts {
                for store_loc in &store_pts {
                    if pta.locations_share_object(load_loc, *store_loc) {
                        saf_log!(absint::transfer, pts, "load same ObjId"; load_loc=load_loc, store_loc=*store_loc, interval=format!("{:?}", stored_interval));
                        result = result.join(stored_interval);
                        found_any = true;
                    }
                }
            }
        }
    }

    // Final fallback: exact pointer match
    if !found_any {
        if let Some(stored_interval) = state.load(ptr) {
            saf_log!(absint::transfer, pts, "load exact ptr match"; interval=format!("{:?}", stored_interval));
            return stored_interval.clone();
        }
    }

    if found_any {
        saf_log!(absint::transfer, pts, "load final result"; result=format!("{:?}", result));
        result
    } else {
        saf_log!(absint::transfer, pts, "load no values found");
        Interval::make_top(DEFAULT_BITS)
    }
}

/// Handle load with PTA and GEP tracking (summaries variant).
// `ptr` (pointer operand), `pta` (analysis), and `pts` (points-to set) are distinct concepts
#[allow(clippy::similar_names)]
fn transfer_load_with_pta_and_gep(
    ptr: ValueId,
    dst: ValueId,
    state: &AbstractState,
    pta: &PtaIntegration<'_>,
) -> Interval {
    let mut result = Interval::make_bottom(DEFAULT_BITS);
    let mut found_any = false;

    // FIRST: Check field-sensitive GEP targets (struct field tracking)
    if let Some(field_targets) = state.resolve_field_gep(ptr) {
        for &(loc, offset) in field_targets {
            if let Some(interval) = state.load_field(loc, offset) {
                result = result.join(interval);
                found_any = true;
            }
        }
    }

    // SECOND: Check location-level GEP targets
    if !found_any {
        if let Some(gep_targets) = state.resolve_gep(ptr) {
            for loc in gep_targets {
                if let Some(interval) = state.load_loc(*loc) {
                    result = result.join(interval);
                    found_any = true;
                }
            }
        }
    }

    // THEN: Fall back to PTA-based tracking
    if !found_any {
        return transfer_load_with_pta(ptr, dst, state, pta);
    }

    result
}

/// Compute call return interval using summaries or inline analysis.
// `pta` (points-to analysis) and `pts` (points-to set) are distinct concepts
#[allow(clippy::similar_names, clippy::too_many_arguments)]
fn compute_call_return_with_summaries(
    inst: &Instruction,
    callee_id: FunctionId,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
) -> (Interval, bool) {
    // Try context-sensitive re-analysis for precise return interval
    if let Some(callee_func) = module
        .functions
        .iter()
        .find(|f| f.id == callee_id && !f.is_declaration)
    {
        // Compute actual argument intervals and points-to sets
        let mut arg_intervals: Vec<Interval> = Vec::new();
        let mut arg_pts: Vec<std::collections::BTreeSet<saf_core::ids::LocId>> = Vec::new();

        for (i, &arg_id) in inst.operands.iter().enumerate() {
            if i >= callee_func.params.len() {
                break;
            }
            arg_intervals.push(resolve_operand(arg_id, state, constant_map));

            // Collect points-to set for this argument
            let pts = if let Some(gep_targets) = state.resolve_gep(arg_id) {
                gep_targets.clone()
            } else {
                pta.points_to(arg_id)
            };
            arg_pts.push(pts);
        }

        // Collect caller's loc_memory for passing to callee
        let caller_loc_memory = state.loc_memory_entries().clone();

        // Only do inline analysis for small, simple functions
        let is_simple = callee_func.blocks.len() <= 5
            && callee_func
                .blocks
                .iter()
                .map(|b| b.instructions.len())
                .sum::<usize>()
                <= 40;

        // Check if the summary return interval is bottom (SCC being computed).
        // If so, skip inline analysis — use the bottom interval directly to
        // avoid infinite recursion in self-recursive functions.
        let summary_is_bottom = return_intervals
            .get(&callee_id)
            .is_some_and(super::domain::AbstractDomain::is_bottom);

        // Check if the callee is already on the inline stack (mutual recursion guard).
        let already_inlining = INLINE_STACK.with(|stack| stack.borrow().contains(&callee_id));

        if is_simple
            && !summary_is_bottom
            && !already_inlining
            && (!arg_intervals
                .iter()
                .all(super::domain::AbstractDomain::is_top)
                || arg_pts.iter().any(|pts| !pts.is_empty()))
        {
            // Push callee onto inline stack before analysis
            INLINE_STACK.with(|stack| stack.borrow_mut().insert(callee_id));

            let (result, callee_loc_memory) = analyze_callee_inline(
                callee_func,
                &arg_intervals,
                &arg_pts,
                &caller_loc_memory,
                constant_map,
                module,
                pta,
                return_intervals,
            );

            // Pop callee from inline stack after analysis
            INLINE_STACK.with(|stack| stack.borrow_mut().remove(&callee_id));

            // Propagate callee's loc_memory stores back to the caller for
            // locations that were passed as pointer arguments. This handles
            // pass-by-pointer patterns where the callee modifies *p.
            let mut arg_reachable_locs = std::collections::BTreeSet::new();
            for pts in &arg_pts {
                arg_reachable_locs.extend(pts.iter().copied());
                // Also include all locs sharing the same base object
                for loc in pts {
                    if let Some(obj) = pta.object_of_location(*loc) {
                        for callee_loc in callee_loc_memory.keys() {
                            if pta
                                .object_of_location(*callee_loc)
                                .is_some_and(|o| o == obj)
                            {
                                arg_reachable_locs.insert(*callee_loc);
                            }
                        }
                    }
                }
            }
            for (loc, interval) in &callee_loc_memory {
                if arg_reachable_locs.contains(loc) {
                    state.store_loc(*loc, interval.clone());
                }
            }

            return (result, true);
        }

        // Fall back to summary
        (
            return_intervals
                .get(&callee_id)
                .cloned()
                .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS)),
            false,
        )
    } else {
        // External function or not found: use summary or TOP
        (
            return_intervals
                .get(&callee_id)
                .cloned()
                .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS)),
            false,
        )
    }
}

// =============================================================================
// Memory side-effect helpers (Plan 084 Phase C)
// =============================================================================

/// Apply memory side-effects from a callee's summary to the caller state.
///
/// Maps each parameter index to the caller's actual argument, resolves PTA
/// targets, and stores the effect interval at those locations.
fn apply_memory_effects(
    inst: &Instruction,
    callee_id: FunctionId,
    state: &mut AbstractState,
    ctx: &TransferContext<'_>,
) {
    let Some(memory_summaries) = ctx.memory_summaries else {
        return;
    };
    let Some(effects) = memory_summaries.get(&callee_id) else {
        return;
    };
    let Some(pta) = ctx.pta else {
        return;
    };
    for (&param_idx, effect_interval) in effects {
        if param_idx < inst.operands.len() {
            let arg_id = inst.operands[param_idx];
            apply_effect_to_arg(arg_id, effect_interval, state, pta);
        }
    }
}

/// Apply global variable store effects from a callee's summary.
///
/// When a function stores a known interval to a global variable (e.g.,
/// `g = 3` in a recursive base case), this propagates that effect to
/// the caller's `loc_memory` at the call site.
fn apply_global_effects(
    callee_id: FunctionId,
    state: &mut AbstractState,
    ctx: &TransferContext<'_>,
) {
    let Some(global_summaries) = ctx.global_summaries else {
        return;
    };
    let Some(effects) = global_summaries.get(&callee_id) else {
        return;
    };
    for (&loc, interval) in effects {
        state.store_loc(loc, interval.clone());
    }
}

/// Store an interval to all PTA/GEP targets of an argument value.
fn apply_effect_to_arg(
    arg_id: ValueId,
    effect_interval: &Interval,
    state: &mut AbstractState,
    pta: &PtaIntegration<'_>,
) {
    let mut targets = pta.points_to(arg_id);
    if let Some(gep_targets) = state.resolve_gep(arg_id) {
        targets.extend(gep_targets.iter().copied());
    }
    for loc in targets {
        state.store_loc(loc, effect_interval.clone());
    }
}

// =============================================================================
// Legacy wrapper functions for backward compatibility
// =============================================================================

/// Apply the transfer function for a single instruction.
///
/// Updates `state` in place with the result of executing `inst`.
/// This is a thin wrapper around `transfer_instruction_with_context` with no context.
#[cfg(test)]
pub fn transfer_instruction(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
) {
    transfer_instruction_with_context(
        inst,
        state,
        constant_map,
        module,
        &TransferContext::default(),
    );
}

/// Apply the transfer function for a single instruction with PTA integration.
///
/// Uses PTA information for alias-aware memory operations:
/// - Store: writes to all locations in points-to set (strong/weak update)
/// - Load: joins intervals from all aliased locations
///
/// This is a thin wrapper around `transfer_instruction_with_context` with PTA but no summaries.
#[cfg(test)]
pub fn transfer_instruction_with_pta(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
) {
    let ctx = TransferContext::new(Some(pta), None);
    transfer_instruction_with_context(inst, state, constant_map, module, &ctx);
}

/// Apply the transfer function with PTA integration and function summaries.
///
/// This extends `transfer_instruction_with_pta` to use function return interval
/// summaries for `CallDirect` instead of returning TOP. This enables interprocedural
/// analysis where call return values propagate correctly through memory operations.
///
/// This is a thin wrapper around `transfer_instruction_with_context` with both PTA and summaries.
///
/// # Arguments
///
/// * `inst` - The instruction to transfer
/// * `state` - The abstract state (modified in place)
/// * `constant_map` - Map of constant values
/// * `module` - The AIR module (for function lookups)
/// * `pta` - PTA integration for alias-aware memory operations
/// * `return_intervals` - Map from `FunctionId` to return value intervals
pub fn transfer_instruction_with_pta_and_summaries(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
) {
    let ctx = TransferContext::new(Some(pta), Some(return_intervals));
    transfer_instruction_with_context(inst, state, constant_map, module, &ctx);
}

/// Propagate branch refinements from SSA values back to `loc_memory`.
///
/// For address-taken locals (which survive mem2reg), values are loaded from
/// alloca (`Load %ptr → %val`), then branches compare `%val`.
/// `refine_branch_condition` refines `%val` but not the alloca's `loc_memory`
/// entry. This function back-propagates: for each Load whose destination was
/// refined, update the source `loc_memory` entry so downstream loads pick up
/// the refined interval.
///
/// After mem2reg, promoted locals no longer have Load instructions, so this
/// function is primarily active for address-taken locals (structs passed by
/// pointer, arrays, globals).
#[allow(clippy::similar_names)]
pub fn propagate_refinement_to_loc_memory(
    refined: &mut AbstractState,
    block: &saf_core::air::AirBlock,
    pre_refinement: &AbstractState,
    pta: &PtaIntegration<'_>,
) {
    // Early return: if no Load instructions in this block, nothing to propagate.
    if !block
        .instructions
        .iter()
        .any(|i| matches!(i.op, Operation::Load))
    {
        return;
    }
    for inst in &block.instructions {
        if let Operation::Load = &inst.op {
            let Some(dst) = inst.dst else {
                continue;
            };
            // Check if this Load's result was refined
            let Some(refined_val) = refined.get_opt(dst).cloned() else {
                continue;
            };
            let Some(pre_val) = pre_refinement.get_opt(dst) else {
                continue;
            };
            if &refined_val == pre_val {
                continue; // No refinement happened
            }
            // Find which loc this was loaded from
            if let Some(&ptr_id) = inst.operands.first() {
                // Check GEP targets first
                let gep_locs = refined.resolve_gep(ptr_id).cloned();
                if let Some(gep_targets) = gep_locs {
                    for loc in gep_targets {
                        refined.store_loc(loc, refined_val.clone());
                    }
                } else {
                    let pts = pta.points_to(ptr_id);
                    if pts.is_empty() {
                        // PTA doesn't track this pointer (e.g., local alloca);
                        // propagate refinement through ValueId-based memory
                        // so subsequent loads from the same pointer pick it up.
                        refined.store(ptr_id, refined_val.clone());
                    } else {
                        for loc in pts {
                            refined.store_loc(loc, refined_val.clone());
                        }
                    }
                }
            }
        }
    }
}

/// Apply branch condition refinement on a `CondBr`.
///
/// Given the condition value (result of an ICmp), refines the operands
/// of that comparison on the true/false edges.
pub fn refine_branch_condition(
    cond_inst: &Instruction,
    state: &AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    take_true: bool,
) -> AbstractState {
    let mut refined = state.clone();

    // The condition instruction must be a BinaryOp (ICmp)
    let Operation::BinaryOp { kind } = &cond_inst.op else {
        return refined;
    };

    if cond_inst.operands.len() < 2 {
        return refined;
    }

    let lhs_id = cond_inst.operands[0];
    let rhs_id = cond_inst.operands[1];
    let lhs = resolve_operand(lhs_id, state, constant_map);
    let rhs = resolve_operand(rhs_id, state, constant_map);

    // Compute refined intervals for both operands.
    let (lhs_refined, rhs_refined) = match (kind, take_true) {
        // x < y is true OR x >= y is false → x < y
        (BinaryOp::ICmpSlt, true) | (BinaryOp::ICmpSge, false) => (
            Some(lhs.refine_slt_true(&rhs)),
            Some(rhs.refine_slt_false(&lhs)),
        ),
        // x < y is false → x >= y
        (BinaryOp::ICmpSlt, false) => (
            Some(lhs.refine_slt_false(&rhs)),
            Some(rhs.refine_slt_true(&lhs)),
        ),
        // x <= y is true OR x > y is false → x <= y
        (BinaryOp::ICmpSle, true) | (BinaryOp::ICmpSgt, false) => (
            Some(lhs.refine_sle_true(&rhs)),
            Some(rhs.refine_sle_false(&lhs)),
        ),
        // x <= y is false → x > y
        (BinaryOp::ICmpSle, false) => (
            Some(lhs.refine_sle_false(&rhs)),
            Some(rhs.refine_sle_true(&lhs)),
        ),
        // x > y is true → x > y, y < x
        (BinaryOp::ICmpSgt, true) => (
            Some(lhs.refine_sle_false(&rhs)),
            Some(rhs.refine_slt_true(&lhs)),
        ),
        // x >= y is true
        (BinaryOp::ICmpSge, true) => (
            Some(lhs.refine_slt_false(&rhs)),
            Some(rhs.refine_sle_true(&lhs)),
        ),
        // x == y is true OR x != y is false → equal
        (BinaryOp::ICmpEq, true) | (BinaryOp::ICmpNe, false) => (
            Some(lhs.refine_eq_true(&rhs)),
            Some(rhs.refine_eq_true(&lhs)),
        ),
        // x == y is false OR x != y is true → not equal
        (BinaryOp::ICmpEq, false) | (BinaryOp::ICmpNe, true) => (
            Some(lhs.refine_eq_false(&rhs)),
            Some(rhs.refine_eq_false(&lhs)),
        ),
        // Unsigned comparisons
        (BinaryOp::ICmpUlt, true) | (BinaryOp::ICmpUge, false) => {
            (Some(lhs.refine_slt_true(&rhs)), None)
        }
        (BinaryOp::ICmpUlt, false) | (BinaryOp::ICmpUge, true) => {
            (Some(lhs.refine_slt_false(&rhs)), None)
        }
        (BinaryOp::ICmpUle, true) | (BinaryOp::ICmpUgt, false) => {
            (Some(lhs.refine_sle_true(&rhs)), None)
        }
        (BinaryOp::ICmpUgt, true) | (BinaryOp::ICmpUle, false) => {
            (Some(lhs.refine_sle_false(&rhs)), None)
        }
        // Non-comparison ops: no refinement
        _ => (None, None),
    };

    // If the LHS refined operand is bottom, the branch is infeasible —
    // mark the entire state as unreachable (Plan 075: critical for recursive
    // function analysis where concrete params make one branch impossible).
    if let Some(ref lhs_iv) = lhs_refined {
        if lhs_iv.is_bottom() {
            return AbstractState::bottom();
        }
    }

    // Apply refinements, skipping bottom values. Bottom refinements for
    // constants are spurious — the constant's value can't change, but the
    // refinement formula may produce bottom when the constraint is
    // unsatisfiable for that operand (e.g., refining "5 < x" when x is
    // [1,5] gives bottom for the constant 5). Storing bottom in the state
    // would shadow the correct constant_map entry.
    if let Some(lhs_iv) = lhs_refined {
        refined.set(lhs_id, lhs_iv);
    }
    if let Some(rhs_iv) = rhs_refined {
        if !rhs_iv.is_bottom() {
            refined.set(rhs_id, rhs_iv);
        }
    }

    refined
}

/// Analyze a callee function inline with actual argument bindings.
///
/// This provides context-sensitive precision for small functions:
/// - Binds parameters to actual argument intervals
/// - Binds parameter GEP targets from caller's points-to info (Plan 062 Phase 3)
/// - Propagates caller's loc_memory for reachable locations (Plan 062 Phase 5)
/// - Runs a simplified single-pass analysis
/// - Returns the joined interval of all return statements
///
/// Used when the summary would be TOP due to parameter abstraction.
// NOTE: Inline callee analysis binds parameters, propagates state, runs single-pass
// analysis, and joins return values. Splitting would fragment the inline analysis.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn analyze_callee_inline(
    callee: &saf_core::air::AirFunction,
    arg_intervals: &[Interval],
    arg_pts: &[std::collections::BTreeSet<saf_core::ids::LocId>],
    caller_loc_memory: &BTreeMap<saf_core::ids::LocId, Interval>,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
) -> (Interval, BTreeMap<saf_core::ids::LocId, Interval>) {
    use super::domain::AbstractDomain;

    // Initialize state with parameter bindings
    let mut state = AbstractState::new();
    for (i, param) in callee.params.iter().enumerate() {
        // Bind interval
        let interval = arg_intervals
            .get(i)
            .cloned()
            .unwrap_or_else(|| Interval::make_top(64));
        state.set(param.id, interval);

        // Bind GEP targets if argument is a pointer with known points-to set
        // This enables the callee to track field-sensitive memory through parameters
        if let Some(locs) = arg_pts.get(i) {
            if !locs.is_empty() {
                saf_log!(absint::transfer, context, "inline binding param to GEP targets"; param=param.id, locs=format!("{:?}", locs));
                state.register_gep(param.id, locs.clone());

                // Collect ObjIds for all argument locations
                // This is needed to propagate ALL array elements, not just the base
                let arg_obj_ids: std::collections::BTreeSet<_> = locs
                    .iter()
                    .filter_map(|loc| pta.object_of_location(*loc))
                    .collect();

                // Propagate caller's memory for ALL locations that share the same base object
                // This is critical for array element tracking through function calls
                // Example: if arg points to arr[0], we propagate arr[0], arr[1], arr[2], etc.
                for (loc, interval) in caller_loc_memory {
                    let should_propagate = if arg_obj_ids.is_empty() {
                        // If we couldn't get ObjIds, fall back to exact match
                        locs.contains(loc)
                    } else if let Some(loc_obj) = pta.object_of_location(*loc) {
                        arg_obj_ids.contains(&loc_obj)
                    } else {
                        locs.contains(loc)
                    };

                    if should_propagate {
                        saf_log!(absint::transfer, context, "inline propagating loc_memory"; loc=*loc, interval=format!("{:?}", interval));
                        state.store_loc(*loc, interval.clone());
                    }
                }
            }
        }
    }

    // Build local constant map for this function
    let local_constants = build_constant_map_for_func(callee, constant_map);

    // Simple forward pass through blocks (no fixpoint, just single pass).
    // This is a heuristic — works for simple functions without complex control flow.
    let mut return_intervals_collected: Vec<Interval> = Vec::new();

    for block in &callee.blocks {
        for inst in &block.instructions {
            // Check for return instruction first
            if let Operation::Ret = &inst.op {
                if let Some(&return_operand) = inst.operands.first() {
                    let interval = resolve_operand(return_operand, &state, &local_constants);
                    return_intervals_collected.push(interval);
                }
                continue;
            }

            // Execute instruction
            transfer_instruction_with_pta_and_summaries(
                inst,
                &mut state,
                &local_constants,
                module,
                pta,
                return_intervals,
            );

            // Post-processing for inline callee: bridge PTA gaps in GEP/Load
            // chains that arise from type mismatches (e.g., `int**` over a
            // flat `int[2][2]`). PTA cannot track through the intermediate
            // `Load ptr` in these cases, leaving downstream GEP/Load with
            // empty points-to sets. We fix this by propagating GEP target
            // information from the source pointer through Loads and GEPs.
            if let Some(dst) = inst.dst {
                let dst_has_gep = state.resolve_gep(dst).is_some();
                if !dst_has_gep && pta.points_to(dst).is_empty() {
                    match &inst.op {
                        // Load fixup: propagate all same-object locations from
                        // the source pointer's GEP targets. The loaded pointer
                        // conceptually "dereferences" into sub-locations.
                        Operation::Load => {
                            if let Some(&src_ptr) = inst.operands.first() {
                                if let Some(src_gep_targets) = state.resolve_gep(src_ptr).cloned() {
                                    let mut descendant_locs = BTreeSet::new();
                                    for parent_loc in &src_gep_targets {
                                        for loc in pta.locations_of_same_object(*parent_loc) {
                                            if loc != *parent_loc {
                                                descendant_locs.insert(loc);
                                            }
                                        }
                                    }
                                    if !descendant_locs.is_empty() {
                                        saf_log!(absint::transfer, context, "inline load fixup"; dst=dst, locs=descendant_locs.len());
                                        state.register_gep(dst, descendant_locs);
                                    }
                                }
                            }
                        }
                        // GEP fixup: when PTA returned empty for this GEP but
                        // the base pointer has known GEP targets (from a prior
                        // Load fixup), narrow them using the GEP index operand.
                        Operation::Gep { .. } => {
                            if let Some(&base_ptr) = inst.operands.first() {
                                if let Some(base_gep_targets) = state.resolve_gep(base_ptr).cloned()
                                {
                                    // Use the last operand as the array/field index
                                    let index_operand = inst.operands[inst.operands.len() - 1];
                                    let index_iv =
                                        resolve_operand(index_operand, &state, &local_constants);
                                    let refined = pta.refine_gep_targets_by_child_index(
                                        &base_gep_targets,
                                        &index_iv,
                                    );
                                    if !refined.is_empty() && refined != base_gep_targets {
                                        saf_log!(absint::transfer, context, "inline GEP fixup"; dst=dst, locs=refined.len());
                                        state.register_gep(dst, refined);
                                    } else if !base_gep_targets.is_empty() {
                                        // Refinement didn't narrow — pass
                                        // through the base targets as-is so
                                        // downstream Loads can still find
                                        // loc_memory entries.
                                        state.register_gep(dst, base_gep_targets);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Join all return intervals
    let return_iv = if return_intervals_collected.is_empty() {
        Interval::make_top(DEFAULT_BITS)
    } else {
        let mut result = return_intervals_collected[0].clone();
        for interval in return_intervals_collected.iter().skip(1) {
            result = result.join(interval);
        }
        result
    };
    // Return both the interval and the callee's final loc_memory
    (return_iv, state.loc_memory_entries().clone())
}

/// Build constant map for a single function.
///
/// Constants in AIR are stored at the module level, not per-function.
/// This function just returns a clone of the global constant map.
fn build_constant_map_for_func(
    _func: &saf_core::air::AirFunction,
    global_constants: &BTreeMap<ValueId, Interval>,
) -> BTreeMap<ValueId, Interval> {
    global_constants.clone()
}

/// Seed `loc_memory` with constant values from global aggregate initializers.
///
/// When a global variable has a `Constant::Aggregate` initializer (e.g.,
/// `int a[2] = {1, 2}`), the individual element values are not stored via
/// explicit `Store` instructions — the IR encodes them as a constant
/// aggregate. This function pre-populates `loc_memory` so that GEP+Load
/// from these globals resolves to concrete intervals.
///
/// For each global with an aggregate initializer, finds all PTA locations
/// sharing the global's `ObjId`, matches their field path to the aggregate
/// element index, and stores scalar constants as singleton intervals.
pub fn seed_global_aggregate_constants(
    state: &mut AbstractState,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
) {
    use saf_core::air::Constant;

    let Some(pta_ref) = pta.pta_ref() else {
        return;
    };

    for global in &module.globals {
        // Extract element list: Aggregate elements or String bytes (as i8).
        let elements: Vec<Constant> = match &global.init {
            Some(Constant::Aggregate { elements: agg }) => agg.clone(),
            Some(Constant::String { value }) => {
                // Treat each byte of a string constant as an i8 element.
                value
                    .bytes()
                    .map(|b| Constant::Int {
                        value: i64::from(b),
                        bits: 8,
                    })
                    .collect()
            }
            _ => continue,
        };

        let obj = global.obj;

        // Find all PTA locations belonging to this global's object
        for (loc_id, location) in pta_ref.locations() {
            if location.obj != obj {
                continue;
            }

            // Match the last path step to an aggregate element index
            let element_index = match location.path.steps.last() {
                Some(crate::pta::PathStep::Field { index }) => Some(*index as usize),
                Some(crate::pta::PathStep::Index(crate::pta::IndexExpr::Constant(idx))) => {
                    // INVARIANT: aggregate element indices are non-negative and small
                    usize::try_from(*idx).ok()
                }
                _ => None,
            };

            let Some(idx) = element_index else {
                continue;
            };

            if idx >= elements.len() {
                continue;
            }

            // Store scalar constants as singleton intervals
            match &elements[idx] {
                Constant::Int { value, bits } => {
                    state.store_loc(*loc_id, Interval::singleton(i128::from(*value), *bits));
                }
                Constant::BigInt { value, bits } => {
                    if let Ok(v) = value.parse::<i128>() {
                        state.store_loc(*loc_id, Interval::singleton(v, *bits));
                    }
                }
                Constant::Null | Constant::Undef | Constant::ZeroInit => {
                    state.store_loc(*loc_id, Interval::singleton(0, DEFAULT_BITS));
                }
                // Nested aggregates, strings, floats, GlobalRef — not directly useful
                // for interval tracking. Pointer-valued elements ({&a, &b}) are
                // handled by PTA constraint extraction, not interval seeding.
                _ => {}
            }
        }
    }
}

/// Follow SSA phi/copy/cast chains (up to 5 hops) to find a `Constant::Float`.
///
/// After mem2reg, a float constant may flow through a phi node before reaching
/// an `FPToSI`/`FPToUI` cast. This function traces the def-use chain backwards
/// through phi incoming values, Copy sources, and Cast sources to find the
/// original float constant in `module.constants`.
///
/// Returns the float value if a unique constant is found, `None` otherwise.
fn find_float_constant_ssa(start: ValueId, module: &AirModule) -> Option<f64> {
    use saf_core::air::Constant;

    let mut worklist = vec![start];
    let mut visited = BTreeSet::new();
    let mut found_value: Option<f64> = None;

    for _hop in 0..5 {
        let mut next_worklist = Vec::new();

        for vid in &worklist {
            if !visited.insert(*vid) {
                continue;
            }

            // Check if this ValueId is a float constant
            if let Some(Constant::Float { value, .. }) = module.constants.get(vid) {
                match found_value {
                    None => found_value = Some(*value),
                    Some(existing) if (existing - *value).abs() < f64::EPSILON => {}
                    Some(_) => return None, // Multiple distinct float constants → give up
                }
                continue;
            }

            // Look for the instruction that defines this ValueId
            for func in &module.functions {
                if func.is_declaration {
                    continue;
                }
                for block in &func.blocks {
                    for inst in &block.instructions {
                        if inst.dst != Some(*vid) {
                            continue;
                        }
                        match &inst.op {
                            Operation::Phi { incoming } => {
                                for (_, val_id) in incoming {
                                    next_worklist.push(*val_id);
                                }
                            }
                            Operation::Copy | Operation::Cast { .. } => {
                                if let Some(&src) = inst.operands.first() {
                                    next_worklist.push(src);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if next_worklist.is_empty() {
            break;
        }
        worklist = next_worklist;
    }

    found_value
}

/// Compute the byte offset for a struct field access from a `FieldPath`.
///
/// Walks the `FieldPath` steps through the type hierarchy:
/// - `FieldStep::Field { index }`: looks up `fields[index].byte_offset` in the current struct type
/// - `FieldStep::Index`: skips (array index step — tracks type navigation only)
///
/// Returns the accumulated byte offset, or `None` if type info is missing.
fn compute_field_byte_offset(
    field_path: &saf_core::air::FieldPath,
    base_type: &AirType,
    type_registry: &BTreeMap<TypeId, AirType>,
) -> Option<u64> {
    let mut offset: u64 = 0;
    let mut current_type = base_type;

    for step in &field_path.steps {
        match step {
            FieldStep::Field { index } => {
                let idx = *index as usize;
                if let AirType::Struct { fields, .. } = current_type {
                    let field = fields.get(idx)?;
                    offset += field.byte_offset?;
                    // Navigate into the field type for further steps
                    if let Some(ty) = type_registry.get(&field.field_type) {
                        current_type = ty;
                    } else {
                        // Can't resolve further type — but we have the offset so far.
                        // If there are more steps, we'll fail on the next iteration.
                        // For a terminal step this is fine.
                        continue;
                    }
                } else {
                    // Not a struct — cannot resolve field offset
                    return None;
                }
            }
            FieldStep::Index => {
                // Array index step — navigate into the element type
                if let AirType::Array { element, .. } = current_type {
                    if let Some(ty) = type_registry.get(element) {
                        current_type = ty;
                    }
                }
                // For pointer-level index (first GEP operand), skip without error
            }
        }
    }

    Some(offset)
}

/// Build a map from `ObjId` (allocation site) to `TypeId` (struct type).
///
/// For each alloca with known `size_bytes`, finds a matching struct type in
/// the type registry by `total_size`. Only records unambiguous matches (one
/// struct type matches the alloca size).
pub fn build_obj_type_map(module: &AirModule) -> BTreeMap<ObjId, TypeId> {
    // Build size → TypeId index for struct types
    let mut size_to_types: BTreeMap<u64, Vec<TypeId>> = BTreeMap::new();
    for (type_id, air_type) in &module.types {
        if let AirType::Struct { total_size, .. } = air_type {
            if *total_size > 0 {
                size_to_types.entry(*total_size).or_default().push(*type_id);
            }
        }
    }

    let mut map = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Alloca {
                    size_bytes: Some(size),
                } = &inst.op
                {
                    // Match alloca size to struct type
                    if let Some(type_ids) = size_to_types.get(size) {
                        if type_ids.len() == 1 {
                            let obj = ObjId::new(inst.id.raw());
                            map.insert(obj, type_ids[0]);
                        }
                    }
                }
            }
        }
    }
    map
}

/// Resolve a computed return bound at a specific call site.
///
/// Uses the argument's allocation size (from alloca) to compute
/// a concrete interval for the return value.
///
/// # Known Limitations
///
/// - Only resolves allocation sizes via syntactic instruction tracing
///   (alloca -> GEP chain). Does not use PTA to resolve through
///   points-to sets, function parameters, or phi nodes. When the
///   argument is not a local alloca, returns TOP (sound fallback).
/// - Future: integrate PTA-based resolution for heap allocations
///   and cross-function pointer flows.
fn resolve_computed_bound(
    bound: &ComputedBound,
    inst: &Instruction,
    state: &AbstractState,
    module: &AirModule,
) -> Interval {
    let param_idx = bound.param_index as usize;
    if param_idx >= inst.operands.len() {
        return Interval::make_top(DEFAULT_BITS);
    }
    let arg_value = inst.operands[param_idx];

    match bound.mode {
        BoundMode::AllocSizeMinusOne | BoundMode::AllocSize => {
            // Try to find allocation size for this argument
            let alloc_size = find_argument_alloc_size(arg_value, module, 8);
            match alloc_size {
                Some(size) if size > 0 => {
                    let upper = if matches!(bound.mode, BoundMode::AllocSizeMinusOne) {
                        size - 1
                    } else {
                        size
                    };
                    Interval::new(0, upper, DEFAULT_BITS)
                }
                _ => Interval::make_top(DEFAULT_BITS),
            }
        }
        BoundMode::ParamValueMinusOne => {
            // Use the argument's interval value, not alloc size
            let arg_interval = state
                .get_opt(arg_value)
                .cloned()
                .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));
            if arg_interval.is_top() || arg_interval.is_bottom() {
                Interval::make_top(DEFAULT_BITS)
            } else {
                Interval::new(-1, arg_interval.hi() - 1, DEFAULT_BITS)
            }
        }
    }
}

/// Find the allocation size for a value that is a pointer argument.
///
/// Walks back through the instruction chain to find the originating
/// alloca, then returns its size in bytes.
///
/// # Performance
///
/// This function performs an O(M) scan over the entire module per call,
/// where M is the total number of instructions. The `depth` parameter
/// limits GEP chain recursion to prevent unbounded traversal on
/// pathological IR (practical GEP chains are typically 1-3 deep).
fn find_argument_alloc_size(value: ValueId, module: &AirModule, depth: u32) -> Option<i128> {
    if depth == 0 {
        return None;
    }
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.dst == Some(value) {
                    match &inst.op {
                        Operation::Alloca {
                            size_bytes: Some(size),
                        } => {
                            return Some(i128::from(*size));
                        }
                        Operation::Gep { .. } => {
                            // GEP from alloca — walk through to base
                            if let Some(base) = inst.operands.first() {
                                return find_argument_alloc_size(*base, module, depth - 1);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{BlockId, InstId};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }

    fn make_binop_inst(id: u128, kind: BinaryOp, lhs: u128, rhs: u128, dst: u128) -> Instruction {
        Instruction::new(iid(id), Operation::BinaryOp { kind })
            .with_operands(vec![vid(lhs), vid(rhs)])
            .with_dst(vid(dst))
    }

    fn make_cast_inst(id: u128, kind: CastKind, src: u128, dst: u128) -> Instruction {
        Instruction::new(
            iid(id),
            Operation::Cast {
                kind,
                target_bits: None,
            },
        )
        .with_operands(vec![vid(src)])
        .with_dst(vid(dst))
    }

    fn empty_module() -> AirModule {
        AirModule::new(saf_core::ids::ModuleId::derive(b"test"))
    }

    #[test]
    fn transfer_add() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 10, 32));
        state.set(vid(2), Interval::new(5, 15, 32));

        let inst = make_binop_inst(100, BinaryOp::Add, 1, 2, 3);
        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(3), 32);
        assert_eq!(result.lo(), 5);
        assert_eq!(result.hi(), 25);
    }

    #[test]
    fn transfer_sub() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(10, 20, 32));
        state.set(vid(2), Interval::new(1, 5, 32));

        let inst = make_binop_inst(100, BinaryOp::Sub, 1, 2, 3);
        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(3), 32);
        assert_eq!(result.lo(), 5);
        assert_eq!(result.hi(), 19);
    }

    #[test]
    fn transfer_mul() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(2, 4, 32));
        state.set(vid(2), Interval::new(3, 5, 32));

        let inst = make_binop_inst(100, BinaryOp::Mul, 1, 2, 3);
        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(3), 32);
        assert_eq!(result.lo(), 6);
        assert_eq!(result.hi(), 20);
    }

    #[test]
    fn transfer_comparison_produces_boolean() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 10, 32));
        state.set(vid(2), Interval::new(5, 15, 32));

        let inst = make_binop_inst(100, BinaryOp::ICmpSlt, 1, 2, 3);
        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(3), 1);
        assert!(result.contains(0));
        assert!(result.contains(1));
    }

    #[test]
    fn transfer_zext() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 100, 8));

        let inst = make_cast_inst(100, CastKind::ZExt, 1, 2);
        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(2), 64);
        assert_eq!(result.lo(), 0);
        assert_eq!(result.hi(), 100);
    }

    #[test]
    fn transfer_load_is_top() {
        let mut state = AbstractState::new();
        let inst = Instruction::new(iid(100), Operation::Load)
            .with_operands(vec![vid(1)])
            .with_dst(vid(2));
        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(2), 64);
        assert!(result.is_top());
    }

    #[test]
    fn transfer_store_then_load_tracks_value() {
        // Test that store followed by load propagates the value through memory
        let mut state = AbstractState::new();
        let module = empty_module();

        // Simulate constant 1 in constant_map
        let mut constant_map = BTreeMap::new();
        constant_map.insert(vid(1), Interval::singleton(1, 32));

        // Store: store vid(1) to ptr vid(2)
        // vid(1) = constant 1, vid(2) = pointer to alloca
        let store_inst =
            Instruction::new(iid(100), Operation::Store).with_operands(vec![vid(1), vid(2)]);

        transfer_instruction(&store_inst, &mut state, &constant_map, &module);

        // Load: load from ptr vid(2) into vid(3)
        let load_inst = Instruction::new(iid(101), Operation::Load)
            .with_operands(vec![vid(2)])
            .with_dst(vid(3));

        transfer_instruction(&load_inst, &mut state, &constant_map, &module);

        // vid(3) should have interval [1,1]
        let result = state.get(vid(3), 32);
        assert_eq!(result.lo(), 1);
        assert_eq!(result.hi(), 1);
    }

    #[test]
    fn transfer_phi_joins() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 5, 32));
        state.set(vid(2), Interval::new(10, 15, 32));

        let inst = Instruction::new(
            iid(100),
            Operation::Phi {
                incoming: vec![(BlockId::new(10), vid(1)), (BlockId::new(20), vid(2))],
            },
        )
        .with_dst(vid(3));

        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(3), 32);
        assert_eq!(result.lo(), 0);
        assert_eq!(result.hi(), 15);
    }

    #[test]
    fn transfer_select_joins() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 1, 1)); // condition
        state.set(vid(2), Interval::new(10, 20, 32)); // true val
        state.set(vid(3), Interval::new(30, 40, 32)); // false val

        let inst = Instruction::new(iid(100), Operation::Select)
            .with_operands(vec![vid(1), vid(2), vid(3)])
            .with_dst(vid(4));

        let constant_map = BTreeMap::new();
        let module = empty_module();

        transfer_instruction(&inst, &mut state, &constant_map, &module);

        let result = state.get(vid(4), 32);
        assert_eq!(result.lo(), 10);
        assert_eq!(result.hi(), 40);
    }

    #[test]
    fn branch_refinement_slt_true() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 100, 32)); // x
        state.set(vid(2), Interval::singleton(50, 32)); // 50

        let cond_inst = make_binop_inst(100, BinaryOp::ICmpSlt, 1, 2, 3);
        let constant_map = BTreeMap::new();

        let refined = refine_branch_condition(&cond_inst, &state, &constant_map, true);

        // x < 50 → x ∈ [0, 49]
        let x = refined.get(vid(1), 32);
        assert_eq!(x.lo(), 0);
        assert_eq!(x.hi(), 49);
    }

    #[test]
    fn branch_refinement_slt_false() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 100, 32)); // x
        state.set(vid(2), Interval::singleton(50, 32)); // 50

        let cond_inst = make_binop_inst(100, BinaryOp::ICmpSlt, 1, 2, 3);
        let constant_map = BTreeMap::new();

        let refined = refine_branch_condition(&cond_inst, &state, &constant_map, false);

        // !(x < 50) → x >= 50 → x ∈ [50, 100]
        let x = refined.get(vid(1), 32);
        assert_eq!(x.lo(), 50);
        assert_eq!(x.hi(), 100);
    }

    #[test]
    fn branch_refinement_eq_true() {
        let mut state = AbstractState::new();
        state.set(vid(1), Interval::new(0, 100, 32));
        state.set(vid(2), Interval::singleton(42, 32));

        let cond_inst = make_binop_inst(100, BinaryOp::ICmpEq, 1, 2, 3);
        let constant_map = BTreeMap::new();

        let refined = refine_branch_condition(&cond_inst, &state, &constant_map, true);

        // x == 42 → x ∈ [42, 42]
        let x = refined.get(vid(1), 32);
        assert!(x.is_singleton());
        assert_eq!(x.lo(), 42);
    }

    // =========================================================================
    // PTA-aware transfer function tests
    // =========================================================================

    use std::sync::Arc;

    use crate::absint::pta_integration::PtaIntegration;
    use crate::{FieldSensitivity, LocationFactory, PointsToMap, PtaDiagnostics, PtaResult};
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

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(pts, Arc::new(factory), PtaDiagnostics::default());
        let pta_integration = PtaIntegration::new(&pta);

        // Simulate constant 42
        let mut constant_map = BTreeMap::new();
        constant_map.insert(vid(1), Interval::singleton(42, 32));

        // Store: store vid(1) to ptr vid(2)
        let store_inst =
            Instruction::new(iid(100), Operation::Store).with_operands(vec![vid(1), vid(2)]);

        transfer_instruction_with_pta(
            &store_inst,
            &mut state,
            &constant_map,
            &module,
            &pta_integration,
        );

        // Load: load from ptr vid(3) into vid(4) - should get 42 via aliasing
        let load_inst = Instruction::new(iid(101), Operation::Load)
            .with_operands(vec![vid(3)])
            .with_dst(vid(4));

        transfer_instruction_with_pta(
            &load_inst,
            &mut state,
            &constant_map,
            &module,
            &pta_integration,
        );

        let result = state.get(vid(4), 32);
        assert_eq!(result.lo(), 42);
        assert_eq!(result.hi(), 42);
    }

    #[test]
    fn transfer_store_weak_update_with_pta() {
        let mut state = AbstractState::new();
        let module = empty_module();

        // Set up PTA: vid(2) points to {lid(100), lid(101)}
        let mut pts = PointsToMap::new();
        let mut set = std::collections::BTreeSet::new();
        set.insert(lid(100));
        set.insert(lid(101));
        pts.insert(vid(2), set);

        let factory = LocationFactory::new(FieldSensitivity::None);
        let pta = PtaResult::new(pts, Arc::new(factory), PtaDiagnostics::default());
        let pta_integration = PtaIntegration::new(&pta);

        // Pre-store values at both locations
        state.store_loc(lid(100), Interval::new(0, 10, 32));
        state.store_loc(lid(101), Interval::new(20, 30, 32));

        // Simulate constant 50
        let mut constant_map = BTreeMap::new();
        constant_map.insert(vid(1), Interval::singleton(50, 32));

        // Store: store vid(1) to ptr vid(2) - should weak update both locations
        let store_inst =
            Instruction::new(iid(100), Operation::Store).with_operands(vec![vid(1), vid(2)]);

        transfer_instruction_with_pta(
            &store_inst,
            &mut state,
            &constant_map,
            &module,
            &pta_integration,
        );

        // Check that both locations were weak-updated
        let loc100 = state.load_loc(lid(100)).unwrap();
        assert_eq!(loc100.lo(), 0); // join of [0,10] and [50,50]
        assert_eq!(loc100.hi(), 50);

        let loc101 = state.load_loc(lid(101)).unwrap();
        assert_eq!(loc101.lo(), 20); // join of [20,30] and [50,50]
        assert_eq!(loc101.hi(), 50);
    }

    #[test]
    fn transfer_fallback_without_pta_info() {
        let mut state = AbstractState::new();
        let module = empty_module();

        // Empty PTA - no points-to info for vid(2)
        let pta_integration = PtaIntegration::empty();

        let mut constant_map = BTreeMap::new();
        constant_map.insert(vid(1), Interval::singleton(42, 32));

        // Store: falls back to ValueId-based tracking
        let store_inst =
            Instruction::new(iid(100), Operation::Store).with_operands(vec![vid(1), vid(2)]);

        transfer_instruction_with_pta(
            &store_inst,
            &mut state,
            &constant_map,
            &module,
            &pta_integration,
        );

        // Load: should retrieve from ValueId-based memory
        let load_inst = Instruction::new(iid(101), Operation::Load)
            .with_operands(vec![vid(2)])
            .with_dst(vid(3));

        transfer_instruction_with_pta(
            &load_inst,
            &mut state,
            &constant_map,
            &module,
            &pta_integration,
        );

        let result = state.get(vid(3), 32);
        assert_eq!(result.lo(), 42);
        assert_eq!(result.hi(), 42);
    }

    // =========================================================================
    // Pure function inference tests
    // =========================================================================

    use saf_core::spec::SpecRegistry;

    #[test]
    fn pure_function_explicit_spec() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strlen
                pure: true
            "#,
        )
        .unwrap();

        assert!(is_pure_function_with_specs("strlen", Some(&registry)));
    }

    #[test]
    fn pure_function_inferred_from_reads_only() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strcmp
                params:
                  - index: 0
                    reads: true
                  - index: 1
                    reads: true
            "#,
        )
        .unwrap();

        // strcmp has only reads params, no modifies, no escapes → inferred pure
        assert!(is_pure_function_with_specs("strcmp", Some(&registry)));
    }

    #[test]
    fn pure_function_not_inferred_with_modifies() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: strcpy
                params:
                  - index: 0
                    modifies: true
                  - index: 1
                    reads: true
            "#,
        )
        .unwrap();

        // strcpy has modifies → not pure
        assert!(!is_pure_function_with_specs("strcpy", Some(&registry)));
    }

    #[test]
    fn pure_function_allocator_never_pure() {
        let registry = SpecRegistry::from_yaml(
            r#"
            version: "1.0"
            specs:
              - name: malloc
                role: allocator
            "#,
        )
        .unwrap();

        // Allocators are never pure
        assert!(!is_pure_function_with_specs("malloc", Some(&registry)));
    }

    #[test]
    fn pure_function_hardcoded_fallback() {
        // Without specs, should fall back to hardcoded list
        assert!(is_pure_function_with_specs("llvm.dbg.value", None));
        assert!(is_pure_function_with_specs("llvm.sin.f64", None));
        assert!(!is_pure_function_with_specs("unknown_func", None));
    }
}
