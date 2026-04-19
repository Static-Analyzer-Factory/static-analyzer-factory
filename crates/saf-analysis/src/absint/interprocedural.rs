//! Interprocedural abstract interpretation.
//!
//! This module extends the intraprocedural abstract interpretation with
//! cross-function analysis via function summaries.
//!
//! Key features:
//! - Function summaries (return value intervals)
//! - Parameter binding (caller arguments → callee parameters)
//! - Return value propagation
//! - Context-sensitive call site refinement
//! - Spec-based summaries for external functions

use super::nullness::Nullness;
use super::{
    AbstractInterpConfig, AbstractInterpResult, Interval,
    domain::AbstractDomain,
    fixpoint::{
        apply_refinements, build_cond_inst_map, collect_refinements, detect_loop_headers,
        narrow_state, refine_for_successor, reverse_postorder,
        solve_abstract_interp_with_pta_and_summaries, solve_function_with_params, widen_state,
    },
    pta_integration::PtaIntegration,
    result::AbstractInterpDiagnostics,
    state::AbstractState,
    threshold::extract_thresholds,
    transfer::{
        TransferContext, build_constant_map, propagate_refinement_to_loc_memory,
        refine_branch_condition, transfer_instruction_with_context,
    },
};
use crate::PtaResult;
use crate::cfg::Cfg;
use crate::graph_algo::{Successors, tarjan_scc};
use crate::pta::mod_ref::compute_all_mod_ref_with_specs;
use saf_core::air::{AirModule, Constant, Operation};
use saf_core::ids::{FunctionId, InstId, LocId, ValueId};
use saf_core::saf_log;
use saf_core::spec::SpecRegistry;
use std::collections::{BTreeMap, BTreeSet};

/// Default bit width for intervals (64-bit).
const DEFAULT_BITS: u8 = 64;

// ---------------------------------------------------------------------------
// Interprocedural Analysis Context
// ---------------------------------------------------------------------------

/// Context for interprocedural analysis.
///
/// This struct encapsulates optional components that can enhance the
/// interprocedural analysis:
/// - `pta`: Pointer analysis results for indirect call resolution and mod/ref
/// - `specs`: External function specifications for summary lookup
pub struct InterproceduralContext<'a> {
    /// Optional pointer analysis result for alias-aware analysis.
    pub pta: Option<&'a PtaResult>,
    /// Optional spec registry for external function summaries.
    pub specs: Option<&'a SpecRegistry>,
}

impl<'a> InterproceduralContext<'a> {
    /// Create a new empty context (baseline analysis).
    #[must_use]
    pub fn new() -> Self {
        Self {
            pta: None,
            specs: None,
        }
    }

    /// Create a context with only PTA.
    #[must_use]
    pub fn with_pta(pta: &'a PtaResult) -> Self {
        Self {
            pta: Some(pta),
            specs: None,
        }
    }

    /// Create a context with only specs.
    #[must_use]
    pub fn with_specs(specs: &'a SpecRegistry) -> Self {
        Self {
            pta: None,
            specs: Some(specs),
        }
    }

    /// Create a context with both PTA and specs.
    #[must_use]
    pub fn with_pta_and_specs(pta: &'a PtaResult, specs: &'a SpecRegistry) -> Self {
        Self {
            pta: Some(pta),
            specs: Some(specs),
        }
    }
}

impl Default for InterproceduralContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a function's abstract behavior.
#[derive(Debug, Clone)]
pub struct FunctionSummary {
    /// Return value interval (None if function is void or returns pointer)
    return_interval: Option<Interval>,
    /// Return value nullness (for pointer-returning functions).
    return_nullness: Option<Nullness>,
    /// Parameter bindings that were analyzed
    param_count: usize,
    /// Locations this function may modify (from mod/ref analysis).
    modified_locs: BTreeSet<LocId>,
    /// Whether function may modify unknown locations.
    modifies_unknown: bool,
    /// Memory side-effects: parameter index → interval written to the parameter's pointee.
    ///
    /// Records `*param[i] = V` effects for interprocedural memory propagation (Plan 084 Phase C).
    /// When a function stores a known interval to a location pointed to by parameter `i`,
    /// this enables callers to apply those effects even when inline analysis is not feasible.
    param_store_effects: BTreeMap<usize, Interval>,
    /// Global variable store effects: `LocId` → interval written to the global.
    ///
    /// Records effects like `g = 3` for interprocedural propagation (Plan 086 Phase H4).
    /// Applied at call sites so callers see precise global values after the call.
    global_store_effects: BTreeMap<LocId, Interval>,
}

impl FunctionSummary {
    /// Create a new empty summary.
    pub fn new() -> Self {
        Self {
            return_interval: None,
            return_nullness: None,
            param_count: 0,
            modified_locs: BTreeSet::new(),
            modifies_unknown: false,
            param_store_effects: BTreeMap::new(),
            global_store_effects: BTreeMap::new(),
        }
    }

    /// Get the return value interval.
    pub fn return_interval(&self) -> Option<&Interval> {
        self.return_interval.as_ref()
    }

    /// Get the return value nullness.
    pub fn return_nullness(&self) -> Option<Nullness> {
        self.return_nullness
    }

    /// Get the modified locations.
    pub fn modified_locations(&self) -> &BTreeSet<LocId> {
        &self.modified_locs
    }

    /// Check if function may modify unknown locations.
    pub fn may_modify_unknown(&self) -> bool {
        self.modifies_unknown
    }

    /// Create a bottom summary (most precise, used for recursive fixpoint initialization).
    pub fn bottom() -> Self {
        Self {
            return_interval: None,
            return_nullness: Some(Nullness::Bottom),
            param_count: 0,
            modified_locs: BTreeSet::new(),
            modifies_unknown: false,
            param_store_effects: BTreeMap::new(),
            global_store_effects: BTreeMap::new(),
        }
    }

    /// Join two summaries (least upper bound).
    ///
    /// Used during iterative fixpoint computation for recursive functions.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        let mut effects = self.param_store_effects.clone();
        for (&idx, other_iv) in &other.param_store_effects {
            effects
                .entry(idx)
                .and_modify(|e| *e = e.join(other_iv))
                .or_insert_with(|| other_iv.clone());
        }
        let mut global_effects = self.global_store_effects.clone();
        for (&loc, other_iv) in &other.global_store_effects {
            global_effects
                .entry(loc)
                .and_modify(|e| *e = e.join(other_iv))
                .or_insert_with(|| other_iv.clone());
        }
        Self {
            return_interval: match (&self.return_interval, &other.return_interval) {
                (Some(a), Some(b)) => Some(a.join(b)),
                (Some(a), None) | (None, Some(a)) => Some(a.clone()),
                (None, None) => None,
            },
            return_nullness: match (self.return_nullness, other.return_nullness) {
                (Some(a), Some(b)) => Some(a.join(b)),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            },
            param_count: self.param_count.max(other.param_count),
            modified_locs: self
                .modified_locs
                .union(&other.modified_locs)
                .copied()
                .collect(),
            modifies_unknown: self.modifies_unknown || other.modifies_unknown,
            param_store_effects: effects,
            global_store_effects: global_effects,
        }
    }

    /// Widen two summaries (for convergence guarantee).
    ///
    /// Used after threshold iterations to ensure termination.
    #[must_use]
    pub fn widen(&self, other: &Self) -> Self {
        let mut effects = self.param_store_effects.clone();
        for (&idx, other_iv) in &other.param_store_effects {
            effects
                .entry(idx)
                .and_modify(|e| *e = e.widen(other_iv))
                .or_insert_with(|| other_iv.clone());
        }
        let mut global_effects = self.global_store_effects.clone();
        for (&loc, other_iv) in &other.global_store_effects {
            global_effects
                .entry(loc)
                .and_modify(|e| *e = e.widen(other_iv))
                .or_insert_with(|| other_iv.clone());
        }
        Self {
            return_interval: match (&self.return_interval, &other.return_interval) {
                (Some(a), Some(b)) => Some(a.widen(b)),
                (Some(a), None) | (None, Some(a)) => Some(a.clone()),
                (None, None) => None,
            },
            // Nullness has finite height, join is sufficient
            return_nullness: match (self.return_nullness, other.return_nullness) {
                (Some(a), Some(b)) => Some(a.join(b)),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            },
            param_count: self.param_count.max(other.param_count),
            modified_locs: self
                .modified_locs
                .union(&other.modified_locs)
                .copied()
                .collect(),
            modifies_unknown: self.modifies_unknown || other.modifies_unknown,
            param_store_effects: effects,
            global_store_effects: global_effects,
        }
    }

    /// Widen two summaries using threshold-based widening for return interval.
    ///
    /// Like `widen`, but uses `widen_with_thresholds` for the return interval
    /// to prevent jumping to TOP when program constants provide natural bounds.
    #[must_use]
    pub fn widen_with_thresholds(&self, other: &Self, thresholds: &BTreeSet<i128>) -> Self {
        let mut effects = self.param_store_effects.clone();
        for (&idx, other_iv) in &other.param_store_effects {
            effects
                .entry(idx)
                .and_modify(|e| *e = e.widen_with_thresholds(other_iv, thresholds))
                .or_insert_with(|| other_iv.clone());
        }
        let mut global_effects = self.global_store_effects.clone();
        for (&loc, other_iv) in &other.global_store_effects {
            global_effects
                .entry(loc)
                .and_modify(|e| *e = e.widen_with_thresholds(other_iv, thresholds))
                .or_insert_with(|| other_iv.clone());
        }
        Self {
            return_interval: match (&self.return_interval, &other.return_interval) {
                (Some(a), Some(b)) => Some(a.widen_with_thresholds(b, thresholds)),
                (Some(a), None) | (None, Some(a)) => Some(a.clone()),
                (None, None) => None,
            },
            // Nullness has finite height, join is sufficient
            return_nullness: match (self.return_nullness, other.return_nullness) {
                (Some(a), Some(b)) => Some(a.join(b)),
                (Some(a), None) | (None, Some(a)) => Some(a),
                (None, None) => None,
            },
            param_count: self.param_count.max(other.param_count),
            modified_locs: self
                .modified_locs
                .union(&other.modified_locs)
                .copied()
                .collect(),
            modifies_unknown: self.modifies_unknown || other.modifies_unknown,
            param_store_effects: effects,
            global_store_effects: global_effects,
        }
    }

    /// Check if this summary equals another (for fixpoint detection).
    pub fn equals(&self, other: &Self) -> bool {
        self.return_interval == other.return_interval
            && self.return_nullness == other.return_nullness
            && self.param_count == other.param_count
            && self.modified_locs == other.modified_locs
            && self.modifies_unknown == other.modifies_unknown
            && self.param_store_effects == other.param_store_effects
            && self.global_store_effects == other.global_store_effects
    }
}

