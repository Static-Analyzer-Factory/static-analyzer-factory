# SAF Architecture Extensibility Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform SAF from a monolithic research framework into an extensible analysis platform that supports new frontends, analysis algorithms, and third-party plugins without invasive modifications.

**Architecture:** Nine phases ordered by dependency and impact. Each phase is independently implementable in a single Claude Code session. Phases 1-3 are non-breaking quick wins. Phases 4-5 introduce new trait-based abstractions. Phases 6-9 are infrastructure improvements. Every phase must pass `make fmt && make lint && make test` before completion.

**Tech Stack:** Rust (saf-core, saf-frontends, saf-analysis, saf-python, saf-wasm), PyO3, LLVM 18, Docker

**Key Constraint:** All changes must maintain backward compatibility. Existing Python API, WASM API, and benchmark harness must continue to work unchanged. PTABen results must not regress (currently 68 Unsound).

---

## Phase 1: Quick Wins (P0)

**Estimated effort:** 1 session (2-3 hours)
**Dependencies:** None
**Impact:** Eliminates the largest repeated cost (SVFG rebuild) and fixes the biggest C++ precision gap (named struct types)

### Task 1.1: Cache SVFG on Python `Project`

Every call to `Project.check()`, `check_custom()`, `check_path_sensitive()`, `check_all()`, and `checker_diagnostics()` independently rebuilds the SVFG. Cache it with `OnceLock`.

**Files:**
- Modify: `crates/saf-python/src/project.rs`
- Modify: `crates/saf-python/src/checkers.rs` (if SVFG construction exists here)
- Test: `make test` (existing Python checker tests validate correctness)

**Step 1: Read current SVFG construction sites**

Read `crates/saf-python/src/project.rs` and `crates/saf-python/src/checkers.rs` to find all places where `SvfgBuilder::build()` or `Svfg::build()` is called. Identify the common arguments (module, callgraph, pta_result, mssa).

**Step 2: Add SVFG cache field to Project**

Add to the `Project` struct:

```rust
use std::sync::OnceLock;
use std::sync::Arc;

// In Project struct:
svfg_cache: OnceLock<Arc<Svfg>>,
```

Initialize in `Project::new()` / `Project::open()`:

```rust
svfg_cache: OnceLock::new(),
```

**Step 3: Create a helper method to get-or-build SVFG**

```rust
fn get_or_build_svfg(&self) -> PyResult<Arc<Svfg>> {
    Ok(Arc::clone(self.svfg_cache.get_or_init(|| {
        // Move the existing SvfgBuilder::build() call here
        // Use self.module, self.call_graph, self.pta_result, etc.
        Arc::new(build_svfg_from_project_state(&self.module, ...))
    })))
}
```

**Step 4: Replace all SVFG construction sites with cache lookup**

Every method that currently calls `SvfgBuilder::build()` should instead call `self.get_or_build_svfg()`.

**Step 5: Run tests**

```bash
make fmt && make lint && make test
```

All existing Python checker tests must pass. The SVFG is now built once per `Project` instance.

**Step 6: Commit**

```
feat(python): cache SVFG on Project to avoid rebuild per checker call
```

---

### Task 1.2: Fix Named Struct Type Parsing in TypeInterner

Named struct types (`%struct.Foo = type { i32, ptr }`) always become `AirType::Opaque` because the LLVM type string parser doesn't handle the `%struct.Name` prefix. This kills field sensitivity for all C++ code.

**Files:**
- Modify: `crates/saf-frontends/src/llvm/type_intern.rs`
- Test: `make test` (add unit test in type_intern.rs)

**Step 1: Read the current parser**

Read `crates/saf-frontends/src/llvm/type_intern.rs`, specifically the `parse_llvm_type_string` method. Find where `%struct.Foo` falls through to `Opaque`.

**Step 2: Write a failing test**

Add to the test module in `type_intern.rs`:

```rust
#[test]
fn named_struct_type_parsed_correctly() {
    let interner = TypeInterner::new();
    // %struct.Foo = type { i32, ptr }
    let type_str = "%struct.Foo = type { i32, ptr }";
    let (ty, _id) = interner.intern_from_string(type_str);
    assert!(
        !matches!(ty, AirType::Opaque),
        "Named struct should not be Opaque, got: {:?}",
        ty
    );
    if let AirType::Struct { fields } = &ty {
        assert_eq!(fields.len(), 2);
    } else {
        panic!("Expected Struct, got {:?}", ty);
    }
}
```

**Step 3: Fix the parser**

In `parse_llvm_type_string`, add handling for named struct types. When the string starts with `%`, extract the body after `= type` and parse it as an anonymous struct:

```rust
// If the type string starts with %, it's a named type like "%struct.Foo = type { i32, ptr }"
if s.starts_with('%') {
    if let Some(body_start) = s.find("= type ") {
        let body = &s[body_start + 7..]; // skip "= type "
        return self.parse_llvm_type_string(body); // recurse on the body
    }
    // Named type reference without definition (e.g., just "%struct.Foo")
    // This is a forward reference — return Opaque
    return AirType::Opaque;
}
```

Note: Also handle the case where the LLVM `print_to_string()` returns just `%struct.Foo` (a reference without the full definition). For those, we need to check if we've seen the definition before. The TypeInterner may need a name→TypeId map.

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Run PTABen to verify no regression**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-struct-fix.json'
```

Read the results file and verify Unsound count is <= 68.

**Step 6: Commit**

```
fix(frontends): parse named struct types instead of treating as Opaque
```

---

### Task 1.3: Add `function_by_name` Index to AirModule

`AirModule::function_by_name()` is O(N) linear scan. Add a name→index map.

**Files:**
- Modify: `crates/saf-core/src/air.rs`
- Test: `make test`

**Step 1: Add the index field**

In `AirModule`:

```rust
/// Index from function name to position in `functions` vec.
#[serde(skip)]
name_index: BTreeMap<String, usize>,
```

**Step 2: Build index alongside function_index**

In the `rebuild_indexes()` method (or wherever `function_index` is built):

```rust
self.name_index = self.functions.iter().enumerate()
    .map(|(i, f)| (f.name.clone(), i))
    .collect();
