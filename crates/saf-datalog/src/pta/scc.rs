//! Offline SCC detection for copy-constraint graph preprocessing.
//!
//! Detects strongly connected components in the copy-edge graph and
//! produces a representative mapping. All values in an SCC are mapped
//! to the minimum `ValueId` in the component (deterministic per NFR-DET).
//!
//! Uses iterative Tarjan's algorithm to avoid stack overflow on large
//! programs with deep copy chains.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::btree_map::Entry;

use saf_core::ids::ValueId;

use crate::facts::PtaFacts;

/// Result of SCC detection.
#[derive(Debug, Clone)]
pub struct SccResult {
    /// Maps non-representative values to their SCC representative.
    /// Values not in any non-trivial SCC are absent.
    pub representatives: BTreeMap<ValueId, ValueId>,
    /// Number of non-trivial SCCs found (size >= 2).
    pub num_sccs: usize,
    /// Total number of values collapsed.
    pub collapsed_count: usize,
}

/// Detect SCCs in the copy-constraint graph.
///
/// Uses iterative Tarjan's algorithm. The representative for each SCC
/// is the minimum `ValueId` in the component (deterministic).
///
/// # Arguments
///
/// * `copy_edges` - Slice of `(dst, src)` copy-constraint pairs.
///   An edge from `src` to `dst` means "dst receives the points-to set of src."
///
/// # Panics
///
/// Panics if internal invariants of Tarjan's algorithm are violated
/// (e.g., a node's lowlink is missing). This should not occur with
/// well-formed input.
// NOTE: This function implements iterative Tarjan's algorithm as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
pub fn detect_scc(copy_edges: &[(ValueId, ValueId)]) -> SccResult {
    // Iterative Tarjan's uses an explicit call stack.
    // Each frame represents a node being visited, with a cursor into its neighbors.
    struct Frame {
        node: ValueId,
        neighbor_idx: usize,
    }

    // Build adjacency list from copy edges. Edge direction: src -> dst,
    // because copy(dst, src) means pts(dst) flows from src.
    let mut adj: BTreeMap<ValueId, Vec<ValueId>> = BTreeMap::new();
    let mut nodes: BTreeSet<ValueId> = BTreeSet::new();
    for &(dst, src) in copy_edges {
        adj.entry(src).or_default().push(dst);
        nodes.insert(src);
        nodes.insert(dst);
    }

    // Sort adjacency lists for determinism (BTreeMap keys are sorted,
    // but Vec values need explicit sorting).
    for neighbors in adj.values_mut() {
        neighbors.sort();
    }

    let empty_neighbors: Vec<ValueId> = Vec::new();
    let mut sccs: Vec<Vec<ValueId>> = Vec::new();

    // Tarjan's state
    let mut index_counter: u32 = 0;
    let mut indices: BTreeMap<ValueId, u32> = BTreeMap::new();
    let mut lowlinks: BTreeMap<ValueId, u32> = BTreeMap::new();
    let mut on_stack: BTreeSet<ValueId> = BTreeSet::new();
    let mut stack: Vec<ValueId> = Vec::new();

    for &start in &nodes {
        let Entry::Vacant(start_entry) = indices.entry(start) else {
            continue;
        };

        let mut call_stack: Vec<Frame> = Vec::new();

        // Initialize the start node
        start_entry.insert(index_counter);
        lowlinks.insert(start, index_counter);
        index_counter += 1;
        on_stack.insert(start);
        stack.push(start);

        call_stack.push(Frame {
            node: start,
            neighbor_idx: 0,
        });

        while let Some(frame) = call_stack.last_mut() {
            let v = frame.node;
            let neighbors = adj.get(&v).unwrap_or(&empty_neighbors);

            if frame.neighbor_idx < neighbors.len() {
                let w = neighbors[frame.neighbor_idx];
                frame.neighbor_idx += 1;

                if let Entry::Vacant(e) = indices.entry(w) {
                    // w not yet visited -- "recurse" into it
                    e.insert(index_counter);
                    lowlinks.insert(w, index_counter);
                    index_counter += 1;
                    on_stack.insert(w);
                    stack.push(w);

                    call_stack.push(Frame {
                        node: w,
                        neighbor_idx: 0,
                    });
                } else if on_stack.contains(&w) {
                    // w is on the stack -- update lowlink
                    let w_index = indices[&w];
                    let v_ll = lowlinks.get_mut(&v).expect("node must have lowlink");
                    if w_index < *v_ll {
                        *v_ll = w_index;
                    }
                }
            } else {
                // All neighbors processed -- this is the "return" from recursion.
                let v = frame.node;
                let v_lowlink = lowlinks[&v];
                let v_index = indices[&v];

                // Pop this frame before propagating lowlink to parent
                call_stack.pop();

                // Propagate lowlink to parent
                if let Some(parent_frame) = call_stack.last() {
                    let parent = parent_frame.node;
                    let parent_ll = lowlinks.get_mut(&parent).expect("parent must have lowlink");
                    if v_lowlink < *parent_ll {
                        *parent_ll = v_lowlink;
                    }
                }

                // If v is the root of an SCC, pop the SCC from the stack
                if v_lowlink == v_index {
                    let mut scc = Vec::new();
                    while let Some(w) = stack.pop() {
                        on_stack.remove(&w);
                        scc.push(w);
                        if w == v {
                            break;
                        }
                    }
                    if scc.len() >= 2 {
                        sccs.push(scc);
                    }
                }
            }
        }
    }

    // Build representative mapping: minimum `ValueId` per SCC
    let mut representatives = BTreeMap::new();
    let mut collapsed_count: usize = 0;
    for scc in &sccs {
        let rep = *scc.iter().min().expect("SCC is non-empty");
        for &v in scc {
            if v != rep {
                representatives.insert(v, rep);
                collapsed_count += 1;
            }
        }
    }

    SccResult {
        representatives,
        num_sccs: sccs.len(),
        collapsed_count,
    }
}

/// Rewrite PTA facts using SCC representatives.
///
/// Replaces every `ValueId` that appears in `reps` with its representative.
/// After rewriting, removes self-copies (where dst == src) since they are
/// no-ops after collapsing.
pub fn rewrite_facts_with_scc(facts: &mut PtaFacts, reps: &BTreeMap<ValueId, ValueId>) {
    if reps.is_empty() {
        return;
    }

    let lookup = |v: ValueId| -> ValueId { reps.get(&v).copied().unwrap_or(v) };

    // Rewrite addr_of: only the pointer side is a ValueId
    for (ptr, _loc) in &mut facts.addr_of {
        *ptr = lookup(*ptr);
    }

    // Rewrite copy edges
    for (dst, src) in &mut facts.copy {
        *dst = lookup(*dst);
        *src = lookup(*src);
    }
    // Remove self-copies after rewriting
    facts.copy.retain(|(dst, src)| dst != src);

    // Rewrite load edges
    for (dst, src) in &mut facts.load {
        *dst = lookup(*dst);
        *src = lookup(*src);
    }

    // Rewrite store edges
    for (dst, src) in &mut facts.store {
        *dst = lookup(*dst);
        *src = lookup(*src);
    }

    // Rewrite GEP edges
    for (dst, src, _path) in &mut facts.gep {
        *dst = lookup(*dst);
        *src = lookup(*src);
    }
}
