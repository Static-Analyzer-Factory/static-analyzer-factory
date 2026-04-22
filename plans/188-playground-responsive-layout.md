# Plan 188 — Playground responsive layout

**Status:** approved
**Area:** frontend
**Branch:** `ui/playground-responsive-layout`

## Problem

The playground's main workspace uses a fixed 3-column CSS grid
(`grid-template-columns: 1fr 1fr 1.2fr` in `playground/src/App.css:37`)
with no media queries. On narrow viewports (phones, portrait tablets, half-
screen windows) the three panels — SOURCE, COMPILED IR, ANALYSIS — each
collapse to ~130px, truncating editor content and rendering the graph
area unusable. The header tabs and ConfigPanel row also overflow.

## Approach

CSS-only, additive changes in `playground/src/App.css`. No React
restructuring, no mobile-only components, no new breakpoints in shared
theme files.

Two breakpoints:

- **`max-width: 1200px`** — relax grid to `1fr 1fr 1fr`. Removes the
  1.2fr bias that over-squeezes the middle (IR) column on laptops and
  small desktop windows.
- **`max-width: 900px`** — collapse to a single column. Stack the three
  panels vertically. Each panel gets `min-height: 360px` so the Monaco
  editor and graph canvas stay usable. Switch `.app` from fixed
  `height: calc(100vh - 48px)` to `min-height`, and allow `.panels` to
  scroll vertically instead of clipping.

Also at `max-width: 900px`:

- `.header-right` gets `flex-wrap: wrap` so the tabs + Settings +
  Analyze button don't overflow horizontally.
- `.config-bar` gets `flex-wrap: wrap` with a slightly tighter gap so
  MEM2REG / VF MODE / PTA SOLVER / MAX ITERATIONS stack onto multiple
  rows instead of overflowing.

## Out of scope

- AnalyzerPanel / QueryPanel / TutorialPanel internal CSS. Revisit only
  if visible testing shows breakage.
- Embed mode (`?embed=true`). `.app.embed .panels` already uses single
  or 2-column layouts and is not affected.
- Any JS / component-tree changes.

## Verification

- Run `pnpm dev` in `playground/`.
- Resize through ~400px (screenshot width), ~700px, ~900px, ~1100px,
  and ≥1280px. At each width, confirm:
  - Panels are laid out sensibly (stacked or gridded).
  - Editor, IR viewer, and graph are readable and interactive.
  - Header and ConfigPanel controls remain visible without horizontal
    overflow.
- Confirm desktop (≥1280px) is visually identical to current behavior.
- No unit / e2e tests required (CSS-only, no behavior change above
  1200px).