impl Default for FunctionSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl InterproceduralResult {
    /// Get the function summaries map.
    pub fn summaries(&self) -> &BTreeMap<FunctionId, FunctionSummary> {
        &self.summaries
    }
}

/// Result of interprocedural abstract interpretation.
#[derive(Debug)]
pub struct InterproceduralResult {
    /// Per-function summaries.
    summaries: BTreeMap<FunctionId, FunctionSummary>,
    /// The underlying intraprocedural result (extended with interprocedural info).
    intraprocedural: AbstractInterpResult,
    /// Instruction-level states with call result refinements.
    refined_inst_states: BTreeMap<InstId, AbstractState>,
}

impl InterproceduralResult {
    /// Get the summary for a function.
    pub fn function_summary(&self, func_id: &FunctionId) -> Option<&FunctionSummary> {
        self.summaries.get(func_id)
    }

    /// Get the abstract state at an instruction (with interprocedural refinements).
    pub fn state_at_inst(&self, inst_id: InstId) -> Option<&AbstractState> {
        self.refined_inst_states
            .get(&inst_id)
            .or_else(|| self.intraprocedural.state_at_inst(inst_id))
    }

    /// Get the underlying intraprocedural result.
    pub fn intraprocedural(&self) -> &AbstractInterpResult {
        &self.intraprocedural
    }

    /// Get diagnostics from the underlying intraprocedural analysis.
    pub fn diagnostics(&self) -> AbstractInterpDiagnostics {
        self.intraprocedural.diagnostics()
    }

    /// Get the interval for a value at an instruction, using refined states.
    ///
    /// This method checks refined call site states first (which contain
    /// interprocedural return value intervals), then falls back to the
    /// intraprocedural result.
    ///
    /// NOTE: Current limitation - call return value intervals are stored at
    /// the call instruction, but in -O0 code the return value typically goes
    /// through memory (alloca + store + load). The refined interval is lost
    /// when the value is stored and a different ValueId is loaded.
    /// A proper fix requires re-running intraprocedural analysis with summaries
    /// or tracking values through memory operations.
    #[must_use]
    pub fn interval_at_inst(&self, inst: InstId, value: ValueId, bits: u8) -> Interval {
        // Check refined states first (call site refinements)
        if let Some(state) = self.refined_inst_states.get(&inst) {
            if let Some(interval) = state.get_opt(value) {
                return interval.clone();
            }
        }
        // Fall back to intraprocedural result
        self.intraprocedural.interval_at_inst(inst, value, bits)
    }

    /// Access the constant map from the underlying intraprocedural result.
    #[must_use]
    pub fn constant_map(&self) -> &BTreeMap<ValueId, Interval> {
        self.intraprocedural.constant_map()
    }
}

// ---------------------------------------------------------------------------
// Spec-based summaries for external functions
// ---------------------------------------------------------------------------

/// Default bit width for intervals from specs (64-bit).
const SPEC_INTERVAL_BITS: u8 = 64;

/// Create a function summary from a spec's return fields.
///
/// Returns `None` if no spec exists or if the spec has neither interval nor nullness.
#[must_use]
pub fn summary_from_spec(name: &str, specs: &SpecRegistry) -> Option<FunctionSummary> {
    let spec = specs.lookup(name)?;
    let returns = spec.returns.as_ref()?;

    // Get interval if specified
    let return_interval = returns
        .interval
        .map(|(min, max)| Interval::new(i128::from(min), i128::from(max), SPEC_INTERVAL_BITS));

    // Get nullness if specified
    let return_nullness = returns.nullness.as_ref().and_then(|n| {
        match n {
            saf_core::spec::Nullness::NotNull => Some(Nullness::NotNull),
            saf_core::spec::Nullness::MaybeNull => Some(Nullness::MaybeNull),
            // RequiredNonnull and Nullable are for parameters, not returns
            _ => None,
        }
    });

    // Only return a summary if we have something useful
    if return_interval.is_none() && return_nullness.is_none() {
        return None;
    }

    Some(FunctionSummary {
        return_interval,
        return_nullness,
        param_count: spec.params.len(),
        modified_locs: BTreeSet::new(),
        modifies_unknown: false,
        param_store_effects: BTreeMap::new(),
        global_store_effects: BTreeMap::new(),
    })
}

// ---------------------------------------------------------------------------
// Unified Interprocedural Analysis Entry Point
// ---------------------------------------------------------------------------

/// Solve abstract interpretation with interprocedural analysis using a context.
///
/// This is the unified entry point that dispatches to the appropriate analysis
/// strategy based on the provided context:
///
/// - **No PTA, no specs**: Single-pass intraprocedural analysis with summary
///   extraction and call site refinement (baseline).
/// - **Specs only**: Same as baseline but with spec-based summaries for
///   external functions.
/// - **PTA only**: Adds mod/ref analysis and PTA-based indirect call resolution.
/// - **PTA + specs**: Full two-pass analysis with bottom-up SCC processing,
///   summary propagation through memory operations, and spec-based external
///   function handling.
///
/// # Arguments
///
/// * `module` - The AIR module to analyze
/// * `config` - Analysis configuration
/// * `ctx` - Interprocedural context with optional PTA and specs
#[must_use]
pub fn solve_interprocedural_with_context(
    module: &AirModule,
    config: &AbstractInterpConfig,
    ctx: &InterproceduralContext<'_>,
) -> InterproceduralResult {
    match (ctx.pta, ctx.specs) {
        // Full two-pass analysis with PTA and specs
        (Some(pta), specs) => solve_with_pta_impl(module, config, pta, specs),
        // Specs only: single-pass with spec-based external summaries
        (None, Some(specs)) => solve_with_specs_impl(module, config, specs),
        // Baseline: single-pass analysis without PTA or specs
        (None, None) => solve_baseline_impl(module, config),
    }
}

/// Baseline interprocedural analysis implementation.
///
/// Single-pass intraprocedural analysis with summary extraction and
/// context-sensitive call site refinement.
fn solve_baseline_impl(module: &AirModule, config: &AbstractInterpConfig) -> InterproceduralResult {
    // Phase 1: Compute function summaries via intraprocedural analysis
    let intraprocedural = super::solve_abstract_interp(module, config);
    let constant_map = build_constant_map(module);

    // Build call graph and find recursive SCCs
    let call_graph = build_call_graph(module);
    let recursive_sccs = find_recursive_sccs(module, &call_graph);

    // Collect functions in recursive SCCs
    let recursive_funcs: BTreeSet<FunctionId> = recursive_sccs
        .iter()
        .flat_map(|scc| scc.iter())
        .copied()
        .collect();

    // Build function ID → function mapping for callee lookup
    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, f))
        .collect();

    // Compute summaries for non-recursive functions first
    let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration || recursive_funcs.contains(&func.id) {
            continue;
        }

        let summary = compute_function_summary(func, module, &intraprocedural, &constant_map);
        summaries.insert(func.id, summary);
    }

    // Phase 1b: Compute summaries for recursive SCCs using iterative fixpoint
    let thresholds = extract_thresholds(module);
    for scc in &recursive_sccs {
        let scc_summaries = compute_recursive_scc_summaries(
            scc,
            module,
            &intraprocedural,
            &constant_map,
            &summaries,
            &thresholds,
        );
        summaries.extend(scc_summaries);
    }

    // Phase 2: Refine call sites using context-sensitive analysis
    let refined_inst_states = refine_call_sites(
        module,
        &intraprocedural,
        &summaries,
        &func_map,
        &constant_map,
        config,
    );

    InterproceduralResult {
        summaries,
        intraprocedural,
        refined_inst_states,
    }
}

/// Interprocedural analysis with spec-based external summaries.
///
/// Single-pass analysis that uses spec registry for external function summaries.
fn solve_with_specs_impl(
    module: &AirModule,
    config: &AbstractInterpConfig,
    specs: &SpecRegistry,
) -> InterproceduralResult {
    // Phase 1: Compute function summaries via intraprocedural analysis
    let intraprocedural = super::solve_abstract_interp(module, config);
    let constant_map = build_constant_map(module);

    // Build call graph and find recursive SCCs
    let call_graph = build_call_graph(module);
    let recursive_sccs = find_recursive_sccs(module, &call_graph);
    let recursive_funcs: BTreeSet<FunctionId> = recursive_sccs
        .iter()
        .flat_map(|scc| scc.iter())
        .copied()
        .collect();

    // Build function ID → function mapping for callee lookup
    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, f))
        .collect();

    // Compute summaries for external and non-recursive functions first
    let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            // Try to get summary from spec for external functions
            if let Some(summary) = summary_from_spec(&func.name, specs) {
                summaries.insert(func.id, summary);
            }
            continue;
        }

        if recursive_funcs.contains(&func.id) {
            continue;
        }

        let summary = compute_function_summary(func, module, &intraprocedural, &constant_map);
        summaries.insert(func.id, summary);
    }

    // Compute summaries for recursive SCCs using iterative fixpoint
    let thresholds = extract_thresholds(module);
    for scc in &recursive_sccs {
        let scc_summaries = compute_recursive_scc_summaries(
            scc,
            module,
            &intraprocedural,
            &constant_map,
            &summaries,
            &thresholds,
        );
        summaries.extend(scc_summaries);
    }

    // Phase 2: Refine call sites using context-sensitive analysis
    let refined_inst_states = refine_call_sites(
        module,
        &intraprocedural,
        &summaries,
        &func_map,
        &constant_map,
        config,
    );

    InterproceduralResult {
        summaries,
        intraprocedural,
        refined_inst_states,
    }
}

