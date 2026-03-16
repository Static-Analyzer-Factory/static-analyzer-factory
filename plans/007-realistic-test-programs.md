# E7: Realistic Test Programs — Design Document

## Overview

Create realistic C, C++, and Rust programs that exercise SAF's full analysis pipeline end-to-end:
source code → LLVM IR → LLVM frontend → AIR → graphs → PTA → ValueFlow → queries → findings.

All test programs validate both the **Rust API** (direct crate usage) and the **Python SDK** (saf package).
The most instructive programs are also wrapped into `tutorials/` with narrative walkthroughs.

## Goals

1. **End-to-end validation**: Real compiled programs through the full SAF pipeline
2. **Broad CWE coverage**: Taint, memory safety, integer, information leak, uninitialized memory
3. **Cross-language**: C, C++, and Rust (including `unsafe`) programs
4. **Dual API coverage**: Every test program has both Rust and Python integration tests
5. **Tutorials**: Human-readable examples for learning SAF

## Epic Structure

**Epic**: E7 — Realistic Test Programs

| Plan | Category | Status |
|------|----------|--------|
| 007 | Taint flow / injection (C + Rust unsafe) | draft |
| 008 | Memory safety (C + C++ + Rust unsafe) | draft |
| 009 | Integer / info / uninitialized (C + C++) | draft |
| 010 | OOP & language patterns (C++ + Rust) | draft |
| 011 | Multi-module / interprocedural (C + C++) | draft |
| 012 | Tutorials | draft |

Plans 007–011 are test programs. Plan 012 builds tutorials from the test programs.
Each plan will get its own detailed implementation file when work begins.

## Directory Structure

```
tests/
  programs/                    # Source files for test programs
    c/
      command_injection.c
      format_string.c
      sql_injection.c
      path_traversal.c
      taint_sanitized.c
      use_after_free.c
      double_free.c
      null_deref.c
      buffer_overflow.c
      integer_overflow.c
      info_leak.c
      uninitialized.c
      callback_fn_ptr.c
    cpp/
      dangling_ptr.cpp
      uninitialized_heap.cpp
      vtable_dispatch.cpp
      raii_resource.cpp
    rust/
      taint_unsafe.rs
      dangling_ptr.rs
      trait_dispatch.rs
    compile.sh                 # Compiles all sources to .ll
  fixtures/
    llvm/                      # Compiled .ll files (checked in)
      command_injection.ll
      ...
  e2e/                         # End-to-end Rust integration tests
    taint_e2e.rs
    memory_e2e.rs
    integer_info_e2e.rs
    oop_patterns_e2e.rs
    multi_module_e2e.rs

python/
  tests/
    e2e/                       # End-to-end Python integration tests
      test_taint_e2e.py
      test_memory_e2e.py
      test_integer_info_e2e.py
      test_oop_patterns_e2e.py
      test_multi_module_e2e.py

tutorials/
  01-command-injection/
    injection.c                # Annotated source
    injection.ll               # Compiled IR
    detect.py                  # Python SDK detection script
    detect.rs                  # Rust API detection example
    README.md                  # Narrative walkthrough
  02-format-string/
    ...
  03-use-after-free/
    ...
  04-buffer-overflow/
    ...
  05-unsafe-rust/
    ...
```

## Compilation

All source files are compiled inside Docker using:

```bash
# C files
clang-18 -S -emit-llvm -O0 -g -o output.ll input.c

# C++ files
clang++-18 -S -emit-llvm -O0 -g -o output.ll input.cpp

# Rust files
rustc --emit=llvm-ir -C opt-level=0 -g -o output.ll input.rs
```

Flags:
- `-S -emit-llvm`: text LLVM IR (readable, diffable)
- `-O0`: no optimizations (preserves source structure)
- `-g`: debug info (provides source spans)

The `.ll` files are checked into `tests/fixtures/llvm/` so tests run without recompilation.
A `compile.sh` script regenerates them when source files change.

## Test Programs

### Plan 007 — Taint Flow / Injection

| Program | Lang | CWE | Pattern | Key Analysis |
|---------|------|-----|---------|-------------|
| `command_injection.c` | C | CWE-78 | `argv` → `system()` | taint_flow, sources.argv, sinks.call |
| `format_string.c` | C | CWE-134 | `scanf` → `printf` format arg | taint_flow, arg_index selectors |
| `sql_injection.c` | C | CWE-89 | `getenv` → `sqlite3_exec` | taint_flow, sources.getenv |
| `path_traversal.c` | C | CWE-22 | `argv` → `fopen` | taint_flow, sinks.arg_to |
| `taint_sanitized.c` | C | CWE-78 | `argv` → sanitize → `system()` | taint_flow with sanitizers (negative test) |
| `taint_unsafe.rs` | Rust | CWE-78 | `env::args` → `libc::system` via unsafe | taint_flow, cross-language |

**Rust test assertions**:
- taint_flow returns findings for unsanitized paths
- taint_flow with sanitizer returns no findings for sanitized path
- Source/sink locations have correct spans

**Python test assertions**:
- `query.taint_flow(sources=sources.argv(), sinks=sinks.call("system"))` finds the flow
- `Finding.to_dict()` contains expected fields
- Sanitizer correctly blocks flow

### Plan 008 — Memory Safety

