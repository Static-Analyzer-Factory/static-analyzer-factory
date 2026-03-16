# Plan 003: Graph Builders (E3)

## Overview

Build CFG, ICFG, CallGraph, and DefUseGraph from AIR. All graphs use `BTreeMap`/`BTreeSet` for deterministic iteration. Export to graph-specific JSON schemas.

## Design Decisions

| Topic | Decision |
|-------|----------|
| CFG/ICFG granularity | Hybrid: block-level CFG, instruction-level call/return in ICFG |
| Graph storage | Typed separate structs with `BTreeMap`/`BTreeSet` |
| Algorithms | Implement ourselves (DFS, BFS, Tarjan SCC, toposort) |
| DefUse representation | Forward map only (value → users) |
| ICFG edge types | Explicit enum: `Intra`, `Call`, `Return` |
| Indirect calls | Per-call-site placeholder nodes |
| External functions | Explicit enum variant in CallGraph nodes |
| JSON export | Graph-specific schemas |
| Testing | Unit + builder + integration + property-based |

## Core Types

### CFG (per function)

```rust
pub struct Cfg {
    pub function: FunctionId,
    pub entry: BlockId,
    pub exits: BTreeSet<BlockId>,
    pub successors: BTreeMap<BlockId, BTreeSet<BlockId>>,
    pub predecessors: BTreeMap<BlockId, BTreeSet<BlockId>>,
}
```

### CallGraph (whole program)

```rust
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallGraphNode {
    Function(FunctionId),
    External { name: String, func: FunctionId },
    IndirectPlaceholder { site: InstId },
}

pub struct CallGraph {
    pub nodes: BTreeSet<CallGraphNode>,
    pub edges: BTreeMap<CallGraphNode, BTreeSet<CallGraphNode>>,
    pub call_sites: BTreeMap<InstId, CallGraphNode>,
}
```

### ICFG (whole program)

```rust
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IcfgEdge {
    Intra,
    Call { site: InstId },
    Return { site: InstId },
}

pub struct Icfg {
    pub cfgs: BTreeMap<FunctionId, Cfg>,
    pub inter_edges: BTreeMap<(BlockId, BlockId), IcfgEdge>,
    pub call_site_map: BTreeMap<InstId, (BlockId, BlockId)>,
}
```

### DefUseGraph (SSA def-use chains)

```rust
pub struct DefUseGraph {
    pub uses: BTreeMap<ValueId, BTreeSet<InstId>>,
    pub defs: BTreeMap<ValueId, Option<InstId>>,
}
```

## Graph Algorithms

Generic algorithms in `graph_algo` module:

```rust
pub fn dfs<N: Ord + Clone>(start: &N, successors: impl Fn(&N) -> &BTreeSet<N>) -> Vec<N>;
pub fn bfs<N: Ord + Clone>(start: &N, successors: impl Fn(&N) -> &BTreeSet<N>) -> Vec<N>;
pub fn post_order<N: Ord + Clone>(start: &N, successors: impl Fn(&N) -> &BTreeSet<N>) -> Vec<N>;
pub fn tarjan_scc<N: Ord + Clone>(nodes: &BTreeSet<N>, successors: impl Fn(&N) -> &BTreeSet<N>) -> Vec<BTreeSet<N>>;
pub fn toposort<N: Ord + Clone>(nodes: &BTreeSet<N>, successors: impl Fn(&N) -> &BTreeSet<N>) -> Option<Vec<N>>;
pub fn reachable<N: Ord + Clone>(start: &N, successors: impl Fn(&N) -> &BTreeSet<N>) -> BTreeSet<N>;
```

## Builders

### CFG Builder

```rust
impl Cfg {
    pub fn build(func: &AirFunction) -> Self;
}
```

Build CFG by extracting successors from terminators:
- `Br { target }` → single successor
- `CondBr { then, else }` → two successors
- `Switch { default, cases }` → default + case targets
- `Ret` / `Unreachable` → no successors, mark as exit

### CallGraph Builder

```rust
impl CallGraph {
    pub fn build(module: &AirModule) -> Self;
}
```

Build by scanning all call instructions:
- Add `Function` node for each function (`External` if `is_declaration`)
- `CallDirect { callee }` → edge to `Function(callee)`
- `CallIndirect` → create `IndirectPlaceholder { site }`, edge to it

### ICFG Builder

