//! Reachability solvers for checker framework.
//!
//! Four solver modes operate on the SVFG graph:
//!
//! - `may_reach`: Reports if source reaches sink on SOME path without sanitizer.
//!   Used for UAF, double-free, null-deref, stack-escape, uninit-use.
//!
//! - `may_reach_guarded`: Guard-aware variant of `may_reach` that accumulates
//!   path conditions during BFS and prunes infeasible paths via guard
//!   contradiction detection. Uses Pulse-style disjunct budget.
//!
//! - `must_not_reach`: Reports if source does NOT reach sanitizer on ALL paths
//!   before function exit. Used for file-descriptor-leak, lock-not-released.
//!
//! - `never_reach_sink`: Reports if source does NOT reach any sink on any path.
//!   Used for memory leak (SVF-style `NEVERFREE` — no deallocation found = leak).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[cfg(not(feature = "z3-solver"))]
use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};
use saf_core::ids::FunctionId;

use crate::svfg::{CallString, Svfg, SvfgEdgeKind, SvfgNodeId};

use super::finding::CheckerFinding;
use super::spec::CheckerSpec;

// ---------------------------------------------------------------------------
// Solver configuration
// ---------------------------------------------------------------------------

/// Configuration for reachability solvers.
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum BFS depth (default: 5000).
    pub max_depth: usize,
    /// Maximum CFL call-string depth for context-sensitive traversal.
    /// 0 = disabled (backward-compatible context-insensitive BFS).
    pub max_context_depth: usize,
    /// When set, only analyze source/sink nodes in these functions.
    /// `None` = analyze all functions (default, backward-compatible).
    pub reachable_functions: Option<BTreeSet<FunctionId>>,
    /// Map from SVFG node to containing function.
    /// Used by `must_not_reach` to scope exit-node matching to the
    /// source's own function, preventing false positives when
    /// interprocedural value-flow enters a callee whose `Ret`
    /// instruction is in the global `exit_nodes` set.
    pub node_to_func: Option<BTreeMap<SvfgNodeId, FunctionId>>,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_depth: 5000,
            max_context_depth: 3,
            reachable_functions: None,
            node_to_func: None,
        }
    }
}

/// Configuration for guard-aware reachability solvers.
#[derive(Debug, Clone)]
pub struct GuardedSolverConfig {
    /// Base solver config.
    pub base: SolverConfig,
    /// Maximum number of guard disjuncts tracked per node.
    /// When exceeded, oldest guards are dropped (under-approximate: may miss bugs, never adds FPs).
    /// Default: 20 (Infer/Pulse finding: 20 disjuncts finds 97% of bugs).
    pub max_disjuncts: usize,
}

impl Default for GuardedSolverConfig {
    fn default() -> Self {
        Self {
            base: SolverConfig::default(),
            max_disjuncts: 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Forward BFS result (enriched for partial leak detection)
// ---------------------------------------------------------------------------

/// Result of enriched forward BFS for partial leak detection.
///
/// Classifies each source into either NEVERFREE (no sink reachable) or
/// reachable (at least one sink found, with forward slice for Phase 2+3).
pub struct ForwardBfsResult {
    /// Sources that reached no sink — NEVERFREE findings.
    pub neverfree_findings: Vec<CheckerFinding>,
    /// Sources that reached at least one sink — candidates for partial leak check.
    pub reachable_sources: Vec<SourceReachability>,
}

/// A source that reached at least one sink during forward BFS.
pub struct SourceReachability {
    /// The source SVFG node.
    pub source: SvfgNodeId,
    /// All SVFG nodes visited during forward BFS (the forward slice).
    pub forward_slice: BTreeSet<SvfgNodeId>,
    /// All sink nodes reached during forward BFS.
    pub reached_sinks: BTreeSet<SvfgNodeId>,
}

// ---------------------------------------------------------------------------
// may_reach solver
// ---------------------------------------------------------------------------

/// Solve a `MayReach` checker: report if any source reaches any sink on
/// some path without passing through a sanitizer.
///
/// Algorithm:
/// 1. For each source node, CFL-aware BFS forward on SVFG.
/// 2. If we encounter a sanitizer node, stop exploring that branch.
/// 3. If we encounter a sink node, record a finding with the trace.
/// 4. Visited-set (keyed by `(node, context)`) prevents revisiting.
/// 5. Bounded by `config.max_depth`.
/// 6. CFL matching on `CallArg`/`Return` edges ensures only realizable
///    interprocedural paths are explored (when `max_context_depth > 0`).
// NOTE: CFL context tracking adds a CallString to each BFS state.
// When max_context_depth == 0, CFL is disabled for backward compatibility.
#[allow(clippy::too_many_lines)]
pub fn may_reach(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    sink_nodes: &BTreeSet<SvfgNodeId>,
    sanitizer_nodes: &BTreeSet<SvfgNodeId>,
    config: &SolverConfig,
) -> Vec<CheckerFinding> {
    let mut findings = Vec::new();
    let cfl_enabled = config.max_context_depth > 0;

    for &source in source_nodes {
        if !svfg.contains_node(source) {
            continue;
        }

        // BFS with parent tracking for trace reconstruction.
        // Context-aware: visited set and parent map are keyed by (node, context).
        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut explored_sinks: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut parent: BTreeMap<(SvfgNodeId, CallString), (SvfgNodeId, CallString)> =
            BTreeMap::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();

        let empty_ctx = CallString::empty();
        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx));

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.max_depth {
                continue;
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    // Compute new context based on edge kind
                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.max_context_depth) {
                            Some(c) => c,
                            None => continue, // Mismatched return — skip
                        }
                    } else {
                        ctx.clone()
                    };

                    let is_sink = sink_nodes.contains(&target) && target != source;

                    if is_sink {
                        if !explored_sinks.insert((target, new_ctx.clone())) {
                            continue;
                        }
                    } else if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    parent.insert((target, new_ctx.clone()), (node, ctx.clone()));

                    if sanitizer_nodes.contains(&target) {
                        continue;
                    }

                    if is_sink {
                        let trace = reconstruct_trace_ctx(&parent, source, target, &new_ctx);
                        findings.push(CheckerFinding {
                            checker_name: spec.name.clone(),
                            severity: spec.severity,
                            source_node: source,
                            sink_node: target,
                            trace,
                            cwe: spec.cwe,
                            message: format!(
                                "{}: {} (source → sink reachable on SVFG)",
                                spec.name, spec.description
                            ),
                            sink_traces: vec![],
                            source_kind: super::finding::NullSourceKind::default(),
                        });
                    }

                    queue.push_back((target, depth + 1, new_ctx));
                }
            }
        }
    }

    // Deduplicate findings by (source, sink) pair
    findings.sort_by(|a, b| {
        a.source_node
            .cmp(&b.source_node)
            .then(a.sink_node.cmp(&b.sink_node))
    });
    findings.dedup_by(|a, b| a.source_node == b.source_node && a.sink_node == b.sink_node);

    findings
}

// ---------------------------------------------------------------------------
// may_reach_guarded solver
// ---------------------------------------------------------------------------

/// Guard-aware `may_reach`: accumulates path conditions during BFS,
/// prunes infeasible paths via guard contradiction detection.
///
/// Uses Pulse-style disjunct budget: each node tracks up to
/// `max_disjuncts` separate guard conditions. When exceeded, the
/// excess guards are dropped (under-approximate: may miss bugs,
/// never adds false positives).
///
/// `dead_blocks` contains `BlockId`s identified as unreachable by SCCP.
/// `block_of` maps `ValueId` to the `BlockId` containing it.
// NOTE: This function implements a guard-aware BFS reachability checker
// as a single cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn may_reach_guarded(
    svfg: &crate::svfg::Svfg,
    spec: &super::spec::CheckerSpec,
    source_nodes: &[crate::svfg::SvfgNodeId],
    sink_nodes: &BTreeSet<crate::svfg::SvfgNodeId>,
    sanitizer_nodes: &BTreeSet<crate::svfg::SvfgNodeId>,
    config: &GuardedSolverConfig,
    dead_blocks: &BTreeSet<saf_core::ids::BlockId>,
    block_of: &BTreeMap<saf_core::ids::ValueId, saf_core::ids::BlockId>,
) -> Vec<super::finding::CheckerFinding> {
    use crate::guard::Guard;
    use crate::svfg::SvfgNodeId;

    /// Check whether a node resides in a dead block (SCCP-unreachable).
    /// Only `Value` nodes can be checked; `MemPhi` nodes have no block.
    fn is_in_dead_block(
        node: SvfgNodeId,
        dead_blocks: &BTreeSet<saf_core::ids::BlockId>,
        block_of: &BTreeMap<saf_core::ids::ValueId, saf_core::ids::BlockId>,
    ) -> bool {
        if let SvfgNodeId::Value(vid) = node {
            if let Some(blk) = block_of.get(&vid) {
                return dead_blocks.contains(blk);
            }
        }
        false
    }

    let mut findings = Vec::new();
    let cfl_enabled = config.base.max_context_depth > 0;

    for &source in source_nodes {
        if !svfg.contains_node(source) {
            continue;
        }

        // Skip sources in dead blocks
        if is_in_dead_block(source, dead_blocks, block_of) {
            continue;
        }

        // BFS with guard accumulation, CFL context, and parent tracking.
        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut explored_sinks: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut parent: BTreeMap<(SvfgNodeId, CallString), (SvfgNodeId, CallString)> =
            BTreeMap::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();
        // Guards accumulated along the path to each (node, context).
        let mut node_guards: BTreeMap<(SvfgNodeId, CallString), Vec<Guard>> = BTreeMap::new();

        let empty_ctx = CallString::empty();
        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx.clone()));
        node_guards.insert((source, empty_ctx), Vec::new());

        // Maximum guard list length before truncation.
        let guard_budget = config.max_disjuncts.saturating_mul(2);

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.base.max_depth {
                continue;
            }

            let current_guards = node_guards
                .get(&(node, ctx.clone()))
                .cloned()
                .unwrap_or_default();

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    // Skip targets in dead blocks
                    if is_in_dead_block(target, dead_blocks, block_of) {
                        continue;
                    }

                    // Compute new context based on edge kind
                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.base.max_context_depth) {
                            Some(c) => c,
                            None => continue,
                        }
                    } else {
                        ctx.clone()
                    };

                    // Accumulate guards from this edge
                    let mut accumulated = current_guards.clone();
                    if let Some(edge_guards) = svfg.edge_guard(node, target) {
                        accumulated.extend_from_slice(edge_guards);
                    }

                    // Enforce disjunct budget: drop oldest guards if over budget
                    if accumulated.len() > guard_budget {
                        let excess = accumulated.len() - config.max_disjuncts;
                        accumulated.drain(..excess);
                    }

                    // Prune infeasible paths: contradictory guards mean this
                    // path is impossible.
                    if has_contradictory_guards(&accumulated) {
                        continue;
                    }

                    let is_sink = sink_nodes.contains(&target) && target != source;

                    if is_sink {
                        if !explored_sinks.insert((target, new_ctx.clone())) {
                            continue;
                        }
                    } else if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    parent.insert((target, new_ctx.clone()), (node, ctx.clone()));
                    node_guards.insert((target, new_ctx.clone()), accumulated);

                    // Skip sanitizer nodes -- prune this path
                    if sanitizer_nodes.contains(&target) {
                        continue;
                    }

                    // Check if we reached a sink
                    if is_sink {
                        let trace = reconstruct_trace_ctx(&parent, source, target, &new_ctx);
                        findings.push(CheckerFinding {
                            checker_name: spec.name.clone(),
                            severity: spec.severity,
                            source_node: source,
                            sink_node: target,
                            trace,
                            cwe: spec.cwe,
                            message: format!(
                                "{}: {} (source → sink reachable on SVFG, guard-aware)",
                                spec.name, spec.description
                            ),
                            sink_traces: vec![],
                            source_kind: super::finding::NullSourceKind::default(),
                        });
                    }

                    queue.push_back((target, depth + 1, new_ctx));
                }
            }
        }
    }

    // Deduplicate findings by (source, sink) pair
    findings.sort_by(|a, b| {
        a.source_node
            .cmp(&b.source_node)
            .then(a.sink_node.cmp(&b.sink_node))
    });
    findings.dedup_by(|a, b| a.source_node == b.source_node && a.sink_node == b.sink_node);

    findings
}