| Program | Lang | CWE | Pattern | Key Analysis |
|---------|------|-----|---------|-------------|
| `use_after_free.c` | C | CWE-416 | `free(p); *p` | PTA, points_to after free |
| `double_free.c` | C | CWE-415 | `free(p); free(p)` | PTA, may_alias confirms same object |
| `null_deref.c` | C | CWE-476 | unchecked NULL from `malloc` | ValueFlow, null propagation |
| `buffer_overflow.c` | C | CWE-122 | heap buffer overrun via loop | GEP, ValueFlow |
| `dangling_ptr.cpp` | C++ | CWE-416 | return reference to local | PTA, stack object lifetime |
| `dangling_ptr.rs` | Rust | CWE-416 | unsafe raw pointer after drop | PTA, Rust unsafe patterns |

**Test assertions**:
- PTA shows freed pointer still points to deallocated object
- may_alias confirms double-free targets are the same
- ValueFlow traces null from malloc failure to dereference

### Plan 009 — Integer / Info / Uninitialized

| Program | Lang | CWE | Pattern | Key Analysis |
|---------|------|-----|---------|-------------|
| `integer_overflow.c` | C | CWE-190 | unchecked `size * count` → `malloc` | ValueFlow, BinaryOp tracking |
| `info_leak.c` | C | CWE-200 | stack buffer → `send()` | taint_flow (sensitive data as source) |
| `uninitialized.c` | C | CWE-457 | uninitialized stack var used in condition | DefUse, no reaching def |
| `uninitialized_heap.cpp` | C++ | CWE-908 | `new` without initialization, used | PTA + ValueFlow |

**Test assertions**:
- ValueFlow traces arithmetic result to allocation size
- DefUse graph shows use without preceding def
- Taint from sensitive data reaches network sink

### Plan 010 — OOP & Language Patterns

| Program | Lang | Pattern | Key Analysis |
|---------|------|---------|-------------|
| `vtable_dispatch.cpp` | C++ | virtual method call through base pointer | Indirect call, PTA resolves vtable |
| `callback_fn_ptr.c` | C | function pointer stored and called later | Indirect call resolution via PTA |
| `raii_resource.cpp` | C++ | constructor acquires / destructor releases | CallGraph, interprocedural flow |
| `trait_dispatch.rs` | Rust | trait object `dyn Trait` dispatch | Indirect call, PTA resolves impl |

**Test assertions**:
- PTA resolves indirect calls to correct target functions
- CallGraph includes edges from indirect call sites to resolved targets
- Interprocedural ValueFlow crosses constructor/destructor boundaries

### Plan 011 — Multi-Module / Interprocedural

| Program | Lang | Pattern | Key Analysis |
|---------|------|---------|-------------|
| `cross_module_taint/` | C | taint source in module A, sink in module B | Multi-file `.ll`, ICFG |
| `library_wrapper/` | C | wrapper function hides dangerous sink | Interprocedural taint through wrapper |
| `callback_chain/` | C | A→B via fn ptr, B→C directly | CallGraph + PTA + taint across chain |

These use `llvm-link-18` to combine multiple `.ll` files into a single module,
or the LLVM frontend's multi-file ingestion if supported.

**Test assertions**:
- Taint flow crosses module boundaries
- CallGraph resolves full call chain
- Wrapper function doesn't block taint propagation

### Plan 012 — Tutorials

| Tutorial | Derived From | Why |
|----------|-------------|-----|
| 01-command-injection | `command_injection.c` | SRS Appendix A canonical example |
| 02-format-string | `format_string.c` | Shows arg_index selectors |
| 03-use-after-free | `use_after_free.c` | PTA + memory queries |
| 04-buffer-overflow | `buffer_overflow.c` | GEP + bounds reasoning |
| 05-unsafe-rust | `dangling_ptr.rs` | Cross-language support demo |

Each tutorial includes:
- **Annotated source** with comments explaining the vulnerability
- **Python script** (`detect.py`) using the SAF Python SDK
- **Rust example** (`detect.rs`) using the SAF Rust API directly
- **README.md** narrative walkthrough: what the bug is, how SAF finds it, how to run it

## Implementation Order

1. **Plan 007** (taint) — most directly tests the Python SDK query API
2. **Plan 008** (memory) — validates PTA correctness on real programs
3. **Plan 009** (integer/info) — ValueFlow and DefUse edge cases
4. **Plan 010** (OOP/patterns) — indirect call resolution
5. **Plan 011** (multi-module) — interprocedural across files
6. **Plan 012** (tutorials) — built after test programs exist

Each plan follows TDD: write the test assertions first (what we expect SAF to find),
then write the source program, compile it, and verify the tests pass.

## Infrastructure Changes

1. Add `tests/programs/compile.sh` with compilation commands
2. Add `make compile-fixtures` target (runs compile.sh in Docker)
3. Add `tests/e2e/` for Rust end-to-end tests
4. Add `python/tests/e2e/` for Python end-to-end tests
5. Update Docker test stage to include e2e tests in the test run
6. Add `tutorials/` directory

## Success Criteria

- All ~22 test programs compile to valid LLVM IR
- LLVM frontend ingests all `.ll` files without errors
- Rust integration tests pass for each program
- Python integration tests pass for each program
- 5 tutorials are complete with working Python and Rust examples
- `make test` runs all e2e tests alongside existing unit tests
