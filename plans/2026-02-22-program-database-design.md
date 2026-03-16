# ProgramDatabase: Graph-First Analysis Framework

*Design document for SAF's unified analysis interface*
*Created: 2026-02-22*

---

## 1. Overview

This document describes the design of `ProgramDatabase` — a two-phase architecture for SAF where **graphs are built first, then queried**. The design provides:

1. A `ProgramDatabase` that owns all precomputed program analysis graphs
2. A JSON-based protocol for LLM agents to query the database
3. Neo4j integration for Cypher-based graph exploration
4. A declarative `AnalysisConfig` schema for property checking

The target audience is LLM agents performing general program analysis — not just vulnerability detection, but code understanding, impact analysis, slicing, metrics, and exploration.

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     LLM Agent                             │
│  Sends JSON requests, receives JSON responses             │
│                                                          │
│  {"action": "check", "name": "use_after_free"}           │
│  {"action": "cypher", "query": "MATCH ..."}              │
│  {"action": "analyze", "config": {...}}                  │
└────────────────────┬─────────────────────────────────────┘
                     │ JSON Protocol
                     ▼
┌──────────────────────────────────────────────────────────┐
│                   SAF API Layer                           │
│  Validates requests, dispatches to appropriate engine      │
│                                                          │
│  ┌────────────┐ ┌────────────┐ ┌───────────────────────┐│
│  │ check()    │ │ analyze()  │ │ cypher()              ││
│  │ check_all()│ │            │ │ export_to_neo4j()     ││
│  │ schema()   │ │            │ │                       ││
│  └─────┬──────┘ └──────┬─────┘ └───────────┬───────────┘│
│        │               │                   │             │
│        ▼               ▼                   ▼             │
│  ┌─────────────────────────┐  ┌────────────────────────┐│
│  │   Query Decomposition   │  │      Neo4j Client      ││
│  │   Engine                │  │   (Python neo4j driver) ││
│  │                         │  │                        ││
│  │   Decomposes declarative│  │   PropertyGraph JSON   ││
│  │   configs into graph    │  │   → Neo4j import       ││
│  │   primitive calls       │  │   → Cypher execution   ││
│  └──────────┬──────────────┘  └────────────────────────┘│
│             │                                            │
│             ▼                                            │
│  ┌──────────────────────────────────────────────────┐   │
│  │              ProgramDatabase (Rust)               │   │
│  │                                                   │   │
│  │  In-memory graphs (built eagerly):                │   │
│  │    CallGraph, ICFG, PTA, DefUse, ValueFlow        │   │
│  │                                                   │   │
│  │  Lazy graphs (built on first access):             │   │
│  │    SVFG, MemSSA                                   │   │
│  │                                                   │   │
│  │  Graph primitives:                                │   │
│  │    pattern_match, reachability, may_alias,         │   │
│  │    can_reach, value_at, must_reach,               │   │
│  │    forward_slice, backward_slice                  │   │
│  │                                                   │   │
│  │  Owned data:                                      │   │
│  │    Arc<AirModule>, AnalyzedSpecRegistry, Config    │   │
│  └──────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

### Two Phases

**Phase 1 — Build:** `ProgramDatabase::build(module, config)` constructs all core graphs (CFG, CG, PTA, DefUse, ValueFlow) eagerly. SVFG and MemSSA are built lazily on first access. This wraps the existing `run_pipeline()` internally.

**Phase 2 — Query:** All analysis is expressed as queries over the precomputed graphs. Queries come in through the JSON protocol and are either decomposed into graph primitive calls (for property checking) or forwarded to Neo4j (for Cypher exploration).

---

## 3. ProgramDatabase (Rust)

### Struct

```rust
// crates/saf-analysis/src/database.rs

pub struct ProgramDatabase {
    // Owned program representation
    module: Arc<AirModule>,
    specs: AnalyzedSpecRegistry,
    config: Config,

    // Core graphs (built eagerly by build())
    call_graph: CallGraph,
    icfg: Icfg,
    pta_result: PtaResult,
    defuse: DefUseGraph,
    valueflow: ValueFlowGraph,

    // Advanced graphs (built lazily on first access)
    svfg: OnceLock<Svfg>,
    mssa: OnceLock<MemorySsa>,

    // Pipeline metadata
    stats: PipelineStats,
}
```

