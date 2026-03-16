# Software Requirements Specification (SRS): Static Analyzer Factory
**Project name:** **Static Analyzer Factory** (“SAF”)  
**Primary implementation:** Rust core + Python SDK (PyO3)  
**Primary interface:** Python SDK (agent-friendly), plus CLI  
**Frontend architecture:** **Modular** (multiple frontends supported)  
- **MVP frontend:** LLVM Bitcode Frontend (`.bc`, optionally `.ll`)  
- **Contract frontend (for tests + future integrations):** AIR JSON Frontend (`.air.json`)  
- **Planned frontends (not required in MVP):** Source-level frontends (e.g., Clang AST / rust-analyzer)  

## 0. Document control
- **Status:** Draft
- **Audience:** contributors implementing the Rust core, the Python SDK, frontends, and the test harness
- **Methodology target:** Agile delivery with TDD (requirements are testable and mapped to acceptance tests)

## 1. End goal, purpose, and scope

### 1.1 End goal (north star)
Build an **agent-friendly** and **extensible** alternative to SVF-style workflows:
- Whole-program pointer analysis and value-flow reasoning (SVF-like core capability)
- A schema-driven Python API that AI agents can use to *author new analyzers quickly*
- A **frontend-agnostic** core so the same analysis pipeline can run over:
  1) LLVM-level IR (MVP) and  
  2) Source-level representations (future), without redesigning the analysis engine

### 1.2 Purpose
Provide a **factory** that turns program representations into analyzable graphs and exposes a stable, deterministic, queryable interface for building analyzers (taint, reachability, policy checks, etc.).

### 1.3 In-scope (MVP)
- A **Frontend API** and engine that can ingest programs via pluggable frontends.
- Implement **LLVM Bitcode Frontend**: ingest `.bc` (and optionally `.ll`) into a canonical IR used by analyses.
- Implement **AIR JSON Frontend**: ingest a simplified canonical IR from `.air.json` for:
  - unit/integration testing of analyses without LLVM
  - serving as a stable contract for future source-level frontends
- Build graph products: CFG, ICFG, CallGraph, SSA Def-Use, MemFlow, ValueFlow.
- Andersen-style context-insensitive points-to analysis (bounded field sensitivity).
- Reachability and taint-like source→sink analyses with sanitizers and deterministic trace extraction.
- Deterministic exports: JSON and SARIF.
- Caching keyed by (frontend input fingerprint, config, tool+schema versions).
- Python SDK with schema discoverability and structured errors.

### 1.4 Out-of-scope (MVP)
- Full-feature source frontends (Clang AST, rust-analyzer, etc.) **implementation**
  - **BUT**: the architecture MUST support adding them later (see §5.1.2).
- Exact SVF CLI/output compatibility.
- Full flow-sensitive / fully context-sensitive precision.
- Concurrency/MHP analysis; symbolic execution.

## 2. Relationship to SVF and reuse policy

### 2.1 Relationship to SVF (informative)
SAF targets a similar problem space as SVF (pointer/value-flow analysis), but it is not a drop-in replacement and does not aim for exact behavioral compatibility.

### 2.2 Reuse policy (normative)
**REQ-IP-001 (MUST):** The project team must not copy/paste, or port SVF source code into this codebase (including internal data-structure layouts or solver implementations).  
**REQ-IP-002 (MUST):** The project team may reuse SVF’s published ideas, high-level logic, and algorithms as conceptual guidance, provided the implementation is independently written in Rust and validated against this SRS and tests.

**Verification:** code review checklist + repository policy checks + passing tests.

## 3. Definitions
- **Frontend:** an ingestion module that converts an input program representation into SAF’s canonical IR (“AIR”) plus metadata.
- **AIR (Analysis Intermediate Representation):** SAF’s canonical, frontend-agnostic IR used by analyses. (Previously referred to as “CoreIR”.)
- **Determinism:** identical inputs/config/version must yield byte-identical outputs (JSON, SARIF, cache manifests).
- **Graph Store:** storage for typed graphs with deterministic iteration/serialization.
- **ValueFlow graph:** propagation graph combining SSA, memory, and interprocedural edges for reachability/taint.

