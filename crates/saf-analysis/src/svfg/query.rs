//! SVFG reachability and path queries.

use std::collections::{BTreeSet, VecDeque};

use saf_core::ids::ValueId;

use super::{Svfg, SvfgNodeId};

impl Svfg {
    /// Find all nodes reachable forward from `from` (BFS on successors).
    #[must_use]
    pub fn forward_reachable(&self, from: SvfgNodeId) -> BTreeSet<SvfgNodeId> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();

        if self.contains_node(from) {
            queue.push_back(from);
            visited.insert(from);
        }

        while let Some(node) = queue.pop_front() {
            if let Some(succs) = self.successors_of(node) {
                for (_, target) in succs {
                    if visited.insert(*target) {
                        queue.push_back(*target);
                    }
                }
            }
        }

        visited
    }

    /// Find all nodes reachable backward from `from` (BFS on predecessors).
    #[must_use]
    pub fn backward_reachable(&self, from: SvfgNodeId) -> BTreeSet<SvfgNodeId> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();

        if self.contains_node(from) {
            queue.push_back(from);
            visited.insert(from);
        }

        while let Some(node) = queue.pop_front() {
            if let Some(preds) = self.predecessors_of(node) {
                for (_, source) in preds {
                    if visited.insert(*source) {
                        queue.push_back(*source);
                    }
                }
            }
        }

        visited
    }

    /// Check if `to` is reachable from `from` via forward BFS.
    #[must_use]
    pub fn reachable(&self, from: ValueId, to: ValueId) -> bool {
        let from_node = SvfgNodeId::Value(from);
        let to_node = SvfgNodeId::Value(to);

        if !self.contains_node(from_node) || !self.contains_node(to_node) {
            return false;
        }

        if from == to {
            return true;
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from_node);
        visited.insert(from_node);

        while let Some(node) = queue.pop_front() {
            if let Some(succs) = self.successors_of(node) {
                for (_, target) in succs {
                    if *target == to_node {
                        return true;
                    }
                    if visited.insert(*target) {
                        queue.push_back(*target);
                    }
                }
            }
        }

        false
    }

    /// Find a value-flow path from `from` to `to`, up to `max_depth` nodes.
    ///
    /// Returns `None` if no path exists or `max_depth` is exceeded.
    #[must_use]
    pub fn value_flow_path(
        &self,
        from: ValueId,
        to: ValueId,
        max_depth: usize,
    ) -> Option<Vec<SvfgNodeId>> {
        let from_node = SvfgNodeId::Value(from);
        let to_node = SvfgNodeId::Value(to);

        if !self.contains_node(from_node) || !self.contains_node(to_node) {
            return None;
        }

        if from == to {
            return Some(vec![from_node]);
        }

        // BFS with parent tracking
        let mut visited = BTreeSet::new();
        let mut parent: std::collections::BTreeMap<SvfgNodeId, SvfgNodeId> =
            std::collections::BTreeMap::new();
        let mut queue = VecDeque::new();

        queue.push_back((from_node, 0usize));
        visited.insert(from_node);

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(succs) = self.successors_of(node) {
                for (_, target) in succs {
                    if visited.insert(*target) {
                        parent.insert(*target, node);
                        if *target == to_node {
                            // Reconstruct path
                            let mut path = vec![to_node];
                            let mut current = to_node;
                            while let Some(&prev) = parent.get(&current) {
                                path.push(prev);
                                current = prev;
                                if current == from_node {
                                    break;
                                }
                            }
                            path.reverse();
                            return Some(path);
                        }
                        queue.push_back((*target, depth + 1));
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mssa::MemAccessId;
    use crate::svfg::SvfgEdgeKind;

    fn make_chain_graph() -> Svfg {
        // v1 --DirectDef--> v2 --IndirectStore--> phi --IndirectLoad--> v3
        let mut g = Svfg::new();
        let v1 = SvfgNodeId::value(ValueId::new(1));
        let v2 = SvfgNodeId::value(ValueId::new(2));
        let phi = SvfgNodeId::mem_phi(MemAccessId::new(100));
        let v3 = SvfgNodeId::value(ValueId::new(3));

        g.add_edge(v1, SvfgEdgeKind::DirectDef, v2);
        g.add_edge(v2, SvfgEdgeKind::IndirectStore, phi);
        g.add_edge(phi, SvfgEdgeKind::IndirectLoad, v3);
        g
    }

    #[test]
    fn forward_reachable_chain() {
        let g = make_chain_graph();
        let reachable = g.forward_reachable(SvfgNodeId::value(ValueId::new(1)));
        assert_eq!(reachable.len(), 4); // v1, v2, phi, v3
        assert!(reachable.contains(&SvfgNodeId::value(ValueId::new(3))));
    }

    #[test]
    fn backward_reachable_chain() {
        let g = make_chain_graph();
        let reachable = g.backward_reachable(SvfgNodeId::value(ValueId::new(3)));
        assert_eq!(reachable.len(), 4); // v3, phi, v2, v1
        assert!(reachable.contains(&SvfgNodeId::value(ValueId::new(1))));
    }

    #[test]
    fn forward_reachable_nonexistent_node() {
        let g = make_chain_graph();
        let reachable = g.forward_reachable(SvfgNodeId::value(ValueId::new(999)));
        assert!(reachable.is_empty());
    }

    #[test]
    fn reachable_through_memory() {
        let g = make_chain_graph();
        assert!(g.reachable(ValueId::new(1), ValueId::new(3)));
        assert!(!g.reachable(ValueId::new(3), ValueId::new(1))); // no backward path
    }

    #[test]
    fn reachable_self() {
        let g = make_chain_graph();
        assert!(g.reachable(ValueId::new(1), ValueId::new(1)));
    }

    #[test]
    fn reachable_nonexistent() {
        let g = make_chain_graph();
        assert!(!g.reachable(ValueId::new(1), ValueId::new(999)));
    }

    #[test]
    fn value_flow_path_exists() {
        let g = make_chain_graph();
        let path = g.value_flow_path(ValueId::new(1), ValueId::new(3), 100);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], SvfgNodeId::value(ValueId::new(1)));
        assert_eq!(path[3], SvfgNodeId::value(ValueId::new(3)));
    }

    #[test]
    fn value_flow_path_not_found() {
        let g = make_chain_graph();
        let path = g.value_flow_path(ValueId::new(3), ValueId::new(1), 100);
        assert!(path.is_none());
    }

    #[test]
    fn value_flow_path_self() {
        let g = make_chain_graph();
        let path = g.value_flow_path(ValueId::new(1), ValueId::new(1), 100);
        assert_eq!(path, Some(vec![SvfgNodeId::value(ValueId::new(1))]));
    }

    #[test]
    fn value_flow_path_max_depth() {
        let g = make_chain_graph();
        // Depth 1 can't reach v3 (need at least 3 hops)
        let path = g.value_flow_path(ValueId::new(1), ValueId::new(3), 1);
        assert!(path.is_none());
    }
}
