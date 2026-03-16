# Plan 185: Human-Readable Display Resolver

**Epic:** usability
**Design:** `docs/plans/2026-04-04-display-resolver-design.md`

## Phase 0: LLVM Frontend — Local Variable Name Extraction

The LLVM frontend currently skips `llvm.dbg.declare` / `#dbg_declare` intrinsics and ignores `DILocalVariable` metadata, so `Instruction.symbol` is never populated with C local variable names. The playground works around this via JS-side LLVM IR text parsing (`extractDebugVarNames()` in `compiler-explorer.ts`). This phase closes the gap so the Rust backend has the same variable name information.

### Task 0.1: Extract DILocalVariable metadata
- In `crates/saf-frontends/src/llvm/debug_info.rs`, add `extract_local_variable_names(func: FunctionValue) -> BTreeMap<ValueId, String>`
- Parse `#dbg_declare(ptr %reg, !N, ...)` and old-style `@llvm.dbg.declare(metadata ptr %reg, metadata !N, ...)` intrinsics
- For each, look up `!N` to find the `DILocalVariable` name
- Map the alloca's `ValueId` (from `%reg`) to the variable name
- Use inkwell's debug metadata API (`get_debug_location`, `get_subprogram`, etc.) or FFI calls to `LLVMGetMetadataOperand` as needed

### Task 0.2: Populate Instruction.symbol for alloca instructions
- In `crates/saf-frontends/src/llvm/mapping.rs`, after building instructions, call the new extraction function
- For each alloca instruction whose `dst` ValueId appears in the variable name map, set `instruction.symbol = Some(Symbol { display_name: var_name, ... })`
- Do NOT change `IntrinsicMapping::Skip` for `llvm.dbg.*` — still skip them as instructions, but pre-pass them for metadata extraction

### Task 0.3: Unit tests
- Add a test fixture `.ll` file with debug info (`-g -O0`): simple C program with local variables
- Verify that after frontend ingestion, alloca instructions have `symbol.display_name` set to the C variable names
- Verify that parameters still get `AirParam.name` as before (no regression)

## Phase 1: Core Resolver (Tier 1 — AIR entities)

### Task 1.1: Core types and module
- Create `crates/saf-analysis/src/display.rs`
- Define `EntityKind`, `SourceLoc`, `HumanLabel` with Serialize/Deserialize
- Define `AirLookupIndex`, `ValueOrigin`
- Add `mod display` to `crates/saf-analysis/src/lib.rs`
- Re-export types from crate root

### Task 1.2: AirLookupIndex construction
- Implement `AirLookupIndex::build(module: &AirModule)` — single pass over all functions, blocks, instructions, params, globals, source files
- Reuse `AirModule.function_index` (`BTreeMap<FunctionId, usize>`) for function lookups instead of duplicating
- Build additional maps: `globals`, `global_objs`, `blocks`, `instructions`, `values`, `files`
- Populate `ValueOrigin` variants for all param, instruction dst, and global value IDs

### Task 1.3: DisplayResolver Tier 1 implementation
- Implement `DisplayResolver::from_module(module: &AirModule)` (not `from_bundle` — `ProgramDatabase` stores `Arc<AirModule>`, not `AirBundle`)
- Implement `resolve()` for AIR entity types (functions, blocks, instructions, values, globals, source files)
- Implement `short_hex()` helper for fallback formatting (strip leading zeros, prefix with `%`)
- Implement `resolve_span()` helper: `Span` + `module.source_files` → `SourceLoc`
- Cache resolved labels in `RefCell<BTreeMap<u128, HumanLabel>>`

### Task 1.4: Unit tests for Tier 1
- Test function resolution (name, span, symbol)
- Test block resolution (label fallback, instruction count context)
- Test instruction resolution (operation formatting, symbol.display_name for local vars from Phase 0)
- Test value resolution (param name, instruction dst with variable name, global name)
- Test fallback chain (missing spans, missing symbols, unknown IDs)
- Test cache behavior (resolve same ID twice → same result)

## Phase 2: Tier 2 — Analysis-Derived IDs

