//! Path-sensitive alias refinement via per-path constraint solving.
//!
//! For path_tests where flow-insensitive PTA merges assignments from
//! exclusive branches, this module provides path-sensitive alias queries
//! by re-solving PTA constraints restricted to each feasible path.
//!
//! # Approach
//!
//! 1. Extract all branch points (CondBr terminators) in the function
//! 2. For each combination of branch choices (2^n paths, capped at max_paths):
//!    a. Determine which blocks are reachable under this path
//!    b. Collect PTA constraints only from reachable blocks
//!    c. Run a mini PTA solver on the restricted constraints
//!    d. Query alias on the per-path result
//! 3. Combine per-path results: NoAlias if all paths say No, etc.

use std::collections::{BTreeMap, BTreeSet};

use rustc_hash::FxHashMap;
use saf_core::air::{AirFunction, AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, LocId, ObjId, ValueId};
use saf_core::saf_log;

use crate::cfg::Cfg;
#[cfg(feature = "z3-solver")]
use crate::guard::PathCondition;

use super::location::{FieldPath, LocationFactory, PathStep};
use super::result::AliasResult;
use super::solver::PointsToMap;

/// Configuration for path-sensitive alias analysis.
pub struct PathSensitiveAliasConfig {
    /// Maximum number of branch points to enumerate (2^n paths).
    pub max_branches: usize,
    /// Maximum PTA iterations per path.
    pub max_iterations: usize,
}

impl Default for PathSensitiveAliasConfig {
    fn default() -> Self {
        Self {
            max_branches: 10, // 2^10 = 1024 max paths
            max_iterations: 100_000,
        }
    }
}

/// Result of path-sensitive alias analysis.
pub struct PathSensitiveAliasResult {
    /// The refined alias result.
    pub alias: AliasResult,
    /// Number of paths enumerated.
    pub paths_total: usize,
    /// Number of feasible paths (reachable from entry).
    pub paths_feasible: usize,
}

/// Perform path-sensitive alias analysis for two pointers within a function.
///
/// Enumerates branch combinations, computes reachable blocks for each path,
/// re-solves PTA constraints on reachable blocks, and combines per-path alias results.
#[must_use]
pub fn path_sensitive_alias(
    p: ValueId,
    q: ValueId,
    func: &AirFunction,
    module: &AirModule,
    config: &PathSensitiveAliasConfig,
) -> PathSensitiveAliasResult {
    let cfg = Cfg::build(func);

    // Find all CondBr terminators in the function
    let mut branches: Vec<BranchPoint> = Vec::new();
    for block in &func.blocks {
        if let Some(term) = block.terminator() {
            if let Operation::CondBr {
                then_target,
                else_target,
            } = &term.op
            {
                let condition = term.operands.first().copied().unwrap_or(ValueId::new(0));
                branches.push(BranchPoint {
                    block_id: block.id,
                    condition,
                    then_target: *then_target,
                    else_target: *else_target,
                });
            }
        }
    }

    if branches.is_empty() {
        // No branches — run flow-sensitive solver on the single linear path.
        // Even without branches, flow-sensitive processing with strong updates
        // is valuable: it resolves sequential store ordering (e.g., `main()`
        // calling `foo()` twice with different globals — the second call's
        // strong updates override the first call's effects).
        let all_blocks: BTreeSet<BlockId> = func.blocks.iter().map(|b| b.id).collect();
        // All edges are active when there are no branches
        let all_edges: BTreeSet<(BlockId, BlockId)> = all_blocks
            .iter()
            .flat_map(|&bid| {
                cfg.successors_of(bid)
                    .into_iter()
                    .flat_map(move |succs| succs.iter().map(move |&s| (bid, s)))
            })
            .collect();
        let mut factory =
            LocationFactory::new(super::config::FieldSensitivity::StructFields { max_depth: 10 });
        let pts = solve_per_path_flow_sensitive(
            func,
            module,
            &cfg,
            &all_blocks,
            &all_edges,
            &mut factory,
            config.max_iterations,
        );
        let alias = compute_alias_from_pts(p, q, &pts, factory.all_locations());
        return PathSensitiveAliasResult {
            alias,
            paths_total: 1,
            paths_feasible: 1,
        };
    }

    // Cap branches
    let n = branches.len().min(config.max_branches);
    let total_paths = 1usize << n;

    let mut per_path_results = Vec::new();
    let mut feasible_count = 0;

    for path_idx in 0..total_paths {
        // Determine branch choices for this path
        let choices: Vec<bool> = (0..n).map(|j| ((path_idx >> j) & 1) == 1).collect();

        // Compute reachable blocks and active edges under these branch choices
        let (reachable, active_edges) =
            compute_reachable_blocks_and_edges(&cfg, &branches[..n], &choices);

        // Skip if entry is not reachable (shouldn't happen, but safety check)
        if !reachable.contains(&cfg.entry) {
            continue;
        }

        feasible_count += 1;

        // Use flow-sensitive per-path solving:
        // Process blocks in topological order with strong updates for stores
        // to singleton pointers. This avoids Andersen's merging of stores from
        // sequential assignments (e.g., `p = &a; p = &b` → only `p → &b`).
        let mut factory =
            LocationFactory::new(super::config::FieldSensitivity::StructFields { max_depth: 10 });
        let pts = solve_per_path_flow_sensitive(
            func,
            module,
            &cfg,
            &reachable,
            &active_edges,
            &mut factory,
            config.max_iterations,
        );

        // Query alias
        let alias = compute_alias_from_pts(p, q, &pts, factory.all_locations());
        per_path_results.push(alias);
    }

    if per_path_results.is_empty() {
        return PathSensitiveAliasResult {
            alias: AliasResult::Unknown,
            paths_total: total_paths,
            paths_feasible: 0,
        };
    }

    // Combine results
    let combined = combine_path_alias_results(&per_path_results);

    PathSensitiveAliasResult {
        alias: combined,
        paths_total: total_paths,
        paths_feasible: feasible_count,
    }
}

