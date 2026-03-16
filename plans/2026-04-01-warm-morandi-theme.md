# Warm Morandi Theme Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the dark navy theme across all four web apps (site, tutorials, playground, docs) with a warm Morandi light theme, powered by a single shared CSS variables file.

**Architecture:** Create `packages/shared/src/ui/theme.css` as the single source of truth for all colors. All apps import it. Create `packages/shared/src/ui/codemirror-morandi.ts` as a custom CodeMirror 6 theme. Update `packages/shared/src/graph/cytoscape-config.ts` graph colors. Mechanically replace hard-coded hex values with `var()` references in ~25 CSS files. Swap `oneDark` imports for the new theme in 5 editor files.

**Tech Stack:** CSS custom properties, CodeMirror 6 `EditorView.theme()` / `HighlightStyle`, Cytoscape.js stylesheet

---

### Task 1: Create shared theme.css

**Files:**
- Create: `packages/shared/src/ui/theme.css`
- Modify: `packages/shared/package.json` (add export)

**Step 1: Create the theme file**

Create `packages/shared/src/ui/theme.css`:

```css
/* ==========================================================================
   SAF Theme — Warm Morandi
   Edit this single file to re-theme the entire site.
   ========================================================================== */

:root {
  /* ── Backgrounds & Surfaces ── */
  --color-bg:              #f5f0eb;
  --color-surface:         #ece5de;
  --color-surface-raised:  #e2d9d0;
  --color-surface-editor:  #eae4dc;
  --color-border:          #d4cac0;
  --color-border-subtle:   #e0d7ce;

  /* ── Text ── */
  --color-text:            #3d3532;
  --color-text-secondary:  #7a6f66;
  --color-text-tertiary:   #a39890;
  --color-text-inverse:    #f5f0eb;

  /* ── Primary Accent (warm clay) ── */
  --color-accent:          #c4856a;
  --color-accent-hover:    #b3745a;
  --color-accent-subtle:   rgba(196, 133, 106, 0.12);

  /* ── Secondary Accent (sage green) ── */
  --color-secondary:       #7d9a7e;
  --color-secondary-hover: #6d8a6e;
  --color-secondary-subtle: rgba(125, 154, 126, 0.12);

  /* ── Semantic ── */
  --color-error:           #c27171;
  --color-error-subtle:    rgba(194, 113, 113, 0.12);
  --color-warning:         #c9a56a;
  --color-warning-subtle:  rgba(201, 165, 106, 0.12);
  --color-info:            #7a9ab5;
  --color-info-subtle:     rgba(122, 154, 181, 0.12);
  --color-success:         #7d9a7e;

  /* ── Scrollbar ── */
  --color-scrollbar-track:       #e8e1d9;
  --color-scrollbar-thumb:       #cdc4ba;
  --color-scrollbar-thumb-hover: #b8aea3;

  /* ── Shadows ── */
  --shadow-sm: 0 1px 2px rgba(61, 53, 50, 0.06);
  --shadow-md: 0 2px 8px rgba(61, 53, 50, 0.08);
}
```

**Step 2: Add export to package.json**

In `packages/shared/package.json`, add this export entry inside `"exports"`:

```json
"./ui/theme.css": "./src/ui/theme.css",
```

**Step 3: Commit**

```bash
git add packages/shared/src/ui/theme.css packages/shared/package.json
git commit -m "feat: add shared Morandi theme CSS variables file"
```

---

### Task 2: Create CodeMirror Morandi theme

**Files:**
- Create: `packages/shared/src/ui/codemirror-morandi.ts`
- Modify: `packages/shared/package.json` (add export)

**Step 1: Create the theme file**

Create `packages/shared/src/ui/codemirror-morandi.ts`:

