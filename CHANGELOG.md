# Changelog

All notable changes to SAF (Static Analyzer Factory) are documented here.

## [0.1.0] - 2026-04-07

### Display & Usability
- Human-readable display resolver for AIR entity labels (function names, variable names from LLVM debug info)
- CLI output enrichment with human-readable names and SARIF support
- Enriched query results (taint_flow, flows, points_to, alias) with display names

### Development Workflows
- SAF Feature Development skill for guided 8-phase feature workflow
- SAF Checker Development skill for spec-first checker creation
- Structured debug logging system (`SAF_LOG`) with module/phase/tag filtering

### Documentation & Education
- Landing page with interactive animations
- Browser-based playground with WASM analysis pipeline
- Interactive tutorials app with step-by-step walkthroughs
- mdBook documentation covering architecture, concepts, and API reference

### Benchmark Infrastructure
- PTABen benchmark integration with compiled LLVM IR test suite
- NIST Juliet C/C++ v1.3 integration (15 CWE categories, ~24,815 tests)
- Per-CWE precision/recall/F1 scoring and SV-COMP compatible scoring

### Advanced Checkers & Analysis
- SVFG-based checker framework with 9 built-in checkers (memory leak, UAF, double-free, null-deref, file-descriptor leak, uninit use, stack escape, lock not released, generic resource leak)
- Path-sensitive checker reachability with Z3 SMT constraint solving
- CFL-reachability checker traversal for interprocedural analysis
- Partial leak detection via SVFG forward/backward slicing
- UAF temporal refinement and null-deref precision improvements
- Abstract interpretation framework with interval and constant-propagation domains

### Python SDK
- PyO3-based Python bindings with layered API (Project, selectors, queries)
- Schema-driven analyzer authoring framework
- Typestate analysis with declarative specs
- IDE solver with built-in edge functions

### CLI
- Full-featured CLI with graph export (JSON, PropertyGraph), SARIF output
- Query interface for flows, taint_flow, points_to, alias analysis
- Configurable analysis pipeline with TOML configuration

### Core Analysis
- Sparse Value-Flow Graph (SVFG) with 4-phase builder and reachability queries
- Andersen's pointer analysis with configurable field sensitivity
- Flow-sensitive and context-sensitive pointer analysis
- Memory SSA for heap-aware value flow
- CFG, ICFG, call graph, and def-use graph builders
- CHA + PTA iterative call graph refinement
- IFDS/IDE framework with typestate analysis client
- Z3-enhanced analysis (assertion proving, alias refinement, path reachability)
- Demand-driven pointer analysis via SVFG backward traversal

### LLVM Frontend
- LLVM 18 bitcode and text IR ingestion
- Frontend-agnostic AIR (Analysis Intermediate Representation)
- AIR-JSON frontend for testing and interoperability
- Debug info extraction for source location mapping

### Foundation
- Cargo workspace with 10+ crates (core, frontends, analysis, python, cli, bench, wasm, etc.)
- Docker-based development environment (Ubuntu 24.04 LTS, LLVM 18)
- BLAKE3-based deterministic ID system (NFR-DET compliance)
- Comprehensive CI/CD with GitHub Actions

## Contributors

- **Yuekang Li** ([@thepatrickstar](https://github.com/thepatrickstar))
- **Wei Li** ([@leeewee](https://github.com/leeewee))
- **Hongxu Chen** ([@hongxuchen](https://github.com/hongxuchen))
