# Plan 163: Align Pyodide Bridge API with Python SDK

## Context

The playground's Pyodide bridge (`playground/src/analysis/pyodide-bridge.ts`) provides a `saf` Python module with a completely different API from the real Python SDK (`crates/saf-python/`). Users who learn the API in the playground can't transfer knowledge to the native SDK. The goal: make the Pyodide bridge feel like a subset of the SDK — same class names, method signatures, and workflow — so users perceive it as "the SDK running in the browser, minus heavy features."

## Two-Part Implementation

### Part 1: Implement Rust JSON Protocol Queries (WASM backend)

Wire up the stubbed `handle_builtin_query` in `database/handler.rs` to support `taint_flow`, `flows`, `points_to`, and `alias` queries. This gives the Pyodide bridge a proper backend.

**File: `crates/saf-analysis/src/database/handler.rs`**

Change `handle_builtin_query` from returning NOT_IMPLEMENTED to actually running queries:

```
"points_to" → parse "pointer" param as hex ValueId → call self.points_to(vid) → return locations as results
"alias" → parse "p" and "q" params → call self.may_alias(p, q) → return {may_alias: bool}
"taint_flow" → parse "sources", "sinks", "sanitizers" as Selector JSON arrays → resolve_selectors() → self.valueflow.taint_flow() → convert Flows to protocol Findings
"flows" → same as taint_flow without sanitizers
```

Key existing infrastructure to reuse:
- `Selector` is `#[derive(Deserialize)]` with `#[serde(tag = "kind")]` — can be parsed directly from JSON params
- `resolve_selectors(&[Selector], &AirModule)` at `crates/saf-analysis/src/selector/mod.rs:142`
- `ValueFlowGraph::taint_flow()` and `flows()` at `crates/saf-analysis/src/valueflow/query.rs:60-82`
- `ProgramDatabase` already owns `module`, `valueflow`, `pta_result` — all needed for these queries
- `Finding::from_flow()` converts `Flow` → `Finding` (used in `query.rs`)

The `handle_builtin_query` method needs to become `&self` (currently static) so it can access `self.module`, `self.valueflow`, etc.

**File: `crates/saf-analysis/src/database/protocol.rs`**

Add a `Response::ok_results()` constructor for query results (points_to, alias return structured data, not findings).

### Part 2: Rewrite Pyodide Bridge Python Module (frontend)

**File: `playground/src/analysis/pyodide-bridge.ts`**

#### A. Extend `_saf_bridge` JS module

Add `query(request_json)` function that delegates to `runQuery()` from `@saf/web-shared/analysis`.

#### B. Rewrite `SAF_PYTHON_MODULE` with SDK-compatible classes

**New classes (mirroring Python SDK):**

| Pyodide Class | SDK Equivalent | Implementation |
|---|---|---|
| `Project` | `saf-python/project.rs` | Entry point; wraps analysis results + bridge calls |
| `Query` | `saf-python/query.rs` | `taint_flow()`, `flows()`, `points_to()`, `may_alias()` via JSON protocol |
| `Selector` / `SelectorSet` | `saf-python/selector.rs` | Stores kind+params; serialized as JSON for protocol calls |
| `Finding` | `saf-python/finding.rs` | `finding_id`, `source_location`, `sink_location`, `trace` |
| `Trace` / `TraceStep` | `saf-python/finding.rs` | `steps`, `pretty()` |
| `GraphStore` | `saf-python/graphs.rs` | `available()`, `export(name, function=None)` |
| `PtaResult` | `saf-python/pta.rs` | `points_to()`, `may_alias()` |

**New module-level factory functions (matching SDK `lib.rs`):**

```python
saf.function_param("main", 0)   → Selector(kind="function_param", function="main", index=0)
saf.function_return("malloc")    → Selector(kind="function_return", function="malloc")
saf.call("gets")                 → Selector(kind="call_to", callee="gets")
saf.arg_to("strcpy", 0)         → Selector(kind="arg_to", callee="strcpy", index=0)
```

