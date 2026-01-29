//! Checker runner: orchestrates checker specs, site classification, and solvers.
//!
//! The runner is the main entry point for running checkers. It:
//! 1. Classifies call sites against the resource table
//! 2. Resolves `SitePattern`s to concrete SVFG node sets
//! 3. Dispatches to the appropriate solver (`may_reach`, `must_not_reach`, or `never_reach_sink`)
//! 4. Collects and returns findings

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, ValueId};

use crate::svfg::Svfg;

use super::finding::CheckerFinding;
use super::resource_table::{ResourceRole, ResourceTable};
use super::site_classifier::{self, ClassifiedSites};
use super::solver::{self, GuardedSolverConfig, SolverConfig};
use super::spec::{CheckerSpec, ReachabilityMode, SitePattern};
use crate::svfg::SvfgNodeId;

// ---------------------------------------------------------------------------
// CheckerResult
// ---------------------------------------------------------------------------

/// Result of running one or more checkers.
#[derive(Debug, Clone)]
pub struct CheckerResult {
    /// All findings.
    pub findings: Vec<CheckerFinding>,
    /// Diagnostics.
    pub diagnostics: CheckerDiagnostics,
}

/// Diagnostics from checker execution.
#[derive(Debug, Clone, Default)]
pub struct CheckerDiagnostics {
    /// Number of checkers run.
    pub checkers_run: usize,
    /// Number of classified call sites.
    pub classified_sites: usize,
    /// Number of source nodes found.
    pub source_nodes: usize,
    /// Number of sink nodes found.
    pub sink_nodes: usize,
    /// Number of sanitizer nodes found.
    pub sanitizer_nodes: usize,
    /// Total findings.
    pub total_findings: usize,
}

/// Context for guard-aware (path-sensitive) checker execution.
/// When provided, `MayReach` checkers use `may_reach_guarded` instead
/// of `may_reach`, enabling SCCP dead-block pruning and guard contradiction detection.
#[derive(Debug, Clone)]
pub struct GuardContext {
    /// Blocks proven unreachable by SCCP.
    pub dead_blocks: BTreeSet<BlockId>,
    /// Maps each `ValueId` to the `BlockId` of its defining instruction.
    pub block_of: BTreeMap<ValueId, BlockId>,
    /// Guard-aware solver configuration.
    pub config: GuardedSolverConfig,
}

// ---------------------------------------------------------------------------
// run_checker / run_checkers
// ---------------------------------------------------------------------------

/// Run a single checker against an SVFG.
pub fn run_checker(
    spec: &CheckerSpec,
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    config: &SolverConfig,
) -> CheckerResult {
    run_checkers(std::slice::from_ref(spec), module, svfg, table, config)
}

/// Run multiple checkers against an SVFG.
#[allow(clippy::too_many_lines)]
pub fn run_checkers(
    specs: &[CheckerSpec],
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    config: &SolverConfig,
) -> CheckerResult {
    // Step 1: Classify sites
    let classified = site_classifier::classify(module, table, svfg);

    // Build node-to-function map. Used for:
    // - Reachability filtering (source/sink scoping to main-reachable functions)
    // - MustNotReach exit scoping (only match exits in the source's own function)
    let node_to_func: BTreeMap<SvfgNodeId, FunctionId> = build_node_to_func_map(module, svfg);

    // Augmented config with the node-to-function map for exit scoping.
    let scoped_config = SolverConfig {
        node_to_func: Some(node_to_func.clone()),
        ..config.clone()
    };

    let mut all_findings = Vec::new();
    let mut diagnostics = CheckerDiagnostics {
        checkers_run: specs.len(),
        classified_sites: classified.len(),
        ..Default::default()
    };

    // Step 2: For each checker spec, resolve patterns to node sets and run solver
    for spec in specs {
        let mut source_nodes = resolve_patterns(&spec.sources, &classified, svfg);
        let mut sink_nodes = resolve_sink_patterns(&spec.sinks, &classified, svfg, &source_nodes);
        let sanitizer_nodes = resolve_patterns(&spec.sanitizers, &classified, svfg);

        // Filter sources and sinks to reachable functions only.
        if let Some(reachable) = &config.reachable_functions {
            source_nodes.retain(|n| {
                node_to_func
                    .get(n)
                    .is_some_and(|fid| reachable.contains(fid))
            });
            sink_nodes.retain(|n| {
                node_to_func
                    .get(n)
                    .is_some_and(|fid| reachable.contains(fid))
            });
        }

        diagnostics.source_nodes += source_nodes.len();
        diagnostics.sink_nodes += sink_nodes.len();
        diagnostics.sanitizer_nodes += sanitizer_nodes.len();

        let findings = match spec.mode {
            ReachabilityMode::MayReach => {
                let sanitizer_set: BTreeSet<SvfgNodeId> = sanitizer_nodes.into_iter().collect();

                // Detect source-sink overlap: when a source node IS also a
                // sink (e.g., stack-escape where alloca value is directly
                // returned). The solver excludes `target == source`, so
                // these must be reported separately.
                //
                // For null-deref: skip null constant overlaps. When a null
                // constant is both a `NullConstant` source and a deref sink
                // (e.g., `load i8, ptr null`), the overlap bypass creates
                // false positives on guarded code. The `direct_null_derefs`
                // mechanism handles these with proper guarded-block awareness.
                let null_source_values = classified.null_source_values();
                let is_null_deref = spec.name == "null-deref";
                let mut findings: Vec<CheckerFinding> = source_nodes
                    .iter()
                    .filter(|s| {
                        sink_nodes.contains(s)
                            && !sanitizer_set.contains(s)
                            && !(is_null_deref && null_source_values.contains(s))
                    })
                    .map(|&source| CheckerFinding {
                        checker_name: spec.name.clone(),
                        severity: spec.severity,
                        source_node: source,
                        sink_node: source,
                        trace: vec![source],
                        cwe: spec.cwe,
                        message: format!(
                            "{}: {} (source value directly used as sink)",
                            spec.name, spec.description
                        ),
                        sink_traces: vec![],
                        source_kind: super::finding::NullSourceKind::default(),
                    })
                    .collect();

                findings.extend(solver::may_reach(
                    svfg,
                    spec,
                    &source_nodes,
                    &sink_nodes,
                    &sanitizer_set,
                    config,
                ));
                findings
            }
            ReachabilityMode::MustNotReach => {
                // Filter out wrapper-internal HeapAlloc sources: if a HeapAlloc
                // (e.g., malloc) is inside a function that is itself an Allocator
                // (e.g., SAFEMALLOC), the internal malloc is not a real leak source —
                // the wrapper's job is to allocate and return to caller.
                let filtered_sources =
                    filter_wrapper_internal_sources(&source_nodes, &classified, module, table);

                // For MustNotReach, "sinks" are exits, "sanitizers" are cleanups.
                // Use `scoped_config` which includes `node_to_func` so the solver
                // only matches exits in the source's own function — callee returns
                // are interprocedural flow points, not leak exits.
                let exit_nodes = resolve_exit_patterns(&spec.sinks, &classified, svfg);
                let sanitizer_set: BTreeSet<_> = sanitizer_nodes.into_iter().collect();
                solver::must_not_reach(
                    svfg,
                    spec,
                    &filtered_sources,
                    &exit_nodes,
                    &sanitizer_set,
                    &scoped_config,
                )
            }
            ReachabilityMode::MultiReach => {
                // Standard solver: handles case with distinct SVFG sink nodes
                let mut findings =
                    solver::multi_reach(svfg, spec, &source_nodes, &sink_nodes, config);

                // Supplementary check: detect same-SSA double-free where 2+
                // call sites map to the same SVFG node (deduplicated by
                // BTreeSet, so the standard solver sees only 1 sink).
                if findings.is_empty() {
                    let extra = detect_same_node_multi_sink(
                        spec,
                        &spec.sinks,
                        &classified,
                        svfg,
                        &source_nodes,
                    );
                    findings.extend(extra);
                }
                findings
            }
            ReachabilityMode::NeverReachSink => {
                let filtered_sources =
                    filter_wrapper_internal_sources(&source_nodes, &classified, module, table);
                // Phase 1: Forward BFS — classifies into NEVERFREE vs reachable
                let fwd_result = solver::forward_bfs_enriched(
                    svfg,
                    spec,
                    &filtered_sources,
                    &sink_nodes,
                    config,
                );
                let mut findings = fwd_result.neverfree_findings;
                // Phase 2+3: For reachable sources, check partial leaks via
                // SVFG backward slice + Z3 guard tautology check.
                let partial = solver::detect_partial_leaks_svfg(
                    svfg,
                    spec,
                    &fwd_result.reachable_sources,
                    &sink_nodes,
                    module,
                );
                findings.extend(partial);
                findings
            }
        };

        // Post-process: classify null-deref source kinds and add direct null derefs
        let findings = postprocess_null_deref_findings(findings, spec, &classified);

        // Post-process: refine UAF findings using temporal ordering
        let findings = refine_uaf_findings(findings, spec, module, table);

        all_findings.extend(findings);
    }

    diagnostics.total_findings = all_findings.len();

    CheckerResult {
        findings: all_findings,
        diagnostics,
    }
}