/// Full interprocedural analysis with PTA and optional specs.
///
/// Two-pass analysis with bottom-up SCC processing, mod/ref summaries,
/// and summary propagation through memory operations.
// NOTE: This function implements the two-pass interprocedural analysis as a
// single cohesive unit. Splitting would obscure the analysis pipeline.
#[allow(clippy::too_many_lines)]
fn solve_with_pta_impl(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaResult,
    specs: Option<&SpecRegistry>,
) -> InterproceduralResult {
    // Phase 1: Compute mod/ref summaries (using specs for external functions)
    let mod_ref_summaries = compute_all_mod_ref_with_specs(module, pta, specs);

    let constant_map = build_constant_map(module);

    // Build call graph
    let call_graph = build_call_graph(module);

    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, f))
        .collect();

    let thresholds = extract_thresholds(module);

    // Phase 2b: Compute summaries in BOTTOM-UP order (callees before callers)
    //
    // This is crucial for correct interprocedural analysis. We process SCCs
    // in reverse topological order so that when we analyze a function, all
    // its callees already have summaries computed.
    let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

    // First, add spec-based summaries for external functions
    for func in &module.functions {
        if func.is_declaration {
            if let Some(registry) = specs {
                if let Some(mut summary) = summary_from_spec(&func.name, registry) {
                    if let Some(mod_ref) = mod_ref_summaries.get(&func.id) {
                        summary.modified_locs.clone_from(&mod_ref.modified_locs);
                        summary.modifies_unknown = mod_ref.modifies_unknown;
                    }
                    summaries.insert(func.id, summary);
                }
            }
        }
    }

    // Get ALL SCCs (not just recursive ones) in reverse topological order
    let all_sccs = compute_sccs_bottom_up(module, &call_graph);

    let pta_integration = PtaIntegration::new(pta);

    // Process each SCC in bottom-up order
    for scc in &all_sccs {
        if scc.len() == 1 {
            let func_id = *scc.iter().next().expect("SCC is non-empty");

            // Skip external functions (already handled above)
            let Some(func) = func_map.get(&func_id) else {
                continue;
            };

            // Check if this is a self-recursive function
            let is_self_recursive = call_graph
                .get(&func_id)
                .is_some_and(|callees| callees.contains(&func_id));

            if is_self_recursive {
                // Collect argument intervals from external callers
                let return_intervals = extract_return_intervals(&summaries);
                let ext_args = collect_external_call_args(
                    module,
                    scc,
                    &constant_map,
                    &pta_integration,
                    &return_intervals,
                    config,
                );

                // Handle self-recursive function with iterative fixpoint
                let scc_summaries = compute_recursive_scc_summaries_with_pta(
                    scc,
                    module,
                    config,
                    &constant_map,
                    &summaries,
                    &pta_integration,
                    &ext_args,
                    &thresholds,
                );
                for (fid, mut summary) in scc_summaries {
                    if let Some(mod_ref) = mod_ref_summaries.get(&fid) {
                        summary.modified_locs.clone_from(&mod_ref.modified_locs);
                        summary.modifies_unknown = mod_ref.modifies_unknown;
                    }
                    summaries.insert(fid, summary);
                }
            } else {
                // Non-recursive function: analyze with existing summaries
                let summary = analyze_non_recursive_function(
                    func,
                    func_id,
                    config,
                    &constant_map,
                    module,
                    &pta_integration,
                    &summaries,
                    &mod_ref_summaries,
                    specs,
                );
                summaries.insert(func_id, summary);
            }
        } else {
            // Collect argument intervals from external callers
            let return_intervals = extract_return_intervals(&summaries);
            let ext_args = collect_external_call_args(
                module,
                scc,
                &constant_map,
                &pta_integration,
                &return_intervals,
                config,
            );

            // Mutually recursive SCC: use iterative fixpoint
            let scc_summaries = compute_recursive_scc_summaries_with_pta(
                scc,
                module,
                config,
                &constant_map,
                &summaries,
                &pta_integration,
                &ext_args,
                &thresholds,
            );
            for (func_id, mut summary) in scc_summaries {
                if let Some(mod_ref) = mod_ref_summaries.get(&func_id) {
                    summary.modified_locs.clone_from(&mod_ref.modified_locs);
                    summary.modifies_unknown = mod_ref.modifies_unknown;
                }
                summaries.insert(func_id, summary);
            }
        }
    }

    // Phase 3: SECOND PASS - Re-analyze with summaries integrated into transfer function
    // This is the key improvement: calls now use summary return intervals during
    // the intraprocedural analysis, so store/load preserves the refined values
    let return_intervals = extract_return_intervals(&summaries);
    let memory_summaries = extract_memory_summaries(&summaries);
    let global_summaries = extract_global_summaries(&summaries);
    let refined_intraprocedural = solve_abstract_interp_with_pta_and_summaries(
        module,
        config,
        &pta_integration,
        &return_intervals,
        Some(&memory_summaries),
        Some(&global_summaries),
    );

    // Phase 4: Additional call site refinement for context-sensitive precision
    // This provides extra precision for call sites where we can re-analyze the callee
    // with specific argument bindings
    let refined_inst_states = refine_call_sites_with_pta(
        module,
        &refined_intraprocedural,
        &summaries,
        &func_map,
        &constant_map,
        config,
        &pta_integration,
    );

    InterproceduralResult {
        summaries,
        intraprocedural: refined_intraprocedural,
        refined_inst_states,
    }
}

/// Solve abstract interpretation with interprocedural function summaries.
///
/// This performs a multi-phase analysis:
/// 1. Bottom-up: Compute function summaries (return value intervals and nullness)
/// 2. Recursive SCC fixpoint: Handle mutually recursive functions via iteration
/// 3. Top-down: Apply summaries at call sites to refine call results
///
/// This is a thin wrapper around [`solve_interprocedural_with_context`] for
/// backward compatibility.
#[must_use]
pub fn solve_interprocedural(
    module: &AirModule,
    config: &AbstractInterpConfig,
) -> InterproceduralResult {
    solve_interprocedural_with_context(module, config, &InterproceduralContext::new())
}

/// Solve abstract interpretation with interprocedural function summaries and specs.
///
/// Extends `solve_interprocedural` with spec-based summaries for external functions.
/// When a spec defines `returns.interval`, the summary will use that interval
/// instead of returning Top for calls to that function.
///
/// This is a thin wrapper around [`solve_interprocedural_with_context`] for
/// backward compatibility.
#[must_use]
pub fn solve_interprocedural_with_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    specs: Option<&SpecRegistry>,
) -> InterproceduralResult {
    let ctx = match specs {
        Some(s) => InterproceduralContext::with_specs(s),
        None => InterproceduralContext::new(),
    };
    solve_interprocedural_with_context(module, config, &ctx)
}

/// Solve abstract interpretation with interprocedural analysis and PTA.
///
/// Extends `solve_interprocedural` with:
/// - Indirect call resolution via PTA
/// - Mod/ref-based selective memory invalidation
/// - Function summaries including side effects
/// - Recursive SCC fixpoint computation
///
/// This is a thin wrapper around [`solve_interprocedural_with_context`] for
/// backward compatibility.
#[must_use]
pub fn solve_interprocedural_with_pta(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaResult,
) -> InterproceduralResult {
    solve_interprocedural_with_context(module, config, &InterproceduralContext::with_pta(pta))
}

/// Solve abstract interpretation with interprocedural analysis, PTA, and specs.
///
/// This implements a **two-pass interprocedural analysis** that correctly propagates
/// function return values through memory operations (store/load sequences).
///
/// ## The Problem (solved by this function)
///
/// In -O0 code, call results typically go through memory before use:
/// ```llvm
/// %10 = call i32 @a()       ; call returns 10
/// store i32 %10, ptr %4     ; store to alloca
/// %12 = load i32, ptr %4    ; load produces new ValueId
/// %13 = icmp sge i32 %12, 5 ; comparison uses %12, not %10
/// ```
///
/// A naive approach that only refines call sites in post-processing fails because
/// the store/load sequence already happened with TOP intervals.
///
/// ## Two-Pass Solution
///
/// 1. **Pass 1**: Initial intraprocedural analysis (calls return TOP) to compute summaries
/// 2. **Pass 2**: Re-analyze with summaries integrated into the transfer function,
///    so `CallDirect` uses summary return intervals and store/load propagates correctly
///
/// ## Arguments
///
/// * `module` - The AIR module to analyze
/// * `config` - Analysis configuration
/// * `pta` - PTA result for alias-aware memory tracking
/// * `specs` - Optional spec registry for external function summaries
///
/// This is a thin wrapper around [`solve_interprocedural_with_context`] for
/// backward compatibility.
#[must_use]
pub fn solve_interprocedural_with_pta_and_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaResult,
    specs: Option<&SpecRegistry>,
) -> InterproceduralResult {
    let ctx = match specs {
        Some(s) => InterproceduralContext::with_pta_and_specs(pta, s),
        None => InterproceduralContext::with_pta(pta),
    };
    solve_interprocedural_with_context(module, config, &ctx)
}

/// Analyze a single non-recursive function and extract its summary.
///
/// Uses existing summaries for callee return intervals and mod/ref information.
// INVARIANT: Each parameter maps to a distinct analysis concern (function, config,
// constants, module, PTA, summaries, mod/ref, specs) — grouping would not reduce conceptual complexity.
#[allow(clippy::too_many_arguments)]
fn analyze_non_recursive_function(
    func: &saf_core::air::AirFunction,
    func_id: FunctionId,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
    mod_ref_summaries: &BTreeMap<FunctionId, crate::pta::ModRefSummary>,
    specs: Option<&SpecRegistry>,
) -> FunctionSummary {
    let return_intervals = extract_return_intervals(summaries);

    saf_log!(absint::interproc, context, "analyzing function"; func=func.name.as_str(), func_id=func_id, summaries=return_intervals.len());

    let mem_summ = extract_memory_summaries(summaries);
    let glob_summ = extract_global_summaries(summaries);
    let func_result = solve_single_function_with_pta_and_summaries(
        func,
        config,
        constant_map,
        module,
        pta,
        &return_intervals,
        None,
        Some(&mem_summ),
        Some(&glob_summ),
        specs,
    );
    let mut summary =
        extract_summary_from_result(func, &func_result, constant_map, module, Some(pta));
    if let Some(mod_ref) = mod_ref_summaries.get(&func_id) {
        summary.modified_locs.clone_from(&mod_ref.modified_locs);
        summary.modifies_unknown = mod_ref.modifies_unknown;
    }

    saf_log!(absint::interproc, result, "computed summary"; func=func.name.as_str(), return_interval=format!("{:?}", summary.return_interval));

    summary
}

