# Plan 114: Standalone Tutorials App

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a standalone React app at `/tutorials/` replacing the split mdBook + playground tutorial system with a unified hybrid experience.

**Architecture:** Vite + React 19 + TypeScript app in `tutorials/`. Loads tutorial content (markdown prose + pre-computed PropertyGraph JSON) from static files. Graph visualization via Cytoscape.js renderers copied from playground. Playground iframes for "try it yourself." CLI blocks for local-only features.

**Tech Stack:** React 19, TypeScript 5.9, Vite 7.3, Cytoscape.js + dagre, CodeMirror 6 (view-only), react-markdown + rehype-highlight + remark-gfm, gray-matter.

**Design doc:** `docs/plans/2026-02-15-tutorials-standalone-design.md`

---

## Execution Strategy: 3-Agent Team + Leader

### Phase 1 — Foundation (Agent A, solo)
Agent A scaffolds the app, copies graph renderers, and creates the content loading system. No other agents run during this phase.

### Phase 2 — Parallel Build (Agent B + Agent C)
- **Agent B** builds all React UI components.
- **Agent C** creates all tutorial content files.
These agents work on non-overlapping file sets and run in parallel.

### Phase 3 — Integration (Leader)
Leader wires routing, updates CI/Makefile, adds cross-app navigation, removes old docs tutorials, and runs comprehensive Playwright testing.

### File Ownership (no overlap)

| Agent | Owns |
|-------|------|
| A | `tutorials/package.json`, `tutorials/tsconfig*`, `tutorials/vite.config.ts`, `tutorials/index.html`, `tutorials/src/main.tsx`, `tutorials/src/vite-env.d.ts`, `tutorials/src/graph/*`, `tutorials/src/types/*`, `tutorials/src/content/*` |
| B | `tutorials/src/components/*`, `tutorials/src/App.css` |
| C | `tutorials/public/content/**/*` |
| Leader | `tutorials/src/App.tsx`, `.github/workflows/playground.yml`, `Makefile`, `CLAUDE.md`, `plans/PROGRESS.md`, `docs/book/src/SUMMARY.md`, `docs/book/src/tutorials/*`, `site/src/*` |

---

## Agent A: Foundation

### A1: Scaffold Vite + React + TypeScript App

Create the project skeleton. Match patterns from `site/` and `playground/`.

**Create: `tutorials/package.json`**
```json
{
  "name": "saf-tutorials",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "react-markdown": "^10.1.0",
    "rehype-highlight": "^7.0.2",
    "remark-gfm": "^4.0.1",
    "gray-matter": "^4.0.3",
    "cytoscape": "^3.33.1",
    "cytoscape-dagre": "^2.5.0",
    "dagre": "^0.8.5",
    "@codemirror/lang-cpp": "^6.0.3",
    "@codemirror/lang-python": "^6.2.1",
    "@codemirror/state": "^6.5.4",
    "@codemirror/theme-one-dark": "^6.1.3",
    "@codemirror/view": "^6.39.13",
    "codemirror": "^6.0.2"
  },
  "devDependencies": {
    "@types/cytoscape": "^3.21.9",
    "@types/cytoscape-dagre": "^2.3.4",
    "@types/react": "^19.2.7",
    "@types/react-dom": "^19.2.3",
    "@vitejs/plugin-react": "^5.1.1",
    "typescript": "~5.9.3",
    "vite": "^7.3.1"
  }
}
```

**Create: `tutorials/vite.config.ts`**
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/static-analyzer-lib/tutorials/' : './',
  build: { outDir: 'dist', target: 'esnext' },
});
```

**Create: `tutorials/index.html`** — standard React entry HTML (match `site/index.html` pattern). Title: "SAF Tutorials — Learn Static Analysis". Include Inter + JetBrains Mono font imports.

**Create: `tutorials/tsconfig.json`** and **`tutorials/tsconfig.node.json`** — match `playground/` patterns.

**Create: `tutorials/src/vite-env.d.ts`** — `/// <reference types="vite/client" />`

