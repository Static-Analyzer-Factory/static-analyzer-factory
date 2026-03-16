# Plan 113: Landing Page, Docs Site & Interactive Tutorials

**Epic:** New — Web Presence
**Status:** approved
**Created:** 2026-02-15
**Design doc:** `docs/plans/2026-02-15-landing-docs-interactive-design.md`

## Overview

Build a complete web presence for SAF: animated landing page at `/`, move existing playground to `/playground/` with embed mode, mdBook docs at `/docs/`, interactive step-based tutorials, and Pyodide Python analyzer authoring. All on GitHub Pages.

## Milestones

- **M1 (Tasks 1-3):** Playground embed mode + URL migration + CI restructure
- **M2 (Tasks 4-5):** Landing page with Motion animations
- **M3 (Tasks 6-8):** mdBook docs site with content
- **M4 (Tasks 9-11):** Interactive step-based tutorials
- **M5 (Tasks 12-14):** Pyodide Python analyzer authoring
- **M6 (Task 15):** Polish — CLAUDE.md maintenance rules, final cleanup

## Verification Gates

After M1: Playground works at `/playground/`, `?embed=true&graph=cfg` renders graph-only view
After M2: Landing page renders at `/`, links to playground and docs work
After M3: `mdbook build` succeeds, docs render at `/docs/`
After M4: `?tutorial=uaf&step=1` loads interactive tutorial, step navigation works
After M5: Pyodide loads in Analyzer tab, Python script executes and reports findings
After M6: `make fmt && npm run build` (both site/ and playground/) pass

## Agent Team Structure

**Leader:** Coordinates, runs verification gates, handles CI workflow updates (Tasks 3, 8, 15)
**Agent A (playground):** Tasks 1, 2, 9, 10, 11 — playground embed mode + tutorials
**Agent B (landing):** Tasks 4, 5 — landing page scaffold + implementation
**Agent C (docs):** Tasks 6, 7 — mdBook setup + content
**Agent D (pyodide):** Tasks 12, 13, 14 — Pyodide bridge + analyzer panel

Parallelism: After M1 (sequential), Agents B/C/D can run in parallel for M2/M3/M5. Agent A does M4 after M1.

---

## Task 1: Add example slugs to playground

**Agent type:** general-purpose
**Depends on:** Nothing
**Files to modify:** `playground/src/examples/index.ts`

Add a `slug` field to the `Example` interface and each example object. Slugs are URL-safe identifiers used by embed mode and tutorials.

### Changes

In `playground/src/examples/index.ts`:

1. Add `slug: string` to the `Example` interface (after `name`).

2. Add slugs to each example object:
   - `pointerAlias` → `slug: 'pointer_alias'`
   - `indirectCall` → `slug: 'indirect_call'`
   - `structField` → `slug: 'struct_field'`
   - `taintFlow` → `slug: 'taint_flow'`
   - `complexCFG` → `slug: 'complex_cfg'`
   - `libraryModeling` → `slug: 'library_modeling'`

### Verify

Run: `cd playground && npx tsc --noEmit`
Expected: No type errors.

### Commit

```
feat(playground): add URL slugs to example programs
```

---

## Task 2: Add embed mode and URL parameter parsing to App.tsx

**Agent type:** general-purpose
**Depends on:** Task 1
**Files to modify:** `playground/src/App.tsx`, `playground/src/App.css`

Add URL parameter parsing so the playground supports embed/widget modes.

### Changes to App.tsx

1. Add a `useUrlParams` function at module scope (before the `App` component):

```typescript
function useUrlParams() {
  const params = new URLSearchParams(window.location.search);
  return {
    embed: params.get('embed') === 'true',
    split: params.get('split') === 'true',
    example: params.get('example'),
    graph: params.get('graph') as GraphType | null,
    tutorial: params.get('tutorial'),
    step: params.has('step') ? parseInt(params.get('step')!, 10) : null,
  };
}
```

2. At top of `App` component, call `const urlParams = useUrlParams();`