## 4. System overview

### 4.1 Components
1. **Engine**
   - Orchestrates pipeline stages
   - Maintains cache and versioned schemas
   - Provides CLI and Python entry points

2. **Frontend API**
   - Uniform ingestion contract producing AIR + metadata
   - Supports multiple frontends (compiled-in for MVP)

3. **Frontends**
   - **LLVM Bitcode Frontend (MVP):** reads `.bc`/`.ll`
   - **AIR JSON Frontend (MVP):** reads `.air.json`
   - **Source frontends (planned):** e.g., Clang AST, rust-analyzer

4. **AIR (canonical IR)**
   - Language-agnostic IR sufficient for pointer/value-flow analyses
   - Uses stable IDs for entities
   - Supports optional source-level constructs and metadata for future frontends

5. **Graph Store**
   - CFG/ICFG/CallGraph/DefUse/MemFlow/ValueFlow, deterministic export

6. **Analyses**
   - CFG/ICFG + CallGraph
   - DefUse builder (SSA)
   - Points-to (Andersen CI)
   - ValueFlow builder
   - Reachability/taint + deterministic trace extraction

7. **Python SDK (PyO3)**
   - Schema discoverability, query API, selectors/presets, exports

### 4.2 External interfaces
- Inputs:
  - LLVM frontend: `.bc` (required), `.ll` (optional)
  - AIR JSON frontend: `.air.json` (contract format)
- Outputs:
  - Deterministic JSON exports (graphs/findings/schema)
  - SARIF findings
- APIs:
  - Python package (import name): `saf`
  - CLI binary: `saf`

---

# 5. Requirements

## 5.1 Non-functional requirements (NFR)

### 5.1.1 Determinism and reproducibility
**NFR-DET-001 (MUST):** For identical *frontend input fingerprint*, identical config, identical tool version, identical schema version, and identical canonicalization settings, SAF shall produce byte-identical:
- JSON exports (graphs/findings/schema)
- SARIF outputs
- cache keys and meta manifests

**NFR-DET-002 (MUST):** Outputs shall not contain timestamps or wall-clock-dependent data.

**NFR-DET-003 (MUST):** All serialized collections shall have deterministic ordering:
- Nodes sorted by `(node_kind, referenced_id_hex)`
- Edges sorted by `(edge_kind, src_id_hex, dst_id_hex, label_hash_hex)`
- Findings sorted by `(sink_location, source_location, rule_id, trace_hash)`

**NFR-DET-004 (MUST):** Each frontend shall define a deterministic `input_fingerprint_bytes()` (see FR-FE-003). Debug info and absolute paths shall not affect fingerprints by default.

**Acceptance tests**
- AT-DET-01: same input+config+version run twice → byte-identical outputs
- AT-DET-02: `--paths.strip-prefix` changes only path strings, not IDs/order

### 5.1.2 Extensibility: frontend modularity and source-level readiness
This is the key requirement to meet the “LLVM IR now, source-level later” end goal.

**NFR-EXT-001 (MUST):** The analysis engine and graph builders shall operate **only** on AIR and shared metadata interfaces. They must not call frontend-specific APIs (LLVM APIs, Clang APIs, etc.).  
**NFR-EXT-002 (MUST):** Adding a new frontend shall not require changes to analysis algorithms; only the ingestion mapping into AIR and metadata adapters may be added/changed.  
**NFR-EXT-003 (MUST):** AIR shall include optional fields to preserve source-level constructs (names, types, lexical scopes, spans) so future source-level analysis can be performed without redesigning the data model.

**Acceptance tests**
- AT-EXT-01: run the same reachability query on an LLVM-produced AIR and an AIR-JSON-produced AIR fixture representing the same program; results are identical (modulo location strings).

### 5.1.3 Safety, robustness, and resource caps
**NFR-SAFE-001 (MUST):** Treat inputs as untrusted; validate sizes to prevent OOM/crashes.  
**NFR-SAFE-002 (MUST):** Honor caps: `pta.max_objects`, `flow.max_depth`, `flow.max_paths`. If capped, emit diagnostics and conservatively collapse to `unknown_mem`.

