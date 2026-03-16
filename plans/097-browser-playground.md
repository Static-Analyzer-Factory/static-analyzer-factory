# Plan 097: Browser Playground

**Epic:** New — Browser Playground
**Status:** approved
**Created:** 2026-02-10
**Design doc:** `docs/plans/2026-02-10-browser-playground-design.md`

## Overview

Client-side web playground: users write C/C++ or LLVM IR in the browser, SAF analyzes it via WASM, results visualized with Cytoscape.js. No server — Compiler Explorer API compiles C/C++ to `.ll`, tree-sitter-llvm parses `.ll` in WASM, TypeScript converts CST to AIR JSON, `saf-wasm` runs analysis.

Analyses exposed: CFG, call graph, def-use, value-flow, points-to.

## Milestones

- **M1 (Tasks 1-2):** WASM foundation — `saf-wasm` crate builds, Z3 feature-gated
- **M2 (Tasks 3-4):** Parser — tree-sitter + CST→AIR converter
- **M3 (Tasks 5-6):** UI — React app with graph rendering
- **M4 (Task 7):** Deploy — CI + GitHub Pages

## Verification Gates

After M1: `make wasm` succeeds, `make test` still passes (no regression)
After M2: CST→AIR golden tests pass against native AIR output
After M3: `make playground-dev` runs, full pipeline works locally
After M4: Live at GitHub Pages

---

## Task 1: Feature-gate Z3 in `saf-analysis`

**Agent type:** Code editor (Rust)
**Depends on:** Nothing
**Estimated lines changed:** ~60

Make Z3 an optional dependency so `saf-analysis` compiles without it (required for `wasm32`).

### Step 1: Edit `crates/saf-analysis/Cargo.toml`

Change the `[features]` section and Z3 dependency:

```toml
# Before:
[features]
experimental = []

# After:
[features]
default = ["z3-solver"]
experimental = []
z3-solver = ["dep:z3"]
```

Change the Z3 dependency line:

```toml
# Before:
z3 = { version = "0.19", features = ["bundled"] }

# After:
z3 = { version = "0.19", features = ["bundled"], optional = true }
```

### Step 2: Gate Z3 modules in `crates/saf-analysis/src/lib.rs`

Add `#[cfg(feature = "z3-solver")]` before these lines:

- Line 70: `pub mod z3_utils;`
- Line 36: `pub use valueflow::taint_z3;`

For line 24 (PTA re-exports), the `Z3IndexChecker` and `Z3IndexDiagnostics` items need gating. Split the `pub use pta::{...}` line: keep non-Z3 items ungated, gate Z3 items separately:

```rust
#[cfg(feature = "z3-solver")]
pub use pta::{Z3IndexChecker, Z3IndexDiagnostics};
```

Similarly for line 40-42 (valueflow re-exports), gate `TaintFlowZ3Result` and `filter_taint_flows_z3`:

```rust
#[cfg(feature = "z3-solver")]
pub use valueflow::{TaintFlowZ3Result, filter_taint_flows_z3};
```

### Step 3: Gate Z3 submodules

Each file below needs `#[cfg(feature = "z3-solver")]` on its `mod` and `pub use` lines:

**`crates/saf-analysis/src/pta/mod.rs`:**
- Line 36: `mod z3_index;` → `#[cfg(feature = "z3-solver")] mod z3_index;`
- Line 89: `pub use z3_index::{...};` → `#[cfg(feature = "z3-solver")] pub use z3_index::{...};`

**`crates/saf-analysis/src/checkers/mod.rs`:**
- Line 46: `pub mod z3solver;` → `#[cfg(feature = "z3-solver")] pub mod z3solver;`
- Line 65: `pub use z3solver::{...};` → `#[cfg(feature = "z3-solver")] pub use z3solver::{...};`

**`crates/saf-analysis/src/ifds/mod.rs`:**
- Line 31: `pub mod taint_z3;` → `#[cfg(feature = "z3-solver")] pub mod taint_z3;`
- Line 32: `pub mod typestate_z3;` → `#[cfg(feature = "z3-solver")] pub mod typestate_z3;`
- Line 46: `pub use taint_z3::{...};` → `#[cfg(feature = "z3-solver")] pub use taint_z3::{...};`

