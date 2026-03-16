# SAF Landing Page, Documentation & Interactive Tutorials Design

**Date:** 2026-02-15
**Status:** Approved

## Goal

Transform SAF's web presence from a standalone playground into a full educational and promotional platform: landing page, documentation site, embeddable widgets, interactive tutorials, and Python analyzer authoring in the browser.

## Audience

Three personas, all served from one site:
1. **Students & learners** — program analysis / compilers courses, want to understand PTA, CFG, value-flow visually
2. **Security researchers** — want to write custom static analyzers for vulnerability detection
3. **AI/agent developers** — building agents that use SAF's schema-driven API

## Site Architecture

All hosted on GitHub Pages at `lyk.github.io/static-analyzer-lib/`:

```
/                          → Landing page (React + Motion animations)
/playground/               → Existing playground (moved from root)
/playground/?embed=true    → Embeddable widget mode (chrome stripped)
/playground/?tutorial=X    → Interactive tutorial mode
/docs/                     → mdBook documentation site
```

Single GitHub Actions workflow builds and deploys all three.

## Component 1: Landing Page

**Tech:** React 19 + Vite + Motion (formerly Framer Motion) for animations.

**Layout:**

### Hero Section
- Headline + tagline (e.g., "Build program analysis tools. Understand code deeply.")
- Animated hero graphic — a CFG graph that draws itself with Motion animations
- Two CTAs: "Try the Playground" → `/playground/`, "Read the Docs" → `/docs/`

### Feature Showcase (3 animated columns, scroll-triggered reveals)
1. **Visualize** — "See how programs actually work" — embedded Graph Viewer widget showing a CFG
2. **Analyze** — "Points-to analysis, taint tracking, value-flow reasoning" — PTA visualization
3. **Build** — "Write custom analyzers in Python" — code snippet of the Python API

### Persona Cards (3 cards with hover effects)
- **Students & Learners** → guided tutorials on CWE vulnerabilities → `/docs/tutorials/`
- **Security Researchers** → author taint and reachability checkers → `/playground/` with Pyodide
- **AI Agent Developers** → schema-driven API for programmatic analysis → `/docs/api-reference/`

### Technical Highlights
- "Runs entirely in your browser — no server, no install"
- "Powered by Rust + WebAssembly"
- "Deterministic, reproducible analysis results"
- Open source badge + GitHub link

## Component 2: mdBook Documentation Site

**Tech:** mdBook, built in CI, output to `/docs/` path.

### Structure

```
docs/book/
  SUMMARY.md
  introduction.md
  getting-started/
    installation.md         # Docker setup, Python SDK install, uv workflow
    first-analysis.md       # Hello world with SAF
    playground-tour.md      # How to use the browser playground
    browser-vs-full.md      # Feature comparison matrix (see WASM Limits below)
  concepts/
    air.md                  # Analysis Intermediate Representation
    cfg-icfg.md             # Control flow graphs
    callgraph.md            # Call graph construction
    points-to.md            # Pointer analysis explained
    value-flow.md           # Value flow graph
    taint-analysis.md       # Taint tracking
  tutorials/                # Rendered from existing tutorials/
    memory-safety/          # CWE-416, 415, 401
    buffer-overflow/        # CWE-120, 121, 131
    integer-issues/         # CWE-190, 191
    resource-safety/        # CWE-775, 667
    information-flow/       # CWE-78, 134
  api-reference/
    python-sdk.md           # Python API reference
    cli.md                  # CLI usage
    property-graph.md       # PropertyGraph export format
  embedding/
    widgets.md              # How to embed SAF widgets in your site
```

Each concept page includes an embedded widget (`<iframe>`) showing the concept on a real example. Tutorials exist in two forms: readable markdown in mdBook + interactive walkthroughs in the playground.

## Component 3: Embeddable Widgets

Two widget types, both implemented as URL parameter modes of the existing playground:

### Graph Viewer
```html
<iframe src="/playground/?embed=true&example=taint&graph=cfg"
        width="600" height="400" />
```
- Shows just the Cytoscape.js graph visualization
- No settings panel, no example picker, no editor
- Clickable nodes still highlight in source (if split mode)

### Split View (Source + Graph)
```html
<iframe src="/playground/?embed=true&example=use_after_free&graph=valueflow&split=true"
        width="800" height="500" />
```
- Source code on the left, graph on the right
- Auto-runs analysis on load
- Compact — no settings panel, no example picker

### Implementation
- Add URL parameter parsing to `App.tsx`
- `embed=true` → hide nav, settings, example picker
- `split=true` → show source + graph side-by-side
- `example=X` → pre-load a specific example
- `graph=X` → auto-select a graph type (cfg, callgraph, defuse, valueflow, pta)

## Component 4: Interactive Step-Based Tutorials

Each tutorial is a guided walkthrough with steps, running inside the playground:

### UI
```
┌─────────────────────────────────────────────────────┐
│ Tutorial: Detecting Use-After-Free (CWE-416)        │
│                                                     │
│  Step 2 of 5: Building the Value-Flow Graph         │
│                                                     │
│  [Explanatory text about this step]                 │
│                                                     │
│  ┌──────────────────┬──────────────────────────┐    │
│  │  [C source code] │   [Graph visualization]  │    │
│  └──────────────────┴──────────────────────────┘    │
│                                                     │
│  [Prompt: "Try modifying the code and re-run"]      │
│                                                     │
│  [← Previous]                        [Next Step →]  │
└─────────────────────────────────────────────────────┘
```

