# SVFG-Based Partial Leak Detection — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Detect partial memory leaks (allocation freed on some paths but not all) using SVF's three-phase algorithm: forward BFS, backward slice, Z3 guard tautology check.

**Architecture:** Replace the CFG-based `detect_partial_leaks` in `solver.rs` with an SVFG-based approach. Phase 1 enriches the existing forward BFS to return forward slice + reached sinks. Phase 2 builds a backward slice (intersection). Phase 3 propagates Z3 guard conditions through the backward slice and checks tautology at sinks.

**Tech Stack:** Rust, Z3 (via `z3` crate already in deps), SAF's SVFG/guard infrastructure

---

### Task 1: Add `forward_bfs_enriched` with unit tests

**Files:**
- Modify: `crates/saf-analysis/src/checkers/solver.rs`

**Step 1: Write failing tests for enriched forward BFS**

Add these tests after the existing `never_reach_sink` tests (after line ~1895):

```rust
// ---- forward_bfs_enriched tests ----

#[test]
fn forward_bfs_enriched_no_sink_is_neverfree() {
    // source -> mid -> end (no sinks)
    let svfg = make_linear_svfg();
    let spec = test_spec_never_reach_sink();
    let config = SolverConfig::default();

    let sources = vec![SvfgNodeId::value(ValueId::new(1))];
    let sinks = BTreeSet::new();

    let result = forward_bfs_enriched(&svfg, &spec, &sources, &sinks, &config);
    assert_eq!(result.neverfree_findings.len(), 1, "No sinks → NEVERFREE");
    assert!(result.reachable_sources.is_empty());
}

#[test]
fn forward_bfs_enriched_with_sink_is_reachable() {
    // source -> mid -> sink
    let svfg = make_linear_svfg();
    let spec = test_spec_never_reach_sink();
    let config = SolverConfig::default();

    let sources = vec![SvfgNodeId::value(ValueId::new(1))];
    let sinks = BTreeSet::from([SvfgNodeId::value(ValueId::new(3))]);

    let result = forward_bfs_enriched(&svfg, &spec, &sources, &sinks, &config);
    assert!(result.neverfree_findings.is_empty(), "Sink reachable → not NEVERFREE");
    assert_eq!(result.reachable_sources.len(), 1);

    let src = &result.reachable_sources[0];
    assert_eq!(src.source, SvfgNodeId::value(ValueId::new(1)));
    assert!(src.reached_sinks.contains(&SvfgNodeId::value(ValueId::new(3))));
    // forward_slice should contain source, mid, sink
    assert!(src.forward_slice.contains(&SvfgNodeId::value(ValueId::new(1))));
    assert!(src.forward_slice.contains(&SvfgNodeId::value(ValueId::new(2))));
    // sink is NOT in forward_slice (we stop at sinks, don't explore past)
}

#[test]
fn forward_bfs_enriched_finds_multiple_sinks() {
    // source -> sink1
    //       \-> mid -> sink2
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink1 = SvfgNodeId::value(ValueId::new(2));
    let mid = SvfgNodeId::value(ValueId::new(3));
    let sink2 = SvfgNodeId::value(ValueId::new(4));

    g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
    g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
    g.add_edge(mid, SvfgEdgeKind::DirectDef, sink2);

    let spec = test_spec_never_reach_sink();
    let config = SolverConfig::default();
    let sources = vec![source];
    let sinks = BTreeSet::from([sink1, sink2]);

    let result = forward_bfs_enriched(&g, &spec, &sources, &sinks, &config);
    assert!(result.neverfree_findings.is_empty());
    assert_eq!(result.reachable_sources.len(), 1);
    assert_eq!(result.reachable_sources[0].reached_sinks.len(), 2);
}

#[test]
fn forward_bfs_enriched_source_not_in_svfg() {
    let g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(42));

    let spec = test_spec_never_reach_sink();
    let config = SolverConfig::default();
    let sinks = BTreeSet::new();

    let result = forward_bfs_enriched(&g, &spec, &[source], &sinks, &config);
    assert_eq!(result.neverfree_findings.len(), 1, "Source not in SVFG → NEVERFREE");
}

#[test]
fn forward_bfs_enriched_source_is_sink() {
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    g.add_node(source);

    let spec = test_spec_never_reach_sink();
    let config = SolverConfig::default();
    let sinks = BTreeSet::from([source]);

    let result = forward_bfs_enriched(&g, &spec, &[source], &sinks, &config);
    assert!(result.neverfree_findings.is_empty());
    assert!(result.reachable_sources.is_empty(), "Source=sink → safe, no further analysis");
}
```