**Create: `tutorials/src/main.tsx`** — minimal React 19 entry: `createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>)`.

**Create: `tutorials/src/App.tsx`** — placeholder that renders `<h1>SAF Tutorials</h1>`. Leader will replace this later.

**Verify:** `cd tutorials && npm install && npm run build` succeeds.

**Commit:** `feat(tutorials): scaffold Vite + React + TypeScript app`

### A2: Copy Graph Renderers

Copy verbatim from playground. These are pure functions with no playground-specific dependencies.

**Copy from `playground/src/graph/` to `tutorials/src/graph/`:**
- `cytoscape-config.ts`
- `cfg-renderer.ts`
- `callgraph-renderer.ts`
- `defuse-renderer.ts`
- `valueflow-renderer.ts`
- `pta-renderer.ts`
- `air-index.ts`

**Copy from `playground/src/types/` to `tutorials/src/types/`:**
- `property-graph.ts`

**Verify:** `cd tutorials && npm run build` succeeds (no import errors).

**Commit:** `feat(tutorials): copy graph renderers from playground`

### A3: Content Types, Registry, and Loader

**Create: `tutorials/src/content/types.ts`**
```typescript
export type Difficulty = 'beginner' | 'intermediate' | 'advanced';
export type Mode = 'browser' | 'local' | 'both';
export type Category = 'getting-started' | 'memory-safety' | 'information-flow' | 'advanced';
export type GraphType = 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';

export interface TutorialMeta {
  id: string;
  title: string;
  description: string;
  difficulty: Difficulty;
  mode: Mode;
  category: Category;
  prerequisites?: string[];
}

export interface TutorialStep {
  title: string;
  content: string;           // Markdown prose
  code?: string;             // Code to display alongside
  codeLanguage?: 'c' | 'python' | 'bash';
  graphType?: GraphType;
  graphFile?: string;        // Filename in graphs/ directory
  playground?: string;       // Playground embed URL params
  localCmd?: string;         // CLI command for local execution
  localScript?: string;      // Filename of downloadable script
  highlightLines?: number[];
  challenge?: string;        // Interactive prompt
}

export interface Tutorial {
  meta: TutorialMeta;
  steps: TutorialStep[];
}
```

**Create: `tutorials/src/content/registry.ts`** — static arrays `CATEGORIES` (4 categories) and `TUTORIALS` (9 tutorial metadata entries). See design doc for the full list. Export both.

**Create: `tutorials/src/content/loader.ts`**
```typescript
import type { TutorialStep } from './types';

const BASE = import.meta.env.BASE_URL;

export async function loadTutorialSteps(tutorialId: string): Promise<TutorialStep[]> {
  const res = await fetch(`${BASE}content/${tutorialId}/steps.json`);
  if (!res.ok) throw new Error(`Tutorial not found: ${tutorialId}`);
  return res.json();
}

export async function loadGraphData(tutorialId: string, filename: string): Promise<unknown> {
  const res = await fetch(`${BASE}content/${tutorialId}/graphs/${filename}`);
  if (!res.ok) throw new Error(`Graph not found: ${tutorialId}/${filename}`);
  return res.json();
}
```

**Verify:** `cd tutorials && npm run build` succeeds.

**Commit:** `feat(tutorials): content types, registry, and loader`

---

## Agent B: UI Components

All files go in `tutorials/src/components/` and `tutorials/src/App.css`. Agent B reads types from `tutorials/src/content/types.ts` and `tutorials/src/types/property-graph.ts` (created by Agent A) but does not modify them.

### B1: GraphViewer Component

**Create: `tutorials/src/components/GraphViewer.tsx`**

A wrapper that takes a `PropertyGraph` JSON object and a `graphType` string, calls the appropriate renderer, and displays the result in a Cytoscape instance.

- Import renderers from `../graph/*`
- Create a `useEffect` that initializes Cytoscape on mount and destroys on unmount
- Switch on `graphType` to select renderer + layout options
- Container div with class `graph-viewer`, min-height 300px