### 5.1.4 Observability
**NFR-OBS-001 (MUST):** CLI supports `--json-errors` machine-readable errors.  
**NFR-OBS-002 (SHOULD):** Stage timings are logged (non-exported, no timestamps in artifacts).

---

## 5.2 Functional requirements (FR)

## 5.2.1 Frontend API and ingestion
**FR-FE-001 (MUST):** SAF shall support selecting a frontend via CLI and Python:
- CLI: `saf index --frontend llvm` / `--frontend air-json`
- Python: `Project.open(..., frontend="llvm")`

**FR-FE-002 (MUST):** Frontends shall implement a common contract:
- `ingest(inputs, config) -> AirBundle`
- `input_fingerprint_bytes(inputs, config) -> bytes` (deterministic, path-normalized)
- `supported_features() -> dict` (schema discoverability)

**FR-FE-003 (MUST):** The engine shall compute the cache key using:
`hash(frontend_id, input_fingerprint_bytes, canonical_config_json, tool_version, schema_version)`.

**FR-FE-004 (MUST):** Provide two built-in frontends for MVP:
- **LLVM Bitcode Frontend**
- **AIR JSON Frontend** (`.air.json`) that can represent small programs for tests.

**FR-FE-005 (MUST):** Frontends shall provide best-effort source mapping metadata:
- `Span { file_id, byte_start, byte_end, line_start, col_start, line_end, col_end }` (when available)
- `Symbol { display_name, mangled_name, namespace_path }` (when available)

**Acceptance tests**
- AT-FE-01: `air-json` frontend loads fixture and produces valid AIR + schema.
- AT-FE-02: `llvm` frontend loads `.bc` fixture and produces valid AIR + schema.
- AT-FE-03: cache key changes when config changes; stable when only paths are stripped.

## 5.2.2 AIR (Analysis Intermediate Representation)
**FR-AIR-001 (MUST):** SAF shall define a canonical AIR data model sufficient for pointer/value-flow analyses and frontend extensibility.

**FR-AIR-002 (MUST):** AIR entities must have deterministic `u128` IDs serialized as lowercase hex with `0x` prefix. Hash function: BLAKE3 preferred (or SHA-256 truncated to 128 bits).  
**FR-AIR-003 (MUST):** Debug info must not contribute to structural IDs by default.

### 5.2.2.1 Canonical ordering and indices
**FR-AIR-004 (MUST):** Each frontend must define canonical ordering for:
- functions within a module
- basic blocks within a function
- instructions within a block
The ordering must be deterministic and documented per frontend.

### 5.2.2.2 ID derivation (normative)
**FR-AIR-005 (MUST):** IDs shall be derived from frontend-agnostic rules:

- `FrontendId`: stable string (`"llvm"`, `"air-json"`, `"clang-ast"`…)
- `ModuleFingerprint = hash(FrontendId, frontend_input_fingerprint_bytes)`
- `FunctionId = hash(ModuleFingerprint, "fn", stable_function_key)`
- `BlockId = hash(FunctionId, "bb", block_index)`
- `InstId = hash(BlockId, "inst", inst_index, opcode_tag)`
- `ValueId`: instruction results, args, globals, constants (canonical const repr)
- `ObjId`: stack allocas, heap allocators, globals, plus fixed `unknown_mem`
- `LocId = hash(ObjId, "loc", field_path_repr)` (canonicalized)

**Notes**
- For LLVM frontend, `frontend_input_fingerprint_bytes` is the bitcode bytes.
- For source frontends, the fingerprint SHOULD include normalized source contents + compilation options in a stable form.

### 5.2.2.3 Minimum operation set (MVP)
**FR-AIR-006 (MUST):** AIR shall represent at least these operations:
- Allocation: `Alloca`, `Global`, `HeapAlloc(kind)`
- Memory: `Load`, `Store`, `GEP(field_path|unknown)`, `Memcpy`, `Memset`
- Control: `Br`, `Switch`, `Ret`
- SSA: `Phi`, `Select`
- Calls: `CallDirect`, `CallIndirect` (conservative target set)
- Transforms: `Cast`, `BinaryOp` (as value transforms)

