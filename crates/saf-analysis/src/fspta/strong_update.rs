//! Strong update condition checking for flow-sensitive PTA.
//!
//! A store `*p = v` can perform a strong update (kill + gen) only when all
//! three conditions hold:
//!
//! 1. `pts(p)` is a singleton (exactly one target location)
//! 2. The target location is not array-indexed (any `PathStep::Index` variant)
//! 3. The store is not inside a recursive function (SCC size > 1 or self-edge)
//!
//! These conditions match SVF's `FlowSensitive::isStrongUpdate()`.

use std::collections::BTreeSet;

use saf_core::ids::{FunctionId, LocId, ValueId};

use crate::PathStep;
use crate::PtaResult;
use crate::callgraph::{CallGraph, CallGraphNode};
use crate::graph_algo;

/// Pre-computed information for strong update checks.
pub struct StrongUpdateInfo {
    /// Functions in recursive SCCs (SCC size > 1 or self-recursive).
    recursive_functions: BTreeSet<FunctionId>,
}

impl StrongUpdateInfo {
    /// Pre-compute strong update info from the call graph.
    #[must_use]
    pub fn new(callgraph: &CallGraph) -> Self {
        let mut recursive_functions = BTreeSet::new();

        // Compute SCCs on the call graph
        let func_nodes: BTreeSet<CallGraphNode> = callgraph.nodes.clone();
        let sccs = graph_algo::tarjan_scc(&func_nodes, &callgraph.edges);

        for scc in &sccs {
            let is_multi_node = scc.len() > 1;

            for node in scc {
                let func_id = match node {
                    CallGraphNode::Function(id) | CallGraphNode::External { func: id, .. } => *id,
                    CallGraphNode::IndirectPlaceholder { .. } => continue,
                };

                if is_multi_node {
                    // Multi-node SCC → all members are recursive
                    recursive_functions.insert(func_id);
                } else {
                    // Single-node SCC → check for self-edge
                    if let Some(succs) = callgraph.edges.get(node) {
                        if succs.contains(node) {
                            recursive_functions.insert(func_id);
                        }
                    }
                }
            }
        }

        Self {
            recursive_functions,
        }
    }

    /// Check if a store can perform a strong update.
    ///
    /// Returns `true` if all three conditions are met:
    /// - `points_to_set(pointer)` is a singleton
    /// - Target location is not array-collapsed
    /// - Store is not in a recursive function
    #[must_use]
    pub fn can_strong_update(
        &self,
        pointer: ValueId,
        func_id: FunctionId,
        points_to_set: &BTreeSet<LocId>,
        pta_result: &PtaResult,
    ) -> bool {
        let _ = pointer; // pointer's points_to_set is provided directly

        // Condition 1: singleton points-to set
        if points_to_set.len() != 1 {
            return false;
        }

        let loc_id = *points_to_set
            .iter()
            .next()
            .expect("singleton set is non-empty");

        // Condition 2: not array-indexed (any Index variant blocks strong update)
        if let Some(location) = pta_result.location(loc_id) {
            for step in &location.path.steps {
                if matches!(step, PathStep::Index(_)) {
                    return false;
                }
            }
        }

        // Condition 3: not in a recursive function
        if self.recursive_functions.contains(&func_id) {
            return false;
        }

        true
    }

