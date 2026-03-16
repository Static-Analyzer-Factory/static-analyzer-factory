# Human-Readable Display Resolver — Design Document

**Date:** 2026-04-04
**Plan:** 185
**Epic:** usability

## Problem

SAF analysis results show raw BLAKE3 hex IDs (`0x00000000000000000000000000001234`) across all output surfaces: PropertyGraph exports, Python SDK findings, CLI output, SARIF reports, and the web playground. The playground's JS `AirIndex` partially mitigates this with client-side label mapping, but this logic is duplicated from the Rust backend and unavailable to CLI/SDK consumers. Users and AI agents working with SAF output must mentally map opaque hex strings to source-level concepts.

## Prerequisite: LLVM Frontend Variable Name Extraction

The LLVM frontend currently extracts function names, parameter names (`AirParam.name`), global names, and source locations (`Instruction.span`), but does **not** extract local variable names. `llvm.dbg.declare` / `#dbg_declare` intrinsics are skipped, and `DILocalVariable` metadata is ignored. `Instruction.symbol` is never populated.

The playground works around this via a separate JS path (`extractDebugVarNames()` in `compiler-explorer.ts`) that parses LLVM IR text for debug metadata. This approach is unavailable to the Rust backend.

**Phase 0 of the implementation** enhances the LLVM frontend to extract `DILocalVariable` metadata into `Instruction.symbol.display_name` for alloca instructions, closing this gap. After Phase 0, the DisplayResolver can resolve local variable names from AIR data alone.

## Solution

A shared Rust `DisplayResolver` in `saf-analysis` that maps any SAF ID (AIR entities + analysis-derived nodes) to a structured `HumanLabel` containing a short name, entity kind, source location, and contextual info. All output surfaces (PropertyGraph, Python SDK, CLI, SARIF, WASM) use this single resolver. The playground's JS `AirIndex` is replaced by WASM calls to the same resolver.

## Core Types

```rust
// saf-analysis/src/display.rs

/// What kind of entity an ID refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Function,
    Block,
    Instruction,
    Value,
    Global,
    SourceFile,
    Location,    // PTA abstract location (heap, stack, global object)
    SvfgNode,    // Value-flow graph node
    Finding,     // Checker result
    Unknown,
}

/// Resolved source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLoc {
    pub file: String,
    pub line_start: u32,
    pub col_start: u32,
    pub line_end: u32,
    pub col_end: u32,
}

/// Structured human-readable label for any SAF ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanLabel {
    pub short_name: String,
    pub kind: EntityKind,
    pub source_loc: Option<SourceLoc>,
    pub context: Option<String>,
}
```

## DisplayResolver Structure

```rust
pub struct DisplayResolver<'a> {
    // Tier 1 — AIR entities (always available)
    module: &'a AirModule,

    // Pre-built indexes for O(1) lookup
    // Reuses AirModule.function_index (BTreeMap<FunctionId, usize>) and
    // AirModule.name_index (BTreeMap<String, usize>) where possible.
    // Builds additional indexes for blocks, instructions, values, globals, files.
    index: AirLookupIndex,

    // Tier 2 — analysis-derived (optional)
    // PtaResult contains PointsToMap (BTreeMap<ValueId, BTreeSet<LocId>>)
    // but NOT constraints. LocId resolution uses the PointsToMap to identify
    // which LocIds exist, then traces back to originating instructions via
    // the Tier 1 value index (AddrConstraint.ptr → ValueId → instruction dst).
    pta_result: Option<&'a PtaResult>,
    svfg: Option<&'a Svfg>,

    // Cache: id → resolved label (lazily populated)
    // Note: RefCell makes this !Send/!Sync — single-threaded use only.
    cache: RefCell<BTreeMap<u128, HumanLabel>>,
}
```

### AirLookupIndex

