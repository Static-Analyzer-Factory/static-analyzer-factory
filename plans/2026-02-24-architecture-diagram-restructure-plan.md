# Architecture Diagram Restructure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restructure all 7 layers of the architecture diagram to show hub/dispatcher relationships and logical sub-groups, making the project's internal structure visually clear.

**Architecture:** The existing flat-grid layer rendering is extended with an optional `groups` field on each layer. When present, groups define hub nodes (rendered as wide cards) and named sub-groups (rendered as labeled sections with indented grids). The node definitions stay in the flat `nodes` array; groups reference them by ID.

**Tech Stack:** TypeScript, React, CSS, JSON data model

---

### Task 1: Update TypeScript types

**Files:**
- Modify: `packages/shared/src/architecture/types.ts`

**Step 1: Add `NodeGroup` interface and update `Layer`**

Replace the entire file with:

```typescript
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

export interface NodeGroup {
  id: string;
  label: string;
  hub?: string;
  children: string[];
}

export interface Layer {
  id: string;
  label: string;
  color: string;
  groups?: NodeGroup[];
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

**Step 2: Verify the site still compiles**

Run: `npx tsc --noEmit` from `site/` directory (or check that the dev server shows no type errors).

**Step 3: Commit**

```
feat: add NodeGroup type for architecture diagram sub-groups
```

---

### Task 2: Add groups to architecture.json

**Files:**
- Modify: `packages/shared/src/architecture/architecture.json`

**Step 1: Add `groups` field to every layer**

Add a `"groups"` array to each layer in `architecture.json`. The node definitions in `"nodes"` remain unchanged. Here are the groups for each layer:

**Shells layer** (after `"color": "#0ea5e9",`):
```json
"groups": [
  { "id": "shells-api", "label": "API", "children": ["python-sdk", "cli"] },
  { "id": "shells-webapps", "label": "Web Apps", "children": ["playground", "tutorials"] },
  { "id": "shells-static", "label": "Static Sites", "children": ["docs", "site"] }
],
```

**Bridges layer** (after `"color": "#8b5cf6",`):
```json
"groups": [
  { "id": "bridges-native", "label": "Native", "children": ["pyo3"] },
  { "id": "bridges-browser", "label": "Browser", "children": ["wasm-bridge", "pyodide", "web-worker"] }
],
```

**Protocol layer** (after `"color": "#f59e0b",`):
```json
"groups": [
  { "id": "protocol-dispatch", "label": "Actions", "hub": "handle-request", "children": ["schema-action", "check-action", "query-action", "analyze-action"] }
],
```

**Query layer** (after `"color": "#10b981",`):
```json
"groups": [
  { "id": "query-input", "label": "Input Specs", "children": ["selectors", "checker-specs"] },
  { "id": "query-output", "label": "Output Formats", "children": ["property-graph", "pta-export", "findings"] }
],
```

**Engines layer** (after `"color": "#ef4444",`):
```json
"groups": [
  { "id": "engines-graphs", "label": "Graph Construction", "hub": "program-db", "children": ["callgraph", "pta", "mssa", "vfg"] },
  { "id": "engines-solvers", "label": "Solvers & Checkers", "children": ["svfg-checkers", "ifds-ide"] }
],
```

**IR layer** (after `"color": "#6366f1",`):
```json
"groups": [
  { "id": "ir-core", "label": "Core", "hub": "air-bundle", "children": [] },
  { "id": "ir-frontends", "label": "Frontends", "children": ["llvm-frontend", "air-json-frontend", "tree-sitter"] }
],
```

**Infra layer** (after `"color": "#64748b",`):
```json
"groups": [
  { "id": "infra-build", "label": "Build Tools", "children": ["docker", "wasm-pack", "maturin"] },
  { "id": "infra-deploy", "label": "Deployment", "children": ["github-pages"] }
],
```

**Step 2: Verify JSON is valid**

Run: `node -e "JSON.parse(require('fs').readFileSync('packages/shared/src/architecture/architecture.json', 'utf8')); console.log('OK')"`

**Step 3: Commit**

```
feat: add group structure to all architecture layers
```

---

### Task 3: Add CSS for hub nodes and sub-group labels

**Files:**
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Add styles for sub-groups, hub nodes, and group labels**

Add the following CSS before the `/* ── Connection SVG Overlay ── */` comment (around line 272):

```css
/* ── Sub-groups ── */

.arch-groups {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0 1rem 1rem;
}