**Step 2: Run tests to verify they fail**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis forward_bfs_enriched 2>&1'`
Expected: compilation error — `forward_bfs_enriched` not defined

**Step 3: Implement data types and `forward_bfs_enriched`**

Add after the `SolverConfig` section (around line 80), before `may_reach`:

```rust
// ---------------------------------------------------------------------------
// Forward BFS result (enriched for partial leak detection)
// ---------------------------------------------------------------------------

/// Result of enriched forward BFS for partial leak detection.
///
/// Classifies each source into either NEVERFREE (no sink reachable) or
/// reachable (at least one sink found, with forward slice for Phase 2+3).
pub struct ForwardBfsResult {
    /// Sources that reached no sink — NEVERFREE findings.
    pub neverfree_findings: Vec<CheckerFinding>,
    /// Sources that reached at least one sink — candidates for partial leak check.
    pub reachable_sources: Vec<SourceReachability>,
}

/// A source that reached at least one sink during forward BFS.
pub struct SourceReachability {
    /// The source SVFG node.
    pub source: SvfgNodeId,
    /// All SVFG nodes visited during forward BFS (the forward slice).
    pub forward_slice: BTreeSet<SvfgNodeId>,
    /// All sink nodes reached during forward BFS.
    pub reached_sinks: BTreeSet<SvfgNodeId>,
}
```

Then add the function after the `never_reach_sink` section (after line ~854):

```rust
/// Enriched forward BFS: classifies sources into NEVERFREE vs reachable.
///
/// Like `never_reach_sink`, but instead of just reporting leaks, also returns
/// forward slice and reached sinks for sources that DO reach sinks. This data
/// feeds Phase 2 (backward slice) and Phase 3 (guard propagation) of SVF-style
/// partial leak detection.
pub fn forward_bfs_enriched(
    svfg: &Svfg,
    spec: &CheckerSpec,
    source_nodes: &[SvfgNodeId],
    sink_nodes: &BTreeSet<SvfgNodeId>,
    config: &SolverConfig,
) -> ForwardBfsResult {
    let mut neverfree_findings = Vec::new();
    let mut reachable_sources = Vec::new();
    let cfl_enabled = config.max_context_depth > 0;

    for &source in source_nodes {
        // Source is itself a sink → safe
        if sink_nodes.contains(&source) {
            continue;
        }

        // Source not in SVFG → definitive NEVERFREE
        if !svfg.contains_node(source) {
            neverfree_findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source,
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation never freed)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
            continue;
        }

        let mut forward_slice = BTreeSet::new();
        let mut reached_sinks = BTreeSet::new();
        let mut visited: BTreeSet<(SvfgNodeId, CallString)> = BTreeSet::new();
        let mut queue: VecDeque<(SvfgNodeId, usize, CallString)> = VecDeque::new();

        let empty_ctx = CallString::empty();
        queue.push_back((source, 0, empty_ctx.clone()));
        visited.insert((source, empty_ctx));
        forward_slice.insert(source);

        while let Some((node, depth, ctx)) = queue.pop_front() {
            if depth >= config.max_depth {
                continue;
            }

            if let Some(succs) = svfg.successors_of(node) {
                for (edge_kind, target) in succs {
                    let target = *target;

                    let new_ctx = if cfl_enabled {
                        match compute_cfl_context(&ctx, edge_kind, config.max_context_depth) {
                            Some(c) => c,
                            None => continue,
                        }
                    } else {
                        ctx.clone()
                    };

                    if !visited.insert((target, new_ctx.clone())) {
                        continue;
                    }

                    if sink_nodes.contains(&target) {
                        reached_sinks.insert(target);
                        // Don't explore past sinks (value consumed by deallocation)
                        continue;
                    }

                    forward_slice.insert(target);
                    queue.push_back((target, depth + 1, new_ctx));
                }
            }
        }

        if reached_sinks.is_empty() {
            // No sinks reachable → NEVERFREE
            neverfree_findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: source,
                sink_node: source,
                trace: vec![source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (allocation never freed)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        } else {
            reachable_sources.push(SourceReachability {
                source,
                forward_slice,
                reached_sinks,
            });
        }
    }

    // Deduplicate NEVERFREE findings
    neverfree_findings.sort_by_key(|f| f.source_node);
    neverfree_findings.dedup_by_key(|f| f.source_node);

    ForwardBfsResult {
        neverfree_findings,
        reachable_sources,
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis forward_bfs_enriched 2>&1'`
Expected: all 5 tests pass

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/checkers/solver.rs
git commit -m "feat: add forward_bfs_enriched for partial leak Phase 1"
```

---

### Task 2: Add `backward_slice` with unit tests

**Files:**
- Modify: `crates/saf-analysis/src/checkers/solver.rs`

**Step 1: Write failing tests for backward slice**

```rust
// ---- backward_slice tests ----

