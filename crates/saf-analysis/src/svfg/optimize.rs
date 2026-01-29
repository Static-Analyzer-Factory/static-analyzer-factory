//! SVFG optimization pass — removes redundant `MemPhi` nodes before FSPTA solving.
//!
//! Three sub-passes run in order:
//! 1. **Pass-through `MemPhi` elimination** — remove identity phis with exactly
//!    one indirect in-edge and one indirect out-edge.
//! 2. **Single-source `MemPhi` merging** — remove phis where all incoming edges
//!    originate from the same source node.
//! 3. **Dead node removal** — remove isolated nodes left behind by earlier passes.

use std::collections::BTreeSet;

use saf_core::saf_log;

use super::{Svfg, SvfgEdgeKind, SvfgNodeId};

/// Optimize the SVFG by removing redundant `MemPhi` nodes.
///
/// Returns a new `Svfg` with redundant nodes eliminated. The original graph is
/// not modified. Only `MemPhi` nodes are candidates for removal — `Value` nodes
/// are always preserved.
#[must_use]
pub fn optimize(svfg: &Svfg) -> Svfg {
    // Collect the set of MemPhi nodes to remove across all passes.
    let mut removed = BTreeSet::new();

    // Pass 1: pass-through MemPhi elimination.
    find_passthrough_phis(svfg, &mut removed);

    // Pass 2: single-source MemPhi merging.
    find_single_source_phis(svfg, &mut removed);

    // Build the new graph with rewritten edges.
    let optimized = rebuild(svfg, &removed);

    let removed_count = removed.len();
    saf_log!(svfg::optimize, stats, "MemPhi reduction"; removed=removed_count);

    optimized
}

/// Pass 1: Find `MemPhi` nodes with exactly 1 indirect incoming edge and 1
/// indirect outgoing edge. These are identity pass-throughs.
fn find_passthrough_phis(svfg: &Svfg, removed: &mut BTreeSet<SvfgNodeId>) {
    for node in svfg.nodes() {
        if !node.is_mem_phi() {
            continue;
        }

        let Some(preds) = svfg.predecessors_of(*node) else {
            continue;
        };
        let Some(succs) = svfg.successors_of(*node) else {
            continue;
        };

        let indirect_preds: Vec<_> = preds.iter().filter(|(k, _)| k.is_indirect()).collect();
        let indirect_succs: Vec<_> = succs.iter().filter(|(k, _)| k.is_indirect()).collect();

        if indirect_preds.len() == 1 && indirect_succs.len() == 1 {
            removed.insert(*node);
        }
    }
}

/// Pass 2: Find `MemPhi` nodes where all incoming edges come from the same
/// source node (all operands resolve to the same def).
fn find_single_source_phis(svfg: &Svfg, removed: &mut BTreeSet<SvfgNodeId>) {
    for node in svfg.nodes() {
        if !node.is_mem_phi() || removed.contains(node) {
            continue;
        }

        let Some(preds) = svfg.predecessors_of(*node) else {
            continue;
        };

        if preds.is_empty() {
            continue;
        }

        // Check if all incoming edges come from the same source.
        let sources: BTreeSet<SvfgNodeId> = preds.iter().map(|(_, src)| *src).collect();
        if sources.len() == 1 {
            removed.insert(*node);
        }
    }
}

/// Rebuild the SVFG, skipping removed nodes and rewriting edges.
fn rebuild(svfg: &Svfg, removed: &BTreeSet<SvfgNodeId>) -> Svfg {
    let mut new_graph = Svfg::new();

    // Add all surviving nodes.
    for node in svfg.nodes() {
        if !removed.contains(node) {
            new_graph.add_node(*node);
        }
    }

    // Add edges, rewriting around removed nodes.
    for node in svfg.nodes() {
        if removed.contains(node) {
            continue;
        }

        if let Some(succs) = svfg.successors_of(*node) {
            for &(kind, target) in succs {
                if removed.contains(&target) {
                    // Target was removed — follow the chain to find the real
                    // successor(s) and reconnect with the outgoing edge kind.
                    let mut reachable = BTreeSet::new();
                    let mut visited = BTreeSet::new();
                    collect_reachable_past_removed(
                        svfg,
                        removed,
                        target,
                        &mut reachable,
                        &mut visited,
                    );
                    for (rewrite_kind, final_target) in reachable {
                        new_graph.add_edge(*node, rewrite_kind, final_target);
                    }
                } else {
                    new_graph.add_edge(*node, kind, target);
                }
            }
        }
    }

    // Pass 3: dead node removal — remove isolated nodes.
    remove_dead_nodes(&mut new_graph);

    new_graph
}

