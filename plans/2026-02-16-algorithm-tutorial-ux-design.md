# Algorithm Tutorial UX Redesign — Design Document

**Date:** 2026-02-16
**Target:** CS graduate students learning static analysis algorithms
**Problem:** Current algorithm stepper is cramped (60/40 side-by-side split), has poor information hierarchy (everything competes for attention), lacks algorithm context (no pseudocode, no phase indicator), and has confusing navigation.

## Layout: "Annotated Visualization" (No-Scroll)

Replace the current `3fr/2fr` grid with a **viewport-filling vertical layout** that eliminates scrolling entirely.

```
┌──────────────────────────────────────────────────┐
│  Andersen's PTA — Step 2/4: Process rank 0       │
│  ○ Extract → [● Process] → ○ Propagate → ○ FP   │  ← Phase Bar (~40px)
├────────────┬─────────────────────────────────────┤
│ PSEUDOCODE │  GRAPH VISUALIZATION                │
│            │                                      │
│ 1 extract  │    [p] ──→ [x]                      │
│ 2 seed wl  │    [q] - - → (copy)                 │  ← Main Area (~65vh)
│ 3▸while:   │    [r] ──→ [y]                      │
│ 4  pop(v)  │                                      │
│ 5  resolve │                                      │
│            │                                      │
│ (~160px)   │  (~rest of width)                    │
├────────────┴──────────┬──────────────────────────┤
│ EXPLANATION           │ STATE INSPECTOR           │
│ "The solver processes │ Worklist: [q] @rank1      │  ← Bottom (~35vh)
│  rank-0 variables..." │ pts(p)={x}  pts(r)={y}   │
│                       │ ✓addr p=&x  ✓addr r=&y   │
└───────────────────────┴──────────────────────────┘
│ ◄◄  ◄  ▶  ►  ►►       Step 2/4      Speed: 1x   │  ← Controls (~44px)
└──────────────────────────────────────────────────┘
```

### Panel Sizing

- **Phase bar:** Fixed ~40px. Always visible.
- **Main area:** `calc(65vh - 40px - 44px)`. Left rail 160px fixed, graph fills remainder.
- **Bottom area:** `calc(35vh)`. 40/60 split (explanation/state).
- **Controls bar:** Fixed ~44px. Pinned to bottom.
- **Total:** `100vh` with no scrolling.

### Responsive (< 900px)

Below 900px, stack vertically with the pseudocode collapsed into the phase bar (tap to expand).

## Component 1: Algorithm Phase Bar

A thin horizontal bar showing the algorithm's high-level phases as connected nodes.

### Phase Definitions

| Algorithm | Phases |
|-----------|--------|
| Andersen's PTA | Initialize → Extract Constraints → Process Worklist → Propagate → Fixed Point |
| IFDS Taint | Build Supergraph → Seed Entry → Tabulate Forward → Compute Summaries → Report |
| Interval Analysis | Init Domain → Forward Pass → Widen → Narrow → Converge |
| Memory SSA | Build SSA → Insert MemoryDefs → Insert MemoryPhis → Clobber Walk → Result |
| k-CFA | Init Contexts → Extract → Process Context → Clone/Merge → Fixed Point |
| Sparse Value-Flow | Build DefUse → Compute SSA → Build VF Edges → Propagate → Query |
| Dominator Tree | Init → Compute Idom → Build Tree → Compute DF → Detect Loops |
| Callgraph | Scan Declarations → Resolve CHA → Refine RTA → Refine VTA → Final |

### Rendering

- Horizontal row of `<span>` elements connected by `→` arrows
- Completed phases: filled circle + muted text
- Current phase: accent-colored filled circle + bold text + subtle glow/border
- Future phases: empty circle + muted text
- No animation — instant state change on step transition

### Data Format

New fields in trace JSON:

```json
{
  "phases": ["Extract", "Process Rank 0", "Propagate", "Fixed Point"],
  "steps": [
    { "id": 0, "phase": 0, "activeLines": [0], ... },
    { "id": 1, "phase": 1, "activeLines": [1, 2, 3], ... }
  ]
}
```

## Component 2: Pseudocode Rail

A narrow left sidebar (~160px) showing algorithm pseudocode with current-line highlighting.

### Pseudocode Content (per algorithm)

**Andersen's PTA:**
```
ANDERSEN-PTA(P)
 1  C ← ExtractConstraints(P)
 2  for each addr(v,loc) ∈ C:
 3    pts(v) ← pts(v) ∪ {loc}
 4    add v to worklist
 5  while worklist ≠ ∅:
 6    v ← pop(worklist)
 7    for each copy(v,w) ∈ C:
 8      if pts(w) ⊅ pts(v):
 9        pts(w) ← pts(w) ∪ pts(v)
10        add w to worklist
11  return pts
```