/// Check if a guard list contains a contradictory pair
/// (same condition + same block, opposite `branch_taken`).
///
/// Uses a `BTreeSet` for O(n log n) detection instead of O(n^2) pairwise scan.
fn has_contradictory_guards(guards: &[crate::guard::Guard]) -> bool {
    use std::collections::BTreeSet;
    // Store (condition, block, branch_taken) tuples already seen
    let mut seen = BTreeSet::new();
    for g in guards {
        // Check if the opposite branch_taken was already seen
        if seen.contains(&(g.condition, g.block, !g.branch_taken)) {
            return true;
        }
        seen.insert((g.condition, g.block, g.branch_taken));
    }
    false
}

// ---------------------------------------------------------------------------
// must_not_reach solver
// ---------------------------------------------------------------------------

/// Solve a `MustNotReach` checker: report if any source does NOT reach
/// a sanitizer on all paths before function exit.
///
/// Algorithm:
/// 1. For each source node, CFL-aware BFS forward on SVFG.
/// 2. Track whether we encounter any exit node that is NOT preceded by a sanitizer.
/// 3. If any path from source reaches a function exit without passing through
///    a sanitizer, report a finding.
// NOTE: This function implements a BFS-based reachability checker as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
pub fn must_not_reach(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    exit_nodes: &BTreeSet<SvfgNodeId>,
    sanitizer_nodes: &BTreeSet<SvfgNodeId>,
    config: &SolverConfig,
) -> Vec<CheckerFinding> {
    let mut findings = Vec::new();
    let cfl_enabled = config.max_context_depth > 0;

    for &source in source_nodes {
        if !svfg.contains_node(source) {
            continue;
        }

        // If the source itself is a sanitizer (e.g., same SSA value is both
        // malloc return and free argument), the allocation is directly freed.
        // BFS from source would see unsanitized exits as false positives
        // (the value flows to exit AFTER being freed — use-after-free, not
        // a leak). Skip these. Note: this cannot cause partial-leak FNs
        // because the SVFG has no edges for control paths where a value
        // exists but has no value flow (no store/load/call/return).
        if sanitizer_nodes.contains(&source) {
            continue;
        }

        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut parent: BTreeMap<(SvfgNodeId, CallString), (SvfgNodeId, CallString)> =
            BTreeMap::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();
        let mut unsanitized_exits: Vec<SvfgNodeId> = Vec::new();

        let empty_ctx = CallString::empty();
        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx));

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.max_depth {
                continue;
            }

            // Check if this is an exit node — if so, we may have reached
            // exit without sanitizer. But if the exit node has Return edges
            // (the value is being returned to a caller), it's an ownership
            // transfer, not a leak. Only terminal exits (no Return edges,
            // e.g., main's return) are true leak points.
            if exit_nodes.contains(&node) && node != source {
                let has_return_to_caller = config.node_to_func.is_some()
                    && svfg.successors_of(node).is_some_and(|succs| {
                        succs
                            .iter()
                            .any(|(kind, _)| matches!(kind, SvfgEdgeKind::Return { .. }))
                    });
                if !has_return_to_caller {
                    // Terminal exit — no caller to return to. Report leak.
                    unsanitized_exits.push(node);
                    continue;
                }
                // Value is returned to caller — continue BFS to find
                // sanitizer there (ownership transferred, not leaked).
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.max_context_depth) {
                            Some(c) => c,
                            None => continue,
                        }
                    } else {
                        ctx.clone()
                    };

                    if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    parent.insert((target, new_ctx.clone()), (node, ctx.clone()));

                    if sanitizer_nodes.contains(&target) {
                        continue;
                    }

                    queue.push_back((target, depth + 1, new_ctx));
                }
            }
        }

        // Extract just the nodes from visited for the reporting logic
        let visited_nodes: BTreeSet<SvfgNodeId> = visited.iter().map(|(n, _)| *n).collect();

        if !unsanitized_exits.is_empty() {
            for exit in &unsanitized_exits {
                // Find any context for this exit to reconstruct trace
                let exit_ctx = visited
                    .iter()
                    .find(|(n, _)| n == exit)
                    .map(|(_, c)| c.clone())
                    .unwrap_or_default();
                let trace = reconstruct_trace_ctx(&parent, source, *exit, &exit_ctx);
                findings.push(CheckerFinding {
                    checker_name: spec.name.clone(),
                    severity: spec.severity,
                    source_node: source,
                    sink_node: *exit,
                    trace,
                    cwe: spec.cwe,
                    message: format!(
                        "{}: {} (source reaches function exit without sanitizer)",
                        spec.name, spec.description
                    ),
                    sink_traces: vec![],
                    source_kind: super::finding::NullSourceKind::default(),
                });
            }
        } else if sanitizer_nodes.is_empty()
            || !visited_nodes.iter().any(|n| sanitizer_nodes.contains(n))
        {
            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source,
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (no sanitizer reachable from source)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        } else if unsanitized_exits.is_empty()
            && !exit_nodes.is_empty()
            && visited_nodes.iter().any(|n| sanitizer_nodes.contains(n))
            // Skip dead-end heuristic when exit scoping is active:
            // interprocedural BFS visits nodes in callees whose dead-end
            // successors are outside the source function. These are normal
            // call returns, not real leaks.
            && config.node_to_func.is_none()
        {
            let has_unsanitized_dead_end = visited_nodes.iter().any(|n| {
                if *n == source || sanitizer_nodes.contains(n) {
                    return false;
                }
                match svfg.successors_of(*n) {
                    None => true,
                    Some(succs) if succs.is_empty() => true,
                    Some(succs) => succs.iter().all(|(_, tgt)| visited_nodes.contains(tgt)),
                }
            });
            if has_unsanitized_dead_end {
                findings.push(CheckerFinding {
                    checker_name: spec.name.clone(),
                    severity: spec.severity,
                    source_node: source,
                    sink_node: source,
                    trace: vec![source],
                    cwe: spec.cwe,
                    message: format!(
                        "{}: {} (sanitizer found but exit unreachable — potential partial leak)",
                        spec.name, spec.description
                    ),
                    sink_traces: vec![],
                    source_kind: super::finding::NullSourceKind::default(),
                });
            }
        }
    }

    // Deduplicate
    findings.sort_by(|a, b| {
        a.source_node
            .cmp(&b.source_node)
            .then(a.sink_node.cmp(&b.sink_node))
    });
    findings.dedup_by(|a, b| a.source_node == b.source_node && a.sink_node == b.sink_node);

    findings
}

// ---------------------------------------------------------------------------
// multi_reach solver
// ---------------------------------------------------------------------------

/// Solve a `MultiReach` checker: report if any source reaches 2+ distinct
/// sink nodes. Used for double-free: allocation reaching multiple deallocations.
///
/// Algorithm:
/// 1. For each source node, BFS forward on SVFG.
/// 2. Collect all sink nodes reachable from this source.
/// 3. If 2+ distinct sinks are reached, report a finding with traces to the first two.
/// 4. Bounded by `config.max_depth`.
pub fn multi_reach(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    sink_nodes: &BTreeSet<SvfgNodeId>,
    config: &SolverConfig,
) -> Vec<CheckerFinding> {
    let mut findings = Vec::new();
    let cfl_enabled = config.max_context_depth > 0;

    for &source in source_nodes {
        if !svfg.contains_node(source) {
            continue;
        }

        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut explored_sinks: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut parent: BTreeMap<(SvfgNodeId, CallString), (SvfgNodeId, CallString)> =
            BTreeMap::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();
        let mut reached_sinks: Vec<SvfgNodeId> = Vec::new();

        let empty_ctx = CallString::empty();

        // If the source node itself is a sink, count it immediately.
        if sink_nodes.contains(&source) {
            reached_sinks.push(source);
            explored_sinks.insert((source, empty_ctx.clone()));
        }

        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx));

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.max_depth {
                continue;
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.max_context_depth) {
                            Some(c) => c,
                            None => continue,
                        }
                    } else {
                        ctx.clone()
                    };

                    let is_sink = sink_nodes.contains(&target) && target != source;

                    if is_sink {
                        if !explored_sinks.insert((target, new_ctx.clone())) {
                            continue;
                        }
                    } else if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    parent.insert((target, new_ctx.clone()), (node, ctx.clone()));

                    if is_sink {
                        reached_sinks.push(target);
                    }

                    queue.push_back((target, depth + 1, new_ctx));
                }
            }
        }

        // Deduplicate reached sinks (same node can be reached in different contexts)
        reached_sinks.sort();
        reached_sinks.dedup();

        // If 2+ distinct sinks reached, report double-free
        if reached_sinks.len() >= 2 {
            let per_sink: Vec<(SvfgNodeId, Vec<SvfgNodeId>)> = reached_sinks
                .iter()
                .map(|&sink| {
                    // Find any context for this sink to reconstruct trace
                    let sink_ctx = explored_sinks
                        .iter()
                        .find(|(n, _)| *n == sink)
                        .map(|(_, c)| c.clone())
                        .or_else(|| {
                            visited
                                .iter()
                                .find(|(n, _)| *n == sink)
                                .map(|(_, c)| c.clone())
                        })
                        .unwrap_or_default();
                    (
                        sink,
                        reconstruct_trace_ctx(&parent, source, sink, &sink_ctx),
                    )
                })
                .collect();

            // Combined trace (first two sinks) for backward compatibility
            let sink1 = reached_sinks[0];
            let mut combined_trace = per_sink[0].1.clone();
            combined_trace.extend(per_sink[1].1.clone());

            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: sink1, // Report first sink as the primary
                trace: combined_trace,
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation reaches {} free calls)",
                    spec.name,
                    spec.description,
                    reached_sinks.len()
                ),
                sink_traces: per_sink,
                source_kind: super::finding::NullSourceKind::default(),
            });
        }
    }

    // Deduplicate by source (one finding per allocation)
    findings.sort_by_key(|f| f.source_node);
    findings.dedup_by_key(|f| f.source_node);

    findings
}

// ---------------------------------------------------------------------------
// never_reach_sink solver
// ---------------------------------------------------------------------------

/// Solve a `NeverReachSink` checker: report if a source does NOT reach any sink.
///
/// SVF-style `NEVERFREE` formulation: forward CFL-aware BFS from each source.
/// If any reachable node is a sink, the source is safe (freed). If BFS
/// exhausts without reaching any sink, report a finding (allocation never freed).
///
/// Sources not present in the SVFG are reported as definitive leaks: if an
/// allocation's return value has zero value flow (no edges at all), the pointer
/// can never reach a deallocator. This handles dead-end flows where the pointer
/// is never stored-and-loaded, never passed to a callee, and never returned.
pub fn never_reach_sink(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    sink_nodes: &BTreeSet<SvfgNodeId>,
    config: &SolverConfig,
) -> Vec<CheckerFinding> {
    let mut findings = Vec::new();
    let cfl_enabled = config.max_context_depth > 0;

    for &source in source_nodes {
        // If source is itself a sink, it is safe (e.g., malloc immediately freed).
        if sink_nodes.contains(&source) {
            continue;
        }

        // If the source has no SVFG representation, the allocation's return
        // value has zero value flow — it is never stored-and-loaded, never
        // passed to a callee, never returned. It can never reach any sink
        // (free), so report as a definitive leak.
        if !svfg.contains_node(source) {
            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source,
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation never freed)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
            continue;
        }

        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();
        let mut reached_any_sink = false;

        let empty_ctx = CallString::empty();
        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx));

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.max_depth {
                continue;
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.max_context_depth) {
                            Some(c) => c,
                            None => {
                                continue;
                            }
                        }
                    } else {
                        ctx.clone()
                    };

                    if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    if sink_nodes.contains(&target) {
                        reached_any_sink = true;
                        break;
                    }

                    queue.push_back((target, depth + 1, new_ctx));
                }
            }

            if reached_any_sink {
                break;
            }
        }

        if !reached_any_sink {
            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source, // no sink found — self-referential
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation never freed)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        }
    }

    // Deduplicate by source node
    findings.sort_by_key(|f| f.source_node);
    findings.dedup_by_key(|f| f.source_node);

    findings
}

// ---------------------------------------------------------------------------
// Enriched forward BFS (Phase 1 of partial leak detection)
// ---------------------------------------------------------------------------

