# SAF Landing Page, Docs & Interactive Tutorials — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a complete web presence for SAF: animated landing page, embeddable playground widgets, mdBook docs site, interactive step-based tutorials, and Pyodide-powered Python analyzer authoring — all on GitHub Pages.

**Architecture:** The site is a monorepo multi-app deploy. The landing page is a new Vite React app at `site/` (deployed to `/`). The existing playground at `playground/` moves to the `/playground/` path. mdBook builds docs from `docs/book/` to `/docs/`. A unified GitHub Actions workflow builds all three and deploys them as one site.

**Tech Stack:** React 19, Vite 7, Motion (Framer Motion) for landing page animations, mdBook for docs, Pyodide for in-browser Python, CodeMirror 6, Cytoscape.js. All existing playground code is reused.

**Design doc:** `docs/plans/2026-02-15-landing-docs-interactive-design.md`

---

## Phase 1: Playground URL Migration + Embed Mode

Move the playground from `/` to `/playground/` and add URL parameter-driven embed modes. This is the foundation everything else builds on.

### Task 1.1: Update Vite config for `/playground/` base path

**Files:**
- Modify: `playground/vite.config.ts`

**Step 1: Update base path**

Change the `base` in `vite.config.ts` so it always serves from `/static-analyzer-lib/playground/` on GitHub Pages (and `./playground/` locally):

```typescript
// playground/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/static-analyzer-lib/playground/' : './',
  build: {
    outDir: 'dist',
    target: 'esnext',
  },
  optimizeDeps: {
    exclude: ['web-tree-sitter'],
  },
  worker: {
    format: 'es',
  },
});
```

**Step 2: Verify local dev still works**

Run: `cd playground && npm run dev`
Expected: playground loads at `http://localhost:5173/`

**Step 3: Commit**

```bash
git add playground/vite.config.ts
git commit -m "chore: move playground base path to /playground/ for site restructure"
```

---

### Task 1.2: Add URL parameter parsing and embed mode to App.tsx

**Files:**
- Modify: `playground/src/App.tsx`
- Modify: `playground/src/App.css`

**Step 1: Add URL parameter parsing**

Add a `useUrlParams` hook at the top of `App.tsx` that reads:
- `embed=true` → hide header, config panel, status bar
- `split=true` → show only source + graph (two-column layout)
- `example=<name>` → auto-load a specific example by slug
- `graph=<type>` → auto-select a graph type (cfg, callgraph, defuse, valueflow, pta)

```typescript
// Add near the top of App.tsx, inside the App component
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

**Step 2: Wire embed mode into rendering**

In the `App` component:
- Call `useUrlParams()` at the top
- If `embed=true`, don't render `<header>`, `<ConfigPanel>`, `<StatusBar>`, or the error banner
- If `split=true`, render only SourcePanel + GraphPanel in a two-column grid
- If `example` is set, find the matching example by slug and auto-load it on mount
- If `graph` is set, override the initial `activeGraph` state
- Auto-trigger analysis on mount when embed params are set

**Step 3: Add embed-mode CSS**

In `App.css`, add:
```css
/* Embed mode: full-bleed graph or split view */
.app.embed {
  height: 100vh;
}

.app.embed .panels {
  grid-template-columns: 1fr;
}

.app.embed.split .panels {
  grid-template-columns: 1fr 1fr;
}
```

**Step 4: Add example slugs**

In `playground/src/examples/index.ts`, add a `slug` field to the `Example` interface:
```typescript
export interface Example {
  name: string;
  slug: string;  // URL-safe identifier
  description: string;
  source: string;
}
```

And add slugs to each example: `pointer_alias`, `indirect_call`, `struct_field`, `taint_flow`, `complex_cfg`, `library_modeling`.

**Step 5: Test embed mode locally**

Navigate to: `http://localhost:5173/?embed=true&example=taint_flow&graph=cfg`
Expected: Only the CFG graph renders, no chrome.

Navigate to: `http://localhost:5173/?embed=true&split=true&example=pointer_alias&graph=pta`
Expected: Source on left, PTA graph on right, no chrome.

**Step 6: Commit**

```bash
git add playground/src/App.tsx playground/src/App.css playground/src/examples/index.ts
git commit -m "feat: add embed mode and URL parameter parsing to playground"
```

---

### Task 1.3: Update GitHub Actions to support multi-app deployment

**Files:**
- Modify: `.github/workflows/playground.yml`

**Step 1: Update the workflow to build playground into a subdirectory**