**SDK-compatible workflow:**
```python
import saf
proj = saf.Project()                        # no path arg in browser
q = proj.query()
findings = q.taint_flow(
    saf.call("gets"),
    saf.arg_to("strcpy", 0)
)
for f in findings:
    print(f.trace.pretty())
    saf.report(f.sink_id, "high", f"Taint flow: {f.source_location} → {f.sink_location}")
```

**Query implementation path:**
`q.taint_flow(sources, sinks)` → serialize selectors to JSON → `_saf_bridge.query('{"action":"query","type":"taint_flow","params":{"sources":[...],"sinks":[...]}}')` → WASM `query()` → `ProgramDatabase::handle_request()` → resolve selectors → BFS on ValueFlowGraph → JSON response → parse into `Finding` objects

**Backward compatibility:**
- `saf.analyze()` still works, returns `AnalysisResult` (same PropertyGraph wrappers)
- `saf.report()` and `saf.source_line()` remain as playground-specific helpers
- All 13 existing templates continue working unchanged

**Unsupported SDK features** (raise `NotImplementedError("Requires native SAF SDK")`):
- `Project.open(path)` — no file I/O in browser
- `proj.ifds_taint()`, `proj.typestate()` — no IFDS/IDE in WASM
- `proj.check()`, `proj.check_all()` — SVFG checkers available but already accessible via JSON protocol's `check`/`check_all` actions, could wire up later
- `proj.flow_sensitive_pta()`, `proj.context_sensitive_pta()`, `proj.demand_pta()` — not in WASM
- All Z3 methods — not in WASM
- `proj.abstract_interp()`, `proj.analyze_combined()` — not in WASM

#### C. Update templates and completions

**File: `playground/src/components/AnalyzerPanel.tsx`**
- Update "custom" template docstring to show new `Project`/`Query`/`Selector` API
- Update 1-2 existing templates (e.g., `detect_uaf`) to use new API as examples

**File: `playground/src/editor/saf-completions.ts`**
- Add completions for `Project`, `Query`, `GraphStore`, selectors, `Finding`, `Trace`

## Files to Modify

| File | Change |
|---|---|
| `crates/saf-analysis/src/database/handler.rs` | Implement `points_to`, `alias`, `taint_flow`, `flows` query handlers |
| `crates/saf-analysis/src/database/protocol.rs` | Add `Response::ok_results()` constructor |
| `playground/src/analysis/pyodide-bridge.ts` | Add `query` to JS bridge; rewrite Python module with SDK classes |
| `playground/src/components/AnalyzerPanel.tsx` | Update custom template + 1-2 examples |
| `playground/src/editor/saf-completions.ts` | Add completions for new API surface |

## Implementation Order

1. **Rust: `handler.rs`** — Implement the 4 query handlers (points_to, alias, taint_flow, flows)
2. **Rust: `protocol.rs`** — Add `ok_results` response helper
3. **WASM rebuild** — `make wasm-dev` to get updated query support
4. **TS: `pyodide-bridge.ts`** — Add `query` JS bridge function + rewrite Python module
5. **TS: `AnalyzerPanel.tsx`** — Update templates
6. **TS: `saf-completions.ts`** — Update autocomplete

## Verification

1. **Rust tests**: `make test` — existing handler tests pass + new query tests
2. **WASM build**: `make wasm-dev` succeeds
3. **Playground smoke test**: `npm run dev` in playground, run existing templates — all still work
4. **New API test in custom template**:
   ```python
   proj = saf.Project()
   q = proj.query()
   # points_to
   for entry in proj.pta_result().all_entries():
       pts = q.points_to(entry)
       print(f"{entry} -> {pts}")
   # taint_flow with selectors
   findings = q.taint_flow(saf.call("gets"), saf.arg_to("printf", 0))
   for f in findings:
       print(f.trace.pretty())
       saf.report(f.sink_id, "high", f.source_location + " → " + f.sink_location)
   ```
5. **Backward compat**: `result = saf.analyze()` still works with PropertyGraph wrappers
