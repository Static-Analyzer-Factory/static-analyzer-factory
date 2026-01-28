//! Call graph refinement via CHA + online PTA.
//!
//! Implements SVF-style online CG construction (Plan 111):
//!
//! 1. Build initial `CallGraph` from the module.
//! 2. Bootstrap: resolve virtual calls via CHA.
//! 3. Extract ALL PTA constraints upfront, HVN preprocess, solve to fixed point.
//! 4. Solve-then-refine loop: resolve indirect calls from pts → add
//!    interprocedural copy edges for new targets → re-solve. Repeat until
//!    no new targets are discovered.
//! 5. Build final ICFG from the refined call graph.
//!
//! Convergence: the algorithm is monotone (only adds edges/constraints) over a
//! finite set, so it always terminates. The online approach avoids redundant
//! full re-solves by adding interprocedural edges incrementally.

use crate::timer::Timer;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use saf_core::air::{AirFunction, AirModule, AirType};
use saf_core::config::PtaSolver;
use saf_core::ids::{FunctionId, InstId, TypeId, ValueId};
use saf_core::saf_log;

use crate::callgraph::CallGraph;
#[cfg(test)]
use crate::callgraph::CallGraphNode;
use crate::cha::ClassHierarchy;
#[cfg(test)]
use crate::graph_algo::Successors;
use crate::icfg::Icfg;
use crate::pta::ptsset::{FxHashPtsSet, PtsSet};
use crate::pta::{
    CopyConstraint, FieldSensitivity, FunctionLocationMap, GenericSolver, LocationFactory,
    PtaConfig, PtaResult, extract_constraints, extract_spec_constraints,
};
use saf_core::spec::SpecRegistry;

// =============================================================================
// Configuration
// =============================================================================

/// Strategy for determining entry points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryPointStrategy {
    /// Use all non-declaration functions as entry points.
    AllDefined,
    /// Use only functions with the specified names.
    Named(Vec<String>),
}

/// Configuration for the refinement loop.
#[derive(Debug, Clone)]
pub struct RefinementConfig {
    /// Maximum number of PTA-refinement iterations.
    pub max_iterations: usize,
    /// Entry point selection strategy.
    pub entry_points: EntryPointStrategy,
    /// PTA solver configuration.
    pub pta_config: PtaConfig,
    /// Field sensitivity for the PTA solver.
    pub field_sensitivity: FieldSensitivity,
    /// Which PTA solver backend to use for the refinement loop.
    pub pta_solver: PtaSolver,
}

impl Default for RefinementConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            entry_points: EntryPointStrategy::AllDefined,
            pta_config: PtaConfig::default(),
            field_sensitivity: FieldSensitivity::default(),
            pta_solver: PtaSolver::default(),
        }
    }
}

// =============================================================================
// Result
// =============================================================================

/// Result of call graph refinement.
pub struct RefinementResult {
    /// The refined call graph.
    pub call_graph: CallGraph,
    /// The ICFG built from the refined call graph.
    pub icfg: Icfg,
    /// PTA result from the last iteration (if PTA ran).
    pub pta_result: Option<PtaResult>,
    /// Class hierarchy (if type hierarchy entries were present).
    pub cha: Option<ClassHierarchy>,
    /// Number of PTA iterations that actually executed.
    pub iterations: usize,
    /// Map from call-site InstId to resolved target FunctionIds.
    pub resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
    /// Time spent on PTA solving (constraint extraction, HVN, solve, refinement loop).
    pub pta_solve_secs: f64,
    /// Constraint counts: [addr, copy, load, store, gep].
    pub constraint_counts: [usize; 5],
    /// Constraint counts after HVN preprocessing: [addr, copy, load, store, gep].
    pub post_hvn_constraint_counts: [usize; 5],
}

/// Intermediate state after solver-agnostic preparation.
///
/// Passed to a solver-specific PTA loop, then to [`refine_finalize`].
pub struct RefinementPrepared {
    /// The call graph being refined.
    pub cg: CallGraph,
    /// Class hierarchy (if type hierarchy present).
    pub cha: Option<ClassHierarchy>,
    /// Sites resolved by CHA.
    pub cha_resolved_sites: BTreeSet<InstId>,
    /// Map from call-site to resolved targets.
    pub resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
    /// Original constraints (pre-HVN).
    pub constraints: crate::pta::ConstraintSet,
    /// Reduced constraints (post-HVN).
    pub reduced: crate::pta::ConstraintSet,
    /// HVN result for later expansion.
    pub hvn_result: crate::pta::HvnResult,
    /// Location factory.
    pub factory: LocationFactory,
    /// Function-to-location mapping.
    pub func_loc_map: FunctionLocationMap,
    /// Indirect call sites.
    pub indirect_sites: Vec<IndirectCallSite>,
    /// Return values per function.
    pub return_values: BTreeMap<FunctionId, Vec<ValueId>>,
    /// Constraint counts: \[addr, copy, load, store, gep\].
    pub constraint_counts: [usize; 5],
    /// Post-HVN constraint counts.
    pub post_hvn_constraint_counts: [usize; 5],
}

/// Output from the solver-specific PTA loop.
pub struct PtaSolveResult {
    /// Points-to map.
    pub pts: crate::pta::PointsToMap,
    /// Location factory (may have been modified by solver).
    pub factory: LocationFactory,
    /// Map from call-site to resolved targets (from PTA resolution).
    pub resolved_calls: BTreeMap<InstId, BTreeSet<FunctionId>>,
    /// Number of PTA iterations.
    pub iterations: usize,
    /// Time spent on PTA solving.
    pub pta_solve_secs: f64,
    /// Whether the solver hit its iteration limit.
    pub iteration_limit_hit: bool,
}

// =============================================================================
// Main entry point
// =============================================================================