The workflow needs to:
1. Build the playground (as before) → output to `playground/dist/`
2. Assemble a final `_site/` directory: `_site/playground/` = `playground/dist/`
3. Deploy `_site/` to GitHub Pages

Later tasks will add landing page and docs builds to this workflow.

```yaml
name: Site

on:
  push:
    branches: [main]
    paths:
      - 'crates/saf-wasm/**'
      - 'crates/saf-core/**'
      - 'crates/saf-analysis/**'
      - 'playground/**'
      - 'site/**'
      - 'docs/book/**'
  pull_request:
    paths:
      - 'crates/saf-wasm/**'
      - 'crates/saf-core/**'
      - 'crates/saf-analysis/**'
      - 'playground/**'
      - 'site/**'
      - 'docs/book/**'

jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: ". -> target"

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM
        run: wasm-pack build crates/saf-wasm --target web --release --out-dir ../../playground/src/wasm

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: playground/package-lock.json

      # Build playground
      - name: Install playground dependencies
        run: cd playground && npm ci

      - name: Build playground
        run: cd playground && npm run build
        env:
          GITHUB_PAGES: '1'

      # Assemble site
      - name: Assemble site directory
        run: |
          mkdir -p _site/playground
          cp -r playground/dist/* _site/playground/

      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main' && github.event_name == 'push'
        uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./_site
```

**Step 2: Commit**

```bash
git add .github/workflows/playground.yml
git commit -m "ci: restructure deployment for multi-app site (playground at /playground/)"
```

---

## Phase 2: Landing Page with Motion Animations

### Task 2.1: Scaffold the landing page Vite app

**Files:**
- Create: `site/package.json`
- Create: `site/vite.config.ts`
- Create: `site/tsconfig.json`
- Create: `site/tsconfig.app.json`
- Create: `site/tsconfig.node.json`
- Create: `site/index.html`
- Create: `site/src/main.tsx`
- Create: `site/src/App.tsx`
- Create: `site/src/index.css`

**Step 1: Create `site/package.json`**

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

**Step 2: Create `site/vite.config.ts`**

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/static-analyzer-lib/' : './',
  build: {
    outDir: 'dist',
    target: 'esnext',
  },
});
```

**Step 3: Create TypeScript configs**

`site/tsconfig.json`:
```json
{
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
```

`site/tsconfig.app.json`:
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

`site/tsconfig.node.json`:
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

**Step 4: Create `site/index.html`**

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

**Step 5: Create minimal `site/src/main.tsx` and `site/src/App.tsx`**

`site/src/main.tsx`:
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

`site/src/App.tsx` — placeholder:
```tsx
export default function App() {
  return <div className="landing">SAF Landing Page — under construction</div>;
}
```

`site/src/index.css` — minimal dark theme base matching playground:
```css
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  color: #e0e0e0;
  background: #0a0a1a;
  scroll-behavior: smooth;
}

body {
  min-height: 100vh;
}

