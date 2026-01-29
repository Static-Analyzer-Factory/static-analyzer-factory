//! Sparse Value-Flow Graph (SVFG).
//!
//! Unifies direct (register) and indirect (memory) value-flow into a single
//! graph. Direct edges capture SSA def-use chains, transforms, and call
//! argument/return flow. Indirect edges capture value flow through memory
//! using Memory SSA clobber analysis to precisely link stores to loads.
//!
//! The SVFG is the foundation for SABER-style memory safety checkers
//! (leak, UAF, double-free) which are graph reachability problems on SVFG.
//!
//! See Plan 026 for full design documentation.

mod builder;
pub mod context;
mod export;
pub mod optimize;
mod program_point;
mod query;

pub use builder::SvfgBuilder;
pub use context::CallString;
pub use export::SvfgExport;
pub use program_point::{ProgramPoint, ProgramPointMap};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::id::id_to_hex;
use saf_core::ids::{InstId, ValueId};

use crate::guard::Guard;
use crate::mssa::MemAccessId;

// ---------------------------------------------------------------------------
// SvfgNodeId
// ---------------------------------------------------------------------------

/// A node in the SVFG: either an SSA value or a Memory SSA `Phi` merge point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SvfgNodeId {
    /// An SSA value (instruction result, parameter, global address).
    Value(ValueId),
    /// A Memory SSA Phi merge point (at control-flow joins).
    MemPhi(MemAccessId),
}

impl SvfgNodeId {
    /// Create a Value node.
    #[must_use]
    pub const fn value(id: ValueId) -> Self {
        Self::Value(id)
    }

    /// Create a `MemPhi` node.
    #[must_use]
    pub const fn mem_phi(id: MemAccessId) -> Self {
        Self::MemPhi(id)
    }

    /// Check if this is a Value node.
    #[must_use]
    pub const fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }

    /// Check if this is a `MemPhi` node.
    #[must_use]
    pub const fn is_mem_phi(&self) -> bool {
        matches!(self, Self::MemPhi(_))
    }

    /// Get the `ValueId` if this is a `Value` node.
    #[must_use]
    pub const fn as_value(&self) -> Option<ValueId> {
        match self {
            Self::Value(id) => Some(*id),
            Self::MemPhi(_) => None,
        }
    }

    /// Get the `MemAccessId` if this is a `MemPhi` node.
    #[must_use]
    pub const fn as_mem_phi(&self) -> Option<MemAccessId> {
        match self {
            Self::MemPhi(id) => Some(*id),
            Self::Value(_) => None,
        }
    }

    /// Format as hex string for export.
    #[must_use]
    pub fn to_hex(&self) -> String {
        match self {
            Self::Value(id) => id_to_hex(id.raw()),
            Self::MemPhi(id) => id_to_hex(id.raw()),
        }
    }
}

impl From<ValueId> for SvfgNodeId {
    fn from(id: ValueId) -> Self {
        Self::Value(id)
    }
}

impl From<MemAccessId> for SvfgNodeId {
    fn from(id: MemAccessId) -> Self {
        Self::MemPhi(id)
    }
}

// ---------------------------------------------------------------------------
// SvfgEdgeKind
// ---------------------------------------------------------------------------

/// Kind of edge in the SVFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SvfgEdgeKind {
    // --- Direct (top-level SSA) ---
    /// SSA def-use chain (including `phi` incoming, select, copy).
    DirectDef,
    /// Binary/unary/cast/`GEP` operand to result.
    DirectTransform,
    /// Actual argument to formal parameter.
    CallArg { call_site: InstId },
    /// Callee return value to caller result.
    Return { call_site: InstId },

    // --- Indirect (through memory, via MSSA) ---
    /// Store's value to load's result when clobber is a `Store`.
    IndirectDef,
    /// Store's value to `MemPhi` node (store feeds a `Phi`).
    IndirectStore,
    /// `MemPhi` node to load's result (load reads from `Phi`).
    IndirectLoad,
    /// `MemPhi` to `MemPhi` (nested `Phi` chaining).
    PhiFlow,
}