3. Initialize `activeGraph` from URL: `useState<GraphType>(urlParams.graph || 'cfg')`

4. Add a `useEffect` to auto-load example from URL on mount:
```typescript
useEffect(() => {
  if (urlParams.example) {
    const idx = examples.findIndex(e => e.slug === urlParams.example);
    if (idx >= 0) {
      handleExampleSelect(idx);
      // Auto-trigger analysis after a tick
      setTimeout(() => handleAnalyze(), 100);
    }
  }
}, []); // mount only
```

5. In the return JSX, conditionally render based on `urlParams.embed`:
   - If `embed && !split`: render ONLY `<GraphPanel>` (no header, config, status bar, source panel)
   - If `embed && split`: render `<SourcePanel>` + `<GraphPanel>` in a two-column grid
   - If not embed: render everything as-is (current behavior)

6. Add `className` logic: `<div className={['app', urlParams.embed && 'embed', urlParams.embed && urlParams.split && 'split'].filter(Boolean).join(' ')}>`

### Changes to App.css

Add at the end of the file:

```css
/* Embed mode */
.app.embed .panels {
  grid-template-columns: 1fr;
}

.app.embed.split .panels {
  grid-template-columns: 1fr 1fr;
}
```

### Verify

Run: `cd playground && npm run dev`
- Navigate to `http://localhost:5173/?embed=true&example=taint_flow&graph=cfg` → should show only CFG graph
- Navigate to `http://localhost:5173/?embed=true&split=true&example=pointer_alias&graph=pta` → source + PTA graph
- Navigate to `http://localhost:5173/` → normal playground (no regression)

### Commit

```
feat(playground): add embed mode and URL parameter parsing
```

---

## Task 3: Restructure CI for multi-app deployment

**Agent type:** general-purpose (leader task)
**Depends on:** Task 1, 2
**Files to modify:** `.github/workflows/playground.yml`, `playground/vite.config.ts`

### Changes to playground/vite.config.ts

Change the base path for GitHub Pages to include `/playground/`:

```typescript
base: process.env.GITHUB_PAGES ? '/static-analyzer-lib/playground/' : './',
```

### Changes to .github/workflows/playground.yml

1. Rename workflow from `Playground` to `Site`.

2. Add path triggers for `site/**` and `docs/book/**`.

3. After the playground build steps, add a site assembly step:

```yaml
      # Assemble site directory
      - name: Assemble site directory
        run: |
          mkdir -p _site/playground
          cp -r playground/dist/* _site/playground/
```

4. Change `publish_dir` from `./playground/dist` to `./_site`.

Later tasks will add landing page and docs builds to this same workflow.

### Verify

The workflow YAML should be valid. Confirm by checking indentation and syntax.
`cd playground && GITHUB_PAGES=1 npx vite build` should succeed with the new base path.

### Commit

```
ci: restructure deployment for multi-app site (/playground/ path)
```

---

## Task 4: Scaffold the landing page Vite app

**Agent type:** general-purpose
**Depends on:** Nothing (can start in parallel with Tasks 1-3)
**Files to create:** `site/package.json`, `site/vite.config.ts`, `site/tsconfig.json`, `site/tsconfig.app.json`, `site/tsconfig.node.json`, `site/index.html`, `site/src/main.tsx`, `site/src/App.tsx`, `site/src/index.css`

Create a new Vite + React 19 app for the landing page at `site/`.

### site/package.json

```json
{
  "name": "saf-site",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "motion": "^12.0.0",
    "react": "^19.2.0",
    "react-dom": "^19.2.0"
  },
  "devDependencies": {
    "@types/react": "^19.2.7",
    "@types/react-dom": "^19.2.3",
    "@vitejs/plugin-react": "^5.1.1",
    "typescript": "~5.9.3",
    "vite": "^7.3.1"
  }
}
```