/// Enriched forward BFS that collects forward slices and all reached sinks.
///
/// Like `never_reach_sink`, this performs CFL-aware forward BFS from each
/// source on the SVFG. Instead of just reporting whether any sink was
/// reached, it collects:
/// - The full forward slice (all visited SVFG nodes)
/// - All reached sink nodes (doesn't stop at the first)
///
/// Sources that reach no sink produce `neverfree_findings` (same as
/// `never_reach_sink`). Sources that reach at least one sink are returned
/// in `reachable_sources` for downstream partial-leak analysis (Phases 2+3).
#[allow(clippy::too_many_lines)]
pub fn forward_bfs_enriched(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    sink_nodes: &BTreeSet<SvfgNodeId>,
    config: &SolverConfig,
) -> ForwardBfsResult {
    let mut neverfree_findings = Vec::new();
    let mut reachable_sources = Vec::new();
    let cfl_enabled = config.max_context_depth > 0;

    for &source in source_nodes {
        // If source is itself a sink, it is safe (e.g., malloc immediately freed).
        if sink_nodes.contains(&source) {
            continue;
        }

        // If the source has no SVFG representation, the allocation's return
        // value has zero value flow — it can never reach any sink (free),
        // so report as a definitive leak.
        if !svfg.contains_node(source) {
            neverfree_findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source,
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation never freed)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
            continue;
        }

        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();
        let mut reached_sinks: BTreeSet<SvfgNodeId> = BTreeSet::new();
        let mut forward_slice: BTreeSet<SvfgNodeId> = BTreeSet::new();

        let empty_ctx = CallString::empty();
        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx));
        forward_slice.insert(source);

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.max_depth {
                continue;
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.max_context_depth) {
                            Some(c) => c,
                            None => {
                                continue;
                            }
                        }
                    } else {
                        ctx.clone()
                    };

                    if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    // If target is a sink, record it but don't explore past it
                    // (value is consumed by deallocation).
                    if sink_nodes.contains(&target) {
                        reached_sinks.insert(target);
                        continue;
                    }

                    forward_slice.insert(target);
                    queue.push_back((target, depth + 1, new_ctx));
                }
            }
        }

        if reached_sinks.is_empty() {
            neverfree_findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source, // no sink found — self-referential
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation never freed)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        } else {
            reachable_sources.push(SourceReachability {
                source,
                forward_slice,
                reached_sinks,
            });
        }
    }

    // Deduplicate neverfree findings by source node
    neverfree_findings.sort_by_key(|f| f.source_node);
    neverfree_findings.dedup_by_key(|f| f.source_node);

    ForwardBfsResult {
        neverfree_findings,
        reachable_sources,
    }
}

// ---------------------------------------------------------------------------
// SVFG backward slice (Phase 2)
// ---------------------------------------------------------------------------

/// Build a backward slice from sinks, intersected with the forward slice.
///
/// Phase 2 of SVF-style partial leak detection. BFS backward on SVFG from
/// each reached sink, only including nodes also in the forward slice. The
/// result contains exactly the nodes on actual source-to-sink paths.
pub fn backward_slice(
    svfg: &Svfg,
    forward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
) -> BTreeSet<SvfgNodeId> {
    let mut slice = BTreeSet::new();
    let mut queue = VecDeque::new();

    for &sink in reached_sinks {
        slice.insert(sink);
        queue.push_back(sink);
    }

    while let Some(node) = queue.pop_front() {
        if let Some(preds) = svfg.predecessors_of(node) {
            for (_, pred) in preds {
                let pred = *pred;
                if forward_slice.contains(&pred) && slice.insert(pred) {
                    queue.push_back(pred);
                }
            }
        }
    }

    slice
}

// ---------------------------------------------------------------------------
// SVFG all-path reachable solve (Phase 3)
// ---------------------------------------------------------------------------

/// Check whether ALL paths from source reach a sink (Z3 tautology check).
///
/// Phase 3 of SVF-style partial leak detection. Forward-propagates Z3
/// boolean conditions through backward-slice nodes. At sinks, collects the
/// disjunction of all conditions. If `NOT(disjunction)` is UNSAT, the
/// disjunction is a tautology — all paths are covered.
///
/// Returns `true` if all paths reach a sink (safe), `false` if partial leak.
// NOTE: This function implements the Z3 tautology check as a single cohesive
// unit (BFS propagation + sink condition collection + SAT check). Splitting
// would obscure the algorithm structure.
#[cfg(feature = "z3-solver")]
#[allow(clippy::too_many_lines)]
pub fn all_path_reachable_solve(
    svfg: &Svfg,
    source: SvfgNodeId,
    backward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
    value_index: &crate::guard::ValueLocationIndex,
    cfgs: &BTreeMap<FunctionId, crate::cfg::Cfg>,
) -> bool {
    use saf_core::ids::ValueId;

    let mut cond_counter: u32 = 0;
    let mut cond_names: BTreeMap<ValueId, String> = BTreeMap::new();
    let mut node_conds: BTreeMap<SvfgNodeId, z3::ast::Bool> = BTreeMap::new();

    // Source condition = TRUE
    node_conds.insert(source, z3::ast::Bool::from_bool(true));

    // Worklist-driven forward propagation
    let mut worklist: VecDeque<SvfgNodeId> = VecDeque::new();
    let mut in_worklist: BTreeSet<SvfgNodeId> = BTreeSet::new();
    worklist.push_back(source);
    in_worklist.insert(source);

    // Iteration cap for termination with cycles
    let max_iterations = backward_slice.len().saturating_mul(3).max(10);
    let mut iterations = 0;

    while let Some(node) = worklist.pop_front() {
        in_worklist.remove(&node);
        iterations += 1;
        if iterations > max_iterations {
            return false; // Conservative: can't converge → assume partial leak
        }

        let cur_cond = match node_conds.get(&node) {
            Some(c) => c.clone(),
            None => continue,
        };

        let Some(succs) = svfg.successors_of(node) else {
            continue;
        };

        for (ek, succ) in succs {
            let succ = *succ;
            let ek = *ek;
            if !backward_slice.contains(&succ) {
                continue;
            }

            let edge_guard = compute_edge_guard_z3(
                svfg,
                node,
                succ,
                &ek,
                value_index,
                cfgs,
                &mut cond_names,
                &mut cond_counter,
            );

            let propagated = z3::ast::Bool::and(&[&cur_cond, &edge_guard]);

            let new_cond = match node_conds.get(&succ) {
                Some(existing) => z3::ast::Bool::or(&[existing, &propagated]),
                None => propagated,
            };

            node_conds.insert(succ, new_cond);

            if !in_worklist.contains(&succ) {
                worklist.push_back(succ);
                in_worklist.insert(succ);
            }
        }
    }

    // Collect disjunction of conditions at all sinks, accounting for
    // conditional deallocation calls. For sinks that are formal parameters
    // (no block location), check if the actual free() call inside the
    // callee is conditional. The backward slice doesn't capture this
    // because it stops at the formal parameter node.
    let mut sink_conds: Vec<z3::ast::Bool> = Vec::new();
    for sink in reached_sinks {
        let Some(propagated) = node_conds.get(sink) else {
            continue;
        };

        // Only apply the deallocation guard to sinks without a block
        // location (formal parameters used as free args). For sinks with
        // a block location, the backward slice edge guards already capture
        // the conditional path.
        let sink_has_block = if let SvfgNodeId::Value(v) = sink {
            value_index.block_of(*v).is_some()
        } else {
            false
        };

        if sink_has_block {
            sink_conds.push(propagated.clone());
            continue;
        }

        // Sink is a formal parameter. Check its CallArg successors to find
        // the actual deallocation call instructions and their path guards.
        let mut dealloc_guards: Vec<z3::ast::Bool> = Vec::new();
        if let Some(succs) = svfg.successors_of(*sink) {
            for (ek, _) in succs {
                if let SvfgEdgeKind::CallArg { call_site } = ek {
                    if let Some((call_func, call_block)) = value_index.block_of_inst(*call_site) {
                        if let Some(cfg) = cfgs.get(&call_func) {
                            let guard = guards_to_z3(
                                &cfg_path_guards(cfg, cfg.entry, call_block, value_index),
                                &mut cond_names,
                                &mut cond_counter,
                            );
                            dealloc_guards.push(guard);
                        }
                    }
                }
            }
        }

        if dealloc_guards.is_empty() {
            sink_conds.push(propagated.clone());
        } else {
            let guard_refs: Vec<&z3::ast::Bool> = dealloc_guards.iter().collect();
            let dealloc_guard = z3::ast::Bool::or(&guard_refs);
            sink_conds.push(z3::ast::Bool::and(&[propagated, &dealloc_guard]));
        }
    }

    if sink_conds.is_empty() {
        return false;
    }

    let refs: Vec<&z3::ast::Bool> = sink_conds.iter().collect();
    let final_cond = z3::ast::Bool::or(&refs);

    // Tautology check: is NOT(final_cond) UNSAT?
    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    params.set_u32("timeout", 5000);
    solver.set_params(&params);
    solver.assert(final_cond.not());
    matches!(solver.check(), z3::SatResult::Unsat)
}

/// Compute the guard conjunction for an SVFG edge.
///
/// Three strategies:
/// 1. Pre-computed SVFG edge guards (from guard extraction pass).
/// 2. For `CallArg` edges: use the call_site `InstId` to find which block the
///    call instruction lives in, then compute the path guard from the function
///    entry to that block. This captures conditions under which interprocedural
///    calls (like conditional `free`) execute.
/// 3. For intraprocedural edges: map nodes to basic blocks and find gating
///    `CondBr` blocks on CFG paths between source and destination blocks.
fn compute_edge_guards(
    svfg: &Svfg,
    from: SvfgNodeId,
    to: SvfgNodeId,
    edge_kind: &SvfgEdgeKind,
    value_index: &crate::guard::ValueLocationIndex,
    cfgs: &BTreeMap<FunctionId, crate::cfg::Cfg>,
) -> Vec<crate::guard::Guard> {
    // Strategy 1: pre-computed SVFG edge guards
    if let Some(guards) = svfg.edge_guard(from, to) {
        if !guards.is_empty() {
            return guards.to_vec();
        }
    }

    // Strategy 2: CallArg edges — extract guard from call site's position in CFG
    if let SvfgEdgeKind::CallArg { call_site } = edge_kind {
        if let Some((call_func, call_block)) = value_index.block_of_inst(*call_site) {
            if let Some(cfg) = cfgs.get(&call_func) {
                // Guard = path condition from function entry to the call site block
                return cfg_path_guards(cfg, cfg.entry, call_block, value_index);
            }
        }
        return Vec::new();
    }

    // Strategy 3: intraprocedural on-the-fly CFG-based guard
    let (SvfgNodeId::Value(from_vid), SvfgNodeId::Value(to_vid)) = (from, to) else {
        return Vec::new();
    };

    let Some((from_func, from_block)) = value_index.block_of(from_vid) else {
        return Vec::new();
    };
    let Some((to_func, to_block)) = value_index.block_of(to_vid) else {
        return Vec::new();
    };

    // Different functions or same block → unconditional
    if from_func != to_func || from_block == to_block {
        return Vec::new();
    }

    let Some(cfg) = cfgs.get(&from_func) else {
        return Vec::new();
    };

    // Find gating CondBr blocks: blocks on paths from from_block to to_block
    // where only one branch can reach to_block.
    cfg_path_guards(cfg, from_block, to_block, value_index)
}

/// Compute the guards for a CFG path from `src` to `dst` within one function.
///
/// Finds all `CondBr` blocks on paths from `src` to `dst`. For each, checks
/// whether `dst` is reachable from only one branch. If so, that branch
/// condition is a gating guard. Returns the conjunction of all gating guards.
fn cfg_path_guards(
    cfg: &crate::cfg::Cfg,
    src: saf_core::ids::BlockId,
    dst: saf_core::ids::BlockId,
    value_index: &crate::guard::ValueLocationIndex,
) -> Vec<crate::guard::Guard> {
    use saf_core::ids::BlockId;

    // Step 1: forward reachability from src
    let fwd_reachable = cfg_reachable_from(cfg, src);

    // dst must be reachable from src
    if !fwd_reachable.contains(&dst) {
        return Vec::new();
    }

    // Step 2: backward reachability to dst (blocks that can reach dst)
    let bwd_reachable = cfg_reachable_backward(cfg, dst);

    // Step 3: blocks on paths from src to dst = fwd ∩ bwd
    let on_path: BTreeSet<BlockId> = fwd_reachable
        .intersection(&bwd_reachable)
        .copied()
        .collect();

    // Step 4: for each CondBr block on the path, check if it gates dst
    let mut guards = Vec::new();

    for &block in &on_path {
        let Some(crate::guard::TerminatorInfo::CondBr {
            condition,
            then_target,
            else_target,
        }) = value_index.terminator_of(block)
        else {
            continue;
        };

        // Check reachability of dst from each branch successor
        let then_reaches = cfg_can_reach(cfg, *then_target, dst);
        let else_reaches = cfg_can_reach(cfg, *else_target, dst);

        let branch_taken = match (then_reaches, else_reaches) {
            (true, false) => true,
            (false, true) => false,
            _ => continue, // both or neither → not a gate
        };

        guards.push(crate::guard::Guard {
            block,
            function: cfg.function,
            condition: *condition,
            branch_taken,
        });
    }

    guards
}