/// Extract return intervals from function summaries for use in the transfer function.
fn extract_return_intervals(
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> BTreeMap<FunctionId, Interval> {
    summaries
        .iter()
        .filter_map(|(id, summary)| {
            summary
                .return_interval
                .as_ref()
                .map(|int| (*id, int.clone()))
        })
        .collect()
}

/// Extract memory side-effect summaries for use in the transfer function.
///
/// Returns a map from function ID to (param index → interval stored to `*param[i]`).
fn extract_memory_summaries(
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> BTreeMap<FunctionId, BTreeMap<usize, Interval>> {
    summaries
        .iter()
        .filter(|(_, s)| !s.param_store_effects.is_empty())
        .map(|(id, s)| (*id, s.param_store_effects.clone()))
        .collect()
}

/// Extract global store-effect summaries for use in the transfer function.
///
/// Returns a map from function ID to (`LocId` → interval stored to that global).
fn extract_global_summaries(
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> BTreeMap<FunctionId, BTreeMap<LocId, Interval>> {
    summaries
        .iter()
        .filter(|(_, s)| !s.global_store_effects.is_empty())
        .map(|(id, s)| (*id, s.global_store_effects.clone()))
        .collect()
}

/// Compute the summary for a single function.
fn compute_function_summary(
    func: &saf_core::air::AirFunction,
    module: &AirModule,
    result: &AbstractInterpResult,
    constant_map: &BTreeMap<ValueId, Interval>,
) -> FunctionSummary {
    let mut summary = FunctionSummary::new();
    summary.param_count = func.params.len();

    // Find all return statements and join their return value intervals
    let mut return_intervals: Vec<Interval> = Vec::new();
    // Also compute return nullness by analyzing return values
    let mut return_nullness_values: Vec<Nullness> = Vec::new();

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::Ret = &inst.op {
                // Ret operand[0] is the return value (if present)
                if let Some(&return_operand) = inst.operands.first() {
                    // Get the interval for the return value
                    let state = result.state_at_inst(inst.id);
                    let interval = state
                        .and_then(|s| s.get_opt(return_operand).cloned())
                        .or_else(|| constant_map.get(&return_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));

                    return_intervals.push(interval);

                    // Compute return nullness by examining the return value
                    let nullness = compute_return_nullness(return_operand, func, module);
                    return_nullness_values.push(nullness);
                }
            }
        }
    }

    // Join all return intervals
    if !return_intervals.is_empty() {
        let mut joined = return_intervals[0].clone();
        for interval in return_intervals.iter().skip(1) {
            joined = joined.join(interval);
        }
        summary.return_interval = Some(joined);
    }

    // Join all return nullness values
    if !return_nullness_values.is_empty() {
        let mut joined = return_nullness_values[0];
        for nullness in return_nullness_values.iter().skip(1) {
            joined = joined.join(*nullness);
        }
        summary.return_nullness = Some(joined);
    }

    summary
}

/// Compute nullness for a return value by tracing back through the function.
///
/// This is a simplified flow-insensitive analysis that examines the instruction
/// that defines the return value to determine its nullness.
fn compute_return_nullness(
    return_value: ValueId,
    func: &saf_core::air::AirFunction,
    module: &AirModule,
) -> Nullness {
    let mut visited = BTreeSet::new();
    compute_return_nullness_inner(return_value, func, module, &mut visited)
}

fn compute_return_nullness_inner(
    return_value: ValueId,
    func: &saf_core::air::AirFunction,
    module: &AirModule,
    visited: &mut BTreeSet<ValueId>,
) -> Nullness {
    // Cycle detection: if we've already visited this value, return MaybeNull
    if !visited.insert(return_value) {
        return Nullness::MaybeNull;
    }

    // Check if it's a constant
    if let Some(constant) = module.constants.get(&return_value) {
        return match constant {
            Constant::Null | Constant::Int { value: 0, .. } => Nullness::Null,
            _ => Nullness::MaybeNull,
        };
    }

    // Find the instruction that defines this value
    for block in &func.blocks {
        for inst in &block.instructions {
            if inst.dst == Some(return_value) {
                return match &inst.op {
                    // Allocations are always non-null
                    Operation::HeapAlloc { .. }
                    | Operation::Alloca { .. }
                    | Operation::Global { .. } => Nullness::NotNull,

                    // GEP on non-null base, Copy/Cast propagate nullness
                    Operation::Gep { .. } | Operation::Copy | Operation::Cast { .. } => {
                        if let Some(&src) = inst.operands.first() {
                            compute_return_nullness_inner(src, func, module, visited)
                        } else {
                            Nullness::MaybeNull
                        }
                    }

                    // Phi - join all incoming values
                    Operation::Phi { incoming } => {
                        let mut result = Nullness::Bottom;
                        for (_, value) in incoming {
                            let incoming_nullness =
                                compute_return_nullness_inner(*value, func, module, visited);
                            result = result.join(incoming_nullness);
                        }
                        result
                    }

                    // Select - join both branches
                    Operation::Select => {
                        if inst.operands.len() >= 3 {
                            let true_nullness = compute_return_nullness_inner(
                                inst.operands[1],
                                func,
                                module,
                                visited,
                            );
                            let false_nullness = compute_return_nullness_inner(
                                inst.operands[2],
                                func,
                                module,
                                visited,
                            );
                            true_nullness.join(false_nullness)
                        } else {
                            Nullness::MaybeNull
                        }
                    }

                    _ => Nullness::MaybeNull,
                };
            }
        }
    }

    // If value is a parameter, it's MaybeNull
    for param in &func.params {
        if param.id == return_value {
            return Nullness::MaybeNull;
        }
    }

    Nullness::MaybeNull
}

// ---------------------------------------------------------------------------
// Recursive Function Summary Fixpoint
// ---------------------------------------------------------------------------

/// Maximum iterations for recursive summary fixpoint.
const MAX_RECURSIVE_ITERATIONS: usize = 15;

/// Widening threshold - after this many iterations, apply widening.
/// Set to 6 to allow recursive patterns (identity, accumulation, mc91)
/// enough join iterations to stabilize before widening jumps to TOP.
const WIDENING_THRESHOLD: usize = 6;

/// Build a call graph representation for SCC computation.
///
/// Returns a map from function ID to the set of functions it calls.
fn build_call_graph(module: &AirModule) -> BTreeMap<FunctionId, BTreeSet<FunctionId>> {
    let mut call_graph: BTreeMap<FunctionId, BTreeSet<FunctionId>> = BTreeMap::new();

    // Initialize all functions
    for func in &module.functions {
        call_graph.entry(func.id).or_default();
    }

    // Add edges for call sites
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        let caller_id = func.id;
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    call_graph.entry(caller_id).or_default().insert(*callee);
                }
                // For indirect calls, we can't statically determine targets
                // They'll be handled conservatively
            }
        }
    }

    call_graph
}

/// Wrapper for call graph that implements Successors trait.
struct CallGraphSuccessors<'a>(&'a BTreeMap<FunctionId, BTreeSet<FunctionId>>);

impl Successors<FunctionId> for CallGraphSuccessors<'_> {
    fn successors(&self, node: &FunctionId) -> Option<&BTreeSet<FunctionId>> {
        self.0.get(node)
    }
}

/// Compute all SCCs in bottom-up order (callees before callers).
///
/// Uses Tarjan's algorithm which naturally produces SCCs in reverse topological
/// order - SCCs with no outgoing edges to unvisited nodes are output first.
/// This means leaf functions (no callees) are processed before their callers.
fn compute_sccs_bottom_up(
    module: &AirModule,
    call_graph: &BTreeMap<FunctionId, BTreeSet<FunctionId>>,
) -> Vec<BTreeSet<FunctionId>> {
    // Get all defined function IDs
    let all_funcs: BTreeSet<FunctionId> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| f.id)
        .collect();

    // Tarjan's SCC outputs in reverse topological order by default
    tarjan_scc(&all_funcs, &CallGraphSuccessors(call_graph))
}

/// Find recursive SCCs in the call graph.
///
/// Returns SCCs that need iterative fixpoint computation:
/// - SCCs with more than one function (mutually recursive)
/// - Single-function SCCs where the function calls itself (self-recursive)
fn find_recursive_sccs(
    module: &AirModule,
    call_graph: &BTreeMap<FunctionId, BTreeSet<FunctionId>>,
) -> Vec<BTreeSet<FunctionId>> {
    // Get all function IDs
    let all_funcs: BTreeSet<FunctionId> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| f.id)
        .collect();

    // Compute SCCs
    let sccs = tarjan_scc(&all_funcs, &CallGraphSuccessors(call_graph));

    // Filter to recursive SCCs
    sccs.into_iter()
        .filter(|scc| {
            if scc.len() > 1 {
                // Mutually recursive
                true
            } else {
                // Check for self-recursion
                let func_id = scc.iter().next().expect("SCC is non-empty");
                call_graph
                    .get(func_id)
                    .is_some_and(|callees| callees.contains(func_id))
            }
        })
        .collect()
}