/// Run CHA + online PTA call graph refinement.
///
/// See module-level documentation for algorithm details.
pub fn refine(
    module: &AirModule,
    config: &RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> RefinementResult {
    let _refine_span = tracing::info_span!("cg_refine").entered();
    let mut prepared = refine_prepare(module, config, specs);

    let solve_result = match config.pta_solver {
        PtaSolver::Worklist => refine_legacy(module, &mut prepared, config),
        PtaSolver::Datalog => {
            // When Datalog is selected but called through this wrapper
            // (which lives in saf-analysis, without saf-datalog dependency),
            // fall back to worklist. Callers that want Datalog should use
            // `saf_datalog::pta::refine_ascent()` directly.
            refine_legacy(module, &mut prepared, config)
        }
    };

    refine_finalize(module, prepared, solve_result)
}

/// Solver-agnostic preparation: CHA, initial CG, constraint extraction, HVN.
#[allow(clippy::too_many_lines)]
pub fn refine_prepare(
    module: &AirModule,
    config: &RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> RefinementPrepared {
    // --- Step 1: build CHA (if type hierarchy present) -----------------------
    let cha = if module.type_hierarchy.is_empty() {
        None
    } else {
        Some(ClassHierarchy::build(&module.type_hierarchy))
    };

    // --- Step 2: build initial call graph ------------------------------------
    let mut cg = CallGraph::build(module);

    // --- Step 3: bootstrap — resolve virtual calls via CHA -------------------
    let mut resolved_sites: BTreeMap<InstId, Vec<FunctionId>> = BTreeMap::new();
    let mut cha_resolved_sites: BTreeSet<InstId> = BTreeSet::new();

    if let Some(ref cha) = cha {
        resolve_virtual_calls_via_cha(module, &mut cg, cha, &mut resolved_sites);
        // Track which sites were CHA-resolved (for later PTA narrowing)
        cha_resolved_sites = resolved_sites.keys().copied().collect();
    }

    // --- Step 4a-4b: Extract constraints and HVN preprocess ------------------
    let func_loc_map = FunctionLocationMap::build(module);
    let indirect_sites = collect_indirect_call_sites(module);
    let return_values = collect_return_values(module);

    // 4a. Extract ALL constraints upfront (not just reachable)
    let mut factory = LocationFactory::new(config.field_sensitivity.clone());
    let mut constraints = extract_constraints(module, &mut factory);

    // Extract additional constraints from function specs for external library calls
    if let Some(specs) = specs {
        extract_spec_constraints(module, specs, &mut factory, &mut constraints);
    }

    let constraint_counts = [
        constraints.addr.len(),
        constraints.copy.len(),
        constraints.load.len(),
        constraints.store.len(),
        constraints.gep.len(),
    ];

    // 4b. HVN preprocessing
    let mut reduced = constraints.clone();
    let hvn_result = crate::pta::hvn::hvn_preprocess(&mut reduced);
    if hvn_result.removed > 0 {
        tracing::debug!(
            "HVN: {} classes, {} constraints removed ({}→{})",
            hvn_result.num_classes,
            hvn_result.removed,
            constraints.total_count(),
            reduced.total_count(),
        );
    }

    let post_hvn_constraint_counts = [
        reduced.addr.len(),
        reduced.copy.len(),
        reduced.load.len(),
        reduced.store.len(),
        reduced.gep.len(),
    ];

    RefinementPrepared {
        cg,
        cha,
        cha_resolved_sites,
        resolved_sites,
        constraints,
        reduced,
        hvn_result,
        factory,
        func_loc_map,
        indirect_sites,
        return_values,
        constraint_counts,
        post_hvn_constraint_counts,
    }
}

/// Run the legacy worklist PTA solver for CG refinement.
// NOTE: This function implements the worklist PTA solve + CG refinement loop
// as a single cohesive unit. Profiling is always compiled in and emitted via saf_log!.
// Timing variables (t_step, t_solver_init, t_solver_solve, t_cg_loop, t_normalize,
// t_hvn_expand, t_total) are intentionally named for readability in profiling output.
#[allow(clippy::too_many_lines, clippy::similar_names)]
fn refine_legacy(
    module: &AirModule,
    prepared: &mut RefinementPrepared,
    config: &RefinementConfig,
) -> PtaSolveResult {
    let pta_start = Timer::now();

    // 4c. Create solver and run initial fixed point
    let t_step = Timer::now();

    let mut solver = GenericSolver::<FxHashPtsSet>::new(&prepared.reduced, &prepared.factory)
        .with_constants(&module.constants);

    let t_solver_init = t_step.elapsed();

    let t_step = Timer::now();

    solver.solve(config.pta_config.max_iterations);

    let t_solver_solve = t_step.elapsed();

    // 4d. Online CG refinement loop
    let t_step = Timer::now();

    let mut resolved_pta_calls: BTreeMap<InstId, BTreeSet<FunctionId>> = BTreeMap::new();
    let mut iterations: usize = 1; // initial solve counts as iteration 1

    for _wave in 0..config.max_iterations {
        let found_new = resolve_and_connect(
            &mut solver,
            &prepared.indirect_sites,
            &prepared.func_loc_map,
            module,
            &prepared.return_values,
            &mut resolved_pta_calls,
            &mut prepared.cg,
        );

        if !found_new {
            break;
        }

        iterations += 1;
        solver.drain_worklist(config.pta_config.max_iterations);
    }

    let t_cg_loop = t_step.elapsed();

    // Print solver profile
    solver.print_stats("cg-refinement");

    // 4e. Normalize solver results to PointsToMap
    let t_step = Timer::now();

    let iteration_limit_hit = solver.iteration_limit_hit;

    let pts_count = solver.pts.len();
    let mut pts: crate::pta::PointsToMap = solver
        .pts
        .into_iter()
        .map(|(v, p)| (v, p.to_btreeset()))
        .collect();

    let t_normalize = t_step.elapsed();

    // Expand HVN mapping
    let t_step = Timer::now();

    let hvn_mapping_count = prepared.hvn_result.mapping.len();
    for (original, rep) in &prepared.hvn_result.mapping {
        if let Some(p) = pts.get(rep).cloned() {
            pts.insert(*original, p);
        }
    }

    let t_hvn_expand = t_step.elapsed();

    let pta_solve_secs = pta_start.elapsed_secs();
    let t_total = pta_start.elapsed();

    // Print pipeline profile
    saf_log!(callgraph::refine, stats, "profile";
        init = t_solver_init,
        solve = t_solver_solve,
        cg_loop = t_cg_loop,
        normalize = t_normalize,
        hvn_expand = t_hvn_expand,
        total = t_total,
        iterations = iterations,
        pts_count = pts_count,
        hvn_mappings = hvn_mapping_count,
    );

    PtaSolveResult {
        pts,
        factory: std::mem::replace(
            &mut prepared.factory,
            LocationFactory::new(FieldSensitivity::default()),
        ),
        resolved_calls: resolved_pta_calls,
        iterations,
        pta_solve_secs,
        iteration_limit_hit,
    }
}

/// Solver-agnostic finalization: CHA narrowing, `PtaResult`, ICFG.
pub fn refine_finalize(
    module: &AirModule,
    mut prepared: RefinementPrepared,
    solve_result: PtaSolveResult,
) -> RefinementResult {
    // 4f. Narrow CHA-resolved sites when PTA provides more precise results
    let pta_newly_resolved: BTreeMap<InstId, Vec<FunctionId>> = solve_result
        .resolved_calls
        .iter()
        .map(|(k, v)| (*k, v.iter().copied().collect()))
        .collect();

    debug_trace_cha_pta_resolution(
        module,
        &prepared.cha_resolved_sites,
        &prepared.resolved_sites,
        &pta_newly_resolved,
    );

    for (site, pta_targets) in &pta_newly_resolved {
        if prepared.cha_resolved_sites.contains(site) {
            let pta_set: BTreeSet<FunctionId> = pta_targets.iter().copied().collect();
            if let Some(prev_targets) = prepared.resolved_sites.get(site) {
                let cha_only: Vec<FunctionId> = prev_targets
                    .iter()
                    .filter(|t| !pta_set.contains(t))
                    .copied()
                    .collect();
                if !cha_only.is_empty() {
                    prepared.cg.remove_indirect_targets(*site, &cha_only);
                }
            }
            prepared.resolved_sites.insert(*site, pta_targets.clone());
        } else {
            prepared
                .resolved_sites
                .entry(*site)
                .or_default()
                .extend(pta_targets.iter().copied());
        }
    }

    for targets in prepared.resolved_sites.values_mut() {
        targets.sort();
        targets.dedup();
    }

    // 4g. Build PtaResult
    let constraint_count = prepared.reduced.total_count();
    let location_count = solve_result.factory.len();
    let diagnostics = crate::pta::PtaDiagnostics {
        iterations: solve_result.iterations,
        iteration_limit_hit: solve_result.iteration_limit_hit,
        constraint_count,
        location_count,
        ..Default::default()
    };
    let last_pta_result = Some(PtaResult::new(
        solve_result.pts,
        Arc::new(solve_result.factory),
        diagnostics,
    ));

    // --- Step 5: build final ICFG --------------------------------------------
    let icfg = Icfg::build(module, &prepared.cg);

    RefinementResult {
        call_graph: prepared.cg,
        icfg,
        pta_result: last_pta_result,
        cha: prepared.cha,
        iterations: solve_result.iterations,
        resolved_sites: prepared.resolved_sites,
        pta_solve_secs: solve_result.pta_solve_secs,
        constraint_counts: prepared.constraint_counts,
        post_hvn_constraint_counts: prepared.post_hvn_constraint_counts,
    }
}

// =============================================================================
// Online CG construction helpers
// =============================================================================

/// An indirect call site tracked for online CG resolution.
pub struct IndirectCallSite {
    /// The instruction ID of the `CallIndirect`.
    pub inst_id: InstId,
    /// All operands of the call (last one is the function pointer).
    pub operands: Vec<ValueId>,
    /// The destination value (call result), if any.
    pub dst: Option<ValueId>,
    /// Expected function signature at this call site, if known.
    /// Used for type-based call graph pruning.
    pub expected_signature: Option<TypeId>,
}

/// Collect all `CallIndirect` sites from the module.
pub fn collect_indirect_call_sites(module: &AirModule) -> Vec<IndirectCallSite> {
    use saf_core::air::Operation;

    let mut sites = Vec::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallIndirect { expected_signature } = &inst.op {
                    sites.push(IndirectCallSite {
                        inst_id: inst.id,
                        operands: inst.operands.clone(),
                        dst: inst.dst,
                        expected_signature: *expected_signature,
                    });
                }
            }
        }
    }
    sites
}

