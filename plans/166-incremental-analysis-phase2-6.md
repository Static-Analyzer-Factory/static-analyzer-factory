# Phases 2–6: Incremental & Compositional Analysis — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan phase-by-phase.

**Goal:** Build on the Phase 0+1 multi-module infrastructure (Plan 165) to deliver per-module constraint caching, incremental PTA solving, selective ValueFlow rebuild, compositional function summaries, and full CLI/SDK integration.

**Design doc:** `plans/2026-02-25-incremental-analysis-design.md` (Sections 5–10)
**Prerequisite:** Plan 165 Phase 0+1 (done) — `AirProgram`, `LinkTable`, `merged_view()`, `ProgramDiff`, `CacheManifest`, `IncrementalConfig`, `Frontend::ingest_multi()`

**Architecture summary:**
```
Phase 2: Per-module constraint cache + constraint diffing + AnalysisSession
Phase 3: Incremental PTA solver + incremental CG refinement + selective VF rebuild
Phase 4: FunctionSummary type + spec unification + SummaryRegistry
Phase 5: Bottom-up compositional analysis + modular parallel analysis
Phase 6: CLI/SDK integration + E2E test suite + polish
```

---

## Phase 2: Per-Module Constraint Cache

**Objective:** Extract, cache, and diff constraints at module granularity. When a module hasn't changed, load its constraints from disk instead of re-extracting. Constraint diffing produces `(added, removed)` sets that feed Phase 3's incremental PTA.

### Task 2.1: `ModuleConstraints` type + serialization

**Files:**
- Create: `crates/saf-analysis/src/pta/module_constraints.rs`
- Modify: `crates/saf-analysis/src/pta/mod.rs` (add `pub mod module_constraints;`)

**Step 1: Define the type**

```rust
//! Per-module constraint extraction and caching.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use saf_core::ids::{FunctionId, ModuleId};

use super::constraint::{
    AddrConstraint, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};

/// Constraints extracted from a single module, suitable for caching.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleConstraints {
    pub module_id: ModuleId,
    pub fingerprint: String,
    pub addr: BTreeSet<AddrConstraint>,
    pub copy: BTreeSet<CopyConstraint>,
    pub load: BTreeSet<LoadConstraint>,
    pub store: BTreeSet<StoreConstraint>,
    pub gep: BTreeSet<GepConstraint>,
    pub function_ids: BTreeSet<FunctionId>,
}
```

**Step 2: Add `ConstraintSet` conversion**

```rust
impl ModuleConstraints {
    /// Convert to a `ConstraintSet` for solver consumption.
    pub fn to_constraint_set(&self) -> ConstraintSet { ... }

    /// Create from a `ConstraintSet` with module metadata.
    pub fn from_constraint_set(
        module_id: ModuleId,
        fingerprint: &str,
        constraints: &ConstraintSet,
        function_ids: BTreeSet<FunctionId>,
    ) -> Self { ... }
}
```

**Step 3: Add disk serialization**

```rust
impl ModuleConstraints {
    /// Save to `{cache_dir}/constraints/{module_id_hex}.json`.
    pub fn save(&self, cache_dir: &Path) -> Result<(), std::io::Error> { ... }

    /// Load from `{cache_dir}/constraints/{module_id_hex}.json`.
    pub fn load(cache_dir: &Path, module_id: &ModuleId) -> Result<Option<Self>, std::io::Error> { ... }
}
```

Use JSON for debuggability; switch to bincode later if size/speed matters.

**Step 4: Write tests**

- Round-trip: create → save → load → assert equal
- `to_constraint_set` → `from_constraint_set` round-trip
- Load missing file returns `None`

**Verification:** `cargo test -p saf-core` (local) + `make test` (Docker)

---

### Task 2.2: Per-module constraint extraction function

**Files:**
- Modify: `crates/saf-analysis/src/pta/extract.rs`

**Step 1: Add `extract_module_constraints()`**

```rust
/// Extract constraints for a single module within a multi-module program.
///
/// Unlike `extract_constraints()` which processes the entire (merged) module,
/// this extracts constraints scoped to the functions in `module`, using a
/// shared `LocationFactory` for cross-module location consistency.
pub fn extract_module_constraints(
    module: &AirModule,
    factory: &mut LocationFactory,
) -> ModuleConstraints {
    let constraints = extract_constraints(module, factory);
    let function_ids: BTreeSet<FunctionId> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| f.id)
        .collect();
    let fingerprint = saf_core::id::id_to_hex(module.id.0);
    ModuleConstraints::from_constraint_set(module.id, &fingerprint, &constraints, function_ids)
}
```

