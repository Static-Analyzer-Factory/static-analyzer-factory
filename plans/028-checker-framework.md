# Plan 028: SABER-Style Checker Framework

**Epic:** E14 — Checker Framework
**Status:** done
**Created:** 2026-01-30

## Overview

Build a declarative, AI-agent-friendly checker framework using SVFG graph reachability. Ships with 9 built-in checkers for memory safety, resource safety, and common bug patterns. Checkers are defined as data (source/sink/sanitizer specs), not code — AI agents can author custom checkers via the Python API without writing Rust.

This is SAF's first bug-finding capability beyond taint analysis, converting SAF from "analysis framework" to "bug-finding tool."

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| Declarative checker specs (not trait-based) | AI agents can author checkers as data, not code. Beats SVF's hardcoded C++ classes. |
| Path-insensitive reachability for E14 | Matches SVF SABER. Path-sensitivity (Pinpoint-style guard conditions) deferred to future epic. |
| Two reachability modes (may_reach, must_not_reach) | Covers all 9 checkers. may_reach = "bad thing happens on some path." must_not_reach = "good thing missing on some path." |
| Built-in resource table + user-extensible | Default table covers C stdlib, C++ operators, POSIX I/O, pthreads. Users add custom pairs via Python API. |
| 9 built-in checkers | Memory leak, UAF, double-free, null-deref, file-descriptor leak, uninit use, stack escape, lock not released, generic resource. All are pre-packaged CheckerSpecs. |
| SVFG-based (not ValueFlow-based) | SVFG (E12) has precise store→load edges via MSSA clobber analysis. More precise than the older ValueFlowGraph. |

## Architecture

```
┌─────────────────────────────────────────────┐
│  Layer 3: Built-in Checkers (9 specs)       │
│  leak, uaf, double_free, null_deref,        │
│  file_leak, uninit_use, stack_escape,       │
│  lock_not_released, generic_resource_leak   │
├─────────────────────────────────────────────┤
│  Layer 2: Reachability Solver               │
│  may_reach(src, sink, sanitizer) → findings │
│  must_not_reach(src, san, exit) → findings  │
│  Both operate on Svfg graph from E12        │
├─────────────────────────────────────────────┤
│  Layer 1: Resource Specification            │
│  ResourceTable: built-in + user-defined     │
│  function name → role (alloc/dealloc/       │
│  acquire/release/null_source/deref/etc.)    │
│  Site classifier: SVFG node → role lookup   │
├─────────────────────────────────────────────┤
│  Foundation: Svfg (E12) + PtaResult (E4/13) │
└─────────────────────────────────────────────┘
```

## Resource Table

### ResourceRole enum

```rust
pub enum ResourceRole {
    Allocator,          // malloc, calloc, new, mmap
    Deallocator,        // free, delete, munmap
    Reallocator,        // realloc (both dealloc old + alloc new)
    Acquire,            // fopen, open, socket, pthread_mutex_lock
    Release,            // fclose, close, pthread_mutex_unlock
    NullSource,         // functions that may return null
    Dereference,        // memcpy, strlen, printf (deref pointer args)
}
```

A function can have multiple roles (e.g., `malloc` is both `Allocator` and `NullSource`).

### Built-in table coverage

- **C stdlib**: malloc, calloc, realloc, free, strdup, aligned_alloc
- **C++ operators**: operator new/delete (and array variants)
- **POSIX I/O**: open/close, fopen/fclose, fdopen, socket, pipe, dup
- **POSIX threads**: pthread_mutex_lock/unlock, pthread_rwlock_rdlock/wrlock/unlock
- **Memory mapping**: mmap/munmap
- **Common wrappers**: xmalloc, g_malloc (GLib)

### User extension (Python)

```python
project.resource_table.add("pool_alloc", saf.Allocator)
project.resource_table.add("pool_free", saf.Deallocator)
```

## Checker Spec