/// Build a map from `FunctionId` to its return values (operand\[0\] of each `Ret`).
pub fn collect_return_values(module: &AirModule) -> BTreeMap<FunctionId, Vec<ValueId>> {
    crate::module_index::collect_return_values(module)
}

/// Generate interprocedural copy constraints for a newly resolved indirect call target.
///
/// For a `CallIndirect` at `site` resolved to `callee_fid`:
/// - arg\[i\] → param\[i\] (actual to formal)
/// - callee return → call dst (return to caller)
pub fn collect_interprocedural_copies(
    site: &IndirectCallSite,
    callee_fid: FunctionId,
    module: &AirModule,
    return_values: &BTreeMap<FunctionId, Vec<ValueId>>,
) -> Vec<CopyConstraint> {
    let mut copies = Vec::new();

    let Some(callee_func) = module.functions.iter().find(|f| f.id == callee_fid) else {
        return copies;
    };

    // For indirect calls, the last operand is the function pointer;
    // all preceding operands are arguments.
    let args = if site.operands.is_empty() {
        &[] as &[ValueId]
    } else {
        &site.operands[..site.operands.len() - 1]
    };

    // arg → param
    for param in &callee_func.params {
        if let Some(&arg) = args.get(param.index as usize) {
            copies.push(CopyConstraint {
                dst: param.id,
                src: arg,
            });
        }
    }

    // return → call result
    if let Some(dst) = site.dst {
        if let Some(ret_vals) = return_values.get(&callee_fid) {
            for &ret_val in ret_vals {
                copies.push(CopyConstraint { dst, src: ret_val });
            }
        }
    }

    copies
}

