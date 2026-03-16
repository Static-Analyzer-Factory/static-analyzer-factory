# Plan 006: Python SDK v1

**Epic:** E6
**Status:** approved
**Created:** 2026-01-29

## Overview

Implement the Python SDK exposing SAF's static analysis capabilities through a layered API designed for both AI agents and human developers.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| API structure | Layered (high + mid-level) | Simple default, power when needed |
| Error handling | Exception hierarchy + .code/.details | Pythonic catching + agent-friendly structured access |
| Selector API | Module-level functions + \| operator | Simple, type-safe, no DSL cognitive burden |
| Schema format | Flat dictionary with examples | Easy to parse, no JSON Schema complexity |
| Phasing | End-to-end thin slice first | TDD alignment, de-risks integration early |
| Testing | Acceptance primary + binding + property | SRS examples as tests, Hypothesis for invariants |

## Public API

### Module Structure

```
saf/
  __init__.py          # Re-exports: Project, Query, Finding, Trace, Config, version()
  sources.py           # Source selectors
  sinks.py             # Sink selectors
  sanitizers.py        # Sanitizer selectors
  exceptions.py        # SafError hierarchy
```

### Core Classes

```python
class Project:
    @staticmethod
    def open(path: str, *, cache_dir: str | None = None,
             config: Config | None = None, frontend: str = "llvm") -> Project
    def schema(self) -> dict
    def query(self) -> Query
    # Mid-level access
    def air(self) -> AirModule
    def graphs(self) -> GraphStore
    def pta_result(self) -> PtaResult | None

class Query:
    def points_to(self, ptr: str) -> list[str]
    def may_alias(self, p: str, q: str) -> bool
    def flows(self, sources: SelectorSet, sinks: SelectorSet, *, limit: int = 1000) -> list[Finding]
    def taint_flow(self, sources: SelectorSet, sinks: SelectorSet,
                   sanitizers: SelectorSet | None = None, *, limit: int = 1000) -> list[Finding]
    def export_graph(self, name: str) -> dict

class Finding:
    finding_id: str
    source_location: str
    sink_location: str
    trace: Trace
    def to_sarif(self, path: str) -> None

class Trace:
    steps: list[TraceStep]
    def pretty(self) -> str
```

### Exception Hierarchy

```python
class SafError(Exception):
    code: str       # "FRONTEND_PARSE_ERROR"
    message: str
    details: dict   # {"path": "...", "line": 42}

class FrontendError(SafError): ...   # FRONTEND_NOT_FOUND, FRONTEND_PARSE_ERROR
class AnalysisError(SafError): ...   # PTA_TIMEOUT, VALUEFLOW_BUILD_ERROR
class QueryError(SafError): ...      # INVALID_SELECTOR, SELECTOR_NO_MATCH
class ConfigError(SafError): ...     # CONFIG_INVALID_FIELD
```

### Selector API

```python
# sources.py
def argv() -> Selector
def getenv(name: str | None = None) -> Selector
def call(callee: str) -> Selector
def function_param(function: str, index: int | None = None) -> Selector
def function_return(function: str) -> Selector

# sinks.py / sanitizers.py
def call(callee: str, *, arg_index: int | None = None) -> Selector
def arg_to(callee: str, index: int) -> Selector

# Combination via | operator
combined = sources.argv() | sources.getenv()  # -> SelectorSet
```

### Schema Format

```python
proj.schema() -> {
    "tool_version": "0.1.0",
    "schema_version": 1,
    "frontends": {"llvm": {...}, "air-json": {...}},
    "graphs": {"cfg": {...}, "valueflow": {...}, ...},
    "queries": {
        "taint_flow": {
            "description": "...",
            "parameters": {...},
            "example": "..."
        }, ...
    },
    "selectors": {
        "sources": {"argv": {"description": "...", "example": "..."}, ...},
        "sinks": {...},
        "sanitizers": {...}
    },
    "config": {
        "frontend": {"type": "str", "default": "llvm", "choices": [...]},
        ...
    }
}
```

## Implementation Phases

### Phase 1: End-to-end thin slice (AIR-JSON only)

**Files:**
- `crates/saf-python/src/lib.rs` - module registration
- `crates/saf-python/src/project.rs` - Project.open() with AIR-JSON
- `crates/saf-python/src/query.rs` - Query.taint_flow()
- `crates/saf-python/src/finding.rs` - Finding, Trace classes
- `crates/saf-python/src/selector.rs` - Selector, SelectorSet (basic)
- `crates/saf-python/src/exceptions.rs` - SafError base
- `python/saf/__init__.py` - re-exports
- `python/saf/sources.py` - function_param() only
- `python/saf/sinks.py` - call() only
- `python/tests/fixtures/taint_simple.air.json`
- `python/tests/test_acceptance.py::test_simple_taint_flow`

**Acceptance criteria:**
- [ ] `Project.open("fixture.air.json")` loads and analyzes
- [ ] `query.taint_flow(sources.function_param(...), sinks.call(...))` returns findings
- [ ] Finding has .source_location, .sink_location, .trace
- [ ] Basic SafError on invalid input

