//! Bottom-up function summary generation through the call graph.
//!
//! Traverses the call graph in reverse topological order (leaves first),
//! generating [`FunctionSummary`] instances from per-function constraints.
//! SCCs (mutual recursion) are handled via fixpoint iteration with a
//! configurable iteration limit.
//!
//! # Algorithm
//!
//! 1. Build a `FunctionId`-level adjacency map from the [`CallGraph`]
//! 2. Compute SCCs via Tarjan's algorithm (leaf SCCs first)
//! 3. For each SCC in bottom-up order:
//!    - **Singleton (no self-loop):** extract constraints, generate summary
//!    - **Multi-node or self-loop:** iterate to fixpoint (max `scc_max_iters`)
//! 4. Return a populated [`SummaryRegistry`]

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{FunctionId, ValueId};
use saf_core::summary::{
    AccessPath, AllocationEffect, CalleeRef, FunctionSummary, MemoryEffect, ReturnEffect,
    SummaryPrecision, SummarySource,
};
use saf_core::summary_registry::SummaryRegistry;

use crate::callgraph::{CallGraph, CallGraphNode};
use crate::graph_algo::{Successors, tarjan_scc};

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for bottom-up summary generation.
#[derive(Debug, Clone)]
pub struct SummaryGenConfig {
    /// Maximum access path depth in generated summaries.
    pub depth_k: u32,
    /// Maximum fixpoint iterations for SCC resolution.
    pub scc_max_iters: u32,
    /// Precision label for generated summaries.
    pub precision: SummaryPrecision,
}

impl Default for SummaryGenConfig {
    fn default() -> Self {
        Self::best_effort()
    }
}

impl SummaryGenConfig {
    /// Sound over-approximation mode.
    ///
    /// Uses deeper access paths (`depth_k=5`), more SCC iterations (`10`),
    /// and treats unknown callees conservatively (may alias anything).
    #[must_use]
    pub fn sound() -> Self {
        Self {
            depth_k: 5,
            scc_max_iters: 10,
            precision: SummaryPrecision::Sound,
        }
    }

    /// Best-effort mode (faster, may miss behaviors).
    ///
    /// Uses shallower access paths (`depth_k=3`), fewer SCC iterations (`5`),
    /// and treats unknown callees as no-ops.
    #[must_use]
    pub fn best_effort() -> Self {
        Self {
            depth_k: 3,
            scc_max_iters: 5,
            precision: SummaryPrecision::BestEffort,
        }
    }
}

// =============================================================================
// Result types
// =============================================================================

/// Result of summary generation for a set of functions.
#[derive(Debug, Clone)]
pub struct SummaryGenResult {
    /// The populated summary registry.
    pub registry: SummaryRegistry,
    /// Number of SCCs processed.
    pub scc_count: usize,
    /// Number of functions that required fixpoint iteration (in non-trivial SCCs).
    pub fixpoint_functions: usize,
    /// Functions that hit the iteration limit without converging.
    pub unconverged: BTreeSet<FunctionId>,
}

/// Result of a cascade update triggered by a function change.
#[derive(Debug, Clone)]
pub struct CascadeResult {
    /// Functions whose summaries were recomputed.
    pub recomputed: BTreeSet<FunctionId>,
    /// Functions where the summary actually changed (subset of recomputed).
    pub changed: BTreeSet<FunctionId>,
    /// Whether the cascade was cut short (summary unchanged, no further propagation).
    pub stopped_early: bool,
}

// =============================================================================
// FunctionId-level call graph adapter
// =============================================================================

/// A `FunctionId`-level adjacency map extracted from a [`CallGraph`].
///
/// This strips away `CallGraphNode` variants (External, IndirectPlaceholder)
/// and retains only defined `Function` nodes, providing a clean graph for
/// SCC computation and bottom-up traversal.
#[derive(Debug, Clone)]
struct FunctionCallGraph {
    /// Forward edges: caller -> set of callees.
    edges: BTreeMap<FunctionId, BTreeSet<FunctionId>>,
    /// All function IDs in the graph (defined functions only).
    functions: BTreeSet<FunctionId>,
}

impl Successors<FunctionId> for FunctionCallGraph {
    fn successors(&self, node: &FunctionId) -> Option<&BTreeSet<FunctionId>> {
        self.edges.get(node)
    }
}