```

**Step 3: Update function_by_name to use the index**

```rust
pub fn function_by_name(&self, name: &str) -> Option<&AirFunction> {
    self.name_index.get(name).map(|&i| &self.functions[i])
}
```

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Commit**

```
perf(core): add name index to AirModule for O(log n) function_by_name lookup
```

---

### Task 1.4: Implement `extract_function_symbol` for C++ Demangled Names

`extract_function_symbol` in `debug_info.rs` returns `None` unconditionally. Implement it so C++ demangled names appear in AIR.

**Files:**
- Modify: `crates/saf-frontends/src/llvm/debug_info.rs`
- Test: `make test`

**Step 1: Read the current stub**

Read `crates/saf-frontends/src/llvm/debug_info.rs` and find `extract_function_symbol`.

**Step 2: Implement using LLVM debug metadata**

Use `inkwell`'s `get_subprogram()` API on the function value to extract the `DISubprogram` metadata. From it, extract:
- `name` → `Symbol.display_name`
- `linkage_name` → `Symbol.mangled_name`

If no subprogram metadata exists, fall back to the LLVM function name.

```rust
pub fn extract_function_symbol(func: &FunctionValue) -> Option<Symbol> {
    // Try to get DISubprogram metadata
    if let Some(subprogram) = func.get_subprogram() {
        let display_name = subprogram.get_name().to_str().ok()?.to_string();
        let mangled_name = func.get_name().to_str().ok().map(String::from);
        Some(Symbol {
            display_name,
            mangled_name,
            namespace_path: Vec::new(), // Could be extracted from DIScope chain
        })
    } else {
        // Fall back to LLVM function name
        let name = func.get_name().to_str().ok()?.to_string();
        Some(Symbol {
            display_name: name.clone(),
            mangled_name: Some(name),
            namespace_path: Vec::new(),
        })
    }
}
```

Note: Check what APIs inkwell exposes for `get_subprogram()`. If not available, use raw `llvm_sys` calls. The exact API depends on inkwell version — read the inkwell source first.

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
feat(frontends): implement extract_function_symbol for debug info names
```

---

## Phase 2: Core Extension Points (P1A)

**Estimated effort:** 1 session (2-3 hours)
**Dependencies:** None (can be done in parallel with Phase 1)
**Impact:** Foundation for all future extensibility — AirVisitor, EntityId, instruction extensions, open config

### Task 2.1: Add `AirVisitor` Trait and `walk_module`

**Files:**
- Create: `crates/saf-core/src/air_visitor.rs`
- Modify: `crates/saf-core/src/lib.rs` (add `pub mod air_visitor;`)
- Test: Add unit tests in the new module

**Step 1: Create the visitor trait**

```rust
// crates/saf-core/src/air_visitor.rs

use crate::air::{AirModule, AirFunction, AirBlock, Instruction};

/// Visitor trait for traversing AIR module structures.
///
/// Default implementations are no-ops, so visitors only need to
/// implement the methods they care about. Return `false` from
/// `visit_function` or `visit_block` to skip visiting children.
pub trait AirVisitor {
    /// Called for each function. Return `false` to skip blocks/instructions.
    fn visit_function(&mut self, func: &AirFunction) -> bool { let _ = func; true }

    /// Called for each block. Return `false` to skip instructions.
    fn visit_block(&mut self, func: &AirFunction, block: &AirBlock) -> bool {
        let _ = (func, block); true
    }

    /// Called for each instruction.
    fn visit_instruction(&mut self, func: &AirFunction, block: &AirBlock, inst: &Instruction) {
        let _ = (func, block, inst);
    }
}

/// Walk an entire module, dispatching to the visitor.
pub fn walk_module(module: &AirModule, visitor: &mut impl AirVisitor) {
    for func in &module.functions {
        if !visitor.visit_function(func) {
            continue;
        }
        for block in &func.blocks {
            if !visitor.visit_block(func, block) {
                continue;
            }
            for inst in &block.instructions {
                visitor.visit_instruction(func, block, inst);
            }
        }
    }
}

/// Walk a single function.
pub fn walk_function(func: &AirFunction, visitor: &mut impl AirVisitor) {
    if !visitor.visit_function(func) {
        return;
    }
    for block in &func.blocks {
        if !visitor.visit_block(func, block) {
            continue;
        }
        for inst in &block.instructions {
            visitor.visit_instruction(func, block, inst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InstructionCounter {
        count: usize,
    }

    impl AirVisitor for InstructionCounter {
        fn visit_instruction(&mut self, _: &AirFunction, _: &AirBlock, _: &Instruction) {
            self.count += 1;
        }
    }

    // Add a test using a minimal AirModule fixture
}
```

**Step 2: Register the module**

In `crates/saf-core/src/lib.rs`:

```rust
pub mod air_visitor;
```

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
feat(core): add AirVisitor trait and walk_module for IR traversal
```

---

### Task 2.2: Add `EntityId` Trait

**Files:**
- Modify: `crates/saf-core/src/ids.rs`
- Test: Unit tests in ids.rs

**Step 1: Add the trait**

In `crates/saf-core/src/ids.rs`, before the macro:

```rust
/// Common interface for all SAF entity ID types.
pub trait EntityId: Copy + Eq + Ord + std::hash::Hash + std::fmt::Display + std::fmt::Debug {
    /// Get the raw u128 value.
    fn raw(self) -> u128;

