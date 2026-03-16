# SAF Invariants Reference

Structural knowledge about SAF that rarely changes. These are the stable anchors
an agent needs regardless of what feature it is implementing.

---

## 1. Crate Dependency Graph

```
saf-core  (no internal deps -- foundation for everything)
  ^
  |--- saf-frontends  (Frontend trait + LLVM/AIR-JSON impls)
  |       ^
  |       |--- saf-analysis  (CFG, callgraph, PTA, valueflow, SVFG, checkers)
  |       |       ^
  |       |       |--- saf-cli      (CLI binary)
  |       |       |--- saf-python   (PyO3 bindings)
  |       |       |--- saf-bench    (benchmark harness)
  |       |
  |       |--- saf-wasm  (WebAssembly build)
  |
  |--- saf-datalog  (Datalog/Ascent analysis engine)
  |--- saf-test-utils  (shared test helpers)
  |--- saf-trace  (tutorial animation traces)
```

**Impact rule:** Changes in `saf-core` affect everything downstream. Changes in
leaf crates (`saf-cli`, `saf-python`, `saf-bench`) affect nothing else.

Source: `crates/*/Cargo.toml`

---

## 2. Docker Requirement

LLVM 18 is **only** available inside the Docker container (Ubuntu 24.04 LTS).

| Crate | Needs LLVM? | Safe to build locally? |
|-------|-------------|----------------------|
| `saf-core` | No | Yes |
| `saf-frontends` | Yes | No |
| `saf-analysis` | Yes (dev-deps) | No |
| `saf-cli` | Yes | No |
| `saf-python` | Yes | No |
| `saf-bench` | Yes | No |

**Build/test commands (always via Docker):**
- `make test` -- run all tests (Rust + Python)
- `make fmt && make lint` -- format then lint (always together)
- `make shell` -- interactive dev shell
- `docker compose run --rm dev sh -c '...'` -- run a specific command

**Never** run `cargo test` or `cargo build` locally for LLVM-dependent crates.
The only exception is `saf-core`.

---

## 3. Determinism Invariant (NFR-DET-001)

Byte-identical outputs for identical inputs. This is non-negotiable.

- **Collections:** Use `BTreeMap`/`BTreeSet` for deterministic iteration order.
  Never `HashMap`/`HashSet` in library code.
- **Exception:** `IndexMap` is allowed in hot paths only (PTA solver internals
  in `crates/saf-analysis/src/pta/solver.rs` and `constraint_index.rs`).
  `rustc_hash::FxHashMap` appears in frontends for build-time performance where
  output order is normalized afterward.
- **IDs:** All entity IDs are `u128`, derived from BLAKE3 hashes.
  Serialized as `0x` + 32 lowercase hex chars (e.g., `0x00a1b2c3...`).
- **ID generation:** `saf_core::id::make_id(domain, data)` and `id_to_hex(id)`.

Source: `crates/saf-core/src/id.rs`

---

## 4. AIR-Only Rule (NFR-EXT-001)

All analysis operates **only** on AIR (Analysis Intermediate Representation).
No frontend-specific types (LLVM, etc.) may leak into `saf-analysis`.

- The boundary is the `Frontend` trait: `crates/saf-frontends/src/api.rs`
- Frontends implement `fn ingest(&self, inputs, config) -> Result<AirBundle, FrontendError>`
- `AirBundle` is the canonical IR that all downstream analysis consumes
- `AirBundle` is defined in `crates/saf-core/src/air.rs`

This means: if you are writing analysis code in `saf-analysis`, you may never
import from `saf-frontends` (except in dev-dependencies for test fixtures).

---

## 5. No SVF Code Reuse (REQ-IP-001)

SAF must contain **independent implementations only**. No code from SVF (Static
Value-Flow analysis framework) may be reused or adapted. Algorithms may be the
same (Andersen's, CFL-reachability, IFDS, etc.) but every implementation must be
written from scratch.

---

## 6. Data Flow Pipeline

```
Input (.ll / .bc / .air.json)
  -> Frontend (implements Frontend trait)
    -> AirBundle (canonical IR)
      -> Graph builders (CFG, ICFG, CallGraph, DefUse)
        -> PTA solver (Andersen CI)
          -> ValueFlow graph
            -> SVFG (Sparse Value-Flow Graph)
              -> Checkers (bug finding, taint, etc.)
                -> Export (JSON, SARIF, PropertyGraph)
```

Key modules along this pipeline:
- `crates/saf-frontends/src/api.rs` -- Frontend trait
- `crates/saf-core/src/air.rs` -- AirBundle definition
- `crates/saf-analysis/src/cfg.rs` -- CFG builder
- `crates/saf-analysis/src/icfg.rs` -- ICFG builder
- `crates/saf-analysis/src/callgraph.rs` -- Call graph
- `crates/saf-analysis/src/defuse.rs` -- Def-use chains
- `crates/saf-analysis/src/pta/` -- Pointer analysis
- `crates/saf-analysis/src/valueflow/` -- Value-flow graph
- `crates/saf-analysis/src/svfg/` -- Sparse value-flow graph
- `crates/saf-analysis/src/checkers/` -- Bug checkers
- `crates/saf-analysis/src/export.rs` -- PropertyGraph export

---

## 7. PTA Constraint Extraction Triple-Update Rule

When adding a new constraint-generation step, you **must** update all three
extraction entry points. Missing one causes silent failures in callgraph
refinement or context-sensitive PTA.

The three functions (all in `crates/saf-analysis/src/pta/extract.rs`):

1. `extract_constraints()` -- whole-module extraction
2. `extract_constraints_reachable()` -- reachable-functions-only extraction
3. `extract_intraprocedural_constraints()` -- single-function extraction

If you add a new `extract_*_constraints` helper, call it from all three.

---

## 8. Key Types Quick Reference

| Type | Location | Purpose |
|------|----------|---------|
| `make_id` / `id_to_hex` | `saf_core::id` | BLAKE3 deterministic ID generation |
| `Config` | `saf_core::config` | Full configuration contract (SRS Section 6) |
| `AirBundle` | `saf_core::air` | Canonical IR bundle from frontend ingestion |
| `Frontend` | `saf_frontends::api` | Trait all frontends implement |
| `CoreError` | `saf_core::error` | Library error types (`thiserror`) |

---

## 9. PropertyGraph Export Format

All graph exports use a unified PropertyGraph JSON format:

```json
{
  "schema_version": "0.1.0",
  "graph_type": "<type>",
  "metadata": {},
  "nodes": [{"id": "0x...", "labels": [...], "properties": {...}}],
  "edges": [{"src": "0x...", "dst": "0x...", "edge_type": "...", "properties": {}}]
}
```

Graph types: `callgraph`, `cfg`, `defuse`, `valueflow`, `svfg`.
Exception: `pta.export()` uses its own format (not PropertyGraph).

---

## 10. SVFG Checker Boundary Pitfalls

Checker specs express instruction-level properties, but the SVFG solver operates
on SSA value nodes. Three known failure modes:

1. **Deduplication:** `BTreeSet<SvfgNodeId>` deduplicates same-SSA call sites
   (e.g., `free(p); free(p);` collapses to 1 node).
2. **Role namespace collision:** `ResourceRole` is flat -- roles shared across
   resource categories (memory/file/lock) cause checker cross-contamination.
3. **Zero-length flows:** Solver's `target != source` guard blocks flows where
   SSA collapses source and sink to the same node.

Source: `crates/saf-analysis/src/checkers/solver.rs`
