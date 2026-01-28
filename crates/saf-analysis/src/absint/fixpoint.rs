//! Forward fixpoint iterator with interleaved widening/narrowing.
//!
//! Implements the standard worklist-based forward abstract interpretation
//! algorithm. Loop headers are detected via back edges (DFS) and receive
//! widening during the ascending phase. A configurable narrowing phase
//! follows to recover precision.
//!
//! ## Noreturn Call Handling
//!
//! When a block ends with a call to a noreturn function (like `exit()` or `abort()`),
//! successors of that block are not added to the worklist since they are unreachable.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::{AirBlock, AirFunction, AirModule, BinaryOp, Constant, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, LocId, ObjId, TypeId, ValueId};
use saf_core::spec::{AnalyzedSpecRegistry, SpecRegistry};

use crate::absint::function_properties::is_noreturn_with_specs;
use crate::cfg::Cfg;
use crate::pta::ptsset::IdBitSet;

use super::config::AbstractInterpConfig;
use super::domain::AbstractDomain;
use super::interval::Interval;
use super::partition::{PartitionKey, PartitionToken, PartitionedState};
use super::pta_integration::PtaIntegration;
use super::result::AbstractInterpResult;
use super::state::AbstractState;
use super::threshold::extract_thresholds;
use super::transfer::{
    TransferContext, build_constant_map, build_obj_type_map, propagate_refinement_to_loc_memory,
    refine_branch_condition, seed_global_aggregate_constants, transfer_instruction_with_context,
};

/// Context for abstract interpretation fixpoint solver.
///
/// This struct encapsulates optional components that can enhance the fixpoint
/// analysis with additional capabilities like pointer analysis, spec-based
/// noreturn pruning, and interprocedural summaries.
#[derive(Default)]
pub struct FixpointContext<'a> {
    /// Optional pointer analysis for alias-aware memory tracking.
    pub pta: Option<&'a PtaIntegration<'a>>,
    /// Optional analyzed spec registry for noreturn function detection and return bounds.
    pub specs: Option<&'a AnalyzedSpecRegistry>,
    /// Optional function return intervals for interprocedural analysis.
    pub return_intervals: Option<&'a BTreeMap<FunctionId, Interval>>,
    /// Optional memory side-effect summaries for interprocedural propagation.
    pub memory_summaries: Option<&'a BTreeMap<FunctionId, BTreeMap<usize, Interval>>>,
    /// Optional global variable store summaries for interprocedural propagation.
    pub global_summaries: Option<&'a BTreeMap<FunctionId, BTreeMap<LocId, Interval>>>,
    /// Optional map from allocation `ObjId` to struct `TypeId` for field-sensitive tracking.
    pub obj_type_map: Option<&'a BTreeMap<ObjId, TypeId>>,
    /// Optional set of blocks proven unreachable by SCCP.
    pub dead_blocks: Option<&'a BTreeSet<BlockId>>,
}

impl<'a> FixpointContext<'a> {
    /// Create a new fixpoint context with the given components.
    #[must_use]
    pub fn new(
        pta: Option<&'a PtaIntegration<'a>>,
        specs: Option<&'a AnalyzedSpecRegistry>,
        return_intervals: Option<&'a BTreeMap<FunctionId, Interval>>,
    ) -> Self {
        Self {
            pta,
            specs,
            return_intervals,
            memory_summaries: None,
            global_summaries: None,
            obj_type_map: None,
            dead_blocks: None,
        }
    }
}

/// Diagnostics from the fixpoint computation.
#[derive(Debug, Clone)]
pub struct FixpointDiagnostics {
    /// Total blocks analyzed across all functions.
    pub blocks_analyzed: u64,
    /// Number of widening applications.
    pub widening_applications: u64,
    /// Number of narrowing iterations performed.
    pub narrowing_iterations_performed: u32,
    /// Whether the analysis converged.
    pub converged: bool,
    /// Number of functions analyzed.
    pub functions_analyzed: u64,
}

impl Default for FixpointDiagnostics {
    fn default() -> Self {
        Self {
            blocks_analyzed: 0,
            widening_applications: 0,
            narrowing_iterations_performed: 0,
            converged: true,
            functions_analyzed: 0,
        }
    }
}

/// Run abstract interpretation on the entire module.
///
/// Analyzes each non-declaration function independently using
/// forward fixpoint iteration with widening and narrowing.
#[must_use]
pub fn solve_abstract_interp(
    module: &AirModule,
    config: &AbstractInterpConfig,
) -> AbstractInterpResult {
    solve_abstract_interp_with_context(module, config, &FixpointContext::default())
}

/// Run abstract interpretation with noreturn function pruning.
///
/// This extends `solve_abstract_interp` with the ability to prune unreachable
/// successors after noreturn function calls (like `exit()`, `abort()`, etc.).
///
/// When a block contains a call to a noreturn function, its successors are
/// not added to the worklist since they cannot be reached at runtime.
#[must_use]
pub fn solve_abstract_interp_with_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    specs: Option<&AnalyzedSpecRegistry>,
) -> AbstractInterpResult {
    let ctx = FixpointContext::new(None, specs, None);
    solve_abstract_interp_with_context(module, config, &ctx)
}

/// Run abstract interpretation with PTA-aware memory tracking.
///
/// This version uses pointer analysis to track memory locations correctly,
/// enabling precise tracking of values through store/load sequences even
/// when different GEP instructions produce different pointer `ValueIds` that
/// point to the same memory location.
///
/// This is essential for tests like `svf_assert(a[b][c] == 8)` where the
/// stored value and loaded value go through different GEP computations.
#[must_use]
pub fn solve_abstract_interp_with_pta(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
) -> AbstractInterpResult {
    let ctx = FixpointContext::new(Some(pta), None, None);
    solve_abstract_interp_with_context(module, config, &ctx)
}

/// Run abstract interpretation with PTA-aware memory tracking and function summaries.
///
/// This is the key function for interprocedural memory propagation. Unlike
/// `solve_abstract_interp_with_pta`, this version uses function return interval
/// summaries during the transfer of `CallDirect` instructions. This means:
///
/// 1. Call return values get the summary interval (not TOP)
/// 2. Subsequent store/load operations preserve the refined interval
/// 3. Assertions that check loaded values can prove conditions
///
/// # Arguments
///
/// * `module` - The AIR module to analyze
/// * `config` - Analysis configuration
/// * `pta` - PTA integration for alias-aware memory operations
/// * `return_intervals` - Map from `FunctionId` to return value intervals
#[must_use]
pub fn solve_abstract_interp_with_pta_and_summaries(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
    memory_summaries: Option<&BTreeMap<FunctionId, BTreeMap<usize, Interval>>>,
    global_summaries: Option<&BTreeMap<FunctionId, BTreeMap<LocId, Interval>>>,
) -> AbstractInterpResult {
    let mut ctx = FixpointContext::new(Some(pta), None, Some(return_intervals));
    ctx.memory_summaries = memory_summaries;
    ctx.global_summaries = global_summaries;
    solve_abstract_interp_with_context(module, config, &ctx)
}