Built once on construction by iterating `AirModule`. Reuses `AirModule.function_index` for function lookups. Builds additional maps for entities without pre-computed indexes. O(n) build, O(1) per lookup:

```rust
struct AirLookupIndex {
    // Reuse AirModule.function_index for functions (no duplication)
    globals: BTreeMap<u128, usize>,                        // global ValueId → index in module.globals
    global_objs: BTreeMap<u128, usize>,                    // global ObjId → index in module.globals
    blocks: BTreeMap<u128, (usize, usize)>,                // BlockId → (func_idx, block_idx)
    instructions: BTreeMap<u128, (usize, usize, usize)>,   // InstId → (func_idx, block_idx, inst_idx)
    values: BTreeMap<u128, ValueOrigin>,                    // ValueId → origin
    files: BTreeMap<u128, usize>,                          // FileId → index in module.source_files
    // Reverse map: LocId → originating ValueId (the ptr in the AddrConstraint)
    // Built from PTA result by scanning which ValueIds point to which LocIds.
    loc_to_value: BTreeMap<u128, u128>,                    // LocId → ValueId that received its address
}

enum ValueOrigin {
    Param { func_idx: usize, param_idx: usize },
    InstructionDst { func_idx: usize, block_idx: usize, inst_idx: usize },
    Global { global_idx: usize },
}
```

### LocId Resolution Strategy

`PtaResult` contains `PointsToMap` (`BTreeMap<ValueId, BTreeSet<LocId>>`) but not the original `Constraints`. To resolve a `LocId` to a human-readable name:

1. During index construction, invert the `PointsToMap`: for each `(value_id, loc_set)`, if a `LocId` appears in exactly one value's points-to set AND that value maps to an alloca/global instruction, record `loc_id → value_id` in `loc_to_value`.
2. At resolve time: `LocId` → `loc_to_value[loc_id]` → `ValueOrigin` → instruction → format as `"heap@malloc:12"` or `"stack_p"`.
3. Fallback: if the `LocId` can't be traced (e.g., field-sensitive sub-locations, merged sets), use `"loc_%a3f2"`.