Props: `{ graph: PropertyGraph; graphType: GraphType; className?: string }`

### B2: CodeBlock Component

**Create: `tutorials/src/components/CodeBlock.tsx`**

Read-only CodeMirror instance with syntax highlighting.

- Use `@codemirror/view` EditorView with `editable: false`, `readOnly: true`
- Use `@codemirror/lang-cpp` for C, `@codemirror/lang-python` for Python
- Use `@codemirror/theme-one-dark` for dark theme
- Optional `highlightLines` prop — use CodeMirror line decorations to highlight specific lines (yellow background)
- Container div with class `code-block`

Props: `{ code: string; language: 'c' | 'python' | 'bash'; highlightLines?: number[] }`

### B3: PlaygroundEmbed and LocalSection Components

**Create: `tutorials/src/components/PlaygroundEmbed.tsx`**

Simple iframe embed of the playground. Shows "Interactive Playground" header with "Open in Playground" external link.

- Compute full URL from relative params: `${BASE}../playground/${url}`
- iframe with `title="SAF Playground"`, sandbox permissions for scripts
- Container class `playground-embed`

Props: `{ url: string }`

**Create: `tutorials/src/components/LocalSection.tsx`**

Collapsible "Run Locally" section with CLI command (copy button) and download link.

- Toggle button with arrow indicator
- `<pre><code>` for CLI command with clipboard copy button
- Download link for script file
- Container class `local-section`

Props: `{ cmd?: string; script?: string; tutorialId?: string }`

### B4: StepContent Component

**Create: `tutorials/src/components/StepContent.tsx`**

Renders a single tutorial step: two-column layout with prose+code (left, scrollable) and graph (right, sticky).

- Left column: step title (h2), markdown prose via `react-markdown` with `remark-gfm` + `rehype-highlight`, CodeBlock if step has code, challenge box if step has challenge, PlaygroundEmbed if step has playground, LocalSection if step has localCmd/localScript
- Right column: GraphViewer if graphData is available, else placeholder
- Container class `step-content` with CSS grid layout

Props: `{ step: TutorialStep; graphData: PropertyGraph | null }`

### B5: Sidebar Component

**Create: `tutorials/src/components/Sidebar.tsx`** and **`tutorials/src/components/Sidebar.css`**

Persistent left sidebar showing all categories and tutorials. Expands to show steps when viewing a tutorial.

- Import `CATEGORIES` and `TUTORIALS` from registry
- Categories are collapsible sections
- Active tutorial's category auto-expands
- Steps shown as ordered list under active tutorial
- Current step highlighted with accent color
- Difficulty badges: beginner=`#10b981`, intermediate=`#f59e0b`, advanced=`#ef4444`
- Mobile (< 768px): hamburger toggle, overlay mode
- Desktop: fixed left, 260px wide

Props: `{ currentTutorialId: string | null; currentStep: number; steps: TutorialStep[]; onNavigate: (id: string | null, step?: number) => void; isOpen: boolean; onToggle: () => void }`

### B6: TutorialPage Component

**Create: `tutorials/src/components/TutorialPage.tsx`** and **`tutorials/src/components/TutorialPage.css`**

Orchestrates a single tutorial view: loads steps, manages step navigation, renders StepContent.

- Loads steps via `loadTutorialSteps(tutorialId)` on mount
- Loads graph data for current step via `loadGraphData()` when step changes
- Step navigation: prev/next buttons, keyboard arrows (Left/Right), progress bar
- Header: tutorial title, step counter "Step N of M", prev/next buttons
- Progress bar: colored fill proportional to `(step + 1) / steps.length`
- Calls `onStepsLoaded(steps)` callback so parent can pass steps to Sidebar

Props: `{ tutorialId: string; step: number; onNavigate: (id: string | null, step?: number) => void; onStepsLoaded: (steps: TutorialStep[]) => void }`

