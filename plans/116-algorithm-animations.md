# Algorithm Animation Tutorials — Implementation Plan

**Goal:** Build 4 interactive algorithm stepper tutorials (PTA, IFDS, Intervals, Memory SSA) with pre-computed JSON traces, Motion animations, and Cytoscape graph visualization.

**Architecture:** Pre-computed JSON traces drive React components that animate algorithm state transitions using Motion + Cytoscape. Traces are static JSON loaded at runtime.

**Design Doc:** `plans/116-algorithm-animations-design.md`

---

## Agent Team Structure

```
Leader (setup + integration + testing)
  ├── Agent A: Types & Registry (foundational, blocks B+C)
  ├── Agent B: Stepper Components (core UI, depends on A)
  ├── Agent C: State Inspectors (4 algorithm inspectors, depends on A)
  ├── Agent D: Tutorial Content (all JSON files, no code deps)
  └── Agent E: Rust Trace Crate (saf-trace scaffold, Docker only)
```

### Execution Phases

| Phase | Who | What | Depends On |
|-------|-----|------|------------|
| 1 | Leader | `npm install motion`, verify | nothing |
| 1 | Agent A | Types + Registry | nothing |
| 1 | Agent D | All tutorial JSON content | nothing |
| 1 | Agent E | saf-trace Rust crate scaffold | nothing |
| 2 | Agent B | Stepper components + orchestrator + CSS | Agent A done |
| 2 | Agent C | 4 state inspector components | Agent A done |
| 3 | Leader | StepContent integration, Playwright testing, PROGRESS.md | Agents B, C, D done |

---

## Leader Tasks

### Phase 1: Add Motion dependency

```bash
cd tutorials && npm install motion
```

Verify: `npm ls motion` shows `motion@12.x.x` under `saf-tutorials`.

Commit:
```bash
git add tutorials/package.json package-lock.json
git commit -m "chore(tutorials): add motion dependency for algorithm animations"
```

### Phase 3: Integrate AlgorithmStepper into StepContent

**File:** `tutorials/src/components/StepContent.tsx`

After Agents B, C, D complete, modify `StepContent.tsx` to detect algorithm steps and render the `AlgorithmStepper` component.

Add import:
```tsx
import AlgorithmStepper from './AlgorithmStepper';
```

Add detection (after `isInteractive` on line 16):
```tsx
const isAlgorithm = step.stepType === 'algorithm' && step.algorithmTrace;
```

Add rendering (before the `isInteractive` branch):
```tsx
{isAlgorithm ? (
  <AlgorithmStepper tutorialId={tutorialId} traceFile={step.algorithmTrace!} />
) : isInteractive ? (
```

Verify build: `cd tutorials && npx tsc --noEmit`

Commit:
```bash
git add tutorials/src/components/StepContent.tsx
git commit -m "feat(tutorials): integrate AlgorithmStepper into StepContent"
```

### Phase 3: Playwright integration testing

Start the tutorials dev server, then run Playwright tests covering all key functionality. Tests should be performed using the Playwright MCP tools (browser_navigate, browser_snapshot, browser_click, etc.).

**Test 1: Algorithms category in sidebar**
- Navigate to `http://localhost:5173/`
- Verify sidebar contains "Algorithms" section
- Verify all 4 tutorials appear: PTA, IFDS, Intervals, Memory SSA

**Test 2: PTA tutorial prose steps load**
- Navigate to `http://localhost:5173/#pta-andersen/0`
- Verify page shows "What is Pointer Analysis?" heading
- Navigate to step 2 (`#pta-andersen/1`)
- Verify the example code block renders

**Test 3: AlgorithmStepper renders**
- Navigate to `http://localhost:5173/#pta-andersen/2`
- Verify the stepper loads (no "Loading..." or error message)
- Verify step counter shows "Step 1 / 4"
- Verify graph container is visible with nodes