### Task 2.1: LocId resolution via PtaResult
- Implement `with_analysis(module, pta_result: Option<&PtaResult>, svfg: Option<&Svfg>)` constructor
- Build `loc_to_value` reverse map by inverting `PtaResult.points_to_map()`: for each `(value_id, loc_set)`, record which values "own" each LocId
- Resolution chain: `LocId` → `loc_to_value` → `ValueId` → Tier 1 value index → instruction → format
- Format: `"heap@malloc:12"` (heap alloc), `"stack_p"` (stack alloca with var name), `"global_stderr"` (global)
- Fallback for untraceable LocIds (field-sensitive sub-locations, merged sets): `"loc_%a3f2"`

### Task 2.2: SVFG node resolution
- Resolve `SvfgNodeId` by matching on enum: `SvfgNodeId::Value(vid)` → resolve `vid` via Tier 1, `SvfgNodeId::MemPhi(access_id)` → format with context
- Delegate to Tier 1 value resolution for the underlying value
- Format: context includes SVFG node kind

### Task 2.3: Finding resolution
- Resolve `FindingId` by formatting source + sink pair
- Resolve source and sink IDs through the normal pipeline

### Task 2.4: ProgramDatabase integration
- Add `display_resolver(&self) -> DisplayResolver<'_>` to `ProgramDatabase`
- Wire: `self.module()` → `&AirModule`, `self.pta_result()` → `Option<&PtaResult>`, `self.get_or_build_svfg()` → `&Svfg`

### Task 2.5: Unit tests for Tier 2
- Test LocId resolution (heap alloc site, global object, stack alloca)
- Test SvfgNodeId resolution (Value variant, MemPhi variant)
- Test FindingId resolution
- Test ProgramDatabase convenience method

## Phase 3: PropertyGraph Export Enrichment

### Task 3.1: Modify export functions
- Add `Option<&DisplayResolver>` parameter to `to_property_graph()` and all graph-specific export helpers
- When resolver is present, add `display_name`, `source_file`, `source_line`, `source_col` properties to each `PgNode`
- Preserve backward compatibility: no extra properties when resolver is `None`

### Task 3.2: Wire resolver in all export call sites
- CFG export
- Callgraph export
- Def-use export
- Valueflow export
- PTA export

### Task 3.3: Integration tests
- Export a PropertyGraph with resolver → verify enriched properties present on nodes
- Export without resolver → verify backward compatibility (no extra properties)
- Verify edge IDs (src/dst) are NOT changed — only node properties enriched

## Phase 4: Python SDK Enrichment

### Task 4.1: PyDisplayResolver wrapper
- New Python class wrapping `DisplayResolver`
- Method `resolve(hex_id: str) -> dict` returning HumanLabel as Python dict
- Method `resolve_batch(hex_ids: list[str]) -> list[dict]`

### Task 4.2: Finding enrichment
- Add `source_name`, `sink_name` to `PyFinding`
- Fill `from_symbol`/`to_symbol` gaps in `PyTraceStep` using resolver
- Resolver obtained from `ProgramDatabase` during finding construction

### Task 4.3: Python tests
- Test PyDisplayResolver.resolve() with known IDs
- Test enriched PyFinding fields
- Test PyTraceStep symbol gap-filling

## Phase 5: CLI Output Enrichment

### Task 5.1: Human output mode
- Format findings with resolved names: `"use-after-free: variable 'p' at main.c:12 → free at main.c:18"`
- Format graph summaries with resolved node names

### Task 5.2: JSON output mode
- Add `display_name` and `source_loc` fields alongside hex IDs in JSON output

### Task 5.3: SARIF enrichment
- Populate `physicalLocation.artifactLocation.uri` from `SourceLoc.file`
- Populate `physicalLocation.region` from `SourceLoc` line/col fields
- Populate `message.text` with resolved names

## Phase 6: WASM Bridge

### Task 6.1: WASM export functions
- Add `resolve_display(id: &str) -> JsValue` to `saf-wasm`
- Add `resolve_display_batch(ids: Vec<String>) -> JsValue`
- Resolver built from WASM's thread-local `ProgramDatabase`

### Task 6.2: Remove old label helpers
- Delete `build_pta_labels()`, `format_inst_short()`, `format_field_path()` from `saf-wasm/src/lib.rs`
- Replace their call sites with `DisplayResolver` usage
- Verify WASM analysis results still include human-readable labels

