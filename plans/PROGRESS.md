# SAF Development Progress

## Current Status
**v0.1.0 — Initial Open-Source Release** (2026-04-07)

## Plans Index
| # | Plan | Area | Status |
|---|------|------|--------|
| 139 | air-type-system | architecture | in-progress | Notes: Phase 1-3a complete. AirType enum, TypeId, type table, LLVM TypeInterner, AIR-JSON type support. Remaining: 9 lower-priority consumers (field-sensitive byte-offsets, value-flow typing, taint selectors, typestate, SARIF, Python SDK, absint, DDA, benchmark). |
| 157 | datalog-integration | architecture | in-progress | Notes: PTA solver + benchmarks complete (15/19 tasks). Checker migration (Tasks 11-14) blocked by circular dependency. Options: (A) extract shared types to saf-core, (B) inline Ascent checker rules in saf-analysis, (C) shared interface crate. Worklist is default solver; Datalog available via `--solver datalog`. |
| 182 | warm-morandi-theme | frontend | approved | Notes: Redesign all web apps from dark navy to warm Morandi light theme. Single shared CSS variables file. Custom CodeMirror parchment theme. ~25 files to modify. |
| 184 | saf-checker-dev-skill | tooling | approved | Notes: Coding-agent skill for SAF checker/analyzer authoring. Spec-first workflow with 3 tiers (declarative, typestate, custom patterns). |
| 186 | llvm-22-support | build | done | Notes: Multi-version LLVM support (18 + 22) shipped. Parameterized Dockerfile + feature-flag passthrough. Two image tags `saf-dev:llvm18` (default/alias) and `saf-dev:llvm22` (opt-in). inkwell 0.8→0.9, Rust 1.85→1.88. Phase 3 IR fixes: null-termination for inkwell 0.9 + strip `inrange i32` from 4 vtable fixtures. Both image streams green: 2188 Rust + 94 pytest per tag. Follow-up: re-enable CI (ci.yml.disabled → ci.yml) once ready. |
| 187 | ui-github-cta-quickstart | frontend | done | Notes: Landing page — added "View on GitHub" CTA in Hero and a QuickStart section below Hero with copy-to-clipboard Docker install block. Mobile overflow fixes (grid `min-width: 0`, scrollable command text). Branch `ui/github-cta-and-quickstart`. |
| 188 | playground-responsive-layout | frontend | done | Notes: (1) Added two CSS breakpoints in `playground/src/App.css`. `max-width: 1200px` relaxes the workspace grid from `1fr 1fr 1.2fr` to `1fr 1fr 1fr`. `max-width: 900px` stacks panels vertically with `min-height: 360px` per panel, switches `.app` to `min-height` + visible overflow so the page scrolls, wraps `.header` / `.header-right` / `.config-bar`, stacks embed-split mode to a single column, and shrinks `.spec-sidebar` from 180px to 140px. (2) Removed `body { min-width: 960px; overflow: hidden }` and `#root { width: 100vw; height: 100vh }` from `playground/src/index.css` — legacy constraints that kept the body at 960px regardless of viewport. Replaced with `body { min-height: 100vh; overflow-x: hidden }` and `#root { width: 100%; min-height: 100vh }`. (3) Fixed `.analyzer-toolbar` horizontal overflow by adding `flex-wrap: wrap`, letting its `select` flex/shrink, and constraining `.help-popover` to `max-width: 100%; overflow-x: auto`. (4) Added a `useEffect` in `App.tsx` that smooth-scrolls the right-most panel into view when the header tab changes, but only when the viewport is ≤900px (stacked layout) — a first-render guard prevents auto-scroll on initial mount. Previously tab changes looked like no-ops on mobile because the affected panel was below the fold. Verified via Playwright at 400/600/800/900/1100/1400px for each right-panel tab (Analysis, Specs, Analyzer, Query) plus embed-split mode, and confirmed desktop (1400px) does not auto-scroll. Branch `ui/playground-responsive-layout`. |
| 189 | site-repositioning-and-comparison | frontend | approved | Notes: Reposition homepage + README + meta tags away from "browser-based static analyzer" toward an extensible program-analysis framework for research and production. Add a 13-row tool comparison (SAF vs SVF, Lotus, Phasar, CodeQL, Infer) to README and a new `/comparison/` page on the website, with a 5-row teaser on the homepage. SEO targets all three query clusters (framework-builder, tool-comparison, capability). Branch `site/repositioning-and-comparison`. |

