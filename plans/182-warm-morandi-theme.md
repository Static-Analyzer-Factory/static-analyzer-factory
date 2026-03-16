# Plan 182: Warm Morandi Theme Redesign

## Goal

Replace the current dark navy theme across all four web apps (landing page, tutorials, playground, docs) with a cohesive warm Morandi light theme. Restructure CSS so the entire palette is defined in one shared file and can be changed by editing a single source of truth.

## Design Decisions

- **Aesthetic**: Warm Morandi — creamy whites, dusty rose/clay accents, warm taupe surfaces, sage green secondary
- **Code editors**: Soft contrast — parchment/paper background, slightly darker than the page, with Morandi-toned syntax highlighting
- **Primary accent**: Warm clay (`#c4856a`)
- **Palette structure**: Single shared CSS variables file in `packages/shared/`; no JS theme objects, no Tailwind
- **Surface hierarchy**: 5 background levels (bg, surface, surface-raised, surface-editor, border)

## Color Palette

### Backgrounds & Surfaces
| Token | Hex | Usage |
|-------|-----|-------|
| `--color-bg` | `#f5f0eb` | Page background — warm cream |
| `--color-surface` | `#ece5de` | Cards, panels, sidebar |
| `--color-surface-raised` | `#e2d9d0` | Panel headers, navbar, elevated |
| `--color-surface-editor` | `#eae4dc` | Code editor background (parchment) |
| `--color-border` | `#d4cac0` | Dividers, panel borders |
| `--color-border-subtle` | `#e0d7ce` | Lighter borders, separators |

### Text
| Token | Hex | Usage |
|-------|-----|-------|
| `--color-text` | `#3d3532` | Primary text — warm dark brown |
| `--color-text-secondary` | `#7a6f66` | Secondary/muted text |
| `--color-text-tertiary` | `#a39890` | Placeholder, disabled text |
| `--color-text-inverse` | `#f5f0eb` | Text on dark/accent backgrounds |

### Accents
| Token | Hex | Usage |
|-------|-----|-------|
| `--color-accent` | `#c4856a` | Primary — buttons, links, active states |
| `--color-accent-hover` | `#b3745a` | Hover state |
| `--color-accent-subtle` | `rgba(196,133,106,0.12)` | Accent tinted backgrounds |
| `--color-secondary` | `#7d9a7e` | Secondary — success, tags, secondary actions |
| `--color-secondary-hover` | `#6d8a6e` | Hover state |
| `--color-secondary-subtle` | `rgba(125,154,126,0.12)` | Secondary tinted backgrounds |

### Semantic
| Token | Hex | Usage |
|-------|-----|-------|
| `--color-error` | `#c27171` | Errors — muted red |
| `--color-error-subtle` | `rgba(194,113,113,0.12)` | Error backgrounds |
| `--color-warning` | `#c9a56a` | Warnings — muted amber |
| `--color-warning-subtle` | `rgba(201,165,106,0.12)` | Warning backgrounds |
| `--color-info` | `#7a9ab5` | Info — muted blue |
| `--color-info-subtle` | `rgba(122,154,181,0.12)` | Info backgrounds |
| `--color-success` | `#7d9a7e` | Same as secondary |

### Scrollbar
| Token | Hex | Usage |
|-------|-----|-------|
| `--color-scrollbar-track` | `#e8e1d9` | Scrollbar track |
| `--color-scrollbar-thumb` | `#cdc4ba` | Scrollbar thumb |
| `--color-scrollbar-thumb-hover` | `#b8aea3` | Scrollbar thumb hover |

## CodeMirror Syntax Theme

Custom CodeMirror 6 theme (`codemirror-morandi.ts`) replacing `@codemirror/theme-one-dark`.

### Editor Chrome
| Element | Color |
|---------|-------|
| Background | `var(--color-surface-editor)` / `#eae4dc` |
| Gutter bg | `var(--color-surface)` / `#ece5de` |
| Gutter text | `var(--color-text-tertiary)` / `#a39890` |
| Cursor | `var(--color-text)` / `#3d3532` |
| Selection | `rgba(196,133,106,0.18)` |
| Active line | `rgba(196,133,106,0.08)` |
| Matching bracket | `rgba(125,154,126,0.3)` |