#[test]
fn backward_slice_linear() {
    // source -> mid -> sink
    let svfg = make_linear_svfg();
    let source = SvfgNodeId::value(ValueId::new(1));
    let mid = SvfgNodeId::value(ValueId::new(2));
    let sink = SvfgNodeId::value(ValueId::new(3));

    let forward = BTreeSet::from([source, mid]);
    let sinks = BTreeSet::from([sink]);

    let bslice = backward_slice(&svfg, &forward, &sinks);
    // backward from sink: sink -> mid -> source, all in forward_slice
    assert!(bslice.contains(&sink));
    assert!(bslice.contains(&mid));
    assert!(bslice.contains(&source));
}

#[test]
fn backward_slice_filters_non_forward() {
    // source -> A -> sink
    //           A -> B (B not on source->sink path if source only connects to A)
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let a = SvfgNodeId::value(ValueId::new(2));
    let sink = SvfgNodeId::value(ValueId::new(3));
    let b = SvfgNodeId::value(ValueId::new(4));

    g.add_edge(source, SvfgEdgeKind::DirectDef, a);
    g.add_edge(a, SvfgEdgeKind::DirectDef, sink);
    g.add_edge(a, SvfgEdgeKind::DirectDef, b);

    // forward_slice excludes b (it's reachable but not a sink, we include it)
    // Actually in Phase 1, b WOULD be in forward_slice. Let's test intersection:
    // If forward_slice only has {source, a}, backward from sink gives {sink, a, source}
    let forward = BTreeSet::from([source, a]);
    let sinks = BTreeSet::from([sink]);

    let bslice = backward_slice(&g, &forward, &sinks);
    assert!(bslice.contains(&sink));
    assert!(bslice.contains(&a));
    assert!(bslice.contains(&source));
    assert!(!bslice.contains(&b), "b not in forward_slice → excluded");
}

#[test]
fn backward_slice_branching() {
    // source -> sink1
    //       \-> mid -> sink2
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink1 = SvfgNodeId::value(ValueId::new(2));
    let mid = SvfgNodeId::value(ValueId::new(3));
    let sink2 = SvfgNodeId::value(ValueId::new(4));

    g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
    g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
    g.add_edge(mid, SvfgEdgeKind::DirectDef, sink2);

    let forward = BTreeSet::from([source, mid]);
    let sinks = BTreeSet::from([sink1, sink2]);

    let bslice = backward_slice(&g, &forward, &sinks);
    assert!(bslice.contains(&source));
    assert!(bslice.contains(&mid));
    assert!(bslice.contains(&sink1));
    assert!(bslice.contains(&sink2));
}
```

**Step 2: Run tests to verify they fail**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis backward_slice 2>&1'`
Expected: compilation error

**Step 3: Implement `backward_slice`**

Add after `forward_bfs_enriched`:

```rust
/// Build a backward slice from sinks, intersected with the forward slice.
///
/// Phase 2 of SVF-style partial leak detection. BFS backward on SVFG from
/// each reached sink, only including nodes also in the forward slice. The
/// result contains exactly the nodes on actual source-to-sink paths.
pub fn backward_slice(
    svfg: &Svfg,
    forward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
) -> BTreeSet<SvfgNodeId> {
    let mut slice = BTreeSet::new();
    let mut queue = VecDeque::new();

    for &sink in reached_sinks {
        slice.insert(sink);
        queue.push_back(sink);
    }

    while let Some(node) = queue.pop_front() {
        if let Some(preds) = svfg.predecessors_of(node) {
            for (_, pred) in preds {
                let pred = *pred;
                if forward_slice.contains(&pred) && slice.insert(pred) {
                    queue.push_back(pred);
                }
            }
        }
    }

    slice
}
```