```rust
pub struct CheckerSpec {
    pub name: String,
    pub description: String,
    pub cwe: Option<u32>,
    pub severity: Severity,
    pub mode: ReachabilityMode,          // MayReach or MustNotReach
    pub sources: Vec<SitePattern>,
    pub sinks: Vec<SitePattern>,
    pub sanitizers: Vec<SitePattern>,
}

pub enum ReachabilityMode {
    MayReach,       // report if source reaches sink on SOME path without sanitizer
    MustNotReach,   // report if source does NOT reach sanitizer on ALL paths before exit
}

pub enum SitePattern {
    Role(ResourceRole),                  // match by resource table role
    FunctionName(String),                // match specific function
    FunctionExit,                        // any function return/exit
    AnyUseOf,                            // any use of value flowing from source
    AllocaInst,                          // stack allocation (alloca)
    StoreToNonStack,                     // store to heap or global (for stack escape)
}
```

### Built-in checker specs

| Checker | Mode | Sources | Sinks | Sanitizers |
|---------|------|---------|-------|------------|
| memory-leak | MustNotReach | Role(Allocator) | FunctionExit | Role(Deallocator) |
| use-after-free | MayReach | Role(Deallocator) | AnyUseOf | — |
| double-free | MayReach | Role(Deallocator) | Role(Deallocator) | — |
| null-deref | MayReach | Role(NullSource) | Role(Dereference) | null-check guard |
| file-descriptor-leak | MustNotReach | Role(Acquire) | FunctionExit | Role(Release) |
| uninit-use | MayReach | Role(Allocator) no-init | AnyUseOf | store to ptr |
| stack-escape | MayReach | AllocaInst | StoreToNonStack / return | — |
| lock-not-released | MustNotReach | Role(Acquire) locks | FunctionExit | Role(Release) locks |
| generic-resource | (user picks) | (user-defined) | (user-defined) | (user-defined) |

## Reachability Solver

### may_reach algorithm

1. Collect all source nodes from ClassifiedSites
2. For each source, BFS forward on SVFG
3. If we reach a sanitizer node, prune that path
4. If we reach a sink node, report finding with trace
5. Visited-set prevents revisiting (handles cycles)
6. Bounded by configurable max_path_depth

### must_not_reach algorithm

1. Collect all source nodes from ClassifiedSites
2. For each source, BFS forward on SVFG
3. Track whether every reachable function exit has a sanitizer on the path
4. If any exit is reachable without sanitizer, report finding
5. Bounded by configurable max_path_depth

### Output

```rust
pub struct CheckerFinding {
    pub checker_name: String,
    pub severity: Severity,
    pub source_node: SvfgNodeId,
    pub sink_node: SvfgNodeId,
    pub trace: Vec<SvfgNodeId>,
    pub cwe: Option<u32>,
    pub message: String,
}
```

## Python API

### Level 1 — Run built-in checker
```python
project = saf.Project.open("target.ll")
findings = project.check("memory-leak")
for f in findings:
    print(f.message, f.cwe, f.severity, f.trace)
```

### Level 2 — Run multiple checkers
```python
findings = project.check_all()
findings = project.check(["memory-leak", "uaf"])
```

### Level 3 — Author custom checker
```python
project.resource_table.add("db_connect", saf.Acquire)
project.resource_table.add("db_disconnect", saf.Release)

findings = project.check_custom(
    name="database-connection-leak",
    sources=saf.role(saf.Acquire, "db_connect"),
    sinks=saf.function_exits(),
    sanitizers=saf.role(saf.Release, "db_disconnect"),
    mode=saf.MustNotReach,
    cwe=772,
    severity=saf.Warning,
)
```

### Schema discovery
```python
project.checker_schema()  # dict of all checkers, specs, resource table
```

### Export
All findings support `to_dict()`, JSON export, and SARIF export with CWE IDs.

## Testing Strategy

### E2E tests (TDD — write tests first)

Each checker gets at least one C and one C++ test program with known bugs, compiled through the full pipeline (source → clang-18 → .ll → LLVM frontend → SVFG → checker).

| Checker | C test | C++ test | Rust test |
|---------|--------|----------|-----------|
| memory-leak | malloc without free, conditional leak | new without delete | Box::into_raw not reclaimed |
| use-after-free | free then dereference | delete then method call | raw pointer use after drop |
| double-free | free called twice in branches | delete in destructor + explicit delete | — |
| null-deref | unchecked malloc return | unchecked dynamic_cast | — |
| file-descriptor-leak | fopen without fclose on error path | ifstream not closed (negative) | — |
| uninit-use | stack variable read before write | class member used before ctor body | — |
| stack-escape | return &local_var | return pointer to local object | — |
| lock-not-released | lock acquired, early return skips unlock | lock_guard (negative) | — |
| generic-resource | custom alloc/free pair | — | — |