    /// Format as hex string with 0x prefix.
    fn to_hex(self) -> String {
        crate::id::id_to_hex(self.raw())
    }
}
```

**Step 2: Add `impl EntityId` to the `define_id_type!` macro**

Inside the macro expansion, add:

```rust
impl EntityId for $name {
    fn raw(self) -> u128 { self.0 }
}
```

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
feat(core): add EntityId trait for generic ID operations
```

---

### Task 2.3: Add `extensions` Bag to Instruction

**Files:**
- Modify: `crates/saf-core/src/air.rs`
- Test: Unit test for serialization round-trip

**Step 1: Add the field**

In the `Instruction` struct:

```rust
/// Frontend-specific extension data.
/// Analyses that don't understand these extensions should ignore them.
#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
pub extensions: BTreeMap<String, serde_json::Value>,
```

**Step 2: Add serde_json dependency to saf-core if not already present**

Check `crates/saf-core/Cargo.toml`. If `serde_json` is not a dependency, add it.

**Step 3: Update any Instruction construction sites**

Search for places that construct `Instruction { ... }` and add `extensions: BTreeMap::new()` (or use `Default::default()` if Instruction derives Default).

**Step 4: Add test**

```rust
#[test]
fn instruction_extensions_roundtrip() {
    let mut inst = Instruction { /* minimal fields */, extensions: BTreeMap::new() };
    inst.extensions.insert("llvm.landingpad".to_string(), serde_json::json!({"cleanup": true}));

    let json = serde_json::to_string(&inst).unwrap();
    let roundtripped: Instruction = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtripped.extensions.len(), 1);

    // Empty extensions should not appear in JSON
    inst.extensions.clear();
    let json2 = serde_json::to_string(&inst).unwrap();
    assert!(!json2.contains("extensions"));
}
```

**Step 5: Run tests**

```bash
make fmt && make lint && make test
```

**Step 6: Commit**

```
feat(core): add extensions bag to Instruction for frontend-specific metadata
```

---

### Task 2.4: Open `Config.frontend` to String-Based Lookup

**Files:**
- Modify: `crates/saf-core/src/config.rs`
- Modify: Any files that match on `Config.frontend` (search for pattern matches)
- Test: `make test`

**Step 1: Read current usage**

Search for all uses of `config.frontend` and `Frontend::Llvm` / `Frontend::AirJson` across the codebase to understand the blast radius.

**Step 2: Change the Frontend field to String**

Replace the `Frontend` enum approach. Keep the enum for backward compat but add a string-based field:

```rust
/// Frontend identifier. Known values: "llvm", "air-json".
/// New frontends can use any string without modifying saf-core.
pub frontend_id: String,
```

Or, less invasively, add an `Other(String)` variant to the existing enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Frontend {
    Llvm,
    AirJson,
    /// Custom frontend — the string is the frontend_id.
    #[serde(untagged)]
    Other(String),
}
```

Evaluate which approach is less disruptive by checking how many `match` sites exist.

**Step 3: Update match sites**

Add `Frontend::Other(_) => { ... }` arms to all match statements, with appropriate fallback behavior.

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Commit**

```
feat(core): support custom frontend identifiers via Frontend::Other variant
```

---

### Task 2.5: Add Escape Hatches to Closed Enums

**Files:**
- Modify: `crates/saf-core/src/spec/types.rs` (Role)
- Modify: `crates/saf-analysis/src/checkers/spec.rs` (SitePattern)
- Modify: `crates/saf-analysis/src/ifds/edge_fn.rs` (BuiltinEdgeFn)
- Test: `make test`

**Step 1: Add `Role::Custom(String)`**

In `crates/saf-core/src/spec/types.rs`:

```rust
pub enum Role {
    // ... existing variants ...
    /// Custom role for domain-specific analyses.
    Custom(String),
}
```

Update any match arms on `Role` to add `Role::Custom(name) => { ... }`.

**Step 2: Add `SitePattern::Custom` variant**

In `crates/saf-analysis/src/checkers/spec.rs`:

```rust
pub enum SitePattern {
    // ... existing variants ...
    /// Custom pattern defined by a predicate function name.
    /// The name is resolved at runtime by the checker runner.
    CustomPredicate(String),
}
```

Note: For serialization safety, use a string identifier rather than `Arc<dyn Fn>` — the predicate is resolved at runtime by a registry pattern.

**Step 3: Add `BuiltinEdgeFn::Custom` variant**

In `crates/saf-analysis/src/ifds/edge_fn.rs`:

```rust
pub enum BuiltinEdgeFn<V: Lattice> {
    // ... existing variants ...
    /// Custom edge function for analyses that need non-standard value transforms.
    Custom(Arc<dyn Fn(V) -> V + Send + Sync>),
}
```

Implement `apply()` for the Custom variant: `BuiltinEdgeFn::Custom(f) => f(input)`.

Handle `Eq`, `Hash`, `Clone` for the Custom variant — use pointer identity for Eq/Hash and Arc::clone for Clone.

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Commit**

```
feat(core,analysis): add escape hatches to Role, SitePattern, and BuiltinEdgeFn enums
```

---

## Phase 3: Configuration & Python API (P1B)

**Estimated effort:** 1 session (2-3 hours)
**Dependencies:** None
**Impact:** Users can customize analysis; dead config paths are wired

### Task 3.1: Expose PipelineConfig to Python

**Files:**
- Modify: `crates/saf-python/src/project.rs`
- Test: Python integration test

**Step 1: Read current hardcoded config**

Read `crates/saf-python/src/project.rs` and find `build_analysis()` or wherever `PipelineConfig` / `PtaConfig` / `RefinementConfig` is constructed. Note the hardcoded values.

**Step 2: Add keyword arguments to `Project.open()`**

Add optional Python parameters that map to `PipelineConfig` fields:

```rust
#[pymethods]
impl Project {
    #[staticmethod]
    #[pyo3(signature = (
        path,
        pta_max_iterations=None,
        field_sensitivity_depth=None,
        vf_mode=None,
        max_refinement_iterations=None,
        pts_repr=None,
    ))]
    fn open(
        path: &str,
        pta_max_iterations: Option<usize>,
        field_sensitivity_depth: Option<usize>,
        vf_mode: Option<&str>,
        max_refinement_iterations: Option<usize>,
        pts_repr: Option<&str>,
    ) -> PyResult<Self> {
        // Use provided values or fall back to current defaults
        let max_iter = pta_max_iterations.unwrap_or(10_000);
        let depth = field_sensitivity_depth.unwrap_or(2);
        // ... build config from these values ...
    }
}
```

**Step 3: Write a Python test**

```python
def test_custom_config():
    project = saf.Project.open(
        "tests/fixtures/llvm/e2e/simple_pointer.ll",
        pta_max_iterations=100,
        vf_mode="fast",
    )
    # Should succeed without error
    cg = project.callgraph()
    assert cg is not None
