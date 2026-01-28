//! Sparse Flow-Sensitive Pointer Analysis (SFS).
//!
//! Implements the SFS algorithm (Hardekopf & Lin, CGO'11) on top of the
//! SVFG infrastructure. Uses Andersen CI as pre-analysis, annotates SVFG
//! indirect edges with object labels, then propagates `IN`/`OUT` dataflow
//! sets per SVFG node for address-taken objects while tracking top-level
//! pointer `pts` maps via direct edges.
//!
//! Strong updates kill stale values at singleton stores in non-recursive
//! functions when the target is not an array-collapsed location.

mod builder;
mod export;
mod solver;
mod strong_update;
mod version_table;

pub use builder::FsSvfgBuilder;
pub use export::FsPtaExport;
pub use solver::solve_flow_sensitive;

// FlowSensitivePtaResult, FsPtaConfig, and FsPtaDiagnostics are already public
// structs defined below in this module

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::ids::{LocId, ValueId};

use crate::pta::ptsset::{PtsConfig, PtsRepresentation};
use crate::svfg::{SvfgEdgeKind, SvfgNodeId};

// ---------------------------------------------------------------------------
// FsSvfgEdge
// ---------------------------------------------------------------------------

/// An edge in the object-labeled SVFG used for flow-sensitive propagation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FsSvfgEdge {
    /// Original SVFG edge kind.
    pub kind: SvfgEdgeKind,
    /// Target node.
    pub target: SvfgNodeId,
    /// Object labels — non-empty for indirect edges, empty for direct edges.
    pub objects: BTreeSet<LocId>,
}

// ---------------------------------------------------------------------------
// StoreInfo / LoadInfo
// ---------------------------------------------------------------------------

/// Metadata for a store SVFG node.
#[derive(Debug, Clone)]
pub struct StoreInfo {
    /// The pointer being stored through.
    pub ptr: ValueId,
    /// The value being stored.
    pub val: ValueId,
}

/// Metadata for a load SVFG node.
#[derive(Debug, Clone)]
pub struct LoadInfo {
    /// The pointer being loaded from.
    pub ptr: ValueId,
    /// The destination (result of the load).
    pub dst: ValueId,
}

// ---------------------------------------------------------------------------
// FsSvfg
// ---------------------------------------------------------------------------

/// Object-labeled SVFG for flow-sensitive pointer analysis.
///
/// Wraps the base SVFG, adding `LocId` object annotations on indirect edges
/// so the SFS solver knows which abstract objects flow along each edge.
#[derive(Debug, Clone)]
pub struct FsSvfg {
    /// Outgoing edges per node.
    successors: BTreeMap<SvfgNodeId, Vec<FsSvfgEdge>>,
    /// Incoming edges per node.
    predecessors: BTreeMap<SvfgNodeId, Vec<FsSvfgEdge>>,
    /// All nodes.
    nodes: BTreeSet<SvfgNodeId>,
    /// Store node metadata (pointer + stored value).
    /// A single SVFG value node may correspond to multiple store instructions
    /// when the same value is stored through different pointers (e.g.,
    /// `store &a, %c` and `store &a, %d` both have stored-value node `&a`).
    store_nodes: BTreeMap<SvfgNodeId, Vec<StoreInfo>>,
    /// Load node metadata (pointer + destination).
    load_nodes: BTreeMap<SvfgNodeId, LoadInfo>,
}