**Step 4: Run tests to verify they pass**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis backward_slice 2>&1'`
Expected: all 3 tests pass

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/checkers/solver.rs
git commit -m "feat: add backward_slice for partial leak Phase 2"
```

---

### Task 3: Add `all_path_reachable_solve` with unit tests

**Files:**
- Modify: `crates/saf-analysis/src/checkers/solver.rs`

**Step 1: Write failing tests**

These tests use synthetic SVFGs with edge guards to test the Z3 tautology check.

```rust
// ---- all_path_reachable_solve tests ----

#[test]
fn all_path_reachable_linear_no_guards() {
    // source -> sink (no branching, no guards → all paths reach sink)
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink = SvfgNodeId::value(ValueId::new(2));
    g.add_edge(source, SvfgEdgeKind::DirectDef, sink);

    let backward = BTreeSet::from([source, sink]);
    let sinks = BTreeSet::from([sink]);

    let result = all_path_reachable_solve(
        &g, source, &backward, &sinks,
    );
    assert!(result, "Linear path with no guards → all paths reach sink");
}

#[test]
fn all_path_reachable_both_branches_reach_sink() {
    // source -> sink1 (guard: cond=true)
    //       \-> sink2 (guard: cond=false)
    // Both branches reach a sink → tautology
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink1 = SvfgNodeId::value(ValueId::new(2));
    let sink2 = SvfgNodeId::value(ValueId::new(3));
    let cond = ValueId::new(100);

    g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
    g.add_edge(source, SvfgEdgeKind::DirectDef, sink2);

    let block = saf_core::ids::BlockId::new(1);
    let func = saf_core::ids::FunctionId::new(1);
    g.set_edge_guard(source, sink1, vec![Guard {
        block, function: func, condition: cond, branch_taken: true,
    }]);
    g.set_edge_guard(source, sink2, vec![Guard {
        block, function: func, condition: cond, branch_taken: false,
    }]);

    let backward = BTreeSet::from([source, sink1, sink2]);
    let sinks = BTreeSet::from([sink1, sink2]);

    let result = all_path_reachable_solve(
        &g, source, &backward, &sinks,
    );
    assert!(result, "Both branches reach sink → all paths covered");
}

#[test]
fn all_path_reachable_one_branch_misses_sink() {
    // source -> sink  (guard: cond=true)
    //       \-> dead  (guard: cond=false, NOT a sink)
    // Only one branch reaches a sink → NOT all paths
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink = SvfgNodeId::value(ValueId::new(2));
    let dead = SvfgNodeId::value(ValueId::new(3));
    let cond = ValueId::new(100);

    g.add_edge(source, SvfgEdgeKind::DirectDef, sink);
    g.add_edge(source, SvfgEdgeKind::DirectDef, dead);

    let block = saf_core::ids::BlockId::new(1);
    let func = saf_core::ids::FunctionId::new(1);
    g.set_edge_guard(source, sink, vec![Guard {
        block, function: func, condition: cond, branch_taken: true,
    }]);
    g.set_edge_guard(source, dead, vec![Guard {
        block, function: func, condition: cond, branch_taken: false,
    }]);

    // backward slice only includes source->sink path (dead is NOT a sink)
    let backward = BTreeSet::from([source, sink]);
    let sinks = BTreeSet::from([sink]);

    let result = all_path_reachable_solve(
        &g, source, &backward, &sinks,
    );
    assert!(!result, "One branch misses sink → partial leak");
}

#[test]
fn all_path_reachable_no_guards_means_true() {
    // source -> mid -> sink  (no guards on any edge)
    // No branching info → conservatively all paths = TRUE
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let mid = SvfgNodeId::value(ValueId::new(2));
    let sink = SvfgNodeId::value(ValueId::new(3));
    g.add_edge(source, SvfgEdgeKind::DirectDef, mid);
    g.add_edge(mid, SvfgEdgeKind::DirectDef, sink);

    let backward = BTreeSet::from([source, mid, sink]);
    let sinks = BTreeSet::from([sink]);

    let result = all_path_reachable_solve(
        &g, source, &backward, &sinks,
    );
    assert!(result, "No guards → condition is TRUE → tautology");
}
```

**Step 2: Run tests to verify they fail**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis all_path_reachable 2>&1'`
Expected: compilation error

**Step 3: Implement `all_path_reachable_solve`**

