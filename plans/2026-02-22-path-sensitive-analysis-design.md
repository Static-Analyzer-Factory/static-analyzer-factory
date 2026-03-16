# Path-Sensitive Analysis Design

**Date**: 2026-02-22
**Status**: Approved
**Goal**: Production-ready path-sensitivity infrastructure across three complementary analysis layers

## Problem

SAF achieves ~52% precision on Juliet benchmarks. The root cause (identified in Plan 146) is path-insensitive analysis: Juliet's good/bad variants differ only by which branch is taken (e.g., `if (GLOBAL_CONST_FIVE == 5)`), and SAF merges both branches, producing one TP and one FP for every test — a mathematical 50% precision ceiling.

Plans 145-147 implemented 12 phases of improvements (NULL analysis, Z3 guards, absint domains, function summaries, field sensitivity, loop widening) but none attacked the core path-insensitivity bottleneck. All were orthogonal to the actual ceiling.

## Literature Review

Production tools converge on proven strategies for path sensitivity without path explosion:

| Strategy | Tool | Core idea |
|----------|------|-----------|
| Guarded facts | Saturn/Coverity | Boolean guards on each fact, SAT/SMT-check at report time |
| Property-directed splitting | ESP (PLDI 2002) | Only split where the property FSM diverges. Bounded by |T| states |
| Disjunctive domain + budget | Infer/Pulse | Fork at branches, drop excess. 20 disjuncts finds 97% of bugs |
| Trace partitioning | Astree (ESOP 2005) | Partition by tokens (if-branch, loop-iter). Merge at endpoints |
| Fuel-based splitting | Frama-C/Eva | Per-function budget. Split while fuel lasts, merge when exhausted |
| CEGAR | CPAchecker | Start merged, refine only where spurious counterexample proves it |
| Fused SMT | Fusion (PLDI 2021) | Path-sensitive sparse analysis without explicit path conditions |

Key lesson: **encode path conditions symbolically and solve on demand** rather than enumerating paths explicitly.

## Solution: Three Complementary Options

Three options forming a pipeline where each feeds the next:

```
          +----------+
  AIR --->|  SCCP    |--> constant_values, dead_blocks
          +----+-----+
               | feeds
          +----v-----+
  AIR --->|  Absint  |--> intervals, buffer sizes, overflow findings
          |  (trace  |    (partitioned by SCCP-identified constants)
          | partition)|
          +----+-----+
               | feeds
          +----v-----+
  SVFG -->|  Guarded |--> UAF, double-free, null-deref, leak findings
          |  BFS     |    (guards simplified by SCCP constants,
          |          |     dead blocks pruned)
          +----------+
```

### Option 1: Guarded Value-Flow (Path-Sensitive SVFG)

**Approach**: Saturn/Coverity-style guarded edges + ESP-style property-directed filtering.

Attach `Option<Guard>` to SVFG edges via a side-table `BTreeMap<(SvfgNodeId, SvfgNodeId), Vec<Guard>>`. During SVFG construction, extract guards from `CondBr` terminators at Phi predecessor blocks. During BFS (`may_reach`), accumulate `PathCondition` per frontier node. At sinks, check feasibility via Z3.

**Budget**: Pulse-style max-disjuncts per node (default 20). When exceeded, drop disjuncts with the most guards. Sound for bug-finding (under-approximate).

**Property-directed filtering (ESP)**: Only track guards involving values relevant to the checker's property. Guards on irrelevant branches are dropped (treated as always-feasible).

**Changes**:
- `svfg/mod.rs`: Add `edge_guards` side-table to `Svfg`
- `svfg/builder.rs`: Extract guards from `CondBr` during Phase 2/4 edge construction
- `checkers/solver.rs`: `may_reach()` and `multi_reach()` accumulate `PathCondition`, Z3-check at sinks
- `z3_utils/guard.rs`: Add `is_relevant_to_property()` filter

**Expected impact**: Temporal CWEs (UAF, double-free, null-deref): ~50% to 65-75% precision.

### Option 2: Trace Partitioning (Path-Sensitive Absint)

**Approach**: Astree-style trace partitioning with Eva-style fuel budget.

Per-block state becomes `BTreeMap<PartitionKey, AbstractState>` instead of a single `AbstractState`. Partition tokens identify which branch was taken at a split point.

```rust
enum PartitionToken {
    Branch { cond_id: ValueId, taken: bool },
    LoopIter { header: BlockId, iteration: u8 },
}
```