### Build

```rust
impl ProgramDatabase {
    pub fn build(module: AirModule, config: &Config) -> Result<Self, CoreError> {
        let pipeline = run_pipeline(&module, &PipelineConfig::from(config))?;
        let specs = build_analyzed_specs(&module, &pipeline.pta_result);
        Ok(Self {
            module: Arc::new(module),
            specs,
            config: config.clone(),
            call_graph: pipeline.call_graph,
            icfg: pipeline.icfg,
            pta_result: pipeline.pta_result,
            defuse: pipeline.defuse,
            valueflow: pipeline.valueflow,
            svfg: OnceLock::new(),
            mssa: OnceLock::new(),
            stats: pipeline.stats,
        })
    }
}
```

### Graph Primitives

```rust
impl ProgramDatabase {
    // === Pattern matching ===
    /// Resolve a declarative pattern to concrete AIR node IDs
    pub fn pattern_match(&self, pattern: &SitePattern) -> BTreeSet<NodeId>;

    // === Traversal ===
    /// Reachability on any graph, forward or backward
    pub fn reachability(
        &self, graph: GraphRef, direction: Direction,
        from: &[NodeId], to: &[NodeId], barriers: &[NodeId],
    ) -> Vec<Path>;

    /// Must-reach: does required occur on ALL paths from → to?
    pub fn must_reach(
        &self, from: &[NodeId], to: &[NodeId], required: &[NodeId],
    ) -> Vec<Violation>;

    // === Pointer analysis ===
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult;
    pub fn points_to(&self, ptr: ValueId) -> Vec<LocId>;

    // === Value analysis ===
    pub fn value_at(&self, var: ValueId, point: InstId) -> Interval;

    // === Slicing ===
    pub fn forward_slice(&self, from: ValueId) -> Slice;
    pub fn backward_slice(&self, to: ValueId) -> Slice;
    pub fn chop(&self, from: ValueId, to: ValueId) -> Slice;

    // === Structural ===
    pub fn node_info(&self, node: NodeId) -> NodeInfo;
    pub fn neighborhood(&self, graph: GraphRef, center: NodeId, depth: usize) -> SubGraph;

    // === Export ===
    pub fn export_graphs(&self) -> Vec<PropertyGraph>;
}

pub enum GraphRef { ValueFlow, Svfg, CallGraph, Cfg(FunctionId), Icfg }
pub enum Direction { Forward, Backward }
```

### Memory Estimates

| Program Size | Estimated Memory (compact PTA) |
|-------------|-------------------------------|
| 10K LOC     | ~50 MB                        |
| 50K LOC     | ~150 MB                       |
| 200K LOC    | ~500 MB                       |
| 1M LOC      | ~2-4 GB                       |

PTA dominates memory. Using `BitVecPtsSet` or `RoaringBitmap` (already supported) keeps memory reasonable for programs up to 200K LOC.

---

## 4. JSON Protocol

All LLM ↔ SAF communication uses structured JSON with a formal schema.

### Request Types

#### 4.1 Schema Discovery

```json
→ {"action": "schema"}

← {
    "status": "ok",
    "checks": [
      {"name": "use_after_free", "description": "Detects dereference of freed heap memory", "cwe": 416, "severity": "error"},
      {"name": "double_free", "description": "Detects double-free of heap memory", "cwe": 415, "severity": "error"},
      {"name": "null_deref", "description": "Detects null pointer dereference", "cwe": 476, "severity": "error"},
      {"name": "memory_leak", "description": "Detects unreleased heap allocations", "cwe": 401, "severity": "warning"},
      {"name": "sql_injection", "description": "Detects unsanitized data reaching SQL queries", "cwe": 89, "severity": "error"},
      {"name": "buffer_overflow", "description": "Detects array access beyond bounds", "cwe": 120, "severity": "error"},
      {"name": "integer_overflow", "description": "Detects arithmetic overflow/underflow", "cwe": 190, "severity": "warning"},
      {"name": "division_by_zero", "description": "Detects potential division by zero", "cwe": 369, "severity": "error"},
      {"name": "file_descriptor_leak", "description": "Detects unclosed file descriptors", "cwe": 775, "severity": "warning"},
      {"name": "uninit_use", "description": "Detects use of uninitialized memory", "cwe": 908, "severity": "error"}
    ],
    "templates": [
      {"name": "taint", "params": {"source": "string", "sink": "string", "barrier": "string?"},
       "description": "Custom taint flow: source → sink with optional barrier"},
      {"name": "resource_leak", "params": {"acquire": "string", "release": "string"},
       "description": "Resource leak: acquire without corresponding release on all paths"},
      {"name": "typestate", "params": {"resource": "string", "states": "list[string]", "transitions": "dict", "error_at_exit": "list[string]"},
       "description": "Custom state machine analysis"}
    ],
    "queries": ["alias", "points_to", "reachable", "value_at", "slice"],
    "graphs": ["cfg", "callgraph", "defuse", "valueflow", "pta", "svfg"]
  }
```

