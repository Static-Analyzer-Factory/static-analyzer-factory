//! Generic graph algorithms for static analysis.
//!
//! All algorithms use `BTreeSet`/`BTreeMap` for deterministic iteration order,
//! ensuring reproducible results across runs (NFR-DET-001).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A trait for types that can provide successors for graph traversal.
pub trait Successors<N> {
    /// Get the successors of a node.
    fn successors(&self, node: &N) -> Option<&BTreeSet<N>>;
}

// Implement for BTreeMap directly
impl<N: Ord> Successors<N> for BTreeMap<N, BTreeSet<N>> {
    fn successors(&self, node: &N) -> Option<&BTreeSet<N>> {
        self.get(node)
    }
}

/// Depth-first search traversal starting from a node.
///
/// Returns nodes in pre-order (visit order). Uses an explicit stack
/// to avoid stack overflow on large graphs.
pub fn dfs<N, G>(start: &N, graph: &G) -> Vec<N>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    let mut visited = BTreeSet::new();
    let mut result = Vec::new();
    let mut stack: Vec<N> = vec![start.clone()];

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());
        result.push(node.clone());

        if let Some(succs) = graph.successors(&node) {
            // Push successors in reverse order so that the smallest (first in
            // `BTreeSet` iteration) is processed first, matching the previous
            // recursive visit order.
            for succ in succs.iter().rev() {
                if !visited.contains(succ) {
                    stack.push(succ.clone());
                }
            }
        }
    }

    result
}

/// Breadth-first search traversal starting from a node.
///
/// Returns nodes in BFS order (level by level).
pub fn bfs<N, G>(start: &N, graph: &G) -> Vec<N>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    let mut visited = BTreeSet::new();
    let mut result = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back(start.clone());
    visited.insert(start.clone());

    while let Some(node) = queue.pop_front() {
        result.push(node.clone());

        if let Some(succs) = graph.successors(&node) {
            for succ in succs {
                if !visited.contains(succ) {
                    visited.insert(succ.clone());
                    queue.push_back(succ.clone());
                }
            }
        }
    }

    result
}

/// Post-order traversal starting from a node.
///
/// Returns nodes in post-order (children before parents). Uses an explicit
/// stack to avoid stack overflow on large graphs.
pub fn post_order<N, G>(start: &N, graph: &G) -> Vec<N>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    let mut visited = BTreeSet::new();
    let mut result = Vec::new();

    // Each stack frame is (node, is_returning). On the first visit
    // (`is_returning == false`) we mark the node visited and push its
    // successors; on the second visit (`is_returning == true`) we emit
    // the node to the result (post-order position).
    let mut stack: Vec<(N, bool)> = vec![(start.clone(), false)];

    while let Some((node, returning)) = stack.pop() {
        if returning {
            result.push(node);
            continue;
        }

        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());

        // Push the node again so we emit it *after* all successors.
        stack.push((node.clone(), true));

        if let Some(succs) = graph.successors(&node) {
            // Push successors in reverse order so that the smallest is
            // processed first, matching the previous recursive visit order.
            for succ in succs.iter().rev() {
                if !visited.contains(succ) {
                    stack.push((succ.clone(), false));
                }
            }
        }
    }

    result
}

