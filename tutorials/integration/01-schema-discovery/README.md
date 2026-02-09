# Tutorial: Schema Discovery

**Difficulty:** Beginner | **Time:** 10 minutes | **Category:** Integration

## What You Will Learn

- How to discover SAF's full API surface at runtime using `schema()`
- What capabilities, queries, selectors, and graph types SAF exposes
- How an AI agent or tool integrator can use schema introspection for auto-discovery
- How to inspect the module structure using the Rust API

## Prerequisites

Complete the [Getting Started](../../getting-started/README.md) tutorials before starting this one.

## Why Schema Discovery?

When building an AI agent or CI/CD integration that uses SAF, you need to know
what the framework can do. Rather than hardcoding assumptions, you can call
`schema()` at runtime to discover:

- **Tool version** -- which SAF version is installed
- **Schema version** -- API compatibility level
- **Available frontends** -- what input formats are supported
- **Available graphs** -- which program representations can be exported
- **Available queries** -- what analysis queries are supported
- **Available selectors** -- how to define sources, sinks, and sanitizers
- **Configuration options** -- what settings can be tuned

This is especially useful for AI agents that need to adapt their behavior based
on the analysis framework's capabilities.

## The Program

A minimal C program is all we need for schema discovery:

```c
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *cmd = argv[1];
    return system(cmd);
}
```

The program itself is not the focus -- we are inspecting the *framework's*
capabilities, not the program's behavior.

## The Pipeline

```
program.c -> clang-18 -> LLVM IR (.ll) -> Project.open() -> schema()
```

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
SAF Schema Discovery
==================================================

Schema keys: ['tool_version', 'schema_version', 'frontends', 'graphs', 'queries', 'selectors', 'config']

--- tool_version ---
  <version string>

--- graphs ---
  - cfg
  - callgraph
  - defuse
  - valueflow

--- queries ---
  - taint_flow
  - points_to
  ...

Full schema (JSON):
{
  "tool_version": "...",
  ...
}
```

## Understanding the Code

### Python (`detect.py`)

```python
from saf import Project

proj = Project.open(str(llvm_ir))

# Discover all capabilities
schema = proj.schema()
print(f"Schema keys: {list(schema.keys())}")

# Pretty-print as JSON for external tools
import json
print(json.dumps(schema, indent=2, default=str))
```

Key points:

- `proj.schema()` returns a Python dictionary containing everything the
  framework exposes. This dictionary is JSON-serializable.
- The schema includes nested structures: `graphs` lists available graph types,
  `queries` lists available query methods, `selectors` lists available source
  and sink selector types.
- Use `json.dumps()` to serialize the schema for consumption by external tools
  or AI agents.

### Schema Keys

| Key | Type | Description |
|-----|------|-------------|
| `tool_version` | `str` | SAF version string |
| `schema_version` | `str` | API schema version for compatibility checking |
| `frontends` | `list` | Supported input formats (e.g., `["llvm", "air-json"]`) |
| `graphs` | `list` | Available graph types for `graphs().export()` |
| `queries` | `list` | Available query methods (e.g., `taint_flow`, `points_to`) |
| `selectors` | `dict` | Available selector types for sources and sinks |
| `config` | `dict` | Available configuration options and defaults |

## Use Cases for Schema Discovery

1. **AI Agent Auto-Configuration**: An agent reads the schema to determine
   which queries and selectors to use, adapting to the installed version.

2. **CI/CD Pipeline Validation**: Before running scans, check that the
   required graph types and queries are available.

3. **Documentation Generation**: Auto-generate API docs from the schema.

4. **Version Compatibility**: Compare `schema_version` across environments
   to ensure consistent behavior.

## Next Steps

- [Tutorial 02: JSON Export](../02-json-export/README.md) - Export SAF's analysis graphs as JSON
- [Tutorial 03: SARIF Reporting](../03-sarif-reporting/README.md) - Generate standards-compliant vulnerability reports
