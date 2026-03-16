# Plan 178: Memory-Leak Checker — NeverReachSink + AllPathReachable

## Status

- Phase 1 (NeverReachSink): **done**
- Phase 2 (AllPathReachable): **planned**

---

## Phase 1: Switch to NeverReachSink (DONE)

### Problem

SAF's memory-leak checker used a `MustNotReach` formulation: forward BFS from
`malloc`, report if any path reaches function exit without passing through
`free`. This has a structural false-negative problem on dead-end flows.

### Solution

Switched to `NeverReachSink` mode (`NEVERFREE` formulation): forward BFS from
`malloc`, check if ANY `free` is reachable. No free found = leak.

### Changes

| File | Change |
|------|--------|
| `spec.rs` | `memory_leak()`: mode → `NeverReachSink`, sinks → `Deallocator`, removed sanitizers |
| `solver.rs` | Added `never_reach_sink()` solver with source-not-in-SVFG defense-in-depth |
| `runner.rs` | Added `NeverReachSink` dispatch arm |
| `builder.rs` | Added standalone SVFG nodes for `HeapAlloc` results and `CallDirect` declaration results |
| `saf-python` | Added `NeverReachSink` to Python export and parse |

### Benchmark Results: Neutral

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| CWE401 TP | 670 | 670 | 0 |
| CWE401 FP | 529 | 529 | 0 |
| CWE401 FN | 16 | 16 | 0 |
| PTABen Exact | 101 | 101 | 0 |

### Remaining Limitation

