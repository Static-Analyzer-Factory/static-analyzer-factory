# Plan 119: Consistent Global Navigation Bar & Tutorials Layout Fix

## Context

The four frontend apps (site, tutorials, playground, docs) have inconsistent navigation:
- **Site** (`/`): Custom `SiteNav` in `site/src/App.tsx` — fixed nav with links
- **Tutorials** (`/tutorials/`): Similar but visually different `NavBar` in `tutorials/src/components/NavBar.tsx`
- **Playground** (`/playground/`): No global nav at all — users get "stuck" with no way to navigate back
- **Docs** (`/docs/`): mdBook's built-in header, no cross-app links

Additionally, the tutorials catalog page has a layout bug: the sidebar is always rendered (260px fixed), but when showing the catalog (`!tutorialId`), the `no-sidebar` class on `.app-main` sets `margin-left: 0` — causing the catalog content to underlap the fixed sidebar. The catalog also constrains itself with `max-width: 1100px` instead of filling the available space.

## Changes

### 1. Create shared GlobalNav CSS

**File: `packages/shared/src/ui/global-nav.css`** (new)

Single CSS file imported by all three React apps. Ensures pixel-perfect consistency without style duplication.

Style spec (matching current site nav, which is the cleaner of the two):
- Fixed top, 48px height, z-index 200
- Background: `rgba(10, 10, 26, 0.95)` with `backdrop-filter: blur(8px)`
- Border-bottom: `1px solid rgba(255, 255, 255, 0.06)`
- Brand: 700 weight, 1.1rem, `#10b981`, left-aligned
- Links: `#a0a0b0`, 0.875rem, 500 weight, right-aligned, 1.5rem gap
- Active link: `#10b981`
- Hover: `#e0e0e0`
- Responsive: links gap shrinks at 768px, hide non-active at 480px

### 2. Update shared package exports

**File: `packages/shared/package.json`**

Add: `"./ui/global-nav.css": "./src/ui/global-nav.css"`

### 3. Refactor Site navbar

**File: `site/src/App.tsx`**
- Import `@saf/web-shared/ui/global-nav.css`
- Change `SiteNav` class names to `global-nav`, `global-nav-brand`, `global-nav-links`, `global-nav-active`

**File: `site/src/App.css`**
- Remove the `.site-nav*` rules (lines 1-38), replaced by shared CSS

### 4. Refactor Tutorials navbar

**File: `tutorials/src/components/NavBar.tsx`**
- Import `@saf/web-shared/ui/global-nav.css`
- Change class names to `global-nav`, `global-nav-brand`, `global-nav-links`, `global-nav-active`
- Keep the hamburger button for mobile sidebar toggle

**File: `tutorials/src/components/NavBar.css`**
- Remove `.navbar*` rules, keep only `.hamburger` mobile rules

### 5. Add GlobalNav to Playground

**File: `playground/src/App.tsx`**
- Import `@saf/web-shared/ui/global-nav.css`
- Add `<nav className="global-nav">...</nav>` above `<header>` in normal mode only (NOT in embed mode)
- Links: Home, Tutorials, Playground (active), Docs, GitHub

**File: `playground/src/App.css`**
- Change `.app` from `height: 100vh` to `height: calc(100vh - 48px); margin-top: 48px;` to account for fixed global nav
- Add `.app.embed` override to restore `height: 100vh; margin-top: 0;` for embed mode

### 6. Add GlobalNav to Docs (mdBook)

mdBook doesn't support React, so inject via `additional-css` + `additional-js`:

**File: `docs/book/src/global-nav.css`** (new)
- Copy of shared CSS rules for the nav bar
- Plus: push mdBook content down with `.page-wrapper { margin-top: 48px; }` and `.sidebar { top: 48px; }`

**File: `docs/book/src/global-nav.js`** (new)
- Small JS that creates the nav DOM and prepends to `<body>`
- Uses mdBook's `path_to_root` global variable for relative path calculation
- Highlights "Docs" as active

**File: `docs/book/book.toml`**
- Add to `[output.html]`: `additional-css = ["src/custom.css", "src/global-nav.css"]` and `additional-js = ["src/global-nav.js"]`

### 7. Fix Tutorials catalog layout

**File: `tutorials/src/App.tsx`** (line 61)
- Remove the `no-sidebar` conditional — the sidebar is always visible, so always keep `margin-left: 260px`
- Change: `className="app-main"` (no conditional)

**File: `tutorials/src/App.css`** (lines 44-46)
- Remove `.app-main.no-sidebar { margin-left: 0; }` rule

**File: `tutorials/src/components/Catalog.css`** (line 2)
- Remove `max-width: 1100px` from `.catalog` — let it fill the available width
- Keep padding for comfortable reading

## Files Modified (Summary)

| File | Action |
|------|--------|
| `packages/shared/src/ui/global-nav.css` | Create |
| `packages/shared/package.json` | Edit |
| `site/src/App.tsx` | Edit |
| `site/src/App.css` | Edit |
| `tutorials/src/components/NavBar.tsx` | Edit |
| `tutorials/src/components/NavBar.css` | Edit |
| `tutorials/src/App.tsx` | Edit |
| `tutorials/src/App.css` | Edit |
| `tutorials/src/components/Catalog.css` | Edit |
| `playground/src/App.tsx` | Edit |
| `playground/src/App.css` | Edit |
| `docs/book/src/global-nav.css` | Create |
| `docs/book/src/global-nav.js` | Create |
| `docs/book/book.toml` | Edit |