/// Run abstract interpretation with a unified context.
///
/// This is the core implementation that all public entry points delegate to.
/// The behavior is controlled by the `FixpointContext`:
///
/// - `ctx.pta`: Enables alias-aware memory tracking via PTA
/// - `ctx.specs`: Enables noreturn function detection and successor pruning
/// - `ctx.return_intervals`: Enables interprocedural summary application
#[must_use]
pub fn solve_abstract_interp_with_context(
    module: &AirModule,
    config: &AbstractInterpConfig,
    ctx: &FixpointContext<'_>,
) -> AbstractInterpResult {
    let mut constant_map = build_constant_map(module);

    // Run SCCP pre-pass to identify constants and dead blocks.
    let sccp_result = super::sccp::run_sccp_module(module);
    for (vid, val) in &sccp_result.constants {
        constant_map
            .entry(*vid)
            .or_insert_with(|| Interval::singleton(*val, 64));
    }

    let thresholds = if config.use_threshold_widening {
        extract_thresholds(module)
    } else {
        BTreeSet::new()
    };

    // Build function name map for noreturn checking (only needed if specs provided)
    let func_names = if ctx.specs.is_some() {
        build_func_names(module)
    } else {
        BTreeMap::new()
    };

    // Build obj→type map for field-sensitive struct tracking (only when PTA available)
    let obj_type_map = if ctx.pta.is_some() {
        build_obj_type_map(module)
    } else {
        BTreeMap::new()
    };
    let enriched_ctx = FixpointContext {
        pta: ctx.pta,
        specs: ctx.specs,
        return_intervals: ctx.return_intervals,
        memory_summaries: ctx.memory_summaries,
        global_summaries: ctx.global_summaries,
        obj_type_map: if obj_type_map.is_empty() {
            ctx.obj_type_map
        } else {
            Some(&obj_type_map)
        },
        dead_blocks: if sccp_result.dead_blocks.is_empty() {
            None
        } else {
            Some(&sccp_result.dead_blocks)
        },
    };
    let ctx = &enriched_ctx;

    let mut all_block_states: BTreeMap<BlockId, AbstractState> = BTreeMap::new();
    let mut all_inst_states: BTreeMap<InstId, AbstractState> = BTreeMap::new();
    let mut diagnostics = FixpointDiagnostics::default();

    for func in &module.functions {
        if func.is_declaration || func.blocks.is_empty() {
            continue;
        }

        diagnostics.functions_analyzed += 1;
        let cfg = Cfg::build(func);

        let (block_states, inst_states, func_diag) = solve_function_impl(
            func,
            &cfg,
            config,
            &constant_map,
            &thresholds,
            module,
            ctx,
            &func_names,
        );

        all_block_states.extend(block_states);
        all_inst_states.extend(inst_states);
        diagnostics.blocks_analyzed += func_diag.blocks_analyzed;
        diagnostics.widening_applications += func_diag.widening_applications;
        if !func_diag.converged {
            diagnostics.converged = false;
        }
    }

    diagnostics.narrowing_iterations_performed = config.narrowing_iterations;

    AbstractInterpResult::new(all_block_states, all_inst_states, constant_map, diagnostics)
}