`NeverReachSink` cannot detect **partial leaks** — conditional frees where
`free` is reachable on some paths but not all. This accounts for 4 of the
16 CWE401 FNs (variant 12's `if(cond) free(p)` pattern). Phase 2 adds
an `AllPathReachable` analysis to detect these.

---

## Phase 2: AllPathReachable — Partial Leak Detection (PLANNED)

### Problem

For programs like:
```c
void *p = malloc(10);
if (cond) free(p);
// p leaks on !cond path
```

`NeverReachSink` BFS from `malloc` finds `free` → reports "no leak." But `free`
is only on one branch — the allocation leaks when `!cond`. SAF currently has
no way to detect this.

### Approach: Three-Phase Algorithm

The idea is to augment `NeverReachSink` with a guard-based all-path check.
When forward BFS finds a sink (free), a second pass determines whether `free`
is reachable on **all** paths or only some.

**Phase 1 — Forward BFS** (existing `never_reach_sink`):
Context-sensitive BFS on SVFG from each source. If any sink is reached,
`somePathReachable = true`. If none reached, report `NEVERFREE`.

**Phase 2 — Backward BFS** (new):
From each reached sink, backward BFS builds the backward slice. The
intersection of forward and backward slices is the actual value-flow path.

**Phase 3 — Guard computation** (new):
Propagates CFG-derived **branch conditions** along the SVFG edges using Z3.
For each SVFG edge `(node → succ)` in the backward slice:
- Compute `ComputeIntraVFGGuard(nodeBB, succBB)` — walks the CFG between
  the two basic blocks, accumulating Z3 symbolic branch conditions
- Propagate: `succCond = succCond OR (nodeCond AND guard)`

Then ask Z3: is the disjunction of all sink conditions equivalent to TRUE?
If not → `PARTIALLEAK`.

**Classification:**
```
somePathReachable=false                        → NEVERFREE
somePathReachable=true, allPathReachable=false  → PARTIALLEAK
somePathReachable=true, allPathReachable=true   → SAFE
```

### Implementation

#### Architecture

Extend `never_reach_sink` with a post-BFS all-path-reachability check. The
existing SAF infrastructure already provides: Z3 solver (`z3_utils/solver.rs`),
guard extraction (`guard.rs`), dominator trees (`z3_utils/dominator.rs`), and
CFG-based path reachability (`z3_utils/reachability.rs`).

```
never_reach_sink() — Phase 1: forward BFS on SVFG
    │
    ├── No sink found → NEVERFREE (existing behavior, no change)
    │
    └── Sink(s) found → Phase 2: AllPathReachable check
            │
            ├── Build backward slice from reached sinks
            ├── Compute VFG guards (CFG branch conditions along SVFG edges)
            ├── Ask Z3: disjunction of sink guards ≡ TRUE?
            │
            ├── YES → SAFE (all paths reach free)
            └── NO  → PARTIALLEAK (report finding)
```

#### Step 1: Build `node_to_block` Map

The guard computation needs to know which basic block each SVFG node belongs to.
Extend the existing `node_to_func` map with block-level granularity.

**File:** `crates/saf-analysis/src/checkers/runner.rs`

```rust
fn build_node_to_block_map(
    module: &AirModule,
) -> BTreeMap<ValueId, (FunctionId, BlockId)> {
    let mut map = BTreeMap::new();
    for func in &module.functions {
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    map.insert(dst, (func.id, block.id));
                }
                for &op in &inst.operands {
                    map.entry(op).or_insert((func.id, block.id));
                }
            }
        }
    }
    map
}
```

#### Step 2: Backward Slice Construction

After forward BFS finds sinks, backward BFS from each reached sink collects
the backward slice (nodes reachable backward from any sink within the forward
slice).

**File:** `crates/saf-analysis/src/checkers/solver.rs`

```rust
fn backward_slice(
    svfg: &Svfg,
    forward_slice: &BTreeSet<SvfgNodeId>,
    reached_sinks: &[SvfgNodeId],
) -> BTreeSet<SvfgNodeId> {
    let mut backward = BTreeSet::new();
    let mut queue: VecDeque<SvfgNodeId> = reached_sinks.iter().copied().collect();

    while let Some(node) = queue.pop_front() {
        if !backward.insert(node) { continue; }
        // Walk predecessors, only within forward slice
        if let Some(preds) = svfg.predecessors_of(node) {
            for (_, pred) in preds {
                if forward_slice.contains(pred) && !backward.contains(pred) {
                    queue.push_back(*pred);
                }
            }
        }
    }
    backward
}
```

**Prerequisite:** `Svfg` needs a `predecessors_of()` method (reverse adjacency).
Currently only `successors_of()` exists. Add a reverse edge index built lazily
or eagerly during SVFG construction.

#### Step 3: VFG Guard Computation

For each SVFG edge `(src → dst)` in the backward slice, compute the
intra-procedural control-flow guard — the condition under which control flows
from `src`'s basic block to `dst`'s basic block.

**File:** `crates/saf-analysis/src/checkers/solver.rs` (new function)

```rust
fn compute_vfg_guard(
    src_block: BlockId,
    dst_block: BlockId,
    cfg: &Cfg,
    module: &AirModule,
    func_id: FunctionId,
) -> Option<PathCondition> {
    // Fast path: if dst post-dominates src, guard is TRUE
    if post_dominates(dst_block, src_block, cfg) {
        return None; // TRUE — unconditional
    }

    // BFS on CFG from src_block, accumulating branch conditions
    // at each CondBr along the path to dst_block
    extract_guards_from_blocks(&block_path, module, func_id)
}
```

This reuses the existing `guard::extract_guards_from_blocks()` and
`z3_utils::dominator::compute_post_dominators()` infrastructure.

#### Step 4: Z3 All-Path Check

Propagate guard conditions along the backward slice from source to sinks.
At each sink, the accumulated condition represents "under what condition does
the value flow reach this free." The disjunction of all sink conditions is
the "all-path guard." Ask Z3 if it equals TRUE.

**File:** `crates/saf-analysis/src/checkers/solver.rs` (new function)

```rust
fn all_path_reachable(
    svfg: &Svfg,
    source: SvfgNodeId,
    reached_sinks: &[SvfgNodeId],
    backward_slice: &BTreeSet<SvfgNodeId>,
    node_to_block: &BTreeMap<ValueId, (FunctionId, BlockId)>,
    cfgs: &BTreeMap<FunctionId, Cfg>,
    module: &AirModule,
    checker: &PathFeasibilityChecker,
) -> bool {
    // Initialize: source condition = TRUE
    let mut vf_cond: BTreeMap<SvfgNodeId, Condition> = BTreeMap::new();
    vf_cond.insert(source, Condition::True);

    // Topological propagation through backward slice
    let topo_order = topological_sort(svfg, source, &backward_slice);

    for node in topo_order {
        let node_cond = vf_cond.get(&node).cloned().unwrap_or(Condition::False);
        for (edge_kind, succ) in svfg.successors_of(node).unwrap_or_default() {
            if !backward_slice.contains(succ) { continue; }

            // Compute VFG guard between node's BB and succ's BB
            let guard = compute_vfg_guard(node_bb, succ_bb, cfg, module, func);

            // Propagate: succ_cond = succ_cond OR (node_cond AND guard)
            let new_cond = Condition::And(node_cond, guard);
            let existing = vf_cond.entry(*succ).or_insert(Condition::False);
            *existing = Condition::Or(existing, new_cond);
        }
    }

    // Disjunction of all sink conditions
    let mut final_guard = Condition::False;
    for sink in reached_sinks {
        if let Some(cond) = vf_cond.get(sink) {
            final_guard = Condition::Or(final_guard, cond);
        }
    }

    // Ask Z3: is final_guard ≡ TRUE?
    checker.is_equivalent_to_true(&final_guard)
}
```

#### Step 5: Integrate into `never_reach_sink`

Modify the solver to call `all_path_reachable` when sinks are found:

```rust
// In never_reach_sink(), after BFS:
if reached_any_sink {
    if config.check_all_paths {
        let forward_slice = visited.iter().map(|(n, _)| *n).collect();
        let bwd_slice = backward_slice(svfg, &forward_slice, &reached_sinks);
        if !all_path_reachable(svfg, source, &reached_sinks, &bwd_slice, ...) {
            // PARTIALLEAK
            findings.push(/* partial leak finding */);
        }
    }
    // else: SAFE (existing behavior — any sink reached = no leak)
} else {
    // NEVERFREE (existing behavior — no sink reached)
    findings.push(/* never freed finding */);
}
```

The `check_all_paths` flag in `SolverConfig` allows enabling/disabling the
more expensive all-path check per invocation.

#### Step 6: Spec Update

Add a `detect_partial_leaks: bool` field to `CheckerSpec` or `SolverConfig`
to control whether the all-path check is performed. Default: `true` for
memory-leak, `false` for other checkers initially.

### Implementation Order

| Step | Description | Files | Prereqs |
|------|-------------|-------|---------|
| 2.1 | Add `predecessors_of()` to `Svfg` | `svfg/mod.rs` | — |
| 2.2 | Build `node_to_block` map | `runner.rs` | — |
| 2.3 | Add post-dominator computation | `z3_utils/dominator.rs` | — |
| 2.4 | Implement `backward_slice()` | `solver.rs` | 2.1 |
| 2.5 | Implement `compute_vfg_guard()` | `solver.rs` | 2.2, 2.3 |
| 2.6 | Implement `all_path_reachable()` | `solver.rs` | 2.4, 2.5 |
| 2.7 | Integrate into `never_reach_sink` | `solver.rs`, `runner.rs` | 2.6 |
| 2.8 | Add `check_all_paths` to config | `solver.rs`, `spec.rs` | 2.7 |
| 2.9 | Test: variant 12 partial leak | `tests/` | 2.7 |
| 2.10 | Benchmark: CWE401 + PTABen | — | 2.9 |

### Test Cases

**New test — partial leak detected:**
```c
// tests/mytests/checks/leak_partial.c
#include <stdlib.h>
extern int cond(void);
int main(void) {
    void *p = malloc(10);
    if (cond()) free(p);
    return 0;
    // Expected: PARTIALLEAK — free on one path only
}
```

**Existing test — no regression:**
- `leak_freed_in_caller.c` — wrapper pattern, free in caller → SAFE (0 findings)
- `leak_dead_end.c` — dead-end flow → NEVERFREE (1 finding)
- `leak_basic.c` — never freed → NEVERFREE (1 finding)

**Juliet variant 12 — the target FNs:**
```llvm
; CWE401_Memory_Leak__char_malloc_12_bad:
%4 = call ptr @malloc(i64 100)
%.0 = phi ptr [ %4, %8 ], [ %11, %10 ]
br i1 %15, label %16, label %17
16: br label %18                           ; skip free → LEAK
17: call void @free(ptr %.0)               ; free on this path only
18: ret void
```

With AllPathReachable: forward BFS finds `free` → `somePathReachable=true`.
Guard computation: the VFG guard from source to `free` includes `%15 == false`.
Z3: `%15 == false ≢ TRUE` → `allPathReachable=false` → **PARTIALLEAK**.
This would convert 4 of the 16 CWE401 FNs to TPs.

### Expected Benchmark Impact

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| CWE401 TP | 670 | 674 | +4 |
| CWE401 FN | 16 | 12 | -4 |
| CWE401 Recall | 97.7% | 98.2% | +0.5pp |

The remaining 12 FNs are `CWE401_Invalid_Free` variants (wrong CWE category —
SAF has no invalid-free checker) and cannot be fixed by partial leak detection.

### Design Notes

- SAF's SVFG uses `IndirectDef` edges through memory SSA (due to `-O0`
  store/load pairs), unlike direct SSA def-use edges. The guard computation
  works on basic blocks, so the SVFG edge type doesn't affect correctness.