#### 4.2 Open Program

```json
→ {
    "action": "open",
    "path": "/path/to/program.ll",
    "config": {
      "pta_max_iterations": 100,
      "field_sensitivity_depth": 3
    }
  }

← {
    "status": "ok",
    "program": {
      "functions": 42,
      "instructions": 5000,
      "pointer_values": 1200,
      "graphs_built": ["cfg", "callgraph", "pta", "defuse", "valueflow"]
    }
  }
```

#### 4.3 Named Check (Tier 0)

```json
→ {"action": "check", "name": "use_after_free"}

← {
    "status": "ok",
    "findings": [
      {
        "check": "use_after_free",
        "severity": "error",
        "cwe": 416,
        "message": "Pointer freed at main.c:15, dereferenced at main.c:20",
        "path": [
          {"location": "main.c:10", "event": "allocated"},
          {"location": "main.c:15", "event": "freed", "state": "Freed"},
          {"location": "main.c:20", "event": "dereferenced", "state": "Freed"}
        ],
        "object": "heap allocation at main.c:10"
      }
    ],
    "metadata": {"elapsed_ms": 1200, "engines_used": ["pta", "svfg"]}
  }
```

#### 4.4 Template Check (Tier 1)

```json
→ {
    "action": "check",
    "name": "taint",
    "params": {
      "source": "parse_request",
      "sink": "db_execute",
      "barrier": "sanitize_sql"
    }
  }

← {
    "status": "ok",
    "findings": [...]
  }
```

#### 4.5 Full Analysis Config (Tier 2)

```json
→ {
    "action": "analyze",
    "config": {
      "name": "validate-before-process",
      "severity": "error",
      "sources": [{"call": {"name": "recv_msg", "match_return": true}}],
      "sinks": [{"call": {"name": "process_msg"}}],
      "barriers": [{"call": {"name": "validate_msg"}}]
    }
  }

← {
    "status": "ok",
    "findings": [
      {
        "check": "validate-before-process",
        "severity": "error",
        "message": "process_msg called without validate_msg on error path",
        "path": [
          {"location": "protocol.c:45", "event": "recv_msg returns"},
          {"location": "protocol.c:48", "event": "branch (error path)"},
          {"location": "protocol.c:52", "event": "process_msg called"}
        ]
      }
    ]
  }
```

#### 4.6 Cypher Query

```json
→ {
    "action": "cypher",
    "query": "MATCH (caller:Function)-[:CALLS]->(f:Function {name: 'free'}) RETURN caller.name ORDER BY caller.name"
  }

← {
    "status": "ok",
    "results": [
      {"caller.name": "cleanup"},
      {"caller.name": "destroy_session"},
      {"caller.name": "free_buffer"}
    ]
  }
```

#### 4.7 Graph Primitive Query

```json
→ {
    "action": "query",
    "type": "alias",
    "params": {"p": "0x1a2b3c4d...", "q": "0x5e6f7a8b..."}
  }

← {"status": "ok", "result": "may"}
```

```json
→ {
    "action": "query",
    "type": "slice",
    "params": {"direction": "backward", "target": "0x1a2b3c4d..."}
  }

← {
    "status": "ok",
    "slice": {
      "nodes": ["0x1a2b...", "0x3c4d...", "0x5e6f..."],
      "edges": [...]
    }
  }
```

#### 4.8 Error Response