/// Run multiple checkers with guard-aware BFS for `MayReach` specs.
///
/// When the SVFG has edge guards, this uses `may_reach_guarded` which
/// prunes infeasible paths via guard contradiction detection and skips
/// SCCP-proven dead blocks.
#[allow(clippy::too_many_lines)]
pub fn run_checkers_guarded(
    specs: &[CheckerSpec],
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    config: &SolverConfig,
    guard_ctx: &GuardContext,
) -> CheckerResult {
    // Step 1: Classify sites
    let classified = site_classifier::classify(module, table, svfg);

    // Build node-to-function map for reachability and exit scoping.
    let node_to_func: BTreeMap<SvfgNodeId, FunctionId> = build_node_to_func_map(module, svfg);

    // Augmented config with the node-to-function map for exit scoping.
    let scoped_config = SolverConfig {
        node_to_func: Some(node_to_func.clone()),
        ..config.clone()
    };

    let mut all_findings = Vec::new();
    let mut diagnostics = CheckerDiagnostics {
        checkers_run: specs.len(),
        classified_sites: classified.len(),
        ..Default::default()
    };

    // Step 2: For each checker spec, resolve patterns to node sets and run solver
    for spec in specs {
        let mut source_nodes = resolve_patterns(&spec.sources, &classified, svfg);
        let mut sink_nodes = resolve_sink_patterns(&spec.sinks, &classified, svfg, &source_nodes);
        let sanitizer_nodes = resolve_patterns(&spec.sanitizers, &classified, svfg);

        // Filter sources and sinks to reachable functions only.
        if let Some(reachable) = &config.reachable_functions {
            source_nodes.retain(|n| {
                node_to_func
                    .get(n)
                    .is_some_and(|fid| reachable.contains(fid))
            });
            sink_nodes.retain(|n| {
                node_to_func
                    .get(n)
                    .is_some_and(|fid| reachable.contains(fid))
            });
        }

        diagnostics.source_nodes += source_nodes.len();
        diagnostics.sink_nodes += sink_nodes.len();
        diagnostics.sanitizer_nodes += sanitizer_nodes.len();

        let findings = match spec.mode {
            ReachabilityMode::MayReach => {
                let sanitizer_set: BTreeSet<SvfgNodeId> = sanitizer_nodes.into_iter().collect();

                // Detect source-sink overlap (same as non-guarded path)
                let mut findings: Vec<CheckerFinding> = source_nodes
                    .iter()
                    .filter(|s| sink_nodes.contains(s) && !sanitizer_set.contains(s))
                    .map(|&source| CheckerFinding {
                        checker_name: spec.name.clone(),
                        severity: spec.severity,
                        source_node: source,
                        sink_node: source,
                        trace: vec![source],
                        cwe: spec.cwe,
                        message: format!(
                            "{}: {} (source value directly used as sink)",
                            spec.name, spec.description
                        ),
                        sink_traces: vec![],
                        source_kind: super::finding::NullSourceKind::default(),
                    })
                    .collect();

                // Use guard-aware BFS when guard context is available
                findings.extend(solver::may_reach_guarded(
                    svfg,
                    spec,
                    &source_nodes,
                    &sink_nodes,
                    &sanitizer_set,
                    &guard_ctx.config,
                    &guard_ctx.dead_blocks,
                    &guard_ctx.block_of,
                ));
                findings
            }
            ReachabilityMode::MustNotReach => {
                let filtered_sources =
                    filter_wrapper_internal_sources(&source_nodes, &classified, module, table);
                let exit_nodes = resolve_exit_patterns(&spec.sinks, &classified, svfg);
                let sanitizer_set: BTreeSet<_> = sanitizer_nodes.into_iter().collect();
                solver::must_not_reach(
                    svfg,
                    spec,
                    &filtered_sources,
                    &exit_nodes,
                    &sanitizer_set,
                    &scoped_config,
                )
            }
            ReachabilityMode::MultiReach => {
                let mut findings =
                    solver::multi_reach(svfg, spec, &source_nodes, &sink_nodes, config);
                if findings.is_empty() {
                    let extra = detect_same_node_multi_sink(
                        spec,
                        &spec.sinks,
                        &classified,
                        svfg,
                        &source_nodes,
                    );
                    findings.extend(extra);
                }
                findings
            }
            ReachabilityMode::NeverReachSink => {
                let filtered_sources =
                    filter_wrapper_internal_sources(&source_nodes, &classified, module, table);
                // Phase 1: Forward BFS — classifies into NEVERFREE vs reachable
                let fwd_result = solver::forward_bfs_enriched(
                    svfg,
                    spec,
                    &filtered_sources,
                    &sink_nodes,
                    config,
                );
                let mut findings = fwd_result.neverfree_findings;
                // Phase 2+3: For reachable sources, check partial leaks via
                // SVFG backward slice + Z3 guard tautology check.
                let partial = solver::detect_partial_leaks_svfg(
                    svfg,
                    spec,
                    &fwd_result.reachable_sources,
                    &sink_nodes,
                    module,
                );
                findings.extend(partial);
                findings
            }
        };

        // Post-process: classify null-deref source kinds and add direct null derefs
        let findings = postprocess_null_deref_findings(findings, spec, &classified);

        // Post-process: refine UAF findings using temporal ordering
        let findings = refine_uaf_findings(findings, spec, module, table);

        all_findings.extend(findings);
    }

    diagnostics.total_findings = all_findings.len();

    CheckerResult {
        findings: all_findings,
        diagnostics,
    }
}