impl FunctionCallGraph {
    /// Extract a `FunctionId`-level call graph from a [`CallGraph`].
    ///
    /// Only includes defined functions (not externals or indirect placeholders).
    fn from_call_graph(cg: &CallGraph) -> Self {
        let mut edges: BTreeMap<FunctionId, BTreeSet<FunctionId>> = BTreeMap::new();
        let mut functions = BTreeSet::new();

        // Collect all defined function IDs
        for node in &cg.nodes {
            if let CallGraphNode::Function(fid) = node {
                functions.insert(*fid);
                edges.entry(*fid).or_default();
            }
        }

        // Build edges between defined functions only
        for (caller, callees) in &cg.edges {
            let CallGraphNode::Function(caller_id) = caller else {
                continue;
            };
            if !functions.contains(caller_id) {
                continue;
            }
            for callee in callees {
                if let Some(callee_id) = callee.function_id() {
                    if functions.contains(&callee_id) {
                        edges.entry(*caller_id).or_default().insert(callee_id);
                    }
                }
            }
        }

        Self { edges, functions }
    }

    /// Get the callers of a function (reverse lookup).
    fn callers_of(&self, target: &FunctionId) -> BTreeSet<FunctionId> {
        let mut callers = BTreeSet::new();
        for (caller, callees) in &self.edges {
            if callees.contains(target) {
                callers.insert(*caller);
            }
        }
        callers
    }
}

// =============================================================================
// Summary generation from constraints
// =============================================================================

/// Generate a [`FunctionSummary`] for a single function from its parameter
/// and return value information.
///
/// This is a lightweight summary that captures:
/// - Which parameters are read/written (based on store/load through params)
/// - Return value aliasing (if return copies a parameter)
/// - Allocation effects (if return is freshly allocated)
/// - Direct callees
///
/// Precision-dependent behavior:
/// - **Sound**: unknown callees conservatively read+write all parameters
/// - `BestEffort`: unknown callees are treated as identity (no-op)
///
/// The `param_values` map provides the `ValueId` for each parameter index,
/// and `return_values` provides the `ValueId`(s) for return values.
/// The `known_functions` set identifies functions with available summaries;
/// callees not in this set are "unknown."
fn generate_summary_for_function(
    function_id: FunctionId,
    param_values: &BTreeMap<u32, ValueId>,
    return_values: &BTreeSet<ValueId>,
    constraints: &FunctionConstraintInfo,
    callee_ids: &BTreeSet<FunctionId>,
    known_functions: &BTreeSet<FunctionId>,
    config: &SummaryGenConfig,
) -> FunctionSummary {
    let mut summary = FunctionSummary::default_for(function_id);
    summary.source = SummarySource::Analysis;
    summary.precision = config.precision;

    // Reverse map: ValueId -> param index
    let value_to_param: BTreeMap<ValueId, u32> =
        param_values.iter().map(|(&idx, &vid)| (vid, idx)).collect();

    // Memory effects: detect reads/writes through parameters
    for store in &constraints.stores {
        if let Some(&param_idx) = value_to_param.get(&store.dst_ptr) {
            let path = AccessPath::Deref(Box::new(AccessPath::Param(param_idx)));
            add_or_merge_memory_effect(&mut summary.memory_effects, path, false, true);
        }
    }

    for load in &constraints.loads {
        if let Some(&param_idx) = value_to_param.get(&load.src_ptr) {
            let path = AccessPath::Deref(Box::new(AccessPath::Param(param_idx)));
            add_or_merge_memory_effect(&mut summary.memory_effects, path, true, false);
        }
    }

    // Sound mode: unknown callees conservatively read+write all params
    if config.precision == SummaryPrecision::Sound {
        let has_unknown_callee = callee_ids.iter().any(|c| !known_functions.contains(c));
        if has_unknown_callee {
            for &param_idx in param_values.keys() {
                let path = AccessPath::Param(param_idx);
                add_or_merge_memory_effect(&mut summary.memory_effects, path, true, true);
            }
        }
    }
    // BestEffort: unknown callees are no-ops (identity) — no extra effects

    // Return effects: check if return value aliases a parameter
    for &ret_val in return_values {
        // Check copy constraints: ret = param
        for copy in &constraints.copies {
            if copy.dst == ret_val {
                if let Some(&param_idx) = value_to_param.get(&copy.src) {
                    summary.return_effects.push(ReturnEffect {
                        aliases: Some(AccessPath::Param(param_idx)),
                        fresh_allocation: false,
                    });
                }
            }
        }

        // Check addr constraints: ret gets address of a location (allocation)
        for addr in &constraints.addrs {
            if addr.ptr == ret_val {
                summary.allocation_effects.push(AllocationEffect {
                    target: AccessPath::Return,
                    heap: true,
                });
                summary.return_effects.push(ReturnEffect {
                    aliases: None,
                    fresh_allocation: true,
                });
                break; // One allocation effect is enough
            }
        }
    }

    // Callees
    for &callee_id in callee_ids {
        summary.callees.insert(CalleeRef::Direct(callee_id));
    }

    // Truncate access paths to depth limit
    truncate_summary_paths(&mut summary, config.depth_k);

    summary
}

