# Algorithm Tutorial Redesign — Implementation Plan (Agent Team)

**Goal:** Redesign algorithm tutorial stepper with side-by-side layout, compact graph nodes, SVG controls, sidebar subcategories, fill 3 placeholder tutorials, and add 4 new algorithm tutorials.

**Architecture:** Replace the vertical-stack `AlgorithmStepper` with a two-panel layout (graph left, state inspector right, controls pinned bottom, explanation collapsible). Add a dedicated algorithm graph renderer with compact node styling and dagre layout. Extend the type system with subcategories for sidebar grouping.

**Tech Stack:** React, TypeScript, Cytoscape.js (via `@saf/web-shared`), Motion (framer-motion), Vite

---

## Team Structure

| Agent | Phase 1 | Phase 2 | Phase 3 |
|-------|---------|---------|---------|
| **Leader** | Orchestrate, verify | Layout rewrite (Task 6), verify | Integration (Task 11), verify |
| **Agent A** | SVG controls (Task 1) | Subcategories (Task 7) | Dominator tree (Task 8) |
| **Agent B** | Algorithm graph renderer (Task 2) | — | Callgraph construction (Task 9) |
| **Agent C** | IFDS tutorial content (Task 3) | — | k-CFA tutorial (Task 10) |
| **Agent D** | Interval tutorial content (Task 4) | — | Sparse value-flow (Task 11a) |
| **Agent E** | Memory SSA tutorial content (Task 5) | — | — |

## Phase Dependencies

```
Phase 1 (all parallel, no file conflicts)
  Tasks 1-5 → all independent
  │
Phase 2 (after Phase 1)
  Task 6 (Leader: layout) ← needs Tasks 1+2 complete
  Task 7 (Agent A: subcategories) ← needs Task 2 complete (types.ts)
  │
Phase 3 (after Phase 2)
  Task 7 must complete first (adds trace-types + subcategory support)
  Tasks 8-11a (parallel, no file conflicts)
  Task 11b (Leader: integration wiring) ← after 8-11a
```

## File Ownership Map (no conflicts within a phase)

### Phase 1
| File | Owner |
|------|-------|
| `StepperControls.tsx`, `StepperControls.css` | Agent A |
| `types.ts`, `GraphViewer.tsx` | Agent B |
| `pta-andersen/pta-andersen-basic.trace.json` | Agent B |
| `ifds-taint/steps.json`, `ifds-taint/*.trace.json` | Agent C |
| `interval-analysis/steps.json`, `interval-analysis/*.trace.json` | Agent D |
| `memory-ssa/steps.json`, `memory-ssa/*.trace.json` | Agent E |

### Phase 2
| File | Owner |
|------|-------|
| `AlgorithmStepper.tsx`, `AlgorithmStepper.css` | Leader |
| `types.ts`, `trace-types.ts`, `registry.ts`, `Sidebar.tsx`, `Sidebar.css` | Agent A |

### Phase 3
| File | Owner |
|------|-------|
| `state-inspectors/DomInspector.tsx`, `content/dominator-tree/*` | Agent A |
| `state-inspectors/CgInspector.tsx`, `content/callgraph-construction/*` | Agent B |
| `state-inspectors/KCfaInspector.tsx`, `content/pta-kcfa/*` | Agent C |
| `state-inspectors/SvfInspector.tsx`, `content/sparse-valueflow/*` | Agent D |
| `StateInspector.tsx`, `registry.ts` (wire all 4) | Leader |

---

## Phase 1: Foundation (5 agents, parallel)

---

### Task 1 — Agent A: SVG Stepper Controls

**Scope:** Replace unicode transport symbols with inline SVG icons.

**Files to modify:**
- `tutorials/src/components/StepperControls.tsx`
- `tutorials/src/components/StepperControls.css`

**Context — current button contents (StepperControls.tsx):**
The component renders 5 buttons with unicode text: `⏮`, `⏪`, `▶`/`⏸`, `⏩`, `⏭`. These render inconsistently across platforms. Replace with inline SVG.

**Changes to StepperControls.tsx:**

Add SVG icon components above the `StepperControls` function:

```tsx
const IconJumpStart = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <rect x="2" y="3" width="2" height="10" />
    <polygon points="14,3 14,13 6,8" />
  </svg>
);

const IconStepBack = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="14,3 14,13 4,8" />
    <rect x="2" y="3" width="2" height="10" />
  </svg>
);

const IconPlay = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="4,2 14,8 4,14" />
  </svg>
);

const IconPause = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <rect x="3" y="2" width="3.5" height="12" />
    <rect x="9.5" y="2" width="3.5" height="12" />
  </svg>
);

const IconStepForward = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="2,3 2,13 12,8" />
    <rect x="12" y="3" width="2" height="10" />
  </svg>
);

const IconJumpEnd = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <polygon points="2,3 2,13 10,8" />
    <rect x="12" y="3" width="2" height="10" />
  </svg>
);
```