```rust
impl Icfg {
    pub fn build(module: &AirModule, callgraph: &CallGraph) -> Self;
}
```

Build by combining CFGs with inter-procedural edges:
- Build CFG for each defined function
- For each resolved call site:
  - `Call` edge: caller block → callee entry
  - `Return` edge: callee exits → caller return block

### DefUseGraph Builder

```rust
impl DefUseGraph {
    pub fn build(module: &AirModule) -> Self;
}
```

Build by scanning definitions and uses:
- Params: `defs[param_id] = None`
- Instructions with `dst`: `defs[dst] = Some(inst_id)`
- Operands: `uses[operand].insert(inst_id)`
- Phi incoming values count as uses

## Export Formats

### CfgExport

```rust
pub struct CfgExport {
    pub function: String,
    pub entry: String,
    pub exits: Vec<String>,
    pub blocks: Vec<CfgBlockExport>,
}

pub struct CfgBlockExport {
    pub id: String,
    pub label: Option<String>,
    pub successors: Vec<String>,
}
```

### CallGraphExport

```rust
pub struct CallGraphExport {
    pub nodes: Vec<CallGraphNodeExport>,
    pub edges: Vec<CallGraphEdgeExport>,
}

pub struct CallGraphNodeExport {
    pub id: String,
    pub kind: String,
    pub name: Option<String>,
}

pub struct CallGraphEdgeExport {
    pub src: String,
    pub dst: String,
}
```

### IcfgExport

```rust
pub struct IcfgExport {
    pub functions: Vec<CfgExport>,
    pub inter_edges: Vec<IcfgEdgeExport>,
}

pub struct IcfgEdgeExport {
    pub src: String,
    pub dst: String,
    pub kind: String,
    pub site: Option<String>,
}
```

### DefUseExport

```rust
pub struct DefUseExport {
    pub definitions: Vec<DefExport>,
    pub uses: Vec<UseExport>,
}

pub struct DefExport {
    pub value: String,
    pub defined_by: Option<String>,
}

pub struct UseExport {
    pub value: String,
    pub used_by: String,
}
```

## Testing Strategy

### Unit Tests

- Graph algorithm correctness (DFS order, SCC detection, etc.)
- Deterministic iteration order

### Builder Tests (inline AIR fixtures)

- CFG: linear, diamond, loop patterns
- CallGraph: direct, indirect, external calls
- DefUse: params, inst results, phi nodes
- ICFG: call/return edge construction

### Property-Based Tests (proptest)

```rust
proptest! {
    fn cfg_edge_count_matches_terminators(func in arb_air_function()) { ... }
    fn predecessors_is_reverse_of_successors(func in arb_air_function()) { ... }
    fn defuse_every_operand_is_recorded(module in arb_air_module()) { ... }
}
```

### Integration Tests (insta snapshots)

- Build graphs from E1 `.air.json` fixtures
- Build graphs from E2 `.bc` fixtures
- Verify deterministic JSON export
- Verify equivalent programs produce equivalent graphs

## Implementation Phases

| Phase | Description | Tests |
|-------|-------------|-------|
| 1 | `graph_algo` module | Unit tests for DFS, BFS, SCC, toposort |
| 2 | `Cfg` struct + builder | Builder tests (linear, diamond, loop) |
| 3 | `CallGraph` struct + builder | Builder tests (direct, indirect, external) |
| 4 | `DefUseGraph` struct + builder | Builder tests (params, results, phi) |
| 5 | `Icfg` struct + builder | Builder tests (call/return edges) |
| 6 | Export structs | Unit tests for deterministic JSON |
| 7 | Property-based tests | Add proptest invariants |
| 8 | Integration tests | Snapshot tests with fixtures |

## File Structure

```
crates/saf-analysis/src/
  lib.rs
  error.rs
  graph_algo.rs      # NEW
  cfg.rs             # expand
  callgraph.rs       # expand
  defuse.rs          # NEW
  icfg.rs            # NEW
  pta.rs             # placeholder
  valueflow.rs       # placeholder
```

## Dependencies

Add to `saf-analysis/Cargo.toml`:
- `proptest` (dev dependency)

## Acceptance Criteria

- [ ] `make test` passes with all new tests
- [ ] `make lint` passes (clippy + rustfmt)
- [ ] All graphs export deterministic JSON (run twice, compare)
- [ ] Property tests pass with default proptest config
- [ ] Integration tests snapshot existing fixtures