// ---------------------------------------------------------------------------
// Null-deref post-processing
// ---------------------------------------------------------------------------

/// Post-process checker findings for null-deref classification.
///
/// Classifies null-deref sources as explicit NULL or function-may-return-null,
/// and adds direct null dereference findings not reachable via SVFG.
fn postprocess_null_deref_findings(
    findings: Vec<CheckerFinding>,
    spec: &CheckerSpec,
    classified: &ClassifiedSites,
) -> Vec<CheckerFinding> {
    if spec.name != "null-deref" {
        return findings;
    }

    let mut result: Vec<_> = findings
        .into_iter()
        .map(|mut f| {
            // Check if source node is an explicit NULL constant value
            if classified.null_source_values().contains(&f.source_node) {
                f.source_kind = super::finding::NullSourceKind::ExplicitNull;
            } else {
                // Check if source matches a NullSource role (e.g., malloc return)
                let null_source_returns =
                    classified.return_nodes_for_role(ResourceRole::NullSource);
                if null_source_returns.contains(&f.source_node) {
                    f.source_kind = super::finding::NullSourceKind::FunctionMayReturnNull;
                }
            }
            f
        })
        .collect();

    // Add definite null-deref findings: instructions that literally
    // dereference null (Load/Store/GEP with null operand) in reachable code.
    // These bypass the solver because SVFG doesn't create pointer->result edges.
    let existing_sinks: BTreeSet<SvfgNodeId> = result.iter().map(|f| f.sink_node).collect();
    for &deref_node in classified.direct_null_derefs() {
        if !existing_sinks.contains(&deref_node) {
            // Find any null source to use as the source node
            let source_node = classified
                .null_source_values()
                .iter()
                .next()
                .copied()
                .unwrap_or(deref_node);
            result.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node,
                sink_node: deref_node,
                trace: vec![source_node, deref_node],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (direct null pointer dereference)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::ExplicitNull,
            });
        }
    }

    result
}

// ---------------------------------------------------------------------------
// UAF temporal refinement
// ---------------------------------------------------------------------------