/// Tarjan's algorithm for finding strongly connected components.
///
/// Returns SCCs in reverse topological order (leaf SCCs first).
/// Uses an explicit call stack to avoid stack overflow on large graphs.
pub fn tarjan_scc<N, G>(nodes: &BTreeSet<N>, graph: &G) -> Vec<BTreeSet<N>>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    // Each frame on the call stack represents a `strong_connect` invocation.
    // `succ_iter` holds the remaining successors to process. We collect them
    // into a `Vec` so the frame owns the data (no borrow on the graph).
    struct Frame<N> {
        node: N,
        succs: Vec<N>,
        succ_idx: usize,
    }

    let mut index: usize = 0;
    let mut indices: BTreeMap<N, usize> = BTreeMap::new();
    let mut low_links: BTreeMap<N, usize> = BTreeMap::new();
    let mut on_stack: BTreeSet<N> = BTreeSet::new();
    let mut tarjan_stack: Vec<N> = Vec::new();
    let mut sccs: Vec<BTreeSet<N>> = Vec::new();

    for root in nodes {
        if indices.contains_key(root) {
            continue;
        }

        // Simulate the initial call to `strong_connect(root)`.
        let mut call_stack: Vec<Frame<N>> = Vec::new();

        // Initialize the root frame.
        indices.insert(root.clone(), index);
        low_links.insert(root.clone(), index);
        index += 1;
        tarjan_stack.push(root.clone());
        on_stack.insert(root.clone());

        let root_succs: Vec<N> = graph
            .successors(root)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();

        call_stack.push(Frame {
            node: root.clone(),
            succs: root_succs,
            succ_idx: 0,
        });

        while let Some(frame) = call_stack.last_mut() {
            if frame.succ_idx < frame.succs.len() {
                let succ = frame.succs[frame.succ_idx].clone();
                frame.succ_idx += 1;

                if !indices.contains_key(&succ) {
                    // Equivalent to the recursive `strong_connect(succ)` call.
                    // Initialize the new frame and push it.
                    indices.insert(succ.clone(), index);
                    low_links.insert(succ.clone(), index);
                    index += 1;
                    tarjan_stack.push(succ.clone());
                    on_stack.insert(succ.clone());

                    let succ_succs: Vec<N> = graph
                        .successors(&succ)
                        .map(|s| s.iter().cloned().collect())
                        .unwrap_or_default();

                    call_stack.push(Frame {
                        node: succ,
                        succs: succ_succs,
                        succ_idx: 0,
                    });
                } else if on_stack.contains(&succ) {
                    // Back-edge to a node on the Tarjan stack.
                    let succ_index = *indices
                        .get(&succ)
                        .expect("succ index exists per contains_key check");
                    let node_low = low_links
                        .get_mut(&frame.node)
                        .expect("node low_link set at frame entry");
                    *node_low = (*node_low).min(succ_index);
                }
            } else {
                // All successors processed — this is the "return" from
                // `strong_connect`. Pop the frame and propagate lowlink
                // to the caller, then check if this node is an SCC root.
                let finished = call_stack
                    .pop()
                    .expect("call_stack is non-empty per while-let guard");

                // Propagate lowlink to caller (mirrors the post-recursion
                // `node_low = min(node_low, succ_low)` in the recursive version).
                if let Some(caller) = call_stack.last() {
                    let finished_low = *low_links
                        .get(&finished.node)
                        .expect("finished node low_link set at frame entry");
                    let caller_low = low_links
                        .get_mut(&caller.node)
                        .expect("caller node low_link set at frame entry");
                    *caller_low = (*caller_low).min(finished_low);
                }

                // If this node is an SCC root, pop its SCC.
                if low_links.get(&finished.node) == indices.get(&finished.node) {
                    let mut scc = BTreeSet::new();
                    loop {
                        let w = tarjan_stack
                            .pop()
                            .expect("stack contains at least node when node is SCC root");
                        on_stack.remove(&w);
                        let is_root = w == finished.node;
                        scc.insert(w);
                        if is_root {
                            break;
                        }
                    }
                    sccs.push(scc);
                }
            }
        }
    }

    sccs
}

/// Topological sort of a directed acyclic graph.
///
/// Returns `None` if the graph contains a cycle, otherwise returns nodes in
/// topological order (dependencies before dependents).
pub fn toposort<N, G>(nodes: &BTreeSet<N>, graph: &G) -> Option<Vec<N>>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    // Check for cycles using SCC - if any SCC has more than one node, there's a cycle
    let sccs = tarjan_scc(nodes, graph);
    for scc in &sccs {
        if scc.len() > 1 {
            return None;
        }
        // Also check for self-loops
        for node in scc {
            if let Some(succs) = graph.successors(node) {
                if succs.contains(node) {
                    return None;
                }
            }
        }
    }

    // SCCs are already in reverse topological order, flatten them
    let mut result = Vec::with_capacity(nodes.len());
    for scc in sccs.into_iter().rev() {
        result.extend(scc);
    }
    Some(result)
}

/// Find all nodes reachable from a starting node.
pub fn reachable<N, G>(start: &N, graph: &G) -> BTreeSet<N>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    dfs(start, graph).into_iter().collect()
}