/// Compute summaries for a recursive SCC using iterative fixpoint.
///
/// Starts with bottom summaries and iterates until convergence,
/// applying widening after a threshold to ensure termination.
fn compute_recursive_scc_summaries(
    scc: &BTreeSet<FunctionId>,
    module: &AirModule,
    intraprocedural: &AbstractInterpResult,
    constant_map: &BTreeMap<ValueId, Interval>,
    existing_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    thresholds: &BTreeSet<i128>,
) -> BTreeMap<FunctionId, FunctionSummary> {
    let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

    // Initialize with bottom return interval summaries.
    // Using bottom (not None/TOP) ensures recursive calls return "no value yet"
    // rather than "any value possible", allowing base cases to seed the fixpoint.
    for &func_id in scc {
        let mut summary = FunctionSummary::bottom();
        summary.return_interval = Some(Interval::make_bottom(DEFAULT_BITS));
        summaries.insert(func_id, summary);
    }

    // Get function references
    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> = module
        .functions
        .iter()
        .filter(|f| scc.contains(&f.id))
        .map(|f| (f.id, f))
        .collect();

    // Iterate until fixpoint
    for iteration in 0..MAX_RECURSIVE_ITERATIONS {
        let mut changed = false;

        for &func_id in scc {
            let Some(func) = func_map.get(&func_id) else {
                continue;
            };

            // Compute new summary using current summaries for recursive calls
            let new_summary = compute_function_summary_with_callees(
                func,
                module,
                intraprocedural,
                constant_map,
                &summaries,
                existing_summaries,
            );

            // Get current summary
            let current = summaries.get(&func_id).cloned().unwrap_or_default();

            // Apply join or threshold-based widening
            let updated = if iteration >= WIDENING_THRESHOLD {
                if thresholds.is_empty() {
                    current.widen(&new_summary)
                } else {
                    current.widen_with_thresholds(&new_summary, thresholds)
                }
            } else {
                current.join(&new_summary)
            };

            if !updated.equals(&current) {
                summaries.insert(func_id, updated);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    summaries
}

/// Collect argument intervals from external callers (outside the SCC) for SCC functions.
///
/// Scans all non-SCC functions for `CallDirect` instructions to SCC members,
/// extracts operand intervals from constants, and joins them per parameter position.
/// This provides concrete parameter bounds for recursive summary computation.
fn collect_external_call_args(
    module: &AirModule,
    scc: &BTreeSet<FunctionId>,
    constant_map: &BTreeMap<ValueId, Interval>,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
    config: &AbstractInterpConfig,
) -> BTreeMap<FunctionId, Vec<Interval>> {
    let mut result: BTreeMap<FunctionId, Vec<Interval>> = BTreeMap::new();

    // Build a quick lookup of SCC function param counts
    let scc_param_counts: BTreeMap<FunctionId, usize> = module
        .functions
        .iter()
        .filter(|f| scc.contains(&f.id) && !f.is_declaration)
        .map(|f| (f.id, f.params.len()))
        .collect();

    // Scan all non-SCC defined functions for calls to SCC members
    for func in &module.functions {
        if func.is_declaration || scc.contains(&func.id) {
            continue;
        }

        // Run a quick analysis of this caller to get argument intervals
        let caller_result = solve_single_function_with_pta_and_summaries(
            func,
            config,
            constant_map,
            module,
            pta,
            return_intervals,
            None,
            None,
            None,
            None, // specs not needed for arg collection
        );

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if !scc.contains(callee) {
                        continue;
                    }
                    let Some(&param_count) = scc_param_counts.get(callee) else {
                        continue;
                    };

                    // Get the state at this call instruction
                    let call_state = caller_result.inst_states.get(&inst.id);

                    // Extract argument intervals
                    let mut arg_intervals = Vec::with_capacity(param_count);
                    for (i, &arg) in inst.operands.iter().enumerate() {
                        if i >= param_count {
                            break;
                        }
                        let from_state = call_state.and_then(|s| s.get_opt(arg).cloned());
                        let from_const = constant_map.get(&arg).cloned();
                        let interval = from_state
                            .clone()
                            .or_else(|| from_const.clone())
                            .unwrap_or_else(|| Interval::make_top(64));
                        saf_log!(absint::interproc, context, "caller callee arg"; caller=func.name.as_str(), arg_idx=i, vid=arg, from_state=format!("{:?}", from_state), from_const=format!("{:?}", from_const), interval=format!("{:?}", interval));
                        arg_intervals.push(interval);
                    }

                    // Pad with TOP if fewer operands than parameters
                    while arg_intervals.len() < param_count {
                        arg_intervals.push(Interval::make_top(64));
                    }

                    // Join with existing args for this callee
                    result
                        .entry(*callee)
                        .and_modify(|existing| {
                            for (i, arg) in arg_intervals.iter().enumerate() {
                                if i < existing.len() {
                                    existing[i] = existing[i].join(arg);
                                }
                            }
                        })
                        .or_insert(arg_intervals);
                }
            }
        }
    }

    result
}

/// Extract recursive call argument intervals from a single-function analysis result.
///
/// Scans the function's instructions for `CallDirect` to SCC members and extracts
/// argument intervals from the analysis state at each call site.
fn extract_recursive_call_args_from_result(
    func: &saf_core::air::AirFunction,
    scc: &BTreeSet<FunctionId>,
    result: &SingleFunctionResult,
    constant_map: &BTreeMap<ValueId, Interval>,
    scc_param_counts: &BTreeMap<FunctionId, usize>,
) -> BTreeMap<FunctionId, Vec<Interval>> {
    let mut collected: BTreeMap<FunctionId, Vec<Interval>> = BTreeMap::new();

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::CallDirect { callee } = &inst.op {
                if !scc.contains(callee) {
                    continue;
                }
                let Some(&param_count) = scc_param_counts.get(callee) else {
                    continue;
                };

                // Skip call sites on unreachable paths (no state recorded)
                let Some(call_state) = result.inst_states.get(&inst.id) else {
                    continue;
                };

                let mut arg_intervals = Vec::with_capacity(param_count);
                for (i, &arg) in inst.operands.iter().enumerate() {
                    if i >= param_count {
                        break;
                    }
                    let interval = call_state
                        .get_opt(arg)
                        .cloned()
                        .or_else(|| constant_map.get(&arg).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));
                    arg_intervals.push(interval);
                }

                while arg_intervals.len() < param_count {
                    arg_intervals.push(Interval::make_top(64));
                }

                collected
                    .entry(*callee)
                    .and_modify(|existing| {
                        for (i, arg) in arg_intervals.iter().enumerate() {
                            if i < existing.len() {
                                existing[i] = existing[i].join(arg);
                            }
                        }
                    })
                    .or_insert(arg_intervals);
            }
        }
    }

    collected
}

/// Compute summaries for a recursive SCC using iterative fixpoint with PTA.
///
/// This is the key fix for Plan 061: Instead of extracting summaries from a
/// fixed intraprocedural result (where calls return TOP), this function
/// RE-ANALYZES each function with current summaries integrated into the
/// transfer function. This allows summary return intervals to propagate
/// correctly through memory operations.
///
/// # Algorithm
///
/// 1. Initialize all functions in SCC with bottom summaries.
/// 2. For each iteration: combine external + current SCC summaries; for each
///    function in the SCC, re-analyze it using
///    `solve_abstract_interp_with_pta_and_summaries` (which uses summary
///    return intervals for calls) and extract a new summary from the fresh
///    result; apply join (or widening after threshold) to update summaries;
///    check for fixpoint.
/// 3. Return converged summaries.
// NOTE: This function implements iterative fixpoint summary computation for
// mutually-recursive SCCs. The initialization, per-function re-analysis,
// summary extraction, and convergence check are tightly coupled.
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::similar_names
)]
fn compute_recursive_scc_summaries_with_pta(
    scc: &BTreeSet<FunctionId>,
    module: &AirModule,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    existing_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    pta: &PtaIntegration<'_>,
    external_call_args: &BTreeMap<FunctionId, Vec<Interval>>,
    thresholds: &BTreeSet<i128>,
) -> BTreeMap<FunctionId, FunctionSummary> {
    let mut summaries: BTreeMap<FunctionId, FunctionSummary> = BTreeMap::new();

    // Initialize with bottom return interval summaries.
    // Using bottom (not None/TOP) ensures recursive calls return "no value yet"
    // rather than "any value possible", allowing base cases to seed the fixpoint.
    for &func_id in scc {
        let mut summary = FunctionSummary::bottom();
        summary.return_interval = Some(Interval::make_bottom(DEFAULT_BITS));
        summaries.insert(func_id, summary);
    }

    // Get function references and param counts
    let func_map: BTreeMap<FunctionId, &saf_core::air::AirFunction> = module
        .functions
        .iter()
        .filter(|f| scc.contains(&f.id))
        .map(|f| (f.id, f))
        .collect();

    let scc_param_counts: BTreeMap<FunctionId, usize> = func_map
        .iter()
        .map(|(&fid, f)| (fid, f.params.len()))
        .collect();

    // Track per-function parameter intervals (starts from external caller args)
    let mut param_intervals_map: BTreeMap<FunctionId, Vec<Interval>> = external_call_args.clone();

    // Iterate until fixpoint
    for iteration in 0..MAX_RECURSIVE_ITERATIONS {
        let mut changed = false;

        // Combine external + current SCC summaries
        let mut all_summaries = existing_summaries.clone();
        all_summaries.extend(summaries.clone());

        // Extract return intervals for the transfer function.
        // For SCC functions with no computed summary yet (return_interval = None),
        // insert bottom so recursive calls produce bottom (→ unreachable) rather
        // than falling through to TOP(DEFAULT_BITS).
        let mut return_intervals = extract_return_intervals(&all_summaries);
        for &func_id in scc {
            return_intervals
                .entry(func_id)
                .or_insert_with(Interval::bottom);
        }

        // Track analysis results for recursive arg extraction
        let mut iteration_results: BTreeMap<FunctionId, SingleFunctionResult> = BTreeMap::new();

        for &func_id in scc {
            let Some(func) = func_map.get(&func_id) else {
                continue;
            };

            // Use caller-argument-bound intervals if available
            let pis = param_intervals_map.get(&func_id);
            saf_log!(absint::interproc, convergence, "recursive iteration"; iter=iteration, func=func.name.as_str(), param_intervals=format!("{:?}", pis));

            let mem_summ = extract_memory_summaries(&all_summaries);
            let glob_summ = extract_global_summaries(&all_summaries);
            let func_result = solve_single_function_with_pta_and_summaries(
                func,
                config,
                constant_map,
                module,
                pta,
                &return_intervals,
                pis.map(Vec::as_slice),
                Some(&mem_summ),
                Some(&glob_summ),
                None, // specs not needed for SCC fixpoint (recursive functions are user-defined)
            );

            // Extract new summary from fresh analysis
            let new_summary =
                extract_summary_from_result(func, &func_result, constant_map, module, Some(pta));
            // Debug: check what ret instruction states look like
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Ret = &inst.op {
                        if let Some(&ret_op) = inst.operands.first() {
                            let st = func_result.inst_states.get(&inst.id);
                            let from_state = st.and_then(|s| s.get_opt(ret_op).cloned());
                            let from_const = constant_map.get(&ret_op).cloned();
                            saf_log!(absint::interproc, convergence, "ret operand"; iter=iteration, func=func.name.as_str(), ret_op=ret_op, from_state=format!("{:?}", from_state), from_const=format!("{:?}", from_const), has_state=st.is_some());
                        }
                    }
                }
            }
            saf_log!(absint::interproc, convergence, "new summary"; iter=iteration, func=func.name.as_str(), return_interval=format!("{:?}", new_summary.return_interval));

            // Get current summary
            let current = summaries.get(&func_id).cloned().unwrap_or_default();

            // Apply join or threshold-based widening
            let updated = if iteration >= WIDENING_THRESHOLD {
                if thresholds.is_empty() {
                    current.widen(&new_summary)
                } else {
                    current.widen_with_thresholds(&new_summary, thresholds)
                }
            } else {
                current.join(&new_summary)
            };

            if !updated.equals(&current) {
                summaries.insert(func_id, updated);
                changed = true;
            }

            iteration_results.insert(func_id, func_result);
        }

        // Update parameter intervals from recursive call arguments
        // Join recursive call args with external caller args for next iteration
        let mut params_changed = false;
        for (&func_id, func) in &func_map {
            if let Some(result) = iteration_results.get(&func_id) {
                let recursive_args = extract_recursive_call_args_from_result(
                    func,
                    scc,
                    result,
                    constant_map,
                    &scc_param_counts,
                );

                for (callee_id, rec_args) in recursive_args {
                    param_intervals_map
                        .entry(callee_id)
                        .and_modify(|existing| {
                            for (i, arg) in rec_args.iter().enumerate() {
                                if i < existing.len() {
                                    let joined = existing[i].join(arg);
                                    if joined != existing[i] {
                                        params_changed = true;
                                    }
                                    existing[i] = joined;
                                }
                            }
                        })
                        .or_insert_with(|| {
                            params_changed = true;
                            rec_args
                        });
                }
            }
        }

        // Continue if either summaries or parameter intervals changed
        if !changed && !params_changed {
            break;
        }
    }

    // Clear stale bottom return_intervals for void functions.
    //
    // The SCC fixpoint seeds every function with `Some(bottom)` to prevent
    // recursive calls from falling through to TOP. For void functions (Ret
    // without an operand), `extract_summary_from_result` produces
    // `return_interval = None`, but `join(None, Some(bottom)) = Some(bottom)`,
    // so the initial bottom persists.  If left in the summary, Phase 3
    // re-analysis treats calls to void functions as "returns bottom" →
    // unreachable, silently dropping all subsequent instructions.
    for (&func_id, summary) in &mut summaries {
        if let Some(ref iv) = summary.return_interval {
            if iv.is_bottom() {
                // Verify this is truly a void function (no return operand)
                let is_void = func_map.get(&func_id).is_some_and(|f| {
                    f.blocks.iter().all(|b| {
                        b.instructions.iter().all(|inst| {
                            !matches!(&inst.op, Operation::Ret) || inst.operands.is_empty()
                        })
                    })
                });
                if is_void {
                    summary.return_interval = None;
                }
            }
        }
    }

    summaries
}