**Step 2: Write tests**

- Extract constraints from a small test module, verify function_ids populated
- Verify consistency: `extract_module_constraints(m).to_constraint_set()` ≈ `extract_constraints(m)`

---

### Task 2.3: `ProgramConstraints` — merged constraint view

**Files:**
- Modify: `crates/saf-analysis/src/pta/module_constraints.rs`

**Step 1: Define `ProgramConstraints`**

```rust
/// Constraints for an entire program, organized per-module with a merged view.
#[derive(Debug, Clone, Default)]
pub struct ProgramConstraints {
    pub modules: BTreeMap<ModuleId, ModuleConstraints>,
    pub merged: ConstraintSet,
}

impl ProgramConstraints {
    /// Build from per-module constraints by merging all into a single `ConstraintSet`.
    pub fn from_modules(modules: Vec<ModuleConstraints>) -> Self {
        let mut merged = ConstraintSet::default();
        let mut module_map = BTreeMap::new();
        for mc in modules {
            merged.addr.extend(mc.addr.iter().cloned());
            merged.copy.extend(mc.copy.iter().cloned());
            merged.load.extend(mc.load.iter().cloned());
            merged.store.extend(mc.store.iter().cloned());
            merged.gep.extend(mc.gep.iter().cloned());
            module_map.insert(mc.module_id, mc);
        }
        Self { modules: module_map, merged }
    }
}
```

**Step 2: Write tests**

- Two modules with non-overlapping constraints → merged contains all
- Two modules with duplicate constraint → merged deduplicates

---

### Task 2.4: Constraint diffing

**Files:**
- Modify: `crates/saf-analysis/src/pta/module_constraints.rs`

**Step 1: Define `ConstraintDiff`**

```rust
/// The difference between two program-level constraint sets.
#[derive(Debug, Clone, Default)]
pub struct ConstraintDiff {
    pub added: ConstraintSet,
    pub removed: ConstraintSet,
    pub changed_module_count: usize,
    pub unchanged_module_count: usize,
}

impl ProgramConstraints {
    /// Compute the constraint diff between `self` (previous) and `current`.
    ///
    /// Uses module-level granularity: for changed modules, the old module's
    /// constraints are "removed" and the new module's constraints are "added".
    /// Unchanged modules contribute nothing to the diff.
    pub fn diff(&self, current: &ProgramConstraints) -> ConstraintDiff { ... }
}
```

**Step 2: Implementation**

For each module in previous:
- If not in current → all its constraints are `removed`
- If in current with same fingerprint → unchanged
- If in current with different fingerprint → old constraints `removed`, new constraints `added`

For each module in current but not in previous → all its constraints are `added`.

**Step 3: Write tests**

- No change → empty diff
- One module added → added constraints equal that module's constraints
- One module removed → removed constraints equal that module's constraints
- One module changed → removed has old, added has new
- Module with identical fingerprint → unchanged

---

### Task 2.5: `AnalysisSession` — persistent session state

**Files:**
- Create: `crates/saf-analysis/src/session.rs`
- Modify: `crates/saf-analysis/src/lib.rs` (add `pub mod session;`)

**Step 1: Define the session type**

```rust
//! Analysis session state for incremental re-analysis.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use saf_core::ids::{FunctionId, ModuleId, ProgramId};

use crate::pta::module_constraints::ProgramConstraints;

/// Tracks the state of a previous analysis run for incremental re-analysis.
#[derive(Debug)]
pub struct AnalysisSession {
    pub cache_dir: PathBuf,
    pub program_id: Option<ProgramId>,
    pub previous_constraints: Option<ProgramConstraints>,
    pub function_staleness: BTreeMap<FunctionId, StalenessInfo>,
    pub run_count: u64,
}

/// Per-function staleness tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StalenessInfo {
    pub pta_version: u64,
    pub vf_version: u64,
    pub is_pta_stale: bool,
    pub is_vf_stale: bool,
}

impl AnalysisSession {
    /// Create a new session (first run).
    pub fn new(cache_dir: PathBuf) -> Self { ... }

    /// Load a previous session from the cache directory.
    pub fn load(cache_dir: &Path) -> Result<Self, std::io::Error> { ... }

    /// Save the current session state.
    pub fn save(&self) -> Result<(), std::io::Error> { ... }

    /// Record completion of a full or incremental analysis run.
    pub fn record_run(&mut self, program_id: ProgramId, constraints: ProgramConstraints) { ... }
}
```

**Step 2: Write tests**

- New session → load returns fresh state
- Save → load round-trip preserves state
- `record_run` increments `run_count`

**Verification:** `make test` in Docker