Add after `backward_slice`:

```rust
/// Check whether ALL paths from source reach a sink (Z3 tautology check).
///
/// Phase 3 of SVF-style partial leak detection. Forward-propagates Z3
/// boolean conditions through backward-slice nodes. At sinks, collects the
/// disjunction of all conditions. If `NOT(disjunction)` is UNSAT, the
/// disjunction is a tautology — all paths are covered.
///
/// Returns `true` if all paths reach a sink (safe), `false` if partial leak.
pub fn all_path_reachable_solve(
    svfg: &Svfg,
    source: SvfgNodeId,
    backward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
) -> bool {
    use std::collections::BTreeMap;

    // Map: branch condition ValueId → Z3 Bool variable name
    let mut cond_counter: u32 = 0;
    let mut cond_names: BTreeMap<ValueId, String> = BTreeMap::new();

    // Map: SvfgNodeId → Z3 Bool expression (SMT-LIB string representation)
    // We build Z3 expressions as z3::ast::Bool directly.
    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    params.set_u32("timeout", 5000); // 5s timeout per source
    solver.set_params(&params);

    // Map: node → accumulated Z3 condition
    let mut node_conds: BTreeMap<SvfgNodeId, z3::ast::Bool> = BTreeMap::new();

    // Source condition = TRUE
    node_conds.insert(source, z3::ast::Bool::from_bool(true));

    // Worklist-driven forward propagation
    let mut worklist: VecDeque<SvfgNodeId> = VecDeque::new();
    let mut in_worklist: BTreeSet<SvfgNodeId> = BTreeSet::new();
    worklist.push_back(source);
    in_worklist.insert(source);

    // Iteration cap to ensure termination with cycles
    let max_iterations = backward_slice.len() * 3;
    let mut iterations = 0;

    while let Some(node) = worklist.pop_front() {
        in_worklist.remove(&node);
        iterations += 1;
        if iterations > max_iterations {
            // Conservative: if we can't converge, assume not all paths covered
            return false;
        }

        let cur_cond = match node_conds.get(&node) {
            Some(c) => c.clone(),
            None => continue,
        };

        let Some(succs) = svfg.successors_of(node) else {
            continue;
        };

        for (_, succ) in succs {
            let succ = *succ;
            if !backward_slice.contains(&succ) {
                continue;
            }

            // Compute edge guard
            let edge_guard = compute_edge_guard_z3(
                svfg, node, succ, &mut cond_names, &mut cond_counter,
            );

            // propagated = cur_cond AND edge_guard
            let propagated = z3::ast::Bool::and(&[&cur_cond, &edge_guard]);

            // succ_cond = existing OR propagated
            let new_cond = match node_conds.get(&succ) {
                Some(existing) => z3::ast::Bool::or(&[existing, &propagated]),
                None => propagated,
            };

            node_conds.insert(succ, new_cond);

            if !in_worklist.contains(&succ) {
                worklist.push_back(succ);
                in_worklist.insert(succ);
            }
        }
    }

    // Collect disjunction of conditions at all sinks
    let sink_conds: Vec<z3::ast::Bool> = reached_sinks
        .iter()
        .filter_map(|s| node_conds.get(s))
        .cloned()
        .collect();

    if sink_conds.is_empty() {
        return false; // No conditions at sinks → not all paths covered
    }

    let refs: Vec<&z3::ast::Bool> = sink_conds.iter().collect();
    let final_cond = z3::ast::Bool::or(&refs);

    // Tautology check: is NOT(final_cond) UNSAT?
    solver.assert(&final_cond.not());
    matches!(solver.check(), z3::SatResult::Unsat)
}

/// Compute the Z3 guard for an SVFG edge.
///
/// If the SVFG has pre-computed guards for this edge, translates them to Z3.
/// Otherwise, returns TRUE (unconditional edge).
fn compute_edge_guard_z3(
    svfg: &Svfg,
    from: SvfgNodeId,
    to: SvfgNodeId,
    cond_names: &mut BTreeMap<ValueId, String>,
    cond_counter: &mut u32,
) -> z3::ast::Bool {
    let guards = match svfg.edge_guard(from, to) {
        Some(g) if !g.is_empty() => g,
        _ => return z3::ast::Bool::from_bool(true), // No guard → unconditional
    };

    // Translate each guard to Z3 and AND them together
    let mut exprs: Vec<z3::ast::Bool> = Vec::new();
    for guard in guards {
        let var_name = cond_names
            .entry(guard.condition)
            .or_insert_with(|| {
                let name = format!("guard_{}", *cond_counter);
                *cond_counter += 1;
                name
            })
            .clone();

        let var = z3::ast::Bool::new_const(var_name.as_str());
        let expr = if guard.branch_taken { var } else { var.not() };
        exprs.push(expr);
    }

    if exprs.len() == 1 {
        exprs.into_iter().next().unwrap()
    } else {
        let refs: Vec<&z3::ast::Bool> = exprs.iter().collect();
        z3::ast::Bool::and(&refs)
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis all_path_reachable 2>&1'`
Expected: all 4 tests pass

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/checkers/solver.rs
git commit -m "feat: add all_path_reachable_solve for partial leak Phase 3"
```

---

### Task 4: Add `detect_partial_leaks_svfg` orchestrator

**Files:**
- Modify: `crates/saf-analysis/src/checkers/solver.rs`

**Step 1: Write failing test**

```rust
#[test]
fn detect_partial_leaks_svfg_reports_partial() {
    // source -> sink  (guard: cond=true)
    //       \-> dead  (guard: cond=false)
    // Phase 1: sink reachable → isSomePathReachable = true
    // Phase 2: backward slice = {source, sink}
    // Phase 3: condition at sink = cond, NOT(cond) is SAT → not tautology
    // → PARTIALLEAK
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink = SvfgNodeId::value(ValueId::new(2));
    let dead = SvfgNodeId::value(ValueId::new(3));
    let cond = ValueId::new(100);

    g.add_edge(source, SvfgEdgeKind::DirectDef, sink);
    g.add_edge(source, SvfgEdgeKind::DirectDef, dead);

    let block = saf_core::ids::BlockId::new(1);
    let func_id = saf_core::ids::FunctionId::new(1);
    g.set_edge_guard(source, sink, vec![Guard {
        block, function: func_id, condition: cond, branch_taken: true,
    }]);
    g.set_edge_guard(source, dead, vec![Guard {
        block, function: func_id, condition: cond, branch_taken: false,
    }]);

    let spec = test_spec_never_reach_sink();
    let sink_nodes = BTreeSet::from([sink]);

    let reachable = vec![SourceReachability {
        source,
        forward_slice: BTreeSet::from([source, dead]),
        reached_sinks: BTreeSet::from([sink]),
    }];

    let findings = detect_partial_leaks_svfg(&g, &spec, &reachable, &sink_nodes);
    assert_eq!(findings.len(), 1, "Should detect partial leak");
    assert!(findings[0].message.contains("partial leak"));
}