```typescript
/**
 * SAF CodeMirror 6 theme — Warm Morandi (parchment light).
 *
 * Editor chrome reads from CSS variables where possible so it updates
 * automatically when theme.css changes. Syntax colors are hard-coded
 * Morandi tones (CodeMirror highlight styles don't support var()).
 */
import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import { Extension } from '@codemirror/state';

/** Editor chrome (backgrounds, gutters, cursor, selection). */
const morandiEditorTheme = EditorView.theme(
  {
    '&': {
      backgroundColor: 'var(--color-surface-editor, #eae4dc)',
      color: 'var(--color-text, #3d3532)',
    },
    '.cm-content': {
      caretColor: 'var(--color-text, #3d3532)',
    },
    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: 'var(--color-text, #3d3532)',
    },
    '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
      backgroundColor: 'rgba(196, 133, 106, 0.18)',
    },
    '.cm-panels': {
      backgroundColor: 'var(--color-surface, #ece5de)',
      color: 'var(--color-text, #3d3532)',
    },
    '.cm-panels.cm-panels-top': {
      borderBottom: '1px solid var(--color-border, #d4cac0)',
    },
    '.cm-panels.cm-panels-bottom': {
      borderTop: '1px solid var(--color-border, #d4cac0)',
    },
    '.cm-searchMatch': {
      backgroundColor: 'rgba(201, 165, 106, 0.3)',
      outline: '1px solid rgba(201, 165, 106, 0.5)',
    },
    '.cm-searchMatch.cm-searchMatch-selected': {
      backgroundColor: 'rgba(196, 133, 106, 0.3)',
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(196, 133, 106, 0.08)',
    },
    '.cm-selectionMatch': {
      backgroundColor: 'rgba(125, 154, 126, 0.2)',
    },
    '&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket': {
      backgroundColor: 'rgba(125, 154, 126, 0.3)',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--color-surface, #ece5de)',
      color: 'var(--color-text-tertiary, #a39890)',
      borderRight: '1px solid var(--color-border-subtle, #e0d7ce)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'rgba(196, 133, 106, 0.1)',
    },
    '.cm-foldPlaceholder': {
      backgroundColor: 'var(--color-surface-raised, #e2d9d0)',
      border: 'none',
      color: 'var(--color-text-secondary, #7a6f66)',
    },
    '.cm-tooltip': {
      border: '1px solid var(--color-border, #d4cac0)',
      backgroundColor: 'var(--color-surface, #ece5de)',
      color: 'var(--color-text, #3d3532)',
    },
    '.cm-tooltip .cm-tooltip-arrow:before': {
      borderTopColor: 'var(--color-border, #d4cac0)',
      borderBottomColor: 'var(--color-border, #d4cac0)',
    },
    '.cm-tooltip .cm-tooltip-arrow:after': {
      borderTopColor: 'var(--color-surface, #ece5de)',
      borderBottomColor: 'var(--color-surface, #ece5de)',
    },
    '.cm-tooltip-autocomplete': {
      '& > ul > li[aria-selected]': {
        backgroundColor: 'var(--color-accent-subtle, rgba(196, 133, 106, 0.12))',
        color: 'var(--color-text, #3d3532)',
      },
    },
  },
  { dark: false },
);

/** Syntax highlighting — Morandi-muted tones. */
const morandiHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: '#b5725a' },
  { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: '#3d3532' },
  { tag: [tags.function(tags.variableName), tags.labelName], color: '#956b7d' },
  { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: '#8a6d52' },
  { tag: [tags.definition(tags.name), tags.separator], color: '#3d3532' },
  { tag: [tags.typeName, tags.className, tags.number, tags.changed, tags.annotation,
          tags.modifier, tags.self, tags.namespace], color: '#5d8a8a' },
  { tag: [tags.operator, tags.operatorKeyword, tags.url, tags.escape,
          tags.regexp, tags.special(tags.string)], color: '#7a6f66' },
  { tag: [tags.meta, tags.comment], color: '#a09488' },
  { tag: tags.strong, fontWeight: 'bold' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.strikethrough, textDecoration: 'line-through' },
  { tag: tags.link, color: '#7a9ab5', textDecoration: 'underline' },
  { tag: tags.heading, fontWeight: 'bold', color: '#b5725a' },
  { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: '#5d8a8a' },
  { tag: [tags.processingInstruction, tags.string, tags.inserted], color: '#6b8a5e' },
  { tag: tags.invalid, color: '#c27171' },
  { tag: [tags.propertyName], color: '#8a6d52' },
]);

/** Complete Morandi theme extension — drop-in replacement for oneDark. */
export const morandiTheme: Extension = [
  morandiEditorTheme,
  syntaxHighlighting(morandiHighlightStyle),
];
```

