# Source Line Propagation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Propagate source line information from LLVM debug metadata through AIR into PropertyGraph nodes, so Python SDK and playground users can access source locations.

**Architecture:** Two ingestion paths (LLVM frontend via inkwell + playground TS converter) populate `Span` on AIR instructions. A shared helper converts spans to PropertyGraph node properties. All four graph builders thread spans into exported nodes.

**Tech Stack:** Rust (inkwell 0.8, llvm-sys, serde_json), TypeScript (CodeMirror, Pyodide), WASM (wasm-bindgen)

---

### Task 1: Implement `extract_span()` in LLVM Frontend

**Files:**
- Modify: `crates/saf-frontends/src/llvm/debug_info.rs`

**Step 1: Implement `extract_span`**

Replace the stub at lines 70-73 with real implementation using inkwell 0.8's `get_debug_location()` API:

```rust
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
pub fn extract_span(inst: InstructionValue<'_>, files: &mut SourceFileTracker) -> Option<Span> {
    let di_loc = inst.get_debug_location()?;
    let line = di_loc.get_line();
    let col = di_loc.get_column();

    // Extract file info via llvm-sys FFI (inkwell doesn't expose DIFile accessors)
    let scope = di_loc.get_scope();
    let (filename, directory) = unsafe { get_file_from_scope(scope.as_mut_ptr()) }?;

    let file_id = files.get_or_create(&filename, &directory);
    Some(Span::point(file_id, 0, line, col))
}

/// Extract filename and directory from a DIScope via llvm-sys FFI.
///
/// # Safety
/// `scope_ref` must be a valid LLVM metadata reference for a DIScope.
unsafe fn get_file_from_scope(scope_ref: llvm_sys::prelude::LLVMMetadataRef) -> Option<(String, String)> {
    let file_ref = llvm_sys::debuginfo::LLVMDIScopeGetFile(scope_ref);
    if file_ref.is_null() {
        return None;
    }

    let mut filename_len: libc::size_t = 0;
    let filename_ptr = llvm_sys::debuginfo::LLVMDIFileGetFilename(file_ref, &mut filename_len);
    if filename_ptr.is_null() {
        return None;
    }
    let filename = std::str::from_utf8(
        std::slice::from_raw_parts(filename_ptr.cast::<u8>(), filename_len)
    ).ok()?.to_string();

    let mut dir_len: libc::size_t = 0;
    let dir_ptr = llvm_sys::debuginfo::LLVMDIFileGetDirectory(file_ref, &mut dir_len);
    let directory = if dir_ptr.is_null() {
        String::new()
    } else {
        std::str::from_utf8(
            std::slice::from_raw_parts(dir_ptr.cast::<u8>(), dir_len)
        ).ok()?.to_string()
    };

    Some((filename, directory))
}
```