**`crates/saf-analysis/src/valueflow/mod.rs`:**
- Line 16: `pub mod taint_z3;` → `#[cfg(feature = "z3-solver")] pub mod taint_z3;`
- Line 29: `pub use taint_z3::{...};` → `#[cfg(feature = "z3-solver")] pub use taint_z3::{...};`

**`crates/saf-analysis/src/absint/mod.rs`:**
- Line 29: `pub mod numeric_z3;` → `#[cfg(feature = "z3-solver")] pub mod numeric_z3;`

### Step 4: Gate Z3 usage in non-Z3 modules

Files that import from `z3_utils` but are NOT themselves Z3-only modules:

**`crates/saf-analysis/src/pta/value_origin.rs` line 23:**
```rust
// Before:
use crate::z3_utils::guard::PathCondition;
// After:
#[cfg(feature = "z3-solver")]
use crate::z3_utils::guard::PathCondition;
```
Find all uses of `PathCondition` in this file and gate them with `#[cfg(feature = "z3-solver")]`. If `PathCondition` is used in a function signature, provide a no-op alternative.

**`crates/saf-analysis/src/pta/path_sensitive.rs` lines 17-19:**
```rust
#[cfg(feature = "z3-solver")]
use crate::z3_utils::dominator::{compute_dominators, extract_dominating_guards};
#[cfg(feature = "z3-solver")]
use crate::z3_utils::guard::{Guard, PathCondition, ValueLocationIndex};
#[cfg(feature = "z3-solver")]
use crate::z3_utils::solver::{FeasibilityResult, PathFeasibilityChecker};
```
Gate the Z3-dependent code paths in this file. The path-sensitive solver should still compile but skip Z3 feasibility checks when the feature is off.

**`crates/saf-analysis/src/checkers/pathsens.rs` lines 8-14:**
Gate all re-exports with `#[cfg(feature = "z3-solver")]`.

**`crates/saf-analysis/src/checkers/pathsens_runner.rs` line 26:**
Gate the import and Z3-dependent code paths.

### Step 5: Verify

Run these commands (main agent runs these, not the subagent):
- `make fmt && make lint` — check compilation with default features (Z3 on)
- `cargo check -p saf-analysis --no-default-features` — check compilation without Z3 (locally, no LLVM needed for just `check`)
- `make test` — full regression test

---

## Task 2: Create `saf-wasm` crate

**Agent type:** Code editor (Rust)
**Depends on:** Task 1
**Estimated lines:** ~120

### Step 1: Create `crates/saf-wasm/Cargo.toml`

```toml
[package]
name = "saf-wasm"
version = "0.1.0"
edition = "2021"
description = "SAF static analysis engine compiled to WebAssembly"
publish = false

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
saf-core = { path = "../saf-core" }
saf-analysis = { path = "../saf-analysis", default-features = false }
saf-frontends = { path = "../saf-frontends", default-features = false }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
serde_json = "1"
```

### Step 2: Create `crates/saf-wasm/src/lib.rs`