### B7: Catalog Component

**Create: `tutorials/src/components/Catalog.tsx`** and **`tutorials/src/components/Catalog.css`**

Tutorial hub / landing page.

- Hero section: "Learn Static Analysis" h1, subtitle, dark gradient background
- Learning path cards: horizontal row of category cards. Clicking filters the grid.
- Filter bar: difficulty dropdown (All / Beginner / Intermediate / Advanced)
- Tutorial grid: responsive card layout (3 cols desktop, 2 tablet, 1 mobile). Each card shows difficulty badge, mode badge (Browser / Local / Both), title, description. Clicking navigates to tutorial.

Props: `{ onNavigate: (tutorialId: string, step?: number) => void }`

### B8: NavBar Component

**Create: `tutorials/src/components/NavBar.tsx`** and **`tutorials/src/components/NavBar.css`**

Cross-app navigation bar. Fixed top, dark background, horizontal links.

Links: Home (`../`), Tutorials (current, active), Playground (`../playground/`), Docs (`../docs/`), GitHub (external).

Use `import.meta.env.BASE_URL` for relative path computation.

### B9: Global Styles

**Create: `tutorials/src/App.css`**

- CSS custom properties for theme: `--bg: #0f0f23`, `--surface: #1a1a2e`, `--text: #e0e0e0`, `--accent: #10b981`, `--accent-secondary: #7c3aed`
- Base layout: nav bar (fixed top, 48px) + sidebar (fixed left, 260px) + main content area
- Font: Inter for prose, JetBrains Mono for code
- Scrollbar styling (dark theme)
- Responsive breakpoints: 1200px (3-col grid), 768px (sidebar collapse), 480px (single col)
- Import `highlight.js/styles/github-dark.css` for code syntax highlighting in markdown

**Verify:** `cd tutorials && npm run build` succeeds.

**Commit after each component or batch:** e.g., `feat(tutorials): GraphViewer, CodeBlock, and embed components`, `feat(tutorials): Sidebar and TutorialPage components`, `feat(tutorials): Catalog, NavBar, and global styles`

---

## Agent C: Tutorial Content

All files go in `tutorials/public/content/`. Agent C creates `steps.json` files and placeholder graph JSON for each tutorial. Does not touch any TypeScript source.

### Content Format

Each tutorial is a directory:
```
tutorials/public/content/<id>/
  steps.json                    # Array of TutorialStep objects
  graphs/
    step-0-cfg.json             # Pre-computed PropertyGraph JSON
    step-2-valueflow.json       # (only for steps that have graphType)
  scripts/                      # (only for local tutorials)
    detect_uaf.py
```

### Placeholder Graph Template

For steps that reference a `graphType`, create a minimal valid PropertyGraph:
```json
{
  "schema_version": "0.1.0",
  "graph_type": "<type>",
  "metadata": {},
  "nodes": [
    {"id": "n0", "labels": ["Block", "Entry"], "properties": {"name": "entry", "function": "main"}},
    {"id": "n1", "labels": ["Block"], "properties": {"name": "bb1", "function": "main"}},
    {"id": "n2", "labels": ["Block"], "properties": {"name": "return", "function": "main"}}
  ],
  "edges": [
    {"src": "n0", "dst": "n1", "edge_type": "FLOWS_TO", "properties": {}},
    {"src": "n1", "dst": "n2", "edge_type": "FLOWS_TO", "properties": {}}
  ]
}
```

Adjust `graph_type`, `labels`, and `edge_type` per graph type:
- `cfg`: labels=`["Block"]`, edge_type=`"FLOWS_TO"`
- `callgraph`: labels=`["Function"]`, edge_type=`"CALLS"`, properties include `name`/`kind`
- `defuse`: labels=`["Value"]`/`["Instruction"]`, edge_type=`"DEFINES"`/`"USED_BY"`
- `valueflow`: properties include `kind` (Value/Location), edge_type=`"Direct"`/`"Store"`/`"Load"`
- `pta`: labels=`["Value"]`/`["Location"]`, edge_type=`"POINTS_TO"`