Replace button contents:
- `⏮` → `<IconJumpStart />`
- `⏪` → `<IconStepBack />`
- `{isPlaying ? '⏸' : '▶'}` → `{isPlaying ? <IconPause /> : <IconPlay />}`
- `⏩` → `<IconStepForward />`
- `⏭` → `<IconJumpEnd />`

**Changes to StepperControls.css:**

Add to `.stepper-btn`:
```css
  display: inline-flex;
  align-items: center;
  justify-content: center;
```

Add:
```css
.stepper-btn svg {
  display: block;
}
```

**Commit:** `feat(tutorials): replace unicode stepper icons with SVG`

---

### Task 2 — Agent B: Algorithm Graph Renderer + PTA Trace Enhancement

**Scope:** Add a dedicated `algorithm` graph type to GraphViewer with compact nodes (36px), directional edges, dagre layout. Enhance PTA trace step 0 with pending constraint edges.

**Files to modify:**
- `tutorials/src/content/types.ts`
- `tutorials/src/components/GraphViewer.tsx`
- `tutorials/public/content/pta-andersen/pta-andersen-basic.trace.json`

**DO NOT touch:** `AlgorithmStepper.tsx` (owned by Leader in Phase 2)

#### Part A: types.ts

Add `'algorithm'` to GraphType union (line 4):
```typescript
export type GraphType = 'cfg' | 'callgraph' | 'defuse' | 'valueflow' | 'pta' | 'algorithm';
```

#### Part B: GraphViewer.tsx

**Current structure:** The component has a `switch(graphType)` that calls shared renderers (renderCFG, renderCallGraph, etc.). Add a new `case 'algorithm'` that builds Cytoscape elements directly with compact styling.

Add `import type { ElementDefinition } from 'cytoscape';` at the top.

Add this case in the switch statement:

```typescript
case 'algorithm': {
  elements = graph.nodes.map(n => ({
    data: {
      id: n.id,
      label: n.properties?.name || n.id,
      highlighted: n.properties?.highlighted === true,
      nodeType: n.labels?.[0] || 'value',
    },
    classes: [
      n.labels?.[0] === 'location' ? 'alg-location' : 'alg-value',
      n.properties?.highlighted ? 'alg-highlighted' : '',
    ].filter(Boolean).join(' '),
  })) as ElementDefinition[];

  const edgeElements = graph.edges.map((e, i) => ({
    data: {
      id: `e${i}`,
      source: e.src,
      target: e.dst,
      label: e.properties?.label || '',
      highlighted: e.properties?.highlighted === true,
      edgeType: e.edge_type,
    },
    classes: [
      `alg-edge-${(e.edge_type || 'default').replace(/[^a-z0-9-]/gi, '')}`,
      e.properties?.highlighted ? 'alg-edge-highlighted' : '',
    ].filter(Boolean).join(' '),
  })) as ElementDefinition[];

  elements = [...elements, ...edgeElements];
  layoutName = 'dagre';
  layoutOptions = {
    name: 'dagre',
    rankDir: 'TB',
    nodeSep: 50,
    rankSep: 60,
    animate: false,
    padding: 20,
  };
  break;
}
```

After the `const cy = createCyInstance(...)` call, add algorithm-specific compact styles:

```typescript
if (graphType === 'algorithm') {
  cy.style()
    .selector('.alg-value')
    .style({
      'width': 36,
      'height': 36,
      'background-color': '#3b82f6',
      'border-width': 2,
      'border-color': '#60a5fa',
      'color': '#e0e0e0',
      'font-size': '12px',
      'text-valign': 'center',
      'text-halign': 'center',
      'shape': 'round-rectangle',
      'label': 'data(label)',
    })
    .selector('.alg-location')
    .style({
      'width': 36,
      'height': 36,
      'background-color': '#065f46',
      'border-width': 2,
      'border-color': '#10b981',
      'color': '#e0e0e0',
      'font-size': '12px',
      'text-valign': 'center',
      'text-halign': 'center',
      'shape': 'ellipse',
      'label': 'data(label)',
    })
    .selector('.alg-highlighted')
    .style({
      'border-width': 3,
      'border-color': '#f59e0b',
    })
    .selector('edge')
    .style({
      'width': 2,
      'curve-style': 'bezier',
      'target-arrow-shape': 'triangle',
      'arrow-scale': 0.8,
      'font-size': '10px',
      'text-rotation': 'autorotate',
      'label': 'data(label)',
      'color': '#a0aec0',
      'text-background-color': '#1a1a2e',
      'text-background-opacity': 0.8,
      'text-background-padding': '2px',
    })
    .selector('.alg-edge-points-to')
    .style({
      'line-color': '#10b981',
      'target-arrow-color': '#10b981',
    })
    .selector('.alg-edge-constraint')
    .style({
      'line-color': '#4a5568',
      'target-arrow-color': '#4a5568',
      'line-style': 'dashed',
    })
    .selector('.alg-edge-highlighted')
    .style({
      'line-color': '#f59e0b',
      'target-arrow-color': '#f59e0b',
      'width': 3,
    })
    .update();

  cy.layout(layoutOptions).run();
}
```

#### Part C: PTA trace enhancement

In `tutorials/public/content/pta-andersen/pta-andersen-basic.trace.json`:

**Step 0** — Add dashed constraint edges (currently `"edges": []`). Replace with:
```json
"edges": [
  { "src": "p", "dst": "x", "type": "constraint", "label": "addr" },
  { "src": "p", "dst": "q", "type": "constraint", "label": "copy" },
  { "src": "r", "dst": "y", "type": "constraint", "label": "addr" }
]
```
Update step 0's `diff.added.edges` to match.

**Steps 1-2** — Keep `points-to` edges for resolved constraints, retain `constraint` type for unresolved ones. In step 1, the copy constraint `p→q` is still unresolved:
```json
"edges": [
  { "src": "p", "dst": "x", "type": "points-to", "label": "addr" },
  { "src": "r", "dst": "y", "type": "points-to", "label": "addr" },
  { "src": "p", "dst": "q", "type": "constraint", "label": "copy" }
]
```

In step 2, all are resolved — all edges become `points-to` type (existing edges plus `q→x`).

**Steps 3** — Same as step 2 (fixed point).

**Commit:** `feat(tutorials): algorithm graph renderer with compact nodes and PTA trace edges`

---

### Task 3 — Agent C: IFDS Taint Analysis Tutorial Content

**Scope:** Replace the single placeholder step with a full 3-step tutorial.

**Files to create/modify:**
- Modify: `tutorials/public/content/ifds-taint/steps.json`
- Create: `tutorials/public/content/ifds-taint/ifds-taint-basic.trace.json`

**DO NOT touch:** Any `.tsx`, `.ts`, or `.css` files.

**Context — existing type schemas (for trace JSON):**

`IfdsState` (from `trace-types.ts`):
```typescript
export interface IfdsState {
  worklist: Array<{ func: string; d1: string; inst: string; d2: string }>;
  summaryEdges: Array<{ func: string; entryFact: string; exitFact: string }>;
  factsAt: Record<string, string[]>;
}
```

`TraceStep` fields: `id` (number), `action` (string), `explanation` (string/markdown), `highlights`, `graph` (nodes+edges), `diff`, `algorithmState`.

`AlgorithmTrace` wraps it: `{ algorithm: "ifds-taint", title, example: { code, language }, steps: TraceStep[] }`.

**steps.json** — Write 3 steps:

1. `"What is IFDS?"` — `stepType: "prose"`. Theory content explaining IFDS as graph reachability on an exploded supergraph. Include a C example:
```c
int source();
void sink(int x);
void propagate(int val) { sink(val); }
int main() { int t = source(); propagate(t); }
```

2. `"IFDS Tabulation: Step by Step"` — `stepType: "algorithm"`, `algorithmTrace: "ifds-taint-basic.trace.json"`. Brief description inviting user to step through.

3. `"Interactive: Trace Your Own Taint Flow"` — `stepType: "interactive"`, `graphType: "valueflow"`, with a default C program containing a taint flow. Include `code` and `codeLanguage: "c"`.

**Trace JSON** — Write `ifds-taint-basic.trace.json` with 5 steps:

1. **Initialize**: Create exploded supergraph nodes. Seed `<main_entry, Λ>` (zero fact).
2. **Process source() return**: Generate taint fact `t`. Path edge: `<main, Λ> → <t=source(), {t}>`.
3. **Process call to propagate()**: Map `t` to `val`. Path edge into callee.
4. **Process sink(val)**: Taint reaches sink. Create summary edge for `propagate`.
5. **Fixed point**: All edges propagated. Taint reaches sink — YES.

Graph nodes: instruction-level nodes (`main_entry`, `t=source()`, `propagate_entry`, `sink(val)`, etc.). Edges: normal flow + call/return flow edges. Use `type: "normal"` for intraprocedural, `type: "call"` for call edges, `type: "return"` for return edges.

**Commit:** `feat(tutorials): IFDS taint analysis tutorial with stepper trace`

---

### Task 4 — Agent D: Interval Analysis Tutorial Content

**Scope:** Replace the single placeholder step with a full 3-step tutorial.

**Files to create/modify:**
- Modify: `tutorials/public/content/interval-analysis/steps.json`
- Create: `tutorials/public/content/interval-analysis/interval-basic.trace.json`

**DO NOT touch:** Any `.tsx`, `.ts`, or `.css` files.

**Context — existing type schema:**

`IntervalState` (from `trace-types.ts`):
```typescript
export interface IntervalState {
  currentBlock: string;
  iteration: number;
  variables: Record<string, { lo: number | '-inf'; hi: number | '+inf' }>;
  operation?: { type: 'join' | 'widen' | 'narrow'; description: string };
}
```

**steps.json** — Write 3 steps:

1. `"What is Interval Abstract Interpretation?"` — `stepType: "prose"`. Keep and expand existing theory (abstract domain, transfer functions, join, widening, narrowing). Include example:
```c
int sum(int n) {
    int s = 0;
    for (int i = 0; i < 10; i++) { s += i; }
    return s;
}
```

2. `"Interval Fixpoint: Step by Step"` — `stepType: "algorithm"`, `algorithmTrace: "interval-basic.trace.json"`.