/// Check if a function's signature is compatible with the expected signature
/// at a call site.
///
/// Compatible means: same number of parameters, compatible parameter types,
/// and compatible return types (void vs non-void, pointer vs non-pointer).
/// Returns `true` if either signature is unknown (conservative).
pub fn signature_compatible(
    expected: TypeId,
    callee_func: &AirFunction,
    module: &AirModule,
) -> bool {
    let Some(AirType::Function {
        params: expected_params,
        return_type: expected_ret,
    }) = module.get_type(expected)
    else {
        return true; // Unknown expected signature -- conservatively compatible
    };

    // Check parameter count
    if callee_func.params.len() != expected_params.len() {
        return false;
    }

    // Check return type compatibility.
    // A void-returning callee is incompatible with a call site that expects a
    // pointer return (and vice versa). We check this by comparing whether the
    // expected return type is void/pointer against what the callee actually
    // returns, inferred from its `Ret` instructions.
    let expected_ret_is_void = matches!(module.get_type(*expected_ret), Some(AirType::Void));
    let expected_ret_is_ptr = module.is_pointer_type(*expected_ret);

    // Determine if the callee function actually returns a value by scanning
    // its `Ret` instructions for operands.
    let callee_returns_value = callee_func.blocks.iter().any(|block| {
        block.instructions.iter().any(|inst| {
            matches!(inst.op, saf_core::air::Operation::Ret) && !inst.operands.is_empty()
        })
    });

    if expected_ret_is_void && callee_returns_value {
        return false;
    }
    if expected_ret_is_ptr && !callee_returns_value {
        return false;
    }

    // Check parameter type compatibility
    for (callee_param, &expected_type_id) in callee_func.params.iter().zip(expected_params.iter()) {
        let Some(callee_type_id) = callee_param.param_type else {
            continue; // Unknown callee param type -- skip check
        };
        if callee_type_id != expected_type_id {
            // Types differ -- check if they are structurally compatible.
            // For now, just check that both are pointers or both are non-pointers.
            let callee_is_ptr = module.is_pointer_type(callee_type_id);
            let expected_is_ptr = module.is_pointer_type(expected_type_id);
            if callee_is_ptr != expected_is_ptr {
                return false;
            }
        }
    }

    true
}