/// Perform interprocedural path-sensitive alias analysis.
///
/// First tries intraprocedural analysis on the oracle's own function.
/// If that returns `Unknown` (e.g., because the function has no branches and the
/// critical context comes from the caller), tries running the per-path solver
/// on each caller function. The `CallDirect` handler in the solver naturally
/// processes callee instructions in the caller's path context.
#[must_use]
pub fn path_sensitive_alias_interprocedural(
    p: ValueId,
    q: ValueId,
    func: &AirFunction,
    module: &AirModule,
    config: &PathSensitiveAliasConfig,
) -> PathSensitiveAliasResult {
    // Try intraprocedural first
    let result = path_sensitive_alias(p, q, func, module, config);
    if !matches!(result.alias, AliasResult::Unknown) {
        return result;
    }

    // Find callers and try each
    for caller_func in &module.functions {
        if caller_func.is_declaration {
            continue;
        }
        if calls_function(caller_func, func.id) {
            let caller_result = path_sensitive_alias(p, q, caller_func, module, config);
            if !matches!(caller_result.alias, AliasResult::Unknown) {
                return caller_result;
            }
        }
    }

    result // Return original if no improvement
}

/// Build a map from `ValueId` to parameter index for a function.
///
/// Traces parameter identity through two patterns:
/// 1. Direct `func.params[i].id → i`
/// 2. SSA propagation: `Phi`, `Copy`, `Cast` chains that derive from a
///    parameter are mapped to the same index. Iterates to fixpoint.
///
/// This is used by interprocedural path-sensitive alias analysis and the PTABen
/// harness to map oracle values back to function parameters for call-site
/// argument remapping.
#[must_use]
pub fn build_param_indices(func: &AirFunction) -> BTreeMap<ValueId, usize> {
    // Step 1: Seed with direct parameter IDs
    let mut param_indices: BTreeMap<ValueId, usize> = func
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id, i))
        .collect();

    // Step 2: SSA propagation — trace through Phi, Copy, Cast chains.
    // Iterate to fixpoint since phi→cast→phi chains may exist.
    for _round in 0..10 {
        let mut changed = false;
        for block in &func.blocks {
            for inst in &block.instructions {
                let Some(dst) = inst.dst else { continue };
                // Skip if already mapped
                if param_indices.contains_key(&dst) {
                    continue;
                }
                match &inst.op {
                    Operation::Phi { incoming } => {
                        // Map phi dst if ALL param-derived incomings agree on the same index
                        let mut found_idx: Option<usize> = None;
                        let mut conflict = false;
                        for (_block_id, value_id) in incoming {
                            if let Some(&idx) = param_indices.get(value_id) {
                                match found_idx {
                                    None => found_idx = Some(idx),
                                    Some(prev) if prev != idx => {
                                        conflict = true;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if !conflict {
                            if let Some(idx) = found_idx {
                                param_indices.insert(dst, idx);
                                changed = true;
                            }
                        }
                    }
                    Operation::Copy | Operation::Cast { .. } => {
                        if let Some(&src) = inst.operands.first() {
                            if let Some(&idx) = param_indices.get(&src) {
                                param_indices.insert(dst, idx);
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if !changed {
            break;
        }
    }

    param_indices
}

/// Like [`path_sensitive_alias_interprocedural`] but also checks `CallIndirect`
/// callers using the resolved call sites map from CG refinement.
///
/// This is critical for virtual diamond inheritance where the oracle function
/// (e.g., `C::h`) is called via `CallIndirect` (virtual dispatch), not `CallDirect`.
///
/// When the oracle values (`p`, `q`) are parameters of the callee, we map them
/// to the actual arguments at the call site before running the caller-level
/// path-sensitive solver. This avoids the problem where per-call-site renaming
/// makes the original parameter `ValueId`s invisible in the caller's PTS.
#[must_use]
pub fn path_sensitive_alias_interprocedural_with_resolved(
    p: ValueId,
    q: ValueId,
    func: &AirFunction,
    module: &AirModule,
    config: &PathSensitiveAliasConfig,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
) -> PathSensitiveAliasResult {
    // Try intraprocedural first
    let result = path_sensitive_alias(p, q, func, module, config);
    // Return early only for definitive results (MustAlias/PartialAlias).
    // NoAlias within a function may mean parameter binding is interprocedural
    // (e.g., virtual method parameters are bound at the caller's call site).
    if matches!(result.alias, AliasResult::Must | AliasResult::Partial) {
        return result;
    }

    // Build parameter index map: callee param ValueId → param position.
    // Traces direct params, -O0 alloca store→load chains, and SSA phi/copy/cast.
    let param_indices = build_param_indices(func);

    // Find callers: check both CallDirect and resolved CallIndirect
    for caller_func in &module.functions {
        if caller_func.is_declaration {
            continue;
        }
        // Find all call sites in this caller that invoke our callee
        let call_sites = find_call_sites_to(caller_func, func.id, resolved_sites);
        if call_sites.is_empty() {
            continue;
        }

        // Approach 1: Map callee params to caller args and query alias.
        // Works when BOTH query values are parameters of the callee function.
        for call_inst in &call_sites {
            let (mapped_p, mapped_q) = map_params_to_args(p, q, call_inst, &param_indices);

            let caller_result =
                path_sensitive_alias(mapped_p, mapped_q, caller_func, module, config);
            if !matches!(caller_result.alias, AliasResult::Unknown) {
                return caller_result;
            }
        }

        // Approach 2: Query renamed callee values in caller's inline context.
        // When a query value is a local (not a param), param-mapping fails because
        // the local ValueId is not defined in the caller. However, the per-path
        // solver inlines the callee at each call site, renaming all callee-defined
        // values. We query using these renamed values so they match the inlined PTS.
        let callee_defs = collect_function_defs(func);
        for call_inst in &call_sites {
            // Derive call_id from inst.id + operands to match the solver's
            // rename derivation (which incorporates operands for uniqueness
            // across different outer call contexts).
            let call_id = {
                let mut h = Vec::new();
                h.extend_from_slice(&call_inst.id.raw().to_le_bytes());
                for op in &call_inst.operands {
                    h.extend_from_slice(&op.raw().to_le_bytes());
                }
                InstId::derive(&h)
            };
            let renamed_p = rename_for_call_site(p, call_id, &callee_defs);
            let renamed_q = rename_for_call_site(q, call_id, &callee_defs);

            saf_log!(pta::solve, reasoning, "path-sensitive rename";
                caller = caller_func.name.as_str(),
                callee = func.name.as_str(),
                p_in_defs = callee_defs.contains(&p),
                q_in_defs = callee_defs.contains(&q),
            );

            let caller_result =
                path_sensitive_alias(renamed_p, renamed_q, caller_func, module, config);

            saf_log!(pta::solve, result, "path-sensitive result";
                alias = format!("{:?}", caller_result.alias).as_str(),
                paths_total = caller_result.paths_total,
                paths_feasible = caller_result.paths_feasible,
            );

            if !matches!(caller_result.alias, AliasResult::Unknown) {
                return caller_result;
            }
        }
    }

    result
}

/// Find all call instructions in `caller` that invoke `callee_id`.
///
/// Returns references to `CallDirect` and resolved `CallIndirect` instructions.
fn find_call_sites_to<'a>(
    caller: &'a AirFunction,
    callee_id: FunctionId,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
) -> Vec<&'a saf_core::air::Instruction> {
    let mut sites = Vec::new();
    for block in &caller.blocks {
        for inst in &block.instructions {
            match &inst.op {
                Operation::CallDirect { callee } if *callee == callee_id => {
                    sites.push(inst);
                }
                Operation::CallIndirect { .. } => {
                    if let Some(targets) = resolved_sites.get(&inst.id) {
                        if targets.contains(&callee_id) {
                            sites.push(inst);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    sites
}

/// Map oracle `ValueId`s from callee parameter space to caller argument space.
///
/// If `p` or `q` is a callee parameter, replaces it with the corresponding
/// argument `ValueId` from the call instruction's operands. Non-parameter values
/// (globals, etc.) are returned unchanged.
fn map_params_to_args(
    p: ValueId,
    q: ValueId,
    call_inst: &saf_core::air::Instruction,
    param_indices: &BTreeMap<ValueId, usize>,
) -> (ValueId, ValueId) {
    let arg_count = if matches!(call_inst.op, Operation::CallIndirect { .. }) {
        // CallIndirect: last operand is the function pointer, rest are args
        call_inst.operands.len().saturating_sub(1)
    } else {
        call_inst.operands.len()
    };

    let map_one = |v: ValueId| -> ValueId {
        if let Some(&idx) = param_indices.get(&v) {
            if idx < arg_count {
                return call_inst.operands[idx];
            }
        }
        v
    };

    (map_one(p), map_one(q))
}

/// Collect all `ValueId`s defined within a function (params + instruction dsts).
///
/// This matches the `callee_defs` set built inside the per-path solver's
/// `CallDirect`/`CallIndirect` handlers, ensuring that the rename mapping
/// is consistent with the solver's inline renaming.
fn collect_function_defs(func: &AirFunction) -> BTreeSet<ValueId> {
    let mut defs = BTreeSet::new();
    for param in &func.params {
        defs.insert(param.id);
    }
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                defs.insert(dst);
            }
        }
    }
    defs
}

/// Rename a `ValueId` as the per-path solver would when inlining a callee.
///
/// If `v` is in `callee_defs`, it gets a call-site-specific rename to avoid
/// pollution between multiple calls to the same function. Global and external
/// values are kept as-is.
fn rename_for_call_site(v: ValueId, call_id: InstId, callee_defs: &BTreeSet<ValueId>) -> ValueId {
    if callee_defs.contains(&v) {
        let mut data = Vec::new();
        data.extend_from_slice(&call_id.raw().to_le_bytes());
        data.extend_from_slice(&v.raw().to_le_bytes());
        ValueId::derive(&data)
    } else {
        v
    }
}

/// Check if a function contains a `CallDirect` to the given callee.
fn calls_function(func: &AirFunction, callee_id: FunctionId) -> bool {
    calls_function_with_resolved(func, callee_id, &BTreeMap::new())
}

/// Check if a function calls the given callee via `CallDirect` or resolved `CallIndirect`.
fn calls_function_with_resolved(
    func: &AirFunction,
    callee_id: FunctionId,
    resolved_sites: &BTreeMap<InstId, Vec<FunctionId>>,
) -> bool {
    for block in &func.blocks {
        for inst in &block.instructions {
            match &inst.op {
                Operation::CallDirect { callee } if *callee == callee_id => {
                    return true;
                }
                Operation::CallIndirect { .. } => {
                    if let Some(targets) = resolved_sites.get(&inst.id) {
                        if targets.contains(&callee_id) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    false
}

struct BranchPoint {
    block_id: BlockId,
    #[allow(dead_code)]
    condition: ValueId,
    then_target: BlockId,
    else_target: BlockId,
}

/// Compute which blocks are reachable and which edges are active from entry
/// given specific branch choices.
///
/// Returns `(reachable_blocks, active_edges)` where active_edges is the set
/// of `(from, to)` block edges that are traversed on this path. This is
/// needed for phi node filtering: a phi incoming from predecessor P is only
/// relevant if the edge `(P, phi_block)` is active — not merely if P is
/// reachable (P may be reachable but take a different branch on this path).
fn compute_reachable_blocks_and_edges(
    cfg: &Cfg,
    branches: &[BranchPoint],
    choices: &[bool],
) -> (BTreeSet<BlockId>, BTreeSet<(BlockId, BlockId)>) {
    // Build a map of restricted successors
    let branch_map: BTreeMap<BlockId, (BlockId, BlockId)> = branches
        .iter()
        .map(|b| (b.block_id, (b.then_target, b.else_target)))
        .collect();

    let mut reachable = BTreeSet::new();
    let mut active_edges = BTreeSet::new();
    let mut worklist = vec![cfg.entry];

    while let Some(block) = worklist.pop() {
        if reachable.contains(&block) {
            continue;
        }
        reachable.insert(block);

        // Get successors, but for CondBr blocks, only follow the chosen branch
        if let Some(&(then_target, else_target)) = branch_map.get(&block) {
            let idx = branches.iter().position(|b| b.block_id == block);
            if let Some(i) = idx {
                if i < choices.len() {
                    if choices[i] {
                        active_edges.insert((block, then_target));
                        worklist.push(then_target);
                    } else {
                        active_edges.insert((block, else_target));
                        worklist.push(else_target);
                    }
                    continue;
                }
            }
        }

        // For non-CondBr blocks or unknown branches, follow all successors
        if let Some(succs) = cfg.successors_of(block) {
            for &s in succs {
                active_edges.insert((block, s));
                worklist.push(s);
            }
        }
    }

    (reachable, active_edges)
}

/// Create a renamed copy of an instruction for call-site-specific inlining.
///
/// Applies the rename function to all `ValueId`s in the instruction (dst and
/// operands). The rename function should return the original `ValueId` for
/// values that shouldn't be renamed (e.g., caller/global references).
fn rename_instruction(
    inst: &saf_core::air::Instruction,
    rename: &dyn Fn(ValueId) -> ValueId,
) -> saf_core::air::Instruction {
    saf_core::air::Instruction {
        id: inst.id,
        op: inst.op.clone(),
        operands: inst.operands.iter().map(|&v| rename(v)).collect(),
        dst: inst.dst.map(rename),
        span: inst.span.clone(),
        symbol: inst.symbol.clone(),
        result_type: inst.result_type,
        extensions: inst.extensions.clone(),
    }
}

/// Decompose an aggregate initializer into per-field PTA entries.
///
/// For vtable globals like `{ [ptr @A::f, ptr @B::g], [ptr @C::h] }`,
/// this creates field-indexed locations so that GEP + Load chains
/// can resolve function pointers through vtable dispatch.
fn decompose_aggregate_init(
    elements: &[saf_core::air::Constant],
    obj: ObjId,
    base_path: &FieldPath,
    factory: &mut LocationFactory,
    pts: &mut PointsToMap,
) {
    for (i, element) in elements.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let field_path = base_path.extend(&FieldPath::field(i as u32));
        match element {
            saf_core::air::Constant::GlobalRef(target_id) => {
                let field_loc = factory.get_or_create(obj, field_path);
                let field_loc_val = ValueId::new(field_loc.raw());
                // Store target's pts into this field location
                let target_pts_set = pts.get(target_id).cloned().unwrap_or_default();
                if !target_pts_set.is_empty() {
                    let set = pts.entry(field_loc_val).or_default();
                    for loc in target_pts_set {
                        set.insert(loc);
                    }
                }
            }
            saf_core::air::Constant::Aggregate { elements: nested } => {
                decompose_aggregate_init(nested, obj, &field_path, factory, pts);
            }
            _ => {}
        }
    }
}

/// Flow-sensitive per-path PTA solver.
///
/// Processes blocks in topological order within the given reachable set.
/// Uses strong updates for stores to singleton pointers (allocas), which
/// allows later assignments to override earlier ones on the same path.
/// This is critical for path2-style tests where entry block stores `p = &a`
/// and then the taken branch stores `p = &b` — with strong updates, only
/// `p → &b` remains.
///
/// Also handles interprocedural flow (callee constraints) with strong updates.
fn solve_per_path_flow_sensitive(
    func: &AirFunction,
    module: &AirModule,
    cfg: &Cfg,
    reachable: &BTreeSet<BlockId>,
    active_edges: &BTreeSet<(BlockId, BlockId)>,
    factory: &mut LocationFactory,
    max_iterations: usize,
) -> PointsToMap {
    let mut pts: PointsToMap = BTreeMap::new();

    // Track which pointers are "strong" (single alloca target, no phi merging)
    let mut strong_ptrs: BTreeSet<ValueId> = BTreeSet::new();

    // Phase 1: Add global addr constraints (always active)
    for global in &module.globals {
        let loc = factory.get_or_create(global.obj, FieldPath::empty());
        pts.entry(global.id).or_default().insert(loc);

        if let Some(saf_core::air::Constant::GlobalRef(target_id)) = &global.init {
            pts.entry(*target_id).or_default();
        }
    }
    // Phase 1b: Process global initializer stores.
    // When `@obj = global ptr @g`, the content at obj_loc should be {g_loc}.
    // This is equivalent to `store @g, @obj` at program start.
    // Also decompose Aggregate initializers (vtables) so that function pointers
    // stored in vtable globals are discoverable through GEP + Load chains.
    for global in &module.globals {
        match &global.init {
            Some(saf_core::air::Constant::GlobalRef(target_id)) => {
                let global_loc = factory.get_or_create(global.obj, FieldPath::empty());
                let target_pts = pts.get(target_id).cloned().unwrap_or_default();
                if !target_pts.is_empty() {
                    let loc_val = ValueId::new(global_loc.raw());
                    let set = pts.entry(loc_val).or_default();
                    for loc in target_pts {
                        set.insert(loc);
                    }
                }
            }
            Some(saf_core::air::Constant::Aggregate { elements }) => {
                // Decompose aggregate (vtable) initializers: for each element
                // that is a GlobalRef, model a store at the corresponding field.
                decompose_aggregate_init(
                    elements,
                    global.obj,
                    &FieldPath::empty(),
                    factory,
                    &mut pts,
                );
            }
            _ => {}
        }
    }

    // Add function addr constraints
    for f in &module.functions {
        let obj = ObjId::new(f.id.raw());
        let loc = factory.get_or_create(obj, FieldPath::empty());
        pts.entry(ValueId::new(f.id.raw())).or_default().insert(loc);
    }

    // Phase 2: Compute topological order of reachable blocks
    let topo_order = compute_topo_order(cfg, reachable);

    // Detect whether the reachable subgraph is acyclic.
    // A back-edge exists if any edge (a, b) in `active_edges` has b appearing
    // before a in `topo_order`.
    let topo_position: BTreeMap<BlockId, usize> = topo_order
        .iter()
        .enumerate()
        .map(|(i, &b)| (b, i))
        .collect();
    let is_acyclic = !active_edges.iter().any(|(a, b)| {
        matches!(
            (topo_position.get(a), topo_position.get(b)),
            (Some(&pa), Some(&pb)) if pb <= pa
        )
    });

    // Phase 3: Process blocks in topological order
    if is_acyclic {
        // Acyclic subgraph: single topological pass is exact.
        for &block_id in &topo_order {
            let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
                continue;
            };
            for inst in &block.instructions {
                process_instruction_flow_sensitive(
                    inst,
                    block_id,
                    module,
                    factory,
                    &mut pts,
                    &mut strong_ptrs,
                    active_edges,
                    0, // inline_depth
                );
            }
        }
    } else {
        // Cyclic subgraph: use fixpoint loop with capped iterations.
        for _iteration in 0..max_iterations.min(10) {
            let mut changed = false;

            for &block_id in &topo_order {
                let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
                    continue;
                };

                for inst in &block.instructions {
                    let inst_changed = process_instruction_flow_sensitive(
                        inst,
                        block_id,
                        module,
                        factory,
                        &mut pts,
                        &mut strong_ptrs,
                        active_edges,
                        0, // inline_depth
                    );
                    changed |= inst_changed;
                }
            }

            if !changed {
                break;
            }
        }
    }

    pts
}

/// Process a single instruction with flow-sensitive strong update semantics.
///
/// Returns true if the points-to map changed.
// `ptr` (pointer operand) and `pts` (points-to set) are distinct concepts
// NOTE: This function implements flow-sensitive processing for all instruction
// types as a single cohesive unit. Splitting would obscure the algorithm.
#[allow(
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
fn process_instruction_flow_sensitive(
    inst: &saf_core::air::Instruction,
    current_block: BlockId,
    module: &AirModule,
    factory: &mut LocationFactory,
    pts: &mut PointsToMap,
    strong_ptrs: &mut BTreeSet<ValueId>,
    active_edges: &BTreeSet<(BlockId, BlockId)>,
    inline_depth: usize,
) -> bool {
    const MAX_INLINE_DEPTH: usize = 5;
    let mut changed = false;

    match &inst.op {
        Operation::Alloca { .. } | Operation::HeapAlloc { .. } => {
            if let Some(dst) = inst.dst {
                let obj = ObjId::new(dst.raw());
                let loc = factory.get_or_create(obj, FieldPath::empty());
                let set = pts.entry(dst).or_default();
                if set.insert(loc) {
                    changed = true;
                }
                strong_ptrs.insert(dst);
            }
        }
        Operation::Global { obj } => {
            if let Some(dst) = inst.dst {
                let loc = factory.get_or_create(*obj, FieldPath::empty());
                let set = pts.entry(dst).or_default();
                if set.insert(loc) {
                    changed = true;
                }
            }
        }
        Operation::Store => {
            // Store: operands[0] = value, operands[1] = ptr
            if inst.operands.len() >= 2 {
                let value = inst.operands[0];
                let ptr = inst.operands[1];

                let value_pts = pts.get(&value).cloned().unwrap_or_default();
                let mut ptr_pts = pts.get(&ptr).cloned().unwrap_or_default();

                // Synthetic location for unknown pointers (e.g., `ptr undef`).
                // When the store destination has no allocation, create a
                // synthetic object so the store can propagate. The subsequent
                // load through the same pointer will find this location.
                // Mirrors the "symbolic object for uninitialized loads" below.
                if ptr_pts.is_empty() {
                    let syn_obj = ObjId::new(ptr.raw());
                    let syn_loc = factory.get_or_create(syn_obj, FieldPath::empty());
                    pts.entry(ptr).or_default().insert(syn_loc);
                    ptr_pts = pts.get(&ptr).cloned().unwrap_or_default();
                    changed = true;
                }

                // Strong update: if ptr points to exactly one location,
                // REPLACE the stored content (don't union)
                if ptr_pts.len() == 1 {
                    for &target_loc in &ptr_pts {
                        // Find the ValueId that represents the content at target_loc
                        // In LLVM IR: `store val, ptr` means *ptr = val
                        // We need to propagate val's pts to all loads from ptr
                        // For now, we store val's pts as what's "at" target_loc
                        // by propagating to any value that loads from ptr
                        let target_val_id = ValueId::new(target_loc.raw());
                        let set = pts.entry(target_val_id).or_default();
                        if *set != value_pts {
                            set.clone_from(&value_pts);
                            changed = true;
                        }
                    }
                } else {
                    // Weak update: union
                    for &target_loc in &ptr_pts {
                        let target_val_id = ValueId::new(target_loc.raw());
                        let set = pts.entry(target_val_id).or_default();
                        for &loc in &value_pts {
                            if set.insert(loc) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        Operation::Load => {
            // Load: dst = *src_ptr
            if let (Some(dst), Some(&src_ptr)) = (inst.dst, inst.operands.first()) {
                let src_pts = pts.get(&src_ptr).cloned().unwrap_or_default();

                let mut loaded = BTreeSet::new();
                for &target_loc in &src_pts {
                    let target_val_id = ValueId::new(target_loc.raw());
                    if let Some(target_pts) = pts.get(&target_val_id) {
                        for &loc in target_pts {
                            loaded.insert(loc);
                        }
                    } else if src_pts.len() == 1 {
                        // Uninitialized location: the alloca was never stored to.
                        // Create a fresh symbolic object representing "whatever
                        // garbage value lives at this uninitialized memory".
                        // This allows `a = b` followed by `*a = &q; obj = *b`
                        // to correctly propagate through the shared symbolic loc.
                        let sym_obj = ObjId::new(target_loc.raw());
                        let sym_loc = factory.get_or_create(sym_obj, FieldPath::empty());
                        loaded.insert(sym_loc);
                    }
                }

                let set = pts.entry(dst).or_default();
                for loc in loaded {
                    if set.insert(loc) {
                        changed = true;
                    }
                }
            }
        }
        Operation::Copy | Operation::Cast { .. } | Operation::Freeze => {
            if let (Some(dst), Some(&src)) = (inst.dst, inst.operands.first()) {
                let src_pts = pts.get(&src).cloned().unwrap_or_default();
                let set = pts.entry(dst).or_default();
                for loc in src_pts {
                    if set.insert(loc) {
                        changed = true;
                    }
                }
            }
        }
        Operation::Gep { field_path } => {
            if let (Some(dst), Some(&src_ptr)) = (inst.dst, inst.operands.first()) {
                let (mut path, _index_operands) = convert_air_field_path(field_path, inst);
                let src_pts = pts.get(&src_ptr).cloned().unwrap_or_default();

                // In LLVM IR, the first GEP index is always the pointer
                // dereference (array offset from the base pointer). For
                // multi-index GEPs (struct/field access), it is 0 and should
                // be skipped to match `decompose_aggregate_init` field paths.
                // For single-index GEPs (pointer arithmetic), a non-zero
                // offset advances by N elements within the containing array.
                let array_offset = if let Some(&PathStep::Field { index: n }) = path.steps.first() {
                    path.steps.remove(0);
                    n
                } else {
                    0
                };

                let mut new_pts = BTreeSet::new();
                for &src_loc in &src_pts {
                    if let Some(loc_info) = factory.all_locations().get(&src_loc) {
                        let mut new_path = loc_info.path.clone();
                        // Apply array offset: advance the last field index
                        // by N (e.g., vtable slot 1 from vtable pointer at
                        // slot 0 means last_index += 1).
                        if array_offset > 0 {
                            if let Some(PathStep::Field { index }) = new_path.steps.last_mut() {
                                *index += array_offset;
                            }
                        }
                        for step in &path.steps {
                            new_path.steps.push(step.clone());
                        }
                        let new_loc = factory.get_or_create(loc_info.obj, new_path);
                        new_pts.insert(new_loc);
                    }
                }
                let set = pts.entry(dst).or_default();
                for loc in new_pts {
                    if set.insert(loc) {
                        changed = true;
                    }
                }
            }
        }
        Operation::Phi { incoming } => {
            if let Some(dst) = inst.dst {
                for (pred_block, value_id) in incoming {
                    // Only process incoming values whose edge (pred → current_block)
                    // is active on this path. This is more precise than checking
                    // block reachability: a predecessor may be reachable but its edge
                    // to this block may not be active (e.g., entry block branches to
                    // if.then, not if.end — entry is reachable but the edge
                    // entry→if.end is not active on the then-path).
                    if !active_edges.contains(&(*pred_block, current_block)) {
                        continue;
                    }
                    let src_pts = pts.get(value_id).cloned().unwrap_or_default();
                    let set = pts.entry(dst).or_default();
                    for loc in src_pts {
                        if set.insert(loc) {
                            changed = true;
                        }
                    }
                }
            }
        }
        Operation::Select => {
            if let Some(dst) = inst.dst {
                for idx in 1..=2 {
                    if let Some(&src) = inst.operands.get(idx) {
                        let src_pts = pts.get(&src).cloned().unwrap_or_default();
                        let set = pts.entry(dst).or_default();
                        for loc in src_pts {
                            if set.insert(loc) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        Operation::Memcpy => {
            if inst.operands.len() >= 2 {
                let src_pts = pts.get(&inst.operands[1]).cloned().unwrap_or_default();
                let set = pts.entry(inst.operands[0]).or_default();
                for loc in src_pts {
                    if set.insert(loc) {
                        changed = true;
                    }
                }
            }
        }
        Operation::CallDirect { callee } => {
            // Handle interprocedural constraints with per-call-site renaming.
            // Each call site gets unique ValueIds for the callee's internal
            // SSA values, preventing pollution across multiple calls to the
            // same function (e.g., foo(a,b,c); foo(d,e,f) in cs2).
            if let Some(callee_func) = module.function(*callee) {
                // Collect all ValueIds defined within the callee
                let mut callee_defs: BTreeSet<ValueId> = BTreeSet::new();
                for param in &callee_func.params {
                    callee_defs.insert(param.id);
                }
                for callee_block in &callee_func.blocks {
                    for callee_inst in &callee_block.instructions {
                        if let Some(dst) = callee_inst.dst {
                            callee_defs.insert(dst);
                        }
                    }
                }

                // Build rename: only renames callee-defined ValueIds.
                // Derive call_id from inst.id + operands so that recursive
                // inlines from different outer call sites get unique renames.
                let call_id = {
                    let mut h = Vec::new();
                    h.extend_from_slice(&inst.id.raw().to_le_bytes());
                    for op in &inst.operands {
                        h.extend_from_slice(&op.raw().to_le_bytes());
                    }
                    InstId::derive(&h)
                };
                let rename = |v: ValueId| -> ValueId {
                    if callee_defs.contains(&v) {
                        let mut data = Vec::new();
                        data.extend_from_slice(&call_id.raw().to_le_bytes());
                        data.extend_from_slice(&v.raw().to_le_bytes());
                        ValueId::derive(&data)
                    } else {
                        v // Keep caller/global references as-is
                    }
                };

                // Arg → renamed Param
                for (i, param) in callee_func.params.iter().enumerate() {
                    if let Some(&arg) = inst.operands.get(i) {
                        let arg_pts = pts.get(&arg).cloned().unwrap_or_default();
                        let renamed_param = rename(param.id);
                        let set = pts.entry(renamed_param).or_default();
                        for loc in arg_pts {
                            if set.insert(loc) {
                                changed = true;
                            }
                        }
                    }
                }
                // Process callee body with renamed instructions.
                // Build "all edges active" set for the callee so phi nodes
                // inside the callee process all incoming values (since we're
                // not doing per-path analysis on the callee).
                if !callee_func.is_declaration {
                    let callee_cfg = Cfg::build(callee_func);
                    let callee_all_edges: BTreeSet<(BlockId, BlockId)> = callee_func
                        .blocks
                        .iter()
                        .flat_map(|b| {
                            let bid = b.id;
                            callee_cfg
                                .successors_of(bid)
                                .into_iter()
                                .flat_map(move |succs| succs.iter().map(move |&s| (bid, s)))
                        })
                        .collect();
                    for callee_block in &callee_func.blocks {
                        for callee_inst in &callee_block.instructions {
                            if inline_depth >= MAX_INLINE_DEPTH
                                && matches!(callee_inst.op, Operation::CallDirect { .. })
                            {
                                continue;
                            }
                            let renamed_inst = rename_instruction(callee_inst, &rename);
                            let callee_changed = process_instruction_flow_sensitive(
                                &renamed_inst,
                                callee_block.id,
                                module,
                                factory,
                                pts,
                                strong_ptrs,
                                &callee_all_edges,
                                inline_depth + 1,
                            );
                            changed |= callee_changed;
                        }
                    }
                }
                // Return → caller (using renamed ret value)
                if let Some(dst) = inst.dst {
                    for callee_block in &callee_func.blocks {
                        for callee_inst in &callee_block.instructions {
                            if let Operation::Ret = &callee_inst.op {
                                if let Some(&ret_val) = callee_inst.operands.first() {
                                    let renamed_ret = rename(ret_val);
                                    let ret_pts =
                                        pts.get(&renamed_ret).cloned().unwrap_or_default();
                                    let set = pts.entry(dst).or_default();
                                    for loc in ret_pts {
                                        if set.insert(loc) {
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Operation::CallIndirect { .. } => {
            // Resolve function pointer through pts and inline the callee.
            // In LLVM IR, the function pointer is the LAST operand.
            if let Some(&fp_value) = inst.operands.last() {
                let fp_pts = pts.get(&fp_value).cloned().unwrap_or_default();
                // Each LocId in fp_pts corresponds to a function object.
                // Try to find the matching function by ObjId(func.id).
                for fp_loc in &fp_pts {
                    // The function addr constraint creates: ObjId(func.id) → LocId
                    // We need to find which function has this LocId
                    let fp_loc_info = factory.all_locations().get(fp_loc);
                    let callee_func_id = fp_loc_info.map(|info| FunctionId::new(info.obj.raw()));

                    let callee_func = callee_func_id.and_then(|fid| module.function(fid));

                    if let Some(callee_func) = callee_func {
                        // Collect callee-defined ValueIds for renaming
                        let mut callee_defs: BTreeSet<ValueId> = BTreeSet::new();
                        for param in &callee_func.params {
                            callee_defs.insert(param.id);
                        }
                        for callee_block in &callee_func.blocks {
                            for callee_inst in &callee_block.instructions {
                                if let Some(dst) = callee_inst.dst {
                                    callee_defs.insert(dst);
                                }
                            }
                        }

                        let call_id = {
                            let mut h = Vec::new();
                            h.extend_from_slice(&inst.id.raw().to_le_bytes());
                            for op in &inst.operands {
                                h.extend_from_slice(&op.raw().to_le_bytes());
                            }
                            InstId::derive(&h)
                        };
                        let rename = |v: ValueId| -> ValueId {
                            if callee_defs.contains(&v) {
                                let mut data = Vec::new();
                                data.extend_from_slice(&call_id.raw().to_le_bytes());
                                data.extend_from_slice(&v.raw().to_le_bytes());
                                ValueId::derive(&data)
                            } else {
                                v
                            }
                        };

                        // Arg → renamed Param (operands minus the last fp value)
                        let arg_count = inst.operands.len().saturating_sub(1);
                        for (i, param) in callee_func.params.iter().enumerate() {
                            if i < arg_count {
                                let arg = inst.operands[i];
                                let arg_pts_set = pts.get(&arg).cloned().unwrap_or_default();
                                let renamed_param = rename(param.id);
                                let set = pts.entry(renamed_param).or_default();
                                for loc in arg_pts_set {
                                    if set.insert(loc) {
                                        changed = true;
                                    }
                                }
                            }
                        }
                        // Process callee body with renamed instructions
                        if !callee_func.is_declaration {
                            let callee_cfg = Cfg::build(callee_func);
                            let callee_all_edges: BTreeSet<(BlockId, BlockId)> = callee_func
                                .blocks
                                .iter()
                                .flat_map(|b| {
                                    let bid = b.id;
                                    callee_cfg
                                        .successors_of(bid)
                                        .into_iter()
                                        .flat_map(move |succs| succs.iter().map(move |&s| (bid, s)))
                                })
                                .collect();
                            for callee_block in &callee_func.blocks {
                                for callee_inst in &callee_block.instructions {
                                    if inline_depth >= MAX_INLINE_DEPTH
                                        && matches!(
                                            callee_inst.op,
                                            Operation::CallDirect { .. }
                                                | Operation::CallIndirect { .. }
                                        )
                                    {
                                        continue;
                                    }
                                    let renamed_inst = rename_instruction(callee_inst, &rename);
                                    let callee_changed = process_instruction_flow_sensitive(
                                        &renamed_inst,
                                        callee_block.id,
                                        module,
                                        factory,
                                        pts,
                                        strong_ptrs,
                                        &callee_all_edges,
                                        inline_depth + 1,
                                    );
                                    changed |= callee_changed;
                                }
                            }
                        }
                        // Return → caller
                        if let Some(dst) = inst.dst {
                            for callee_block in &callee_func.blocks {
                                for callee_inst in &callee_block.instructions {
                                    if let Operation::Ret = &callee_inst.op {
                                        if let Some(&ret_val) = callee_inst.operands.first() {
                                            let renamed_ret = rename(ret_val);
                                            let ret_pts_set =
                                                pts.get(&renamed_ret).cloned().unwrap_or_default();
                                            let set = pts.entry(dst).or_default();
                                            for loc in ret_pts_set {
                                                if set.insert(loc) {
                                                    changed = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    changed
}

/// Compute a topological order of reachable blocks.
fn compute_topo_order(cfg: &Cfg, reachable: &BTreeSet<BlockId>) -> Vec<BlockId> {
    fn dfs(
        block: BlockId,
        cfg: &Cfg,
        reachable: &BTreeSet<BlockId>,
        visited: &mut BTreeSet<BlockId>,
        in_stack: &mut BTreeSet<BlockId>,
        order: &mut Vec<BlockId>,
    ) {
        if visited.contains(&block) {
            return;
        }
        visited.insert(block);
        in_stack.insert(block);

        if let Some(succs) = cfg.successors_of(block) {
            for &succ in succs {
                if reachable.contains(&succ) && !in_stack.contains(&succ) {
                    dfs(succ, cfg, reachable, visited, in_stack, order);
                }
            }
        }

        in_stack.remove(&block);
        order.push(block);
    }

    let mut order = Vec::new();
    let mut visited = BTreeSet::new();
    let mut in_stack = BTreeSet::new();

    if reachable.contains(&cfg.entry) {
        dfs(
            cfg.entry,
            cfg,
            reachable,
            &mut visited,
            &mut in_stack,
            &mut order,
        );
    }

    // Reverse for topological order (entry first)
    order.reverse();
    order
}

/// Convert AIR `FieldPath` to PTA `FieldPath`, extracting index operands.
fn convert_air_field_path(
    air_path: &saf_core::air::FieldPath,
    inst: &saf_core::air::Instruction,
) -> (FieldPath, Vec<ValueId>) {
    use super::location::IndexExpr;

    let mut index_operands = Vec::new();
    let mut operand_idx = 1; // Skip operands[0] which is the base pointer

    let steps = air_path
        .steps
        .iter()
        .map(|step| match step {
            saf_core::air::FieldStep::Index => {
                if let Some(&operand) = inst.operands.get(operand_idx) {
                    index_operands.push(operand);
                }
                operand_idx += 1;
                PathStep::Index(IndexExpr::Unknown)
            }
            saf_core::air::FieldStep::Field { index } => {
                operand_idx += 1;
                PathStep::Field { index: *index }
            }
        })
        .collect();

    (FieldPath { steps }, index_operands)
}

/// Compute alias result from two points-to sets.
fn compute_alias_from_pts(
    p: ValueId,
    q: ValueId,
    pts: &PointsToMap,
    locations: &FxHashMap<LocId, super::location::Location>,
) -> AliasResult {
    let p_pts = pts.get(&p);
    let q_pts = pts.get(&q);

    match (p_pts, q_pts) {
        (None, _) | (_, None) => AliasResult::Unknown,
        (Some(p_set), Some(q_set)) => {
            if p_set.is_empty() || q_set.is_empty() {
                AliasResult::No // On this path, pointer has no targets
            } else if p_set == q_set && p_set.len() == 1 {
                AliasResult::Must
            } else if p_set == q_set {
                AliasResult::May
            } else if p_set.is_disjoint(q_set) && !has_field_overlap(p_set, q_set, locations) {
                AliasResult::No
            } else if p_set.is_subset(q_set) || q_set.is_subset(p_set) {
                AliasResult::Partial
            } else {
                AliasResult::May
            }
        }
    }
}

/// Check if two location sets have field path overlap.
fn has_field_overlap(
    p_set: &BTreeSet<LocId>,
    q_set: &BTreeSet<LocId>,
    locations: &FxHashMap<LocId, super::location::Location>,
) -> bool {
    for &p_loc in p_set {
        for &q_loc in q_set {
            if let (Some(p), Some(q)) = (locations.get(&p_loc), locations.get(&q_loc)) {
                if p.obj == q.obj {
                    let p_path = &p.path.steps;
                    let q_path = &q.path.steps;
                    let (shorter, longer) = if p_path.len() <= q_path.len() {
                        (p_path, q_path)
                    } else {
                        (q_path, p_path)
                    };
                    if longer.starts_with(shorter) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Combine alias results from multiple paths.
///
/// `Unknown` results are filtered out before combining: they represent paths
/// where the query point is unreachable, and shouldn't influence the combined
/// result. For example, a loop back-edge path that never reaches the alias
/// check should not dilute 7 unanimous `NoAlias` paths into `May`.
fn combine_path_alias_results(results: &[AliasResult]) -> AliasResult {
    // Filter out Unknown (unreachable query point on that path)
    let reachable: Vec<AliasResult> = results
        .iter()
        .copied()
        .filter(|&r| r != AliasResult::Unknown)
        .collect();

    if reachable.is_empty() {
        return AliasResult::Unknown;
    }

    let first = reachable[0];
    if reachable.iter().all(|&r| r == first) {
        return first;
    }

    let any_must = reachable.iter().any(|&r| r == AliasResult::Must);
    let any_no = reachable.iter().any(|&r| r == AliasResult::No);
    let all_no = reachable.iter().all(|&r| r == AliasResult::No);
    let all_must = reachable.iter().all(|&r| r == AliasResult::Must);

    if all_no {
        AliasResult::No
    } else if all_must {
        AliasResult::Must
    } else if any_must && any_no {
        AliasResult::May
    } else if any_no {
        // Some paths don't alias — overall may
        AliasResult::May
    } else {
        AliasResult::May
    }
}

// Keep the old types for API compatibility but mark unused fields
pub struct ValueOriginMap {
    _empty: (),
}

impl ValueOriginMap {
    /// Build - now a no-op since we use per-path constraint solving.
    ///
    /// The real path-sensitive analysis is done by `path_sensitive_alias()`
    /// which re-solves PTA constraints per path, making origin tracking unnecessary.
    #[must_use]
    pub fn build(_module: &AirModule, _pts: &BTreeMap<ValueId, BTreeSet<LocId>>) -> Self {
        Self { _empty: () }
    }

    /// Check if a value has any origin tracking — always false for the stub.
    #[must_use]
    pub fn has_origins(&self, _value: ValueId) -> bool {
        false
    }

    /// Get origins for a (value, location) pair — always `None` for the stub.
    #[must_use]
    pub fn get_origins(&self, _value: ValueId, _loc: LocId) -> Option<&Vec<Vec<BranchCondition>>> {
        None
    }
}

/// A branch condition under which a points-to entry exists (stub for API compatibility).
pub struct BranchCondition {
    /// The comparison instruction.
    pub condition: ValueId,
    /// Which branch was taken.
    pub branch_taken: bool,
    /// Block containing the branch.
    pub block: BlockId,
}

/// Filter - now a no-op since we use per-path constraint solving
#[cfg(feature = "z3-solver")]
#[must_use]
pub fn filter_pts_for_path(
    _value: ValueId,
    pts: &BTreeSet<LocId>,
    _origin_map: &ValueOriginMap,
    _path: &PathCondition,
) -> BTreeSet<LocId> {
    pts.clone()
}