/// Merge a read/write flag into an existing memory effect for the same path,
/// or add a new one.
fn add_or_merge_memory_effect(
    effects: &mut Vec<MemoryEffect>,
    path: AccessPath,
    reads: bool,
    writes: bool,
) {
    if let Some(existing) = effects.iter_mut().find(|e| e.path == path) {
        existing.reads |= reads;
        existing.writes |= writes;
    } else {
        effects.push(MemoryEffect {
            path,
            reads,
            writes,
        });
    }
}

/// Truncate all access paths in a summary to the configured depth limit.
fn truncate_summary_paths(summary: &mut FunctionSummary, depth_k: u32) {
    for effect in &mut summary.memory_effects {
        effect.path = effect.path.truncate(depth_k);
    }
    for effect in &mut summary.allocation_effects {
        effect.target = effect.target.truncate(depth_k);
    }
    for effect in &mut summary.return_effects {
        if let Some(ref path) = effect.aliases {
            effect.aliases = Some(path.truncate(depth_k));
        }
    }
    for tp in &mut summary.taint_propagation {
        tp.from = tp.from.truncate(depth_k);
        tp.to = tp.to.truncate(depth_k);
    }
}

// =============================================================================
// Constraint info (lightweight per-function constraint snapshot)
// =============================================================================

/// Lightweight per-function constraint information for summary generation.
///
/// This avoids depending on the full `ConstraintSet` type, making the
/// summary generation module testable without the PTA solver.
#[derive(Debug, Clone, Default)]
pub struct FunctionConstraintInfo {
    /// Address-of constraints (ptr gets address of loc).
    pub addrs: Vec<AddrInfo>,
    /// Copy constraints (dst = src).
    pub copies: Vec<CopyInfo>,
    /// Load constraints (dst = *src_ptr).
    pub loads: Vec<LoadInfo>,
    /// Store constraints (*dst_ptr = src).
    pub stores: Vec<StoreInfo>,
}

/// Simplified address-of constraint.
#[derive(Debug, Clone)]
pub struct AddrInfo {
    /// The pointer value.
    pub ptr: ValueId,
}

/// Simplified copy constraint.
#[derive(Debug, Clone)]
pub struct CopyInfo {
    /// Destination value.
    pub dst: ValueId,
    /// Source value.
    pub src: ValueId,
}

/// Simplified load constraint.
#[derive(Debug, Clone)]
pub struct LoadInfo {
    /// Destination value.
    pub dst: ValueId,
    /// Source pointer.
    pub src_ptr: ValueId,
}

/// Simplified store constraint.
#[derive(Debug, Clone)]
pub struct StoreInfo {
    /// Destination pointer.
    pub dst_ptr: ValueId,
    /// Source value.
    pub src: ValueId,
}

/// Per-function analysis input for summary generation.
#[derive(Debug, Clone)]
pub struct FunctionAnalysisInput {
    /// Function ID.
    pub function_id: FunctionId,
    /// Parameter index -> ValueId mapping.
    pub param_values: BTreeMap<u32, ValueId>,
    /// Return value IDs.
    pub return_values: BTreeSet<ValueId>,
    /// Constraint info extracted from the function.
    pub constraints: FunctionConstraintInfo,
    /// Direct callee function IDs.
    pub callee_ids: BTreeSet<FunctionId>,
}

// =============================================================================
// Core algorithm
// =============================================================================

