# Plan 105: SAF Scalability Research Synthesis

**Epic:** Scalability
**Status:** Research complete
**Date:** 2026-02-11

## Context

SAF's current bottleneck is fspta/SVFG scalability:
- bunzip2: 27s total (21.8s in fspta alone), PTA phase only 0.14s after Plan 103
- bash, libcurl: timeout at >10 minutes
- SAF is 32-112x slower than SVF on CruxBC benchmarks

Four parallel research agents investigated:
1. Unused/unwired fast algorithms already in SAF
2. Novel graph algorithms from recent papers (2020-2026)
3. Fast data structures (sparse bit vectors, compressed bitmaps, etc.)
4. SVF design patterns not yet adopted by SAF

---

## Part 1: Unused/Unwired Fast Algorithms Already in SAF

### 1A. Steensgaard PTA — O(n·α(n)) unification-based (EXPERIMENTAL)
- **File:** `crates/saf-analysis/src/pta/steensgaard.rs` (~640 lines)
- **Status:** Fully implemented, behind `experimental` feature flag, never called from production code
- **Opportunity:** Use as fast pre-analysis for CG bootstrapping or to seed object clustering (see 4F below)

### 1B. Incremental PTA — SILVA-style delta propagation (EXPERIMENTAL)
- **File:** `crates/saf-analysis/src/pta/incremental.rs` (~750 lines)
- **Status:** Fully implemented, behind `experimental` feature flag, never called
- **Opportunity:** Wire for IDE/CI incremental re-analysis workflows

### 1C. CG Refinement Ignores User's PtsConfig (BUG)
- **File:** `crates/saf-analysis/src/cg_refinement.rs:136`
- **Status:** Hardcodes `PtsConfig::default()` instead of using `config.pta_config.pts_config`
- **Impact:** BitVec/BDD representations never selected for large programs in CG refinement loop
- **Fix:** One-line change: `&PtsConfig::default()` → `&config.pta_config.pts_config`

### 1D. FSPTA Solver Hardcodes BTreeMap/BTreeSet (DESIGN GAP)
- **File:** `crates/saf-analysis/src/fspta/solver.rs`
- **Status:** `FsPtaConfig.pts_config` plumbing exists but solver hardcodes `BTreeMap<ValueId, BTreeSet<LocId>>`
- **Impact:** Flow-sensitive PTA can't benefit from BitVec/BDD representations

### 1E. Object Clustering Rarely Fires
- **File:** `crates/saf-analysis/src/pta/clustering.rs` (~750 lines)
- **Status:** Wired into CI-PTA but `BTreePtsSet::BENEFITS_FROM_CLUSTERING = false`, and BTreeSet is almost always selected (due to 1C, 1D)
- **Impact:** Clustering infrastructure goes unused in practice

---

## Part 2: Novel Graph Algorithms from Recent Papers

### Tier 1 — Directly addresses fspta/SVFG bottleneck

| Algorithm | Paper | Key Idea | Expected Speedup | Effort |
|-----------|-------|----------|-----------------|--------|
| **CG-FSPTA** | arXiv 2508.01974 (2025) | Flow-sensitive constraint graph replaces CFG-based fspta; enables cycle elimination + constraint folding | 7.27x over SOTA fspta, −33% memory | Medium |
| **SILVA** | TOSEM 2024 | Incremental layered SVFA; decomposes SVFG into incremental PTA + incremental Mod-Ref | 7x over SVF | Medium-High |
| **PUS** | ICSE 2022 (Distinguished) | Partial constraint solving; only considers causally-related constraints per stage | 2x CI, 7x CS-PTA | Medium |
| **Semi-Sparse** | Hardekopf & Lin, CGO'11 | Only address-taken vars need FS tracking; SSA vars are already precise | 197x over naive FS | Low-Medium |

### Tier 2 — Parallel/scalable

| Algorithm | Paper | Key Idea | Expected Speedup | Effort |
|-----------|-------|----------|-----------------|--------|
| **Octopus** | TOSEM 2024 | Parallel value-flow via segment decomposition | 123x at 40 threads | High |
| **GraCFL** | ICS 2025 | Parallel CFL reachability with temporal vectors | Significant over SOTA | High |
| **Graspan** | ASPLOS'17 / TOCS'21 | Disk-based parallel graph analysis for huge programs | Enables kernel-scale | High |