**IFDS Taint:**
```
IFDS-TABULATE(G, D)
 1  Insert (s_main, 0) → (s_main, 0)
 2  worklist ← {(s_main, 0)}
 3  while worklist ≠ ∅:
 4    (n, d) ← pop(worklist)
 5    for each succ m of n:
 6      for each d' ∈ F_edge(d):
 7        if (n,d)→(m,d') is new:
 8          add path edge
 9          add (m,d') to worklist
10    if n is call site:
11      propagate into callee
12      record summary edge
13  return reachable facts
```

**Interval Analysis:**
```
INTERVAL-ANALYSIS(CFG)
 1  for each var v: I(v) ← ⊥
 2  worklist ← {entry}
 3  while worklist ≠ ∅:
 4    b ← pop(worklist)
 5    old ← state(b)
 6    new ← ⊔ { transfer(p) | p ∈ pred(b) }
 7    if b is loop header:
 8      new ← widen(old, new)
 9    if new ≠ old:
10      state(b) ← new
11      add successors to worklist
12  NARROW(CFG)
13  return state
```

**Memory SSA:**
```
MSSA-CLOBBER-WALK(use)
 1  def ← use.defining_access
 2  while def ≠ LiveOnEntry:
 3    if def is MemoryDef:
 4      a ← alias(def.loc, use.loc)
 5      if a = MustAlias: return def
 6      if a = MayAlias: return def
 7      def ← def.defining_access
 8    if def is MemoryPhi:
 9      for each operand op:
10        walk(op)  // recurse
11  return LiveOnEntry
```

**k-CFA:**
```
K-CFA-PTA(P, k)
 1  C ← ExtractConstraints(P)
 2  ctx ← [∅]  // initial context
 3  for each call site cs:
 4    ctx' ← push(ctx, cs)[0:k]
 5    seed worklist with (ctx', v)
 6  while worklist ≠ ∅:
 7    (ctx, v) ← pop(worklist)
 8    resolve constraints for v in ctx
 9    propagate pts changes
10    if indirect call resolved:
11      clone context for callee
12  return per-context pts
```

**Sparse Value-Flow:**
```
SPARSE-VF(P)
 1  Build def-use chains (SSA)
 2  for each def d:
 3    for each use u of d:
 4      add VF edge d → u
 5  for each store *p = v:
 6    locs ← pts(p)
 7    for each load w = *q:
 8      if pts(q) ∩ locs ≠ ∅:
 9        add indirect edge v → w
10  Propagate facts along VF edges
11  return reaching definitions
```

**Dominator Tree:**
```
DOMINATORS(CFG)
 1  idom(entry) ← entry
 2  for each b in RPO (skip entry):
 3    idom(b) ← first processed pred
 4  repeat:
 5    for each b in RPO:
 6      new_idom ← intersect preds
 7      if new_idom ≠ idom(b):
 8        idom(b) ← new_idom
 9  until no changes
10  COMPUTE-DF(idom)
11  DETECT-LOOPS(idom, back_edges)
```

**Callgraph Construction:**
```
CG-CONSTRUCTION(P)
 1  // Phase 1: CHA
 2  for each call site cs:
 3    targets ← type_hierarchy(cs)
 4    add edges cs → targets
 5  // Phase 2: RTA
 6  reachable ← {main}
 7  for each r in reachable:
 8    for each call in r:
 9      filter targets by instantiated
10  // Phase 3: VTA
11  for each call in reachable:
12    refine by PTA(receiver)
13  return callgraph
```

### Styling

- Monospace font, 12px
- Line numbers in muted color (left gutter)
- **Active line(s):** `border-left: 3px solid var(--accent)` + `background: rgba(accent, 0.1)`
- **Visited lines:** Subtle faded checkmark in gutter
- **Future lines:** Normal text, no decoration
- Algorithm title in bold at top of rail

### Data Format

New `pseudocode` field in trace JSON:

```json
{
  "pseudocode": {
    "title": "ANDERSEN-PTA(P)",
    "lines": [
      "C ← ExtractConstraints(P)",
      "for each addr(v,loc) ∈ C:",
      "  pts(v) ← pts(v) ∪ {loc}",
      "  add v to worklist",
      "while worklist ≠ ∅:",
      "  v ← pop(worklist)",
      "  for each copy(v,w) ∈ C:",
      "    if pts(w) ⊅ pts(v):",
      "      pts(w) ← pts(w) ∪ pts(v)",
      "      add w to worklist",
      "return pts"
    ]
  },
  "steps": [
    { "id": 0, "activeLines": [0], ... },
    { "id": 1, "activeLines": [1, 2, 3], ... }
  ]
}
```

## Component 3: Improved Algorithm Explanations