/// Generate function summaries for all defined functions in a call graph
/// using bottom-up traversal.
///
/// The `inputs` map provides per-function analysis inputs (parameters,
/// constraints, callees). Functions not present in `inputs` are skipped.
///
/// Existing summaries in `base_registry` are used for callee lookups
/// (e.g., for external functions with YAML specs).
pub fn generate_summaries(
    call_graph: &CallGraph,
    inputs: &BTreeMap<FunctionId, FunctionAnalysisInput>,
    base_registry: &SummaryRegistry,
    config: &SummaryGenConfig,
) -> SummaryGenResult {
    let func_cg = FunctionCallGraph::from_call_graph(call_graph);
    let sccs = tarjan_scc(&func_cg.functions, &func_cg);

    let mut registry = SummaryRegistry::new();
    let mut fixpoint_functions = 0;
    let mut unconverged = BTreeSet::new();

    // Build the set of "known" functions: those in inputs + those in base_registry.
    // In Sound mode, callees not in this set trigger conservative effects.
    let mut known_functions: BTreeSet<FunctionId> = inputs.keys().copied().collect();
    for (&fid, _) in base_registry.iter() {
        known_functions.insert(fid);
    }

    // SCCs are returned in reverse topological order (leaf SCCs first),
    // which is exactly the bottom-up order we need.
    for scc in &sccs {
        let is_trivial = scc.len() == 1 && {
            let fid = scc.iter().next().expect("SCC is non-empty");
            !func_cg
                .edges
                .get(fid)
                .is_some_and(|callees| callees.contains(fid))
        };

        if is_trivial {
            // Single function, no self-recursion: generate once
            let fid = *scc.iter().next().expect("SCC is non-empty");
            if let Some(input) = inputs.get(&fid) {
                let summary = generate_summary_for_function(
                    fid,
                    &input.param_values,
                    &input.return_values,
                    &input.constraints,
                    &input.callee_ids,
                    &known_functions,
                    config,
                );
                registry.insert_computed(summary);
            }
        } else {
            // Non-trivial SCC: iterate to fixpoint
            fixpoint_functions += scc.len();

            // Initialize with empty summaries
            for &fid in scc {
                if inputs.contains_key(&fid) {
                    registry.insert_computed(FunctionSummary::default_for(fid));
                }
            }

            let mut converged = false;
            for iter_num in 0..config.scc_max_iters {
                let mut any_changed = false;

                for &fid in scc {
                    let Some(input) = inputs.get(&fid) else {
                        continue;
                    };

                    let new_summary = generate_summary_for_function(
                        fid,
                        &input.param_values,
                        &input.return_values,
                        &input.constraints,
                        &input.callee_ids,
                        &known_functions,
                        config,
                    );

                    // Check if summary changed
                    let changed = match registry.get(&fid) {
                        Some(old) => !summaries_equivalent(old, &new_summary),
                        None => true,
                    };

                    if changed {
                        any_changed = true;
                        let mut versioned = new_summary;
                        versioned.version = u64::from(iter_num) + 1;
                        registry.insert_computed(versioned);
                    }
                }

                if !any_changed {
                    converged = true;
                    break;
                }
            }

            if !converged {
                for &fid in scc {
                    if inputs.contains_key(&fid) {
                        unconverged.insert(fid);
                    }
                }
            }
        }
    }

    SummaryGenResult {
        registry,
        scc_count: sccs.len(),
        fixpoint_functions,
        unconverged,
    }
}

/// Check if two summaries have equivalent effects (ignoring version).
fn summaries_equivalent(a: &FunctionSummary, b: &FunctionSummary) -> bool {
    a.return_effects == b.return_effects
        && a.memory_effects == b.memory_effects
        && a.allocation_effects == b.allocation_effects
        && a.callees == b.callees
        && a.role == b.role
        && a.pure == b.pure
        && a.noreturn == b.noreturn
        && a.param_nullness == b.param_nullness
        && a.return_nullness == b.return_nullness
        && a.taint_propagation == b.taint_propagation
}

// =============================================================================
// Cascade update
// =============================================================================