3. `"Interactive: Analyze Variable Bounds"` — `stepType: "interactive"`, `graphType: "cfg"`, with a default C program.

**Trace JSON** — Write `interval-basic.trace.json` with 6 steps:

1. **Initialize**: `s=[0,0]`, `i=[0,0]`, enter loop header.
2. **First iteration**: Transfer through `i++`, `s+=i`. `i=[1,1]`, `s=[0,0]` (first iteration).
3. **Join at loop header**: Join `[0,0]` with `[1,1]` → `[0,1]`.
4. **Widening**: Old `[0,1]`, new `[0,2]` → widen to `[0, +inf)`. Operation: `{ type: "widen", description: "Upper bound increasing: 1 → 2 → ∞" }`.
5. **Narrowing**: Loop condition `i < 10` narrows `i` to `[0,9]`. Operation: `{ type: "narrow", description: "Loop bound i < 10 constrains upper bound" }`.
6. **Fixed point**: Final intervals: `i ∈ [0,9]`, `s ∈ [0,45]`.

Graph: Simple CFG (entry → loop_header → loop_body → loop_exit → return). Nodes are basic blocks.

**Commit:** `feat(tutorials): interval analysis tutorial with stepper trace`

---

### Task 5 — Agent E: Memory SSA Tutorial Content

**Scope:** Replace the single placeholder step with a full 3-step tutorial.

**Files to create/modify:**
- Modify: `tutorials/public/content/memory-ssa/steps.json`
- Create: `tutorials/public/content/memory-ssa/mssa-basic.trace.json`

**DO NOT touch:** Any `.tsx`, `.ts`, or `.css` files.

**Context — existing type schema:**

`MssaState` (from `trace-types.ts`):
```typescript
export interface MssaState {
  query: string;
  walkChain: Array<{
    inst: string;
    type: 'load' | 'store' | 'phi';
    aliasQuery?: { ptr1: string; ptr2: string; result: 'may' | 'must' | 'no' };
  }>;
  pointsToContext: Record<string, string[]>;
}
```

**steps.json** — Write 3 steps:

1. `"What is Memory SSA?"` — `stepType: "prose"`. Explain SSA for memory, demand-driven clobber walking, alias queries. Include example:
```c
void foo(int *p, int *q) {
    *p = 10;
    *q = 20;
    int x = *p;  // Is this 10 or 20? Depends on alias(p,q)
}
```

2. `"Clobber Walking: Step by Step"` — `stepType: "algorithm"`, `algorithmTrace: "mssa-basic.trace.json"`.

3. `"Interactive: Explore Memory Dependencies"` — `stepType: "interactive"`, `graphType: "defuse"`, with a default C program.

**Trace JSON** — Write `mssa-basic.trace.json` with 5 steps:

1. **Query**: "What is the reaching definition for `*p` at `x = *p`?"
2. **Walk step 1**: Walk back from load. Find `*q = 20`. Alias check: `alias(p, q) = may`. May-alias — continue walking.
3. **Walk step 2**: Find `*p = 10`. Alias check: `alias(p, p) = must`. Must-alias — definite clobber.
4. **Walk step 3**: No more stores to walk past. Walk complete.
5. **Result**: Reaching definitions: `*p = 10` (must) and `*q = 20` (may).

Graph: Three instruction nodes connected by def-use edges, with alias query annotations.

**Commit:** `feat(tutorials): Memory SSA clobber walking tutorial with stepper trace`

---

## Phase 2: Layout + Subcategories (after Phase 1)

---

### Task 6 — Leader: Side-by-Side Stepper Layout

**Scope:** Rewrite `AlgorithmStepper` layout from vertical stack to side-by-side panels with pinned controls. This is the highest-impact visual change.

**Files to modify:**
- `tutorials/src/components/AlgorithmStepper.tsx`
- `tutorials/src/components/AlgorithmStepper.css`

**Prerequisites:** Task 1 (SVG controls) and Task 2 (algorithm graph type) must be merged.

**Changes to AlgorithmStepper.tsx:**

Replace the return JSX (`<div className="algorithm-stepper">` block) with:

```tsx
return (
  <div className="algorithm-stepper">
    <div className="stepper-header">
      <h3 className="stepper-action">{currentStep.action}</h3>
      <span className="stepper-step-label">Step {stepIndex + 1} / {trace.steps.length}</span>
    </div>
    <div className="stepper-panels">
      <div className="stepper-panel-left">
        <div className="stepper-graph">
          <GraphViewer graph={graph} graphType="algorithm" />
        </div>
      </div>
      <div className="stepper-panel-right">
        <StateInspector
          algorithm={trace.algorithm}
          state={currentStep.algorithmState}
          diff={currentStep.diff}
        />
      </div>
    </div>
    <StepperControls
      currentStep={stepIndex}
      totalSteps={trace.steps.length}
      isPlaying={isPlaying}
      speed={speed}
      onStepForward={stepForward}
      onStepBack={stepBack}
      onJumpToStart={jumpToStart}
      onJumpToEnd={jumpToEnd}
      onTogglePlay={togglePlay}
      onSpeedChange={setSpeed}
    />
    <details className="stepper-explanation" open>
      <summary>Explanation</summary>
      <ExplanationPanel
        stepId={currentStep.id}
        action={currentStep.action}
        explanation={currentStep.explanation}
      />
    </details>
  </div>
);
```

