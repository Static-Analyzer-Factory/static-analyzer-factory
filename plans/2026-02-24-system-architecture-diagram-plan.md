# SAF System Architecture Diagram — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a complete system architecture diagram for SAF with two renderings (fancy interactive React + Mermaid) from a single JSON data source, deployed as a standalone `/architecture` page.

**Architecture:** A single `architecture.json` defines all layers, nodes, and connections. The React renderer on the site's `#architecture` route reads it directly to produce an interactive, Shannon-inspired dark-themed diagram with hover/click interactions. A TypeScript script generates `architecture.mmd` from the same JSON for use in docs/PRs.

**Tech Stack:** React 19, framer-motion (already in site as `motion`), CSS modules, SVG for connections, TypeScript build script for Mermaid generation.

**Design doc:** `docs/plans/2026-02-24-system-architecture-diagram-design.md`

---

### Task 1: Create architecture.json data source

**Files:**
- Create: `packages/shared/src/architecture/architecture.json`
- Create: `packages/shared/src/architecture/types.ts`

**Step 1: Create TypeScript type definitions**

Create `packages/shared/src/architecture/types.ts`:

```ts
export interface ArchitectureData {
  meta: { title: string; version: string };
  actors: Actor[];
  layers: Layer[];
  connections: Connection[];
}

export interface Actor {
  id: string;
  label: string;
  icon: 'robot' | 'user';
}

export interface Layer {
  id: string;
  label: string;
  color: string;
  nodes: ArchNode[];
}

export interface ArchNode {
  id: string;
  label: string;
  desc: string;
  crate?: string;
  path?: string;
  tags?: string[];
}

export interface Connection {
  from: string;
  to: string;
  type: 'uses' | 'data' | 'compiles-to' | 'deploys' | 'planned';
  label?: string;
}
```

**Step 2: Create the architecture JSON data**

Create `packages/shared/src/architecture/architecture.json` with the full SAF architecture. All 8 layers (actors through infra), every node from the design doc, and all connections. This is the single source of truth — approximately 250 lines of JSON encoding the architecture model from the design doc.

Key structure:
- `meta`: title + version
- `actors`: `[{id: "ai-agents", ...}, {id: "humans", ...}]`
- `layers`: 7 layers (shells, bridges, protocol, query, engines, ir, infra) each with their nodes
- `connections`: ~30 connections with typed arrows between node IDs

Reference the design doc's Nodes and Connections sections for exact content.

**Step 3: Add the export to shared package**

Add to `packages/shared/package.json` exports:
```json
"./architecture": "./src/architecture/types.ts",
"./architecture/data": "./src/architecture/architecture.json"
```

**Step 4: Verify TypeScript compiles**

Run: `cd site && npx tsc --noEmit`
Expected: No errors related to the new types.

**Step 5: Commit**

```bash
git add packages/shared/src/architecture/
git commit -m "feat: add architecture.json data source and types"
```

---

### Task 2: Build the React ArchitectureDiagram component

**Files:**
- Create: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx`
- Create: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

This is the main component. It reads `architecture.json` and renders the full diagram.

**Step 1: Create the component file**

Create `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx`:

The component should:
- Import `architecture.json` directly (Vite supports JSON imports)
- Import types from the shared package
- Render the `ActorBar` at the top (two columns: AI Agents | Humans)
- Render each `Layer` as a colored-bordered section with its nodes in a grid
- Render an SVG overlay for connections between nodes
- Use `motion` (framer-motion) for entrance animations (stagger nodes per layer)

Structure:
```tsx
import archData from '../../../../packages/shared/src/architecture/architecture.json';
import type { ArchitectureData } from '@saf/web-shared/architecture';
import { motion } from 'motion/react';
import './ArchitectureDiagram.css';

// Cast the imported JSON
const data = archData as ArchitectureData;