```json
← {
    "status": "error",
    "error": {
      "code": "UNKNOWN_CHECK",
      "message": "No check named 'use_ater_free'. Did you mean 'use_after_free'?",
      "suggestions": ["use_after_free"]
    }
  }
```

---

## 5. Neo4j Integration

### Architecture

Neo4j serves as a **read-only graph exploration layer**. Analysis engines use in-memory graphs for performance; Neo4j provides Cypher queries for ad-hoc exploration.

```
ProgramDatabase::build()
    │
    ├─ In-memory graphs (Rust)     ← used by check/analyze (fast)
    │
    └─ export_graphs() → PropertyGraph JSON
         │
         ▼
    Neo4j import (Python)          ← used by cypher() (flexible)
```

### Neo4j Graph Schema

SAF's `PropertyGraph` export maps directly to Neo4j's property graph model:

**Node labels:**

| Label | Source Graph | Properties |
|-------|-------------|------------|
| `:Function` | CallGraph | `name`, `is_external`, `param_count` |
| `:Block` | CFG | `function_id`, `is_entry`, `is_exit` |
| `:Value` | DefUse, ValueFlow | `value_id`, `type_name`, `function` |
| `:Location` | PTA | `loc_id`, `kind` |
| `:Instruction` | AIR | `opcode`, `function`, `block`, `source_line` |

**Relationship types:**

| Type | Source Graph | Properties |
|------|-------------|------------|
| `:CALLS` | CallGraph | — |
| `:FLOWS_TO` | CFG | — |
| `:DEFINES` | DefUse | — |
| `:USED_BY` | DefUse | — |
| `:POINTS_TO` | PTA | — |
| `:VALUE_FLOW` | ValueFlow | `edge_kind` (Direct, Store, Load, ...) |

### Import Pipeline

```python
class Neo4jExporter:
    def __init__(self, uri="bolt://localhost:7687"):
        self.driver = neo4j.GraphDatabase.driver(uri)

    def export(self, project):
        """Export all ProgramDatabase graphs to Neo4j."""
        graphs = project.database.export_graphs()
        with self.driver.session() as session:
            # Create indexes for query performance
            session.run("CREATE INDEX IF NOT EXISTS FOR (n:Function) ON (n.name)")
            session.run("CREATE INDEX IF NOT EXISTS FOR (n:Value) ON (n.id)")
            session.run("CREATE INDEX IF NOT EXISTS FOR (n:Location) ON (n.id)")
            session.run("CREATE INDEX IF NOT EXISTS FOR (n:Block) ON (n.id)")

            # Batch import using UNWIND for performance
            for graph in graphs:
                nodes = graph["nodes"]
                edges = graph["edges"]

                # Batch create nodes
                session.run(
                    "UNWIND $nodes AS n "
                    "CALL apoc.create.node(n.labels, n.properties) YIELD node "
                    "SET node.id = n.id",
                    nodes=nodes
                )

                # Batch create relationships
                session.run(
                    "UNWIND $edges AS e "
                    "MATCH (src {id: e.src}), (dst {id: e.dst}) "
                    "CALL apoc.create.relationship(src, e.edge_type, e.properties, dst) "
                    "YIELD rel RETURN count(rel)",
                    edges=edges
                )
```

### Docker Compose

```yaml
services:
  dev:
    # ... existing dev container
    depends_on:
      - neo4j
    environment:
      NEO4J_URI: bolt://neo4j:7687

  neo4j:
    image: neo4j:5-community
    ports:
      - "7474:7474"   # Neo4j Browser UI
      - "7687:7687"   # Bolt protocol
    environment:
      NEO4J_AUTH: none
      NEO4J_PLUGINS: '["apoc"]'
    volumes:
      - neo4j-data:/data

volumes:
  neo4j-data:
```

### Example Cypher Queries for LLM Agents