**Step 2: Add export to package.json**

In `packages/shared/package.json`, add this export entry inside `"exports"`:

```json
"./ui/codemirror-morandi": "./src/ui/codemirror-morandi.ts",
```

**Step 3: Install @codemirror/language and @lezer/highlight in shared package**

These are needed by the theme file. Check if they're already available as transitive deps from the apps — if not:

```bash
cd packages/shared && npm install @codemirror/language @codemirror/view @codemirror/state @lezer/highlight
```

**Step 4: Commit**

```bash
git add packages/shared/src/ui/codemirror-morandi.ts packages/shared/package.json
git commit -m "feat: add CodeMirror Morandi theme for parchment editors"
```

---

### Task 3: Update Cytoscape graph colors

**Files:**
- Modify: `packages/shared/src/graph/cytoscape-config.ts`

The `COLORS` object and `darkStylesheet` need Morandi equivalents. The graph backgrounds should use `--color-surface` and node/edge colors should use Morandi-muted versions.

**Step 1: Update COLORS and stylesheet**

In `packages/shared/src/graph/cytoscape-config.ts`:

Replace the `COLORS` object (lines 10-28) with:
```typescript
export const COLORS = {
  bg: '#ece5de',
  nodeBg: '#f5f0eb',
  nodeText: '#3d3532',
  edgeDefault: '#b8aea3',
  entry: '#7d9a7e',
  exit: '#c27171',
  external: '#c9a56a',
  selected: '#c4856a',
  trueBranch: '#7d9a7e',
  falseBranch: '#c27171',
  store: '#c9a56a',
  load: '#7a9ab5',
  defineEdge: '#7d9a7e',
  useEdge: '#7a9ab5',
  value: '#7a9ab5',
  location: '#7d9a7e',
  unknownMem: '#c27171',
} as const;
```

Rename `darkStylesheet` to `stylesheet` (update export in `index.ts` too). Update the hard-coded dark colors inside the stylesheet array:
- `'#4a5568'` border → `'#d4cac0'`
- `'#a0aec0'` edge label color → `'#7a6f66'`
- `'#2d1b69'` selected bg → `'rgba(196, 133, 106, 0.15)'`
- `'#1e3a5f'` value-node bg → `'rgba(122, 154, 181, 0.1)'`
- `'#1a3a2e'` location-node bg → `'rgba(125, 154, 126, 0.1)'`
- `'#3a1a1a'` unknown-mem bg → `'rgba(194, 113, 113, 0.1)'`
- `'#1a2e3e'` instruction bg → `'rgba(122, 154, 181, 0.08)'`
- `'#4a90d9'` instruction border → `'#7a9ab5'`
- `'#f59e0b'` back-edge → `'#c9a56a'`
- text-outline-color references to `COLORS.bg` stay (they now reference the light bg)

**Step 2: Update index.ts export**

In `packages/shared/src/graph/index.ts`, rename the export:
```typescript
export { COLORS, stylesheet, createCyInstance } from './cytoscape-config';
```

**Step 3: Update any imports of `darkStylesheet`**

Search for `darkStylesheet` across all apps and update to `stylesheet`. This is likely only used internally by `createCyInstance` in cytoscape-config.ts itself.

**Step 4: Commit**

```bash
git add packages/shared/src/graph/cytoscape-config.ts packages/shared/src/graph/index.ts
git commit -m "feat: update Cytoscape graph colors to Morandi palette"
```

---

### Task 4: Update shared global nav

**Files:**
- Modify: `packages/shared/src/ui/global-nav.css`
- Modify: `packages/shared/src/ui/SiteNav.tsx` (add theme.css import)

**Step 1: Add theme.css import to SiteNav.tsx**

At line 1 of `packages/shared/src/ui/SiteNav.tsx`, before the existing `import './global-nav.css';`, add:

```typescript
import './theme.css';
import './global-nav.css';
```

This ensures theme.css is loaded for all React apps that use SiteNav.

**Step 2: Update global-nav.css**

Replace the hard-coded colors in `packages/shared/src/ui/global-nav.css`:

```css
.global-nav {
  /* ... keep position/layout unchanged ... */
  background: rgba(226, 217, 208, 0.95);  /* --color-surface-raised with alpha */
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--color-border, #d4cac0);
}

.global-nav-brand {
  /* ... keep font unchanged ... */
  color: var(--color-accent, #c4856a);
}

.global-nav-links a {
  color: var(--color-text-secondary, #7a6f66);
  /* ... keep rest unchanged ... */
}

.global-nav-links a:hover {
  color: var(--color-text, #3d3532);
}

.global-nav-links .global-nav-active {
  color: var(--color-accent, #c4856a);
}
```

**Step 3: Commit**

```bash
git add packages/shared/src/ui/global-nav.css packages/shared/src/ui/SiteNav.tsx
git commit -m "feat: update global nav to Morandi theme variables"
```

---

### Task 5: Update site (landing page)

**Files:**
- Modify: `site/src/index.css`
- Modify: `site/src/App.css`
- Modify: `site/src/components/ArchitectureDiagram/ArchitectureDiagram.css`

**Step 1: Update index.css**

Replace the entire content of `site/src/index.css`:

```css
@import '@saf/web-shared/ui/theme.css';

*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  color: var(--color-text);
  background: var(--color-bg);
  scroll-behavior: smooth;
}
body { min-height: 100vh; }
a { color: var(--color-accent); text-decoration: none; }
a:hover { text-decoration: underline; }
```

Note: The `@import` may not work via the package export path depending on the bundler. If it doesn't resolve, import theme.css via the SiteNav component import chain (SiteNav.tsx already imports theme.css from Task 4, so simply using `<SiteNav>` in App.tsx is sufficient). In that case, just replace the hard-coded colors with `var()` references without the `@import`.

**Step 2: Update App.css**

This is the largest file (~396 lines). Systematically replace every hard-coded dark color:

**Color mapping for site/src/App.css:**
| Old | New |
|-----|-----|
| `#0a0a1a`, `#0f0f23` (dark bg) | `var(--color-bg)` |
| `#16213e`, `#1a1a2e` (card bg) | `var(--color-surface)` |
| `#0f3460` (darker panel) | `var(--color-surface-raised)` |
| `#e0e0e0` (primary text) | `var(--color-text)` |
| `#a0a0b0`, `#718096` (muted text) | `var(--color-text-secondary)` |
| `#7c3aed` (violet accent) | `var(--color-accent)` |
| `#10b981` (teal/green) | `var(--color-secondary)` |
| `#ef4444` (red) | `var(--color-error)` |
| `#f59e0b` (amber) | `var(--color-warning)` |
| `#3b82f6` (blue) | `var(--color-info)` |
| `#1e2d4a`, `#2a2a4a`, `#3d3d5f` (borders) | `var(--color-border)` |
| `rgba(124, 58, 237, 0.1)` (accent bg) | `var(--color-accent-subtle)` |
| `rgba(16, 185, 129, 0.1)` (green bg) | `var(--color-secondary-subtle)` |
| White text on dark → | Keep `var(--color-text)` or `var(--color-text-inverse)` as appropriate |

**Gradient backgrounds:** Replace dark gradients like `linear-gradient(135deg, #0a0a1a, #16213e)` with `linear-gradient(135deg, var(--color-bg), var(--color-surface))`.

**Box shadows:** Replace dark shadows with `var(--shadow-sm)` or `var(--shadow-md)`.

**Persona cards** colored top bars: use Morandi accent colors:
- Researcher → `var(--color-accent)` (warm clay)
- Developer → `var(--color-info)` (muted blue)
- Student → `var(--color-secondary)` (sage green)

**Step 3: Update ArchitectureDiagram.css**

Replace architecture tag colors (~35 tags) with Morandi equivalents. General mapping:
- Blue tones → `rgba(122, 154, 181, 0.12)` bg / `#7a9ab5` text
- Green tones → `rgba(125, 154, 126, 0.12)` bg / `#7d9a7e` text
- Red tones → `rgba(194, 113, 113, 0.12)` bg / `#c27171` text
- Amber tones → `rgba(201, 165, 106, 0.12)` bg / `#c9a56a` text
- Purple/violet tones → `rgba(149, 107, 125, 0.12)` bg / `#956b7d` text
- Clay/orange tones → `rgba(196, 133, 106, 0.12)` bg / `#c4856a` text