```rust
//! SAF WebAssembly entry point.
//!
//! Provides a single `analyze()` function that takes AIR JSON,
//! runs the full analysis pipeline, and returns PropertyGraph
//! JSON results.

use wasm_bindgen::prelude::*;

use saf_core::air::AirBundle;
use saf_frontends::air_json::AirJsonFrontend;
use saf_frontends::api::Frontend;

/// Analyze an AIR JSON bundle and return results as JSON.
///
/// # Arguments
/// * `air_json` - AIR JSON string (matches `saf-frontends` air_json schema)
/// * `config_json` - Configuration JSON string (currently unused, pass `"{}"`)
///
/// # Returns
/// JSON string containing analysis results with keys:
/// `cfg`, `callgraph`, `defuse`, `valueflow`, `pta`, `stats`
#[wasm_bindgen]
pub fn analyze(air_json: &str, config_json: &str) -> String {
    // Set up panic hook for better WASM error messages
    console_error_panic_hook::set_once();

    match run_analysis(air_json, config_json) {
        Ok(result) => result,
        Err(e) => {
            let error = serde_json::json!({ "error": e });
            serde_json::to_string(&error).unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
        }
    }
}

fn run_analysis(air_json: &str, _config_json: &str) -> Result<String, String> {
    // 1. Parse AIR JSON into AirBundle
    let bundle: AirBundle = serde_json::from_str(air_json)
        .map_err(|e| format!("Failed to parse AIR JSON: {e}"))?;

    // 2. Build analysis artifacts
    let module = &bundle.module;

    // Build CFG for each function
    let mut cfg_results = serde_json::Map::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = saf_analysis::cfg::build_cfg(func);
        let pg = saf_analysis::export::cfg_to_property_graph(&cfg, func);
        cfg_results.insert(
            func.name.clone(),
            serde_json::to_value(&pg).map_err(|e| e.to_string())?,
        );
    }

    // Build call graph
    let callgraph = saf_analysis::callgraph::build_callgraph(module);
    let cg_pg = saf_analysis::export::callgraph_to_property_graph(&callgraph, module);

    // Build def-use
    let mut defuse_results = serde_json::Map::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let defuse = saf_analysis::defuse::build_defuse(func);
        let pg = saf_analysis::export::defuse_to_property_graph(&defuse, func);
        defuse_results.insert(
            func.name.clone(),
            serde_json::to_value(&pg).map_err(|e| e.to_string())?,
        );
    }

    // Run PTA
    let pta_config = saf_analysis::pta::PtaConfig::default();
    let pta_result = saf_analysis::pta::solve_andersen(module, &pta_config);
    let pta_export = pta_result.export();

    // Build value-flow
    let vfg = saf_analysis::valueflow::build_valueflow(module, &pta_result);
    let vf_pg = saf_analysis::export::valueflow_to_property_graph(&vfg);

    // 3. Assemble results
    let stats = serde_json::json!({
        "functions": module.functions.len(),
        "blocks": module.functions.iter().map(|f| f.blocks.len()).sum::<usize>(),
        "instructions": module.functions.iter()
            .flat_map(|f| f.blocks.iter())
            .map(|b| b.instructions.len())
            .sum::<usize>(),
    });

    let result = serde_json::json!({
        "cfg": cfg_results,
        "callgraph": serde_json::to_value(&cg_pg).map_err(|e| e.to_string())?,
        "defuse": defuse_results,
        "valueflow": serde_json::to_value(&vf_pg).map_err(|e| e.to_string())?,
        "pta": serde_json::to_value(&pta_export).map_err(|e| e.to_string())?,
        "stats": stats,
    });

    serde_json::to_string(&result).map_err(|e| e.to_string())
}
```

Note: The exact API calls (`build_cfg`, `build_callgraph`, `solve_andersen`, `build_valueflow`, `export::*_to_property_graph`) must be verified against the actual `saf-analysis` public API. Read `crates/saf-analysis/src/lib.rs` re-exports and `crates/saf-analysis/src/export/mod.rs` to confirm function names and signatures. The above is a structural template — adjust function names to match the real API.

### Step 3: Add `console-error-panic-hook` dependency

Add to `crates/saf-wasm/Cargo.toml`:
```toml
console-error-panic-hook = "0.1"
```

### Step 4: Add to workspace

Edit the root `Cargo.toml` workspace members list — add `"crates/saf-wasm"`.

### Step 5: Add Makefile targets

Add to the root `Makefile`:

```makefile
.PHONY: wasm wasm-dev

wasm: ## Build saf-wasm (release)
	wasm-pack build crates/saf-wasm --target web --release --out-dir ../../playground/src/wasm

wasm-dev: ## Build saf-wasm (debug, faster)
	wasm-pack build crates/saf-wasm --target web --dev --out-dir ../../playground/src/wasm
```

### Step 6: Create smoke test

Create `crates/saf-wasm/tests/smoke.rs`:

```rust
//! Smoke test for saf-wasm — runs natively (not in browser).

#[test]
fn analyze_minimal_air_json() {
    // Minimal valid AIR JSON with one empty function
    let air_json = r#"{
        "frontend_id": "test",
        "schema_version": "0.1.0",
        "module": {
            "id": "0x00000000000000000000000000000001",
            "name": "test",
            "functions": [],
            "globals": [],
            "source_files": [],
            "type_hierarchy": null,
            "constants": {}
        }
    }"#;

    let result = saf_wasm::analyze(air_json, "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert!(parsed.get("error").is_none(), "Got error: {result}");
    assert!(parsed["cfg"].is_object());
    assert!(parsed["callgraph"].is_object());
    assert!(parsed["stats"].is_object());
}
```

Note: The AIR JSON schema must match what `AirBundle`'s `Deserialize` expects. Read `crates/saf-core/src/air.rs` to verify the exact field names and structure. Adjust the test fixture accordingly.

### Verification

Main agent runs:
- `cargo check -p saf-wasm` — compiles natively
- `cargo test -p saf-wasm` — smoke test passes
- `make wasm` — WASM build succeeds (requires `wasm-pack` installed)
- `make test` — no regression

---

## Task 3: Scaffold React playground app

**Agent type:** Code editor (TypeScript/React)
**Depends on:** Nothing (parallel with Tasks 1-2)
**Estimated lines:** ~500

### Step 1: Initialize project

Run from repo root:
```bash
cd playground
npm create vite@latest . -- --template react-ts
npm install
npm install cytoscape @types/cytoscape
npm install @anthropic-ai/sdk  # remove this - just placeholder
npm install codemirror @codemirror/lang-javascript @codemirror/theme-one-dark
npm install web-tree-sitter
```

### Step 2: Configure Vite for WASM

Create `playground/vite.config.ts`:
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: './',  // relative paths for GitHub Pages
  build: {
    outDir: 'dist',
  },
  optimizeDeps: {
    exclude: ['web-tree-sitter'],  // WASM modules need special handling
  },
});
```

### Step 3: Create directory structure

```
playground/src/
├── main.tsx
├── App.tsx
├── components/
│   ├── SourcePanel.tsx
│   ├── CompiledIRPanel.tsx
│   ├── AIRPanel.tsx
│   ├── GraphPanel.tsx
│   ├── ExamplesMenu.tsx
│   └── StatusBar.tsx
├── analysis/
│   ├── compiler-explorer.ts
│   ├── tree-sitter.ts
│   ├── cst-to-air.ts       (stub — implemented in Task 4)
│   └── saf-wasm.ts
├── graph/
│   ├── cytoscape-config.ts
│   ├── cfg-renderer.ts
│   ├── callgraph-renderer.ts
│   ├── defuse-renderer.ts
│   ├── valueflow-renderer.ts
│   └── pta-renderer.ts
├── examples/
│   ├── pointer-alias.c
│   ├── indirect-call.c
│   ├── struct-field.c
│   └── taint-flow.c
├── types/
│   ├── air.ts
│   └── property-graph.ts
└── wasm/                    (gitignored — output of make wasm)
```

### Step 4: Implement core components

**`App.tsx`** — Top-level state and layout. State shape:

```typescript
interface AppState {
  inputMode: 'c' | 'llvm';
  sourceCode: string;
  compiledIR: string | null;
  airJSON: object | null;
  results: AnalysisResults | null;
  activeGraph: 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';
  selectedNode: string | null;
  status: 'idle' | 'compiling' | 'parsing' | 'analyzing' | 'ready' | 'error';
  error: string | null;
}
```

Pipeline on Analyze click:
1. If C/C++: call Compiler Explorer → set `compiledIR`
2. Parse `.ll` with tree-sitter → CST
3. Convert CST → AIR JSON → set `airJSON`
4. Call `saf-wasm.analyze(airJSON)` → set `results`
5. Render active graph tab

**`SourcePanel.tsx`** — CodeMirror editor. Two modes: C/C++ (editable) and LLVM IR (editable). Tab bar: `[C/C++] [Compiled IR ●] [AIR ●] [LLVM IR]`. Line counter showing `N / 200 lines`. Analyze button disabled above 200 lines.

**`GraphPanel.tsx`** — Tab bar for graph types. Cytoscape.js container. Takes PropertyGraph JSON, converts to Cytoscape elements, renders. Click handler emits selected node ID to parent.

**`StatusBar.tsx`** — Shows status string and timing stats.

**`analysis/compiler-explorer.ts`:**
```typescript
const GODBOLT_API = 'https://godbolt.org/api/compiler/clang1800/compile';