impl SvfgEdgeKind {
    /// Get a human-readable name for the edge kind.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DirectDef => "direct_def",
            Self::DirectTransform => "direct_transform",
            Self::CallArg { .. } => "call_arg",
            Self::Return { .. } => "return",
            Self::IndirectDef => "indirect_def",
            Self::IndirectStore => "indirect_store",
            Self::IndirectLoad => "indirect_load",
            Self::PhiFlow => "phi_flow",
        }
    }

    /// Check if this is a direct (top-level SSA) edge.
    #[must_use]
    pub const fn is_direct(&self) -> bool {
        matches!(
            self,
            Self::DirectDef | Self::DirectTransform | Self::CallArg { .. } | Self::Return { .. }
        )
    }

    /// Check if this is an indirect (memory) edge.
    #[must_use]
    pub const fn is_indirect(&self) -> bool {
        matches!(
            self,
            Self::IndirectDef | Self::IndirectStore | Self::IndirectLoad | Self::PhiFlow
        )
    }
}

// ---------------------------------------------------------------------------
// SvfgDiagnostics
// ---------------------------------------------------------------------------

/// Diagnostics from SVFG construction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SvfgDiagnostics {
    /// Number of direct (top-level SSA) edges.
    pub direct_edge_count: usize,
    /// Number of indirect (memory) edges.
    pub indirect_edge_count: usize,
    /// Number of `MemPhi` nodes created.
    pub mem_phi_count: usize,
    /// Number of call-clobber cases skipped (deferred to future epic).
    pub skipped_call_clobbers: usize,
    /// Number of `LiveOnEntry` clobbers skipped.
    pub skipped_live_on_entry: usize,
}

// ---------------------------------------------------------------------------
// Svfg
// ---------------------------------------------------------------------------

/// Sparse Value-Flow Graph.
///
/// Unifies direct (register) and indirect (memory) value-flow into one graph.
/// Nodes are SSA values or Memory SSA Phi merge points. Edges capture how
/// data flows through the program.
#[derive(Debug, Clone)]
pub struct Svfg {
    /// Outgoing edges: node -> set of (kind, target).
    successors: BTreeMap<SvfgNodeId, BTreeSet<(SvfgEdgeKind, SvfgNodeId)>>,
    /// Incoming edges: node -> set of (kind, source).
    predecessors: BTreeMap<SvfgNodeId, BTreeSet<(SvfgEdgeKind, SvfgNodeId)>>,
    /// All nodes in the graph.
    nodes: BTreeSet<SvfgNodeId>,
    /// Guards on edges: (from, to) -> list of guards that must hold for the edge.
    /// Used for path-sensitive analysis (Plan 148 Phase B).
    edge_guards: BTreeMap<(SvfgNodeId, SvfgNodeId), Vec<Guard>>,
    /// Construction diagnostics.
    diagnostics: SvfgDiagnostics,
}