#[test]
fn detect_partial_leaks_svfg_no_report_when_all_paths() {
    // source -> sink1 (guard: cond=true)
    //       \-> sink2 (guard: cond=false)
    // Both branches freed → safe
    let mut g = Svfg::new();
    let source = SvfgNodeId::value(ValueId::new(1));
    let sink1 = SvfgNodeId::value(ValueId::new(2));
    let sink2 = SvfgNodeId::value(ValueId::new(3));
    let cond = ValueId::new(100);

    g.add_edge(source, SvfgEdgeKind::DirectDef, sink1);
    g.add_edge(source, SvfgEdgeKind::DirectDef, sink2);

    let block = saf_core::ids::BlockId::new(1);
    let func_id = saf_core::ids::FunctionId::new(1);
    g.set_edge_guard(source, sink1, vec![Guard {
        block, function: func_id, condition: cond, branch_taken: true,
    }]);
    g.set_edge_guard(source, sink2, vec![Guard {
        block, function: func_id, condition: cond, branch_taken: false,
    }]);

    let spec = test_spec_never_reach_sink();
    let sink_nodes = BTreeSet::from([sink1, sink2]);

    let reachable = vec![SourceReachability {
        source,
        forward_slice: BTreeSet::from([source]),
        reached_sinks: BTreeSet::from([sink1, sink2]),
    }];

    let findings = detect_partial_leaks_svfg(&g, &spec, &reachable, &sink_nodes);
    assert!(findings.is_empty(), "All paths reach sinks → safe");
}
```

**Step 2: Run tests to verify they fail**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis detect_partial_leaks_svfg 2>&1'`
Expected: compilation error

**Step 3: Implement `detect_partial_leaks_svfg`**