Note: This changes `graphType="cfg"` to `graphType="algorithm"` (from Task 2).

**Changes to AlgorithmStepper.css:**

Replace the top section (`.algorithm-stepper` through `.stepper-graph .graph-viewer`) with:

```css
.algorithm-stepper {
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg-secondary);
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 228px);
}

.algorithm-stepper-loading,
.algorithm-stepper-error {
  padding: 32px;
  text-align: center;
  color: var(--text-muted);
}

.algorithm-stepper-error {
  color: #ef4444;
}

.stepper-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border);
  min-height: 40px;
}

.stepper-action {
  color: var(--accent);
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.stepper-step-label {
  font-size: 12px;
  color: var(--text-muted);
  font-family: monospace;
  margin-left: 12px;
  flex-shrink: 0;
}

.stepper-panels {
  display: grid;
  grid-template-columns: 3fr 2fr;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.stepper-panel-left {
  border-right: 1px solid var(--border);
  overflow: hidden;
  display: flex;
}

.stepper-panel-left .stepper-graph {
  flex: 1;
  min-height: 250px;
}

.stepper-panel-left .stepper-graph .graph-viewer {
  height: 100% !important;
  min-height: unset !important;
}

.stepper-panel-right {
  overflow-y: auto;
  padding: 0;
}

.stepper-explanation {
  border-top: 1px solid var(--border);
  max-height: 200px;
  overflow-y: auto;
  flex-shrink: 0;
}

.stepper-explanation summary {
  padding: 8px 16px;
  cursor: pointer;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  user-select: none;
}

.stepper-explanation summary:hover {
  color: var(--text);
}

@media (max-width: 900px) {
  .stepper-panels {
    grid-template-columns: 1fr;
    grid-template-rows: 300px 1fr;
  }

  .stepper-panel-left {
    border-right: none;
    border-bottom: 1px solid var(--border);
  }

  .stepper-panel-right {
    max-height: 200px;
  }
}
```

Remove old `.explanation-panel`, `.stepper-graph`, `.state-inspector-container` top-level styles (they are replaced by the panel layout). Keep all inspector-specific styles (`.state-inspector` through `.walk-arrow`) unchanged.

**Verify with Playwright:** Navigate to `http://localhost:8080/tutorials/#pta-andersen/2`, confirm side-by-side layout.

**Commit:** `feat(tutorials): side-by-side stepper layout with pinned controls`

---

### Task 7 — Agent A: Subcategories + New Trace Types

**Scope:** Add subcategory support to types/registry/sidebar, and extend trace-types with 4 new algorithm state types (needed by Phase 3 agents).

**Files to modify:**
- `tutorials/src/content/types.ts`
- `tutorials/src/content/trace-types.ts`
- `tutorials/src/content/registry.ts`
- `tutorials/src/components/Sidebar.tsx`
- `tutorials/src/components/Sidebar.css`

#### Part A: types.ts

Add below existing types:
```typescript
export type Subcategory = 'pointer-analysis' | 'dataflow-analysis' | 'memory-modeling' | 'graph-foundations';
```

Add `subcategory?: Subcategory;` to `TutorialMeta` interface (after `category`).

#### Part B: trace-types.ts

Extend `AlgorithmType` union:
```typescript
export type AlgorithmType =
  | 'andersen-pta' | 'ifds-taint' | 'interval-absint' | 'memory-ssa'
  | 'kcfa-pta' | 'sparse-vf' | 'dominator-tree' | 'callgraph-construction';
```

Add 4 new state interfaces after `MssaState`:

```typescript
export interface KCfaState {
  currentContext: string[];
  worklist: Array<{ context: string[]; variable: string }>;
  pointsTo: Record<string, Record<string, string[]>>;
  contextCount: number;
}

export interface SvfState {
  currentNode: string;
  vfEdges: Array<{ src: string; dst: string; kind: 'direct' | 'store' | 'load'; processed: boolean }>;
  facts: Record<string, string[]>;
}

export interface DomState {
  processed: string[];
  idom: Record<string, string>;
  domFrontier: Record<string, string[]>;
  loopHeaders: string[];
  backEdges: Array<{ src: string; dst: string }>;
}

export interface CgState {
  algorithm: 'cha' | 'rta' | 'vta';
  resolvedCalls: Array<{ callsite: string; targets: string[]; precision: 'exact' | 'over-approx' }>;
  reachableMethods: string[];
  unresolvedCalls: string[];
}
```

Update `AlgorithmState` union:
```typescript
export type AlgorithmState =
  | PtaState | IfdsState | IntervalState | MssaState
  | KCfaState | SvfState | DomState | CgState;
```

#### Part C: registry.ts