impl Svfg {
    /// Create a new empty SVFG.
    #[must_use]
    pub fn new() -> Self {
        Self {
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
            nodes: BTreeSet::new(),
            edge_guards: BTreeMap::new(),
            diagnostics: SvfgDiagnostics::default(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: SvfgNodeId) {
        if self.nodes.insert(node) {
            self.successors.entry(node).or_default();
            self.predecessors.entry(node).or_default();
        }
    }

    /// Remove a specific edge from the graph (nodes are retained).
    pub fn remove_edge(&mut self, from: SvfgNodeId, kind: SvfgEdgeKind, to: SvfgNodeId) {
        if let Some(succs) = self.successors.get_mut(&from) {
            succs.remove(&(kind, to));
        }
        if let Some(preds) = self.predecessors.get_mut(&to) {
            preds.remove(&(kind, from));
        }
        self.edge_guards.remove(&(from, to));
    }

    /// Add an edge to the graph. Automatically adds both endpoints as nodes.
    pub fn add_edge(&mut self, from: SvfgNodeId, kind: SvfgEdgeKind, to: SvfgNodeId) {
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
    pub fn nodes(&self) -> &BTreeSet<SvfgNodeId> {
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

    /// Get guards on a specific edge.
    #[must_use]
    pub fn edge_guard(&self, from: SvfgNodeId, to: SvfgNodeId) -> Option<&[Guard]> {
        self.edge_guards.get(&(from, to)).map(Vec::as_slice)
    }

    /// Accumulate guards on a specific edge.
    ///
    /// SVFG is a multigraph, so multiple edge kinds may connect the same
    /// `(from, to)` pair. Guards are appended rather than overwritten.
    pub fn set_edge_guard(&mut self, from: SvfgNodeId, to: SvfgNodeId, guards: Vec<Guard>) {
        if !guards.is_empty() {
            self.edge_guards
                .entry((from, to))
                .or_default()
                .extend(guards);
        }
    }

    /// Number of guarded edges.
    #[must_use]
    pub fn guarded_edge_count(&self) -> usize {
        self.edge_guards.len()
    }

    /// Check if a node exists in the graph.
    #[must_use]
    pub fn contains_node(&self, node: SvfgNodeId) -> bool {
        self.nodes.contains(&node)
    }

    /// Get outgoing edges from a node.
    #[must_use]
    pub fn successors_of(&self, node: SvfgNodeId) -> Option<&BTreeSet<(SvfgEdgeKind, SvfgNodeId)>> {
        self.successors.get(&node)
    }

    /// Get incoming edges to a node.
    #[must_use]
    pub fn predecessors_of(
        &self,
        node: SvfgNodeId,
    ) -> Option<&BTreeSet<(SvfgEdgeKind, SvfgNodeId)>> {
        self.predecessors.get(&node)
    }

    /// Get the construction diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &SvfgDiagnostics {
        &self.diagnostics
    }

    /// Get mutable reference to diagnostics (for builder).
    pub(crate) fn diagnostics_mut(&mut self) -> &mut SvfgDiagnostics {
        &mut self.diagnostics
    }

    /// Get mutable reference to nodes (for optimizer).
    pub(crate) fn nodes_mut(&mut self) -> &mut BTreeSet<SvfgNodeId> {
        &mut self.nodes
    }

    /// Get mutable reference to successors (for optimizer).
    pub(crate) fn successors_mut(
        &mut self,
    ) -> &mut BTreeMap<SvfgNodeId, BTreeSet<(SvfgEdgeKind, SvfgNodeId)>> {
        &mut self.successors
    }

    /// Get mutable reference to predecessors (for optimizer).
    pub(crate) fn predecessors_mut(
        &mut self,
    ) -> &mut BTreeMap<SvfgNodeId, BTreeSet<(SvfgEdgeKind, SvfgNodeId)>> {
        &mut self.predecessors
    }

    /// Iterate over all value nodes.
    pub fn value_nodes(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.nodes.iter().filter_map(SvfgNodeId::as_value)
    }

    /// Iterate over all `MemPhi` nodes.
    pub fn mem_phi_nodes(&self) -> impl Iterator<Item = MemAccessId> + '_ {
        self.nodes.iter().filter_map(SvfgNodeId::as_mem_phi)
    }

    /// Get successors of a value node (convenience).
    #[must_use]
    pub fn successors_of_value(
        &self,
        value: ValueId,
    ) -> Option<&BTreeSet<(SvfgEdgeKind, SvfgNodeId)>> {
        self.successors_of(SvfgNodeId::Value(value))
    }

    /// Get predecessors of a value node (convenience).
    #[must_use]
    pub fn predecessors_of_value(
        &self,
        value: ValueId,
    ) -> Option<&BTreeSet<(SvfgEdgeKind, SvfgNodeId)>> {
        self.predecessors_of(SvfgNodeId::Value(value))
    }

    /// Export to JSON-serializable format.
    #[must_use]
    pub fn export(&self) -> SvfgExport {
        export::export_svfg(self)
    }

    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        export::to_property_graph(self, resolver)
    }
}

impl Default for Svfg {
    fn default() -> Self {
        Self::new()
    }
}

/// Remove SVFG edges that correspond to PHI incoming values from dead
/// predecessor blocks (SCCP-unreachable).
///
/// When SCCP determines that a branch condition is a compile-time constant,
/// PHI nodes may have incoming edges from dead (never-taken) blocks. These
/// edges carry infeasible data-flow (e.g., null from a dead else-branch)
/// that produces false positives. This function removes those edges.
pub fn prune_dead_phi_edges(
    svfg: &mut Svfg,
    module: &saf_core::air::AirModule,
    dead_blocks: &std::collections::BTreeSet<saf_core::ids::BlockId>,
) -> usize {
    use saf_core::air::Operation;

    if dead_blocks.is_empty() {
        return 0;
    }

    // Collect edges to remove: (from_value, to_phi_result, edge_kind)
    let mut edges_to_remove = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Phi { incoming } = &inst.op {
                    if let Some(dst) = inst.dst {
                        let to = SvfgNodeId::Value(dst);
                        for (pred_block_id, value) in incoming {
                            if dead_blocks.contains(pred_block_id) {
                                let from = SvfgNodeId::Value(*value);
                                edges_to_remove.push((from, SvfgEdgeKind::DirectDef, to));
                            }
                        }
                    }
                }
            }
        }
    }

    let count = edges_to_remove.len();
    for (from, kind, to) in edges_to_remove {
        svfg.remove_edge(from, kind, to);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let graph = Svfg::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn add_value_node() {
        let mut graph = Svfg::new();
        let node = SvfgNodeId::value(ValueId::new(1));
        graph.add_node(node);

        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(node));
        assert!(node.is_value());
        assert_eq!(node.as_value(), Some(ValueId::new(1)));
    }

    #[test]
    fn add_mem_phi_node() {
        let mut graph = Svfg::new();
        let node = SvfgNodeId::mem_phi(MemAccessId::new(42));
        graph.add_node(node);

        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(node));
        assert!(node.is_mem_phi());
        assert_eq!(node.as_mem_phi(), Some(MemAccessId::new(42)));
    }

    #[test]
    fn add_duplicate_node() {
        let mut graph = Svfg::new();
        let node = SvfgNodeId::value(ValueId::new(1));
        graph.add_node(node);
        graph.add_node(node);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn add_edge_creates_nodes() {
        let mut graph = Svfg::new();
        let n1 = SvfgNodeId::value(ValueId::new(1));
        let n2 = SvfgNodeId::value(ValueId::new(2));

        graph.add_edge(n1, SvfgEdgeKind::DirectDef, n2);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.contains_node(n1));
        assert!(graph.contains_node(n2));
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn successors_and_predecessors() {
        let mut graph = Svfg::new();
        let n1 = SvfgNodeId::value(ValueId::new(1));
        let n2 = SvfgNodeId::value(ValueId::new(2));
        let n3 = SvfgNodeId::value(ValueId::new(3));

        graph.add_edge(n1, SvfgEdgeKind::DirectDef, n2);
        graph.add_edge(n1, SvfgEdgeKind::DirectDef, n3);

        let succs = graph.successors_of(n1).unwrap();
        assert_eq!(succs.len(), 2);
        assert!(succs.contains(&(SvfgEdgeKind::DirectDef, n2)));
        assert!(succs.contains(&(SvfgEdgeKind::DirectDef, n3)));

        let preds = graph.predecessors_of(n2).unwrap();
        assert_eq!(preds.len(), 1);
        assert!(preds.contains(&(SvfgEdgeKind::DirectDef, n1)));
    }

    #[test]
    fn multiple_edge_kinds() {
        let mut graph = Svfg::new();
        let n1 = SvfgNodeId::value(ValueId::new(1));
        let n2 = SvfgNodeId::value(ValueId::new(2));

        graph.add_edge(n1, SvfgEdgeKind::DirectDef, n2);
        graph.add_edge(n1, SvfgEdgeKind::DirectTransform, n2);

        let succs = graph.successors_of(n1).unwrap();
        assert_eq!(succs.len(), 2);
    }

    #[test]
    fn indirect_edges() {
        let mut graph = Svfg::new();
        let store_val = SvfgNodeId::value(ValueId::new(1));
        let phi = SvfgNodeId::mem_phi(MemAccessId::new(100));
        let load_val = SvfgNodeId::value(ValueId::new(2));

        graph.add_edge(store_val, SvfgEdgeKind::IndirectStore, phi);
        graph.add_edge(phi, SvfgEdgeKind::IndirectLoad, load_val);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        let succs_phi = graph.successors_of(phi).unwrap();
        assert!(succs_phi.contains(&(SvfgEdgeKind::IndirectLoad, load_val)));
    }

    #[test]
    fn edge_kind_classification() {
        assert!(SvfgEdgeKind::DirectDef.is_direct());
        assert!(SvfgEdgeKind::DirectTransform.is_direct());
        assert!(
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(0)
            }
            .is_direct()
        );
        assert!(
            SvfgEdgeKind::Return {
                call_site: InstId::new(0)
            }
            .is_direct()
        );

        assert!(SvfgEdgeKind::IndirectDef.is_indirect());
        assert!(SvfgEdgeKind::IndirectStore.is_indirect());
        assert!(SvfgEdgeKind::IndirectLoad.is_indirect());
        assert!(SvfgEdgeKind::PhiFlow.is_indirect());

        assert!(!SvfgEdgeKind::DirectDef.is_indirect());
        assert!(!SvfgEdgeKind::IndirectDef.is_direct());
    }

    #[test]
    fn edge_kind_serialization() {
        let kinds = [
            SvfgEdgeKind::DirectDef,
            SvfgEdgeKind::DirectTransform,
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(0),
            },
            SvfgEdgeKind::Return {
                call_site: InstId::new(0),
            },
            SvfgEdgeKind::IndirectDef,
            SvfgEdgeKind::IndirectStore,
            SvfgEdgeKind::IndirectLoad,
            SvfgEdgeKind::PhiFlow,
        ];

        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: SvfgEdgeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn edge_kind_snake_case_format() {
        assert_eq!(
            serde_json::to_string(&SvfgEdgeKind::DirectDef).unwrap(),
            "\"direct_def\""
        );
        assert_eq!(
            serde_json::to_string(&SvfgEdgeKind::IndirectStore).unwrap(),
            "\"indirect_store\""
        );
        assert_eq!(
            serde_json::to_string(&SvfgEdgeKind::PhiFlow).unwrap(),
            "\"phi_flow\""
        );
    }

    #[test]
    fn node_hex_format() {
        let val = SvfgNodeId::value(ValueId::new(0x1234));
        let hex = val.to_hex();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 34); // 0x + 32 hex chars

        let phi = SvfgNodeId::mem_phi(MemAccessId::new(0x5678));
        let hex2 = phi.to_hex();
        assert!(hex2.starts_with("0x"));
    }

    #[test]
    fn node_from_conversions() {
        let vid = ValueId::new(1);
        let node: SvfgNodeId = vid.into();
        assert_eq!(node, SvfgNodeId::Value(vid));

        let mid = MemAccessId::new(2);
        let node2: SvfgNodeId = mid.into();
        assert_eq!(node2, SvfgNodeId::MemPhi(mid));
    }

    #[test]
    fn value_and_mem_phi_iterators() {
        let mut graph = Svfg::new();
        graph.add_node(SvfgNodeId::value(ValueId::new(1)));
        graph.add_node(SvfgNodeId::value(ValueId::new(2)));
        graph.add_node(SvfgNodeId::mem_phi(MemAccessId::new(100)));

        let values: Vec<_> = graph.value_nodes().collect();
        assert_eq!(values.len(), 2);

        let phis: Vec<_> = graph.mem_phi_nodes().collect();
        assert_eq!(phis.len(), 1);
    }

    #[test]
    fn diagnostics_default() {
        let diag = SvfgDiagnostics::default();
        assert_eq!(diag.direct_edge_count, 0);
        assert_eq!(diag.indirect_edge_count, 0);
        assert_eq!(diag.mem_phi_count, 0);
        assert_eq!(diag.skipped_call_clobbers, 0);
        assert_eq!(diag.skipped_live_on_entry, 0);
    }

    #[test]
    fn convenience_value_methods() {
        let mut graph = Svfg::new();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        graph.add_edge(
            SvfgNodeId::value(v1),
            SvfgEdgeKind::DirectDef,
            SvfgNodeId::value(v2),
        );

        assert!(graph.successors_of_value(v1).is_some());
        assert!(graph.predecessors_of_value(v2).is_some());
    }
}