/// BFS forward reachability from a block.
fn cfg_reachable_from(
    cfg: &crate::cfg::Cfg,
    start: saf_core::ids::BlockId,
) -> BTreeSet<saf_core::ids::BlockId> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);
    while let Some(b) = queue.pop_front() {
        if let Some(succs) = cfg.successors.get(&b) {
            for &s in succs {
                if visited.insert(s) {
                    queue.push_back(s);
                }
            }
        }
    }
    visited
}

/// BFS backward reachability to a block (using predecessor edges).
fn cfg_reachable_backward(
    cfg: &crate::cfg::Cfg,
    target: saf_core::ids::BlockId,
) -> BTreeSet<saf_core::ids::BlockId> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    visited.insert(target);
    queue.push_back(target);
    while let Some(b) = queue.pop_front() {
        if let Some(preds) = cfg.predecessors.get(&b) {
            for &p in preds {
                if visited.insert(p) {
                    queue.push_back(p);
                }
            }
        }
    }
    visited
}

/// Check if `dst` is reachable from `src` in the CFG.
fn cfg_can_reach(
    cfg: &crate::cfg::Cfg,
    src: saf_core::ids::BlockId,
    dst: saf_core::ids::BlockId,
) -> bool {
    if src == dst {
        return true;
    }
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    visited.insert(src);
    queue.push_back(src);
    while let Some(b) = queue.pop_front() {
        if let Some(succs) = cfg.successors.get(&b) {
            for &s in succs {
                if s == dst {
                    return true;
                }
                if visited.insert(s) {
                    queue.push_back(s);
                }
            }
        }
    }
    false
}

/// Compute the Z3 guard for an SVFG edge.
#[cfg(feature = "z3-solver")]
#[allow(clippy::too_many_arguments)]
fn compute_edge_guard_z3(
    svfg: &Svfg,
    from: SvfgNodeId,
    to: SvfgNodeId,
    edge_kind: &SvfgEdgeKind,
    value_index: &crate::guard::ValueLocationIndex,
    cfgs: &BTreeMap<FunctionId, crate::cfg::Cfg>,
    cond_names: &mut BTreeMap<saf_core::ids::ValueId, String>,
    cond_counter: &mut u32,
) -> z3::ast::Bool {
    let guards = compute_edge_guards(svfg, from, to, edge_kind, value_index, cfgs);
    guards_to_z3(&guards, cond_names, cond_counter)
}

/// Translate pre-computed SVFG guards to a Z3 conjunction.
#[cfg(feature = "z3-solver")]
fn guards_to_z3(
    guards: &[crate::guard::Guard],
    cond_names: &mut BTreeMap<saf_core::ids::ValueId, String>,
    cond_counter: &mut u32,
) -> z3::ast::Bool {
    let mut exprs: Vec<z3::ast::Bool> = Vec::new();
    for guard in guards {
        let var_name = cond_names
            .entry(guard.condition)
            .or_insert_with(|| {
                let name = format!("guard_{}", *cond_counter);
                *cond_counter += 1;
                name
            })
            .clone();

        let var = z3::ast::Bool::new_const(var_name.as_str());
        let expr = if guard.branch_taken { var } else { var.not() };
        exprs.push(expr);
    }

    match exprs.len() {
        0 => z3::ast::Bool::from_bool(true),
        1 => exprs
            .into_iter()
            .next()
            .expect("guard list has one element"),
        _ => {
            let refs: Vec<&z3::ast::Bool> = exprs.iter().collect();
            z3::ast::Bool::and(&refs)
        }
    }
}

/// Translate guards to a BDD conjunction.
#[cfg(not(feature = "z3-solver"))]
fn guards_to_bdd(
    guards: &[crate::guard::Guard],
    vars: &BddVariableSet,
    cond_vars: &BTreeMap<saf_core::ids::ValueId, BddVariable>,
) -> Bdd {
    let mut expr = vars.mk_true();
    for guard in guards {
        let Some(var) = cond_vars.get(&guard.condition).copied() else {
            // Conservative: unknown guard variables reduce path coverage,
            // which can only produce extra partial-leak findings.
            return vars.mk_false();
        };
        let literal = vars.mk_literal(var, guard.branch_taken);
        expr = expr.and(&literal);
    }
    expr
}

/// Collect branch conditions used by the non-Z3 partial-leak fallback.
#[cfg(not(feature = "z3-solver"))]
fn collect_partial_leak_guard_conditions(
    svfg: &Svfg,
    reachable_sources: &[SourceReachability],
    module: &saf_core::air::AirModule,
    value_index: &crate::guard::ValueLocationIndex,
) -> BTreeSet<saf_core::ids::ValueId> {
    let mut conditions = BTreeSet::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            let Some(crate::guard::TerminatorInfo::CondBr { condition, .. }) =
                value_index.terminator_of(block.id)
            else {
                continue;
            };
            conditions.insert(*condition);
        }
    }

    for src in reachable_sources {
        let mut relevant_nodes = src.forward_slice.clone();
        relevant_nodes.extend(src.reached_sinks.iter().copied());

        for &node in &relevant_nodes {
            let Some(succs) = svfg.successors_of(node) else {
                continue;
            };
            for (_, succ) in succs {
                if !relevant_nodes.contains(succ) {
                    continue;
                }
                if let Some(guards) = svfg.edge_guard(node, *succ) {
                    for guard in guards {
                        conditions.insert(guard.condition);
                    }
                }
            }
        }
    }

    conditions
}

/// Build the BDD variable environment for no-Z3 partial-leak detection.
#[cfg(not(feature = "z3-solver"))]
fn build_partial_leak_bdd_env(
    svfg: &Svfg,
    reachable_sources: &[SourceReachability],
    module: &saf_core::air::AirModule,
    value_index: &crate::guard::ValueLocationIndex,
    max_conditions: usize,
) -> Option<(
    BddVariableSet,
    BTreeMap<saf_core::ids::ValueId, BddVariable>,
)> {
    let conditions =
        collect_partial_leak_guard_conditions(svfg, reachable_sources, module, value_index);

    if conditions.len() >= max_conditions {
        return None;
    }

    let vars = BddVariableSet::new_anonymous(
        u16::try_from(conditions.len()).expect("condition count is checked against BDD limits"),
    );
    let mut cond_vars = BTreeMap::new();
    for (condition, var) in conditions.into_iter().zip(vars.variables()) {
        cond_vars.insert(condition, var);
    }

    Some((vars, cond_vars))
}

/// BDD-based all-path reachable solve for builds without `z3-solver`.
#[cfg(not(feature = "z3-solver"))]
#[allow(clippy::too_many_lines)]
fn all_path_reachable_solve_bdd(
    svfg: &Svfg,
    source: SvfgNodeId,
    backward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
    value_index: &crate::guard::ValueLocationIndex,
    cfgs: &BTreeMap<FunctionId, crate::cfg::Cfg>,
    vars: &BddVariableSet,
    cond_vars: &BTreeMap<saf_core::ids::ValueId, BddVariable>,
) -> bool {
    let mut node_conds: BTreeMap<SvfgNodeId, Bdd> = BTreeMap::new();
    node_conds.insert(source, vars.mk_true());

    let mut worklist: VecDeque<SvfgNodeId> = VecDeque::new();
    let mut in_worklist: BTreeSet<SvfgNodeId> = BTreeSet::new();
    worklist.push_back(source);
    in_worklist.insert(source);

    let max_iterations = backward_slice.len().saturating_mul(3).max(10);
    let mut iterations = 0;

    while let Some(node) = worklist.pop_front() {
        in_worklist.remove(&node);
        iterations += 1;
        if iterations > max_iterations {
            return false;
        }

        let Some(cur_cond) = node_conds.get(&node).cloned() else {
            continue;
        };

        let Some(succs) = svfg.successors_of(node) else {
            continue;
        };

        for (ek, succ) in succs {
            let succ = *succ;
            let ek = *ek;
            if !backward_slice.contains(&succ) {
                continue;
            }

            let edge_guard = guards_to_bdd(
                &compute_edge_guards(svfg, node, succ, &ek, value_index, cfgs),
                vars,
                cond_vars,
            );
            let propagated = cur_cond.and(&edge_guard);

            let changed = match node_conds.get(&succ) {
                Some(existing) => {
                    let new_cond = existing.or(&propagated);
                    if new_cond == *existing {
                        false
                    } else {
                        node_conds.insert(succ, new_cond);
                        true
                    }
                }
                None => {
                    node_conds.insert(succ, propagated);
                    true
                }
            };

            if changed && !in_worklist.contains(&succ) {
                worklist.push_back(succ);
                in_worklist.insert(succ);
            }
        }
    }

    let mut sink_conds: Vec<Bdd> = Vec::new();
    for sink in reached_sinks {
        let Some(propagated) = node_conds.get(sink).cloned() else {
            continue;
        };

        let sink_has_block = if let SvfgNodeId::Value(v) = sink {
            value_index.block_of(*v).is_some()
        } else {
            false
        };

        if sink_has_block {
            sink_conds.push(propagated);
            continue;
        }

        let mut dealloc_guards: Vec<Bdd> = Vec::new();
        if let Some(succs) = svfg.successors_of(*sink) {
            for (ek, _) in succs {
                if let SvfgEdgeKind::CallArg { call_site } = ek {
                    if let Some((call_func, call_block)) = value_index.block_of_inst(*call_site) {
                        if let Some(cfg) = cfgs.get(&call_func) {
                            let guard = guards_to_bdd(
                                &cfg_path_guards(cfg, cfg.entry, call_block, value_index),
                                vars,
                                cond_vars,
                            );
                            dealloc_guards.push(guard);
                        }
                    }
                }
            }
        }

        if dealloc_guards.is_empty() {
            sink_conds.push(propagated);
        } else {
            let mut dealloc_guard = vars.mk_false();
            for guard in dealloc_guards {
                dealloc_guard = dealloc_guard.or(&guard);
            }
            sink_conds.push(propagated.and(&dealloc_guard));
        }
    }

    if sink_conds.is_empty() {
        return false;
    }

    let mut final_cond = vars.mk_false();
    for cond in sink_conds {
        final_cond = final_cond.or(&cond);
    }

    final_cond.not().is_false()
}

// ---------------------------------------------------------------------------
// SVFG-based partial leak detection (orchestrator)
// ---------------------------------------------------------------------------

/// Detect partial leaks using SVFG-based three-phase analysis.
///
/// For each source that reached at least one sink (Phase 1 result), builds
/// a backward slice (Phase 2) and checks all-path reachability via Z3 guard
/// propagation (Phase 3). Reports a partial leak if `isAllPathReachable` is
/// false.
#[cfg(feature = "z3-solver")]
pub fn detect_partial_leaks_svfg(
    svfg: &Svfg,
    spec: &CheckerSpec,
    reachable_sources: &[SourceReachability],
    _sink_nodes: &BTreeSet<SvfgNodeId>,
    module: &saf_core::air::AirModule,
) -> Vec<CheckerFinding> {
    let value_index = crate::guard::ValueLocationIndex::build(module);
    let cfgs: BTreeMap<FunctionId, crate::cfg::Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration && !f.blocks.is_empty())
        .map(|f| (f.id, crate::cfg::Cfg::build(f)))
        .collect();
    let mut findings = Vec::new();

    for src in reachable_sources {
        // Phase 2: backward slice
        let bslice = backward_slice(svfg, &src.forward_slice, &src.reached_sinks);

        // Phase 3: Z3 tautology check
        let all_path = all_path_reachable_solve(
            svfg,
            src.source,
            &bslice,
            &src.reached_sinks,
            &value_index,
            &cfgs,
        );

        if !all_path {
            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: src.source,
                sink_node: src.source,
                trace: vec![src.source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (partial leak: allocation freed on some paths but not all)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        }
    }

    findings.sort_by_key(|f| f.source_node);
    findings.dedup_by_key(|f| f.source_node);
    findings
}

