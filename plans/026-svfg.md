# Plan 026 — Sparse Value-Flow Graph (SVFG)

**Epic:** E12 — SVFG
**Status:** approved
**Depends on:** E11 (Memory SSA), E10 (CG Refinement), E4 (PTA)

## Goal

Build a Sparse Value-Flow Graph that unifies direct (register) and indirect (memory) value-flow into one graph. Replaces the imprecise `value → Location → value` edges in the existing ValueFlowGraph with precise `value → value` edges computed via Memory SSA clobber analysis.

SVFG is the foundation for SABER-style memory safety checkers (leak, UAF, double-free) which are graph reachability problems on SVFG. Those checkers are a follow-up epic; E12 delivers the graph, queries, export, and Python bindings.

## Design

### Node Model

```rust
/// SVFG node: either an SSA value or a Memory SSA Phi merge point.
pub enum SvfgNodeId {
    Value(ValueId),         // SSA value (most nodes)
    MemPhi(MemAccessId),    // Memory SSA Phi merge point (at control-flow joins)
}
```

ValueId-based: most nodes are SSA values. MemPhi is the only non-value variant, representing control-flow joins where multiple memory definitions merge.

### Edge Model

```rust
pub enum SvfgEdgeKind {
    // Direct (top-level SSA) — same semantics as existing ValueFlow
    DirectDef,       // SSA def-use chain
    DirectTransform, // binary/unary op operand → result
    CallArg,         // actual argument → formal parameter
    Return,          // return value → caller result

    // Indirect (through memory, via MSSA)
    IndirectDef,     // store's value → load's result (clobber is a Store)
    IndirectStore,   // store's value → MemPhi node (store feeds a Phi)
    IndirectLoad,    // MemPhi node → load's result (load reads from Phi)
    PhiFlow,         // MemPhi → MemPhi (nested Phi chaining)
}
```

### How Indirect Edges Work

- **Store clobbers load directly:** `store %val, %ptr` → `%result = load %ptr` becomes `Value(%val) --IndirectDef--> Value(%result)`
- **Phi merge:** If the clobber is a Memory Phi, edges fan through it: `Value(%val1) --IndirectStore--> MemPhi(phi) --IndirectLoad--> Value(%result)`
- **Nested Phi:** `MemPhi(outer) --PhiFlow--> MemPhi(inner)`

### E12 Scope Limitation

Only create indirect edges when the clobber is a Store instruction. When the clobber is a Call (interprocedural memory modification) or LiveOnEntry, skip the indirect edge. Interprocedural memory flow through call bodies is deferred. CallArg/Return edges already capture top-level flow across calls. Skipped clobbers are tracked in diagnostics.

### Construction Algorithm (4 phases)

**Phase 1 — Collect store→value mappings.** Scan all store instructions, record `BTreeMap<MemAccessId, ValueId>` mapping each MSSA Def to its stored value.

**Phase 2 — Build Memory Phi edges.** For each MSSA Phi node, walk its operands:
- Operand is a Def → `IndirectStore` edge from stored value to MemPhi
- Operand is another Phi → `PhiFlow` edge
- Operand is LiveOnEntry → skip

**Phase 3 — Build indirect store→load edges.** For each load instruction:
1. Get the load's MSSA Use access
2. For each location in `pta.points_to(load_ptr)`:
   - Call `mssa.clobber_for(use_id, loc)`
   - Clobber is Store Def → `IndirectDef` edge: `Value(stored_val) → Value(load_result)`
   - Clobber is Phi → `IndirectLoad` edge: `MemPhi(phi_id) → Value(load_result)`
   - Clobber is Call Def → skip (tracked in diagnostics)
   - Clobber is LiveOnEntry → skip

**Phase 4 — Build direct edges.** Scan all instructions, create DirectDef, DirectTransform, CallArg, Return edges. Same logic as existing ValueFlow builder.

### Inputs

```rust
SvfgBuilder::new(
    module: &AirModule,
    defuse: &DefUseGraph,
    callgraph: &CallGraph,
    pta: &PtaResult,
    mssa: &mut MemorySsa,  // mut because clobber_for() caches
) -> Svfg
```

### Query API

```rust
impl Svfg {
    pub fn forward_reachable(&self, from: SvfgNodeId) -> BTreeSet<SvfgNodeId>;
    pub fn backward_reachable(&self, from: SvfgNodeId) -> BTreeSet<SvfgNodeId>;
    pub fn reachable(&self, from: ValueId, to: ValueId) -> bool;
    pub fn value_flow_path(&self, from: ValueId, to: ValueId, max_depth: usize) -> Option<Vec<SvfgNodeId>>;
}
```

### Diagnostics

```rust
pub struct SvfgDiagnostics {
    pub direct_edge_count: usize,
    pub indirect_edge_count: usize,
    pub mem_phi_count: usize,
    pub skipped_call_clobbers: usize,
    pub skipped_live_on_entry: usize,
}
```

### Export Format

```json
{
  "schema_version": "0.1.0",
  "node_count": 42,
  "edge_count": 67,
  "diagnostics": {
    "direct_edge_count": 50,
    "indirect_edge_count": 17,
    "mem_phi_count": 3,
    "skipped_call_clobbers": 2,
    "skipped_live_on_entry": 1
  },
  "nodes": [
    {"kind": "value", "id": "0x..."},
    {"kind": "mem_phi", "id": "0x...", "block": "0x..."}
  ],
  "edges": [
    {"src": "0x...", "dst": "0x...", "kind": "direct_def"},
    {"src": "0x...", "dst": "0x...", "kind": "indirect_def"}
  ]
}
```

