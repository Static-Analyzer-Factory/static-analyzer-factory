# Algorithm Animation Tutorials — Design Document

**Date:** 2026-02-15
**Status:** Approved

## Overview

Interactive step-by-step algorithm visualizations for SAF's core analyses. Users control the pace (play/pause/step) and watch data structures evolve on each iteration. Built on pre-computed JSON traces generated from real SAF runs on curated example programs.

## Algorithms (4 total)

1. **Andersen's PTA** — Worklist-based inclusion constraint solving. Constraint extraction, wave-front processing, diff-based propagation, cycle detection via SCC.
2. **IFDS Taint Analysis** — Interprocedural path edge propagation. Source seeding, call/return flow, summary edge creation, fact accumulation.
3. **Interval Abstract Interpretation** — Forward fixpoint with widening/narrowing. Block-level state transfer, join at merge points, widening at loop headers.
4. **Memory SSA Clobber Walking** — Demand-driven backward walk through def chains with PTA alias queries at each step.

## Architecture Decision: Pre-computed Traces (Approach A)

**Chosen over:**
- Live WASM instrumented solvers (Approach B) — auto-syncs but adds instrumentation hooks to solver hot paths; every refactor must maintain hooks
- TypeScript simulation (Approach C) — silently diverges from Rust implementation

**Rationale:** Curated examples don't need live computation. Trace regeneration is a single command. When Rust algorithms change, tutorial prose needs manual updating anyway — regenerating traces is the easy part. No modification to solver internals required.

## Trace Format

Every algorithm trace is a JSON file:

```json
{
  "algorithm": "andersen-pta",
  "title": "Andersen's Inclusion-Based Pointer Analysis",
  "example": { "code": "int *p = &x; ...", "language": "c" },
  "initialState": {},
  "steps": [
    {
      "id": 0,
      "action": "Initialize addr constraints",
      "explanation": "Seed points-to sets from address-of operations",
      "highlights": { "nodes": ["p"], "edges": ["p->x"] },
      "state": {},
      "diff": { "added": [], "removed": [], "changed": [] }
    }
  ]
}
```

Each step has a **full state snapshot** (for random access / scrubbing) and a **diff** (for efficient animation of what changed). The `explanation` field provides tutorial prose for each step.

## Trace Generation

### New Crate: `saf-trace`

A binary crate that reuses solver internals but snapshots state between iterations. Does not modify the solvers — wraps them with an observer pattern.

```
crates/
  saf-trace/
    src/
      main.rs           # CLI: reads .ll, runs solver, dumps trace JSON
      pta_trace.rs      # PTA observer — snapshots after each worklist wave
      ifds_trace.rs     # IFDS observer — snapshots after each path edge pop
      absint_trace.rs   # Abstract interp observer — snapshots after each block
      mssa_trace.rs     # Memory SSA observer — snapshots each clobber walk step
    traces.toml         # Manifest: algorithm + input → output mapping
```

### Observer Pattern

The trace generator:
1. Extracts constraints / builds the ICFG (same as normal)
2. Runs a step-by-step reimplementation of the worklist loop, calling the same constraint handlers but snapshotting between each step
3. Uses the solver's public data structures (points-to maps, worklists, etc.)

The solver's hot path is never touched. If a solver refactor changes internal structure, only the trace generator needs updating.

### CLI Usage

```bash
# Single trace
cargo run -p saf-trace -- \
  --algorithm pta \
  --input tests/fixtures/llvm/e2e/simple_alias.ll \
  --output tutorials/public/content/algorithms/pta-andersen-basic.trace.json

# Regenerate all traces from manifest
cargo run -p saf-trace -- --all \
  --output-dir tutorials/public/content/algorithms/

# Validate existing traces
cargo run -p saf-trace -- --all --validate
```

### Trace Manifest

```toml
# crates/saf-trace/traces.toml

[[trace]]
algorithm = "pta"
input = "tests/fixtures/llvm/e2e/simple_alias.ll"
output = "pta-andersen-basic.trace.json"
title = "Andersen's PTA: Basic Pointer Assignment"

[[trace]]
algorithm = "pta"
input = "tests/fixtures/llvm/e2e/pta_cycle.ll"
output = "pta-andersen-cycles.trace.json"
title = "Andersen's PTA: Cycle Detection"

[[trace]]
algorithm = "ifds"
input = "tests/fixtures/llvm/e2e/taint_simple.ll"
output = "ifds-taint-basic.trace.json"
title = "IFDS: Tracking Tainted Data"

[[trace]]
algorithm = "absint"
input = "tests/fixtures/llvm/e2e/interval_loop.ll"
output = "interval-widening.trace.json"
title = "Interval Analysis: Widening at Loop Headers"

[[trace]]
algorithm = "mssa"
input = "tests/fixtures/llvm/e2e/mssa_alias.ll"
output = "mssa-clobber.trace.json"
title = "Memory SSA: Clobber Walk with Aliasing"
```