### C1: UAF Detection Tutorial (migrate from playground)

**Create: `tutorials/public/content/uaf-detection/steps.json`**

Migrate the 5 steps from `playground/public/tutorials/uaf-detection.json`. Convert `text` → `content` (use markdown formatting). Add `graphFile` references, `playground` embed URLs, and `localCmd` fields.

The C code is the same across all steps (the UAF example from the existing tutorial). Add `playground` URL params for each step: `?embed=true&split=true&example=use_after_free&graph=<type>`.

Add `localCmd` to each step:
```
docker compose run --rm dev sh -c 'python3 -c "import saf; r = saf.analyze(\"tests/fixtures/llvm/e2e/use_after_free.ll\"); print(r.export(\"<type>\"))"'
```

**Create graph files:** `step-0-cfg.json`, `step-1-cfg.json`, `step-2-valueflow.json`, `step-3-pta.json`, `step-4-valueflow.json` (use placeholder template, real graphs generated later).

### C2: Getting Started Tutorials

**Create: `tutorials/public/content/first-analysis/steps.json`** — 3 steps:
1. "What is SAF?" — markdown intro, no graph
2. "Loading a Program" — simple C program (hello world with pointer), show CFG
3. "Understanding the Output" — same program, show value flow

**Create: `tutorials/public/content/understanding-graphs/steps.json`** — 5 steps:
1. "Control Flow Graph" — explain blocks/edges, show CFG
2. "Call Graph" — explain function calls, show callgraph
3. "Def-Use Chains" — explain definitions/uses, show defuse
4. "Value Flow" — explain data movement, show valueflow
5. "Points-To Analysis" — explain pointer aliasing, show PTA

C code: use the "Pointer Aliasing" example from playground (`struct Point` with field assignments).

### C3: Memory Safety Tutorials

**Create: `tutorials/public/content/double-free/steps.json`** — 4 steps:
1. "The Vulnerable Program" — C code with double free via aliasing
2. "Tracking Aliased Pointers" — show PTA (both pointers alias same allocation)
3. "Value Flow Through Free" — show valueflow
4. "Detecting the Bug" — combine PTA + VF, add playground embed + local command

**Create: `tutorials/public/content/memory-leak/steps.json`** — 4 steps (local-only mode):
1. "Missing Deallocations" — C code with leaked malloc
2. "Allocation Tracking" — show valueflow from malloc
3. "Using Specs for Detection" — explain specs YAML, local command to run with specs
4. "Running the Detector" — full local CLI command + downloadable Python script

**Create: `tutorials/public/content/memory-leak/scripts/detect_leak.py`** — Python SDK script.

### C4: Information Flow Tutorials

**Create: `tutorials/public/content/taint-basics/steps.json`** — 4 steps:
1. "Sources and Sinks" — explain taint concepts, C code with `scanf`→`printf`
2. "Tracing Data Flow" — show valueflow from source to sink
3. "Taint Analysis" — explain SAF's taint_flow query, show valueflow
4. "Finding the Flow" — playground embed with taint_flow example

**Create: `tutorials/public/content/command-injection/steps.json`** — 4 steps:
1. "The Injection Vulnerability" — C code with `gets`→`system`
2. "Tracing User Input" — show valueflow from gets to system
3. "Specs for Library Functions" — explain how specs model gets() as taint source
4. "Automated Detection" — playground embed + local command

### C5: Advanced Tutorials

**Create: `tutorials/public/content/custom-analyzer/steps.json`** — 5 steps (local-only):
1. "Setting Up" — Docker install, make shell, Python SDK import
2. "Loading LLVM IR" — `saf.analyze()` API, loading a `.ll` file
3. "Querying Graphs" — export CFG, callgraph, valueflow; iterate nodes/edges
4. "Writing a Checker" — detect use-after-free pattern programmatically
5. "Running Your Analyzer" — full script, running in Docker, interpreting output

