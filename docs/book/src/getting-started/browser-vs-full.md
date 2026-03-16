# Browser vs Full SAF

SAF is available in two forms: the browser-based playground (WebAssembly) and the
full local installation (Docker). This page compares the two.

## Feature Comparison

| Feature | Browser (WASM) | Full SAF (Docker) |
|---------|---------------|-------------------|
| **Setup** | None -- open a URL | Docker + `make shell` |
| **LLVM frontend** | No (tree-sitter) | Yes (inkwell, full LLVM API) |
| **Z3 solver** | No | Yes |
| **Context-sensitive PTA** | Limited | Yes |
| **Large programs (>1000 LOC)** | Slow | Fast |
| **Python SDK** | Pyodide (limited) | Full |
| **Custom specs** | Yes | Yes |
| **Checker framework** | 9 built-in checkers (via query protocol) | 9 built-in checkers + custom checkers |
| **SARIF export** | Not available | Standards-compliant output via CLI |
| **Graph visualization** | Built-in Cytoscape.js | Export JSON for external tools |
| **Batch scanning** | Single file | Multi-file projects |
| **Offline use** | Requires Compiler Explorer API for C | Fully offline |

## When to Use the Browser

The browser playground is ideal for:

- **Learning**: Experiment with program analysis concepts interactively
- **Quick checks**: Paste a snippet and see the CFG or call graph immediately
- **Demonstrations**: Show analysis capabilities without requiring setup
- **Embedding**: Include analysis widgets in documentation or blog posts

## When to Use Full SAF

The full installation is needed for:

- **Production analysis**: Scanning real codebases for vulnerabilities
- **Custom checkers**: Writing Python scripts that use the SDK
- **CI/CD integration**: Automated scanning with SARIF output
- **Advanced analysis**: Z3-based path refinement, IFDS taint tracking
- **Bitcode input**: Analyzing pre-compiled `.bc` files
- **Batch scanning**: Processing multiple files or entire projects

## Architecture Differences

### Browser

```
C code --> Compiler Explorer API --> LLVM IR (.ll text)
  --> tree-sitter-llvm (WASM) --> CST
    --> TypeScript converter --> AIR JSON
      --> saf-wasm (WASM) --> PropertyGraph JSON
        --> Cytoscape.js visualization
```

The browser path uses tree-sitter to parse LLVM IR text and a TypeScript layer
to convert the parse tree into AIR JSON. The `saf-wasm` crate contains
`saf-core` and `saf-analysis` compiled to WebAssembly, without LLVM, Z3, or
threading dependencies.

### Full SAF

```
C code --> clang-18 --> LLVM IR (.ll or .bc)
  --> inkwell (LLVM C API) --> AIR
    --> saf-analysis (native) --> Analysis results
      --> Python SDK / CLI / JSON / SARIF
```

The full path uses inkwell to parse LLVM IR/bitcode natively with full access to
LLVM's type system and metadata. Analysis runs in native Rust with threading
support.

## Migrating from Browser to Full SAF

If you start with the playground and want to move to the full SDK:

1. Install Docker and clone the repository (see [Installation](installation.md))
2. Save your C code to a `.c` file
3. Write a Python detection script using the same analysis concepts
4. Run inside `make shell`

The analysis concepts (sources, sinks, graph types) are the same in both
environments. The full SDK simply provides more control and additional features.
