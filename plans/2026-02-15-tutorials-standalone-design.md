# Standalone Tutorials App Design

**Date:** 2026-02-15
**Status:** Approved

## Problem

Tutorials currently live in two places — 7 markdown pages in mdBook (`/docs/tutorials/`) and 1 interactive JSON tutorial in the playground (`?tutorial=uaf-detection`). This causes four problems:

1. **Buried**: Users navigate through docs to find tutorials — they aren't prominent or discoverable
2. **Too static**: Markdown tutorials can't match the interactivity of the playground
3. **Wrong framing**: `/docs/` feels like reference material, but tutorials should attract new users
4. **Duplication**: Maintaining both markdown and JSON versions of the same tutorial is wasteful

## Solution

A standalone React app at `/tutorials/` — separate from docs and playground. Each tutorial page mixes narrative prose with embedded playground widgets for interactive parts and CLI/script blocks for local execution of features not available in the browser.

## Architecture

### URL Structure

```
/               → Landing page (site/)
/tutorials/     → Tutorial hub & catalog (tutorials/)    ← NEW
/playground/    → Interactive playground
/docs/          → API reference & concepts (mdBook)
```

### Key Decisions

- **Pre-computed graphs**: Tutorials ship with pre-rendered PropertyGraph JSON. No WASM analysis engine needed — the app is lightweight.
- **Playground embeds for "try it yourself"**: At key points, tutorials embed the playground via iframe (`?embed=true&split=true&example=...`) so users can modify code and re-analyze.
- **CLI blocks for local-only features**: Features not available in the browser (Z3 feasibility, full specs, custom analyzers) get copyable CLI command blocks and downloadable Python scripts.

### Content Format

Markdown files, not JSON. Each tutorial is a directory:

```
tutorials/content/
  uaf-detection/
    meta.yaml          # title, difficulty, category, prerequisites
    step-1.md          # Markdown with frontmatter for graph/code refs
    step-2.md
    graphs/            # Pre-computed PropertyGraph JSON
      step-1-cfg.json
      step-2-valueflow.json
    scripts/           # Downloadable Python scripts
      detect_uaf.py
```

### Tutorial Step Schema

```typescript
interface TutorialStep {
  title: string;
  content: string;          // Markdown prose (rendered)
  code?: string;            // C/Python code shown alongside
  graph?: PrecomputedGraph; // Pre-rendered graph JSON (no WASM needed)
  playground?: string;      // Playground embed URL for "try it" sections
  localCmd?: string;        // CLI command for local execution
  localScript?: string;     // Path to downloadable Python script
  highlightLines?: number[];
  challenge?: string;       // "Now try..." prompt
}
```

## Layout

### Catalog Page (`/tutorials/`)

1. Hero: "Learn Static Analysis" headline, difficulty legend
2. Learning paths as horizontal cards (one per audience)
3. Tutorial grid filterable by difficulty and category
4. Each card: title, difficulty badge, "Browser" / "Local" / "Both" tag

### Tutorial Page (`/tutorials/<id>`)

```
┌──────────────┬──────────────────────┬──────────────────────────┐
│  TOC Sidebar │   Narrative prose    │   Graph visualization    │
│              │   (markdown)         │   (Cytoscape/precomputed)│
│  > Getting   │                      │                          │
│    Started   │   Code snippet       │                          │
│  v Memory    │   (highlighted)      ├──────────────────────────┤
│    Safety    │                      │  > Try it in Playground  │
│    > Step 1  │                      │                          │
│      Step 2  ├──────────────────────┴──────────────────────────┤
│      Step 3  │  Run Locally (collapsible)                      │
│  > Info Flow │  $ docker compose run --rm dev sh -c '...'      │
│  > Advanced  │  Download detect_uaf.py                         │
└──────────────┴─────────────────────────────────────────────────┘
```

- **TOC Sidebar** (left, always visible): All categories, expanded to show steps within active tutorial, current step highlighted, collapsible. Mobile: hamburger menu.
- **Left column**: Narrative + code (scrollable)
- **Right column**: Graph visualization (sticky)
- **Bottom**: Collapsible "Run Locally" section with CLI commands and downloadable scripts
- Step navigation at top + keyboard arrows + progress bar

## Tutorial Inventory (9 tutorials)

### Getting Started (newcomer-focused)

| Tutorial | Difficulty | Mode | Description |
|---|---|---|---|
| Your First Analysis | Beginner | Browser | Load C program, see CFG + value flow |
| Understanding the Graphs | Beginner | Browser | All 5 graph types on a single program |

### Memory Safety (student + evaluator)

| Tutorial | Difficulty | Mode | Description |
|---|---|---|---|
| Detecting Use-After-Free | Beginner | Both | Migrated from existing, enhanced with local path |
| Double Free Detection | Intermediate | Both | Track aliased pointers through free calls |
| Memory Leak Detection | Intermediate | Local | Requires specs modeling |

### Information Flow (security-focused)

| Tutorial | Difficulty | Mode | Description |
|---|---|---|---|
| Taint Analysis Basics | Beginner | Browser | scanf to printf flow, source/sink concepts |
| Command Injection | Intermediate | Both | CWE-78 detection with specs |

### Advanced (evaluator + power user)

| Tutorial | Difficulty | Mode | Description |
|---|---|---|---|
| Writing a Custom Analyzer | Advanced | Local | Python SDK script from scratch |
| Specs Authoring | Advanced | Local | Model library functions for taint/PTA |

## Tech Stack

- React 19 + TypeScript + Vite (same as playground and site)
- Cytoscape.js + dagre for graph rendering (copied from playground)
- CodeMirror for syntax-highlighted code blocks
- `react-markdown` + `rehype-highlight` for tutorial prose
- No WASM, no tree-sitter, no Pyodide

## Build & Deploy

Extends existing CI workflow:

```
.github/workflows/playground.yml
  ├── Build landing page   → _site/
  ├── Build playground     → _site/playground/
  ├── Build docs (mdBook)  → _site/docs/
  └── Build tutorials      → _site/tutorials/   ← NEW
```

### Cross-App Navigation

All four apps get a consistent top nav bar:

```
[SAF Logo]   Tutorials   Playground   Docs   GitHub
```

## Content Migration

| Current location | Action |
|---|---|
| `docs/book/src/tutorials/*.md` (7 pages) | Remove — replaced by tutorials app |
| `playground/public/tutorials/*.json` (1) | Migrate — data moves to `tutorials/content/` |
| `docs/book/src/concepts/*.md` (6 pages) | Keep — tutorials link to these as "deeper reading" |
| `playground/src/examples/` (6 examples) | Keep — playground examples remain separate |

## Out of Scope

- No user accounts or progress persistence (localStorage bookmark at most)
- No in-browser code execution in the tutorials app itself (playground embeds handle this)
- No tutorial authoring UI (markdown in git is the authoring interface)
- No search at launch (TOC sidebar + filters sufficient for 9 tutorials)
- No video or animation (prose + code + graphs + playground embeds are the media)