Add subcategory info:
```typescript
import type { Category, Subcategory } from './types';

export interface SubcategoryInfo {
  id: Subcategory;
  title: string;
}

export const SUBCATEGORIES: SubcategoryInfo[] = [
  { id: 'pointer-analysis', title: 'Pointer Analysis' },
  { id: 'dataflow-analysis', title: 'Dataflow Analysis' },
  { id: 'memory-modeling', title: 'Memory Modeling' },
  { id: 'graph-foundations', title: 'Graph Foundations' },
];
```

Add `subcategory` to existing algorithm tutorials:
- `pta-andersen`: `subcategory: 'pointer-analysis'`
- `ifds-taint`: `subcategory: 'dataflow-analysis'`
- `interval-analysis`: `subcategory: 'dataflow-analysis'`
- `memory-ssa`: `subcategory: 'memory-modeling'`

#### Part D: Sidebar.tsx

Import `SUBCATEGORIES` and `Subcategory` type. When rendering the `algorithms` category, group tutorials by subcategory.

Extract the tutorial item rendering into a helper function (to avoid duplication), then:

```tsx
{isExpanded && cat.id === 'algorithms' ? (
  <div className="sidebar-subcategories">
    {SUBCATEGORIES.map(sub => {
      const subTutorials = catTutorials.filter(t => t.subcategory === sub.id);
      if (subTutorials.length === 0) return null;
      return (
        <div key={sub.id} className="sidebar-subcategory">
          <span className="sidebar-subcategory-label">{sub.title}</span>
          <ul className="sidebar-tutorial-list">
            {subTutorials.map(t => renderTutorialItem(t))}
          </ul>
        </div>
      );
    })}
  </div>
) : isExpanded ? (
  <ul className="sidebar-tutorial-list">
    {catTutorials.map(t => renderTutorialItem(t))}
  </ul>
) : null}
```

#### Part E: Sidebar.css

Add:
```css
.sidebar-subcategories {
  padding-left: 8px;
}

.sidebar-subcategory {
  margin-bottom: 4px;
}

.sidebar-subcategory-label {
  display: block;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  padding: 6px 12px 2px;
  opacity: 0.7;
}
```

**Commit:** `feat(tutorials): sidebar subcategories and new algorithm trace types`

---

## Phase 3: New Algorithm Tutorials (4 agents parallel, after Phase 2)

Each agent creates ONLY its own inspector component + content files. The Leader wires them into `StateInspector.tsx` and `registry.ts` afterward (Task 11b).

---

### Task 8 — Agent A: Dominator Tree & Loop Detection Tutorial

**Scope:** Create inspector component + tutorial content. DO NOT modify `StateInspector.tsx` or `registry.ts`.

**Files to create:**
- `tutorials/src/components/state-inspectors/DomInspector.tsx`
- `tutorials/public/content/dominator-tree/steps.json`
- `tutorials/public/content/dominator-tree/dom-basic.trace.json`

**Context — DomState type (from trace-types.ts, added in Task 7):**
```typescript
export interface DomState {
  processed: string[];
  idom: Record<string, string>;
  domFrontier: Record<string, string[]>;
  loopHeaders: string[];
  backEdges: Array<{ src: string; dst: string }>;
}
```

**Context — inspector pattern (follow PtaInspector.tsx):**
- Import `{ AnimatePresence, motion }` from `'motion/react'`
- Import `DomState, TraceDiff` from `'../../content/trace-types'`
- Render sections: Processed Blocks, Immediate Dominators (table), Dominance Frontier, Loop Headers, Back Edges
- Use CSS classes from `AlgorithmStepper.css` (`.state-inspector`, `.inspector-section`, `.inspector-section h4`, `.inspector-empty`)
- Add component-specific CSS classes inline with the same naming pattern (`.dom-*`)

**DomInspector.tsx sections:**
1. **Processed** — List of block names with badges
2. **Immediate Dominators** — Table: `block → idom(block)`, highlight changed entries
3. **Loop Headers** — Green badges for loop header blocks
4. **Back Edges** — List of `src → dst` with amber badge

**steps.json (3 steps):**
1. Theory: Explain dominators, post-dominators, dominance frontiers, loop detection via back edges. Example:
```c
int sum(int n) {
    int s = 0;
    for (int i = 0; i < n; i++) s += i;
    return s;
}
```
2. Stepper: `{ stepType: "algorithm", algorithmTrace: "dom-basic.trace.json" }`
3. Interactive: `{ stepType: "interactive", graphType: "cfg" }`

**Trace JSON (5 steps):**
CFG: `entry → loop_header → loop_body → loop_exit → return`. Plus back edge `loop_body → loop_header`.

1. **Init**: Start DFS, process entry block. `idom(entry) = entry`.
2. **Process loop_header**: `idom(loop_header) = entry`.
3. **Process loop_body**: `idom(loop_body) = loop_header`.
4. **Detect back edge**: Edge `loop_body → loop_header` is a back edge (dst dominates src). Mark `loop_header` as loop header.
5. **Compute dominance frontier**: `DF(loop_header) = {loop_header}` (join point). Final idom tree complete.

**Commit:** `feat(tutorials): dominator tree & loop detection tutorial`

---

