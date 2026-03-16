# Plan 005: ValueFlow + Taint Analysis (E5)

## Overview

Implement value flow graph construction and reachability/taint queries. The ValueFlow graph captures how data propagates through a program — via SSA definitions, transformations, function calls, and memory operations.

## Requirements Coverage

- FR-FLOW-001: ValueFlowGraph with edges (DEF_USE, TRANSFORM, CALL_ARG, RETURN, STORE, LOAD)
- FR-FLOW-002: Two modes — `precise` (uses PTA) and `fast` (unknown_mem only)
- FR-FLOW-003: Query API — `flows()` and `taint_flow()`
- FR-FLOW-004: Deterministic trace extraction via BFS with edge ordering

## Design Decisions

| Topic | Decision | Future Extension |
|-------|----------|------------------|
| Node representation | Hybrid ValueId + LocId | — |
| Location bounding | Configurable (All/Accessed/None) | — |
| TRANSFORM edges | All ops by default, configurable whitelist | — |
| Fast mode | Single unknown_mem, no PTA | Demand-driven PTA |
| Query API | Layered: ValueId core + Selectors | — |
| Sanitizers | Return values block paths | — |
| Traces | IDs + edges, enrichment at export | — |
| Export | JSON + SARIF with rich metadata | — |

## Core Types