.arch-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.arch-group-label {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  font-size: 0.65rem;
  font-weight: 600;
  color: #718096;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding-left: 0.25rem;
}

/* ── Hub Node (wide card) ── */

.arch-hub-node {
  grid-column: 1 / -1;
}

.arch-hub-node .arch-node {
  border-left: 3px solid var(--layer-color, #7c3aed);
  background: linear-gradient(135deg, #16213e 0%, rgba(var(--layer-color-rgb, 124, 58, 237), 0.06) 100%);
}

/* ── Group Children Grid ── */

.arch-group-children {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 0.75rem;
  padding-left: 1rem;
  border-left: 1px solid rgba(255, 255, 255, 0.06);
  margin-left: 0.5rem;
}

@media (min-width: 1201px) {
  .arch-group-children {
    grid-template-columns: repeat(4, 1fr);
  }
}

@media (min-width: 769px) and (max-width: 1200px) {
  .arch-group-children {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .arch-group-children {
    grid-template-columns: 1fr;
    padding-left: 0.5rem;
  }
}
```

**Step 2: Commit**

```
feat: add CSS styles for architecture hub nodes and sub-groups
```

---

### Task 4: Update LayerSection rendering to support groups

**Files:**
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx`

This is the main rendering change. The `LayerSection` component needs to check if the layer has `groups` and render them with hubs and sub-group labels instead of a flat grid.

**Step 1: Add a helper to look up a node by ID**

Add this helper function near the other helpers (around line 70, after `getNodeConnections`):

```typescript
/** Look up a node definition by ID from any layer */
function findNode(nodeId: string): ArchNode | undefined {
  for (const layer of data.layers) {
    const found = layer.nodes.find((n) => n.id === nodeId);
    if (found) return found;
  }
  return undefined;
}
```

**Step 2: Replace the LayerSection component**

Replace the `LayerSection` function (lines 317-360) with the following. The props interface remains unchanged:

```typescript
function LayerSection({ layer, isOpen, onToggle, hoveredNode, connectedToHovered, selectedNode, traceNodeIds, onNodeHover, onNodeClick, nodeRefCallback }: LayerSectionProps) {
  // Build set of node IDs that appear in groups (to avoid duplicates in flat fallback)
  const groupedNodeIds = useMemo(() => {
    if (!layer.groups) return new Set<string>();
    const ids = new Set<string>();
    for (const group of layer.groups) {
      if (group.hub) ids.add(group.hub);
      for (const childId of group.children) ids.add(childId);
    }
    return ids;
  }, [layer.groups]);

  const nodeCount = layer.nodes.length;

  const renderNode = (nodeId: string, index: number) => {
    const node = layer.nodes.find((n) => n.id === nodeId);
    if (!node) return null;
    return (
      <NodeBox
        key={node.id}
        node={node}
        layerColor={layer.color}
        index={index}
        hoveredNode={hoveredNode}
        connectedToHovered={connectedToHovered}
        selectedNode={selectedNode}
        traceNodeIds={traceNodeIds}
        onHover={onNodeHover}
        onClick={onNodeClick}
        nodeRefCallback={nodeRefCallback}
      />
    );
  };

  return (
    <div className="arch-layer" style={{ '--layer-color': layer.color } as React.CSSProperties}>
      <button className="arch-layer-header" onClick={onToggle} aria-expanded={isOpen}>
        <span className="arch-layer-label">{layer.id.toUpperCase()}</span>
        <span className="arch-layer-name">{layer.label}</span>
        <span className="arch-layer-count">{nodeCount}</span>
        <span className={`arch-layer-chevron${isOpen ? ' arch-layer-chevron--open' : ''}`}>
          <ChevronDown />
        </span>
      </button>
      <AnimatePresence initial={false}>
        {isOpen && (
          <motion.div
            key="content"
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.25, ease: 'easeInOut' }}
            style={{ overflow: 'hidden' }}
          >
            {layer.groups ? (
              <div className="arch-groups">
                {layer.groups.map((group) => {
                  let nodeIndex = 0;
                  return (
                    <div key={group.id} className="arch-group">
                      {group.hub && (
                        <div
                          className="arch-hub-node"
                          style={{
                            '--layer-color-rgb': hexToRgb(layer.color),
                          } as React.CSSProperties}
                        >
                          {renderNode(group.hub, nodeIndex++)}
                        </div>
                      )}
                      <div className="arch-group-label">{group.label}</div>
                      {group.children.length > 0 && (
                        <div className="arch-group-children">
                          {group.children.map((childId) => renderNode(childId, nodeIndex++))}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            ) : (
              <div className="arch-node-grid">
                {layer.nodes.map((node, i) => (
                  <NodeBox
                    key={node.id}
                    node={node}
                    layerColor={layer.color}
                    index={i}
                    hoveredNode={hoveredNode}
                    connectedToHovered={connectedToHovered}
                    selectedNode={selectedNode}
                    traceNodeIds={traceNodeIds}
                    onHover={onNodeHover}
                    onClick={onNodeClick}
                    nodeRefCallback={nodeRefCallback}
                  />
                ))}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
```

Note: Add `useMemo` to the import on line 3 if not already present (it already is).

**Step 3: Import `NodeGroup` type**

Update the import on line 2:

```typescript
import type { ArchitectureData, ArchNode, Connection, NodeGroup } from '../../../../packages/shared/src/architecture/types';
```

**Step 4: Verify the dev server renders correctly**

Open `http://localhost:<port>/#architecture` and confirm:
- Protocol layer shows `handle_request()` as a wide hub card with 4 actions indented below it
- Engines layer shows `ProgramDatabase` as hub, with "Graph Construction" and "Solvers & Checkers" sub-groups
- All other layers show labeled sub-groups
- Hover/click/trace interactions still work

**Step 5: Commit**

```
feat: render architecture diagram with hub nodes and sub-groups
```

---

### Task 5: Update Mermaid generator for groups

**Files:**
- Modify: `packages/shared/src/architecture/generate-mermaid.ts`

**Step 1: Update layer subgraph generation to use groups**

Replace the `// --- Layer subgraphs ---` section (lines 29-37) with:

```typescript
// --- Layer subgraphs ---
for (const layer of data.layers) {
  lines.push(`  subgraph ${layer.id}["${layer.label}"]`);
  if (layer.groups) {
    for (const group of layer.groups) {
      lines.push(`    subgraph ${group.id}["${group.label}"]`);
      if (group.hub) {
        const hubNode = layer.nodes.find((n) => n.id === group.hub);
        if (hubNode) lines.push(`      ${hubNode.id}["${hubNode.label}"]`);
      }
      for (const childId of group.children) {
        const child = layer.nodes.find((n) => n.id === childId);
        if (child) lines.push(`      ${child.id}["${child.label}"]`);
      }
      lines.push('    end');
    }
  } else {
    for (const node of layer.nodes) {
      lines.push(`    ${node.id}["${node.label}"]`);
    }
  }
  lines.push('  end');
  lines.push('');
}
```

**Step 2: Regenerate Mermaid file**

Run: `npx tsx packages/shared/src/architecture/generate-mermaid.ts` from the repo root.

Verify: `packages/shared/src/architecture/architecture.mmd` contains nested subgraphs.

**Step 3: Commit**

```
feat: update Mermaid generator to render nested sub-groups
```

---

### Task 6: Visual polish and verification

**Files:**
- Possibly tweak: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Open the diagram in the browser and verify all layers**

Check each layer one by one:
1. **Shells**: 3 sub-groups (API, Web Apps, Static Sites) with labeled sections
2. **Bridges**: 2 sub-groups (Native, Browser)
3. **Protocol**: Hub card for `handle_request()` + 4 action children indented below
4. **Query**: 2 sub-groups (Input Specs, Output Formats)
5. **Engines**: Hub card for `ProgramDatabase` + 2 sub-groups (Graph Construction, Solvers & Checkers)
6. **IR**: Hub card for `AIR Bundle` + Frontends sub-group
7. **Infra**: 2 sub-groups (Build Tools, Deployment)

**Step 2: Verify hover highlights still work**

Hover over `handle_request()` — it should highlight the 4 action nodes and all upstream nodes (bridges, shells). The SVG connection lines should highlight.

**Step 3: Verify click detail panel still works**

Click any node — the detail panel should still slide in from the right with metadata and connections.

**Step 4: Verify trace mode**

Click a node → click "Trace data flow" → trace banner appears, nodes on the path highlight.

**Step 5: Fix any visual issues found**

Possible adjustments:
- Hub card padding/margin
- Sub-group label spacing
- Indentation depth on mobile

**Step 6: Commit if any CSS tweaks were made**

```
style: polish architecture diagram hub and sub-group visuals
```
