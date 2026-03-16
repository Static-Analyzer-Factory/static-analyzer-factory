# Plan 115: Shared Web Package + Interactive Tutorials

## Context

The tutorials app displays static pre-computed content while the playground has a working WASM analysis pipeline. Making tutorials interactive requires the analysis pipeline — but copying it would duplicate ~2,400 more lines on top of ~1,500 already duplicated (types + graph renderers). This plan consolidates shared code into an npm workspace package, then adds interactive analysis to tutorials.

## Agent Team Structure

```
Leader ─── Agent A (shared package)
       ├── Agent B (playground rewire)  ← blocked on A
       └── Agent C (tutorials + interactive)  ← blocked on A
```

**Execution order:**
1. **Leader** creates root `package.json` (1 file)
2. **Agent A** creates `packages/shared/` with all moved files (sequential, must complete first)
3. **Agent B** + **Agent C** run in parallel after A completes
4. **Leader** does CI/Makefile updates + builds + Playwright verification

---

## Leader Tasks

### L1: Create root workspace config

Create `package.json` at project root:
```json
{ "private": true, "workspaces": ["packages/shared", "playground", "tutorials", "site"] }
```

### L2: CI workflow update (after all agents done)

**`.github/workflows/playground.yml`** changes:
- wasm-pack output: `--out-dir ../../packages/shared/src/wasm` (was `../../playground/src/wasm`)
- Replace per-app `npm ci` with single root `npm ci`
- Update `cache-dependency-path` to root `package-lock.json`
- Add step to copy tree-sitter WASM files to `tutorials/public/`
- Add `packages/shared/**` to path triggers

### L3: Makefile update

- `wasm`/`wasm-dev` targets: output to `packages/shared/src/wasm`
- `tutorials-dev`: copy tree-sitter WASM files from `playground/public/` first

### L4: Build verification

- `npm install` at root
- Copy tree-sitter WASM: `cp playground/public/{tree-sitter-llvm,web-tree-sitter}.wasm tutorials/public/`
- `cd playground && npm run build`
- `cd tutorials && npm run build`
- `cd site && npm run build`
- Delete old per-app `package-lock.json` files

### L5: Playwright E2E testing

- `make site-dev-all` → test all routes
- Navigate to C+graph tutorial step → edit code → click Analyze → verify live graph
- Click Reset → verify static graph restores
- Verify Python/Bash steps remain read-only
- Verify playground still works

---

## Agent A: Create Shared Package

**Scope:** Only creates/writes files under `packages/shared/`. Does NOT modify playground/ or tutorials/.

### Instructions

Create `packages/shared/` with this structure:

```
packages/shared/
  package.json
  tsconfig.json
  src/
    types/
      index.ts              # Re-export all types
      air.ts                # Copy from tutorials/src/types/air.ts
      property-graph.ts     # Copy from tutorials/src/types/property-graph.ts
    graph/
      index.ts              # Re-export all renderers + config + air-index
      cytoscape-config.ts   # Copy from playground/src/graph/cytoscape-config.ts
      cfg-renderer.ts       # Copy from tutorials/src/graph/cfg-renderer.ts (superset)
      callgraph-renderer.ts # Copy from playground/src/graph/callgraph-renderer.ts
      defuse-renderer.ts    # Copy from playground/src/graph/defuse-renderer.ts
      valueflow-renderer.ts # Copy from playground/src/graph/valueflow-renderer.ts
      pta-renderer.ts       # Copy from playground/src/graph/pta-renderer.ts
      air-index.ts          # Copy from tutorials/src/graph/air-index.ts (has block.id guard)
    analysis/
      index.ts              # Re-export: compileToLLVM, initParser, parseLLVMIR, convertToAIR, initWasm, runAnalysis, loadAllSpecs
      compiler-explorer.ts  # Copy from playground/src/analysis/compiler-explorer.ts
      tree-sitter.ts        # Copy from playground/src/analysis/tree-sitter.ts
      cst-to-air.ts         # Copy from playground/src/analysis/cst-to-air.ts
      saf-wasm.ts           # Copy from playground/src/analysis/saf-wasm.ts
      specs.ts              # Copy from playground/src/analysis/specs.ts
    wasm/
      .gitignore            # * (ignore all wasm-pack output)
      saf_wasm.js           # Copy from playground/src/wasm/saf_wasm.js
      saf_wasm_bg.wasm      # Copy from playground/src/wasm/saf_wasm_bg.wasm
      saf_wasm.d.ts         # Copy from playground/src/wasm/saf_wasm.d.ts
      saf_wasm_bg.wasm.d.ts # Copy from playground/src/wasm/saf_wasm_bg.wasm.d.ts
      package.json          # Copy from playground/src/wasm/package.json
```