**Create: `tutorials/public/content/custom-analyzer/scripts/my_analyzer.py`** — complete Python script.

**Create: `tutorials/public/content/specs-authoring/steps.json`** — 4 steps (local-only):
1. "What Are Specs?" — explain YAML spec format, why they matter
2. "Modeling malloc/free" — YAML spec for heap allocation functions
3. "Modeling Taint Sources" — YAML spec for scanf/gets as taint sources
4. "Testing Your Specs" — run analysis with custom specs, verify detection

**Verify:** All `steps.json` files are valid JSON, graph files are valid PropertyGraph JSON.

**Commit per batch:** `feat(tutorials): UAF detection content`, `feat(tutorials): getting started content`, `feat(tutorials): memory safety content`, `feat(tutorials): information flow content`, `feat(tutorials): advanced tutorial content`

---

## Leader: Integration and Testing

After Agents A, B, C complete their tasks.

### L1: Wire App.tsx Routing

**Modify: `tutorials/src/App.tsx`**

Replace the placeholder with full routing:
- Hash-based routing: `#` = catalog, `#<tutorialId>/<step>` = tutorial page
- State: `tutorialId`, `step`, `steps`, `sidebarOpen`
- Layout: `<NavBar>` + `<Sidebar>` + main area (`<Catalog>` or `<TutorialPage>`)
- Navigation callback updates `window.location.hash`
- Listen to `hashchange` event

**Verify:** `cd tutorials && npm run dev` — catalog loads, clicking a tutorial navigates, steps work.

**Commit:** `feat(tutorials): wire routing in App.tsx`

### L2: Update CI Workflow

**Modify: `.github/workflows/playground.yml`**

Add to `paths` trigger:
```yaml
      - 'tutorials/**'
```

Add build steps after docs build:
```yaml
      - name: Install tutorials deps
        run: cd tutorials && npm ci

      - name: Build tutorials
        run: cd tutorials && npm run build
        env:
          GITHUB_PAGES: '1'
```

Update assemble step:
```yaml
      - name: Assemble site directory
        run: |
          mkdir -p _site/playground _site/docs _site/tutorials
          cp -r site/dist/* _site/
          cp -r playground/dist/* _site/playground/
          cp -r docs/book/build/* _site/docs/
          cp -r tutorials/dist/* _site/tutorials/
```

**Commit:** `ci: add tutorials app to build and deploy pipeline`

### L3: Update Makefile

**Modify: `Makefile`**

Add targets:
```makefile
tutorials: ## Build tutorials app for production
	cd tutorials && npm ci && GITHUB_PAGES=1 npm run build

tutorials-dev: ## Start tutorials dev server (http://localhost:5174)
	cd tutorials && npm install && npm run dev -- --port 5174
```

Update `site` target to include tutorials build + assembly.

Update `.PHONY` line to include `tutorials tutorials-dev`.

**Commit:** `chore: add tutorials Makefile targets`

### L4: Cross-App Navigation Updates

**Modify: `site/src/App.tsx`** — add a nav bar component (or import pattern) matching the tutorials NavBar, with links to all 4 apps. Add it above `<Hero>`.

**Verify:** Full site build (`make site`) — nav bar appears on landing page and tutorials, links work between apps.

**Commit:** `feat: cross-app navigation bar on landing page`

### L5: Remove Old Tutorials from Docs

**Delete:** `docs/book/src/tutorials/` (entire directory)

**Modify: `docs/book/src/SUMMARY.md`** — replace the Tutorials section:
```markdown
# Tutorials

- [Interactive Tutorials](/tutorials/)
```

**Verify:** `cd docs/book && mdbook build` succeeds.

**Commit:** `refactor: remove tutorials from mdBook, link to standalone app`

### L6: Playwright Visual Testing

Build the full site locally and run comprehensive Playwright tests.

**Build and serve:**
```bash
make site  # This builds all 4 apps and serves at localhost:8080
```