export async function compileToLLVM(source: string): Promise<string> {
  const response = await fetch(GODBOLT_API, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
    body: JSON.stringify({
      source,
      options: {
        userArguments: '-S -emit-llvm -O0 -g',
        compilerOptions: { skipAsm: false },
        filters: { binary: false, execute: false },
      },
    }),
  });

  const data = await response.json();
  if (data.code !== 0) {
    const stderr = data.stderr?.map((l: any) => l.text).join('\n') || 'Unknown error';
    throw new Error(stderr);
  }
  return data.asm.map((l: any) => l.text).join('\n');
}
```

**`analysis/saf-wasm.ts`:**
```typescript
import init, { analyze } from '../wasm/saf_wasm';

let initialized = false;

export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}

export function runAnalysis(airJson: string): AnalysisResults {
  return JSON.parse(analyze(airJson, '{}'));
}
```

**`graph/cytoscape-config.ts`** — Shared Cytoscape config: dark theme, node styles (green for entry, red for exit, blue for selected), edge styles (green/red for true/false branches), layout defaults.

**`analysis/cst-to-air.ts`** — Stub that throws "not implemented". Replaced in Task 4.

### Step 5: Create example C files

Write 4 small C programs (each under 30 lines) in `playground/src/examples/`:

**`pointer-alias.c`** — Two pointers to same struct, demonstrate aliasing
**`indirect-call.c`** — Function pointer call, demonstrate call graph resolution
**`struct-field.c`** — Struct with multiple fields, demonstrate field-sensitive PTA
**`taint-flow.c`** — Value flows from input to output, demonstrate value-flow graph

### Step 6: Add Makefile targets

Add to root `Makefile`:
```makefile
.PHONY: playground playground-dev playground-deploy

playground: wasm ## Build playground for production
	cd playground && npm ci && npm run build

playground-dev: wasm-dev ## Start playground dev server (http://localhost:5173)
	cd playground && npm install && npm run dev

playground-deploy: playground ## Deploy to GitHub Pages
	cd playground && npx gh-pages -d dist
```

### Step 7: Add `.gitignore` entries

Add to `playground/.gitignore`:
```
node_modules/
dist/
src/wasm/
```

### Verification

Main agent runs:
- `cd playground && npm install && npm run build` — builds without errors
- `make playground-dev` — dev server starts at localhost:5173
- Open in browser — layout renders, examples dropdown works, Analyze button shows "not implemented" for CST→AIR step

---

## Task 4: Implement CST → AIR converter

**Agent type:** Code editor (TypeScript)
**Depends on:** Task 3 (scaffold exists)
**Estimated lines:** ~1000-1200

This is the largest single task. The converter translates tree-sitter-llvm's concrete syntax tree into AIR JSON matching the schema that `saf-frontends/src/air_json.rs` consumes.

### Context files to read first

Before writing code, the agent MUST read these files to understand the target schema:

1. `crates/saf-frontends/src/air_json_schema.rs` — the JSON schema types
2. `crates/saf-frontends/src/air_json.rs` — how AIR JSON is parsed (the inverse of what we're building)
3. `crates/saf-core/src/air.rs` — the AIR types (AirBundle, AirModule, AirFunction, AirBlock, Instruction, Operation enum)
4. `crates/saf-frontends/src/llvm/mapping.rs` — reference for what data to extract from each instruction type

### Step 1: Set up tree-sitter integration

Create `playground/src/analysis/tree-sitter.ts`:

```typescript
import Parser from 'web-tree-sitter';

let parser: Parser | null = null;

export async function initParser(): Promise<void> {
  await Parser.init();
  parser = new Parser();
  const lang = await Parser.Language.load('/tree-sitter-llvm.wasm');
  parser.setLanguage(lang);
}