---

### Task 2.6: Wire constraint caching into pipeline

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs`

**Step 1: Add `IncrementalPipelineConfig`**

Extend `PipelineConfig` (or add a sibling) with:

```rust
pub struct IncrementalPipelineConfig {
    pub session: Option<AnalysisSession>,
    pub program: Option<AirProgram>,
    pub incremental_config: IncrementalConfig,
}
```

**Step 2: Add `run_pipeline_incremental()`**

```rust
/// Run analysis pipeline with incremental constraint caching.
///
/// 1. For each module: check cache → hit: load constraints, miss: extract + save
/// 2. Merge per-module constraints into `ProgramConstraints`
/// 3. If previous session exists: compute constraint diff
/// 4. Run PTA on merged constraints (full solve for now — Phase 3 will make this incremental)
/// 5. Build VFG (full rebuild for now — Phase 3 will make this selective)
/// 6. Update session state
pub fn run_pipeline_incremental(
    program: &AirProgram,
    config: &PipelineConfig,
    session: &mut AnalysisSession,
) -> PipelineResult { ... }
```

For now, this function extracts constraints per-module with caching, but still does full PTA solve on the merged set. The incremental solve comes in Phase 3.

**Step 3: Integration test**

Create `crates/saf-analysis/tests/incremental_pipeline.rs`:
- Load two-module fixture (main + lib)
- Run `run_pipeline_incremental` → verify results match `run_pipeline` on merged view
- Modify one module (lib_v2) → re-run → verify constraint cache hit for unchanged module
- Verify `ConstraintDiff` has correct added/removed sets

**Verification:** `make test` in Docker

---

### Task 2.7: Extend `CacheManifest` with constraint fingerprints

**Files:**
- Modify: `crates/saf-core/src/manifest.rs`

**Step 1: Add constraint hash to `ManifestEntry`**

```rust
pub struct ManifestEntry {
    pub module_id: ModuleId,
    pub fingerprint: String,
    pub input_path: String,
    pub constraint_hash: Option<String>,  // BLAKE3 of serialized ModuleConstraints
}
```

**Step 2: Update `diff()` to detect constraint staleness**

When `fingerprint` matches but `constraint_hash` doesn't (e.g., because extraction logic changed), flag the module as "constraint-stale" in a new `ManifestDiff` field.

**Step 3: Write tests**

- Same fingerprint + same constraint hash → truly unchanged
- Same fingerprint + different constraint hash → constraint-stale

**Verification:** `cargo test -p saf-core` (local)

---

## Phase 3: Incremental PTA & Selective Rebuild

**Objective:** Instead of full re-solving when constraints change, apply incremental updates to the PTA state. Only rebuild ValueFlow for affected functions.

### Task 3.1: Activate and upgrade `incremental.rs`

**Files:**
- Modify: `crates/saf-analysis/src/pta/incremental.rs`
- Modify: `crates/saf-analysis/src/pta/mod.rs` (remove `#[cfg(feature = "experimental")]`)

**Step 1: Remove feature gate**

Change `#[cfg(feature = "experimental")] mod incremental;` → `pub mod incremental;`

**Step 2: Implement constraint removal (lazy over-approximate)**

In `apply_incremental_update()`, instead of ignoring `removed_constraints`:

```rust
/// Lazy over-approximate removal strategy.
///
/// Removed constraints are not actively retracted from the PTS.
/// Instead, we track a "dirty generation" counter. After N incremental
/// updates, or if removed_constraints exceed a threshold, trigger a
/// full recompute to garbage-collect stale entries.
pub fn apply_incremental_update(
    state: &mut IncrementalPtaState,
    added_constraints: &ConstraintSet,
    removed_constraints: &ConstraintSet,
    config: &IncrementalConfig,
) -> IncrementalResult {
    // Track removal debt
    state.removal_debt += removed_constraints.total_count();

    // Add new constraints and mark affected nodes
    for addr in &added_constraints.addr {
        state.pts.entry(addr.ptr).or_default().add(addr.loc);
        state.mark_changed(addr.ptr);
    }
    for copy in &added_constraints.copy {
        state.copy_edges.entry(copy.dst).or_default().insert(copy.src);
        state.mark_changed(copy.dst);
    }
    // ... load, store similarly ...

    // Propagate until fixpoint
    propagate_worklist(state, config)
}
```

**Step 3: Add removal debt tracking + periodic GC**

```rust
impl IncrementalPtaState {
    /// Check if full recompute is needed due to accumulated removal debt.
    pub fn needs_gc(&self, threshold: usize) -> bool {
        self.removal_debt > threshold
    }

    /// Reset state for full recompute.
    pub fn reset(&mut self) { ... }
}
```