Alternative (simpler): The resolver also accepts an optional `&ConstraintSet` for cases where constraints are available (e.g., in the pipeline before they're discarded). When present, `AddrConstraint.loc` → `AddrConstraint.ptr` → Tier 1 value index provides a direct mapping.

### Construction

```rust
impl<'a> DisplayResolver<'a> {
    /// Tier 1 only — works after frontend ingestion.
    pub fn from_module(module: &'a AirModule) -> Self;

    /// Full resolver with analysis results for Tier 2.
    pub fn with_analysis(
        module: &'a AirModule,
        pta_result: Option<&'a PtaResult>,
        svfg: Option<&'a Svfg>,
    ) -> Self;
}
```

### Resolution API

```rust
impl<'a> DisplayResolver<'a> {
    /// Resolve any ID to a human label.
    pub fn resolve(&self, id: u128) -> HumanLabel;

    /// Batch resolve for export efficiency.
    pub fn resolve_all(&self, ids: &[u128]) -> Vec<HumanLabel>;
}
```

Resolution order:
1. Check cache → return if hit
2. Try `AirModule.function_index` (function lookup)
3. Try global index
4. Try block index
5. Try instruction index
6. Try value index (params, instruction dsts)
7. Try source file index
8. Try `loc_to_value` map (LocId → originating value → instruction)
9. Try SVFG node lookup (`SvfgNodeId::Value(vid)` → resolve `vid` via Tier 1)
10. Return `EntityKind::Unknown` with short hex fallback

### Fallback Strategy

| Entity | Best case | Partial debug | No debug |
|---|---|---|---|
| Function | `"main"` | `"main"` | `"main"` |
| Block | `"while.cond"` | `"bb2 (5 instr)"` | `"%a3f2"` |
| Instruction | `"call @malloc"` | `"call"` | `"call"` |
| Value (param) | `"p"` | `"arg0"` | `"arg0"` |
| Value (inst dst) | `"p (alloca)"` | `"%a3f2 (alloca)"` | `"%a3f2"` |
| Global | `"stderr"` | `"stderr"` | `"stderr"` |
| Location | `"heap@malloc:12"` | `"heap@malloc"` | `"loc_%a3f2"` |
| SvfgNode | traced to Value | traced to Value | `"vfn_%a3f2"` |
| Finding | `"source → sink"` | IDs of source/sink | `"%a3f2 → %b4c1"` |
| Unknown | — | — | `"%a3f2"` |

Short hex format: strip leading zeros, prefix with `%` (e.g., `"0x000...1234"` → `"%1234"`).

**Source highlighting precision improvement:** The current playground `instLines` maps instruction IDs to the *containing block's* line range. The resolver's `source_loc` from `Instruction.span` gives the instruction's *own* line. This is more precise — clicking a node will highlight the exact source line, not the entire block.

## Integration Points

### 1. ProgramDatabase

```rust
impl ProgramDatabase {
    pub fn display_resolver(&self) -> DisplayResolver<'_>;
}
```

Passes `self.module()` (`&AirModule`), `self.pta_result()` (`Option<&PtaResult>`), and `self.get_or_build_svfg()` (`&Svfg`).

### 2. PropertyGraph Export

`to_property_graph()` gains `Option<&DisplayResolver>` parameter. When present, each `PgNode` gets additive properties:
- `display_name: String`
- `source_file: Option<String>`
- `source_line: Option<u32>`
- `source_col: Option<u32>`

Node `id` stays as hex (machine-stable). All 5 graph types (CFG, callgraph, defuse, valueflow, PTA) benefit.

### 3. Python SDK

- `PyFinding` gains `source_name`, `sink_name` fields (resolved `short_name`)
- `PyTraceStep.from_symbol`/`to_symbol` gaps filled by resolver
- New `PyDisplayResolver` class exposed to Python for ad-hoc lookups

### 4. CLI Output

- Human mode: findings formatted as `"use-after-free: variable 'p' at main.c:12 → free at main.c:18"`
- JSON mode: `display_name` and `source_loc` fields alongside hex IDs
- SARIF: `physicalLocation` fully populated from `SourceLoc`

### 5. WASM

- `saf-wasm` exports `resolve_display(id: &str) -> JsValue` returning serialized `HumanLabel`
- `resolve_display_batch(ids: Vec<String>) -> JsValue` for bulk resolution
- Replaces existing `build_pta_labels()`, `format_inst_short()`, and `format_field_path()` helper functions in `saf-wasm/src/lib.rs`

### 6. Playground (Replace JS AirIndex)

- Remove `packages/shared/src/graph/air-index.ts`
- All 5 renderers (cfg, callgraph, defuse, valueflow, pta) call WASM `resolve_display()` instead of `AirIndex` lookups
- `GraphPanel` no longer builds `AirIndex`; passes WASM resolver handle to renderers
- Source line highlighting: WASM resolver provides `source_loc` → same highlight behavior
- Also update `tutorials/src/components/InteractiveStep.tsx` which imports `buildAirIndex`

## Non-Goals

- Changing how IDs are generated or stored
- Modifying AIR serialization format
- Adding new analysis capabilities

## Estimated Size

~1800-2300 lines across:
- LLVM frontend variable name extraction (~150-200 lines): Phase 0
- `display.rs` (~500-700 lines): core resolver + types + index + tests
- Export modifications (~200 lines): PropertyGraph enrichment
- Python bindings (~200 lines): PyDisplayResolver + finding enrichment
- CLI output (~100 lines): human/JSON/SARIF formatting
- WASM bridge (~100 lines): resolve functions + removal of old label helpers
- Playground + Tutorials TypeScript (~300-400 lines): renderer rewrites, AirIndex deletion
- Tests (~200 lines): unit + integration