Replace layer backgrounds, node backgrounds, connection lines, detail panel colors similarly.

**Step 4: Commit**

```bash
git add site/src/index.css site/src/App.css site/src/components/ArchitectureDiagram/ArchitectureDiagram.css
git commit -m "feat: apply Morandi theme to landing page"
```

---

### Task 6: Update tutorials app

**Files:**
- Modify: `tutorials/src/App.css`
- Modify: `tutorials/src/components/NavBar.css`
- Modify: `tutorials/src/components/Sidebar.css`
- Modify: `tutorials/src/components/Catalog.css`
- Modify: `tutorials/src/components/TutorialPage.css`
- Modify: `tutorials/src/components/AlgorithmStepper.css`
- Modify: `tutorials/src/components/PseudocodeRail.css`
- Modify: `tutorials/src/components/PhaseBar.css`
- Modify: `tutorials/src/components/InteractiveStep.css`
- Modify: `tutorials/src/components/StepperControls.css`
- Modify: `tutorials/src/components/CodeBlock.tsx`

**Step 1: Update App.css root variables**

Replace the `:root` block (lines 5-11) in `tutorials/src/App.css`:

```css
:root {
  --bg: var(--color-bg, #f5f0eb);
  --surface: var(--color-surface, #ece5de);
  --text: var(--color-text, #3d3532);
  --accent: var(--color-accent, #c4856a);
  --accent-secondary: var(--color-secondary, #7d9a7e);
}
```

This bridges the existing `--bg`/`--surface`/`--text`/`--accent` variables (used throughout tutorial CSS files) to the shared theme variables. All component CSS files that already use `var(--bg)` etc. will pick up the new colors automatically.

Then do a sweep of the file for any remaining hard-coded colors:
- `#0f0f23` → `var(--bg)`
- `#1a1a2e` → `var(--surface)`
- `#e0e0e0` → `var(--text)`
- `#10b981` → `var(--accent)`
- `#7c3aed` → `var(--accent-secondary)`
- `#2a2a4a` (borders) → `var(--color-border)`
- `#ef4444` (error) → `var(--color-error)`
- `#fcd54f`, `rgba(245, 158, 11, 0.15)` (highlight) → `var(--color-accent)` / `var(--color-accent-subtle)`
- Scrollbar colors → `var(--color-scrollbar-*)`

**Step 2: Sweep all component CSS files**

For each component CSS file, replace hard-coded colors with `var()` references using the same mapping. Most already use `var(--bg)`, `var(--surface)`, etc., but some have inline hex values.

Key replacements across component files:
| Pattern | Replacement |
|---------|-------------|
| `#0f0f23` | `var(--bg)` |
| `#1a1a2e` | `var(--surface)` |
| `#2a2a4a`, `#2d2d4a` | `var(--color-border)` |
| `#3a3a5a` | `var(--color-scrollbar-thumb-hover)` |
| `#10b981` | `var(--accent)` |
| `#7c3aed` | `var(--accent-secondary)` |
| `#e0e0e0` | `var(--text)` |
| `#a0a0b0`, `#aaa` | `var(--color-text-secondary)` |
| `#ef4444` | `var(--color-error)` |
| `#f59e0b` | `var(--color-warning)` |
| `rgba(16, 185, 129, ...)` | `var(--color-secondary-subtle)` or specific Morandi equivalent |
| `rgba(124, 58, 237, ...)` | `var(--color-accent-subtle)` or specific |

**Step 3: Update CodeBlock.tsx**

In `tutorials/src/components/CodeBlock.tsx`:
- Replace `import { oneDark } from '@codemirror/theme-one-dark';` with:
  ```typescript
  import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';
  ```
- In the extensions array, replace `oneDark` with `morandiTheme`.
- Update any hard-coded highlight colors (e.g., `rgba(245, 158, 11, 0.15)` → `rgba(196, 133, 106, 0.15)`, `#fcd54f` → `#c4856a`).

**Step 4: Commit**

```bash
git add tutorials/src/
git commit -m "feat: apply Morandi theme to tutorials app"
```

---

### Task 7: Update playground app

