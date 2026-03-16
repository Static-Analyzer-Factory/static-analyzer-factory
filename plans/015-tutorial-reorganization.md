# Plan 015: Tutorial Reorganization & New Tutorial Categories (Design Overview)

## Overview

Reorganize the existing 5 taint-only tutorials into a category-based structure and add 15 new tutorials covering graph exploration, pointer analysis, and tool integration. Tutorials span three audiences (security researchers, analysis students, tool integrators), three source languages (C, C++, Rust), and a range of program sizes (10 lines to 800+ lines).

## Current State

All 5 tutorials live in `tutorials/01-*` through `tutorials/05-*`. They all use `q.taint_flow()` with different source/sink labels. No tutorials cover `graphs().export()`, `pta_result()`, `points_to()`, `may_alias()`, `schema()`, `Finding.to_dict()`, SARIF output, or batch scanning.

## Target Structure

```
tutorials/
├── SETUP.md                          (updated — new category table)
├── taint/
│   ├── README.md                     (category overview)
│   ├── 01-command-injection/         (moved from tutorials/01-taint-detection/)
│   ├── 02-format-string/             (moved from tutorials/02-format-string/)
│   ├── 03-use-after-free/            (moved from tutorials/03-use-after-free/)
│   ├── 04-buffer-overflow/           (moved from tutorials/04-buffer-overflow/)
│   ├── 05-unsafe-rust/               (moved from tutorials/05-unsafe-rust/)
│   └── 06-cross-module-taint/        (NEW)
├── graphs/
│   ├── README.md
│   ├── 01-callgraph-visualization/
│   ├── 02-cfg-exploration/
│   ├── 03-defuse-chains/
│   └── 04-valueflow-graph/
├── pta/
│   ├── README.md
│   ├── 01-pointer-aliasing/
│   ├── 02-indirect-calls/
│   ├── 03-cpp-virtual-dispatch/
│   ├── 04-field-sensitive-structs/
│   └── 05-large-cpp-program/
└── integration/
    ├── README.md
    ├── 01-schema-discovery/
    ├── 02-json-export-pipeline/
    ├── 03-sarif-reporting/
    ├── 04-batch-scanning/
    └── 05-large-c-codebase/
```

Each tutorial directory contains:
- Source file(s): `vulnerable.c`, `vulnerable.cpp`, `vulnerable.rs`, or multiple `.c` files
- `detect.py` — Python detection/exploration script
- `detect.rs` — Rust equivalent
- `README.md` — Walkthrough with explanation

## Implementation Plans

This design is implemented across 6 phase plans:

| Plan | Phase | Description |
|------|-------|-------------|
| 016 | Phase 1 | Restructure directories — move existing taint tutorials, create category dirs |
| 017 | Phase 2 | Taint addition — `taint/06-cross-module-taint` |
| 018 | Phase 3 | Graph tutorials — `graphs/01` through `graphs/04` |
| 019 | Phase 4 | PTA tutorials — `pta/01` through `pta/05` |
| 020 | Phase 5 | Integration tutorials — `integration/01` through `integration/05` |
| 021 | Phase 6 | Final verification — Docker test all tutorials, update PROGRESS.md + FUTURE.md |

Each plan is independently implementable in a single Claude Code session. See individual plan files for task details.

## Known Risks

- **Large programs (plans 019/020)** may expose PTA performance issues or analysis bugs — intentional stress tests
- **Cross-module loading (plan 017)** — may need multiple `.ll` file support in `Project.open()` or pre-linking
- **Graph export tutorials (plan 018)** — may reveal that exported JSON structure is hard to interpret without documentation
- **Virtual dispatch (plan 019)** — LLVM lowers vtables to GEPs and indirect calls; PTA must resolve these
- **Field sensitivity (plan 019)** — linked list traversal may exceed `max_depth`; tutorial should demonstrate this

## Implementation Notes

- All tutorials must be truly end-to-end: source code → compile → LLVM IR → LLVM frontend → analysis → output
- No hand-crafted AIR-JSON fixtures
- Each tutorial should work inside Docker via `make shell`
- detect.rs files use the Rust crate API directly (same pipeline, different language)
- README.md files should explain the vulnerability/concept, walk through the code, and explain the detection output
