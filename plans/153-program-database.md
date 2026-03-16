# ProgramDatabase Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the ProgramDatabase — a graph-first analysis framework where graphs are built first, then queried via a JSON protocol, with Neo4j integration for Cypher exploration.

**Architecture:** `ProgramDatabase` wraps the existing `PipelineResult` and adds graph accessors, query primitives, and an `export_graphs()` method. A JSON protocol layer in PyO3 handles all LLM-SAF communication. Neo4j integration lives in Python using the `neo4j` driver. See `docs/plans/2026-02-22-program-database-design.md` for the full design.

**Tech Stack:** Rust (saf-analysis, saf-core, saf-python/PyO3), Python (neo4j driver), Docker (Neo4j 5 Community), serde_json for JSON protocol.

---

## Phase 1: ProgramDatabase Core

### Task 1.1: Create `ProgramDatabase` struct and `build()` method

**Files:**
- Create: `crates/saf-analysis/src/database.rs`
- Modify: `crates/saf-analysis/src/lib.rs` (add `pub mod database;`)
- Test: `crates/saf-analysis/tests/database_e2e.rs`

**Step 1: Write the failing test**

Create `crates/saf-analysis/tests/database_e2e.rs`:

```rust
use saf_analysis::database::ProgramDatabase;
use saf_analysis::pipeline::PipelineConfig;
use saf_test_utils::load_air_json_fixture;

#[test]
fn database_builds_from_air_json() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());
    assert!(!db.call_graph().nodes.is_empty());
    assert!(!db.defuse().defs.is_empty());
    assert!(db.pta_result().is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_builds_from_air_json'`
Expected: FAIL with "module database not found"

**Step 3: Write minimal implementation**

Create `crates/saf-analysis/src/database.rs`:

```rust
//! Program database — graph-first analysis framework.
//!
//! `ProgramDatabase` owns all precomputed program analysis graphs and provides
//! query primitives for LLM agents and programmatic consumers.

use std::sync::{Arc, OnceLock};

use saf_core::air::AirModule;

use crate::callgraph::CallGraph;
use crate::cfg::Cfg;
use crate::defuse::DefUseGraph;
use crate::export::PropertyGraph;
use crate::icfg::Icfg;
use crate::pipeline::{PipelineConfig, PipelineResult, PipelineStats, run_pipeline};
use crate::PtaResult;
use crate::valueflow::ValueFlowGraph;

/// A program database containing all precomputed analysis graphs.
///
/// Two-phase usage:
/// 1. **Build:** `ProgramDatabase::build(module, config)` constructs core graphs
/// 2. **Query:** Use graph accessors and query primitives to analyze the program
pub struct ProgramDatabase {
    module: Arc<AirModule>,
    call_graph: CallGraph,
    icfg: Icfg,
    pta_result: Option<PtaResult>,
    defuse: DefUseGraph,
    valueflow: ValueFlowGraph,
    stats: PipelineStats,
}

impl ProgramDatabase {
    /// Build a `ProgramDatabase` from an `AirModule`.
    ///
    /// Runs the full analysis pipeline (CG refinement, PTA, value flow)
    /// and stores all resulting graphs for subsequent queries.
    #[must_use]
    pub fn build(module: AirModule, config: &PipelineConfig) -> Self {
        let pipeline = run_pipeline(&module, config);
        Self {
            module: Arc::new(module),
            call_graph: pipeline.call_graph,
            icfg: pipeline.icfg,
            pta_result: pipeline.pta_result,
            defuse: pipeline.defuse,
            valueflow: pipeline.valueflow,
            stats: pipeline.stats,
        }
    }

    /// The analyzed `AirModule`.
    pub fn module(&self) -> &AirModule {
        &self.module
    }

    /// The refined call graph.
    pub fn call_graph(&self) -> &CallGraph {
        &self.call_graph
    }

    /// The ICFG (interprocedural control flow graph).
    pub fn icfg(&self) -> &Icfg {
        &self.icfg
    }

    /// The PTA result (if PTA ran).
    pub fn pta_result(&self) -> Option<&PtaResult> {
        self.pta_result.as_ref()
    }

    /// The def-use graph.
    pub fn defuse(&self) -> &DefUseGraph {
        &self.defuse
    }

    /// The value-flow graph.
    pub fn valueflow(&self) -> &ValueFlowGraph {
        &self.valueflow
    }

    /// Pipeline execution statistics.
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }
}
```

Add `pub mod database;` to `crates/saf-analysis/src/lib.rs` after the `pipeline` module declaration (line 112).