```rust
/// Detect partial leaks using SVFG-based three-phase analysis.
///
/// For each source that reached at least one sink (Phase 1 result), builds
/// a backward slice (Phase 2) and checks all-path reachability via Z3 guard
/// propagation (Phase 3). Reports a partial leak if `isAllPathReachable` is
/// false.
pub fn detect_partial_leaks_svfg(
    svfg: &Svfg,
    spec: &CheckerSpec,
    reachable_sources: &[SourceReachability],
    _sink_nodes: &BTreeSet<SvfgNodeId>,
) -> Vec<CheckerFinding> {
    let mut findings = Vec::new();

    for src in reachable_sources {
        // Phase 2: backward slice
        let bslice = backward_slice(svfg, &src.forward_slice, &src.reached_sinks);

        // Phase 3: Z3 tautology check
        let all_path = all_path_reachable_solve(
            svfg, src.source, &bslice, &src.reached_sinks,
        );

        if !all_path {
            findings.push(CheckerFinding {
                checker_name: spec.name.clone(),
                severity: spec.severity,
                source_node: src.source,
                sink_node: src.source,
                trace: vec![src.source],
                cwe: spec.cwe,
                message: format!(
                    "{}: {} (partial leak: allocation freed on some paths but not all)",
                    spec.name, spec.description
                ),
                sink_traces: vec![],
                source_kind: super::finding::NullSourceKind::default(),
            });
        }
    }

    findings.sort_by_key(|f| f.source_node);
    findings.dedup_by_key(|f| f.source_node);
    findings
}
```

**Step 4: Run tests to verify they pass**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis detect_partial_leaks_svfg 2>&1'`
Expected: both tests pass

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/checkers/solver.rs
git commit -m "feat: add detect_partial_leaks_svfg orchestrator"
```

---

### Task 5: Integrate into runner and remove old code

**Files:**
- Modify: `crates/saf-analysis/src/checkers/runner.rs:232-251`
- Modify: `crates/saf-analysis/src/checkers/solver.rs` (remove old helpers)

**Step 1: Update runner.rs to use new functions**

Replace the `NeverReachSink` arm (lines 232-251) with:

```rust
ReachabilityMode::NeverReachSink => {
    let filtered_sources =
        filter_wrapper_internal_sources(&source_nodes, &classified, module, table);
    // Phase 1: Forward BFS — classifies into NEVERFREE vs reachable
    let fwd_result =
        solver::forward_bfs_enriched(svfg, spec, &filtered_sources, &sink_nodes, config);
    let mut findings = fwd_result.neverfree_findings;
    // Phase 2+3: For reachable sources, check partial leaks
    let partial = solver::detect_partial_leaks_svfg(
        svfg,
        spec,
        &fwd_result.reachable_sources,
        &sink_nodes,
    );
    findings.extend(partial);
    findings
}
```

**Step 2: Remove old functions from solver.rs**

Remove these functions (they are no longer called):
- `detect_partial_leaks` (lines ~1158-1261)
- `svfg_bfs_find_sanitizers` (lines ~869-906)
- `SvfgBfsResult` struct (lines ~861-866)
- `find_value_location` (lines ~912-926)
- `find_sanitizer_blocks_in_function` (lines ~945-1052)
- `is_known_deallocator` (lines ~1055-1070)
- `find_exit_block_indices` (lines ~1073-1085)
- `can_reach_exit_bypassing_blocks` (lines ~1090-1140)

Keep `never_reach_sink` — it may still be useful for non-memory-leak checkers or testing.

**Step 3: Run all existing tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis 2>&1' | tee /tmp/test-output.txt`
Expected: all tests pass. Check that existing `never_reach_sink_*` tests still pass.

**Step 4: Run format and lint**

Run: `docker compose run --rm dev sh -c 'cargo fmt -p saf-analysis && cargo clippy -p saf-analysis -- -D warnings 2>&1'`
Expected: no errors

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/checkers/runner.rs crates/saf-analysis/src/checkers/solver.rs
git commit -m "feat: integrate SVFG partial leak detection, remove old CFG-based approach"
```

---

### Task 6: Run Juliet CWE401 benchmark and validate

**Files:** None (validation only)

**Step 1: Run the Juliet CWE401 benchmark**

Run: `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet --cwe CWE401 -o /workspace/tests/benchmarks/sv-benchmarks/cwe401-results.json 2>&1' | tee /tmp/juliet-output.txt`

