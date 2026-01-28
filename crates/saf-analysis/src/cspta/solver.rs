//! Context-sensitive worklist solver for k-CFA pointer analysis.
//!
//! Implements a k-call-site-sensitive Andersen-style points-to analysis.
//! Each value is qualified by a `CallSiteContext` representing the call
//! chain leading to the function that contains it. Functions in recursive
//! SCCs use k-1 limiting (truncating the context to `k-1` entries) to
//! maintain partial context sensitivity while ensuring termination.
//!
//! Supports multiple points-to set representations through the `PtsSet` trait
//! for scalability with large programs.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, InstId, LocId, ObjId, ValueId};

use crate::callgraph::CallGraph;
use crate::graph_algo::tarjan_scc;
use crate::pta::ptsset::{
    BTreePtsSet, BddPtsSet, FxHashPtsSet, PtsConfig, PtsRepresentation, PtsSet, RoaringPtsSet,
};
use crate::pta::{
    ConstraintSet, FieldPath, IndexSensitivity, LocationFactory, extract_global_initializers,
    extract_intraprocedural_constraints, precompute_indexed_locations,
};

use super::context::CallSiteContext;

/// Configuration for context-sensitive pointer analysis.
#[derive(Debug, Clone)]
pub struct CsPtaConfig {
    /// Context depth (1, 2, or 3).
    pub k: u32,
    /// Field sensitivity setting.
    pub field_sensitivity: crate::pta::FieldSensitivity,
    /// Maximum solver iterations.
    pub max_iterations: usize,
    /// Maximum abstract objects.
    pub max_objects: usize,
    /// Points-to set representation configuration.
    pub pts_config: PtsConfig,
}

impl Default for CsPtaConfig {
    fn default() -> Self {
        Self {
            k: 1,
            field_sensitivity: crate::pta::FieldSensitivity::StructFields { max_depth: 2 },
            max_iterations: 2_000_000,
            max_objects: 200_000,
            pts_config: PtsConfig::default(),
        }
    }
}

impl CsPtaConfig {
    /// Create a config that uses `BTreeSet` for points-to sets.
    #[must_use]
    pub fn with_btreeset(mut self) -> Self {
        self.pts_config = PtsConfig::btreeset();
        self
    }

    /// Create a config that uses `BitVector` for points-to sets.
    #[must_use]
    pub fn with_bitvector(mut self) -> Self {
        self.pts_config = PtsConfig::bitvector();
        self
    }

    /// Create a config that uses `BDD` for points-to sets.
    #[must_use]
    pub fn with_bdd(mut self) -> Self {
        self.pts_config = PtsConfig::bdd();
        self
    }

    /// Set the points-to set representation explicitly.
    #[must_use]
    pub fn with_pts_representation(mut self, repr: PtsRepresentation) -> Self {
        self.pts_config = self.pts_config.with_representation(repr);
        self
    }
}

/// A context-qualified value: (value, context) pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CtxValue {
    /// The SSA value.
    pub value: ValueId,
    /// The call-site context.
    pub ctx: CallSiteContext,
}

/// Diagnostics for context-sensitive PTA.
#[derive(Debug, Clone, Default)]
pub struct CsPtaDiagnostics {
    /// Number of solver iterations.
    pub iterations: usize,
    /// Whether the iteration limit was hit.
    pub iteration_limit_hit: bool,
    /// Number of unique contexts created.
    pub context_count: usize,
    /// Maximum points-to set size.
    pub max_pts_size: usize,
    /// Number of SCC functions (using k-1 limited context).
    pub scc_function_count: usize,
    /// Number of constraint types.
    pub constraint_count: usize,
    /// Number of locations.
    pub location_count: usize,
    /// Number of heap-cloned objects.
    pub heap_clone_count: usize,
}

/// Result of context-sensitive pointer analysis.
pub struct CsPtaResult {
    /// Full context-sensitive points-to map.
    cs_pts: BTreeMap<CtxValue, BTreeSet<LocId>>,
    /// Context-insensitive summary (union over all contexts per value).
    ci_summary: BTreeMap<ValueId, BTreeSet<LocId>>,
    /// All abstract locations.
    locations: BTreeMap<LocId, crate::pta::Location>,
    /// Allocation multiplicity classification per base object.
    multiplicities: BTreeMap<ObjId, crate::pta::AllocationMultiplicity>,
    /// Analysis diagnostics.
    diagnostics: CsPtaDiagnostics,
}

impl CsPtaResult {
    /// Get the context-specific points-to set for a value.
    #[must_use]
    pub fn points_to(&self, ptr: ValueId, ctx: &CallSiteContext) -> Vec<LocId> {
        let key = CtxValue {
            value: ptr,
            ctx: ctx.clone(),
        };
        self.cs_pts
            .get(&key)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get the CI summary points-to set (union across all contexts).
    #[must_use]
    pub fn points_to_any(&self, ptr: ValueId) -> Vec<LocId> {
        self.ci_summary
            .get(&ptr)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Check if a location is provably unique (one concrete object).
    fn is_unique(&self, loc: LocId) -> bool {
        self.locations
            .get(&loc)
            .and_then(|location| self.multiplicities.get(&location.obj))
            .copied()
            .unwrap_or_default()
            == crate::pta::AllocationMultiplicity::Unique
    }

    /// Context-qualified alias query.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn may_alias(
        &self,
        p: ValueId,
        p_ctx: &CallSiteContext,
        q: ValueId,
        q_ctx: &CallSiteContext,
    ) -> crate::pta::AliasResult {
        let p_key = CtxValue {
            value: p,
            ctx: p_ctx.clone(),
        };
        let q_key = CtxValue {
            value: q,
            ctx: q_ctx.clone(),
        };
        match (self.cs_pts.get(&p_key), self.cs_pts.get(&q_key)) {
            (None, _) | (_, None) => crate::pta::AliasResult::Unknown,
            (Some(ps), Some(qs)) => {
                if ps.is_empty() || qs.is_empty() {
                    crate::pta::AliasResult::Unknown
                } else if ps.is_disjoint(qs) {
                    crate::pta::AliasResult::No
                } else if ps == qs && ps.len() == 1 {
                    let loc = *ps.iter().next().unwrap();
                    if self.is_unique(loc) {
                        crate::pta::AliasResult::Must
                    } else {
                        crate::pta::AliasResult::May
                    }
                } else if ps == qs {
                    // Non-singleton equal sets: MayAlias
                    crate::pta::AliasResult::May
                } else if ps.is_subset(qs) || qs.is_subset(ps) {
                    // One is a proper subset of the other means PartialAlias
                    crate::pta::AliasResult::Partial
                } else {
                    crate::pta::AliasResult::May
                }
            }
        }
    }

    /// CI summary alias query (union across all contexts).
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn may_alias_any(&self, p: ValueId, q: ValueId) -> crate::pta::AliasResult {
        // Identity check: same SSA value always aliases with itself
        if p == q {
            return match self.ci_summary.get(&p) {
                Some(pts) if !pts.is_empty() => crate::pta::AliasResult::Must,
                Some(_) => crate::pta::AliasResult::No,
                None => crate::pta::AliasResult::Unknown,
            };
        }
        match (self.ci_summary.get(&p), self.ci_summary.get(&q)) {
            (None, _) | (_, None) => crate::pta::AliasResult::Unknown,
            (Some(ps), Some(qs)) => {
                if ps.is_empty() || qs.is_empty() {
                    crate::pta::AliasResult::Unknown
                } else if ps.is_disjoint(qs) {
                    crate::pta::AliasResult::No
                } else if ps == qs && ps.len() == 1 {
                    let loc = *ps.iter().next().unwrap();
                    if self.is_unique(loc) {
                        crate::pta::AliasResult::Must
                    } else {
                        crate::pta::AliasResult::May
                    }
                } else if ps == qs {
                    // Non-singleton equal sets: MayAlias
                    crate::pta::AliasResult::May
                } else if ps.is_subset(qs) || qs.is_subset(ps) {
                    // One is a proper subset of the other means PartialAlias
                    crate::pta::AliasResult::Partial
                } else {
                    crate::pta::AliasResult::May
                }
            }
        }
    }

    /// Enumerate all contexts seen for a value.
    #[must_use]
    pub fn contexts_for(&self, value: ValueId) -> Vec<CallSiteContext> {
        self.cs_pts
            .keys()
            .filter(|k| k.value == value)
            .map(|k| k.ctx.clone())
            .collect()
    }

    /// Get the analysis diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &CsPtaDiagnostics {
        &self.diagnostics
    }

