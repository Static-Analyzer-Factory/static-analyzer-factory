# Introduction

**SAF** (Static Analyzer Factory) is a Rust + WebAssembly framework for building
program analysis tools. It provides whole-program pointer analysis, value-flow
reasoning, and graph-based program representations -- all accessible through a
Python SDK or directly in the browser via WebAssembly.

## Who Is SAF For?

SAF is designed for three audiences:

- **Students learning program analysis.** The interactive
  [Playground](/playground/) lets you paste C code, visualize control-flow
  graphs, call graphs, and value-flow graphs, and step through pointer analysis
  -- all without installing anything.

- **Security researchers writing custom checkers.** The Python SDK exposes a
  schema-driven API for authoring analyzers that detect use-after-free, double
  free, taint-flow violations, and other vulnerability classes. You write the
  checker logic; SAF handles the underlying analysis infrastructure.

- **AI and agent developers.** The same schema-driven API that humans use is
  designed for LLM agents to consume. Agents can query points-to sets, traverse
  value-flow edges, and emit SARIF findings programmatically.

## What Can You Do With SAF?

| Capability | Description |
|---|---|
| **Visualize program graphs** | Render CFGs, ICFGs, call graphs, def-use chains, and value-flow graphs as interactive diagrams |
| **Run pointer analysis** | Andersen-style inclusion-based analysis with field sensitivity and on-the-fly call graph construction |
| **Detect vulnerabilities** | Built-in checkers for use-after-free, double free, memory leaks, null dereference, file descriptor leaks, and more |
| **Write custom analyzers** | Author new checkers in Python using the SDK, with full access to points-to and value-flow results |
| **Embed analysis widgets** | Embed interactive analysis `<iframe>` widgets into any web page for live program visualization |
| **Export results** | Output findings as SARIF, and graphs as the unified PropertyGraph JSON format |

## Key Differentiators

- **Runs in the browser.** The core analysis engine compiles to WebAssembly. The
  playground and embeddable widgets require no server -- everything executes
  client-side.

- **Deterministic.** Given identical inputs, SAF produces byte-identical outputs.
  All internal data structures use ordered collections, and all IDs are derived
  from BLAKE3 hashes. This makes results reproducible and diffable.

- **Frontend-agnostic.** SAF operates on its own intermediate representation
  (AIR), not on LLVM IR directly. Frontends translate source languages into AIR;
  the analysis core never sees frontend-specific types.

- **Open source.** SAF is open source and designed for extensibility. The crate
  architecture cleanly separates the core IR, frontends, analysis passes, and
  language bindings.

## Quick Start

The fastest way to try SAF:

1. **In the browser:** Open the [Playground](/playground/) and paste a C snippet.
   Click "Analyze" to see the CFG, call graph, and points-to results.

2. **Locally with Docker:**

   ```bash
   git clone https://github.com/ThePatrickStar/static-analyzer-factory.git
   cd static-analyzer-factory
   make shell
   # Inside the container:
   python3 tutorials/memory-safety/02-use-after-free/detect.py
   ```

3. **With the CLI:**

   ```bash
   saf run program.ll --format sarif --output results.sarif
   ```

## Where to Go Next

- **[Installation](getting-started/installation.md)** -- Set up SAF locally with
  Docker or install the Python package.
- **[First Analysis](getting-started/first-analysis.md)** -- Walk through
  analyzing a small C program end to end.
- **[Playground Tour](getting-started/playground-tour.md)** -- Learn how to use
  the browser-based playground.
- **[Tutorials](/tutorials/)** -- Step-by-step guides for
  detecting specific vulnerability classes.
- **[API Reference](api-reference/python-sdk.md)** -- Full reference for the
  Python SDK, CLI, and PropertyGraph format.

## Architecture Overview

```
Input (.c / .ll / .bc / .air.json)
  -> Frontend (LLVM, AIR-JSON)
    -> AirBundle (canonical IR)
      -> Graph builders (CFG, ICFG, CallGraph, DefUse)
        -> PTA solver (Andersen)
          -> ValueFlow graph
            -> Queries (flows, taint_flow, points_to)
              -> Export (JSON, SARIF)
```

SAF is structured as a set of Rust crates:

| Crate | Purpose |
|---|---|
| `saf-core` | AIR definitions, config, deterministic ID generation |
| `saf-frontends` | Frontend trait + LLVM/AIR-JSON implementations |
| `saf-analysis` | CFG, call graph, PTA, value-flow, checkers |
| `saf-cli` | Command-line interface |
| `saf-python` | PyO3 bindings for the Python SDK |