```

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Commit**

```
feat(python): expose PipelineConfig parameters to Project.open()
```

---

### Task 3.2: Wire `Config.analysis.mode` Through to PipelineConfig

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs`
- Modify: `crates/saf-python/src/project.rs`
- Test: `make test`

**Step 1: Read Config.analysis.mode**

Read `crates/saf-core/src/config.rs` to see `AnalysisMode::Fast` and `AnalysisMode::Precise`.

**Step 2: Map AnalysisMode to PipelineConfig defaults**

In `pipeline.rs`, add a constructor:

```rust
impl PipelineConfig {
    /// Create a PipelineConfig from an AnalysisMode with appropriate defaults.
    pub fn from_mode(mode: AnalysisMode) -> Self {
        match mode {
            AnalysisMode::Fast => PipelineConfig {
                refinement: RefinementConfig {
                    max_iterations: 1_000,
                    pta_config: PtaConfig {
                        field_sensitivity: FieldSensitivity::Insensitive,
                        max_iterations: 1_000,
                        ..PtaConfig::default()
                    },
                    ..RefinementConfig::default()
                },
                valueflow: ValueFlowConfig {
                    mode: ValueFlowMode::Fast,
                    ..ValueFlowConfig::default()
                },
                ..PipelineConfig::default()
            },
            AnalysisMode::Precise => PipelineConfig::default(), // current defaults are "precise"
        }
    }
}
```

**Step 3: Use in Python when no explicit config given**

```rust
// In Project::open(), if no explicit config params are provided:
let config = PipelineConfig::from_mode(AnalysisMode::Precise);
```

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Commit**

```
feat(analysis): wire Config.analysis.mode to PipelineConfig defaults
```

---

### Task 3.3: Add `CoreError` Type

**Files:**
- Modify: `crates/saf-core/src/error.rs`
- Modify: `crates/saf-core/src/lib.rs` (re-export)
- Test: Unit test

**Step 1: Implement CoreError**

```rust
// crates/saf-core/src/error.rs

use thiserror::Error;

/// Errors from saf-core operations.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Schema version mismatch during deserialization.
    #[error("schema version mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },

    /// Invalid ID format.
    #[error("invalid entity ID: {0}")]
    InvalidId(String),

    /// Spec registry error.
    #[error("spec error: {0}")]
    Spec(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Step 2: Re-export from lib.rs**

```rust
pub use error::CoreError;
```

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
feat(core): add CoreError type for structured error handling
```

---

## Phase 4: AnalysisPass Framework (P2A)

**Estimated effort:** 1-2 sessions (3-5 hours)
**Dependencies:** Phases 1-3 (for the caching pattern)
**Impact:** Composable, cacheable analysis pipeline. This is the highest-impact architectural change.

### Task 4.1: Define `AnalysisPass` Trait and `AnalysisContext`

**Files:**
- Create: `crates/saf-analysis/src/pass.rs`
- Modify: `crates/saf-analysis/src/lib.rs`
- Test: Unit tests

**Step 1: Design the trait**

```rust
// crates/saf-analysis/src/pass.rs

use std::any::Any;
use std::collections::BTreeMap;
use saf_core::air::AirModule;

/// Unique identifier for an analysis pass.
pub type PassId = &'static str;

/// Accumulated results from prior analysis passes.
///
/// Each pass reads results from previous passes and writes its own.
/// The context is a type-erased bag keyed by `PassId`.
pub struct AnalysisContext {
    results: BTreeMap<PassId, Box<dyn Any + Send + Sync>>,
}

impl AnalysisContext {
    pub fn new() -> Self {
        Self { results: BTreeMap::new() }
    }

    /// Store a pass result.
    pub fn insert<T: Any + Send + Sync>(&mut self, pass_id: PassId, result: T) {
        self.results.insert(pass_id, Box::new(result));
    }

    /// Retrieve a pass result by type.
    pub fn get<T: Any + Send + Sync>(&self, pass_id: PassId) -> Option<&T> {
        self.results.get(pass_id)?.downcast_ref()
    }

    /// Check if a pass result exists.
    pub fn has(&self, pass_id: PassId) -> bool {
        self.results.contains_key(pass_id)
    }
}

/// Trait for composable analysis passes.
///
/// Each pass declares its dependencies, runs on an `AirModule` with access
/// to prior results via `AnalysisContext`, and stores its result back.
pub trait AnalysisPass: Send + Sync {
    /// Unique identifier for this pass (e.g., "defuse", "pta", "valueflow").
    fn id(&self) -> PassId;

    /// IDs of passes that must run before this one.
    fn dependencies(&self) -> &[PassId] { &[] }

    /// Execute the pass, reading from `ctx` and storing results back into it.
    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError>;
}

use crate::error::AnalysisError;
```