### Task 9 — Agent B: Callgraph Construction Tutorial (CHA/RTA/VTA)

**Scope:** Create inspector component + tutorial content. DO NOT modify `StateInspector.tsx` or `registry.ts`.

**Files to create:**
- `tutorials/src/components/state-inspectors/CgInspector.tsx`
- `tutorials/public/content/callgraph-construction/steps.json`
- `tutorials/public/content/callgraph-construction/cg-basic.trace.json`

**Context — CgState type (from trace-types.ts, added in Task 7):**
```typescript
export interface CgState {
  algorithm: 'cha' | 'rta' | 'vta';
  resolvedCalls: Array<{ callsite: string; targets: string[]; precision: 'exact' | 'over-approx' }>;
  reachableMethods: string[];
  unresolvedCalls: string[];
}
```

**CgInspector.tsx sections:**
1. **Algorithm** — Badge showing current algorithm (CHA/RTA/VTA) with distinct colors
2. **Resolved Calls** — Table: callsite → targets, with precision badge (exact=green, over-approx=amber)
3. **Reachable Methods** — List of function names
4. **Unresolved Calls** — List of remaining indirect calls (shrinks as algorithm refines)

**steps.json (3 steps):**
1. Theory: CHA vs RTA vs VTA, why precision matters for call edges. Example with function pointers:
```c
void dog_bark() { }
void cat_meow() { }
typedef void (*SpeakFn)(void);
void make_noise(SpeakFn fn) { fn(); }
int main() { make_noise(dog_bark); }
```
2. Stepper: Shows progressive refinement across 3 algorithms
3. Interactive: `{ stepType: "interactive", graphType: "callgraph" }`

**Trace JSON (6 steps):**
1. **Init**: Program has indirect call `fn()` in `make_noise`.
2. **CHA**: All compatible function-pointer types → targets = `{dog_bark, cat_meow}`. Over-approx.
3. **RTA**: Only types instantiated at runtime → `{dog_bark, cat_meow}` (both are address-taken). Still over-approx.
4. **VTA**: Track value flow of `fn` parameter. `main` passes `dog_bark` only → targets = `{dog_bark}`. Exact.
5. **Comparison**: CHA=2 targets, RTA=2, VTA=1. Precision improved.
6. **Final callgraph**: `main → make_noise → dog_bark` only.

**Commit:** `feat(tutorials): callgraph construction CHA/RTA/VTA tutorial`

---

### Task 10 — Agent C: Context-Sensitive PTA (k-CFA) Tutorial

**Scope:** Create inspector component + tutorial content. DO NOT modify `StateInspector.tsx` or `registry.ts`.

**Files to create:**
- `tutorials/src/components/state-inspectors/KCfaInspector.tsx`
- `tutorials/public/content/pta-kcfa/steps.json`
- `tutorials/public/content/pta-kcfa/kcfa-basic.trace.json`

**Context — KCfaState type (from trace-types.ts, added in Task 7):**
```typescript
export interface KCfaState {
  currentContext: string[];
  worklist: Array<{ context: string[]; variable: string }>;
  pointsTo: Record<string, Record<string, string[]>>;
  contextCount: number;
}
```

**KCfaInspector.tsx sections:**
1. **Current Context** — Call-site chain display (e.g., `[main@L5] → [identity@L1]`)
2. **Worklist** — List of `(context, variable)` pairs with context shown as breadcrumbs
3. **Points-To Sets (per context)** — Nested table: context → variable → locations. Highlight changed entries.
4. **Context Count** — Counter badge showing total cloned contexts

**steps.json (3 steps):**
1. Theory: Why context insensitivity loses precision. Classic identity function:
```c
int *identity(int *x) { return x; }
int main() {
    int a, b;
    int *p = identity(&a);  // call-site 1
    int *q = identity(&b);  // call-site 2
}
```
CI merges both calls → `pts(p) = pts(q) = {a,b}`. 1-CFA clones per call-site → `pts(p)={a}`, `pts(q)={b}`.

2. Stepper: `{ stepType: "algorithm", algorithmTrace: "kcfa-basic.trace.json" }`
3. Interactive: `{ stepType: "interactive", graphType: "pta" }`