**Step 2: Check results**

Read `tests/benchmarks/sv-benchmarks/cwe401-results.json` and compare:
- Previous: TP=582, FP=134, FN=104
- Target: FN reduction by ~90 (malloc_realloc cases)
- Verify FP does not increase

**Step 3: If FN improved, commit results**

```bash
git add tests/benchmarks/sv-benchmarks/cwe401-results.json
git commit -m "bench: CWE401 results after SVFG partial leak detection"
```

**Step 4: If FN not improved, debug**

The most likely issue: SVFG edge guards may not be pre-computed for the intra-callee edges
(inside `ldv_reference_realloc`). In that case, `compute_edge_guard_z3` returns TRUE for
all edges and the tautology check incorrectly says "all paths covered."

Debugging steps:
1. Add `eprintln!` in `all_path_reachable_solve` to print guard counts per edge
2. Check if `svfg.edge_guard(from, to)` returns guards for edges inside `ldv_reference_realloc`
3. If no guards: need to add on-the-fly CFG-based guard computation (Task 7)

---

### Task 7 (Contingency): On-the-fly CFG-based guard computation

**Only needed if Task 6 shows missing SVFG edge guards.**

**Files:**
- Modify: `crates/saf-analysis/src/checkers/solver.rs`

**Step 1: Enhance `compute_edge_guard_z3` with CFG fallback**

When `svfg.edge_guard(from, to)` returns no guards, compute guards from the CFG:

```rust
fn compute_edge_guard_z3(
    svfg: &Svfg,
    from: SvfgNodeId,
    to: SvfgNodeId,
    cond_names: &mut BTreeMap<ValueId, String>,
    cond_counter: &mut u32,
    value_index: &ValueLocationIndex, // NEW parameter
) -> z3::ast::Bool {
    // Try pre-computed guards first
    if let Some(guards) = svfg.edge_guard(from, to) {
        if !guards.is_empty() {
            return translate_guards_to_z3(guards, cond_names, cond_counter);
        }
    }

    // Fallback: compute from CFG
    let (SvfgNodeId::Value(from_vid), SvfgNodeId::Value(to_vid)) = (from, to) else {
        return z3::ast::Bool::from_bool(true);
    };

    let Some((_, from_block)) = value_index.block_of(from_vid) else {
        return z3::ast::Bool::from_bool(true);
    };
    let Some((_, to_block)) = value_index.block_of(to_vid) else {
        return z3::ast::Bool::from_bool(true);
    };

    // Same block → unconditional
    if from_block == to_block {
        return z3::ast::Bool::from_bool(true);
    }

    // Check if from_block has a CondBr terminator targeting to_block
    if let Some(TerminatorInfo::CondBr { condition, then_target, else_target }) =
        value_index.terminator_of(from_block)
    {
        let var_name = cond_names
            .entry(*condition)
            .or_insert_with(|| {
                let name = format!("guard_{}", *cond_counter);
                *cond_counter += 1;
                name
            })
            .clone();
        let var = z3::ast::Bool::new_const(var_name.as_str());

        if to_block == *then_target {
            return var;
        } else if to_block == *else_target {
            return var.not();
        }
    }

    // No guard info available → TRUE (unconditional)
    z3::ast::Bool::from_bool(true)
}
```

This requires updating `all_path_reachable_solve` to accept and pass `value_index`.

**Step 2: Update `all_path_reachable_solve` signature**

Add `value_index: &ValueLocationIndex` parameter. Update all call sites.

**Step 3: Update `detect_partial_leaks_svfg` to build and pass `ValueLocationIndex`**

```rust
pub fn detect_partial_leaks_svfg(
    svfg: &Svfg,
    spec: &CheckerSpec,
    reachable_sources: &[SourceReachability],
    _sink_nodes: &BTreeSet<SvfgNodeId>,
    module: &AirModule,  // NEW parameter
) -> Vec<CheckerFinding> {
    let value_index = crate::guard::ValueLocationIndex::build(module);
    // ... pass value_index to all_path_reachable_solve
}
```

Update `runner.rs` call to pass `module`.

**Step 4: Re-run Juliet benchmark**

Same command as Task 6 Step 1. Verify FN reduction.

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/checkers/solver.rs crates/saf-analysis/src/checkers/runner.rs
git commit -m "feat: add on-the-fly CFG guard computation for partial leak detection"
```
