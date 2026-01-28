//! Call graph construction.
//!
//! See FR-CALL-001, FR-CALL-002 for requirements.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, InstId, ObjId, ValueId};
use saf_core::spec::SpecRegistry;

use crate::PtaResult;
use crate::display::DisplayResolver;
use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node, span_to_property};
use crate::graph_algo::Successors;

/// A node in the call graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallGraphNode {
    /// A defined function in the module.
    Function(FunctionId),
    /// An external/declared function (no body).
    External {
        /// Function name.
        name: String,
        /// Function ID.
        func: FunctionId,
    },
    /// Placeholder for indirect call target (resolved by PTA later).
    IndirectPlaceholder {
        /// The call site instruction.
        site: InstId,
    },
}

impl CallGraphNode {
    /// Get the function ID if this is a Function or External node.
    #[must_use]
    pub fn function_id(&self) -> Option<FunctionId> {
        match self {
            Self::Function(id) | Self::External { func: id, .. } => Some(*id),
            Self::IndirectPlaceholder { .. } => None,
        }
    }

    /// Check if this is an indirect placeholder.
    #[must_use]
    pub fn is_indirect(&self) -> bool {
        matches!(self, Self::IndirectPlaceholder { .. })
    }
}

/// Whole-program call graph.
///
/// Tracks call relationships between functions, including indirect calls
/// that need PTA resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallGraph {
    /// All nodes in the call graph.
    pub nodes: BTreeSet<CallGraphNode>,
    /// Edges: caller -> set of callees.
    pub edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>>,
    /// Reverse edges: callee -> set of callers.
    ///
    /// Maintained in sync with `edges` so that `callers_of` is O(log n)
    /// instead of a full scan of all edges.
    pub(crate) reverse_edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>>,
    /// Call site to target node mapping.
    pub call_sites: BTreeMap<InstId, CallGraphNode>,
    /// Index from `FunctionId` to `CallGraphNode` for O(log n) lookup.
    /// Populated during `build()`, covers both `Function` and `External` nodes.
    pub(crate) func_index: BTreeMap<FunctionId, CallGraphNode>,
}