**Step 4: Write tests**

- Add constraints → incremental result matches full solve
- Add then remove → result is over-approximate (superset of full solve)
- GC threshold triggers → full recompute produces exact result

---

### Task 3.2: Bridge `IncrementalPtaState` to production `PointsToMap`

**Files:**
- Modify: `crates/saf-analysis/src/pta/incremental.rs`

**Step 1: Add conversion to/from `PointsToMap`**

The current `IncrementalPtaState` uses `BTreeSet<LocId>`. Bridge it to the production `PointsToMap` (`BTreeMap<ValueId, PtsSet>` with `BitVecPtsSet` support):

```rust
impl IncrementalPtaState {
    /// Initialize from a production PtaResult.
    pub fn from_pta_result(result: &PtaResult) -> Self { ... }

    /// Export to a production PointsToMap.
    pub fn to_points_to_map(&self, factory: &LocationFactory) -> PointsToMap { ... }
}
```

**Step 2: Write tests**

- Round-trip: PtaResult → IncrementalPtaState → PointsToMap matches original

---

### Task 3.3: Incremental CG refinement

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs` (or create `crates/saf-analysis/src/cg_refinement/incremental.rs`)

**Step 1: Add `refine_incremental()`**

```rust
/// Incrementally update call graph after PTA changes.
///
/// For each indirect call site, re-resolve targets using updated PTS.
/// If any targets changed, update CG edges and return the set of
/// newly reachable functions (whose constraints need extraction).
pub fn refine_incremental(
    call_graph: &mut CallGraph,
    module: &AirModule,
    pta_state: &IncrementalPtaState,
    previous_indirect_targets: &BTreeMap<ValueId, BTreeSet<FunctionId>>,
) -> CgRefinementDiff {
    // ...
}

pub struct CgRefinementDiff {
    pub added_edges: Vec<(FunctionId, FunctionId)>,
    pub removed_edges: Vec<(FunctionId, FunctionId)>,
    pub newly_reachable: BTreeSet<FunctionId>,
}
```

**Step 2: Write tests**

- Indirect call site gains new target → edge added, function marked reachable
- Indirect call site loses target → edge removed (lazy — kept but marked stale)

---

### Task 3.4: Selective ValueFlow rebuild

**Files:**
- Modify: `crates/saf-analysis/src/valueflow/builder.rs`

**Step 1: Add `rebuild_affected()`**

```rust
/// Selectively rebuild ValueFlow subgraph for affected functions.
///
/// Tears out nodes/edges belonging to `affected_functions` and rebuilds
/// only those subgraphs. Stable functions retain their VF nodes.
pub fn rebuild_affected(
    existing: &mut ValueFlowGraph,
    affected_functions: &BTreeSet<FunctionId>,
    module: &AirModule,
    defuse: &DefUseGraph,
    callgraph: &CallGraph,
    pta: Option<&PtaResult>,
    config: &ValueFlowConfig,
) { ... }
```

**Step 2: Implementation**

1. Collect all `NodeId`s belonging to affected functions (scan instructions)
2. Remove those nodes and their edges from `existing`
3. Build VF subgraph for affected functions only
4. Merge new subgraph into `existing`

**Step 3: Write tests**

- Rebuild one function → rest of graph unchanged
- Rebuilt function's VF matches full-rebuild VF for that function

---

### Task 3.5: Wire incremental solving into pipeline

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs`

**Step 1: Update `run_pipeline_incremental()`**

Replace the full PTA solve with:
```rust
if let Some(prev) = &session.previous_constraints {
    let diff = prev.diff(&current_constraints);
    if diff.is_empty() {
        // No constraint changes — reuse previous results entirely
        return previous_result;
    }
    // Incremental PTA
    let pta_result = incremental_solve(session, &diff);
    // Incremental CG refinement
    let cg_diff = refine_incremental(&mut call_graph, module, &pta_state, &prev_targets);
    if !cg_diff.newly_reachable.is_empty() {
        // Extract constraints for newly reachable functions, loop
    }
    // Selective VF rebuild
    let affected = compute_affected_functions(&diff, &cg_diff, &pta_changes);
    rebuild_affected(&mut valueflow, &affected, ...);
} else {
    // First run — full analysis
}
```

**Step 2: Integration tests**

- Two-module program → modify one module → incremental pipeline → results match full re-analysis
- Add new module → incremental detects addition → results correct
- Remove module → incremental detects removal → results correct (over-approximate OK)

**Validation gate:** For any edit on the two-module fixture, incremental results are a superset of full re-analysis results (sound over-approximation due to lazy removal).

