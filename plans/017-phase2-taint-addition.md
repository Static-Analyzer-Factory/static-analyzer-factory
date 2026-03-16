# Plan 017: Phase 2 — Taint Cross-Module Tutorial

**Parent:** Plan 015 (Tutorial Reorganization)
**Epic:** E8
**Prerequisite:** Plan 016 (directories restructured)

## Goal

Create `tutorials/taint/06-cross-module-taint/` — a tutorial demonstrating taint flow across module boundaries using two C source files.

## Tutorial Spec

**Source programs:** Two C files (~30 lines each)
- `module_a.c` — reads user input via `getenv()`, passes to `module_b_process()`
- `module_b.c` — receives data, calls `system()` without sanitization

**Key concept:** The taint source (`getenv`) and sink (`system`) live in different translation units. SAF must track the flow across the module boundary.

**detect.py:**
1. Compile both C files to LLVM IR via `clang-18 -S -emit-llvm -O0 -g`
2. Load via `Project.open()` (may need to link/combine the two `.ll` files — investigate how Project.open handles multi-file input)
3. Run `q.taint_flow(sources=sources.call("getenv"), sinks=sinks.call("system", arg_index=0))`
4. Print findings and trace showing cross-module path

**detect.rs:** Same pipeline using Rust crate API.

**README.md:** Explain cross-module taint, why it's harder than single-file, how SAF's interprocedural analysis handles it.

## Known Risk

`Project.open()` currently takes a single file path. Cross-module analysis may require:
- Option A: `llvm-link` to combine `.ll` files before loading
- Option B: Extending `Project.open()` to accept multiple paths
- Option C: Writing both functions in a single `.c` file (simpler but less authentic)

Investigate which approach works, prefer A or B over C.

## Verification

- `python detect.py` finds at least 1 cross-module taint flow
- Trace shows the flow crossing from module_a to module_b

## On Completion

Update `PROGRESS.md`:
- Set plan 017 status to `done`
- Update task checklist: T2 → `done`
- Update "Next Steps" to point to plan 018