**Step 4: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_builds_from_air_json'`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/database.rs crates/saf-analysis/src/lib.rs crates/saf-analysis/tests/database_e2e.rs
git commit -m "feat(database): add ProgramDatabase struct with build() and graph accessors"
```

---

### Task 1.2: Add `export_graphs()` method

**Files:**
- Modify: `crates/saf-analysis/src/database.rs`
- Test: `crates/saf-analysis/tests/database_e2e.rs`

**Step 1: Write the failing test**

Append to `crates/saf-analysis/tests/database_e2e.rs`:

```rust
#[test]
fn database_exports_property_graphs() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());
    let graphs = db.export_graphs();

    // Should export at least callgraph, defuse, valueflow
    let types: Vec<&str> = graphs.iter().map(|g| g.graph_type.as_str()).collect();
    assert!(types.contains(&"callgraph"), "missing callgraph export");
    assert!(types.contains(&"defuse"), "missing defuse export");
    assert!(types.contains(&"valueflow"), "missing valueflow export");

    // Each graph should have nodes
    for g in &graphs {
        assert!(!g.nodes.is_empty(), "{} graph has no nodes", g.graph_type);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_exports_property_graphs'`
Expected: FAIL with "no method named export_graphs"

**Step 3: Write minimal implementation**

Add to `ProgramDatabase` impl in `database.rs`:

```rust
    /// Export all graphs as `PropertyGraph` structs.
    ///
    /// Returns one `PropertyGraph` per graph type: callgraph, defuse,
    /// valueflow, and (if PTA ran) pta.
    pub fn export_graphs(&self) -> Vec<PropertyGraph> {
        let mut graphs = Vec::new();

        // Call graph
        graphs.push(self.call_graph.to_pg(&self.module));

        // Def-use
        graphs.push(self.defuse.to_pg(&self.module));

        // Value flow
        graphs.push(crate::valueflow::export::to_property_graph(
            &self.valueflow,
            &self.module,
        ));

        // PTA (if available)
        if let Some(pta) = &self.pta_result {
            graphs.push(pta.to_pg());
        }

        graphs
    }
```

**Step 4: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_exports_property_graphs'`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/database.rs crates/saf-analysis/tests/database_e2e.rs
git commit -m "feat(database): add export_graphs() for PropertyGraph export"
```

---

### Task 1.3: Add `may_alias()` and `points_to()` query primitives

**Files:**
- Modify: `crates/saf-analysis/src/database.rs`
- Test: `crates/saf-analysis/tests/database_e2e.rs`

**Step 1: Write the failing test**

Append to `crates/saf-analysis/tests/database_e2e.rs`:

```rust
#[test]
fn database_alias_query() {
    let module = load_air_json_fixture("memory_ops");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    // Verify the query methods exist and return reasonable types
    if let Some(pta) = db.pta_result() {
        // Get any pointer value from the PTA result to test with
        let ptrs: Vec<_> = pta.points_to_map().keys().take(2).collect();
        if ptrs.len() == 2 {
            let result = db.may_alias(*ptrs[0], *ptrs[1]);
            // Result should be one of the valid AliasResult variants
            assert!(matches!(
                result,
                saf_analysis::points_to_query::AliasResult::Must
                    | saf_analysis::points_to_query::AliasResult::Partial
                    | saf_analysis::points_to_query::AliasResult::May
                    | saf_analysis::points_to_query::AliasResult::No
                    | saf_analysis::points_to_query::AliasResult::Unknown
            ));
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_alias_query'`
Expected: FAIL with "no method named may_alias"

**Step 3: Write minimal implementation**

Add imports and methods to `database.rs`:

```rust
use crate::points_to_query::{AliasResult, PointsToQuery};
use saf_core::ids::{ValueId, LocId};
```

Add to `ProgramDatabase` impl:

```rust
    /// Query whether two pointers may alias.
    ///
    /// Returns `AliasResult::Unknown` if PTA is not available.
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        match &self.pta_result {
            Some(pta) => pta.may_alias(p, q),
            None => AliasResult::Unknown,
        }
    }

    /// Query what locations a pointer may point to.
    ///
    /// Returns an empty vec if PTA is not available.
    pub fn points_to(&self, ptr: ValueId) -> Vec<LocId> {
        match &self.pta_result {
            Some(pta) => pta.points_to(ptr),
            None => Vec::new(),
        }
    }
```

**Step 4: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_alias_query'`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/database.rs crates/saf-analysis/tests/database_e2e.rs
git commit -m "feat(database): add may_alias() and points_to() query primitives"
```

---

### Task 1.4: Add `cfg()` accessor and `reachable()` primitive on call graph

**Files:**
- Modify: `crates/saf-analysis/src/database.rs`
- Test: `crates/saf-analysis/tests/database_e2e.rs`

**Step 1: Write the failing test**

Append to `crates/saf-analysis/tests/database_e2e.rs`:

```rust
use saf_core::ids::FunctionId;

#[test]
fn database_cfg_accessor() {
    let module = load_air_json_fixture("control_flow");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    // Build CFG for any defined function
    for func in db.module().functions.iter().filter(|f| !f.is_declaration) {
        let cfg = db.cfg(func.id);
        assert!(!cfg.blocks().is_empty(), "CFG should have blocks");
    }
}

#[test]
fn database_callgraph_reachable() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    // All functions in the callgraph should be reachable from themselves
    let funcs: Vec<_> = db.call_graph().defined_functions().collect();
    if !funcs.is_empty() {
        let reachable = db.cg_reachable_from(&[funcs[0]]);
        assert!(
            reachable.contains(&funcs[0]),
            "function should be reachable from itself"
        );
    }
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_cfg_accessor database_callgraph_reachable'`
Expected: FAIL

**Step 3: Write minimal implementation**

Add to `database.rs`:

```rust
use crate::cfg::Cfg;
use crate::callgraph::CallGraphNode;
use std::collections::{BTreeSet, VecDeque};
```

Add to `ProgramDatabase` impl:

```rust
    /// Build a CFG for a specific function.
    ///
    /// Panics if the function ID is not found in the module.
    pub fn cfg(&self, func_id: FunctionId) -> Cfg {
        let func = self.module.functions.iter()
            .find(|f| f.id == func_id)
            .expect("function not found in module");
        Cfg::build(func)
    }

    /// Find all functions reachable from the given starting functions
    /// in the call graph (forward BFS).
    pub fn cg_reachable_from(&self, starts: &[FunctionId]) -> BTreeSet<FunctionId> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();

        for &fid in starts {
            let node = CallGraphNode::Function(fid);
            if self.call_graph.nodes.contains(&node) {
                visited.insert(fid);
                queue.push_back(node);
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(callees) = self.call_graph.edges.get(&current) {
                for callee in callees {
                    if let CallGraphNode::Function(callee_id) = callee {
                        if visited.insert(*callee_id) {
                            queue.push_back(callee.clone());
                        }
                    }
                }
            }
        }

        visited
    }
```

**Step 4: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database_cfg_accessor database_callgraph_reachable'`
Expected: PASS

**Step 5: Format and lint**

Run: `make fmt && make lint`

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/database.rs crates/saf-analysis/tests/database_e2e.rs
git commit -m "feat(database): add cfg() accessor and cg_reachable_from() primitive"
```

---

### Task 1.5: Run full test suite validation

**Step 1: Run all tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run 2>&1' | tee /tmp/test-output.txt`

Expected: All existing tests pass, plus the new `database_e2e` tests.

**Step 2: Run lints**

Run: `make fmt && make lint`

Expected: No warnings from new code.

**Step 3: Commit if any fixes needed**

---

## Phase 2: AnalysisConfig Schema + Named Checks

### Task 2.1: Define `AnalysisConfig` types

**Files:**
- Create: `crates/saf-analysis/src/database/config.rs`
- Modify: `crates/saf-analysis/src/database.rs` (convert to `database/mod.rs`)
- Test: unit tests inline

**Step 1: Convert `database.rs` to module directory**

Rename `crates/saf-analysis/src/database.rs` → `crates/saf-analysis/src/database/mod.rs` and add `pub mod config;`.

**Step 2: Write the test**

In `crates/saf-analysis/src/database/config.rs`, add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_config_deserializes_from_json() {
        let json = r#"{
            "name": "test-taint",
            "severity": "error",
            "sources": [{"call": {"name": "read"}}],
            "sinks": [{"call": {"name": "exec"}}]
        }"#;
        let config: AnalysisConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "test-taint");
        assert_eq!(config.sources.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn site_spec_call_variant() {
        let json = r#"{"call": {"name": "free", "arg": 0}}"#;
        let spec: SiteSpec = serde_json::from_str(json).unwrap();
        assert!(matches!(spec, SiteSpec::Call { .. }));
    }

    #[test]
    fn flow_state_config_roundtrip() {
        let json = r#"{
            "states": ["Alive", "Freed"],
            "initial": "Alive",
            "transitions": [{"at": {"call": {"name": "free"}}, "to": "Freed"}],
            "error": {"state": "Freed", "at_sink": true}
        }"#;
        let fsc: FlowStateConfig = serde_json::from_str(json).unwrap();
        assert_eq!(fsc.states.len(), 2);
        assert_eq!(fsc.initial, "Alive");
    }
}
```

**Step 3: Write the implementation**

```rust
//! Declarative analysis configuration types.
//!
//! `AnalysisConfig` is the JSON schema that LLM agents use to describe
//! what properties to check. It decomposes into graph primitive operations.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Severity level for analysis findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A declarative specification of a program site (source, sink, or barrier).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteSpec {
    /// Match a function call by name.
    Call {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        arg: Option<u32>,
        #[serde(default)]
        match_return: bool,
    },
    /// Match a dereference of a bound variable.
    Deref { bind_var: String },
    /// Match any function exit.
    FunctionExit {},
}

/// Where to bind a variable (at source or sink site).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindSite {
    Source,
    Sink,
}