export default function ArchitectureDiagram() {
  // State for: hoveredNode, selectedNode, collapsedLayers
  // Refs for: node positions (for SVG connections)
  // Effect: calculate connection paths after layout

  return (
    <div className="arch-diagram">
      <ActorBar actors={data.actors} />
      <div className="arch-layers">
        {data.layers.map((layer, i) => (
          <LayerSection key={layer.id} layer={layer} index={i} />
        ))}
      </div>
      <ConnectionOverlay connections={data.connections} />
      {selectedNode && <DetailPanel node={selectedNode} />}
    </div>
  );
}
```

Inline the sub-components (ActorBar, LayerSection, NodeBox, ConnectionOverlay, DetailPanel) in the same file to start — they can be extracted later if needed.

**Step 2: Create the CSS file**

Create `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`:

Style matching the existing site dark theme (`#0a0a1a`, `#0f0f23`, `#16213e`, `#1e2d4a`). Key styles:
- `.arch-diagram`: full-width, dark background, padding
- `.arch-actor-bar`: flex row with two actor boxes
- `.arch-layer`: bordered section with layer color as border-left, monospace header
- `.arch-node-grid`: CSS grid, responsive columns
- `.arch-node`: rounded box, `#16213e` background, `#1e2d4a` border, hover glow effect
- `.arch-node--hovered`: border changes to layer color, subtle box-shadow
- `.arch-node--connected`: highlighted when a related node is hovered
- `.arch-detail-panel`: fixed/absolute positioned slide-in from right
- `.arch-connection-svg`: absolute positioned SVG overlay, pointer-events none

Use CSS custom properties for layer colors: `--layer-color` set inline per layer.

**Step 3: Verify it renders**

Run: `cd site && npm run dev`
Navigate to localhost and temporarily render `<ArchitectureDiagram />` in App.tsx to see it.
Expected: All layers and nodes visible, dark themed, no console errors.

**Step 4: Commit**

```bash
git add site/src/components/ArchitectureDiagram/
git commit -m "feat: add ArchitectureDiagram React component"
```

---

### Task 3: Wire up the /architecture route

**Files:**
- Modify: `site/src/App.tsx`
- Modify: `site/src/App.css`

**Step 1: Add hash-based routing to App.tsx**

The site has no router. Use a simple hash-based approach:

```tsx
import { useState, useEffect } from 'react';
import ArchitectureDiagram from './components/ArchitectureDiagram/ArchitectureDiagram';

// ... existing imports ...

export default function App() {
  const [route, setRoute] = useState(window.location.hash);

  useEffect(() => {
    const onHash = () => setRoute(window.location.hash);
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  if (route === '#architecture') {
    return (
      <div className="landing">
        <SiteNav />
        <ArchitectureDiagram />
        <Footer />
      </div>
    );
  }

  return (
    <div className="landing">
      <SiteNav />
      <Hero />
      <Features />
      <Personas />
      <TechHighlights />
      <Footer />
    </div>
  );
}
```

**Step 2: Add architecture link to SiteNav**

In the `SiteNav` component, add a link:
```tsx
<a href={`${base}#architecture`}>Architecture</a>
```

Place it between "Home" and "Tutorials".

**Step 3: Verify routing works**

Run: `cd site && npm run dev`
- Navigate to `http://localhost:5173/` → landing page as before
- Navigate to `http://localhost:5173/#architecture` → architecture diagram
- Click "Architecture" in nav → diagram appears
Expected: Both routes work, nav updates correctly.

**Step 4: Commit**

```bash
git add site/src/App.tsx
git commit -m "feat: wire up #architecture route in site app"
```

---

### Task 4: Implement node interactions (hover + click)

**Files:**
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx`
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Add hover state**

When a node is hovered:
- Set `hoveredNodeId` state
- Compute `connectedNodeIds` from `data.connections` (all nodes connected to hovered node)
- Apply `.arch-node--hovered` class to the hovered node
- Apply `.arch-node--connected` class to all connected nodes
- Apply `.arch-connection--highlighted` class to relevant connection paths
- Dim non-connected nodes with `.arch-node--dimmed` (opacity: 0.3)

**Step 2: Add click → DetailPanel**

When a node is clicked:
- Set `selectedNode` state
- Render `<DetailPanel>` — a slide-in panel from the right containing:
  - Node label (h3) and description
  - Crate name with link to `crates/{crate}/src/` on GitHub
  - Tags as colored badges
  - "Connected to:" list of related nodes
  - Close button (X) and click-outside-to-close

**Step 3: Add connection hover animation**

When hovering a connection line:
- Add `pointer-events: stroke` to connection paths
- On hover, animate a dashed pulse along the path (CSS `stroke-dashoffset` animation)
- Show a small label tooltip at the midpoint with the connection label

**Step 4: Add layer collapse/expand**

Click on a layer header to toggle `collapsedLayers` set:
- Collapsed: only show header bar, nodes hidden with `motion` height animation
- Expanded (default): full node grid visible
- Store collapsed state in component state

**Step 5: Verify all interactions**

Run: `cd site && npm run dev`, navigate to `#architecture`
- Hover nodes → connected nodes highlight, others dim
- Click node → detail panel slides in from right
- Hover connection line → pulse animation
- Click layer header → collapses/expands
Expected: All interactions smooth, no console errors.