impl FsSvfg {
    /// Create a new empty `FsSvfg`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
            nodes: BTreeSet::new(),
            store_nodes: BTreeMap::new(),
            load_nodes: BTreeMap::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: SvfgNodeId) {
        self.nodes.insert(node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, from: SvfgNodeId, edge: FsSvfgEdge) {
        let target = edge.target;
        let rev = FsSvfgEdge {
            kind: edge.kind,
            target: from,
            objects: edge.objects.clone(),
        };
        self.nodes.insert(from);
        self.nodes.insert(target);
        self.successors.entry(from).or_default().push(edge);
        self.predecessors.entry(target).or_default().push(rev);
    }

    /// Register store metadata for a node.
    ///
    /// Multiple stores may share the same SVFG value node when the same value
    /// is stored through different pointers. Each call appends to the list.
    pub fn add_store_info(&mut self, node: SvfgNodeId, info: StoreInfo) {
        self.store_nodes.entry(node).or_default().push(info);
    }

    /// Register load metadata for a node.
    pub fn set_load_info(&mut self, node: SvfgNodeId, info: LoadInfo) {
        self.load_nodes.insert(node, info);
    }

    /// Get outgoing edges for a node.
    #[must_use]
    pub fn successors_of(&self, node: SvfgNodeId) -> &[FsSvfgEdge] {
        self.successors.get(&node).map_or(&[], Vec::as_slice)
    }

    /// Get store metadata for a node.
    ///
    /// Returns all store infos for this node. A node may have multiple
    /// store infos when the same value is stored through different pointers.
    #[must_use]
    pub fn store_infos(&self, node: SvfgNodeId) -> &[StoreInfo] {
        self.store_nodes.get(&node).map_or(&[], Vec::as_slice)
    }

    /// Get load metadata for a node.
    #[must_use]
    pub fn load_info(&self, node: SvfgNodeId) -> Option<&LoadInfo> {
        self.load_nodes.get(&node)
    }

    /// All nodes in the graph.
    #[must_use]
    pub fn nodes(&self) -> &BTreeSet<SvfgNodeId> {
        &self.nodes
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.successors.values().map(Vec::len).sum()
    }

    /// Number of store nodes (counting each store info entry).
    #[must_use]
    pub fn store_count(&self) -> usize {
        self.store_nodes.values().map(Vec::len).sum()
    }

    /// Number of load nodes.
    #[must_use]
    pub fn load_count(&self) -> usize {
        self.load_nodes.len()
    }

    /// Get all load node metadata.
    #[must_use]
    pub fn load_nodes(&self) -> &BTreeMap<SvfgNodeId, LoadInfo> {
        &self.load_nodes
    }
}

impl Default for FsSvfg {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FsPtaConfig
// ---------------------------------------------------------------------------

/// Configuration for flow-sensitive pointer analysis.
#[derive(Debug, Clone)]
pub struct FsPtaConfig {
    /// Maximum worklist iterations before giving up.
    pub max_iterations: usize,
    /// Points-to set representation configuration.
    ///
    /// Controls the internal points-to set representation used by the
    /// FS-PTA solver. Currently dispatches to `BTreePtsSet`; the generic
    /// infrastructure supports `RoaringPtsSet` and `BddPtsSet` for future use.
    pub pts_config: PtsConfig,
    /// Version table compaction interval (VFS solver).
    ///
    /// Every `compact_interval` worklist iterations, unreferenced versions
    /// are garbage-collected from the version table. Set to 0 to disable.
    pub compact_interval: usize,
    /// Skip conversion of `df_in`/`df_out` to `BTreeMap` at solver completion.
    ///
    /// The conversion expands shared VFS versions into per-entry `BTreeSet`s
    /// which can be extremely memory-intensive for large programs (e.g., tmux
    /// expands from ~2 MB of deduplicated versions to ~20 GB of materialized
    /// `BTreeSets`). Set to `true` when only `pts` and `diagnostics` are needed.
    pub skip_df_materialization: bool,
}

impl Default for FsPtaConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100_000,
            pts_config: PtsConfig::default(),
            compact_interval: 10_000,
            skip_df_materialization: false,
        }
    }
}

impl FsPtaConfig {
    /// Create a config that uses `BTreeSet` for points-to sets.
    #[must_use]
    pub fn with_btreeset(mut self) -> Self {
        self.pts_config = PtsConfig::btreeset();
        self
    }

    /// Create a config that uses `BitVector` for points-to sets.
    #[must_use]
    pub fn with_bitvector(mut self) -> Self {
        self.pts_config = PtsConfig::bitvector();
        self
    }