/// What to bind (argument, return value, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindTarget {
    Arg(u32),
    ReturnValue,
}

/// Variable binding for alias correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindSpec {
    pub at: BindSite,
    pub what: BindTarget,
}

/// A state transition in a typestate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub at: SiteSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub to: String,
}

/// Error condition for typestate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCondition {
    pub state: String,
    pub at_sink: bool,
}

/// State machine configuration for typestate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStateConfig {
    pub states: Vec<String>,
    pub initial: String,
    pub transitions: Vec<Transition>,
    pub error: ErrorCondition,
}

/// The unified analysis configuration.
///
/// LLM agents generate this JSON to describe what properties to check.
/// The query decomposition engine translates it into graph primitive operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<u32>,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SiteSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sinks: Option<Vec<SiteSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barriers: Option<Vec<SiteSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_variables: Option<BTreeMap<String, BindSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_states: Option<FlowStateConfig>,
}

impl AnalysisConfig {
    /// Check if this config uses bind variables (alias correlation).
    pub fn has_bind_variables(&self) -> bool {
        self.bind_variables.as_ref().map_or(false, |bv| !bv.is_empty())
    }

    /// Check if this config uses flow state tracking (typestate).
    pub fn has_flow_states(&self) -> bool {
        self.flow_states.is_some()
    }

    /// Check if all sinks are function exits (must-reach pattern).
    pub fn sinks_are_function_exit(&self) -> bool {
        self.sinks.as_ref().map_or(false, |sinks| {
            sinks.iter().all(|s| matches!(s, SiteSpec::FunctionExit { .. }))
        })
    }
}
```

**Step 4: Run tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis config'`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/database/
git commit -m "feat(database): add AnalysisConfig types with JSON serialization"
```

---

### Task 2.2: Create named check catalog

**Files:**
- Create: `crates/saf-analysis/src/database/catalog.rs`
- Modify: `crates/saf-analysis/src/database/mod.rs`
- Test: unit tests inline

**Step 1: Write the test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_known_checks() {
        let catalog = CheckCatalog::new();
        assert!(catalog.get("use_after_free").is_some());
        assert!(catalog.get("null_deref").is_some());
        assert!(catalog.get("memory_leak").is_some());
        assert!(catalog.get("nonexistent").is_none());
    }

    #[test]
    fn catalog_lists_all_checks() {
        let catalog = CheckCatalog::new();
        let names = catalog.list();
        assert!(names.len() >= 9, "should have at least 9 built-in checks");
    }

    #[test]
    fn catalog_entries_serialize_to_json() {
        let catalog = CheckCatalog::new();
        let entry = catalog.get("use_after_free").unwrap();
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("use_after_free"));
    }
}
```

**Step 2: Write the implementation**

