# Playground Tour

The SAF Playground is a browser-based interactive environment for writing code,
running analysis, and visualizing program graphs. Everything runs client-side
via WebAssembly -- no server needed.

## Opening the Playground

Navigate to the [Playground](/playground/) in your browser. You will see three
main areas:

1. **Code Editor** (left) -- Write C code or LLVM IR
2. **Graph Viewer** (right) -- Interactive visualization of analysis results
3. **Toolbar** (top) -- Controls for compilation, analysis, and graph selection

## Writing Code

The editor supports two input modes:

- **C/C++**: Write standard C code. The playground compiles it to LLVM IR using
  the Compiler Explorer API, then analyzes the resulting IR.
- **LLVM IR**: Write `.ll` format directly for full control over the IR structure.

### Example: Command Injection

Paste this into the editor:

```c
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *cmd = argv[1];
    return system(cmd);
}
```

## Running Analysis

Click **Analyze** to start the pipeline:

1. C code is compiled to LLVM IR via Compiler Explorer
2. The IR is parsed by tree-sitter-llvm (in WASM)
3. The parsed IR is converted to AIR JSON
4. SAF's WASM module runs the full analysis pipeline
5. Results are displayed as interactive graphs

## Viewing Graphs

Use the graph selector tabs to switch between:

| Graph | What It Shows |
|-------|---------------|
| **CFG** | Control flow within each function -- basic blocks and branches |
| **Call Graph** | Which functions call which -- the program's call hierarchy |
| **Def-Use** | Where values are defined and where they are used |
| **Value Flow** | How data flows through the program, including across functions |
| **Points-To** | What each pointer may point to after analysis |

### Interacting with Graphs

- **Pan**: Click and drag the background
- **Zoom**: Scroll wheel or pinch gesture
- **Select**: Click a node to highlight it and its connections
- **Details**: Selected nodes show their properties in a detail panel

## Built-In Examples

The playground includes pre-loaded examples demonstrating different analysis
scenarios:

- **Command Injection** -- Taint flow from `argv` to `system()`
- **Use-After-Free** -- Pointer used after `free()`
- **Memory Leak** -- Allocation without deallocation
- **Double Free** -- Same pointer freed twice
- **Struct Field Access** -- Field-sensitive pointer analysis

Select an example from the dropdown to load it into the editor.

## Limitations

The browser playground uses a simplified analysis pipeline compared to the
full Docker-based SAF:

| Feature | Browser | Full SAF |
|---------|---------|----------|
| C/C++ compilation | Compiler Explorer API | Local clang-18 |
| LLVM bitcode (`.bc`) | Not supported | Supported |
| Python SDK | Not available | Full access |
| Checker framework | Not available | 9 built-in checkers |
| Z3 path refinement | Not available | Supported |
| SARIF export | Not available | Supported |
| Graph visualization | Built-in | Export to external tools |

See [Browser vs Full SAF](browser-vs-full.md) for a detailed comparison.

## Next Steps

- [Browser vs Full SAF](browser-vs-full.md) -- Understand what each mode offers
- [First Analysis](first-analysis.md) -- Set up the full local environment
- [Concepts](../concepts/air.md) -- Learn about the underlying analysis