### Syntax Colors
| Token Type | Color | Hex |
|-----------|-------|-----|
| Keywords | Warm clay | `#b5725a` |
| Strings | Sage green | `#6b8a5e` |
| Numbers | Muted teal | `#5d8a8a` |
| Comments | Warm gray | `#a09488` |
| Functions | Dusty plum | `#956b7d` |
| Types/Classes | Muted blue | `#6b819a` |
| Variables | Primary text | `#3d3532` |
| Operators | Secondary text | `#7a6f66` |
| Properties | Warm brown | `#8a6d52` |

## Implementation

### New Files
1. `packages/shared/src/ui/theme.css` — All CSS variable definitions
2. `packages/shared/src/ui/codemirror-morandi.ts` — Custom CodeMirror 6 theme

### Files to Modify

**Shared (`packages/shared/`)**
- `src/ui/global-nav.css` — Replace hard-coded dark colors with `var()` tokens

**Landing Page (`site/`)**
- `src/index.css` — Import `theme.css`, replace hard-coded colors
- `src/App.css` — Replace all hex colors with `var()` tokens
- `src/components/ArchitectureDiagram/ArchitectureDiagram.css` — Replace architecture tag colors with Morandi variants

**Tutorials (`tutorials/`)**
- `src/App.css` — Replace existing `:root` vars with import of shared `theme.css`
- `src/components/NavBar.css` — Update to `var()` tokens
- `src/components/TutorialPage.css` — Update to `var()` tokens
- `src/components/Sidebar.css` — Update to `var()` tokens
- `src/components/Catalog.css` — Update to `var()` tokens
- `src/components/AlgorithmStepper.css` — Update to `var()` tokens
- `src/components/PseudocodeRail.css` — Update to `var()` tokens
- `src/components/PhaseBar.css` — Update to `var()` tokens
- `src/components/InteractiveStep.css` — Update to `var()` tokens
- `src/components/StepperControls.css` — Update to `var()` tokens
- `src/components/CodeBlock.tsx` — Swap `oneDark` for `codemirrorMorandi`

**Playground (`playground/`)**
- `src/index.css` — Import `theme.css`, replace hard-coded colors
- `src/App.css` — Replace all hex colors with `var()` tokens
- `src/components/AnalyzerPanel.css` — Update to `var()` tokens
- `src/components/QueryPanel.css` — Update to `var()` tokens
- `src/components/TutorialPanel.css` — Update to `var()` tokens
- `src/components/SourcePanel.tsx` — Swap `oneDark` for `codemirrorMorandi`
- `src/components/AnalyzerPanel.tsx` — Swap `oneDark` for `codemirrorMorandi`
- `src/components/QueryPanel.tsx` — Swap `oneDark` for `codemirrorMorandi`

**Docs (`docs/book/`)**
- `src/custom.css` — Replace hard-coded colors with `var()` tokens
- `src/global-nav.css` — Replace hard-coded colors with `var()` tokens
- `book.toml` — Add `theme.css` to `additional-css` list

### Implementation Order
1. Create `theme.css` with all CSS variable definitions
2. Create `codemirror-morandi.ts` with custom CM6 theme
3. Update shared global nav
4. Update each app (site, tutorials, playground, docs) — can be parallelized
5. Visual verification across all apps

### Architecture Tag Colors (Landing Page)

The architecture diagram has ~10 colored tags. Replace with Morandi variants:
| Tag | Current | New |
|-----|---------|-----|
| PyO3 | Bright sky blue | Muted blue `rgba(107,129,154,0.15)` / `#6b819a` |
| API | Bright teal | Sage green `rgba(125,154,126,0.15)` / `#7d9a7e` |
| Binary | Bright red | Muted rose `rgba(194,113,113,0.15)` / `#c27171` |
| React | Bright indigo | Dusty plum `rgba(149,107,125,0.15)` / `#956b7d` |
| WASM | Bright amber | Muted amber `rgba(201,165,106,0.15)` / `#c9a56a` |
| Browser | Bright violet | Warm clay `rgba(196,133,106,0.15)` / `#c4856a` |

### Severity Colors (Playground)
| Level | New Color |
|-------|-----------|
| Critical/High | `#c27171` (muted red) |
| Medium | `#c9a56a` (muted amber) |
| Low/Info | `#7a9ab5` (muted blue) |

## Changing the Theme Later

To re-theme the entire site, edit only `packages/shared/src/ui/theme.css`. All 25+ CSS files reference these variables. The CodeMirror theme in `codemirror-morandi.ts` reads from CSS variables where possible, so most editor chrome colors update automatically too — only the syntax highlighting colors need manual adjustment in that file.