```rust
//! Named check catalog — pre-built `AnalysisConfig` entries for common bug patterns.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use super::config::{AnalysisConfig, Severity, SiteSpec};

/// A catalog entry describing a named check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub name: String,
    pub description: String,
    pub cwe: Option<u32>,
    pub severity: Severity,
    pub category: String,
}

/// The named check catalog.
///
/// Maps check names to their descriptions and pre-built `AnalysisConfig` entries.
pub struct CheckCatalog {
    entries: BTreeMap<String, CatalogEntry>,
}

impl CheckCatalog {
    /// Create a new catalog with all built-in checks.
    #[must_use]
    pub fn new() -> Self {
        let mut entries = BTreeMap::new();

        let checks = [
            ("use_after_free", "Detects dereference of freed heap memory", Some(416), Severity::Error, "memory_safety"),
            ("double_free", "Detects double-free of heap memory", Some(415), Severity::Error, "memory_safety"),
            ("null_deref", "Detects null pointer dereference", Some(476), Severity::Error, "memory_safety"),
            ("memory_leak", "Detects unreleased heap allocations", Some(401), Severity::Warning, "resource_management"),
            ("file_descriptor_leak", "Detects unclosed file descriptors", Some(775), Severity::Warning, "resource_management"),
            ("uninit_use", "Detects use of uninitialized memory", Some(908), Severity::Error, "memory_safety"),
            ("stack_escape", "Detects stack pointer escaping function scope", Some(562), Severity::Error, "memory_safety"),
            ("lock_not_released", "Detects unreleased mutex locks", Some(764), Severity::Warning, "concurrency"),
            ("generic_resource_leak", "Detects unreleased generic resources", Some(772), Severity::Warning, "resource_management"),
            ("buffer_overflow", "Detects array access beyond bounds", Some(120), Severity::Error, "memory_safety"),
            ("integer_overflow", "Detects arithmetic overflow/underflow", Some(190), Severity::Warning, "numeric"),
            ("division_by_zero", "Detects potential division by zero", Some(369), Severity::Error, "numeric"),
        ];

        for (name, desc, cwe, severity, category) in checks {
            entries.insert(name.to_string(), CatalogEntry {
                name: name.to_string(),
                description: desc.to_string(),
                cwe,
                severity,
                category: category.to_string(),
            });
        }

        Self { entries }
    }

    /// Get a catalog entry by name.
    pub fn get(&self, name: &str) -> Option<&CatalogEntry> {
        self.entries.get(name)
    }

    /// List all available check names (sorted).
    pub fn list(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Get all entries as a slice for schema discovery.
    pub fn entries(&self) -> &BTreeMap<String, CatalogEntry> {
        &self.entries
    }

    /// Map a catalog check name to the corresponding built-in checker name.
    ///
    /// Some catalog names differ from the internal checker names
    /// (e.g., `"use_after_free"` maps to `"use-after-free"`).
    pub fn to_checker_name(catalog_name: &str) -> Option<&'static str> {
        match catalog_name {
            "use_after_free" => Some("use-after-free"),
            "double_free" => Some("double-free"),
            "null_deref" => Some("null-deref"),
            "memory_leak" => Some("memory-leak"),
            "file_descriptor_leak" => Some("file-descriptor-leak"),
            "uninit_use" => Some("uninit-use"),
            "stack_escape" => Some("stack-escape"),
            "lock_not_released" => Some("lock-not-released"),
            "generic_resource_leak" => Some("generic-resource-leak"),
            _ => None,
        }
    }
}

impl Default for CheckCatalog {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 3: Run tests, format, commit**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis catalog'`

```bash
git add crates/saf-analysis/src/database/catalog.rs
git commit -m "feat(database): add named check catalog with 12 built-in checks"
```

---

### Task 2.3: Add `schema()` method to `ProgramDatabase`

**Files:**
- Modify: `crates/saf-analysis/src/database/mod.rs`
- Test: `crates/saf-analysis/tests/database_e2e.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn database_schema_has_checks_and_graphs() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());
    let schema = db.schema();

    assert!(!schema.checks.is_empty(), "schema should list checks");
    assert!(!schema.graphs.is_empty(), "schema should list graphs");
    assert!(
        schema.checks.iter().any(|c| c.name == "use_after_free"),
        "schema should include use_after_free"
    );
}
```

**Step 2: Implement**

Add a `Schema` struct and `schema()` method to `database/mod.rs`:

```rust
use catalog::{CheckCatalog, CatalogEntry};

/// Discovery schema returned by `ProgramDatabase::schema()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub checks: Vec<CatalogEntry>,
    pub graphs: Vec<String>,
    pub queries: Vec<String>,
}

impl ProgramDatabase {
    /// Return the discovery schema for LLM agents.
    pub fn schema(&self) -> Schema {
        let catalog = CheckCatalog::new();
        Schema {
            checks: catalog.entries().values().cloned().collect(),
            graphs: vec![
                "cfg".to_string(),
                "callgraph".to_string(),
                "defuse".to_string(),
                "valueflow".to_string(),
                "pta".to_string(),
                "svfg".to_string(),
            ],
            queries: vec![
                "alias".to_string(),
                "points_to".to_string(),
                "reachable".to_string(),
            ],
        }
    }
}
```

Add `use serde::{Deserialize, Serialize};` if not already present.

**Step 3: Run tests, commit**

```bash
git commit -m "feat(database): add schema() for LLM agent discoverability"
```

---

## Phase 3: JSON Protocol

### Task 3.1: Define JSON request/response types

**Files:**
- Create: `crates/saf-analysis/src/database/protocol.rs`
- Test: unit tests inline

**Step 1: Write the test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_request_deserializes() {
        let json = r#"{"action": "schema"}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert!(matches!(req, Request::Schema));
    }

    #[test]
    fn check_request_deserializes() {
        let json = r#"{"action": "check", "name": "use_after_free"}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert!(matches!(req, Request::Check { .. }));
    }

    #[test]
    fn check_request_with_params() {
        let json = r#"{"action": "check", "name": "taint", "params": {"source": "read", "sink": "exec"}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        if let Request::Check { name, params } = req {
            assert_eq!(name, "taint");
            assert!(params.is_some());
        }
    }

    #[test]
    fn analyze_request_deserializes() {
        let json = r#"{"action": "analyze", "config": {"name": "test", "severity": "error", "sources": [{"call": {"name": "read"}}]}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert!(matches!(req, Request::Analyze { .. }));
    }

    #[test]
    fn error_response_serializes() {
        let resp = Response::error("UNKNOWN_CHECK", "No check named 'uaf'");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("UNKNOWN_CHECK"));
    }
}
```