**FR-AIR-007 (SHOULD):** AIR should support attaching optional *source-level* metadata on each instruction/value:
- `span` (Span)
- `symbol` (Symbol)
- `type_repr` (frontend-specific string, stable if possible)
This enables future source-level analysis without redesigning AIR.

## 5.2.3 Graph store
**FR-GRAPH-001 (MUST):** SAF shall maintain typed graphs with stable node/edge IDs and deterministic iteration order.  
**FR-GRAPH-002 (MUST):** Graph store shall export canonical JSON for supported graphs and optional DOT.

**Required graphs (MVP)**
- CFG (per function)
- ICFG (whole program, instruction-level)
- CallGraph
- SSA DefUseGraph
- MemFlowGraph
- ValueFlowGraph
- PointsToConstraintGraph (optional export)

## 5.2.4 Call modeling and externals
**FR-CALL-001 (MUST):** For direct calls, add callgraph edges, ICFG call/return edges, and ValueFlow actual→formal and return→caller edges.  
**FR-CALL-002 (MUST):** For unresolved/indirect calls, route through an `ExternalSummary` node and apply configurable side-effects.

## 5.2.5 Points-to analysis (Andersen, context-insensitive)
**FR-PTA-001 (MUST):** Implement Andersen-style inclusion-based, context-insensitive points-to analysis with bounded field sensitivity.  
**FR-PTA-002 (MUST):** Constraint extraction includes Addr/Copy/Load/Store/GEP.  
**FR-PTA-003 (MUST):** Solver is deterministic (stable worklist ordering, stable constraint processing).  
**FR-PTA-004 (MUST):** Expose:
- `points_to(ptr) -> sorted list[ObjOrLoc]`
- `may_alias(p,q) -> bool`

## 5.2.6 Value flow and reachability/taint analysis
**FR-FLOW-001 (MUST):** Construct a ValueFlowGraph with edges:
- SSA DEF_USE (including phi)
- TRANSFORM (casts/arithmetic propagation)
- CALL_ARG (actual→formal)
- RETURN (callee ret→caller result)
- STORE/LOAD via PTA locations (fallback `unknown_mem`)

**FR-FLOW-002 (MUST):** Provide two modes:
- `precise`: uses PTA-derived locations
- `fast`: bounded memory abstraction (still deterministic)

**FR-FLOW-003 (MUST):** Provide queries:
- `flows(sources, sinks, limits, mode) -> list[Flow]`
- `taint_flow(sources, sinks, sanitizers, limits, mode) -> list[Finding]`

**FR-FLOW-004 (MUST):** Trace extraction uses BFS with deterministic tie-breaking:
expand outgoing edges ordered by `(edge_kind, dst_id_hex)`.

## 5.2.7 Python SDK
**FR-PY-001 (MUST):** Provide Python package (import name `saf`) built with `maturin` + `pyo3`.  
**FR-PY-002 (MUST):** Expose:
- `Project.open(path, cache_dir=None, config=Config(), frontend="llvm")`
- `Project.schema() -> dict`
- `Project.query() -> Query`
- `Project.explain(entity_id: str) -> dict`

**FR-PY-003 (MUST):** `Query` includes:
- `points_to`, `may_alias`
- `flows`, `taint_flow`
- `export_graph(name) -> dict`

**FR-PY-004 (MUST):** Provide composable selectors/presets for sources/sinks/sanitizers.

## 5.2.8 CLI
**FR-CLI-001 (MUST):** Provide binary `saf` with commands:
- `index`, `run`, `query`, `export`, `schema`  
**FR-CLI-002 (MUST):** CLI exits nonzero on errors; supports `--json-errors`.

## 5.2.9 Schema discoverability
**FR-SCHEMA-001 (MUST):** `Project.schema()` returns a stable schema including:
- tool version + schema version
- supported frontends and their features
- supported graphs (node/edge kinds)
- supported queries/args
- selector helpers/presets
- config fields + defaults