**Step 2: Register the module**

In `crates/saf-analysis/src/lib.rs`:

```rust
pub mod pass;
pub use pass::{AnalysisPass, AnalysisContext, PassId};
```

**Step 3: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct CountPass;

    impl AnalysisPass for CountPass {
        fn id(&self) -> PassId { "count" }

        fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
            let count = module.functions.len();
            ctx.insert("count", count);
            Ok(())
        }
    }

    #[test]
    fn pass_stores_and_retrieves_result() {
        let module = /* minimal AirModule fixture */;
        let mut ctx = AnalysisContext::new();
        let pass = CountPass;
        pass.run(&module, &mut ctx).unwrap();
        assert!(ctx.has("count"));
        let count: &usize = ctx.get("count").unwrap();
        assert!(*count >= 0);
    }
}
```

**Step 4: Run tests**

```bash
make fmt && make lint && make test
```

**Step 5: Commit**

```
feat(analysis): add AnalysisPass trait and AnalysisContext for composable pipelines
```

---

### Task 4.2: Implement PassManager with Dependency Resolution

**Files:**
- Modify: `crates/saf-analysis/src/pass.rs`
- Test: Unit tests

**Step 1: Add PassManager**

```rust
/// Manages and executes a sequence of analysis passes in dependency order.
pub struct PassManager {
    passes: Vec<Box<dyn AnalysisPass>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Register a pass. Passes will be topologically sorted before execution.
    pub fn register(&mut self, pass: Box<dyn AnalysisPass>) {
        self.passes.push(pass);
    }

    /// Execute all registered passes in dependency order.
    pub fn run_all(&self, module: &AirModule) -> Result<AnalysisContext, AnalysisError> {
        let ordered = self.topological_sort()?;
        let mut ctx = AnalysisContext::new();
        for pass in &ordered {
            pass.run(module, &mut ctx)?;
        }
        Ok(ctx)
    }

    fn topological_sort(&self) -> Result<Vec<&dyn AnalysisPass>, AnalysisError> {
        // Standard topological sort using dependencies
        // Error on missing dependencies or cycles
        // ...
    }
}
```

**Step 2: Write tests for dependency resolution**

```rust
#[test]
fn pass_manager_respects_dependencies() {
    let mut pm = PassManager::new();
    // Register passes in reverse order — PM should still run them correctly
    pm.register(Box::new(PassB)); // depends on "a"
    pm.register(Box::new(PassA)); // no dependencies
    let ctx = pm.run_all(&module).unwrap();
    // Verify both results exist
    assert!(ctx.has("a"));
    assert!(ctx.has("b"));
}
```

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
feat(analysis): add PassManager with topological dependency resolution
```

---

### Task 4.3: Wrap Existing Pipeline Stages as Passes

**Files:**
- Create: `crates/saf-analysis/src/passes/mod.rs`
- Create: `crates/saf-analysis/src/passes/defuse_pass.rs`
- Create: `crates/saf-analysis/src/passes/pta_pass.rs`
- Create: `crates/saf-analysis/src/passes/valueflow_pass.rs`
- Modify: `crates/saf-analysis/src/lib.rs`
- Test: Integration test

**Step 1: Create DefUsePass**

```rust
// crates/saf-analysis/src/passes/defuse_pass.rs
use crate::pass::{AnalysisPass, AnalysisContext, PassId};
use crate::defuse::DefUseGraph;
use saf_core::air::AirModule;
use crate::error::AnalysisError;

pub struct DefUsePass;

impl AnalysisPass for DefUsePass {
    fn id(&self) -> PassId { "defuse" }

    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let graph = DefUseGraph::build(module);
        ctx.insert("defuse", graph);
        Ok(())
    }
}
```

**Step 2: Create PtaPass (wrapping cg_refinement::refine)**

```rust
// crates/saf-analysis/src/passes/pta_pass.rs
pub struct PtaPass {
    pub config: RefinementConfig,
    pub specs: Option<SpecRegistry>,
}

impl AnalysisPass for PtaPass {
    fn id(&self) -> PassId { "pta" }

    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let result = crate::cg_refinement::refine(module, &self.config, self.specs.as_ref());
        ctx.insert("pta", result);
        Ok(())
    }
}
```

**Step 3: Create ValueFlowPass**

```rust
// crates/saf-analysis/src/passes/valueflow_pass.rs
pub struct ValueFlowPass {
    pub config: ValueFlowConfig,
}

impl AnalysisPass for ValueFlowPass {
    fn id(&self) -> PassId { "valueflow" }
    fn dependencies(&self) -> &[PassId] { &["defuse", "pta"] }

    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let defuse = ctx.get::<DefUseGraph>("defuse")
            .ok_or_else(|| AnalysisError::Config("defuse pass not run".into()))?;
        let pta = ctx.get::<RefinementResult>("pta")
            .ok_or_else(|| AnalysisError::Config("pta pass not run".into()))?;
        let vfg = crate::build_valueflow(&self.config, module, defuse, &pta.call_graph, pta.pta_result.as_ref());
        ctx.insert("valueflow", vfg);
        Ok(())
    }
}
```

