# Plan 117: Algorithm Tutorial Redesign

## Problem

The PTA algorithm stepper page (`#pta-andersen/2`) has several UX issues:
1. **Layout**: Explanation, graph, state inspector, and controls are stacked vertically in a single scroll container. Users must scroll past the graph to reach controls or state inspector.
2. **Graph**: Nodes are ~120px circles filling the 300px area. No edges at step 0. Uses CFG renderer for a constraint graph. `cose` layout scatters nodes randomly.
3. **Missing content**: IFDS, Interval Analysis, and Memory SSA tutorials are single-page placeholders saying "coming soon."
4. **Limited algorithm coverage**: Only 4 algorithms. No tutorials for callgraph construction, dominator trees, sparse value-flow, or context-sensitive PTA.
5. **Flat sidebar**: All algorithm tutorials sit unsorted under a single "Algorithms" category.

## Design

### 1. Side-by-Side Panel Layout

Replace the vertical stack with a two-panel layout inside a new `AlgorithmStepperLayout` component:

```
┌──────────────────────────────────────────────────────────┐
│ Step title + action                          Step 2 / 4  │
├────────────────────────────┬─────────────────────────────┤
│                            │  WORKLIST                   │
│                            │  R0: [p] [r]  R1: [q]      │
│     Graph                  │                             │
│   (Cytoscape)              │  POINTS-TO SETS             │
│                            │  p → {x}  changed           │
│   compact nodes            │  r → {y}  changed           │
│   with visible edges       │                             │
│                            │  CONSTRAINTS                │
│                            │  [addr] x → p  done         │
│                            │  [copy] p → q  pending      │
│                            │  [addr] y → r  done         │
├────────────────────────────┴─────────────────────────────┤
│  |< < [play] > >|         Step 2 / 4              1x    │
├──────────────────────────────────────────────────────────┤
│  Explanation text (collapsible, current step narrative)   │
└──────────────────────────────────────────────────────────┘
```

- **Left panel (~60%):** Graph visualization, independently scrollable
- **Right panel (~40%):** State inspector, independently scrollable
- **Pinned controls bar:** Always visible, never scrolled away. SVG icons for all buttons (no mixed unicode/emoji).
- **Collapsible explanation:** Below controls, shows step narrative in markdown
- Entire stepper fits within the viewport — no page-level scroll needed
- Responsive: collapses to stacked layout below 900px

### 2. Graph Fixes (Cytoscape Config)

Add a dedicated `algorithm` graph type to `GraphViewer` instead of reusing the CFG renderer:

- **Node size:** 30-36px diameter (down from ~120px)
- **Font size:** 12px
- **Node colors:** Distinct fill for `value` nodes (pointers) vs `location` nodes (pointees)
- **Edge styling:** Directed arrows, colored by constraint type (addr=blue, copy=amber, load=green, store=pink)
- **Highlighted elements:** Accent border/glow on active nodes and edges
- **Layout algorithm:** `breadthfirst` or `dagre` (pointers top, pointees bottom) instead of `cose`
- **Step 0 enhancement:** Show dashed "pending constraint" edges so users see the graph structure before processing begins (currently 5 isolated circles)

### 3. SVG Stepper Controls

Replace mixed unicode characters with consistent inline SVG icons:
- Jump to start, step back, play/pause, step forward, jump to end
- Simple geometric shapes (triangles, bars) — no external icon library
- Consistent sizing and alignment

### 4. Sidebar Reorganization

Add subcategories to group related algorithms:

```
Algorithms
  Pointer Analysis
    Andersen's PTA (intermediate)
    Context-Sensitive PTA (k-CFA) (advanced)         [new]
  Dataflow Analysis
    IFDS Taint Analysis (intermediate)
    Interval Abstract Interpretation (intermediate)
  Memory Modeling
    Memory SSA & Clobber Walking (advanced)
    Sparse Value-Flow Analysis (advanced)             [new]
  Graph Foundations
    Dominator Trees & Loop Detection (intermediate)   [new]
    Callgraph Construction (CHA/RTA/VTA) (intermediate) [new]
```

Type system changes:
- Add optional `subcategory` field to `TutorialMeta`
- Add `SubcategoryInfo` type with `id`, `title`
- Sidebar renders subcategories as indented groups within the Algorithms category

### 5. Tutorial Content Structure

Each algorithm tutorial follows a 3-part structure:

**Part 1: Theory intro** (1-2 prose steps)
- What the algorithm does, why it matters, key concepts
- Example C program that will be analyzed

**Part 2: Algorithm stepper** (1 step with multi-step trace)
- Hand-crafted trace JSON showing step-by-step execution
- Side-by-side layout with graph + state inspector

**Part 3: Interactive analysis** (1 step)
- User writes/modifies C code, clicks Analyze, sees live results
- Uses existing `InteractiveStep` component with appropriate graph type