export function parseLLVM(source: string): Parser.Tree {
  if (!parser) throw new Error('Parser not initialized');
  return parser.parse(source);
}
```

The `tree-sitter-llvm.wasm` file must be built from the tree-sitter-llvm grammar and placed in `playground/public/`. Build instructions:
```bash
git clone https://github.com/benwilliamgraham/tree-sitter-llvm
cd tree-sitter-llvm
npx tree-sitter build --wasm
cp tree-sitter-llvm.wasm /path/to/playground/public/
```

### Step 2: Implement `cst-to-air.ts`

The file `playground/src/analysis/cst-to-air.ts` converts a tree-sitter `Tree` into AIR JSON.

**ID generation:** Use a simple counter-based scheme (no need for BLAKE3 in the browser). IDs are `u128` serialized as `"0x"` + 32 hex chars. Use a monotonic counter formatted as hex:

```typescript
let idCounter = 0;
function makeId(prefix: string): string {
  idCounter++;
  const hex = idCounter.toString(16).padStart(32, '0');
  return `0x${hex}`;
}
```

**Top-level function:**

```typescript
export function convertToAIR(tree: Parser.Tree): AirBundle {
  idCounter = 0;  // reset per conversion
  const root = tree.rootNode;
  const moduleId = makeId('module');

  const functions: AirFunction[] = [];
  const globals: AirGlobal[] = [];

  for (const child of root.children) {
    if (child.type === 'fn_define') {
      functions.push(convertFunction(child));
    } else if (child.type === 'declare') {
      functions.push(convertDeclaration(child));
    } else if (child.type === 'global_global') {
      globals.push(convertGlobal(child));
    }
  }

  return {
    frontend_id: 'tree-sitter',
    schema_version: '0.1.0',
    module: { id: moduleId, functions, globals, source_files: [], type_hierarchy: null, constants: {} },
  };
}
```

**Instruction mapping:** For each tree-sitter `instruction_*` node type, map to the corresponding AIR `Operation`. The mapping follows `mapping.rs` lines 712-820:

| tree-sitter node type | AIR Operation |
|---|---|
| `instruction_alloca` | `{ op: "alloca", size_bytes: <parsed> }` |
| `instruction_load` | `{ op: "load" }` |
| `instruction_store` | `{ op: "store" }` |
| `instruction_getelementptr` | `{ op: "gep", field_path: <parsed> }` |
| `instruction_call` | `{ op: "call_direct", callee: <id> }` or `{ op: "call_indirect" }` |
| `instruction_ret` | `{ op: "ret" }` |
| `instruction_br` | `{ op: "br" }` or `{ op: "cond_br" }` (check child count) |
| `instruction_switch` | `{ op: "switch" }` |
| `instruction_phi` | `{ op: "phi", incoming: [...] }` |
| `instruction_select` | `{ op: "select" }` |
| `instruction_icmp` | `{ op: "binary_op", kind: <icmp_predicate> }` |
| `instruction_fcmp` | `{ op: "binary_op", kind: <fcmp_predicate> }` |
| `instruction_bin_op` | `{ op: "binary_op", kind: <from bin_op_keyword child> }` |
| `instruction_cast` | `{ op: "cast", kind: <from cast_inst child>, target_bits: <parsed> }` |
| `instruction_freeze` | `{ op: "freeze" }` |
| `instruction_unreachable` | `{ op: "unreachable" }` |
| `instruction_fence`, `instruction_cmpxchg`, `instruction_atomicrmw` | Skip (return null) |

**SSA resolution:** Maintain a `Map<string, string>` mapping `%name` → `ValueId` within each function. Reset at function boundaries. When a `local_var` node is encountered as an operand, look up or create its ValueId.

**Key helper functions to implement:**

- `convertFunction(node)` → `AirFunction`
- `convertDeclaration(node)` → `AirFunction` (is_declaration = true)
- `convertGlobal(node)` → `AirGlobal`
- `convertBlock(node, blockIds)` → `AirBlock`
- `convertInstruction(node, blockIds)` → `Instruction | null`
- `collectOperands(node)` → `ValueId[]`
- `extractAllocaSize(node)` → `number | null`
- `extractGepFieldPath(node)` → `FieldPath`
- `parseType(node)` → type info for size computation
- `resolveValue(node)` → `ValueId` (handles local_var, global_var, constants)

### Step 3: Define TypeScript types

Create `playground/src/types/air.ts` matching the AIR JSON schema. Read `crates/saf-frontends/src/air_json_schema.rs` for the exact field names.

Create `playground/src/types/property-graph.ts` matching the PropertyGraph export format (nodes array, edges array, metadata).

### Verification

Main agent runs:
- `cd playground && npm test` — golden file tests pass
- Manual: paste a simple `.ll` file, check the AIR tab shows valid JSON

---

## Task 5: Graph renderers

**Agent type:** Code editor (TypeScript/React)
**Depends on:** Task 3 (scaffold)
**Estimated lines:** ~400

Implement the five Cytoscape.js renderers in `playground/src/graph/`. Each converts a PropertyGraph JSON object into Cytoscape elements.

### Context file to read first

Read `crates/saf-analysis/src/export/mod.rs` or the PropertyGraph format description in `CLAUDE.md` (the "SAF graph exports use a unified PropertyGraph format" section) to understand the JSON structure.

PropertyGraph format:
```json
{
  "schema_version": "0.1.0",
  "graph_type": "<type>",
  "metadata": {},
  "nodes": [{"id": "0x...", "labels": [...], "properties": {...}}, ...],
  "edges": [{"src": "0x...", "dst": "0x...", "edge_type": "...", "properties": {}}, ...]
}
```

### Step 1: Shared config (`cytoscape-config.ts`)

Dark theme colors, node shapes, edge styles. Provide a function `createCyInstance(container: HTMLElement, elements: ElementDefinition[]) → Core`.

Layout presets:
- CFG: `dagre` (hierarchical top-to-bottom)
- Call graph: `cose` (force-directed)
- Def-use: `dagre`
- Value-flow: `cose`
- PTA: `grid` or custom (points-to is more tabular)

### Step 2: Implement each renderer

Each renderer file exports a function: `(pg: PropertyGraph) → ElementDefinition[]`

**`cfg-renderer.ts`:**
- Nodes: labels from `properties.name`, color by entry/exit
- Edges: `edge_type: "FLOWS_TO"`, color true/false branches differently
- Show instruction summary in node body

**`callgraph-renderer.ts`:**
- Nodes: `properties.name` (function name), `properties.kind` (defined/external)
- Edges: `edge_type: "CALLS"`
- External functions in different color

**`defuse-renderer.ts`:**
- Nodes: `labels: ["Value"]` or `["Instruction"]`
- Edges: `edge_type: "DEFINES"` or `"USED_BY"`

**`valueflow-renderer.ts`:**
- Nodes: `properties.kind` (Value/Location/UnknownMem)
- Edges: `edge_type: "Direct"|"Store"|"Load"|...`

**`pta-renderer.ts`:**
- PTA results are NOT PropertyGraph format — they're `{ points_to: [{value, locations}] }`
- Render as a bipartite graph: value nodes on left, location nodes on right, edges for points-to relations
- Or as a table view (simpler): value → {locations} list

### Step 3: Wire into `GraphPanel.tsx`

The GraphPanel component:
1. Receives `results` and `activeGraph` from App state
2. Calls the appropriate renderer to get Cytoscape elements
3. Renders into a Cytoscape container
4. On node click: emit `selectedNode` to parent for cross-linking

### Verification

Main agent runs:
- `make playground-dev` — all five graph tabs render with sample data
- Click through each tab — no console errors

---

## Task 6: Wire end-to-end pipeline + Web Worker

**Agent type:** Code editor (TypeScript)
**Depends on:** Tasks 2, 3, 4, 5
**Estimated lines:** ~200

### Step 1: Create Web Worker (`playground/src/analysis/worker.ts`)

```typescript
// Web Worker that runs: tree-sitter parse → CST→AIR → saf-wasm analyze
import { initParser, parseLLVM } from './tree-sitter';
import { convertToAIR } from './cst-to-air';
import { initWasm, runAnalysis } from './saf-wasm';

