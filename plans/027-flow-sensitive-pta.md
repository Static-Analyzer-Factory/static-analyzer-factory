# Plan 027: Flow-Sensitive Pointer Analysis (Sparse, SVFG-based)

**Epic:** E13 — Flow-Sensitive PTA
**Status:** done
**Depends on:** E4 (Andersen PTA), E11 (Memory SSA), E12 (SVFG)

## Overview

Implement sparse flow-sensitive pointer analysis (SFS) following Hardekopf & Lin's CGO'11 algorithm, adapted for SAF's SVFG infrastructure. This tracks points-to information per program point using Memory SSA-based SVFG propagation, with strong updates at stores where the destination pointer is a provable singleton.

### Key References

- Hardekopf & Lin, CGO'11: "Flow-Sensitive Pointer Analysis for Millions of Lines of Code"
- Hardekopf & Lin, POPL'09: "Semi-Sparse Flow-Sensitive Pointer Analysis"
- SVF `FlowSensitive.cpp`: production implementation of SFS on SVFG
- Bossut et al., CGO'21: "Object Versioning for Flow-Sensitive Pointer Analysis" (future optimization)

## Design

### Algorithm: Sparse Flow-Sensitive (SFS)

**Staged approach:**
1. Run Andersen CI (existing E4) as pre-analysis → initial points-to sets
2. Build SVFG (existing E12) using Andersen results
3. Build FsSvfg: annotate SVFG indirect edges with LocId object labels
4. Run SFS solver: worklist propagation with IN/OUT dataflow sets on FsSvfg
5. Result: per-SVFG-node flow-sensitive points-to information

**Two tracking mechanisms (following SVF):**
- **Top-level pointers** (SSA values): `pts: Map<ValueId, Set<LocId>>` — propagated along direct SVFG edges
- **Address-taken objects** (memory locations): `dfIn[node][loc] → Set<LocId>` and `dfOut[node][loc] → Set<LocId>` — propagated along indirect SVFG edges labeled with the relevant object

**Strong update conditions (all must hold):**
1. `pts(store_ptr).len() == 1` — singleton destination
2. Target location is not array-collapsed (`PathStep::Index`)
3. Store is not in a recursive function (SCC size > 1 or self-edge in call graph)

### New Types

```
crates/saf-analysis/src/fspta/
  mod.rs          — FsSvfg, FsSvfgEdge, FsPtaConfig, FlowSensitivePtaResult, DfPointsTo
  builder.rs      — FsSvfg construction (annotate indirect edges with LocIds)
  solver.rs       — SFS worklist solver with IN/OUT propagation
  strong_update.rs — Strong update condition checking
  export.rs       — JSON export of flow-sensitive results
```

**FsSvfg** — Object-labeled SVFG:
```rust
pub struct FsSvfgEdge {
    pub kind: SvfgEdgeKind,
    pub target: SvfgNodeId,
    pub objects: BTreeSet<LocId>,  // non-empty for indirect edges
}

pub struct FsSvfg {
    successors: BTreeMap<SvfgNodeId, Vec<FsSvfgEdge>>,
    predecessors: BTreeMap<SvfgNodeId, Vec<FsSvfgEdge>>,
    nodes: BTreeSet<SvfgNodeId>,
    store_nodes: BTreeMap<SvfgNodeId, StoreInfo>,  // ptr + val ValueIds
    load_nodes: BTreeMap<SvfgNodeId, LoadInfo>,    // ptr + dst ValueIds
}
```

**FlowSensitivePtaResult** — Query API:
```rust
pub type DfPointsTo = BTreeMap<LocId, BTreeSet<LocId>>;

pub struct FlowSensitivePtaResult {
    pts: BTreeMap<ValueId, BTreeSet<LocId>>,
    df_in: BTreeMap<SvfgNodeId, DfPointsTo>,
    df_out: BTreeMap<SvfgNodeId, DfPointsTo>,
    diagnostics: FsPtaDiagnostics,
}

impl FlowSensitivePtaResult {
    pub fn points_to(&self, value: ValueId) -> &BTreeSet<LocId>
    pub fn points_to_at(&self, loc: LocId, node: SvfgNodeId) -> &BTreeSet<LocId>
    pub fn may_alias_at(&self, p: ValueId, q: ValueId, node: SvfgNodeId) -> AliasResult
    pub fn export(&self) -> FsPtaExport
}
```

**FsPtaConfig**:
```rust
pub struct FsPtaConfig {
    pub max_iterations: usize,       // default: 100_000
}
```

### Solver Algorithm

```
solve(fs_svfg, andersen_pta, module) → FlowSensitivePtaResult:

  1. INITIALIZE
     - Seed pts from Andersen results for all top-level pointers
     - dfIn/dfOut start empty
     - Add all nodes with non-empty pts to worklist

  2. PROCESS WORKLIST (BTreeSet for determinism)
     While worklist non-empty and iterations < max:
       node = worklist.pop_first()

       // Process node's instruction
       match node_type(node):
         Store(ptr, val):
           for loc in pts[ptr]:
             if strong_update_ok(loc, ptr, ...):
               df_out[node][loc] = pts[val]         // KILL + GEN
             else:
               df_out[node][loc] |= pts[val]        // GEN only
           // pass-through for unaffected objects:
           for (obj, obj_pts) in df_in[node] where obj not stored:
             df_out[node][obj] = obj_pts

         Load(ptr, dst):
           for loc in pts[ptr]:
             pts[dst] |= df_in[node][loc]

         Other (Phi, Copy, Cast, BinaryOp, Gep):
           // top-level: union operand pts into result
           // address-taken: df_out[node] = df_in[node]

       // Propagate to successors
       for edge in fs_svfg.successors(node):
         changed = false
         if edge.is_direct():
           changed = union top-level pts(src) into pts(dst)
         else:
           for obj in edge.objects:
             changed |= union df_out[src][obj] into df_in[target][obj]
         if changed:
           worklist.insert(target)

  3. RETURN result
```