Each algorithm tutorial's prose step (step 1 in steps.json) restructured with 4 subsections:

### Structure

1. **Motivation & Context** (~100 words) — Why this algorithm exists, what problem it solves, where it fits in the SAF analysis pipeline.

2. **Key Concepts** (~150 words) — Glossary-style definitions of essential terms. Each term in **bold** with a one-sentence definition. This is the prerequisite knowledge.

3. **Algorithm Overview** (~200 words) — High-level walkthrough in plain English, mapping to the pseudocode phases. References the phase bar.

4. **Formal Properties** (~100 words) — Complexity, soundness/completeness, monotonicity. One paragraph for grad students who care about theoretical guarantees.

### Example: Andersen's PTA

> **Motivation & Context**
>
> When analyzing C programs, we need to know which pointers can refer to which memory locations. Without this, every pointer dereference is ambiguous — we can't track data flow, detect use-after-free, or reason about aliasing. Pointer analysis is the foundation that makes all other SAF analyses precise. Andersen's analysis gives us a sound, whole-program answer using an inclusion-based approach.
>
> **Key Concepts**
>
> - **Points-to set** `pts(v)`: The set of memory locations that pointer `v` may reference at any point during execution
> - **Inclusion constraint** `pts(q) ⊇ pts(p)`: Everything `p` points to, `q` also points to (from assignment `q = p`)
> - **Address constraint** `loc ∈ pts(p)`: Pointer `p` directly takes the address of location `loc` (from `p = &x`)
> - **Fixed point**: The state where no more points-to information can be derived — the algorithm terminates
> - **Worklist**: A queue of pointer variables whose points-to sets recently changed and need their constraints re-evaluated
>
> **Algorithm Overview**
>
> Andersen's analysis proceeds in three phases. First, **extract constraints** from the program: each `p = &x` becomes an address constraint, each `q = p` becomes a copy constraint, and pointer dereferences create load/store constraints. Second, **process the worklist**: starting with variables that have address constraints (rank 0), resolve each variable's constraints and propagate new points-to information to dependent variables. Third, **reach fixed point**: when the worklist is empty and no new information flows, the analysis is complete. The result maps every pointer to its possible targets.
>
> **Formal Properties**
>
> Flow-insensitive (ignores statement order), context-insensitive (merges all calling contexts). Subset-based / inclusion-based. Time complexity O(n³) worst case where n is the number of pointer variables. **Sound**: never misses a real alias pair. **May be imprecise**: can report aliases that never actually occur at runtime. Monotone over a lattice of points-to sets ordered by ⊆.

## Component 4: Bottom Panel (Explanation + State)

The bottom ~35% of the viewport splits explanation and state inspector side by side.

### Layout

- **Left (40%):** Explanation panel — current step's narrative text, always visible (no longer hidden in `<details>`)
- **Right (60%):** State inspector — algorithm-specific state rendering (existing PtaInspector, IfdsInspector, etc.)
- Subtle top border separating from graph area
- Section headers in `text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted)`

### State Inspector Improvements

The existing inspectors stay but benefit from more horizontal space:
- Points-to tables can use full width
- Worklist waves have room to display without wrapping
- Constraint lists aren't cramped into a 40%-width panel

## Files to Modify

### Trace JSON files (8 files — add `phases`, `pseudocode`, `activeLines`)
- `tutorials/public/content/pta-andersen/pta-andersen-basic.trace.json`
- `tutorials/public/content/ifds-taint/ifds-taint-basic.trace.json`
- `tutorials/public/content/interval-analysis/interval-analysis-basic.trace.json`
- `tutorials/public/content/memory-ssa/memory-ssa-basic.trace.json`
- `tutorials/public/content/pta-kcfa/pta-kcfa-basic.trace.json`
- `tutorials/public/content/sparse-valueflow/sparse-vf-basic.trace.json`
- `tutorials/public/content/dominator-tree/dominator-tree-basic.trace.json`
- `tutorials/public/content/callgraph-construction/callgraph-construction-basic.trace.json`

### Tutorial content (8 steps.json — rewrite prose step)
- Same 8 directories' `steps.json` files

### Components (modify/create)
- `tutorials/src/components/AlgorithmStepper.tsx` — Major rewrite (new layout)
- `tutorials/src/components/AlgorithmStepper.css` — Major rewrite (new layout)
- `tutorials/src/components/PhaseBar.tsx` — **New** component
- `tutorials/src/components/PseudocodeRail.tsx` — **New** component
- `tutorials/src/content/trace-types.ts` — Add `phases`, `pseudocode`, `activeLines` to types

### No changes needed
- State inspectors (8 files) — same components, just get more space
- StepperControls.tsx — same transport bar
- GraphViewer.tsx — same renderer
- ExplanationPanel.tsx — same renderer (just no longer in `<details>`)