**Test 4: Stepper controls work**
- On the PTA stepper step, click the step-forward button
- Verify counter changes to "Step 2 / 4"
- Click step-forward again → "Step 3 / 4"
- Click step-back → "Step 2 / 4"
- Click jump-to-end → "Step 4 / 4"
- Click jump-to-start → "Step 1 / 4"

**Test 5: PTA inspector shows algorithm state**
- On step 2, verify "Points-To Sets" section is visible
- Verify worklist section shows rank information
- Verify constraints section shows constraint types

**Test 6: Placeholder tutorials load**
- Navigate to `http://localhost:5173/#ifds-taint/0`
- Verify "What is IFDS?" heading appears
- Navigate to `http://localhost:5173/#interval-analysis/0`
- Verify heading appears
- Navigate to `http://localhost:5173/#memory-ssa/0`
- Verify heading appears

**Test 7: No console errors**
- Check browser console for errors on each page
- Ignore known warnings (Cytoscape deprecation warnings are OK)

### Phase 3: Update PROGRESS.md

Add plan 116 to index, update session log.

### Phase 3: Docker — build saf-trace in Docker

After Agent E completes, verify the Rust crate compiles:
```bash
docker compose run --rm dev sh -c 'cargo check -p saf-trace'
```

---

## Agent A: Types & Registry

**Summary:** Add trace type definitions and register the 4 new algorithm tutorials. This is foundational — Agents B and C depend on these types.

**Files to create:**
- `tutorials/src/content/trace-types.ts`

**Files to modify:**
- `tutorials/src/content/types.ts`
- `tutorials/src/content/registry.ts`

### 1. Extend TutorialStep type

In `tutorials/src/content/types.ts`, add two fields to the `TutorialStep` interface:

```typescript
export interface TutorialStep {
  // ... existing fields unchanged ...
  stepType?: 'prose' | 'interactive' | 'algorithm';
  algorithmTrace?: string;
}
```

Also add `'algorithms'` to the `Category` type union:

```typescript
export type Category = 'getting-started' | 'memory-safety' | 'information-flow' | 'algorithms' | 'advanced';
```

### 2. Create trace type definitions

Create `tutorials/src/content/trace-types.ts` with all TypeScript interfaces for the trace JSON format:

```typescript
export type AlgorithmType = 'andersen-pta' | 'ifds-taint' | 'interval-absint' | 'memory-ssa';

export interface TraceNode {
  id: string;
  label: string;
  type: string;
  properties?: Record<string, unknown>;
}

export interface TraceEdge {
  src: string;
  dst: string;
  label?: string;
  type: string;
}

export interface TraceGraphState {
  nodes: TraceNode[];
  edges: TraceEdge[];
}

export interface TraceDiff {
  added: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
  removed: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
  changed: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
}

export interface PtaState {
  worklist: Array<{ rank: number; values: string[] }>;
  pointsTo: Record<string, string[]>;
  constraints: Array<{
    type: 'addr' | 'copy' | 'load' | 'store' | 'gep';
    from: string;
    to: string;
    processed: boolean;
  }>;
}

export interface IfdsState {
  worklist: Array<{ func: string; d1: string; inst: string; d2: string }>;
  summaryEdges: Array<{ func: string; entryFact: string; exitFact: string }>;
  factsAt: Record<string, string[]>;
}

export interface IntervalState {
  currentBlock: string;
  iteration: number;
  variables: Record<string, { lo: number | '-inf'; hi: number | '+inf' }>;
  operation?: { type: 'join' | 'widen' | 'narrow'; description: string };
}

export interface MssaState {
  query: string;
  walkChain: Array<{
    inst: string;
    type: 'load' | 'store' | 'phi';
    aliasQuery?: { ptr1: string; ptr2: string; result: 'may' | 'must' | 'no' };
  }>;
  pointsToContext: Record<string, string[]>;
}

export type AlgorithmState = PtaState | IfdsState | IntervalState | MssaState;

export interface TraceStep {
  id: number;
  action: string;
  explanation: string;
  highlights: { nodes: string[]; edges: Array<{ src: string; dst: string }> };
  graph: TraceGraphState;
  diff: TraceDiff;
  algorithmState: AlgorithmState;
}

export interface AlgorithmTrace {
  algorithm: AlgorithmType;
  title: string;
  example: { code: string; language: string };
  steps: TraceStep[];
}
```

