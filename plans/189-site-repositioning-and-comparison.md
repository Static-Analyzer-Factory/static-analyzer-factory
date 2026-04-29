# Plan 189 — Site Repositioning and Tool Comparison

## Status
**approved** (2026-04-27)

## Branch
`site/repositioning-and-comparison`

## Goal

Reposition SAF's marketing surfaces (homepage, README, meta tags) away from a
"browser-based static analyzer" framing toward an **extensible program-analysis
framework** framing for research and production use. Add a credible
tool-vs-tool comparison (SAF vs SVF, Lotus, Phasar, CodeQL, Infer) to the
README and a dedicated `/comparison/` page on the website, with a
homepage teaser linking to it.

The browser playground stays — it's a real feature — but it is no longer the
headline.

## Decisions captured during brainstorming (2026-04-27)

| # | Decision | Choice |
|---|----------|--------|
| Q1 | Headline identity | **Research framework first**, with explicit production-framework framing — not just bug-finding, but a programmable platform for building analyses. |
| Q2 | Comparison set | SVF, Lotus, Phasar, CodeQL, Infer (option C). |
| Q3 | Comparison placement | Homepage teaser + dedicated `/comparison/` page (option C). |
| Q4 | Comparison dimensions | Hybrid: capability rows for credibility + differentiation rows for positioning (option C). Each peer also gets a "best at" prose blurb. |
| Q5 | Homepage scope | Surgical + reframe sections (option B): edit the existing Hero, Features, Personas, TechHighlights for coherence with the new positioning. No new top-level sections beyond the comparison teaser. |
| Q6 | SEO targets | All three clusters (option D): framework-builder queries (homepage), tool-comparison queries (`/comparison/`), capability queries (sprinkled across feature/concept pages and docs). |

## Comparison dimensions (final)

**Capability rows (credibility):**
1. Primary IR / target language(s)
2. Pointer-analysis variants (CI / FS / CS / DDA)
3. Value-flow / SVFG support
4. IFDS / IDE solver
5. Taint analysis
6. Built-in checker library size
7. SARIF export
8. License

**Differentiation rows (positioning):**
9. Primary SDK / authoring language
10. Multi-LLVM version support
11. Browser / WASM playground
12. Byte-deterministic output guarantee
13. AI-agent skills (coding-agent integration)

Total: 13 rows. Homepage teaser shows 5 most-differentiating rows (rows 9-13);
`/comparison/` page shows all 13 + per-tool prose blurbs.

## Files

### To modify

| File | Change |
|---|---|
| `README.md` | Add "Comparison with Other Tools" section (markdown table + per-tool prose). |
| `site/index.html` | New `<title>`, meta description, OG, Twitter, JSON-LD descriptions. Drop "browser-based" from the lead. |
| `site/public/sitemap.xml` | Add `/comparison/` URL (hash route → site root for crawling). |
| `site/src/App.tsx` | Add hash route `#comparison` → `<ComparisonPage />`. |
| `site/src/components/Hero.tsx` | New subtitle / sub-line. |
| `site/src/components/Features.tsx` | Reword Features to lead with framework framing. |
| `site/src/components/Personas.tsx` | Foreground researcher / framework-builder. |
| `site/src/components/TechHighlights.tsx` | Lead with multi-LLVM + extensibility. |
| `packages/shared/src/ui/SiteNav.tsx` | Add `comparison` to active-union and Comparison nav link. |

### To add

| File | Purpose |
|---|---|
| `site/src/data/comparison.ts` | Comparison table data + per-tool blurbs. |
| `site/src/components/Comparison.tsx` | Homepage teaser (5 rows + CTA). |
| `site/src/components/ComparisonPage.tsx` | Full `/comparison/` page (table + prose + caveats). |
| `plans/189-research-notes.md` | Research notes for peer-tool capability claims. Sources, repo links, and verification status per dimension. |

## Phases

### Phase 1 — Research peer tools