/// Re-analyze a changed function and cascade updates to its callers.
///
/// Algorithm:
/// 1. Regenerate summary for `changed_function`
/// 2. Compare with previous summary
/// 3. If unchanged: stop (no cascade needed)
/// 4. If changed: find callers and re-analyze them recursively
///
/// The `inputs` map must contain entries for the changed function and
/// potentially its callers (for re-analysis).
pub fn cascade_summary_update(
    changed_function: FunctionId,
    call_graph: &CallGraph,
    inputs: &BTreeMap<FunctionId, FunctionAnalysisInput>,
    registry: &mut SummaryRegistry,
    config: &SummaryGenConfig,
) -> CascadeResult {
    let func_cg = FunctionCallGraph::from_call_graph(call_graph);
    let known_functions: BTreeSet<FunctionId> = inputs.keys().copied().collect();
    let mut recomputed = BTreeSet::new();
    let mut changed = BTreeSet::new();
    let mut worklist: Vec<FunctionId> = vec![changed_function];

    while let Some(fid) = worklist.pop() {
        if recomputed.contains(&fid) {
            continue;
        }
        recomputed.insert(fid);

        let Some(input) = inputs.get(&fid) else {
            continue;
        };

        let new_summary = generate_summary_for_function(
            fid,
            &input.param_values,
            &input.return_values,
            &input.constraints,
            &input.callee_ids,
            &known_functions,
            config,
        );

        let did_change = match registry.get(&fid) {
            Some(old) => !summaries_equivalent(old, &new_summary),
            None => true,
        };

        if did_change {
            changed.insert(fid);
            registry.insert_computed(new_summary);

            // Add callers to worklist
            let callers = func_cg.callers_of(&fid);
            for caller in callers {
                if !recomputed.contains(&caller) {
                    worklist.push(caller);
                }
            }
        }
        // If summary didn't change, stop propagating from this function
    }

    let stopped_early = changed.len() < recomputed.len();

    CascadeResult {
        recomputed,
        changed,
        stopped_early,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::FunctionId;

    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    /// Build a minimal CallGraph from a set of (caller, callee) pairs.
    fn make_call_graph(functions: &[FunctionId], edges: &[(FunctionId, FunctionId)]) -> CallGraph {
        let mut nodes = BTreeSet::new();
        let mut cg_edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>> = BTreeMap::new();
        let mut func_index = BTreeMap::new();

        for &f in functions {
            let node = CallGraphNode::Function(f);
            nodes.insert(node.clone());
            cg_edges.entry(node.clone()).or_default();
            func_index.insert(f, node);
        }

        for &(caller, callee) in edges {
            let caller_node = CallGraphNode::Function(caller);
            let callee_node = CallGraphNode::Function(callee);
            cg_edges.entry(caller_node).or_default().insert(callee_node);
        }

        // Build reverse edges
        let mut reverse_edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>> = BTreeMap::new();
        for (caller, callees) in &cg_edges {
            for callee in callees {
                reverse_edges
                    .entry(callee.clone())
                    .or_default()
                    .insert(caller.clone());
            }
        }

        CallGraph {
            nodes,
            edges: cg_edges,
            reverse_edges,
            call_sites: BTreeMap::new(),
            func_index,
        }
    }

    /// Create a simple input with one param and a return value.
    fn make_input(function_id: FunctionId, callee_ids: &[FunctionId]) -> FunctionAnalysisInput {
        FunctionAnalysisInput {
            function_id,
            param_values: [(0, vid(function_id.raw() * 100 + 1))]
                .into_iter()
                .collect(),
            return_values: [vid(function_id.raw() * 100 + 99)].into_iter().collect(),
            constraints: FunctionConstraintInfo::default(),
            callee_ids: callee_ids.iter().copied().collect(),
        }
    }

    // -- Linear chain: main -> helper -> leaf --

    #[test]
    fn linear_chain_summaries_generated_bottom_up() {
        let main_id = fid(1);
        let helper_id = fid(2);
        let leaf_id = fid(3);

        let cg = make_call_graph(
            &[main_id, helper_id, leaf_id],
            &[(main_id, helper_id), (helper_id, leaf_id)],
        );

        let mut inputs = BTreeMap::new();
        inputs.insert(leaf_id, make_input(leaf_id, &[]));
        inputs.insert(helper_id, make_input(helper_id, &[leaf_id]));
        inputs.insert(main_id, make_input(main_id, &[helper_id]));

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        // All three functions should have summaries
        assert!(result.registry.get(&leaf_id).is_some());
        assert!(result.registry.get(&helper_id).is_some());
        assert!(result.registry.get(&main_id).is_some());

        // Leaf has no callees
        assert!(
            result
                .registry
                .get(&leaf_id)
                .expect("leaf")
                .callees
                .is_empty()
        );

        // Helper has leaf as callee
        assert!(
            result
                .registry
                .get(&helper_id)
                .expect("helper")
                .callees
                .contains(&CalleeRef::Direct(leaf_id))
        );

        // Main has helper as callee
        assert!(
            result
                .registry
                .get(&main_id)
                .expect("main")
                .callees
                .contains(&CalleeRef::Direct(helper_id))
        );

        assert_eq!(result.scc_count, 3);
        assert_eq!(result.fixpoint_functions, 0);
        assert!(result.unconverged.is_empty());
    }

    // -- SCC: a <-> b (mutual recursion) --

    #[test]
    fn scc_mutual_recursion_converges() {
        let a_id = fid(1);
        let b_id = fid(2);

        let cg = make_call_graph(&[a_id, b_id], &[(a_id, b_id), (b_id, a_id)]);

        let mut inputs = BTreeMap::new();
        inputs.insert(a_id, make_input(a_id, &[b_id]));
        inputs.insert(b_id, make_input(b_id, &[a_id]));

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        assert!(result.registry.get(&a_id).is_some());
        assert!(result.registry.get(&b_id).is_some());
        assert_eq!(result.scc_count, 1);
        assert_eq!(result.fixpoint_functions, 2);
        // Should converge since no constraints change between iterations
        assert!(result.unconverged.is_empty());
    }

    // -- Self-recursive function --

    #[test]
    fn self_recursive_function() {
        let fib_id = fid(1);

        let cg = make_call_graph(&[fib_id], &[(fib_id, fib_id)]);

        let mut inputs = BTreeMap::new();
        inputs.insert(fib_id, make_input(fib_id, &[fib_id]));

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        assert!(result.registry.get(&fib_id).is_some());
        // Self-loop means it's a non-trivial SCC
        assert_eq!(result.fixpoint_functions, 1);
    }

    // -- Constraint-based summary generation --

    #[test]
    fn store_through_param_generates_write_effect() {
        let f_id = fid(1);
        let param0_vid = vid(100);
        let src_vid = vid(101);

        let cg = make_call_graph(&[f_id], &[]);

        let input = FunctionAnalysisInput {
            function_id: f_id,
            param_values: [(0, param0_vid)].into_iter().collect(),
            return_values: BTreeSet::new(),
            constraints: FunctionConstraintInfo {
                stores: vec![StoreInfo {
                    dst_ptr: param0_vid,
                    src: src_vid,
                }],
                ..Default::default()
            },
            callee_ids: BTreeSet::new(),
        };

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input);

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert_eq!(summary.memory_effects.len(), 1);
        assert_eq!(
            summary.memory_effects[0].path,
            AccessPath::Deref(Box::new(AccessPath::Param(0)))
        );
        assert!(summary.memory_effects[0].writes);
        assert!(!summary.memory_effects[0].reads);
    }

    #[test]
    fn load_from_param_generates_read_effect() {
        let f_id = fid(1);
        let param0_vid = vid(100);
        let dst_vid = vid(101);

        let cg = make_call_graph(&[f_id], &[]);

        let input = FunctionAnalysisInput {
            function_id: f_id,
            param_values: [(0, param0_vid)].into_iter().collect(),
            return_values: BTreeSet::new(),
            constraints: FunctionConstraintInfo {
                loads: vec![LoadInfo {
                    dst: dst_vid,
                    src_ptr: param0_vid,
                }],
                ..Default::default()
            },
            callee_ids: BTreeSet::new(),
        };

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input);

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert_eq!(summary.memory_effects.len(), 1);
        assert!(summary.memory_effects[0].reads);
        assert!(!summary.memory_effects[0].writes);
    }

    #[test]
    fn return_copies_param_generates_alias_effect() {
        let f_id = fid(1);
        let param0_vid = vid(100);
        let ret_vid = vid(199);

        let cg = make_call_graph(&[f_id], &[]);

        let input = FunctionAnalysisInput {
            function_id: f_id,
            param_values: [(0, param0_vid)].into_iter().collect(),
            return_values: [ret_vid].into_iter().collect(),
            constraints: FunctionConstraintInfo {
                copies: vec![CopyInfo {
                    dst: ret_vid,
                    src: param0_vid,
                }],
                ..Default::default()
            },
            callee_ids: BTreeSet::new(),
        };

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input);

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert!(
            summary
                .return_effects
                .iter()
                .any(|e| e.aliases == Some(AccessPath::Param(0)) && !e.fresh_allocation)
        );
    }

    #[test]
    fn return_addr_generates_allocation_effect() {
        let f_id = fid(1);
        let ret_vid = vid(199);

        let cg = make_call_graph(&[f_id], &[]);

        let input = FunctionAnalysisInput {
            function_id: f_id,
            param_values: BTreeMap::new(),
            return_values: [ret_vid].into_iter().collect(),
            constraints: FunctionConstraintInfo {
                addrs: vec![AddrInfo { ptr: ret_vid }],
                ..Default::default()
            },
            callee_ids: BTreeSet::new(),
        };

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input);

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert!(
            summary
                .allocation_effects
                .iter()
                .any(|e| e.target == AccessPath::Return && e.heap)
        );
        assert!(summary.return_effects.iter().any(|e| e.fresh_allocation));
    }

    // -- Cascade tests --

    #[test]
    fn cascade_stops_when_summary_unchanged() {
        let main_id = fid(1);
        let helper_id = fid(2);
        let leaf_id = fid(3);

        let cg = make_call_graph(
            &[main_id, helper_id, leaf_id],
            &[(main_id, helper_id), (helper_id, leaf_id)],
        );

        let mut inputs = BTreeMap::new();
        inputs.insert(leaf_id, make_input(leaf_id, &[]));
        inputs.insert(helper_id, make_input(helper_id, &[leaf_id]));
        inputs.insert(main_id, make_input(main_id, &[helper_id]));

        // First: generate all summaries
        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let initial = generate_summaries(&cg, &inputs, &base, &config);
        let mut registry = initial.registry;

        // Cascade from leaf with same constraints -> summary unchanged
        let cascade = cascade_summary_update(leaf_id, &cg, &inputs, &mut registry, &config);

        // Leaf was recomputed but summary didn't change, so cascade stops
        assert!(cascade.recomputed.contains(&leaf_id));
        assert!(!cascade.changed.contains(&leaf_id));
        assert!(!cascade.recomputed.contains(&main_id));
        assert!(cascade.stopped_early);
    }

    #[test]
    fn cascade_propagates_when_summary_changes() {
        let main_id = fid(1);
        let helper_id = fid(2);

        let cg = make_call_graph(&[main_id, helper_id], &[(main_id, helper_id)]);

        let mut inputs = BTreeMap::new();
        inputs.insert(helper_id, make_input(helper_id, &[]));
        inputs.insert(main_id, make_input(main_id, &[helper_id]));

        let config = SummaryGenConfig::default();
        let base = SummaryRegistry::new();
        let initial = generate_summaries(&cg, &inputs, &base, &config);
        let mut registry = initial.registry;

        // Now change helper's constraints (add a store through param)
        let param0_vid = vid(helper_id.raw() * 100 + 1);
        inputs
            .get_mut(&helper_id)
            .expect("helper")
            .constraints
            .stores
            .push(StoreInfo {
                dst_ptr: param0_vid,
                src: vid(999),
            });

        let cascade = cascade_summary_update(helper_id, &cg, &inputs, &mut registry, &config);

        // Helper changed, main was recomputed
        assert!(cascade.changed.contains(&helper_id));
        assert!(cascade.recomputed.contains(&main_id));
    }

    // -- FunctionCallGraph extraction --

    #[test]
    fn function_call_graph_extracts_defined_functions_only() {
        let f1 = fid(1);
        let f2 = fid(2);

        // Build a CallGraph with one defined and one external function
        let mut nodes = BTreeSet::new();
        let f1_node = CallGraphNode::Function(f1);
        let f2_node = CallGraphNode::External {
            name: "ext".to_string(),
            func: f2,
        };
        nodes.insert(f1_node.clone());
        nodes.insert(f2_node.clone());

        let mut edges = BTreeMap::new();
        let mut f1_callees = BTreeSet::new();
        f1_callees.insert(f2_node.clone());
        edges.insert(f1_node.clone(), f1_callees);
        edges.insert(f2_node, BTreeSet::new());

        let cg = CallGraph {
            nodes,
            edges,
            reverse_edges: BTreeMap::new(),
            call_sites: BTreeMap::new(),
            func_index: BTreeMap::new(),
        };

        let func_cg = FunctionCallGraph::from_call_graph(&cg);

        // Only f1 (defined) should be in the graph, not f2 (external)
        assert!(func_cg.functions.contains(&f1));
        assert!(!func_cg.functions.contains(&f2));
        // f1 should have no callees (f2 is external, filtered out)
        assert!(func_cg.edges.get(&f1).expect("f1 edges").is_empty());
    }

    // -- Precision mode tests --

    #[test]
    fn sound_config_has_deeper_depth_and_more_iterations() {
        let sound = SummaryGenConfig::sound();
        let best = SummaryGenConfig::best_effort();

        assert_eq!(sound.depth_k, 5);
        assert_eq!(sound.scc_max_iters, 10);
        assert_eq!(sound.precision, SummaryPrecision::Sound);

        assert_eq!(best.depth_k, 3);
        assert_eq!(best.scc_max_iters, 5);
        assert_eq!(best.precision, SummaryPrecision::BestEffort);
    }

    #[test]
    fn sound_mode_unknown_callee_adds_conservative_effects() {
        let f_id = fid(1);
        let unknown_callee = fid(99);
        let param0_vid = vid(100);

        let cg = make_call_graph(&[f_id], &[]);

        let input = FunctionAnalysisInput {
            function_id: f_id,
            param_values: [(0, param0_vid)].into_iter().collect(),
            return_values: BTreeSet::new(),
            constraints: FunctionConstraintInfo::default(),
            callee_ids: [unknown_callee].into_iter().collect(),
        };

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input);

        // Sound mode: unknown callee -> conservative read+write on all params
        let config = SummaryGenConfig::sound();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        // Should have a memory effect on Param(0) with reads=true, writes=true
        assert!(
            summary
                .memory_effects
                .iter()
                .any(|e| e.path == AccessPath::Param(0) && e.reads && e.writes),
            "Sound mode should add conservative read+write for unknown callee"
        );
    }

    #[test]
    fn best_effort_mode_unknown_callee_no_extra_effects() {
        let f_id = fid(1);
        let unknown_callee = fid(99);
        let param0_vid = vid(100);

        let cg = make_call_graph(&[f_id], &[]);

        let input = FunctionAnalysisInput {
            function_id: f_id,
            param_values: [(0, param0_vid)].into_iter().collect(),
            return_values: BTreeSet::new(),
            constraints: FunctionConstraintInfo::default(),
            callee_ids: [unknown_callee].into_iter().collect(),
        };

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input);

        // BestEffort mode: unknown callee -> no extra effects
        let config = SummaryGenConfig::best_effort();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert!(
            summary.memory_effects.is_empty(),
            "BestEffort mode should not add effects for unknown callee"
        );
    }

    #[test]
    fn sound_mode_known_callee_no_conservative_effects() {
        let f_id = fid(1);
        let known_callee = fid(2);
        let param0_vid = vid(100);

        let cg = make_call_graph(&[f_id, known_callee], &[(f_id, known_callee)]);

        let input_f = FunctionAnalysisInput {
            function_id: f_id,
            param_values: [(0, param0_vid)].into_iter().collect(),
            return_values: BTreeSet::new(),
            constraints: FunctionConstraintInfo::default(),
            callee_ids: [known_callee].into_iter().collect(),
        };
        let input_callee = make_input(known_callee, &[]);

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, input_f);
        inputs.insert(known_callee, input_callee);

        // Sound mode: callee is known -> no conservative effects
        let config = SummaryGenConfig::sound();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert!(
            summary
                .memory_effects
                .iter()
                .all(|e| e.path != AccessPath::Param(0)),
            "Sound mode should not add conservative effects when callee is known"
        );
    }

    #[test]
    fn sound_mode_labels_precision_correctly() {
        let f_id = fid(1);
        let cg = make_call_graph(&[f_id], &[]);

        let mut inputs = BTreeMap::new();
        inputs.insert(f_id, make_input(f_id, &[]));

        let config = SummaryGenConfig::sound();
        let base = SummaryRegistry::new();
        let result = generate_summaries(&cg, &inputs, &base, &config);

        let summary = result.registry.get(&f_id).expect("summary");
        assert_eq!(summary.precision, SummaryPrecision::Sound);
    }
}