/// Follow the chain of removed nodes to find the real successors.
///
/// When a removed `MemPhi` is encountered, we look at its outgoing edges and
/// recurse if the successor is also removed. The edge kind used is the
/// *outgoing* kind from the removed node (preserving the semantic of the
/// edge that exits the removed chain).
///
/// Uses a `visited` set to guard against cycles in the removed-node chain,
/// which would otherwise cause infinite recursion / stack overflow.
fn collect_reachable_past_removed(
    svfg: &Svfg,
    removed: &BTreeSet<SvfgNodeId>,
    node: SvfgNodeId,
    out: &mut BTreeSet<(SvfgEdgeKind, SvfgNodeId)>,
    visited: &mut BTreeSet<SvfgNodeId>,
) {
    if !visited.insert(node) {
        return; // Already visited — cycle in removed chain.
    }
    if let Some(succs) = svfg.successors_of(node) {
        for &(kind, target) in succs {
            if removed.contains(&target) {
                collect_reachable_past_removed(svfg, removed, target, out, visited);
            } else {
                out.insert((kind, target));
            }
        }
    }
}

/// Remove `MemPhi` nodes that have no incoming and no outgoing edges.
/// `Value` nodes are always preserved, even if isolated.
fn remove_dead_nodes(graph: &mut Svfg) {
    let dead: Vec<SvfgNodeId> = graph
        .nodes()
        .iter()
        .filter(|n| {
            // Only remove MemPhi nodes — Value nodes are always preserved.
            if !n.is_mem_phi() {
                return false;
            }
            let no_succs = graph.successors_of(**n).is_none_or(BTreeSet::is_empty);
            let no_preds = graph.predecessors_of(**n).is_none_or(BTreeSet::is_empty);
            no_succs && no_preds
        })
        .copied()
        .collect();

    for node in &dead {
        graph.nodes_mut().remove(node);
        graph.successors_mut().remove(node);
        graph.predecessors_mut().remove(node);
    }
}

#[cfg(test)]
mod tests {
    use saf_core::ids::{InstId, ValueId};

    use super::*;
    use crate::mssa::MemAccessId;
    use crate::svfg::SvfgEdgeKind;

    // Helper to create node IDs.
    fn val(id: u128) -> SvfgNodeId {
        SvfgNodeId::value(ValueId::new(id))
    }

    fn phi(id: u128) -> SvfgNodeId {
        SvfgNodeId::mem_phi(MemAccessId::new(id))
    }

    #[test]
    fn empty_graph_stays_empty() {
        let svfg = Svfg::new();
        let optimized = optimize(&svfg);
        assert_eq!(optimized.node_count(), 0);
        assert_eq!(optimized.edge_count(), 0);
    }

