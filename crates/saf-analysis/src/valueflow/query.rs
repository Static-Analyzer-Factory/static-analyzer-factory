//! Query functions for value flow reachability and taint analysis.
//!
//! Implements BFS-based reachability queries with deterministic ordering.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use saf_core::ids::ValueId;

use super::ValueFlowGraph;
use super::node::NodeId;
use super::trace::{Trace, TraceStep};

/// Limits for query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLimits {
    /// Maximum path length (number of edges).
    pub max_depth: usize,
    /// Maximum number of paths to return.
    pub max_paths: usize,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_depth: 100,
            max_paths: 100,
        }
    }
}

impl QueryLimits {
    /// Create new query limits.
    #[must_use]
    pub const fn new(max_depth: usize, max_paths: usize) -> Self {
        Self {
            max_depth,
            max_paths,
        }
    }
}

/// A flow from source to sink.
#[derive(Debug, Clone)]
pub struct Flow {
    /// Source value ID.
    pub source: ValueId,
    /// Sink value ID.
    pub sink: ValueId,
    /// Trace from source to sink.
    pub trace: Trace,
}

impl ValueFlowGraph {
    /// Find all paths from sources to sinks.
    ///
    /// Uses BFS with deterministic edge ordering to ensure reproducible results.
    #[must_use]
    pub fn flows(
        &self,
        sources: &BTreeSet<ValueId>,
        sinks: &BTreeSet<ValueId>,
        limits: &QueryLimits,
    ) -> Vec<Flow> {
        self.bfs_flows(sources, sinks, &BTreeSet::new(), limits)
    }

    /// Find taint flows from sources to sinks, excluding paths through sanitizers.
    ///
    /// A sanitizer blocks the flow at that point - paths that go through
    /// sanitizers are not reported.
    #[must_use]
    pub fn taint_flow(
        &self,
        sources: &BTreeSet<ValueId>,
        sinks: &BTreeSet<ValueId>,
        sanitizers: &BTreeSet<ValueId>,
        limits: &QueryLimits,
    ) -> Vec<Flow> {
        self.bfs_flows(sources, sinks, sanitizers, limits)
    }

    /// Generic BFS reachability from sources to sinks with optional blocked nodes.
    ///
    /// This is the shared core for both `flows()` and `taint_flow()`. When
    /// `blocked_nodes` is empty, all paths are explored. When non-empty, any
    /// node in the blocked set (other than the source) terminates that path
    /// without reporting a flow.
    fn bfs_flows(
        &self,
        sources: &BTreeSet<ValueId>,
        sinks: &BTreeSet<ValueId>,
        blocked: &BTreeSet<ValueId>,
        limits: &QueryLimits,
    ) -> Vec<Flow> {
        let sink_nodes: BTreeSet<NodeId> = sinks.iter().map(|v| NodeId::value(*v)).collect();
        let blocked_nodes: BTreeSet<NodeId> = blocked.iter().map(|v| NodeId::value(*v)).collect();

        let mut results = Vec::new();

        // Process sources in sorted order for determinism
        for source in sources {
            let source_node = NodeId::value(*source);

            // Direct flow: source == sink (same ValueId used as both).
            // This happens when a call return value is passed directly as an
            // argument to another call (no intermediate store/load).
            // When blocked nodes are present, skip if the value is also blocked.
            if sinks.contains(source) && !blocked.contains(source) {
                results.push(Flow {
                    source: *source,
                    sink: *source,
                    trace: Trace::new(),
                });
                if results.len() >= limits.max_paths {
                    return results;
                }
                continue;
            }

            if !self.contains_node(source_node) {
                continue;
            }

            // BFS from this source
            let mut queue: VecDeque<(NodeId, Trace)> = VecDeque::new();
            let mut visited: BTreeMap<NodeId, usize> = BTreeMap::new();

            queue.push_back((source_node, Trace::new()));

            while let Some((node, trace)) = queue.pop_front() {
                // Check if this node is blocked (sanitizer) - stop this path
                if !blocked_nodes.is_empty() && blocked_nodes.contains(&node) && node != source_node
                {
                    continue;
                }

                // Check if we've found a sink
                if sink_nodes.contains(&node) && node != source_node {
                    if let Some(sink_value) = node.as_value() {
                        results.push(Flow {
                            source: *source,
                            sink: sink_value,
                            trace: trace.clone(),
                        });

                        if results.len() >= limits.max_paths {
                            return results;
                        }
                    }
                    continue;
                }

                // Check depth limit
                if trace.len() >= limits.max_depth {
                    continue;
                }

                // Check if we've visited this node at a shorter depth
                if let Some(&prev_depth) = visited.get(&node) {
                    if prev_depth <= trace.len() {
                        continue;
                    }
                }
                visited.insert(node, trace.len());

                // Expand to successors (deterministic order)
                if let Some(successors) = self.successors_of(node) {
                    for (edge, next) in successors {
                        let step = TraceStep::new(node, *edge, *next);
                        queue.push_back((*next, trace.with_step(step)));
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valueflow::EdgeKind;

    fn make_graph() -> ValueFlowGraph {
        let mut graph = ValueFlowGraph::new();

        // Create a simple flow: v1 -> v2 -> v3 -> v4
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let v3 = NodeId::value(ValueId::new(3));
        let v4 = NodeId::value(ValueId::new(4));

        graph.add_edge(v1, EdgeKind::DefUse, v2);
        graph.add_edge(v2, EdgeKind::Transform, v3);
        graph.add_edge(v3, EdgeKind::DefUse, v4);

        graph
    }

    #[test]
    fn flows_direct() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(2)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, ValueId::new(1));
        assert_eq!(results[0].sink, ValueId::new(2));
        assert_eq!(results[0].trace.len(), 1);
    }

    #[test]
    fn flows_multi_hop() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, ValueId::new(1));
        assert_eq!(results[0].sink, ValueId::new(4));
        assert_eq!(results[0].trace.len(), 3);
    }

    #[test]
    fn flows_no_path() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.flows(&sources, &sinks, &limits);
        assert!(results.is_empty());
    }