**Step 6: Commit**

```bash
git add site/src/components/ArchitectureDiagram/
git commit -m "feat: add hover, click, and collapse interactions to architecture diagram"
```

---

### Task 5: Implement SVG connection paths

**Files:**
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx`
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Position tracking with refs**

Use `useRef` to create a map of node element refs:
```tsx
const nodeRefs = useRef<Map<string, HTMLDivElement>>(new Map());
```

After render, calculate positions with `getBoundingClientRect()` relative to the diagram container.

**Step 2: Draw SVG paths**

Create a `<svg>` overlay positioned absolutely over the diagram. For each connection:
- Get source and target node positions (center-bottom → center-top for vertical flow)
- Draw a cubic bezier path: `M x1,y1 C x1,cy x2,cy x2,y2` where `cy` is the midpoint
- Style by connection type:
  - `uses`: solid, 2px, white with 0.4 opacity
  - `data`: solid, 2px, layer color
  - `compiles-to`: dashed (4,4), 1.5px, gray
  - `deploys`: dotted (2,4), 1px, gray
  - `planned`: dashed, 1px, faded white

**Step 3: Recalculate on resize**

Add a `ResizeObserver` on the diagram container to recalculate connection paths when layout changes.

**Step 4: Add entrance animation**

Use `motion.path` with `pathLength` animation (same pattern as the existing `Hero` CFG graphic):
- Connections draw in sequentially, staggered by layer depth
- Each path animates from `pathLength: 0` to `pathLength: 1`

**Step 5: Verify connections render**

Run: `cd site && npm run dev`
Expected: SVG paths visible between connected nodes, resize handles correctly, entrance animation plays.

**Step 6: Commit**

```bash
git add site/src/components/ArchitectureDiagram/
git commit -m "feat: add SVG connection paths with animation"
```

---

### Task 6: Implement "Trace" flow highlighting

**Files:**
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx`
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Build reachability graph**

From `data.connections`, build a directed adjacency list. Given a clicked node, BFS/DFS to find:
- All upstream ancestors (what connects TO this node)
- All downstream descendants (what this node connects TO)
- The full trace path from actors → clicked node → leaf nodes

**Step 2: Add "Trace" mode**

When a node is clicked and "Trace" button in DetailPanel is pressed:
- Enter trace mode: highlight all nodes and edges on the full path
- Animate a "pulse" traveling along the trace path (moving gradient dot)
- Dim all non-path nodes and edges
- Show a "Clear trace" button to exit trace mode

**Step 3: Verify trace works**

Run: `cd site && npm run dev`
- Click "Python SDK" → Trace → should highlight: AI Agents → Python SDK → PyO3 → handle_request → ... → AIR Bundle
- Click "Clear trace" → returns to normal view
Expected: Full path highlighted, pulse animation visible.

**Step 4: Commit**

```bash
git add site/src/components/ArchitectureDiagram/
git commit -m "feat: add trace flow highlighting to architecture diagram"
```

---

### Task 7: Responsive layout

**Files:**
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Add responsive breakpoints**

- `> 1200px`: full horizontal layout, 4-column node grids, SVG connections visible
- `768px–1200px`: 2-column node grids, connections still visible
- `< 768px`: 1-column stacked layout, hide SVG connections (too cluttered), DetailPanel becomes full-screen overlay instead of side panel

**Step 2: Test at each breakpoint**

Use browser DevTools responsive mode.
Expected: Readable at all sizes, no horizontal overflow.