- Context sensitivity uses existing CFL-based call-string matching.
- Guard computation reuses existing `guard.rs` + `z3_utils/solver.rs`.
- Global escape is handled by existing `reachable_from_main` filter.
- Strong-update edge removal (for singleton stores) is not yet needed but
  can be added later for precision.

### Risks and Mitigations

1. **Performance**: Z3 calls for every partial-leak candidate add latency.
   *Mitigation*: Only invoke AllPathReachable when sinks are found (most
   allocations are either never-freed or always-freed). Gate behind config flag.

2. **Interprocedural guards**: VFG guards between basic blocks in different
   functions (across Call/Return edges) need special handling.
   *Mitigation*: Start with intraprocedural-only guards (both source and sink
   in same function). Interprocedural can be added incrementally.

3. **SVFG predecessors**: Adding reverse adjacency doubles memory for edges.
   *Mitigation*: Build lazily only when `check_all_paths` is enabled. Or
   compute backward slice using forward edge traversal from source (slower
   but no extra storage).

---

## Appendix: Phase 1 Detailed Analysis

### Breakdown of the 16 CWE401 FNs

All 16 FN tests are variant 12 (`globalReturnsTrueOrFalse()` control flow).

**12 `CWE401_Invalid_Free` bad variants** (6 from s01, 6 from s03):
The actual bug is freeing stack memory (alloca pointer through a phi).
SAF has no invalid-free checker. These are NOT fixable by partial leak
detection.

**4 `CWE401_Memory_Leak` variant 12 bad variants:**
- `char_calloc_12_bad`, `char_malloc_12_bad`, `char_realloc_12_bad` (s01)
- `strdup_char_12_bad` (s02)

The bug is a conditional free — one branch frees, the other leaks. These ARE
fixable by AllPathReachable (Phase 2).

### PTABen mem_leak Results (Phase 1)

| Metric | Baseline | Phase 1 | Delta |
|--------|----------|---------|-------|
| Exact  | 101      | 101     | 0     |
| Sound  | 7        | 7       | 0     |
| Unsound| 38       | 38      | 0     |