**Trace JSON (6 steps):**
1. **Extract constraints** (same as Andersen's but noting 2 call-sites)
2. **CI solve**: Both calls merge. `pts(p) = {a,b}`, `pts(q) = {a,b}`. 2 spurious edges.
3. **Switch to 1-CFA**: Clone `identity()` per call-site. Context count: 2.
4. **Solve context [main@L5]**: `identity` receives `&a` → `pts(p)={a}` in this context.
5. **Solve context [main@L6]**: `identity` receives `&b` → `pts(q)={b}` in this context.
6. **Compare**: CI: 4 points-to edges (2 spurious). 1-CFA: 2 edges (exact).

**Commit:** `feat(tutorials): context-sensitive PTA (k-CFA) tutorial`

---

### Task 11a — Agent D: Sparse Value-Flow Analysis Tutorial

**Scope:** Create inspector component + tutorial content. DO NOT modify `StateInspector.tsx` or `registry.ts`.

**Files to create:**
- `tutorials/src/components/state-inspectors/SvfInspector.tsx`
- `tutorials/public/content/sparse-valueflow/steps.json`
- `tutorials/public/content/sparse-valueflow/svf-basic.trace.json`

**Context — SvfState type (from trace-types.ts, added in Task 7):**
```typescript
export interface SvfState {
  currentNode: string;
  vfEdges: Array<{ src: string; dst: string; kind: 'direct' | 'store' | 'load'; processed: boolean }>;
  facts: Record<string, string[]>;
}
```

**SvfInspector.tsx sections:**
1. **Current Node** — Highlighted current processing node
2. **Value-Flow Edges** — Table: src → dst, kind badge (direct=gray, store=amber, load=blue), processed checkbox
3. **Facts at Nodes** — Map: node → set of reaching facts/values. Highlight changed entries.

**steps.json (3 steps):**
1. Theory: Dense vs sparse dataflow. SSA + def-use chains skip irrelevant program points:
```c
int main() {
    int x = source();
    int y = 42;
    int z = x + 1;
    sink(z);
}
```
Dense visits ALL statements. Sparse follows value-flow: `source()→x→z→sink()`, skipping `y=42`.

2. Stepper: `{ stepType: "algorithm", algorithmTrace: "svf-basic.trace.json" }`
3. Interactive: `{ stepType: "interactive", graphType: "valueflow" }`

**Trace JSON (5 steps):**
1. **Build value-flow graph**: Construct def-use edges from SSA form.
2. **Seed source**: Mark `source()` return as "tainted". Start propagation.
3. **Propagate to x**: Direct edge `source()→x`. Facts at `x`: `{tainted}`.
4. **Propagate to z**: Direct edge `x→z` (through `x+1`). Facts at `z`: `{tainted}`. Note: `y=42` was never visited.
5. **Reach sink**: Direct edge `z→sink()`. Taint found at sink. Complete.

**Commit:** `feat(tutorials): sparse value-flow analysis tutorial`

---

### Task 11b — Leader: Integration Wiring

**Scope:** Wire 4 new inspectors into `StateInspector.tsx` and register 4 new tutorials in `registry.ts`.

**Prerequisites:** Tasks 7, 8, 9, 10, 11a all complete.

**Files to modify:**
- `tutorials/src/components/StateInspector.tsx`
- `tutorials/src/content/registry.ts`

#### StateInspector.tsx

Add imports:
```typescript
import DomInspector from './state-inspectors/DomInspector';
import CgInspector from './state-inspectors/CgInspector';
import KCfaInspector from './state-inspectors/KCfaInspector';
import SvfInspector from './state-inspectors/SvfInspector';
import type { KCfaState, SvfState, DomState, CgState } from '../content/trace-types';
```

Add cases to the switch:
```typescript
case 'dominator-tree':
  return <DomInspector state={state as DomState} diff={diff} />;
case 'callgraph-construction':
  return <CgInspector state={state as CgState} diff={diff} />;
case 'kcfa-pta':
  return <KCfaInspector state={state as KCfaState} diff={diff} />;
case 'sparse-vf':
  return <SvfInspector state={state as SvfState} diff={diff} />;
```

#### registry.ts

Add 4 new tutorials to the `TUTORIALS` array (after `memory-ssa`):

```typescript
{
  id: 'pta-kcfa',
  title: 'Context-Sensitive PTA (k-CFA)',
  description: 'See how cloning functions per call-site eliminates spurious points-to edges.',
  difficulty: 'advanced',
  mode: 'browser',
  category: 'algorithms',
  subcategory: 'pointer-analysis',
  prerequisites: ['pta-andersen'],
},
{
  id: 'sparse-valueflow',
  title: 'Sparse Value-Flow Analysis',
  description: 'Watch how SSA and def-use chains enable efficient sparse dataflow propagation.',
  difficulty: 'advanced',
  mode: 'browser',
  category: 'algorithms',
  subcategory: 'memory-modeling',
},
{
  id: 'dominator-tree',
  title: 'Dominator Trees & Loop Detection',
  description: 'Compute immediate dominators, identify back-edges, and build the loop nesting tree.',
  difficulty: 'intermediate',
  mode: 'browser',
  category: 'algorithms',
  subcategory: 'graph-foundations',
},
{
  id: 'callgraph-construction',
  title: 'Callgraph Construction (CHA/RTA/VTA)',
  description: 'See how callgraph precision improves from CHA through RTA to VTA.',
  difficulty: 'intermediate',
  mode: 'browser',
  category: 'algorithms',
  subcategory: 'graph-foundations',
},
```

**Verify:** `cd tutorials && npm run build` — no TypeScript errors. Navigate to each tutorial in browser.

**Commit:** `feat(tutorials): wire 4 new algorithm tutorials into registry and inspector`

---

## Final Verification (Leader)

1. `cd tutorials && npm run build` — clean build
2. Navigate through all 8 algorithm tutorials in browser
3. Verify sidebar subcategory groupings render correctly
4. Test responsive layout at < 900px width
5. Step through each algorithm stepper to verify controls, graph, and state inspector
