# Information Flow 05: IFDS Precision -- Comparing Analysis Algorithms

**Category:** Information Flow Analysis
**Difficulty:** Advanced
**Estimated Time:** 25 minutes

## What You'll Learn

- What the IFDS (Interprocedural Finite Distributive Subset) algorithm is
- How IFDS compares to BFS-based taint analysis in precision and performance
- How to use SAF's `ifds_taint()` API for precise interprocedural taint tracking
- How to inspect IFDS diagnostics: iterations, path edges, and summary edges

## Prerequisites

> Complete [SETUP.md](../../../SETUP.md) before starting this tutorial.
> Familiarity with the basic taint analysis tutorials (01-04) is recommended.

## BFS vs IFDS: Understanding the Tradeoff

Previous tutorials used `q.taint_flow()`, which performs a BFS (breadth-first search)
over the ValueFlow graph. This approach is fast and works well for many cases, but
it has limitations with complex interprocedural flows.

**IFDS** (Interprocedural Finite Distributive Subset) is a more sophisticated algorithm
based on the Reps/Horwitz/Sagiv tabulation method. It provides:

| Feature | BFS `taint_flow()` | IFDS `ifds_taint()` |
|---------|-------------------|---------------------|
| Speed | Fast | Slower (more precise) |
| Interprocedural precision | May over-approximate | Precise call/return matching |
| Summary edges | No | Yes (avoids re-analyzing callees) |
| Tainted-value tracking | Path-based | Fact-based (per-instruction) |

## The Vulnerability

This tutorial uses a program with interprocedural taint flow through a helper function:

```c
// CWE-78: Interprocedural Command Injection

#include <stdlib.h>

// A helper function that passes input through unchanged.
// IFDS tracks taint from the parameter to the return value
// across this function boundary.
char *process_input(char *input) {
    return input;
}

int main() {
    // SOURCE: getenv() returns attacker-influenced data
    char *data = getenv("USER_CMD");

    // Taint flows into process_input() and back out
    char *processed = process_input(data);

    // SINK: system() executes the string as a shell command
    return system(processed);
}
```

The taint flows:
1. `getenv("USER_CMD")` returns tainted data
2. Tainted data enters `process_input()` as a parameter
3. IFDS tracks that the return value is tainted (because `return input;` propagates taint)
4. Tainted return value reaches `system()`

## How IFDS Works

IFDS uses a worklist algorithm that:

1. **Propagates facts** -- At each instruction, IFDS tracks which values are tainted
2. **Builds summary edges** -- When analyzing a function call, IFDS creates summary
   edges that capture how taint flows from parameters to return values
3. **Reuses summaries** -- If the same function is called multiple times, IFDS reuses
   the summary instead of re-analyzing the callee

This makes IFDS precise for interprocedural flows while avoiding exponential blowup.

## Run the Detector

```bash
python3 tutorials-new/information-flow/05-ifds-precision/detect.py
```

Expected output:

```
IFDS taint analysis complete.
  Taint reaches sink: True
  Iterations: <N>
  Path edges explored: <M>
  Summary edges created: <K>
  Tainted values: <count>
    <value descriptions>
  Exported <count> instruction facts

VULNERABILITY DETECTED: Tainted data from getenv() reaches system()
```

## Understanding the Code

```python
#!/usr/bin/env python3
"""Detect interprocedural command injection using IFDS taint analysis."""

import subprocess
from pathlib import Path
from saf import Project, sources, sinks

def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g0", "-fno-discard-value-names",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load via LLVM frontend
    proj = Project.open(str(llvm_ir))

    # Step 3: Run IFDS taint analysis
    #   SOURCE: getenv() return value
    #   SINK:   first argument to system()
    result = proj.ifds_taint(
        sources=sources.call("getenv"),
        sinks=sinks.call("system", arg_index=0),
    )

    # Step 4: Check if taint reaches the sink
    sink_sel = sinks.call("system", arg_index=0)
    taint_found = result.has_taint_at_sink(sink_sel)

    print(f"IFDS taint analysis complete.")
    print(f"  Taint reaches sink: {taint_found}")

    # Step 5: Inspect diagnostics
    diag = result.diagnostics()
    print(f"  Iterations: {diag['iterations']}")
    print(f"  Path edges explored: {diag['path_edges_explored']}")
    print(f"  Summary edges created: {diag['summary_edges_created']}")

    # Step 6: List tainted values
    tainted = result.tainted_values()
    print(f"  Tainted values: {len(tainted)}")
    for v in tainted:
        print(f"    {v}")

    # Step 7: Export full result
    export = result.export()
    print(f"  Exported {len(export['facts'])} instruction facts")

    if taint_found:
        print("\nVULNERABILITY DETECTED: Tainted data from getenv() reaches system()")
    else:
        print("\nNo taint flow detected.")

if __name__ == "__main__":
    main()
```

### Key API Differences from BFS

| BFS API | IFDS API |
|---------|----------|
| `q.taint_flow(sources, sinks)` | `proj.ifds_taint(sources, sinks)` |
| Returns `List[Finding]` | Returns `IFDSResult` object |
| `finding.trace` for path | `result.tainted_values()` for facts |
| No diagnostics | `result.diagnostics()` for stats |

### IFDS-Specific Features

- **`result.has_taint_at_sink(sink_sel)`** -- Checks if taint reached a specific sink
- **`result.diagnostics()`** -- Returns algorithm statistics:
  - `iterations`: Number of worklist iterations
  - `path_edges_explored`: Number of (node, fact) pairs processed
  - `summary_edges_created`: Number of function summaries built
- **`result.tainted_values()`** -- Lists all values that hold tainted data
- **`result.export()`** -- Full fact-level export for debugging

## When to Use IFDS vs BFS

Use **BFS `taint_flow()`** when:
- You need fast results for simple flows
- The program has few interprocedural boundaries
- You want path traces (the specific sequence of edges)

Use **IFDS `ifds_taint()`** when:
- Precision matters more than speed
- Taint flows through multiple function calls/returns
- You need to know which specific values are tainted
- You want to debug with algorithm diagnostics

## Compilation Flags Note

The compilation uses additional flags for IFDS:

```bash
clang-18 -S -emit-llvm -O0 -g0 -fno-discard-value-names ...
```

- `-g0`: No debug info (smaller IR, faster analysis)
- `-fno-discard-value-names`: Preserve SSA value names for readable output

## Next Steps

Continue to [Tutorial 06: Z3 Refinement](../06-z3-refinement/README.md)
to learn how SAF uses Z3 constraint solving to filter false positives
by checking path feasibility.