## Next Steps
- **Plan 189 — Site repositioning + comparison:** in flight. Branch `site/repositioning-and-comparison`. Phase 1 (peer-tool research) → Phase 2 (README + data) → Phase 3 (/comparison/ page) → Phase 4 (homepage reframe) → Phase 5 (SEO) → Phase 6 (build) → Phase 7 (PR).
- **Plan 139 Phase 3:** 12 analysis consumers for the AIR type system (PTA constraint filtering, field-sensitive byte-offset locations, type-based CG pruning, etc.). Each is 30-80 lines, independent of each other.
- **Plan 157 — Datalog integration (blocked on Tasks 11-14):** Checker migration blocked by circular dependency. See plan notes for resolution options.
- **Plan 182 — Warm Morandi theme:** Redesign all web apps to warm Morandi light theme.
- ~~**Plan 186 — LLVM 22 support:**~~ Done. Follow-ups from PR quality review (2026-04-19):
  - Re-enable CI (`.github/workflows/ci.yml.disabled` → `ci.yml`) when ready to run GitHub Actions.
  - Publish `saf:llvm18` / `saf:llvm22` images to GHCR (release job is scaffolded but commented out).
  - **DbgRecord handling (LLVM 19+ migration).** `crates/saf-frontends/src/llvm/debug_info.rs` extracts local variable names from `llvm.dbg.declare` / `#dbg_declare` by scanning IR text. LLVM 19+ emits `DbgRecord` objects attached to instructions; the text scan misses them. Audit inkwell 0.9's `Instruction::get_debug_records()` and add a version-gated branch — materially impacts users analyzing clang-19+ IR with `-g`.
  - **Parse-error wrapping.** When LLVM 18 rejects clang-22 IR the frontend returns "expected ')' at end of argument list" with no hint that it's a version mismatch. Wrap frontend parse failures with a hint referencing the bound `saf_frontends::LLVM_VERSION`.
  - **Richer intrinsic classification.** `intrinsics.rs` routes new LLVM 19–22 intrinsics (`llvm.ptrauth.*`, `llvm.vector.reverse`, `llvm.stepvector`, LLVM 22's reshape `@llvm.masked.*`) to `IntrinsicOp::External`. Safe but loses modeling opportunities.
  - **Clippy 1.88+ workspace allows.** `.cargo/config.toml` silences `format_push_string`, `doc_lazy_continuation`, `needless_continue` across the whole tree to avoid a large churn; schedule per-site cleanup.
  - **Volume prune note.** `docker-compose.yml` renamed `cargo-registry` → `cargo-registry-llvm18` etc.; developers switching to this branch should `docker volume prune` to reclaim orphaned volumes.
  - **`cha_extract.rs` name-parsing robustness.** Align `parse_function_pointers_from_ir_string` with `extract_at_name` (quoted `@"..."` globals); add `tracing::warn!` in the Itanium demangler on failure.
- **Scalability targets:** With Load at 3s and Ander at ~10s, the solver is the dominant phase. Remaining hot spots: handle_load 4.41s (75.6%), process_location 3.69s. Opportunities: algorithmic changes, batch load processing, semi-sparse analysis.
- **Future directions** (see `plans/FUTURE.md`): Tier 1: compositional/incremental. Tier 2: numeric domains. Tier 3: race detection. Tier 4: SILVA/clustering.

## Known Issues
- `taint_sanitized.ll` — SIGSEGV in LLVM frontend (tests ignored)
- `format_string.ll` — requires PTA alias analysis (test downgraded)
- `taint_unsafe.ll` — mangled Rust names (test downgraded)
- Rust `-C debuginfo=2` generates `#dbg_declare` intrinsics SAF can't handle (use `-C debuginfo=0`)
- LLVM 19+ `DbgRecord` local-variable-name format not read by `debug_info.rs` (only scans IR text). See plan 186 follow-ups.

## Key Decisions
| Category | Decision |
|----------|----------|
| Build | Docker-only; dual-LLVM tags (`saf-dev:llvm18` default, `saf-dev:llvm22` opt-in) — plan 186 |
| Determinism | IndexMap (PTA hot paths) + BTreeMap (public API, non-hot), BLAKE3 IDs |
| AIR | Flat enum for instructions, newtypes for IDs |
| Graphs | Block-level CFG, instruction-level ICFG call/return |
| PTA | Configurable field sensitivity, five-valued AliasResult, array index sensitivity |
| IFDS/IDE | Zero fact in trait, instruction-level within blocks |
| Memory | Hybrid MSSA (skeleton + demand-driven clobber) |
| Checkers | Declarative specs, may_reach/must_not_reach/multi_reach/never_reach_sink modes |
| AI | AbstractDomain trait, wrapped intervals, threshold widening |
| CS-PTA | k-CFA, SCC collapse, top-down context seeding |
| Z3 | Bundled crate, guard extraction, two-stage SMOKE |
| DDA | CFL-reachability, budget + CI fallback, persistent cache |
| PtsSet | Roaring default (≥10K alloc sites), FxHash (<10K), frozen indexer for lock-free solving |

## Session Log
| Date | Area | Summary |
|------|------|---------|
| 2026-04-07 | release | Initial open-source release (v0.1.0). |
| 2026-04-18 | build | Plan 186 approved. Design for LLVM 22 support via multi-version Docker tags (saf:llvm18 default, saf:llvm22 opt-in) using parameterized Dockerfile + feature passthrough. inkwell 0.9 (released 2026-04-12) unblocks llvm22-1 binding. |
| 2026-04-18 | build | Plan 186 shipped. Parameterized Dockerfile (`ARG LLVM_VERSION`), two image tags (`saf-dev:llvm18` default + `saf-dev:llvm22` opt-in). inkwell 0.9 null-termination fix in adapter. 4 vtable fixtures scrubbed of `inrange` (post-LLVM 18 syntax). Rust MSRV 1.85→1.88 (inkwell let-chains). Bulk-allowed 3 new-1.88 clippy lints at workspace level (format_push_string, doc_lazy_continuation, needless_continue). `make shell-llvm22` / `make test-llvm22` / `make build-llvm22` convenience targets. Full test suite green on both LLVM versions: 2188 Rust + 94 pytest. |
| 2026-04-19 | review | Plan 186 PR quality review + follow-up fixes. Discovered + fixed a second site of the single-index constant-GEP fallback bug (`resolve_constant_gep_element` in `mapping.rs`) in addition to the one already fixed in `decompose_constant_gep`. Both now bail on single-index GEPs since SAF's FieldPath can't represent pointer-level offsets. Added regression test (`saf-analysis::constant_gep_e2e`). Docs: surfaced LLVM 22 path in `installation.md`, noted unpublished image tags in `llvm-versions.md`, clarified benchmark LLVM-18 wording in CLAUDE.md. |
| 2026-04-19 | frontend | Plan 187 implemented on branch `ui/github-cta-and-quickstart`: Hero GitHub CTA, QuickStart install terminal card below Hero, mobile horizontal-overflow fixes (`min-width:0` on grid cards, flex-shrinkable command scroller). Merged origin/main — took main's richer SEO meta + shipped `saf-logo.png` favicon. |
| 2026-04-20 | frontend | Plan 188 implemented on branch `ui/playground-responsive-layout`: fixed the playground's fixed 3-column grid collapsing to unusable slivers on narrow viewports. Two CSS media queries in `playground/src/App.css` — 1200px relaxes the grid bias, 900px stacks vertically with scrollable page and per-panel `min-height`. Verified via Playwright at 400/700/900/1100/1400px. |
| 2026-04-27 | frontend | Plan 189 approved. Repositioning homepage + README + meta tags from "browser-based static analyzer" to "extensible program-analysis framework, research + production." Adds a 13-row tool comparison (SAF vs SVF, Lotus, Phasar, CodeQL, Infer) to README and a `/comparison/` page on the website, with a 5-row teaser on the homepage. SEO targets framework-builder, tool-comparison, and capability query clusters. Branch `site/repositioning-and-comparison`. |