```cypher
-- Find all taint flows (data from read() reaching exec())
MATCH path = (src:Value)-[:VALUE_FLOW*]->(sink:Value)
WHERE src.function = 'read' AND sink.function = 'exec'
RETURN path LIMIT 10

-- Call chain analysis
MATCH (caller:Function)-[:CALLS*]->(callee:Function {name: 'free'})
RETURN caller.name, length(shortestPath((caller)-[:CALLS*]->(callee))) AS depth
ORDER BY depth

-- Impact analysis: what's affected by changing function X?
MATCH (target:Function {name: 'process_data'})
MATCH (caller:Function)-[:CALLS*]->(target)
RETURN DISTINCT caller.name AS affected_function

-- Dead code: functions not reachable from main
MATCH (main:Function {name: 'main'})
MATCH (f:Function)
WHERE NOT EXISTS { MATCH (main)-[:CALLS*]->(f) } AND f.name <> 'main'
RETURN f.name AS dead_function

-- Fan-in/fan-out metrics
MATCH (f:Function)
OPTIONAL MATCH (f)-[:CALLS]->(callee:Function)
OPTIONAL MATCH (caller:Function)-[:CALLS]->(f)
RETURN f.name, count(DISTINCT callee) AS fan_out, count(DISTINCT caller) AS fan_in
ORDER BY fan_out DESC

-- Find all aliases of a pointer
MATCH (p:Value {id: $ptr_id})-[:POINTS_TO]->(loc:Location)
MATCH (q:Value)-[:POINTS_TO]->(loc)
WHERE q.id <> $ptr_id
RETURN q.id, q.function
```

---

## 6. Query Decomposition Engine

The decomposition engine translates declarative `AnalysisConfig` requests into graph primitive operations.

### Decomposition Rules

The dispatcher examines which fields are present in the config:

```
has_sources + has_sinks only?          → Pattern 1: Reachability
  → pattern_match(sources) + pattern_match(sinks)
  → reachability(valueflow, Forward, src_nodes, snk_nodes, barriers)

has_bind_vars + has_flow_states?       → Pattern 2: Alias + State
  → pattern_match(sources) + pattern_match(sinks)
  → may_alias(bound_var_at_src, bound_var_at_snk)  // filter pairs
  → typestate solver on alias-connected pairs

has_flow_states + sinks_are_exit?      → Pattern 4: Must-Reach
  → pattern_match(sources)
  → must_reach(src_nodes, exit_nodes, release_nodes)

has_numeric_check only?                → Pattern 5: Numeric
  → pattern_match(sinks)
  → value_at(operand, sink_point)  // check condition

has_sources + has_numeric_check?       → Pattern 3: Reachability + Numeric
  → pattern_match(sources) + pattern_match(sinks)
  → reachability(valueflow, Forward, src, snk)
  → for each reached sink: value_at(operand, point)

has_flow_states + has_numeric_check?   → Pattern 6: State + Numeric
  → Pattern 2 (state tracking) + Pattern 5 (numeric check at error states)
```

### Pattern Coverage

| Pattern | Primitives Used | CWEs Covered |
|---------|----------------|--------------|
| 1. Reachability | pattern_match, reachability | CWE-89, 79, 78, 22, 134, 457 |
| 2. Alias + State | pattern_match, may_alias, typestate | CWE-416, 415, 667 |
| 3. Reachability + Numeric | pattern_match, reachability, value_at | CWE-120 (tainted) |
| 4. Must-Reach | pattern_match, must_reach | CWE-401, 775, 764 |
| 5. Numeric | pattern_match, value_at | CWE-190, 369, 476 |
| 6. State + Numeric | pattern_match, may_alias, typestate, value_at | CWE-120 (heap) |

---

## 7. AnalysisConfig Schema

### Rust Types

```rust
pub struct AnalysisConfig {
    pub name: String,
    pub description: Option<String>,
    pub cwe: Option<u32>,
    pub severity: Severity,
    pub sources: Option<Vec<SiteSpec>>,
    pub sinks: Option<Vec<SiteSpec>>,
    pub barriers: Option<Vec<SiteSpec>>,
    pub bind_variables: Option<BTreeMap<String, BindSpec>>,
    pub flow_states: Option<FlowStateConfig>,
    pub numeric_check: Option<NumericCondition>,
}

pub enum SiteSpec {
    Call { name: String, arg: Option<u32>, match_return: bool },
    Deref { bind_var: String },
    FunctionExit,
    Role { role: ResourceRole },
    Custom { predicate: String },
}

pub struct BindSpec {
    pub at: BindSite,     // source | sink
    pub what: BindTarget, // arg(n) | return_value | deref
}

pub struct FlowStateConfig {
    pub states: Vec<String>,
    pub initial: String,
    pub transitions: Vec<Transition>,
    pub error: ErrorCondition,
}

pub struct Transition {
    pub at: SiteSpec,
    pub from: Option<String>,
    pub to: String,
}

pub struct ErrorCondition {
    pub state: String,
    pub at_sink: bool,
}
```

