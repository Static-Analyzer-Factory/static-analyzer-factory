# Incremental Analysis for Large Programs — Design Document

**Date:** 2026-02-25
**Status:** Approved design, pending implementation planning

## 1. Motivation

SAF currently performs full whole-program analysis on every run. For small programs (bash, curl), this takes seconds to minutes. For the target scale of 1M+ LOC programs (Linux kernel, Chromium), monolithic whole-program analysis is infeasible — it would require hours of compute and tens of gigabytes of memory.

This design introduces incremental and compositional analysis to SAF, enabling:
- **Whole-program scalability** to 1M+ LOC via modular analysis with function summaries
- **CI/CD integration** where only changed code is re-analyzed
- **IDE/LSP support** (future) for near-real-time analysis updates on single-file edits

## 2. Requirements

| Dimension | Decision |
|-----------|----------|
| Scale target | 1M+ LOC (Linux kernel, Chromium) |
| Primary goal | Whole-program scalability |
| Secondary goals | CI/CD diff-analysis, IDE/LSP incremental |
| Precision | Configurable: sound with controlled imprecision OR best-effort |
| Input model | Both multi-file (.ll per TU) and single-file (auto-split) |
| Timeline | Foundation first (Layers 1-2), research exploration in parallel (Layer 3) |

## 3. Architecture Overview

Three layers, each delivering standalone value:

```
+----------------------------------------------------------------+
|                    Layer 3: Compositional                        |
|  Function summaries, modular PTA, demand-driven resolution      |
|  (Research-grade, built incrementally)                          |
+----------------------------------------------------------------+
|                    Layer 2: Incremental                          |
|  Constraint diffing, incremental PTA, selective rebuild          |
|  (CI/CD-quality re-analysis)                                    |
+----------------------------------------------------------------+
|                    Layer 1: Multi-Module Infrastructure          |
|  AirProgram linking, per-module cache, change detection          |
|  (Foundation for everything above)                              |
+----------------------------------------------------------------+
|                    Existing: Single-Module Pipeline              |
|  AirModule -> DefUse -> CG+PTA -> ValueFlow -> SVFG -> Checkers |
+----------------------------------------------------------------+
```

### Data flow (Layer 1 + Layer 2)

```
Input files (.ll/.bc/.air.json)
  |
  +-- file1.ll -> BLAKE3 fingerprint -> cache hit?  -> reuse AirModule_1
  +-- file2.ll -> BLAKE3 fingerprint -> cache miss  -> ingest -> AirModule_2 (new)
  +-- file3.ll -> BLAKE3 fingerprint -> cache hit?  -> reuse AirModule_3
  |
  v
AirProgram::link([AirModule_1, AirModule_2, AirModule_3])
  |  Resolve extern declarations -> definitions
  |  Merge global variables
  |  Build cross-module call graph edges
  |
  v
Constraint Extraction (per-module, cached)
  |  Changed modules: re-extract constraints
  |  Unchanged modules: load cached constraints
  |
  v
Constraint Diff
  |  Compare new constraint set vs previous run
  |  Produce (added_constraints, removed_constraints)
  |
  v
Incremental PTA Solve
  |  Start from previous PointsToMap
  |  Process only changed constraints
  |  Propagate through affected worklist nodes
  |
  v
Selective Rebuild
  |  Only rebuild ValueFlow/SVFG subgraphs for affected functions
  |  Re-run checkers only on affected paths
  |
  v
Results (with per-function staleness tracking)
```

### Key principle: content-addressed everything

SAF's BLAKE3 ID system is the foundation. Every entity's ID is derived from its content:
- `ModuleId` <- BLAKE3(raw bitcode bytes)
- `FunctionId` <- BLAKE3("fn", function name)
- `ValueId` <- BLAKE3("val", instruction content)

If the content hasn't changed, the ID is identical. Change detection is a set comparison of IDs — no diffing algorithm needed.

## 4. Layer 1: Multi-Module Infrastructure

### 4.1 New core types

```rust
/// A whole program composed of multiple linked modules.
pub struct AirProgram {
    pub id: ProgramId,                              // BLAKE3(sorted module fingerprints)
    pub modules: Vec<AirModule>,
    pub link_table: LinkTable,
    pub merged_view: AirModule,                     // flattened view for existing pipeline
}

/// Cross-module symbol resolution table.
pub struct LinkTable {
    pub function_resolutions: BTreeMap<FunctionId, FunctionId>,
    pub global_resolutions: BTreeMap<ValueId, ValueId>,
    pub conflicts: Vec<LinkConflict>,
}

/// Tracks what changed between two runs.
pub struct ProgramDiff {
    pub added_modules: Vec<ModuleId>,
    pub removed_modules: Vec<ModuleId>,
    pub changed_modules: Vec<ModuleId>,
    pub unchanged_modules: Vec<ModuleId>,
    pub added_functions: BTreeSet<FunctionId>,
    pub removed_functions: BTreeSet<FunctionId>,
    pub changed_functions: BTreeSet<FunctionId>,
}
```