    /// Get all locations.
    #[must_use]
    pub fn locations(&self) -> &BTreeMap<LocId, crate::pta::Location> {
        &self.locations
    }

    /// Get the raw context-sensitive points-to map.
    #[must_use]
    pub fn cs_points_to_map(&self) -> &BTreeMap<CtxValue, BTreeSet<LocId>> {
        &self.cs_pts
    }

    /// Get the CI summary map.
    #[must_use]
    pub fn ci_summary_map(&self) -> &BTreeMap<ValueId, BTreeSet<LocId>> {
        &self.ci_summary
    }
}

/// Solve context-sensitive points-to analysis.
///
/// Runs a k-CFA worklist algorithm on the given module. Functions in
/// recursive SCCs use k-1 limiting for context sensitivity. Heap
/// allocations are cloned per-context for precision.
///
/// Automatically selects the points-to set representation based on
/// `config.pts_config`:
/// - `Auto`: Select based on allocation site count
/// - `BTreeSet`: Simple baseline, good for small programs
/// - `BitVector`: Fast operations for medium programs
/// - `Bdd`: Compact representation for large programs
pub fn solve_context_sensitive(
    module: &AirModule,
    callgraph: &CallGraph,
    config: &CsPtaConfig,
) -> CsPtaResult {
    solve_context_sensitive_with_resolved(module, callgraph, config, &BTreeMap::new())
}

/// Solve context-sensitive PTA with pre-resolved indirect call sites.
///
/// This variant accepts a map from `InstId` to resolved callee `FunctionId`s,
/// typically produced by CHA (Class Hierarchy Analysis) for virtual calls.
/// Resolved indirect calls are treated the same as direct calls for
/// interprocedural argument/return propagation.
pub fn solve_context_sensitive_with_resolved(
    module: &AirModule,
    callgraph: &CallGraph,
    config: &CsPtaConfig,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
) -> CsPtaResult {
    // Determine which representation to use
    let repr = config.pts_config.select_for_module(module);

    // Dispatch to the appropriate generic solver
    match repr {
        PtsRepresentation::Auto | PtsRepresentation::BTreeSet => {
            solve_cs_generic_with_resolved::<BTreePtsSet>(module, callgraph, config, resolved_sites)
        }
        PtsRepresentation::FxHash => solve_cs_generic_with_resolved::<FxHashPtsSet>(
            module,
            callgraph,
            config,
            resolved_sites,
        ),
        PtsRepresentation::BitVector | PtsRepresentation::Roaring => {
            solve_cs_generic_with_resolved::<RoaringPtsSet>(
                module,
                callgraph,
                config,
                resolved_sites,
            )
        }
        PtsRepresentation::Bdd => {
            solve_cs_generic_with_resolved::<BddPtsSet>(module, callgraph, config, resolved_sites)
        }
    }
}

/// Solve context-sensitive PTA using a specific points-to set representation.
///
/// This is the generic version that can use any `PtsSet` implementation.
pub fn solve_cs_generic<P: PtsSet>(
    module: &AirModule,
    callgraph: &CallGraph,
    config: &CsPtaConfig,
) -> CsPtaResult {
    solve_cs_generic_with_resolved::<P>(module, callgraph, config, &BTreeMap::new())
}

/// Solve context-sensitive PTA with resolved indirect calls.
pub fn solve_cs_generic_with_resolved<P: PtsSet>(
    module: &AirModule,
    callgraph: &CallGraph,
    config: &CsPtaConfig,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
) -> CsPtaResult {
    let mut solver =
        GenericCsSolver::<P>::new_with_resolved(module, callgraph, config, resolved_sites);
    solver.solve();
    solver.into_result()
}

// =============================================================================
// Internal solver
// =============================================================================

/// Call-site info extracted from the module.
struct CallSiteInfo {
    /// The call instruction ID.
    inst_id: InstId,
    /// The callee function ID.
    callee: FunctionId,
    /// Actual arguments (ValueId per position).
    args: Vec<ValueId>,
    /// Destination (call result) if any.
    dst: Option<ValueId>,
}