### Tier 3 — Efficiency optimizations

| Algorithm | Paper | Key Idea | Expected Speedup | Effort |
|-----------|-------|----------|-----------------|--------|
| **FLARE** | OOPSLA 2022 | CFL reachability indexing; near-constant-time queries | 81–270K× per query | Medium |
| **Zipper/Zipper-e** | OOPSLA 2018 | Selective context sensitivity via precision flow graphs | 3.4–88x for CS-PTA | Medium |
| **Wave/Deep Propagation** | CGO 2009 | Level-by-level or depth-first constraint propagation | Competitive with best PTA | Low |
| **Push-Pull CG** | ISMM 2014 | Asymmetric load/store propagation; lazy at dereferences | 21–33% over Deep | Low-Medium |

---

## Part 3: Fast Data Structures

### Tier 1 — Highest impact/effort ratio

| Data Structure | Crate | Where in SAF | Expected Benefit | Effort | Determinism |
|---------------|-------|-------------|-----------------|--------|-------------|
| **IndexMap + nohash-hasher** | `indexmap`, `nohash-hasher` | Replace 2,114 BTreeMap/BTreeSet occurrences | 2–5x PTA solver (O(1) vs O(log n) lookup) | Low | Yes (insertion order) |
| **SmallVec** | `smallvec` | Constraint index `Vec<usize>` (most have 1-3 entries) | 20–40% fewer heap allocations | Very Low | Yes |
| **Roaring Bitmaps** | `roaring` | Points-to set representation (sparse sets) | 2–10x memory, 1.5–3x speed | Medium | Yes (sorted) |

### Tier 2 — Medium-term wins

| Data Structure | Crate | Where in SAF | Expected Benefit | Effort |
|---------------|-------|-------------|-----------------|--------|
| **im-rc** (persistent HAMT) | `im-rc` | Flow-sensitive states (df_in/df_out cloning) | 10–100x clone speedup | Medium |
| **Petgraph CSR** | `petgraph` | Frozen graphs (CFG, ICFG, CallGraph) | 3–5x graph traversal | Medium |
| **SlotMap** | `slotmap` | Graph node storage | 3–10x traversal | Medium |
| **bumpalo** | `bumpalo` | Pass-scoped temporary allocations | 2–5x allocation-heavy passes | Low-Medium |
| **SoA layout** | Manual / `soa_derive` | Constraint storage (hot iteration) | 2–4x constraint processing | Medium |

### Tier 3 — Longer-term / situational

| Data Structure | Where | Expected Benefit |
|---------------|-------|-----------------|
| **OxiDD** (modern BDD) | BDD points-to sets | 2–5x over custom BDD |
| **crossbeam + DashMap** | Parallel analysis | Near-linear scaling (determinism risk) |

---

## Part 4: SVF Design Patterns Not Yet in SAF

### Critical — Directly addresses fspta/SVFG bottleneck

| Technique | SVF Module | SAF Status | Expected Speedup | Effort |
|-----------|-----------|------------|-----------------|--------|
| **Object Versioning (VFS)** | `VersionedFlowSensitive` | **No** | 5–26x for fspta | High |
| **SVFG Node Optimization** | `SVFGOPT` | **No** | 1.3–2x (30-50% fewer nodes) | Medium |
| **Memory Region Partitioning** | `MRGenerator` | **Partial** | 2–5x for MSSA/SVFG | High |

### High — PTA pre-analysis improvements

| Technique | SVF Module | SAF Status | Expected Speedup | Effort |
|-----------|-----------|------------|-----------------|--------|
| **Wave Propagation (2-phase)** | `AndersenWaveDiff` | **Partial** | 1.2–1.5x for PTA | Low |
| **HVN Pre-processing** | Andersen `initialize()` | **No** | 1.2–1.4x (20-40% fewer vars) | Medium |
| **Steensgaard-seeded Clustering** | `NodeIDAllocator::Clusterer` | **Partial** | 1.1–1.3x | Low |
| **LCD/HCD Cycle Detection** | Constraint solving | **No** | 1.05–1.2x | Medium |
| **PWC/SFR Field Handling** | `AndersenSFR` | **Partial** | Prevents timeouts | Medium |