### Python Bindings

- `Project.svfg()` → `PySvfg`
- `PySvfg.node_count` → int
- `PySvfg.edge_count` → int
- `PySvfg.reachable(from_hex: str, to_hex: str)` → bool
- `PySvfg.forward_reachable(from_hex: str)` → list[str]
- `PySvfg.value_flow_path(from_hex: str, to_hex: str)` → list[str] | None
- `PySvfg.export()` → dict

## E2E Test Programs

| # | File | Language | Scenario |
|---|------|----------|----------|
| 1 | `svfg_store_load_disambig.c` | C | Two non-aliasing pointers. Store through both, load from one. Verify correct store→load indirect edge. |
| 2 | `svfg_phi_merge.c` | C | Diamond CFG: if-branch stores val1, else-branch stores val2, load after join. Verify MemPhi + IndirectStore/IndirectLoad edges. |
| 3 | `svfg_interproc_direct.c` | C | Function returns tainted value, caller stores and loads back. Tests Return + IndirectDef edge combination. |
| 4 | `svfg_reachability.c` | C | source()→store→load→sink(). Tests `reachable()` query finds the path through memory. |
| 5 | `svfg_class_member.cpp` | C++ | Constructor stores to member field, getter loads. Tests struct field flow. |
| 6 | `svfg_unsafe_ptr.rs` | Rust | Raw pointer write + read in `unsafe` block. Tests Rust path. |

Each test asserts:
- SVFG builds without panic
- Node/edge counts > 0
- Specific indirect edges exist (or reachability holds)
- Export produces valid JSON
- Export is deterministic (byte-identical across two runs)

## Files

### New Files

```
crates/saf-analysis/src/svfg/
  mod.rs          — Svfg struct, SvfgNodeId, SvfgEdgeKind, public API
  builder.rs      — SvfgBuilder (4-phase construction)
  query.rs        — forward_reachable, backward_reachable, reachable, value_flow_path
  export.rs       — JSON export (SvfgExport, serde)

crates/saf-python/src/svfg.rs  — PySvfg Python bindings

tests/programs/c/svfg_store_load_disambig.c
tests/programs/c/svfg_phi_merge.c
tests/programs/c/svfg_interproc_direct.c
tests/programs/c/svfg_reachability.c
tests/programs/cpp/svfg_class_member.cpp
tests/programs/rust/svfg_unsafe_ptr.rs

crates/saf-analysis/tests/svfg_e2e.rs
python/tests/test_svfg.py

tutorials/graphs/06-svfg/
  vulnerable.c
  detect.py
  detect.rs
  README.md
```

### Modified Files

```
crates/saf-analysis/src/lib.rs         — add `pub mod svfg;`
crates/saf-python/src/lib.rs           — register svfg module
crates/saf-python/src/project.rs       — add Project.svfg() method
tests/programs/compile.sh              — add svfg_* compilation commands
docs/tool-comparison.md                — mark SVFG/sparse analysis as implemented
plans/PROGRESS.md                      — E12 tracking
plans/FUTURE.md                        — update SVFG entry status
```

## Implementation Phases (TDD)

### Phase 1 — Core types + graph operations
- `SvfgNodeId` enum (Value, MemPhi)
- `SvfgEdgeKind` enum (8 variants)
- `Svfg` struct with add_node, add_edge, successors, predecessors, node_count, edge_count
- `SvfgDiagnostics` struct
- **Tests:** unit tests for graph add/query operations

### Phase 2 — Builder: direct edges (Phase 4 of algorithm)
- Process all instructions for DefUse, Transform, CallArg, Return edges
- Same logic as ValueFlow builder but producing SvfgEdgeKind variants
- **Tests:** unit test: build from simple AIR module, verify direct edges exist

### Phase 3 — Builder: indirect edges (Phases 1-3 of algorithm)
- Phase 1: scan stores, build MemAccessId → ValueId map
- Phase 2: walk MSSA Phis, create IndirectStore/PhiFlow edges
- Phase 3: for each load, query clobber_for(), create IndirectDef/IndirectLoad edges
- **Tests:** unit test: build from AIR with stores/loads, verify indirect edges

### Phase 4 — Query API
- `forward_reachable()`: BFS on successors
- `backward_reachable()`: BFS on predecessors
- `reachable()`: forward BFS from source, check if sink reached
- `value_flow_path()`: BFS with parent tracking, reconstruct path
- **Tests:** unit tests for each query method

### Phase 5 — Export
- `SvfgExport` struct with serde Serialize
- `Svfg::export()` → SvfgExport
- JSON serialization with schema_version, nodes, edges, diagnostics
- **Tests:** unit test: export roundtrip, verify JSON structure

### Phase 6 — E2E tests
- Write 6 source programs (4 C, 1 C++, 1 Rust)
- Compile to .ll fixtures in Docker
- Write `svfg_e2e.rs` with Rust E2E tests
- **Tests:** 6+ Rust E2E tests (build, edges, reachability, determinism)

### Phase 7 — Python bindings
- `PySvfg` struct in `crates/saf-python/src/svfg.rs`
- `Project.svfg()` method
- Register in lib.rs
- Write `test_svfg.py`
- **Tests:** 6+ Python E2E tests

### Phase 8 — Tutorial
- `tutorials/graphs/06-svfg/` with vulnerable.c + detect.py + detect.rs + README.md
- Verify end-to-end in Docker

### Phase 9 — Documentation updates
- `docs/tool-comparison.md`: mark SVFG/sparse analysis as no longer a gap
- `plans/PROGRESS.md`: E12 complete
- `plans/FUTURE.md`: update SVFG entry