/// Generic context-sensitive solver, parameterized by points-to set type.
struct GenericCsSolver<'a, P: PtsSet> {
    config: &'a CsPtaConfig,

    /// Template set used to create new empty sets via `clone_empty()`.
    /// Shares indexer state with all sets created from it, enabling fast
    /// bitwise operations on indexed representations.
    template: P,

    /// Base constraints (intraprocedural only).
    constraints: ConstraintSet,
    /// Location factory.
    factory: LocationFactory,

    /// Context-sensitive points-to sets for values.
    cs_pts: BTreeMap<CtxValue, P>,
    /// Context-sensitive points-to sets for locations (for Store/Load).
    /// Used for callee-local cloned locations to maintain per-context precision.
    loc_pts: BTreeMap<(LocId, CallSiteContext), P>,
    /// Context-insensitive location points-to mirror.
    /// ALL stores also update this map, ensuring stores from any context are
    /// visible to loads in any other context. This fixes cross-context
    /// Store/Load invisibility for caller/global locations.
    global_loc_pts: BTreeMap<LocId, P>,
    /// Worklist.
    worklist: BTreeSet<CtxValue>,
    /// Location worklist (context-sensitive).
    loc_worklist: BTreeSet<(LocId, CallSiteContext)>,
    /// Location worklist (context-insensitive).
    global_loc_worklist: BTreeSet<LocId>,

    /// Heap cloning: maps (original_loc, context) → cloned_loc for callee-local
    /// allocations. Enables per-context separation of callee allocas (e.g., -O0
    /// retval allocas) while keeping caller locations globally visible.
    clone_map: BTreeMap<(LocId, CallSiteContext), LocId>,
    /// Reverse clone map: cloned_loc → original_loc.
    clone_reverse: BTreeMap<LocId, LocId>,
    /// Set of all cloned LocIds (for fast lookup).
    cloned_locs: BTreeSet<LocId>,

    /// Function ID → parameter `ValueId` values.
    func_params: BTreeMap<FunctionId, Vec<ValueId>>,
    /// Function ID → return value `ValueId` values.
    func_returns: BTreeMap<FunctionId, Vec<ValueId>>,
    /// All call sites.
    call_sites: Vec<CallSiteInfo>,
    /// Function ID → call site indices (calls TO this function).
    calls_to: BTreeMap<FunctionId, Vec<usize>>,
    /// Function ID → call site indices (calls FROM this function).
    calls_from: BTreeMap<FunctionId, Vec<usize>>,
    /// Value ID → containing function ID.
    value_to_func: BTreeMap<ValueId, FunctionId>,
    /// SCC functions (using k-1 limited context).
    scc_functions: BTreeSet<FunctionId>,

    /// Function ID → Addr constraints (for context-qualified re-seeding).
    func_addr_constraints: BTreeMap<FunctionId, Vec<(ValueId, LocId)>>,
    /// Module-level addr constraints whose `ptr` is not in any function
    /// (e.g., global variable addresses). Must be seeded in every new
    /// context so that stores using global values as operands can find
    /// the global's points-to set in the callee context.
    global_addr_constraints: Vec<(ValueId, LocId)>,
    /// (Function, Context) pairs already initialized.
    initialized_contexts: BTreeSet<(FunctionId, CallSiteContext)>,

    /// Reverse index: `ValueId` → set of `CallSiteContext` values that have
    /// entries in `cs_pts` for that value. Avoids full `cs_pts` scan in
    /// `process_global_location`.
    value_contexts: BTreeMap<ValueId, BTreeSet<CallSiteContext>>,

    /// Diagnostics.
    diagnostics: CsPtaDiagnostics,
}