### Data Model
Tutorials defined as JSON/YAML files:
```yaml
id: uaf-detection
title: "Detecting Use-After-Free (CWE-416)"
steps:
  - title: "The Vulnerable Program"
    text: "This program frees a pointer and then uses it..."
    code: |
      void foo() { int *p = malloc(4); free(p); *p = 42; }
    graph: cfg
    highlight_lines: [3]
  - title: "Building the Value-Flow Graph"
    text: "The value-flow graph tracks how values move..."
    graph: valueflow
    highlight_nodes: ["free_call", "deref"]
  # ... more steps
```

### Features
- Each step loads a specific program + graph type
- Tutorial text panel explains what to look at
- Users can edit code and re-run — graph updates live
- "Next Step" advances to next concept (may switch graph type)
- Progress tracked in localStorage
- URL: `/playground/?tutorial=uaf&step=2`

## Component 5: Pyodide Python Analyzer Authoring

Users write Python analyzer scripts in the browser, all client-side.

### UI
New "Analyzer" tab in the playground:
- **Left pane:** C source code (same as existing)
- **Center pane:** Python editor (CodeMirror) for analyzer scripts
- **Right pane:** Analyzer output (detected bugs, annotations)

### Architecture
```
User's Python script (Pyodide / CPython in WASM)
  ↓ calls via JS bridge
saf-wasm API (existing Rust WASM module)
  ↓ returns
PropertyGraph JSON → Python objects
  ↓ user's script processes
saf.report() calls → rendered in output pane
```

### Python API Surface (browser)
```python
import saf

# Analyze the current program (already compiled in playground)
result = saf.analyze()

# Access graphs as PropertyGraph objects
cfg = result.cfg()
cg = result.callgraph()
vf = result.valueflow()
pta = result.points_to()

# Graph queries
for node in cfg.nodes():
    print(node.id, node.labels, node.properties)

for edge in vf.edges():
    if edge.edge_type == "Store":
        ...

# Report findings (rendered in output pane + source highlights)
saf.report(node_id="0x...", severity="high", message="Use-after-free detected")
```

### Implementation
1. Load Pyodide (~6.4 MB) on "Analyzer" tab activation; show loading spinner (~4s cold, ~2s warm)
2. Inject JS bridge functions that call `saf-wasm` into Pyodide global scope
3. Wrap in a Python `saf` module with clean API (PropertyGraph wrapper classes)
4. "Run" button executes user script in Pyodide sandbox
5. Capture `saf.report()` calls → render in output pane + highlight source lines
6. Capture `print()` output → show in console sub-pane

### Starter Templates
Provide 3-4 starter analyzer templates:
- "Find all free() calls" (simple graph traversal)
- "Detect use-after-free" (value-flow path query)
- "Taint analysis for command injection" (source/sink checker)
- "Null pointer dereference detector" (PTA + CFG)

## Handling WASM Limitations

Features unavailable in the browser build:

| Feature | Full SAF | saf-wasm | Reason |
|---------|----------|----------|--------|
| Z3 path feasibility | Yes | No | Z3 doesn't compile to WASM |
| LLVM native frontend | Yes | tree-sitter | LLVM ~100MB, can't bundle |
| Large programs (>1K LOC) | Yes | OOM risk | WASM memory limits |
| Context-sensitive PTA | Yes | Slow | Compute-heavy in WASM |

### Strategy: Graceful Degradation

**In the Playground UI:**
- Settings that require Z3 show an inline banner: *"Path feasibility requires Z3, which isn't available in the browser. Results shown without path filtering. [Install SAF locally →]"*
- Settings panel marks unsupported features with a "Desktop only" badge
- Analysis still runs without the missing feature — users get useful (if less precise) results

**In the Docs:**
- `browser-vs-full.md` has a clear comparison table
- Each tutorial notes if it requires features beyond the browser
- Z3-dependent tutorials show the concept with pre-computed result screenshots, plus "try it locally" code blocks

**On the Landing Page:**
- "Try it instantly in your browser" for playground CTA
- "Full analysis power with the Python SDK" for install CTA
- Feature showcase shows full SAF capabilities with a note that browser covers a subset

**Principle:** Never silently give wrong results. Always tell the user what's limited and offer the path to the full experience.

## Maintenance Discipline

New CLAUDE.md section — "Frontend & Docs Maintenance":
- When adding a new analysis feature → update docs concept page + add playground example
- When adding a new graph type or API endpoint → add an embeddable example + API reference entry
- Playground spec files (YAML) must stay in sync with `saf-analysis` specs
- Tutorials in `tutorials/` must be rendered in both mdBook (readable) and playground (interactive)
- Pyodide `saf` module must mirror the real Python SDK API surface
- Landing page feature claims must match actual capabilities

## Phased Implementation

### Phase 1: Landing Page + Playground URL Migration
- Build landing page with Motion animations
- Move playground from `/` to `/playground/`
- Update GitHub Actions deployment

### Phase 2: Embeddable Widgets
- Add URL parameter parsing for `embed`, `split`, `example`, `graph`
- Embed mode: strip chrome, auto-run analysis
- Test with iframe embedding

### Phase 3: mdBook Documentation Site
- Set up mdBook structure
- Migrate existing tutorials
- Write concept pages with embedded widgets
- Add API reference
- Feature comparison matrix (browser vs full)

### Phase 4: Interactive Tutorials
- Tutorial data model (YAML/JSON step definitions)
- Tutorial UI component (step navigation, text panel)
- Build 3-5 initial tutorials from existing material
- Progress tracking (localStorage)

### Phase 5: Pyodide Analyzer Authoring
- Integrate Pyodide loader
- Build JS bridge layer (saf-wasm ↔ Pyodide)
- Implement Python `saf` module wrapper
- Analyzer tab UI (Python editor + output pane)
- Starter templates
- WASM limitation banners

### Phase 6: Polish & Maintenance
- CLAUDE.md maintenance rules
- CI checks for docs/playground sync
- Performance optimization (Pyodide pre-warming, lazy loading)