**Fix import paths in copied analysis files:**
- `saf-wasm.ts` imports `'../wasm/saf_wasm.js'` — this is correct (same relative path in new location)
- `saf-wasm.ts` imports `from '../types/air'` — change to `from '../types'` (using barrel)
- `saf-wasm.ts` imports `from '../types/property-graph'` — change to `from '../types'`
- All graph renderers import from `'../types/property-graph'` — change to `'../types'`
- `air-index.ts` imports from `'../types/air'` — change to `'../types'`

**`packages/shared/package.json`:**
```json
{
  "name": "@saf/web-shared",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "exports": {
    "./types": "./src/types/index.ts",
    "./graph": "./src/graph/index.ts",
    "./analysis": "./src/analysis/index.ts"
  },
  "dependencies": {
    "cytoscape": "^3.33.1",
    "cytoscape-dagre": "^2.5.0",
    "dagre": "^0.8.5",
    "web-tree-sitter": "^0.26.5",
    "tree-sitter-llvm": "^1.1.0"
  },
  "devDependencies": {
    "@types/cytoscape": "^3.21.9",
    "@types/cytoscape-dagre": "^2.3.4",
    "typescript": "~5.9.3"
  }
}
```

**`packages/shared/tsconfig.json`:**
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "verbatimModuleSyntax": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "skipLibCheck": true,
    "composite": true,
    "declaration": true,
    "declarationMap": true
  },
  "include": ["src"]
}
```

---

## Agent B: Rewire Playground

**Scope:** Only modifies files under `playground/`. Assumes `packages/shared/` already exists and `@saf/web-shared` is available via workspace.

### Instructions

1. **Update `playground/package.json`:**
   - Add `"@saf/web-shared": "workspace:*"` to dependencies
   - Remove `web-tree-sitter`, `tree-sitter-llvm`, `tree-sitter-cli` from dependencies (now in shared)
   - Remove `@types/cytoscape`, `@types/cytoscape-dagre` from devDependencies (now in shared)

2. **Update `playground/tsconfig.app.json`:**
   - Add to compilerOptions: `"paths": { "@saf/web-shared/*": ["../packages/shared/src/*"] }`

3. **Delete these directories/files:**
   - `playground/src/types/` (entire directory)
   - `playground/src/graph/` (entire directory)
   - `playground/src/wasm/` (entire directory)
   - `playground/src/analysis/compiler-explorer.ts`
   - `playground/src/analysis/tree-sitter.ts`
   - `playground/src/analysis/cst-to-air.ts`
   - `playground/src/analysis/saf-wasm.ts`
   - `playground/src/analysis/specs.ts`

   **Keep these:** `playground/src/analysis/worker.ts`, `playground/src/analysis/pyodide-bridge.ts`, `playground/src/analysis/__tests__/`

4. **Update imports in all playground components:**

   Replace these import patterns:
   - `from '../types/air'` → `from '@saf/web-shared/types'`
   - `from '../types/property-graph'` → `from '@saf/web-shared/types'`
   - `from './types/air'` → `from '@saf/web-shared/types'`
   - `from './types/property-graph'` → `from '@saf/web-shared/types'`
   - `from '../graph/cytoscape-config'` → `from '@saf/web-shared/graph'`
   - `from '../graph/cfg-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/callgraph-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/defuse-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/valueflow-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/pta-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/air-index'` → `from '@saf/web-shared/graph'`
   - `from './analysis/compiler-explorer'` → `from '@saf/web-shared/analysis'`
   - `from './analysis/tree-sitter'` → `from '@saf/web-shared/analysis'`
   - `from './analysis/cst-to-air'` → `from '@saf/web-shared/analysis'`
   - `from './analysis/saf-wasm'` → `from '@saf/web-shared/analysis'`
   - `from './analysis/specs'` → `from '@saf/web-shared/analysis'`

   Also update dynamic imports in `worker.ts` and `pyodide-bridge.ts`:
   - `await import('./tree-sitter')` → `await import('@saf/web-shared/analysis')`
   - `await import('./cst-to-air')` → `await import('@saf/web-shared/analysis')`
   - `await import('./saf-wasm')` → `await import('@saf/web-shared/analysis')`

   Files to check: `App.tsx`, `components/GraphPanel.tsx`, `components/AnalyzerPanel.tsx`, `components/AIRPanel.tsx`, `components/ConfigPanel.tsx`, `components/SourcePanel.tsx`, `components/StatusBar.tsx`, `components/TutorialPanel.tsx`, `analysis/worker.ts`, `analysis/pyodide-bridge.ts`

5. **Verify:** TypeScript compiles without errors (`npx tsc --noEmit`)

---

## Agent C: Rewire Tutorials + Add Interactive Analysis

**Scope:** Only modifies files under `tutorials/`. Assumes `packages/shared/` already exists.

### Part 1: Rewire imports

1. **Update `tutorials/package.json`:**
   - Add `"@saf/web-shared": "workspace:*"` to dependencies
   - Remove `cytoscape`, `cytoscape-dagre`, `dagre`, `@types/cytoscape`, `@types/cytoscape-dagre` from dependencies (now in shared)

2. **Update `tutorials/tsconfig.app.json`:**
   - Add to compilerOptions: `"paths": { "@saf/web-shared/*": ["../packages/shared/src/*"] }`

3. **Delete these directories:**
   - `tutorials/src/types/` (entire directory)
   - `tutorials/src/graph/` (entire directory)

4. **Update imports in all tutorials components:**

   Replace these import patterns:
   - `from '../types/property-graph'` → `from '@saf/web-shared/types'`
   - `from '../types/air'` → `from '@saf/web-shared/types'`
   - `from '../graph/cytoscape-config'` → `from '@saf/web-shared/graph'`
   - `from '../graph/cfg-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/callgraph-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/defuse-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/valueflow-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/pta-renderer'` → `from '@saf/web-shared/graph'`
   - `from '../graph/air-index'` → `from '@saf/web-shared/graph'`
   - `from '../content/types'` stays as-is (TutorialStep/TutorialMeta are tutorial-specific)

   Files to check: `components/GraphViewer.tsx`, `components/StepContent.tsx`, `components/TutorialPage.tsx`

### Part 2: Add interactive analysis

5. **Update `tutorials/vite.config.ts`:**
   - Add `optimizeDeps: { exclude: ['web-tree-sitter'] }`

6. **Create `tutorials/src/components/InteractiveStep.tsx`:**

   An editable C code editor + "Analyze" button + graph viewer that can show live results.

   Props: `{ step: TutorialStep, staticGraph: PropertyGraph | null, tutorialId: string }`

   State:
   - `code` (editable string, init from step.code)
   - `status` ('idle' | 'compiling' | 'parsing' | 'converting' | 'analyzing' | 'ready' | 'error')
   - `liveGraph` (PropertyGraph | null from analysis)
   - `error` (string | null)

   Layout: two-column grid matching existing `.step-content` CSS.
   - Left: editable CodeMirror (C syntax, oneDark theme) + toolbar with Analyze/Reset buttons + status text
   - Right: GraphViewer showing `liveGraph ?? staticGraph`

   Analyze handler uses dynamic `import('@saf/web-shared/analysis')` to lazy-load:
   ```
   compileToLLVM(code) → initParser() + parseLLVMIR(ir) → convertToAIR(tree) → initWasm() + runAnalysis(airJson)
   ```
   Then extract `results[step.graphType]` as the liveGraph.

   Reset: restore code to `step.code`, clear liveGraph → reverts to staticGraph.

   Keyboard: Cmd+Enter / Ctrl+Enter triggers Analyze.

7. **Create `tutorials/src/components/InteractiveStep.css`:**
   - Styles for the toolbar (analyze/reset buttons), status indicator, editable editor chrome
   - Match existing tutorials dark theme (`--bg`, `--surface`, `--accent` vars)

8. **Modify `tutorials/src/components/StepContent.tsx`:**

   For steps with `codeLanguage === 'c'` AND `graphType`, render `InteractiveStep` instead of separate `CodeBlock` + `GraphViewer`:
   ```tsx
   {step.code && step.codeLanguage === 'c' && step.graphType ? (
     <InteractiveStep step={step} staticGraph={graphData} tutorialId={tutorialId} />
   ) : (
     // existing layout: CodeBlock (read-only) + GraphViewer (static)
   )}
   ```

9. **Verify:** TypeScript compiles without errors (`npx tsc --noEmit`)

---

## Verification (Leader)

1. `npm install` at root — all workspace deps resolve
2. `cd playground && npm run build` — passes
3. `cd tutorials && npm run build` — passes
4. `cd site && npm run build` — passes
5. Playwright: tutorials static graphs still render
6. Playwright: edit C code → Analyze → live graph appears
7. Playwright: Reset → static graph restores
8. Playwright: Python/Bash steps have no Analyze button
9. Playwright: playground analysis still works end-to-end