**Verification:** `make test` in Docker

---

### Task 3.6: `InvalidationController`

**Files:**
- Create: `crates/saf-analysis/src/invalidation.rs`
- Modify: `crates/saf-analysis/src/lib.rs`

**Step 1: Define the invalidation protocol**

```rust
//! Cascading invalidation with early-termination gates.

/// Products that can be invalidated.
pub enum ProductId {
    Module(ModuleId),
    ModuleConstraints(ModuleId),
    Pta,
    CallGraph,
    DefUse(FunctionId),
    ValueFlow,
    Svfg,
    CheckerResults,
    Summary(FunctionId),
}

/// A recompute action produced by the invalidation controller.
pub enum RecomputeStep {
    ReingestModule(ModuleId),
    ReextractConstraints(ModuleId),
    IncrementalPta { added: ConstraintSet, removed: ConstraintSet },
    RebuildCallGraph,
    RecomputeSummary(FunctionId),
    RebuildValueFlow(BTreeSet<FunctionId>),
    InvalidateSvfg,
    RerunCheckers(BTreeSet<FunctionId>),
}

pub struct InvalidationController {
    versions: BTreeMap<ProductId, u64>,
    dependency_graph: BTreeMap<ProductId, BTreeSet<ProductId>>,
}

impl InvalidationController {
    /// Given a set of changed modules, compute the minimum set of
    /// recompute steps needed, with early termination when products
    /// haven't actually changed.
    pub fn plan_recompute(
        &self,
        trigger: InvalidationTrigger,
    ) -> Vec<RecomputeStep> { ... }
}
```

**Step 2: Wire into `run_pipeline_incremental()`**

Replace the ad-hoc logic in Task 3.5 with `InvalidationController::plan_recompute()`.

**Step 3: Write tests**

- Module changed but constraints identical → early termination after constraint check
- Constraints changed but PTS unchanged → early termination after PTA
- Full cascade: module → constraints → PTA → CG → VF → checkers

---

## Phase 4: Summary Infrastructure

**Objective:** Introduce `FunctionSummary` as the universal representation for function behavior. Unify YAML specs and analysis-computed summaries. Build `SummaryRegistry` for lookup.

### Task 4.1: `FunctionSummary` type + `AccessPath`

**Files:**
- Create: `crates/saf-core/src/summary.rs`
- Modify: `crates/saf-core/src/lib.rs`

**Step 1: Define core types**

```rust
//! Function summaries for compositional analysis.

/// Describes a memory access location relative to function parameters/globals.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AccessPath {
    Param(u32),
    Global(ValueId),
    Deref(Box<AccessPath>),
    Field(Box<AccessPath>, u32),
    Return,
}

/// A compact description of a function's pointer/memory behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSummary {
    pub function_id: FunctionId,
    pub version: u64,
    pub return_effects: Vec<ReturnEffect>,
    pub memory_effects: Vec<MemoryEffect>,
    pub allocation_effects: Vec<AllocationEffect>,
    pub callees: BTreeSet<CalleeRef>,
    pub role: Option<Role>,
    pub pure: bool,
    pub noreturn: bool,
    pub param_nullness: BTreeMap<u32, Nullness>,
    pub return_nullness: Option<Nullness>,
    pub taint_propagation: Vec<TaintPropagation>,
    pub return_bound: Option<ComputedBound>,
    pub param_freed: BTreeMap<u32, bool>,
    pub param_dereferenced: BTreeMap<u32, bool>,
    pub source: SummarySource,
    pub precision: SummaryPrecision,
}

// Effect sub-types, CalleeRef, Nullness, etc.
```

**Step 2: Implement `AccessPath` parsing and depth limiting**

```rust
impl AccessPath {
    /// Parse from string: "param.0", "param.0->deref", "param.0->field(8)", "return"
    pub fn parse(s: &str) -> Result<Self, ParseError> { ... }

    /// Depth of this path (number of Deref/Field steps).
    pub fn depth(&self) -> usize { ... }

    /// Truncate to maximum depth k.
    pub fn truncate(&self, k: u32) -> Self { ... }
}
```

**Step 3: Write tests**

- Parse round-trip for all access path forms
- Depth limiting at k=2, k=3
- Serialize/deserialize `FunctionSummary`

---

### Task 4.2: Spec → Summary conversion (lossless)

**Files:**
- Modify: `crates/saf-core/src/summary.rs`

**Step 1: Implement `FunctionSummary::from_spec()`**