### Curated Example Programs

| Algorithm | File | What It Shows |
|-----------|------|---------------|
| PTA (basic) | `trace_pta_basic.c` | `int *p = &x; int *q = p;` — copy propagation, two distinct points-to sets |
| PTA (cycles) | `trace_pta_cycle.c` | Mutual pointer assignment in a loop — cycle detection and SCC collapse |
| IFDS | `trace_ifds_taint.c` | `user_input -> transform -> sink` — source-to-sink flow, summary edges |
| Intervals | `trace_interval_loop.c` | `for (int i = 0; i < 100; i++)` — widening at loop header, narrowing |
| Memory SSA | `trace_mssa_alias.c` | Store through alias, then load — clobber walk with PTA disambiguation |

These live in `tests/programs/c/` and compile to `.ll` fixtures via the existing workflow.

## Component Architecture

### Layout (Three-Panel)

```
┌─────────────────────────────────────────────────────┐
│  Tutorial Sidebar  │        Main Content Area        │
│                    │                                  │
│  > Getting Started │  ┌──────────────────────────┐   │
│  > Memory Safety   │  │   Explanation Panel       │   │
│  v Algorithms      │  │   (markdown for step)     │   │
│    * Andersen PTA  │  ├──────────────────────────┤   │
│    o IFDS Taint    │  │                          │   │
│    o Intervals     │  │   Graph Visualization    │   │
│    o Memory SSA    │  │   (Cytoscape + Motion)   │   │
│  > Advanced        │  │                          │   │
│                    │  ├──────────────────────────┤   │
│                    │  │  State Inspector          │   │
│                    │  │  (worklist, pts sets...)  │   │
│                    │  ├──────────────────────────┤   │
│                    │  │ <<  <  > Play  >  >>     │   │
│                    │  │ Step 3 / 12    Speed: 1x │   │
│                    │  └──────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Three Panels

1. **Explanation Panel** — Markdown prose from `steps[i].explanation`. Fade crossfade between steps.
2. **Graph Visualization** — Cytoscape graph updated per step. Motion animates overlays (glow, pulse, highlights).
3. **State Inspector** — Algorithm-specific data structure view (see Per-Algorithm Inspectors below).

### Stepper Controls

- Step back / Step forward (single step)
- Play / Pause (auto-advance with configurable speed: 0.5x, 1x, 2x)
- Jump to start / Jump to end
- Step counter: "Step 3 of 12"

### New Components

```
tutorials/src/components/
  AlgorithmStepper.tsx       # Main orchestrator (loads trace, manages step index)
  AlgorithmStepper.css       # Layout styles
  StepperControls.tsx        # Play/pause/step transport bar
  ExplanationPanel.tsx       # Animated markdown for current step
  StateInspector.tsx         # Routes to algorithm-specific inspector
  state-inspectors/
    PtaInspector.tsx         # Worklist + points-to sets
    IfdsInspector.tsx        # Path edges + summaries
    IntervalInspector.tsx    # Variable interval bars
    MssaInspector.tsx        # Def chain walker
```

### Integration with Tutorial System

New `stepType` field in `steps.json`:

```json
{
  "title": "Andersen's PTA: Step by Step",
  "content": "Watch how the solver processes constraints...",
  "stepType": "algorithm",
  "algorithmTrace": "pta-andersen-basic.trace.json"
}
```

`InteractiveStep` checks `stepType` — if `"algorithm"`, renders `AlgorithmStepper` instead of the code editor + static graph.

## Motion Animation Strategy

### What Gets Animated

| Element | Animation | Duration | Easing |
|---------|-----------|----------|--------|
| New nodes | Scale 0->1 + green glow | 400ms | spring(0.6) |
| New edges | Path draw-in (stroke-dashoffset) | 300ms | easeOut |
| Changed nodes | Yellow pulse + scale bump | 500ms | easeInOut |
| Changed edges | Color transition + thickness pulse | 300ms | easeInOut |
| Removed elements | Fade to 0.2 opacity (dimmed) | 200ms | easeOut |
| Explanation text | Fade crossfade | 250ms | easeInOut |
| Inspector rows | Layout animation (slide in, flash) | 300ms | spring(0.7) |
| Active worklist item | Pulse ring highlight | 600ms | repeat |

### Hybrid Motion + Cytoscape

1. **Graph structure changes** — Cytoscape API with `animate()` for position transitions
2. **Visual overlays** (glow, pulse, annotations) — Motion `<div>` positioned over Cytoscape nodes via `cy.node.renderedPosition()`
3. **Non-graph UI** (explanation, inspector, controls) — Pure Motion `<motion.div>` with `AnimatePresence`

### Step Transition Sequence

```
1. Compute diff between steps[N] and steps[N+1]
2. Fade out old explanation, fade in new (250ms)
3. Simultaneously:
   a. Dimmed animation for removed elements (200ms)
   b. Highlight pulse for changed elements (300ms)
   c. Appear animation for new elements (400ms)
   d. Inspector layout animation (300ms)
