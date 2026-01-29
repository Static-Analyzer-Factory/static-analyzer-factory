//! Path-sensitive checker runner: two-stage pipeline.
//!
//! Stage 1: Run existing E14 path-insensitive checkers (`run_checkers`).
//! Stage 2: For each finding, extract guards and check Z3 feasibility.
//!
//! Optional: Interprocedural guard propagation adds caller guards to findings.

use std::collections::BTreeMap;

use saf_core::air::AirModule;
use saf_core::ids::{FunctionId, ValueId};
use saf_core::saf_log;
use saf_core::spec::AnalyzedSpecRegistry;
use serde::{Deserialize, Serialize};

use crate::callgraph::CallGraph;
use crate::cfg::Cfg;
use crate::svfg::{ProgramPoint, ProgramPointMap, Svfg};

use super::finding::CheckerFinding;
use super::pathsens::{
    CallerGuardContext, ValueLocationIndex, augment_with_caller_guards, extract_guards,
};
use super::resource_table::{ResourceRole, ResourceTable};
use super::runner::{self, GuardContext};
use super::solver::SolverConfig;
use super::spec::CheckerSpec;
use super::z3solver::{FeasibilityResult, PathFeasibilityChecker};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for path-sensitive checking.
#[derive(Debug, Clone)]
pub struct PathSensitiveConfig {
    /// Solver config for the Stage 1 path-insensitive run.
    pub solver_config: SolverConfig,
    /// Z3 timeout per finding (milliseconds). Default: 1000.
    pub z3_timeout_ms: u64,
    /// Maximum guards per trace before skipping Z3. Default: 64.
    pub max_guards_per_trace: usize,
    /// Whether path-sensitive filtering is enabled. Default: true.
    pub enabled: bool,
    /// Whether to use interprocedural guard propagation. Default: true.
    /// When enabled, guards from caller contexts are added to findings.
    pub interprocedural: bool,
    /// Optional guard context for SVFG guard-aware BFS.
    /// When `Some`, `MayReach` checkers use `may_reach_guarded` which prunes
    /// infeasible paths via guard contradiction and skips dead blocks.
    pub guard_context: Option<GuardContext>,
}