### 4.2 Multi-file ingestion

The `Frontend` trait gains `ingest_multi()`:

```rust
pub trait Frontend {
    fn ingest(&self, inputs: &[&Path], config: &Config) -> Result<AirBundle, FrontendError>;

    fn ingest_multi(
        &self,
        inputs: &[&Path],
        config: &Config,
        cache: Option<&BundleCache>,
    ) -> Result<Vec<AirBundle>, FrontendError>;
}
```

For LLVM frontend: each file produces one `AirBundle`. Cache checked per-file via existing `input_fingerprint_bytes()`.

For single-file input (pre-linked bitcode): auto-split into per-function `AirModule`s using `source_files` metadata. If no source metadata, each function becomes its own module.

### 4.3 AIR linking

The linker resolves cross-module references:

1. For each extern declaration, search all modules for a matching definition (by name)
2. If found: record in `LinkTable.function_resolutions`
3. If not found: library call — leave as declaration
4. If multiple definitions: record in `LinkTable.conflicts`
5. Globals follow the same pattern

The `merged_view` flattens `AirProgram` into a single `AirModule` for backward compatibility. Every existing analysis pass works unchanged.

### 4.4 Per-module cache

```
.saf-cache/
  manifest.json              # ProgramId -> list of (ModuleId, fingerprint)
  modules/
    {fingerprint_hex}.air.json
  constraints/
    {module_id_hex}.constraints
  summaries/
    {function_id_hex}.summary
```

### 4.5 Single-file auto-split strategy

| Strategy | Granularity | Use case |
|----------|-------------|----------|
| `BySourceFile` | Group by `source_files` metadata | Default for C/C++ |
| `ByFunction` | Each function is its own module | Maximum incrementality |
| `Monolithic` | No splitting | Backward compatibility |
| `Auto` | `BySourceFile` if metadata present, else `ByFunction` | Default |

## 5. Layer 2: Incremental Analysis

### 5.1 Per-module constraint extraction with caching

```rust
pub struct ModuleConstraints {
    pub module_id: ModuleId,
    pub fingerprint: Vec<u8>,
    pub addr: Vec<AddrConstraint>,
    pub copy: Vec<CopyConstraint>,
    pub load: Vec<LoadConstraint>,
    pub store: Vec<StoreConstraint>,
    pub gep: Vec<GepConstraint>,
    pub function_ids: BTreeSet<FunctionId>,
}

pub struct ProgramConstraints {
    pub modules: BTreeMap<ModuleId, ModuleConstraints>,
    pub merged: ConstraintSet,
}
```

Constraint diffing operates at module granularity:

```
added_constraints   = union(changed_modules.new_constraints)
                    + union(added_modules.constraints)

removed_constraints = union(changed_modules.old_constraints)
                    + union(removed_modules.constraints)
```

### 5.2 Incremental PTA solver extension

Three extensions to the existing `incremental.rs`:

**Extension 1 — Constraint removal:**

Two strategies:

| Strategy | Cost | Precision |
|----------|------|-----------|
| Lazy over-approximate | O(1) removal, periodic full recompute | Sound over-approx between GC |
| Exact recompute | O(affected nodes * avg PTS) | Exact |

Start with lazy over-approximate (simpler, always sound).

**Extension 2 — Incremental HVN:**

Re-run HVN only when diff > 10% of total constraints. For small diffs, skip HVN.

**Extension 3 — Incremental CG refinement:**

After incremental PTA update, check if indirect call sites resolve to different targets. If yes, add/remove call graph edges, re-extract constraints for newly reachable functions, feed back into incremental PTA. Fixed-point when no new edges discovered.

### 5.3 Selective ValueFlow rebuild

```rust
pub struct ValueFlowDiff {
    pub affected_functions: BTreeSet<FunctionId>,
    pub stable_functions: BTreeSet<FunctionId>,
}
```

Affected = functions in changed modules + functions whose PTS changed + callers/callees of those. Tear out and rebuild VF subgraph for affected functions only.

### 5.4 Analysis session state

