# Plan 121: Source Line Propagation Through Analysis Pipeline

## Problem

Source line information is only available in the playground via a TypeScript-side hack that correlates Compiler Explorer debug metadata with AIR instruction positions. Python SDK and CLI users have no access to source line numbers. The `saf.source_line()` function only works in the browser.

## Design

### Approach: A+B Combined

Two ingestion paths feed spans into AIR instructions, then a shared pipeline propagates them to PropertyGraph nodes.

### Data Flow

```
Path A (CLI/Python):
  .ll/.bc → LLVM Frontend (inkwell 0.8)
    → extract_span() uses inst.get_debug_location() → DILocation
    → llvm-sys FFI for file info (LLVMDIScopeGetFile, LLVMDIFileGetFilename, LLVMDIFileGetDirectory)
    → AIR instructions get Span { file_id, line_start, col_start, line_end, col_end }
    → AirModule.source_files populated via SourceFileTracker

Path B (Playground):
  C source → Compiler Explorer → .ll text → tree-sitter → convertToAIR(tree, instSourceLines)
    → TS converter populates instruction span fields using instSourceLines data
    → AIR JSON instructions get span: { file_id: 0, line_start, col_start: 0, ... }

Shared (both paths):
  AIR (with spans) → WASM/native analysis
    → Graph builders read instruction.span when creating PropertyGraph nodes
    → PgNode.properties gets "span": { "file": "main.c", "line_start": 12, ... }
    → Python/Pyodide saf.source_line(node_id) reads from graph node span properties
```

### Component Changes

#### 1. LLVM Frontend (`crates/saf-frontends/src/llvm/debug_info.rs`)

Implement the three stub functions (currently return None):

- **`extract_span(inst, files)`**: Call `inst.get_debug_location()` → `DILocation`. Use `get_line()`, `get_column()` for location. Use `llvm-sys` FFI (`LLVMDIScopeGetFile` → `LLVMDIFileGetFilename` + `LLVMDIFileGetDirectory`) for file info. Register in `SourceFileTracker`. Return `Span::point(file_id, line, col)`.
- **`extract_function_span(func, files)`**: Use `func.get_subprogram()` → `DISubprogram` for function declaration line.
- **`extract_function_symbol(func)`**: Lower priority, can stay `None` initially.

Best-effort, silent: return `None` when debug info is missing. No warnings.

Only new unsafe code: 3 `llvm-sys` FFI calls. Everything else uses inkwell safe APIs. No changes to `mapping.rs` (already calls these functions).

#### 2. Playground TypeScript (`packages/shared/src/analysis/cst-to-air.ts`)

- `convertToAIR(tree, instSourceLines?)`: When building instructions, look up positional key in `instSourceLines` and populate `span` field.
- `file_id: 0` (playground always compiles single file).

#### 3. Graph Builders (`crates/saf-analysis/src/`)

Add span to PropertyGraph node properties. Full span object format:
```json
"span": { "file": "main.c", "line_start": 5, "col_start": 3, "line_end": 5, "col_end": 3 }
```

Affected builders:
- **CFG** (`cfg.rs`): Block nodes get first instruction's span.
- **CallGraph** (`callgraph.rs`): Function nodes get `AirFunction.span`.
- **DefUse** (`defuse.rs`): Value/Instruction nodes get originating instruction's span.
- **ValueFlow**: Thread existing `SpanInfo` into PropertyGraph node properties during export.

Shared helper in `export.rs`:
```rust
fn span_to_properties(span: &Span, source_files: &[SourceFile]) -> BTreeMap<String, Value>
```

#### 4. App.tsx Cleanup

- Pass `instSourceLines` to `convertToAIR()`
- Remove `hexIdSourceLines` mapping, `AnalyzeReturn` type, `instSourceLines` prop plumbing
- Source lines now flow through PropertyGraph nodes

#### 5. Pyodide Bridge Cleanup

- Remove `get_inst_source_lines` from JS bridge
- Remove `_inst_source_lines` dict from Python module
- `source_line(node_id)` reads span from PropertyGraph nodes instead:
  ```python
  def source_line(node_id: str) -> int | None:
      for graph in [_result.cfg, _result.callgraph, _result.defuse, _result.valueflow]:
          for node in graph.nodes:
              if node.id == node_id:
                  span = node.properties.get("span")
                  if span:
                      return span.get("line_start")
      return None
  ```

### Testing

**LLVM Frontend**: Unit test loading `.ll` with `-g`, verify `extract_span()` returns correct lines. Negative test: `.ll` without debug info returns `None`.

**Graph Builders**: Extend existing e2e test to verify PropertyGraph nodes have span in properties.

**Differential Test (Path A vs B)**:
- Test fixture: C file compiled with `-g` to `.ll` inside Docker
- Path A: Rust binary ingests `.ll` via LLVM frontend, outputs JSON map `{ "func:block:idx": line, ... }`
- Path B: Node.js script runs tree-sitter + `convertToAIR()` + `extractSourceLines()`, outputs same format
- Comparison: Assert both maps have same keys and identical line numbers
- Runs in CI inside Docker container (Node.js available for playground build)

**Playground**: Manual test — UAF example shows source lines in findings.

### Decisions

- **Span format on PropertyGraph nodes**: Full span object (`file`, `line_start`, `col_start`, `line_end`, `col_end`)
- **Missing debug info**: Best-effort, silent (no warnings)
- **Playground migration**: Replace TS-side hack entirely, single source of truth through pipeline
- **Python API**: Keep `saf.source_line()` as convenience wrapper over span data