**Step 4: Keep backward compat — `run_pipeline` delegates to PassManager internally**

The existing `run_pipeline()` function should use the new PassManager internally but maintain its existing signature and return type. This is a refactor, not a breaking change.

**Step 5: Run tests**

```bash
make fmt && make lint && make test
```

All existing consumers of `run_pipeline` should continue working.

**Step 6: Commit**

```
refactor(analysis): wrap pipeline stages as AnalysisPass implementations
```

---

## Phase 5: Generic Abstract Interpretation (P2B)

**Estimated effort:** 1 session (3-4 hours)
**Dependencies:** Phase 4 (AnalysisContext for domain-agnostic results)
**Impact:** Pluggable abstract domains — new domains work with the standard fixpoint engine

### Task 5.1: Unify Lattice and AbstractDomain Traits

**Files:**
- Modify: `crates/saf-analysis/src/absint/domain.rs`
- Modify: `crates/saf-analysis/src/ifds/lattice.rs`
- Modify: All implementors of `Lattice` (search for `impl Lattice`)
- Test: `make test`

**Step 1: Make AbstractDomain extend Lattice**

In `absint/domain.rs`:

```rust
use crate::ifds::lattice::Lattice;

pub trait AbstractDomain: Lattice + Clone + std::fmt::Debug {
    fn is_bottom(&self) -> bool { self.leq(&Self::bottom()) }
    fn is_top(&self) -> bool { Self::top().leq(self) }
    fn widen(&self, other: &Self) -> Self { self.join(other) }
    fn narrow(&self, other: &Self) -> Self { self.meet(other) }
}
```

Note: `Lattice` requires `Ord` but `AbstractDomain` does not currently. Check if all `AbstractDomain` implementors already implement `Ord`. If not, either relax `Lattice`'s `Ord` bound or add `Ord` to domain types. Evaluate which is less disruptive.

If `Ord` is too restrictive for `AbstractDomain`, keep them separate but add a blanket impl:

```rust
// Any type that implements AbstractDomain also satisfies Lattice
// (if it also has Ord)
```

This may require a case-by-case evaluation. Read the existing implementors first.

**Step 2: Update all implementations**

For each type that implements both `Lattice` and `AbstractDomain`, remove the `AbstractDomain` lattice methods and keep only `widen`/`narrow`/`is_bottom`/`is_top` overrides.

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
refactor(analysis): unify Lattice and AbstractDomain trait hierarchy
```

---

### Task 5.2: Create Generic TransferFn Trait

**Files:**
- Create: `crates/saf-analysis/src/absint/transfer_fn.rs`
- Modify: `crates/saf-analysis/src/absint/mod.rs`
- Test: Unit test

**Step 1: Define the trait**

```rust
// crates/saf-analysis/src/absint/transfer_fn.rs

use crate::absint::domain::AbstractDomain;
use saf_core::air::{AirFunction, AirBlock, Instruction};

/// Abstract transfer function for a specific domain.
///
/// Implementors define how instructions transform abstract states.
pub trait TransferFn<D: AbstractDomain> {
    /// Apply the transfer function for a single instruction.
    fn transfer_instruction(
        &self,
        inst: &Instruction,
        pre_state: &D,
        func: &AirFunction,
        block: &AirBlock,
    ) -> D;
}
```

**Step 2: Wrap existing Interval transfer as an implementation**

Create `IntervalTransfer` struct that implements `TransferFn<Interval>` by delegating to the existing `transfer_instruction_with_context` function.

**Step 3: Run tests**

```bash
make fmt && make lint && make test
```

**Step 4: Commit**

```
feat(analysis): add generic TransferFn trait for pluggable abstract domains
```

---

### Task 5.3: Make Fixpoint Solver Generic

**Files:**
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs`
- Test: `make test`

**Step 1: Parameterize the solver**

This is the most complex task in this phase. The current `solve_abstract_interp` is hardwired to `AbstractState` (which wraps `Interval`). The goal is to make it generic:

```rust
pub fn solve_generic<D, T>(
    func: &AirFunction,
    transfer: &T,
    config: &AbstractInterpConfig,
    // ... optional PTA, specs, etc.
) -> GenericAbstractInterpResult<D>
where
    D: AbstractDomain,
    T: TransferFn<D>,
{
    // The same fixpoint algorithm, but using D instead of AbstractState
    // and T::transfer_instruction instead of the hardcoded transfer
}
```

**Important:** Keep the existing `solve_abstract_interp` function as a thin wrapper that calls `solve_generic::<Interval, IntervalTransfer>(...)`. This maintains backward compatibility.

**Step 2: Run tests**

```bash
make fmt && make lint && make test
```

All existing abstract interpretation tests must pass unchanged.

**Step 3: Commit**

```
refactor(analysis): make fixpoint solver generic over AbstractDomain and TransferFn
```

---

## Phase 6: Logging & Diagnostics (P2C)

**Estimated effort:** 1 session (2-3 hours)
**Dependencies:** None
**Impact:** Observable analysis progress, machine-readable profiling

### Task 6.1: Add `#[derive(Serialize)]` to SolverStats

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver_stats.rs`
- Test: Verify serialization compiles

Add `#[derive(Serialize, Deserialize)]` to `SolverStats` and all inner types. Keep `#[cfg(feature = "solver-stats")]` gating.

---

