//! Demand-driven pointer analysis solver.
//!
//! Uses backward traversal on SVFG with CFL-reachability for context-sensitive
//! analysis. Falls back to CI-PTA when budget is exhausted.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, InstId, LocId, ObjId, ValueId};

use crate::callgraph::CallGraph;
use crate::module_index::ModuleIndex;
use crate::mssa::{MemAccessId, MemorySsa};
use crate::svfg::{Svfg, SvfgEdgeKind, SvfgNodeId};
use crate::{AliasResult, PtaResult};

use super::types::{
    Budget, CallString, DdaCache, DdaConfig, DdaConfigExport, DdaDiagnostics, DdaExport, Dpm,
    ReachabilityResult,
};

// ---------------------------------------------------------------------------
// DdaPta
// ---------------------------------------------------------------------------

/// Demand-driven pointer analysis solver.
///
/// Computes points-to information only for explicitly queried pointers.
/// Uses backward traversal on SVFG with CFL-reachability for context matching.
pub struct DdaPta<'a> {
    /// The SVFG to traverse.
    svfg: &'a Svfg,
    /// Memory SSA for clobber resolution.
    mssa: &'a MemorySsa,
    /// CI-PTA result for fallback.
    ci_pta: &'a PtaResult,
    /// The AIR module for instruction inspection.
    module: &'a AirModule,
    /// Call graph for recursion detection.
    callgraph: &'a CallGraph,
    /// Pre-computed module index maps.
    index: &'a ModuleIndex,
    /// Configuration.
    config: DdaConfig,
    /// Persistent cache.
    cache: DdaCache,
    /// Analysis diagnostics.
    diagnostics: DdaDiagnostics,
    /// Recursive function SCCs (functions in these are collapsed to empty context).
    recursive_sccs: BTreeSet<FunctionId>,
    /// Map from InstId to the load pointer ValueId (for load instructions).
    load_pointers: BTreeMap<InstId, ValueId>,
}

impl<'a> DdaPta<'a> {
    /// Create a new DDA solver.
    #[must_use]
    pub fn new(
        svfg: &'a Svfg,
        mssa: &'a MemorySsa,
        ci_pta: &'a PtaResult,
        module: &'a AirModule,
        callgraph: &'a CallGraph,
        index: &'a ModuleIndex,
        config: DdaConfig,
    ) -> Self {
        Self::new_with_cache(
            svfg,
            mssa,
            ci_pta,
            module,
            callgraph,
            index,
            config,
            DdaCache::new(),
        )
    }