4. If topology changed, Cytoscape layout with animation (500ms)
5. Mark step complete, enable controls
```

Auto-play chains steps with configurable pause (default 1.5s at 1x speed).

Stepping backward: set graph to `steps[N-1].state` (full snapshots make this trivial).

## Per-Algorithm State Inspectors

### PTA Inspector

- **Worklist**: Horizontal rank groups, current wave highlighted
- **Points-to sets**: Table (value -> locations), new/changed rows animate with Motion `layoutId`
- **Constraints**: Checklist (Addr/Copy/Load/Store/Gep), processed vs pending

### IFDS Inspector

- **Worklist**: Current path edge as readable tuple `(func, d1, inst, d2)`
- **Summary edges**: Per-function summaries showing fact transformations
- **Facts at instructions**: Scrollable list, current instruction pinned

### Interval Inspector

- **Range bars**: Horizontal bars per variable, Motion animates width changes (widening is visually dramatic — bar snaps to full width)
- **Operation log**: Shows the specific join/widen/narrow operation

### Memory SSA Inspector

- **Walk visualization**: Vertical chain of defs walked backward, each with PTA alias query result
- **Points-to context**: Shows PTA sets that answered each alias query

### Shared Inspector Behaviors

- Row enter: slide-in from left (`opacity: 0, x: -20`)
- Row highlight: yellow flash background
- Row reorder: Motion `layout` prop
- Expand/collapse: `AnimatePresence` for detail sections

## Tutorial Content Structure

### New Category in Registry

```json
{
  "id": "algorithms",
  "title": "Algorithms",
  "tutorials": [
    { "id": "pta-andersen", "title": "Pointer Analysis (Andersen's)", "difficulty": "intermediate", "mode": "browser" },
    { "id": "ifds-taint", "title": "IFDS Taint Analysis", "difficulty": "intermediate", "mode": "browser" },
    { "id": "interval-analysis", "title": "Interval Abstract Interpretation", "difficulty": "intermediate", "mode": "browser" },
    { "id": "memory-ssa", "title": "Memory SSA & Clobber Walking", "difficulty": "advanced", "mode": "browser" }
  ]
}
```

### Tutorial Step Flow (PTA Example)

| Step | Type | Content |
|------|------|---------|
| 1 | prose | "What is Pointer Analysis?" — problem and motivation |
| 2 | prose + code | Show example C program, explain expected aliases |
| 3 | prose | "Constraint Extraction" — Addr/Copy/Load/Store/Gep types |
| 4 | algorithm | Stepper: watch constraints extracted from the program |
| 5 | prose | "Worklist Solving" — wave-front processing, topological ordering |
| 6 | algorithm | Stepper: watch worklist drain, points-to sets grow to fixpoint |
| 7 | prose | "Cycle Detection" — when and why SCCs form |
| 8 | algorithm | Stepper: second example with mutual pointers, cycle collapse |
| 9 | prose | "Using PTA Results" — motivate downstream analyses |

### File Organization

```
tutorials/public/content/algorithms/
  pta-andersen/
    steps.json
    pta-andersen-basic.trace.json
    pta-andersen-cycles.trace.json
  ifds-taint/
    steps.json
    ifds-taint-basic.trace.json
  interval-analysis/
    steps.json
    interval-widening.trace.json
  memory-ssa/
    steps.json
    mssa-clobber.trace.json
```

## Testing

### Unit Tests (Vitest)

- `AlgorithmStepper.test.tsx` — load minimal trace, verify step navigation
- `StepperControls.test.tsx` — play/pause state, speed, boundary behavior
- `StateInspector.test.tsx` — correct inspector rendered per algorithm type
- Per-inspector tests with fixture trace fragments

### Trace Validation (Rust)

`saf-trace --validate` checks:
- Every step has both `state` and `diff`
- Diffs are consistent with consecutive state snapshots
- All IDs in `highlights` exist in the state
- No empty steps (every step changes something)

### Integration Tests (Playwright)

- Navigate to each algorithm tutorial
- Verify stepper loads and displays step 1
- Step forward, verify counter updates
- Play, verify auto-advance
- Step back, verify state reverts
- Verify graph has visible nodes

### CI

- `cargo run -p saf-trace -- --all --validate` ensures traces stay valid against current solver behavior
- Breaks if Rust changes invalidate trace generation