**When to split** (Astree heuristic):
- At `CondBr` comparing against a program constant (literal integer, global constant)
- At loop headers for first K iterations (K=3, matching Eva's widening delay)
- NOT at branches comparing two runtime variables

**When to merge** (partition endpoints):
- When partition count exceeds fuel budget (default 16 per block)
- At function exit
- At loop headers after K iterations
- Merge strategy: join the two most similar partitions (semantic-directed clumping)

**Changes**:
- New `absint/partition.rs`: `PartitionToken`, `PartitionKey`, management logic (~200 lines)
- `absint/fixpoint.rs`: State map becomes partitioned. Worklist carries `(BlockId, PartitionKey)`. Split/merge logic.

**Expected impact**: Buffer overflow CWEs +5-15pp precision, integer overflow +5-10pp.

### Option 3: SCCP Pre-Pass

**Approach**: Classical Wegman-Zadeck SCCP adapted for AIR, running before absint and SVFG.

Standard SSA-based SCCP with dual worklists (SSA edges for value propagation, CFG edges for reachability). Three-level lattice: `Top` (unknown) -> `Constant(i128)` -> `Bottom` (overdetermined). Converges in two passes — O(instructions).

**Outputs**:
- `constant_values: BTreeMap<ValueId, i128>` — proven single-constant values
- `dead_blocks: BTreeSet<BlockId>` — unreachable via constant branch resolution

**Integration with other options**:

| Consumer | How it uses SCCP |
|----------|-----------------|
| Guarded SVFG | Dead blocks prune SVFG edges. Constants resolve guards without Z3 |
| Trace partitioning | Dead blocks skip. Constants inform `should_partition()` |
| Absint fixpoint | Pre-seed `constant_map`. Dead blocks skip from worklist |
| Checkers | Filter findings in unreachable code |

**Interprocedural extension**: Propagate constants through `CallDirect` edges where all call sites pass the same constant for a parameter.

**Changes**:
- New `absint/sccp.rs`: SCCP solver (~300-400 lines)
- `absint/fixpoint.rs`: Call `run_sccp()` before fixpoint, seed constants, skip dead blocks
- `checkers/solver.rs`: Accept `dead_blocks`, filter SVFG nodes
- `svfg/builder.rs`: Accept `dead_blocks`, skip dead block edges

**Expected impact**: +2-5pp alone; multiplied effect as enabler for Options 1+2.

## Implementation Phases

| Phase | What | Depends on | New code | Risk |
|-------|------|-----------|----------|------|
| A: SCCP | `absint/sccp.rs` pre-pass | Nothing | ~400 lines | Low |
| B: Guarded SVFG | Edge guards + guarded BFS | SCCP (dead blocks) | ~800 lines | Medium |
| C: Trace Partitioning | Partitioned fixpoint | SCCP (constants, dead blocks) | ~600 lines | Medium |

Phase A first. Phases B and C are independent and can be developed in parallel after A.

## Shared Infrastructure

| Component | Used by | Exists? |
|-----------|---------|---------|
| `Guard` struct | SVFG guards, SCCP simplification | Yes |
| `ValueLocationIndex` | Guard extraction, constant resolution | Yes |
| Z3 feasibility checking | Guarded BFS | Yes |
| `refine_branch_condition()` | Partition split logic | Yes |
| `detect_loop_headers()` | Partition merge points | Yes |
| Function effect summaries | Cross-function guarded BFS | Yes |
| `constant_map` | SCCP output consumption | Yes |
| Dead block set | All three | New (SCCP output) |

## Success Criteria

1. **Correctness**: No PTABen regressions (67 unsound baseline). No existing test failures.
2. **Soundness**: SCCP and dead-block pruning sound. Guarded BFS under-approximation acceptable (Pulse model).
3. **Performance**: SCCP < 5% overhead. Guarded BFS within 2x current BFS. Trace partitioning within fuel x current absint.
4. **Configurability**: All three behind config knobs, enabled/disabled independently, sensible defaults.
5. **Juliet**: After A: +2-5pp. After A+B: +12-25pp on temporal CWEs. After A+B+C: +5-15pp on value CWEs.

## References

- Saturn: Xie & Aiken, POPL 2005 / TOPLAS 2007 — guarded location sets
- ESP: Das, Lerner, Seigle, PLDI 2002 — property-directed path sensitivity
- Astree: Rival & Mauborgne, ESOP 2005 / TOPLAS 2007 — trace partitioning
- Infer/Pulse: Incorrectness Logic, POPL 2020 — disjunctive under-approximation
- Frama-C/Eva: slevel fuel-based precision control
- Wegman & Zadeck, POPL 1991 — sparse conditional constant propagation
- Fusion: Shi et al., PLDI 2021 — path-sensitive sparse analysis without path conditions
- Sparrow: selective precision, PLDI 2014 — property-directed feature activation