    /// Create a config that uses `BDD` for points-to sets.
    #[must_use]
    pub fn with_bdd(mut self) -> Self {
        self.pts_config = PtsConfig::bdd();
        self
    }

    /// Set the points-to set representation explicitly.
    #[must_use]
    pub fn with_pts_representation(mut self, repr: PtsRepresentation) -> Self {
        self.pts_config = self.pts_config.with_representation(repr);
        self
    }
}

// ---------------------------------------------------------------------------
// DfPointsTo
// ---------------------------------------------------------------------------

/// Per-location dataflow points-to map: `LocId` → `Set<LocId>`.
pub type DfPointsTo = BTreeMap<LocId, BTreeSet<LocId>>;

// ---------------------------------------------------------------------------
// FlowSensitivePtaResult
// ---------------------------------------------------------------------------

/// Result of flow-sensitive pointer analysis.
///
/// Contains per-value top-level points-to sets and per-node dataflow
/// `IN`/`OUT` sets for address-taken objects.
#[derive(Debug, Clone)]
pub struct FlowSensitivePtaResult {
    /// Top-level pointer points-to sets.
    pts: BTreeMap<ValueId, BTreeSet<LocId>>,
    /// Dataflow IN sets per SVFG node per location.
    df_in: BTreeMap<SvfgNodeId, DfPointsTo>,
    /// Dataflow OUT sets per SVFG node per location.
    df_out: BTreeMap<SvfgNodeId, DfPointsTo>,
    /// Analysis diagnostics.
    diagnostics: FsPtaDiagnostics,
}

impl FlowSensitivePtaResult {
    /// Get the top-level points-to set for a value.
    #[must_use]
    pub fn points_to(&self, value: ValueId) -> &BTreeSet<LocId> {
        static EMPTY: BTreeSet<LocId> = BTreeSet::new();
        self.pts.get(&value).unwrap_or(&EMPTY)
    }

    /// Get the dataflow points-to set for a location at a specific SVFG node.
    ///
    /// Returns the `IN` set at `node` for `loc`.
    #[must_use]
    pub fn points_to_at(&self, loc: LocId, node: SvfgNodeId) -> &BTreeSet<LocId> {
        static EMPTY: BTreeSet<LocId> = BTreeSet::new();
        self.df_in
            .get(&node)
            .and_then(|m| m.get(&loc))
            .unwrap_or(&EMPTY)
    }

    /// Check whether two pointers may alias at a specific SVFG node.
    ///
    /// Uses the flow-sensitive `IN` sets at `node` to refine alias queries.
    /// If neither pointer's target is tracked in the dataflow `IN` set at
    /// the given program point, falls back to the global top-level `pts` map.
    #[must_use]
    pub fn may_alias_at(&self, p: ValueId, q: ValueId, node: SvfgNodeId) -> crate::AliasResult {
        // Try flow-sensitive resolution: look up the IN set at this node and
        // compute per-program-point points-to sets for p and q.
        let (p_fs_owned, q_fs_owned) = if let Some(in_map) = self.df_in.get(&node) {
            (
                self.flow_sensitive_pts_at(p, in_map),
                self.flow_sensitive_pts_at(q, in_map),
            )
        } else {
            (None, None)
        };

        // Resolve each pointer's points-to set: prefer flow-sensitive if
        // available, otherwise fall back to the global top-level `pts` map.
        let p_ref = p_fs_owned.as_ref().or_else(|| self.pts.get(&p));
        let q_ref = q_fs_owned.as_ref().or_else(|| self.pts.get(&q));

        match (p_ref, q_ref) {
            (None, _) | (_, None) => crate::AliasResult::Unknown,
            (Some(ps), Some(qs)) => {
                if ps.is_empty() || qs.is_empty() {
                    crate::AliasResult::Unknown
                } else if ps.is_disjoint(qs) {
                    crate::AliasResult::No
                } else if ps == qs && ps.len() == 1 {
                    // Singleton sets pointing to same location: MustAlias
                    crate::AliasResult::Must
                } else if ps == qs {
                    // Non-singleton equal sets: MayAlias
                    crate::AliasResult::May
                } else if ps.is_subset(qs) || qs.is_subset(ps) {
                    // One is a proper subset of the other means PartialAlias
                    crate::AliasResult::Partial
                } else {
                    crate::AliasResult::May
                }
            }
        }
    }

