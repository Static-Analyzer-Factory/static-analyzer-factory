//! Value flow graph and reachability/taint analysis.
//!
//! This module provides value flow graph construction and query capabilities
//! for tracking data propagation through programs via SSA definitions,
//! transformations, function calls, and memory operations.
//!
//! See FR-FLOW-001 through FR-FLOW-004 for requirements.

mod builder;
mod config;
mod edge;
mod export;
mod finding;
mod node;
mod query;
#[cfg(feature = "z3-solver")]
pub mod taint_z3;
mod trace;

pub use builder::{ValueFlowBuilder, build_valueflow};
pub use config::{IncludeLocations, OpKind, TransformPropagation, ValueFlowConfig, ValueFlowMode};
pub use edge::EdgeKind;
pub use export::{
    EXPORT_SCHEMA_VERSION, ExportedConfig, ExportedFinding, SarifExport, ValueFlowExport,
    to_property_graph,
};
pub use finding::{Finding, FindingId};
pub use node::NodeId;
pub use query::{Flow, QueryLimits};
#[cfg(feature = "z3-solver")]
pub use taint_z3::{TaintFlowZ3Result, filter_taint_flows_z3};
pub use trace::{EnrichedStep, EnrichedTrace, NodeInfo, SpanInfo, Trace, TraceStep};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::ids::ValueId;

/// Diagnostics from value flow graph construction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValueFlowDiagnostics {
    /// Number of locations collapsed to `unknown_mem` due to exceeding max threshold.
    pub locations_collapsed: usize,
    /// Number of indirect calls encountered (may have imprecise edges).
    pub indirect_calls: usize,
    /// Warning messages generated during construction.
    pub warnings: Vec<String>,
}

/// Value flow graph for tracking data propagation.
///
/// The graph captures how data flows through a program via:
/// - SSA definitions (`DefUse` edges)
/// - Transformations like binary ops (`Transform` edges)
/// - Function calls (`CallArg` and `Return` edges)
/// - Memory operations (`Store` and `Load` edges)
#[derive(Debug, Clone)]
pub struct ValueFlowGraph {
    /// Outgoing edges: node → set of (kind, target).
    successors: BTreeMap<NodeId, BTreeSet<(EdgeKind, NodeId)>>,
    /// Incoming edges: node → set of (kind, source).
    predecessors: BTreeMap<NodeId, BTreeSet<(EdgeKind, NodeId)>>,
    /// All nodes in the graph.
    nodes: BTreeSet<NodeId>,
    /// Construction diagnostics.
    diagnostics: ValueFlowDiagnostics,
}

impl ValueFlowGraph {
    /// Create a new empty value flow graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
            nodes: BTreeSet::new(),
            diagnostics: ValueFlowDiagnostics::default(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: NodeId) {
        if self.nodes.insert(node) {
            self.successors.entry(node).or_default();
            self.predecessors.entry(node).or_default();
        }
    }

    /// Add an edge to the graph.
    ///
    /// Automatically adds both nodes if they don't exist.
    pub fn add_edge(&mut self, from: NodeId, kind: EdgeKind, to: NodeId) {
        self.add_node(from);
        self.add_node(to);
        self.successors.entry(from).or_default().insert((kind, to));
        self.predecessors
            .entry(to)
            .or_default()
            .insert((kind, from));
    }

    /// Get all nodes in the graph.
    #[must_use]
    pub fn nodes(&self) -> &BTreeSet<NodeId> {
        &self.nodes
    }

    /// Get the number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.successors.values().map(BTreeSet::len).sum()
    }

    /// Check if a node exists in the graph.
    #[must_use]
    pub fn contains_node(&self, node: NodeId) -> bool {
        self.nodes.contains(&node)
    }

    /// Get outgoing edges from a node.
    #[must_use]
    pub fn successors_of(&self, node: NodeId) -> Option<&BTreeSet<(EdgeKind, NodeId)>> {
        self.successors.get(&node)
    }

    /// Get incoming edges to a node.
    #[must_use]
    pub fn predecessors_of(&self, node: NodeId) -> Option<&BTreeSet<(EdgeKind, NodeId)>> {
        self.predecessors.get(&node)
    }

    /// Get the construction diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &ValueFlowDiagnostics {
        &self.diagnostics
    }

    /// Get mutable reference to diagnostics (for builder).
    pub fn diagnostics_mut(&mut self) -> &mut ValueFlowDiagnostics {
        &mut self.diagnostics
    }

    /// Remove a node and all its incident edges from the graph.
    ///
    /// Returns `true` if the node existed and was removed.
    pub fn remove_node(&mut self, node: NodeId) -> bool {
        if !self.nodes.remove(&node) {
            return false;
        }

        // Remove outgoing edges and clean up predecessors of targets
        if let Some(succs) = self.successors.remove(&node) {
            for (kind, target) in &succs {
                if let Some(preds) = self.predecessors.get_mut(target) {
                    preds.remove(&(*kind, node));
                }
            }
        }

        // Remove incoming edges and clean up successors of sources
        if let Some(preds) = self.predecessors.remove(&node) {
            for (kind, source) in &preds {
                if let Some(succs) = self.successors.get_mut(source) {
                    succs.remove(&(*kind, node));
                }
            }
        }

        true
    }

    /// Get all value nodes in the graph.
    pub fn value_nodes(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.nodes.iter().filter_map(NodeId::as_value)
    }

    /// Get edges from a value node (convenience method).
    #[must_use]
    pub fn successors_of_value(&self, value: ValueId) -> Option<&BTreeSet<(EdgeKind, NodeId)>> {
        self.successors_of(NodeId::value(value))
    }

    /// Get edges to a value node (convenience method).
    #[must_use]
    pub fn predecessors_of_value(&self, value: ValueId) -> Option<&BTreeSet<(EdgeKind, NodeId)>> {
        self.predecessors_of(NodeId::value(value))
    }

    /// Merge all nodes and edges from `other` into this graph.
    ///
    /// Duplicate nodes are silently ignored. Duplicate edges are
    /// deduplicated by the underlying `BTreeSet`.
    pub fn merge(&mut self, other: &ValueFlowGraph) {
        for &node in &other.nodes {
            self.add_node(node);
        }
        for (&from, edges) in &other.successors {
            for &(kind, to) in edges {
                self.add_edge(from, kind, to);
            }
        }
    }
}