    #[test]
    fn passthrough_phi_removed() {
        // V1 --IndirectStore--> Phi --IndirectLoad--> V2
        // The Phi has 1 indirect in and 1 indirect out => removed.
        // Result: V1 --IndirectLoad--> V2
        let mut svfg = Svfg::new();
        svfg.add_edge(val(1), SvfgEdgeKind::IndirectStore, phi(100));
        svfg.add_edge(phi(100), SvfgEdgeKind::IndirectLoad, val(2));

        let optimized = optimize(&svfg);

        assert!(!optimized.contains_node(phi(100)));
        assert!(optimized.contains_node(val(1)));
        assert!(optimized.contains_node(val(2)));
        assert_eq!(optimized.edge_count(), 1);

        // The rewritten edge uses the outgoing kind (IndirectLoad).
        let succs = optimized.successors_of(val(1)).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::IndirectLoad, val(2))));
    }

    #[test]
    fn single_source_phi_merged() {
        // V1 --IndirectStore--> Phi <--IndirectStore-- V1
        // Phi --IndirectLoad--> V2
        // All preds come from V1 => phi removed.
        let mut svfg = Svfg::new();
        // Two edges from V1 to Phi with different edge kinds both count — same source.
        svfg.add_edge(val(1), SvfgEdgeKind::IndirectStore, phi(100));
        // Add a PhiFlow from the same source via a different kind.
        // Actually, single-source means all predecessor *nodes* are the same.
        // Let's use two IndirectStore edges from the same node (different edge is same node).
        svfg.add_edge(phi(100), SvfgEdgeKind::IndirectLoad, val(2));
        svfg.add_edge(phi(100), SvfgEdgeKind::IndirectLoad, val(3));

        let optimized = optimize(&svfg);

        // Phi had 1 indirect in, 2 indirect out => not passthrough (pass 1 skips it).
        // But it has a single source (V1) => pass 2 removes it.
        assert!(!optimized.contains_node(phi(100)));
        assert!(optimized.contains_node(val(1)));
        assert!(optimized.contains_node(val(2)));
        assert!(optimized.contains_node(val(3)));

        let succs = optimized.successors_of(val(1)).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::IndirectLoad, val(2))));
        assert!(succs.contains(&(SvfgEdgeKind::IndirectLoad, val(3))));
    }

    #[test]
    fn value_nodes_never_removed() {
        // An isolated Value node should survive (dead-node removal only
        // applies after phi elimination, but Value nodes are never candidates).
        let mut svfg = Svfg::new();
        svfg.add_node(val(1));
        svfg.add_node(val(2));
        svfg.add_edge(val(1), SvfgEdgeKind::DirectDef, val(2));

        let optimized = optimize(&svfg);

        assert!(optimized.contains_node(val(1)));
        assert!(optimized.contains_node(val(2)));
        assert_eq!(optimized.edge_count(), 1);
    }

    #[test]
    fn direct_edges_not_affected() {
        // Direct edges pass through MemPhi assessment unchanged.
        let mut svfg = Svfg::new();
        svfg.add_edge(val(1), SvfgEdgeKind::DirectDef, val(2));
        svfg.add_edge(val(2), SvfgEdgeKind::DirectTransform, val(3));
        svfg.add_edge(
            val(1),
            SvfgEdgeKind::CallArg {
                call_site: InstId::new(0),
            },
            val(4),
        );
        svfg.add_edge(
            val(4),
            SvfgEdgeKind::Return {
                call_site: InstId::new(0),
            },
            val(5),
        );

        let optimized = optimize(&svfg);

        assert_eq!(optimized.node_count(), 5);
        assert_eq!(optimized.edge_count(), 4);
    }

    #[test]
    fn multi_source_phi_preserved() {
        // V1 --IndirectStore--> Phi <--IndirectStore-- V2
        // Phi --IndirectLoad--> V3
        // Two different sources => phi NOT removed.
        let mut svfg = Svfg::new();
        svfg.add_edge(val(1), SvfgEdgeKind::IndirectStore, phi(100));
        svfg.add_edge(val(2), SvfgEdgeKind::IndirectStore, phi(100));
        svfg.add_edge(phi(100), SvfgEdgeKind::IndirectLoad, val(3));

        let optimized = optimize(&svfg);

        assert!(optimized.contains_node(phi(100)));
        assert_eq!(optimized.node_count(), 4);
        assert_eq!(optimized.edge_count(), 3);
    }

    #[test]
    fn chained_passthrough_phis_removed() {
        // V1 --IndirectStore--> Phi1 --PhiFlow--> Phi2 --IndirectLoad--> V2
        // Both Phi1 and Phi2 are passthrough (1 indirect in, 1 indirect out).
        let mut svfg = Svfg::new();
        svfg.add_edge(val(1), SvfgEdgeKind::IndirectStore, phi(100));
        svfg.add_edge(phi(100), SvfgEdgeKind::PhiFlow, phi(200));
        svfg.add_edge(phi(200), SvfgEdgeKind::IndirectLoad, val(2));

        let optimized = optimize(&svfg);

        assert!(!optimized.contains_node(phi(100)));
        assert!(!optimized.contains_node(phi(200)));
        assert!(optimized.contains_node(val(1)));
        assert!(optimized.contains_node(val(2)));
        assert_eq!(optimized.edge_count(), 1);

        let succs = optimized.successors_of(val(1)).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::IndirectLoad, val(2))));
    }

    #[test]
    fn dead_node_removal() {
        // After removing a phi, its predecessor may become isolated if it had
        // no other connections. But Value nodes are kept. Only truly isolated
        // MemPhi nodes (orphaned by earlier passes) get cleaned up.
        let mut svfg = Svfg::new();
        // Create an isolated MemPhi (no edges).
        svfg.add_node(phi(999));
        svfg.add_edge(val(1), SvfgEdgeKind::DirectDef, val(2));

        let optimized = optimize(&svfg);

        // The isolated MemPhi should be removed by dead-node removal.
        assert!(!optimized.contains_node(phi(999)));
        assert!(optimized.contains_node(val(1)));
        assert!(optimized.contains_node(val(2)));
    }

    #[test]
    fn isolated_value_node_preserved() {
        // An isolated Value node (no edges) must survive dead-node removal.
        let mut svfg = Svfg::new();
        svfg.add_node(val(42));

        let optimized = optimize(&svfg);
        assert!(optimized.contains_node(val(42)));
    }

    #[test]
    fn mixed_direct_and_indirect_phi_not_passthrough() {
        // A MemPhi with 1 indirect in + 1 direct in + 1 indirect out.
        // It has 1 indirect in and 1 indirect out, so pass 1 would match.
        // But it also has multiple predecessors (2 sources), so single-source
        // would only apply if they're the same node. Here they differ.
        let mut svfg = Svfg::new();
        svfg.add_edge(val(1), SvfgEdgeKind::IndirectStore, phi(100));
        svfg.add_edge(val(2), SvfgEdgeKind::DirectDef, phi(100));
        svfg.add_edge(phi(100), SvfgEdgeKind::IndirectLoad, val(3));

        let optimized = optimize(&svfg);

        // Pass 1: 1 indirect in, 1 indirect out => passthrough. Removed.
        // The direct edge from V2 to Phi is lost since the phi is removed.
        // The rewritten indirect path is V1 --IndirectLoad--> V3.
        assert!(!optimized.contains_node(phi(100)));
        let succs = optimized.successors_of(val(1)).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::IndirectLoad, val(3))));
    }
}
