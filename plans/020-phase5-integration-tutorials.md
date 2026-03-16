# Plan 020: Phase 5 — Integration Tutorials

**Parent:** Plan 015 (Tutorial Reorganization)
**Epic:** E8
**Prerequisite:** Plan 016 (directories restructured)

## Goal

Create 5 tutorials in `tutorials/integration/` that teach tool integrators and AI agents how to use SAF programmatically: schema discovery, JSON export, SARIF reporting, batch scanning, and large-codebase stress testing.

**Audience:** Tool integrators, AI agents, and CI/CD pipeline builders.

## Tutorials

### integration/01-schema-discovery

**Source:** Reuses a simple ~20 line C program (e.g., command injection).

**detect.py:**
- `proj = Project.open(str(ll_path))`
- `schema = proj.schema()`
- Print each schema key and its value
- Show how an AI agent could use this to auto-discover available queries, selectors, and graph types without hardcoded knowledge

**Teaches:** Programmatic API introspection. The schema dict describes SAF's full capabilities at runtime.

### integration/02-json-export-pipeline

**Source:** ~40 lines C program with multiple functions and pointer usage.

**detect.py:**
- `graphs = proj.graphs()`
- `for name in graphs.available():` export each graph to a JSON file
- Optionally: load exported callgraph into Python `json` module, compute basic metrics (node count, edge count, max fan-out)
- Show the JSON structure for each graph type

**Teaches:** Graph export workflow. How to feed SAF output into external tools (jq, networkx, custom scripts).

### integration/03-sarif-reporting

**Source:** ~30 lines C with known CWE-78 (command injection).

**detect.py:**
- Run `taint_flow()` to find the vulnerability
- Build SARIF 2.1.0 envelope from `Finding.to_dict()`:
  ```python
  sarif = {
      "version": "2.1.0",
      "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
      "runs": [{
          "tool": { "driver": { "name": "SAF", "version": "0.1.0" } },
          "results": [finding.to_dict() for finding in findings]
      }]
  }
  ```
- Write to `report.sarif.json`

**Teaches:** Standards-compliant SARIF output for GitHub Code Scanning, VS Code SARIF Viewer, DefectDojo, etc.

### integration/04-batch-scanning

**Source:** 3 small programs (~20 lines each):
- `injection.c` — C command injection (argv → system)
- `dangling.cpp` — C++ dangling pointer (return reference to local)
- `unsafe_ffi.rs` — Rust unsafe FFI (env_args → libc::system)

**detect.py:**
- Compile each source to LLVM IR (clang-18 for C/C++, rustc for Rust)
- Scan in a loop: `Project.open()` → `query().taint_flow()` → collect findings
- Aggregate findings across all programs
- Print summary: programs scanned, total findings, findings per language

**Teaches:** Multi-language batch scanning workflow. How to build a simple scanning pipeline.

### integration/05-large-c-codebase

**Source:** ~800+ lines C, simplified HTTP request handler:
- `http_parse_request()` — parses method, path, headers, body from raw buffer
- `router_dispatch()` — matches path to handler function
- `auth_check_token()` — validates auth header
- `db_query()` — constructs SQL from request parameters (SQLi sink)
- `render_response()` — builds HTTP response with user data (XSS sink)
- `log_access()` — writes request info to log file (log injection sink)
- ~15 functions, multiple taint sources (buffer input, headers, query params)
- Multiple vulnerability classes: SQLi, XSS, log injection

**detect.py:**
- Full pipeline scan with multiple source/sink pairs
- Find SQLi: `sources → sinks.call("db_query", ...)`
- Find log injection: `sources → sinks.call("log_access", ...)`
- Export SARIF report
- Print summary statistics (functions analyzed, findings by category)

**Teaches:** Real-ish codebase stress test for the full SAF pipeline. Can SAF handle ~15 functions with multiple vulnerability classes?

**Risk:** Largest program in the tutorial set. May expose performance or analysis issues. Document limitations if found.

## Implementation Order

Implement sequentially: 01 → 02 → 03 → 04 → 05. Each is a safe checkpoint. Update PROGRESS.md task checklist after each one.

Tutorials 01-03 are low risk. Tutorial 04 is medium risk (multi-language). Tutorial 05 is high risk (large program).

## Each Tutorial Contains

- Source file(s) (`program.c`, `program.cpp`, `program.rs`, or multiple files)
- `detect.py` — Python exploration/scanning script
- `detect.rs` — Rust equivalent
- `README.md` — Concept explanation, code walkthrough, output interpretation

## Verification

For each tutorial: `python detect.py` (in Docker) produces expected output matching the README description.

## On Completion

Update `PROGRESS.md`:
- Set plan 020 status to `done`
- Update task checklist: T12, T13, T14, T15, T16 → `done` (update individually)
- Update "Next Steps" to point to plan 021
- Update `tutorials/integration/README.md` from placeholder to real overview
- If bugs found, document in `FUTURE.md`