a {
  color: #7c3aed;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
```

**Step 6: Install dependencies and verify**

Run: `cd site && npm install && npm run dev`
Expected: Landing page loads at `http://localhost:5173/`

**Step 7: Commit**

```bash
git add site/
git commit -m "feat: scaffold landing page Vite app at site/"
```

---

### Task 2.2: Build the landing page with Motion animations

**Files:**
- Modify: `site/src/App.tsx`
- Create: `site/src/App.css`
- Create: `site/src/components/Hero.tsx`
- Create: `site/src/components/Features.tsx`
- Create: `site/src/components/Personas.tsx`
- Create: `site/src/components/TechHighlights.tsx`
- Create: `site/src/components/Footer.tsx`

This is a large creative task. Use the `frontend-design` skill for the actual implementation. The key requirements:

**Hero Section:**
- Large headline "Static Analyzer Factory" with tagline "Build program analysis tools. Understand code deeply."
- Animated hero graphic: a stylized CFG graph that draws itself using Motion path animations
- Two CTA buttons: "Try the Playground" (→ `/playground/`) and "Read the Docs" (→ `/docs/`)
- Motion animations: headline fades in and slides up, CTAs stagger in, graph draws itself

**Feature Showcase:**
- Three columns, each revealed on scroll via Motion `whileInView`
- Column 1 "Visualize": icon + text + embedded iframe widget (CFG)
- Column 2 "Analyze": icon + text + embedded iframe widget (PTA)
- Column 3 "Build": icon + text + Python code snippet with syntax highlighting
- Each column animates in from below with stagger

**Persona Cards:**
- Three cards with subtle hover lift animation (Motion `whileHover`)
- Students → tutorials link, Researchers → playground link, AI Developers → API docs link

**Technical Highlights:**
- Horizontal strip with 3-4 badges: "Browser-native", "Rust + WASM", "Deterministic", "Open Source"
- Each badge has an icon and short text
- Fade-in on scroll

**Footer:**
- GitHub link, license info, "Built with Rust + React"

**Design constraints:**
- Dark theme matching playground palette (#0a0a1a background, #7c3aed purple accent, #e0e0e0 text)
- Responsive: looks good on 1024px+ screens, gracefully stacks on mobile
- No external font CDNs — use system font stack with Inter as preferred
- All animations use Motion's `motion.div` with `initial`, `animate`, `whileInView`, `transition`

**Step N: Commit after each major section is complete**

```bash
git commit -m "feat: landing page hero section with Motion animations"
git commit -m "feat: landing page feature showcase with embedded widgets"
git commit -m "feat: landing page persona cards and tech highlights"
```

---

### Task 2.3: Add landing page to CI/CD workflow

**Files:**
- Modify: `.github/workflows/playground.yml`

**Step 1: Add landing page build steps**

Add after the playground build steps:

```yaml
      # Build landing page
      - name: Install landing page dependencies
        run: cd site && npm ci

      - name: Build landing page
        run: cd site && npm run build
        env:
          GITHUB_PAGES: '1'
```

**Step 2: Update site assembly**

Update the "Assemble site directory" step:

```yaml
      - name: Assemble site directory
        run: |
          mkdir -p _site/playground
          cp -r site/dist/* _site/
          cp -r playground/dist/* _site/playground/
```

The landing page goes at the root (`_site/`), the playground goes in `_site/playground/`.

**Step 3: Add `site/package-lock.json` to npm cache key**

Update the Node.js setup step to cache both:
```yaml
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
```

(Remove the single cache-dependency-path and let it auto-detect, or list both.)

**Step 4: Commit**

```bash
git add .github/workflows/playground.yml
git commit -m "ci: add landing page build to deployment workflow"
```

---

## Phase 3: mdBook Documentation Site

### Task 3.1: Set up mdBook skeleton

**Files:**
- Create: `docs/book/book.toml`
- Create: `docs/book/src/SUMMARY.md`
- Create: `docs/book/src/introduction.md`

**Step 1: Create `docs/book/book.toml`**

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

**Step 2: Create `docs/book/src/SUMMARY.md`**

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

- [Analysis Intermediate Representation](concepts/air.md)
- [Control Flow Graphs](concepts/cfg-icfg.md)
- [Call Graphs](concepts/callgraph.md)
- [Points-To Analysis](concepts/points-to.md)
- [Value Flow](concepts/value-flow.md)
- [Taint Analysis](concepts/taint-analysis.md)

# Tutorials

- [Memory Safety](tutorials/memory-safety.md)
  - [Leak Detection (CWE-401)](tutorials/memory-safety/leak-detection.md)
  - [Use After Free (CWE-416)](tutorials/memory-safety/use-after-free.md)
  - [Double Free (CWE-415)](tutorials/memory-safety/double-free.md)
- [Information Flow](tutorials/information-flow.md)
  - [Command Injection (CWE-78)](tutorials/information-flow/command-injection.md)
  - [Format String (CWE-134)](tutorials/information-flow/format-string.md)
- [Buffer Overflow](tutorials/buffer-overflow.md)
  - [Taint Detection (CWE-120)](tutorials/buffer-overflow/taint-detection.md)

# API Reference

- [Python SDK](api-reference/python-sdk.md)
- [CLI](api-reference/cli.md)
- [PropertyGraph Format](api-reference/property-graph.md)

# Embedding

- [Embedding SAF Widgets](embedding/widgets.md)
```

**Step 3: Create `docs/book/src/introduction.md`**

Write a concise introduction to SAF: what it is, who it's for, what you can do with it. Link to the playground and key tutorial sections.

**Step 4: Create custom CSS for dark theme**

`docs/book/src/custom.css`:
```css
/* Embed playground widgets in docs via iframe */
.saf-widget {
  border: 1px solid #333;
  border-radius: 8px;
  overflow: hidden;
  margin: 1em 0;
}

.saf-widget iframe {
  width: 100%;
  height: 400px;
  border: none;
}
```

**Step 5: Verify locally**

```bash
# Install mdbook if needed
cargo install mdbook
# Build docs
cd docs/book && mdbook build
# Serve locally
mdbook serve --open
```

**Step 6: Commit**

```bash
git add docs/book/
git commit -m "feat: set up mdBook skeleton for documentation site"
```

---

### Task 3.2: Write core documentation content

**Files:**
- Create: `docs/book/src/getting-started/installation.md`
- Create: `docs/book/src/getting-started/first-analysis.md`
- Create: `docs/book/src/getting-started/playground-tour.md`
- Create: `docs/book/src/getting-started/browser-vs-full.md`
- Create: `docs/book/src/concepts/air.md`
- Create: `docs/book/src/concepts/cfg-icfg.md`
- Create: `docs/book/src/concepts/callgraph.md`
- Create: `docs/book/src/concepts/points-to.md`
- Create: `docs/book/src/concepts/value-flow.md`
- Create: `docs/book/src/concepts/taint-analysis.md`
- Create: `docs/book/src/api-reference/python-sdk.md`
- Create: `docs/book/src/api-reference/cli.md`
- Create: `docs/book/src/api-reference/property-graph.md`
- Create: `docs/book/src/embedding/widgets.md`

**Content guidelines:**
- Each concept page should include 1-2 embedded playground widget iframes showing the concept on a real example
- Use the embed URLs from Phase 1: `<iframe src="/static-analyzer-lib/playground/?embed=true&split=true&example=<slug>&graph=<type>" />`
- `browser-vs-full.md` must contain the feature comparison matrix (Z3, LLVM frontend, etc.)
- `widgets.md` documents the iframe API (all URL parameters, sizing recommendations)
- Reference existing content from `tutorials/README.md`, `docs/static_analyzer_factory_srs.md`, and existing tutorial READMEs

**Step 1-6: Write each section, committing in logical groups**

```bash
git commit -m "docs: getting-started section (installation, first analysis, playground tour)"
git commit -m "docs: concept pages with embedded widgets (AIR, CFG, callgraph, PTA, VF, taint)"
git commit -m "docs: API reference (Python SDK, CLI, PropertyGraph format)"
git commit -m "docs: embedding guide for SAF widgets"
```

---

### Task 3.3: Migrate existing tutorials to mdBook format

**Files:**
- Create: `docs/book/src/tutorials/memory-safety.md` (index)
- Create: `docs/book/src/tutorials/memory-safety/leak-detection.md`
- Create: `docs/book/src/tutorials/memory-safety/use-after-free.md`
- Create: `docs/book/src/tutorials/memory-safety/double-free.md`
- Create: `docs/book/src/tutorials/information-flow.md` (index)
- Create: `docs/book/src/tutorials/information-flow/command-injection.md`
- Create: `docs/book/src/tutorials/information-flow/format-string.md`
- Create: `docs/book/src/tutorials/buffer-overflow.md` (index)
- Create: `docs/book/src/tutorials/buffer-overflow/taint-detection.md`

**Approach:**
- Read each tutorial's `README.md` and `detect.py` from `tutorials/`
- Adapt the content for mdBook (fix relative links, add code blocks, add iframe widgets)
- Each tutorial page should have:
  - Brief description of the vulnerability (CWE reference)
  - The vulnerable C code
  - Step-by-step explanation of how SAF detects it
  - An embedded playground widget showing the analysis
  - A "Try it locally" code block with the Python `detect.py` script
  - Link to the interactive tutorial version (Phase 4): "Try the interactive version →"

Start with the top 6 most impactful tutorials (3 memory-safety, 2 information-flow, 1 buffer-overflow). The rest can be added incrementally.

**Step: Commit**

```bash
git commit -m "docs: migrate memory-safety tutorials to mdBook"
git commit -m "docs: migrate information-flow and buffer-overflow tutorials to mdBook"
```

---

### Task 3.4: Add mdBook build to CI/CD

**Files:**
- Modify: `.github/workflows/playground.yml`

**Step 1: Add mdBook install and build steps**

```yaml
      # Build documentation
      - name: Install mdBook
        run: |
          mkdir -p $HOME/.local/bin
          curl -sSL https://github.com/rust-lang/mdBook/releases/download/v0.4.40/mdbook-v0.4.40-x86_64-unknown-linux-gnu.tar.gz | tar xz -C $HOME/.local/bin
          echo "$HOME/.local/bin" >> $GITHUB_PATH

      - name: Build docs
        run: cd docs/book && mdbook build
```

**Step 2: Update site assembly**

```yaml
      - name: Assemble site directory
        run: |
          mkdir -p _site/playground _site/docs
          cp -r site/dist/* _site/
          cp -r playground/dist/* _site/playground/
          cp -r docs/book/build/* _site/docs/
```

**Step 3: Commit**

```bash
git add .github/workflows/playground.yml
git commit -m "ci: add mdBook documentation build to deployment workflow"
```

---

## Phase 4: Interactive Step-Based Tutorials

### Task 4.1: Define tutorial data model and load system

**Files:**
- Create: `playground/src/tutorials/types.ts`
- Create: `playground/src/tutorials/loader.ts`
- Create: `playground/public/tutorials/uaf-detection.json`

**Step 1: Define the tutorial data model**

`playground/src/tutorials/types.ts`:
```typescript
export interface TutorialStep {
  title: string;
  text: string;              // Markdown-formatted explanation
  code?: string;             // C source to load (overrides current code if set)
  graph: 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta';
  highlightLines?: number[]; // Source lines to highlight
  prompt?: string;           // "Try this:" instruction for the user
}

export interface Tutorial {
  id: string;
  title: string;
  description: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
  steps: TutorialStep[];
}
```

**Step 2: Create the tutorial loader**

`playground/src/tutorials/loader.ts`:
```typescript
import type { Tutorial } from './types';

const BASE = import.meta.env.BASE_URL;

export async function loadTutorial(id: string): Promise<Tutorial> {
  const response = await fetch(`${BASE}tutorials/${id}.json`);
  if (!response.ok) throw new Error(`Tutorial not found: ${id}`);
  return response.json();
}

export async function loadTutorialIndex(): Promise<{ id: string; title: string; difficulty: string }[]> {
  const response = await fetch(`${BASE}tutorials/index.json`);
  if (!response.ok) throw new Error('Tutorial index not found');
  return response.json();
}
```

**Step 3: Create the first tutorial JSON**

`playground/public/tutorials/uaf-detection.json`: A 5-step tutorial on detecting use-after-free:
1. "The Vulnerable Program" — load a UAF C program, show CFG
2. "Understanding the Control Flow" — same program, highlight the free→use path in CFG
3. "Building the Value-Flow Graph" — switch to valueflow graph, explain edges
4. "Points-To Analysis" — switch to PTA, show what `p` points to
5. "Writing a Detector" — prompt user to look at the analysis results

`playground/public/tutorials/index.json`: Array of `{id, title, difficulty}` for all available tutorials.

**Step 4: Commit**

```bash
git add playground/src/tutorials/ playground/public/tutorials/
git commit -m "feat: tutorial data model and first tutorial (UAF detection)"
```

---

### Task 4.2: Build the TutorialPanel UI component

**Files:**
- Create: `playground/src/components/TutorialPanel.tsx`
- Create: `playground/src/components/TutorialPanel.css`

**Step 1: Build the tutorial panel**

`playground/src/components/TutorialPanel.tsx`:

A React component that:
- Accepts a `Tutorial` object and current step index as props
- Renders: step title, step counter ("Step 2 of 5"), markdown text, "Try this" prompt
- "Previous" and "Next Step" buttons that call `onStepChange(index)`
- Stores progress in `localStorage` keyed by tutorial ID
- Has a clean, readable design with the same dark theme

CSS: Fixed-height top panel (about 150px) above the main source+graph area, with text content that scrolls if too long.

**Step 2: Commit**

```bash
git add playground/src/components/TutorialPanel.tsx playground/src/components/TutorialPanel.css
git commit -m "feat: TutorialPanel UI component for step-based tutorials"
```

---

### Task 4.3: Integrate tutorial mode into App.tsx

**Files:**
- Modify: `playground/src/App.tsx`
- Modify: `playground/src/App.css`

**Step 1: Add tutorial state management**

When URL has `?tutorial=uaf-detection`:
- Load the tutorial JSON via `loadTutorial()`
- Track current step index in state
- On step change: update source code (if step has `code`), switch graph type, highlight lines
- Auto-trigger analysis when source changes from a step
- Render `TutorialPanel` above the main panels area
- Hide the examples menu, config, and specs tab (tutorial takes control)
- Update URL when step changes: `?tutorial=uaf-detection&step=2`

**Step 2: Add tutorial entry point**

Add a "Tutorials" button in the header (visible in non-tutorial mode) that shows a dropdown/modal with available tutorials. Clicking one navigates to `?tutorial=<id>`.

**Step 3: Test end-to-end**

Navigate to: `http://localhost:5173/?tutorial=uaf-detection`
Expected: Tutorial panel appears, step 1 loads, source code and graph display correctly.
Click "Next Step" → step 2 loads, graph type may change.

**Step 4: Commit**

```bash
git add playground/src/App.tsx playground/src/App.css
git commit -m "feat: integrate tutorial mode into playground app"
```

---

### Task 4.4: Create 3-5 initial tutorial content files

**Files:**
- Create: `playground/public/tutorials/uaf-detection.json` (already started in 4.1)
- Create: `playground/public/tutorials/taint-flow.json`
- Create: `playground/public/tutorials/pointer-aliasing.json`
- Create: `playground/public/tutorials/callgraph-resolution.json`
- Update: `playground/public/tutorials/index.json`

**Content for each tutorial:**

1. **UAF Detection** (5 steps) — CWE-416, walks through CFG → value-flow → PTA → detection
2. **Taint Flow Analysis** (4 steps) — source/sink pattern, value-flow edges, reporting
3. **Pointer Aliasing** (4 steps) — PTA basics, what "may-alias" means, visualizing points-to sets
4. **Call Graph Resolution** (4 steps) — direct vs indirect calls, how PTA resolves function pointers

Each tutorial reuses or extends the existing C examples from `playground/src/examples/`.

**Step: Commit**

```bash
git add playground/public/tutorials/
git commit -m "feat: add 4 interactive tutorials (UAF, taint, aliasing, callgraph)"
```

---

## Phase 5: Pyodide Python Analyzer Authoring

### Task 5.1: Add Pyodide loader and JS bridge

**Files:**
- Create: `playground/src/analysis/pyodide-bridge.ts`

**Step 1: Create the Pyodide loader**

`playground/src/analysis/pyodide-bridge.ts`:

```typescript
// Lazy-load Pyodide from CDN
let pyodideInstance: any = null;

export async function initPyodide(): Promise<any> {
  if (pyodideInstance) return pyodideInstance;

  // Dynamically load Pyodide from CDN
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
```

**Step 2: Create the bridge that injects SAF functions into Pyodide**

The bridge:
1. Takes the current `AnalysisResults` from the playground
2. Registers JS functions in Pyodide's global scope that return graph data
3. Loads a Python `saf` module (defined as a string) that wraps these JS functions

```typescript
export async function setupSafBridge(
  pyodide: any,
  analysisResults: AnalysisResults | null,
  reportCallback: (report: { nodeId: string; severity: string; message: string }) => void,
) {
  // Register the analysis results as accessible JS objects
  pyodide.registerJsModule('_saf_bridge', {
    get_cfg: () => JSON.stringify(analysisResults?.cfg || null),
    get_callgraph: () => JSON.stringify(analysisResults?.callgraph || null),
    get_defuse: () => JSON.stringify(analysisResults?.defuse || null),
    get_valueflow: () => JSON.stringify(analysisResults?.valueflow || null),
    get_pta: () => JSON.stringify(analysisResults?.pta || null),
    report: (nodeId: string, severity: string, message: string) => {
      reportCallback({ nodeId, severity, message });
    },
  });

  // Load the Python saf module wrapper
  await pyodide.runPythonAsync(SAF_PYTHON_MODULE);
}
```

Where `SAF_PYTHON_MODULE` is a string containing the Python `saf` module:

```python
"""SAF browser API — wraps JS bridge for Pyodide."""
import json
from _saf_bridge import get_cfg, get_callgraph, get_defuse, get_valueflow, get_pta, report as _report

class PgNode:
    def __init__(self, data):
        self.id = data["id"]
        self.labels = data.get("labels", [])
        self.properties = data.get("properties", {})
    def __repr__(self):
        return f"PgNode({self.id}, labels={self.labels})"

class PgEdge:
    def __init__(self, data):
        self.src = data["src"]
        self.dst = data["dst"]
        self.edge_type = data.get("edge_type", "")
        self.properties = data.get("properties", {})
    def __repr__(self):
        return f"PgEdge({self.src} -> {self.dst}, type={self.edge_type})"

class PropertyGraph:
    def __init__(self, raw_json):
        data = json.loads(raw_json) if isinstance(raw_json, str) else raw_json
        self.graph_type = data.get("graph_type", "")
        self.metadata = data.get("metadata", {})
        self._nodes = [PgNode(n) for n in data.get("nodes", [])]
        self._edges = [PgEdge(e) for e in data.get("edges", [])]

    def nodes(self):
        return list(self._nodes)

    def edges(self):
        return list(self._edges)

    def node_by_id(self, node_id):
        for n in self._nodes:
            if n.id == node_id:
                return n
        return None

class AnalysisResult:
    def cfg(self):
        raw = get_cfg()
        return PropertyGraph(raw) if raw else None

    def callgraph(self):
        raw = get_callgraph()
        return PropertyGraph(raw) if raw else None

    def defuse(self):
        raw = get_defuse()
        return PropertyGraph(raw) if raw else None

    def valueflow(self):
        raw = get_valueflow()
        return PropertyGraph(raw) if raw else None

    def points_to(self):
        raw = get_pta()
        return json.loads(raw) if raw else None

def analyze():
    return AnalysisResult()

def report(node_id, severity="info", message=""):
    _report(node_id, severity, message)
```

**Step 3: Commit**

```bash
git add playground/src/analysis/pyodide-bridge.ts
git commit -m "feat: Pyodide loader and saf Python bridge module"
```

---

### Task 5.2: Build the AnalyzerPanel UI component

**Files:**
- Create: `playground/src/components/AnalyzerPanel.tsx`
- Create: `playground/src/components/AnalyzerPanel.css`

**Step 1: Build the component**

`AnalyzerPanel.tsx` — a three-section panel:

**Top:** Toolbar with "Run" button, analyzer template selector dropdown, Pyodide status indicator (loading spinner / ready badge)

**Middle:** CodeMirror editor configured for Python syntax, pre-loaded with a starter template

**Bottom:** Output pane with two sub-tabs:
- "Findings" — rendered `saf.report()` calls as cards (severity badge + node ID + message)
- "Console" — captured `print()` output (stdout from Pyodide)

**Interaction flow:**
1. Component receives `analysisResults` as prop (from parent App)
2. On "Run" click: initialize Pyodide if needed (show spinner), set up bridge with current results, execute user's Python code, capture output
3. Findings appear in the Findings tab, console output in Console tab
4. Errors from Python execution show as red-highlighted output

**Step 2: Add starter templates**

Starter templates (selectable from dropdown):

```typescript
export const analyzerTemplates = [
  {
    name: 'Find all function calls',
    code: `import saf

result = saf.analyze()
cfg = result.cfg()

if cfg:
    for edge in cfg.edges():
        print(f"Flow: {edge.src} -> {edge.dst}")
    print(f"Total: {len(cfg.nodes())} blocks, {len(cfg.edges())} edges")
`,
  },
  {
    name: 'Detect use-after-free',
    code: `import saf

result = saf.analyze()
vf = result.valueflow()

if vf:
    # Find free-related nodes
    free_nodes = [n for n in vf.nodes() if 'free' in str(n.properties.get('label', '')).lower()]

    for fn in free_nodes:
        # Check for uses after the free
        for edge in vf.edges():
            if edge.src == fn.id:
                saf.report(edge.dst, severity="high",
                          message=f"Potential use-after-free: value flows from free at {fn.id}")

    if not free_nodes:
        print("No free() calls found in this program")
`,
  },
  // ... more templates
];
```

**Step 3: Commit**

```bash
git add playground/src/components/AnalyzerPanel.tsx playground/src/components/AnalyzerPanel.css
git commit -m "feat: AnalyzerPanel UI with Python editor, output pane, and starter templates"
```

---

### Task 5.3: Integrate AnalyzerPanel into App.tsx

**Files:**
- Modify: `playground/src/App.tsx`
- Modify: `playground/src/App.css`

**Step 1: Add "Analyzer" as a right-panel tab**

In `App.tsx`, extend the `RightPanel` type:
```typescript
type RightPanel = 'analysis' | 'specs' | 'analyzer';
```

Add a third header tab button "Analyzer" next to "Analysis" and "Specs".

When `rightPanel === 'analyzer'`, render `<AnalyzerPanel results={results} />` instead of `GraphPanel` or `SpecViewer`.

**Step 2: Install CodeMirror Python language support**

```bash
cd playground && npm install @codemirror/lang-python
```

**Step 3: Test end-to-end**

1. Load a C example, click "Analyze"
2. Switch to "Analyzer" tab
3. See the Python editor with a starter template
4. Click "Run" → Pyodide loads (spinner, ~4s), then script executes
5. See output in Console tab

**Step 4: Commit**

```bash
git add playground/src/App.tsx playground/src/App.css playground/package.json playground/package-lock.json
git commit -m "feat: integrate Pyodide analyzer tab into playground"
```

---

### Task 5.4: Add WASM limitation banners

**Files:**
- Modify: `playground/src/components/ConfigPanel.tsx`
- Create: `playground/src/components/WasmBanner.tsx`

**Step 1: Create the banner component**

`WasmBanner.tsx` — a subtle yellow/amber info banner:
```
⚠ [Feature name] requires [Z3/LLVM/etc.], which isn't available in the browser.
Results shown without [feature]. Install SAF locally for full analysis →
```

**Step 2: Show banners contextually**

In `ConfigPanel.tsx`, when certain settings are enabled, show the banner:
- If a future Z3-dependent toggle is added → show "Path feasibility requires Z3..."
- For now, add a general info line in the settings panel: "Some advanced features are desktop-only. See docs for details."

This is lightweight for now — the main value comes from the `browser-vs-full.md` docs page.

**Step 3: Commit**

```bash
git add playground/src/components/ConfigPanel.tsx playground/src/components/WasmBanner.tsx
git commit -m "feat: WASM limitation banners for desktop-only features"
```

---

## Phase 6: Polish & Maintenance

### Task 6.1: Update CLAUDE.md with frontend & docs maintenance rules

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add new section**

Add a "## Frontend & Docs Maintenance" section after "### Testing":

```markdown
## Frontend & Docs Maintenance

When any of these events happen, the corresponding updates are REQUIRED:

| Event | Required Update |
|-------|----------------|
| New analysis feature added | Update docs concept page + add playground example if applicable |
| New graph type or export format | Add embeddable example + update PropertyGraph docs + update API reference |
| New PTA/VF capability | Update `browser-vs-full.md` if WASM support differs from full build |
| Playground spec files changed | Ensure `playground/public/specs/` matches `saf-analysis` specs |
| New tutorial written in `tutorials/` | Add mdBook version in `docs/book/` AND interactive JSON in `playground/public/tutorials/` |
| Python SDK API changed | Update Pyodide `saf` bridge module in `playground/src/analysis/pyodide-bridge.ts` |
| Landing page feature claims | Verify claims match actual shipped capabilities |

### Site Structure
- Landing page: `site/` → deployed to `/`
- Playground: `playground/` → deployed to `/playground/`
- Documentation: `docs/book/` → deployed to `/docs/`
- Tutorials (interactive): `playground/public/tutorials/*.json`
- Tutorials (reference): `docs/book/src/tutorials/`
- CI workflow: `.github/workflows/playground.yml` (renamed to `site.yml` conceptually, builds all three)
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add frontend & docs maintenance rules to CLAUDE.md"
```

---

### Task 6.2: Add Pyodide pre-warming for performance

**Files:**
- Modify: `playground/src/analysis/pyodide-bridge.ts`

**Step 1: Pre-warm Pyodide when the Analyzer tab is hovered**

Add a `preloadPyodide()` function that starts loading (but doesn't block) when the user hovers over the "Analyzer" tab. This reduces the perceived 4-5s cold start.

```typescript
export function preloadPyodide(): void {
  // Start loading in background, don't await
  initPyodide().catch(() => {
    // Silently ignore preload failures
  });
}
```

Wire this to an `onMouseEnter` handler on the Analyzer tab button in `App.tsx`.

**Step 2: Commit**

```bash
git add playground/src/analysis/pyodide-bridge.ts playground/src/App.tsx
git commit -m "perf: pre-warm Pyodide on Analyzer tab hover"
```

---

### Task 6.3: Rename workflow file and final cleanup

**Files:**
- Rename: `.github/workflows/playground.yml` → `.github/workflows/site.yml`

**Step 1: Rename the workflow**

```bash
git mv .github/workflows/playground.yml .github/workflows/site.yml
```

**Step 2: Update CLAUDE.md references**

Any references to `playground.yml` in CLAUDE.md or other docs should point to `site.yml`.

**Step 3: Final commit**

```bash
git add .github/workflows/site.yml CLAUDE.md
git commit -m "chore: rename workflow to site.yml, final cleanup"
```

---

## Summary

| Phase | Tasks | Key Deliverable |
|-------|-------|----------------|
| 1 | 1.1-1.3 | Playground at `/playground/` with embed mode + CI restructure |
| 2 | 2.1-2.3 | Animated landing page at `/` with Motion |
| 3 | 3.1-3.4 | mdBook docs site at `/docs/` with tutorials and API reference |
| 4 | 4.1-4.4 | Interactive step-based tutorials in the playground |
| 5 | 5.1-5.4 | Pyodide Python analyzer authoring + WASM limitation banners |
| 6 | 6.1-6.3 | Maintenance rules, perf optimization, final cleanup |