## 5.2.10 Output formats
**FR-OUT-001 (MUST):** Findings JSON includes stable `finding_id`, rule metadata, source/sink locations, deterministic trace.  
**FR-OUT-002 (MUST):** SARIF outputs follow canonicalization:
- no timestamps
- stable driver name/version
- deterministic fingerprints and ordering
- normalized paths via config

## 5.2.11 Caching
**FR-CACHE-001 (MUST):** Cache key:
`hash(frontend_id, frontend_input_fingerprint_bytes, canonical_config_json, tool_version, schema_version)`  
**FR-CACHE-002 (MUST):** Cache stores AIR, optional graph materializations, PTA results, and meta manifest with hashes/versions.

---

# 6. Configuration contract (normative)
Config is JSON and hashed into cache keys.
- `frontend`: `"llvm" | "air-json"` (default `llvm`)
- `analysis.mode`: `"fast" | "precise"` (default `precise`)
- `pta.enabled`: bool
- `pta.field_sensitivity`: `"none" | "struct_fields"`
- `pta.max_objects`: int
- `flow.max_depth`: int
- `flow.max_paths`: int
- `external_side_effects`: `"none" | "unknown_write" | "unknown_readwrite"` (default `unknown_readwrite`)
- `paths.strip_prefixes`: list[str]
- `paths.normalize_separators`: bool
- `rust.demangle`: bool

---

# 7. Agile delivery plan (epics + definition of done)

## 7.1 Epics (MVP)
- **E0 Scaffolding:** workspace, CLI skeleton, config, deterministic JSON utilities
- **E1 Frontend API + AIR JSON frontend:** contract format + tests
- **E2 LLVM frontend:** `.bc` → AIR + source mapping best-effort
- **E3 Graph builders:** CFG/ICFG/CallGraph/DefUse + export
- **E4 PTA:** constraint extraction + solver + queries
- **E5 ValueFlow + taint:** flows + sanitizers + deterministic traces + SARIF
- **E6 Python SDK v1:** stable API, selectors, examples

## 7.2 Definition of done (per epic)
A story/feature is “Done” when:
- unit tests + integration tests for the relevant requirement IDs pass
- determinism tests pass for affected outputs
- public APIs are documented + an executable example exists
- schema_version bump is performed for any breaking schema change

---

# 8. Verification and test plan (TDD-first)

## 8.1 Test suite structure
- Rust unit tests: ID stability, ordering, constraint extraction, solver, trace tie-breaking
- Integration tests (LLVM): curated C and Rust fixtures compiled to `.bc`
- Integration tests (AIR JSON): `.air.json` fixtures representing equivalent programs
- Determinism tests: repeated runs produce byte-identical JSON + SARIF

## 8.2 Fixture requirements (MVP)
Each fixture includes:
- LLVM path: build script to produce `.bc`
- AIR JSON path: `.air.json` equivalent
- expected finding count and key trace properties

Suggested fixtures:
- C: `argv → system`, `argv → printf format`, `getenv → fopen`
- Rust: `env::args → Command::status`, `env::var → Command::spawn`

---

# Appendix A. MVP Python example (must work)
```python
from saf import Project, sources, sinks, sanitizers

proj = Project.open("app.bc", cache_dir=".cache/saf", frontend="llvm")
q = proj.query()

findings = q.taint_flow(
    sources=sources.argv() | sources.getenv(),
    sinks=sinks.call(r"(printf|sprintf|snprintf)", arg_index=0),
    sanitizers=sanitizers.call(r"(sanitize|escape)", arg_index=0),
).run(mode="precise")

for f in findings:
    print(f.rule_id, f.title)
    print(f.source_location, "->", f.sink_location)
    print(f.trace.pretty())
    f.to_sarif("out.sarif")
```

# Appendix B. AIR JSON contract (MVP, normative sketch)
The `.air.json` format is a stable, minimal representation of AIR for tests and external frontend integration.
It must include: `frontend_id`, `module`, `functions`, `blocks`, `instructions`, `values`, `objects`, and optional `spans`.
Schema details are versioned by `schema_version` and returned by `Project.schema()`.