For each of **SVF, Lotus, Phasar, CodeQL, Infer**, verify each of the 13
dimensions. Use:
- `mcp__plugin_context7_context7__query-docs` for SVF, Phasar, CodeQL, Infer.
- WebSearch + WebFetch + GitHub repo inspection for Lotus (less documented).

Cite a source for every non-obvious claim in `plans/189-research-notes.md`.
Wherever a peer tool ambiguously supports a feature, note it as "partial" with
a sentence explaining what it does and what it doesn't do — never claim
"unsupported" without verification.

### Phase 2 — Comparison data + README section

- Build `site/src/data/comparison.ts` from the verified data.
- Add the README section: short intro paragraph, the markdown table, then
  per-tool blurbs (1-2 sentences each, fair).

### Phase 3 — Comparison page on website

- New `ComparisonPage.tsx`: full table + per-tool prose + a "Methodology and
  caveats" subsection (links to research notes + GitHub sources).
- `App.tsx`: hash route `#comparison`. JS-side `document.title` set on route
  change.
- `SiteNav.tsx`: add `comparison` to the active union, add the link.
- `sitemap.xml`: add `/#comparison`.

### Phase 4 — Homepage repositioning

- `Hero.tsx`: new subtitle ("Build program analyses. For research and
  production.") and new sub-line ("An extensible static analysis framework
  with a Python SDK, multi-LLVM support, and a Rust core.").
- `Features.tsx`: rewrite the three card descriptions to lead with framework
  framing rather than browser framing. The "Build" card already mentions the
  Python SDK — strengthen it.
- `Personas.tsx`: replace the "Students & Learners" card with "Framework
  Researchers" or similar (researcher persona); keep the security-research and
  AI-agent personas (those still fit). Reorder: researcher first.
- `TechHighlights.tsx`: replace "Browser-native" with "Multi-LLVM" (or "LLVM
  18 + 22"); keep Rust + WASM (it's still true), Deterministic, Open Source.

Add a `<Comparison />` teaser section between `TechHighlights` and `Footer`.

### Phase 5 — SEO + structured data

- `<title>`: "SAF — Static Analyzer Factory | Program Analysis Framework for
  LLVM IR".
- Meta description: research-framework framing.
- Meta keywords: keep "program analysis", "static analysis", add "framework",
  "research", "Python SDK", "multi-LLVM"; demote (don't remove) "WebAssembly".
- OG / Twitter descriptions: aligned with the new framing.
- JSON-LD `SoftwareApplication.description`: aligned.

### Phase 6 — Build verification

- Run `cd site && npm run build` to verify TypeScript compiles.
- If a dev server is feasible, open in a browser to confirm the route + nav
  link work.

### Phase 7 — Commits + PR

- One commit per phase.
- Open PR titled `site: research-framework repositioning + tool comparison
  (plan 189)`.

## Out of scope

- Re-running benchmark numbers — README already has Juliet results from a
  prior plan; no new benchmark runs.
- Changes to `playground/`, `tutorials/`, or `docs/book/` content (those have
  their own positioning surfaces; only the homepage and README change here).
- New content for the comparison page beyond the table + per-tool blurbs +
  caveats — no separate "deep dive" pages per peer.

## Risks and notes

- **Peer-tool claims must be verifiable.** This is the most fragile part of
  the plan. The research notes file is the authoritative source; the
  comparison data file references it. Any claim a peer-tool maintainer might
  reasonably push back on must have a citation.
- **`/comparison/` is a hash route, not a real path** (the site is a
  Vite-bundled SPA on GitHub Pages). Search engines crawl the hash-fragment
  URL but treat it as the same page; we set `document.title` on route change
  so the browser tab and OpenGraph crawler see different titles.
- **Personas — keep the file gentle**. Researchers, security researchers, and
  AI-agent developers all overlap; the goal is to *foreground* the researcher
  persona, not exclude others.
