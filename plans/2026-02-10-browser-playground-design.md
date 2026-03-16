# SAF Browser Playground — Design Document

**Date:** 2026-02-10
**Status:** Approved
**Scope:** Interactive browser-based playground for SAF static analysis

## Overview

A client-side web application that lets users write C/C++ or LLVM IR, run SAF's analysis pipeline entirely in the browser via WebAssembly, and visualize results (CFG, call graph, def-use, value-flow, points-to) interactively.

No server infrastructure required. All analysis runs in WASM. C/C++ compilation uses the public Compiler Explorer API.

## Motivation

- Lower the barrier to trying SAF — no Docker, no install, just open a URL
- Showcase SAF's analysis capabilities interactively
- Educational tool: users see the full pipeline from C source → LLVM IR → AIR → analysis results

## Architecture

### High-Level Data Flow

```
User writes C/C++ or LLVM IR
  │
  ├─ (if C/C++) ─→ Compiler Explorer API ─→ .ll text
  │
  ▼
tree-sitter-llvm (WASM, ~100KB) parses .ll to CST
  │
  ▼
TypeScript converts CST → AIR JSON
  │
  ▼
saf-wasm (WASM) ingests AIR JSON, runs analysis
  │
  ▼
PropertyGraph JSON exported for each graph type
  │
  ▼
Cytoscape.js renders interactive graphs
```

### Three Layers

1. **Parse layer** — tree-sitter-llvm in WASM, TypeScript glue for CST → AIR conversion
2. **Analysis layer** — `saf-wasm` crate (saf-core + saf-analysis, no Z3/LLVM), exposes `analyze(air_json) → results_json`
3. **UI layer** — React + Vite + Cytoscape.js + CodeMirror

Each layer communicates via JSON strings across the WASM boundary. No shared memory, no complex bindings.

### Why tree-sitter-llvm (not inkwell in WASM)

inkwell wraps LLVM's C API via FFI (`llvm-sys`), linking against ~100MB of native C++ libraries. It cannot compile to `wasm32`. A pure-Rust `.ll` parser would work but creates ongoing maintenance burden tracking LLVM IR format changes.