impl Default for ValueFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Selectively rebuild the value-flow subgraph for a set of affected functions.
///
/// This is the incremental counterpart to a full [`build_valueflow`] call.
/// Instead of rebuilding the entire graph, it:
///
/// 1. Collects all `NodeId`s belonging to affected functions (scanning their
///    instructions for defined values and parameter nodes).
/// 2. Removes those nodes and their incident edges from `existing`.
/// 3. Builds a fresh VF graph for the full module (using existing builder
///    infrastructure), then extracts only nodes/edges involving affected
///    function values.
/// 4. Merges the extracted subgraph into `existing`.
///
/// This preserves VF nodes from unaffected functions while refreshing the
/// parts that may have changed due to code edits or PTA updates.
#[allow(dead_code)] // TODO(incremental): Remove once wired into incremental pipeline
pub fn rebuild_affected(
    existing: &mut ValueFlowGraph,
    affected_functions: &BTreeSet<saf_core::ids::FunctionId>,
    module: &saf_core::air::AirModule,
    defuse: &crate::defuse::DefUseGraph,
    callgraph: &crate::callgraph::CallGraph,
    pta: Option<&crate::PtaResult>,
    config: &ValueFlowConfig,
) {
    if affected_functions.is_empty() {
        return;
    }

    // 1. Collect all NodeIds belonging to affected functions
    let mut affected_values = BTreeSet::new();
    for func in &module.functions {
        if func.is_declaration || !affected_functions.contains(&func.id) {
            continue;
        }
        for param in &func.params {
            affected_values.insert(param.id);
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    affected_values.insert(dst);
                }
            }
        }
    }

    let nodes_to_remove: BTreeSet<NodeId> =
        affected_values.iter().map(|v| NodeId::value(*v)).collect();

    // 2. Remove those nodes from existing graph
    for node in &nodes_to_remove {
        existing.remove_node(*node);
    }

    // 3. Build fresh VF graph using the standard builder, then extract
    //    edges involving affected values (including cross-function edges
    //    like CallArg and Return that connect affected and unaffected code)
    let fresh = build_valueflow(config, module, defuse, callgraph, pta);

    // 4. Extract and merge edges that touch affected values
    for (&from, edges) in &fresh.successors {
        for &(kind, to) in edges {
            let from_affected = nodes_to_remove.contains(&from);
            let to_affected = nodes_to_remove.contains(&to);
            // Include edges where at least one endpoint is an affected value,
            // or edges involving location/unknown_mem nodes connected to
            // affected values
            if from_affected || to_affected {
                existing.add_edge(from, kind, to);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::LocId;

    #[test]
    fn empty_graph() {
        let graph = ValueFlowGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn add_single_node() {
        let mut graph = ValueFlowGraph::new();
        let node = NodeId::value(ValueId::new(1));
        graph.add_node(node);

        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(node));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn add_duplicate_node() {
        let mut graph = ValueFlowGraph::new();
        let node = NodeId::value(ValueId::new(1));
        graph.add_node(node);
        graph.add_node(node);

        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn add_edge_creates_nodes() {
        let mut graph = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));

        graph.add_edge(n1, EdgeKind::DefUse, n2);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.contains_node(n1));
        assert!(graph.contains_node(n2));
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn successors_and_predecessors() {
        let mut graph = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));
        let n3 = NodeId::value(ValueId::new(3));

        graph.add_edge(n1, EdgeKind::DefUse, n2);
        graph.add_edge(n1, EdgeKind::DefUse, n3);

        let succs = graph.successors_of(n1).unwrap();
        assert_eq!(succs.len(), 2);
        assert!(succs.contains(&(EdgeKind::DefUse, n2)));
        assert!(succs.contains(&(EdgeKind::DefUse, n3)));

        let preds_n2 = graph.predecessors_of(n2).unwrap();
        assert_eq!(preds_n2.len(), 1);
        assert!(preds_n2.contains(&(EdgeKind::DefUse, n1)));
    }

    #[test]
    fn multiple_edge_kinds() {
        let mut graph = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));

        graph.add_edge(n1, EdgeKind::DefUse, n2);
        graph.add_edge(n1, EdgeKind::Transform, n2);

        // Both edges should exist (different kinds)
        let succs = graph.successors_of(n1).unwrap();
        assert_eq!(succs.len(), 2);
        assert!(succs.contains(&(EdgeKind::DefUse, n2)));
        assert!(succs.contains(&(EdgeKind::Transform, n2)));
    }

    #[test]
    fn value_and_location_nodes() {
        let mut graph = ValueFlowGraph::new();
        let v1 = NodeId::value(ValueId::new(1));
        let loc = NodeId::location(LocId::new(100));
        let v2 = NodeId::value(ValueId::new(2));

        graph.add_edge(v1, EdgeKind::Store, loc);
        graph.add_edge(loc, EdgeKind::Load, v2);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        let values: Vec<_> = graph.value_nodes().collect();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn unknown_mem_node() {
        let mut graph = ValueFlowGraph::new();
        let v1 = NodeId::value(ValueId::new(1));
        let um = NodeId::unknown_mem();
        let v2 = NodeId::value(ValueId::new(2));

        graph.add_edge(v1, EdgeKind::Store, um);
        graph.add_edge(um, EdgeKind::Load, v2);

        assert_eq!(graph.node_count(), 3);
        assert!(graph.contains_node(um));
    }

    #[test]
    fn convenience_value_methods() {
        let mut graph = ValueFlowGraph::new();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);

        graph.add_edge(NodeId::value(v1), EdgeKind::DefUse, NodeId::value(v2));

        let succs = graph.successors_of_value(v1).unwrap();
        assert_eq!(succs.len(), 1);

        let preds = graph.predecessors_of_value(v2).unwrap();
        assert_eq!(preds.len(), 1);
    }

    #[test]
    fn diagnostics_access() {
        let mut graph = ValueFlowGraph::new();
        graph.diagnostics_mut().locations_collapsed = 5;
        graph
            .diagnostics_mut()
            .warnings
            .push("test warning".to_string());

        assert_eq!(graph.diagnostics().locations_collapsed, 5);
        assert_eq!(graph.diagnostics().warnings.len(), 1);
    }

    #[test]
    fn diagnostics_default() {
        let diag = ValueFlowDiagnostics::default();
        assert_eq!(diag.locations_collapsed, 0);
        assert_eq!(diag.indirect_calls, 0);
        assert!(diag.warnings.is_empty());
    }

    #[test]
    fn remove_node_basic() {
        let mut graph = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));
        let n3 = NodeId::value(ValueId::new(3));

        graph.add_edge(n1, EdgeKind::DefUse, n2);
        graph.add_edge(n2, EdgeKind::DefUse, n3);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        // Remove n2 - should remove both edges
        assert!(graph.remove_node(n2));
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);
        assert!(!graph.contains_node(n2));
        assert!(graph.contains_node(n1));
        assert!(graph.contains_node(n3));

        // n1's successors should be empty
        let succs = graph.successors_of(n1);
        assert!(succs.is_none() || succs.unwrap().is_empty());

        // n3's predecessors should be empty
        let preds = graph.predecessors_of(n3);
        assert!(preds.is_none() || preds.unwrap().is_empty());
    }

    #[test]
    fn remove_node_nonexistent() {
        let mut graph = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));

        assert!(!graph.remove_node(n1));
    }

    #[test]
    fn remove_node_preserves_other_edges() {
        let mut graph = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));
        let n3 = NodeId::value(ValueId::new(3));

        graph.add_edge(n1, EdgeKind::DefUse, n2);
        graph.add_edge(n1, EdgeKind::DefUse, n3);

        // Remove n2, n1->n3 should survive
        graph.remove_node(n2);
        assert_eq!(graph.edge_count(), 1);
        let succs = graph.successors_of(n1).unwrap();
        assert!(succs.contains(&(EdgeKind::DefUse, n3)));
        assert!(!succs.contains(&(EdgeKind::DefUse, n2)));
    }

    #[test]
    fn merge_graphs() {
        let mut g1 = ValueFlowGraph::new();
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));
        g1.add_edge(n1, EdgeKind::DefUse, n2);

        let mut g2 = ValueFlowGraph::new();
        let n3 = NodeId::value(ValueId::new(3));
        g2.add_edge(n2, EdgeKind::Transform, n3);

        g1.merge(&g2);

        assert_eq!(g1.node_count(), 3);
        assert_eq!(g1.edge_count(), 2);
        assert!(g1.contains_node(n3));
        let succs = g1.successors_of(n2).unwrap();
        assert!(succs.contains(&(EdgeKind::Transform, n3)));
    }
}