### Task 6.2: Add `tracing::info_span!` to Pipeline Stages

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs`
- Modify: `crates/saf-analysis/src/cg_refinement.rs`
- Test: `make test`

Wrap each major pipeline stage in a tracing span:

```rust
pub fn run_pipeline(module: &AirModule, config: &PipelineConfig) -> PipelineResult {
    let _defuse_span = tracing::info_span!("defuse_build").entered();
    let defuse = DefUseGraph::build(module);
    drop(_defuse_span);

    let _pta_span = tracing::info_span!("cg_refinement").entered();
    let refinement = cg_refinement::refine(module, &config.refinement, config.specs.as_ref());
    drop(_pta_span);

    let _vf_span = tracing::info_span!("valueflow_build").entered();
    let valueflow = build_valueflow(...);
    drop(_vf_span);

    // ...
}
```

---

### Task 6.3: Add PipelineStats to PipelineResult

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs`
- Test: `make test`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub defuse_build_secs: f64,
    pub pta_solve_secs: f64,
    pub refinement_iterations: usize,
    pub valueflow_build_secs: f64,
    pub total_secs: f64,
}
```

Add `stats: PipelineStats` to `PipelineResult`. Populate using `Instant::now()` around each stage.

---

### Task 6.4: Migrate SolverStats.print_summary to tracing

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver_stats.rs`
- Test: `make test`

Replace `eprintln!` calls in `print_summary()` with `tracing::debug!` structured events. Keep the human-readable format as the default tracing output.

---

**Commit all Phase 6 together:**

```
feat(analysis): add structured logging, PipelineStats, and serializable SolverStats
```

---

## Phase 7: Memory & Performance (P3A)

**Estimated effort:** 1 session (2-3 hours)
**Dependencies:** None
**Impact:** Reduced memory usage for large programs

### Task 7.1: Use `Arc<LocationFactory>` in PtaResult

**Files:**
- Modify: `crates/saf-analysis/src/pta/result.rs`
- Modify: `crates/saf-analysis/src/pta/location.rs`
- Modify: Any code that constructs `PtaResult`
- Test: `make test` + PTABen

Replace the `O(N)` clone of all locations in `PtaResult::new()` with `Arc<LocationFactory>`:

```rust
pub struct PtaResult {
    // Change from BTreeMap<LocId, Location> to:
    locations: Arc<LocationFactory>,
    points_to: BTreeMap<ValueId, BTreeSet<LocId>>,
    // ...
}
```

Update `PtaResult::new()` to take `Arc<LocationFactory>` instead of cloning the map.

---

### Task 7.2: Implement Fingerprint-Keyed AirBundle Disk Cache

**Files:**
- Create: `crates/saf-core/src/cache.rs`
- Modify: `crates/saf-core/src/lib.rs`
- Test: Unit test

```rust
// crates/saf-core/src/cache.rs

use std::path::{Path, PathBuf};
use crate::air::AirBundle;

/// Simple filesystem cache for AirBundle results.
pub struct BundleCache {
    cache_dir: PathBuf,
}

impl BundleCache {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self { cache_dir: cache_dir.into() }
    }

    /// Try to load a cached bundle for the given fingerprint.
    pub fn get(&self, fingerprint: &[u8]) -> Option<AirBundle> {
        let key = hex::encode(fingerprint);
        let path = self.cache_dir.join(format!("{key}.air.json"));
        if path.exists() {
            let data = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    /// Store a bundle under the given fingerprint.
    pub fn put(&self, fingerprint: &[u8], bundle: &AirBundle) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.cache_dir)?;
        let key = hex::encode(fingerprint);
        let path = self.cache_dir.join(format!("{key}.air.json"));
        let data = serde_json::to_string(bundle)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, data)
    }
}
```

This is opt-in — consumers choose to use it. Don't wire it into the frontend automatically.

---

**Commit Phase 7:**

```
perf(analysis,core): Arc<LocationFactory> in PtaResult + AirBundle disk cache
```

---

## Phase 8: AIR Multi-Language Type Support (P3B)

**Estimated effort:** 1 session (3-4 hours)
**Dependencies:** None
**Impact:** Required for Java/WASM/Rust frontends

### Task 8.1: Add `AirType::Reference` and `AirType::Vector`

**Files:**
- Modify: `crates/saf-core/src/air.rs`
- Modify: `crates/saf-core/src/layout.rs` (layout computation for new types)
- Modify: `crates/saf-frontends/src/air_json_schema.rs` (JSON schema)
- Test: Serialization round-trip tests

```rust
pub enum AirType {
    // ... existing variants ...

    /// Non-null reference type (for Java, Rust, Kotlin).
    /// Distinct from Pointer in that it cannot be null.
    Reference {
        /// Whether this reference can be null.
        nullable: bool,
    },

    /// SIMD vector type.
    Vector {
        element: TypeId,
        lanes: u32,
    },
}
```

Update `layout.rs` to handle these types:
- `Reference` → same size as `Pointer` (8 bytes on 64-bit, 4 on 32-bit)
- `Vector` → `element_size * lanes`

### Task 8.2: Add `target_pointer_width` to AirModule

**Files:**
- Modify: `crates/saf-core/src/air.rs`
- Modify: `crates/saf-core/src/layout.rs`
- Modify: `crates/saf-frontends/src/llvm/mapping.rs` (populate from LLVM target)
- Test: `make test`

```rust
pub struct AirModule {
    // ... existing fields ...

    /// Target pointer width in bytes (4 for 32-bit, 8 for 64-bit).
    /// Used by layout computation. Defaults to 8.
    #[serde(default = "default_pointer_width")]
    pub target_pointer_width: u32,
}

fn default_pointer_width() -> u32 { 8 }
```

Update `layout.rs` to use `module.target_pointer_width` instead of hardcoded 8.

### Task 8.3: Add `StructField::name`

