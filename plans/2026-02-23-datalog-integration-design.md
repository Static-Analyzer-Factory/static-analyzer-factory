# Datalog Integration for SAF via Ascent

**Date:** 2026-02-23
**Status:** Approved

## Motivation

Three goals drive this integration:

1. **Fix constraint sync bug.** SAF's PTA has 3 constraint extraction entry points (`extract_constraints`, `extract_constraints_reachable`, `extract_intraprocedural_constraints`) that must be manually kept in sync. Missing one causes silent analysis failures. A single Ascent fact-extraction function eliminates this hazard.

2. **Expressiveness for new checkers.** The current checker framework supports only 3 hardcoded reachability modes (`may_reach`, `must_not_reach`, `multi_reach`). Custom analyses require dropping to imperative Python graph traversal. Ascent rules enable declarative checker authoring, and a future Datalog interpreter enables user-authored rules at runtime.

3. **Performance via optimized engine.** Ascent provides semi-naive evaluation, automatic parallelism (`ascent_par!`), and BYODS (custom data structures for relations) — optimizations that the hand-written solver implements manually.

## Engine Choice: Ascent

**Selected:** [Ascent](https://github.com/s-arash/ascent) (Rust proc macro, MIT license)

Key features driving the choice:
- **Lattices:** First-class `lattice` keyword for PTA points-to sets (set union as join)
- **Aggregation:** `count`, `min`, `max`, `sum` inside rules (needed for `multi_reach` checkers)
- **Parallelism:** `ascent_par!` macro for automatic Rayon-based parallelism
- **BYODS:** Plug custom data structures into relations (union-find for SCC recovery)
- **WASM support:** `wasm-bindgen` feature flag for playground compatibility
- **Modularity:** `ascent_source!` + `include_source!` for splitting rules across modules

Rejected alternatives:
- **Crepe:** No lattices, no aggregation, no parallelism. Simpler but insufficient for PTA.
- **Souffle:** Most optimized, but C++ codegen — no WASM, external toolchain dependency.
- **DataFrog:** Not declarative (manual join API). Used in Polonius but too low-level.

## Architecture

```
                    +----------------------------------+
                    |        SAF Analysis Engine        |
                    |                                   |
Input -> Frontend-> |  AIR -> Fact Extractor (single fn)|
                    |           |                       |
                    |           v                       |
                    |    HVN Preprocessing (Rust)       |
                    |           |                       |
                    |           v                       |
                    |   +------------------+            |
                    |   |  Ascent Engine   |            |
                    |   |  (compile-time)  |            |
                    |   |                  |            |
                    |   |  - Offline SCC   |            |
                    |   |  - PTA rules     |            |
                    |   |  - BYODS UF      |            |
                    |   +--------+---------+            |
                    |            |                      |
                    |            v                      |
                    |   PointsToMap (same output type)  |
                    |            |                      |
                    |            v                      |
                    |   SVFG Builder (Rust, unchanged)  |
                    |            |                      |
                    |            v                      |
                    |   +------------------+            |
                    |   | Ascent Checkers  |            |
                    |   | (9 SVFG rules)   |            |
                    |   +--------+---------+            |
                    |            |                      |
                    |            v                      |
                    |   Findings (unified protocol)     |
                    +----------------------------------+

Feature flag `legacy-pta`: old worklist solver as fallback
```

### Crate structure

```
crates/
  saf-datalog/
    Cargo.toml            # depends on ascent, saf-core, saf-analysis
    src/
      lib.rs              # public API: solve_pta(), run_checker()
      facts.rs            # extract_facts() - single entry point from AIR
      hvn.rs              # HVN preprocessing (ported from saf-analysis)
      pta/
        mod.rs            # Ascent PTA program
        scc.rs            # Offline SCC as stratified rules
      checkers/
        mod.rs            # All 9 checker rule sets
        base_facts.rs     # SVFG -> Ascent input relations
      interpreter/        # Phase 2 (deferred)
        mod.rs
        parser.rs
        evaluator.rs
```

### Feature flags

```toml
# saf-analysis/Cargo.toml
[features]
default = []
legacy-pta = []  # Enable old worklist solver as fallback
```

When `legacy-pta` is enabled, both solvers are available and can be selected via config. When disabled (default), only the Ascent solver is compiled.

## Phase 1: Ascent PTA Solver

### Fact extraction (fixes sync bug)

Single function replaces three:

```rust
pub fn extract_facts(module: &AirModule, scope: AnalysisScope) -> PtaFacts {
    // scope: WholeProgram | Reachable(callgraph) | Intraprocedural(func_id)
    // Same logic, scope filters which functions to scan
}

pub struct PtaFacts {
    pub addr_of: Vec<(ValueId, LocationId)>,
    pub copy: Vec<(ValueId, ValueId)>,
    pub load: Vec<(ValueId, ValueId)>,
    pub store: Vec<(ValueId, ValueId)>,
    pub gep: Vec<(ValueId, ValueId, FieldPath)>,
}
```

### HVN preprocessing

Ported from current `hvn.rs`. Runs on `PtaFacts`, rewrites value IDs to representative IDs. Pure Rust, no Ascent — this is syntactic analysis of the constraint structure.

### Core PTA rules (Ascent)

```rust
ascent! {
    // --- Input facts ---
    relation addr_of(ValueId, LocationId);
    relation copy(ValueId, ValueId);
    relation load(ValueId, ValueId);
    relation store(ValueId, ValueId);
    relation gep(ValueId, ValueId, FieldPath);

    // --- PTA solution ---
    lattice points_to(ValueId, PointsToSet);

    // Addr: p = &x
    points_to(p, PointsToSet::singleton(loc)) <-- addr_of(p, loc);

    // Copy: p = q  (with BYODS union-find for cycle equivalence)
    points_to(p, pts) <-- copy(p, q), points_to(q, pts);

    // Store: *p = q
    points_to(loc_val, pts) <--
        store(p, q), points_to(p, p_pts),
        for loc in p_pts.iter(),
        let loc_val = loc.as_value(),
        points_to(q, pts);

    // Load: p = *q
    points_to(p, pts) <--
        load(p, q), points_to(q, q_pts),
        for loc in q_pts.iter(),
        let loc_val = loc.as_value(),
        points_to(loc_val, pts);

    // GEP: p = &q[offset]
    points_to(p, PointsToSet::singleton(field_loc)) <--
        gep(p, q, path), points_to(q, q_pts),
        for loc in q_pts.iter(),
        let field_loc = loc.with_field(path);
}
```

### Offline SCC (stratified Ascent rules)

Detect SCCs in the static copy-constraint graph before main PTA solving:

```rust
ascent! {
    // Stratum 1: Transitive closure on copy edges
    relation copy_edge(ValueId, ValueId);
    relation reaches(ValueId, ValueId);
    reaches(x, y) <-- copy_edge(x, y);
    reaches(x, z) <-- reaches(x, y), copy_edge(y, z);

    // Stratum 2: Cycle detection + representative selection
    relation in_cycle(ValueId, ValueId);
    in_cycle(x, y) <-- reaches(x, y), reaches(y, x);

    lattice representative(ValueId, Min<ValueId>);
    representative(x, Min(x)) <-- copy_edge(x, _);
    representative(y, Min(x)) <-- in_cycle(x, y);

    // Stratum 3: Rewrite all facts to use representatives
    // (omitted for brevity — same pattern for all 5 constraint types)
}
```

### BYODS union-find for copy equivalence

```rust
ascent! {
    #[ds(trrel_uf)]
    relation copy_equiv(ValueId, ValueId);

    copy_equiv(x, y) <-- copy(x, y);
    copy_equiv(y, x) <-- copy(x, y);

    // Points-to propagates through equivalence classes
    lattice points_to(ValueId, PointsToSet);
    points_to(y, pts) <-- copy_equiv(x, y), points_to(x, pts);
}
```

### Parallelism

CLI and Python paths use `ascent_par!` (same rules, parallel evaluation). WASM path uses sequential `ascent!` (no threads in browser). Selected at compile time via cfg:

```rust
#[cfg(not(target_arch = "wasm32"))]
ascent_par! { /* PTA rules */ }

#[cfg(target_arch = "wasm32")]
ascent! { /* same PTA rules */ }
```

### Validation strategy

1. Run both solvers (Ascent + legacy) on PTABen benchmark suite
2. Compare PointsToMap outputs for each test case
3. Require identical points-to sets (not just similar — byte-identical after sorting)
4. Run Juliet CWE benchmarks to verify checker results unchanged
5. Run CruxBc benchmarks to verify performance on real-world bitcode
6. Only after correctness parity + acceptable performance: make Ascent the default, legacy behind feature flag

## Phase 1: Ascent Checker Rules

### Base facts from SVFG

```rust
pub fn extract_checker_facts(svfg: &Svfg, spec: &CheckerSpec) -> CheckerFacts {
    // Classify SVFG nodes by spec patterns
    // Export as Ascent input relations
}

pub struct CheckerFacts {
    pub svfg_edge: Vec<(SvfgNodeId, SvfgNodeId, EdgeType)>,
    pub source: Vec<SvfgNodeId>,
    pub sink: Vec<SvfgNodeId>,
    pub sanitizer: Vec<SvfgNodeId>,
    pub exit: Vec<SvfgNodeId>,
}
```

### Reachability patterns (replaces 3 BFS solvers)

```rust
ascent! {
    // Shared base
    relation svfg_edge(SvfgNodeId, SvfgNodeId);
    relation source(SvfgNodeId);
    relation sink(SvfgNodeId);
    relation sanitizer(SvfgNodeId);
    relation exit_node(SvfgNodeId);

    // Unsanitized flow (transitive closure minus sanitizer nodes)
    relation flows_unsanitized(SvfgNodeId, SvfgNodeId);
    flows_unsanitized(a, b) <-- svfg_edge(a, b), !sanitizer(b);
    flows_unsanitized(a, c) <-- flows_unsanitized(a, b), svfg_edge(b, c), !sanitizer(c);

    // may_reach: source -> sink without sanitizer
    relation may_reach_finding(SvfgNodeId, SvfgNodeId);
    may_reach_finding(src, snk) <-- source(src), flows_unsanitized(src, snk), sink(snk);

    // must_not_reach: source -> exit without sanitizer (resource leak)
    relation must_not_reach_finding(SvfgNodeId, SvfgNodeId);
    must_not_reach_finding(src, ex) <-- source(src), flows_unsanitized(src, ex), exit_node(ex);

    // multi_reach: source -> 2+ distinct sinks
    relation reaches_sink(SvfgNodeId, SvfgNodeId);
    reaches_sink(src, snk) <-- source(src), flows_unsanitized(src, snk), sink(snk);
    relation multi_reach_finding(SvfgNodeId);
    multi_reach_finding(src) <-- agg n = count() in reaches_sink(src, _), if *n >= 2;
}
```

### Cross-checker suppression

The existing `suppress_cross_checker_findings()` logic moves into post-processing after all checker Ascent programs run. Same algorithm, just consuming Ascent outputs instead of BFS outputs.

## Unified Query Protocol

### Request schema

```json
{
  "action": "query",
  "language": "builtin" | "python" | "datalog",
  "source": "<checker name or rule text>",
  "params": {}
}
```

### Response schema

```json
{
  "status": "ok",
  "findings": [
    {
      "check": "<name>",
      "severity": "critical",
      "cwe": 416,
      "message": "...",
      "source": { "function": "main", "line": 12 },
      "sink": { "function": "main", "line": 15 },
      "path": [...]
    }
  ],
  "metadata": { "engine": "ascent", "elapsed_ms": 42 }
}
```

### Backward compatibility

Existing actions (`check`, `check_all`, `schema`) continue to work. Internally rewritten to the unified protocol.

## Phase 2: Datalog Interpreter (Deferred)

Lightweight semi-naive evaluator (~1-2k lines Rust):
- Parses simple Datalog syntax (rules, stratified negation, basic aggregation)
- Loads base facts exported from Ascent analysis results
- Evaluates to fixpoint
- Collects `bug(...)` tuples as findings
- No lattices, no BYODS, no parallelism (Ascent-only features)
- WASM-compatible (pure Rust, no dependencies)

User rules access pre-computed base facts:
- `flow(X, Y)` — SVFG edges
- `points_to(Ptr, Loc)` — PTA results
- `call_to(Node, Name)` — call sites
- `call_arg(Node, Name, Idx)` — call arguments
- `alloca(Node, Func)` — stack allocations
- `deref(Node, Kind)` — load/store sites
- `in_function(Node, Name)` — node-to-function mapping
- `source_loc(Node, File, Line)` — debug info

Convention: user rules write findings into `bug(Node, Severity?, Message?)` relation.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Ascent PTA slower than hand-tuned solver | Legacy solver behind `legacy-pta` feature flag. BYODS + `ascent_par!` + offline SCC recover performance. |
| Ascent library abandoned/buggy | MIT licensed, 159 commits, published at CC'22 + OOPSLA. Small enough to fork if needed. |
| BYODS union-find doesn't work for PTA | Fall back to pure lattice rules (slower but correct). Offline SCC handles static cycles. |
| Checker rules produce different results | Run both Ascent and BFS checkers on all test fixtures, require identical findings. |
| WASM binary size increase | Ascent compiles to native Rust — proc macro adds code, not runtime deps. Monitor WASM size. |
