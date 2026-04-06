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

## Next Steps
- **Plan 139 Phase 3:** 12 analysis consumers for the AIR type system (PTA constraint filtering, field-sensitive byte-offset locations, type-based CG pruning, etc.). Each is 30-80 lines, independent of each other.
- **Plan 157 — Datalog integration (blocked on Tasks 11-14):** Checker migration blocked by circular dependency. See plan notes for resolution options.
- **Plan 182 — Warm Morandi theme:** Redesign all web apps to warm Morandi light theme.
- **Scalability targets:** With Load at 3s and Ander at ~10s, the solver is the dominant phase. Remaining hot spots: handle_load 4.41s (75.6%), process_location 3.69s. Opportunities: algorithmic changes, batch load processing, semi-sparse analysis.
- **Future directions** (see `plans/FUTURE.md`): Tier 1: compositional/incremental. Tier 2: numeric domains. Tier 3: race detection. Tier 4: SILVA/clustering.

## Known Issues
- `taint_sanitized.ll` — SIGSEGV in LLVM frontend (tests ignored)
- `format_string.ll` — requires PTA alias analysis (test downgraded)
- `taint_unsafe.ll` — mangled Rust names (test downgraded)
- Rust `-C debuginfo=2` generates `#dbg_declare` intrinsics SAF can't handle (use `-C debuginfo=0`)

## Key Decisions
| Category | Decision |
|----------|----------|
| Build | Docker-only (LLVM 18 inside container) |
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