```rust
impl FunctionSummary {
    /// Convert a `FunctionSpec` to a `FunctionSummary`.
    ///
    /// Lossless: every spec field maps to a summary field.
    pub fn from_spec(spec: &FunctionSpec, function_id: FunctionId) -> Self { ... }
}
```

Map spec fields: `role` → `role`, `pure` → `pure`, `noreturn` → `noreturn`, etc.

**Step 2: Implement `FunctionSummary::to_spec()` (lossy)**

```rust
impl FunctionSummary {
    /// Convert to a `FunctionSpec` (simple YAML).
    ///
    /// Lossy: access paths collapsed to parameter-level reads/modifies.
    pub fn to_simple_spec(&self) -> FunctionSpec { ... }

    /// Convert to extended YAML format (lossless).
    pub fn to_extended_yaml(&self) -> String { ... }
}
```

**Step 3: Write tests**

- Spec → Summary → simple spec round-trip preserves key fields
- Extended YAML → parse → compare

---

### Task 4.3: `SummaryRegistry`

**Files:**
- Create: `crates/saf-core/src/summary_registry.rs` (or extend `summary.rs`)

**Step 1: Define the registry**

```rust
/// Unified lookup for function summaries from all sources.
pub struct SummaryRegistry {
    /// User-edited YAML specs (highest priority).
    user_specs: BTreeMap<FunctionId, FunctionSummary>,
    /// Analysis-computed summaries.
    computed: BTreeMap<FunctionId, FunctionSummary>,
    /// Default shipped specs.
    defaults: BTreeMap<FunctionId, FunctionSummary>,
}

impl SummaryRegistry {
    /// Look up summary with priority: user > computed > default.
    pub fn get(&self, id: &FunctionId) -> Option<&FunctionSummary> { ... }

    /// Insert an analysis-computed summary.
    pub fn insert_computed(&mut self, summary: FunctionSummary) { ... }

    /// Load YAML specs and convert to summaries.
    pub fn load_specs(&mut self, registry: &SpecRegistry) { ... }
}
```

**Step 2: Write tests**

- Priority ordering: user spec overrides computed overrides default
- Insert computed → get returns it
- Load specs → get returns converted summaries

---

### Task 4.4: Summary-based constraint instantiation

**Files:**
- Modify: `crates/saf-analysis/src/pta/extract.rs`

**Step 1: Add `instantiate_summary_at_callsite()`**

```rust
/// Generate PTA constraints from a function summary at a specific call site.
///
/// Replaces `extract_spec_constraints()` — works with any summary regardless
/// of whether it came from YAML specs, analysis computation, or derived overlay.
pub fn instantiate_summary_at_callsite(
    summary: &FunctionSummary,
    call_site: &Instruction,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) { ... }
```

Map access paths to concrete `ValueId`s using call-site arguments.

**Step 2: Write tests**

- Simple summary (returns param.0) → generates Copy constraint
- Allocation summary → generates Addr constraint
- Field access summary → generates GEP constraint

---

### Task 4.5: Disk persistence for summaries

**Files:**
- Modify: `crates/saf-core/src/summary.rs`

**Step 1: Save/load to cache directory**

```rust
impl FunctionSummary {
    /// Save to `{cache_dir}/summaries/{function_id_hex}.json`.
    pub fn save(&self, cache_dir: &Path) -> Result<(), std::io::Error> { ... }

    /// Load from `{cache_dir}/summaries/{function_id_hex}.json`.
    pub fn load(cache_dir: &Path, function_id: &FunctionId) -> Result<Option<Self>, std::io::Error> { ... }
}
```

**Step 2: Write tests**

- Save → load round-trip
- Load missing returns None

---

## Phase 5: Compositional Analysis

**Objective:** Bottom-up summary generation. Analyze modules independently and in parallel. Summary cascade with early termination.

### Task 5.1: Bottom-up summary generation

**Files:**
- Create: `crates/saf-analysis/src/summary_gen.rs`
- Modify: `crates/saf-analysis/src/lib.rs`

**Step 1: Implement bottom-up traversal**

```rust
/// Generate function summaries bottom-up through the call graph.
///
/// 1. Topological sort of call graph (reverse post-order)
/// 2. Analyze leaf functions first → produce summaries
/// 3. For each caller: instantiate callee summaries → analyze → produce summary
/// 4. SCCs: iterate to fixed-point with widening
pub fn generate_summaries(
    module: &AirModule,
    call_graph: &CallGraph,
    factory: &mut LocationFactory,
    config: &SummaryGenConfig,
) -> SummaryRegistry { ... }

pub struct SummaryGenConfig {
    pub depth_k: u32,           // access path depth limit (default 3)
    pub scc_max_iters: u32,     // max iterations for SCC fixpoint (default 5)
    pub precision: SummaryPrecision,
}
```