### Built-in Check Catalog

~10 named checks ship with SAF, each mapping to a pre-built `AnalysisConfig`:

| Name | CWE | Pattern | Sources | Sinks |
|------|-----|---------|---------|-------|
| `use_after_free` | 416 | Alias + State | free(ptr) | deref(ptr) |
| `double_free` | 415 | Alias + State | free(ptr) | free(ptr) |
| `null_deref` | 476 | Reachability / Numeric | null constant | load/store deref |
| `memory_leak` | 401 | Must-Reach | malloc return | function exit |
| `sql_injection` | 89 | Reachability | user input funcs | SQL exec funcs |
| `buffer_overflow` | 120 | Reach + Numeric | taint source | array access |
| `integer_overflow` | 190 | Numeric | arithmetic ops | — |
| `division_by_zero` | 369 | Numeric | div ops | — |
| `file_descriptor_leak` | 775 | Must-Reach | open/fopen | function exit |
| `uninit_use` | 908 | Reachability | alloca | use without store |

---

## 8. Python API

### Entry Point

```python
class Project:
    """The main SAF entry point."""

    @staticmethod
    def open(path: str, *, config: dict = None) -> "Project":
        """Load a program and build the ProgramDatabase."""

    def request(self, req: dict) -> dict:
        """Send a JSON request, get a JSON response. Core protocol method."""

    # ── Discovery ──
    def schema(self) -> dict:
        """Return available checks, templates, queries, and graphs."""

    # ── Property Checking ──
    def check(self, name: str, **params) -> dict:
        """Run a named check or template. Returns JSON response."""

    def check_all(self) -> dict:
        """Run all named checks. Returns JSON response."""

    def analyze(self, config: dict) -> dict:
        """Run a full AnalysisConfig. Returns JSON response."""

    # ── Graph Exploration (Neo4j) ──
    def export_to_neo4j(self, uri: str = "bolt://localhost:7687") -> None:
        """Export all graphs to Neo4j."""

    def cypher(self, query: str, **params) -> dict:
        """Run a Cypher query. Returns JSON response."""

    # ── Power Mode (Graph Primitives) ──
    @property
    def database(self) -> ProgramDatabase:
        """Access the ProgramDatabase for direct graph primitive calls."""

    # ── Backward Compatible ──
    # All existing methods (air(), graphs(), pta_result(), query(), etc.) remain unchanged.
```

### Finding Format

```python
{
    "check": "use_after_free",
    "severity": "error",
    "cwe": 416,
    "message": "Pointer freed at main.c:15, dereferenced at main.c:20",
    "path": [
        {"location": "main.c:10", "event": "allocated"},
        {"location": "main.c:15", "event": "freed", "state": "Freed"},
        {"location": "main.c:20", "event": "dereferenced", "state": "Freed"}
    ],
    "object": "heap allocation at main.c:10"
}
```

---

## 9. LLM Agent Usage Scenarios

### Scenario 1: Security Audit

```
User: "Check this code for security bugs"

LLM sends: {"action": "check_all"}
SAF returns: {"findings": [...10 findings across 5 CWEs...]}

LLM: "Found 10 issues: 3 use-after-free, 2 null derefs, ..."
```

### Scenario 2: Custom Taint Analysis

```
User: "Does user input from parse_request reach the database?"

LLM sends: {"action": "check", "name": "taint",
            "params": {"source": "parse_request", "sink": "db_execute"}}
SAF returns: {"findings": [{...path from parse_request to db_execute...}]}

LLM: "Yes, data flows through process_query to db_execute without sanitization"
```

### Scenario 3: Code Exploration

```
User: "What functions call free() in this codebase?"

LLM sends: {"action": "cypher",
            "query": "MATCH (n:Function)-[:CALLS]->(f:Function {name:'free'}) RETURN n.name"}
SAF returns: {"results": [{"n.name": "cleanup"}, {"n.name": "destroy_session"}]}

LLM: "Two functions call free(): cleanup() and destroy_session()"
```

### Scenario 4: Impact Analysis