### Medium — Architectural improvements

| Technique | Description | SAF Status | Effort |
|-----------|------------|------------|--------|
| **SUPA Demand-Driven FS-PTA** | Skip whole-program fspta; answer queries on-demand | Partial (DDA exists) | Low |
| **Parallel MSSA** | Per-region parallel construction | No | High |
| **Unified PAG** | Single shared constraint graph across analyses | Partial | High |

---

## Synthesis: Recommended Implementation Roadmap

### Phase 1: Quick Wins (1–2 days, ~3–8x cumulative speedup)

1. **Fix CG refinement PtsConfig bug** (1C) — one-line fix, enables BitVec/BDD in CG loop
2. **Wire Steensgaard → clustering seed** (1A + 4F) — enable `experimental`, connect existing code
3. **IndexMap + nohash-hasher migration** (3.1) — drop-in BTreeMap replacement, 2–5x PTA solver
4. **SmallVec for constraint indices** (3.2) — drop-in Vec replacement, 20–40% fewer allocations
5. **Wave propagation restructure** (4D) — reorganize solver main loop for 2-phase wave+diff

### Phase 2: FSPTA Overhaul (3–5 days, ~5–26x fspta speedup)

6. **Object Versioning (VFS)** (4.1) — replace df_in/df_out maps with version table. Single biggest win.
7. **SVFG Node Optimization** (4.2) — eliminate redundant interprocedural nodes before FS-PTA
8. **im-rc for remaining FS states** (3.4) — persistent data structures where versioning doesn't apply
9. **FSPTA PtsSet generics** (1D) — wire BitVec/Roaring into flow-sensitive solver

### Phase 3: MSSA/SVFG Scalability (3–5 days, ~2–5x SVFG construction)

10. **Memory Region Partitioning** (4.3) — partition objects by aliasing; separate MSSA per region
11. **Roaring Bitmaps for points-to sets** (3.3) — adaptive compression for sparse sets
12. **HVN Pre-processing** (4E) — reduce constraint graph 20–40% before solving

### Phase 4: Advanced Algorithms (5–10 days, potential 7–10x additional)

13. **CG-FSPTA** (2.1) — flow-sensitive constraint graph (replaces SVFG-based fspta entirely)
14. **PUS Partial Constraint Solving** (2.3) — staged partial constraints
15. **Semi-Sparse audit** (2.4) — verify SAF only tracks address-taken vars flow-sensitively
16. **Zipper selective CS** (2.9) — context sensitivity only where needed
17. **FLARE reachability indexing** (2.8) — near-instant DDA queries

### Phase 5: Parallelism (5+ days, near-linear scaling)

18. **Octopus parallel value-flow** (2.5) — segment decomposition for parallel analysis
19. **Petgraph CSR for frozen graphs** (3.5) — cache-friendly read-only traversal
20. **Parallel MSSA construction** (4.11) — per-region parallel MSSA

---

## Expected Cumulative Impact

| Phase | Target | Estimated Speedup | bunzip2 fspta | bash/libcurl |
|-------|--------|-------------------|---------------|--------------|
| Current | — | Baseline | 21.8s | Timeout (>10min) |
| Phase 1 | Quick wins | 3–8x | ~5s | Still slow |
| Phase 2 | VFS + SVFGOPT | 5–26x on fspta | <1s | ~2–5min |
| Phase 3 | MSSA regions | 2–5x on SVFG | <0.5s | ~1–2min |
| Phase 4 | CG-FSPTA etc. | 7–10x additional | <0.1s | <30s |
| Phase 5 | Parallelism | Near-linear cores | <0.05s | <10s |

**Conservative estimate:** Phases 1–3 alone should bring bash/libcurl from timeout to analyzable (minutes). Phases 4–5 would make SAF competitive with or faster than SVF on all CruxBC benchmarks.