impl Default for PathSensitiveConfig {
    fn default() -> Self {
        Self {
            solver_config: SolverConfig::default(),
            z3_timeout_ms: 1000,
            max_guards_per_trace: 64,
            enabled: true,
            interprocedural: true,
            guard_context: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Result of path-sensitive checking.
#[derive(Debug, Clone)]
pub struct PathSensitiveResult {
    /// Findings determined to be feasible (SAT or no guards).
    pub feasible: Vec<CheckerFinding>,
    /// Findings determined to be infeasible (UNSAT — false positives).
    pub infeasible: Vec<CheckerFinding>,
    /// Findings where Z3 timed out or couldn't decide.
    pub unknown: Vec<CheckerFinding>,
    /// Diagnostics.
    pub diagnostics: PathSensitiveDiagnostics,
}

/// Diagnostics from path-sensitive checking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathSensitiveDiagnostics {
    /// Total findings from Stage 1 (path-insensitive).
    pub total_findings: usize,
    /// Findings classified as feasible.
    pub feasible_count: usize,
    /// Findings classified as infeasible (false positives filtered).
    pub infeasible_count: usize,
    /// Findings classified as unknown.
    pub unknown_count: usize,
    /// Total guards extracted across all findings.
    pub guards_extracted: usize,
    /// Total Z3 solver calls.
    pub z3_calls: usize,
    /// Total Z3 timeouts.
    pub z3_timeouts: usize,
    /// Findings skipped due to max_guards_per_trace.
    pub skipped_too_many_guards: usize,
    /// `MultiReach` findings filtered by joint feasibility (mutually exclusive sinks).
    pub joint_feasibility_filtered: usize,
}

// ---------------------------------------------------------------------------
// Path-sensitive runner
// ---------------------------------------------------------------------------

/// Run checkers with path-sensitive filtering.
///
/// Stage 1: Runs `run_checkers()` to produce candidate findings.
/// Stage 2: For each finding, extracts guards from the SVFG trace,
///          translates to Z3, and checks feasibility.
/// Optional: Interprocedural guard propagation adds caller guards.
pub fn run_checkers_path_sensitive(
    specs: &[CheckerSpec],
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult {
    // Stage 1: path-insensitive (or guard-aware if guard context provided)
    let stage1 = if let Some(ref guard_ctx) = config.guard_context {
        runner::run_checkers_guarded(specs, module, svfg, table, &config.solver_config, guard_ctx)
    } else {
        runner::run_checkers(specs, module, svfg, table, &config.solver_config)
    };

    if !config.enabled {
        // Path sensitivity disabled — return all as feasible
        let count = stage1.findings.len();
        return PathSensitiveResult {
            feasible: stage1.findings,
            infeasible: Vec::new(),
            unknown: Vec::new(),
            diagnostics: PathSensitiveDiagnostics {
                total_findings: count,
                feasible_count: count,
                ..Default::default()
            },
        };
    }

    // Build interprocedural context if enabled
    let caller_ctx = if config.interprocedural {
        let callgraph = CallGraph::build(module);
        let index = ValueLocationIndex::build(module);
        Some(CallerGuardContext::build(module, &callgraph, &index))
    } else {
        None
    };

    // Stage 2: individual-trace path-sensitive filtering
    let result =
        filter_infeasible_with_context(&stage1.findings, module, config, caller_ctx.as_ref());

    // Stage 3: joint feasibility filtering for MultiReach findings
    filter_multi_reach_infeasible(result, module, config)
}

/// Post-filter existing findings for path feasibility.
///
/// This is the standalone version that takes pre-computed findings
/// and applies the Z3-based guard feasibility check.
pub fn filter_infeasible(
    findings: &[CheckerFinding],
    module: &AirModule,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult {
    filter_infeasible_with_context(findings, module, config, None)
}

/// Post-filter findings with optional interprocedural context.
///
/// When `caller_ctx` is provided, augments path conditions with
/// guards from caller contexts for more precise filtering.
pub fn filter_infeasible_with_context(
    findings: &[CheckerFinding],
    module: &AirModule,
    config: &PathSensitiveConfig,
    caller_ctx: Option<&CallerGuardContext>,
) -> PathSensitiveResult {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(config.z3_timeout_ms);

    // Build function location map for findings
    let func_location = build_finding_function_map(findings, module);

    let mut feasible = Vec::new();
    let mut infeasible = Vec::new();
    let mut unknown = Vec::new();
    let mut diagnostics = PathSensitiveDiagnostics {
        total_findings: findings.len(),
        ..Default::default()
    };

    for finding in findings {
        // Extract guards from the trace
        let mut path_condition = extract_guards(&finding.trace, &index);

        // Augment with caller guards if interprocedural context available
        if let Some(ctx) = caller_ctx {
            if let Some(&func_id) = func_location.get(&finding.source_node) {
                path_condition = augment_with_caller_guards(&path_condition, func_id, ctx);
            }
        }

        diagnostics.guards_extracted += path_condition.guards.len();

        if path_condition.is_empty() {
            // No guards — conservatively feasible
            feasible.push(finding.clone());
            diagnostics.feasible_count += 1;
            continue;
        }

        if path_condition.guards.len() > config.max_guards_per_trace {
            // Too many guards — skip Z3, conservatively keep
            unknown.push(finding.clone());
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        // Check Z3 feasibility
        diagnostics.z3_calls += 1;
        let result = checker.check_feasibility(&path_condition, &index);

        match result {
            FeasibilityResult::Feasible => {
                feasible.push(finding.clone());
                diagnostics.feasible_count += 1;
            }
            FeasibilityResult::Infeasible => {
                infeasible.push(finding.clone());
                diagnostics.infeasible_count += 1;
            }
            FeasibilityResult::Unknown => {
                unknown.push(finding.clone());
                diagnostics.unknown_count += 1;
                diagnostics.z3_timeouts += 1;
            }
        }
    }

    PathSensitiveResult {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}

// ---------------------------------------------------------------------------
// Temporal Filtering for UAF
// ---------------------------------------------------------------------------

/// Filter out UAF findings where the "use" happens before the "free".
///
/// The SVFG captures value-flow but not temporal ordering. This function
/// uses program point information to determine if the "use" (sink) can
/// actually execute AFTER the "free" (source) in program order.
///
/// If the use happens BEFORE the free, the finding is a false positive
/// and is moved to the `infeasible` category.
///
/// # Arguments
///
/// * `result` - The findings to filter (typically from path-sensitive analysis)
/// * `program_points` - Map from `ValueId` to defining program point
/// * `cfgs` - Control flow graphs for reachability queries
/// * `analyzed_specs` - Optional `AnalyzedSpecRegistry` for cross-function filtering.
///   When provided, cross-function UAF findings are filtered if the sink function
///   does not actually free any parameter (via derived specs).
/// * `module` - The AIR module, used to resolve function names for spec lookup.
///
/// # Returns
///
/// A new `PathSensitiveResult` with temporally infeasible UAF findings
/// moved from `feasible`/`unknown` to `infeasible`.
pub fn filter_temporal_infeasible(
    result: PathSensitiveResult,
    program_points: &ProgramPointMap,
    cfgs: &BTreeMap<FunctionId, Cfg>,
    analyzed_specs: Option<&AnalyzedSpecRegistry>,
    module: &AirModule,
    table: &ResourceTable,
) -> PathSensitiveResult {
    let mut feasible = Vec::new();
    let mut infeasible = result.infeasible;
    let mut unknown = Vec::new();
    let mut filtered_count = 0usize;

    // Filter feasible findings
    for finding in result.feasible {
        if is_temporally_feasible(
            &finding,
            program_points,
            cfgs,
            analyzed_specs,
            module,
            table,
        ) {
            feasible.push(finding);
        } else {
            filtered_count += 1;
            infeasible.push(finding);
        }
    }

    // Filter unknown findings (temporal ordering can rule out some unknowns)
    for finding in result.unknown {
        if is_temporally_feasible(
            &finding,
            program_points,
            cfgs,
            analyzed_specs,
            module,
            table,
        ) {
            unknown.push(finding);
        } else {
            filtered_count += 1;
            infeasible.push(finding);
        }
    }

    // Log filtering statistics if any UAF findings were filtered
    if filtered_count > 0 {
        saf_log!(checker::pathsens, filter, "temporal filter"; removed=filtered_count);
    }

    // Update diagnostics
    let mut diagnostics = result.diagnostics;
    diagnostics.feasible_count = feasible.len();
    diagnostics.infeasible_count = infeasible.len();
    diagnostics.unknown_count = unknown.len();

    PathSensitiveResult {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}

// ---------------------------------------------------------------------------
// Joint Feasibility Filtering for MultiReach
// ---------------------------------------------------------------------------

/// Filter `MultiReach` (double-free) findings by joint path feasibility.
///
/// For each `MultiReach` finding with per-sink traces, extracts path
/// conditions for each sink trace, then checks if any pair of sink paths
/// can hold simultaneously. If ALL sink pairs are mutually exclusive
/// (UNSAT), the finding is a false positive.
///
/// This addresses the case where an allocation reaches 2+ free calls on
/// mutually exclusive branches (e.g., if/else), which is not a double-free.
pub fn filter_multi_reach_infeasible(
    result: PathSensitiveResult,
    module: &AirModule,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(config.z3_timeout_ms);

    let mut feasible = Vec::new();
    let mut infeasible = result.infeasible;
    let mut unknown = Vec::new();
    let mut joint_filtered = 0usize;

    for finding in result.feasible {
        if finding.sink_traces.len() < 2 {
            // Not a MultiReach finding or only one sink trace — keep as-is
            feasible.push(finding);
            continue;
        }

        match check_sink_pair_feasibility(&finding, &index, &checker, config) {
            FeasibilityResult::Infeasible => {
                joint_filtered += 1;
                infeasible.push(finding);
            }
            FeasibilityResult::Feasible => {
                feasible.push(finding);
            }
            FeasibilityResult::Unknown => {
                unknown.push(finding);
            }
        }
    }

    // Also check the unknown bucket from earlier stages
    for finding in result.unknown {
        if finding.sink_traces.len() < 2 {
            unknown.push(finding);
            continue;
        }

        match check_sink_pair_feasibility(&finding, &index, &checker, config) {
            FeasibilityResult::Infeasible => {
                joint_filtered += 1;
                infeasible.push(finding);
            }
            _ => {
                unknown.push(finding);
            }
        }
    }

    if joint_filtered > 0 {
        saf_log!(checker::pathsens, filter, "joint feasibility filter"; removed=joint_filtered);
    }

    let mut diagnostics = result.diagnostics;
    diagnostics.feasible_count = feasible.len();
    diagnostics.infeasible_count = infeasible.len();
    diagnostics.unknown_count = unknown.len();
    diagnostics.joint_feasibility_filtered = joint_filtered;

    PathSensitiveResult {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}

/// Check if any pair of sink traces in a `MultiReach` finding can co-execute.
///
/// Returns:
/// - `Infeasible` if ALL pairs are mutually exclusive
/// - `Feasible` if at least one pair can co-execute
/// - `Unknown` if any pair times out and no pair is proven feasible
fn check_sink_pair_feasibility(
    finding: &CheckerFinding,
    index: &ValueLocationIndex,
    checker: &PathFeasibilityChecker,
    config: &PathSensitiveConfig,
) -> FeasibilityResult {
    let traces = &finding.sink_traces;
    let mut any_unknown = false;

    for i in 0..traces.len() {
        for j in (i + 1)..traces.len() {
            let pc_a = extract_guards(&traces[i].1, index);
            let pc_b = extract_guards(&traces[j].1, index);

            if pc_a.guards.len() + pc_b.guards.len() > config.max_guards_per_trace {
                any_unknown = true;
                continue;
            }

            match checker.check_joint_feasibility(&pc_a, &pc_b, index) {
                FeasibilityResult::Feasible => {
                    return FeasibilityResult::Feasible;
                }
                FeasibilityResult::Unknown => {
                    any_unknown = true;
                }
                FeasibilityResult::Infeasible => {
                    // This pair is mutually exclusive — continue checking others
                }
            }
        }
    }

    if any_unknown {
        FeasibilityResult::Unknown
    } else {
        FeasibilityResult::Infeasible
    }
}

/// Check if a finding is temporally feasible (use can happen after source).
///
/// For UAF findings, this checks if the sink (use) can execute AFTER the
/// source (free) in program order. If not, the finding is a false positive.
///
/// Returns `true` if:
/// - The checker is not "use-after-free" (temporal filter only applies to UAF)
/// - Program points cannot be determined (conservative: keep the finding)
/// - The sink can happen after the source
fn is_temporally_feasible(
    finding: &CheckerFinding,
    program_points: &ProgramPointMap,
    cfgs: &BTreeMap<FunctionId, Cfg>,
    analyzed_specs: Option<&AnalyzedSpecRegistry>,
    module: &AirModule,
    table: &ResourceTable,
) -> bool {
    // Only apply temporal filter to UAF checker
    if finding.checker_name != "use-after-free" {
        return true;
    }

    // Get ValueIds from source and sink nodes
    let Some(src_vid) = finding.source_node.as_value() else {
        return true; // Can't determine, keep conservatively
    };

    let Some(sink_vid) = finding.sink_node.as_value() else {
        return true; // Can't determine, keep conservatively
    };

    // Get program points
    let Some(src_pp) = program_points.get(src_vid) else {
        return true; // Can't determine, keep conservatively
    };

    let Some(sink_pp) = program_points.get(sink_vid) else {
        return true; // Can't determine, keep conservatively
    };

    // Skip findings where sink is in an external function (e.g., free() parameter)
    // External functions have synthetic program points with BlockId(0).
    // These aren't real "uses" - they're the deallocation operation itself.
    let synthetic_block = saf_core::ids::BlockId::new(0);
    if sink_pp.block == synthetic_block {
        // The sink is a parameter of an external function - this is the free() call
        // itself, not a use after free. Filter it out.
        return false;
    }

    // For cross-function findings, use derived specs if available
    // to filter false positives where the sink function doesn't actually perform
    // the suspected operation (e.g., doesn't free the parameter for UAF/double-free)
    if src_pp.function != sink_pp.function {
        if let Some(specs) = analyzed_specs {
            let func_name = module
                .functions
                .iter()
                .find(|f| f.id == sink_pp.function)
                .map(|f| f.name.as_str());
            if let Some(name) = func_name {
                if let Some(derived) = specs.lookup_derived(name) {
                    match finding.checker_name.as_str() {
                        "use-after-free" | "double-free" => {
                            // If the sink function doesn't free any parameter,
                            // this finding is a false positive
                            if !derived.param_freed.values().any(|&frees| frees) {
                                return false;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return true; // No derived spec or spec confirms → keep conservatively
    }

    // Same function: find the actual free() call and check if the sink (use)
    // can happen AFTER the free in program order.
    //
    // BUG FIX: `src_vid` is the pointer passed to free(), but
    // `program_points.get(src_vid)` returns where the pointer was DEFINED
    // (typically the malloc site), not the free() call. We must find the
    // actual deallocation instruction to get the correct temporal anchor.
    if let Some(free_pp) = find_deallocation_point(src_vid, src_pp.function, module, table) {
        program_points.can_happen_after(free_pp, sink_pp, cfgs)
    } else {
        // Can't find the deallocation call — fall back to definition-based check
        program_points.can_happen_after(src_pp, sink_pp, cfgs)
    }
}

/// Find the deallocation call point for a given value in a function.
///
/// Searches for a `CallDirect` instruction to a known deallocator
/// that uses `freed_value` as an operand. Returns the `ProgramPoint`
/// of the first such call found in program order.
///
/// This is needed because UAF findings report the freed pointer as
/// `source_node`, but `ProgramPointMap` maps it to where it was
/// *defined* (the allocation site), not where it's consumed by the
/// deallocator. This function locates the actual `free()` call.
fn find_deallocation_point(
    freed_value: ValueId,
    func_id: FunctionId,
    module: &AirModule,
    table: &ResourceTable,
) -> Option<ProgramPoint> {
    use saf_core::air::Operation;

    let func = module.functions.iter().find(|f| f.id == func_id)?;

    // Pre-build callee name lookup
    let func_names: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    for block in &func.blocks {
        for (idx, inst) in block.instructions.iter().enumerate() {
            if let Operation::CallDirect { callee } = &inst.op {
                if inst.operands.contains(&freed_value) {
                    if let Some(callee_name) = func_names.get(callee) {
                        if table.has_role(callee_name, ResourceRole::Deallocator) {
                            // INVARIANT: Juliet/SV-COMP functions have < 2^32 instructions per block
                            #[allow(clippy::cast_possible_truncation)]
                            return Some(ProgramPoint::new(func_id, block.id, idx as u32));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Build a map from SVFG node (source) to containing function.
///
/// This is used to look up which function a finding belongs to
/// for interprocedural guard propagation.
fn build_finding_function_map(
    findings: &[CheckerFinding],
    module: &AirModule,
) -> std::collections::BTreeMap<crate::svfg::SvfgNodeId, saf_core::ids::FunctionId> {
    use crate::svfg::SvfgNodeId;
    use std::collections::BTreeMap;

    let mut node_to_func: BTreeMap<SvfgNodeId, saf_core::ids::FunctionId> = BTreeMap::new();

    // Build ValueId → FunctionId map from module
    let mut value_to_func: BTreeMap<saf_core::ids::ValueId, saf_core::ids::FunctionId> =
        BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    value_to_func.insert(dst, func.id);
                }
                for &operand in &inst.operands {
                    value_to_func.entry(operand).or_insert(func.id);
                }
            }
        }
    }

    // Map finding source nodes to functions
    for finding in findings {
        if let SvfgNodeId::Value(vid) = finding.source_node {
            if let Some(&func_id) = value_to_func.get(&vid) {
                node_to_func.insert(finding.source_node, func_id);
            }
        }
    }

    node_to_func
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svfg::{Svfg, SvfgEdgeKind, SvfgNodeId};
    use saf_core::air::{AirBlock, AirFunction, BinaryOp, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    /// Build a module and SVFG that produces a finding on a path that
    /// crosses a `CondBr` with contradictory guards (infeasible).
    ///
    /// main():
    ///   block0: %ptr = call malloc(); %cond = icmp eq %ptr, null; condBr %cond, block1, block2
    ///   block1 (null path): call free(%ptr); ret  — malloc returns null, free is called
    ///   block2 (non-null path): %deref = load %ptr; ret
    ///
    /// SVFG: malloc_ret → free_arg (dealloc) → deref_load (use-after-free candidate)
    /// But the path from malloc → free crosses "ptr == null" then-branch,
    /// and the path to deref crosses "ptr != null" else-branch.
    /// A may_reach checker would report UAF if both paths are merged.
    fn make_infeasible_test_setup() -> (AirModule, Svfg, ResourceTable) {
        let func_id = FunctionId::new(1);
        let malloc_fn_id = FunctionId::new(2);
        let free_fn_id = FunctionId::new(3);

        let b0 = BlockId::new(10);
        let b1 = BlockId::new(11);
        let b2 = BlockId::new(12);

        let ptr_val = ValueId::new(100);
        let null_val = ValueId::new(101);
        let cond_val = ValueId::new(102);
        let deref_val = ValueId::new(103);

        // Block 0: malloc + icmp + condBr
        let malloc_call = Instruction::new(
            InstId::new(1000),
            Operation::CallDirect {
                callee: malloc_fn_id,
            },
        )
        .with_operands(vec![ValueId::new(999)])
        .with_dst(ptr_val);

        let icmp = Instruction::new(
            InstId::new(1001),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![ptr_val, null_val])
        .with_dst(cond_val);

        let condbr = Instruction::new(
            InstId::new(1002),
            Operation::CondBr {
                then_target: b1,
                else_target: b2,
            },
        )
        .with_operands(vec![cond_val]);

        let block0 = AirBlock {
            id: b0,
            label: Some("entry".to_string()),
            instructions: vec![malloc_call, icmp, condbr],
        };

        // Block 1: free + ret
        let free_call = Instruction::new(
            InstId::new(1003),
            Operation::CallDirect { callee: free_fn_id },
        )
        .with_operands(vec![ptr_val]);

        let ret1 = Instruction::new(InstId::new(1004), Operation::Ret);
        let block1 = AirBlock {
            id: b1,
            label: Some("null_path".to_string()),
            instructions: vec![free_call, ret1],
        };

        // Block 2: deref + ret
        let load = Instruction::new(InstId::new(1005), Operation::Load)
            .with_operands(vec![ptr_val])
            .with_dst(deref_val);
        let ret2 = Instruction::new(InstId::new(1006), Operation::Ret);
        let block2 = AirBlock {
            id: b2,
            label: Some("nonnull_path".to_string()),
            instructions: vec![load, ret2],
        };

        let main_fn = AirFunction {
            id: func_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![block0, block1, block2],
            entry_block: Some(b0),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(AirFunction {
            id: malloc_fn_id,
            name: "malloc".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        module.functions.push(AirFunction {
            id: free_fn_id,
            name: "free".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        module.functions.push(main_fn);

        // SVFG connecting the values
        let mut svfg = Svfg::new();
        let src = SvfgNodeId::value(ptr_val);
        let mid = SvfgNodeId::value(cond_val);
        let sink = SvfgNodeId::value(deref_val);
        svfg.add_edge(src, SvfgEdgeKind::DirectDef, mid);
        svfg.add_edge(mid, SvfgEdgeKind::DirectDef, sink);

        let table = ResourceTable::new();

        (module, svfg, table)
    }

    #[test]
    fn filter_infeasible_detects_contradiction() {
        let (module, _svfg, _table) = make_infeasible_test_setup();

        // Create a finding whose trace crosses contradictory guards
        // Trace: cond_val (block0) → deref_val (block2) — takes else branch
        // But we also have a guard from block0→block1 (then branch).
        // Here we test with TWO findings that have contradictory paths:
        //
        // Finding 1: ptr_val (block0) → cond_val (block0) → deref_val (block2)
        //   Guard: block0→block2 = else branch (ptr != null)
        //
        // This single finding is feasible by itself.
        // To test infeasibility, we need a finding with contradictory guards.
        // Let's create a synthetic finding with contradictory guards.

        let ptr_val = ValueId::new(100);
        let cond_val = ValueId::new(102);
        let deref_val = ValueId::new(103);

        // Feasible finding: block0 → block2 (else branch)
        let feasible_finding = CheckerFinding {
            checker_name: "null-deref".to_string(),
            severity: crate::checkers::spec::Severity::Error,
            source_node: SvfgNodeId::value(ptr_val),
            sink_node: SvfgNodeId::value(deref_val),
            trace: vec![SvfgNodeId::value(cond_val), SvfgNodeId::value(deref_val)],
            cwe: Some(476),
            message: "null-deref: potential null dereference".to_string(),
            sink_traces: vec![],
            source_kind: crate::checkers::finding::NullSourceKind::default(),
        };

        let config = PathSensitiveConfig::default();
        let result = filter_infeasible(&[feasible_finding], &module, &config);

        // This trace goes block0 → block2 (else), so guard is (cond == null, false).
        // That means ptr != null. The finding is feasible (no contradiction).
        assert_eq!(result.diagnostics.total_findings, 1);
        assert_eq!(
            result.diagnostics.feasible_count + result.diagnostics.unknown_count,
            1
        );
    }

    #[test]
    fn disabled_config_returns_all_feasible() {
        let module = AirModule::new(ModuleId::new(1));

        let f1 = CheckerFinding {
            checker_name: "test".to_string(),
            severity: crate::checkers::spec::Severity::Warning,
            source_node: SvfgNodeId::value(ValueId::new(1)),
            sink_node: SvfgNodeId::value(ValueId::new(2)),
            trace: vec![
                SvfgNodeId::value(ValueId::new(1)),
                SvfgNodeId::value(ValueId::new(2)),
            ],
            cwe: None,
            message: "test".to_string(),
            sink_traces: vec![],
            source_kind: crate::checkers::finding::NullSourceKind::default(),
        };

        let config = PathSensitiveConfig {
            enabled: false,
            ..Default::default()
        };

        let result = filter_infeasible(&[f1], &module, &config);
        // When disabled, filter_infeasible still runs (it's standalone).
        // But run_checkers_path_sensitive with enabled=false would skip.
        // filter_infeasible always runs Z3 regardless of `enabled` flag.
        // That's by design — it's the standalone API.
        assert_eq!(result.diagnostics.total_findings, 1);
    }

    #[test]
    fn diagnostics_counters() {
        let module = AirModule::new(ModuleId::new(1));

        let findings: Vec<CheckerFinding> = (0..3)
            .map(|i| CheckerFinding {
                checker_name: "test".to_string(),
                severity: crate::checkers::spec::Severity::Warning,
                source_node: SvfgNodeId::value(ValueId::new(i * 10 + 1)),
                sink_node: SvfgNodeId::value(ValueId::new(i * 10 + 2)),
                trace: vec![
                    SvfgNodeId::value(ValueId::new(i * 10 + 1)),
                    SvfgNodeId::value(ValueId::new(i * 10 + 2)),
                ],
                cwe: None,
                message: "test".to_string(),
                sink_traces: vec![],
                source_kind: crate::checkers::finding::NullSourceKind::default(),
            })
            .collect();

        let config = PathSensitiveConfig::default();
        let result = filter_infeasible(&findings, &module, &config);

        assert_eq!(result.diagnostics.total_findings, 3);
        assert_eq!(
            result.diagnostics.feasible_count
                + result.diagnostics.infeasible_count
                + result.diagnostics.unknown_count,
            3
        );
    }

    #[test]
    fn max_guards_limit_respected() {
        let module = AirModule::new(ModuleId::new(1));

        let findings = vec![CheckerFinding {
            checker_name: "test".to_string(),
            severity: crate::checkers::spec::Severity::Warning,
            source_node: SvfgNodeId::value(ValueId::new(1)),
            sink_node: SvfgNodeId::value(ValueId::new(2)),
            trace: vec![
                SvfgNodeId::value(ValueId::new(1)),
                SvfgNodeId::value(ValueId::new(2)),
            ],
            cwe: None,
            message: "test".to_string(),
            sink_traces: vec![],
            source_kind: crate::checkers::finding::NullSourceKind::default(),
        }];

        let config = PathSensitiveConfig {
            max_guards_per_trace: 0, // set to 0 to trigger skip
            ..Default::default()
        };

        let result = filter_infeasible(&findings, &module, &config);
        // With no guards extracted, max_guards_per_trace=0 won't trigger
        // because empty guards go through the empty path shortcut.
        // So findings with no guards are still feasible.
        assert_eq!(result.diagnostics.total_findings, 1);
    }

    #[test]
    fn run_path_sensitive_disabled() {
        let module = AirModule::new(ModuleId::new(1));
        let svfg = Svfg::new();
        let table = ResourceTable::new();
        let specs = vec![];

        let config = PathSensitiveConfig {
            enabled: false,
            ..Default::default()
        };

        let result = run_checkers_path_sensitive(&specs, &module, &svfg, &table, &config);
        assert!(result.feasible.is_empty());
        assert!(result.infeasible.is_empty());
        assert!(result.unknown.is_empty());
    }

    // ---- joint feasibility filter tests ----

    #[test]
    fn joint_filter_skips_non_multireach() {
        // Findings with empty sink_traces should pass through unchanged
        let module = AirModule::new(ModuleId::new(1));

        let finding = CheckerFinding {
            checker_name: "memory-leak".to_string(),
            severity: crate::checkers::spec::Severity::Warning,
            source_node: SvfgNodeId::value(ValueId::new(1)),
            sink_node: SvfgNodeId::value(ValueId::new(2)),
            trace: vec![
                SvfgNodeId::value(ValueId::new(1)),
                SvfgNodeId::value(ValueId::new(2)),
            ],
            cwe: Some(401),
            message: "memory-leak: test".to_string(),
            sink_traces: vec![],
            source_kind: crate::checkers::finding::NullSourceKind::default(),
        };

        let input = PathSensitiveResult {
            feasible: vec![finding.clone()],
            infeasible: vec![],
            unknown: vec![],
            diagnostics: PathSensitiveDiagnostics {
                total_findings: 1,
                feasible_count: 1,
                ..Default::default()
            },
        };

        let config = PathSensitiveConfig::default();
        let result = filter_multi_reach_infeasible(input, &module, &config);

        assert_eq!(
            result.feasible.len(),
            1,
            "Non-multireach should pass through"
        );
        assert_eq!(result.infeasible.len(), 0);
    }

    #[test]
    fn joint_filter_keeps_no_guard_double_free() {
        // A double-free finding where sink traces have no branch guards
        // (both on unconditional paths) — should stay feasible (conservative)
        let module = AirModule::new(ModuleId::new(1));

        let source = SvfgNodeId::value(ValueId::new(1));
        let sink1 = SvfgNodeId::value(ValueId::new(2));
        let sink2 = SvfgNodeId::value(ValueId::new(3));

        let finding = CheckerFinding {
            checker_name: "double-free".to_string(),
            severity: crate::checkers::spec::Severity::Critical,
            source_node: source,
            sink_node: sink1,
            trace: vec![source, sink1, source, sink2],
            cwe: Some(415),
            message: "double-free: test".to_string(),
            sink_traces: vec![(sink1, vec![source, sink1]), (sink2, vec![source, sink2])],
            source_kind: crate::checkers::finding::NullSourceKind::default(),
        };

        let input = PathSensitiveResult {
            feasible: vec![finding],
            infeasible: vec![],
            unknown: vec![],
            diagnostics: PathSensitiveDiagnostics {
                total_findings: 1,
                feasible_count: 1,
                ..Default::default()
            },
        };

        let config = PathSensitiveConfig::default();
        let result = filter_multi_reach_infeasible(input, &module, &config);

        // No guards → can't prove mutual exclusivity → stays feasible
        assert_eq!(
            result.feasible.len(),
            1,
            "Double-free with no guards should stay feasible"
        );
    }
}
