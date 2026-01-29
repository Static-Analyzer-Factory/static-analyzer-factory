//! Thread discovery phase of MTA analysis.
//!
//! This module identifies all thread creation sites (`pthread_create`) and assigns
//! unique thread IDs based on calling context.

use saf_core::air::{AirModule, Instruction, Operation};
use saf_core::ids::{FunctionId, InstId, ObjId};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::callgraph::CallGraph;
use crate::pta::{FunctionLocationMap, LocationFactory, PointsToMap};

use super::MtaConfig;
use super::types::{CallsiteLabel, ThreadConcurrencyGraph, ThreadId};

/// Thread discovery analysis.
pub struct ThreadDiscovery<'a> {
    module: &'a AirModule,
    #[allow(dead_code)] // Reserved for future use
    callgraph: &'a CallGraph,
    config: &'a MtaConfig,

    /// Map from function ID to function name.
    func_names: BTreeMap<FunctionId, String>,

    /// Optional PTA results for resolving function pointers in pthread_create.
    pts: Option<&'a PointsToMap>,
    /// Location factory for looking up location objects.
    factory: Option<&'a LocationFactory>,
    /// Map from ObjId to FunctionId for function pointer resolution.
    func_loc_map: Option<FunctionLocationMap>,
}

impl<'a> ThreadDiscovery<'a> {
    /// Create a new thread discovery context without PTA.
    pub fn new(module: &'a AirModule, callgraph: &'a CallGraph, config: &'a MtaConfig) -> Self {
        Self::with_pta(module, callgraph, config, None, None)
    }

    /// Create a new thread discovery context with PTA results for function pointer resolution.
    ///
    /// When PTA results are provided, thread functions passed to `pthread_create` via
    /// function pointers can be resolved.
    pub fn with_pta(
        module: &'a AirModule,
        callgraph: &'a CallGraph,
        config: &'a MtaConfig,
        pts: Option<&'a PointsToMap>,
        factory: Option<&'a LocationFactory>,
    ) -> Self {
        // Build function name lookup
        let func_names: BTreeMap<FunctionId, String> = module
            .functions
            .iter()
            .map(|f| (f.id, f.name.clone()))
            .collect();

        // Build function location map for PTA-based resolution
        let func_loc_map = if pts.is_some() && factory.is_some() {
            Some(FunctionLocationMap::build(module))
        } else {
            None
        };

        Self {
            module,
            callgraph,
            config,
            func_names,
            pts,
            factory,
            func_loc_map,
        }
    }

    /// Run thread discovery and return the thread concurrency graph.
    pub fn discover(&self) -> ThreadConcurrencyGraph {
        let mut graph = ThreadConcurrencyGraph::new();

        // Find main function and add as thread 0
        if let Some(main_fn) = self.find_main_function() {
            graph.add_main_thread(main_fn, "main".to_string());
        } else {
            // No main function found - return empty graph
            return graph;
        }

        // Build callsite label map from source annotations
        let callsite_labels = self.extract_callsite_labels();

        // Discover thread creation sites using BFS from main
        self.discover_threads_bfs(&mut graph, &callsite_labels);

        // Process join constraints
        self.discover_joins(&mut graph);

        graph
    }

    /// Find the main function in the module.
    fn find_main_function(&self) -> Option<FunctionId> {
        for func in &self.module.functions {
            if func.name == "main" {
                return Some(func.id);
            }
        }
        None
    }

    /// Get function name by ID.
    fn get_func_name(&self, id: FunctionId) -> Option<&str> {
        self.func_names.get(&id).map(String::as_str)
    }

    /// Check if a function name matches any thread create function.
    fn is_thread_create_name(&self, name: &str) -> bool {
        self.config.thread_create_funcs.iter().any(|n| n == name)
    }

    /// Check if a function name matches any thread join function.
    fn is_thread_join_name(&self, name: &str) -> bool {
        self.config.thread_join_funcs.iter().any(|n| n == name)
    }

    /// Extract callsite labels from source code comments/labels.
    ///
    /// Labels like "cs1:" before pthread_create calls are extracted.
    fn extract_callsite_labels(&self) -> BTreeMap<InstId, String> {
        let mut labels = BTreeMap::new();

        // In practice, we would extract labels from debug info or source annotations.
        // For PTABen tests, labels are in the source like "cs1: pthread_create(...)".
        //
        // For now, we assign synthetic labels based on call site order within each function.
        let mut func_counters: BTreeMap<FunctionId, u32> = BTreeMap::new();

        for func in &self.module.functions {
            let counter = func_counters.entry(func.id).or_insert(0);

            for block in &func.blocks {
                for inst in &block.instructions {
                    if self.is_thread_create_call(inst) || self.is_thread_join_call(inst) {
                        *counter += 1;
                        labels.insert(inst.id, format!("cs{counter}"));
                    }
                }
            }
        }

        labels
    }

