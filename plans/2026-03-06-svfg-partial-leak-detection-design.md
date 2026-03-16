# SVFG-Based Partial Leak Detection

Date: 2026-03-06

## Problem

SAF produces 104 FN vs SVF's 20 FN on CWE401 (memory leak) in the Juliet benchmark.
90 FN come from `malloc_realloc` patterns where `realloc` conditionally frees the old
pointer. SAF's `never_reach_sink` finds that the old pointer reaches `free` inside the
realloc wrapper and marks it "safe", but the free is conditional (only on realloc success).
The existing `detect_partial_leaks` (CFG-based bypass check) fails because it treats the
entire call-site block as an unconditional sanitizer.

## Design: SVF's Three-Phase Algorithm

Replaces the CFG-based `detect_partial_leaks` with an SVFG-based approach matching SVF's
`SrcSnkDDA` architecture.

### Classification

| `isSomePathReachable` | `isAllPathReachable` | Result |
|---|---|---|
| false | false | NEVERFREE (existing `never_reach_sink`) |
| true | true | SAFE |
| true | false | **PARTIALLEAK (new)** |

### Phase 1: Forward BFS with Enriched Results

Modify the existing forward BFS to return richer data instead of just `reached_any_sink: bool`.

**Return type:**

```rust
struct ForwardBfsResult {
    neverfree_findings: Vec<CheckerFinding>,
    reachable_sources: Vec<SourceReachability>,
}

struct SourceReachability {
    source: SvfgNodeId,
    forward_slice: BTreeSet<SvfgNodeId>,
    reached_sinks: BTreeSet<SvfgNodeId>,
}
```

**Changes from current `never_reach_sink`:**
- Collect `forward_slice` (all visited nodes) and `reached_sinks` (all sinks found)
- Do NOT break on first sink — continue BFS to find ALL reachable sinks and build the
  complete forward slice
- When a sink is found, add to `reached_sinks` but don't explore past it (value consumed
  by deallocation)
- Context sensitivity (CFL-aware BFS with `CallString`) stays the same
- Forward slice stores `SvfgNodeId` without context (Phase 2+3 work at node level)

### Phase 2: Backward Slice Construction

For each source with reached sinks, BFS backward on SVFG from each sink using
`predecessors_of`. Only include nodes also in the forward slice (intersection).

```rust
fn backward_slice(
    svfg: &Svfg,
    forward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
) -> BTreeSet<SvfgNodeId>
```

**Properties:**
- backward_slice is a subset of forward_slice (by construction)
- Source node is in backward_slice (reachable backward from sinks)
- No context sensitivity in backward BFS (same as SVF)

### Phase 3: Guard Propagation + Z3 Tautology Check

Forward-propagate Z3 conditions through only the backward-slice nodes. At sinks, check
if the disjunction of all sink conditions is a tautology.

```rust
fn all_path_reachable_solve(
    svfg: &Svfg,
    source: SvfgNodeId,
    backward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &BTreeSet<SvfgNodeId>,
    module: &AirModule,
    value_index: &ValueLocationIndex,
) -> bool
```

**Algorithm:**
1. Set `node_conds[source] = TRUE`
2. Worklist-driven forward propagation through backward slice:
   - For each SVFG edge `(node, succ)` where `succ` is in backward slice:
   - Compute edge guard (Z3 boolean formula)
   - `propagated = cur_cond AND edge_guard`
   - `succ_cond = succ_cond OR propagated`
3. At sinks: `final = OR(cond @ all sinks)`
4. Tautology check: `NOT(final)` UNSAT via Z3 → all paths covered → return true

**Guard computation (`compute_edge_guard`):**
1. Check pre-computed SVFG edge guards (`svfg.edge_guard(node, succ)`) first
2. Fallback: on-the-fly CFG-based guards:
   - Map nodes to basic blocks via `ValueLocationIndex`
   - Same block → TRUE
   - `dstBB` post-dominates `srcBB` → TRUE
   - Otherwise → extract `CondBr` branch condition, create Z3 `Bool` variable
3. Inter-procedural (CallArg/Return): combine caller-side and callee-side guards

**Z3 usage:**
- Single `z3::Solver` per source
- Each unique branch condition (`ValueId`) → fresh `z3::ast::Bool` (cached in `BTreeMap`)
- `z3_and`, `z3_or`, `z3_not` → standard Z3 Bool operations
- Tautology: `solver.assert(NOT(final)); solver.check() == Unsat`

**Convergence:**
- Worklist re-adds nodes when condition updated via OR
- Termination ensured by Z3 structural comparison or iteration cap for cycles

### Integration

**runner.rs changes:**

```rust
ReachabilityMode::NeverReachSink => {
    let filtered_sources = filter_wrapper_internal_sources(...);
    let fwd_result = solver::forward_bfs_enriched(
        svfg, spec, &filtered_sources, &sink_nodes, config
    );
    let mut findings = fwd_result.neverfree_findings;
    let partial = solver::detect_partial_leaks_svfg(
        svfg, spec, &fwd_result.reachable_sources, &sink_nodes, module
    );
    findings.extend(partial);
    findings
}
```

**Removed:**
- `detect_partial_leaks` (old CFG-based bypass)
- `svfg_bfs_find_sanitizers`, `find_sanitizer_blocks_in_function`,
  `can_reach_exit_bypassing_blocks`, `find_exit_block_indices`, `is_known_deallocator`

**Unchanged:**
- All other solver functions (`may_reach`, `may_reach_guarded`, etc.)
- Runner dispatch structure
- Checker specs

### Testing

- Run Juliet CWE401 benchmark: target 90 FN reduction (malloc_realloc cases)
- Verify no FP increase
- Existing tests must continue passing

## Files Changed

- `crates/saf-analysis/src/checkers/solver.rs` — new functions, remove old helpers
- `crates/saf-analysis/src/checkers/runner.rs` — updated NeverReachSink dispatch
- `crates/saf-analysis/src/guard.rs` or `z3_utils/solver.rs` — Z3 tautology helper
