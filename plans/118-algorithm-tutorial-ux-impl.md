# Algorithm Tutorial UX Redesign — Implementation Plan (Plan 118)

**Goal:** Redesign the algorithm stepper with pseudocode rail, phase bar, improved explanations, no-scroll viewport layout.

**Design doc:** `docs/plans/2026-02-16-algorithm-tutorial-ux-design.md`

---

## Team Structure

```
Phase 1 (all 5 agents parallel — no dependencies between them):
  Agent A: Types + new components (PhaseBar, PseudocodeRail)
  Agent B: Trace JSON files 1-4 (pta-andersen, ifds-taint, interval-analysis, memory-ssa)
  Agent C: Trace JSON files 5-8 (pta-kcfa, sparse-valueflow, dominator-tree, callgraph-construction)
  Agent D: Prose rewrites 1-4 (same 4 tutorials as Agent B)
  Agent E: Prose rewrites 5-8 (same 4 tutorials as Agent C)

Phase 2 (after Agent A completes):
  Agent F: AlgorithmStepper.tsx + CSS layout rewrite

Phase 3 (after all agents):
  Leader: npm build + comprehensive Playwright testing of all 8 tutorials
```

**File ownership is strict — no two agents touch the same file.**

---

## Task A: Types + PhaseBar + PseudocodeRail Components

**Agent type:** general-purpose

**Files (all in `tutorials/src/`):**
- Modify: `content/trace-types.ts`
- Create: `components/PhaseBar.tsx`
- Create: `components/PhaseBar.css`
- Create: `components/PseudocodeRail.tsx`
- Create: `components/PseudocodeRail.css`

### A.1: Extend trace-types.ts

Read `tutorials/src/content/trace-types.ts`. Add these types after the `AlgorithmState` union (around line 94):

```typescript
export interface PseudocodeLine {
  text: string;
}

export interface Pseudocode {
  title: string;
  lines: PseudocodeLine[];
}
```

Extend `TraceStep` (around line 96) — add two fields at the end:

```typescript
  phase: number;          // index into AlgorithmTrace.phases
  activeLines: number[];  // indices into AlgorithmTrace.pseudocode.lines
```

Extend `AlgorithmTrace` (around line 106) — add two fields before `steps`:

```typescript
  phases: string[];          // e.g. ["Extract", "Process Rank 0", ...]
  pseudocode: Pseudocode;    // algorithm pseudocode for rail display
```

### A.2: Create PhaseBar component

Write `tutorials/src/components/PhaseBar.tsx`:

```tsx
import './PhaseBar.css';

interface PhaseBarProps {
  phases: string[];
  currentPhase: number;
}

export default function PhaseBar({ phases, currentPhase }: PhaseBarProps) {
  return (
    <div className="phase-bar">
      {phases.map((phase, i) => {
        const status = i < currentPhase ? 'completed' : i === currentPhase ? 'active' : 'pending';
        return (
          <div key={i} className="phase-item">
            {i > 0 && <span className="phase-arrow">→</span>}
            <span className={`phase-node phase-${status}`}>
              <span className="phase-dot" />
              <span className="phase-label">{phase}</span>
            </span>
          </div>
        );
      })}
    </div>
  );
}
```

Write `tutorials/src/components/PhaseBar.css`:

```css
.phase-bar {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  gap: 0;
  overflow-x: auto;
  flex-shrink: 0;
}

.phase-item {
  display: flex;
  align-items: center;
  white-space: nowrap;
}

.phase-arrow {
  color: var(--text-muted);
  font-size: 12px;
  margin: 0 6px;
  opacity: 0.5;
}

.phase-node {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  padding: 2px 0;
}

.phase-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.phase-pending .phase-dot {
  border: 1.5px solid var(--text-muted);
  background: transparent;
}
.phase-pending .phase-label {
  color: var(--text-muted);
}
.phase-active .phase-dot {
  background: var(--accent);
  box-shadow: 0 0 6px rgba(250, 204, 21, 0.4);
}
.phase-active .phase-label {
  color: var(--accent);
  font-weight: 600;
}
.phase-completed .phase-dot {
  background: var(--text-muted);
}
.phase-completed .phase-label {
  color: var(--text-muted);
}
```

### A.3: Create PseudocodeRail component

Write `tutorials/src/components/PseudocodeRail.tsx`:

```tsx
import type { Pseudocode } from '../content/trace-types';
import './PseudocodeRail.css';

interface PseudocodeRailProps {
  pseudocode: Pseudocode;
  activeLines: number[];
  visitedLines: number[];
}

export default function PseudocodeRail({ pseudocode, activeLines, visitedLines }: PseudocodeRailProps) {
  return (
    <div className="pseudocode-rail">
      <div className="pseudocode-title">{pseudocode.title}</div>
      <div className="pseudocode-lines">
        {pseudocode.lines.map((line, i) => {
          const isActive = activeLines.includes(i);
          const isVisited = visitedLines.includes(i) && !isActive;
          const cls = [
            'pseudocode-line',
            isActive ? 'pseudocode-line-active' : '',
            isVisited ? 'pseudocode-line-visited' : '',
          ].filter(Boolean).join(' ');
          return (
            <div key={i} className={cls}>
              <span className="pseudocode-gutter">
                {isVisited ? <span className="pseudocode-check">✓</span>
                           : <span className="pseudocode-lineno">{i + 1}</span>}
              </span>
              <span className="pseudocode-text">{line.text}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

Write `tutorials/src/components/PseudocodeRail.css`:

```css
.pseudocode-rail {
  width: 180px;
  min-width: 180px;
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-secondary);
}
.pseudocode-title {
  font-family: monospace;
  font-size: 11px;
  font-weight: 700;
  color: var(--accent);
  padding: 8px 10px 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.pseudocode-lines {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}
.pseudocode-line {
  display: flex;
  align-items: flex-start;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.5;
  padding: 0 6px 0 0;
  border-left: 3px solid transparent;
}
.pseudocode-line-active {
  border-left-color: var(--accent);
  background: rgba(250, 204, 21, 0.08);
}
.pseudocode-line-visited { opacity: 0.6; }
.pseudocode-gutter {
  width: 24px;
  min-width: 24px;
  text-align: right;
  padding-right: 6px;
  color: var(--text-muted);
  font-size: 10px;
  line-height: 1.5;
  user-select: none;
}
.pseudocode-check { color: #22c55e; font-size: 10px; }
.pseudocode-lineno { opacity: 0.5; }
.pseudocode-text { white-space: pre; color: var(--text); }
.pseudocode-line-active .pseudocode-text { color: var(--accent); font-weight: 500; }
@media (max-width: 900px) { .pseudocode-rail { display: none; } }
```

### A.4: Commit

```bash
git add tutorials/src/content/trace-types.ts tutorials/src/components/PhaseBar.tsx tutorials/src/components/PhaseBar.css tutorials/src/components/PseudocodeRail.tsx tutorials/src/components/PseudocodeRail.css
git commit -m "feat(tutorials): add trace types, PhaseBar, and PseudocodeRail components"
```

---

## Task B: Trace JSON files 1-4 (phases + pseudocode + activeLines)

**Agent type:** general-purpose

**Files (ONLY these 4 — do NOT touch any other files):**
- Modify: `tutorials/public/content/pta-andersen/pta-andersen-basic.trace.json`
- Modify: `tutorials/public/content/ifds-taint/ifds-taint-basic.trace.json`
- Modify: `tutorials/public/content/interval-analysis/interval-basic.trace.json`
- Modify: `tutorials/public/content/memory-ssa/mssa-basic.trace.json`

**Instructions:** For each trace JSON file, read it first, then add three things:
1. A `"phases"` array (top-level, after `"example"`)
2. A `"pseudocode"` object (top-level, after `"phases"`)
3. `"phase"` (number) and `"activeLines"` (number array) on EACH step object

The `phase` value is an index into the `phases` array. The `activeLines` are indices (0-based) into `pseudocode.lines`. Map each step's `action` text to the appropriate phase and pseudocode lines.

### B.1: pta-andersen

Add after `"example"`:
```json
"phases": ["Extract Constraints", "Process Rank 0", "Propagate Copies", "Fixed Point"],
"pseudocode": {
  "title": "ANDERSEN-PTA(P)",
  "lines": [
    {"text": "C ← ExtractConstraints(P)"},
    {"text": "for each addr(v,loc) ∈ C:"},
    {"text": "  pts(v) ← pts(v) ∪ {loc}"},
    {"text": "  add v to worklist"},
    {"text": "while worklist ≠ ∅:"},
    {"text": "  v ← pop(worklist)"},
    {"text": "  for each copy(v,w) ∈ C:"},
    {"text": "    if pts(w) ⊅ pts(v):"},
    {"text": "      pts(w) ← pts(w) ∪ pts(v)"},
    {"text": "      add w to worklist"},
    {"text": "return pts"}
  ]
},
```

Per-step additions:
- Step 0 (action "Extract constraints"): `"phase": 0, "activeLines": [0]`
- Step 1 (action "Process wave at rank 0..."): `"phase": 1, "activeLines": [1, 2, 3]`
- Step 2 (action "Process wave at rank 1..."): `"phase": 2, "activeLines": [4, 5, 6, 7, 8, 9]`
- Step 3 (action "Fixed point reached"): `"phase": 3, "activeLines": [10]`

### B.2: ifds-taint

Add after `"example"`:
```json
"phases": ["Build Supergraph", "Seed Entry", "Tabulate Forward", "Compute Summaries", "Report"],
"pseudocode": {
  "title": "IFDS-TABULATE(G, D)",
  "lines": [
    {"text": "Insert (s_main, 0) → (s_main, 0)"},
    {"text": "worklist ← {(s_main, 0)}"},
    {"text": "while worklist ≠ ∅:"},
    {"text": "  (n, d) ← pop(worklist)"},
    {"text": "  for each succ m of n:"},
    {"text": "    for each d' ∈ F_edge(d):"},
    {"text": "      if (n,d)→(m,d') is new:"},
    {"text": "        add path edge"},
    {"text": "        add (m,d') to worklist"},
    {"text": "  if n is call site:"},
    {"text": "    propagate into callee"},
    {"text": "    record summary edge"},
    {"text": "return reachable facts"}
  ]
},
```

Read the trace steps, map each step to phases/activeLines based on action text. For example, seeding → phase 1 + lines [0,1], processing main → phase 2 + lines [2,3,4,5,6,7,8], call propagation → phase 3 + lines [9,10,11], report → phase 4 + lines [12].

### B.3: interval-analysis

```json
"phases": ["Init Domain", "Forward Pass", "Widen", "Narrow", "Converge"],
"pseudocode": {
  "title": "INTERVAL-ANALYSIS(CFG)",
  "lines": [
    {"text": "for each var v: I(v) ← ⊥"},
    {"text": "worklist ← {entry}"},
    {"text": "while worklist ≠ ∅:"},
    {"text": "  b ← pop(worklist)"},
    {"text": "  old ← state(b)"},
    {"text": "  new ← ⊔ {transfer(p) | p ∈ pred(b)}"},
    {"text": "  if b is loop header:"},
    {"text": "    new ← widen(old, new)"},
    {"text": "  if new ≠ old:"},
    {"text": "    state(b) ← new"},
    {"text": "    add successors to worklist"},
    {"text": "NARROW(CFG)"},
    {"text": "return state"}
  ]
},
```

### B.4: memory-ssa

```json
"phases": ["Build SSA", "Insert MemoryDefs", "Insert MemoryPhis", "Clobber Walk", "Result"],
"pseudocode": {
  "title": "MSSA-CLOBBER-WALK(use)",
  "lines": [
    {"text": "def ← use.defining_access"},
    {"text": "while def ≠ LiveOnEntry:"},
    {"text": "  if def is MemoryDef:"},
    {"text": "    a ← alias(def.loc, use.loc)"},
    {"text": "    if a = MustAlias: return def"},
    {"text": "    if a = MayAlias: return def"},
    {"text": "    def ← def.defining_access"},
    {"text": "  if def is MemoryPhi:"},
    {"text": "    for each operand op:"},
    {"text": "      walk(op)  // recurse"},
    {"text": "  return LiveOnEntry"}
  ]
},
```

### B.5: Validate + Commit

Validate all 4 parse: `cd tutorials && node -e "['pta-andersen/pta-andersen-basic','ifds-taint/ifds-taint-basic','interval-analysis/interval-basic','memory-ssa/mssa-basic'].forEach(f=>{const d=JSON.parse(require('fs').readFileSync('public/content/'+f+'.trace.json','utf8'));console.log(f,'ok','phases:'+d.phases.length,'pseudo:'+d.pseudocode.lines.length,'steps-have-phase:'+d.steps.every(s=>typeof s.phase==='number'),'steps-have-activeLines:'+d.steps.every(s=>Array.isArray(s.activeLines)))})"`

Commit:
```bash
git add tutorials/public/content/pta-andersen/pta-andersen-basic.trace.json tutorials/public/content/ifds-taint/ifds-taint-basic.trace.json tutorials/public/content/interval-analysis/interval-basic.trace.json tutorials/public/content/memory-ssa/mssa-basic.trace.json
git commit -m "feat(tutorials): add phases/pseudocode/activeLines to PTA, IFDS, Interval, MSSA traces"
```

---

## Task C: Trace JSON files 5-8 (phases + pseudocode + activeLines)

**Agent type:** general-purpose

**Files (ONLY these 4 — do NOT touch any other files):**
- Modify: `tutorials/public/content/pta-kcfa/kcfa-basic.trace.json`
- Modify: `tutorials/public/content/sparse-valueflow/svf-basic.trace.json`
- Modify: `tutorials/public/content/dominator-tree/dom-basic.trace.json`
- Modify: `tutorials/public/content/callgraph-construction/cg-basic.trace.json`

**Instructions:** Same pattern as Task B. Read each file, add `phases` array, `pseudocode` object at top level, and `phase`+`activeLines` on each step.

### C.1: pta-kcfa

```json
"phases": ["Init Contexts", "Extract Constraints", "Process Context", "Clone/Merge", "Fixed Point"],
"pseudocode": {
  "title": "K-CFA-PTA(P, k)",
  "lines": [
    {"text": "C ← ExtractConstraints(P)"},
    {"text": "ctx ← [∅]  // initial context"},
    {"text": "for each call site cs:"},
    {"text": "  ctx' ← push(ctx, cs)[0:k]"},
    {"text": "  seed worklist with (ctx', v)"},
    {"text": "while worklist ≠ ∅:"},
    {"text": "  (ctx, v) ← pop(worklist)"},
    {"text": "  resolve constraints for v in ctx"},
    {"text": "  propagate pts changes"},
    {"text": "  if indirect call resolved:"},
    {"text": "    clone context for callee"},
    {"text": "return per-context pts"}
  ]
},
```

### C.2: sparse-valueflow

```json
"phases": ["Build Def-Use", "Compute SSA", "Build VF Edges", "Propagate", "Query"],
"pseudocode": {
  "title": "SPARSE-VF(P)",
  "lines": [
    {"text": "Build def-use chains (SSA)"},
    {"text": "for each def d:"},
    {"text": "  for each use u of d:"},
    {"text": "    add VF edge d → u"},
    {"text": "for each store *p = v:"},
    {"text": "  locs ← pts(p)"},
    {"text": "  for each load w = *q:"},
    {"text": "    if pts(q) ∩ locs ≠ ∅:"},
    {"text": "      add indirect edge v → w"},
    {"text": "Propagate facts along VF edges"},
    {"text": "return reaching definitions"}
  ]
},
```

### C.3: dominator-tree

```json
"phases": ["Init", "Compute Idom", "Build Tree", "Compute DF", "Detect Loops"],
"pseudocode": {
  "title": "DOMINATORS(CFG)",
  "lines": [
    {"text": "idom(entry) ← entry"},
    {"text": "for each b in RPO (skip entry):"},
    {"text": "  idom(b) ← first processed pred"},
    {"text": "repeat:"},
    {"text": "  for each b in RPO:"},
    {"text": "    new_idom ← intersect preds"},
    {"text": "    if new_idom ≠ idom(b):"},
    {"text": "      idom(b) ← new_idom"},
    {"text": "  until no changes"},
    {"text": "COMPUTE-DF(idom)"},
    {"text": "DETECT-LOOPS(idom, back_edges)"}
  ]
},
```

### C.4: callgraph-construction

```json
"phases": ["Scan Declarations", "Resolve CHA", "Refine RTA", "Refine VTA", "Final"],
"pseudocode": {
  "title": "CG-CONSTRUCTION(P)",
  "lines": [
    {"text": "// Phase 1: CHA"},
    {"text": "for each call site cs:"},
    {"text": "  targets ← type_hierarchy(cs)"},
    {"text": "  add edges cs → targets"},
    {"text": "// Phase 2: RTA"},
    {"text": "reachable ← {main}"},
    {"text": "for each r in reachable:"},
    {"text": "  for each call in r:"},
    {"text": "    filter targets by instantiated"},
    {"text": "// Phase 3: VTA"},
    {"text": "for each call in reachable:"},
    {"text": "  refine by PTA(receiver)"},
    {"text": "return callgraph"}
  ]
},
```

### C.5: Validate + Commit

Validate: `cd tutorials && node -e "['pta-kcfa/kcfa-basic','sparse-valueflow/svf-basic','dominator-tree/dom-basic','callgraph-construction/cg-basic'].forEach(f=>{const d=JSON.parse(require('fs').readFileSync('public/content/'+f+'.trace.json','utf8'));console.log(f,'ok','phases:'+d.phases.length,'pseudo:'+d.pseudocode.lines.length,'steps-have-phase:'+d.steps.every(s=>typeof s.phase==='number'),'steps-have-activeLines:'+d.steps.every(s=>Array.isArray(s.activeLines)))})"`

Commit:
```bash
git add tutorials/public/content/pta-kcfa/kcfa-basic.trace.json tutorials/public/content/sparse-valueflow/svf-basic.trace.json tutorials/public/content/dominator-tree/dom-basic.trace.json tutorials/public/content/callgraph-construction/cg-basic.trace.json
git commit -m "feat(tutorials): add phases/pseudocode/activeLines to kCFA, SVF, Dom, CG traces"
```

---

## Task D: Prose rewrites 1-4 (steps.json content)

**Agent type:** general-purpose

**Files (ONLY these 4 — do NOT touch trace.json files or any component files):**
- Modify: `tutorials/public/content/pta-andersen/steps.json`
- Modify: `tutorials/public/content/ifds-taint/steps.json`
- Modify: `tutorials/public/content/interval-analysis/steps.json`
- Modify: `tutorials/public/content/memory-ssa/steps.json`

**Instructions:** Each file is a flat JSON array of step objects. Replace the `content` field of the FIRST step (index 0) with a 4-section markdown format. Keep all other steps and fields unchanged.

The 4 sections are:
```
## Motivation & Context
(~100 words) Why this algorithm exists, what problem it solves

## Key Concepts
(~150 words) Glossary: bold term + one-sentence definition, 4-6 terms

## Algorithm Overview
(~200 words) Plain English walkthrough of the algorithm phases

## Formal Properties
(~100 words) Complexity, soundness, completeness, key citations
```

### D.1: pta-andersen/steps.json

Replace first step `content` with:

```
## Motivation & Context\n\nWhen analyzing C programs, we need to know which pointers can refer to which memory locations. Without this, every pointer dereference is ambiguous — we can't track data flow, detect use-after-free, or reason about aliasing. Pointer analysis is the foundation that makes all other SAF analyses precise. Andersen's analysis gives us a sound, whole-program answer using an inclusion-based approach.\n\n## Key Concepts\n\n- **Points-to set** `pts(v)`: The set of memory locations that pointer `v` may reference at any point during execution\n- **Inclusion constraint** `pts(q) ⊇ pts(p)`: Everything `p` points to, `q` also points to (from assignment `q = p`)\n- **Address constraint** `loc ∈ pts(p)`: Pointer `p` directly takes the address of location `loc` (from `p = &x`)\n- **Fixed point**: The state where no more points-to information can be derived — the algorithm terminates\n- **Worklist**: A queue of pointer variables whose points-to sets recently changed and need their constraints re-evaluated\n\n## Algorithm Overview\n\nAndersen's analysis proceeds in three phases. First, **extract constraints** from the program: each `p = &x` becomes an address constraint, each `q = p` becomes a copy constraint, and pointer dereferences create load/store constraints. Second, **process the worklist**: starting with variables that have address constraints (rank 0), resolve each variable's constraints and propagate new points-to information to dependent variables. Third, **reach fixed point**: when the worklist is empty and no new information flows, the analysis is complete. The result maps every pointer to its possible targets.\n\n## Formal Properties\n\nFlow-insensitive (ignores statement order), context-insensitive (merges all calling contexts). Subset-based / inclusion-based. Time complexity O(n³) worst case where n is the number of pointer variables. **Sound**: never misses a real alias pair. **May be imprecise**: can report aliases that never actually occur at runtime. Monotone over a lattice of points-to sets ordered by ⊆.
```

### D.2: ifds-taint/steps.json

Write prose about IFDS taint analysis with:
- Motivation: Interprocedural dataflow as graph reachability on the exploded supergraph
- Key Concepts: exploded supergraph, path edges, summary edges, zero fact (Λ), distributive flow functions, realizable paths
- Algorithm Overview: Seed entry node with zero fact, tabulate forward using flow functions, propagate across call boundaries, compute summary edges for procedures
- Formal Properties: O(E·D³) time, sound and complete for distributive problems, Reps/Horwitz/Sagiv POPL 1995

### D.3: interval-analysis/steps.json

Write prose about interval abstract interpretation with:
- Motivation: Detect numeric errors (overflow, division by zero, out-of-bounds) statically
- Key Concepts: abstract domain [lo, hi], transfer functions (arithmetic over intervals), join (⊔), widening (∇), narrowing (△), ascending chain condition
- Algorithm Overview: Init variables to ⊥, forward worklist pass applying transfer functions, widen at loop headers to force convergence, narrow pass to regain precision
- Formal Properties: Sound, convergence guaranteed by widening, Cousot & Cousot 1977

### D.4: memory-ssa/steps.json

Write prose about Memory SSA with:
- Motivation: Extend SSA to memory so we can answer "which store last wrote to this location?"
- Key Concepts: MemoryDef, MemoryUse, MemoryPhi, clobber walk, LiveOnEntry, alias queries (must/may/no alias)
- Algorithm Overview: Insert MemoryDefs for stores, MemoryPhis at join points, demand-driven clobber walking using alias queries
- Formal Properties: Demand-driven (lazy evaluation), precision depends on underlying alias analysis, bridges PTA to value-flow

### D.5: Validate + Commit

Validate JSON: `cd tutorials && node -e "['pta-andersen','ifds-taint','interval-analysis','memory-ssa'].forEach(d=>{const s=JSON.parse(require('fs').readFileSync('public/content/'+d+'/steps.json','utf8'));console.log(d,'steps:'+s.length,'has-motivation:'+s[0].content.includes('Motivation'))})"`

Commit:
```bash
git add tutorials/public/content/pta-andersen/steps.json tutorials/public/content/ifds-taint/steps.json tutorials/public/content/interval-analysis/steps.json tutorials/public/content/memory-ssa/steps.json
git commit -m "feat(tutorials): rewrite PTA, IFDS, Interval, MSSA prose with 4-section format"
```

---

## Task E: Prose rewrites 5-8 (steps.json content)

**Agent type:** general-purpose

**Files (ONLY these 4 — do NOT touch trace.json files or any component files):**
- Modify: `tutorials/public/content/pta-kcfa/steps.json`
- Modify: `tutorials/public/content/sparse-valueflow/steps.json`
- Modify: `tutorials/public/content/dominator-tree/steps.json`
- Modify: `tutorials/public/content/callgraph-construction/steps.json`

**Instructions:** Same as Task D — replace first step's `content` with 4-section markdown.

### E.1: pta-kcfa/steps.json

Write prose about k-CFA context-sensitive PTA:
- Motivation: Context-insensitive PTA (Andersen's) merges all calling contexts, losing precision at call sites with multiple callers. k-CFA distinguishes contexts by tracking k most recent call sites.
- Key Concepts: calling context, context stack/string, context cloning, context-sensitive points-to set, k parameter (precision vs. cost tradeoff), context sensitivity
- Algorithm Overview: Same worklist-based constraint solving as Andersen's, but each variable's points-to set is maintained per-context. On function call, push call site onto context stack (truncate to length k). Propagate within each context independently.
- Formal Properties: k=0 equivalent to Andersen's CI. k=1 is 1-CFA. Worst-case exponential in k (context explosion). Practical with bounded k and context limits. Sound. Strictly more precise than CI for k≥1.

### E.2: sparse-valueflow/steps.json

Write prose about sparse value-flow analysis:
- Motivation: Dense dataflow propagates facts through every program point, even when nothing changes. Sparse value-flow uses SSA def-use chains to skip irrelevant points.
- Key Concepts: SSA form, def-use chains, direct value-flow edges (assignment), indirect value-flow edges (through memory via PTA), sparse vs. dense propagation
- Algorithm Overview: Build SSA def-use chains, create direct VF edges for each def→use, use PTA to match stores (*p=v) with loads (w=*q) where pts(p) ∩ pts(q) ≠ ∅, propagate dataflow facts along VF edges only
- Formal Properties: Same precision as dense analysis with far fewer propagation steps. O(E·D) where E = VF edge count. Foundation for taint tracking, typestate, and reaching definitions.

### E.3: dominator-tree/steps.json

Write prose about dominator trees:
- Motivation: Dominance relationships and loops are fundamental to SSA construction, loop optimizations, and control flow analysis. A node d dominates n if every path from entry to n goes through d.
- Key Concepts: immediate dominator (idom), dominance frontier (DF), reverse post-order (RPO), back edges (edges to dominator = natural loop), natural loops, loop headers
- Algorithm Overview: Cooper-Harvey-Kennedy iterative algorithm — compute idom for each block using RPO traversal and LCA-based intersection. Build dominance frontier from idom tree. Detect loops by identifying back edges.
- Formal Properties: O(n²) worst case, near-linear in practice on reducible CFGs. idom is unique. DF drives φ-function placement in SSA construction. Cooper/Harvey/Kennedy 2001.

### E.4: callgraph-construction/steps.json

Write prose about call graph construction:
- Motivation: Knowing which functions can call which is essential for all interprocedural analyses (PTA, value-flow, IFDS). Without a call graph, we can't scope analysis or handle indirect calls.
- Key Concepts: CHA (Class Hierarchy Analysis), RTA (Rapid Type Analysis), VTA (Variable Type Analysis), direct calls (target known), indirect calls (function pointers), precision hierarchy
- Algorithm Overview: Three-phase refinement. Phase 1 (CHA): resolve indirect calls using type hierarchy — all type-compatible targets. Phase 2 (RTA): filter to only instantiated/reachable types. Phase 3 (VTA): use PTA to resolve receiver objects for maximum precision.
- Formal Properties: CHA ⊇ RTA ⊇ VTA (each step strictly more precise, more costly). CHA is fast but imprecise. VTA is precise but requires running PTA. Practical systems use iterative CG-PTA refinement.

### E.5: Validate + Commit

Validate: `cd tutorials && node -e "['pta-kcfa','sparse-valueflow','dominator-tree','callgraph-construction'].forEach(d=>{const s=JSON.parse(require('fs').readFileSync('public/content/'+d+'/steps.json','utf8'));console.log(d,'steps:'+s.length,'has-motivation:'+s[0].content.includes('Motivation'))})"`

Commit:
```bash
git add tutorials/public/content/pta-kcfa/steps.json tutorials/public/content/sparse-valueflow/steps.json tutorials/public/content/dominator-tree/steps.json tutorials/public/content/callgraph-construction/steps.json
git commit -m "feat(tutorials): rewrite kCFA, SVF, Dom, CG prose with 4-section format"
```

---

## Task F: Rewrite AlgorithmStepper layout (after Task A)

**Agent type:** general-purpose

**Files (ONLY these 2):**
- Modify: `tutorials/src/components/AlgorithmStepper.tsx`
- Modify: `tutorials/src/components/AlgorithmStepper.css`

**Context needed:** Read these files first:
- `tutorials/src/components/AlgorithmStepper.tsx` (current code to replace)
- `tutorials/src/components/AlgorithmStepper.css` (current CSS — keep inspector styles, replace layout)
- `tutorials/src/components/PhaseBar.tsx` (new component from Task A)
- `tutorials/src/components/PseudocodeRail.tsx` (new component from Task A)
- `tutorials/src/content/trace-types.ts` (updated types from Task A)

### F.1: Rewrite AlgorithmStepper.tsx

Replace the entire file content. The new layout has 4 zones:
1. **Header + PhaseBar** (top, fixed height)
2. **PseudocodeRail + GraphViewer** (middle, flex-1)
3. **ExplanationPanel + StateInspector** (bottom, 35% height)
4. **StepperControls** (bottom bar, fixed)

Key changes from current code:
- Import `PhaseBar` and `PseudocodeRail`
- Add `visitedLines` computed via `useMemo` (collects all activeLines from steps before current)
- Replace `.stepper-panels` (3fr/2fr grid) with `.stepper-main` (flex row: rail + graph)
- Move `ExplanationPanel` from `<details>` into `.stepper-bottom-left`
- Move `StateInspector` into `.stepper-bottom-right`
- Use `??` fallbacks for `phase` and `activeLines` (backward compat with old traces)

Write the full new component:

```tsx
import { useState, useEffect, useCallback, useMemo } from 'react';
import type { AlgorithmTrace } from '../content/trace-types';
import type { PropertyGraph } from '@saf/web-shared/types';
import GraphViewer from './GraphViewer';
import ExplanationPanel from './ExplanationPanel';
import StateInspector from './StateInspector';
import StepperControls from './StepperControls';
import PhaseBar from './PhaseBar';
import PseudocodeRail from './PseudocodeRail';
import './AlgorithmStepper.css';

interface AlgorithmStepperProps {
  tutorialId: string;
  traceFile: string;
}

function traceToPropertyGraph(trace: AlgorithmTrace, stepIndex: number): PropertyGraph {
  const step = trace.steps[stepIndex];
  return {
    schema_version: '0.1.0',
    graph_type: 'cfg',
    metadata: { algorithm: trace.algorithm },
    nodes: step.graph.nodes.map(n => ({
      id: n.id,
      labels: [n.type],
      properties: {
        name: n.label,
        ...n.properties,
        highlighted: step.highlights.nodes.includes(n.id),
      },
    })),
    edges: step.graph.edges.map(e => ({
      src: e.src,
      dst: e.dst,
      edge_type: e.type,
      properties: {
        label: e.label,
        highlighted: step.highlights.edges.some(
          h => h.src === e.src && h.dst === e.dst
        ),
      },
    })),
  };
}

export default function AlgorithmStepper({ tutorialId, traceFile }: AlgorithmStepperProps) {
  const [trace, setTrace] = useState<AlgorithmTrace | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [stepIndex, setStepIndex] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [speed, setSpeed] = useState(1);

  const baseUrl = import.meta.env.BASE_URL || '/';

  useEffect(() => {
    const url = `${baseUrl}content/${tutorialId}/${traceFile}`;
    fetch(url)
      .then(res => {
        if (!res.ok) throw new Error(`Failed to load trace: ${res.status}`);
        return res.json();
      })
      .then((data: AlgorithmTrace) => setTrace(data))
      .catch(err => setError(err.message));
  }, [tutorialId, traceFile, baseUrl]);

  const stepForward = useCallback(() => {
    setStepIndex(i => (trace ? Math.min(i + 1, trace.steps.length - 1) : i));
  }, [trace]);
  const stepBack = useCallback(() => {
    setStepIndex(i => Math.max(i - 1, 0));
  }, []);
  const jumpToStart = useCallback(() => setStepIndex(0), []);
  const jumpToEnd = useCallback(() => {
    if (trace) setStepIndex(trace.steps.length - 1);
  }, [trace]);
  const togglePlay = useCallback(() => setIsPlaying(p => !p), []);

  const visitedLines = useMemo(() => {
    if (!trace) return [];
    const visited = new Set<number>();
    for (let i = 0; i < stepIndex; i++) {
      const step = trace.steps[i];
      if (step.activeLines) {
        for (const line of step.activeLines) visited.add(line);
      }
    }
    return Array.from(visited);
  }, [trace, stepIndex]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      switch (e.key) {
        case 'ArrowRight': case 'l': e.preventDefault(); stepForward(); break;
        case 'ArrowLeft': case 'h': e.preventDefault(); stepBack(); break;
        case ' ': e.preventDefault(); togglePlay(); break;
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [stepForward, stepBack, togglePlay]);

  if (error) return <div className="algorithm-stepper algorithm-stepper-error">Error: {error}</div>;
  if (!trace) return <div className="algorithm-stepper algorithm-stepper-loading">Loading trace...</div>;

  const currentStep = trace.steps[stepIndex];
  const graph = traceToPropertyGraph(trace, stepIndex);

  return (
    <div className="algorithm-stepper">
      <div className="stepper-header">
        <div className="stepper-header-top">
          <h3 className="stepper-action">{currentStep.action}</h3>
          <span className="stepper-step-label">Step {stepIndex + 1} / {trace.steps.length}</span>
        </div>
        {trace.phases && (
          <PhaseBar phases={trace.phases} currentPhase={currentStep.phase ?? 0} />
        )}
      </div>
      <div className="stepper-main">
        {trace.pseudocode && (
          <PseudocodeRail
            pseudocode={trace.pseudocode}
            activeLines={currentStep.activeLines ?? []}
            visitedLines={visitedLines}
          />
        )}
        <div className="stepper-graph">
          <GraphViewer graph={graph} graphType="algorithm" />
        </div>
      </div>
      <div className="stepper-bottom">
        <div className="stepper-bottom-left">
          <div className="stepper-section-header">Explanation</div>
          <ExplanationPanel
            stepId={currentStep.id}
            action={currentStep.action}
            explanation={currentStep.explanation}
          />
        </div>
        <div className="stepper-bottom-right">
          <div className="stepper-section-header">Algorithm State</div>
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
    </div>
  );
}
```

### F.2: Rewrite AlgorithmStepper.css

Read the current CSS. It has two sections:
1. **Layout CSS** (lines 1-157) — REPLACE this entirely
2. **Inspector CSS** (lines 159-469, starting from `.state-inspector`) — KEEP unchanged

Replace lines 1-157 with:

```css
.algorithm-stepper {
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg-secondary);
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  height: calc(100vh - 228px);
}
.algorithm-stepper-loading,
.algorithm-stepper-error {
  padding: 32px;
  text-align: center;
  color: var(--text-muted);
}
.algorithm-stepper-error { color: #ef4444; }

.stepper-header {
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.stepper-header-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
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

.stepper-main {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.stepper-main .stepper-graph {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
}
.stepper-main .stepper-graph .graph-viewer {
  height: 100% !important;
  min-height: unset !important;
}

.stepper-bottom {
  display: grid;
  grid-template-columns: 2fr 3fr;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
  height: 35%;
  min-height: 140px;
  max-height: 260px;
  overflow: hidden;
}
.stepper-bottom-left {
  border-right: 1px solid var(--border);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
.stepper-bottom-right {
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
.stepper-section-header {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  padding: 6px 12px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  font-weight: 500;
}
.stepper-bottom-left .explanation-panel { padding: 8px 12px; flex: 1; }
.stepper-bottom-left .explanation-action { display: none; }
.stepper-bottom-left .explanation-body { font-size: 13px; line-height: 1.5; }
.stepper-bottom-right .state-inspector { padding: 8px 12px; }

@media (max-width: 900px) {
  .stepper-main { flex-direction: column; }
  .stepper-bottom {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
  }
  .stepper-bottom-left {
    border-right: none;
    border-bottom: 1px solid var(--border);
    max-height: 120px;
  }
}
```

Then append all the existing inspector CSS (from `.state-inspector` through end of file) unchanged.

### F.3: Verify build

Run: `cd tutorials && npx tsc --noEmit && npx vite build`

Both should succeed with zero errors.

### F.4: Commit

```bash
git add tutorials/src/components/AlgorithmStepper.tsx tutorials/src/components/AlgorithmStepper.css
git commit -m "feat(tutorials): viewport-filling layout with pseudocode rail and phase bar"
```

---

## Task G: Leader — Build + Playwright Testing

**Executed by leader after all agents complete.**

### G.1: Build tutorials app

```bash
cd tutorials && npm run build
```

Must succeed with zero errors.

### G.2: Start preview server

```bash
cd tutorials && npx vite preview --port 4175
```

### G.3: Playwright — Test all 8 algorithm tutorials

For EACH of these 8 tutorials, navigate to the algorithm step (step 2 or 3 — the one with `stepType: "algorithm"`):

1. `pta-andersen` — step 3 (index 2)
2. `ifds-taint` — step 2 (index 1)
3. `interval-analysis` — step 2 (index 1)
4. `memory-ssa` — step 2 (index 1)
5. `pta-kcfa` — step 2 (index 1)
6. `sparse-valueflow` — step 2 (index 1)
7. `dominator-tree` — step 2 (index 1)
8. `callgraph-construction` — step 2 (index 1)

**For each tutorial, verify ALL of these:**

| Check | How to verify |
|-------|---------------|
| Phase bar renders | Look for `.phase-bar` with multiple `.phase-item` children |
| Phase bar has correct count | Count `.phase-item` elements (should be 4-5) |
| Active phase highlighted | `.phase-active` class exists on exactly one phase |
| Pseudocode rail renders | Look for `.pseudocode-rail` with `.pseudocode-lines` children |
| Active lines highlighted | `.pseudocode-line-active` class exists on at least one line |
| Graph fills main area | `.stepper-graph` is visible and has width > 200px |
| Explanation visible (not in details) | `.stepper-bottom-left` contains `.explanation-panel` directly (no `<details>` wrapper) |
| State inspector visible | `.stepper-bottom-right` contains `.state-inspector` |
| Step forward works | Click step-forward button → step counter increments |
| Phase updates on step | After stepping, `.phase-active` may move to a different phase |
| Pseudocode updates on step | After stepping, different `.pseudocode-line-active` lines |
| Visited lines show checkmarks | After stepping forward once, step back — previously active lines should have `.pseudocode-line-visited` |
| Zero console errors | No errors in browser console |

### G.4: Playwright — Test prose content

Navigate to each of the 8 algorithm tutorial's FIRST step (prose step). Verify:
- Content contains "Motivation" heading
- Content contains "Key Concepts" heading
- Content contains "Algorithm Overview" heading
- Content contains "Formal Properties" heading

### G.5: Playwright — Test navigation

- Navigate between tutorials using sidebar
- Verify step counter shows correct numbers
- Verify transport controls (play/pause/forward/back/jump)

### G.6: Fix any issues found

If Playwright testing reveals bugs, fix them directly and commit.

### G.7: Final commit

```bash
git add -A
git commit -m "feat(tutorials): algorithm stepper UX redesign complete (plan 118)"
```

### G.8: Update PROGRESS.md

Set plan 118 status from "approved" to "done". Add session log entry.