```rust
pub struct AnalysisSession {
    pub program_id: ProgramId,
    pub previous_constraints: ProgramConstraints,
    pub previous_pta_state: IncrementalPtaState,
    pub previous_call_graph: CallGraph,
    pub previous_defuse: DefUseGraph,
    pub previous_valueflow: ValueFlowGraph,
    pub function_staleness: BTreeMap<FunctionId, StalenessInfo>,
}

pub struct StalenessInfo {
    pub pta_version: u64,
    pub vf_version: u64,
    pub is_pta_stale: bool,
    pub is_vf_stale: bool,
}
```

Serialized to `.saf-cache/session/` in binary format (bincode). First run does full analysis and saves session. Subsequent runs use incremental path.

## 6. Layer 3: Compositional Summaries

### 6.1 The core problem

Andersen-style PTA is whole-program: a `store` in function F can affect a `load` in function G. To analyze F in isolation, you need summaries — compact descriptions of pointer behavior independent of calling context.

### 6.2 Summary representation

```rust
pub struct FunctionSummary {
    pub function_id: FunctionId,
    pub version: u64,

    // Pointer effects
    pub return_effects: Vec<ReturnEffect>,
    pub memory_effects: Vec<MemoryEffect>,
    pub allocation_effects: Vec<AllocationEffect>,
    pub callees: BTreeSet<CalleeRef>,

    // Carried from FunctionSpec (checker/absint)
    pub role: Option<Role>,
    pub pure: bool,
    pub noreturn: bool,
    pub param_nullness: BTreeMap<u32, Nullness>,
    pub return_nullness: Option<Nullness>,
    pub taint_propagation: Vec<TaintPropagation>,

    // Carried from DerivedSpec
    pub return_bound: Option<ComputedBound>,
    pub param_freed: BTreeMap<u32, bool>,
    pub param_dereferenced: BTreeMap<u32, bool>,

    // Provenance
    pub source: SummarySource,
    pub precision: SummaryPrecision,
}

pub enum AccessPath {
    Param(u32),
    Global(ValueId),
    Deref(Box<AccessPath>),
    Field(Box<AccessPath>, u32),
    Return,
}

pub enum SummarySource {
    YamlSpec,
    AnalysisComputed,
    DerivedOverlay,
    Merged,
}

pub enum SummaryPrecision {
    Exact,
    OverApproximate,
    UnderApproximate,
}
```

Access paths are depth-limited to `k` (default 3). Paths deeper than `k` collapse to a wildcard (over-approximation in sound mode, truncation in best-effort mode).

### 6.3 Spec-summary unification

`FunctionSummary` is the universal representation. Three producers, one consumer:

1. **YAML specs** -> converted to `FunctionSummary` at load time (lossless)
2. **Analysis-computed** -> generated bottom-up by Layer 3
3. **DerivedSpec overlay** -> folded from absint/checker analysis

Consumer: `instantiate_summary_at_callsite()` replaces `extract_spec_constraints()`, generating PTA constraints from any summary regardless of source.

### 6.4 Bidirectional spec-summary conversion

**Spec -> Summary:** Lossless. Every `FunctionSpec` field maps to a `FunctionSummary` field.

**Summary -> Spec (extended YAML):** Lossless. Access paths serialized as:

```yaml
- name: my_linked_list_append
  effects:
    returns:
      - source: "param.0"
        kind: direct
    memory:
      - target: "param.0->field(8)"
        source: "param.1"
        kind: store
    allocations: []
  precision: exact
  source: analysis_computed
```

**Summary -> Spec (simple YAML):** Lossy but sound. Field paths collapsed to parameter-level `modifies`/`reads`.

Access path syntax: `param.N`, `param.N->deref`, `param.N->field(16)`, `param.N->deref->field(8)`, `global.@name`, `return`, `null`.

**Priority** (highest to lowest):
1. User-edited YAML spec (explicit override)
2. Analysis-computed summary
3. Shipped default YAML specs

### 6.5 Bottom-up summary generation

1. Build initial call graph (CHA + direct calls)
2. Analyze leaf functions -> produce summaries
3. For each caller: instantiate callee summaries, analyze caller -> produce summary
4. Repeat up call graph
5. Recursive SCCs: iterate to fixed-point with widening

### 6.6 Two precision modes

| Component | Sound mode | Best-effort mode |
|-----------|-----------|-----------------|
| Unknown callees | Top summary (may alias anything) | Identity (does nothing) |
| Library functions without specs | Conservative (may modify all reachable) | Ignored (no-ops) |
| Access path depth overflow | Collapse to wildcard | Truncate |
| Recursive SCC | Widen after N iterations | Stop after N iterations |

### 6.7 Modular analysis architecture