### site/vite.config.ts

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/static-analyzer-lib/' : './',
  build: { outDir: 'dist', target: 'esnext' },
});
```

### site/tsconfig.json

```json
{
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
```

### site/tsconfig.app.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true
  },
  "include": ["src"]
}
```

### site/tsconfig.node.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2023"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "noEmit": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

### site/index.html

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>SAF — Static Analyzer Factory</title>
    <meta name="description" content="Build program analysis tools. Understand code deeply. Browser-based static analysis powered by Rust + WebAssembly." />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

### site/src/main.tsx

```tsx
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

### site/src/App.tsx

Placeholder — will be replaced in Task 5:
```tsx
export default function App() {
  return <div>SAF — under construction</div>;
}
```

### site/src/index.css

Dark theme base:
```css
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  color: #e0e0e0;
  background: #0a0a1a;
  scroll-behavior: smooth;
}
body { min-height: 100vh; }
a { color: #7c3aed; text-decoration: none; }
a:hover { text-decoration: underline; }
```

### Verify

Run: `cd site && npm install && npm run dev`
Expected: Page loads at localhost with "SAF — under construction".

### Commit

```
feat: scaffold landing page Vite app at site/
```

---

## Task 5: Build the landing page with Motion animations

**Agent type:** general-purpose
**Depends on:** Task 4
**Files to create/modify:** `site/src/App.tsx`, `site/src/App.css`, `site/src/components/Hero.tsx`, `site/src/components/Features.tsx`, `site/src/components/Personas.tsx`, `site/src/components/TechHighlights.tsx`, `site/src/components/Footer.tsx`

**IMPORTANT:** Use the `frontend-design` skill for this task. This is a creative design task.

### Design Requirements

**Color palette** (match playground):
- Background: `#0a0a1a` (deepest), `#0f0f23` (section alternate)
- Text: `#e0e0e0` primary, `#a0a0b0` secondary
- Accent: `#7c3aed` (purple), `#10b981` (green for CTAs)
- Panel: `#16213e`

**Sections (top to bottom):**

1. **Hero** — "Static Analyzer Factory" + "Build program analysis tools. Understand code deeply."
   - Animated CFG-like SVG graphic drawing itself (Motion path animation)
   - Two CTA buttons: "Try the Playground" (`/playground/`) and "Read the Docs" (`/docs/`)
   - Motion: headline slides up + fades in, CTAs stagger in, SVG draws

2. **Features** (3 columns, scroll-triggered reveal via `whileInView`):
   - "Visualize" — CFG/graph icon + text + `<iframe>` widget (`/playground/?embed=true&example=complex_cfg&graph=cfg`)
   - "Analyze" — PTA icon + text + `<iframe>` widget (`/playground/?embed=true&example=pointer_alias&graph=pta`)
   - "Build" — code icon + text + Python code snippet showing `saf.analyze()` API

3. **Personas** (3 cards with `whileHover` lift):
   - Students → tutorials, Researchers → playground, AI Devs → API docs

4. **Tech Highlights** — horizontal strip: "Browser-native", "Rust + WASM", "Deterministic", "Open Source"

5. **Footer** — GitHub link, "Built with Rust + React"

**Responsive:** Stacks to single column below 768px.

### Verify

Run: `cd site && npm run dev`
- Page loads with all 5 sections
- Scroll down → feature columns animate in
- Hover persona cards → lift effect
- CTAs link to `/playground/` and `/docs/` (404 is OK for now)
- `npm run build` succeeds

### Commit (can be multiple)

```
feat: landing page hero with Motion animations
feat: landing page features, personas, footer
```

---

## Task 6: Set up mdBook skeleton

**Agent type:** general-purpose
**Depends on:** Nothing (can start in parallel)
**Files to create:** `docs/book/book.toml`, `docs/book/src/SUMMARY.md`, `docs/book/src/introduction.md`, `docs/book/src/custom.css`

### docs/book/book.toml

```toml
[book]
authors = ["SAF Contributors"]
language = "en"
multilingual = false
src = "src"
title = "SAF Documentation"

[build]
build-dir = "build"

[output.html]
default-theme = "navy"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/lyk/static-analyzer-lib"
additional-css = ["src/custom.css"]
```

### docs/book/src/SUMMARY.md

```markdown
# Summary

[Introduction](introduction.md)

---

# Getting Started

- [Installation](getting-started/installation.md)
- [First Analysis](getting-started/first-analysis.md)
- [Playground Tour](getting-started/playground-tour.md)
- [Browser vs Full SAF](getting-started/browser-vs-full.md)

# Concepts

- [Analysis IR (AIR)](concepts/air.md)
- [Control Flow Graphs](concepts/cfg-icfg.md)
- [Call Graphs](concepts/callgraph.md)
- [Points-To Analysis](concepts/points-to.md)
- [Value Flow](concepts/value-flow.md)
- [Taint Analysis](concepts/taint-analysis.md)

# Tutorials

- [Memory Safety](tutorials/memory-safety.md)
  - [Use After Free (CWE-416)](tutorials/memory-safety/use-after-free.md)
  - [Double Free (CWE-415)](tutorials/memory-safety/double-free.md)
  - [Memory Leak (CWE-401)](tutorials/memory-safety/leak-detection.md)
- [Information Flow](tutorials/information-flow.md)
  - [Command Injection (CWE-78)](tutorials/information-flow/command-injection.md)
- [Buffer Overflow](tutorials/buffer-overflow.md)
  - [Taint-Based Detection (CWE-120)](tutorials/buffer-overflow/taint-detection.md)

# API Reference

- [Python SDK](api-reference/python-sdk.md)
- [CLI](api-reference/cli.md)
- [PropertyGraph Format](api-reference/property-graph.md)

# Embedding

- [SAF Widgets](embedding/widgets.md)
```

### docs/book/src/introduction.md

Write a concise introduction (~100 lines): what SAF is, who it's for (3 personas), what you can do (analyze, visualize, build custom analyzers), links to playground and key sections.

### docs/book/src/custom.css

```css
.saf-widget { border: 1px solid #333; border-radius: 8px; overflow: hidden; margin: 1em 0; }
.saf-widget iframe { width: 100%; height: 400px; border: none; }
```

### Verify

```bash
# Install mdbook if not present
which mdbook || cargo install mdbook
cd docs/book && mdbook build
```

Expected: Build succeeds (may warn about missing linked pages — that's OK, content comes in Task 7).

### Commit

```
feat: set up mdBook documentation skeleton
```

---

## Task 7: Write documentation content

**Agent type:** general-purpose
**Depends on:** Task 6
**Files to create:** All `.md` files referenced in `SUMMARY.md` that don't exist yet

This is a content-writing task. Create all the markdown files listed in `docs/book/src/SUMMARY.md`.

### Content Sources

Read these files for content to adapt:
- `tutorials/README.md` — learning paths, CWE cross-reference
- `docs/static_analyzer_factory_srs.md` — architecture, API specification
- `tutorials/memory-safety/02-use-after-free/README.md` — UAF tutorial
- `tutorials/memory-safety/03-double-free/README.md` — double-free tutorial
- `tutorials/memory-safety/01-leak-detection/README.md` — leak tutorial
- `tutorials/information-flow/01-command-injection/README.md` — CWE-78 tutorial
- `tutorials/buffer-overflow/01-taint-detection/README.md` — CWE-120 tutorial

### Content Guidelines

- **getting-started/installation.md**: Docker setup, `make shell`, Python SDK install with `uv`
- **getting-started/first-analysis.md**: Hello world with SAF Python SDK
- **getting-started/playground-tour.md**: How to use the browser playground (screenshots not needed — describe the UI)
- **getting-started/browser-vs-full.md**: Feature comparison matrix table (Z3, LLVM, large programs, CS-PTA)
- **concepts/*.md**: Each page explains ONE concept with an embedded widget iframe:
  ```html
  <div class="saf-widget">
  <iframe src="/static-analyzer-lib/playground/?embed=true&split=true&example=SLUG&graph=TYPE"></iframe>
  </div>
  ```
- **tutorials/*.md**: Adapt from existing tutorial READMEs. Include vulnerable C code, explanation, detection approach, and "Try it interactively" link
- **api-reference/python-sdk.md**: Document key classes/functions from `saf-python` (AnalysisContext, Graph, PropertyGraph export)
- **api-reference/cli.md**: CLI usage from `saf-cli`
- **api-reference/property-graph.md**: Document the PropertyGraph JSON schema (nodes, edges, types)
- **embedding/widgets.md**: Document all URL parameters (`embed`, `split`, `example`, `graph`), sizing recommendations, iframe examples

### Verify

```bash
cd docs/book && mdbook build
```

Expected: Build succeeds with no broken link warnings.

### Commit (2-3 commits)

```
docs: getting-started and concepts sections
docs: tutorials adapted from existing content
docs: API reference and embedding guide
```

---

## Task 8: Add landing page + mdBook builds to CI

**Agent type:** general-purpose (leader task)
**Depends on:** Tasks 3, 5, 7

### Changes to .github/workflows/playground.yml (now the Site workflow)

Add these steps after the playground build:

```yaml
      # Build landing page
      - name: Install landing page deps
        run: cd site && npm ci

      - name: Build landing page
        run: cd site && npm run build
        env:
          GITHUB_PAGES: '1'

      # Build documentation
      - name: Install mdBook
        run: |
          mkdir -p $HOME/.local/bin
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v0.4.40/mdbook-v0.4.40-x86_64-unknown-linux-gnu.tar.gz | tar xz -C $HOME/.local/bin
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Build docs
        run: cd docs/book && mdbook build
```

Update the site assembly step:

```yaml
      - name: Assemble site directory
        run: |
          mkdir -p _site/playground _site/docs
          cp -r site/dist/* _site/
          cp -r playground/dist/* _site/playground/
          cp -r docs/book/build/* _site/docs/
```

### Verify

YAML syntax is valid. All three apps build independently.

### Commit

```
ci: add landing page + mdBook docs builds to site workflow
```

---

## Task 9: Define tutorial data model and create first tutorial

**Agent type:** general-purpose
**Depends on:** Task 2 (embed mode must exist)
**Files to create:** `playground/src/tutorials/types.ts`, `playground/src/tutorials/loader.ts`, `playground/public/tutorials/index.json`, `playground/public/tutorials/uaf-detection.json`

### playground/src/tutorials/types.ts

```typescript
export interface TutorialStep {
  title: string;
  text: string;
  code?: string;
  graph: 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';
  highlightLines?: number[];
  prompt?: string;
}

export interface Tutorial {
  id: string;
  title: string;
  description: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  steps: TutorialStep[];
}

export interface TutorialIndexEntry {
  id: string;
  title: string;
  difficulty: string;
}
```

### playground/src/tutorials/loader.ts

```typescript
import type { Tutorial, TutorialIndexEntry } from './types';

const BASE = import.meta.env.BASE_URL;

export async function loadTutorial(id: string): Promise<Tutorial> {
  const res = await fetch(`${BASE}tutorials/${id}.json`);
  if (!res.ok) throw new Error(`Tutorial not found: ${id}`);
  return res.json();
}

export async function loadTutorialIndex(): Promise<TutorialIndexEntry[]> {
  const res = await fetch(`${BASE}tutorials/index.json`);
  if (!res.ok) throw new Error('Tutorial index not found');
  return res.json();
}
```

### playground/public/tutorials/index.json

```json
[
  { "id": "uaf-detection", "title": "Detecting Use-After-Free (CWE-416)", "difficulty": "beginner" }
]
```

### playground/public/tutorials/uaf-detection.json

A 5-step tutorial. Read `playground/src/examples/index.ts` (the `libraryModeling` example) for the UAF C code reference. Each step should have:
1. "The Vulnerable Program" — show the code, graph=cfg, explain the UAF
2. "Control Flow" — same code, graph=cfg, explain which blocks matter
3. "Value Flow" — switch to graph=valueflow, explain the malloc→free→use chain
4. "Points-To Analysis" — graph=pta, explain what `p`/`copy` point to
5. "Finding the Bug" — graph=valueflow, explain how the use-after-free path is visible

Each step must have `title`, `text` (2-4 sentences), `graph`, and optionally `prompt` ("Try editing...").

### Verify

`cd playground && npx tsc --noEmit` — no type errors.
Verify JSON files are valid: `python3 -c "import json; json.load(open('playground/public/tutorials/uaf-detection.json'))"`

### Commit

```
feat(playground): tutorial data model and UAF detection tutorial
```

---

## Task 10: Build TutorialPanel UI component

**Agent type:** general-purpose
**Depends on:** Task 9
**Files to create:** `playground/src/components/TutorialPanel.tsx`, `playground/src/components/TutorialPanel.css`

### TutorialPanel.tsx

Props:
```typescript
interface TutorialPanelProps {
  tutorial: Tutorial;
  currentStep: number;
  onStepChange: (step: number) => void;
  onExit: () => void;
}
```

Renders:
- Tutorial title + "Exit" button (top-right)
- Step counter: "Step N of M"
- Step title (bold)
- Step text (plain text, not markdown)
- Step prompt in a highlighted box (if present)
- "Previous" / "Next Step" buttons (disabled at boundaries)

### TutorialPanel.css

- Fixed height ~160px panel
- Dark theme matching playground (#16213e background, #e0e0e0 text)
- Step counter in muted text (#718096)
- Prompt box with left border accent (#7c3aed)
- Nav buttons styled like existing `.btn-primary`

### Verify

Import the component in a test file and verify it renders without errors.
`cd playground && npx tsc --noEmit`

### Commit

```
feat(playground): TutorialPanel step-based navigation component
```

---

## Task 11: Integrate tutorial mode into App.tsx

**Agent type:** general-purpose
**Depends on:** Tasks 2, 10
**Files to modify:** `playground/src/App.tsx`, `playground/src/App.css`

### Changes

1. Import `TutorialPanel`, `loadTutorial`, `loadTutorialIndex`, and `Tutorial` type.

2. Add tutorial state:
```typescript
const [tutorial, setTutorial] = useState<Tutorial | null>(null);
const [tutorialStep, setTutorialStep] = useState(0);
```

3. Add `useEffect` to load tutorial from URL params:
```typescript
useEffect(() => {
  if (urlParams.tutorial) {
    loadTutorial(urlParams.tutorial).then(t => {
      setTutorial(t);
      setTutorialStep(urlParams.step ?? 0);
    }).catch(console.error);
  }
}, []);
```

4. Add step change handler that:
   - Updates `tutorialStep` state
   - If step has `code`, sets `sourceCode` and triggers analysis
   - If step has `graph`, sets `activeGraph`
   - Updates URL: `history.replaceState(null, '', ?tutorial=${id}&step=${newStep})`

5. In the JSX, when `tutorial` is set:
   - Render `<TutorialPanel>` between the header and panels
   - Hide `<ExamplesMenu>`, `<ConfigPanel>`, specs tab
   - On initial load, set source from step 0's `code` and auto-analyze

6. Add an exit handler that clears tutorial state and resets URL.

### CSS

Add tutorial layout styles:
```css
.tutorial-bar {
  flex-shrink: 0;
}
```

### Verify

Run: `cd playground && npm run dev`
Navigate to `?tutorial=uaf-detection` → tutorial loads, step 1 shows, graph renders.
Click "Next Step" → advances, graph type may change.
Click "Exit" → returns to normal playground.

### Commit

```
feat(playground): integrate tutorial mode with step navigation
```

---

## Task 12: Pyodide loader and SAF bridge module

**Agent type:** general-purpose
**Depends on:** Nothing (can start in parallel)
**Files to create:** `playground/src/analysis/pyodide-bridge.ts`

### pyodide-bridge.ts

Create a module that:

1. **Lazy-loads Pyodide** from CDN (`https://cdn.jsdelivr.net/pyodide/v0.27.5/full/pyodide.js`):
```typescript
let pyodideInstance: any = null;

export async function initPyodide(): Promise<any> {
  if (pyodideInstance) return pyodideInstance;
  const script = document.createElement('script');
  script.src = 'https://cdn.jsdelivr.net/pyodide/v0.27.5/full/pyodide.js';
  document.head.appendChild(script);
  await new Promise<void>((resolve, reject) => {
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Failed to load Pyodide'));
  });
  pyodideInstance = await (window as any).loadPyodide();
  return pyodideInstance;
}

export function isPyodideReady(): boolean {
  return pyodideInstance !== null;
}

export function preloadPyodide(): void {
  initPyodide().catch(() => {});
}
```

2. **Sets up the SAF bridge** — injects analysis results into Pyodide:
```typescript
export async function setupSafBridge(
  pyodide: any,
  results: AnalysisResults | null,
  onReport: (r: { nodeId: string; severity: string; message: string }) => void,
): Promise<void> {
  pyodide.registerJsModule('_saf_bridge', {
    get_cfg: () => results?.cfg ? JSON.stringify(results.cfg) : 'null',
    get_callgraph: () => results?.callgraph ? JSON.stringify(results.callgraph) : 'null',
    get_defuse: () => results?.defuse ? JSON.stringify(results.defuse) : 'null',
    get_valueflow: () => results?.valueflow ? JSON.stringify(results.valueflow) : 'null',
    get_pta: () => results?.pta ? JSON.stringify(results.pta) : 'null',
    report: (nodeId: string, severity: string, message: string) => {
      onReport({ nodeId, severity, message });
    },
  });
  await pyodide.runPythonAsync(SAF_PYTHON_MODULE);
}
```

3. **Defines `SAF_PYTHON_MODULE`** — a string constant containing the Python `saf` module with classes `PgNode`, `PgEdge`, `PropertyGraph`, `AnalysisResult`, and functions `analyze()`, `report()`. (See `docs/plans/2026-02-15-landing-docs-interactive-plan.md` Task 5.1 for the full Python source.)

4. **Runs user code**:
```typescript
export async function runUserScript(
  pyodide: any,
  code: string,
): Promise<{ stdout: string; error: string | null }> {
  pyodide.setStdout({ batched: (s: string) => { /* collect */ } });
  // ... execute and capture
}
```

Import `AnalysisResults` from `'../types/air'`.

### Verify

`cd playground && npx tsc --noEmit` — no type errors.

### Commit

```
feat(playground): Pyodide loader and SAF Python bridge module
```

---

## Task 13: Build AnalyzerPanel UI component

**Agent type:** general-purpose
**Depends on:** Task 12
**Files to create:** `playground/src/components/AnalyzerPanel.tsx`, `playground/src/components/AnalyzerPanel.css`
**Package to install:** `@codemirror/lang-python`

### AnalyzerPanel.tsx

Props:
```typescript
interface AnalyzerPanelProps {
  results: AnalysisResults | null;
}
```

Layout (3 sections, vertical):
- **Toolbar:** "Run" button, template selector `<select>`, Pyodide status (loading/ready)
- **Editor:** CodeMirror with Python syntax highlighting, pre-loaded with starter template
- **Output:** Two sub-tabs — "Console" (print output) and "Findings" (saf.report cards)

Behavior:
- On "Run": call `initPyodide()` (show spinner), `setupSafBridge()`, `runUserScript()`. Display results.
- Template selector: 3-4 starters (list all free calls, detect UAF, taint analysis, null deref)
- Findings render as colored cards: severity badge (high=red, medium=amber, info=blue) + message

### AnalyzerPanel.css

Dark theme. Editor fills available vertical space. Output pane has fixed ~200px height.

### Install dependency

```bash
cd playground && npm install @codemirror/lang-python
```

### Verify

`cd playground && npx tsc --noEmit`

### Commit

```
feat(playground): AnalyzerPanel with Python editor and output pane
```

---

## Task 14: Integrate Analyzer tab into App.tsx

**Agent type:** general-purpose
**Depends on:** Tasks 13, 2
**Files to modify:** `playground/src/App.tsx`

### Changes

1. Import `AnalyzerPanel` and `preloadPyodide`.

2. Extend `RightPanel` type: `'analysis' | 'specs' | 'analyzer'`

3. Add a third header tab button "Analyzer" alongside "Analysis" and "Specs".

4. On the "Analyzer" button, add `onMouseEnter={() => preloadPyodide()}` to pre-warm Pyodide.

5. When `rightPanel === 'analyzer'`, render `<AnalyzerPanel results={results} />`.

### Verify

Run: `cd playground && npm run dev`
1. Load an example, click "Analyze"
2. Switch to "Analyzer" tab
3. See Python editor with starter template
4. Click "Run" → Pyodide loads (spinner ~4s), script executes, output appears
5. Switch back to "Analysis" tab → graphs still there (no state loss)

### Commit

```
feat(playground): integrate Pyodide analyzer tab
```

---

## Task 15: CLAUDE.md maintenance rules + final cleanup

**Agent type:** general-purpose (leader task)
**Depends on:** All other tasks

### Changes to CLAUDE.md

Add a new `## Frontend & Docs Maintenance` section after `### Testing`:

```markdown
## Frontend & Docs Maintenance

### Site Structure
- Landing page: `site/` → deployed to `/` on GitHub Pages
- Playground: `playground/` → deployed to `/playground/`
- Documentation: `docs/book/` → deployed to `/docs/`
- Interactive tutorials: `playground/public/tutorials/*.json`
- CI workflow: `.github/workflows/playground.yml` builds all three apps

### When to Update
| Event | Required Update |
|-------|----------------|
| New analysis feature | Update relevant docs concept page |
| New graph type / export format | Update PropertyGraph docs + add embeddable example |
| Playground spec files changed | Sync `playground/public/specs/` with `saf-analysis` |
| New tutorial in `tutorials/` | Add mdBook version + interactive JSON |
| Python SDK API changed | Update Pyodide bridge in `playground/src/analysis/pyodide-bridge.ts` |
| WASM capability changed | Update `docs/book/src/getting-started/browser-vs-full.md` |
```

### Final Verification

- `cd playground && npm run build` succeeds
- `cd site && npm run build` succeeds
- `cd docs/book && mdbook build` succeeds

### Commit

```
docs: add frontend & docs maintenance rules to CLAUDE.md
```

---

## Agent Execution Summary

| Task | Agent | Parallel Group | Description |
|------|-------|---------------|-------------|
| 1 | A | Group 1 | Example slugs |
| 2 | A | Group 2 (after 1) | Embed mode + URL params |
| 3 | Leader | Group 3 (after 2) | CI restructure |
| 4 | B | Group 1 | Landing page scaffold |
| 5 | B | Group 2 (after 4) | Landing page Motion animations |
| 6 | C | Group 1 | mdBook skeleton |
| 7 | C | Group 2 (after 6) | Docs content |
| 8 | Leader | Group 4 (after 3,5,7) | CI: add landing + docs builds |
| 9 | A | Group 3 (after 2) | Tutorial data model |
| 10 | A | Group 4 (after 9) | TutorialPanel component |
| 11 | A | Group 5 (after 10) | Tutorial mode integration |
| 12 | D | Group 1 | Pyodide bridge |
| 13 | D | Group 2 (after 12) | AnalyzerPanel component |
| 14 | D | Group 3 (after 13,2) | Analyzer tab integration |
| 15 | Leader | Final (after all) | CLAUDE.md + cleanup |

**Maximum parallelism:** Group 1 runs A(1), B(4), C(6), D(12) simultaneously (4 agents).