/// Refine UAF findings using temporal ordering.
///
/// The SVFG is a pure data-flow graph: forward traversal from `free(ptr)`'s
/// argument reaches ALL uses of `ptr`, regardless of control-flow ordering.
/// This produces two problems:
///   1. Pre-free dereferences are reported as UAF (false positives)
///   2. Post-free dereferences that share the same SVFG node as the source
///      are blocked by the solver's `target != source` guard (false negatives)
///
/// This refinement replaces the solver's findings with precise per-instruction
/// findings that only include dereferences reachable AFTER the `free()` call.
///
/// For non-UAF checkers, returns findings unchanged.
// NOTE: This function implements UAF temporal refinement as a single cohesive
// unit: collect freed sources → build program points/CFGs → derive transitive
// pointer values → scan for post-free uses. Splitting would fragment the logic.
#[allow(clippy::too_many_lines)]
fn refine_uaf_findings(
    findings: Vec<CheckerFinding>,
    spec: &CheckerSpec,
    module: &AirModule,
    table: &ResourceTable,
) -> Vec<CheckerFinding> {
    use crate::cfg::Cfg;
    use crate::svfg::{ProgramPoint, ProgramPointMap};

    if spec.name != "use-after-free" || findings.is_empty() {
        return findings;
    }

    // Collect all source nodes (freed pointer values) from existing findings
    let freed_sources: BTreeSet<_> = findings
        .iter()
        .filter_map(|f| f.source_node.as_value())
        .collect();

    if freed_sources.is_empty() {
        return findings;
    }

    // Build program point map
    let mut program_points = ProgramPointMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let pp = ProgramPoint::new(func.id, block.id, inst_idx as u32);
                if let Some(dst) = inst.dst {
                    program_points.insert(dst, pp);
                }
                for &operand in &inst.operands {
                    program_points.insert(operand, pp);
                }
            }
        }
    }

    // Build CFGs
    let cfgs: BTreeMap<FunctionId, Cfg> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (f.id, Cfg::build(f)))
        .collect();

    // Pre-build callee name lookup
    let func_names: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    // For each freed source, find all post-free dereferences
    let mut refined = Vec::new();

    for &src_vid in &freed_sources {
        let Some(src_pp) = program_points.get(src_vid) else {
            continue;
        };
        // Find the free() call point
        let Some(free_pp) =
            find_free_call_point(src_vid, src_pp.function, module, table, &func_names)
        else {
            // Can't find the free call — keep original findings for this source
            refined.extend(
                findings
                    .iter()
                    .filter(|f| f.source_node.as_value() == Some(src_vid))
                    .cloned(),
            );
            continue;
        };

        // Scan for dereference instructions that use src_vid or any value
        // derived from it (via GEP, PHI, Copy/Bitcast) AFTER the free call.
        let Some(func) = module.functions.iter().find(|f| f.id == src_pp.function) else {
            continue;
        };

        // Build the set of all values derived from the freed pointer.
        // Start with src_vid, then transitively follow GEP/PHI/Copy/Bitcast.
        let mut derived: BTreeSet<ValueId> = BTreeSet::new();
        derived.insert(src_vid);
        let mut changed = true;
        while changed {
            changed = false;
            for block in &func.blocks {
                for inst in &block.instructions {
                    let Some(dst) = inst.dst else { continue };
                    if derived.contains(&dst) {
                        continue;
                    }
                    let produces_derived = match &inst.op {
                        // GEP base is first operand
                        Operation::Gep { .. } => {
                            inst.operands.first().is_some_and(|v| derived.contains(v))
                        }
                        // PHI: if any incoming value is derived
                        Operation::Phi { incoming } => {
                            incoming.iter().any(|(_, v)| derived.contains(v))
                        }
                        // Copy/Bitcast: operand is derived
                        Operation::Cast { .. } | Operation::Copy => {
                            inst.operands.first().is_some_and(|v| derived.contains(v))
                        }
                        _ => false,
                    };
                    if produces_derived {
                        derived.insert(dst);
                        changed = true;
                    }
                }
            }
        }

        let source_node = SvfgNodeId::Value(src_vid);

        'outer: for block in &func.blocks {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                // Check if this instruction uses a derived value in a way
                // that constitutes a dereference or use-after-free.
                let is_uaf_use = match &inst.op {
                    // load from derived pointer
                    Operation::Load => inst.operands.first().is_some_and(|v| derived.contains(v)),
                    // store through derived pointer (address is operand[1])
                    Operation::Store => inst.operands.get(1).is_some_and(|v| derived.contains(v)),
                    // Passing freed/derived pointer to a function = UAF
                    // (the callee will dereference it)
                    Operation::CallDirect { callee } => {
                        // Don't count the free() call itself as a UAF use
                        let is_free = func_names
                            .get(callee)
                            .is_some_and(|n| table.has_role(n, ResourceRole::Deallocator));
                        !is_free && inst.operands.iter().any(|v| derived.contains(v))
                    }
                    Operation::CallIndirect { .. } => {
                        // Skip function pointer (operand 0), check args
                        inst.operands.iter().skip(1).any(|v| derived.contains(v))
                    }
                    _ => false,
                };

                if !is_uaf_use {
                    continue;
                }

                // Check temporal ordering: can this instruction happen after free?
                #[allow(clippy::cast_possible_truncation)]
                let deref_pp = ProgramPoint::new(func.id, block.id, inst_idx as u32);
                if program_points.can_happen_after(free_pp, deref_pp, &cfgs) {
                    let sink_node = inst.dst.map_or(source_node, SvfgNodeId::Value);
                    refined.push(CheckerFinding {
                        checker_name: spec.name.clone(),
                        severity: spec.severity,
                        source_node,
                        sink_node,
                        trace: vec![source_node, sink_node],
                        cwe: spec.cwe,
                        message: format!(
                            "{}: {} (source → sink reachable on SVFG)",
                            spec.name, spec.description
                        ),
                        sink_traces: vec![],
                        source_kind: super::finding::NullSourceKind::default(),
                    });
                    // One finding per source is enough
                    break 'outer;
                }
            }
        }
    }

    refined
}