Each module analyzed independently and in parallel. Only summaries + currently-analyzed module in memory. For a 1M LOC program: ~10MB summaries + ~50MB current module = ~60MB working set.

Incremental summary update: function changed -> re-analyze -> compare summary -> cascade to callers only if summary changed. Empirically 80-90% of code changes don't change summaries.

## 7. Invalidation Protocol

### 7.1 Dependency graph

```
Source file changed
  -> AirModule (re-ingested)
    -> ModuleConstraints (re-extracted)
      -> ProgramConstraints.merged (re-merged)
        -> IncrementalPtaState (incremental update)
          -> CallGraph (edges may change)
            -> ICFG (rebuilt)
          -> PtaResult (rebuilt)
            -> ValueFlowGraph (selectively rebuilt)
              -> Svfg (invalidated, rebuilt lazily)
                -> Checker results (re-run on affected paths)
    -> DefUseGraph (re-built for changed functions)
    -> FunctionSummary (re-computed for changed functions)
      -> Callers' summaries (if summary changed)
    -> ModuleIndex lookup tables
```

### 7.2 Invalidation events and rules

| Trigger | Invalidated | Action |
|---------|------------|--------|
| `ModuleChanged(M)` | M's AIR, constraints, summaries | Re-ingest, re-extract, recompute |
| `ModuleAdded(M)` | Program constraints grow | Ingest, extract, incremental PTA with additions |
| `ModuleRemoved(M)` | M's constraints, summaries | Remove, incremental PTA with removals |
| `SpecChanged` | Spec-derived summaries, constraints | Reconvert, diff, incremental PTA |
| `ConfigChanged` | Everything | Full re-analysis (no safe incremental path) |

### 7.3 Cascading invalidation with early termination

Invalidation stops propagating when a product doesn't actually change:

- Module changed but constraints identical (local refactoring) -> STOP
- Constraints changed but PTS unchanged (redundant constraint) -> STOP
- PTS changed but summary unchanged (internal pointer change) -> STOP for callers

Each level is a change-detection gate. Most edits don't propagate past the first or second gate.

### 7.4 Version tracking

```rust
pub struct CacheEntry<T> {
    pub data: T,
    pub version: u64,
    pub input_versions: InputVersions,
}

pub struct InputVersions {
    pub module_version: u64,
    pub constraints_version: u64,
    pub pta_version: u64,
    pub spec_version: u64,
    pub config_hash: u64,
}
```

Staleness check is O(1) — compare version numbers.

### 7.5 Invalidation controller

```rust
pub struct InvalidationController {
    versions: VersionTable,
    dependency_graph: BTreeMap<ProductId, BTreeSet<ProductId>>,
    work_queue: VecDeque<ProductId>,
}

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
```

## 8. Configuration

```rust
pub struct IncrementalConfig {
    pub enabled: bool,                              // default: false
    pub cache_dir: PathBuf,                         // default: .saf-cache/
    pub split_strategy: SplitStrategy,              // default: Auto
    pub precision_mode: PrecisionMode,              // default: Sound
    pub summary_depth_k: u32,                       // default: 3
    pub hvn_rerun_threshold: f64,                   // default: 0.10
    pub removal_strategy: RemovalStrategy,          // default: LazyOverApproximate
    pub max_cascade_depth: u32,                     // default: 10
    pub export_summaries: Option<PathBuf>,          // default: None
    pub module_parallelism: usize,                  // default: num_cpus
}
```

When `enabled = false`, the system behaves exactly as today. Zero risk to existing users.

### CLI interface

```bash
saf analyze --incremental src/*.ll                          # incremental, sound mode
saf analyze --incremental --mode best-effort src/*.ll       # best-effort mode
saf analyze --incremental --export-summaries lib.yaml src/*.ll
saf analyze --incremental --clean src/*.ll                  # force full re-analysis
saf analyze --incremental --plan src/*.ll                   # dry run
```

### Python SDK interface

```python
session = saf.AnalysisSession(cache_dir=".saf-cache", mode="sound")
project = session.analyze(["src/main.ll", "src/parser.ll"])

# After edits:
project = session.analyze(["src/main.ll", "src/parser.ll"])
diff = session.last_diff()
session.export_summaries("summaries.yaml")
```

## 9. Testing Strategy

### 9.1 Testing principles

For any sequence of edits, incremental analysis must produce results that are:
- **Identical** to full re-analysis (exact mode)
- **Sound over-approximation** of full re-analysis (lazy removal / sound mode)
- **Never unsound** in sound mode

### 9.2 Test categories