```
User: "I want to refactor process_data. What's affected?"

LLM sends: {"action": "cypher",
            "query": "MATCH (n:Function)-[:CALLS*]->(t:Function {name:'process_data'}) RETURN DISTINCT n.name"}
SAF returns: {"results": [{"n.name": "main"}, {"n.name": "handle_request"}]}

LLM: "Changing process_data affects main() and handle_request() (callers)"
```

### Scenario 5: Custom Property

```
User: "Our protocol should always validate before processing. Verify this."

LLM sends: {"action": "analyze", "config": {
    "name": "validate-before-process",
    "sources": [{"call": {"name": "recv_msg", "match_return": true}}],
    "sinks": [{"call": {"name": "process_msg"}}],
    "barriers": [{"call": {"name": "validate_msg"}}],
    "severity": "error"
}}
SAF returns: {"findings": [{...path where validate is skipped...}]}

LLM: "Found 1 path where validation is skipped on the error branch"
```

---

## 10. Relationship to Existing Code

### What Already Exists (No Changes Needed)

| Component | Location | Status |
|-----------|----------|--------|
| `run_pipeline()` | `saf-analysis/src/lib.rs` | Complete — becomes `ProgramDatabase::build()` internals |
| `PipelineResult` | `saf-analysis/src/lib.rs` | Complete — decomposed into `ProgramDatabase` fields |
| PropertyGraph export | `saf-analysis/src/export/` | Complete — becomes Neo4j import source |
| Checker framework | `saf-analysis/src/checkers/` | Complete — called by query decomposition |
| IFDS/IDE solvers | `saf-analysis/src/ifds/` | Complete — called for typestate patterns |
| Absint framework | `saf-analysis/src/absint/` | Complete — called for numeric patterns |
| PTA solver | `saf-analysis/src/pta/` | Complete — core of ProgramDatabase |
| SVFG builder | `saf-analysis/src/svfg/` | Complete — lazy-built in ProgramDatabase |

### What Needs Building

| Component | Location | Estimated LOC |
|-----------|----------|---------------|
| `ProgramDatabase` struct + build | `saf-analysis/src/database.rs` | ~300 |
| Graph primitives (pattern_match, reachability, slice, etc.) | `saf-analysis/src/database/` | ~800 |
| `AnalysisConfig` schema + catalog | `saf-analysis/src/config/` | ~400 |
| Query decomposition engine | `saf-analysis/src/database/decompose.rs` | ~500 |
| JSON protocol handler | `saf-python/src/protocol.rs` | ~400 |
| `Neo4jExporter` Python class | `python/saf/neo4j.py` | ~200 |
| Cypher query bridge | `python/saf/neo4j.py` | ~100 |
| Docker Compose Neo4j service | `docker-compose.yml` | ~15 |
| Named check catalog (YAML) | `crates/saf-analysis/checks/` | ~300 |
| **Total** | | **~3,000 LOC** |

### What Does NOT Change

All existing analysis engines (PTA, IFDS, absint, checkers, SVFG) remain untouched. The `ProgramDatabase` and query decomposition are a new orchestration layer on top.

---

## 11. Implementation Phases

**Phase 1: ProgramDatabase Core**
- `ProgramDatabase` struct wrapping `PipelineResult`
- Graph accessors (pta, callgraph, valueflow, cfg, svfg)
- `export_graphs()` method
- Basic graph primitives (pattern_match, reachability, may_alias)

**Phase 2: AnalysisConfig + Named Checks**
- `AnalysisConfig` JSON schema and Rust types
- Named check catalog (~10 checks)
- Template system (taint, resource_leak, typestate)
- Query decomposition engine (6 patterns)

**Phase 3: JSON Protocol**
- Request/response handler in PyO3
- `Project.request()` method
- Error handling with suggestions
- Schema discovery endpoint

**Phase 4: Neo4j Integration**
- Docker Compose service
- `Neo4jExporter` Python class
- PropertyGraph → Neo4j import pipeline
- `Project.cypher()` method
- Index creation for query performance

**Phase 5: Advanced Primitives + Evaluation**
- Slicing (forward, backward, chop)
- Must-reach analysis
- Value-at queries
- End-to-end LLM agent evaluation

---

*Document version: 1.0*
*Last updated: 2026-02-22*