**Files:**
- Modify: `playground/src/index.css`
- Modify: `playground/src/App.css`
- Modify: `playground/src/components/AnalyzerPanel.css`
- Modify: `playground/src/components/QueryPanel.css`
- Modify: `playground/src/components/TutorialPanel.css`
- Modify: `playground/src/components/SourcePanel.tsx`
- Modify: `playground/src/components/AnalyzerPanel.tsx`
- Modify: `playground/src/components/QueryPanel.tsx`
- Modify: `playground/src/components/CompiledIRPanel.tsx`

**Step 1: Update index.css**

Replace hard-coded colors in `playground/src/index.css`:

```css
:root {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Cascadia Code', monospace;
  line-height: 1.4;
  font-weight: 400;
  color: var(--color-text);
  background-color: var(--color-bg);
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

Replace `select` styles:
- `background: #16213e` → `background: var(--color-surface)`
- `color: #e0e0e0` → `color: var(--color-text)`
- `border: 1px solid #2d2d3f` → `border: 1px solid var(--color-border)`
- `outline: 1px solid #533483` → `outline: 1px solid var(--color-accent)`

Replace scrollbar styles:
- `#16213e` → `var(--color-scrollbar-track)`
- `#2d2d3f` → `var(--color-scrollbar-thumb)`
- `#3d3d5f` → `var(--color-scrollbar-thumb-hover)`

**Step 2: Update App.css**

Systematic replacement (~498 lines). Key mappings:
| Old | New |
|-----|-----|
| `#1a1a2e` (bg) | `var(--color-bg)` |
| `#16213e` (panel bg) | `var(--color-surface)` |
| `#0f3460` (header) | `var(--color-surface-raised)` |
| `#e0e0e0` (text) | `var(--color-text)` |
| `#a0aec0`, `#718096` (muted) | `var(--color-text-secondary)` |
| `#7c3aed` (violet) | `var(--color-accent)` |
| `#10b981` (green) | `var(--color-secondary)` |
| `#ef4444` (red) | `var(--color-error)` |
| `#f59e0b` (amber) | `var(--color-warning)` |
| `#4a5568` (gray) | `var(--color-text-tertiary)` |
| `#1e2d4a`, `#2d2d5f` (border) | `var(--color-border)` |
| `#533483` (focus) | `var(--color-accent)` |

Status dots:
- `.idle` bg `#4a5568` → `var(--color-text-tertiary)`
- `.ready` bg `#10b981` → `var(--color-success)`
- `.error` bg `#ef4444` → `var(--color-error)`
- `.compiling` etc. bg `#f59e0b` → `var(--color-warning)`

**Step 3: Update component CSS files**

Apply the same hex→var() mapping to `AnalyzerPanel.css`, `QueryPanel.css`, `TutorialPanel.css`.

Severity colors in AnalyzerPanel.css and QueryPanel.css:
- `.severity-critical`, `.severity-high` → `color: var(--color-error)`
- `.severity-medium` → `color: var(--color-warning)`
- `.severity-low`, `.severity-info` → `color: var(--color-info)`

**Step 4: Update editor TypeScript files**

In each of these files, replace the `oneDark` import and usage:

**`playground/src/components/SourcePanel.tsx`:**
- Replace `import { oneDark } from '@codemirror/theme-one-dark';` → `import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';`
- Replace `oneDark,` in extensions → `morandiTheme,`
- Update `highlightTheme` colors: `rgba(255, 213, 79, 0.15)` → `rgba(196, 133, 106, 0.15)`, `#ffd54f` → `var(--color-accent)`

**`playground/src/components/AnalyzerPanel.tsx`:**
- Replace `import { oneDark } from '@codemirror/theme-one-dark';` → `import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';`
- Replace `oneDark,` in extensions → `morandiTheme,`

**`playground/src/components/QueryPanel.tsx`:**
- Replace `import { oneDark } from '@codemirror/theme-one-dark';` → `import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';`
- Replace `oneDark,` in extensions → `morandiTheme,`

**`playground/src/components/CompiledIRPanel.tsx`:**
- Replace `import { oneDark } from '@codemirror/theme-one-dark';` → `import { morandiTheme } from '@saf/web-shared/ui/codemirror-morandi';`
- Replace `oneDark,` in extensions → `morandiTheme,`