**Layer 1 tests:** Multi-file ingestion, AIR linking, merged view equivalence, cache hit/miss, ProgramDiff correctness, single-file auto-split.

**Layer 2 tests:** Constraint caching correctness, constraint diffing, incremental PTA (add/remove), CG refinement, selective VF rebuild, session persistence round-trip.

**Layer 3 tests:** Summary generation (pure, allocator, copy, field-sensitive), depth limiting, summary instantiation, cascade termination, spec round-trip (lossless and lossy), modular vs monolithic equivalence.

**Differential testing:** For any program and edit sequence, compare incremental results vs full re-analysis. The cornerstone correctness guarantee.

### 9.3 Real-world E2E test suite

Five real open-source C projects of increasing size:

| Project | LOC | TU count | Purpose |
|---------|-----|----------|---------|
| lua | ~30K | ~35 | Smoke test |
| sqlite | ~150K | amalgamation + individual | Auto-split vs multi-file comparison |
| curl | ~100K | ~120 | Already benchmarked, indirect calls |
| nginx | ~200K | ~180 | Function pointer dispatch |
| redis | ~120K | ~100 | Deep call chains |

### 9.4 Git-history-driven edit sequences

Replay real commits from each project's git history:

```yaml
project: curl
base_commit: "curl-8_5_0"
steps:
  - commit: "abc1234"
    changed_files: ["lib/transfer.c"]
    checks:
      - type: incremental_matches_full
      - type: modules_reanalyzed
        expected: ["lib/transfer"]
      - type: modules_cached
        min_count: 115
```

Each step verifies: incremental matches full, correct modules cached/reanalyzed, constraint diffs correct, summary stability.

### 9.5 E2E test runner

Orchestrates: compile base version -> full analysis -> for each edit step: recompile changed files -> incremental analysis -> full re-analysis (oracle) -> compare.

### 9.6 CI integration

- **Every PR:** lua E2E test (~30s)
- **Weekly:** full suite on all 5 projects (~10 min)
- **Metrics:** cache hit rate, constraint diff size, PTA speedup, correctness pass/fail

## 10. Phased Implementation Roadmap

### Phase 0: Groundwork [~1-2 weeks]

Wire `BundleCache` into `LlvmFrontend` behind feature flag. Extend `Frontend::ingest()` for multiple files. Set up E2E test infrastructure. Add `IncrementalConfig` to `Config`.

### Phase 1: Multi-Module Linking [~2-3 weeks]

`AirProgram`, `LinkTable`, `merged_view()`, `ProgramDiff`. Single-file auto-split.
**Validation gate:** PTABen through merged view produces identical results.

### Phase 2: Per-Module Constraint Cache [~1-2 weeks]

`ModuleConstraints` with serialization. Refactor `extract_constraints()` per-module. Constraint diff computation. `AnalysisSession` persistence.
**Validation gate:** Cached constraints produce identical PTA results.

### Phase 3: Incremental PTA [~2-3 weeks]

Extend `incremental.rs` with removal support. Incremental CG refinement. Selective VF rebuild. `InvalidationController`.

**Research Checkpoint A:** Measure speedup on lua and curl. Decide if Layer 3 is needed.

### Phase 4: Summary Infrastructure [~3-4 weeks]

`FunctionSummary` type. Spec unification (bidirectional). `SummaryRegistry`. Summary generation for leaf functions. Library summary catalog.
**Validation gate:** Summary-based spec constraints produce identical PTA results.

### Phase 5: Compositional Analysis [~3-4 weeks]

Bottom-up summary propagation. SCC handling. Parallel per-module analysis. Summary cascade with early termination.

**Research Checkpoint B:** Measure precision loss, memory usage, speedup. Tune depth k and widening. Document findings.

### Phase 6: Integration & Polish [~2 weeks]

CLI commands, Python SDK, `--plan` dry-run, `--export-summaries`, E2E test suite in CI, documentation.

### Phase dependencies

```
Phase 0 -> Phase 1 -> Phase 2 -> Phase 3 -> Phase 4 -> Phase 5 -> Phase 6
                                    |            |                    ^
                                    |            +-- spec unification |
                                    |                can start during |
                                    |                Phase 2-3        |
                                    +-- CLI stubs can start here -----+
```

### Research questions to explore in parallel

1. Access path depth `k`: optimal value for real C programs (literature suggests k=2-3)
2. SCC handling: iterations before widening, cheap over-approximate summaries
3. Context sensitivity: per-call-site vs per-function summaries
4. Partial summaries: summarize module boundary only, keep internals as constraints
5. Ascent integration: Datalog-based summaries as Ascent rules