/// Compute reverse post-order (useful for dataflow analysis).
pub fn reverse_post_order<N, G>(start: &N, graph: &G) -> Vec<N>
where
    N: Ord + Clone,
    G: Successors<N>,
{
    let mut result = post_order(start, graph);
    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph(edges: &[(i32, i32)]) -> BTreeMap<i32, BTreeSet<i32>> {
        let mut graph = BTreeMap::new();
        for &(from, to) in edges {
            graph.entry(from).or_insert_with(BTreeSet::new).insert(to);
            // Ensure all nodes exist in the map
            graph.entry(to).or_insert_with(BTreeSet::new);
        }
        graph
    }

    #[test]
    fn dfs_linear() {
        // 1 -> 2 -> 3
        let graph = make_graph(&[(1, 2), (2, 3)]);
        let result = dfs(&1, &graph);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn dfs_branching() {
        // 1 -> 2, 1 -> 3, 2 -> 4, 3 -> 4
        let graph = make_graph(&[(1, 2), (1, 3), (2, 4), (3, 4)]);
        let result = dfs(&1, &graph);
        // BTreeSet ensures deterministic order: visits 2 before 3
        assert_eq!(result, vec![1, 2, 4, 3]);
    }

    #[test]
    fn dfs_cycle() {
        // 1 -> 2 -> 3 -> 1
        let graph = make_graph(&[(1, 2), (2, 3), (3, 1)]);
        let result = dfs(&1, &graph);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn bfs_linear() {
        // 1 -> 2 -> 3
        let graph = make_graph(&[(1, 2), (2, 3)]);
        let result = bfs(&1, &graph);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn bfs_branching() {
        // 1 -> 2, 1 -> 3, 2 -> 4, 3 -> 4
        let graph = make_graph(&[(1, 2), (1, 3), (2, 4), (3, 4)]);
        let result = bfs(&1, &graph);
        // BFS visits level by level: 1, then 2 and 3, then 4
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn post_order_linear() {
        // 1 -> 2 -> 3
        let graph = make_graph(&[(1, 2), (2, 3)]);
        let result = post_order(&1, &graph);
        assert_eq!(result, vec![3, 2, 1]);
    }

    #[test]
    fn post_order_branching() {
        // 1 -> 2, 1 -> 3, 2 -> 4
        let graph = make_graph(&[(1, 2), (1, 3), (2, 4)]);
        let result = post_order(&1, &graph);
        // Post-order: children before parents
        assert_eq!(result, vec![4, 2, 3, 1]);
    }

    #[test]
    fn tarjan_no_scc() {
        // DAG: 1 -> 2 -> 3
        let nodes: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        let graph = make_graph(&[(1, 2), (2, 3)]);
        let sccs = tarjan_scc(&nodes, &graph);
        // Each node is its own SCC, in reverse topo order
        assert_eq!(sccs.len(), 3);
        assert!(sccs.iter().all(|scc| scc.len() == 1));
    }

    #[test]
    fn tarjan_single_scc() {
        // Cycle: 1 -> 2 -> 3 -> 1
        let nodes: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        let graph = make_graph(&[(1, 2), (2, 3), (3, 1)]);
        let sccs = tarjan_scc(&nodes, &graph);
        // All nodes in one SCC
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0], nodes);
    }

    #[test]
    fn tarjan_multiple_sccs() {
        // Two SCCs: {1,2} and {3,4}
        // 1 <-> 2 -> 3 <-> 4
        let nodes: BTreeSet<i32> = [1, 2, 3, 4].into_iter().collect();
        let graph = make_graph(&[(1, 2), (2, 1), (2, 3), (3, 4), (4, 3)]);
        let sccs = tarjan_scc(&nodes, &graph);
        assert_eq!(sccs.len(), 2);
        // SCCs in reverse topo order: {3,4} before {1,2}
        let scc1: BTreeSet<i32> = [3, 4].into_iter().collect();
        let scc2: BTreeSet<i32> = [1, 2].into_iter().collect();
        assert_eq!(sccs[0], scc1);
        assert_eq!(sccs[1], scc2);
    }

    #[test]
    fn toposort_dag() {
        // DAG: 1 -> 2 -> 3, 1 -> 3
        let nodes: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        let graph = make_graph(&[(1, 2), (2, 3), (1, 3)]);
        let result = toposort(&nodes, &graph);
        assert!(result.is_some());
        let order = result.unwrap();
        // 1 must come before 2 and 3, 2 must come before 3
        let pos = |x: &i32| order.iter().position(|n| n == x).unwrap();
        assert!(pos(&1) < pos(&2));
        assert!(pos(&2) < pos(&3));
    }

    #[test]
    fn toposort_cycle_returns_none() {
        // Cycle: 1 -> 2 -> 3 -> 1
        let nodes: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        let graph = make_graph(&[(1, 2), (2, 3), (3, 1)]);
        let result = toposort(&nodes, &graph);
        assert!(result.is_none());
    }

    #[test]
    fn toposort_self_loop_returns_none() {
        // Self-loop: 1 -> 1
        let nodes: BTreeSet<i32> = [1].into_iter().collect();
        let graph = make_graph(&[(1, 1)]);
        let result = toposort(&nodes, &graph);
        assert!(result.is_none());
    }

    #[test]
    fn reachable_from_node() {
        // 1 -> 2 -> 3, 4 (disconnected)
        let graph = make_graph(&[(1, 2), (2, 3)]);
        let result = reachable(&1, &graph);
        let expected: BTreeSet<i32> = [1, 2, 3].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn reverse_post_order_gives_rpo() {
        // 1 -> 2 -> 3
        let graph = make_graph(&[(1, 2), (2, 3)]);
        let result = reverse_post_order(&1, &graph);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn algorithms_are_deterministic() {
        // Run each algorithm twice, verify same result
        let graph = make_graph(&[(1, 2), (1, 3), (2, 4), (3, 4), (4, 5)]);
        let nodes: BTreeSet<i32> = [1, 2, 3, 4, 5].into_iter().collect();

        let dfs1 = dfs(&1, &graph);
        let dfs2 = dfs(&1, &graph);
        assert_eq!(dfs1, dfs2);

        let bfs1 = bfs(&1, &graph);
        let bfs2 = bfs(&1, &graph);
        assert_eq!(bfs1, bfs2);

        let po1 = post_order(&1, &graph);
        let po2 = post_order(&1, &graph);
        assert_eq!(po1, po2);

        let scc1 = tarjan_scc(&nodes, &graph);
        let scc2 = tarjan_scc(&nodes, &graph);
        assert_eq!(scc1, scc2);
    }
}
