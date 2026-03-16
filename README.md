# SAF — Static Analyzer Factory

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Python 3.12+](https://img.shields.io/badge/Python-3.12+-green.svg)](https://www.python.org)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org)

A Rust-powered static analysis framework with a Python SDK for finding bugs in C/C++ programs. SAF turns LLVM IR into analyzable graphs — pointer analysis, value-flow, taint tracking — and exposes them through a clean Python API and CLI.

## Key Features

- **Pointer analysis** — Andersen-style with field sensitivity, context-sensitive (k-CFA), and flow-sensitive variants
- **Value-flow graphs** — SSA + memory + interprocedural edges for precise data-flow tracking
- **Taint analysis** — source/sink/sanitizer framework with trace extraction
- **IFDS solver** — interprocedural, finite, distributive subset analysis
- **Built-in checkers** — memory leaks, null dereference, double-free, use-after-free, and more
- **Python SDK** — first-class API for scripting custom analyses
- **CLI** — full analysis pipeline from the command line
- **Deterministic** — identical inputs always produce byte-identical outputs
- **SARIF export** — standard format for IDE and CI integration

## Quick Start

SAF runs inside Docker (LLVM 18 + all dependencies included).

```bash
git clone https://github.com/ThePatrickStar/static-analyzer-factory.git
cd static-analyzer-factory
make shell
```

### Python SDK

```python
from saf import Project, sources, sinks

proj = Project.open("program.ll")
q = proj.query()

# Find taint flows from user input to dangerous sinks
findings = q.taint_flow(
    sources=sources.function_param("main", 1),   # argv
    sinks=sinks.call("system", arg_index=0),
)

for f in findings:
    print(f"{f.severity}: {f.message}")
    print(f"  {f.source_location} -> {f.sink_location}")
```

### CLI

```bash
# Run all built-in checkers
saf run program.ll --checkers all --format json --output findings.json

# Export call graph as DOT
saf export callgraph --input program.ll --format dot --output cg.dot

# Query points-to set for a specific value
saf query points-to 0x00000042 --input program.ll
```

## Architecture

```
crates/
  saf-core/       # AIR (Analysis IR), config, deterministic IDs
  saf-frontends/  # LLVM bitcode + AIR-JSON frontends
  saf-analysis/   # CFG, call graph, PTA, value-flow, checkers
  saf-cli/        # Command-line interface
  saf-python/     # Python SDK (PyO3 bindings)
  saf-wasm/       # Browser build (playground)
```

**Data flow:**

```
Input (.ll / .bc)
  → Frontend → AIR (canonical IR)
    → Graph builders (CFG, call graph, def-use)
      → Pointer analysis → Value-flow graph
        → Queries & checkers → Findings (JSON / SARIF)
```

## Known Limitations

**Analysis precision:**
- Default pointer analysis is context-insensitive; context-sensitive (k-CFA), flow-sensitive, and demand-driven variants are available but may be slower on large programs
- Array elements are treated as a single abstract object — no per-index tracking

**Indirect calls:**
- Indirect call resolution depends on PTA precision — targets that PTA misses are invisible to downstream analyses (ICFG, IFDS, taint)
- When a call site resolves to multiple targets, only the first is used in value-flow and SVFG

**Not yet supported:**
- Source-level frontends (Clang AST, rust-analyzer) — architecture is ready, implementation is planned
- Symbolic execution

## Documentation

- [**Docs**](https://thepatrickstar.github.io/static-analyzer-factory/docs/) — concepts, API reference, getting started
- [**Tutorials**](https://thepatrickstar.github.io/static-analyzer-factory/tutorials/) — step-by-step guides from hello-taint to custom checkers
- [**Playground**](https://thepatrickstar.github.io/static-analyzer-factory/playground/) — try SAF in the browser (WASM build)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding conventions, and PR guidelines.

## License

[MIT](LICENSE)