/// Solve a single function with PTA and summaries for summary extraction.
///
/// This runs fixpoint analysis on a single function (not the whole module),
/// using the provided summaries for call return intervals.
// NOTE: This function implements single-function fixpoint analysis with PTA
// and summary integration as a cohesive unit. Splitting would obscure the
// analysis flow.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn solve_single_function_with_pta_and_summaries(
    func: &saf_core::air::AirFunction,
    config: &AbstractInterpConfig,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    return_intervals: &BTreeMap<FunctionId, Interval>,
    param_intervals: Option<&[Interval]>,
    memory_summaries: Option<&BTreeMap<FunctionId, BTreeMap<usize, Interval>>>,
    global_summaries: Option<&BTreeMap<FunctionId, BTreeMap<LocId, Interval>>>,
    _specs: Option<&SpecRegistry>,
) -> SingleFunctionResult {
    use std::collections::VecDeque;

    let cfg = Cfg::build(func);
    let cond_inst_map = build_cond_inst_map(func);
    let loop_headers = detect_loop_headers(&cfg);
    let thresholds = if config.use_threshold_widening {
        extract_thresholds(module)
    } else {
        BTreeSet::new()
    };

    // Track conditional refinements for persistence (Plan 084 Phase A).
    let mut block_refinements: BTreeMap<saf_core::ids::BlockId, BTreeMap<ValueId, Interval>> =
        BTreeMap::new();

    let mut block_entry_states: BTreeMap<saf_core::ids::BlockId, AbstractState> = BTreeMap::new();
    for block in &func.blocks {
        block_entry_states.insert(block.id, AbstractState::bottom());
    }

    // Entry state: parameters use caller-provided intervals if available, else TOP
    let mut entry_state = AbstractState::new();
    for (i, param) in func.params.iter().enumerate() {
        let interval = param_intervals
            .and_then(|pis| pis.get(i))
            .cloned()
            .unwrap_or_else(|| Interval::make_top(64));
        entry_state.set(param.id, interval);
    }

    // Seed loc_memory with constant values from global aggregate initializers
    // (Plan 084 Phase B). Pre-populates intervals for globals like
    // `int a[2] = {1, 2}` so that GEP+Load resolves to concrete values.
    super::transfer::seed_global_aggregate_constants(&mut entry_state, module, pta);

    block_entry_states.insert(cfg.entry, entry_state);

    // Worklist-based forward analysis
    let mut worklist: VecDeque<saf_core::ids::BlockId> = VecDeque::new();
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
            saf_log!(absint::interproc, filter, "block unreachable"; block=block_id);
            continue;
        }

        saf_log!(absint::interproc, delta, "processing block"; block=block_id, iter_count=iteration_count, loc_memory_count=block_entry.loc_memory_entries().len(), values_count=block_entry.entries().len());
        for (loc, iv) in block_entry.loc_memory_entries() {
            saf_log!(absint::interproc, delta, "loc memory entry"; loc=*loc, interval=format!("{:?}", iv));
        }

        // Compute reached blocks for phi predecessor filtering
        let mut reached = crate::pta::ptsset::IdBitSet::<saf_core::ids::BlockId>::empty();
        for (id, s) in &block_entry_states {
            if !s.is_unreachable() {
                reached.insert(*id);
            }
        }
        let transfer_ctx = TransferContext {
            pta: Some(pta),
            return_intervals: Some(return_intervals),
            reached_blocks: Some(&reached),
            memory_summaries,
            global_summaries,
            // NOTE: InterproceduralContext uses &SpecRegistry (not AnalyzedSpecRegistry).
            // Computed return bounds from AnalyzedSpecRegistry are NOT applied here.
            // This is acceptable because:
            // 1. Return intervals from YAML specs already flow through summary_from_spec()
            // 2. The buffer overflow checker uses the direct fixpoint path where
            //    AnalyzedSpecRegistry IS wired in
            // Future: wire AnalyzedSpecRegistry into interprocedural solver for
            // computed bounds in cross-function temporal analysis.
            specs: None,
            obj_type_map: None,
        };

        // Execute with summaries-aware transfer
        let mut current_state = block_entry.clone();
        for inst in &block.instructions {
            saf_log!(absint::interproc, delta, "transfer inst"; op=format!("{:?}", inst.op), loc_mem_count=current_state.loc_memory_entries().len());
            transfer_instruction_with_context(
                inst,
                &mut current_state,
                constant_map,
                module,
                &transfer_ctx,
            );
        }

        saf_log!(absint::interproc, delta, "after block"; block=block_id, loc_memory_count=current_state.loc_memory_entries().len());
        for (loc, iv) in current_state.loc_memory_entries() {
            saf_log!(absint::interproc, delta, "loc memory after"; loc=*loc, interval=format!("{:?}", iv));
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
            if !propagated_state.is_unreachable() {
                propagate_refinement_to_loc_memory(
                    &mut propagated_state,
                    block,
                    &current_state,
                    pta,
                );
                if let Some(succ_block) = func.blocks.iter().find(|b| b.id == *succ_id) {
                    propagate_refinement_to_loc_memory(
                        &mut propagated_state,
                        succ_block,
                        &current_state,
                        pta,
                    );
                }
            }

            // Record refinements for persistence (Plan 084 Phase A).
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
                                let should_persist = if other_refined.is_unreachable() {
                                    true
                                } else {
                                    let _ = other_target;
                                    let pred_count = cfg
                                        .predecessors_of(*succ_id)
                                        .map_or(0, std::collections::BTreeSet::len);
                                    pred_count <= 1
                                };
                                if should_persist {
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
                                }
                            }
                        }
                    }
                }
            }

            if let Some(term) = terminator {
                if let Operation::CondBr { then_target, .. } = &term.op {
                    let take_true = *succ_id == *then_target;
                    saf_log!(absint::interproc, delta, "condbr refine"; take_true=take_true, unreachable=propagated_state.is_unreachable());
                }
            }

            let old_state = block_entry_states.get(succ_id).cloned();
            let mut new_state = if let Some(old) = &old_state {
                if loop_headers.contains(*succ_id) {
                    widen_state(old, &propagated_state, &thresholds)
                } else {
                    old.join(&propagated_state)
                }
            } else {
                propagated_state.clone()
            };

            // Re-apply persisted refinements after join/widen (Plan 084 Phase A).
            if let Some(refinements) = block_refinements.get(succ_id) {
                apply_refinements(&mut new_state, refinements);
            }

            // Check convergence using leq: if the new state is contained in the
            // old state, the fixpoint has converged for this edge. This avoids
            // unnecessary iterations when refinements tighten the state within
            // the same lattice level.
            let converged = if let Some(old) = &old_state {
                new_state.leq(old) && !old.is_unreachable()
            } else {
                false
            };
            if !converged {
                block_entry_states.insert(*succ_id, new_state);
                if !worklist.contains(succ_id) {
                    worklist.push_back(*succ_id);
                }
            }
        }
    }

    // Narrowing phase — two-pass per iteration.
    //
    // Pass 1: process all blocks in RPO, run transfer functions, and
    // accumulate the join of all incoming propagated states per successor
    // (without including the successor's old state).
    //
    // Pass 2: for each block with accumulated incoming state, narrow the
    // old block state with the incoming join. This correctly handles
    // multi-predecessor blocks (e.g., loop headers) by joining ALL incoming
    // edges before narrowing, rather than joining with the old state
    // per-edge (which would prevent any tightening since old.join(sub) = old
    // when sub is already a subset of old after widening fixpoint).
    let rpo = reverse_postorder(&cfg);
    for _ in 0..config.narrowing_iterations {
        let mut narrowing_changed = false;

        // Pass 1: accumulate incoming propagated states per block.
        let mut incoming_states: BTreeMap<saf_core::ids::BlockId, AbstractState> = BTreeMap::new();

        for &block_id in &rpo {
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

            let mut narrow_reached =
                crate::pta::ptsset::IdBitSet::<saf_core::ids::BlockId>::empty();
            for (id, s) in &block_entry_states {
                if !s.is_unreachable() {
                    narrow_reached.insert(*id);
                }
            }
            let narrow_ctx = TransferContext {
                pta: Some(pta),
                return_intervals: Some(return_intervals),
                reached_blocks: Some(&narrow_reached),
                memory_summaries,
                global_summaries,
                // NOTE: Computed return bounds from AnalyzedSpecRegistry not available here.
                // See comment in widening phase above for rationale.
                specs: None,
                obj_type_map: None,
            };

            let mut current_state = block_entry.clone();
            for inst in &block.instructions {
                transfer_instruction_with_context(
                    inst,
                    &mut current_state,
                    constant_map,
                    module,
                    &narrow_ctx,
                );
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
                if !propagated_state.is_unreachable() {
                    propagate_refinement_to_loc_memory(
                        &mut propagated_state,
                        block,
                        &current_state,
                        pta,
                    );
                    if let Some(succ_block) = func.blocks.iter().find(|b| b.id == *succ_id) {
                        propagate_refinement_to_loc_memory(
                            &mut propagated_state,
                            succ_block,
                            &current_state,
                            pta,
                        );
                    }
                }

                // Accumulate: join all incoming edges for each successor.
                incoming_states
                    .entry(*succ_id)
                    .and_modify(|existing| *existing = existing.join(&propagated_state))
                    .or_insert(propagated_state);
            }
        }

        // Pass 2: narrow each block's state with the accumulated incoming.
        for (block_id, incoming) in &incoming_states {
            let old_state = block_entry_states
                .get(block_id)
                .cloned()
                .unwrap_or_else(AbstractState::bottom);

            let mut narrowed = narrow_state(&old_state, incoming, &thresholds);

            // Re-apply persisted refinements after narrowing (Plan 084 Phase A).
            if let Some(refinements) = block_refinements.get(block_id) {
                apply_refinements(&mut narrowed, refinements);
            }

            if narrowed != old_state {
                block_entry_states.insert(*block_id, narrowed);
                narrowing_changed = true;
            }
        }

        if !narrowing_changed {
            break;
        }
    }

    // Collect instruction states for summary extraction
    let mut inst_states: BTreeMap<saf_core::ids::InstId, AbstractState> = BTreeMap::new();
    let mut final_reached = crate::pta::ptsset::IdBitSet::<saf_core::ids::BlockId>::empty();
    for (id, s) in &block_entry_states {
        if !s.is_unreachable() {
            final_reached.insert(*id);
        }
    }
    let final_ctx = TransferContext {
        pta: Some(pta),
        return_intervals: Some(return_intervals),
        reached_blocks: Some(&final_reached),
        memory_summaries,
        global_summaries,
        // NOTE: Computed return bounds from AnalyzedSpecRegistry not available here.
        // See comment in widening phase above for rationale.
        specs: None,
        obj_type_map: None,
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
                constant_map,
                module,
                &final_ctx,
            );
        }
    }

    SingleFunctionResult { inst_states }
}