**Step 5: Commit**

```bash
git add playground/src/
git commit -m "feat: apply Morandi theme to playground app"
```

---

### Task 8: Update docs (mdBook)

**Files:**
- Modify: `docs/book/src/custom.css`
- Modify: `docs/book/src/global-nav.css`
- Modify: `docs/book/book.toml`
- Create: `docs/book/src/theme.css` (standalone copy for mdBook — can't use package imports)

**Step 1: Create standalone theme.css for mdBook**

mdBook can't import from `packages/shared/` via bundler. Create a standalone copy of the CSS variables at `docs/book/src/theme.css`:

```css
/* SAF Theme variables for mdBook (standalone copy of packages/shared/src/ui/theme.css) */
:root {
  --color-bg:              #f5f0eb;
  --color-surface:         #ece5de;
  --color-surface-raised:  #e2d9d0;
  --color-border:          #d4cac0;
  --color-border-subtle:   #e0d7ce;
  --color-text:            #3d3532;
  --color-text-secondary:  #7a6f66;
  --color-text-tertiary:   #a39890;
  --color-accent:          #c4856a;
  --color-accent-hover:    #b3745a;
  --color-secondary:       #7d9a7e;
  --color-error:           #c27171;
  --color-warning:         #c9a56a;
  --color-info:            #7a9ab5;
}
```

**Step 2: Update book.toml**

Change the default theme from `navy` to `light` and add theme.css:

```toml
[output.html]
default-theme = "light"
preferred-dark-theme = "light"
git-repository-url = "https://github.com/ThePatrickStar/static-analyzer-factory"
additional-css = ["src/theme.css", "src/custom.css", "src/global-nav.css"]
additional-js = ["src/global-nav.js"]
```

**Step 3: Update global-nav.css**

Replace colors in `docs/book/src/global-nav.css` with the same values as the shared version (Task 4):

```css
.global-nav {
  background: rgba(226, 217, 208, 0.95);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--color-border, #d4cac0);
  /* rest unchanged */
}

.global-nav-brand {
  color: var(--color-accent, #c4856a);
}

.global-nav-links a {
  color: var(--color-text-secondary, #7a6f66);
}

.global-nav-links a:hover {
  color: var(--color-text, #3d3532);
}

.global-nav-links .global-nav-active {
  color: var(--color-accent, #c4856a);
}
```

**Step 4: Update custom.css**

```css
.saf-widget { border: 1px solid var(--color-border, #d4cac0); border-radius: 8px; overflow: hidden; margin: 1em 0; }
.saf-widget iframe { width: 100%; height: 400px; border: none; }
```

**Step 5: Commit**

```bash
git add docs/book/
git commit -m "feat: apply Morandi theme to docs (mdBook)"
```

---

### Task 9: Visual verification and cleanup

**Step 1: Build and check each app**

Run the dev server for each app and visually inspect:

```bash
cd site && npm run dev          # Check landing page
cd tutorials && npm run dev     # Check tutorials
cd playground && npm run dev    # Check playground
cd docs/book && mdbook serve    # Check docs
```

**Step 2: Check for any remaining hard-coded dark colors**

Search for leftover dark hex codes across all frontend files:

```bash
grep -rn '#0a0a1a\|#0f0f23\|#1a1a2e\|#16213e\|#0f3460' site/src/ tutorials/src/ playground/src/ docs/book/src/ packages/shared/src/
```

Any hits should be replaced with the appropriate `var()` reference.

**Step 3: Check for contrast issues**

Verify text readability on all background levels. Key checks:
- Primary text (`#3d3532`) on bg (`#f5f0eb`) — ratio ~8.5:1 (AAA)
- Secondary text (`#7a6f66`) on bg (`#f5f0eb`) — ratio ~4.5:1 (AA)
- Accent (`#c4856a`) on bg (`#f5f0eb`) — ratio ~3.2:1 (check large text only)
- Inverse text (`#f5f0eb`) on accent (`#c4856a`) — ratio ~3.2:1 (use for large text/buttons only)

If accent text on bg fails AA for small text, use `var(--color-accent-hover)` (`#b3745a`, ~4:1) for small link text.

**Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: clean up remaining dark theme artifacts"
```