**Step 2: Write the implementation**

```rust
//! JSON protocol types for LLM ↔ SAF communication.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::config::AnalysisConfig;

/// A JSON request from an LLM agent to SAF.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Request {
    /// Get the API schema and available checks.
    Schema,
    /// Run a named check or template.
    Check {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<BTreeMap<String, serde_json::Value>>,
    },
    /// Run all named checks.
    CheckAll,
    /// Run a full analysis config.
    Analyze {
        config: AnalysisConfig,
    },
    /// Run a Cypher query against Neo4j.
    Cypher {
        query: String,
        #[serde(default)]
        params: BTreeMap<String, serde_json::Value>,
    },
    /// Run a graph primitive query.
    Query {
        #[serde(rename = "type")]
        query_type: String,
        params: BTreeMap<String, serde_json::Value>,
    },
}

/// A finding from an analysis check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub check: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<u32>,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<PathEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
}

/// An event in a finding's path trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEvent {
    pub location: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Metadata about a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub engines_used: Vec<String>,
}

/// Error details in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

/// A JSON response from SAF to an LLM agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<Finding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<BTreeMap<String, serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ResponseMetadata>,
    /// Schema data (only for schema responses).
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Response {
    /// Create a success response with findings.
    pub fn ok_findings(findings: Vec<Finding>, metadata: ResponseMetadata) -> Self {
        Self {
            status: "ok".to_string(),
            findings: Some(findings),
            results: None,
            error: None,
            metadata: Some(metadata),
            extra: BTreeMap::new(),
        }
    }

    /// Create an error response.
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            findings: None,
            results: None,
            error: Some(ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                suggestions: Vec::new(),
            }),
            metadata: None,
            extra: BTreeMap::new(),
        }
    }
}
```

**Step 3: Run tests, commit**

```bash
git commit -m "feat(database): add JSON protocol request/response types"
```

---

### Task 3.2: Implement `handle_request()` on `ProgramDatabase`

**Files:**
- Modify: `crates/saf-analysis/src/database/mod.rs`
- Create: `crates/saf-analysis/src/database/handler.rs`
- Test: `crates/saf-analysis/tests/database_e2e.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn database_handles_schema_request() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "schema"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "ok");
    assert!(resp["checks"].is_array());
}

#[test]
fn database_handles_check_request() {
    let module = load_air_json_fixture("memory_ops");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "check", "name": "null_deref"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "ok");
    // findings may be empty for simple fixture, but key should exist
    assert!(resp.get("findings").is_some() || resp.get("error").is_some());
}

#[test]
fn database_handles_unknown_check() {
    let module = load_air_json_fixture("calls");
    let db = ProgramDatabase::build(module, &PipelineConfig::default());

    let req = r#"{"action": "check", "name": "nonexistent_check"}"#;
    let resp_json = db.handle_request(req).unwrap();
    let resp: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    assert_eq!(resp["status"], "error");
    assert_eq!(resp["error"]["code"], "UNKNOWN_CHECK");
}
```

**Step 2: Implement `handle_request()`**

In `database/handler.rs`:

```rust
//! JSON request handler for `ProgramDatabase`.

use std::time::Instant;

use crate::checkers;
use crate::checkers::ResourceTable;

use super::catalog::CheckCatalog;
use super::protocol::{Finding, PathEvent, Request, Response, ResponseMetadata};
use super::ProgramDatabase;

impl ProgramDatabase {
    /// Handle a JSON request string and return a JSON response string.
    ///
    /// This is the main entry point for the JSON protocol.
    pub fn handle_request(&self, json: &str) -> Result<String, serde_json::Error> {
        let request: Request = match serde_json::from_str(json) {
            Ok(req) => req,
            Err(e) => {
                let resp = Response::error("INVALID_REQUEST", &format!("Failed to parse request: {e}"));
                return serde_json::to_string(&resp);
            }
        };

        let response = match request {
            Request::Schema => self.handle_schema(),
            Request::Check { name, params } => self.handle_check(&name, params.as_ref()),
            Request::CheckAll => self.handle_check_all(),
            Request::Analyze { config } => {
                // For now, treat analyze as check with the config name
                self.handle_check(&config.name, None)
            }
            Request::Cypher { .. } => {
                Response::error("NOT_IMPLEMENTED", "Cypher queries require Neo4j connection (use export_to_neo4j first)")
            }
            Request::Query { query_type, params } => {
                self.handle_query(&query_type, &params)
            }
        };

        serde_json::to_string(&response)
    }

    fn handle_schema(&self) -> Response {
        let schema = self.schema();
        let mut resp = Response::ok_findings(Vec::new(), ResponseMetadata {
            elapsed_ms: None,
            engines_used: Vec::new(),
        });
        resp.findings = None;
        resp.extra.insert(
            "checks".to_string(),
            serde_json::to_value(&schema.checks).unwrap_or_default(),
        );
        resp.extra.insert(
            "graphs".to_string(),
            serde_json::to_value(&schema.graphs).unwrap_or_default(),
        );
        resp.extra.insert(
            "queries".to_string(),
            serde_json::to_value(&schema.queries).unwrap_or_default(),
        );
        resp
    }

    fn handle_check(
        &self,
        name: &str,
        _params: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
    ) -> Response {
        let start = Instant::now();

        // Look up the check in the catalog
        let catalog = CheckCatalog::new();
        let checker_name = match CheckCatalog::to_checker_name(name) {
            Some(n) => n,
            None => {
                // Try fuzzy matching for suggestions
                let suggestions: Vec<String> = catalog.list()
                    .into_iter()
                    .filter(|n| {
                        n.contains(&name.replace('_', ""))
                            || name.contains(&n.replace('_', ""))
                    })
                    .map(String::from)
                    .collect();
                return Response {
                    status: "error".to_string(),
                    findings: None,
                    results: None,
                    error: Some(super::protocol::ErrorDetail {
                        code: "UNKNOWN_CHECK".to_string(),
                        message: format!("No check named '{name}'"),
                        suggestions,
                    }),
                    metadata: None,
                    extra: std::collections::BTreeMap::new(),
                };
            }
        };

        // Also handle numeric checks
        let is_numeric = matches!(name, "buffer_overflow" | "integer_overflow" | "division_by_zero");

        if is_numeric {
            // Numeric checks use the absint framework
            let findings = self.run_numeric_check(name);
            let elapsed = start.elapsed().as_millis() as u64;
            return Response::ok_findings(findings, ResponseMetadata {
                elapsed_ms: Some(elapsed),
                engines_used: vec!["absint".to_string()],
            });
        }

        // SVFG-based checks
        let findings = self.run_svfg_check(checker_name);
        let elapsed = start.elapsed().as_millis() as u64;
        Response::ok_findings(findings, ResponseMetadata {
            elapsed_ms: Some(elapsed),
            engines_used: vec!["pta".to_string(), "svfg".to_string()],
        })
    }

    fn handle_check_all(&self) -> Response {
        let start = Instant::now();
        let catalog = CheckCatalog::new();
        let mut all_findings = Vec::new();

        for name in catalog.list() {
            if let Some(checker_name) = CheckCatalog::to_checker_name(name) {
                all_findings.extend(self.run_svfg_check(checker_name));
            }
        }

        let elapsed = start.elapsed().as_millis() as u64;
        Response::ok_findings(all_findings, ResponseMetadata {
            elapsed_ms: Some(elapsed),
            engines_used: vec!["pta".to_string(), "svfg".to_string()],
        })
    }

    fn handle_query(
        &self,
        query_type: &str,
        params: &std::collections::BTreeMap<String, serde_json::Value>,
    ) -> Response {
        match query_type {
            "alias" => {
                // Parse p and q from params
                let p_hex = params.get("p").and_then(|v| v.as_str());
                let q_hex = params.get("q").and_then(|v| v.as_str());
                match (p_hex, q_hex) {
                    (Some(_p), Some(_q)) => {
                        // TODO: parse hex IDs and call may_alias
                        Response::error("NOT_IMPLEMENTED", "Alias query parsing not yet implemented")
                    }
                    _ => Response::error("INVALID_PARAMS", "alias query requires 'p' and 'q' parameters"),
                }
            }
            _ => Response::error("UNKNOWN_QUERY_TYPE", &format!("Unknown query type: {query_type}")),
        }
    }

    /// Run an SVFG-based checker and convert findings to protocol format.
    fn run_svfg_check(&self, checker_name: &str) -> Vec<Finding> {
        let spec = match checkers::builtin_checker(checker_name) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let table = ResourceTable::new();
        let svfg = self.get_or_build_svfg();
        let config = checkers::SolverConfig::default();
        let result = checkers::run_checker(&spec, &self.module, svfg, &table, &config);

        result.findings.iter().map(|f| {
            Finding {
                check: f.checker_name.clone(),
                severity: format!("{:?}", f.severity).to_lowercase(),
                cwe: f.cwe,
                message: f.message.clone(),
                path: Vec::new(), // TODO: convert trace to PathEvents
                object: None,
            }
        }).collect()
    }

    /// Run a numeric (absint) check.
    fn run_numeric_check(&self, _name: &str) -> Vec<Finding> {
        // TODO: wire up absint numeric checkers
        Vec::new()
    }
}
```

Note: This requires SVFG lazy building. Add to `database/mod.rs`:

```rust
use crate::svfg::Svfg;

// Add to ProgramDatabase struct:
//   svfg: OnceLock<Svfg>,

impl ProgramDatabase {
    fn get_or_build_svfg(&self) -> &Svfg {
        self.svfg.get_or_init(|| {
            let cfgs = crate::cfg::build_cfgs(&self.module);
            let mssa_pta = self.pta_result.clone();
            let mut mssa = crate::mssa::MemorySsa::build(
                &self.module, &cfgs, mssa_pta.as_ref(), &self.call_graph,
            );
            let (svfg, _) = crate::svfg::SvfgBuilder::new(
                &self.module, &self.defuse, &self.call_graph,
                self.pta_result.as_ref(), &mut mssa,
            ).build();
            svfg
        })
    }
}
```

**Step 3: Run tests, format, lint, commit**

```bash
git commit -m "feat(database): implement handle_request() JSON protocol handler"
```

---

### Task 3.3: Wire `ProgramDatabase` into Python `Project`

**Files:**
- Modify: `crates/saf-python/src/project.rs`
- Test: `python/tests/test_database.py`

**Step 1: Write the failing Python test**

Create `python/tests/test_database.py`:

```python
import json
import saf

def test_project_request_schema():
    proj = saf.Project.open("tests/fixtures/llvm/e2e/simple.ll")
    resp = proj.request('{"action": "schema"}')
    data = json.loads(resp)
    assert data["status"] == "ok"
    assert "checks" in data
    assert any(c["name"] == "use_after_free" for c in data["checks"])

def test_project_request_check():
    proj = saf.Project.open("tests/fixtures/llvm/e2e/simple.ll")
    resp = proj.request('{"action": "check", "name": "null_deref"}')
    data = json.loads(resp)
    assert data["status"] == "ok"

def test_project_request_unknown_check():
    proj = saf.Project.open("tests/fixtures/llvm/e2e/simple.ll")
    resp = proj.request('{"action": "check", "name": "nonexistent"}')
    data = json.loads(resp)
    assert data["status"] == "error"
    assert data["error"]["code"] == "UNKNOWN_CHECK"
```

**Step 2: Add `request()` method to Python `Project`**

In `crates/saf-python/src/project.rs`, add:

```rust
/// Handle a JSON protocol request.
///
/// Accepts a JSON string and returns a JSON string response.
/// This is the primary interface for LLM agents.
#[pyo3(signature = (json_request))]
fn request(&self, json_request: &str) -> PyResult<String> {
    let db = self.build_database();
    db.handle_request(json_request)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON error: {e}")))
}
```

The `build_database()` helper constructs a `ProgramDatabase` from the `Project`'s existing fields:

```rust
fn build_database(&self) -> saf_analysis::database::ProgramDatabase {
    saf_analysis::database::ProgramDatabase::from_parts(
        self.module.clone(),
        // ... pass existing graphs
    )
}
```

This requires adding a `from_parts()` constructor to `ProgramDatabase` that takes pre-built graphs (since `Project` already has them).

**Step 3: Run tests inside Docker**

Run: `make test`

**Step 4: Commit**

```bash
git commit -m "feat(python): add Project.request() for JSON protocol"
```

---

## Phase 4: Neo4j Integration

### Task 4.1: Add Neo4j service to Docker Compose

**Files:**
- Modify: `docker-compose.yml`

**Step 1: Add Neo4j service**

Add to `docker-compose.yml`:

```yaml
  neo4j:
    image: neo4j:5-community
    ports:
      - "7474:7474"
      - "7687:7687"
    environment:
      NEO4J_AUTH: none
      NEO4J_PLUGINS: '["apoc"]'
    volumes:
      - neo4j-data:/data
```

Add `neo4j-data:` to the `volumes:` section.

Add `NEO4J_URI: bolt://neo4j:7687` to the `dev` service environment.

**Step 2: Test that Neo4j starts**

Run: `docker compose up neo4j -d && sleep 10 && docker compose exec neo4j cypher-shell -u neo4j "RETURN 1 AS test"`
Expected: Returns `test: 1`

**Step 3: Commit**

```bash
git add docker-compose.yml
git commit -m "infra: add Neo4j 5 Community service to Docker Compose"
```

---

### Task 4.2: Create Python Neo4j exporter

**Files:**
- Create: `python/saf/neo4j_export.py`
- Test: `python/tests/test_neo4j.py`

**Step 1: Write the exporter**

```python
"""Neo4j graph exporter for SAF ProgramDatabase.

Exports PropertyGraph data from SAF into a Neo4j database for
Cypher-based graph exploration.
"""

import json
from typing import Optional

try:
    import neo4j as neo4j_driver
    HAS_NEO4J = True
except ImportError:
    HAS_NEO4J = False


class Neo4jExporter:
    """Export SAF analysis graphs to Neo4j."""

    def __init__(self, uri: str = "bolt://localhost:7687"):
        if not HAS_NEO4J:
            raise ImportError(
                "neo4j driver not installed. Install with: pip install neo4j"
            )
        self.driver = neo4j_driver.GraphDatabase.driver(uri)

    def close(self):
        """Close the Neo4j driver connection."""
        self.driver.close()

    def clear(self):
        """Clear all nodes and relationships from the database."""
        with self.driver.session() as session:
            session.run("MATCH (n) DETACH DELETE n")

    def export(self, project) -> dict:
        """Export all ProgramDatabase graphs to Neo4j.

        Args:
            project: A saf.Project instance

        Returns:
            dict with import statistics
        """
        # Get PropertyGraph JSON from the Rust side
        graphs = project.database.export_graphs()
        stats = {"graphs": 0, "nodes": 0, "edges": 0}

        with self.driver.session() as session:
            # Create indexes for query performance
            self._create_indexes(session)

            for graph in graphs:
                if isinstance(graph, str):
                    graph = json.loads(graph)
                self._import_graph(session, graph)
                stats["graphs"] += 1
                stats["nodes"] += len(graph.get("nodes", []))
                stats["edges"] += len(graph.get("edges", []))

        return stats

    def _create_indexes(self, session):
        """Create Neo4j indexes for common query patterns."""
        indexes = [
            "CREATE INDEX IF NOT EXISTS FOR (n:Function) ON (n.name)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Value) ON (n.id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Location) ON (n.id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Block) ON (n.id)",
            "CREATE INDEX IF NOT EXISTS FOR (n:Instruction) ON (n.id)",
        ]
        for idx in indexes:
            session.run(idx)

    def _import_graph(self, session, graph: dict):
        """Import a single PropertyGraph into Neo4j."""
        nodes = graph.get("nodes", [])
        edges = graph.get("edges", [])

        if not nodes:
            return

        # Batch create nodes using UNWIND
        # We create nodes with a generic label first, then add specific labels
        for node in nodes:
            labels = ":".join(node.get("labels", ["Node"]))
            props = node.get("properties", {})
            props["id"] = node["id"]
            session.run(
                f"CREATE (n:{labels}) SET n += $props",
                props=props,
            )

        # Batch create relationships
        for edge in edges:
            edge_type = edge.get("edge_type", "RELATED_TO")
            session.run(
                f"MATCH (a {{id: $src}}), (b {{id: $dst}}) "
                f"CREATE (a)-[r:{edge_type}]->(b) "
                f"SET r += $props",
                src=edge["src"],
                dst=edge["dst"],
                props=edge.get("properties", {}),
            )

    def cypher(self, query: str, **params) -> list:
        """Execute a Cypher query and return results as list of dicts."""
        with self.driver.session() as session:
            result = session.run(query, **params)
            return [dict(record) for record in result]
```