/// Resolve indirect calls from the solver's current points-to sets.
///
/// Returns `true` if any new call targets were discovered and added
/// to the solver.
// NOTE: This function implements the resolve-then-connect pattern as a single
// cohesive unit. Splitting would obscure the two-phase (read/write) structure.
#[allow(clippy::too_many_lines)]
fn resolve_and_connect<P: PtsSet>(
    solver: &mut GenericSolver<'_, P>,
    indirect_sites: &[IndirectCallSite],
    func_loc_map: &FunctionLocationMap,
    module: &AirModule,
    return_values: &BTreeMap<FunctionId, Vec<ValueId>>,
    resolved_calls: &mut BTreeMap<InstId, BTreeSet<FunctionId>>,
    cg: &mut CallGraph,
) -> bool {
    // Phase 1: Read — collect new targets and copy constraints
    let mut new_copies: Vec<CopyConstraint> = Vec::new();
    let mut found_new = false;

    for site in indirect_sites {
        // The callee pointer is always the last operand; skip argument operands
        // to avoid spuriously resolving argument values as call targets.
        if let Some(&callee_val) = site.operands.last() {
            if let Some(pts_set) = solver.pts.get(&callee_val) {
                for loc_id in pts_set.iter() {
                    if let Some(loc) = solver.factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            // Type-based pruning: skip targets with incompatible signatures
                            if let Some(sig) = site.expected_signature {
                                if !module
                                    .function(fid)
                                    .is_none_or(|f| signature_compatible(sig, f, module))
                                {
                                    continue;
                                }
                            }
                            if resolved_calls.entry(site.inst_id).or_default().insert(fid) {
                                new_copies.extend(collect_interprocedural_copies(
                                    site,
                                    fid,
                                    module,
                                    return_values,
                                ));
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }

        // Also check dst value's points-to set
        if let Some(dst) = site.dst {
            if let Some(pts_set) = solver.pts.get(&dst) {
                for loc_id in pts_set.iter() {
                    if let Some(loc) = solver.factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            // Type-based pruning: skip targets with incompatible signatures
                            if let Some(sig) = site.expected_signature {
                                if !module
                                    .function(fid)
                                    .is_none_or(|f| signature_compatible(sig, f, module))
                                {
                                    continue;
                                }
                            }
                            if resolved_calls.entry(site.inst_id).or_default().insert(fid) {
                                new_copies.extend(collect_interprocedural_copies(
                                    site,
                                    fid,
                                    module,
                                    return_values,
                                ));
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_new {
        return false;
    }

    // Update CG edges for all newly resolved calls
    for (inst_id, targets) in &*resolved_calls {
        let target_vec: Vec<FunctionId> = targets.iter().copied().collect();
        cg.resolve_indirect(*inst_id, &target_vec);
    }

    // Phase 2: Write — add all new constraints to solver
    for copy in new_copies {
        solver.add_copy_constraint(copy);
    }
    solver.recompute_topo_order();

    true
}

/// Resolve indirect calls from a points-to map (solver-agnostic).
///
/// Reads `pts` to discover new indirect call targets, adds them to `cg`,
/// and returns the interprocedural copy constraints that need to be added
/// to the solver's constraint set.
///
/// Returns an empty vec if no new targets were found.
#[allow(clippy::too_many_arguments)]
pub fn resolve_indirect_calls_from_pts(
    pts: &crate::pta::PointsToMap,
    factory: &LocationFactory,
    indirect_sites: &[IndirectCallSite],
    func_loc_map: &FunctionLocationMap,
    module: &AirModule,
    return_values: &BTreeMap<FunctionId, Vec<ValueId>>,
    resolved_calls: &mut BTreeMap<InstId, BTreeSet<FunctionId>>,
    cg: &mut CallGraph,
) -> Vec<CopyConstraint> {
    let mut new_copies: Vec<CopyConstraint> = Vec::new();
    let mut found_new = false;

    for site in indirect_sites {
        if let Some(&callee_val) = site.operands.last() {
            if let Some(pts_set) = pts.get(&callee_val) {
                for &loc_id in pts_set {
                    if let Some(loc) = factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            if let Some(sig) = site.expected_signature {
                                if !module
                                    .function(fid)
                                    .is_none_or(|f| signature_compatible(sig, f, module))
                                {
                                    continue;
                                }
                            }
                            if resolved_calls.entry(site.inst_id).or_default().insert(fid) {
                                new_copies.extend(collect_interprocedural_copies(
                                    site,
                                    fid,
                                    module,
                                    return_values,
                                ));
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }

        // Also check dst value's points-to set
        if let Some(dst) = site.dst {
            if let Some(pts_set) = pts.get(&dst) {
                for &loc_id in pts_set {
                    if let Some(loc) = factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            if let Some(sig) = site.expected_signature {
                                if !module
                                    .function(fid)
                                    .is_none_or(|f| signature_compatible(sig, f, module))
                                {
                                    continue;
                                }
                            }
                            if resolved_calls.entry(site.inst_id).or_default().insert(fid) {
                                new_copies.extend(collect_interprocedural_copies(
                                    site,
                                    fid,
                                    module,
                                    return_values,
                                ));
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if found_new {
        // Update CG edges for all newly resolved calls
        for (inst_id, targets) in &*resolved_calls {
            let target_vec: Vec<FunctionId> = targets.iter().copied().collect();
            cg.resolve_indirect(*inst_id, &target_vec);
        }
    }

    new_copies
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Print debug traces comparing CHA vs PTA resolution for each call site.
///
/// Only produces output when `SAF_DEBUG_CHA` env var is set.
fn debug_trace_cha_pta_resolution(
    module: &AirModule,
    cha_resolved_sites: &BTreeSet<InstId>,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
    newly_resolved: &BTreeMap<InstId, Vec<FunctionId>>,
) {
    if std::env::var("SAF_DEBUG_CHA").is_err() {
        return;
    }

    for site in cha_resolved_sites {
        let cha_targets = resolved_sites.get(site).map_or(0, Vec::len);
        let pta_count = newly_resolved.get(site).map_or(0, Vec::len);

        let resolve_names = |targets: &[FunctionId]| -> Vec<&str> {
            targets
                .iter()
                .filter_map(|fid| {
                    module
                        .functions
                        .iter()
                        .find(|f| f.id == *fid)
                        .map(|f| f.name.as_str())
                })
                .collect()
        };

        let pta_names: Vec<_> = newly_resolved
            .get(site)
            .map_or(vec![], |t| resolve_names(t));
        let cha_names: Vec<_> = resolved_sites
            .get(site)
            .map_or(vec![], |t| resolve_names(t));

        eprintln!(
            "CHA site {site:?}: cha=[{cha_targets}]{cha_names:?}, pta=[{pta_count}]{pta_names:?}"
        );
    }
}

// =============================================================================
// Incremental CG Refinement
// =============================================================================

/// Difference produced by incremental call graph refinement.
///
/// Tracks which edges were added or removed and which functions became newly
/// reachable after re-resolving indirect call sites with updated PTA results.
#[derive(Debug, Clone, Default)]
pub struct CgRefinementDiff {
    /// Edges added during incremental refinement (caller, callee).
    pub added_edges: Vec<(FunctionId, FunctionId)>,
    /// Edges removed during incremental refinement (caller, callee).
    pub removed_edges: Vec<(FunctionId, FunctionId)>,
    /// Functions that became newly reachable after adding edges.
    pub newly_reachable: BTreeSet<FunctionId>,
}

/// Incrementally refine a call graph using updated PTA results.
///
/// For each indirect call site in `module`, re-resolves targets using the
/// current points-to map. Compares with `previous_indirect_targets` and
/// updates `call_graph` with added/removed edges.
///
/// This is the incremental counterpart to the full CG refinement loop in
/// [`refine`]. Instead of re-running the entire CHA+PTA pipeline, it only
/// re-resolves indirect call sites that may have changed.
pub fn refine_incremental(
    call_graph: &mut CallGraph,
    module: &AirModule,
    pts: &crate::pta::PointsToMap,
    factory: &LocationFactory,
    func_loc_map: &FunctionLocationMap,
    previous_indirect_targets: &BTreeMap<InstId, BTreeSet<FunctionId>>,
) -> CgRefinementDiff {
    let mut diff = CgRefinementDiff::default();

    // Collect all indirect call sites
    let indirect_sites = collect_indirect_call_sites(module);

    // Build caller lookup: InstId -> caller FunctionId
    let caller_of: BTreeMap<InstId, FunctionId> = {
        let mut map = BTreeMap::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if matches!(inst.op, saf_core::air::Operation::CallIndirect { .. }) {
                        map.insert(inst.id, func.id);
                    }
                }
            }
        }
        map
    };

    // Re-resolve each indirect call site using updated PTS
    for site in &indirect_sites {
        let mut current_targets = BTreeSet::new();

        // Resolve from the function pointer operand (last operand)
        if let Some(&callee_val) = site.operands.last() {
            if let Some(pts_set) = pts.get(&callee_val) {
                for &loc_id in pts_set {
                    if let Some(loc) = factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            current_targets.insert(fid);
                        }
                    }
                }
            }
        }

        let previous = previous_indirect_targets
            .get(&site.inst_id)
            .cloned()
            .unwrap_or_default();

        let Some(&caller_fid) = caller_of.get(&site.inst_id) else {
            continue;
        };

        // Find added targets
        for &target in &current_targets {
            if !previous.contains(&target) {
                diff.added_edges.push((caller_fid, target));
                diff.newly_reachable.insert(target);

                // Add edge in callgraph
                call_graph.resolve_indirect(site.inst_id, &[target]);
            }
        }

        // Find removed targets
        for &target in &previous {
            if !current_targets.contains(&target) {
                diff.removed_edges.push((caller_fid, target));

                // Remove edge from callgraph
                call_graph.remove_indirect_targets(site.inst_id, &[target]);
            }
        }
    }

    diff
}

/// Determine the entry-point `FunctionId`s based on the strategy.
#[cfg(test)]
fn entry_function_ids(module: &AirModule, strategy: &EntryPointStrategy) -> BTreeSet<FunctionId> {
    match strategy {
        EntryPointStrategy::AllDefined => module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| f.id)
            .collect(),
        EntryPointStrategy::Named(names) => {
            let name_set: BTreeSet<&str> = names.iter().map(String::as_str).collect();
            module
                .functions
                .iter()
                .filter(|f| name_set.contains(f.name.as_str()))
                .map(|f| f.id)
                .collect()
        }
    }
}

/// Compute the set of reachable `FunctionId`s from entry points via multi-source
/// BFS on the call graph. This avoids redundant per-entry DFS traversals by
/// starting from all entries simultaneously.
#[cfg(test)]
fn compute_reachable(cg: &CallGraph, entries: &BTreeSet<FunctionId>) -> BTreeSet<FunctionId> {
    use std::collections::VecDeque;

    // Multi-source BFS: start from ALL entries simultaneously
    let mut reachable = BTreeSet::new();
    let mut queue: VecDeque<CallGraphNode> = VecDeque::new();
    let mut visited: BTreeSet<CallGraphNode> = BTreeSet::new();

    for &entry in entries {
        let node = CallGraphNode::Function(entry);
        if cg.nodes.contains(&node) && visited.insert(node.clone()) {
            queue.push_back(node);
            reachable.insert(entry);
        }
    }

    while let Some(node) = queue.pop_front() {
        if let Some(succs) = cg.successors(&node) {
            for succ in succs {
                if visited.insert(succ.clone()) {
                    if let Some(fid) = succ.function_id() {
                        reachable.insert(fid);
                    }
                    queue.push_back(succ.clone());
                }
            }
        }
    }
    reachable
}

/// Virtual call pattern info extracted from instruction chain.
#[derive(Debug)]
struct VirtualCallInfo {
    /// The vtable slot index (0-based, after skipping vtable metadata).
    slot_index: usize,
}

/// Build a map from ValueId to the instruction that defines it, for a single function.
fn build_def_map(
    func: &saf_core::air::AirFunction,
) -> BTreeMap<ValueId, &saf_core::air::Instruction> {
    let mut defs = BTreeMap::new();
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                defs.insert(dst, inst);
            }
        }
    }
    defs
}

/// Try to match the virtual call pattern and extract the vtable slot index.
///
/// Virtual call pattern in LLVM IR / AIR (4 instructions):
/// ```text
/// %vptr = load ptr, ptr %obj           ; (1) Load vtable pointer FROM object
/// %slot = getelementptr ptr, %vptr, N  ; (2) Index INTO loaded vtable pointer
/// %fn   = load ptr, ptr %slot          ; (3) Load function pointer from slot
/// call void %fn(...)                   ; (4) Indirect call
/// ```
///
/// Function-pointer-in-class pattern (NOT a virtual call — only 3 instructions):
/// ```text
/// %field = getelementptr %class, ptr %obj, i32 0, i32 1  ; GEP into object struct
/// %fn    = load ptr, ptr %field                            ; Load function pointer
/// call void %fn(...)                                       ; Indirect call
/// ```
///
/// The key difference: for virtual calls, the GEP base (`%vptr`) is itself the
/// result of a Load instruction (the vptr was loaded from the object). For
/// function pointer members, the GEP base is the object pointer directly.
///
/// Returns `Some(VirtualCallInfo)` if the pattern matches, `None` otherwise.
fn match_virtual_call_pattern(
    call_inst: &saf_core::air::Instruction,
    def_map: &BTreeMap<ValueId, &saf_core::air::Instruction>,
) -> Option<VirtualCallInfo> {
    use saf_core::air::{FieldStep, Operation};

    // CallIndirect's LAST operand is the function pointer being called
    // (LLVM IR puts the callee as the last operand in get_operand())
    let fn_ptr_val = *call_inst.operands.last()?;

    // Step 3: The function pointer should be loaded from memory (Load instruction)
    let fn_load_inst = def_map.get(&fn_ptr_val)?;
    if !matches!(fn_load_inst.op, Operation::Load) {
        return None;
    }

    // The Load's operand is the pointer to the vtable slot (result of GEP)
    let slot_ptr_val = *fn_load_inst.operands.first()?;

    // Step 2: The slot pointer should come from a GEP instruction
    let gep_inst = def_map.get(&slot_ptr_val)?;
    let Operation::Gep { field_path } = &gep_inst.op else {
        return None;
    };

    // Extract slot index from field_path
    let slot_index = match field_path.steps.first()? {
        FieldStep::Field { index } => *index as usize,
        FieldStep::Index => {
            // Dynamic index - can't determine slot statically
            return None;
        }
    };

    // Step 1: The GEP's base pointer (%vptr) must itself be a Load result.
    // This distinguishes virtual calls (load vptr from object, then GEP into vptr)
    // from function-pointer-in-class access (GEP directly into object struct).
    let vptr_val = *gep_inst.operands.first()?;
    let vptr_load_inst = def_map.get(&vptr_val)?;
    if !matches!(vptr_load_inst.op, Operation::Load) {
        return None;
    }

    // Verify the vptr was loaded from the receiver object (validates the pattern)
    let _receiver_operand = vptr_load_inst.operands.first()?;

    Some(VirtualCallInfo { slot_index })
}

/// Resolve virtual calls via CHA.
///
/// Scans all `CallIndirect` instructions in the module and attempts to match
/// the virtual call pattern (Load vptr → GEP slot → Load fn_ptr → `CallIndirect`).
///
/// For matched patterns, resolves using CHA at the specific vtable slot,
/// restricted to root classes only (to prevent cross-hierarchy contamination).
///
/// For unmatched patterns (e.g., function pointer member calls), the call is
/// left unresolved for CHA — PTA-based resolution in the iterative loop will
/// handle it.
fn resolve_virtual_calls_via_cha(
    module: &AirModule,
    cg: &mut CallGraph,
    cha: &ClassHierarchy,
    resolved_sites: &mut BTreeMap<InstId, Vec<FunctionId>>,
) {
    use saf_core::air::Operation;

    // Pre-compute root classes for resolution.
    // Resolving at root classes covers every subclass within each independent
    // hierarchy without cross-hierarchy contamination (RC1 fix).
    let root_classes = cha.root_classes();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        let def_map = build_def_map(func);

        for block in &func.blocks {
            for inst in &block.instructions {
                if !matches!(inst.op, Operation::CallIndirect { .. }) {
                    continue;
                }

                // Try to match virtual call pattern to get specific slot
                let Some(vcall_info) = match_virtual_call_pattern(inst, &def_map) else {
                    // Pattern didn't match — not a virtual call (e.g., function
                    // pointer member or non-standard indirect call). Skip CHA
                    // resolution; PTA will handle it in the refinement loop.
                    continue;
                };

                let mut targets = BTreeSet::new();

                // Resolve at root classes only — this covers every subclass
                // within each hierarchy without cross-hierarchy contamination.
                for root in &root_classes {
                    let resolved = cha.resolve_virtual(root, vcall_info.slot_index);
                    targets.extend(resolved);
                }

                if !targets.is_empty() {
                    let target_vec: Vec<FunctionId> = targets.into_iter().collect();
                    cg.resolve_indirect(inst.id, &target_vec);
                    resolved_sites.insert(inst.id, target_vec);
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{
        AirBlock, AirFunction, AirGlobal, Constant, Instruction, Operation, TypeHierarchyEntry,
        VirtualMethodSlot,
    };
    use saf_core::ids::{BlockId, InstId, ModuleId, ObjId, ValueId};

    fn make_module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions,
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    fn make_function(id: u128, name: &str, blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_declaration(id: u128, name: &str) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_block(id: u128, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions,
        }
    }

    // -------------------------------------------------------------------------
    // 1. Refinement with no indirect calls converges in 1 iteration
    // -------------------------------------------------------------------------
    #[test]
    fn refine_no_indirect_calls() {
        // main calls helper directly — no refinement needed
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let config = RefinementConfig::default();
        let result = refine(&module, &config, None);

        // Should converge quickly
        assert_eq!(result.iterations, 1);
        assert!(result.resolved_sites.is_empty());
        assert!(result.cha.is_none());
    }

    // -------------------------------------------------------------------------
    // 2. Entry point strategy: Named filters correctly
    // -------------------------------------------------------------------------
    #[test]
    fn entry_point_named_strategy() {
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![Instruction::new(InstId::new(100), Operation::Ret)],
            )],
        );
        let other = make_function(
            2,
            "other",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, other]);

        let entries = entry_function_ids(
            &module,
            &EntryPointStrategy::Named(vec!["main".to_string()]),
        );
        assert_eq!(entries.len(), 1);
        assert!(entries.contains(&FunctionId::new(1)));
    }

    // -------------------------------------------------------------------------
    // 3. Entry point strategy: AllDefined includes all non-declarations
    // -------------------------------------------------------------------------
    #[test]
    fn entry_point_all_defined_strategy() {
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![Instruction::new(InstId::new(100), Operation::Ret)],
            )],
        );
        let ext = make_declaration(2, "printf");
        let helper = make_function(
            3,
            "helper",
            vec![make_block(
                3,
                vec![Instruction::new(InstId::new(300), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, ext, helper]);

        let entries = entry_function_ids(&module, &EntryPointStrategy::AllDefined);
        // Only non-declarations
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&FunctionId::new(1)));
        assert!(entries.contains(&FunctionId::new(3)));
    }

    // -------------------------------------------------------------------------
    // 4. compute_reachable follows call edges
    // -------------------------------------------------------------------------
    #[test]
    fn compute_reachable_follows_edges() {
        // main -> helper -> leaf, orphan is not reachable from main
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![
                    Instruction::new(
                        InstId::new(200),
                        Operation::CallDirect {
                            callee: FunctionId::new(3),
                        },
                    ),
                    Instruction::new(InstId::new(201), Operation::Ret),
                ],
            )],
        );
        let leaf = make_function(
            3,
            "leaf",
            vec![make_block(
                3,
                vec![Instruction::new(InstId::new(300), Operation::Ret)],
            )],
        );
        let orphan = make_function(
            4,
            "orphan",
            vec![make_block(
                4,
                vec![Instruction::new(InstId::new(400), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper, leaf, orphan]);
        let cg = CallGraph::build(&module);

        let entries: BTreeSet<FunctionId> = [FunctionId::new(1)].into_iter().collect();
        let reachable = compute_reachable(&cg, &entries);

        assert!(reachable.contains(&FunctionId::new(1)));
        assert!(reachable.contains(&FunctionId::new(2)));
        assert!(reachable.contains(&FunctionId::new(3)));
        assert!(!reachable.contains(&FunctionId::new(4)));
    }

    // -------------------------------------------------------------------------
    // 5. CHA resolves virtual calls (with proper vtable load pattern)
    // -------------------------------------------------------------------------
    #[test]
    fn refine_with_cha_resolves_virtual_calls() {
        use saf_core::air::FieldPath as AirFieldPath;

        let base_func = FunctionId::derive(b"Base::process");
        let derived_func = FunctionId::derive(b"Derived::process");

        // Build a proper virtual call pattern:
        //   %obj = alloca             ; receiver object
        //   %vptr = load ptr, %obj    ; load vtable pointer from object
        //   %slot = gep %vptr, 0      ; index into vtable at slot 0
        //   %fn = load ptr, %slot     ; load function pointer from slot
        //   call %fn(...)             ; indirect call
        let obj_val = ValueId::derive(b"obj");
        let vptr_val = ValueId::derive(b"vptr");
        let slot_val = ValueId::derive(b"slot_ptr");
        let fn_val = ValueId::derive(b"fn_ptr");

        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    // %obj = alloca
                    Instruction::new(InstId::new(96), Operation::Alloca { size_bytes: None })
                        .with_dst(obj_val),
                    // %vptr = load %obj
                    Instruction::new(InstId::new(97), Operation::Load)
                        .with_operands(vec![obj_val])
                        .with_dst(vptr_val),
                    // %slot = gep %vptr, field(0)
                    Instruction::new(
                        InstId::new(98),
                        Operation::Gep {
                            field_path: AirFieldPath::field(0),
                        },
                    )
                    .with_operands(vec![vptr_val])
                    .with_dst(slot_val),
                    // %fn = load %slot
                    Instruction::new(InstId::new(99), Operation::Load)
                        .with_operands(vec![slot_val])
                        .with_dst(fn_val),
                    // call indirect %fn
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallIndirect {
                            expected_signature: None,
                        },
                    )
                    .with_operands(vec![fn_val]),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        // Base::process and Derived::process as defined functions
        let base_process = make_function(
            base_func.raw() as u128,
            "Base_process",
            vec![make_block(
                10,
                vec![Instruction::new(InstId::new(1000), Operation::Ret)],
            )],
        );
        let derived_process = make_function(
            derived_func.raw() as u128,
            "Derived_process",
            vec![make_block(
                20,
                vec![Instruction::new(InstId::new(2000), Operation::Ret)],
            )],
        );

        let mut module = make_module(vec![main, base_process, derived_process]);

        // Add type hierarchy (Base is the root, Derived inherits from Base)
        module.type_hierarchy = vec![
            TypeHierarchyEntry {
                type_name: "Base".to_string(),
                base_types: Vec::new(),
                virtual_methods: vec![VirtualMethodSlot {
                    index: 0,
                    function: Some(base_func),
                }],
            },
            TypeHierarchyEntry {
                type_name: "Derived".to_string(),
                base_types: vec!["Base".to_string()],
                virtual_methods: vec![VirtualMethodSlot {
                    index: 0,
                    function: Some(derived_func),
                }],
            },
        ];

        let config = RefinementConfig::default();
        let result = refine(&module, &config, None);

        // CHA should have resolved the virtual call
        assert!(result.cha.is_some());
        assert!(!result.resolved_sites.is_empty());

        // The indirect call at InstId(100) should be resolved to both functions
        let site_targets = result.resolved_sites.get(&InstId::new(100));
        assert!(site_targets.is_some());
        let targets = site_targets.unwrap();
        assert!(targets.contains(&base_func));
        assert!(targets.contains(&derived_func));
    }

    // -------------------------------------------------------------------------
    // 6. PTA resolves function pointer calls via global initializer
    // -------------------------------------------------------------------------
    #[test]
    fn refine_with_pta_resolves_fptr_call() {
        let _target_id = FunctionId::new(42);

        // main has an indirect call with an operand that is the function pointer
        let fptr_value = ValueId::derive(b"fptr_loaded");
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    // Alloca for the function pointer
                    Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
                        .with_dst(ValueId::derive(b"alloca_ptr")),
                    // Load the function pointer
                    Instruction::new(InstId::new(101), Operation::Load)
                        .with_operands(vec![ValueId::derive(b"alloca_ptr")])
                        .with_dst(fptr_value),
                    // Indirect call using the loaded function pointer
                    Instruction::new(
                        InstId::new(102),
                        Operation::CallIndirect {
                            expected_signature: None,
                        },
                    )
                    .with_operands(vec![fptr_value]),
                    Instruction::new(InstId::new(103), Operation::Ret),
                ],
            )],
        );

        // target function
        let target_fn = make_function(
            42,
            "target_func",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );

        let mut module = make_module(vec![main, target_fn]);

        // Add a global initializer with the function pointer
        let global_obj = ObjId::derive(b"fptr_global");
        let global_value = ValueId::derive(b"fptr_global_val");
        let mut global = AirGlobal::new(global_value, global_obj, "fptr_global");
        global.init = Some(Constant::Aggregate {
            elements: vec![Constant::Int {
                value: 42,
                bits: 64,
            }],
        });
        module.globals.push(global);

        let config = RefinementConfig::default();
        let result = refine(&module, &config, None);

        // PTA should have produced a result
        assert!(result.pta_result.is_some());
        // At least one iteration ran
        assert!(result.iterations >= 1);
    }

    // -------------------------------------------------------------------------
    // 7. Empty module yields empty result
    // -------------------------------------------------------------------------
    #[test]
    fn refine_empty_module() {
        let module = make_module(Vec::new());
        let config = RefinementConfig::default();
        let result = refine(&module, &config, None);

        assert_eq!(result.iterations, 1);
        assert!(result.resolved_sites.is_empty());
        assert!(result.cha.is_none());
    }

    // -------------------------------------------------------------------------
    // 8. Determinism: running refine twice yields identical results
    // -------------------------------------------------------------------------
    #[test]
    fn refine_is_deterministic() {
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(
                        InstId::new(101),
                        Operation::CallIndirect {
                            expected_signature: None,
                        },
                    ),
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let config = RefinementConfig::default();

        let r1 = refine(&module, &config, None);
        let r2 = refine(&module, &config, None);

        assert_eq!(r1.iterations, r2.iterations);
        assert_eq!(r1.resolved_sites, r2.resolved_sites);
        assert_eq!(r1.call_graph, r2.call_graph);
        assert_eq!(r1.icfg, r2.icfg);
    }

    // -------------------------------------------------------------------------
    // Incremental CG refinement
    // -------------------------------------------------------------------------

    #[test]
    fn cg_refinement_diff_default() {
        let diff = CgRefinementDiff::default();
        assert!(diff.added_edges.is_empty());
        assert!(diff.removed_edges.is_empty());
        assert!(diff.newly_reachable.is_empty());
    }

    #[test]
    fn refine_incremental_no_indirect_calls() {
        // Module with only direct calls - incremental refinement should be a no-op
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let mut cg = CallGraph::build(&module);
        let factory = crate::pta::LocationFactory::new(FieldSensitivity::None);
        let func_loc_map = FunctionLocationMap::build(&module);
        let pts = crate::pta::PointsToMap::new();
        let previous = BTreeMap::new();

        let diff = refine_incremental(&mut cg, &module, &pts, &factory, &func_loc_map, &previous);

        assert!(diff.added_edges.is_empty());
        assert!(diff.removed_edges.is_empty());
        assert!(diff.newly_reachable.is_empty());
    }

    #[test]
    fn refine_incremental_detects_new_targets() {
        // Module with indirect call
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallIndirect {
                            expected_signature: None,
                        },
                    )
                    .with_operands(vec![ValueId::new(50)]),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let target = make_function(
            2,
            "target",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, target]);
        let mut cg = CallGraph::build(&module);

        // Set up PTS and func_loc_map so that val(50) points to the
        // location for target function.
        // FunctionLocationMap::build maps ObjId::new(func.id.raw()) -> func.id,
        // so ObjId::new(2) -> FunctionId::new(2) is automatically created.
        let mut factory = crate::pta::LocationFactory::new(FieldSensitivity::None);
        let obj = ObjId::new(2); // Same as target's function id raw value
        let loc = factory.get_or_create(obj, crate::pta::FieldPath::empty());
        let func_loc_map = FunctionLocationMap::build(&module);

        let mut pts = crate::pta::PointsToMap::new();
        let mut pts_set = BTreeSet::new();
        pts_set.insert(loc);
        pts.insert(ValueId::new(50), pts_set);

        let previous = BTreeMap::new(); // No previous targets

        let diff = refine_incremental(&mut cg, &module, &pts, &factory, &func_loc_map, &previous);

        assert_eq!(diff.added_edges.len(), 1);
        assert_eq!(
            diff.added_edges[0],
            (FunctionId::new(1), FunctionId::new(2))
        );
        assert!(diff.newly_reachable.contains(&FunctionId::new(2)));
        assert!(diff.removed_edges.is_empty());
    }

    #[test]
    fn refine_incremental_detects_removed_targets() {
        // Module with indirect call
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallIndirect {
                            expected_signature: None,
                        },
                    )
                    .with_operands(vec![ValueId::new(50)]),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let target = make_function(
            2,
            "target",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, target]);
        let mut cg = CallGraph::build(&module);

        let factory = crate::pta::LocationFactory::new(FieldSensitivity::None);
        let func_loc_map = FunctionLocationMap::build(&module);
        let pts = crate::pta::PointsToMap::new(); // Empty PTS -> no targets

        // Previous had target resolved
        let mut previous = BTreeMap::new();
        let mut prev_targets = BTreeSet::new();
        prev_targets.insert(FunctionId::new(2));
        previous.insert(InstId::new(100), prev_targets);

        let diff = refine_incremental(&mut cg, &module, &pts, &factory, &func_loc_map, &previous);

        assert!(diff.added_edges.is_empty());
        assert_eq!(diff.removed_edges.len(), 1);
        assert_eq!(
            diff.removed_edges[0],
            (FunctionId::new(1), FunctionId::new(2))
        );
    }
}