    /// Compute flow-sensitive points-to for a top-level pointer at a program point.
    ///
    /// Looks up `self.pts` for `value` to find the locations it may point to,
    /// then refines through the `IN` dataflow map at the given node: for each
    /// location in `pts(value)`, collects the objects that the `IN` set says
    /// are stored there. Returns `None` if no dataflow info is available for
    /// any of the target locations (meaning we should fall back to global `pts`).
    fn flow_sensitive_pts_at(
        &self,
        value: ValueId,
        in_map: &DfPointsTo,
    ) -> Option<BTreeSet<LocId>> {
        // The df_in maps track address-taken locations, not top-level pointers.
        // First, look up the global pts to find which LocIds this pointer may
        // point to, then for each such location check if the IN set at this
        // program point has flow-sensitive content.
        let pointer_targets = self.pts.get(&value)?;
        let mut fs_set = BTreeSet::new();
        let mut found_any = false;

        for loc in pointer_targets {
            if let Some(loc_pts) = in_map.get(loc) {
                found_any = true;
                fs_set.extend(loc_pts.iter().copied());
            }
        }

        if found_any { Some(fs_set) } else { None }
    }

    /// Get the analysis diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &FsPtaDiagnostics {
        &self.diagnostics
    }

    /// Get the raw top-level points-to map.
    #[must_use]
    pub fn points_to_map(&self) -> &BTreeMap<ValueId, BTreeSet<LocId>> {
        &self.pts
    }

    /// Get the raw `dfIn` map.
    #[must_use]
    pub fn df_in(&self) -> &BTreeMap<SvfgNodeId, DfPointsTo> {
        &self.df_in
    }

    /// Get the raw `dfOut` map.
    #[must_use]
    pub fn df_out(&self) -> &BTreeMap<SvfgNodeId, DfPointsTo> {
        &self.df_out
    }

    /// Compute flow-sensitive points-to sets for load destinations.
    ///
    /// For each load node in the `FsSvfg`, reconstruct the flow-sensitive
    /// points-to set for the load destination by reading `df_in` at that node.
    /// This gives per-program-point results instead of the monotonically
    /// accumulated global `pts` map.
    ///
    /// The returned map only contains entries for load destinations where the
    /// flow-sensitive result differs from (is more precise than) the global `pts`.
    #[must_use]
    pub fn compute_load_sensitive_pts(
        &self,
        fs_svfg: &FsSvfg,
    ) -> BTreeMap<ValueId, BTreeSet<LocId>> {
        let mut load_pts = BTreeMap::new();

        for (node, load_info) in fs_svfg.load_nodes() {
            let pointer_pts = self.pts.get(&load_info.ptr).cloned().unwrap_or_default();
            let in_map = self.df_in.get(node).cloned().unwrap_or_default();

            // Reconstruct: for each location the pointer points to,
            // collect what df_in says is stored at that location
            let mut fs_set = BTreeSet::new();
            for loc in &pointer_pts {
                if let Some(loc_pt_set) = in_map.get(loc) {
                    fs_set.extend(loc_pt_set.iter().copied());
                }
            }

            // Only include if non-empty (empty means no flow-sensitive info)
            if !fs_set.is_empty() {
                load_pts.insert(load_info.dst, fs_set);
            }
        }

        load_pts
    }

    /// Export to JSON-serializable format.
    #[must_use]
    pub fn export(&self) -> FsPtaExport {
        export::export_fs_pta(self)
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

// ---------------------------------------------------------------------------
// FsPtaDiagnostics
// ---------------------------------------------------------------------------

/// Diagnostics from flow-sensitive PTA.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FsPtaDiagnostics {
    /// Total solver iterations.
    pub iterations: usize,
    /// Whether the iteration limit was hit.
    pub iteration_limit_hit: bool,
    /// Number of strong updates performed.
    pub strong_updates: usize,
    /// Number of weak updates performed.
    pub weak_updates: usize,
    /// Number of `FsSvfg` nodes.
    pub fs_svfg_nodes: usize,
    /// Number of `FsSvfg` edges.
    pub fs_svfg_edges: usize,
    /// Number of store nodes tracked.
    pub store_nodes: usize,
    /// Number of load nodes tracked.
    pub load_nodes: usize,
    /// Number of versions in the version table at solver completion.
    pub version_count: usize,
    /// Number of compaction passes performed.
    pub compactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mssa::MemAccessId;

    #[test]
    fn fs_svfg_edge_empty_graph() {
        let graph = FsSvfg::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn fs_svfg_add_node() {
        let mut graph = FsSvfg::new();
        let n = SvfgNodeId::Value(ValueId::new(1));
        graph.add_node(n);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.nodes().contains(&n));
    }

    #[test]
    fn fs_svfg_add_direct_edge() {
        let mut graph = FsSvfg::new();
        let n1 = SvfgNodeId::Value(ValueId::new(1));
        let n2 = SvfgNodeId::Value(ValueId::new(2));

        graph.add_edge(
            n1,
            FsSvfgEdge {
                kind: SvfgEdgeKind::DirectDef,
                target: n2,
                objects: BTreeSet::new(),
            },
        );

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let succs = graph.successors_of(n1);
        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0].kind, SvfgEdgeKind::DirectDef);
        assert_eq!(succs[0].target, n2);
        assert!(succs[0].objects.is_empty());
    }

    #[test]
    fn fs_svfg_add_indirect_edge_with_objects() {
        let mut graph = FsSvfg::new();
        let n1 = SvfgNodeId::Value(ValueId::new(1));
        let phi = SvfgNodeId::MemPhi(MemAccessId::new(100));

        let mut objects = BTreeSet::new();
        objects.insert(LocId::new(0));
        objects.insert(LocId::new(1));

        graph.add_edge(
            n1,
            FsSvfgEdge {
                kind: SvfgEdgeKind::IndirectStore,
                target: phi,
                objects,
            },
        );

        let succs = graph.successors_of(n1);
        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0].objects.len(), 2);
    }

    #[test]
    fn fs_svfg_store_load_info() {
        let mut graph = FsSvfg::new();
        let store_node = SvfgNodeId::Value(ValueId::new(10));
        let load_node = SvfgNodeId::Value(ValueId::new(20));

        graph.add_store_info(
            store_node,
            StoreInfo {
                ptr: ValueId::new(1),
                val: ValueId::new(2),
            },
        );
        graph.set_load_info(
            load_node,
            LoadInfo {
                ptr: ValueId::new(3),
                dst: ValueId::new(4),
            },
        );

        let si = graph.store_infos(store_node);
        assert_eq!(si.len(), 1);
        assert_eq!(si[0].ptr, ValueId::new(1));
        assert_eq!(si[0].val, ValueId::new(2));

        let li = graph.load_info(load_node).expect("load info exists");
        assert_eq!(li.ptr, ValueId::new(3));
        assert_eq!(li.dst, ValueId::new(4));

        assert_eq!(graph.store_count(), 1);
        assert_eq!(graph.load_count(), 1);
    }

    #[test]
    fn fs_pta_result_empty() {
        let result = FlowSensitivePtaResult {
            pts: BTreeMap::new(),
            df_in: BTreeMap::new(),
            df_out: BTreeMap::new(),
            diagnostics: FsPtaDiagnostics::default(),
        };
        assert!(result.points_to(ValueId::new(1)).is_empty());
        assert!(
            result
                .points_to_at(LocId::new(0), SvfgNodeId::Value(ValueId::new(1)))
                .is_empty()
        );
    }

    #[test]
    fn fs_pta_config_default() {
        let config = FsPtaConfig::default();
        assert_eq!(config.max_iterations, 100_000);
    }
}