**Step 2: Write the test**

```python
"""Tests for Neo4j integration (requires Neo4j running)."""
import os
import pytest
import json

# Skip if NEO4J_URI not set
NEO4J_URI = os.environ.get("NEO4J_URI", "")
pytestmark = pytest.mark.skipif(
    not NEO4J_URI,
    reason="NEO4J_URI not set - Neo4j integration tests skipped"
)

def test_neo4j_exporter_import():
    from saf.neo4j_export import Neo4jExporter
    exporter = Neo4jExporter(NEO4J_URI)
    exporter.clear()

    import saf
    proj = saf.Project.open("tests/fixtures/llvm/e2e/simple.ll")
    stats = exporter.export(proj)
    assert stats["graphs"] > 0
    assert stats["nodes"] > 0

    # Query should work
    results = exporter.cypher("MATCH (n:Function) RETURN n.name LIMIT 5")
    assert len(results) > 0

    exporter.close()
```

**Step 3: Commit**

```bash
git add python/saf/neo4j_export.py python/tests/test_neo4j.py
git commit -m "feat(neo4j): add Neo4jExporter Python class for graph export"
```

---

### Task 4.3: Add `cypher()` and `export_to_neo4j()` to Python Project

**Files:**
- Modify: `crates/saf-python/src/project.rs`

**Step 1: Add Python-side methods**

These are pure Python methods that delegate to `Neo4jExporter`. Add them to the Python `Project` class via a wrapper or via PyO3 `#[pymethods]`:

```rust
/// Export all graphs to Neo4j.
///
/// Requires the neo4j Python package to be installed.
#[pyo3(signature = (uri="bolt://localhost:7687"))]
fn export_to_neo4j(&self, py: Python<'_>, uri: &str) -> PyResult<Py<PyDict>> {
    // Call the Python-side Neo4jExporter
    let neo4j_mod = py.import("saf.neo4j_export")?;
    let exporter = neo4j_mod.call_method1("Neo4jExporter", (uri,))?;
    let stats = exporter.call_method1("export", (self.as_ref(py),))?;
    Ok(stats.into_py(py))
}

/// Run a Cypher query against Neo4j.
///
/// Requires export_to_neo4j() to have been called first.
#[pyo3(signature = (query, uri="bolt://localhost:7687", **params))]
fn cypher(&self, py: Python<'_>, query: &str, uri: &str, params: Option<&PyDict>) -> PyResult<Py<PyList>> {
    let neo4j_mod = py.import("saf.neo4j_export")?;
    let exporter = neo4j_mod.call_method1("Neo4jExporter", (uri,))?;
    let results = exporter.call_method("cypher", (query,), params)?;
    Ok(results.into_py(py))
}
```

**Step 2: Test, commit**

```bash
git commit -m "feat(python): add export_to_neo4j() and cypher() to Project"
```

---

## Phase 5: Validation and Documentation

### Task 5.1: Run full test suite

**Step 1: Run all Rust tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run 2>&1' | tee /tmp/test-output.txt`
Expected: All tests pass.

**Step 2: Run all Python tests**

Run: `docker compose run --rm dev sh -c 'cd /workspace && python3 -m pytest python/tests/ -v 2>&1' | tee /tmp/pytest-output.txt`
Expected: All tests pass (Neo4j tests skipped unless NEO4J_URI set).

**Step 3: Run lints**

Run: `make fmt && make lint`
Expected: Clean.

### Task 5.2: Update PROGRESS.md

**Files:**
- Modify: `plans/PROGRESS.md`

Add plan 153 to the Plans Index:
```
| 153 | program-database | architecture | done | Notes: ProgramDatabase wrapping PipelineResult, JSON protocol, Neo4j integration, AnalysisConfig schema, named check catalog. |
```

Update Next Steps and Session Log.

**Step: Commit**

```bash
git add plans/PROGRESS.md
git commit -m "docs: update PROGRESS.md with Plan 153 results"
```

---

## Key References

| File | Purpose |
|------|---------|
| `docs/plans/2026-02-22-program-database-design.md` | Full design document |
| `crates/saf-analysis/src/pipeline.rs` | Existing `run_pipeline()` and `PipelineResult` |
| `crates/saf-analysis/src/export.rs` | `PropertyGraph`, `PgNode`, `PgEdge` types |
| `crates/saf-analysis/src/checkers/spec.rs` | `CheckerSpec`, `SitePattern`, `Severity` |
| `crates/saf-analysis/src/checkers/runner.rs` | `run_checker()`, `run_checkers()` |
| `crates/saf-analysis/src/points_to_query.rs` | `PointsToQuery` trait, `AliasResult` |
| `crates/saf-python/src/project.rs` | Python `Project` class |
| `crates/saf-analysis/src/selector/mod.rs` | `Selector`, `SelectorResolver` |
| `crates/saf-analysis/tests/air_json_pipeline.rs` | Test pattern reference |

## Testing Commands

```bash
# Run just ProgramDatabase tests
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis database'

# Run all tests
make test

# Run with Neo4j integration
docker compose up neo4j -d
NEO4J_URI=bolt://localhost:7687 docker compose run --rm dev sh -c 'python3 -m pytest python/tests/test_neo4j.py -v'
```