/// Unified fixpoint solver for a single function.
///
/// This replaces the duplicated `solve_function`, `solve_function_with_noreturn`,
/// `solve_function_with_pta`, and `solve_function_with_pta_and_summaries` functions
/// with a single implementation that dispatches based on the context.
// NOTE: This function implements the fixpoint iteration algorithm as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
fn solve_function_impl(
    func: &AirFunction,
    cfg: &Cfg,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    thresholds: &BTreeSet<i128>,
    module: &AirModule,
    ctx: &FixpointContext<'_>,
    func_names: &BTreeMap<FunctionId, &str>,
) -> (
    BTreeMap<BlockId, AbstractState>,
    BTreeMap<InstId, AbstractState>,
    FixpointDiagnostics,
) {
    let mut diag = FixpointDiagnostics::default();

    let cond_inst_map = build_cond_inst_map(func);
    let loop_headers = detect_loop_headers(cfg);
    let loop_bounds = extract_loop_bound_constants(cfg, &loop_headers, func, module);

    // Pre-compute which blocks contain Load instructions, so
    // `propagate_refinement_to_loc_memory` is only called for blocks that
    // actually have loads (pure optimization — the function returns early
    // anyway, but this avoids two calls + two scans per branch edge).
    let blocks_with_loads: BTreeSet<BlockId> = func
        .blocks
        .iter()
        .filter(|b| {
            b.instructions
                .iter()
                .any(|i| matches!(i.op, Operation::Load))
        })
        .map(|b| b.id)
        .collect();

    let mut block_entry_states: BTreeMap<BlockId, PartitionedState> = BTreeMap::new();
    for block in &func.blocks {
        block_entry_states.insert(
            block.id,
            PartitionedState::from_single(AbstractState::bottom()),
        );
    }

    let mut entry_state = AbstractState::new();
    for param in &func.params {
        entry_state.set(param.id, Interval::make_top(64));
    }

    // Seed loc_memory with constant values from global aggregate initializers
    // (Plan 084 Phase B). This pre-populates intervals for globals like
    // `int a[2] = {1, 2}` so that GEP+Load resolves to concrete values.
    if let Some(pta) = ctx.pta {
        seed_global_aggregate_constants(&mut entry_state, module, pta);
    }

    block_entry_states.insert(cfg.entry, PartitionedState::from_single(entry_state));

    // Track conditional refinements: for each block, the set of ValueId
    // refinements inherited from dominating branch conditions. After
    // join/widen at block entry, these are re-applied via meet so that
    // branch narrowing survives fixpoint iteration (Plan 084 Phase A).
    let mut block_refinements: BTreeMap<BlockId, BTreeMap<ValueId, Interval>> = BTreeMap::new();

    // =================================================================
    // Ascending phase (widening)
    // =================================================================
    let mut worklist: VecDeque<BlockId> = VecDeque::new();
    worklist.push_back(cfg.entry);
    let mut iteration_count: u32 = 0;

    while let Some(block_id) = worklist.pop_front() {
        iteration_count += 1;
        #[allow(clippy::cast_possible_truncation)]
        if iteration_count > config.max_widening_iterations * (func.blocks.len() as u32 + 1) {
            diag.converged = false;
            break;
        }
        diag.blocks_analyzed += 1;

        let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
            continue;
        };

        let partitioned_entry = block_entry_states
            .get(&block_id)
            .cloned()
            .unwrap_or_else(|| PartitionedState::from_single(AbstractState::bottom()));

        if partitioned_entry.is_unreachable() {
            continue;
        }

        // Merge partitions for transfer function execution
        let entry_state = partitioned_entry.merge_all();

        // Skip SCCP-proven dead blocks.
        if let Some(dead) = ctx.dead_blocks {
            if dead.contains(&block_id) {
                continue;
            }
        }

        // Compute reached blocks for phi predecessor filtering
        let mut reached = IdBitSet::<BlockId>::empty();
        for (id, ps) in &block_entry_states {
            if !ps.is_unreachable() {
                reached.insert(*id);
            }
        }

        // Execute instructions with context-appropriate transfer function
        let mut current_state = entry_state.clone();
        for inst in &block.instructions {
            apply_transfer(
                inst,
                &mut current_state,
                constant_map,
                module,
                ctx,
                Some(&reached),
            );
        }

        // Skip successors if block ends with noreturn call (only when specs provided)
        if ctx.specs.is_some()
            && block_ends_noreturn(block, func_names, ctx.specs.map(AnalyzedSpecRegistry::yaml))
        {
            continue;
        }

        // Propagate to successors
        let terminator = block.terminator();
        let succs = cfg.successors_of(block_id).cloned().unwrap_or_default();

        for succ_id in &succs {
            let mut propagated_state = refine_for_successor(
                terminator,
                *succ_id,
                &current_state,
                &cond_inst_map,
                constant_map,
            );

            // Propagate branch refinements to loc_memory (critical for -O0 alloca pattern).
            if let Some(pta) = ctx.pta {
                propagate_loc_memory_for_edge(
                    &mut propagated_state,
                    block,
                    block_id,
                    *succ_id,
                    &current_state,
                    pta,
                    func,
                    &blocks_with_loads,
                );
            }

            // Record refinements for persistence (Plan 084 Phase A).
            // Only applies to CondBr edges during the ascending phase.
            if let Some(term) = terminator {
                if let Operation::CondBr {
                    then_target,
                    else_target,
                } = &term.op
                {
                    if !propagated_state.is_unreachable() {
                        let take_true = *succ_id == *then_target;
                        if let Some(&cond_operand) = term.operands.first() {
                            if let Some(cond_inst) = cond_inst_map.get(&cond_operand) {
                                let other_target = if take_true {
                                    *else_target
                                } else {
                                    *then_target
                                };
                                let other_refined = refine_branch_condition(
                                    cond_inst,
                                    &current_state,
                                    constant_map,
                                    !take_true,
                                );
                                if other_refined.is_unreachable() {
                                    // The other branch is infeasible — this refinement
                                    // is unconditional and should survive joins.
                                    let mut refinements = BTreeMap::new();
                                    collect_refinements(
                                        &current_state,
                                        &propagated_state,
                                        &mut refinements,
                                    );
                                    if !refinements.is_empty() {
                                        block_refinements
                                            .entry(*succ_id)
                                            .or_default()
                                            .extend(refinements);
                                    }
                                } else if succs.len() == 2 {
                                    // Both branches feasible but only two successors —
                                    // persist refinements when the successor has exactly
                                    // one predecessor so re-applying after join is safe.
                                    let _ = other_target;
                                    let pred_count = cfg
                                        .predecessors_of(*succ_id)
                                        .map_or(0, std::collections::BTreeSet::len);
                                    let mut refinements = BTreeMap::new();
                                    collect_refinements(
                                        &current_state,
                                        &propagated_state,
                                        &mut refinements,
                                    );
                                    if pred_count <= 1 && !refinements.is_empty() {
                                        block_refinements
                                            .entry(*succ_id)
                                            .or_default()
                                            .extend(refinements);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Determine if this edge is a partition split point
            let should_split = config.partition.enabled
                && terminator.is_some()
                && is_partition_split_point(terminator, *succ_id, &cond_inst_map, constant_map);

            // Wrap propagated state with partition key
            let propagated_partitioned = if should_split {
                if let Some(term) = terminator {
                    if let (Operation::CondBr { then_target, .. }, Some(&cond_operand)) =
                        (&term.op, term.operands.first())
                    {
                        let take_true = *succ_id == *then_target;
                        let mut key = PartitionKey::new();
                        key.push(PartitionToken::Branch {
                            cond_id: cond_operand,
                            taken: take_true,
                        });
                        let mut ps = PartitionedState::empty();
                        ps.insert(key, propagated_state.clone());
                        ps
                    } else {
                        PartitionedState::from_single(propagated_state.clone())
                    }
                } else {
                    PartitionedState::from_single(propagated_state.clone())
                }
            } else {
                PartitionedState::from_single(propagated_state.clone())
            };

            let old_partitioned = block_entry_states
                .get(succ_id)
                .cloned()
                .unwrap_or_else(|| PartitionedState::from_single(AbstractState::bottom()));

            let new_partitioned = if loop_headers.contains(*succ_id) {
                // At loop header: merge all partitions, widen, wrap back
                diag.widening_applications += 1;
                let merged_thresholds = if let Some(bounds) = loop_bounds.get(succ_id) {
                    let mut m = thresholds.clone();
                    m.extend(bounds.iter().copied());
                    m
                } else {
                    thresholds.clone()
                };
                let old_merged = old_partitioned.merge_all();
                let prop_merged = propagated_partitioned.merge_all();
                let mut widened = widen_state(&old_merged, &prop_merged, &merged_thresholds);
                // Re-apply persisted refinements after widening
                if let Some(refinements) = block_refinements.get(succ_id) {
                    apply_refinements(&mut widened, refinements);
                }
                PartitionedState::from_single(widened)
            } else {
                let mut joined = old_partitioned.join(&propagated_partitioned);
                joined.reduce_to_budget(config.partition.max_partitions);
                // Re-apply persisted refinements after join (Plan 084 Phase A).
                if let Some(refinements) = block_refinements.get(succ_id) {
                    for state in joined.partitions_mut() {
                        apply_refinements(state, refinements);
                    }
                }
                joined
            };

            // State change detection on partitioned state
            let state_changed =
                !new_partitioned.leq(&old_partitioned) || old_partitioned.is_unreachable();
            if state_changed {
                block_entry_states.insert(*succ_id, new_partitioned);
                if !worklist.contains(succ_id) {
                    worklist.push_back(*succ_id);
                }
            }
        }
    }

    // =================================================================
    // Descending phase (narrowing)
    // =================================================================
    let rpo = reverse_postorder(cfg);

    for _narrow_iter in 0..config.narrowing_iterations {
        let mut changed = false;

        for &block_id in &rpo {
            let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
                continue;
            };

            let entry_state = block_entry_states
                .get(&block_id)
                .cloned()
                .unwrap_or_else(|| PartitionedState::from_single(AbstractState::bottom()))
                .merge_all();

            if entry_state.is_unreachable() {
                continue;
            }

            // Compute reached blocks for phi predecessor filtering
            let mut reached = IdBitSet::<BlockId>::empty();
            for (id, ps) in &block_entry_states {
                if !ps.is_unreachable() {
                    reached.insert(*id);
                }
            }

            let mut current_state = entry_state.clone();
            for inst in &block.instructions {
                apply_transfer(
                    inst,
                    &mut current_state,
                    constant_map,
                    module,
                    ctx,
                    Some(&reached),
                );
            }

            // Skip successors if block ends with noreturn call (only when specs provided)
            if ctx.specs.is_some()
                && block_ends_noreturn(block, func_names, ctx.specs.map(AnalyzedSpecRegistry::yaml))
            {
                continue;
            }

            let terminator = block.terminator();
            let succs = cfg.successors_of(block_id).cloned().unwrap_or_default();

            for succ_id in &succs {
                let mut propagated_state = refine_for_successor(
                    terminator,
                    *succ_id,
                    &current_state,
                    &cond_inst_map,
                    constant_map,
                );

                // Propagate branch refinements to loc_memory.
                if let Some(pta) = ctx.pta {
                    propagate_loc_memory_for_edge(
                        &mut propagated_state,
                        block,
                        block_id,
                        *succ_id,
                        &current_state,
                        pta,
                        func,
                        &blocks_with_loads,
                    );
                }

                let old_merged = block_entry_states
                    .get(succ_id)
                    .cloned()
                    .unwrap_or_else(|| PartitionedState::from_single(AbstractState::bottom()))
                    .merge_all();

                let joined = old_merged.join(&propagated_state);
                let mut narrowed = narrow_state(&old_merged, &joined, thresholds);

                // Re-apply persisted refinements after narrowing (Plan 084 Phase A).
                if let Some(refinements) = block_refinements.get(succ_id) {
                    apply_refinements(&mut narrowed, refinements);
                }

                if narrowed != old_merged {
                    block_entry_states.insert(*succ_id, PartitionedState::from_single(narrowed));
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    // =================================================================
    // Compute per-instruction states
    // =================================================================
    let mut inst_states: BTreeMap<InstId, AbstractState> = BTreeMap::new();
    let mut reached = IdBitSet::<BlockId>::empty();
    for (id, ps) in &block_entry_states {
        if !ps.is_unreachable() {
            reached.insert(*id);
        }
    }

    for block in &func.blocks {
        let entry_state = block_entry_states
            .get(&block.id)
            .cloned()
            .unwrap_or_else(|| PartitionedState::from_single(AbstractState::bottom()))
            .merge_all();

        if entry_state.is_unreachable() {
            continue;
        }

        let mut current_state = entry_state;
        for inst in &block.instructions {
            inst_states.insert(inst.id, current_state.clone());
            apply_transfer(
                inst,
                &mut current_state,
                constant_map,
                module,
                ctx,
                Some(&reached),
            );
        }
    }

    // Merge partitioned states to flat block states for return
    let merged_block_states: BTreeMap<BlockId, AbstractState> = block_entry_states
        .into_iter()
        .map(|(bid, ps)| (bid, ps.merge_all()))
        .collect();

    (merged_block_states, inst_states, diag)
}

/// Apply the appropriate transfer function based on context.
///
/// Builds a `TransferContext` from the `FixpointContext` and delegates to the
/// unified `transfer_instruction_with_context`. When `reached_blocks` is
/// provided, phi nodes skip incoming values from unreachable predecessors.
fn apply_transfer(
    inst: &Instruction,
    state: &mut AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    ctx: &FixpointContext<'_>,
    reached_blocks: Option<&IdBitSet<BlockId>>,
) {
    let transfer_ctx = TransferContext {
        pta: ctx.pta,
        return_intervals: ctx.return_intervals,
        reached_blocks,
        memory_summaries: ctx.memory_summaries,
        global_summaries: ctx.global_summaries,
        specs: ctx.specs,
        obj_type_map: ctx.obj_type_map,
    };
    transfer_instruction_with_context(inst, state, constant_map, module, &transfer_ctx);
}

/// Analyze a function with specific parameter bindings (for interprocedural analysis).
///
/// This performs a simplified forward analysis with bound parameters to compute
/// the return value interval for a specific calling context.
// NOTE: This function implements context-sensitive fixpoint iteration as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn solve_function_with_params(
    func: &AirFunction,
    config: &AbstractInterpConfig,
    param_bindings: &BTreeMap<ValueId, Interval>,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
) -> BTreeMap<ValueId, Interval> {
    let thresholds = if config.use_threshold_widening {
        extract_thresholds(module)
    } else {
        BTreeSet::new()
    };

    let cfg = Cfg::build(func);

    // Create a modified constant map with parameter bindings
    let mut extended_constant_map = constant_map.clone();
    extended_constant_map.extend(param_bindings.clone());

    // Build condition instruction map for branch refinement
    let cond_inst_map = build_cond_inst_map(func);

    // Detect loop headers
    let loop_headers = detect_loop_headers(&cfg);
    let loop_bounds = extract_loop_bound_constants(&cfg, &loop_headers, func, module);

    // Initialize states
    let mut block_entry_states: BTreeMap<BlockId, AbstractState> = BTreeMap::new();
    for block in &func.blocks {
        block_entry_states.insert(block.id, AbstractState::bottom());
    }

    // Entry block: use provided parameter bindings
    let mut entry_state = AbstractState::new();
    for param in &func.params {
        if let Some(binding) = param_bindings.get(&param.id) {
            entry_state.set(param.id, binding.clone());
        } else {
            entry_state.set(param.id, Interval::make_top(64));
        }
    }
    block_entry_states.insert(cfg.entry, entry_state);

    // Worklist-based forward analysis
    let mut worklist: VecDeque<BlockId> = VecDeque::new();
    worklist.push_back(cfg.entry);
    let mut iteration_count: u32 = 0;

    while let Some(block_id) = worklist.pop_front() {
        iteration_count += 1;
        #[allow(clippy::cast_possible_truncation)]
        if iteration_count > config.max_widening_iterations * (func.blocks.len() as u32 + 1) {
            break;
        }

        let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
            continue;
        };

        let block_entry = block_entry_states
            .get(&block_id)
            .cloned()
            .unwrap_or_else(AbstractState::bottom);

        if block_entry.is_unreachable() {
            continue;
        }

        // Compute reached blocks for phi predecessor filtering
        let mut reached = IdBitSet::<BlockId>::empty();
        for (id, s) in &block_entry_states {
            if !s.is_unreachable() {
                reached.insert(*id);
            }
        }
        let phi_ctx = TransferContext {
            reached_blocks: Some(&reached),
            ..TransferContext::default()
        };

        let mut current_state = block_entry.clone();
        for inst in &block.instructions {
            transfer_instruction_with_context(
                inst,
                &mut current_state,
                &extended_constant_map,
                module,
                &phi_ctx,
            );
        }

        // Propagate to successors
        let terminator = block.terminator();
        let succs = cfg.successors_of(block_id).cloned().unwrap_or_default();

        for succ_id in &succs {
            let propagated_state = if let Some(term) = terminator {
                if let Operation::CondBr { then_target, .. } = &term.op {
                    let take_true = *succ_id == *then_target;
                    if let Some(&cond_operand) = term.operands.first() {
                        if let Some(cond_inst) = cond_inst_map.get(&cond_operand) {
                            refine_branch_condition(
                                cond_inst,
                                &current_state,
                                &extended_constant_map,
                                take_true,
                            )
                        } else {
                            current_state.clone()
                        }
                    } else {
                        current_state.clone()
                    }
                } else if let Operation::Switch { default, cases } = &term.op {
                    refine_switch_edge(term, cases, *default, *succ_id, &current_state)
                } else {
                    current_state.clone()
                }
            } else {
                current_state.clone()
            };

            let old_state = block_entry_states.get(succ_id).cloned();
            let new_state = if let Some(old) = &old_state {
                if loop_headers.contains(*succ_id) {
                    let merged_thresholds = if let Some(bounds) = loop_bounds.get(succ_id) {
                        let mut m = thresholds.clone();
                        m.extend(bounds.iter().copied());
                        m
                    } else {
                        thresholds.clone()
                    };
                    widen_state(old, &propagated_state, &merged_thresholds)
                } else {
                    old.join(&propagated_state)
                }
            } else {
                propagated_state.clone()
            };

            if old_state.as_ref() != Some(&new_state) {
                block_entry_states.insert(*succ_id, new_state);
                if !worklist.contains(succ_id) {
                    worklist.push_back(*succ_id);
                }
            }
        }
    }

    // Collect instruction states for return value extraction
    let mut inst_states: BTreeMap<InstId, AbstractState> = BTreeMap::new();
    let mut reached_final = IdBitSet::<BlockId>::empty();
    for (id, s) in &block_entry_states {
        if !s.is_unreachable() {
            reached_final.insert(*id);
        }
    }
    let phi_ctx_final = TransferContext {
        reached_blocks: Some(&reached_final),
        ..TransferContext::default()
    };
    for block in &func.blocks {
        let block_entry = block_entry_states
            .get(&block.id)
            .cloned()
            .unwrap_or_else(AbstractState::bottom);

        if block_entry.is_unreachable() {
            continue;
        }

        let mut current_state = block_entry;
        for inst in &block.instructions {
            inst_states.insert(inst.id, current_state.clone());
            transfer_instruction_with_context(
                inst,
                &mut current_state,
                &extended_constant_map,
                module,
                &phi_ctx_final,
            );
        }
    }

    // Collect return values
    let mut return_intervals: Vec<Interval> = Vec::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::Ret = &inst.op {
                if let Some(&return_operand) = inst.operands.first() {
                    let interval = inst_states
                        .get(&inst.id)
                        .and_then(|s| s.get_opt(return_operand).cloned())
                        .or_else(|| extended_constant_map.get(&return_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));

                    return_intervals.push(interval);
                }
            }
        }
    }

    // Join all return intervals
    let mut result = BTreeMap::new();
    if !return_intervals.is_empty() {
        let mut joined = return_intervals[0].clone();
        for interval in return_intervals.iter().skip(1) {
            joined = joined.join(interval);
        }
        result.insert(ValueId::new(0), joined);
    }
    result
}

/// Widen state point-wise using threshold widening.
pub fn widen_state(
    old: &AbstractState,
    new: &AbstractState,
    thresholds: &BTreeSet<i128>,
) -> AbstractState {
    if old.is_unreachable() {
        return new.clone();
    }
    if new.is_unreachable() {
        return old.clone();
    }

    let mut result = AbstractState::new();

    // Widen value entries present in both states
    for key in old.entries().keys() {
        if let (Some(a_val), Some(b_val)) = (old.get_opt(*key), new.get_opt(*key)) {
            let widened = if thresholds.is_empty() {
                a_val.widen(b_val)
            } else {
                a_val.widen_with_thresholds(b_val, thresholds)
            };
            if !widened.is_top() {
                result.set(*key, widened);
            }
        }
        // One-sided entries in old → treated as TOP after widening (dropped)
    }

    // Include new-only value entries (values from back-edge not yet in old).
    // These represent values computed in the loop body that flow into the
    // loop header for the first time. Without including them, phi inputs
    // from the back-edge are lost and the fixpoint converges prematurely.
    for (key, new_val) in new.entries() {
        if old.get_opt(*key).is_none() && !new_val.is_top() {
            result.set(*key, new_val.clone());
        }
    }

    // Widen loc_memory from both states
    for (loc, old_val) in old.loc_memory_entries() {
        if let Some(new_val) = new.load_loc(*loc) {
            let widened = if thresholds.is_empty() {
                old_val.widen(new_val)
            } else {
                old_val.widen_with_thresholds(new_val, thresholds)
            };
            if !widened.is_top() {
                result.store_loc(*loc, widened);
            }
        }
    }
    // Include new-only loc_memory entries
    for (loc, new_val) in new.loc_memory_entries() {
        if old.load_loc(*loc).is_none() && !new_val.is_top() {
            result.store_loc(*loc, new_val.clone());
        }
    }

    // Widen memory from both states
    for (ptr, old_val) in old.memory_entries() {
        if let Some(new_val) = new.load(*ptr) {
            let widened = if thresholds.is_empty() {
                old_val.widen(new_val)
            } else {
                old_val.widen_with_thresholds(new_val, thresholds)
            };
            if !widened.is_top() {
                result.store(*ptr, widened);
            }
        }
    }
    // Include new-only memory entries
    for (ptr, new_val) in new.memory_entries() {
        if old.load(*ptr).is_none() && !new_val.is_top() {
            result.store(*ptr, new_val.clone());
        }
    }

    // Preserve gep_targets from both states (union)
    for (vid, targets) in old.gep_targets() {
        let mut merged = targets.clone();
        if let Some(new_targets) = new.resolve_gep(*vid) {
            merged.extend(new_targets);
        }
        result.register_gep(*vid, merged);
    }
    for (vid, targets) in new.gep_targets() {
        if old.resolve_gep(*vid).is_none() {
            result.register_gep(*vid, targets.clone());
        }
    }

    result
}

/// Narrow state point-wise.
pub fn narrow_state(
    old: &AbstractState,
    new: &AbstractState,
    _thresholds: &BTreeSet<i128>,
) -> AbstractState {
    if old.is_unreachable() {
        return AbstractState::bottom();
    }
    if new.is_unreachable() {
        return AbstractState::bottom();
    }

    let mut result = AbstractState::new();

    // Narrow value entries
    for (key, old_val) in old.entries() {
        if let Some(new_val) = new.get_opt(*key) {
            let narrowed = old_val.narrow(new_val);
            if !narrowed.is_top() {
                result.set(*key, narrowed);
            }
        } else {
            // new has top for this key → narrow top with old → old
            if !old_val.is_top() {
                result.set(*key, old_val.clone());
            }
        }
    }

    // Narrow loc_memory: narrow entries present in both, keep old-only entries
    for (loc, old_val) in old.loc_memory_entries() {
        if let Some(new_val) = new.load_loc(*loc) {
            let narrowed = old_val.narrow(new_val);
            if !narrowed.is_top() {
                result.store_loc(*loc, narrowed);
            }
        } else if !old_val.is_top() {
            result.store_loc(*loc, old_val.clone());
        }
    }

    // Narrow memory entries
    for (ptr, old_val) in old.memory_entries() {
        if let Some(new_val) = new.load(*ptr) {
            let narrowed = old_val.narrow(new_val);
            if !narrowed.is_top() {
                result.store(*ptr, narrowed);
            }
        } else if !old_val.is_top() {
            result.store(*ptr, old_val.clone());
        }
    }

    // Preserve gep_targets from old state
    for (vid, targets) in old.gep_targets() {
        result.register_gep(*vid, targets.clone());
    }

    result
}

/// Compute the refined state for a CFG edge based on the block terminator.
///
/// Given the terminator instruction, successor ID, current block state, condition
/// instruction map, and constant map, returns the appropriately refined state:
/// - `CondBr`: refines using the branch condition for the taken/not-taken edge
/// - `Switch`: refines using `refine_switch_edge_with_pruning`
/// - All other terminators: returns a clone of the current state
///
/// This is the shared core of successor-state computation used in both the
/// ascending and narrowing phases of fixpoint iteration.
pub(crate) fn refine_for_successor(
    terminator: Option<&Instruction>,
    succ_id: BlockId,
    current_state: &AbstractState,
    cond_inst_map: &BTreeMap<ValueId, Instruction>,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> AbstractState {
    let Some(term) = terminator else {
        return current_state.clone();
    };

    if let Operation::CondBr { then_target, .. } = &term.op {
        let take_true = succ_id == *then_target;
        if let Some(&cond_operand) = term.operands.first() {
            if let Some(cond_inst) = cond_inst_map.get(&cond_operand) {
                return refine_branch_condition(cond_inst, current_state, constant_map, take_true);
            }
        }
        return current_state.clone();
    }

    if let Operation::Switch { default, cases } = &term.op {
        return refine_switch_edge_with_pruning(
            term,
            cases,
            *default,
            succ_id,
            current_state,
            constant_map,
        );
    }

    current_state.clone()
}

/// Propagate branch refinements to `loc_memory` for store/load alias tracking.
///
/// When a refined value was loaded from a location, this updates the location's
/// stored interval so that subsequent loads in the current block or successor
/// block pick up the refined value. This is critical for the `-O0` alloca
/// pattern where branch conditions refine values loaded from stack allocations.
///
/// The `blocks_with_loads` set is an optimization: propagation is only performed
/// for blocks that actually contain `Load` instructions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn propagate_loc_memory_for_edge(
    refined: &mut AbstractState,
    block: &AirBlock,
    block_id: BlockId,
    succ_id: BlockId,
    current_state: &AbstractState,
    pta: &PtaIntegration<'_>,
    func: &AirFunction,
    blocks_with_loads: &BTreeSet<BlockId>,
) {
    if refined.is_unreachable() {
        return;
    }
    if blocks_with_loads.contains(&block_id) {
        propagate_refinement_to_loc_memory(refined, block, current_state, pta);
    }
    if blocks_with_loads.contains(&succ_id) {
        if let Some(succ_block) = func.blocks.iter().find(|b| b.id == succ_id) {
            propagate_refinement_to_loc_memory(refined, succ_block, current_state, pta);
        }
    }
}

/// Collect ValueId refinements by comparing pre-refinement and post-refinement states.
///
/// Records entries where the refined state has a strictly tighter interval
/// than the pre-refinement state.
pub(crate) fn collect_refinements(
    pre: &AbstractState,
    post: &AbstractState,
    refinements: &mut BTreeMap<ValueId, Interval>,
) {
    for (vid, post_val) in post.entries() {
        if post_val.is_top() || post_val.is_bottom() {
            continue;
        }
        let pre_val = pre.get_opt(*vid);
        let is_tighter = match pre_val {
            None => true, // pre was implicitly top
            Some(pv) => post_val != pv && post_val.leq(pv),
        };
        if is_tighter {
            refinements.insert(*vid, post_val.clone());
        }
    }
}

/// Re-apply persisted refinements to a state via meet (intersection).
///
/// For each refined ValueId, computes `state[id] = state[id] ∩ refinement`.
/// This preserves the refinement through join/widen operations.
pub(crate) fn apply_refinements(
    state: &mut AbstractState,
    refinements: &BTreeMap<ValueId, Interval>,
) {
    if state.is_unreachable() {
        return;
    }
    for (vid, refinement) in refinements {
        if let Some(current) = state.get_opt(*vid) {
            let met = current.meet(refinement);
            if !met.is_bottom() {
                state.set(*vid, met);
            }
            // If meet is bottom, keep the current value (don't make state unreachable
            // due to a stale refinement from a prior iteration).
        }
        // If the ValueId isn't in the state (implicitly top), apply the refinement
        else if !refinement.is_top() {
            state.set(*vid, refinement.clone());
        }
    }
}

/// Refine the abstract state along a Switch edge with infeasible path pruning.
///
/// For a case edge, the discriminant is narrowed to the matching case value.
/// For the default edge, the state is propagated without refinement (conservative).
///
/// **Pruning (Plan 084 Phase A):** When the discriminant is a singleton or narrow
/// interval, non-matching case edges produce an unreachable (bottom) state. This
/// prevents infeasible paths from polluting join results at the switch exit.
pub(crate) fn refine_switch_edge_with_pruning(
    term: &Instruction,
    cases: &[(i64, BlockId)],
    default: BlockId,
    succ_id: BlockId,
    current_state: &AbstractState,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> AbstractState {
    let Some(&discriminant) = term.operands.first() else {
        return current_state.clone();
    };

    // Resolve the discriminant's current interval
    let disc_interval = super::transfer::resolve_operand(discriminant, current_state, constant_map);

    // Check if this successor is a specific case target
    for &(case_val, target) in cases {
        if target == succ_id {
            // Check if this case is feasible given the discriminant interval
            if !disc_interval.is_top() && !disc_interval.contains(i128::from(case_val)) {
                // The discriminant cannot be this case value — infeasible path
                return AbstractState::bottom();
            }
            let mut refined = current_state.clone();
            refined.set(discriminant, Interval::singleton(i128::from(case_val), 64));
            return refined;
        }
    }

    // Default edge: prune when all discriminant values are covered by cases
    if succ_id == default {
        if !disc_interval.is_top() && disc_interval.lo() == disc_interval.hi() {
            // Singleton discriminant: if any case matches, default is unreachable
            let val = disc_interval.lo();
            if cases.iter().any(|&(cv, _)| i128::from(cv) == val) {
                return AbstractState::bottom();
            }
        }
        return current_state.clone();
    }

    current_state.clone()
}

/// Detect loop headers as targets of back edges.
///
/// A back edge is an edge from a node to a node that dominates it.
/// We approximate this using DFS: an edge to an already-visited node
/// that is still on the stack is a back edge.
pub fn detect_loop_headers(cfg: &Cfg) -> IdBitSet<BlockId> {
    let mut loop_headers = IdBitSet::<BlockId>::empty();
    let mut visited = BTreeSet::new();
    let mut on_stack = BTreeSet::new();

    dfs_find_back_edges(
        cfg.entry,
        cfg,
        &mut visited,
        &mut on_stack,
        &mut loop_headers,
    );

    loop_headers
}

fn dfs_find_back_edges(
    node: BlockId,
    cfg: &Cfg,
    visited: &mut BTreeSet<BlockId>,
    on_stack: &mut BTreeSet<BlockId>,
    loop_headers: &mut IdBitSet<BlockId>,
) {
    visited.insert(node);
    on_stack.insert(node);

    if let Some(succs) = cfg.successors_of(node) {
        for succ in succs {
            if on_stack.contains(succ) {
                // Back edge → succ is a loop header
                loop_headers.insert(*succ);
            } else if !visited.contains(succ) {
                dfs_find_back_edges(*succ, cfg, visited, on_stack, loop_headers);
            }
        }
    }

    on_stack.remove(&node);
}

/// Extract comparison constants from loop header blocks for per-loop widening thresholds.
///
/// For each loop header, scans instructions for `ICmp` comparisons and extracts
/// their constant operands as widening thresholds. Each constant gets +/-1 neighbors
/// (same as global threshold extraction in `threshold.rs`).
///
/// This enables loop-specific widening bounds that capture iteration limits
/// from conditions like `i < N`, preventing widening from jumping past loop bounds.
#[must_use]
pub fn extract_loop_bound_constants(
    cfg: &Cfg,
    loop_headers: &IdBitSet<BlockId>,
    func: &AirFunction,
    module: &AirModule,
) -> BTreeMap<BlockId, BTreeSet<i128>> {
    let int_constants: BTreeMap<ValueId, i128> = module
        .constants
        .iter()
        .filter_map(|(vid, c)| match c {
            Constant::Int { value, .. } => Some((*vid, i128::from(*value))),
            Constant::BigInt { value, bits: _ } => value.parse::<i128>().ok().map(|v| (*vid, v)),
            _ => None,
        })
        .collect();

    let mut result = BTreeMap::new();

    for header_id in loop_headers.iter() {
        let mut constants = BTreeSet::new();

        // Collect blocks to scan: the header, predecessors, and successor
        // blocks (the loop body). Loop bodies contain increment constants
        // (e.g., `i + 1`) that are useful widening thresholds.
        let mut blocks_to_scan = vec![header_id];
        if let Some(preds) = cfg.predecessors_of(header_id) {
            for pred_id in preds {
                blocks_to_scan.push(*pred_id);
            }
        }
        // Also scan successors of the header (loop body entry blocks)
        // and their successors (one level deep covers most loop bodies).
        let header_succs: Vec<BlockId> = cfg
            .successors_of(header_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        for succ_id in &header_succs {
            if !blocks_to_scan.contains(succ_id) {
                blocks_to_scan.push(*succ_id);
            }
            if let Some(inner_succs) = cfg.successors_of(*succ_id) {
                for inner_succ in inner_succs {
                    if !blocks_to_scan.contains(inner_succ) {
                        blocks_to_scan.push(*inner_succ);
                    }
                }
            }
        }

        for block_id in blocks_to_scan {
            let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
                continue;
            };
            for inst in &block.instructions {
                if let Operation::BinaryOp { kind } = &inst.op {
                    if matches!(
                        kind,
                        BinaryOp::ICmpSlt
                            | BinaryOp::ICmpSle
                            | BinaryOp::ICmpSgt
                            | BinaryOp::ICmpSge
                            | BinaryOp::ICmpEq
                            | BinaryOp::ICmpNe
                            | BinaryOp::ICmpUlt
                            | BinaryOp::ICmpUle
                            | BinaryOp::ICmpUgt
                            | BinaryOp::ICmpUge
                            | BinaryOp::Add
                            | BinaryOp::Sub
                    ) {
                        for operand in &inst.operands {
                            if let Some(&v) = int_constants.get(operand) {
                                constants.insert(v.saturating_sub(1));
                                constants.insert(v);
                                constants.insert(v.saturating_add(1));
                            }
                        }
                    }
                }
            }
        }

        if !constants.is_empty() {
            result.insert(header_id, constants);
        }
    }

    result
}

/// Compute reverse postorder traversal of the CFG.
pub fn reverse_postorder(cfg: &Cfg) -> Vec<BlockId> {
    let mut visited = BTreeSet::new();
    let mut postorder = Vec::new();

    dfs_postorder(cfg.entry, cfg, &mut visited, &mut postorder);

    postorder.reverse();
    postorder
}

fn dfs_postorder(
    node: BlockId,
    cfg: &Cfg,
    visited: &mut BTreeSet<BlockId>,
    postorder: &mut Vec<BlockId>,
) {
    if visited.contains(&node) {
        return;
    }
    visited.insert(node);

    if let Some(succs) = cfg.successors_of(node) {
        for succ in succs {
            dfs_postorder(*succ, cfg, visited, postorder);
        }
    }

    postorder.push(node);
}

/// Build a map from condition `ValueId` to the `Instruction` that defines it.
///
/// This lets us look up the comparison instruction for a `CondBr` condition.
pub fn build_cond_inst_map(func: &AirFunction) -> BTreeMap<ValueId, Instruction> {
    let mut map = BTreeMap::new();

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                if let Operation::BinaryOp { kind } = &inst.op {
                    if matches!(
                        kind,
                        BinaryOp::ICmpSlt
                            | BinaryOp::ICmpSle
                            | BinaryOp::ICmpSgt
                            | BinaryOp::ICmpSge
                            | BinaryOp::ICmpEq
                            | BinaryOp::ICmpNe
                            | BinaryOp::ICmpUlt
                            | BinaryOp::ICmpUle
                            | BinaryOp::ICmpUgt
                            | BinaryOp::ICmpUge
                    ) {
                        map.insert(dst, inst.clone());
                    }
                }
            }
        }
    }

    map
}

// ---------------------------------------------------------------------------
// Noreturn call detection
// ---------------------------------------------------------------------------

/// Check if a block ends with a call to a noreturn function.
///
/// When a noreturn call is detected, successors of this block are unreachable
/// and should not be added to the analysis worklist.
///
/// # Arguments
///
/// * `block` - The basic block to check
/// * `func_names` - Mapping from `FunctionId` to function name for callees
/// * `specs` - Optional spec registry for looking up noreturn functions
#[must_use]
pub(crate) fn block_ends_noreturn(
    block: &AirBlock,
    func_names: &BTreeMap<FunctionId, &str>,
    specs: Option<&SpecRegistry>,
) -> bool {
    // Check all instructions in the block for noreturn calls
    // (not just the terminator, since a noreturn call might be followed by unreachable)
    for inst in &block.instructions {
        if let Operation::CallDirect { callee } = &inst.op {
            if let Some(&name) = func_names.get(callee) {
                if is_noreturn_with_specs(name, specs) {
                    return true;
                }
            }
        }
    }
    false
}

#[must_use]
/// Refine the abstract state along a Switch edge.
///
/// For a case edge, the discriminant is narrowed to the matching case value.
/// For the default edge, the state is propagated without refinement (conservative).
fn refine_switch_edge(
    term: &Instruction,
    cases: &[(i64, BlockId)],
    default: BlockId,
    succ_id: BlockId,
    current_state: &AbstractState,
) -> AbstractState {
    let Some(&discriminant) = term.operands.first() else {
        return current_state.clone();
    };

    // Check if this successor is a specific case target
    for &(case_val, target) in cases {
        if target == succ_id {
            let mut refined = current_state.clone();
            refined.set(discriminant, Interval::singleton(i128::from(case_val), 64));
            return refined;
        }
    }

    // Default edge: prune when discriminant is a singleton matching a case
    if succ_id == default {
        let disc_interval = current_state.get(discriminant, 64);
        if !disc_interval.is_top() && disc_interval.lo() == disc_interval.hi() {
            let val = disc_interval.lo();
            if cases.iter().any(|&(cv, _)| i128::from(cv) == val) {
                return AbstractState::bottom();
            }
        }
        return current_state.clone();
    }

    current_state.clone()
}

/// Build a function ID to name mapping for noreturn checking.
#[must_use]
pub(crate) fn build_func_names(module: &AirModule) -> BTreeMap<FunctionId, &str> {
    module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect()
}

/// Check if a terminator creates a partition split point.
///
/// A split point is a `CondBr` where the condition compares a value against
/// a program constant (detected via `constant_map`).
fn is_partition_split_point(
    terminator: Option<&Instruction>,
    _succ_id: BlockId,
    cond_inst_map: &BTreeMap<ValueId, Instruction>,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> bool {
    let Some(term) = terminator else {
        return false;
    };
    let Operation::CondBr { .. } = &term.op else {
        return false;
    };
    let Some(&cond_operand) = term.operands.first() else {
        return false;
    };
    // Check if the condition is an ICmp comparing against a constant
    let Some(cond_inst) = cond_inst_map.get(&cond_operand) else {
        return false;
    };
    let Operation::BinaryOp { kind } = &cond_inst.op else {
        return false;
    };
    // Must be a comparison
    if !matches!(
        kind,
        BinaryOp::ICmpEq
            | BinaryOp::ICmpNe
            | BinaryOp::ICmpSlt
            | BinaryOp::ICmpSle
            | BinaryOp::ICmpSgt
            | BinaryOp::ICmpSge
            | BinaryOp::ICmpUlt
            | BinaryOp::ICmpUle
            | BinaryOp::ICmpUgt
            | BinaryOp::ICmpUge
    ) {
        return false;
    }
    // At least one operand must be a singleton constant
    if cond_inst.operands.len() < 2 {
        return false;
    }
    let lhs_const = constant_map
        .get(&cond_inst.operands[0])
        .is_some_and(Interval::is_singleton);
    let rhs_const = constant_map
        .get(&cond_inst.operands[1])
        .is_some_and(Interval::is_singleton);
    lhs_const || rhs_const
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::AirFunction;
    use saf_core::ids::{InstId, ModuleId};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }

    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }

    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }

    /// Build a simple loop function:
    /// ```text
    /// entry:
    ///   i = 0
    ///   goto header
    /// header:
    ///   i_phi = phi(entry: 0, body: i_inc)
    ///   cond = i_phi < 10
    ///   br cond, body, exit
    /// body:
    ///   i_inc = i_phi + 1
    ///   goto header
    /// exit:
    ///   ret
    /// ```
    fn make_simple_loop() -> (AirModule, FunctionId) {
        let fid = fid(1);
        let entry = bid(10);
        let header = bid(20);
        let body = bid(30);
        let exit = bid(40);

        // Values
        let v_zero = vid(100); // constant 0
        let v_ten = vid(101); // constant 10
        let v_one = vid(102); // constant 1
        let v_i_phi = vid(200); // loop counter
        let v_cond = vid(201); // comparison result
        let v_i_inc = vid(202); // incremented counter

        // Entry block: set constants, branch to header
        let mut entry_block = AirBlock::new(entry);
        // In real IR, constants come from const instructions
        // We'll set them up via the constant map instead
        entry_block
            .instructions
            .push(Instruction::new(iid(1), Operation::Br { target: header }));

        // Header block: phi, compare, branch
        let mut header_block = AirBlock::new(header);
        header_block.instructions.push(
            Instruction::new(
                iid(2),
                Operation::Phi {
                    incoming: vec![(entry, v_zero), (body, v_i_inc)],
                },
            )
            .with_dst(v_i_phi),
        );
        header_block.instructions.push(
            Instruction::new(
                iid(3),
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpSlt,
                },
            )
            .with_operands(vec![v_i_phi, v_ten])
            .with_dst(v_cond),
        );
        header_block.instructions.push(
            Instruction::new(
                iid(4),
                Operation::CondBr {
                    then_target: body,
                    else_target: exit,
                },
            )
            .with_operands(vec![v_cond]),
        );

        // Body block: increment and loop back
        let mut body_block = AirBlock::new(body);
        body_block.instructions.push(
            Instruction::new(
                iid(5),
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
            )
            .with_operands(vec![v_i_phi, v_one])
            .with_dst(v_i_inc),
        );
        body_block
            .instructions
            .push(Instruction::new(iid(6), Operation::Br { target: header }));

        // Exit block
        let mut exit_block = AirBlock::new(exit);
        exit_block
            .instructions
            .push(Instruction::new(iid(7), Operation::Ret));

        let func = AirFunction {
            id: fid,
            name: "test_loop".to_string(),
            params: Vec::new(),
            blocks: vec![entry_block, header_block, body_block, exit_block],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::derive(b"test_loop"));
        module.functions.push(func);

        (module, fid)
    }

    #[test]
    fn loop_header_detection() {
        let (module, fid) = make_simple_loop();
        let func = module.function(fid).unwrap();
        let cfg = Cfg::build(func);

        let headers = detect_loop_headers(&cfg);
        assert!(
            headers.contains(bid(20)),
            "header block should be detected as loop header"
        );
    }

    #[test]
    fn simple_loop_converges() {
        let (module, _fid) = make_simple_loop();
        let config = AbstractInterpConfig::default();

        let result = solve_abstract_interp(&module, &config);

        assert!(result.diagnostics().converged);
        assert!(result.diagnostics().functions_analyzed > 0);
    }

    #[test]
    fn straight_line_constants() {
        // Function: x = 5 + 3 → should compute [8, 8]
        let fid = fid(1);
        let entry = bid(10);

        let v_five = vid(100);
        let v_three = vid(101);
        let v_result = vid(200);

        let mut entry_block = AirBlock::new(entry);
        // In real IR, constants come from constant operands or PHI
        // We'll use the constant map for v_five=5, v_three=3
        entry_block.instructions.push(
            Instruction::new(
                iid(1),
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
            )
            .with_operands(vec![v_five, v_three])
            .with_dst(v_result),
        );
        entry_block
            .instructions
            .push(Instruction::new(iid(2), Operation::Ret));

        let func = AirFunction {
            id: fid,
            name: "add_constants".to_string(),
            params: Vec::new(),
            blocks: vec![entry_block],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::derive(b"test_const"));
        module.functions.push(func);

        let config = AbstractInterpConfig::default();
        let result = solve_abstract_interp(&module, &config);

        // Since v_five and v_three are unknown (top), result should be top too
        // (no constant information in the AIR for those values)
        assert!(result.diagnostics().converged);
    }

    #[test]
    fn deterministic_results() {
        let (module, _) = make_simple_loop();
        let config = AbstractInterpConfig::default();

        let result1 = solve_abstract_interp(&module, &config);
        let result2 = solve_abstract_interp(&module, &config);

        // Same input → same diagnostics
        assert_eq!(
            result1.diagnostics().blocks_analyzed,
            result2.diagnostics().blocks_analyzed
        );
        assert_eq!(
            result1.diagnostics().widening_applications,
            result2.diagnostics().widening_applications
        );
    }

    #[test]
    fn block_ends_noreturn_detects_exit() {
        // Create a block with a call to exit()
        let exit_fid = FunctionId::new(100);
        let mut block = AirBlock::new(bid(1));
        block.instructions.push(
            Instruction::new(iid(1), Operation::CallDirect { callee: exit_fid })
                .with_operands(vec![vid(1)]),
        );

        // Create func_names mapping
        let mut func_names: BTreeMap<FunctionId, &str> = BTreeMap::new();
        func_names.insert(exit_fid, "exit");

        // Without specs, should use hardcoded list
        assert!(block_ends_noreturn(&block, &func_names, None));
    }

    #[test]
    fn block_ends_noreturn_detects_abort() {
        let abort_fid = FunctionId::new(101);
        let mut block = AirBlock::new(bid(1));
        block.instructions.push(Instruction::new(
            iid(1),
            Operation::CallDirect { callee: abort_fid },
        ));

        let mut func_names: BTreeMap<FunctionId, &str> = BTreeMap::new();
        func_names.insert(abort_fid, "abort");

        assert!(block_ends_noreturn(&block, &func_names, None));
    }

    #[test]
    fn block_ends_noreturn_false_for_normal_call() {
        let printf_fid = FunctionId::new(102);
        let mut block = AirBlock::new(bid(1));
        block.instructions.push(
            Instruction::new(iid(1), Operation::CallDirect { callee: printf_fid })
                .with_operands(vec![vid(1)]),
        );

        let mut func_names: BTreeMap<FunctionId, &str> = BTreeMap::new();
        func_names.insert(printf_fid, "printf");

        // printf is not noreturn
        assert!(!block_ends_noreturn(&block, &func_names, None));
    }

    #[test]
    fn block_ends_noreturn_empty_block() {
        let block = AirBlock::new(bid(1));
        let func_names: BTreeMap<FunctionId, &str> = BTreeMap::new();

        // Empty block has no noreturn calls
        assert!(!block_ends_noreturn(&block, &func_names, None));
    }

    // ======================================================================
    // is_partition_split_point tests (M-5)
    // ======================================================================

    #[test]
    fn split_point_none_terminator_returns_false() {
        let cond_inst_map = BTreeMap::new();
        let constant_map = BTreeMap::new();
        assert!(!is_partition_split_point(
            None,
            bid(1),
            &cond_inst_map,
            &constant_map
        ));
    }

    #[test]
    fn split_point_non_condbr_returns_false() {
        let cond_inst_map = BTreeMap::new();
        let constant_map = BTreeMap::new();
        // A Br (unconditional branch) is not a CondBr
        let term = Instruction::new(iid(10), Operation::Br { target: bid(2) });
        assert!(!is_partition_split_point(
            Some(&term),
            bid(2),
            &cond_inst_map,
            &constant_map
        ));
    }

    #[test]
    fn split_point_condbr_no_cond_inst_returns_false() {
        // CondBr whose condition operand is not in the cond_inst_map
        let cond_vid = vid(50);
        let term = Instruction::new(
            iid(10),
            Operation::CondBr {
                then_target: bid(2),
                else_target: bid(3),
            },
        )
        .with_operands(vec![cond_vid]);

        let cond_inst_map = BTreeMap::new(); // empty -- no mapping for cond_vid
        let constant_map = BTreeMap::new();

        assert!(!is_partition_split_point(
            Some(&term),
            bid(2),
            &cond_inst_map,
            &constant_map
        ));
    }

    #[test]
    fn split_point_condbr_with_icmp_eq_and_constant_returns_true() {
        let cond_vid = vid(50);
        let lhs_vid = vid(60);
        let rhs_vid = vid(61);

        // The condition is an ICmpEq comparing lhs against rhs
        let cond_inst = Instruction::new(
            iid(20),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![lhs_vid, rhs_vid])
        .with_dst(cond_vid);

        let mut cond_inst_map = BTreeMap::new();
        cond_inst_map.insert(cond_vid, cond_inst);

        // rhs is a singleton constant
        let mut constant_map = BTreeMap::new();
        constant_map.insert(rhs_vid, Interval::singleton(42, 32));

        let term = Instruction::new(
            iid(10),
            Operation::CondBr {
                then_target: bid(2),
                else_target: bid(3),
            },
        )
        .with_operands(vec![cond_vid]);

        assert!(is_partition_split_point(
            Some(&term),
            bid(2),
            &cond_inst_map,
            &constant_map
        ));
    }

    #[test]
    fn split_point_condbr_icmp_eq_no_constant_returns_false() {
        let cond_vid = vid(50);
        let lhs_vid = vid(60);
        let rhs_vid = vid(61);

        // The condition is an ICmpEq but neither operand is a constant
        let cond_inst = Instruction::new(
            iid(20),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![lhs_vid, rhs_vid])
        .with_dst(cond_vid);

        let mut cond_inst_map = BTreeMap::new();
        cond_inst_map.insert(cond_vid, cond_inst);

        // No constants in the map
        let constant_map = BTreeMap::new();

        let term = Instruction::new(
            iid(10),
            Operation::CondBr {
                then_target: bid(2),
                else_target: bid(3),
            },
        )
        .with_operands(vec![cond_vid]);

        assert!(!is_partition_split_point(
            Some(&term),
            bid(2),
            &cond_inst_map,
            &constant_map
        ));
    }

    #[test]
    fn split_point_condbr_icmp_slt_with_lhs_constant_returns_true() {
        // Test with a different ICmp kind and constant on the LHS
        let cond_vid = vid(50);
        let lhs_vid = vid(60);
        let rhs_vid = vid(61);

        let cond_inst = Instruction::new(
            iid(20),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
        )
        .with_operands(vec![lhs_vid, rhs_vid])
        .with_dst(cond_vid);

        let mut cond_inst_map = BTreeMap::new();
        cond_inst_map.insert(cond_vid, cond_inst);

        // lhs is a singleton constant (constant on the left side)
        let mut constant_map = BTreeMap::new();
        constant_map.insert(lhs_vid, Interval::singleton(0, 32));

        let term = Instruction::new(
            iid(10),
            Operation::CondBr {
                then_target: bid(2),
                else_target: bid(3),
            },
        )
        .with_operands(vec![cond_vid]);

        assert!(is_partition_split_point(
            Some(&term),
            bid(2),
            &cond_inst_map,
            &constant_map
        ));
    }

    #[test]
    fn split_point_condbr_non_singleton_constant_returns_false() {
        // A range interval (not singleton) should not trigger a split point
        let cond_vid = vid(50);
        let lhs_vid = vid(60);
        let rhs_vid = vid(61);

        let cond_inst = Instruction::new(
            iid(20),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![lhs_vid, rhs_vid])
        .with_dst(cond_vid);

        let mut cond_inst_map = BTreeMap::new();
        cond_inst_map.insert(cond_vid, cond_inst);

        // rhs is a range interval [0, 100], not a singleton
        let mut constant_map = BTreeMap::new();
        constant_map.insert(rhs_vid, Interval::new(0, 100, 32));

        let term = Instruction::new(
            iid(10),
            Operation::CondBr {
                then_target: bid(2),
                else_target: bid(3),
            },
        )
        .with_operands(vec![cond_vid]);

        assert!(!is_partition_split_point(
            Some(&term),
            bid(2),
            &cond_inst_map,
            &constant_map
        ));
    }
}