self.onmessage = async (e: MessageEvent) => {
  const { llSource } = e.data;
  try {
    self.postMessage({ type: 'status', status: 'parsing' });
    await initParser();
    const tree = parseLLVM(llSource);

    self.postMessage({ type: 'status', status: 'converting' });
    const air = convertToAIR(tree);
    const airJson = JSON.stringify(air);

    self.postMessage({ type: 'air', air });
    self.postMessage({ type: 'status', status: 'analyzing' });

    await initWasm();
    const results = runAnalysis(airJson);

    self.postMessage({ type: 'results', results });
  } catch (err) {
    self.postMessage({ type: 'error', error: String(err) });
  }
};
```

### Step 2: Wire into App.tsx

The Analyze button handler:
1. If C/C++ mode: `await compileToLLVM(sourceCode)` → set `compiledIR`
2. Post `compiledIR` (or `sourceCode` if LLVM IR mode) to Web Worker
3. Worker sends back status updates, AIR JSON, and final results
4. App state updates trigger re-renders of AIR panel and graph panel

### Step 3: Cross-linking

When user clicks a node in GraphPanel:
- Extract the source line/function from node properties
- SourcePanel scrolls to and highlights that line
- If function selected in call graph, switch CFG tab to that function

### Verification

Main agent runs:
- `make playground-dev`
- Open browser, select "Pointer Alias" example, click Analyze
- See: Compiled IR tab populated, AIR tab populated, CFG renders
- Click through all five graph tabs
- Click a node in call graph → source panel highlights

---

## Task 7: CI + GitHub Pages deploy

**Agent type:** Code editor (YAML/config)
**Depends on:** Tasks 1-6
**Estimated lines:** ~80

### Step 1: Create GitHub Actions workflow

Create `.github/workflows/playground.yml`:

```yaml
name: Playground