Also update imports at the top of the file. Add `use saf_core::span::Span;` (remove the `#[cfg]` guard if present since it's now used). Add `use llvm_sys;`. Check if `libc` needs to be added to `saf-frontends` Cargo.toml dependencies (`libc::size_t` is alias for `usize`; alternatively use `usize` directly).

Update the stale doc comments (references to "inkwell 0.5").

**Step 2: Implement `extract_function_span`**

Replace the stub at lines 92-98:

```rust
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
pub fn extract_function_span(
    func: FunctionValue<'_>,
    files: &mut SourceFileTracker,
) -> Option<Span> {
    // Get the function's subprogram debug info
    let subprogram = func.get_subprogram()?;
    let line = subprogram.get_line();

    // Get file from the subprogram's scope
    let scope_ref = subprogram.as_debug_info_scope().as_mut_ptr();
    let (filename, directory) = unsafe { get_file_from_scope(scope_ref) }?;

    let file_id = files.get_or_create(&filename, &directory);
    Some(Span::point(file_id, 0, line, 0))
}
```

Check if `func.get_subprogram()` exists in inkwell 0.8 — if not, this can return `None` for now and be implemented later.

**Step 3: Build and test inside Docker**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-frontends 2>&1' | tail -30`
Expected: All existing tests pass. No new test yet (unit test for debug info extraction is Task 6).

**Step 4: Commit**

```bash
git add crates/saf-frontends/src/llvm/debug_info.rs
git commit -m "feat(frontends): implement extract_span via inkwell debug location API"
```

---

### Task 2: Add Span-to-Properties Helper in Export

**Files:**
- Modify: `crates/saf-analysis/src/export.rs`
- Modify: `crates/saf-core/src/span.rs` (if `SourceFile` needs a lookup helper)

**Step 1: Add the helper function**

In `crates/saf-analysis/src/export.rs`, add a public function that converts a `Span` + source file registry into a JSON property map:

```rust
use saf_core::span::{SourceFile, Span};

/// Convert a `Span` into a JSON property value suitable for `PgNode.properties`.
///
/// Returns a `serde_json::Value::Object` with keys: `file`, `line_start`, `col_start`,
/// `line_end`, `col_end`. The `file` value is looked up from `source_files` by `file_id`;
/// if not found, it's omitted.
#[must_use]
pub fn span_to_property(span: &Span, source_files: &[SourceFile]) -> serde_json::Value {
    let mut map = serde_json::Map::new();

    // Look up file path from source file registry
    if let Some(sf) = source_files.iter().find(|sf| sf.id == span.file_id) {
        map.insert("file".to_string(), serde_json::json!(sf.path));
    }

    map.insert("line_start".to_string(), serde_json::json!(span.line_start));
    map.insert("col_start".to_string(), serde_json::json!(span.col_start));
    map.insert("line_end".to_string(), serde_json::json!(span.line_end));
    map.insert("col_end".to_string(), serde_json::json!(span.col_end));

    serde_json::Value::Object(map)
}
```

Check `SourceFile` struct — it has `pub id: u128` and `pub path: String` (in `crates/saf-core/src/span.rs` lines 129-151).

**Step 2: Build**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis 2>&1' | tail -10`
Expected: Compiles without errors.

**Step 3: Commit**

```bash
git add crates/saf-analysis/src/export.rs
git commit -m "feat(analysis): add span_to_property helper for PropertyGraph export"
```

---

### Task 3: Thread Spans into Graph Builders

**Files:**
- Modify: `crates/saf-analysis/src/cfg.rs` (lines 243-289, `to_pg` method)
- Modify: `crates/saf-analysis/src/callgraph.rs` (lines 527-599, `to_pg` method)
- Modify: `crates/saf-analysis/src/defuse.rs` (lines 179-247, `to_pg` method)
- Modify: `crates/saf-analysis/src/valueflow/export.rs` (lines 416-469, `to_property_graph`)

**Design principle:** Build a `BTreeMap<u128, &Span>` lookup from instruction/value IDs to spans at the start of each `to_pg`, then look up each node's ID when building properties.

**Step 1: CFG `to_pg` — add span from first instruction in block**

`Cfg::to_pg` already receives `&AirFunction`. For each block node, find the block in `func.blocks` and use the first instruction's span:

```rust
// At the top of to_pg, build block_id → first instruction span lookup
let block_spans: BTreeMap<BlockId, &Span> = func
    .blocks
    .iter()
    .filter_map(|b| {
        let span = b.instructions.first()?.span.as_ref()?;
        Some((b.id, span))
    })
    .collect();
```

Then when building properties (around line 262-269), add:

```rust
if let Some(span) = block_spans.get(block_id) {
    properties.insert(
        "span".to_string(),
        crate::export::span_to_property(span, &[]), // no source_files in CFG context
    );
}
```

Note: CFG's `to_pg` doesn't have access to `source_files`. Two options:
- (a) Add `source_files: &[SourceFile]` parameter to `to_pg`
- (b) Pass empty `&[]` — file info will be absent but line numbers still work

Go with (a) — add `source_files` parameter. This changes the signature from `to_pg(&self, func: &AirFunction)` to `to_pg(&self, func: &AirFunction, source_files: &[SourceFile])`.

**Step 2: CallGraph `to_pg` — add span from function declaration**

`CallGraph::to_pg` already receives `&AirModule`. Build function ID → span lookup:

```rust
let func_spans: BTreeMap<FunctionId, &Span> = module
    .functions
    .iter()
    .filter_map(|f| Some((f.id, f.span.as_ref()?)))
    .collect();
```

For `CallGraphNode::Function(fid)` (line 538-547), add span:

```rust
if let Some(span) = func_spans.get(fid) {
    props.insert("span".to_string(), span_to_property(span, &module.source_files));
}
```

**Step 3: DefUse `to_pg` — add `module` parameter**

`DefUseGraph::to_pg()` currently takes no arguments. Change signature to `to_pg(&self, module: &AirModule)`.

Build ID → span lookup from all instructions:

```rust
let mut id_spans: BTreeMap<u128, &Span> = BTreeMap::new();
for func in &module.functions {
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(span) = &inst.span {
                id_spans.insert(inst.id.raw(), span);
                if let Some(dst) = inst.dst {
                    id_spans.insert(dst.raw(), span);
                }
            }
        }
    }
}
```

Then when creating PgNode for values and instructions, look up span by ID (parse hex back to u128, or use the raw value). Since `self.defs` uses `ValueId`/`InstructionId` which have `.raw()`, use that:

```rust
// For value nodes (line 188-192):
let mut properties = BTreeMap::new();
if let Some(span) = id_spans.get(&value.raw()) {
    properties.insert("span".to_string(), span_to_property(span, &module.source_files));
}

// For instruction nodes (line 197-201):
let mut properties = BTreeMap::new();
if let Some(span) = id_spans.get(&inst.raw()) {
    properties.insert("span".to_string(), span_to_property(span, &module.source_files));
}
```

**Step 4: ValueFlow `to_property_graph` — add `module` parameter**

Change signature from `pub fn to_property_graph(vfg: &super::ValueFlowGraph) -> PropertyGraph` to `pub fn to_property_graph(vfg: &super::ValueFlowGraph, module: &AirModule) -> PropertyGraph`.

Build the same ID → span lookup as DefUse. For each value/location node, look up span by the node's raw ID.

**Step 5: Update all callers of changed signatures**

Callers that need updating:
- `crates/saf-wasm/src/lib.rs`: lines 126, 131, 153, 172
- `crates/saf-python/src/graphs.rs`: lines 48, 61, 67, 68
- `crates/saf-analysis/src/lib.rs`: line 46 (re-export)
- Any tests that call `to_pg()` or `to_property_graph()`

For WASM (`saf-wasm/src/lib.rs`):
- Line 126: `cfg.to_pg(func)` → `cfg.to_pg(func, &module.source_files)`
- Line 131: `defuse.to_pg()` → `defuse.to_pg(module)`
- Line 172: `to_property_graph(&vfg)` → `to_property_graph(&vfg, module)`

For Python (`saf-python/src/graphs.rs`):
- Line 48: `self.callgraph.to_pg(&self.module)` — already passes module, no change
- Line 61: `cfg.to_pg(func)` → `cfg.to_pg(func, &self.module.source_files)`
- Line 67: `self.defuse.to_pg()` → `self.defuse.to_pg(&self.module)`
- Line 68: `to_property_graph(&self.valueflow)` → `to_property_graph(&self.valueflow, &self.module)`

**Step 6: Build all crates**

Run: `docker compose run --rm dev sh -c 'cargo check --workspace 2>&1' | tail -20`
Expected: Compiles (may have warnings about unused `source_files` when spans are None, which is fine).

**Step 7: Commit**

```bash
git add crates/saf-analysis/src/cfg.rs crates/saf-analysis/src/callgraph.rs \
        crates/saf-analysis/src/defuse.rs crates/saf-analysis/src/valueflow/export.rs \
        crates/saf-analysis/src/lib.rs crates/saf-wasm/src/lib.rs crates/saf-python/src/graphs.rs
git commit -m "feat(analysis): thread spans from AIR into PropertyGraph node properties"
```

---

### Task 4: Playground TypeScript — Populate Spans in `convertToAIR`

**Files:**
- Modify: `packages/shared/src/analysis/cst-to-air.ts`
- Modify: `playground/src/App.tsx`

**Step 1: Update `convertToAIR` to accept and apply `instSourceLines`**

In `cst-to-air.ts`, find the function signature and add an optional parameter. Find where instructions are created and set `span` when a source line is available.

The conversion walks the CST and builds instructions. Find where `AirInstruction` objects are constructed (they should have an `id` field and `op` field). After creating each instruction, if the positional key maps to a source line, set:

```typescript
inst.span = {
  file_id: 0,
  byte_start: 0,
  byte_end: 0,
  line_start: srcLine,
  col_start: 0,
  line_end: srcLine,
  col_end: 0,
};
```

The positional key is built from `functionName:blockLabel:instructionIndex`. Need to track these as instructions are created.

**Step 2: Update `App.tsx` to pass `instSourceLines` to `convertToAIR`**

In `App.tsx` around line 95, change:
```typescript
const { air, registerMap } = convertToAIR(tree);
```
to:
```typescript
const { air, registerMap } = convertToAIR(tree, instSourceLines);
```

**Step 3: Remove the `hexIdSourceLines` mapping code**

Remove the hex ID mapping code added in the previous session (lines 108-127 in current App.tsx). Remove the `hexIdSourceLines` variable, `AnalyzeReturn` type, and `instSourceLines` from the return value. Revert `handleAnalyze` to return `AnalysisResults | null`.

Remove `instSourceLines` state from App.tsx if no longer needed by other components. Keep the `instSourceLines` local variable inside `handleAnalyze` for passing to `convertToAIR`.

**Step 4: Update `AnalyzerPanel.tsx`**

Revert `AnalyzerPanelProps` to:
```typescript
interface AnalyzerPanelProps {
  onAnalyze: () => Promise<AnalysisResults | null>;
}
```

In `handleRun`, change back to using `currentResults` directly:
```typescript
const currentResults = await onAnalyze();
// ...
await setupSafBridge(pyodide, currentResults, ...);
```

Remove `AnalyzeReturn` export, remove `instSourceLines` from bridge call.

**Step 5: Build and verify**

Run: `cd playground && npx tsc --noEmit && npx vite build`
Expected: No type errors, build succeeds.

**Step 6: Commit**

```bash
git add packages/shared/src/analysis/cst-to-air.ts playground/src/App.tsx playground/src/components/AnalyzerPanel.tsx
git commit -m "feat(playground): populate AIR instruction spans from source lines in convertToAIR"
```

---

### Task 5: Update Pyodide Bridge `source_line()`

**Files:**
- Modify: `playground/src/analysis/pyodide-bridge.ts`
- Modify: `playground/src/editor/saf-completions.ts` (doc update only)

**Step 1: Remove `get_inst_source_lines` from JS bridge and Python module**

In `pyodide-bridge.ts`, remove `get_inst_source_lines` from the `registerJsModule` call and from the `setupSafBridge` signature. Remove `instSourceLines` parameter.

In the Python module string, remove `_inst_source_lines` dict and the old `source_line()` function.

**Step 2: Implement new `source_line()` that reads from graph data**

In the Python module string, replace `source_line` with:

```python
def source_line(node_id: str) -> int | None:
    """Look up the C source line number for a node ID.

    Searches all analysis graphs for a node matching the given ID and
    returns its source line from the span property, or None if not found.
    """
    if _result is None:
        return None
    for graph in [_result.cfg, _result.callgraph, _result.defuse, _result.valueflow]:
        for node in graph.nodes:
            if node.id == node_id:
                span = node.properties.get("span")
                if span and isinstance(span, dict):
                    line = span.get("line_start")
                    if line is not None:
                        return int(line)
    return None
```

**Step 3: Update `setupSafBridge` callers**

In `AnalyzerPanel.tsx`, remove `instSourceLines` from the `setupSafBridge` call (already done in Task 4).

**Step 4: Build and verify**

Run: `cd playground && npx tsc --noEmit && npx vite build`
Expected: Clean build.

**Step 5: Commit**

```bash
git add playground/src/analysis/pyodide-bridge.ts
git commit -m "feat(playground): source_line reads from PropertyGraph span instead of separate mapping"
```

---

### Task 6: LLVM Frontend Unit Tests

**Files:**
- Modify: `crates/saf-frontends/src/llvm/debug_info.rs` (add tests)
- May need: a test `.ll` fixture compiled with `-g`

**Step 1: Create a test `.ll` file with debug info**

Inside Docker, compile a minimal C program with debug info:

```bash
docker compose run --rm dev sh -c '
  echo "int add(int a, int b) { return a + b; }
int main() { return add(1, 2); }" > /tmp/test_debug.c &&
  clang -S -emit-llvm -g -O0 /tmp/test_debug.c -o /workspace/tests/fixtures/llvm/e2e/debug_info.ll'
```

**Step 2: Write integration test**

Add a test in `crates/saf-frontends/tests/` or extend an existing one that loads `debug_info.ll` via the LLVM frontend and verifies instructions have non-None spans:

```rust
#[test]
fn debug_info_spans_populated() {
    let bundle = load_ll_fixture("debug_info.ll");
    let module = &bundle.module;

    // At least one function should have a span
    let func_with_span = module.functions.iter().any(|f| f.span.is_some());
    assert!(func_with_span, "No function spans found — debug info not extracted");

    // At least one instruction should have a span
    let inst_with_span = module.functions.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.instructions.iter().any(|i| i.span.is_some())
        })
    });
    assert!(inst_with_span, "No instruction spans found — debug info not extracted");

    // Verify span has reasonable line numbers (> 0)
    for func in &module.functions {
        if let Some(span) = &func.span {
            assert!(span.line_start > 0, "Function span has line 0");
        }
    }
}

#[test]
fn no_debug_info_spans_none() {
    // Load a fixture compiled without -g (e.g., an existing fixture)
    let bundle = load_ll_fixture("simple_ptr.ll"); // or any fixture without debug info
    let module = &bundle.module;

    // All spans should be None
    for func in &module.functions {
        assert!(func.span.is_none(), "Function should have no span without debug info");
    }
}
```

**Step 3: Run tests**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-frontends -- debug_info 2>&1' | tail -20`
Expected: Both tests pass.

**Step 4: Commit**

```bash
git add tests/fixtures/llvm/e2e/debug_info.ll crates/saf-frontends/
git commit -m "test(frontends): verify debug info span extraction from LLVM IR"
```

---

### Task 7: Graph Export E2E Test

**Files:**
- Modify: an existing e2e test in `crates/saf-analysis/tests/`

**Step 1: Extend an e2e test to verify span in PropertyGraph**

Find an existing test that loads a `.ll` fixture and produces PropertyGraph output. Add assertions that when the `.ll` has debug info, the PropertyGraph nodes contain span properties.

If no existing test uses debug info, create a new test using the `debug_info.ll` fixture from Task 6:

```rust
#[test]
fn property_graph_nodes_have_spans() {
    // Load the debug-info fixture
    let bundle = load_ll_fixture("debug_info.ll");
    let module = &bundle.module;

    // Build CFG and export
    let func = module.functions.iter().find(|f| f.name == "main").unwrap();
    let cfg = Cfg::build(func);
    let pg = cfg.to_pg(func, &module.source_files);

    // At least one node should have a span property
    let has_span = pg.nodes.iter().any(|n| n.properties.contains_key("span"));
    assert!(has_span, "CFG PropertyGraph nodes missing span properties");

    // Verify span structure
    for node in &pg.nodes {
        if let Some(span_val) = node.properties.get("span") {
            let span_obj = span_val.as_object().expect("span should be an object");
            assert!(span_obj.contains_key("line_start"), "span missing line_start");
            let line = span_obj["line_start"].as_u64().unwrap();
            assert!(line > 0, "span line_start should be > 0");
        }
    }
}
```

**Step 2: Run test**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-analysis -- property_graph_nodes_have_spans 2>&1' | tail -20`
Expected: PASS.

**Step 3: Commit**

```bash
git add crates/saf-analysis/tests/
git commit -m "test(analysis): verify PropertyGraph nodes include span properties"
```

---

### Task 8: Differential Test (Path A vs Path B)

**Files:**
- Create: `tests/differential/source_lines.rs` or `tests/differential/source_lines_test.sh`
- Create: `tests/differential/extract_path_b.mjs` (Node.js script for Path B)

**Step 1: Create a Rust binary that outputs Path A source lines**

Create a small test binary or integration test that:
1. Loads `tests/fixtures/llvm/e2e/debug_info.ll` via the LLVM frontend
2. Walks the AIR and outputs a JSON map: `{ "funcName:blockLabel:instIdx": lineNumber, ... }`

```rust
// tests/differential/path_a.rs (or a test in saf-frontends)
fn extract_source_lines_path_a(bundle: &AirBundle) -> BTreeMap<String, u32> {
    let mut result = BTreeMap::new();
    for func in &bundle.module.functions {
        if func.is_declaration { continue; }
        for block in &func.blocks {
            let label = block.label.as_deref().unwrap_or("entry");
            for (idx, inst) in block.instructions.iter().enumerate() {
                if let Some(span) = &inst.span {
                    let key = format!("{}:{}:{}", func.name, label, idx);
                    result.insert(key, span.line_start);
                }
            }
        }
    }
    result
}
```

**Step 2: Create a Node.js script that outputs Path B source lines**

```javascript
// tests/differential/extract_path_b.mjs
import { readFileSync } from 'fs';
// Import the shared analysis modules (need to build them first or use compiled JS)
// This script:
// 1. Reads the .ll file
// 2. Calls extractSourceLines() from compiler-explorer.ts
// 3. Outputs the instLines as JSON
```

This is the trickiest part — the TS code lives in `packages/shared/src/analysis/compiler-explorer.ts` and uses tree-sitter. We'd need to either:
- (a) Bundle the TS into a standalone Node.js script
- (b) Write a simpler comparison that just checks the C file → LLVM IR → source lines match

Option (b) is simpler for CI: compile the C file with `-g`, run Path A (Rust), compare against known expected values from the C source.

**Step 3: Create the differential comparison**

A test script that:
1. Compiles `tests/programs/c/debug_simple.c` with `-g` to `.ll` (inside Docker)
2. Runs Path A (Rust LLVM frontend) → extracts `{ pos_key: line }` JSON
3. Runs Path B (TS compiler-explorer) → extracts `{ pos_key: line }` JSON
4. Compares both maps — asserts identical line numbers

Implementation: This could be a shell script run inside Docker that:
- Uses `cargo run --release` for a small Path A binary
- Uses `node` for a bundled Path B script
- Compares with `diff` or `python3 -c 'import json; ...'`

**Step 4: Commit**

```bash
git add tests/differential/
git commit -m "test: add differential test for source line extraction Path A vs B"
```

---

### Task 9: Browser Integration Test

**Step 1: Manual test in playground**

1. Start dev server: `cd playground && npx vite`
2. Navigate to `http://localhost:8080`
3. Select "Use-After-Free" example
4. Switch to Analyzer tab
5. Select "Detect Use-After-Free" template
6. Click Run
7. Verify the finding shows source line numbers (e.g., "Value (line 5) passed to free(), used after free at line 12, 15")
8. Check console output also shows line numbers

**Step 2: Verify other templates still work**

Run all four templates on the UAF example — verify no regressions.

**Step 3: Final production build**

Run: `cd playground && npx vite build`
Expected: Clean build, no size regression.

---

### Task 10: Run Full Test Suite and Commit

**Step 1: Format and lint**

Run: `make fmt && make lint`
Expected: Clean.

**Step 2: Run all tests**

Run: `make test`
Expected: All tests pass.

**Step 3: Final commit if any remaining changes**

```bash
git add -A
git commit -m "chore: source line propagation cleanup and final fixes"
```
