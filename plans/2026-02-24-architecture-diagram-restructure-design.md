# Architecture Diagram Restructure — Design

## Problem

The current architecture diagram renders all nodes in each layer as a flat grid of equal-sized cards. This fails to convey important structural relationships:

- **Protocol layer**: `handle_request()` is a dispatcher/gateway to `schema`, `check`, `query`, `analyze` — but they look like peers
- **Engines layer**: `ProgramDatabase` is a facade that owns all engines — but it's just another card
- **IR layer**: `AIR Bundle` is the canonical core output, while the others are frontends that produce it
- **Shells layer**: Mixes APIs, web apps, and static sites without distinction
- **Query layer**: Mixes input specs with output formats
- All layers lack internal grouping that would make the project structure immediately clear

## Solution: Hub + Sub-group Architecture

### Data Model Changes

Add optional `groups` to each layer. Nodes remain in the flat `nodes` array (single source of truth); groups reference them by ID.

```typescript
// types.ts additions
export interface NodeGroup {
  id: string;
  label: string;        // e.g. "Graph Construction", "Actions"
  hub?: string;          // node ID that acts as the hub/gateway
  children: string[];    // node IDs in this group
}

// Layer gains optional groups field
export interface Layer {
  id: string;
  label: string;
  color: string;
  groups?: NodeGroup[];  // NEW: structured groups within a layer
  nodes: ArchNode[];     // node definitions (always present)
}
```

### Visual Rendering

When a layer has `groups`:
1. **Hub node** — rendered as a wide card spanning full grid width, with a left-border accent
2. **Children** — rendered as a standard 4-column grid below the hub, slightly indented
3. **Sub-group labels** — small caps label above each sub-group's children
4. **No-hub groups** — just a labeled sub-section with a grid of children

When a layer has no `groups`, it renders as the current flat grid (backwards compatible).

### Layer Restructuring

#### 1. SHELLS — Entry Points / Shells
3 sub-groups, no hub:
- **API** — Python SDK, CLI
- **Web Apps** — Playground, Tutorials
- **Static Sites** — Documentation, Landing Site

#### 2. BRIDGES — Compilation Targets
2 sub-groups, no hub:
- **Native** — PyO3 FFI
- **Browser** — WASM Bridge, Pyodide Runtime, Web Worker

#### 3. PROTOCOL — JSON Protocol
Hub + children:
- **Hub**: handle_request() (wide, full-width card)
- **Actions**: schema, check/check_all, query, analyze (indented grid)

#### 4. QUERY — Query & Export
2 sub-groups, no hub:
- **Input Specs** — Selectors, Checker Specs
- **Output Formats** — PropertyGraph Export, PTA Export, Findings/SARIF

#### 5. ENGINES — Analysis Engines
Hub + 2 sub-groups:
- **Hub**: ProgramDatabase
- **Graph Construction** — Call Graph, Pointer Analysis, Memory SSA, Value Flow Graph
- **Solvers & Checkers** — SVFG Checkers, IFDS/IDE Solvers

#### 6. IR — IR & Frontends
Core + sub-group:
- **Core**: AIR Bundle (hub, prominent centered card)
- **Frontends** — LLVM Frontend, AIR-JSON Frontend, Tree-sitter (browser)

#### 7. INFRA — Build & Deploy
2 sub-groups, no hub:
- **Build Tools** — Docker (dev), wasm-pack, Maturin
- **Deployment** — GitHub Pages

## Files to Modify

1. `packages/shared/src/architecture/types.ts` — add `NodeGroup` interface, update `Layer`
2. `packages/shared/src/architecture/architecture.json` — add `groups` to every layer
3. `site/src/components/ArchitectureDiagram/ArchitectureDiagram.tsx` — update `LayerSection` to render groups
4. `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css` — styles for hub cards, sub-group labels, indented children
5. `packages/shared/src/architecture/generate-mermaid.ts` — update Mermaid generation if it reads groups