/// Collect value-to-function mapping and call site information from the module.
///
/// Returns `(value_to_func, call_sites, calls_to, calls_from)`.
#[allow(clippy::type_complexity)]
fn collect_call_site_info(
    module: &AirModule,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
) -> (
    BTreeMap<ValueId, FunctionId>,
    Vec<CallSiteInfo>,
    BTreeMap<FunctionId, Vec<usize>>,
    BTreeMap<FunctionId, Vec<usize>>,
) {
    let mut value_to_func: BTreeMap<ValueId, FunctionId> = BTreeMap::new();
    let mut call_sites: Vec<CallSiteInfo> = Vec::new();
    let mut calls_to: BTreeMap<FunctionId, Vec<usize>> = BTreeMap::new();
    let mut calls_from: BTreeMap<FunctionId, Vec<usize>> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for param in &func.params {
            value_to_func.insert(param.id, func.id);
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    value_to_func.insert(dst, func.id);
                }
                match &inst.op {
                    Operation::CallDirect { callee } => {
                        let idx = call_sites.len();
                        call_sites.push(CallSiteInfo {
                            inst_id: inst.id,
                            callee: *callee,
                            args: inst.operands.clone(),
                            dst: inst.dst,
                        });
                        calls_to.entry(*callee).or_default().push(idx);
                        calls_from.entry(func.id).or_default().push(idx);
                    }
                    Operation::CallIndirect { .. } => {
                        if let Some(targets) = resolved_sites.get(&inst.id) {
                            let args = if inst.operands.is_empty() {
                                Vec::new()
                            } else {
                                inst.operands[..inst.operands.len() - 1].to_vec()
                            };
                            for &target in targets {
                                let idx = call_sites.len();
                                call_sites.push(CallSiteInfo {
                                    inst_id: inst.id,
                                    callee: target,
                                    args: args.clone(),
                                    dst: inst.dst,
                                });
                                calls_to.entry(target).or_default().push(idx);
                                calls_from.entry(func.id).or_default().push(idx);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    (value_to_func, call_sites, calls_to, calls_from)
}

impl<'a, P: PtsSet> GenericCsSolver<'a, P> {
    fn new_with_resolved(
        module: &'a AirModule,
        callgraph: &'a CallGraph,
        config: &'a CsPtaConfig,
        resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
    ) -> Self {
        let mut factory = LocationFactory::new(config.field_sensitivity.clone());
        // Use intraprocedural-only constraints; interprocedural flow is handled
        // by the context-qualified collect_call_arg_updates / collect_return_updates.
        let mut constraints = extract_intraprocedural_constraints(module, &mut factory);

        // Extract constraints from global aggregate initializers (vtables,
        // function pointer tables, struct initializers with pointer fields)
        extract_global_initializers(module, &mut factory, &mut constraints);

        // Pre-create field locations for all GEP constraints to ensure proper
        // field sensitivity (without this, different struct fields may collapse
        // to the same base location)
        precompute_indexed_locations(
            &constraints,
            &mut factory,
            &module.constants,
            IndexSensitivity::Collapsed, // Use same default as CI-PTA
        );

        // Classify allocation multiplicity for must-alias soundness
        crate::pta::multiplicity::classify_multiplicity(module, &mut factory);

        // Build function→params map
        let func_params: BTreeMap<FunctionId, Vec<ValueId>> = module
            .functions
            .iter()
            .map(|f| (f.id, f.params.iter().map(|p| p.id).collect()))
            .collect();

        // Build function→return values
        let func_returns: BTreeMap<FunctionId, Vec<ValueId>> = module
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
            .collect();

        let (value_to_func, call_sites, calls_to, calls_from) =
            collect_call_site_info(module, resolved_sites);

        // Compute SCC functions on call graph
        let scc_functions = compute_scc_functions(callgraph);

        // Build function→Addr constraints for context-qualified re-seeding.
        // Also collect module-level addr constraints (globals) that are not
        // associated with any function — these must be seeded in every context.
        let mut func_addr_constraints: BTreeMap<FunctionId, Vec<(ValueId, LocId)>> =
            BTreeMap::new();
        let mut global_addr_constraints: Vec<(ValueId, LocId)> = Vec::new();
        for addr in &constraints.addr {
            if let Some(&fid) = value_to_func.get(&addr.ptr) {
                func_addr_constraints
                    .entry(fid)
                    .or_default()
                    .push((addr.ptr, addr.loc));
            } else {
                global_addr_constraints.push((addr.ptr, addr.loc));
            }
        }

        let diagnostics = CsPtaDiagnostics {
            constraint_count: constraints.total_count(),
            location_count: factory.len(),
            scc_function_count: scc_functions.len(),
            ..Default::default()
        };

        Self {
            config,
            template: P::empty(),
            constraints,
            factory,
            cs_pts: BTreeMap::new(),
            loc_pts: BTreeMap::new(),
            global_loc_pts: BTreeMap::new(),
            worklist: BTreeSet::new(),
            loc_worklist: BTreeSet::new(),
            global_loc_worklist: BTreeSet::new(),
            clone_map: BTreeMap::new(),
            clone_reverse: BTreeMap::new(),
            cloned_locs: BTreeSet::new(),
            func_params,
            func_returns,
            call_sites,
            calls_to,
            calls_from,
            value_to_func,
            scc_functions,
            func_addr_constraints,
            global_addr_constraints,
            initialized_contexts: BTreeSet::new(),
            value_contexts: BTreeMap::new(),
            diagnostics,
        }
    }

    fn solve(&mut self) {
        // Phase 1: Initialize from Addr constraints with empty context
        let empty_ctx = CallSiteContext::empty();
        // Mark all functions as initialized in the empty context
        let all_func_ids: Vec<FunctionId> = self.func_addr_constraints.keys().copied().collect();
        for fid in &all_func_ids {
            self.initialized_contexts.insert((*fid, empty_ctx.clone()));
        }
        for addr in &self.constraints.addr {
            let key = CtxValue {
                value: addr.ptr,
                ctx: empty_ctx.clone(),
            };
            self.value_contexts
                .entry(key.value)
                .or_default()
                .insert(key.ctx.clone());
            if self
                .cs_pts
                .entry(key.clone())
                .or_insert_with(|| self.template.clone_empty())
                .insert(addr.loc)
            {
                self.worklist.insert(key);
            }
        }

        // Phase 1.5: Top-down context creation for all call sites.
        // For each call site, create the callee context and seed its Addr
        // constraints. This ensures functions are analyzed in context even
        // when no pointer arguments flow (e.g. factory patterns that only
        // return pointers).
        self.seed_call_site_contexts();

        // Phase 2: Fixed-point iteration
        let mut iterations = 0;
        while iterations < self.config.max_iterations {
            iterations += 1;

            if let Some(cv) = self.worklist.pop_first() {
                self.process_value(&cv);
                continue;
            }

            if let Some(loc_ctx) = self.loc_worklist.pop_first() {
                self.process_location(&loc_ctx);
                continue;
            }

            if let Some(loc) = self.global_loc_worklist.pop_first() {
                self.process_global_location(loc);
                continue;
            }

            break;
        }

        self.diagnostics.iterations = iterations;
        self.diagnostics.iteration_limit_hit = iterations >= self.config.max_iterations;

        // Compute context count and max pts size
        let mut contexts: BTreeSet<CallSiteContext> = BTreeSet::new();
        let mut max_pts = 0;
        for (cv, locs) in &self.cs_pts {
            contexts.insert(cv.ctx.clone());
            if locs.len() > max_pts {
                max_pts = locs.len();
            }
        }
        self.diagnostics.context_count = contexts.len();
        self.diagnostics.max_pts_size = max_pts;
        self.diagnostics.location_count = self.factory.len();
    }

    // NOTE: Processes all constraint types (Copy, Load, Store, GEP, Call)
    // for a single context-sensitive value in one worklist iteration.
    #[allow(clippy::too_many_lines)]
    fn process_value(&mut self, cv: &CtxValue) {
        let v_pts = match self.cs_pts.get(cv) {
            Some(s) => s.clone(),
            None => return,
        };

        // Collect updates to apply (avoids borrowing self immutably and mutably)
        let mut value_updates: Vec<(CtxValue, P)> = Vec::new();
        let mut loc_updates: Vec<((LocId, CallSiteContext), P)> = Vec::new();

        // Intraprocedural Copy constraints: same context
        for copy in &self.constraints.copy {
            if copy.src == cv.value {
                let dst_cv = CtxValue {
                    value: copy.dst,
                    ctx: cv.ctx.clone(),
                };
                value_updates.push((dst_cv, v_pts.clone()));
            }
        }

        // Load constraints: same context
        // Try context-sensitive loc_pts first (for cloned callee-local locs),
        // then fall back to global_loc_pts (for cross-context visibility).
        for load in &self.constraints.load {
            if load.src_ptr == cv.value {
                for loc in v_pts.iter() {
                    let loc_set = self.load_from_location(loc, &cv.ctx);
                    if !loc_set.is_empty() {
                        let dst_cv = CtxValue {
                            value: load.dst,
                            ctx: cv.ctx.clone(),
                        };
                        value_updates.push((dst_cv, loc_set));
                    }
                }
            }
        }

        // Store constraints: same context
        for store in &self.constraints.store {
            if store.dst_ptr == cv.value {
                let src_cv = CtxValue {
                    value: store.src,
                    ctx: cv.ctx.clone(),
                };
                if let Some(src_pts) = self.cs_pts.get(&src_cv).cloned() {
                    for loc in v_pts.iter() {
                        let loc_key = (loc, cv.ctx.clone());
                        loc_updates.push((loc_key, src_pts.clone()));
                    }
                }
            }
            if store.src == cv.value {
                let dst_cv = CtxValue {
                    value: store.dst_ptr,
                    ctx: cv.ctx.clone(),
                };
                if let Some(dst_pts) = self.cs_pts.get(&dst_cv).cloned() {
                    for loc in dst_pts.iter() {
                        let loc_key = (loc, cv.ctx.clone());
                        loc_updates.push((loc_key, v_pts.clone()));
                    }
                }
            }
        }

        // Gep constraints: same context
        for gep in &self.constraints.gep {
            if gep.src_ptr == cv.value {
                for loc in v_pts.iter() {
                    // For cloned locs, resolve back to original to look up in factory
                    let effective_loc = self.clone_reverse.get(&loc).copied().unwrap_or(loc);
                    if let Some(base_loc) = self.factory.get(effective_loc) {
                        let new_path = base_loc.path.extend(&gep.path);
                        if let Some(field_loc) =
                            self.find_or_approximate_location(base_loc.obj, &new_path)
                        {
                            // If the base loc was cloned, use the cloned field loc
                            let effective_field = if self.cloned_locs.contains(&loc) {
                                self.clone_map
                                    .get(&(field_loc, cv.ctx.clone()))
                                    .copied()
                                    .unwrap_or(field_loc)
                            } else {
                                field_loc
                            };
                            let new_pts = {
                                let mut s = self.template.clone_empty();
                                s.insert(effective_field);
                                s
                            };
                            let dst_cv = CtxValue {
                                value: gep.dst,
                                ctx: cv.ctx.clone(),
                            };
                            value_updates.push((dst_cv, new_pts));
                        }
                    }
                }
            }
        }

        // Interprocedural: call arg propagation
        let mut new_contexts: Vec<(FunctionId, CallSiteContext)> = Vec::new();
        self.collect_call_arg_updates(cv, &v_pts, &mut value_updates, &mut new_contexts);

        // Interprocedural: return propagation
        self.collect_return_updates(cv, &v_pts, &mut value_updates);

        // Apply all collected updates
        for (dst_cv, locs) in value_updates {
            if self.union_into_value(&dst_cv, &locs) {
                self.worklist.insert(dst_cv);
            }
        }
        for (loc_key, locs) in loc_updates {
            if self.union_into_location(&loc_key, &locs) {
                self.loc_worklist.insert(loc_key);
            }
        }

        // Seed Addr constraints for newly-entered function contexts
        for (func_id, ctx) in new_contexts {
            self.seed_function_in_context(func_id, &ctx);
        }
    }

    /// Collect argument→parameter propagation updates and new contexts.
    fn collect_call_arg_updates(
        &self,
        cv: &CtxValue,
        v_pts: &P,
        updates: &mut Vec<(CtxValue, P)>,
        new_contexts: &mut Vec<(FunctionId, CallSiteContext)>,
    ) {
        let Some(&func_id) = self.value_to_func.get(&cv.value) else {
            return;
        };
        let Some(site_indices) = self.calls_from.get(&func_id) else {
            return;
        };

        for &idx in site_indices {
            let site = &self.call_sites[idx];
            for (i, &arg) in site.args.iter().enumerate() {
                if arg != cv.value {
                    continue;
                }
                let Some(params) = self.func_params.get(&site.callee) else {
                    continue;
                };
                let Some(&param_id) = params.get(i) else {
                    continue;
                };

                let callee_ctx = self.callee_context(&cv.ctx, site.inst_id, site.callee);

                // Track if this is a new (function, context) pair
                let key = (site.callee, callee_ctx.clone());
                if !self.initialized_contexts.contains(&key) {
                    new_contexts.push(key);
                }

                updates.push((
                    CtxValue {
                        value: param_id,
                        ctx: callee_ctx,
                    },
                    v_pts.clone(),
                ));
            }
        }
    }

    /// Collect return value→call result propagation updates.
    fn collect_return_updates(&self, cv: &CtxValue, v_pts: &P, updates: &mut Vec<(CtxValue, P)>) {
        let Some(&func_id) = self.value_to_func.get(&cv.value) else {
            return;
        };
        let Some(ret_vals) = self.func_returns.get(&func_id) else {
            return;
        };
        if !ret_vals.contains(&cv.value) {
            return;
        }
        let Some(site_indices) = self.calls_to.get(&func_id) else {
            return;
        };

        // Pop the most recent call site to recover the caller's context
        // and the specific call site ID that created this callee context.
        // For SCC functions with k-1 limiting, the context may still be
        // non-empty (when k>=2), allowing call-site filtering. When the
        // context is empty (k=1 or no prior call sites), we fall back to
        // filtering by function identity — only propagating to callers
        // that actually call this specific function (not other SCC members).
        let (caller_ctx, matching_site) = cv.ctx.pop();

        for &idx in site_indices {
            let site = &self.call_sites[idx];
            let Some(dst) = site.dst else {
                continue;
            };

            // Filter return propagation by call site.
            // When the context contains the call site (non-empty context),
            // only propagate to the specific call site that created this
            // context. This prevents returns from context [CS1] leaking to
            // call sites CS2, CS3, etc.
            if let Some(site_id) = matching_site {
                if site.inst_id != site_id {
                    continue;
                }
            }

            // For SCC functions with empty callee context (k=1), verify
            // the caller context is consistent: recompute what the callee
            // context would be from this call site's caller context, and
            // only propagate if it matches the current callee context.
            // This prevents SCC return values from being broadcast to all
            // callers when multiple callers exist with different contexts.
            if matching_site.is_none() && self.scc_functions.contains(&func_id) {
                let expected_callee_ctx = self.callee_context(&caller_ctx, site.inst_id, func_id);
                if expected_callee_ctx != cv.ctx {
                    continue;
                }
            }

            updates.push((
                CtxValue {
                    value: dst,
                    ctx: caller_ctx.clone(),
                },
                v_pts.clone(),
            ));
        }
    }

    /// Compute the context for entering a callee.
    ///
    /// For non-SCC functions, appends the call site and k-limits normally.
    /// For SCC functions, uses k-1 limiting: appends the call site then
    /// truncates to `k-1` entries. This preserves some context sensitivity
    /// within recursive SCCs instead of collapsing everything to the empty
    /// context, while still ensuring termination (the context can grow to
    /// at most `k-1` entries, bounding the state space).
    fn callee_context(
        &self,
        caller_ctx: &CallSiteContext,
        call_site: InstId,
        callee: FunctionId,
    ) -> CallSiteContext {
        if self.scc_functions.contains(&callee) {
            // k-1 limiting: push then truncate to k-1 entries.
            // When k=1 this gives empty context (same as before);
            // when k>=2 it retains k-1 levels of precision.
            let full = caller_ctx.push(call_site, self.config.k);
            let limit = self.config.k.saturating_sub(1) as usize;
            full.truncate(limit)
        } else {
            caller_ctx.push(call_site, self.config.k)
        }
    }

    /// Seed a function's `Addr` constraints in a new context.
    ///
    /// Called when the solver first enters a function in a particular context.
    /// For non-empty contexts (callee entries), creates **cloned** `LocId`s for
    /// each callee-local allocation. This ensures different calling contexts
    /// get separate abstract locations for callee-local allocas (e.g., -O0
    /// retval allocas), enabling precise per-context data flow.
    fn seed_function_in_context(&mut self, func_id: FunctionId, ctx: &CallSiteContext) {
        let key = (func_id, ctx.clone());
        if !self.initialized_contexts.insert(key) {
            return; // already initialized
        }

        let should_clone = !ctx.is_empty();

        // Seed function-local addr constraints (with cloning for callee-local allocs)
        if let Some(addr_pairs) = self.func_addr_constraints.get(&func_id).cloned() {
            for (ptr, loc) in addr_pairs {
                let effective_loc = if should_clone {
                    // Create a cloned LocId for this (loc, context) pair
                    self.clone_location(loc, ctx)
                } else {
                    loc
                };

                let cv = CtxValue {
                    value: ptr,
                    ctx: ctx.clone(),
                };
                self.value_contexts
                    .entry(cv.value)
                    .or_default()
                    .insert(cv.ctx.clone());
                if self
                    .cs_pts
                    .entry(cv.clone())
                    .or_insert_with(|| self.template.clone_empty())
                    .insert(effective_loc)
                {
                    self.worklist.insert(cv);
                }
            }
        }

        // Seed module-level addr constraints (globals) in this context.
        // Global variables are defined at module scope — their ValueId is not
        // in any function's instruction list.  Without this seeding, stores
        // that use a global value as source (e.g., `store @obj, %q`) would
        // silently fail because the global's PTS only exists in the empty
        // context.  Globals are never cloned (they are shared across contexts).
        if should_clone {
            for &(ptr, loc) in &self.global_addr_constraints {
                let cv = CtxValue {
                    value: ptr,
                    ctx: ctx.clone(),
                };
                self.value_contexts
                    .entry(cv.value)
                    .or_default()
                    .insert(cv.ctx.clone());
                if self
                    .cs_pts
                    .entry(cv.clone())
                    .or_insert_with(|| self.template.clone_empty())
                    .insert(loc)
                {
                    self.worklist.insert(cv);
                }
            }
        }
    }

    /// Create a cloned `LocId` for a callee-local allocation in a specific context.
    ///
    /// The clone is derived from the original `LocId` + context, ensuring
    /// deterministic IDs. Also clones all field-sensitive sub-locations for
    /// the same object.
    fn clone_location(&mut self, original: LocId, ctx: &CallSiteContext) -> LocId {
        // Check if already cloned
        if let Some(&cloned) = self.clone_map.get(&(original, ctx.clone())) {
            return cloned;
        }

        // Derive a new LocId from original + context
        let mut data = Vec::new();
        data.extend_from_slice(&original.raw().to_le_bytes());
        for site in ctx.sites() {
            data.extend_from_slice(&site.raw().to_le_bytes());
        }
        let cloned = LocId::derive(&data);

        self.clone_map.insert((original, ctx.clone()), cloned);
        self.clone_reverse.insert(cloned, original);
        self.cloned_locs.insert(cloned);

        // Also clone field sub-locations for the same object
        if let Some(orig_loc) = self.factory.get(original) {
            let obj = orig_loc.obj;
            let field_locs: Vec<(LocId, FieldPath)> = self
                .factory
                .all_locations()
                .iter()
                .filter(|(_, loc)| loc.obj == obj && !loc.path.steps.is_empty())
                .map(|(&id, loc)| (id, loc.path.clone()))
                .collect();
            for (field_id, _path) in field_locs {
                if field_id != original {
                    let mut field_data = Vec::new();
                    field_data.extend_from_slice(&field_id.raw().to_le_bytes());
                    for site in ctx.sites() {
                        field_data.extend_from_slice(&site.raw().to_le_bytes());
                    }
                    let cloned_field = LocId::derive(&field_data);
                    self.clone_map.insert((field_id, ctx.clone()), cloned_field);
                    self.clone_reverse.insert(cloned_field, field_id);
                    self.cloned_locs.insert(cloned_field);
                }
            }
        }

        cloned
    }

    /// Create callee contexts for all call sites (top-down).
    ///
    /// Iterates over all call sites, computes the callee context from the
    /// caller's context, and seeds the callee's Addr constraints. Uses a
    /// worklist to propagate transitively (callees that call other functions).
    fn seed_call_site_contexts(&mut self) {
        // BFS worklist: (function, context) pairs to process
        let mut func_worklist: BTreeSet<(FunctionId, CallSiteContext)> = BTreeSet::new();

        // Start from all functions already initialized (all in empty context)
        for (fid, ctx) in &self.initialized_contexts.clone() {
            func_worklist.insert((*fid, ctx.clone()));
        }

        while let Some((func_id, caller_ctx)) = func_worklist.pop_first() {
            // Get call sites FROM this function
            let Some(site_indices) = self.calls_from.get(&func_id).cloned() else {
                continue;
            };

            for idx in site_indices {
                let callee = self.call_sites[idx].callee;
                let inst_id = self.call_sites[idx].inst_id;

                let callee_ctx = self.callee_context(&caller_ctx, inst_id, callee);
                let key = (callee, callee_ctx.clone());

                if !self.initialized_contexts.contains(&key) {
                    self.seed_function_in_context(callee, &callee_ctx);
                    // Also process this callee's call sites
                    func_worklist.insert(key);
                }
            }
        }
    }

    fn process_location(&mut self, loc_ctx: &(LocId, CallSiteContext)) {
        let loc_set = match self.loc_pts.get(loc_ctx) {
            Some(s) => s.clone(),
            None => return,
        };

        let (loc, ctx) = loc_ctx;

        // Collect updates first
        let mut value_updates: Vec<(CtxValue, P)> = Vec::new();
        for load in &self.constraints.load {
            let ptr_cv = CtxValue {
                value: load.src_ptr,
                ctx: ctx.clone(),
            };
            if let Some(ptr_pts) = self.cs_pts.get(&ptr_cv) {
                if ptr_pts.contains(*loc) {
                    let dst_cv = CtxValue {
                        value: load.dst,
                        ctx: ctx.clone(),
                    };
                    // Use the full load_from_location which checks both
                    // context-sensitive and global entries
                    let full_loc_set = self.load_from_location(*loc, ctx);
                    if full_loc_set.is_empty() {
                        value_updates.push((dst_cv, loc_set.clone()));
                    } else {
                        value_updates.push((dst_cv, full_loc_set));
                    }
                }
            }
        }

        // Apply updates
        for (dst_cv, locs) in value_updates {
            if self.union_into_value(&dst_cv, &locs) {
                self.worklist.insert(dst_cv);
            }
        }
    }

    fn union_into_value(&mut self, cv: &CtxValue, locs: &P) -> bool {
        self.value_contexts
            .entry(cv.value)
            .or_default()
            .insert(cv.ctx.clone());
        let entry = self
            .cs_pts
            .entry(cv.clone())
            .or_insert_with(|| self.template.clone_empty());
        entry.union(locs)
    }

    fn union_into_location(&mut self, loc_ctx: &(LocId, CallSiteContext), locs: &P) -> bool {
        let changed_cs = {
            let entry = self
                .loc_pts
                .entry(loc_ctx.clone())
                .or_insert_with(|| self.template.clone_empty());
            entry.union(locs)
        };
        // Only update the global (CI) mirror for NON-CLONED locations.
        // Cloned callee-local locs should NOT merge into the global mirror
        // because they have per-context precision via the clone mechanism.
        // Non-cloned locations (caller/global) need global visibility so that
        // stores from one context are visible to loads in another.
        if !self.cloned_locs.contains(&loc_ctx.0) {
            let changed_global = {
                let entry = self
                    .global_loc_pts
                    .entry(loc_ctx.0)
                    .or_insert_with(|| self.template.clone_empty());
                entry.union(locs)
            };
            if changed_global {
                self.global_loc_worklist.insert(loc_ctx.0);
            }
            return changed_cs || changed_global;
        }
        changed_cs
    }

    /// Load content from a location, checking context-sensitive and global entries.
    ///
    /// For **cloned** callee-local locations: uses ONLY the context-sensitive entry
    /// (precise per-context data; the clone provides isolation).
    /// For **non-cloned** locations (caller/global): unions the CS entry with
    /// the global mirror.  The CS entry has stores from this context, while the
    /// global mirror has stores from ALL contexts (including callees that store
    /// through parameter pointers).  Both must be visible so the caller can see
    /// callee-side effects.
    fn load_from_location(&self, loc: LocId, ctx: &CallSiteContext) -> P {
        if self.cloned_locs.contains(&loc) {
            // Cloned callee-local: CS entry is authoritative
            return self
                .loc_pts
                .get(&(loc, ctx.clone()))
                .cloned()
                .unwrap_or_else(|| self.template.clone_empty());
        }
        // Non-cloned: union CS entry with global mirror for cross-context
        // visibility.
        let mut result = self.template.clone_empty();
        if let Some(cs_pts) = self.loc_pts.get(&(loc, ctx.clone())) {
            result.union(cs_pts);
        }
        if let Some(global_pts) = self.global_loc_pts.get(&loc) {
            result.union(global_pts);
        }
        result
    }

    /// Process updates when a global (CI) location's content changes.
    ///
    /// Re-triggers Load constraints in contexts where there is NO existing
    /// context-sensitive entry for this location. This ensures cross-context
    /// visibility without interfering with per-context precision.
    fn process_global_location(&mut self, loc: LocId) {
        let loc_set = match self.global_loc_pts.get(&loc) {
            Some(s) => s.clone(),
            None => return,
        };

        let mut value_updates: Vec<(CtxValue, P)> = Vec::new();

        for load in &self.constraints.load {
            // Use `value_contexts` reverse index instead of scanning all
            // `cs_pts` entries. O(|contexts for src_ptr|) instead of
            // O(|all cs_pts entries|) per load constraint.
            let matching_cvs: Vec<CallSiteContext> = self
                .value_contexts
                .get(&load.src_ptr)
                .map(|ctxs| ctxs.iter().cloned().collect())
                .unwrap_or_default();

            for ctx in matching_cvs {
                // Only propagate global data if no CS entry exists for
                // this (loc, ctx). If CS data exists, it's authoritative.
                if let Some(cs_pts) = self.loc_pts.get(&(loc, ctx.clone())) {
                    if !cs_pts.is_empty() {
                        continue;
                    }
                }

                let ptr_cv = CtxValue {
                    value: load.src_ptr,
                    ctx: ctx.clone(),
                };
                if let Some(ptr_pts) = self.cs_pts.get(&ptr_cv) {
                    if ptr_pts.contains(loc) {
                        let dst_cv = CtxValue {
                            value: load.dst,
                            ctx,
                        };
                        value_updates.push((dst_cv, loc_set.clone()));
                    }
                }
            }
        }

        for (dst_cv, locs) in value_updates {
            if self.union_into_value(&dst_cv, &locs) {
                self.worklist.insert(dst_cv);
            }
        }
    }

    fn find_or_approximate_location(&self, obj: ObjId, path: &FieldPath) -> Option<LocId> {
        // Delegate to `LocationFactory::lookup_approx` which performs the same
        // 3-step fallback (exact → parent → base) using O(1) hash lookups
        // instead of 3 full linear scans over all locations.
        self.factory.lookup_approx(obj, path)
    }

    fn into_result(self) -> CsPtaResult {
        // Normalize generic points-to sets to BTreeSet<LocId> for stable API
        let normalized_cs_pts: BTreeMap<CtxValue, BTreeSet<LocId>> = self
            .cs_pts
            .into_iter()
            .map(|(cv, pts)| (cv, pts.to_btreeset()))
            .collect();

        // Build CI summary by unioning across all contexts per value
        let mut ci_summary: BTreeMap<ValueId, BTreeSet<LocId>> = BTreeMap::new();
        for (cv, locs) in &normalized_cs_pts {
            ci_summary.entry(cv.value).or_default().extend(locs);
        }

        let locations = self
            .factory
            .all_locations()
            .iter()
            .map(|(&id, loc)| (id, loc.clone()))
            .collect();

        let multiplicities = self.factory.multiplicities().clone();

        CsPtaResult {
            cs_pts: normalized_cs_pts,
            ci_summary,
            locations,
            multiplicities,
            diagnostics: self.diagnostics,
        }
    }
}

/// Compute which functions are in recursive SCCs on the call graph.
fn compute_scc_functions(callgraph: &CallGraph) -> BTreeSet<FunctionId> {
    use crate::callgraph::CallGraphNode;

    // Build a BTreeMap of successors for the call graph
    let mut func_succs: BTreeMap<FunctionId, BTreeSet<FunctionId>> = BTreeMap::new();
    let mut all_funcs: BTreeSet<FunctionId> = BTreeSet::new();

    for (src, dsts) in &callgraph.edges {
        if let CallGraphNode::Function(src_id) = src {
            all_funcs.insert(*src_id);
            let entry = func_succs.entry(*src_id).or_default();
            for dst in dsts {
                if let CallGraphNode::Function(dst_id) = dst {
                    all_funcs.insert(*dst_id);
                    entry.insert(*dst_id);
                }
            }
        }
    }

    // Compute SCCs
    let sccs = tarjan_scc(&all_funcs, &func_succs);

    // Functions in SCCs with size > 1, or self-recursive (size 1 with self-edge)
    let mut scc_functions = BTreeSet::new();
    for scc in &sccs {
        match scc.len().cmp(&1) {
            std::cmp::Ordering::Greater => {
                scc_functions.extend(scc);
            }
            std::cmp::Ordering::Equal => {
                let func = scc.iter().next().expect("SCC with len 1 has one element");
                // Check for self-loop
                if let Some(succs) = func_succs.get(func) {
                    if succs.contains(func) {
                        scc_functions.insert(*func);
                    }
                }
            }
            std::cmp::Ordering::Less => {}
        }
    }
    scc_functions
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction, Operation};
    use saf_core::ids::{BlockId, ModuleId};

    fn make_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    #[test]
    fn empty_module_produces_empty_result() {
        let module = make_module();
        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();
        let result = solve_context_sensitive(&module, &callgraph, &config);

        assert!(result.ci_summary_map().is_empty());
        assert_eq!(result.diagnostics().iterations, 1); // one iteration to check empty worklist
    }

    #[test]
    fn single_alloca_has_pts() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        let ptr = ValueId::derive(b"ptr");
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(ptr),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();
        let result = solve_context_sensitive(&module, &callgraph, &config);

        // CI summary should have the pointer
        let pts = result.points_to_any(ptr);
        assert!(!pts.is_empty(), "ptr should have a points-to set");
    }

    #[test]
    fn context_separation_two_call_sites() {
        // Build: main calls wrapper(x) at site1 and wrapper(y) at site2
        // wrapper(p) returns p. With k=1, site1 and site2 have different contexts.
        let mut module = make_module();

        // fn wrapper(param) { ret param }
        let wrapper_id = FunctionId::derive(b"wrapper");
        let param_id = ValueId::derive(b"wp");
        let mut wrapper = AirFunction::new(wrapper_id, "wrapper");
        wrapper.params.push(AirParam::new(param_id, 0));
        let mut wb = AirBlock::new(BlockId::derive(b"w_entry"));
        wb.instructions.push(
            Instruction::new(InstId::derive(b"w_ret"), Operation::Ret)
                .with_operands(vec![param_id]),
        );
        wrapper.blocks.push(wb);
        module.functions.push(wrapper);

        // fn main() { x = alloca; y = alloca; r1 = call wrapper(x); r2 = call wrapper(y) }
        let main_id = FunctionId::derive(b"main");
        let mut main_fn = AirFunction::new(main_id, "main");
        let x = ValueId::derive(b"x");
        let y = ValueId::derive(b"y");
        let r1 = ValueId::derive(b"r1");
        let r2 = ValueId::derive(b"r2");

        let mut mb = AirBlock::new(BlockId::derive(b"m_entry"));
        mb.instructions.push(
            Instruction::new(
                InstId::derive(b"ax"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(x),
        );
        mb.instructions.push(
            Instruction::new(
                InstId::derive(b"ay"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(y),
        );
        mb.instructions.push(
            Instruction::new(
                InstId::derive(b"call1"),
                Operation::CallDirect { callee: wrapper_id },
            )
            .with_operands(vec![x])
            .with_dst(r1),
        );
        mb.instructions.push(
            Instruction::new(
                InstId::derive(b"call2"),
                Operation::CallDirect { callee: wrapper_id },
            )
            .with_operands(vec![y])
            .with_dst(r2),
        );
        mb.instructions
            .push(Instruction::new(InstId::derive(b"m_ret"), Operation::Ret));
        main_fn.blocks.push(mb);
        module.functions.push(main_fn);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig {
            k: 1,
            ..CsPtaConfig::default()
        };
        let result = solve_context_sensitive(&module, &callgraph, &config);

        // CI summary: r1 and r2 should each have some points-to set
        let r1_pts = result.points_to_any(r1);
        let r2_pts = result.points_to_any(r2);
        assert!(!r1_pts.is_empty(), "r1 should have points-to");
        assert!(!r2_pts.is_empty(), "r2 should have points-to");

        // x and y should point to different locations
        let x_pts = result.points_to_any(x);
        let y_pts = result.points_to_any(y);
        assert_ne!(x_pts, y_pts, "x and y should point to different allocas");

        // With CS: check alias (CI summary may still merge, but CS should separate)
        let _alias = result.may_alias_any(r1, r2);
    }

    #[test]
    fn scc_collapse_recursive() {
        // Build: fn rec() { call rec(); } — self-recursive
        let mut module = make_module();
        let rec_id = FunctionId::derive(b"rec");
        let mut rec_fn = AirFunction::new(rec_id, "rec");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(Instruction::new(
            InstId::derive(b"self_call"),
            Operation::CallDirect { callee: rec_id },
        ));
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        rec_fn.blocks.push(block);
        module.functions.push(rec_fn);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();
        let result = solve_context_sensitive(&module, &callgraph, &config);

        // Should terminate (SCC collapse prevents infinite contexts)
        assert!(
            !result.diagnostics().iteration_limit_hit,
            "should not hit iteration limit for recursive function"
        );
        assert!(
            result.diagnostics().scc_function_count > 0,
            "should detect SCC function"
        );
    }

    #[test]
    fn diagnostics_populated() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"f"), "f");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(InstId::derive(b"a"), Operation::Alloca { size_bytes: None })
                .with_dst(ValueId::derive(b"p")),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();
        let result = solve_context_sensitive(&module, &callgraph, &config);

        let diag = result.diagnostics();
        assert!(diag.iterations > 0);
        assert!(diag.constraint_count > 0);
        assert!(diag.location_count > 0);
    }

    // Tests for different points-to set representations

    #[test]
    fn solve_with_bitvector_representation() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        let ptr = ValueId::derive(b"ptr");
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(ptr),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default().with_bitvector();
        let result = solve_context_sensitive(&module, &callgraph, &config);

        let pts = result.points_to_any(ptr);
        assert!(!pts.is_empty(), "ptr should have a points-to set");
    }

    #[test]
    fn solve_with_bdd_representation() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        let ptr = ValueId::derive(b"ptr");
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(ptr),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default().with_bdd();
        let result = solve_context_sensitive(&module, &callgraph, &config);

        let pts = result.points_to_any(ptr);
        assert!(!pts.is_empty(), "ptr should have a points-to set");
    }

    #[test]
    fn all_representations_produce_equivalent_results() {
        let mut module = make_module();
        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        let p = ValueId::derive(b"p");
        let q = ValueId::derive(b"q");

        // p = alloca
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca_p"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(p),
        );
        // q = alloca
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca_q"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(q),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));

        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);

        // Solve with each representation
        let result_btree =
            solve_context_sensitive(&module, &callgraph, &CsPtaConfig::default().with_btreeset());
        let result_bitvec = solve_context_sensitive(
            &module,
            &callgraph,
            &CsPtaConfig::default().with_bitvector(),
        );
        let result_bdd =
            solve_context_sensitive(&module, &callgraph, &CsPtaConfig::default().with_bdd());

        // All should produce the same CI summary
        assert_eq!(
            result_btree.ci_summary_map(),
            result_bitvec.ci_summary_map(),
            "BTree and BitVec should produce same CI summary"
        );
        assert_eq!(
            result_btree.ci_summary_map(),
            result_bdd.ci_summary_map(),
            "BTree and BDD should produce same CI summary"
        );
    }
}