/// Find the program point of the `free()` call that consumes `freed_value`.
fn find_free_call_point(
    freed_value: ValueId,
    func_id: FunctionId,
    module: &AirModule,
    table: &ResourceTable,
    func_names: &BTreeMap<FunctionId, &str>,
) -> Option<crate::svfg::ProgramPoint> {
    let func = module.functions.iter().find(|f| f.id == func_id)?;

    for block in &func.blocks {
        for (idx, inst) in block.instructions.iter().enumerate() {
            if let Operation::CallDirect { callee } = &inst.op {
                if inst.operands.contains(&freed_value) {
                    if let Some(callee_name) = func_names.get(callee) {
                        if table.has_role(callee_name, ResourceRole::Deallocator) {
                            // INVARIANT: Juliet/SV-COMP functions have < 2^32
                            // instructions per block
                            #[allow(clippy::cast_possible_truncation)]
                            return Some(crate::svfg::ProgramPoint::new(
                                func_id, block.id, idx as u32,
                            ));
                        }
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Pattern resolution
// ---------------------------------------------------------------------------

/// Resolve a list of `SitePattern`s to SVFG node IDs.
///
/// Returns a Vec of source/sanitizer nodes.
fn resolve_patterns(
    patterns: &[SitePattern],
    classified: &ClassifiedSites,
    _svfg: &Svfg,
) -> Vec<SvfgNodeId> {
    let mut nodes = Vec::new();

    for pattern in patterns {
        match pattern {
            SitePattern::Role { role, match_return } => {
                if *match_return {
                    nodes.extend(classified.return_nodes_for_role(*role));
                } else {
                    nodes.extend(classified.first_arg_nodes_for_role(*role));
                }
            }
            SitePattern::FunctionName { name, match_return } => {
                for site in classified.all() {
                    if site.callee_name == *name {
                        if *match_return {
                            if let Some(ret) = site.return_value {
                                nodes.push(ret);
                            }
                        } else if let Some(arg) = site.arguments.first() {
                            nodes.push(*arg);
                        }
                    }
                }
            }
            SitePattern::AllocaInst => {
                nodes.extend(classified.alloca_values());
            }
            SitePattern::FunctionExit => {
                nodes.extend(classified.function_exits());
            }
            SitePattern::AnyUseOf | SitePattern::CustomPredicate { .. } => {
                // AnyUseOf is special — handled differently for sinks
                // (it means "any SVFG node that uses the source value")
                // CustomPredicate is resolved at runtime by external runners
                // For source/sanitizer patterns both are no-ops
            }
            SitePattern::LoadDeref => {
                nodes.extend(classified.load_deref_pointers());
            }
            SitePattern::StoreDeref => {
                nodes.extend(classified.store_deref_pointers());
            }
            SitePattern::GepDeref => {
                nodes.extend(classified.gep_deref_pointers());
            }
            SitePattern::NullConstant => {
                nodes.extend(classified.null_source_values());
            }
            SitePattern::DirectNullDeref => {
                nodes.extend(classified.direct_null_derefs());
            }
            SitePattern::NullCheckBranch => {
                // For sanitizers: return nodes that are guarded by null checks
                nodes.extend(classified.null_check_guarded());
            }
        }
    }

    // Deduplicate
    nodes.sort();
    nodes.dedup();
    nodes
}

/// Resolve sink patterns to a set of SVFG node IDs.
///
/// `AnyUseOf` is special: it means any SVFG successor of a source node
/// is a potential sink. For may_reach, we collect all forward-reachable
/// nodes from sources and mark them as potential sinks.
fn resolve_sink_patterns(
    patterns: &[SitePattern],
    classified: &ClassifiedSites,
    svfg: &Svfg,
    source_nodes: &[SvfgNodeId],
) -> BTreeSet<SvfgNodeId> {
    let mut nodes = BTreeSet::new();
    let mut has_any_use_of = false;

    for pattern in patterns {
        match pattern {
            SitePattern::Role { role, match_return } => {
                if *match_return {
                    for node in classified.return_nodes_for_role(*role) {
                        nodes.insert(node);
                    }
                } else {
                    for node in classified.first_arg_nodes_for_role(*role) {
                        nodes.insert(node);
                    }
                }
            }
            SitePattern::FunctionName { name, match_return } => {
                for site in classified.all() {
                    if site.callee_name == *name {
                        if *match_return {
                            if let Some(ret) = site.return_value {
                                nodes.insert(ret);
                            }
                        } else if let Some(arg) = site.arguments.first() {
                            nodes.insert(*arg);
                        }
                    }
                }
            }
            SitePattern::FunctionExit => {
                nodes.extend(classified.function_exits());
                nodes.extend(classified.ret_values());
            }
            SitePattern::AllocaInst => {
                nodes.extend(classified.alloca_values());
            }
            SitePattern::AnyUseOf => {
                has_any_use_of = true;
            }
            SitePattern::LoadDeref => {
                nodes.extend(classified.load_deref_pointers());
            }
            SitePattern::StoreDeref => {
                nodes.extend(classified.store_deref_pointers());
            }
            SitePattern::GepDeref => {
                nodes.extend(classified.gep_deref_pointers());
            }
            SitePattern::NullConstant => {
                nodes.extend(classified.null_source_values());
            }
            SitePattern::DirectNullDeref => {
                nodes.extend(classified.direct_null_derefs());
            }
            SitePattern::NullCheckBranch => {
                // NullCheckBranch is a sanitizer pattern, not typically a sink
                // But handle it here for completeness
                nodes.extend(classified.null_check_guarded());
            }
            SitePattern::CustomPredicate { .. } => {
                // Custom predicates are resolved at runtime by external runners
            }
        }
    }

    // AnyUseOf: collect all nodes in the SVFG that are forward-reachable
    // from any source, excluding the source itself. This allows the solver
    // to detect any use of the freed/allocated pointer.
    if has_any_use_of {
        for &source in source_nodes {
            if let Some(succs) = svfg.successors_of(source) {
                for (_, target) in succs {
                    nodes.insert(*target);
                }
            }
            // Also include all nodes reachable from source's successors
            let reachable = svfg.forward_reachable(source);
            for node in reachable {
                if node != source {
                    nodes.insert(node);
                }
            }
        }
    }

    nodes
}

/// Resolve exit patterns specifically (for `must_not_reach` sinks).
fn resolve_exit_patterns(
    patterns: &[SitePattern],
    classified: &ClassifiedSites,
    _svfg: &Svfg,
) -> BTreeSet<SvfgNodeId> {
    let mut nodes = BTreeSet::new();

    for pattern in patterns {
        match pattern {
            SitePattern::FunctionExit => {
                nodes.extend(classified.function_exits());
                nodes.extend(classified.ret_values());
            }
            // For other patterns, fall through to regular resolution
            SitePattern::Role { role, match_return } => {
                if *match_return {
                    for node in classified.return_nodes_for_role(*role) {
                        nodes.insert(node);
                    }
                } else {
                    for node in classified.first_arg_nodes_for_role(*role) {
                        nodes.insert(node);
                    }
                }
            }
            _ => {} // Other patterns less relevant for exits
        }
    }

    nodes
}

// ---------------------------------------------------------------------------
// Same-node multi-sink detection (for MultiReach / double-free)
// ---------------------------------------------------------------------------

/// Detect double-free when 2+ call sites share the same argument `SvfgNodeId`.
///
/// The standard `multi_reach` solver counts distinct SVFG sink nodes. When
/// two `free(p)` calls use the same SSA value `p`, both map to the same
/// `SvfgNodeId` and the solver sees only 1 sink. This function counts call
/// sites per argument node and reports when 2+ sites share an argument that
/// is reachable from a source.
fn detect_same_node_multi_sink(
    spec: &CheckerSpec,
    sink_patterns: &[SitePattern],
    classified: &ClassifiedSites,
    svfg: &Svfg,
    source_nodes: &[SvfgNodeId],
) -> Vec<CheckerFinding> {
    // Count call sites per argument/return node for each sink pattern
    let mut node_site_count: BTreeMap<SvfgNodeId, usize> = BTreeMap::new();

    for pattern in sink_patterns {
        if let SitePattern::Role { role, match_return } = pattern {
            for site in classified.with_role(*role) {
                let node = if *match_return {
                    site.return_value
                } else {
                    site.arguments.first().copied()
                };
                if let Some(n) = node {
                    *node_site_count.entry(n).or_default() += 1;
                }
            }
        }
    }

    let mut findings = Vec::new();
    for &source in source_nodes {
        if !svfg.contains_node(source) {
            continue;
        }
        let reachable = svfg.forward_reachable(source);
        for (&arg_node, &count) in &node_site_count {
            if count >= 2 && reachable.contains(&arg_node) {
                findings.push(CheckerFinding {
                    checker_name: spec.name.clone(),
                    severity: spec.severity,
                    source_node: source,
                    sink_node: arg_node,
                    trace: vec![source, arg_node],
                    cwe: spec.cwe,
                    message: format!(
                        "{}: {} (allocation argument passed to {} deallocation calls)",
                        spec.name, spec.description, count
                    ),
                    sink_traces: vec![],
                    source_kind: super::finding::NullSourceKind::default(),
                });
            }
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// Wrapper-internal source filtering
// ---------------------------------------------------------------------------

/// Filter out `HeapAlloc` sources that are inside allocator wrapper functions.
///
/// When a wrapper like `SAFEMALLOC(n) { return malloc(n); }` is compiled,
/// the classifier creates TWO Allocator sources: the internal `HeapAlloc(malloc)`
/// inside the wrapper body, and the `CallDirect(SAFEMALLOC)` at the caller.
/// The internal malloc is not a real leak source — the wrapper's purpose is to
/// allocate and return to the caller. Keeping it would generate spurious findings
/// because the wrapper's `ret` instruction is an exit node reachable without `free`.
fn filter_wrapper_internal_sources(
    source_nodes: &[SvfgNodeId],
    classified: &ClassifiedSites,
    module: &AirModule,
    table: &ResourceTable,
) -> Vec<SvfgNodeId> {
    // Build FunctionId → name lookup
    let func_names: std::collections::BTreeMap<saf_core::ids::FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    source_nodes
        .iter()
        .copied()
        .filter(|&node| {
            // Look up the ClassifiedSite that produced this source node
            let Some(site) = classified.site_for_return_node(node) else {
                return true; // Keep nodes we can't trace
            };

            // Check if this is a HeapAlloc inside an allocator wrapper:
            // The site's containing function must itself be an Allocator
            let container_name = func_names
                .get(&site.containing_function)
                .copied()
                .unwrap_or("");
            let container_is_allocator = table.has_role(container_name, ResourceRole::Allocator);

            // Keep the source UNLESS it's a HeapAlloc inside an allocator wrapper.
            // HeapAlloc sites have callee_name like "malloc", "calloc" — the internal
            // alloc kind. CallDirect sites have the wrapper name as callee_name.
            // If the callee_name differs from the container name AND the container
            // is an allocator, this is a wrapper-internal HeapAlloc → filter it out.
            if container_is_allocator && site.callee_name != container_name {
                return false; // Filter out wrapper-internal HeapAlloc
            }

            true
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Reachability helpers
// ---------------------------------------------------------------------------

/// Build a map from SVFG node → containing `FunctionId`.
///
/// Walks all functions/blocks/instructions in the module and maps each
/// `ValueId` (instruction dst and operands) to its containing function.
fn build_node_to_func_map(module: &AirModule, svfg: &Svfg) -> BTreeMap<SvfgNodeId, FunctionId> {
    let mut map = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for param in &func.params {
            let node = SvfgNodeId::Value(param.id);
            if svfg.contains_node(node) {
                map.insert(node, func.id);
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    let node = SvfgNodeId::Value(dst);
                    if svfg.contains_node(node) {
                        map.insert(node, func.id);
                    }
                }
                for &operand in &inst.operands {
                    let node = SvfgNodeId::Value(operand);
                    if svfg.contains_node(node) {
                        map.entry(node).or_insert(func.id);
                    }
                }
            }
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svfg::SvfgEdgeKind;
    use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    /// Build a minimal module + SVFG for testing the runner.
    ///
    /// Module: main() { ptr = malloc(10); free(ptr); return; }
    /// SVFG: malloc_ret --DirectDef--> free_arg --DirectDef--> ret_val
    fn make_test_setup() -> (AirModule, Svfg, ResourceTable) {
        let malloc_id = FunctionId::new(100);
        let free_id = FunctionId::new(200);
        let main_id = FunctionId::new(300);

        let ptr_val = ValueId::new(1);
        let size_val = ValueId::new(2);
        let ret_val = ValueId::new(3);

        // Module
        let mut module = AirModule::new(ModuleId::new(1));
        let malloc_fn = {
            let mut f = AirFunction::new(malloc_id, "malloc");
            f.is_declaration = true;
            f
        };
        let free_fn = {
            let mut f = AirFunction::new(free_id, "free");
            f.is_declaration = true;
            f
        };

        let malloc_call =
            Instruction::new(InstId::new(10), Operation::CallDirect { callee: malloc_id })
                .with_operands(vec![size_val])
                .with_dst(ptr_val);

        let free_call =
            Instruction::new(InstId::new(20), Operation::CallDirect { callee: free_id })
                .with_operands(vec![ptr_val]);

        let ret = Instruction::new(InstId::new(30), Operation::Ret).with_operands(vec![ret_val]);

        let entry_block = {
            let mut b = AirBlock::new(BlockId::new(1));
            b.instructions = vec![malloc_call, free_call, ret];
            b
        };

        let main_fn = {
            let mut f = AirFunction::new(main_id, "main");
            f.blocks = vec![entry_block];
            f
        };

        module.functions = vec![malloc_fn, free_fn, main_fn];

        // SVFG: connect the values
        let mut svfg = Svfg::new();
        let malloc_ret = SvfgNodeId::value(ptr_val);
        let _free_arg = SvfgNodeId::value(ptr_val); // Same value!
        let ret_node = SvfgNodeId::value(ret_val);

        svfg.add_node(malloc_ret);
        svfg.add_node(ret_node);
        svfg.add_edge(malloc_ret, SvfgEdgeKind::DirectDef, ret_node);

        let table = ResourceTable::new();

        (module, svfg, table)
    }

    #[test]
    fn run_single_checker() {
        let (module, svfg, table) = make_test_setup();
        let spec = super::super::spec::memory_leak();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert_eq!(result.diagnostics.checkers_run, 1);
        assert!(result.diagnostics.classified_sites > 0);
    }

    #[test]
    fn run_multiple_checkers() {
        let (module, svfg, table) = make_test_setup();
        let specs = super::super::spec::builtin_checkers();
        let config = SolverConfig::default();

        let result = run_checkers(&specs, &module, &svfg, &table, &config);
        assert_eq!(result.diagnostics.checkers_run, 9);
    }

    #[test]
    fn diagnostics_populated() {
        let (module, svfg, table) = make_test_setup();
        let spec = super::super::spec::memory_leak();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert!(result.diagnostics.classified_sites > 0);
        assert_eq!(result.diagnostics.checkers_run, 1);
    }

    #[test]
    fn run_with_empty_svfg() {
        let module = AirModule::new(ModuleId::new(1));
        let svfg = Svfg::new();
        let table = ResourceTable::new();
        let spec = super::super::spec::memory_leak();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert!(result.findings.is_empty());
    }

    // ---- Bug fix regression tests ----

    /// Bug 1: double_free with same SSA value.
    ///
    /// Two `free(ptr)` calls using the same ValueId map to the same
    /// `SvfgNodeId`. The MultiReach solver's BTreeSet deduplication
    /// reduces them to 1 sink, missing the double-free. The fix counts
    /// call sites per argument node.
    #[test]
    fn double_free_same_ssa_detected() {
        // Module: main() { ptr = malloc(10); free(ptr); free(ptr); return; }
        let malloc_id = FunctionId::new(100);
        let free_id = FunctionId::new(200);
        let main_id = FunctionId::new(300);

        let ptr_val = ValueId::new(1);
        let size_val = ValueId::new(2);
        let ret_val = ValueId::new(3);

        let mut module = AirModule::new(ModuleId::new(1));
        let malloc_fn = {
            let mut f = AirFunction::new(malloc_id, "malloc");
            f.is_declaration = true;
            f
        };
        let free_fn = {
            let mut f = AirFunction::new(free_id, "free");
            f.is_declaration = true;
            f
        };

        let malloc_call =
            Instruction::new(InstId::new(10), Operation::CallDirect { callee: malloc_id })
                .with_operands(vec![size_val])
                .with_dst(ptr_val);
        // Two free() calls using the SAME ptr_val
        let free_call_1 =
            Instruction::new(InstId::new(20), Operation::CallDirect { callee: free_id })
                .with_operands(vec![ptr_val]);
        let free_call_2 =
            Instruction::new(InstId::new(21), Operation::CallDirect { callee: free_id })
                .with_operands(vec![ptr_val]);
        let ret = Instruction::new(InstId::new(30), Operation::Ret).with_operands(vec![ret_val]);

        let entry_block = {
            let mut b = AirBlock::new(BlockId::new(1));
            b.instructions = vec![malloc_call, free_call_1, free_call_2, ret];
            b
        };
        let main_fn = {
            let mut f = AirFunction::new(main_id, "main");
            f.blocks = vec![entry_block];
            f
        };
        module.functions = vec![malloc_fn, free_fn, main_fn];

        // SVFG: ptr_val is a single node (both frees share it)
        let mut svfg = Svfg::new();
        svfg.add_node(SvfgNodeId::value(ptr_val));
        svfg.add_node(SvfgNodeId::value(ret_val));
        svfg.add_edge(
            SvfgNodeId::value(ptr_val),
            SvfgEdgeKind::DirectDef,
            SvfgNodeId::value(ret_val),
        );

        let table = ResourceTable::new();
        let spec = super::super::spec::double_free();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert!(
            !result.findings.is_empty(),
            "double-free with same SSA value should be detected (found {} findings)",
            result.findings.len()
        );
        assert_eq!(result.findings[0].checker_name, "double-free");
    }

    /// Bug 2: fopen should NOT trigger lock-not-released.
    ///
    /// Previously, both `file-descriptor-leak` and `lock-not-released` used
    /// `ResourceRole::Acquire` as their source. This caused `lock-not-released`
    /// to spuriously trigger on `fopen` calls. The fix separates lock functions
    /// into `Lock`/`Unlock` roles.
    #[test]
    fn fopen_does_not_trigger_lock_not_released() {
        // Module: main() { f = fopen("test.txt", "r"); return 0; }
        let fopen_id = FunctionId::new(100);
        let main_id = FunctionId::new(300);

        let file_ptr = ValueId::new(1);
        let filename_arg = ValueId::new(2);
        let mode_arg = ValueId::new(3);
        let ret_val = ValueId::new(4);

        let mut module = AirModule::new(ModuleId::new(1));
        let fopen_fn = {
            let mut f = AirFunction::new(fopen_id, "fopen");
            f.is_declaration = true;
            f
        };

        let fopen_call =
            Instruction::new(InstId::new(10), Operation::CallDirect { callee: fopen_id })
                .with_operands(vec![filename_arg, mode_arg])
                .with_dst(file_ptr);
        let ret = Instruction::new(InstId::new(20), Operation::Ret).with_operands(vec![ret_val]);

        let entry_block = {
            let mut b = AirBlock::new(BlockId::new(1));
            b.instructions = vec![fopen_call, ret];
            b
        };
        let main_fn = {
            let mut f = AirFunction::new(main_id, "main");
            f.blocks = vec![entry_block];
            f
        };
        module.functions = vec![fopen_fn, main_fn];

        // SVFG: file_ptr and ret_val are separate nodes
        let mut svfg = Svfg::new();
        svfg.add_node(SvfgNodeId::value(file_ptr));
        svfg.add_node(SvfgNodeId::value(filename_arg));
        svfg.add_node(SvfgNodeId::value(ret_val));

        let table = ResourceTable::new();
        let spec = super::super::spec::lock_not_released();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert!(
            result.findings.is_empty(),
            "lock-not-released should NOT fire for fopen (found {} findings)",
            result.findings.len()
        );
    }

    /// Bug 3: stack-escape with alloca value directly returned.
    ///
    /// When the alloca result and the ret operand are the same ValueId
    /// (same `SvfgNodeId`), the solver's `target != source` check prevents
    /// detection. The fix detects source-sink overlap before running the solver.
    #[test]
    fn stack_escape_alloca_directly_returned() {
        // Module: foo() { x = alloca i32; return x; }
        let foo_id = FunctionId::new(300);
        let x_val = ValueId::new(1);

        let mut module = AirModule::new(ModuleId::new(1));

        let alloca = Instruction::new(
            InstId::new(10),
            Operation::Alloca {
                size_bytes: Some(4),
            },
        )
        .with_dst(x_val);
        let ret = Instruction::new(InstId::new(20), Operation::Ret).with_operands(vec![x_val]);

        let entry_block = {
            let mut b = AirBlock::new(BlockId::new(1));
            b.instructions = vec![alloca, ret];
            b
        };
        let foo_fn = {
            let mut f = AirFunction::new(foo_id, "foo");
            f.blocks = vec![entry_block];
            f
        };
        module.functions = vec![foo_fn];

        // SVFG: x_val is a single node (alloca dst = ret operand)
        let mut svfg = Svfg::new();
        svfg.add_node(SvfgNodeId::value(x_val));

        let table = ResourceTable::new();
        let spec = super::super::spec::stack_escape();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert!(
            !result.findings.is_empty(),
            "stack-escape with alloca directly returned should be detected (found {} findings)",
            result.findings.len()
        );
        assert_eq!(result.findings[0].checker_name, "stack-escape");
    }

    /// Bug 1 edge case: double-free where one free uses the same SSA value as
    /// the malloc return (source==sink overlap) and another free uses a
    /// different SSA value (distinct sink).
    ///
    /// `p = malloc(); q = p; free(p); free(q);`
    ///
    /// Previously, `multi_reach` skipped `free(p)` (target==source) and only
    /// found `free(q)` — 1 sink, below the threshold of 2. The supplementary
    /// `detect_same_node_multi_sink` didn't help because the two frees use
    /// different SVFG nodes.  Fix: pre-add source to reached_sinks when it's
    /// in the sink set.
    #[test]
    fn double_free_source_overlaps_one_sink() {
        // p = malloc(10); q = p; free(p); free(q); return;
        let malloc_id = FunctionId::new(100);
        let free_id = FunctionId::new(200);
        let main_id = FunctionId::new(300);

        let p_val = ValueId::new(1); // malloc return
        let q_val = ValueId::new(2); // copy of p
        let size_val = ValueId::new(3);
        let ret_val = ValueId::new(4);

        let mut module = AirModule::new(ModuleId::new(1));
        let malloc_fn = {
            let mut f = AirFunction::new(malloc_id, "malloc");
            f.is_declaration = true;
            f
        };
        let free_fn = {
            let mut f = AirFunction::new(free_id, "free");
            f.is_declaration = true;
            f
        };

        let malloc_call =
            Instruction::new(InstId::new(10), Operation::CallDirect { callee: malloc_id })
                .with_operands(vec![size_val])
                .with_dst(p_val);
        // q = p (copy)
        let copy = Instruction::new(InstId::new(15), Operation::Copy)
            .with_operands(vec![p_val])
            .with_dst(q_val);
        // free(p) — argument is p_val (same as malloc return = source SVFG node)
        let free_call_1 =
            Instruction::new(InstId::new(20), Operation::CallDirect { callee: free_id })
                .with_operands(vec![p_val]);
        // free(q) — argument is q_val (different SVFG node)
        let free_call_2 =
            Instruction::new(InstId::new(21), Operation::CallDirect { callee: free_id })
                .with_operands(vec![q_val]);
        let ret = Instruction::new(InstId::new(30), Operation::Ret).with_operands(vec![ret_val]);

        let entry_block = {
            let mut b = AirBlock::new(BlockId::new(1));
            b.instructions = vec![malloc_call, copy, free_call_1, free_call_2, ret];
            b
        };
        let main_fn = {
            let mut f = AirFunction::new(main_id, "main");
            f.blocks = vec![entry_block];
            f
        };
        module.functions = vec![malloc_fn, free_fn, main_fn];

        // SVFG: p_val -> q_val (value flow from malloc to copy)
        // Both p_val and q_val are sink nodes (free arguments)
        // p_val is ALSO the source node (malloc return)
        let mut svfg = Svfg::new();
        svfg.add_node(SvfgNodeId::value(p_val));
        svfg.add_node(SvfgNodeId::value(q_val));
        svfg.add_node(SvfgNodeId::value(ret_val));
        svfg.add_edge(
            SvfgNodeId::value(p_val),
            SvfgEdgeKind::DirectDef,
            SvfgNodeId::value(q_val),
        );

        let table = ResourceTable::new();
        let spec = super::super::spec::double_free();
        let config = SolverConfig::default();

        let result = run_checker(&spec, &module, &svfg, &table, &config);
        assert!(
            !result.findings.is_empty(),
            "double-free with source==sink1 + distinct sink2 should be detected \
             (found {} findings)",
            result.findings.len()
        );
        assert_eq!(result.findings[0].checker_name, "double-free");
    }
}