### 6. Per-Tutorial Details

| Tutorial | Trace Steps | State Inspector | Graph Type | Interactive Graph |
|----------|------------|-----------------|------------|-------------------|
| Andersen's PTA | 4 (existing, enhance) | PtaInspector (existing) | constraint graph | pta |
| k-CFA | ~6 | New: KCfaInspector | constraint graph w/ contexts | pta |
| IFDS Taint | ~5 | IfdsInspector (existing) | exploded supergraph | valueflow |
| Interval Analysis | ~6 | IntervalInspector (existing) | CFG w/ annotations | cfg |
| Memory SSA | ~5 | MssaInspector (existing) | def-use chain | defuse |
| Sparse Value-Flow | ~5 | New: SvfInspector | value-flow graph | valueflow |
| Dominator Tree | ~5 | New: DomInspector | CFG + dom tree | cfg |
| Callgraph (CHA/RTA/VTA) | ~6 | New: CgInspector | callgraph | callgraph |

New state inspectors needed: KCfaInspector, SvfInspector, DomInspector, CgInspector (4 total).

The CHA/RTA/VTA tutorial is unique: it shows 3 algorithms in sequence on the same program, demonstrating progressive precision improvement.

## Implementation Phases

### Phase 1: Layout + Graph Fixes
- New `AlgorithmStepperLayout` component (side-by-side panels)
- SVG stepper control icons
- New `algorithm` graph type in GraphViewer (compact nodes, dagre layout)
- Enhance PTA trace with dashed pending edges at step 0
- Update `AlgorithmStepper.tsx` to use new layout
- Update `AlgorithmStepper.css` for panel layout

### Phase 2: Fill Existing Placeholders (IFDS, Interval, Memory SSA)
- Write trace JSON files (~5 steps each)
- Add theory intro steps + interactive analysis steps
- Update steps.json for each tutorial (currently 1 step each → 3-4 steps)

### Phase 3: Sidebar Subcategories
- Add `subcategory` to type system (`types.ts`)
- Add subcategory metadata to registry
- Update Sidebar component to render nested groups
- Update Catalog page to show subcategory groupings

### Phase 4: New Algorithm Tutorials
- Dominator tree & loop detection (DomInspector + trace + content)
- Callgraph construction CHA/RTA/VTA (CgInspector + trace + content)
- Context-sensitive PTA k-CFA (KCfaInspector + trace + content)
- Sparse value-flow analysis (SvfInspector + trace + content)
- Register all in registry with subcategories

Each phase is independently shippable.

## Files Modified/Created

### Phase 1
- `tutorials/src/components/AlgorithmStepper.tsx` — rewrite layout
- `tutorials/src/components/AlgorithmStepper.css` — side-by-side panel styles
- `tutorials/src/components/StepperControls.tsx` — SVG icons
- `tutorials/src/components/StepperControls.css` — updated icon styles
- `tutorials/src/components/GraphViewer.tsx` — add `algorithm` graph type
- `tutorials/public/content/pta-andersen/pta-andersen-basic.trace.json` — add pending edges

### Phase 2
- `tutorials/public/content/ifds-taint/steps.json` — expand to 3-4 steps
- `tutorials/public/content/ifds-taint/ifds-taint-basic.trace.json` — new
- `tutorials/public/content/interval-analysis/steps.json` — expand to 3-4 steps
- `tutorials/public/content/interval-analysis/interval-basic.trace.json` — new
- `tutorials/public/content/memory-ssa/steps.json` — expand to 3-4 steps
- `tutorials/public/content/memory-ssa/mssa-basic.trace.json` — new

### Phase 3
- `tutorials/src/content/types.ts` — add subcategory types
- `tutorials/src/content/registry.ts` — add subcategory metadata
- `tutorials/src/components/Sidebar.tsx` — nested group rendering
- `tutorials/src/components/Sidebar.css` — subcategory styles
- `tutorials/src/components/Catalog.tsx` — subcategory display

### Phase 4
- `tutorials/public/content/pta-kcfa/` — new tutorial directory
- `tutorials/public/content/sparse-valueflow/` — new tutorial directory
- `tutorials/public/content/dominator-tree/` — new tutorial directory
- `tutorials/public/content/callgraph-construction/` — new tutorial directory
- `tutorials/src/components/state-inspectors/KCfaInspector.tsx` — new
- `tutorials/src/components/state-inspectors/SvfInspector.tsx` — new
- `tutorials/src/components/state-inspectors/DomInspector.tsx` — new
- `tutorials/src/components/state-inspectors/CgInspector.tsx` — new
- `tutorials/src/content/trace-types.ts` — add new algorithm state types
- `tutorials/src/content/registry.ts` — register new tutorials