**Step 2: Write tests with small call graphs**

- Linear chain: `main → helper → leaf` — summaries propagate upward
- SCC: `a → b → a` — converges within iterations
- Leaf with allocation → caller sees allocation through summary

---

### Task 5.2: Per-module parallel analysis

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs`

**Step 1: Add `analyze_modules_parallel()`**

```rust
/// Analyze modules independently in parallel.
///
/// Each module gets its own PTA context, produces its own summaries.
/// Cross-module calls resolved via summary instantiation.
pub fn analyze_modules_parallel(
    program: &AirProgram,
    summaries: &SummaryRegistry,
    config: &PipelineConfig,
    parallelism: usize,
) -> Vec<ModuleAnalysisResult> { ... }
```

Use `rayon` for parallelism (already a dependency).

**Step 2: Write tests**

- Two independent modules → parallel analysis → results match sequential
- Module with cross-module call → summary instantiation produces correct constraints

---

### Task 5.3: Summary cascade with early termination

**Files:**
- Modify: `crates/saf-analysis/src/summary_gen.rs`

**Step 1: Implement cascade logic**

```rust
/// Incrementally update summaries after a function changed.
///
/// Re-analyze the changed function → compare new summary with old.
/// If summary changed → cascade to callers. If unchanged → STOP.
pub fn cascade_summary_update(
    changed_function: FunctionId,
    module: &AirModule,
    call_graph: &CallGraph,
    registry: &mut SummaryRegistry,
    factory: &mut LocationFactory,
    config: &SummaryGenConfig,
) -> CascadeResult {
    // ...
}

pub struct CascadeResult {
    pub functions_reanalyzed: usize,
    pub cascades_stopped_early: usize,
    pub summary_changes: BTreeMap<FunctionId, SummaryChange>,
}