### 3. Add Algorithms category and tutorials to registry

In `tutorials/src/content/registry.ts`:

Add to `CATEGORIES` array (insert before 'advanced'):
```typescript
{
  id: 'algorithms',
  title: 'Algorithms',
  description: 'Step through SAF\'s core analysis algorithms and watch them solve.',
  icon: '📊',
},
```

Add 4 tutorials to `TUTORIALS` array (insert before `custom-analyzer`):
```typescript
{
  id: 'pta-andersen',
  title: 'Pointer Analysis (Andersen\'s)',
  description: 'Watch inclusion-based constraint solving: worklist processing, points-to set propagation, and cycle detection.',
  difficulty: 'intermediate',
  mode: 'browser',
  category: 'algorithms',
},
{
  id: 'ifds-taint',
  title: 'IFDS Taint Analysis',
  description: 'Follow path edges through the IFDS tabulation algorithm as tainted data flows between functions.',
  difficulty: 'intermediate',
  mode: 'browser',
  category: 'algorithms',
},
{
  id: 'interval-analysis',
  title: 'Interval Abstract Interpretation',
  description: 'See how abstract intervals widen at loop headers and narrow to precise bounds.',
  difficulty: 'intermediate',
  mode: 'browser',
  category: 'algorithms',
},
{
  id: 'memory-ssa',
  title: 'Memory SSA & Clobber Walking',
  description: 'Trace demand-driven backward walks through memory definitions with alias queries.',
  difficulty: 'advanced',
  mode: 'browser',
  category: 'algorithms',
},
```

### 4. Verify and commit

```bash
cd tutorials && npx tsc --noEmit
git add tutorials/src/content/types.ts tutorials/src/content/trace-types.ts tutorials/src/content/registry.ts
git commit -m "feat(tutorials): add algorithm trace types and Algorithms registry category"
```

---

## Agent B: Stepper Components

**Summary:** Build the core stepper UI: transport controls, explanation panel, state inspector router, and the main AlgorithmStepper orchestrator with CSS.

**Depends on:** Agent A (trace types must exist)

**Files to create:**
- `tutorials/src/components/StepperControls.tsx`
- `tutorials/src/components/StepperControls.css`
- `tutorials/src/components/ExplanationPanel.tsx`
- `tutorials/src/components/StateInspector.tsx`
- `tutorials/src/components/AlgorithmStepper.tsx`
- `tutorials/src/components/AlgorithmStepper.css`