/// BDD-based fallback for builds without `z3-solver`.
///
/// Partial-leak detection only needs Boolean coverage over extracted branch
/// guards, so a lightweight BDD solver can replace the Z3 tautology check in
/// `default-features = false` builds.
#[cfg(not(feature = "z3-solver"))]
pub fn detect_partial_leaks_svfg(
    svfg: &Svfg,
    spec: &CheckerSpec,
    reachable_sources: &[SourceReachability],
    _sink_nodes: &BTreeSet<SvfgNodeId>,
    module: &saf_core::air::AirModule,
) -> Vec<CheckerFinding> {
    let value_index = crate::guard::ValueLocationIndex::build(module);
    let cfgs: BTreeMap<FunctionId, crate::cfg::Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration && !f.blocks.is_empty())
        .map(|f| (f.id, crate::cfg::Cfg::build(f)))
        .collect();
    let Some((vars, cond_vars)) = build_partial_leak_bdd_env(
        svfg,
        reachable_sources,
        module,
        &value_index,
        usize::from(u16::MAX - 1),
    ) else {
        return Vec::new();
    };

    let mut findings = Vec::new();

    for src in reachable_sources {
        let bslice = backward_slice(svfg, &src.forward_slice, &src.reached_sinks);
        let all_path = all_path_reachable_solve_bdd(
            svfg,
            src.source,
            &bslice,
            &src.reached_sinks,
            &value_index,
            &cfgs,
            &vars,
            &cond_vars,
        );

        if !all_path {
            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: src.source,
                sink_node: src.source,
                trace: vec![src.source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (partial leak: allocation freed on some paths but not all)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        }
    }

    findings.sort_by_key(|f| f.source_node);
    findings.dedup_by_key(|f| f.source_node);
    findings
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the CFL context after traversing an edge.
///
/// Returns `Some(new_context)` if the edge is allowed, `None` if a mismatched
/// return should block traversal.
fn compute_cfl_context(
    ctx: &CallString,
    edge_kind: &SvfgEdgeKind,
    max_depth: usize,
) -> Option<CallString> {
    match edge_kind {
        SvfgEdgeKind::CallArg { call_site } => {
            // Entering callee: push call site
            if ctx.depth() < max_depth {
                Some(ctx.push(*call_site))
            } else {
                // At depth limit: don't push, just propagate (conservative)
                Some(ctx.clone())
            }
        }
        SvfgEdgeKind::Return { call_site } => {
            if ctx.is_empty() {
                // Empty context: allow return conservatively
                Some(ctx.clone())
            } else if ctx.matches(*call_site) {
                // Matched return: pop the call site
                Some(ctx.pop().expect("context is non-empty").0)
            } else {
                // Mismatched return: this is an unrealizable path
                None
            }
        }
        _ => Some(ctx.clone()),
    }
}

/// Reconstruct a trace from a context-aware parent map, returning only node IDs.
fn reconstruct_trace_ctx(
    parent: &BTreeMap<(SvfgNodeId, CallString), (SvfgNodeId, CallString)>,
    source: SvfgNodeId,
    target: SvfgNodeId,
    target_ctx: &CallString,
) -> Vec<SvfgNodeId> {
    let mut path = vec![target];
    let mut current = (target, target_ctx.clone());

    while let Some((prev_node, prev_ctx)) = parent.get(&current) {
        path.push(*prev_node);
        if *prev_node == source {
            break;
        }
        current = (*prev_node, prev_ctx.clone());
    }

    path.reverse();
    path
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::spec::Severity;
    use super::*;
    use crate::guard::Guard;
    use crate::mssa::MemAccessId;
    use saf_core::ids::{BlockId, InstId, ValueId};

    fn make_linear_svfg() -> Svfg {
        // source ---> mid ---> sink
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink);
        g
    }

    fn make_sanitized_svfg() -> Svfg {
        // source ---> sanitizer ---> sink
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sanitizer = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sanitizer);
        g.add_edge(sanitizer, SvfgEdgeKind::DirectDef, sink);
        g
    }

    fn make_branching_svfg() -> Svfg {
        // source ---> sanitizer ---> exit1
        //        \--> mid ---------> exit2
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sanitizer = SvfgNodeId::value(ValueId::new(2));
        let mid = SvfgNodeId::value(ValueId::new(3));
        let exit1 = SvfgNodeId::value(ValueId::new(4));
        let exit2 = SvfgNodeId::value(ValueId::new(5));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sanitizer);
        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(sanitizer, SvfgEdgeKind::DirectDef, exit1);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, exit2);
        g
    }

    fn test_spec_may_reach() -> CheckerSpec {
        CheckerSpec {
            name: "test-may-reach".to_string(),
            description: "test checker".to_string(),
            cwe: Some(416),
            severity: Severity::Error,
            mode: super::super::spec::ReachabilityMode::MayReach,
            sources: vec![],
            sinks: vec![],
            sanitizers: vec![],
        }
    }

    fn test_spec_must_not_reach() -> CheckerSpec {
        CheckerSpec {
            name: "test-must-not-reach".to_string(),
            description: "test checker".to_string(),
            cwe: Some(401),
            severity: Severity::Warning,
            mode: super::super::spec::ReachabilityMode::MustNotReach,
            sources: vec![],
            sinks: vec![],
            sanitizers: vec![],
        }
    }

    // ---- may_reach tests ----

    #[test]
    fn may_reach_finds_unsanitized_path() {
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();

        let findings = may_reach(&svfg, &spec, &sources, &sinks, &sanitizers, &config);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].checker_name, "test-may-reach");
    }

    #[test]
    fn may_reach_pruned_by_sanitizer() {
        let svfg = make_sanitized_svfg();
        let spec = test_spec_may_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::from([SvfgNodeId::value(ValueId::new(2))]);

        let findings = may_reach(&svfg, &spec, &sources, &sinks, &sanitizers, &config);
        assert!(findings.is_empty(), "Sanitizer should prune the path");
    }

    #[test]
    fn may_reach_no_path() {
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(3))]; // sink as source
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(1))]); // unreachable
        let sanitizers = BTreeSet::new();

        let findings = may_reach(&svfg, &spec, &sources, &sinks, &sanitizers, &config);
        assert!(findings.is_empty());
    }

    #[test]
    fn may_reach_trace_has_correct_endpoints() {
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();

        let findings = may_reach(&svfg, &spec, &sources, &sinks, &sanitizers, &config);
        assert_eq!(findings.len(), 1);

        let trace = &findings[0].trace;
        assert_eq!(trace.first(), Some(&SvfgNodeId::value(ValueId::new(1))));
        assert_eq!(trace.last(), Some(&SvfgNodeId::value(ValueId::new(3))));
    }

    #[test]
    fn may_reach_with_cycle() {
        // source -> A -> B -> A (cycle), B -> sink
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let a = SvfgNodeId::value(ValueId::new(2));
        let b = SvfgNodeId::value(ValueId::new(3));
        let sink = SvfgNodeId::value(ValueId::new(4));

        g.add_edge(source, SvfgEdgeKind::DirectDef, a);
        g.add_edge(a, SvfgEdgeKind::DirectDef, b);
        g.add_edge(b, SvfgEdgeKind::DirectDef, a); // cycle
        g.add_edge(b, SvfgEdgeKind::DirectDef, sink);

        let spec = test_spec_may_reach();
        let config = SolverConfig::default();

        let findings = may_reach(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([sink]),
            &BTreeSet::new(),
            &config,
        );
        assert_eq!(findings.len(), 1, "Should find sink despite cycle");
    }

    #[test]
    fn may_reach_max_depth() {
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = SolverConfig {
            max_depth: 1,
            max_context_depth: 3,
            ..Default::default()
        };

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();

        let findings = may_reach(&svfg, &spec, &sources, &sinks, &sanitizers, &config);
        assert!(
            findings.is_empty(),
            "Max depth 1 should not reach sink at depth 2"
        );
    }

    // ---- must_not_reach tests ----

    #[test]
    fn must_not_reach_reports_unsanitized_exit() {
        let svfg = make_linear_svfg();
        let spec = test_spec_must_not_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let exits = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new(); // no sanitizer

        let findings = must_not_reach(&svfg, &spec, &sources, &exits, &sanitizers, &config);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].checker_name, "test-must-not-reach");
    }

    #[test]
    fn must_not_reach_all_paths_sanitized() {
        let svfg = make_sanitized_svfg();
        let spec = test_spec_must_not_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let exits = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::from([SvfgNodeId::value(ValueId::new(2))]);

        let findings = must_not_reach(&svfg, &spec, &sources, &exits, &sanitizers, &config);
        assert!(findings.is_empty(), "All paths go through sanitizer");
    }

    #[test]
    fn must_not_reach_one_path_unsanitized() {
        let svfg = make_branching_svfg();
        let spec = test_spec_must_not_reach();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        // Both exit1 and exit2 are exits
        let exits = BTreeSet::from([
            SvfgNodeId::value(ValueId::new(4)),
            SvfgNodeId::value(ValueId::new(5)),
        ]);
        // Only the path through node 2 is sanitized
        let sanitizers = BTreeSet::from([SvfgNodeId::value(ValueId::new(2))]);

        let findings = must_not_reach(&svfg, &spec, &sources, &exits, &sanitizers, &config);
        // exit2 (node 5) is reachable without sanitizer
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].sink_node, SvfgNodeId::value(ValueId::new(5)));
    }

    #[test]
    fn must_not_reach_no_exit_no_sanitizer_reports_leak() {
        // source with no successors — no exit reachable AND no sanitizer found.
        // This represents a heap allocation that is never freed and never
        // flows to a return instruction (common for `ret void` functions).
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        g.add_node(source);

        let spec = test_spec_must_not_reach();
        let config = SolverConfig::default();

        let exits = BTreeSet::from([SvfgNodeId::value(ValueId::new(99))]);
        let sanitizers = BTreeSet::new();

        let findings = must_not_reach(&g, &spec, &[source], &exits, &sanitizers, &config);
        assert_eq!(
            findings.len(),
            1,
            "No sanitizer reachable → should report leak"
        );
        assert_eq!(findings[0].source_node, source);
        assert_eq!(findings[0].sink_node, source); // self-referential
        assert!(
            findings[0]
                .message
                .contains("no sanitizer reachable from source")
        );
    }

    #[test]
    fn must_not_reach_through_memory() {
        // source -> store -> phi -> load -> exit
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let phi = SvfgNodeId::mem_phi(MemAccessId::new(100));
        let load = SvfgNodeId::value(ValueId::new(2));
        let exit = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::IndirectStore, phi);
        g.add_edge(phi, SvfgEdgeKind::IndirectLoad, load);
        g.add_edge(load, SvfgEdgeKind::DirectDef, exit);

        let spec = test_spec_must_not_reach();
        let config = SolverConfig::default();

        let findings = must_not_reach(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([exit]),
            &BTreeSet::new(),
            &config,
        );
        assert_eq!(findings.len(), 1, "Should track value through memory");
    }

    #[test]
    fn must_not_reach_source_is_sanitizer_no_finding() {
        // source -> exit, where source IS also a sanitizer
        // (models: %call = malloc(4); free(%call); ret — same SSA value
        // is both the allocator return and the free argument)
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let exit = SvfgNodeId::value(ValueId::new(2));
        g.add_edge(source, SvfgEdgeKind::DirectDef, exit);

        let spec = test_spec_must_not_reach();
        let config = SolverConfig::default();
        let exit_nodes = BTreeSet::from([exit]);
        let sanitizer_nodes = BTreeSet::from([source]); // source IS the sanitizer

        let findings = must_not_reach(&g, &spec, &[source], &exit_nodes, &sanitizer_nodes, &config);
        assert!(
            findings.is_empty(),
            "Source=sanitizer means allocation is freed, no leak"
        );
    }

    // ---- multi_reach tests ----

    fn test_spec_multi_reach() -> CheckerSpec {
        CheckerSpec {
            name: "test-multi-reach".to_string(),
            description: "test checker for double-free".to_string(),
            cwe: Some(415),
            severity: Severity::Critical,
            mode: super::super::spec::ReachabilityMode::MultiReach,
            sources: vec![],
            sinks: vec![],
            sanitizers: vec![],
        }
    }

    #[test]
    fn multi_reach_finds_double_free() {
        // source (malloc) -> mid -> sink1 (free)
        //                \-> sink2 (free)
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1)); // malloc return
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink1 = SvfgNodeId::value(ValueId::new(3)); // first free
        let sink2 = SvfgNodeId::value(ValueId::new(4)); // second free

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink2);

        let spec = test_spec_multi_reach();
        let config = SolverConfig::default();

        let sinks = BTreeSet::from([sink1, sink2]);
        let findings = multi_reach(&g, &spec, &[source], &sinks, &config);

        assert_eq!(findings.len(), 1, "Should detect double-free");
        assert_eq!(findings[0].source_node, source);
        assert!(findings[0].message.contains("2 free calls"));
        assert_eq!(
            findings[0].sink_traces.len(),
            2,
            "Should have per-sink traces"
        );
    }

    #[test]
    fn multi_reach_single_free_no_finding() {
        // source (malloc) -> sink (single free)
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink = SvfgNodeId::value(ValueId::new(2));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink);

        let spec = test_spec_multi_reach();
        let config = SolverConfig::default();

        let sinks = BTreeSet::from([sink]);
        let findings = multi_reach(&g, &spec, &[source], &sinks, &config);

        assert!(
            findings.is_empty(),
            "Single free should not trigger double-free"
        );
    }

    #[test]
    fn multi_reach_no_free_no_finding() {
        // source (malloc) -> mid -> end (no free)
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let end = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, end);

        let spec = test_spec_multi_reach();
        let config = SolverConfig::default();

        let sinks = BTreeSet::new(); // no sinks
        let findings = multi_reach(&g, &spec, &[source], &sinks, &config);

        assert!(findings.is_empty(), "No free means no double-free");
    }

    #[test]
    fn multi_reach_triple_free() {
        // source -> sink1, sink2, sink3 (triple free)
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink1 = SvfgNodeId::value(ValueId::new(2));
        let sink2 = SvfgNodeId::value(ValueId::new(3));
        let sink3 = SvfgNodeId::value(ValueId::new(4));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(source, SvfgEdgeKind::DirectDef, sink2);
        g.add_edge(source, SvfgEdgeKind::DirectDef, sink3);

        let spec = test_spec_multi_reach();
        let config = SolverConfig::default();

        let sinks = BTreeSet::from([sink1, sink2, sink3]);
        let findings = multi_reach(&g, &spec, &[source], &sinks, &config);

        assert_eq!(
            findings.len(),
            1,
            "Should detect triple-free as double-free"
        );
        assert!(findings[0].message.contains("3 free calls"));
        assert_eq!(
            findings[0].sink_traces.len(),
            3,
            "Should have per-sink traces for all 3"
        );
    }

    #[test]
    fn multi_reach_stores_individual_traces() {
        // source -> mid -> sink1
        //              \-> sink2
        // Verify each sink_trace has correct sink node and non-empty trace from source
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink1 = SvfgNodeId::value(ValueId::new(3));
        let sink2 = SvfgNodeId::value(ValueId::new(4));

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink2);

        let spec = test_spec_multi_reach();
        let config = SolverConfig::default();
        let sinks = BTreeSet::from([sink1, sink2]);
        let findings = multi_reach(&g, &spec, &[source], &sinks, &config);

        assert_eq!(findings.len(), 1);
        let st = &findings[0].sink_traces;
        assert_eq!(st.len(), 2);

        // Each trace should start at source and end at its sink
        for (sink_node, trace) in st {
            assert!(!trace.is_empty(), "Trace should not be empty");
            assert_eq!(trace[0], source, "Trace should start at source");
            assert_eq!(
                *trace.last().unwrap(),
                *sink_node,
                "Trace should end at its sink"
            );
        }
    }

    // ---- never_reach_sink tests ----

    fn empty_test_module() -> saf_core::air::AirModule {
        saf_core::air::AirModule {
            id: saf_core::ids::ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: Vec::new(),
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: std::collections::BTreeMap::new(),
            name_index: std::collections::BTreeMap::new(),
        }
    }

    fn test_spec_never_reach_sink() -> CheckerSpec {
        CheckerSpec {
            name: "test-source-to-sink".to_string(),
            description: "test checker for never-freed".to_string(),
            cwe: Some(401),
            severity: Severity::Warning,
            mode: super::super::spec::ReachabilityMode::NeverReachSink,
            sources: vec![],
            sinks: vec![],
            sanitizers: vec![],
        }
    }

    #[test]
    fn never_reach_sink_no_sink_reports_leak() {
        // source -> mid -> end (no sinks in graph)
        let svfg = make_linear_svfg();
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::new(); // no sinks at all

        let findings = never_reach_sink(&svfg, &spec, &sources, &sinks, &config);
        assert_eq!(findings.len(), 1, "No sinks → should report leak");
        assert!(findings[0].message.contains("allocation never freed"));
    }

    #[test]
    fn never_reach_sink_with_sink_no_leak() {
        // source -> mid -> sink (sink is reachable)
        let svfg = make_linear_svfg();
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]); // node 3 is a sink

        let findings = never_reach_sink(&svfg, &spec, &sources, &sinks, &config);
        assert!(findings.is_empty(), "Sink reachable → no leak");
    }

    #[test]
    fn never_reach_sink_dead_end_reports_leak() {
        // Isolated source node with no successors and no sinks
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        g.add_node(source);

        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();

        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(99))]); // unreachable sink
        let findings = never_reach_sink(&g, &spec, &[source], &sinks, &config);
        assert_eq!(findings.len(), 1, "Dead-end source → should report leak");
    }

    #[test]
    fn never_reach_sink_source_is_sink_no_leak() {
        // source IS a sink (e.g., malloc return is also free argument — same SSA)
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        g.add_node(source);

        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();

        let sinks = BTreeSet::from([source]); // source IS the sink
        let findings = never_reach_sink(&g, &spec, &[source], &sinks, &config);
        assert!(
            findings.is_empty(),
            "Source=sink means allocation is freed, no leak"
        );
    }

    #[test]
    fn never_reach_sink_source_not_in_svfg_reports_leak() {
        // Source node does not exist in the SVFG at all (empty graph).
        // This represents an allocation whose return value has zero value
        // flow — e.g., `void *p = malloc(10);` with p never used.
        let g = Svfg::new(); // empty graph, 0 nodes
        let source = SvfgNodeId::value(ValueId::new(42));

        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();

        let sinks = BTreeSet::new();
        let findings = never_reach_sink(&g, &spec, &[source], &sinks, &config);
        assert_eq!(
            findings.len(),
            1,
            "Source not in SVFG → allocation has no value flow → definitive leak"
        );
        assert!(findings[0].message.contains("allocation never freed"));
    }

    // ---- MustNotReach vs NeverReachSink comparison tests ----

    #[test]
    fn must_not_reach_fp_on_freed_in_caller() {
        // Allocator wrapper pattern:
        //   my_alloc() { return malloc(10); }
        //   main()     { void *p = my_alloc(); free(p); }
        //
        // SVFG:
        //   V1 (malloc ret in wrapper)
        //     → V2 (DirectDef: copy/store-load in wrapper)
        //       → V3 (Return edge: wrapper returns to caller)
        //         → V4 (CallArg edge: caller passes to free)
        //
        // V2 is a function-exit node (ret operand in wrapper).
        // V4 is a sanitizer (free's argument).
        //
        // MustNotReach WITHOUT exit scoping sees V2 as a terminal exit
        // reached without sanitizer → reports FP. The pointer IS freed
        // in the caller, but MustNotReach stops at the wrapper's exit.
        let mut g = Svfg::new();
        let v_malloc = SvfgNodeId::value(ValueId::new(1)); // malloc return
        let v_ret = SvfgNodeId::value(ValueId::new(2)); // wrapper ret operand
        let v_caller = SvfgNodeId::value(ValueId::new(3)); // call result in main
        let v_free_arg = SvfgNodeId::value(ValueId::new(4)); // free's argument

        let call_site = InstId::new(200); // the call to my_alloc in main

        g.add_edge(v_malloc, SvfgEdgeKind::DirectDef, v_ret);
        g.add_edge(v_ret, SvfgEdgeKind::Return { call_site }, v_caller);
        g.add_edge(
            v_caller,
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(201),
            },
            v_free_arg,
        );

        let exit_nodes = BTreeSet::from([v_ret]); // wrapper's exit
        let sanitizer_nodes = BTreeSet::from([v_free_arg]); // free's arg

        // MustNotReach without exit scoping (node_to_func = None):
        // BFS hits v_ret (exit) before reaching v_free_arg (sanitizer).
        // Since node_to_func is None, has_return_to_caller = false,
        // so v_ret is treated as a terminal exit → FP.
        let spec_mnr = test_spec_must_not_reach();
        let config_no_scoping = SolverConfig::default(); // node_to_func = None

        let findings_mnr = must_not_reach(
            &g,
            &spec_mnr,
            &[v_malloc],
            &exit_nodes,
            &sanitizer_nodes,
            &config_no_scoping,
        );
        assert_eq!(
            findings_mnr.len(),
            1,
            "MustNotReach without exit scoping: FP on freed-in-caller"
        );
        assert!(
            findings_mnr[0]
                .message
                .contains("source reaches function exit without sanitizer")
        );

        // NeverReachSink: BFS from v_malloc follows Return edge into
        // caller, reaches v_free_arg (sink) → no leak reported. Correct.
        let spec_nrs = test_spec_never_reach_sink();
        let config = SolverConfig::default();
        let sink_nodes = BTreeSet::from([v_free_arg]); // free's arg is the sink

        let findings_nrs = never_reach_sink(&g, &spec_nrs, &[v_malloc], &sink_nodes, &config);
        assert!(
            findings_nrs.is_empty(),
            "NeverReachSink: no FP — BFS follows Return edge and finds free in caller"
        );
    }

    // ---- has_contradictory_guards tests ----

    #[test]
    fn contradictory_guards_detected() {
        use crate::guard::Guard;
        use saf_core::ids::{BlockId, FunctionId};

        let blk = BlockId::new(10);
        let func = FunctionId::new(1);
        let cond = ValueId::new(100);

        let guards = vec![
            Guard {
                block: blk,
                function: func,
                condition: cond,
                branch_taken: true,
            },
            Guard {
                block: blk,
                function: func,
                condition: cond,
                branch_taken: false,
            },
        ];
        assert!(
            has_contradictory_guards(&guards),
            "Same condition, same block, opposite branch_taken should be contradictory"
        );
    }

    #[test]
    fn non_contradictory_guards_pass() {
        use crate::guard::Guard;
        use saf_core::ids::{BlockId, FunctionId};

        let blk = BlockId::new(10);
        let func = FunctionId::new(1);

        // Same condition, same block, same branch_taken -- not contradictory
        let guards = vec![
            Guard {
                block: blk,
                function: func,
                condition: ValueId::new(100),
                branch_taken: true,
            },
            Guard {
                block: blk,
                function: func,
                condition: ValueId::new(100),
                branch_taken: true,
            },
        ];
        assert!(!has_contradictory_guards(&guards));

        // Different conditions -- not contradictory
        let guards2 = vec![
            Guard {
                block: blk,
                function: func,
                condition: ValueId::new(100),
                branch_taken: true,
            },
            Guard {
                block: blk,
                function: func,
                condition: ValueId::new(200),
                branch_taken: false,
            },
        ];
        assert!(!has_contradictory_guards(&guards2));

        // Different blocks -- not contradictory
        let guards3 = vec![
            Guard {
                block: BlockId::new(10),
                function: func,
                condition: ValueId::new(100),
                branch_taken: true,
            },
            Guard {
                block: BlockId::new(20),
                function: func,
                condition: ValueId::new(100),
                branch_taken: false,
            },
        ];
        assert!(!has_contradictory_guards(&guards3));
    }

    #[test]
    fn empty_guards_not_contradictory() {
        assert!(!has_contradictory_guards(&[]));
    }

    // ---- may_reach_guarded tests ----

    #[test]
    fn guarded_finds_unsanitized_path_no_guards() {
        // Without any edge guards, should behave like `may_reach`.
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &svfg,
            &spec,
            &sources,
            &sinks,
            &sanitizers,
            &config,
            &dead_blocks,
            &block_of,
        );
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("guard-aware"));
    }

    #[test]
    fn guarded_pruned_by_sanitizer() {
        let svfg = make_sanitized_svfg();
        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::from([SvfgNodeId::value(ValueId::new(2))]);
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &svfg,
            &spec,
            &sources,
            &sinks,
            &sanitizers,
            &config,
            &dead_blocks,
            &block_of,
        );
        assert!(findings.is_empty(), "Sanitizer should prune the path");
    }

    #[test]
    fn guarded_prunes_contradictory_path() {
        use crate::guard::Guard;
        use saf_core::ids::{BlockId, FunctionId};

        // source ---> mid ---> sink
        // Edge source->mid has guard (cond=100, taken=true)
        // Edge mid->sink has guard (cond=100, taken=false) -- contradicts!
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink);

        let blk = BlockId::new(10);
        let func = FunctionId::new(1);
        let cond = ValueId::new(100);

        g.set_edge_guard(
            source,
            mid,
            vec![Guard {
                block: blk,
                function: func,
                condition: cond,
                branch_taken: true,
            }],
        );
        g.set_edge_guard(
            mid,
            sink,
            vec![Guard {
                block: blk,
                function: func,
                condition: cond,
                branch_taken: false,
            }],
        );

        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([sink]),
            &BTreeSet::new(),
            &config,
            &dead_blocks,
            &block_of,
        );
        assert!(
            findings.is_empty(),
            "Contradictory guards should prune the path to sink"
        );
    }

    #[test]
    fn guarded_allows_consistent_path() {
        use crate::guard::Guard;
        use saf_core::ids::{BlockId, FunctionId};

        // source ---> mid ---> sink
        // Edge source->mid has guard (cond=100, taken=true)
        // Edge mid->sink has guard (cond=200, taken=false) -- different cond, no contradiction
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink);

        let blk = BlockId::new(10);
        let func = FunctionId::new(1);

        g.set_edge_guard(
            source,
            mid,
            vec![Guard {
                block: blk,
                function: func,
                condition: ValueId::new(100),
                branch_taken: true,
            }],
        );
        g.set_edge_guard(
            mid,
            sink,
            vec![Guard {
                block: blk,
                function: func,
                condition: ValueId::new(200),
                branch_taken: false,
            }],
        );

        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([sink]),
            &BTreeSet::new(),
            &config,
            &dead_blocks,
            &block_of,
        );
        assert_eq!(findings.len(), 1, "Consistent guards should allow the path");
    }

    #[test]
    fn guarded_skips_dead_block_source() {
        use saf_core::ids::BlockId;

        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();

        let source_vid = ValueId::new(1);
        let source_blk = BlockId::new(10);

        let sources = vec![SvfgNodeId::value(source_vid)];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();

        // Mark source's block as dead
        let dead_blocks = BTreeSet::from([source_blk]);
        let mut block_of = BTreeMap::new();
        block_of.insert(source_vid, source_blk);

        let findings = may_reach_guarded(
            &svfg,
            &spec,
            &sources,
            &sinks,
            &sanitizers,
            &config,
            &dead_blocks,
            &block_of,
        );
        assert!(
            findings.is_empty(),
            "Source in dead block should be skipped"
        );
    }

    #[test]
    fn guarded_skips_dead_block_target() {
        use saf_core::ids::BlockId;

        // source ---> mid ---> sink
        // mid is in a dead block
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();

        let mid_vid = ValueId::new(2);
        let mid_blk = BlockId::new(20);

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();

        // Mark mid's block as dead
        let dead_blocks = BTreeSet::from([mid_blk]);
        let mut block_of = BTreeMap::new();
        block_of.insert(mid_vid, mid_blk);

        let findings = may_reach_guarded(
            &svfg,
            &spec,
            &sources,
            &sinks,
            &sanitizers,
            &config,
            &dead_blocks,
            &block_of,
        );
        assert!(
            findings.is_empty(),
            "Path through dead block should be pruned"
        );
    }

    #[test]
    fn guarded_respects_max_depth() {
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig {
            base: SolverConfig {
                max_depth: 1,
                max_context_depth: 3,
                ..Default::default()
            },
            max_disjuncts: 20,
        };

        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &svfg,
            &spec,
            &sources,
            &sinks,
            &sanitizers,
            &config,
            &dead_blocks,
            &block_of,
        );
        assert!(
            findings.is_empty(),
            "Max depth 1 should not reach sink at depth 2"
        );
    }

    #[test]
    fn guarded_memphi_node_not_dead_blocked() {
        // MemPhi nodes don't have blocks, so they should never be
        // treated as dead-blocked.
        use saf_core::ids::BlockId;

        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let phi = SvfgNodeId::mem_phi(MemAccessId::new(100));
        let sink = SvfgNodeId::value(ValueId::new(2));

        g.add_edge(source, SvfgEdgeKind::IndirectStore, phi);
        g.add_edge(phi, SvfgEdgeKind::IndirectLoad, sink);

        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();

        // Mark some arbitrary block as dead -- it should not affect MemPhi
        let dead_blocks = BTreeSet::from([BlockId::new(999)]);
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([sink]),
            &BTreeSet::new(),
            &config,
            &dead_blocks,
            &block_of,
        );
        assert_eq!(
            findings.len(),
            1,
            "MemPhi nodes should not be affected by dead blocks"
        );
    }

    #[test]
    fn guarded_budget_truncation() {
        use crate::guard::Guard;
        use saf_core::ids::{BlockId, FunctionId};

        // source ---> sink
        // Edge has many guards (more than budget) -- should be truncated, not crash.
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink = SvfgNodeId::value(ValueId::new(2));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink);

        let func = FunctionId::new(1);
        // Add 50 guards (way over max_disjuncts=5)
        let many_guards: Vec<Guard> = (0u128..50)
            .map(|i| Guard {
                block: BlockId::new(i),
                function: func,
                condition: ValueId::new(i + 1000),
                branch_taken: true,
            })
            .collect();
        g.set_edge_guard(source, sink, many_guards);

        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig {
            base: SolverConfig::default(),
            max_disjuncts: 5,
        };
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([sink]),
            &BTreeSet::new(),
            &config,
            &dead_blocks,
            &block_of,
        );
        // Should still find the sink (no contradictions, just truncation)
        assert_eq!(
            findings.len(),
            1,
            "Budget truncation should not prevent finding"
        );
    }

    #[test]
    fn guarded_deduplicates_findings() {
        // Two source entries for the same source; both reach the same sink.
        // Should produce only one finding after dedup.
        let svfg = make_linear_svfg();
        let spec = test_spec_may_reach();
        let config = GuardedSolverConfig::default();

        let source = SvfgNodeId::value(ValueId::new(1));
        let sources = vec![source, source]; // duplicate
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let sanitizers = BTreeSet::new();
        let dead_blocks = BTreeSet::new();
        let block_of = BTreeMap::new();

        let findings = may_reach_guarded(
            &svfg,
            &spec,
            &sources,
            &sinks,
            &sanitizers,
            &config,
            &dead_blocks,
            &block_of,
        );
        assert_eq!(findings.len(), 1, "Duplicate findings should be deduped");
    }

    // ---- CFL context tests ----

    #[test]
    fn cfl_mismatched_return_blocks_path() {
        // source --CallArg(site1)--> param --Return(site2)--> target
        // Mismatched call/return should block the path.
        use saf_core::ids::InstId;

        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let param = SvfgNodeId::value(ValueId::new(2));
        let target = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(
            source,
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(100),
            },
            param,
        );
        g.add_edge(
            param,
            SvfgEdgeKind::Return {
                call_site: InstId::new(200),
            },
            target,
        );

        let spec = test_spec_may_reach();
        let config = SolverConfig {
            max_depth: 5000,
            max_context_depth: 3,
            ..Default::default()
        };

        let findings = may_reach(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([target]),
            &BTreeSet::new(),
            &config,
        );
        assert!(
            findings.is_empty(),
            "Mismatched return should block path: found {} findings",
            findings.len()
        );
    }

    #[test]
    fn cfl_matched_return_allows_path() {
        // source --CallArg(site1)--> param --Return(site1)--> target
        // Matched call/return should allow the path.
        use saf_core::ids::InstId;

        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let param = SvfgNodeId::value(ValueId::new(2));
        let target = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(
            source,
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(100),
            },
            param,
        );
        g.add_edge(
            param,
            SvfgEdgeKind::Return {
                call_site: InstId::new(100),
            },
            target,
        );

        let spec = test_spec_may_reach();
        let config = SolverConfig {
            max_depth: 5000,
            max_context_depth: 3,
            ..Default::default()
        };

        let findings = may_reach(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([target]),
            &BTreeSet::new(),
            &config,
        );
        assert_eq!(findings.len(), 1, "Matched return should allow path");
    }

    #[test]
    fn cfl_empty_context_return_allowed() {
        // source --Return(site1)--> target
        // Empty context + return = conservative allow.
        use saf_core::ids::InstId;

        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let target = SvfgNodeId::value(ValueId::new(2));

        g.add_edge(
            source,
            SvfgEdgeKind::Return {
                call_site: InstId::new(100),
            },
            target,
        );

        let spec = test_spec_may_reach();
        let config = SolverConfig {
            max_depth: 5000,
            max_context_depth: 3,
            ..Default::default()
        };

        let findings = may_reach(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([target]),
            &BTreeSet::new(),
            &config,
        );
        assert_eq!(
            findings.len(),
            1,
            "Empty context return should be allowed conservatively"
        );
    }

    #[test]
    fn cfl_multi_reach_identity_fn_no_fp() {
        // Two sources each go through an identity function via different call sites.
        // source1 --CallArg(siteA)--> param --Return(siteA)--> sink1
        // source2 --CallArg(siteB)--> param --Return(siteB)--> sink2
        // multi_reach from source1 should only find sink1, not sink2.
        use saf_core::ids::InstId;

        let mut g = Svfg::new();
        let source1 = SvfgNodeId::value(ValueId::new(1));
        let source2 = SvfgNodeId::value(ValueId::new(2));
        let param = SvfgNodeId::value(ValueId::new(10));
        let sink1 = SvfgNodeId::value(ValueId::new(3));
        let sink2 = SvfgNodeId::value(ValueId::new(4));

        let site_a = InstId::new(100);
        let site_b = InstId::new(200);

        g.add_edge(source1, SvfgEdgeKind::CallArg { call_site: site_a }, param);
        g.add_edge(source2, SvfgEdgeKind::CallArg { call_site: site_b }, param);
        g.add_edge(param, SvfgEdgeKind::Return { call_site: site_a }, sink1);
        g.add_edge(param, SvfgEdgeKind::Return { call_site: site_b }, sink2);

        let spec = test_spec_multi_reach();
        let config = SolverConfig {
            max_depth: 5000,
            max_context_depth: 3,
            ..Default::default()
        };

        let sinks = BTreeSet::from([sink1, sink2]);

        // source1 should only reach sink1 (matched via site_a), not sink2
        let findings1 = multi_reach(&g, &spec, &[source1], &sinks, &config);
        assert!(
            findings1.is_empty(),
            "source1 should reach only 1 sink (sink1), not trigger double-free"
        );

        // source2 should only reach sink2 (matched via site_b), not sink1
        let findings2 = multi_reach(&g, &spec, &[source2], &sinks, &config);
        assert!(
            findings2.is_empty(),
            "source2 should reach only 1 sink (sink2), not trigger double-free"
        );
    }

    #[test]
    fn cfl_disabled_when_k_zero() {
        // Same as cfl_mismatched_return_blocks_path, but with max_context_depth: 0.
        // CFL is disabled, so the mismatched path should be allowed (backward compat).
        use saf_core::ids::InstId;

        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let param = SvfgNodeId::value(ValueId::new(2));
        let target = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(
            source,
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(100),
            },
            param,
        );
        g.add_edge(
            param,
            SvfgEdgeKind::Return {
                call_site: InstId::new(200),
            },
            target,
        );

        let spec = test_spec_may_reach();
        let config = SolverConfig {
            max_depth: 5000,
            max_context_depth: 0,
            ..Default::default()
        };

        let findings = may_reach(
            &g,
            &spec,
            &[source],
            &BTreeSet::from([target]),
            &BTreeSet::new(),
            &config,
        );
        assert_eq!(
            findings.len(),
            1,
            "With CFL disabled (k=0), mismatched return should be allowed"
        );
    }

    // ---- forward_bfs_enriched tests ----

    #[test]
    fn forward_bfs_enriched_no_sink_is_neverfree() {
        let svfg = make_linear_svfg();
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();
        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::new();
        let result = forward_bfs_enriched(&svfg, &spec, &sources, &sinks, &config);
        assert_eq!(result.neverfree_findings.len(), 1);
        assert!(result.reachable_sources.is_empty());
    }

    #[test]
    fn forward_bfs_enriched_with_sink_is_reachable() {
        let svfg = make_linear_svfg();
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();
        let sources = vec![SvfgNodeId::value(ValueId::new(1))];
        let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);
        let result = forward_bfs_enriched(&svfg, &spec, &sources, &sinks, &config);
        assert!(result.neverfree_findings.is_empty());
        assert_eq!(result.reachable_sources.len(), 1);
        let src = &result.reachable_sources[0];
        assert_eq!(src.source, SvfgNodeId::value(ValueId::new(1)));
        assert!(
            src.reached_sinks
                .contains(&SvfgNodeId::value(ValueId::new(3)))
        );
        assert!(
            src.forward_slice
                .contains(&SvfgNodeId::value(ValueId::new(1)))
        );
        assert!(
            src.forward_slice
                .contains(&SvfgNodeId::value(ValueId::new(2)))
        );
    }

    #[test]
    fn forward_bfs_enriched_finds_multiple_sinks() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink1 = SvfgNodeId::value(ValueId::new(2));
        let mid = SvfgNodeId::value(ValueId::new(3));
        let sink2 = SvfgNodeId::value(ValueId::new(4));
        g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink2);
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();
        let sources = vec![source];
        let sinks = BTreeSet::from([sink1, sink2]);
        let result = forward_bfs_enriched(&g, &spec, &sources, &sinks, &config);
        assert!(result.neverfree_findings.is_empty());
        assert_eq!(result.reachable_sources.len(), 1);
        assert_eq!(result.reachable_sources[0].reached_sinks.len(), 2);
    }

    #[test]
    fn forward_bfs_enriched_source_not_in_svfg() {
        let g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(42));
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();
        let sinks = BTreeSet::new();
        let result = forward_bfs_enriched(&g, &spec, &[source], &sinks, &config);
        assert_eq!(result.neverfree_findings.len(), 1);
    }

    #[test]
    fn forward_bfs_enriched_source_is_sink() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        g.add_node(source);
        let spec = test_spec_never_reach_sink();
        let config = SolverConfig::default();
        let sinks = BTreeSet::from([source]);
        let result = forward_bfs_enriched(&g, &spec, &[source], &sinks, &config);
        assert!(result.neverfree_findings.is_empty());
        assert!(result.reachable_sources.is_empty());
    }

    // ---- backward_slice tests ----

    #[test]
    fn backward_slice_linear() {
        // source -> mid -> sink
        let svfg = make_linear_svfg();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));

        let forward = BTreeSet::from([source, mid]);
        let sinks = BTreeSet::from([sink]);

        let bslice = backward_slice(&svfg, &forward, &sinks);
        assert!(bslice.contains(&sink));
        assert!(bslice.contains(&mid));
        assert!(bslice.contains(&source));
    }

    #[test]
    fn backward_slice_filters_non_forward() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let a = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));
        let b = SvfgNodeId::value(ValueId::new(4));

        g.add_edge(source, SvfgEdgeKind::DirectDef, a);
        g.add_edge(a, SvfgEdgeKind::DirectDef, sink);
        g.add_edge(a, SvfgEdgeKind::DirectDef, b);

        // forward_slice only has {source, a} — b is excluded
        let forward = BTreeSet::from([source, a]);
        let sinks = BTreeSet::from([sink]);

        let bslice = backward_slice(&g, &forward, &sinks);
        assert!(bslice.contains(&sink));
        assert!(bslice.contains(&a));
        assert!(bslice.contains(&source));
        assert!(!bslice.contains(&b), "b not in forward_slice → excluded");
    }

    #[test]
    fn backward_slice_branching() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink1 = SvfgNodeId::value(ValueId::new(2));
        let mid = SvfgNodeId::value(ValueId::new(3));
        let sink2 = SvfgNodeId::value(ValueId::new(4));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink2);

        let forward = BTreeSet::from([source, mid]);
        let sinks = BTreeSet::from([sink1, sink2]);

        let bslice = backward_slice(&g, &forward, &sinks);
        assert!(bslice.contains(&source));
        assert!(bslice.contains(&mid));
        assert!(bslice.contains(&sink1));
        assert!(bslice.contains(&sink2));
    }

    // ---- all_path_reachable_solve tests ----

    #[cfg(feature = "z3-solver")]
    #[test]
    fn all_path_reachable_linear_no_guards() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink = SvfgNodeId::value(ValueId::new(2));
        g.add_edge(source, SvfgEdgeKind::DirectDef, sink);

        let bslice = BTreeSet::from([source, sink]);
        let sinks = BTreeSet::from([sink]);

        let vi = crate::guard::ValueLocationIndex::from_conditions(vec![]);
        let cfgs = BTreeMap::new();
        assert!(all_path_reachable_solve(
            &g, source, &bslice, &sinks, &vi, &cfgs
        ));
    }

    #[cfg(feature = "z3-solver")]
    #[test]
    fn all_path_reachable_both_branches_reach_sink() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink1 = SvfgNodeId::value(ValueId::new(2));
        let sink2 = SvfgNodeId::value(ValueId::new(3));
        let cond = ValueId::new(100);

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(source, SvfgEdgeKind::DirectDef, sink2);

        let block = BlockId::new(1);
        let func = FunctionId::new(1);
        g.set_edge_guard(
            source,
            sink1,
            vec![Guard {
                block,
                function: func,
                condition: cond,
                branch_taken: true,
            }],
        );
        g.set_edge_guard(
            source,
            sink2,
            vec![Guard {
                block,
                function: func,
                condition: cond,
                branch_taken: false,
            }],
        );

        let bslice = BTreeSet::from([source, sink1, sink2]);
        let sinks = BTreeSet::from([sink1, sink2]);

        let vi = crate::guard::ValueLocationIndex::from_conditions(vec![]);
        let cfgs = BTreeMap::new();
        assert!(all_path_reachable_solve(
            &g, source, &bslice, &sinks, &vi, &cfgs
        ));
    }

    #[cfg(feature = "z3-solver")]
    #[test]
    fn all_path_reachable_one_branch_misses_sink() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink = SvfgNodeId::value(ValueId::new(2));
        let dead = SvfgNodeId::value(ValueId::new(3));
        let cond = ValueId::new(100);

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink);
        g.add_edge(source, SvfgEdgeKind::DirectDef, dead);

        let block = BlockId::new(1);
        let func = FunctionId::new(1);
        g.set_edge_guard(
            source,
            sink,
            vec![Guard {
                block,
                function: func,
                condition: cond,
                branch_taken: true,
            }],
        );
        g.set_edge_guard(
            source,
            dead,
            vec![Guard {
                block,
                function: func,
                condition: cond,
                branch_taken: false,
            }],
        );

        // backward slice only has source->sink (dead is NOT a sink)
        let bslice = BTreeSet::from([source, sink]);
        let sinks = BTreeSet::from([sink]);

        let vi = crate::guard::ValueLocationIndex::from_conditions(vec![]);
        let cfgs = BTreeMap::new();
        assert!(!all_path_reachable_solve(
            &g, source, &bslice, &sinks, &vi, &cfgs
        ));
    }

    #[cfg(feature = "z3-solver")]
    #[test]
    fn all_path_reachable_no_guards_means_true() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));
        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink);

        let bslice = BTreeSet::from([source, mid, sink]);
        let sinks = BTreeSet::from([sink]);

        let vi = crate::guard::ValueLocationIndex::from_conditions(vec![]);
        let cfgs = BTreeMap::new();
        assert!(all_path_reachable_solve(
            &g, source, &bslice, &sinks, &vi, &cfgs
        ));
    }

    // ---- detect_partial_leaks_svfg tests ----

    #[test]
    fn detect_partial_leaks_svfg_reports_partial() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink = SvfgNodeId::value(ValueId::new(2));
        let dead = SvfgNodeId::value(ValueId::new(3));
        let cond = ValueId::new(100);

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink);
        g.add_edge(source, SvfgEdgeKind::DirectDef, dead);

        let block = BlockId::new(1);
        let func_id = FunctionId::new(1);
        g.set_edge_guard(
            source,
            sink,
            vec![Guard {
                block,
                function: func_id,
                condition: cond,
                branch_taken: true,
            }],
        );
        g.set_edge_guard(
            source,
            dead,
            vec![Guard {
                block,
                function: func_id,
                condition: cond,
                branch_taken: false,
            }],
        );

        let spec = test_spec_never_reach_sink();
        let sink_nodes = BTreeSet::from([sink]);

        let reachable = vec![SourceReachability {
            source,
            forward_slice: BTreeSet::from([source, dead]),
            reached_sinks: BTreeSet::from([sink]),
        }];

        let module = empty_test_module();
        let findings = detect_partial_leaks_svfg(&g, &spec, &reachable, &sink_nodes, &module);
        assert_eq!(findings.len(), 1, "Should detect partial leak");
        assert!(findings[0].message.contains("partial leak"));
    }

    #[test]
    fn detect_partial_leaks_svfg_no_report_when_all_paths() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink1 = SvfgNodeId::value(ValueId::new(2));
        let sink2 = SvfgNodeId::value(ValueId::new(3));
        let cond = ValueId::new(100);

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
        g.add_edge(source, SvfgEdgeKind::DirectDef, sink2);

        let block = BlockId::new(1);
        let func_id = FunctionId::new(1);
        g.set_edge_guard(
            source,
            sink1,
            vec![Guard {
                block,
                function: func_id,
                condition: cond,
                branch_taken: true,
            }],
        );
        g.set_edge_guard(
            source,
            sink2,
            vec![Guard {
                block,
                function: func_id,
                condition: cond,
                branch_taken: false,
            }],
        );

        let spec = test_spec_never_reach_sink();
        let sink_nodes = BTreeSet::from([sink1, sink2]);

        let reachable = vec![SourceReachability {
            source,
            forward_slice: BTreeSet::from([source]),
            reached_sinks: BTreeSet::from([sink1, sink2]),
        }];

        let module = empty_test_module();
        let findings = detect_partial_leaks_svfg(&g, &spec, &reachable, &sink_nodes, &module);
        assert!(findings.is_empty(), "All paths reach sinks → safe");
    }

    #[cfg(not(feature = "z3-solver"))]
    #[test]
    fn all_path_reachable_solve_bdd_cycle_converges_when_state_stable() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let mid = SvfgNodeId::value(ValueId::new(2));
        let sink = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, source);
        g.add_edge(mid, SvfgEdgeKind::DirectDef, sink);

        let backward_slice = BTreeSet::from([source, mid, sink]);
        let reached_sinks = BTreeSet::from([sink]);
        let vars = BddVariableSet::new_anonymous(0);
        let cond_vars = BTreeMap::new();
        let value_index = crate::guard::ValueLocationIndex::from_conditions(vec![]);
        let cfgs = BTreeMap::new();

        assert!(
            all_path_reachable_solve_bdd(
                &g,
                source,
                &backward_slice,
                &reached_sinks,
                &value_index,
                &cfgs,
                &vars,
                &cond_vars,
            ),
            "stable BDD states in a cycle should converge instead of hitting max_iterations"
        );
    }

    #[cfg(not(feature = "z3-solver"))]
    #[test]
    fn build_partial_leak_bdd_env_limit_overflow_returns_none() {
        let mut g = Svfg::new();
        let source = SvfgNodeId::value(ValueId::new(1));
        let sink = SvfgNodeId::value(ValueId::new(2));

        g.add_edge(source, SvfgEdgeKind::DirectDef, sink);
        g.set_edge_guard(
            source,
            sink,
            vec![
                Guard {
                    block: BlockId::new(1),
                    function: FunctionId::new(1),
                    condition: ValueId::new(100),
                    branch_taken: true,
                },
                Guard {
                    block: BlockId::new(2),
                    function: FunctionId::new(1),
                    condition: ValueId::new(101),
                    branch_taken: false,
                },
            ],
        );

        let reachable = vec![SourceReachability {
            source,
            forward_slice: BTreeSet::from([source]),
            reached_sinks: BTreeSet::from([sink]),
        }];
        let module = empty_test_module();
        let value_index = crate::guard::ValueLocationIndex::build(&module);

        assert!(
            build_partial_leak_bdd_env(&g, &reachable, &module, &value_index, 2).is_none(),
            "overflowing the BDD guard budget should return None instead of forcing leak findings"
        );
    }
}