pub enum SummaryChange {
    Unchanged,
    Modified,
    Added,
    Removed,
}
```

**Step 2: Write tests**

- Internal change (no summary impact) → cascade stops at function
- Interface change (summary changes) → cascade propagates to callers
- Deep chain: change at leaf → early termination at first unchanged summary

---

### Task 5.4: Two precision modes

**Files:**
- Modify: `crates/saf-analysis/src/summary_gen.rs`

**Step 1: Implement Sound vs BestEffort**

| Component | Sound | BestEffort |
|-----------|-------|------------|
| Unknown callees | Top summary (may alias anything) | Identity (no-op) |
| Access path overflow | Wildcard | Truncate |
| SCC convergence | Widen after N iters | Stop after N iters |

```rust
impl SummaryGenConfig {
    pub fn sound() -> Self { ... }
    pub fn best_effort() -> Self { ... }
}
```

**Step 2: Write tests**

- Unknown callee in sound mode → conservative summary
- Unknown callee in best-effort mode → no-op
- Deep access path → sound: wildcard, best-effort: truncated

---

## Phase 6: Integration & Polish

**Objective:** Wire everything into CLI, Python SDK, add E2E tests on real projects, documentation.

### Task 6.1: CLI `--incremental` flag

**Files:**
- Modify: `crates/saf-cli/src/main.rs` (or relevant CLI module)

**Step 1: Add CLI arguments**

```
saf analyze --incremental src/*.ll
saf analyze --incremental --mode best-effort src/*.ll
saf analyze --incremental --export-summaries lib.yaml src/*.ll
saf analyze --incremental --clean src/*.ll
saf analyze --incremental --plan src/*.ll
```

**Step 2: Wire to `run_pipeline_incremental()`**

---

### Task 6.2: Python SDK `AnalysisSession`

**Files:**
- Modify: `crates/saf-python/src/project.rs`

**Step 1: Add `AnalysisSession` Python class**

```python
session = saf.AnalysisSession(cache_dir=".saf-cache", mode="sound")
project = session.analyze(["src/main.ll", "src/parser.ll"])
diff = session.last_diff()
session.export_summaries("summaries.yaml")
```

---

### Task 6.3: `--plan` dry run

**Files:**
- Modify: CLI + `InvalidationController`

**Step 1: Implement plan mode**

`--plan` computes `ManifestDiff` + `ConstraintDiff` + `InvalidationController::plan_recompute()` and prints the plan without executing it:

```
$ saf analyze --incremental --plan src/*.ll
Incremental analysis plan:
  3 modules unchanged (cached)
  1 module changed: src/parser.ll
  Actions:
    1. Re-extract constraints for src/parser.ll
    2. Incremental PTA update (+12 constraints, -8 constraints)
    3. Rebuild ValueFlow for 3 affected functions
    4. Re-run checkers on 3 functions
  Estimated savings: ~75% vs full re-analysis
```

---

### Task 6.4: E2E test suite with real projects

**Files:**
- Create: `tests/incremental_e2e/` directory

**Step 1: Lua smoke test**

Compile lua source files to `.ll`, run full analysis, modify one file, run incremental, verify results match full re-analysis.

**Step 2: Git-history-driven test runner**

```rust
/// Replay real commits and verify incremental correctness.
struct IncrementalTestRunner {
    project_dir: PathBuf,
    base_commit: String,
    steps: Vec<EditStep>,
}
```

This is a stretch goal — implement if time allows, otherwise defer to a follow-up plan.

---

### Task 6.5: `--export-summaries` for library authors

**Files:**
- Modify: CLI + `SummaryRegistry`

Export analysis-computed summaries as extended YAML:

```bash
saf analyze --incremental --export-summaries libfoo.yaml libfoo/*.ll
```

Produces a `.yaml` file that can be shipped alongside a library and loaded by downstream consumers as specs.

---

### Task 6.6: Documentation

- Update `docs/book/` with incremental analysis guide
- Add example workflow to tutorials
- Document `IncrementalConfig` fields and defaults

---

## Phase Dependencies

```
Phase 2 (constraint cache)
  └── Task 2.1-2.4: Core types (no analysis deps)
  └── Task 2.5: AnalysisSession
  └── Task 2.6: Pipeline wiring (depends on 2.1-2.5)
  └── Task 2.7: Manifest extension

Phase 3 (incremental PTA) — depends on Phase 2
  └── Task 3.1-3.2: Incremental solver activation
  └── Task 3.3: CG refinement (depends on 3.1)
  └── Task 3.4: Selective VF (depends on 3.1)
  └── Task 3.5: Pipeline wiring (depends on 3.1-3.4)
  └── Task 3.6: InvalidationController (can start during 3.3-3.4)

Phase 4 (summaries) — can start during Phase 3
  └── Task 4.1-4.2: Core types (saf-core only, no analysis deps)
  └── Task 4.3: Registry (depends on 4.1-4.2)
  └── Task 4.4: Constraint instantiation (depends on 4.1)
  └── Task 4.5: Disk persistence (depends on 4.1)

Phase 5 (compositional) — depends on Phase 4
  └── Task 5.1: Bottom-up generation (depends on 4.1-4.4)
  └── Task 5.2: Parallel analysis (depends on 5.1)
  └── Task 5.3: Cascade (depends on 5.1)
  └── Task 5.4: Precision modes (depends on 5.1)

Phase 6 (integration) — depends on Phase 3 + Phase 5
  └── Task 6.1-6.2: CLI/SDK (depends on Phase 3 pipeline)
  └── Task 6.3: Plan mode (depends on Phase 3 InvalidationController)
  └── Task 6.4: E2E tests (depends on Phase 3 pipeline)
  └── Task 6.5-6.6: Export/docs (depends on Phase 4-5)
```

## Validation Gates

| Gate | Criterion | Phase |
|------|-----------|-------|
| V1 | Cached constraints produce identical PTA results vs fresh extraction | Phase 2 |
| V2 | Incremental PTA results are superset of full PTA (sound over-approx) | Phase 3 |
| V3 | Selective VF rebuild matches full VF rebuild for affected functions | Phase 3 |
| V4 | Summary-based constraints match spec-based constraints for known specs | Phase 4 |
| V5 | Summary cascade terminates (no infinite loops on SCCs) | Phase 5 |
| V6 | E2E: incremental on real project matches full re-analysis | Phase 6 |

## Test Count Estimates

| Phase | Unit | Integration | E2E | Total |
|-------|------|-------------|-----|-------|
| 2 | ~15 | ~5 | 1 | ~21 |
| 3 | ~20 | ~8 | 2 | ~30 |
| 4 | ~15 | ~5 | 1 | ~21 |
| 5 | ~12 | ~5 | 1 | ~18 |
| 6 | ~5 | ~3 | ~5 | ~13 |
| **Total** | **~67** | **~26** | **~10** | **~103** |

## Research Checkpoints

**Checkpoint A (after Phase 3):** Measure speedup on multi-module fixture. If constraint caching alone provides 50%+ speedup on unchanged-module scenarios, Phase 4-5 may not be needed for the common CI/CD case.

**Checkpoint B (after Phase 5):** Measure precision loss from compositional analysis. If summaries cause >5% additional unsound cases on PTABen, tune depth k and widening. Document findings.