    #[test]
    fn flows_multiple_paths() {
        let mut graph = ValueFlowGraph::new();

        // v1 -> v2 -> v4
        // v1 -> v3 -> v4
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let v3 = NodeId::value(ValueId::new(3));
        let v4 = NodeId::value(ValueId::new(4));

        graph.add_edge(v1, EdgeKind::DefUse, v2);
        graph.add_edge(v1, EdgeKind::DefUse, v3);
        graph.add_edge(v2, EdgeKind::DefUse, v4);
        graph.add_edge(v3, EdgeKind::DefUse, v4);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn flows_max_depth_limit() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let limits = QueryLimits::new(2, 100); // Max depth 2

        let results = graph.flows(&sources, &sinks, &limits);
        assert!(results.is_empty()); // Path length is 3, exceeds limit
    }

    #[test]
    fn flows_max_paths_limit() {
        let mut graph = ValueFlowGraph::new();

        // Multiple paths from v1 to different sinks
        let v1 = NodeId::value(ValueId::new(1));
        for i in 2..=10 {
            let vi = NodeId::value(ValueId::new(i));
            graph.add_edge(v1, EdgeKind::DefUse, vi);
        }

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = (2..=10).map(ValueId::new).collect();
        let limits = QueryLimits::new(100, 3); // Max 3 paths

        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn flows_through_memory() {
        let mut graph = ValueFlowGraph::new();

        // v1 -> unknown_mem -> v2
        let v1 = NodeId::value(ValueId::new(1));
        let um = NodeId::unknown_mem();
        let v2 = NodeId::value(ValueId::new(2));

        graph.add_edge(v1, EdgeKind::Store, um);
        graph.add_edge(um, EdgeKind::Load, v2);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(2)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].trace.len(), 2);
    }

    #[test]
    fn flows_cycle_handling() {
        let mut graph = ValueFlowGraph::new();

        // v1 -> v2 -> v3 -> v2 (cycle), v3 -> v4
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let v3 = NodeId::value(ValueId::new(3));
        let v4 = NodeId::value(ValueId::new(4));

        graph.add_edge(v1, EdgeKind::DefUse, v2);
        graph.add_edge(v2, EdgeKind::DefUse, v3);
        graph.add_edge(v3, EdgeKind::DefUse, v2); // Back edge
        graph.add_edge(v3, EdgeKind::DefUse, v4);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn taint_basic() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let sanitizers: BTreeSet<_> = BTreeSet::new();
        let limits = QueryLimits::default();

        let results = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn taint_sanitizer_blocks() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let sanitizers: BTreeSet<_> = [ValueId::new(2)].into_iter().collect(); // Block at v2
        let limits = QueryLimits::default();

        let results = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        assert!(results.is_empty()); // Path blocked
    }

    #[test]
    fn taint_sanitizer_one_path() {
        let mut graph = ValueFlowGraph::new();

        // Two paths: v1 -> v2 -> v4 and v1 -> v3 -> v4
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let v3 = NodeId::value(ValueId::new(3));
        let v4 = NodeId::value(ValueId::new(4));

        graph.add_edge(v1, EdgeKind::DefUse, v2);
        graph.add_edge(v1, EdgeKind::DefUse, v3);
        graph.add_edge(v2, EdgeKind::DefUse, v4);
        graph.add_edge(v3, EdgeKind::DefUse, v4);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let sanitizers: BTreeSet<_> = [ValueId::new(2)].into_iter().collect(); // Block v2 path
        let limits = QueryLimits::default();

        let results = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        assert_eq!(results.len(), 1); // Only v3 path remains
    }

    #[test]
    fn taint_multiple_sources() {
        let mut graph = ValueFlowGraph::new();

        // v1 -> v3, v2 -> v3
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let v3 = NodeId::value(ValueId::new(3));

        graph.add_edge(v1, EdgeKind::DefUse, v3);
        graph.add_edge(v2, EdgeKind::DefUse, v3);

        let sources: BTreeSet<_> = [ValueId::new(1), ValueId::new(2)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(3)].into_iter().collect();
        let sanitizers: BTreeSet<_> = BTreeSet::new();
        let limits = QueryLimits::default();

        let results = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn flows_deterministic() {
        let graph = make_graph();

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(4)].into_iter().collect();
        let limits = QueryLimits::default();

        // Run twice and compare
        let results1 = graph.flows(&sources, &sinks, &limits);
        let results2 = graph.flows(&sources, &sinks, &limits);

        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.source, r2.source);
            assert_eq!(r1.sink, r2.sink);
            assert_eq!(r1.trace, r2.trace);
        }
    }

    #[test]
    fn query_limits_default() {
        let limits = QueryLimits::default();
        assert_eq!(limits.max_depth, 100);
        assert_eq!(limits.max_paths, 100);
    }

    #[test]
    fn flows_direct_source_equals_sink() {
        // When a source ValueId is also a sink (direct flow with no
        // intermediate steps), it should be reported as a zero-step flow.
        let mut graph = ValueFlowGraph::new();

        // v1 has an edge so it exists in the graph
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        graph.add_edge(v1, EdgeKind::CallArg, v2);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(1)].into_iter().collect(); // same as source

        let limits = QueryLimits::default();
        let results = graph.flows(&sources, &sinks, &limits);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, ValueId::new(1));
        assert_eq!(results[0].sink, ValueId::new(1));
        assert!(results[0].trace.is_empty());
    }

    #[test]
    fn taint_direct_source_equals_sink() {
        let mut graph = ValueFlowGraph::new();

        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        graph.add_edge(v1, EdgeKind::CallArg, v2);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sanitizers: BTreeSet<_> = BTreeSet::new();
        let limits = QueryLimits::default();

        let results = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, ValueId::new(1));
        assert_eq!(results[0].sink, ValueId::new(1));
        assert!(results[0].trace.is_empty());
    }

    #[test]
    fn taint_direct_source_equals_sink_sanitized() {
        // When source == sink but the value is also a sanitizer, no flow reported
        let mut graph = ValueFlowGraph::new();

        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        graph.add_edge(v1, EdgeKind::CallArg, v2);

        let sources: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sinks: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let sanitizers: BTreeSet<_> = [ValueId::new(1)].into_iter().collect();
        let limits = QueryLimits::default();

        let results = graph.taint_flow(&sources, &sinks, &sanitizers, &limits);
        assert!(results.is_empty());
    }
}