**Files:**
- Modify: `crates/saf-core/src/air.rs`
- Modify: `crates/saf-frontends/src/llvm/type_intern.rs` (populate from debug info if available)
- Test: `make test`

```rust
pub struct StructField {
    pub field_type: TypeId,
    pub byte_offset: u64,
    pub byte_size: u64,
    /// Optional field name from debug info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
```

---

**Commit Phase 8:**

```
feat(core): add AirType::Reference, AirType::Vector, target_pointer_width, StructField.name
```

---

## Phase 9: API Stability & Testing (P3C)

**Estimated effort:** 1 session (2-3 hours)
**Dependencies:** Phases 1-8 (validates everything)
**Impact:** Production hardening, reduced public surface, faster compilation

### Task 9.1: Reduce `saf-analysis` Public Surface

**Files:**
- Modify: `crates/saf-analysis/src/lib.rs`
- Test: `make test` + ensure saf-python, saf-wasm, saf-bench still compile

Change `pub mod pta` to `mod pta` with explicit re-exports. The goal is that internal types like `constraint.rs`, `hvn.rs`, `clustering.rs` are not accessible outside the crate.

Carefully audit which types from `pta` are used by:
- `saf-python/src/` — search for `use saf_analysis::pta::`
- `saf-wasm/src/` — search for `use saf_analysis::pta::`
- `saf-bench/src/` — search for `use saf_analysis::pta::`

Only re-export what downstream crates actually use.

### Task 9.2: Add Fine-Grained Feature Flags

**Files:**
- Modify: `crates/saf-analysis/Cargo.toml`
- Modify: `crates/saf-analysis/src/lib.rs` (conditional compilation)
- Test: `make test`

```toml
[features]
default = ["z3-solver"]
z3-solver = ["dep:z3"]
solver-stats = []
# Gate heavy, rarely-used modules:
analysis-mta = []
analysis-cegar = ["z3-solver"]
analysis-invariants = ["z3-solver"]
```

Gate the modules:

```rust
#[cfg(feature = "analysis-mta")]
pub mod mta;

#[cfg(feature = "analysis-cegar")]
pub mod cegar;

#[cfg(feature = "analysis-invariants")]
pub mod invariants;
```

### Task 9.3: Add Smoke Test for saf-core

**Files:**
- Create: `crates/saf-core/tests/smoke.rs`
- Test: `make test`

```rust
//! Smoke test for saf-core — verifies basic AIR construction, ID derivation, and serialization.

use saf_core::air::*;
use saf_core::ids::*;

#[test]
fn air_module_round_trip() {
    let module = AirModule {
        id: ModuleId::derive(b"test"),
        name: "test".to_string(),
        functions: vec![],
        globals: vec![],
        types: std::collections::BTreeMap::new(),
        constants: std::collections::BTreeMap::new(),
        type_hierarchy: vec![],
        ..Default::default()
    };
    let json = serde_json::to_string(&module).unwrap();
    let roundtripped: AirModule = serde_json::from_str(&json).unwrap();
    assert_eq!(module.name, roundtripped.name);
}

#[test]
fn id_domain_separation() {
    let func_id = FunctionId::derive(b"main");
    let block_id = BlockId::derive(b"main");
    assert_ne!(func_id.raw(), block_id.raw(), "Different domains must produce different IDs");
}

#[test]
fn id_deterministic_across_calls() {
    let a = ValueId::derive(b"test_value");
    let b = ValueId::derive(b"test_value");
    assert_eq!(a, b, "Same input must produce identical IDs");
}
```

### Task 9.4: Add AIR-JSON Integration Tests (no Docker required)

**Files:**
- Create: `crates/saf-analysis/tests/air_json_pipeline.rs`
- Create: `tests/fixtures/air_json/simple_pta.air.json` (if not exists)
- Test: `make test`

Create integration tests that use `.air.json` fixtures instead of LLVM IR. These can run without Docker, improving developer iteration speed.

```rust
use saf_test_utils::load_air_json_fixture;

#[test]
fn air_json_pta_pipeline() {
    let bundle = load_air_json_fixture("simple_pta.air.json");
    let module = &bundle.module;
    let result = saf_analysis::pipeline::run_pipeline(module, &PipelineConfig::default());
    assert!(result.pta_result.is_some());
    assert!(!result.call_graph.nodes().is_empty());
}
```

---

**Commit Phase 9:**

```
refactor(analysis): reduce public API surface, add feature flags, smoke tests, and AIR-JSON integration tests
```

---

## Verification Checklist (Run After Each Phase)

```bash
# Inside Docker:
make fmt && make lint && make test

# PTABen regression check (run in background, takes 30-120s):
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-arch.json'

# Verify: Unsound count must be <= 68
python3 -c "
import json
with open('tests/benchmarks/ptaben/results-arch.json') as f:
    data = json.load(f)
print(f'Unsound: {data[\"summary\"][\"unsound\"]}')
assert data['summary']['unsound'] <= 68, 'PTABen regression!'
"
```

---

## Phase Dependency Graph

```
Phase 1 (Quick Wins) ────────────────────────────┐
Phase 2 (Core Extension Points) ─────────────────┤
Phase 3 (Config & API) ──────────────────────────┤
                                                   ├──→ Phase 9 (Stability)
Phase 4 (AnalysisPass) ──→ Phase 5 (Generic AI) ─┤
Phase 6 (Logging) ────────────────────────────────┤
Phase 7 (Memory) ─────────────────────────────────┤
Phase 8 (Multi-Language) ─────────────────────────┘
```

Phases 1, 2, 3, 6, 7, 8 are independent and can run in any order.
Phase 5 depends on Phase 4.
Phase 9 should run last as it validates everything.