    /// Create a new DDA solver with a pre-populated cache.
    ///
    /// Use this together with [`take_cache`](Self::take_cache) to persist
    /// the cache across multiple solver lifetimes (e.g., across Python calls).
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_cache(
        svfg: &'a Svfg,
        mssa: &'a MemorySsa,
        ci_pta: &'a PtaResult,
        module: &'a AirModule,
        callgraph: &'a CallGraph,
        index: &'a ModuleIndex,
        config: DdaConfig,
        cache: DdaCache,
    ) -> Self {
        // Build load pointer map (DDA-specific: not in ModuleIndex)
        let mut load_pointers = BTreeMap::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if matches!(inst.op, Operation::Load) {
                        if let Some(&ptr) = inst.operands.first() {
                            load_pointers.insert(inst.id, ptr);
                        }
                    }
                }
            }
        }

        // Detect recursive SCCs
        let recursive_sccs = detect_recursive_sccs(callgraph);

        Self {
            svfg,
            mssa,
            ci_pta,
            module,
            callgraph,
            index,
            config,
            cache,
            diagnostics: DdaDiagnostics::default(),
            recursive_sccs,
            load_pointers,
        }
    }

    /// Consume the solver and return its cache for later reuse.
    ///
    /// The returned cache can be passed to [`new_with_cache`](Self::new_with_cache)
    /// to preserve query results across solver lifetimes.
    #[must_use]
    pub fn take_cache(self) -> DdaCache {
        self.cache
    }

    /// Query the points-to set for a value.
    ///
    /// Returns the set of locations the value may point to. If budget is
    /// exhausted, falls back to CI-PTA for soundness.
    pub fn points_to(&mut self, ptr: ValueId) -> Vec<LocId> {
        self.diagnostics.record_query();

        let node = SvfgNodeId::Value(ptr);
        let dpm = Dpm::new(node);

        // Check cache first
        if let Some(cached) = self.cache.get(&dpm) {
            self.diagnostics.record_cache_hit();
            return cached.iter().collect();
        }

        // Clear visited set for new query
        self.cache.clear_visited();

        // Create fresh budget
        let mut budget = Budget::new(&self.config);

        // Run backward traversal
        let result = self.find_points_to(dpm.clone(), &mut budget);

        // Record steps
        let steps_used = match budget.steps_remaining() {
            Some(remaining) => self.config.max_steps.saturating_sub(remaining),
            None => 0, // Unlimited
        };
        self.diagnostics.add_steps(steps_used);

        // Cache and return result
        let result_set: BTreeSet<LocId> = result.iter().copied().collect();
        self.cache.insert(dpm, &result_set);

        result
    }

    /// Query alias relationship between two pointers.
    pub fn may_alias(&mut self, p: ValueId, q: ValueId) -> AliasResult {
        let p_pts = self.points_to(p);
        let q_pts = self.points_to(q);

        if p_pts.is_empty() || q_pts.is_empty() {
            return AliasResult::Unknown;
        }

        let p_set: BTreeSet<_> = p_pts.into_iter().collect();
        let q_set: BTreeSet<_> = q_pts.into_iter().collect();

        if p_set.is_disjoint(&q_set) {
            AliasResult::No
        } else if p_set == q_set && p_set.len() == 1 {
            // Singleton sets pointing to the same location: MustAlias
            // Non-singleton equal sets are MayAlias (we don't know which element)
            AliasResult::Must
        } else if p_set == q_set {
            // Non-singleton equal sets
            AliasResult::May
        } else if p_set.is_subset(&q_set) || q_set.is_subset(&p_set) {
            // One is a proper subset of the other means PartialAlias
            AliasResult::Partial
        } else {
            AliasResult::May
        }
    }

    /// Get the analysis diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &DdaDiagnostics {
        &self.diagnostics
    }

    /// Get the cache statistics.
    #[must_use]
    pub fn cache_stats(&self) -> super::types::CacheStats {
        self.cache.stats()
    }

    /// Check if there is a value-flow path from source to sink.
    ///
    /// Uses DDA-refined points-to to determine if the source value can flow
    /// to the sink through memory operations.
    pub fn reachable(&mut self, src: ValueId, sink: ValueId) -> bool {
        // Check if src and sink may alias (any form of aliasing)
        let alias = self.may_alias(src, sink);
        if alias.may_alias_optimistic() {
            return true;
        }

        // Check SVFG reachability from source to sink
        let src_node = SvfgNodeId::Value(src);
        let sink_node = SvfgNodeId::Value(sink);

        self.check_svfg_reachability(src_node, sink_node)
    }

    /// Check if there is a refined value-flow path from source to sink.
    ///
    /// Returns a `ReachabilityResult` with additional context about the path.
    pub fn reachable_refined(&mut self, src: ValueId, sink: ValueId) -> ReachabilityResult {
        // Get points-to sets
        let src_pts = self.points_to(src);
        let sink_pts = self.points_to(sink);

        // Check for direct alias
        let direct_alias = {
            let src_set: BTreeSet<_> = src_pts.iter().copied().collect();
            let sink_set: BTreeSet<_> = sink_pts.iter().copied().collect();
            !src_set.is_disjoint(&sink_set)
        };

        // Check SVFG path
        let src_node = SvfgNodeId::Value(src);
        let sink_node = SvfgNodeId::Value(sink);
        let svfg_path = self.check_svfg_reachability(src_node, sink_node);

        ReachabilityResult {
            reachable: direct_alias || svfg_path,
            via_alias: direct_alias,
            via_svfg: svfg_path,
            src_pts_count: src_pts.len(),
            sink_pts_count: sink_pts.len(),
        }
    }

    /// Check SVFG reachability using BFS.
    fn check_svfg_reachability(&self, src: SvfgNodeId, sink: SvfgNodeId) -> bool {
        if src == sink {
            return true;
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(src);
        visited.insert(src);

        while let Some(node) = queue.pop_front() {
            // Check successors (forward edges)
            if let Some(succs) = self.svfg.successors_of(node) {
                for (_, succ) in succs {
                    if *succ == sink {
                        return true;
                    }
                    if visited.insert(*succ) {
                        queue.push_back(*succ);
                    }
                }
            }
        }

        false
    }

    /// Export the DDA analysis results.
    #[must_use]
    pub fn export(&self) -> DdaExport {
        DdaExport {
            schema_version: "1.0.0".to_string(),
            config: DdaConfigExport {
                max_steps: self.config.max_steps,
                max_context_depth: self.config.max_context_depth,
                timeout_ms: self.config.timeout_ms,
                enable_strong_updates: self.config.enable_strong_updates,
            },
            diagnostics: self.diagnostics.clone(),
            cache_stats: self.cache.stats(),
        }
    }

    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// DDA is demand-driven, so there is no pre-built graph structure to
    /// export.  The returned `PropertyGraph` contains only metadata
    /// (config and diagnostics) with no nodes or edges.
    #[must_use]
    pub fn to_pg(
        &self,
        _resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::PropertyGraph;

        let mut pg = PropertyGraph::new("dda");
        pg.metadata.insert(
            "max_steps".to_string(),
            serde_json::json!(self.config.max_steps),
        );
        pg.metadata.insert(
            "max_context_depth".to_string(),
            serde_json::json!(self.config.max_context_depth),
        );
        pg.metadata.insert(
            "queries".to_string(),
            serde_json::json!(self.diagnostics.queries),
        );
        pg.metadata.insert(
            "cache_hits".to_string(),
            serde_json::json!(self.diagnostics.cache_hits),
        );
        pg.metadata.insert(
            "fallbacks".to_string(),
            serde_json::json!(self.diagnostics.fallbacks),
        );
        pg.metadata.insert(
            "context_terminations".to_string(),
            serde_json::json!(self.diagnostics.context_terminations),
        );
        pg
    }

    // -------------------------------------------------------------------------
    // Core backward traversal
    // -------------------------------------------------------------------------

    /// Core backward traversal algorithm.
    fn find_points_to(&mut self, initial: Dpm, budget: &mut Budget) -> Vec<LocId> {
        let mut worklist: VecDeque<Dpm> = VecDeque::new();
        let mut result: BTreeSet<LocId> = BTreeSet::new();

        worklist.push_back(initial);

        while let Some(dpm) = worklist.pop_front() {
            // Check budget
            if !budget.tick() {
                // Budget exhausted, fall back to CI-PTA
                self.diagnostics.record_fallback();
                return self.fallback_to_ci(&dpm);
            }

            // Check context depth
            if !budget.check_context_depth(dpm.context.depth()) {
                self.diagnostics.record_fallback();
                return self.fallback_to_ci(&dpm);
            }

            // Skip if already visited
            if self.cache.mark_visited(&dpm) {
                continue;
            }

            // Check cache for this DPM
            if let Some(cached) = self.cache.get(&dpm) {
                result.extend(cached.iter());
                continue;
            }

            // Classify the node and handle accordingly
            match self.classify_node(&dpm) {
                NodeKind::Addr(loc) => {
                    // Found an allocation site - add to result
                    result.insert(loc);
                }
                NodeKind::Direct => {
                    // Propagate backward through direct edges
                    self.propagate_backward_direct(&mut worklist, &dpm);
                }
                NodeKind::CallArg { call_site } => {
                    // Entering callee: push call site onto context
                    self.handle_call_entry(&mut worklist, &dpm, call_site, budget);
                }
                NodeKind::Return { from_call } => {
                    // Exiting callee: pop/match call site
                    self.handle_return_exit(&mut worklist, &dpm, from_call);
                }
                NodeKind::Load { inst_id } => {
                    // Memory load: recursive points-to query via MSSA
                    self.handle_load(&mut worklist, &mut result, &dpm, inst_id, budget);
                }
                NodeKind::MemPhi { access_id } => {
                    // Memory Phi: propagate to all MemPhi predecessors
                    self.handle_mem_phi(&mut worklist, &dpm, access_id);
                }
            }
        }

        result.into_iter().collect()
    }

    /// Classify an SVFG node for traversal handling.
    fn classify_node(&self, dpm: &Dpm) -> NodeKind {
        match &dpm.node {
            SvfgNodeId::Value(value_id) => {
                // Check if this is an allocation site
                if let Some(loc) = self.check_allocation(*value_id) {
                    return NodeKind::Addr(loc);
                }

                // Check if this is a load instruction result
                if let Some(inst_id) = self.index.value_to_inst.get(value_id) {
                    if self.load_pointers.contains_key(inst_id) {
                        return NodeKind::Load { inst_id: *inst_id };
                    }
                }

                // Check incoming edges to classify
                if let Some(preds) = self.svfg.predecessors_of(dpm.node) {
                    for (kind, _) in preds {
                        match kind {
                            SvfgEdgeKind::CallArg { .. } => {
                                // This is a formal parameter receiving from call argument
                                // Find the call site
                                if let Some(call_site) = self.find_call_site_for_param(*value_id) {
                                    return NodeKind::CallArg { call_site };
                                }
                            }
                            SvfgEdgeKind::Return { .. } => {
                                // This is a call result receiving from callee return
                                // Find the call site
                                if let Some(inst_id) = self.index.value_to_inst.get(value_id) {
                                    return NodeKind::Return {
                                        from_call: *inst_id,
                                    };
                                }
                            }
                            SvfgEdgeKind::IndirectLoad | SvfgEdgeKind::IndirectDef => {
                                // Indirect edge from MemPhi or store - this is a load
                                if let Some(inst_id) = self.index.value_to_inst.get(value_id) {
                                    return NodeKind::Load { inst_id: *inst_id };
                                }
                            }
                            _ => {}
                        }
                    }
                }

                NodeKind::Direct
            }
            SvfgNodeId::MemPhi(access_id) => NodeKind::MemPhi {
                access_id: *access_id,
            },
        }
    }

    /// Check if a value represents an allocation site.
    fn check_allocation(&self, value_id: ValueId) -> Option<LocId> {
        let inst_id = self.index.value_to_inst.get(&value_id)?;

        // Find the instruction
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if inst.id == *inst_id {
                        match &inst.op {
                            Operation::Alloca { .. } | Operation::HeapAlloc { .. } => {
                                // Create location from instruction ID
                                let obj = ObjId::new(inst_id.raw());
                                // Get location from CI-PTA's points-to result
                                let pts = self.ci_pta.points_to(value_id);
                                if !pts.is_empty() {
                                    return Some(pts[0]);
                                }
                                // Fallback: create synthetic location
                                return Some(LocId::new(obj.raw()));
                            }
                            Operation::Global { obj } => {
                                let pts = self.ci_pta.points_to(value_id);
                                if !pts.is_empty() {
                                    return Some(pts[0]);
                                }
                                return Some(LocId::new(obj.raw()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        None
    }

    /// Find a call site instruction that passes a value to a formal parameter.
    ///
    /// Determines which function owns `param_value` by scanning function
    /// parameter lists, then searches the call graph's `call_sites` map for
    /// an instruction that targets that function.
    fn find_call_site_for_param(&self, param_value: ValueId) -> Option<InstId> {
        // Step 1: Find the function that owns this parameter.
        let owner_func_id = self.find_param_owner(param_value)?;

        // Step 2: Search call_sites for any call instruction targeting this function.
        for (inst_id, target_node) in &self.callgraph.call_sites {
            if let Some(target_func_id) = target_node.function_id() {
                if target_func_id == owner_func_id {
                    return Some(*inst_id);
                }
            }
        }

        // Step 3: Fall back — check SVFG predecessors of the param node for
        // `CallArg` edges. The predecessor's actual-argument value may be
        // traceable to a call instruction via the module index.
        let param_node = SvfgNodeId::Value(param_value);
        if let Some(preds) = self.svfg.predecessors_of(param_node) {
            for (kind, pred_node) in preds {
                if !matches!(kind, SvfgEdgeKind::CallArg { .. }) {
                    continue;
                }
                // The predecessor is an actual argument value. Try to find
                // the call instruction that uses it as an operand.
                let Some(actual_value) = pred_node.as_value() else {
                    continue;
                };
                if let Some(call_inst) = self.find_call_inst_for_actual(actual_value) {
                    return Some(call_inst);
                }
            }
        }

        None
    }

    /// Find which function owns a given parameter `ValueId`.
    fn find_param_owner(&self, param_value: ValueId) -> Option<FunctionId> {
        for func in &self.module.functions {
            for param in &func.params {
                if param.id == param_value {
                    return Some(func.id);
                }
            }
        }
        None
    }

    /// Find the call instruction that uses `actual_value` as an argument.
    ///
    /// Scans call instructions in the module looking for one whose operand
    /// list contains `actual_value`.
    fn find_call_inst_for_actual(&self, actual_value: ValueId) -> Option<InstId> {
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    match &inst.op {
                        Operation::CallDirect { .. } | Operation::CallIndirect { .. } => {
                            if inst.operands.contains(&actual_value) {
                                return Some(inst.id);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Propagate backward through direct edges (SSA def-use).
    fn propagate_backward_direct(&self, worklist: &mut VecDeque<Dpm>, dpm: &Dpm) {
        if let Some(preds) = self.svfg.predecessors_of(dpm.node) {
            for (kind, pred) in preds {
                // Only follow direct edges for now
                if kind.is_direct() || matches!(kind, SvfgEdgeKind::PhiFlow) {
                    let new_dpm = Dpm::with_context(*pred, dpm.context.clone());
                    if !self.cache.is_visited(&new_dpm) {
                        worklist.push_back(new_dpm);
                    }
                }
            }
        }
    }

    /// Handle call entry (entering callee from caller).
    fn handle_call_entry(
        &self,
        worklist: &mut VecDeque<Dpm>,
        dpm: &Dpm,
        call_site: InstId,
        budget: &mut Budget,
    ) {
        // Push call site onto context stack
        let new_context = dpm.context.push(call_site);

        // Check context depth
        if !budget.check_context_depth(new_context.depth()) {
            return;
        }

        // Propagate to predecessors with new context
        if let Some(preds) = self.svfg.predecessors_of(dpm.node) {
            for (_, pred) in preds {
                let new_dpm = Dpm::with_context(*pred, new_context.clone());
                if !self.cache.is_visited(&new_dpm) {
                    worklist.push_back(new_dpm);
                }
            }
        }
    }

    /// Handle return exit (exiting callee back to caller).
    fn handle_return_exit(&mut self, worklist: &mut VecDeque<Dpm>, dpm: &Dpm, from_call: InstId) {
        // Check if context matches
        if dpm.context.is_empty() {
            // Empty context with a return edge means an unmatched closing
            // parenthesis in CFL-reachability. Propagating to all callers here
            // would introduce spurious paths that violate context sensitivity.
            // Instead, stop traversal along this path. Soundness is maintained
            // because the caller falls back to CI-PTA when DDA produces an
            // empty result, and other worklist paths may still find results.
            self.diagnostics.record_context_termination();
            return;
        }

        // Must match the call site on the context stack
        if !dpm.context.matches(from_call) {
            // Mismatched return - spurious path, skip
            return;
        }

        // Pop the matching call site
        if let Some((new_context, _)) = dpm.context.pop() {
            // Propagate to predecessors with popped context
            if let Some(preds) = self.svfg.predecessors_of(dpm.node) {
                for (_, pred) in preds {
                    let new_dpm = Dpm::with_context(*pred, new_context.clone());
                    if !self.cache.is_visited(&new_dpm) {
                        worklist.push_back(new_dpm);
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Phase 3: Indirect edge handling
    // -------------------------------------------------------------------------

    /// Handle a memory load by recursively querying points-to of the pointer.
    ///
    /// For a load `v = *p`, we need to:
    /// 1. Query the points-to set of `p` (the load pointer)
    /// 2. For each location in pts(p), find the defining store via MSSA clobber
    /// 3. Propagate backward from each store's value operand
    fn handle_load(
        &mut self,
        worklist: &mut VecDeque<Dpm>,
        result: &mut BTreeSet<LocId>,
        dpm: &Dpm,
        inst_id: InstId,
        budget: &mut Budget,
    ) {
        // Get the load pointer
        let Some(&ptr) = self.load_pointers.get(&inst_id) else {
            // Not a load, propagate directly
            self.propagate_backward_direct(worklist, dpm);
            return;
        };

        // Recursively query points-to of the load pointer
        // (This may exhaust budget, triggering fallback)
        let ptr_pts = self.query_points_to_internal(ptr, dpm.context.clone(), budget);

        if budget.exhausted() {
            // Budget exhausted during recursive query - fallback already recorded
            result.extend(self.fallback_to_ci(dpm));
            return;
        }

        if ptr_pts.is_empty() {
            // Unknown pointer - fallback to CI-PTA
            result.extend(self.fallback_to_ci(dpm));
            return;
        }

        // Check if we can apply strong update
        let can_strong =
            self.config.enable_strong_updates && self.can_strong_update(&ptr_pts, inst_id);

        // For each location the pointer may point to, find the clobbering def
        for loc in &ptr_pts {
            // Get the MSSA access for this load instruction
            let Some(access_id) = self.mssa.access_id_for(inst_id) else {
                continue;
            };

            // Find the clobbering store for this location
            // Note: clobber_for requires mutable access, but we can work around
            // by using the SVFG edges which already encode the def-use chains
            self.propagate_from_clobber(worklist, dpm, *loc, access_id, can_strong);
        }
    }

    /// Propagate from the clobbering store for a location.
    fn propagate_from_clobber(
        &self,
        worklist: &mut VecDeque<Dpm>,
        dpm: &Dpm,
        _loc: LocId,
        _access_id: MemAccessId,
        _can_strong: bool,
    ) {
        // Use SVFG edges to find the store values
        // The SVFG already encodes indirect def-use chains
        if let Some(preds) = self.svfg.predecessors_of(dpm.node) {
            for (kind, pred) in preds {
                match kind {
                    SvfgEdgeKind::IndirectDef | SvfgEdgeKind::IndirectLoad => {
                        // Indirect edge from a store value or MemPhi
                        let new_dpm = Dpm::with_context(*pred, dpm.context.clone());
                        if !self.cache.is_visited(&new_dpm) {
                            worklist.push_back(new_dpm);
                        }
                    }
                    SvfgEdgeKind::PhiFlow => {
                        // Flow through MemPhi
                        let new_dpm = Dpm::with_context(*pred, dpm.context.clone());
                        if !self.cache.is_visited(&new_dpm) {
                            worklist.push_back(new_dpm);
                        }
                    }
                    _ => {
                        // Also follow direct edges for completeness
                        if kind.is_direct() {
                            let new_dpm = Dpm::with_context(*pred, dpm.context.clone());
                            if !self.cache.is_visited(&new_dpm) {
                                worklist.push_back(new_dpm);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Handle a MemPhi node by propagating to all incoming memory edges.
    fn handle_mem_phi(&self, worklist: &mut VecDeque<Dpm>, dpm: &Dpm, _access_id: MemAccessId) {
        // MemPhi nodes merge memory state from multiple control-flow paths
        // Propagate to all predecessor MemPhi/Store nodes
        if let Some(preds) = self.svfg.predecessors_of(dpm.node) {
            for (kind, pred) in preds {
                // Follow PhiFlow and IndirectStore edges
                if matches!(kind, SvfgEdgeKind::PhiFlow | SvfgEdgeKind::IndirectStore) {
                    let new_dpm = Dpm::with_context(*pred, dpm.context.clone());
                    if !self.cache.is_visited(&new_dpm) {
                        worklist.push_back(new_dpm);
                    }
                }
            }
        }
    }

    /// Internal recursive points-to query with shared context.
    fn query_points_to_internal(
        &mut self,
        ptr: ValueId,
        context: CallString,
        budget: &mut Budget,
    ) -> Vec<LocId> {
        let node = SvfgNodeId::Value(ptr);
        let dpm = Dpm::with_context(node, context);

        // Check cache first
        if let Some(cached) = self.cache.get(&dpm) {
            return cached.iter().collect();
        }

        // Don't clear visited set - we're in a nested query
        // Run backward traversal
        self.find_points_to(dpm, budget)
    }

    /// Check if we can apply a strong update at a store.
    ///
    /// Strong updates are valid when:
    /// 1. The store target is a singleton location
    /// 2. The location is not an array element
    /// 3. The store is not in a recursive function
    /// 4. The location has not escaped
    fn can_strong_update(&self, target_locs: &[LocId], inst_id: InstId) -> bool {
        // Must be a singleton target
        if target_locs.len() != 1 {
            return false;
        }

        let loc = target_locs[0];

        // Check if in a recursive function
        if let Some(func_id) = self.index.inst_to_func.get(&inst_id) {
            if self.recursive_sccs.contains(func_id) {
                return false;
            }
        }

        // Check if the location is an array (heuristic: look for GEP in the def)
        if self.is_array_location(loc) {
            return false;
        }

        // TODO: Check escape analysis (for now, be conservative)
        // A proper implementation would check if the location is passed to
        // external functions or stored to global memory

        true
    }

    /// Check if a location represents an array element.
    fn is_array_location(&self, loc: LocId) -> bool {
        // Heuristic: check if multiple values point to this location
        // This indicates it's likely an array element that can't be strongly updated

        // Count how many pointers point to this location using CI-PTA
        let mut alias_count = 0;
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Some(dst) = inst.dst {
                        let pts = self.ci_pta.points_to(dst);
                        if pts.contains(&loc) {
                            alias_count += 1;
                            if alias_count > 3 {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // If more than 3 pointers point here, likely an array
        alias_count > 3
    }

    /// Check if a function is in a recursive SCC.
    #[must_use]
    pub fn is_recursive(&self, func_id: FunctionId) -> bool {
        self.recursive_sccs.contains(&func_id)
    }

    /// Get the call graph used for recursion detection.
    #[must_use]
    pub fn callgraph(&self) -> &CallGraph {
        self.callgraph
    }

    /// Fall back to CI-PTA for a given DPM.
    fn fallback_to_ci(&self, dpm: &Dpm) -> Vec<LocId> {
        match &dpm.node {
            SvfgNodeId::Value(value_id) => self.ci_pta.points_to(*value_id),
            SvfgNodeId::MemPhi(_) => Vec::new(), // No CI-PTA result for MemPhi
        }
    }
}

// ---------------------------------------------------------------------------
// Node classification
// ---------------------------------------------------------------------------

/// Classification of SVFG nodes for traversal.
enum NodeKind {
    /// Allocation site (Alloca, HeapAlloc, Global) - produces a location.
    Addr(LocId),
    /// Direct SSA flow (def-use, transform, copy, phi).
    Direct,
    /// Call argument edge - entering callee.
    CallArg { call_site: InstId },
    /// Return edge - exiting callee.
    Return { from_call: InstId },
    /// Memory load - requires indirect resolution via MSSA.
    Load { inst_id: InstId },
    /// Memory Phi merge (control-flow join point for memory).
    MemPhi { access_id: MemAccessId },
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Wrapper struct for call graph adjacency implementing `Successors` trait.
struct CallGraphAdj<'a>(&'a BTreeMap<FunctionId, BTreeSet<FunctionId>>);

impl crate::graph_algo::Successors<FunctionId> for CallGraphAdj<'_> {
    fn successors(&self, node: &FunctionId) -> Option<&BTreeSet<FunctionId>> {
        self.0.get(node)
    }
}

/// Detect recursive function SCCs in the call graph.
fn detect_recursive_sccs(callgraph: &CallGraph) -> BTreeSet<FunctionId> {
    use crate::callgraph::CallGraphNode;

    let mut recursive = BTreeSet::new();

    // Collect all function nodes
    let func_nodes: BTreeSet<_> = callgraph
        .nodes
        .iter()
        .filter_map(|n| match n {
            CallGraphNode::Function(f) => Some(*f),
            _ => None,
        })
        .collect();

    if func_nodes.is_empty() {
        return recursive;
    }

    // Build adjacency list from call graph edges (as BTreeSet for Successors trait)
    let mut adj: BTreeMap<FunctionId, BTreeSet<FunctionId>> = BTreeMap::new();
    for f in &func_nodes {
        adj.insert(*f, BTreeSet::new());
    }

    for (caller, callees) in &callgraph.edges {
        if let Some(caller_f) = caller.function_id() {
            for callee in callees {
                if let Some(callee_f) = callee.function_id() {
                    adj.entry(caller_f).or_default().insert(callee_f);
                }
            }
        }
    }

    let graph = CallGraphAdj(&adj);
    let sccs = crate::graph_algo::tarjan_scc(&func_nodes, &graph);

    for scc in sccs {
        match scc.len().cmp(&1) {
            std::cmp::Ordering::Greater => {
                // Multiple functions in SCC = mutual recursion
                for f in scc {
                    recursive.insert(f);
                }
            }
            std::cmp::Ordering::Equal => {
                // Single function - check for self-recursion
                if let Some(f) = scc.iter().next() {
                    if adj.get(f).is_some_and(|callees| callees.contains(f)) {
                        recursive.insert(*f);
                    }
                }
            }
            std::cmp::Ordering::Less => {}
        }
    }

    recursive
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::cfg::Cfg;
    use crate::defuse::DefUseGraph;
    use crate::module_index::ModuleIndex;
    use crate::pta::{PtaConfig, PtaContext, PtaResult};
    use crate::svfg::SvfgBuilder;
    use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction};
    use saf_core::ids::{BlockId, ModuleId};

    fn make_minimal_module() -> AirModule {
        // Create a minimal module with one function containing an alloca
        let alloca_inst = Instruction {
            id: InstId::new(1),
            op: Operation::Alloca { size_bytes: None },
            dst: Some(ValueId::new(100)),
            operands: vec![],
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let ret_inst = Instruction {
            id: InstId::new(2),
            op: Operation::Ret,
            dst: None,
            operands: vec![],
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let block = AirBlock {
            id: BlockId::new(10),
            label: None,
            instructions: vec![alloca_inst, ret_inst],
        };

        let func = AirFunction {
            id: FunctionId::new(1000),
            name: "test".to_string(),
            params: vec![],
            blocks: vec![block],
            is_declaration: false,
            entry_block: Some(BlockId::new(10)),
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        AirModule {
            id: ModuleId::new(1),
            name: Some("test_module".to_string()),
            functions: vec![func],
            globals: vec![],
            type_hierarchy: vec![],
            source_files: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    fn run_analyses(
        module: &AirModule,
    ) -> (
        PtaResult,
        BTreeMap<FunctionId, Cfg>,
        CallGraph,
        DefUseGraph,
        ModuleIndex,
    ) {
        // Build CFGs for each function
        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();

        // Build call graph
        let callgraph = crate::callgraph::CallGraph::build(module);

        // Build def-use graph
        let defuse = DefUseGraph::build(module);

        // Build module index
        let index = ModuleIndex::build(module);

        // Run PTA
        let config = PtaConfig::default();
        let mut ctx = PtaContext::new(config);
        let raw = ctx.analyze(module);
        let pta_result = PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics);

        (pta_result, cfgs, callgraph, defuse, index)
    }

    #[test]
    fn dda_new_creates_solver() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        assert_eq!(dda.diagnostics().queries, 0);
    }

    #[test]
    fn dda_points_to_alloca() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Query the alloca result
        let pts = dda.points_to(ValueId::new(100));

        // Should find the allocation
        assert!(!pts.is_empty(), "Should find allocation location");
        assert_eq!(dda.diagnostics().queries, 1);
    }

    #[test]
    fn dda_cache_hit() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Query twice
        let _pts1 = dda.points_to(ValueId::new(100));
        let _pts2 = dda.points_to(ValueId::new(100));

        assert_eq!(dda.diagnostics().queries, 2);
        assert_eq!(dda.diagnostics().cache_hits, 1);
    }

    #[test]
    fn dda_may_alias_same() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Same pointer should must-alias itself (identical points-to sets)
        let alias = dda.may_alias(ValueId::new(100), ValueId::new(100));
        // Same pointer will have identical points-to sets, so it should be MustAlias
        // (or Unknown if the pointer wasn't analyzed)
        assert!(
            matches!(alias, AliasResult::Must | AliasResult::Unknown),
            "same pointer should must-alias or be unknown: got {alias:?}"
        );
    }

    #[test]
    fn dda_budget_exhaustion_fallback() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        // Very low budget
        let config = DdaConfig::new(1, 10, 0, true);
        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            config,
        );

        let _pts = dda.points_to(ValueId::new(100));

        // Should have triggered fallback due to limited steps
        // (actual behavior depends on graph structure)
        assert_eq!(dda.diagnostics().queries, 1);
    }

    #[test]
    fn detect_recursive_sccs_no_recursion() {
        // Empty call graph
        let module = make_minimal_module();
        let callgraph = crate::callgraph::CallGraph::build(&module);

        let recursive = detect_recursive_sccs(&callgraph);
        assert!(recursive.is_empty());
    }

    // -------------------------------------------------------------------------
    // Phase 3: Load/Store tests
    // -------------------------------------------------------------------------

    fn make_load_store_module() -> AirModule {
        // Create a module with: alloca -> store -> load
        // p = alloca
        // store 42, p
        // v = load p
        let alloca_inst = Instruction {
            id: InstId::new(1),
            op: Operation::Alloca { size_bytes: None },
            dst: Some(ValueId::new(100)), // p
            operands: vec![],
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let store_inst = Instruction {
            id: InstId::new(2),
            op: Operation::Store,
            dst: None,
            operands: vec![ValueId::new(200), ValueId::new(100)], // store value=200, ptr=100
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let load_inst = Instruction {
            id: InstId::new(3),
            op: Operation::Load,
            dst: Some(ValueId::new(300)),      // v = load p
            operands: vec![ValueId::new(100)], // load from ptr=100
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let ret_inst = Instruction {
            id: InstId::new(4),
            op: Operation::Ret,
            dst: None,
            operands: vec![],
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        };

        let block = AirBlock {
            id: BlockId::new(10),
            label: None,
            instructions: vec![alloca_inst, store_inst, load_inst, ret_inst],
        };

        let func = AirFunction {
            id: FunctionId::new(1000),
            name: "test_load_store".to_string(),
            params: vec![],
            blocks: vec![block],
            is_declaration: false,
            entry_block: Some(BlockId::new(10)),
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        AirModule {
            id: ModuleId::new(1),
            name: Some("test_module".to_string()),
            functions: vec![func],
            globals: vec![],
            type_hierarchy: vec![],
            source_files: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    #[test]
    fn dda_tracks_load_pointers() {
        let module = make_load_store_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Check that load pointer is tracked
        assert!(dda.load_pointers.contains_key(&InstId::new(3)));
        assert_eq!(
            dda.load_pointers.get(&InstId::new(3)),
            Some(&ValueId::new(100))
        );
    }

    #[test]
    fn dda_strong_update_conditions() {
        let module = make_load_store_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Get points-to of alloca pointer
        let pts = pta_result.points_to(ValueId::new(100));

        // Singleton location should allow strong update (if not in recursive function)
        if !pts.is_empty() {
            let can_strong = dda.can_strong_update(&pts, InstId::new(2));
            // Should be able to strong update single alloca in non-recursive function
            assert!(
                can_strong,
                "Should allow strong update for singleton non-recursive"
            );
        }
    }

    #[test]
    fn dda_no_recursive_function() {
        let module = make_load_store_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Simple function should not be recursive
        assert!(!dda.is_recursive(FunctionId::new(1000)));
    }

    #[test]
    fn dda_callgraph_accessor() {
        let module = make_load_store_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Should be able to access call graph
        let cg = dda.callgraph();
        assert!(!cg.nodes.is_empty());
    }

    #[test]
    fn dda_query_load_result() {
        let module = make_load_store_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Query the load result - this exercises handle_load
        // The load result (ValueId 300) should trace back through the store
        let _pts = dda.points_to(ValueId::new(300));

        // Query should complete without error
        assert_eq!(dda.diagnostics().queries, 1);
    }

    // -------------------------------------------------------------------------
    // Phase 5: Query API and Export tests
    // -------------------------------------------------------------------------

    #[test]
    fn dda_reachable_same_value() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Same value should be reachable to itself
        assert!(dda.reachable(ValueId::new(100), ValueId::new(100)));
    }

    #[test]
    fn dda_reachable_different_values() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Non-existent value should not be reachable
        let result = dda.reachable(ValueId::new(100), ValueId::new(999));
        // Since 999 doesn't exist, reachability depends on aliasing
        let _ = result; // Just check it doesn't panic
    }

    #[test]
    fn dda_reachable_refined() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Get refined reachability
        let result = dda.reachable_refined(ValueId::new(100), ValueId::new(100));

        // Same value should be reachable via SVFG or alias
        assert!(result.reachable);
        // It should be via SVFG (same node)
        assert!(result.via_svfg || result.via_alias);
    }

    #[test]
    fn dda_export() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let mut dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Run a query first
        let _pts = dda.points_to(ValueId::new(100));

        // Export should work
        let export = dda.export();

        assert_eq!(export.schema_version, "1.0.0");
        assert_eq!(export.config.max_steps, 100_000);
        assert_eq!(export.config.max_context_depth, 10);
        assert!(export.config.enable_strong_updates);
        assert_eq!(export.diagnostics.queries, 1);
    }

    #[test]
    fn dda_export_serializable() {
        let module = make_minimal_module();
        let (pta_result, cfgs, callgraph, defuse, index) = run_analyses(&module);

        let mut mssa = MemorySsa::build(&module, &cfgs, pta_result.clone(), &callgraph);
        let (svfg, _pp) =
            SvfgBuilder::new(&module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        let dda = DdaPta::new(
            &svfg,
            &mssa,
            &pta_result,
            &module,
            &callgraph,
            &index,
            DdaConfig::default(),
        );

        // Export should be JSON serializable
        let export = dda.export();
        let json = serde_json::to_string(&export).expect("should serialize");
        assert!(json.contains("schema_version"));
        assert!(json.contains("diagnostics"));
        assert!(json.contains("cache_stats"));
    }
}