[tree-sitter-llvm](https://github.com/benwilliamgraham/tree-sitter-llvm) provides:
- First-class WASM compilation via `tree-sitter build --wasm` (~100KB output)
- Grammar that closely follows LLVM's `LLParser` — format updates handled upstream
- 100% coverage of SAF's instruction needs (all 45+ opcodes, all types, constant expressions)

The native build keeps inkwell (handles `.bc` bitcode, provides semantic resolution). The browser uses tree-sitter for `.ll` text only. Both produce the same AIR.

## The `saf-wasm` Crate

New crate at `crates/saf-wasm/` — thin WASM entry point.

```toml
# crates/saf-wasm/Cargo.toml
[package]
name = "saf-wasm"

[lib]
crate-type = ["cdylib"]

[dependencies]
saf-core = { path = "../saf-core" }
saf-analysis = { path = "../saf-analysis", default-features = false }
saf-frontends = { path = "../saf-frontends", default-features = false, features = ["air-json"] }
wasm-bindgen = "0.2"
serde_json = "1"
```

Minimal public API — one function:

```rust
#[wasm_bindgen]
pub fn analyze(air_json: &str, config_json: &str) -> String {
    // 1. Parse AIR JSON via existing air_json frontend
    // 2. Build CFG, call graph, PTA, def-use, value-flow
    // 3. Export each as PropertyGraph JSON
    // 4. Return combined results JSON
}
```

No inkwell, no Z3, no rayon. Build with `wasm-pack build crates/saf-wasm --target web`.

## Changes to Existing Crates

Minimal changes — feature-gating only, no analysis logic changes.

### saf-analysis

Z3 becomes optional (~30 lines):

```toml
[features]
default = ["z3-solver"]
z3-solver = ["dep:z3"]
```

```rust
#[cfg(feature = "z3-solver")]
pub mod z3_utils;

#[cfg(feature = "z3-solver")]
pub mod z3_index;

// Path feasibility becomes a no-op without Z3
fn check_path_feasibility(&self, ...) -> bool {
    #[cfg(feature = "z3-solver")]
    { self.z3_check(...) }
    #[cfg(not(feature = "z3-solver"))]
    { true }
}
```

### saf-frontends

No changes needed. With `default-features = false`, only the AIR JSON frontend compiles. LLVM/inkwell is already behind feature flags.

### saf-core

One optional addition for future-proofing (~20 lines):

```rust
impl SpecRegistry {
    pub fn from_yaml_strings(specs: &[(&str, &str)]) -> Result<Self, RegistryError> {
        // Filesystem-free loading — only needed if checkers added later
    }
}
```

Existing `make test` and `make lint` are unaffected (use default features).

## CST → AIR Converter

TypeScript module at `playground/src/analysis/cst-to-air.ts`. The equivalent of `mapping.rs` (~1583 lines Rust), expected ~1000-1200 lines TypeScript.

Responsibilities:
1. **Module structure** — extract `define`/`declare` functions, globals
2. **Function bodies** — iterate basic blocks, map labels to block IDs
3. **Instructions** — map each tree-sitter `instruction_*` node to an AIR operation
4. **SSA resolution** — track `%name` → ValueId within each function
5. **Constants** — extract integer/float/null/string from value nodes
6. **Types** — parse type nodes for alloca sizes and cast target bits
7. **Globals** — extract initializers, detect vtable globals

Validated by golden file comparison against native inkwell → AIR output.

## UI Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  SAF Playground                                        [★ GitHub]  │
├────────────────────────────┬────────────────────────────────────────┤
│ [C/C++] [Compiled IR ●]   │  [CFG] [Call Graph] [Def-Use]         │
│ [AIR ●] [LLVM IR]         │  [ValueFlow] [PTA]                    │
├────────────────────────────┤────────────────────────────────────────┤
│                            │                                        │
│  CodeMirror editor         │     Cytoscape.js graph view            │
│  (editable in C/C++        │     (interactive zoom/pan/select)      │
│   and LLVM IR modes)       │                                        │
│                            │     Click node → highlight source      │
│                            │     Hover → tooltip with source line   │
│                            │                                        │
├────────────────────────────┴────────────────────────────────────────┤
│ [Examples ▾]  25/200 lines                       [▶ Analyze]       │
├─────────────────────────────────────────────────────────────────────┤
│ ● Ready  3 functions · 25 blocks · 87 insns  PTA:12ms  CFG:2ms    │
└─────────────────────────────────────────────────────────────────────┘
```

### Input Tabs

| Tab | Editable? | Content |
|---|---|---|
| **C/C++** | Yes | User writes C or C++ source |
| **Compiled IR** | Read-only | `.ll` returned by Compiler Explorer |
| **AIR** | Read-only | AIR JSON produced by CST→AIR converter |
| **LLVM IR** | Yes | User writes/uploads `.ll` directly |

### Input Modes

| Mode | Flow |
|---|---|
| C/C++ | Source → Compiler Explorer API (Clang 18, `-S -emit-llvm -O0 -g`) → `.ll` → tree-sitter → AIR → analyze |
| LLVM IR | `.ll` → tree-sitter → AIR → analyze |

Line limit: **200 lines**. Enforced in editor, Analyze button disabled above limit. Keeps API calls fast and analysis results comprehensible for a demo.

### Graph Tabs

Five tabs: CFG, Call Graph, Def-Use, Value Flow, Points-To. Each renders the corresponding PropertyGraph with Cytoscape.js.

### Interactions

- Click a function in call graph → source scrolls to function, CFG switches to that function
- Click a CFG block → source highlights those instructions
- Hover a node → tooltip shows the instruction and source line
- Green/red borders = entry/exit blocks, blue = selected

### Bundled Examples

4-5 small C programs in a dropdown, demonstrating pointer aliasing, indirect calls, struct field sensitivity, data flow. Pre-compiled `.ll` cached for instant first-click experience.

## Data Flow & State Management

```
App State
─────────
inputMode:    "c" | "llvm"
sourceCode:   string              ← user edits
compiledIR:   string | null       ← from Compiler Explorer
airJSON:      object | null       ← from cst-to-air
results:      AnalysisResults     ← from saf-wasm
activeGraph:  "cfg" | "callgraph" | "defuse" | "valueflow" | "pta"
selectedNode: string | null       ← click in graph
status:       "idle" | "compiling" | "parsing" | "analyzing" | "ready" | "error"
error:        string | null
```

### Pipeline on Analyze Click

```
Step 1 (C/C++ only):  sourceCode → Compiler Explorer API → compiledIR
Step 2:               compiledIR → tree-sitter parse → CST
Step 3:               CST → cst-to-air.ts → airJSON
Step 4:               airJSON → saf-wasm.analyze() → results
Step 5:               results → Cytoscape.js renders active graph tab
```

### Web Worker

Steps 2-4 run in a Web Worker to avoid blocking the UI:

```
Main thread                    Web Worker
───────────                    ──────────
click Analyze
  → fetch Compiler Explorer
  ← compiledIR
  → postMessage(compiledIR)
                               tree-sitter → CST
                               CST → AIR JSON
                               saf-wasm.analyze(AIR)
                               ← postMessage(results)
  render graphs
```

### Timing Budget (200-line C program)

| Step | Target |
|---|---|
| Compiler Explorer | < 3s (network) |
| tree-sitter parse | < 50ms |
| CST → AIR | < 100ms |
| saf-wasm analyze | < 500ms |
| Graph render | < 200ms |
| **Total** | **< 4s** |

## Build Pipeline & Makefile Targets

```
crates/saf-wasm/  ──wasm-pack──→  playground/src/wasm/saf_wasm.{wasm,js,d.ts}
playground/       ──vite build──→  playground/dist/  (static files → GitHub Pages)
```

| Target | What it does |
|---|---|
| `make wasm` | `wasm-pack build crates/saf-wasm --target web --release` |
| `make wasm-dev` | Same without `--release` (faster iteration) |
| `make playground` | `make wasm` + `cd playground && npm run build` |
| `make playground-dev` | `make wasm-dev` + `cd playground && npm run dev` (http://localhost:5173) |
| `make playground-deploy` | `make playground` + `gh-pages -d playground/dist` |

These run on the host — no Docker needed. WASM build targets `wasm32-unknown-unknown` with no LLVM/Z3.

## Project Structure

```
crates/saf-wasm/
├── Cargo.toml
├── src/
│   └── lib.rs                       # #[wasm_bindgen] analyze()

playground/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
├── public/
│   └── tree-sitter-llvm.wasm        # prebuilt grammar WASM
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── SourcePanel.tsx           # CodeMirror (C/C++ and LLVM IR modes)
│   │   ├── CompiledIRPanel.tsx       # Read-only .ll viewer
│   │   ├── AIRPanel.tsx              # Read-only AIR JSON (collapsible)
│   │   ├── GraphPanel.tsx            # Cytoscape.js + tab switching
│   │   ├── FileUpload.tsx            # Drag-and-drop
│   │   ├── ExamplesMenu.tsx          # Bundled examples dropdown
│   │   └── StatusBar.tsx             # Progress, stats, timing
│   ├── analysis/
│   │   ├── compiler-explorer.ts      # Compiler Explorer API client
│   │   ├── tree-sitter.ts            # Load WASM grammar, parse .ll
│   │   ├── cst-to-air.ts            # CST → AIR JSON (~1000 lines)
│   │   └── saf-wasm.ts              # Load + call saf_wasm.analyze()
│   ├── graph/
│   │   ├── cytoscape-config.ts       # Shared layout/styling
│   │   ├── cfg-renderer.ts
│   │   ├── callgraph-renderer.ts
│   │   ├── defuse-renderer.ts
│   │   ├── valueflow-renderer.ts
│   │   └── pta-renderer.ts
│   ├── examples/
│   │   ├── pointer-alias.c
│   │   ├── indirect-call.c
│   │   ├── struct-field.c
│   │   └── taint-flow.c
│   ├── wasm/                         # ← wasm-pack output (gitignored)
│   │   ├── saf_wasm.wasm
│   │   ├── saf_wasm.js
│   │   └── saf_wasm.d.ts
│   └── types/
│       ├── air.ts                    # AIR JSON type definitions
│       └── property-graph.ts         # PropertyGraph JSON type definitions
```

`analysis/` is framework-agnostic (pure TS, no React) — extractable as a standalone library later.

## Testing Strategy

### Layer 1: WASM Build

```rust
// crates/saf-wasm/tests/smoke.rs
#[test]
fn analyze_returns_valid_json() {
    let air = include_str!("fixtures/simple.air.json");
    let result = saf_wasm::analyze(air, "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(parsed["cfg"].is_object());
}
```

CI: `wasm-pack build` succeeds on every PR.

### Layer 2: CST → AIR Correctness

Golden file comparison — generate AIR JSON from native inkwell path, compare structurally against tree-sitter path output:

```bash
# One-time, generate golden files inside Docker
cargo run -p saf-cli -- convert tests/fixtures/llvm/e2e/simple.ll \
  -o tests/fixtures/air-json/simple.air.json
```

```typescript
// playground/src/analysis/__tests__/cst-to-air.test.ts
test('simple.ll produces correct AIR', () => {
  const cst = parse(readFileSync('tests/fixtures/llvm/e2e/simple.ll'));
  const air = convertToAIR(cst);
  expect(air.modules[0].functions.length).toBe(golden.modules[0].functions.length);
  expect(operationTypes(air)).toEqual(operationTypes(golden));
});
```

Compare structural equivalence (function count, operation types, operand counts) not exact IDs.

### Layer 3: End-to-End

Playwright tests against the running playground:

```typescript
test('analyze C example produces CFG', async ({ page }) => {
  await page.goto('http://localhost:5173');
  await page.click('text=Examples');
  await page.click('text=Pointer Alias');
  await page.click('text=Analyze');
  await page.waitForSelector('.cytoscape-container canvas');
});
```

### Makefile Targets

| Target | Scope |
|---|---|
| `make test-wasm` | `cargo test -p saf-wasm` (native, no Docker) |
| `make test-playground` | `cd playground && npm test` (vitest) |
| `make test-playground-e2e` | `cd playground && npx playwright test` |

### Not Tested

- Compiler Explorer API (external — mocked in tests)
- Cytoscape layout aesthetics
- WASM performance (benchmarked separately)

## Implementation Milestones

### M1: WASM Foundation (3-4 days)

- Create `crates/saf-wasm/` with `analyze()` entry point
- Feature-gate Z3 in `saf-analysis`
- `wasm-pack build` succeeds
- Smoke test: hardcoded AIR JSON → PropertyGraph JSON
- `make wasm` / `make wasm-dev` targets
- **Demo:** call `analyze()` from a Node.js script

### M2: Tree-sitter + CST → AIR Converter (1-2 weeks)

- Scaffold `playground/` with Vite + React
- Integrate `web-tree-sitter` + `tree-sitter-llvm.wasm`
- Implement `cst-to-air.ts`
- Golden file tests against native AIR output
- **Demo:** paste `.ll`, see AIR JSON

### M3: Analysis Pipeline End-to-End (2-3 days)

- Wire tree-sitter → CST → AIR → `saf-wasm.analyze()` in Web Worker
- Render CFG with Cytoscape.js
- Status bar with timing
- **Demo:** paste `.ll`, click Analyze, see CFG

### M4: Full UI (1 week)

- All five graph tabs
- CodeMirror with C/C++ and LLVM IR modes
- Compiler Explorer integration
- Compiled IR and AIR read-only tabs
- Examples dropdown
- Node click → source highlight cross-linking
- `make playground-dev` / `make playground`
- **Demo:** full playground, works locally

### M5: Deploy & Polish (2-3 days)

- GitHub Actions CI: build WASM + test + deploy
- `make playground-deploy`
- Loading states, error messages
- Playwright e2e tests
- **Demo:** live at GitHub Pages

**Total: ~4-5 weeks**

## Out of Scope

- Z3 / path feasibility checking
- Checkers / IFDS (future Tier C)
- Python bindings in WASM
- `.bc` bitcode input (`.ll` text only)
- Multi-file / whole-program linking
- Editing AIR or analysis results
- Mobile-optimized layout