**Step 3: Commit**

```bash
git add site/src/components/ArchitectureDiagram/
git commit -m "feat: add responsive layout for architecture diagram"
```

---

### Task 8: Mermaid generator script

**Files:**
- Create: `packages/shared/src/architecture/generate-mermaid.ts`
- Create: `packages/shared/src/architecture/architecture.mmd` (generated)

**Step 1: Write the generator script**

Create `packages/shared/src/architecture/generate-mermaid.ts`:

```ts
import { readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const data = JSON.parse(readFileSync(join(__dirname, 'architecture.json'), 'utf-8'));

let mmd = 'graph TB\n';

// Actors
mmd += '    subgraph Actors\n';
for (const actor of data.actors) {
  const shape = actor.icon === 'robot' ? `{{${actor.label}}}` : `([${actor.label}])`;
  mmd += `        ${actor.id}${shape}\n`;
}
mmd += '    end\n\n';

// Layers → subgraphs
for (const layer of data.layers) {
  mmd += `    subgraph ${layer.label}\n`;
  for (const node of layer.nodes) {
    mmd += `        ${node.id}["${node.label}"]\n`;
  }
  mmd += '    end\n\n';
}

// Connections → arrows
const arrowStyle: Record<string, string> = {
  uses: '-->',
  data: '==>',
  'compiles-to': '-.->',
  deploys: '-..->',
  planned: '-.->'
};

for (const conn of data.connections) {
  const arrow = arrowStyle[conn.type] || '-->';
  const label = conn.label ? `|${conn.label}|` : '';
  mmd += `    ${conn.from} ${arrow}${label} ${conn.to}\n`;
}

// Layer styling
for (const layer of data.layers) {
  for (const node of layer.nodes) {
    mmd += `    style ${node.id} fill:#16213e,stroke:${layer.color},color:#e0e0e0\n`;
  }
}

const outPath = join(__dirname, 'architecture.mmd');
writeFileSync(outPath, mmd);
console.log(`Generated ${outPath}`);
```

**Step 2: Run the generator**

Run: `npx tsx packages/shared/src/architecture/generate-mermaid.ts`
Expected: Creates `packages/shared/src/architecture/architecture.mmd` with valid Mermaid syntax.

**Step 3: Validate the Mermaid output**

Run: `npx -y @mermaid-js/mermaid-cli mmdc -i packages/shared/src/architecture/architecture.mmd -o /tmp/arch-test.svg`
Expected: Produces a valid SVG. Or paste the `.mmd` content into https://mermaid.live to validate visually.

**Step 4: Add npm script**

Add to `packages/shared/package.json` scripts:
```json
"generate:mermaid": "tsx src/architecture/generate-mermaid.ts"
```

**Step 5: Commit**

```bash
git add packages/shared/src/architecture/generate-mermaid.ts packages/shared/src/architecture/architecture.mmd packages/shared/package.json
git commit -m "feat: add Mermaid generator from architecture.json"
```

---

### Task 9: TypeScript type-check and final polish

**Files:**
- Possibly modify: `site/tsconfig.app.json` (if resolveJsonModule needed)
- Modify: various files for type-check fixes

**Step 1: Enable JSON module resolution if needed**

If TS complains about JSON imports, add to `site/tsconfig.app.json`:
```json
"resolveJsonModule": true
```

**Step 2: Run full type-check**

Run: `cd site && npx tsc --noEmit`
Expected: No errors.

**Step 3: Build the site**

Run: `cd site && npm run build`
Expected: Build succeeds, `dist/` contains the site with architecture page.

**Step 4: Test the build locally**

Run: `cd site && npm run preview`
- Navigate to `http://localhost:4173/#architecture`
Expected: Architecture diagram renders correctly in production build.

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: fix type-check and verify production build"
```

---

### Task 10: Update PROGRESS.md

**Files:**
- Modify: `plans/PROGRESS.md`

**Step 1: Update progress**

Add a new plan entry to the Plans Index:
- `164` — System Architecture Diagram — status: done

Update Session Log with summary of work.

**Step 2: Commit**

```bash
git add plans/PROGRESS.md
git commit -m "docs: update PROGRESS.md with architecture diagram work"
```