**Test the catalog page (`/tutorials/`):**
- Page loads without errors
- Hero section visible with "Learn Static Analysis" heading
- All 4 category cards rendered (Getting Started, Memory Safety, Information Flow, Advanced)
- All 9 tutorial cards rendered with correct difficulty badges and mode badges
- Difficulty filter works (selecting "Beginner" shows only beginner tutorials)
- Category filter works (clicking a category card filters the grid)
- Take screenshot of catalog page

**Test tutorial navigation:**
- Click "Detecting Use-After-Free" card — navigates to `#uaf-detection/0`
- Step 1 renders: title visible, markdown prose rendered, code block with syntax highlighting
- Graph visualization rendered in right column (Cytoscape canvas present)
- Click "Next" — advances to step 2, URL updates to `#uaf-detection/1`
- Click "Previous" — returns to step 1
- Keyboard: press ArrowRight → advances, ArrowLeft → returns
- Progress bar updates with each step
- Navigate through all 5 steps — no errors, graphs render

**Test sidebar:**
- Sidebar visible on left with all categories
- Current tutorial highlighted
- Steps listed under active tutorial
- Current step highlighted
- Click a different tutorial in sidebar — navigates to it
- Click "SAF Tutorials" header — returns to catalog

**Test all 9 tutorials load:**
- Navigate to each tutorial: `#first-analysis/0`, `#understanding-graphs/0`, `#uaf-detection/0`, `#double-free/0`, `#memory-leak/0`, `#taint-basics/0`, `#command-injection/0`, `#custom-analyzer/0`, `#specs-authoring/0`
- Each loads without console errors
- Steps render with content

**Test responsive behavior:**
- Resize to 768px width — sidebar collapses, hamburger menu appears
- Click hamburger — sidebar slides in as overlay
- Resize to 480px — single-column layout

**Test cross-app navigation:**
- Click "Playground" in nav bar — navigates to `/playground/`
- Click "Docs" — navigates to `/docs/`
- Click "Home" — navigates to landing page
- From landing page, click nav bar "Tutorials" — returns to tutorials

**Test playground embed (if any step has playground URL):**
- Playground iframe loads
- "Open in Playground" link opens in new tab

**Check console for errors:**
- No JavaScript errors
- No 404s for content files
- No CORS issues

**Take screenshots** of: catalog page, tutorial step with graph, sidebar expanded, mobile view.

### L7: Update CLAUDE.md and PROGRESS.md

**Modify: `CLAUDE.md`** — update "Site Structure" section:
```markdown
### Site Structure
- Landing page: `site/` → deployed to `/` on GitHub Pages
- Tutorials: `tutorials/` → deployed to `/tutorials/`
- Playground: `playground/` → deployed to `/playground/`
- Documentation: `docs/book/` → deployed to `/docs/`
- Tutorial content: `tutorials/public/content/` (steps.json + pre-computed graphs)
- CI workflow: `.github/workflows/playground.yml` builds all four apps
```

Add to "When to Update" table:
```markdown
| New tutorial needed | Add to `tutorials/public/content/<id>/` + update registry |
| Tutorial content outdated | Update `steps.json` + regenerate graphs if needed |
```

**Modify: `plans/PROGRESS.md`** — add Plan 114, update session log.

**Commit:** `docs: update CLAUDE.md and PROGRESS.md for tutorials app (Plan 114)`

---

## Task Dependencies (Gantt)

```
Phase 1:  [A1]─[A2]─[A3]
                         \
Phase 2:                  ├─[B1]─[B2]─[B3]─[B4]─[B5]─[B6]─[B7]─[B8]─[B9]
                          └─[C1]─[C2]─[C3]─[C4]─[C5]
                                                      \
Phase 3:                                               └─[L1]─[L2]─[L3]─[L4]─[L5]─[L6]─[L7]
```

Phase 2 agents (B, C) start after Agent A completes. Leader (L) starts after both B and C complete. L6 (Playwright testing) requires all prior tasks.