### Node and Edge

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    Value(ValueId),
    Location(LocId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum EdgeKind {
    DefUse,     // SSA def → use (including phi incoming)
    Transform,  // Binary/unary op operand → result
    CallArg,    // Actual argument → formal parameter
    Return,     // Callee return → caller result
    Store,      // Value → memory location
    Load,       // Memory location → value
}
```

### Graph Structure

```rust
pub struct ValueFlowGraph {
    /// Outgoing edges: node → [(edge_kind, target)]
    successors: BTreeMap<NodeId, BTreeSet<(EdgeKind, NodeId)>>,
    /// Incoming edges: node → [(edge_kind, source)]
    predecessors: BTreeMap<NodeId, BTreeSet<(EdgeKind, NodeId)>>,
    /// All nodes in the graph
    nodes: BTreeSet<NodeId>,
    /// Diagnostics from construction
    diagnostics: ValueFlowDiagnostics,
}
```

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueFlowConfig {
    /// Analysis mode
    pub mode: ValueFlowMode,
    /// Which memory locations to include as nodes
    pub include_locations: IncludeLocations,
    /// Max locations per STORE/LOAD before collapsing to unknown_mem
    pub max_locations_per_access: usize,
    /// Collapse field paths to base object
    pub collapse_field_paths: bool,
    /// Which operations create TRANSFORM edges
    pub transform_propagation: TransformPropagation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ValueFlowMode {
    /// Use PTA for memory precision
    #[default]
    Precise,
    /// All memory through unknown_mem (no PTA)
    Fast,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum IncludeLocations {
    /// All PTA locations become nodes
    All,
    /// Only locations with STORE/LOAD
    #[default]
    Accessed,
    /// No location nodes (values only, memory implicit)
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformPropagation {
    /// All binary/unary ops create edges (default, sound)
    All,
    /// Only whitelisted operations
    Whitelist(BTreeSet<OpKind>),
}
```

**Defaults:**
- `mode: Precise`
- `include_locations: Accessed`
- `max_locations_per_access: 100`
- `collapse_field_paths: false`
- `transform_propagation: All`

## Graph Builder

### Builder Signature

```rust
pub struct ValueFlowBuilder<'a> {
    config: &'a ValueFlowConfig,
    module: &'a AirModule,
    defuse: &'a DefUseGraph,
    callgraph: &'a CallGraph,
    pta: Option<&'a PtaResult>,  // None in fast mode
}

impl<'a> ValueFlowBuilder<'a> {
    pub fn new(
        config: &'a ValueFlowConfig,
        module: &'a AirModule,
        defuse: &'a DefUseGraph,
        callgraph: &'a CallGraph,
        pta: Option<&'a PtaResult>,
    ) -> Self;

    pub fn build(self) -> ValueFlowGraph;
}
```

### Edge Construction Rules

| AIR Pattern | Edge(s) Created |
|-------------|-----------------|
| SSA def → use | `DefUse(def_value, use_value)` |
| Phi incoming | `DefUse(incoming_value, phi_result)` for each |
| Select | `DefUse(true_val, result)`, `DefUse(false_val, result)` |
| BinaryOp / UnaryOp | `Transform(operand, result)` for each operand (if config allows) |
| Cast | `Transform(src, dst)` |
| Call (arg) | `CallArg(actual, formal)` for each pointer arg |
| Call (return) | `Return(callee_ret, caller_result)` |
| Store (fast) | `Store(value, unknown_mem)` |
| Store (precise) | `Store(value, loc)` for each loc in points_to(ptr) |
| Load (fast) | `Load(unknown_mem, result)` |
| Load (precise) | `Load(loc, result)` for each loc in points_to(ptr) |

If `points_to(ptr).len() > max_locations_per_access`, collapse to `unknown_mem` and emit warning.

## Query API

### Core Methods

```rust
impl ValueFlowGraph {
    /// Find all paths from sources to sinks
    pub fn flows(
        &self,
        sources: &BTreeSet<ValueId>,
        sinks: &BTreeSet<ValueId>,
        limits: &QueryLimits,
    ) -> Vec<Flow>;

    /// Find taint flows, excluding paths through sanitizers
    pub fn taint_flow(
        &self,
        sources: &BTreeSet<ValueId>,
        sinks: &BTreeSet<ValueId>,
        sanitizers: &BTreeSet<ValueId>,
        limits: &QueryLimits,
    ) -> Vec<Finding>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLimits {
    /// Maximum path length (edges)
    pub max_depth: usize,
    /// Maximum paths to return
    pub max_paths: usize,
}
```

### BFS Algorithm

```
function flows(sources, sinks, limits):
    results = []
    for src in sources (sorted):
        queue = [(src, empty_trace)]
        visited = {}

        while queue not empty and results.len() < max_paths:
            (node, trace) = queue.pop_front()

            if node in sinks:
                results.push(Flow { source: src, sink: node, trace })
                continue

            if trace.len() >= max_depth:
                continue

            if visited[node] at shorter depth:
                continue
            visited[node] = trace.len()

            # Deterministic expansion: sort by (edge_kind, dst_id)
            for (edge, next) in successors[node].iter():
                queue.push((next, trace.append(edge, next)))

    return results
```

### Sanitizer Handling

- Convert sanitizer ValueIds to NodeIds
- During BFS, if `node` is a sanitizer node, don't expand (path dies)
- Other paths through the original tainted value continue

## Traces and Findings

### Trace Structure

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    pub steps: Vec<TraceStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceStep {
    pub from: NodeId,
    pub edge: EdgeKind,
    pub to: NodeId,
}
```

### Flow and Finding

```rust
#[derive(Debug, Clone)]
pub struct Flow {
    pub source: ValueId,
    pub sink: ValueId,
    pub trace: Trace,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub id: FindingId,           // Deterministic hash
    pub source: ValueId,
    pub sink: ValueId,
    pub trace: Trace,
    pub rule_id: Option<String>, // User-provided or auto-generated
}
```

### Enrichment

```rust
impl Trace {
    pub fn enrich(&self, module: &AirModule) -> EnrichedTrace;
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrichedTrace {
    pub steps: Vec<EnrichedStep>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrichedStep {
    pub from: NodeInfo,
    pub edge: String,
    pub to: NodeInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeInfo {
    pub id: String,
    pub kind: String,
    pub symbol: Option<String>,
    pub span: Option<SpanInfo>,
}
```

## Selectors

### Selector Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Selector {
    /// Return values of calls matching pattern
    CallReturn { pattern: String },
    /// Argument at index for functions matching pattern
    Argument { func_pattern: String, index: usize },
    /// Address of globals matching pattern
    GlobalAddress { pattern: String },
    /// Union of multiple selectors
    Union(Vec<Selector>),
    /// Intersection of multiple selectors
    Intersect(Vec<Selector>),
}

impl Selector {
    pub fn resolve(
        &self,
        module: &AirModule,
        callgraph: &CallGraph,
    ) -> BTreeSet<ValueId>;
}
```

### Convenience Constructors

```rust
// selector/sources.rs
pub fn argv() -> Selector;
pub fn getenv() -> Selector;

// selector/sinks.rs
pub fn system_call() -> Selector;
pub fn call(pattern: &str) -> Selector;

// selector/sanitizers.rs
pub fn call_return(pattern: &str) -> Selector;
```

## Export Formats

### JSON

```rust
#[derive(Debug, Serialize)]
pub struct FindingsExport {
    pub schema_version: String,
    pub mode: String,
    pub findings: Vec<FindingExport>,
    pub diagnostics: DiagnosticsExport,
}

#[derive(Debug, Serialize)]
pub struct FindingExport {
    pub id: String,
    pub rule_id: Option<String>,
    pub source: NodeInfo,
    pub sink: NodeInfo,
    pub trace: Vec<EnrichedStep>,
}
```

### SARIF

Mapping:
- `run.tool.driver.name` ← tool_name
- `run.tool.driver.version` ← tool_version
- `result.ruleId` ← finding.rule_id (or auto-generated)
- `result.message.text` ← "Data flows from {source} to {sink}"
- `result.locations[0]` ← sink span
- `result.relatedLocations[0]` ← source span
- `result.codeFlows[0].threadFlows[0].locations` ← trace steps

Determinism:
- No timestamps
- Findings sorted by `(sink_span, source_span, rule_id, trace_hash)`

## File Structure

```
crates/saf-analysis/src/
├── valueflow/
│   ├── mod.rs           # ValueFlowGraph, re-exports
│   ├── config.rs        # ValueFlowConfig, modes, options
│   ├── node.rs          # NodeId, NodeKind
│   ├── edge.rs          # EdgeKind, Edge
│   ├── builder.rs       # ValueFlowBuilder, edge construction
│   ├── query.rs         # flows(), taint_flow(), BFS
│   ├── trace.rs         # Trace, TraceStep, enrichment
│   ├── finding.rs       # Finding, FindingId
│   └── export.rs        # JSON/SARIF export
├── selector/
│   ├── mod.rs           # Selector enum, resolve(), operators
│   ├── sources.rs       # argv(), getenv(), etc.
│   ├── sinks.rs         # system_call(), call(), etc.
│   └── sanitizers.rs    # call_return(), etc.
└── (existing modules)
```

## Implementation Phases

| Phase | Description | Tests |
|-------|-------------|-------|
| 1 | `valueflow/config.rs` — ValueFlowConfig | Config serde, defaults |
| 2 | `valueflow/node.rs`, `edge.rs` — NodeId, EdgeKind | Ord/Hash basics |
| 3 | `valueflow/mod.rs` — ValueFlowGraph struct | Empty graph, add node/edge |
| 4 | `valueflow/builder.rs` — DEF_USE edges | SSA, phi, select |
| 5 | `valueflow/builder.rs` — TRANSFORM edges | Binary ops |
| 6 | `valueflow/builder.rs` — CALL edges | CALL_ARG, RETURN |
| 7 | `valueflow/builder.rs` — memory (fast) | unknown_mem |
| 8 | `valueflow/builder.rs` — memory (precise) | PTA locations |
| 9 | `valueflow/builder.rs` — bounding | Config enforcement |
| 10 | `valueflow/trace.rs` — Trace, TraceStep | Structure, enrichment |
| 11 | `valueflow/query.rs` — flows() | BFS reachability |
| 12 | `valueflow/query.rs` — taint_flow() | Sanitizer filtering |
| 13 | `valueflow/finding.rs` — Finding | ID generation |
| 14 | `valueflow/export.rs` — JSON | Enriched export |
| 15 | `valueflow/export.rs` — SARIF | SARIF format |
| 16 | `selector/mod.rs`, `sources.rs` | Selector resolve |
| 17 | `selector/sinks.rs`, `sanitizers.rs` | Remaining selectors |
| 18 | Integration tests | Fixtures, determinism |
| 19 | Property-based tests | Proptest invariants |

## Test Scenarios

### Core Graph Construction (Must Have)
1. DEF_USE edges from SSA
2. TRANSFORM edges from binary ops
3. CALL_ARG edges
4. RETURN edges
5. STORE edge (precise mode)
6. LOAD edge (precise mode)
7. Memory through unknown_mem (fast mode)
8. Phi node merging
9. Select merging

### Location Bounding (Should Have)
10. `include_locations: Accessed`
11. `include_locations: None`
12. `max_locations_per_access` exceeded
13. `collapse_field_paths: true`

### Query: flows() (Must Have)
14. Direct flow (source → sink)
15. Multi-hop flow
16. Flow through memory
17. No flow exists
18. Multiple paths
19. Cycle handling
20. max_depth limit
21. max_paths limit

### Query: taint_flow() (Must Have)
22. Basic taint propagation
23. Sanitizer blocks path
24. Sanitizer on one path only
25. Multiple sources
26. Multiple sinks

### Selectors (Should Have)
27. CallReturn selector
28. Argument selector
29. GlobalAddress selector
30. Union of selectors
31. No matches

### Trace & Export (Must Have)
32. Trace contains edges
33. Deterministic trace order
34. Enriched trace has spans
35. JSON export structure
36. SARIF export structure
37. Finding ID determinism

### Determinism (Must Have)
38. Repeated query
39. Graph export determinism
40. Edge ordering in BFS

### Integration with Fixtures (Must Have)
41. argv → system (C fixture)
42. getenv → fopen (C fixture)
43. AIR JSON equivalent

## Acceptance Criteria

- [ ] `make test` passes with all new tests
- [ ] `make lint` passes (clippy + rustfmt)
- [ ] ValueFlow graph export is deterministic (run twice, compare)
- [ ] Property tests pass with default proptest config
- [ ] Integration tests cover both LLVM and AIR JSON fixtures
- [ ] SARIF output validates against schema
- [ ] Taint queries work with selectors