### Task 6.3: TypeScript types
- Add `HumanLabel` TypeScript interface in `packages/shared/src/types/`
- Add `resolveDisplay(id: string): HumanLabel` wrapper calling WASM (synchronous — matches existing WASM call pattern)

## Phase 7: Playground & Tutorials — Replace JS AirIndex

### Task 7.1: Update playground renderers to use WASM resolver
- `cfg-renderer.ts`: replace `index?.blockLabel.get()` with WASM `resolveDisplay()`
- `callgraph-renderer.ts`: replace `index?.funcName.get()` / `index?.funcLines.get()` with WASM resolver
- `defuse-renderer.ts`: replace `index?.instLabel.get()` / `index?.valueLabel.get()`
- `valueflow-renderer.ts`: replace `index?.valueLabel.get()` / `index?.funcName.get()`
- `pta-renderer.ts`: replace `index?.valueLabel.get()`
- All renderers: change `index?: AirIndex | null` parameter to WASM resolver handle

### Task 7.2: Update tutorials InteractiveStep
- `tutorials/src/components/InteractiveStep.tsx` also imports and calls `buildAirIndex()`
- Replace with WASM resolver calls (same pattern as playground renderers)

### Task 7.3: Update source line highlighting
- Source line ranges now come from `HumanLabel.source_loc` instead of `AirIndex.blockLines`/`instLines`/`funcLines`
- Update `GraphPanel` click handlers to extract `source_loc` from WASM resolver
- Note: this is a precision improvement — instruction clicks highlight the exact source line, not the entire block range

### Task 7.4: Remove JS AirIndex
- Delete `packages/shared/src/graph/air-index.ts`
- Remove `buildAirIndex()` calls from `GraphPanel.tsx` and `App.tsx`
- Remove `AirIndex` type imports from all renderers and InteractiveStep
- Clean up `_varNames`, `_blockSourceLines`, `_instSourceLines` extension handling from `App.tsx`

### Task 7.5: Playground integration testing
- Verify all 5 graph types render with human-readable labels
- Verify source highlighting still works on node click (now per-instruction precision)
- Verify fallback behavior (missing debug info shows short hex, not crash)
- Verify tutorial interactive steps still render correctly

## Phase 8: Quality & Cleanup

### Task 8.1: Cross-platform consistency check
- Run same analysis through CLI, Python SDK, and playground
- Verify identical `display_name` / `short_name` values across all three
- Document any intentional format differences (e.g., playground may show shorter labels)

### Task 8.2: Tutorial content verification
- Check `tutorials/public/content/` pre-computed graphs
- Update if they reference AirIndex or contain raw hex IDs
- Tutorials use synthetic node IDs (p, q, r) in traces — these should be unaffected

### Task 8.3: Playwright E2E tests
Existing Playwright tests are in `playground/e2e/` with 2 test suites (`differential.spec.ts`, `run-all-suppression.spec.ts`). Add a new test suite `display-resolver.spec.ts` that verifies human-readable labels across all output surfaces.

**Graph label tests** (all 15 built-in examples):
- For each example: select example → click Analyze → switch to each graph tab (CFG, Call Graph, Def-Use, Value Flow, PTA)
- Verify NO raw hex IDs (`0x00000000...`) appear in visible graph node labels
- Verify nodes show human-readable names (function names, variable names, instruction descriptions)
- Verify source highlighting works on node click (highlighted line range appears in source panel)

**Analyzer/checker finding tests** (9 checker examples: UAF, leak, double-free, null-deref, FD-leak, lock, uninit, stack-escape, resource-leak):
- For each: select example → Analyze → run query checker
- Verify findings show human-readable source/sink names (not hex IDs)
- Verify finding path trace shows source locations (`file:line:col`)

**Query result tests:**
- Run points-to queries → verify results show variable names, not hex IDs
- Run flow queries → verify trace steps show human-readable labels

**Fallback behavior test:**
- Use a minimal C program without `-g` debug info (if supported by playground)
- Verify short hex fallback (`%a3f2`) appears instead of full 32-char hex IDs
- Verify no crashes or blank labels

Run with: `cd playground && npx playwright test`

### Task 8.4: Final lint + test pass
- `make fmt && make lint`
- `make test`
- `make wasm`
- Run Playwright E2E: `cd playground && npx playwright test`
- Verify existing `differential.spec.ts` and `run-all-suppression.spec.ts` still pass (no regressions)