**Existing files to understand (read but don't modify):**
- `tutorials/src/components/GraphViewer.tsx` — reuse for graph rendering
- `tutorials/src/content/trace-types.ts` — type imports (created by Agent A)
- `packages/shared/src/types/property-graph.ts` — `PropertyGraph`, `PgNode`, `PgEdge` types

### 1. StepperControls — transport bar

Create `tutorials/src/components/StepperControls.tsx` — play/pause, step forward/back, jump to start/end, speed selector, step counter.

Props interface:
```typescript
interface StepperControlsProps {
  currentStep: number;
  totalSteps: number;
  isPlaying: boolean;
  speed: number;
  onStepForward: () => void;
  onStepBack: () => void;
  onJumpToStart: () => void;
  onJumpToEnd: () => void;
  onTogglePlay: () => void;
  onSpeedChange: (speed: number) => void;
}
```

Key behaviors:
- Auto-play timer: `setInterval(onStepForward, 1500 / speed)` when playing
- Stop auto-play when reaching last step
- Speed cycles through `[0.5, 1, 2]` on button click
- Disable back buttons at step 0, forward buttons at last step

Create `tutorials/src/components/StepperControls.css` — flexbox layout with left (buttons), center (counter), right (speed).

### 2. ExplanationPanel — animated markdown

Create `tutorials/src/components/ExplanationPanel.tsx` using Motion's `AnimatePresence` for crossfade:

```tsx
import { AnimatePresence, motion } from 'motion/react';
import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
```

Props: `stepId: number, action: string, explanation: string`

Use `AnimatePresence mode="wait"` with `motion.div` keyed by `stepId`. Show `action` as a heading, `explanation` as markdown prose. Transition: 250ms fade with 8px y-shift.

### 3. StateInspector — router

Create `tutorials/src/components/StateInspector.tsx` — switches on `algorithm` type to render the correct inspector component:

```typescript
import type { AlgorithmType, AlgorithmState, TraceDiff } from '../content/trace-types';
```

Switch cases:
- `'andersen-pta'` → `<PtaInspector state={...} diff={...} />`
- `'ifds-taint'` → `<IfdsInspector state={...} diff={...} />`
- `'interval-absint'` → `<IntervalInspector state={...} />`
- `'memory-ssa'` → `<MssaInspector state={...} />`

**Note:** The 4 inspector components are built by Agent C. Import them from `./state-inspectors/`. If Agent C hasn't finished yet, create stub imports that will resolve once their files exist.

### 4. AlgorithmStepper — main orchestrator

Create `tutorials/src/components/AlgorithmStepper.tsx`:

Props: `tutorialId: string, traceFile: string`

Responsibilities:
1. **Load trace JSON** via `fetch()` from `${BASE_URL}content/algorithms/${tutorialId}/${traceFile}`
2. **Manage step state**: `stepIndex`, `isPlaying`, `speed`
3. **Convert trace graph to PropertyGraph** for Cytoscape (map `TraceNode`/`TraceEdge` to `PgNode`/`PgEdge`)
4. **Keyboard shortcuts**: ArrowRight/l = forward, ArrowLeft/h = back, Space = play/pause
5. **Wire children**: ExplanationPanel + GraphViewer + StateInspector + StepperControls

Layout (vertical stack):
```
┌──────────────────────┐
│ ExplanationPanel     │ max-height: 180px, scrollable
├──────────────────────┤
│ GraphViewer          │ height: 300px
├──────────────────────┤
│ StateInspector       │ max-height: 220px, scrollable
├──────────────────────┤
│ StepperControls      │ fixed height transport bar
└──────────────────────┘
```

Important: When passing to `GraphViewer`, use `graphType="cfg"` as a default layout. The trace graph is a custom format but the `cfg` layout (dagre) works well for algorithm visualization.

### 5. AlgorithmStepper.css — all styles

Create `tutorials/src/components/AlgorithmStepper.css` containing styles for:
- `.algorithm-stepper` — outer container with border, border-radius, dark background
- `.explanation-panel` — padding, border-bottom, max-height
- `.explanation-action` — accent color heading for current action
- `.stepper-graph` — fixed 300px height
- `.state-inspector` — shared inspector styles (section headings, empty state)
- All PTA inspector styles (`.worklist-waves`, `.pts-table`, `.pts-row`, `.constraint-row`)
- All IFDS inspector styles (`.path-edge`, `.summary-edge`, `.facts-row`)
- All Interval inspector styles (`.interval-bars`, `.interval-bar-track`, `.interval-bar-fill`)
- All MSSA inspector styles (`.walk-chain`, `.walk-entry`, `.walk-alias`)
- Loading/error states

Use CSS variables for theme consistency: `var(--bg-primary)`, `var(--bg-secondary)`, `var(--border)`, `var(--text)`, `var(--text-muted)`, `var(--accent)`.

### 6. Verify and commit

```bash
cd tutorials && npx tsc --noEmit
git add tutorials/src/components/StepperControls.tsx tutorials/src/components/StepperControls.css \
       tutorials/src/components/ExplanationPanel.tsx tutorials/src/components/StateInspector.tsx \
       tutorials/src/components/AlgorithmStepper.tsx tutorials/src/components/AlgorithmStepper.css
git commit -m "feat(tutorials): add AlgorithmStepper, StepperControls, ExplanationPanel, StateInspector"
```

---

## Agent C: State Inspectors

**Summary:** Build the 4 algorithm-specific state inspector components that visualize internal data structures (worklists, points-to sets, intervals, etc.) with Motion animations.

**Depends on:** Agent A (trace types must exist)

**Files to create:**
- `tutorials/src/components/state-inspectors/PtaInspector.tsx`
- `tutorials/src/components/state-inspectors/IfdsInspector.tsx`
- `tutorials/src/components/state-inspectors/IntervalInspector.tsx`
- `tutorials/src/components/state-inspectors/MssaInspector.tsx`

**Key imports all inspectors share:**
```typescript
import { AnimatePresence, motion } from 'motion/react';
// Types from: '../../content/trace-types'
```

**Shared Motion patterns:**
- Row enter: `initial={{ opacity: 0, x: -20 }}` slide-in
- Row highlight: `animate={{ backgroundColor: changedNodes.has(id) ? 'rgba(250, 204, 21, 0.15)' : 'transparent' }}`
- Row reorder: `layout` prop on `motion.div`
- Enter/exit: Wrap in `AnimatePresence`

### 1. PtaInspector

Props: `state: PtaState, diff: { nodes: string[]; edges: Array<{ src: string; dst: string }> }`

Three sections:
- **Worklist**: Render `state.worklist` as horizontal rank groups. Empty = "Fixed point reached".
- **Points-To Sets**: Table of `value → { locations }`. Use `motion.div` with `layout` for row animations. Highlight changed rows (compare against `diff.nodes`). Show "changed" badge.
- **Constraints**: Checklist with type label, from→to, processed checkmark.

### 2. IfdsInspector

Props: `state: IfdsState, diff: { nodes: string[]; edges: ... }`

Three sections:
- **Worklist**: Show first 5 path edges as `(func, d1, inst, d2)` tuples. "+N more" if truncated.
- **Summary Edges**: Per-function `(entryFact) → (exitFact)`.
- **Facts at Instructions**: Animated rows with `motion.div layout`. Highlight changed instructions.

### 3. IntervalInspector

Props: `state: IntervalState` (no diff needed — visual bars show changes via animation)

Two sections:
- **Variable interval bars**: Each variable gets a horizontal bar. Compute shared visual range across all variables. Use `motion.div animate={{ left, width }}` with spring transition for dramatic widening effects. Show `[lo, hi]` label. Format infinity as unicode `∞`.
- **Operation log**: If `state.operation` exists, show type (WIDEN/NARROW/JOIN) with color coding (red=widen, green=narrow, blue=join) and description.

### 4. MssaInspector

Props: `state: MssaState`

Three sections:
- **Query**: Italic text showing the human-readable clobber query.
- **Clobber Walk**: Vertical chain of def entries. Each entry shows instruction, type badge (load/store/phi), and optional alias query result (color-coded: yellow=may, red=must, green=no). Down arrows between entries. Use `motion.div` with staggered entry (`delay: i * 0.1`).
- **Points-To Context**: Simple `ptr → { locations }` table (same style as PTA inspector).

### 5. Verify and commit

```bash
cd tutorials && npx tsc --noEmit
git add tutorials/src/components/state-inspectors/
git commit -m "feat(tutorials): add PTA, IFDS, Interval, and MSSA state inspector components"
```

---

## Agent D: Tutorial Content

**Summary:** Create all tutorial JSON content files — 4 tutorials with `steps.json` and a hand-crafted PTA trace for testing.

**No code dependencies.** This agent only creates files in `tutorials/public/content/algorithms/`.

**Files to create:**
- `tutorials/public/content/algorithms/pta-andersen/steps.json`
- `tutorials/public/content/algorithms/pta-andersen/pta-andersen-basic.trace.json`
- `tutorials/public/content/algorithms/ifds-taint/steps.json`
- `tutorials/public/content/algorithms/interval-analysis/steps.json`
- `tutorials/public/content/algorithms/memory-ssa/steps.json`

### 1. PTA Andersen tutorial — steps.json

Create `tutorials/public/content/algorithms/pta-andersen/steps.json` with 3 steps:

1. **"What is Pointer Analysis?"** — prose explaining the problem, why it matters, Andersen's approach
2. **"Our Example Program"** — prose + code block showing `int *p = &x; int *q = p; int *r = &y;` and the constraints extracted
3. **"Andersen's PTA: Step by Step"** — `stepType: "algorithm"`, `algorithmTrace: "pta-andersen-basic.trace.json"`

### 2. PTA trace JSON — hand-crafted

Create `tutorials/public/content/algorithms/pta-andersen/pta-andersen-basic.trace.json` — a 4-step trace for the basic 3-variable example:

**Step 0**: "Extract constraints" — Show all 5 nodes (p, q, r, x, y), no edges yet. Worklist seeded with `[{rank:0, values:["p","r"]}, {rank:1, values:["q"]}]`. All constraints pending.

**Step 1**: "Initialize from Addr constraints" — Add edges p→x and r→y. Points-to: `{p:["x"], r:["y"]}`. Addr constraints processed. Worklist: only `[{rank:1, values:["q"]}]` remains.

**Step 2**: "Process wave at rank 1: propagate copy q ⊇ p" — Add edge q→x. Points-to: `{p:["x"], q:["x"], r:["y"]}`. All constraints processed. Worklist empty.

**Step 3**: "Fixed point reached" — Same graph, empty worklist. Explanation summarizes: p and q alias (both → x), r is independent.

The trace JSON format must match the `AlgorithmTrace` interface from `trace-types.ts`:
```json
{
  "algorithm": "andersen-pta",
  "title": "...",
  "example": { "code": "...", "language": "c" },
  "steps": [
    {
      "id": 0,
      "action": "...",
      "explanation": "... (markdown) ...",
      "highlights": { "nodes": [...], "edges": [...] },
      "graph": { "nodes": [...], "edges": [...] },
      "diff": { "added": {...}, "removed": {...}, "changed": {...} },
      "algorithmState": { "worklist": [...], "pointsTo": {...}, "constraints": [...] }
    }
  ]
}
```

Each node: `{ "id": "p", "label": "p", "type": "value" }` or `{ "id": "x", "label": "x", "type": "location" }`
Each edge: `{ "src": "p", "dst": "x", "type": "points-to", "label": "addr" }`

### 3. IFDS placeholder tutorial

Create `tutorials/public/content/algorithms/ifds-taint/steps.json` — single prose step:
- Title: "What is IFDS?"
- Content: Explain IFDS as graph reachability on exploded supergraph (Reps/Horwitz/Sagiv POPL 1995), used for taint analysis. Note: interactive stepper coming soon.

### 4. Interval placeholder tutorial

Create `tutorials/public/content/algorithms/interval-analysis/steps.json` — single prose step:
- Title: "What is Interval Abstract Interpretation?"
- Content: Explain forward fixpoint iteration, widening at loop headers, narrowing. Used for overflow/null detection. Note: interactive stepper coming soon.

### 5. Memory SSA placeholder tutorial

Create `tutorials/public/content/algorithms/memory-ssa/steps.json` — single prose step:
- Title: "What is Memory SSA?"
- Content: Explain Memory SSA def chains, clobber walking with PTA alias queries. Bridges pointer analysis to precise value tracking. Note: interactive stepper coming soon.

### 6. Commit

```bash
git add tutorials/public/content/algorithms/
git commit -m "feat(tutorials): add PTA tutorial with trace + IFDS/Interval/MSSA placeholders"
```

---

## Agent E: Rust Trace Crate

**Summary:** Create the `saf-trace` crate — a CLI binary that will generate algorithm trace JSON from `.ll` files. This task scaffolds the crate with CLI parsing and a manifest format. The actual PTA trace generation logic is a stretch goal.

**No frontend dependencies.** This agent works entirely in Rust and must compile inside Docker.

**Files to create:**
- `crates/saf-trace/Cargo.toml`
- `crates/saf-trace/src/main.rs`
- `crates/saf-trace/traces.toml`

**Files to modify:**
- `Cargo.toml` (root — add workspace member)

### 1. Add to workspace

In root `Cargo.toml`, add `"crates/saf-trace"` to `members`:

```toml
members = [
  "crates/saf-core",
  "crates/saf-frontends",
  "crates/saf-analysis",
  "crates/saf-cli",
  "crates/saf-python",
  "crates/saf-test-utils",
  "crates/saf-bench",
  "crates/saf-wasm",
  "crates/saf-trace",
]
```

### 2. Create Cargo.toml

Model after `crates/saf-bench/Cargo.toml`:

```toml
[package]
name = "saf-trace"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Algorithm trace generator for interactive tutorial animations"

[[bin]]
name = "saf-trace"
path = "src/main.rs"

[dependencies]
saf-core = { workspace = true }
saf-frontends = { workspace = true, features = ["llvm-18"] }
saf-analysis = { workspace = true }
clap = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = "0.8"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

Add `toml = "0.8"` to `[workspace.dependencies]` in root `Cargo.toml` if not already present.

### 3. Create main.rs

CLI with clap: `--algorithm`, `--input`, `--output`, `--all`, `--output-dir`, `--validate`.

Match on algorithm: `"pta"`, `"ifds"`, `"absint"`, `"mssa"` — all print "not yet implemented" for now.

For `--all`, parse `traces.toml` manifest using the `toml` crate.

Use `tracing_subscriber` for logging to stderr (same pattern as `saf-bench/src/main.rs`).

### 4. Create traces.toml manifest

```toml
[[trace]]
algorithm = "pta"
input = "tests/fixtures/llvm/e2e/simple_alias.ll"
output = "pta-andersen/pta-andersen-basic.trace.json"
title = "Andersen's PTA: Basic Pointer Assignment"
```

Comment out future entries for IFDS, Interval, MSSA.

### 5. Verify compilation

**IMPORTANT:** This crate depends on LLVM-18 (via `saf-frontends`). Do NOT run `cargo check` locally. The leader will verify in Docker:

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-trace'
```

### 6. Commit

```bash
git add Cargo.toml crates/saf-trace/
git commit -m "feat: scaffold saf-trace crate for algorithm trace generation"
```

---

## Verification Checklist

After all agents complete and leader finishes integration + Playwright testing:

- [ ] `cd tutorials && npx tsc --noEmit` — no type errors
- [ ] `cd tutorials && npx vite build` — production build succeeds
- [ ] Playwright: sidebar shows "Algorithms" category with 4 entries
- [ ] Playwright: PTA tutorial prose steps load correctly
- [ ] Playwright: AlgorithmStepper renders with trace on step 3
- [ ] Playwright: stepper controls work (forward, back, play, jump)
- [ ] Playwright: PTA inspector shows worklist, points-to sets, constraints
- [ ] Playwright: all 3 placeholder tutorials (IFDS, Intervals, MSSA) load
- [ ] Playwright: no console errors on any page
- [ ] `docker compose run --rm dev sh -c 'cargo check -p saf-trace'` compiles