on:
  push:
    branches: [main]
    paths:
      - 'crates/saf-wasm/**'
      - 'crates/saf-core/**'
      - 'crates/saf-analysis/**'
      - 'crates/saf-frontends/**'
      - 'playground/**'
  pull_request:
    paths:
      - 'crates/saf-wasm/**'
      - 'playground/**'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM
        run: make wasm

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: playground/package-lock.json

      - name: Build playground
        run: cd playground && npm ci && npm run build

      - name: Run tests
        run: cd playground && npm test

      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main' && github.event_name == 'push'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./playground/dist
```

### Step 2: Configure Vite base path for GitHub Pages

If the repo is deployed to `https://<user>.github.io/<repo>/`, set the base path in `playground/vite.config.ts`:

```typescript
export default defineConfig({
  base: process.env.GITHUB_PAGES ? '/static-analyzer-lib/' : './',
  // ... rest of config
});
```

Update the build script in `playground/package.json`:
```json
{
  "scripts": {
    "build": "GITHUB_PAGES=1 vite build",
    "build:local": "vite build"
  }
}
```

### Step 3: Enable GitHub Pages

In the repo settings, set GitHub Pages source to the `gh-pages` branch.

### Verification

- Push to a branch, verify the CI workflow runs and passes
- Merge to main, verify deployment to GitHub Pages
- Visit the live URL, verify the playground works

---

## Task Dependency Graph

```
Task 1 (Z3 feature-gate)  ──→  Task 2 (saf-wasm crate)  ──→  Task 6 (wire e2e)
                                                                  ↑
Task 3 (scaffold React)   ──→  Task 4 (CST→AIR)  ────────────────┤
         │                                                         │
         └──→  Task 5 (graph renderers)  ──────────────────────────┘
                                                                    │
                                                               Task 7 (CI/deploy)
```

**Parallel work:**
- Tasks 1+3 can run in parallel (Rust vs TS, no overlap)
- Task 4 can start once Task 3 is done (needs scaffold)
- Task 5 can start once Task 3 is done (needs scaffold)
- Tasks 4+5 can run in parallel
- Task 2 depends on Task 1
- Task 6 depends on Tasks 2, 3, 4, 5
- Task 7 depends on Task 6

**Agent context management:**
- Tasks 1, 2: Rust-only, read ~5 files each, edit ~10 files each
- Task 3: TS/React only, creates files from scratch, no SAF crate reading needed
- Task 4: TS only, reads 4 SAF schema files for reference, then writes converter
- Task 5: TS only, reads PropertyGraph format doc, writes renderers
- Task 6: TS only, wires existing pieces together, ~200 lines
- Task 7: YAML/config only, minimal context needed