impl CallGraph {
    /// Build a call graph from an AIR module.
    ///
    /// Creates nodes for all functions and edges for all call instructions.
    /// Indirect calls create placeholder nodes that can be resolved later by PTA.
    ///
    /// # Panics
    ///
    /// Panics if internal invariants are violated (function node not found after insertion).
    /// This should never happen with valid input.
    #[must_use]
    pub fn build(module: &AirModule) -> Self {
        let mut nodes = BTreeSet::new();
        let mut edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>> = BTreeMap::new();
        let mut call_sites = BTreeMap::new();

        // Create nodes for all functions
        let mut func_nodes: BTreeMap<FunctionId, CallGraphNode> = BTreeMap::new();
        for func in &module.functions {
            let node = if func.is_declaration {
                CallGraphNode::External {
                    name: func.name.clone(),
                    func: func.id,
                }
            } else {
                CallGraphNode::Function(func.id)
            };
            nodes.insert(node.clone());
            edges.insert(node.clone(), BTreeSet::new());
            func_nodes.insert(func.id, node);
        }

        // Scan all call instructions to build edges
        for func in &module.functions {
            if func.is_declaration {
                continue; // Declarations have no body
            }

            let caller_node = func_nodes
                .get(&func.id)
                .expect("function node was inserted in initialization loop")
                .clone();

            for block in &func.blocks {
                for inst in &block.instructions {
                    match &inst.op {
                        Operation::CallDirect { callee } => {
                            // Direct call: edge to the callee function
                            if let Some(callee_node) = func_nodes.get(callee) {
                                edges
                                    .get_mut(&caller_node)
                                    .expect("caller node was inserted in initialization loop")
                                    .insert(callee_node.clone());
                                call_sites.insert(inst.id, callee_node.clone());
                            } else {
                                // Callee not in module - create external node
                                let external_node = CallGraphNode::External {
                                    name: format!("unknown_{}", callee.to_hex()),
                                    func: *callee,
                                };
                                if !nodes.contains(&external_node) {
                                    nodes.insert(external_node.clone());
                                    edges.insert(external_node.clone(), BTreeSet::new());
                                    // Also index by FunctionId so later
                                    // `func_index` lookups succeed.
                                    func_nodes.insert(*callee, external_node.clone());
                                }
                                edges
                                    .get_mut(&caller_node)
                                    .expect("caller node was inserted in initialization loop")
                                    .insert(external_node.clone());
                                call_sites.insert(inst.id, external_node);
                            }
                        }
                        Operation::CallIndirect { .. } => {
                            // Indirect call: create placeholder node
                            let placeholder = CallGraphNode::IndirectPlaceholder { site: inst.id };
                            nodes.insert(placeholder.clone());
                            edges.insert(placeholder.clone(), BTreeSet::new());
                            edges
                                .get_mut(&caller_node)
                                .expect("caller node was inserted in initialization loop")
                                .insert(placeholder.clone());
                            call_sites.insert(inst.id, placeholder);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Build reverse edges from the forward edges.
        let mut reverse_edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>> = BTreeMap::new();
        for (caller, callees) in &edges {
            for callee in callees {
                reverse_edges
                    .entry(callee.clone())
                    .or_default()
                    .insert(caller.clone());
            }
        }

        Self {
            nodes,
            edges,
            reverse_edges,
            call_sites,
            func_index: func_nodes,
        }
    }

    /// Get all callers of a node.
    ///
    /// Uses the `reverse_edges` index for O(log n) lookup instead of
    /// scanning all edges.
    #[must_use]
    pub fn callers_of(&self, node: &CallGraphNode) -> BTreeSet<CallGraphNode> {
        self.reverse_edges.get(node).cloned().unwrap_or_default()
    }

    /// Get all callees of a node.
    #[must_use]
    pub fn callees_of(&self, node: &CallGraphNode) -> Option<&BTreeSet<CallGraphNode>> {
        self.edges.get(node)
    }

    /// Get the target node for a call site.
    #[must_use]
    pub fn call_site_target(&self, site: InstId) -> Option<&CallGraphNode> {
        self.call_sites.get(&site)
    }

    /// Look up a node by `FunctionId` in O(log n).
    #[must_use]
    pub fn node_for_function(&self, func: FunctionId) -> Option<&CallGraphNode> {
        self.func_index.get(&func)
    }

    /// Get all indirect call placeholders.
    #[must_use]
    pub fn indirect_calls(&self) -> Vec<&CallGraphNode> {
        self.nodes.iter().filter(|n| n.is_indirect()).collect()
    }

    /// Resolve an indirect call placeholder to specific targets.
    ///
    /// This is called by PTA to update indirect call edges.
    ///
    /// # Panics
    ///
    /// Panics if the caller node for the indirect call site is not in the edge map.
    /// This indicates the call graph is in an inconsistent state.
    pub fn resolve_indirect(&mut self, site: InstId, targets: &[FunctionId]) {
        let placeholder = CallGraphNode::IndirectPlaceholder { site };

        // Find callers of the placeholder
        let callers: Vec<_> = self.callers_of(&placeholder).into_iter().collect();

        // Add edges from callers to resolved targets
        for caller in callers {
            for target_id in targets {
                if let Some(target) = self.func_index.get(target_id).cloned() {
                    if self
                        .edges
                        .get_mut(&caller)
                        .expect("caller node should exist in edges map")
                        .insert(target.clone())
                    {
                        // Maintain reverse_edges
                        self.reverse_edges
                            .entry(target)
                            .or_default()
                            .insert(caller.clone());
                    }
                }
            }
        }

        // Update `call_sites` so downstream consumers (SVFG, value-flow) see
        // the resolved `Function`/`External` node(s) instead of the stale
        // `IndirectPlaceholder`.
        // For single-target resolution, store the target directly; for
        // multi-target, store the first target (consumers that need all
        // targets should also consult `edges`).
        for target_id in targets {
            if let Some(target) = self.func_index.get(target_id).cloned() {
                self.call_sites.insert(site, target);
                break;
            }
        }
    }

    /// Remove specific resolved targets for an indirect call site.
    ///
    /// Used by CG refinement to narrow CHA-resolved targets when PTA
    /// provides more precise resolution.
    pub fn remove_indirect_targets(&mut self, site: InstId, targets: &[FunctionId]) {
        let placeholder = CallGraphNode::IndirectPlaceholder { site };

        // Find callers of the placeholder
        let callers: Vec<_> = self.callers_of(&placeholder).into_iter().collect();

        // Remove edges from callers to the specified targets
        for caller in callers {
            if let Some(callees) = self.edges.get_mut(&caller) {
                for target_id in targets {
                    if let Some(target) = self.func_index.get(target_id) {
                        if callees.remove(target) {
                            // Maintain reverse_edges
                            if let Some(rev) = self.reverse_edges.get_mut(target) {
                                rev.remove(&caller);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Add callback edges based on function specs and PTA.
    ///
    /// Scans CallDirect instructions, checks if the callee has callback parameters
    /// in its spec, and uses PTA to resolve the callback targets.
    ///
    /// For example, `pthread_create(tid, attr, start_routine, arg)` has `start_routine`
    /// (param 2) marked as `callback: true` in the spec. This method uses PTA to find
    /// what function pointers could flow to that parameter and adds edges accordingly.
    ///
    /// # Arguments
    ///
    /// * `module` - The AIR module
    /// * `pta` - Points-to analysis result for resolving function pointers
    /// * `specs` - Spec registry for callback parameter information
    ///
    /// # Returns
    ///
    /// The number of callback edges added.
    pub fn add_callback_edges(
        &mut self,
        module: &AirModule,
        pta: &PtaResult,
        specs: &SpecRegistry,
    ) -> usize {
        let mut edges_added = 0;

        // Build function name map
        let func_names: BTreeMap<FunctionId, &str> = module
            .functions
            .iter()
            .map(|f| (f.id, f.name.as_str()))
            .collect();

        // Build function ID map for resolving pointers to functions
        let func_ids: BTreeMap<&str, FunctionId> = module
            .functions
            .iter()
            .map(|f| (f.name.as_str(), f.id))
            .collect();

        // Scan all call instructions
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            let caller_node = CallGraphNode::Function(func.id);

            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::CallDirect { callee } = &inst.op {
                        // Get callee name and spec
                        let Some(&callee_name) = func_names.get(callee) else {
                            continue;
                        };

                        let Some(spec) = specs.lookup(callee_name) else {
                            continue;
                        };

                        // Find callback parameters
                        let callback_indices: Vec<u32> = spec
                            .params
                            .iter()
                            .filter(|p| p.callback == Some(true))
                            .map(|p| p.index)
                            .collect();

                        if callback_indices.is_empty() {
                            continue;
                        }

                        // For each callback parameter, resolve via PTA
                        for idx in callback_indices {
                            if let Some(&arg_value) = inst.operands.get(idx as usize) {
                                // Use PTA to find what the callback pointer may point to
                                let callback_targets =
                                    resolve_callback_targets(arg_value, pta, module, &func_ids);

                                // Add edges from caller to callback targets
                                for target_id in callback_targets {
                                    if let Some(target_node) = self.func_index.get(&target_id) {
                                        let callees =
                                            self.edges.entry(caller_node.clone()).or_default();
                                        if callees.insert(target_node.clone()) {
                                            // Maintain reverse_edges
                                            self.reverse_edges
                                                .entry(target_node.clone())
                                                .or_default()
                                                .insert(caller_node.clone());
                                            edges_added += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        edges_added
    }
}

/// Resolve a callback argument to possible function targets.
///
/// This traces the callback argument through the AIR to find function references.
/// In LLVM-derived AIR, function pointers are typically:
/// - Global references to function symbols
/// - Copy chains from such references
///
/// For more complex cases (function pointers loaded from memory), full PTA
/// integration would be needed.
// NOTE: `pta` is passed through for future PTA-based resolution of function
// pointers loaded from memory. Currently only copy-chain and global-symbol
// resolution is implemented.
#[allow(clippy::only_used_in_recursion)]
fn resolve_callback_targets(
    callback_arg: ValueId,
    pta: &PtaResult,
    module: &AirModule,
    func_ids: &BTreeMap<&str, FunctionId>,
) -> Vec<FunctionId> {
    let mut targets = Vec::new();

    // Build a map from ObjId to global name for global lookups
    let obj_to_name: BTreeMap<ObjId, &str> = module
        .globals
        .iter()
        .map(|g| (g.obj, g.name.as_str()))
        .collect();

    // Scan all functions to find where callback_arg comes from
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                // Check if this instruction defines the callback argument
                if let Some(dst) = inst.dst {
                    if dst == callback_arg {
                        // Check the operation type
                        match &inst.op {
                            // Direct function reference via Global (function symbol)
                            Operation::Global { obj } => {
                                // Look up the global's name
                                if let Some(&global_name) = obj_to_name.get(obj) {
                                    // Check if this global name matches a function
                                    if let Some(&func_id) = func_ids.get(global_name) {
                                        if !targets.contains(&func_id) {
                                            targets.push(func_id);
                                        }
                                    }
                                }
                            }
                            // Follow copy chains
                            Operation::Copy => {
                                if let Some(&src) = inst.operands.first() {
                                    let inner_targets =
                                        resolve_callback_targets(src, pta, module, func_ids);
                                    for t in inner_targets {
                                        if !targets.contains(&t) {
                                            targets.push(t);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    targets
}

impl Successors<CallGraphNode> for CallGraph {
    fn successors(&self, node: &CallGraphNode) -> Option<&BTreeSet<CallGraphNode>> {
        self.edges.get(node)
    }
}

/// Compute the set of `FunctionId`s reachable from `main` via the call graph.
///
/// Returns `Some(set)` if a defined `main` function exists, `None` otherwise.
/// The returned set always includes `main` itself.
pub fn reachable_from_main(cg: &CallGraph, module: &AirModule) -> Option<BTreeSet<FunctionId>> {
    let main_func = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)?;

    let main_id = main_func.id;

    let Some(main_node) = cg.node_for_function(main_id) else {
        let mut set = BTreeSet::new();
        set.insert(main_id);
        return Some(set);
    };

    let reachable_nodes = crate::graph_algo::reachable(main_node, cg);

    let mut result: BTreeSet<FunctionId> = reachable_nodes
        .iter()
        .filter_map(CallGraphNode::function_id)
        .collect();
    result.insert(main_id);
    Some(result)
}

// =============================================================================
// Export types
// =============================================================================

/// Exportable call graph representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphExport {
    /// All nodes.
    pub nodes: Vec<CallGraphNodeExport>,
    /// All edges.
    pub edges: Vec<CallGraphEdgeExport>,
}

/// Exportable node representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphNodeExport {
    /// Node ID (function ID or placeholder ID).
    pub id: String,
    /// Node kind: "function", "external", or "indirect".
    pub kind: String,
    /// Function name (for function/external nodes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Exportable edge representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphEdgeExport {
    /// Source node ID.
    pub src: String,
    /// Destination node ID.
    pub dst: String,
}

impl CallGraph {
    /// Export call graph to serializable format.
    #[must_use]
    pub fn export(&self, module: &AirModule) -> CallGraphExport {
        let func_names: BTreeMap<FunctionId, String> = module
            .functions
            .iter()
            .map(|f| (f.id, f.name.clone()))
            .collect();

        let nodes: Vec<_> = self
            .nodes
            .iter()
            .map(|node| {
                let (id, kind, name) = match node {
                    CallGraphNode::Function(fid) => (
                        fid.to_hex(),
                        "function".to_string(),
                        func_names.get(fid).cloned(),
                    ),
                    CallGraphNode::External { name, func } => {
                        (func.to_hex(), "external".to_string(), Some(name.clone()))
                    }
                    CallGraphNode::IndirectPlaceholder { site } => {
                        (site.to_hex(), "indirect".to_string(), None)
                    }
                };
                CallGraphNodeExport { id, kind, name }
            })
            .collect();

        let mut edges = Vec::new();
        for (src, dsts) in &self.edges {
            let src_id = match src {
                CallGraphNode::Function(fid) | CallGraphNode::External { func: fid, .. } => {
                    fid.to_hex()
                }
                CallGraphNode::IndirectPlaceholder { site } => site.to_hex(),
            };
            for dst in dsts {
                let dst_id = match dst {
                    CallGraphNode::Function(fid) | CallGraphNode::External { func: fid, .. } => {
                        fid.to_hex()
                    }
                    CallGraphNode::IndirectPlaceholder { site } => site.to_hex(),
                };
                edges.push(CallGraphEdgeExport {
                    src: src_id.clone(),
                    dst: dst_id,
                });
            }
        }

        CallGraphExport { nodes, edges }
    }

    /// Export call graph as a unified `PropertyGraph`.
    ///
    /// Each function node has labels `["Function"]` (for both defined and
    /// external functions) or `["Indirect"]` depending on the node kind.
    /// The `kind` property distinguishes `"function"` from `"external"`.
    /// All edges have
    /// `edge_type` = "CALLS".
    #[must_use]
    pub fn to_pg(
        &self,
        module: &AirModule,
        resolver: Option<&DisplayResolver<'_>>,
    ) -> PropertyGraph {
        let func_names: BTreeMap<FunctionId, String> = module
            .functions
            .iter()
            .map(|f| (f.id, f.name.clone()))
            .collect();

        // Build function ID → span lookup for function declaration spans
        let func_spans: BTreeMap<FunctionId, &saf_core::span::Span> = module
            .functions
            .iter()
            .filter_map(|f| Some((f.id, f.span.as_ref()?)))
            .collect();

        let mut pg = PropertyGraph::new("callgraph");

        for node in &self.nodes {
            let (id, labels, properties) = match node {
                CallGraphNode::Function(fid) => {
                    let mut props = BTreeMap::new();
                    if let Some(name) = func_names.get(fid) {
                        props.insert("name".to_string(), serde_json::Value::String(name.clone()));
                    }
                    props.insert(
                        "kind".to_string(),
                        serde_json::Value::String("function".to_string()),
                    );
                    if let Some(span) = func_spans.get(fid) {
                        props.insert(
                            "span".to_string(),
                            span_to_property(span, &module.source_files),
                        );
                    }
                    (fid.to_hex(), vec!["Function".to_string()], props)
                }
                CallGraphNode::External { name, func } => {
                    let mut props = BTreeMap::new();
                    props.insert("name".to_string(), serde_json::Value::String(name.clone()));
                    props.insert(
                        "kind".to_string(),
                        serde_json::Value::String("external".to_string()),
                    );
                    (func.to_hex(), vec!["Function".to_string()], props)
                }
                CallGraphNode::IndirectPlaceholder { site } => {
                    let mut props = BTreeMap::new();
                    props.insert(
                        "kind".to_string(),
                        serde_json::Value::String("indirect".to_string()),
                    );
                    (site.to_hex(), vec!["Indirect".to_string()], props)
                }
            };
            let mut pg_node = PgNode {
                id,
                labels,
                properties,
            };
            enrich_node(&mut pg_node, resolver);
            pg.nodes.push(pg_node);
        }

        for (src, dsts) in &self.edges {
            let src_id = match src {
                CallGraphNode::Function(fid) | CallGraphNode::External { func: fid, .. } => {
                    fid.to_hex()
                }
                CallGraphNode::IndirectPlaceholder { site } => site.to_hex(),
            };
            for dst in dsts {
                let dst_id = match dst {
                    CallGraphNode::Function(fid) | CallGraphNode::External { func: fid, .. } => {
                        fid.to_hex()
                    }
                    CallGraphNode::IndirectPlaceholder { site } => site.to_hex(),
                };
                pg.edges.push(PgEdge {
                    src: src_id.clone(),
                    dst: dst_id,
                    edge_type: "CALLS".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        pg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction};
    use saf_core::ids::{BlockId, ModuleId};

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

    fn make_block_with_call(id: u128, calls: Vec<Operation>) -> AirBlock {
        let mut instructions: Vec<_> = calls
            .into_iter()
            .enumerate()
            .map(|(i, op)| Instruction::new(InstId::new(id * 100 + i as u128), op))
            .collect();
        // Add terminator
        instructions.push(Instruction::new(InstId::new(id * 100 + 99), Operation::Ret));
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions,
        }
    }

    #[test]
    fn callgraph_no_calls() {
        // Single function with no calls
        let func = make_function(
            1,
            "main",
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![Instruction::new(InstId::new(100), Operation::Ret)],
            }],
        );
        let module = make_module(vec![func]);
        let cg = CallGraph::build(&module);

        assert_eq!(cg.nodes.len(), 1);
        assert!(
            cg.nodes
                .contains(&CallGraphNode::Function(FunctionId::new(1)))
        );
        assert!(cg.edges[&CallGraphNode::Function(FunctionId::new(1))].is_empty());
    }

    #[test]
    fn callgraph_direct_call() {
        // main calls helper
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);

        assert_eq!(cg.nodes.len(), 2);

        // main -> helper edge
        let main_node = CallGraphNode::Function(FunctionId::new(1));
        let helper_node = CallGraphNode::Function(FunctionId::new(2));
        assert!(cg.edges[&main_node].contains(&helper_node));
        assert!(cg.edges[&helper_node].is_empty());
    }

    #[test]
    fn callgraph_external_call() {
        // main calls external printf
        let printf = make_declaration(2, "printf");
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let module = make_module(vec![main, printf]);
        let cg = CallGraph::build(&module);

        assert_eq!(cg.nodes.len(), 2);

        // Check that printf is marked as external
        let printf_node = cg
            .nodes
            .iter()
            .find(|n| matches!(n, CallGraphNode::External { name, .. } if name == "printf"))
            .expect("printf should be external");
        assert!(matches!(printf_node, CallGraphNode::External { .. }));

        // main -> printf edge
        let main_node = CallGraphNode::Function(FunctionId::new(1));
        assert!(cg.edges[&main_node].contains(printf_node));
    }

    #[test]
    fn callgraph_indirect_call() {
        // main has indirect call
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallIndirect {
                    expected_signature: None,
                }],
            )],
        );
        let module = make_module(vec![main]);
        let cg = CallGraph::build(&module);

        // Should have function node + indirect placeholder
        assert_eq!(cg.nodes.len(), 2);

        let indirect_nodes: Vec<_> = cg.indirect_calls();
        assert_eq!(indirect_nodes.len(), 1);

        // main -> indirect placeholder edge
        let main_node = CallGraphNode::Function(FunctionId::new(1));
        assert!(cg.edges[&main_node].iter().any(|n| n.is_indirect()));
    }

    #[test]
    fn callgraph_multiple_calls() {
        // main calls helper twice, and also calls printf
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![
                    Operation::CallDirect {
                        callee: FunctionId::new(2),
                    },
                    Operation::CallDirect {
                        callee: FunctionId::new(3),
                    },
                    Operation::CallDirect {
                        callee: FunctionId::new(2),
                    },
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );
        let printf = make_declaration(3, "printf");
        let module = make_module(vec![main, helper, printf]);
        let cg = CallGraph::build(&module);

        // main -> helper and main -> printf edges (no duplicates in set)
        let main_node = CallGraphNode::Function(FunctionId::new(1));
        assert_eq!(cg.edges[&main_node].len(), 2);
    }

    #[test]
    fn callgraph_recursive() {
        // fib calls itself
        let fib = make_function(
            1,
            "fib",
            vec![make_block_with_call(
                1,
                vec![
                    Operation::CallDirect {
                        callee: FunctionId::new(1),
                    },
                    Operation::CallDirect {
                        callee: FunctionId::new(1),
                    },
                ],
            )],
        );
        let module = make_module(vec![fib]);
        let cg = CallGraph::build(&module);

        let fib_node = CallGraphNode::Function(FunctionId::new(1));
        assert!(cg.edges[&fib_node].contains(&fib_node));
    }

    #[test]
    fn callgraph_callers_of() {
        // a calls b, c calls b
        let a = make_function(
            1,
            "a",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(3),
                }],
            )],
        );
        let c = make_function(
            2,
            "c",
            vec![make_block_with_call(
                2,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(3),
                }],
            )],
        );
        let b = make_function(
            3,
            "b",
            vec![AirBlock {
                id: BlockId::new(3),
                label: None,
                instructions: vec![Instruction::new(InstId::new(300), Operation::Ret)],
            }],
        );
        let module = make_module(vec![a, c, b]);
        let cg = CallGraph::build(&module);

        let b_node = CallGraphNode::Function(FunctionId::new(3));
        let callers = cg.callers_of(&b_node);
        assert_eq!(callers.len(), 2);
        assert!(callers.contains(&CallGraphNode::Function(FunctionId::new(1))));
        assert!(callers.contains(&CallGraphNode::Function(FunctionId::new(2))));
    }

    #[test]
    fn callgraph_call_site_tracking() {
        // Verify call sites are properly tracked
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);

        // Call site ID is 100 (block 1 * 100 + index 0)
        let call_site = InstId::new(100);
        let target = cg.call_site_target(call_site);
        assert!(target.is_some());
        assert_eq!(
            target.unwrap(),
            &CallGraphNode::Function(FunctionId::new(2))
        );
    }

    #[test]
    fn callgraph_export_is_deterministic() {
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);

        let export1 = serde_json::to_string(&cg.export(&module)).unwrap();
        let export2 = serde_json::to_string(&cg.export(&module)).unwrap();
        assert_eq!(export1, export2);
    }

    #[test]
    fn callgraph_to_pg_basic() {
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);
        let pg = cg.to_pg(&module, None);

        assert_eq!(pg.graph_type, "callgraph");
        assert_eq!(pg.schema_version, "0.1.0");
        assert_eq!(pg.nodes.len(), 2);
        assert_eq!(pg.edges.len(), 1);

        // Both nodes should be Function labeled
        assert!(pg.nodes.iter().all(|n| n.labels == vec!["Function"]));

        // Edge should be CALLS
        assert_eq!(pg.edges[0].edge_type, "CALLS");

        // Nodes should have name properties
        let names: Vec<_> = pg
            .nodes
            .iter()
            .filter_map(|n| n.properties.get("name"))
            .filter_map(|v| v.as_str())
            .collect();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"helper"));
    }

    #[test]
    fn callgraph_to_pg_external_node() {
        let printf = make_declaration(2, "printf");
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let module = make_module(vec![main, printf]);
        let cg = CallGraph::build(&module);
        let pg = cg.to_pg(&module, None);

        // External functions should have "Function" label (kind distinguishes them)
        let ext = pg
            .nodes
            .iter()
            .find(|n| n.properties.get("kind").and_then(|v| v.as_str()) == Some("external"))
            .expect("should have an external node");
        assert_eq!(ext.labels, vec!["Function"]);
        assert_eq!(
            ext.properties.get("name").and_then(|v| v.as_str()),
            Some("printf")
        );
    }

    #[test]
    fn callgraph_to_pg_is_deterministic() {
        let main = make_function(
            1,
            "main",
            vec![make_block_with_call(
                1,
                vec![Operation::CallDirect {
                    callee: FunctionId::new(2),
                }],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);

        let pg1 = serde_json::to_string(&cg.to_pg(&module, None)).unwrap();
        let pg2 = serde_json::to_string(&cg.to_pg(&module, None)).unwrap();
        assert_eq!(pg1, pg2);
    }
}