Negative tests validate that RAII patterns (unique_ptr, lock_guard, ifstream) do NOT trigger false positives.

### Python E2E tests

Mirror the Rust E2E tests through the Python API. Also test:
- `check_all()` runs all checkers
- `check_custom()` with user-defined resource pairs
- `checker_schema()` returns expected structure
- `checker_diagnostics()` returns stats
- SARIF export includes CWE IDs

### Tutorials

1. `tutorials/checkers/01-memory-safety/` — realistic C program (e.g., HTTP request parser) with memory leak, UAF, and null-deref. detect.py runs memory checkers and explains findings.

2. `tutorials/checkers/02-resource-safety/` — C program that opens files/sockets with error paths that leak. Plus custom checker demo for user-defined resource.

## Implementation Phases

| Phase | What | Depends on |
|-------|------|------------|
| 1 | ResourceTable: built-in function table, ResourceRole enum, add/lookup API, unit tests | — |
| 2 | Site classifier: walk SVFG + AIR, match call sites to ResourceTable, produce ClassifiedSites, unit tests | Phase 1 |
| 3 | CheckerSpec + ReachabilityMode: data types, built-in specs (all 9), unit tests | Phase 1 |
| 4 | may_reach solver: forward BFS on SVFG with sanitizer pruning, CheckerFinding output, unit tests | Phase 2, 3 |
| 5 | must_not_reach solver: forward BFS checking all-paths-to-exit, unit tests | Phase 2, 3 |
| 6 | Checker runner: orchestrator (specs + classified sites + solver → findings), JSON + SARIF export, unit tests | Phase 4, 5 |
| 7 | E2E tests: compile C/C++/Rust sources, run full pipeline, assert expected findings (positive + negative) | Phase 6 |
| 8 | Python bindings: Project.check(), check_all(), check_custom(), resource_table, checker_schema(), checker_diagnostics(), Python E2E tests | Phase 6 |
| 9 | Tutorials: 2 tutorials (memory-safety, resource-safety), verified end-to-end in Docker | Phase 8 |
| 10 | Documentation: update tool-comparison.md, PROGRESS.md, FUTURE.md (path-sensitivity as upgrade) | Phase 9 |

## Future Enhancements (not in E14)

- **Path-sensitive reachability**: guard conditions on SVFG edges, prune infeasible paths (Pinpoint-style). Major precision upgrade.
- **Interprocedural wrapper detection**: auto-detect functions that wrap malloc/free via SVFG reachability.
- **Glob/pattern matching in ResourceTable**: e.g., `*_alloc` matches any function ending in `_alloc`.
- **Typestate checker mode**: extend beyond 2-state (source/sink) to N-state finite automata.
- **Checker composition**: combine findings from multiple checkers (e.g., "null-deref after failed malloc" = leak + null-deref).

## References

- SVF SABER: [ISSTA 2012 paper](https://dl.acm.org/doi/10.1145/2338965.2336784) — memory leak detection via full-sparse value-flow analysis
- Pinpoint: [PLDI 2018 paper](https://dl.acm.org/doi/10.1145/3192366.3192418) — fast path-sensitive SVFG-based checking for 2MLOC
- SVF source: [SABER module](https://github.com/SVF-tools/SVF/tree/master/svf/include/SABER) — LeakChecker, DoubleFreeChecker, FileChecker, SrcSnkDDA
- Infer Pulse: [fbinfer.com](https://fbinfer.com/docs/next/checker-pulse/) — replaced biabduction for memory/null/leak/taint checking
- IKOS: [NASA GitHub](https://github.com/NASA-SW-VnV/ikos) — 12 abstract-interpretation-based checkers including buffer overflow, null deref, integer overflow
- CWE Top 25 (2025): [MITRE](https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html) — UAF (#7), out-of-bounds write (#5), null deref (#12)