/// Result from analyzing a single function (for summary extraction).
struct SingleFunctionResult {
    inst_states: BTreeMap<saf_core::ids::InstId, AbstractState>,
}

/// Extract a function summary from single-function analysis result.
fn extract_summary_from_result(
    func: &saf_core::air::AirFunction,
    result: &SingleFunctionResult,
    constant_map: &BTreeMap<ValueId, Interval>,
    module: &AirModule,
    pta: Option<&PtaIntegration<'_>>,
) -> FunctionSummary {
    let mut summary = FunctionSummary::new();
    summary.param_count = func.params.len();

    // Find all return statements and extract intervals
    let mut return_intervals: Vec<Interval> = Vec::new();
    let mut return_nullness_values: Vec<Nullness> = Vec::new();

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::Ret = &inst.op {
                if let Some(&return_operand) = inst.operands.first() {
                    // Get the interval from the analysis result.
                    // Skip ret instructions on unreachable paths (no state
                    // recorded) — falling back to TOP would poison the
                    // summary with an imprecise bound.
                    let Some(state) = result.inst_states.get(&inst.id) else {
                        continue;
                    };
                    if state.is_unreachable() {
                        continue;
                    }

                    let interval = state
                        .get_opt(return_operand)
                        .cloned()
                        .or_else(|| constant_map.get(&return_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));

                    return_intervals.push(interval);

                    // Compute return nullness
                    let nullness = compute_return_nullness(return_operand, func, module);
                    return_nullness_values.push(nullness);
                }
            }
        }
    }

    // Join all return intervals
    if !return_intervals.is_empty() {
        let mut joined = return_intervals[0].clone();
        for interval in return_intervals.iter().skip(1) {
            joined = joined.join(interval);
        }
        summary.return_interval = Some(joined);
    }

    // Join all return nullness values
    if !return_nullness_values.is_empty() {
        let mut joined = return_nullness_values[0];
        for nullness in return_nullness_values.iter().skip(1) {
            joined = joined.join(*nullness);
        }
        summary.return_nullness = Some(joined);
    }

    // Extract param_store_effects: for each parameter, check if the function
    // stores a known interval to *param[i] (Plan 084 Phase C).
    if let Some(pta) = pta {
        for (param_idx, param) in func.params.iter().enumerate() {
            let param_pts = pta.points_to(param.id);
            if param_pts.is_empty() {
                continue;
            }
            let mut effect_intervals: Vec<Interval> = Vec::new();
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Ret = &inst.op {
                        if let Some(state) = result.inst_states.get(&inst.id) {
                            for loc in &param_pts {
                                if let Some(iv) = state.load_loc(*loc) {
                                    if !iv.is_top() {
                                        effect_intervals.push(iv.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !effect_intervals.is_empty() {
                let mut joined = effect_intervals[0].clone();
                for iv in effect_intervals.iter().skip(1) {
                    joined = joined.join(iv);
                }
                if !joined.is_top() {
                    summary.param_store_effects.insert(param_idx, joined);
                }
            }
        }
    }

    // Extract global store effects (Plan 086 Phase H4).
    if let Some(pta) = pta {
        extract_global_store_effects(func, result, module, pta, &mut summary);
    }

    summary
}

/// Extract global variable store effects from a function's analysis result.
///
/// For each global variable, checks `loc_memory` at block exit states to find
/// known intervals stored to the global.  Scans ALL block exits (not just
/// `Ret` states) because the regular `join()` drops one-sided `loc_memory`
/// entries — a global written on only one branch disappears at the merge point.
fn extract_global_store_effects(
    func: &saf_core::air::AirFunction,
    result: &SingleFunctionResult,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    summary: &mut FunctionSummary,
) {
    let Some(pta_ref) = pta.pta_ref() else {
        return;
    };
    // Collect LocIds for global objects (base path only)
    let global_locs: BTreeSet<LocId> = module
        .globals
        .iter()
        .flat_map(|g| {
            pta_ref
                .locations()
                .iter()
                .filter(move |(_, loc)| loc.obj == g.obj && loc.path.steps.is_empty())
                .map(|(id, _)| *id)
        })
        .collect();
    if global_locs.is_empty() {
        return;
    }
    // Scan block exit states (last instruction in each block) rather than only
    // Ret states.  The regular state `join()` drops one-sided `loc_memory`
    // entries, so a global written only on one branch disappears at the merge
    // point.  By collecting from each block's exit state we capture the value
    // before it is lost in the join.
    for block in &func.blocks {
        let Some(last_inst) = block.instructions.last() else {
            continue;
        };
        let Some(state) = result.inst_states.get(&last_inst.id) else {
            continue;
        };
        if state.is_unreachable() {
            continue;
        }
        for &loc in &global_locs {
            if let Some(iv) = state.load_loc(loc) {
                if !iv.is_top() {
                    summary
                        .global_store_effects
                        .entry(loc)
                        .and_modify(|e| *e = e.join(iv))
                        .or_insert_with(|| iv.clone());
                }
            }
        }
    }
}

/// Compute function summary considering callee summaries for recursive calls.
///
/// This is similar to `compute_function_summary` but uses the provided
/// summaries map for calls within the SCC, allowing iterative refinement.
fn compute_function_summary_with_callees(
    func: &saf_core::air::AirFunction,
    module: &AirModule,
    result: &AbstractInterpResult,
    constant_map: &BTreeMap<ValueId, Interval>,
    scc_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    external_summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> FunctionSummary {
    let mut summary = FunctionSummary::new();
    summary.param_count = func.params.len();

    // Find all return statements and join their return value intervals
    let mut return_intervals: Vec<Interval> = Vec::new();
    let mut return_nullness_values: Vec<Nullness> = Vec::new();

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::Ret = &inst.op {
                if let Some(&return_operand) = inst.operands.first() {
                    // Get the interval for the return value
                    let state = result.state_at_inst(inst.id);
                    let interval = state
                        .and_then(|s| s.get_opt(return_operand).cloned())
                        .or_else(|| constant_map.get(&return_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(DEFAULT_BITS));

                    return_intervals.push(interval);

                    // Compute return nullness with callee awareness
                    let nullness = compute_return_nullness_with_callees(
                        return_operand,
                        func,
                        module,
                        scc_summaries,
                        external_summaries,
                    );
                    return_nullness_values.push(nullness);
                }
            }
        }
    }

    // Join all return intervals
    if !return_intervals.is_empty() {
        let mut joined = return_intervals[0].clone();
        for interval in return_intervals.iter().skip(1) {
            joined = joined.join(interval);
        }
        summary.return_interval = Some(joined);
    }

    // Join all return nullness values
    if !return_nullness_values.is_empty() {
        let mut joined = return_nullness_values[0];
        for nullness in return_nullness_values.iter().skip(1) {
            joined = joined.join(*nullness);
        }
        summary.return_nullness = Some(joined);
    }

    summary
}

/// Compute nullness for a return value with callee summary awareness.
///
/// Like `compute_return_nullness` but uses provided summaries for call sites.
fn compute_return_nullness_with_callees(
    return_value: ValueId,
    func: &saf_core::air::AirFunction,
    module: &AirModule,
    scc_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    external_summaries: &BTreeMap<FunctionId, FunctionSummary>,
) -> Nullness {
    let mut visited = BTreeSet::new();
    compute_return_nullness_with_callees_inner(
        return_value,
        func,
        module,
        scc_summaries,
        external_summaries,
        &mut visited,
    )
}

fn compute_return_nullness_with_callees_inner(
    return_value: ValueId,
    func: &saf_core::air::AirFunction,
    module: &AirModule,
    scc_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    external_summaries: &BTreeMap<FunctionId, FunctionSummary>,
    visited: &mut BTreeSet<ValueId>,
) -> Nullness {
    // Cycle detection: if we've already visited this value, return MaybeNull
    if !visited.insert(return_value) {
        return Nullness::MaybeNull;
    }

    // Check if it's a constant
    if let Some(constant) = module.constants.get(&return_value) {
        return match constant {
            Constant::Null | Constant::Int { value: 0, .. } => Nullness::Null,
            _ => Nullness::MaybeNull,
        };
    }

    // Find the instruction that defines this value
    for block in &func.blocks {
        for inst in &block.instructions {
            if inst.dst == Some(return_value) {
                return match &inst.op {
                    // Allocations are always non-null
                    Operation::HeapAlloc { .. }
                    | Operation::Alloca { .. }
                    | Operation::Global { .. } => Nullness::NotNull,

                    // GEP on non-null base, Copy/Cast propagate nullness
                    Operation::Gep { .. } | Operation::Copy | Operation::Cast { .. } => {
                        if let Some(&src) = inst.operands.first() {
                            compute_return_nullness_with_callees_inner(
                                src,
                                func,
                                module,
                                scc_summaries,
                                external_summaries,
                                visited,
                            )
                        } else {
                            Nullness::MaybeNull
                        }
                    }

                    // Phi - join all incoming values
                    Operation::Phi { incoming } => {
                        let mut result = Nullness::Bottom;
                        for (_, value) in incoming {
                            let incoming_nullness = compute_return_nullness_with_callees_inner(
                                *value,
                                func,
                                module,
                                scc_summaries,
                                external_summaries,
                                visited,
                            );
                            result = result.join(incoming_nullness);
                        }
                        result
                    }

                    // Select - join both branches
                    Operation::Select => {
                        if inst.operands.len() >= 3 {
                            let true_nullness = compute_return_nullness_with_callees_inner(
                                inst.operands[1],
                                func,
                                module,
                                scc_summaries,
                                external_summaries,
                                visited,
                            );
                            let false_nullness = compute_return_nullness_with_callees_inner(
                                inst.operands[2],
                                func,
                                module,
                                scc_summaries,
                                external_summaries,
                                visited,
                            );
                            true_nullness.join(false_nullness)
                        } else {
                            Nullness::MaybeNull
                        }
                    }

                    // Function calls - use summaries
                    Operation::CallDirect { callee } => {
                        // First check SCC summaries (recursive calls)
                        if let Some(summary) = scc_summaries.get(callee) {
                            summary.return_nullness.unwrap_or(Nullness::MaybeNull)
                        }
                        // Then check external summaries
                        else if let Some(summary) = external_summaries.get(callee) {
                            summary.return_nullness.unwrap_or(Nullness::MaybeNull)
                        } else {
                            Nullness::MaybeNull
                        }
                    }

                    _ => Nullness::MaybeNull,
                };
            }
        }
    }

    // If value is a parameter, it's MaybeNull
    for param in &func.params {
        if param.id == return_value {
            return Nullness::MaybeNull;
        }
    }

    Nullness::MaybeNull
}

/// Refine call sites by applying context-sensitive analysis.
///
/// For each call site, we:
/// 1. Collect the argument intervals at the call site
/// 2. Re-analyze the callee with those arguments bound to parameters
/// 3. Use the resulting return interval for context-specific precision
fn refine_call_sites(
    module: &AirModule,
    result: &AbstractInterpResult,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
    func_map: &BTreeMap<FunctionId, &saf_core::air::AirFunction>,
    constant_map: &BTreeMap<ValueId, Interval>,
    config: &AbstractInterpConfig,
) -> BTreeMap<InstId, AbstractState> {
    let mut refined_states = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(dst) = inst.dst {
                        // Get the existing state at this instruction
                        let base_state = result.state_at_inst(inst.id);

                        // Try context-sensitive analysis first
                        if let Some(callee_func) = func_map.get(callee) {
                            // Collect argument intervals at this call site
                            let mut param_bindings: BTreeMap<ValueId, Interval> = BTreeMap::new();
                            let mut all_args_known = true;

                            for (i, &arg) in inst.operands.iter().enumerate() {
                                if i < callee_func.params.len() {
                                    let param_id = callee_func.params[i].id;
                                    // Get the argument interval from the call site state
                                    let arg_interval = base_state
                                        .and_then(|s| s.get_opt(arg).cloned())
                                        .or_else(|| constant_map.get(&arg).cloned());

                                    if let Some(interval) = arg_interval {
                                        param_bindings.insert(param_id, interval);
                                    } else {
                                        // Unknown argument - fall back to TOP
                                        all_args_known = false;
                                        break;
                                    }
                                }
                            }

                            // If we have concrete argument bindings, re-analyze the callee
                            if all_args_known && !param_bindings.is_empty() {
                                let context_result = solve_function_with_params(
                                    callee_func,
                                    config,
                                    &param_bindings,
                                    constant_map,
                                    module,
                                );

                                // Get the return interval from context-sensitive analysis
                                if let Some(return_interval) = context_result.get(&ValueId::new(0))
                                {
                                    if !return_interval.is_top() {
                                        let mut refined = base_state.cloned().unwrap_or_default();
                                        refined.set(dst, return_interval.clone());
                                        refined_states.insert(inst.id, refined);
                                        continue;
                                    }
                                }
                            }
                        }

                        // Fall back to context-insensitive summary
                        if let Some(summary) = summaries.get(callee) {
                            if let Some(return_interval) = &summary.return_interval {
                                if !return_interval.is_top() {
                                    let mut refined = base_state.cloned().unwrap_or_default();
                                    refined.set(dst, return_interval.clone());
                                    refined_states.insert(inst.id, refined);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    refined_states
}

/// Refine call sites with PTA support for indirect calls.
// INVARIANT: Call site refinement requires module, absint result, summaries,
// function map, constant map, config, and PTA integration for indirect resolution.
#[allow(clippy::too_many_arguments)]
fn refine_call_sites_with_pta(
    module: &AirModule,
    result: &AbstractInterpResult,
    summaries: &BTreeMap<FunctionId, FunctionSummary>,
    func_map: &BTreeMap<FunctionId, &saf_core::air::AirFunction>,
    constant_map: &BTreeMap<ValueId, Interval>,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
) -> BTreeMap<InstId, AbstractState> {
    let mut refined_states = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::CallDirect { callee } => {
                        // Existing direct call handling
                        if let Some(dst) = inst.dst {
                            let base_state = result.state_at_inst(inst.id);

                            if let Some(callee_func) = func_map.get(callee) {
                                if let Some(refined) = try_context_sensitive_call(
                                    inst,
                                    dst,
                                    callee_func,
                                    base_state,
                                    constant_map,
                                    config,
                                    module,
                                ) {
                                    refined_states.insert(inst.id, refined);
                                    continue;
                                }
                            }

                            if let Some(summary) = summaries.get(callee) {
                                if let Some(return_interval) = &summary.return_interval {
                                    if !return_interval.is_top() {
                                        let mut refined = base_state.cloned().unwrap_or_default();
                                        refined.set(dst, return_interval.clone());
                                        refined_states.insert(inst.id, refined);
                                    }
                                }
                            }
                        }
                    }
                    Operation::CallIndirect { .. } => {
                        if let Some(dst) = inst.dst {
                            // Get function pointer from operands (first operand)
                            let Some(&fn_ptr) = inst.operands.first() else {
                                continue;
                            };

                            let targets = pta.resolve_indirect_call(fn_ptr);
                            if targets.is_empty() {
                                continue; // No targets found, keep TOP
                            }

                            let base_state = result.state_at_inst(inst.id);

                            // Join return intervals from all possible targets
                            let mut return_interval = Interval::make_bottom(DEFAULT_BITS);

                            for target_id in &targets {
                                if let Some(summary) = summaries.get(target_id) {
                                    if let Some(ret) = &summary.return_interval {
                                        return_interval = return_interval.join(ret);
                                    } else {
                                        // Void or unknown return - go to top
                                        return_interval = Interval::make_top(DEFAULT_BITS);
                                        break;
                                    }
                                } else {
                                    // Unknown function - go to top
                                    return_interval = Interval::make_top(DEFAULT_BITS);
                                    break;
                                }
                            }

                            if !return_interval.is_top() && !return_interval.is_bottom() {
                                let mut refined = base_state.cloned().unwrap_or_default();
                                refined.set(dst, return_interval);
                                refined_states.insert(inst.id, refined);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    refined_states
}

/// Try context-sensitive analysis for a direct call.
fn try_context_sensitive_call(
    inst: &saf_core::air::Instruction,
    dst: ValueId,
    callee_func: &saf_core::air::AirFunction,
    base_state: Option<&AbstractState>,
    constant_map: &BTreeMap<ValueId, Interval>,
    config: &AbstractInterpConfig,
    module: &AirModule,
) -> Option<AbstractState> {
    let mut param_bindings: BTreeMap<ValueId, Interval> = BTreeMap::new();

    for (i, &arg) in inst.operands.iter().enumerate() {
        if i < callee_func.params.len() {
            let param_id = callee_func.params[i].id;
            let arg_interval = base_state
                .and_then(|s| s.get_opt(arg).cloned())
                .or_else(|| constant_map.get(&arg).cloned());

            if let Some(interval) = arg_interval {
                param_bindings.insert(param_id, interval);
            } else {
                return None; // Unknown argument
            }
        }
    }

    if param_bindings.is_empty() {
        return None;
    }

    let context_result =
        solve_function_with_params(callee_func, config, &param_bindings, constant_map, module);

    if let Some(return_interval) = context_result.get(&ValueId::new(0)) {
        if !return_interval.is_top() {
            let mut refined = base_state.cloned().unwrap_or_default();
            refined.set(dst, return_interval.clone());
            return Some(refined);
        }
    }

    None
}