    /// Check if an instruction is a thread creation call.
    fn is_thread_create_call(&self, inst: &Instruction) -> bool {
        match &inst.op {
            Operation::CallDirect { callee } => {
                if let Some(name) = self.get_func_name(*callee) {
                    self.is_thread_create_name(name)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if an instruction is a thread join call.
    fn is_thread_join_call(&self, inst: &Instruction) -> bool {
        match &inst.op {
            Operation::CallDirect { callee } => {
                if let Some(name) = self.get_func_name(*callee) {
                    self.is_thread_join_name(name)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Discover threads using BFS traversal from main.
    ///
    /// The visited set uses `(ThreadId, FunctionId, Option<InstId>)` tuples
    /// to enable context-sensitive exploration: the same function can be
    /// re-explored when reached from a different call site within the same
    /// thread. This is critical for patterns where a wrapper function
    /// containing `pthread_create` is called from multiple sites.
    ///
    /// Thread creation via `get_thread_functions` resolves ALL PTA targets
    /// for indirect forks, creating one thread per resolved function.
    fn discover_threads_bfs(
        &self,
        graph: &mut ThreadConcurrencyGraph,
        callsite_labels: &BTreeMap<InstId, String>,
    ) {
        // Worklist: (current_thread, call_chain, functions_to_visit_with_call_site)
        // Each function is paired with the call site that led to it (None for the
        // initial main entry point).
        #[allow(clippy::type_complexity)]
        let mut worklist: VecDeque<(
            ThreadId,
            Vec<CallsiteLabel>,
            BTreeSet<(FunctionId, Option<InstId>)>,
        )> = VecDeque::new();

        // Start from main
        if let Some(main_fn) = self.find_main_function() {
            let mut initial_funcs = BTreeSet::new();
            initial_funcs.insert((main_fn, None));
            worklist.push_back((ThreadId::MAIN, Vec::new(), initial_funcs));
        }

        // Track visited (thread, function, call_site) tuples to avoid infinite
        // loops while still allowing re-exploration of the same function from
        // different call sites (needed for context-sensitive thread indexing).
        let mut visited: BTreeSet<(ThreadId, FunctionId, Option<InstId>)> = BTreeSet::new();

        while let Some((current_thread, call_chain, functions)) = worklist.pop_front() {
            for (func_id, call_site) in &functions {
                if visited.contains(&(current_thread, *func_id, *call_site)) {
                    continue;
                }
                visited.insert((current_thread, *func_id, *call_site));

                let Some(func) = self.module.functions.iter().find(|f| f.id == *func_id) else {
                    continue;
                };

                for block in &func.blocks {
                    for inst in &block.instructions {
                        // Check for thread creation — resolve ALL PTA targets
                        if self.is_thread_create_call(inst) {
                            let thread_fns = self.get_thread_functions(inst);
                            for thread_fn in thread_fns {
                                // Build the call chain for this new thread
                                let label = callsite_labels
                                    .get(&inst.id)
                                    .cloned()
                                    .unwrap_or_else(|| format!("cs{:?}", inst.id));

                                let thread_fn_name = self
                                    .get_func_name(thread_fn)
                                    .map_or_else(|| "unknown".to_string(), ToString::to_string);

                                let mut new_chain = call_chain.clone();
                                new_chain.push(CallsiteLabel::new(
                                    label,
                                    thread_fn_name.clone(),
                                    inst.id,
                                ));

                                // Add the new thread
                                let new_thread = graph.add_thread(
                                    new_chain.clone(),
                                    inst.id,
                                    thread_fn,
                                    thread_fn_name,
                                    current_thread,
                                );

                                // Queue the thread function for exploration
                                let mut new_funcs = BTreeSet::new();
                                new_funcs.insert((thread_fn, Some(inst.id)));
                                worklist.push_back((new_thread, new_chain, new_funcs));
                            }
                        }

                        // Track regular calls for call chain building
                        if let Operation::CallDirect { callee } = &inst.op {
                            // Skip threading functions themselves
                            let callee_name = self.get_func_name(*callee);
                            let is_thread_fn = callee_name.is_some_and(|n| {
                                self.is_thread_create_name(n) || self.is_thread_join_name(n)
                            });

                            if !is_thread_fn {
                                let label = callsite_labels
                                    .get(&inst.id)
                                    .cloned()
                                    .unwrap_or_else(|| format!("call{:?}", inst.id));

                                let callee_name_str = callee_name
                                    .map_or_else(|| "unknown".to_string(), ToString::to_string);

                                let mut extended_chain = call_chain.clone();
                                extended_chain.push(CallsiteLabel::new(
                                    label,
                                    callee_name_str,
                                    inst.id,
                                ));

                                // Limit context depth
                                if extended_chain.len() <= self.config.max_context_depth {
                                    let mut callee_funcs = BTreeSet::new();
                                    callee_funcs.insert((*callee, Some(inst.id)));
                                    worklist.push_back((
                                        current_thread,
                                        extended_chain,
                                        callee_funcs,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get ALL thread functions from a `pthread_create` call.
    ///
    /// `pthread_create` signature: `pthread_create(thread, attr, start_routine, arg)`
    /// The third argument (index 2) is the thread function.
    ///
    /// When the thread function is passed via a function pointer (indirect fork),
    /// PTA may resolve it to multiple targets. This method returns ALL resolved
    /// targets so that one thread is created per target.
    ///
    /// Resolution strategies (in order):
    /// 1. Check if operand `ValueId` directly corresponds to a function's `ObjId`
    /// 2. Use PTA to resolve function pointers — collect ALL targets (if PTA results available)
    /// 3. Follow `GlobalRef` constants to find the target function
    fn get_thread_functions(&self, inst: &Instruction) -> Vec<FunctionId> {
        let mut results = Vec::new();

        // pthread_create has 4 arguments: thread, attr, start_routine, arg
        // The thread function is at index 2 (0-indexed)
        let Some(thread_fn_operand) = inst.operands.get(2) else {
            return results;
        };

        // Strategy 1: Check if operand directly corresponds to a function's ObjId
        // This handles cases where the frontend encodes function pointers as ObjIds
        // (e.g., when passing a function directly: pthread_create(..., foo, ...))
        let obj_id = ObjId::new(thread_fn_operand.raw());
        for func in &self.module.functions {
            if ObjId::new(func.id.raw()) == obj_id {
                results.push(func.id);
                // Direct match is definitive — no need to check other strategies
                return results;
            }
        }

        // Strategy 2: Use PTA to resolve function pointers — collect ALL targets
        // For indirect forks (function pointer passed to pthread_create), PTA may
        // resolve the pointer to multiple function targets.
        if let (Some(pts), Some(factory), Some(func_loc_map)) =
            (&self.pts, &self.factory, &self.func_loc_map)
        {
            if let Some(loc_set) = pts.get(thread_fn_operand) {
                for &loc_id in loc_set {
                    if let Some(loc) = factory.all_locations().get(&loc_id) {
                        if let Some(func_id) = func_loc_map.get(loc.obj) {
                            if !results.contains(&func_id) {
                                results.push(func_id);
                            }
                        }
                    }
                }
            }
        }

        if !results.is_empty() {
            return results;
        }

        // Strategy 3: Follow GlobalRef constant to find target function
        if let Some(saf_core::air::Constant::GlobalRef(target_value_id)) =
            self.module.constants.get(thread_fn_operand)
        {
            // The target might be a function address
            let target_obj_id = ObjId::new(target_value_id.raw());
            for func in &self.module.functions {
                if ObjId::new(func.id.raw()) == target_obj_id {
                    results.push(func.id);
                    break;
                }
            }
        }

        results
    }

    /// Discover join constraints.
    fn discover_joins(&self, _graph: &mut ThreadConcurrencyGraph) {
        for func in &self.module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if self.is_thread_join_call(inst) {
                        // pthread_join(thread, retval) - first arg is the thread handle
                        // We need to resolve which thread is being joined
                        // This requires tracking the thread handle from pthread_create

                        // For now, mark this as a join point
                        // More sophisticated analysis would track the thread handle value
                        if let Operation::CallDirect { .. } = &inst.op {
                            // Record that a join happens here
                            // Full implementation would resolve which thread
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Integration tests in crates/saf-analysis/tests/mta_e2e.rs
}