    /// Check if a function is recursive.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_recursive(&self, func_id: FunctionId) -> bool {
        self.recursive_functions.contains(&func_id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use saf_core::ids::ObjId;

    use crate::{FieldPath, FieldSensitivity, LocationFactory, PointsToMap, PtaDiagnostics};

    fn make_pta_with_locations() -> (PtaResult, LocId, LocId) {
        let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });
        let loc_base = factory.get_or_create(ObjId::new(1), FieldPath::empty());
        let loc_array = factory.get_or_create(ObjId::new(2), FieldPath::index());
        let pts_map = PointsToMap::new();
        let pta = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());
        (pta, loc_base, loc_array)
    }

    fn make_empty_callgraph() -> CallGraph {
        CallGraph {
            nodes: BTreeSet::new(),
            edges: std::collections::BTreeMap::new(),
            reverse_edges: std::collections::BTreeMap::new(),
            call_sites: std::collections::BTreeMap::new(),
            func_index: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn singleton_non_array_non_recursive_allows_strong_update() {
        let (pta, loc_base, _) = make_pta_with_locations();
        let cg = make_empty_callgraph();
        let info = StrongUpdateInfo::new(&cg);

        let mut pts = BTreeSet::new();
        pts.insert(loc_base);

        assert!(info.can_strong_update(ValueId::new(1), FunctionId::new(1), &pts, &pta));
    }

    #[test]
    fn non_singleton_rejects_strong_update() {
        let (pta, loc_base, loc_array) = make_pta_with_locations();
        let cg = make_empty_callgraph();
        let info = StrongUpdateInfo::new(&cg);

        // Use both locations from the same factory to ensure distinct LocIds
        let mut pts = BTreeSet::new();
        pts.insert(loc_base);
        pts.insert(loc_array);

        assert!(!info.can_strong_update(ValueId::new(1), FunctionId::new(1), &pts, &pta));
    }

    #[test]
    fn empty_pts_rejects_strong_update() {
        let (pta, _, _) = make_pta_with_locations();
        let cg = make_empty_callgraph();
        let info = StrongUpdateInfo::new(&cg);

        let pts = BTreeSet::new();
        assert!(!info.can_strong_update(ValueId::new(1), FunctionId::new(1), &pts, &pta));
    }

    #[test]
    fn array_collapsed_rejects_strong_update() {
        let (pta, _, loc_array) = make_pta_with_locations();
        let cg = make_empty_callgraph();
        let info = StrongUpdateInfo::new(&cg);

        let mut pts = BTreeSet::new();
        pts.insert(loc_array);

        assert!(!info.can_strong_update(ValueId::new(1), FunctionId::new(1), &pts, &pta));
    }

    #[test]
    fn recursive_function_rejects_strong_update() {
        let (pta, loc_base, _) = make_pta_with_locations();

        // Build a CG with a self-recursive function
        let func_node = CallGraphNode::Function(FunctionId::new(1));
        let mut nodes = BTreeSet::new();
        nodes.insert(func_node.clone());
        let mut edges = std::collections::BTreeMap::new();
        let mut self_edges = BTreeSet::new();
        self_edges.insert(func_node.clone());
        edges.insert(func_node, self_edges);

        let mut func_index = std::collections::BTreeMap::new();
        func_index.insert(
            FunctionId::new(1),
            CallGraphNode::Function(FunctionId::new(1)),
        );
        // Build reverse_edges from edges
        let mut reverse_edges = std::collections::BTreeMap::new();
        for (caller, callees) in &edges {
            for callee in callees {
                reverse_edges
                    .entry(callee.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(caller.clone());
            }
        }
        let cg = CallGraph {
            nodes,
            edges,
            reverse_edges,
            call_sites: std::collections::BTreeMap::new(),
            func_index,
        };
        let info = StrongUpdateInfo::new(&cg);

        let mut pts = BTreeSet::new();
        pts.insert(loc_base);

        assert!(!info.can_strong_update(ValueId::new(1), FunctionId::new(1), &pts, &pta));
        assert!(info.is_recursive(FunctionId::new(1)));
    }

    #[test]
    fn mutual_recursion_rejects_strong_update() {
        let (pta, loc_base, _) = make_pta_with_locations();

        let f1 = CallGraphNode::Function(FunctionId::new(1));
        let f2 = CallGraphNode::Function(FunctionId::new(2));

        let mut nodes = BTreeSet::new();
        nodes.insert(f1.clone());
        nodes.insert(f2.clone());

        let mut edges = std::collections::BTreeMap::new();
        let mut f1_succs = BTreeSet::new();
        f1_succs.insert(f2.clone());
        edges.insert(f1.clone(), f1_succs);

        let mut f2_succs = BTreeSet::new();
        f2_succs.insert(f1.clone());
        edges.insert(f2, f2_succs);

        let mut func_index = std::collections::BTreeMap::new();
        func_index.insert(FunctionId::new(1), f1);
        func_index.insert(
            FunctionId::new(2),
            CallGraphNode::Function(FunctionId::new(2)),
        );
        // Build reverse_edges from edges
        let mut reverse_edges2 = std::collections::BTreeMap::new();
        for (caller, callees) in &edges {
            for callee in callees {
                reverse_edges2
                    .entry(callee.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(caller.clone());
            }
        }
        let cg = CallGraph {
            nodes,
            edges,
            reverse_edges: reverse_edges2,
            call_sites: std::collections::BTreeMap::new(),
            func_index,
        };
        let info = StrongUpdateInfo::new(&cg);

        let mut pts = BTreeSet::new();
        pts.insert(loc_base);

        assert!(!info.can_strong_update(ValueId::new(1), FunctionId::new(1), &pts, &pta));
        assert!(info.is_recursive(FunctionId::new(1)));
        assert!(info.is_recursive(FunctionId::new(2)));
    }
}