### Integration

```
Pipeline position:
  AIR Module
    → CFG, CallGraph, DefUse (existing)
      → Andersen PTA (existing E4)
        → Memory SSA (existing E11)
          → SVFG (existing E12)
            → FsSvfg (NEW: annotate with LocIds)
              → SFS Solver (NEW)
                → FlowSensitivePtaResult

Rust entry point:
  pub fn solve_flow_sensitive(
      module: &AirModule,
      svfg: &Svfg,
      pta: &PtaResult,
      mssa: &mut MemorySsa,
      callgraph: &CallGraph,
      config: &FsPtaConfig,
  ) -> FlowSensitivePtaResult

Python entry point:
  project.flow_sensitive_pta() → PyFlowSensitivePtaResult
```

## Implementation Phases

### Phase 1: Core types + FsSvfg builder
- [x]`fspta/mod.rs`: `FsSvfgEdge`, `FsSvfg`, `FsPtaConfig`, `StoreInfo`, `LoadInfo`
- [x]`fspta/builder.rs`: Build `FsSvfg` from `Svfg` + `PtaResult` + `MemorySsa`
  - Replay SVFG builder Phase 3 logic to recover LocId labels on indirect edges
  - Collect store/load node metadata (ptr, val/dst ValueIds)
- [x]Unit tests: FsSvfg construction, edge labels, node metadata

### Phase 2: Strong update analysis
- [x]`fspta/strong_update.rs`: `strong_update_ok()` function
- [x]Pre-compute recursive functions set from call graph SCCs
- [x]Check: singleton pts, non-array location, non-recursive function
- [x]Unit tests: each condition independently, combined logic

### Phase 3: SFS solver
- [x]`fspta/solver.rs`: `solve_flow_sensitive()` entry point
- [x]Initialize pts from Andersen, empty dfIn/dfOut
- [x]Worklist loop: process Store (strong/weak), Load, pass-through
- [x]Propagation: direct edges (top-level pts), indirect edges (per-object dfIn/dfOut)
- [x]Convergence: monotone union, BTreeSet worklist, iteration limit
- [x]`FlowSensitivePtaResult` with query methods
- [x]Unit tests: small graphs with known fixpoints, convergence, strong vs weak update effect

### Phase 4: Export
- [x]`fspta/export.rs`: `FsPtaExport` JSON serialization
- [x]Schema: top-level pts + per-node dfIn/dfOut (summary, not full dump for large programs)
- [x]`FlowSensitivePtaResult::export()`
- [x]Unit tests: export format, determinism

### Phase 5: E2E tests (Rust)
- [x]Source programs (6 files):
  - `fspta_strong_update.c` — sequential stores, strong update kills stale value
  - `fspta_branch_merge.c` — if/else pointer assignment, per-branch tracking
  - `fspta_loop_weak_update.c` — loop store to may-alias pointer, weak update
  - `fspta_interproc.c` — interprocedural store/load through parameter
  - `fspta_cpp_field.cpp` — C++ class field pointer reassignment across methods
  - `fspta_rust_unsafe.rs` — Rust unsafe raw pointer sequential writes
- [x]Compile all to `.ll` in Docker
- [x]`crates/saf-analysis/tests/fspta_e2e.rs`: 6+ tests asserting precision > Andersen
- [x]Each test compares `fs_pta.points_to(v)` vs `andersen.points_to(v)`, asserts smaller set

### Phase 6: Python bindings
- [x]`crates/saf-python/src/fspta.rs`: `PyFlowSensitivePtaResult`
  - `points_to(value_hex) → list[str]`
  - `points_to_at(loc_hex, node_hex) → list[str]`
  - `may_alias_at(p_hex, q_hex, node_hex) → bool`
  - `diagnostics() → dict`
  - `export() → dict`
- [x]`Project.flow_sensitive_pta()` method
- [x]Register in Python module

### Phase 7: Python E2E tests
- [x]`python/tests/test_fspta.py`: mirror Rust E2E tests
- [x]Test precision comparison (flow-sensitive vs Andersen)
- [x]Test Python API (points_to, points_to_at, diagnostics, export)

### Phase 8: Tutorial
- [x]`tutorials/pta/07-flow-sensitive-pta/`
  - `vulnerable.c` — connection pool pointer reuse scenario
  - `detect.py` — compare Andersen vs flow-sensitive results
  - `detect.rs` — Rust version
  - `README.md` — walkthrough explaining flow-sensitivity and strong updates

### Phase 9: Documentation
- [x]Update `docs/tool-comparison.md`: mark Flow-sensitive PTA as implemented
- [x]Update `plans/FUTURE.md`: move Flow-sensitive PTA entry to implemented
- [x]Update `plans/PROGRESS.md`: E13 epic, plan 027, session log