### Phase 2: Selector expansion & schema

**Files:**
- `python/saf/sources.py` - all selectors (argv, getenv, call, function_return)
- `python/saf/sinks.py` - all selectors (call, arg_to)
- `python/saf/sanitizers.py` - all selectors
- `crates/saf-python/src/selector.rs` - SelectorSet with __or__
- `crates/saf-python/src/schema.rs` - schema() builder
- `crates/saf-python/src/exceptions.rs` - full hierarchy
- `python/saf/exceptions.py` - re-exports
- `python/tests/test_acceptance.py::test_combined_selectors`
- `python/tests/test_acceptance.py::test_sanitizer_blocks_flow`
- `python/tests/test_acceptance.py::test_schema_completeness`
- `python/tests/fixtures/taint_sanitizer.air.json`

**Acceptance criteria:**
- [ ] `sources.argv() | sources.getenv()` returns SelectorSet
- [ ] Sanitizers block taint propagation
- [ ] `proj.schema()` returns complete dict
- [ ] FrontendError, AnalysisError, QueryError all have .code/.details

### Phase 3: PTA queries & mid-level access

**Files:**
- `crates/saf-python/src/query.rs` - points_to(), may_alias(), export_graph()
- `crates/saf-python/src/air.rs` - AirModule wrapper
- `crates/saf-python/src/graphs.rs` - GraphStore wrapper
- `crates/saf-python/src/pta.rs` - PtaResult wrapper
- `crates/saf-python/src/project.rs` - air(), graphs(), pta_result()
- `python/tests/test_acceptance.py::test_points_to_query`
- `python/tests/test_acceptance.py::test_may_alias`
- `python/tests/test_acceptance.py::test_graph_export`
- `python/tests/test_acceptance.py::test_mid_level_access`

**Acceptance criteria:**
- [ ] `query.points_to(ptr_id)` returns list of location IDs
- [ ] `query.may_alias(p, q)` returns bool
- [ ] `query.export_graph("cfg")` returns dict
- [ ] `proj.air()` returns AirModule with functions, globals
- [ ] `proj.graphs()` returns GraphStore

### Phase 4: LLVM frontend & SRS example

**Files:**
- `crates/saf-python/src/project.rs` - LLVM frontend integration
- `crates/saf-python/src/finding.rs` - to_sarif()
- `python/tests/fixtures/` - LLVM test fixtures (.c + .bc)
- `python/tests/test_acceptance.py::test_srs_appendix_a_example`
- `python/tests/test_acceptance.py::test_sarif_export`
- `python/tests/test_properties.py` - Hypothesis tests
- `python/tests/test_bindings.py` - critical binding tests

**Acceptance criteria:**
- [ ] SRS Appendix A example works exactly as written
- [ ] `finding.to_sarif("out.sarif")` produces valid SARIF
- [ ] Property test: determinism (same inputs -> same outputs)
- [ ] Property test: no crashes with random valid selectors
- [ ] All tests pass with `make test`

## Test Structure

```
python/tests/
  __init__.py
  conftest.py              # Shared fixtures
  test_acceptance.py       # SRS-driven tests (primary)
  test_bindings.py         # PyO3 binding tests (selective)
  test_properties.py       # Hypothesis invariant tests
  fixtures/
    taint_simple.air.json
    taint_sanitizer.air.json
    multi_path.air.json
    srs_example/           # LLVM fixtures for Phase 4
```

## PyO3 Crate Structure

```
crates/saf-python/src/
  lib.rs              # Module registration
  project.rs          # Project class
  query.rs            # Query class
  finding.rs          # Finding, Trace classes
  selector.rs         # Selector, SelectorSet classes
  config.rs           # Config wrapper
  exceptions.rs       # Exception hierarchy
  schema.rs           # schema() dict builder
  air.rs              # AirModule wrapper
  graphs.rs           # GraphStore wrapper
  pta.rs              # PtaResult wrapper
```

## Out of Scope

- CLI binary (`saf` command) - separate epic
- Caching (FR-CACHE-*) - separate epic
- Low-level API layer - not needed yet

## Dependencies

**Rust:** pyo3 0.22, saf-core, saf-frontends, saf-analysis (already in workspace)

**Python:**
- Runtime: none (pure native extension)
- Dev: pytest>=8.0, hypothesis>=6.100

**Tooling:** maturin>=1.5, uv for Python package management

## Acceptance Test (North Star)

```python
# SRS Appendix A - must work at end of E6
from saf import Project, sources, sinks, sanitizers

proj = Project.open("app.bc", cache_dir=".cache/saf", frontend="llvm")
q = proj.query()

findings = q.taint_flow(
    sources=sources.argv() | sources.getenv(),
    sinks=sinks.call(r"(printf|sprintf|snprintf)", arg_index=0),
    sanitizers=sanitizers.call(r"(sanitize|escape)", arg_index=0),
)

for f in findings:
    print(f.rule_id, f.title)
    print(f.source_location, "->", f.sink_location)
    print(f.trace.pretty())
    f.to_sarif("out.sarif")
```
