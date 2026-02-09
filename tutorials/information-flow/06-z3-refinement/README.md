# Information Flow 06: Z3 Taint Refinement -- Path-Sensitive Analysis

**Category:** Information Flow Analysis
**Difficulty:** Advanced
**Estimated Time:** 30 minutes

## What You'll Learn

- Why path-insensitive analysis produces false positives
- How Z3 SMT solver refinement filters infeasible taint paths
- How to use SAF's `check_all_path_sensitive()` API
- How to interpret path-sensitive analysis diagnostics

## Prerequisites

> Complete [SETUP.md](../../../SETUP.md) before starting this tutorial.
> Familiarity with the basic taint analysis tutorials (01-05) is recommended.

## The Problem: False Positives from Branch Merging

Path-insensitive analysis treats all branches as reachable simultaneously. When
data is sanitized on one branch but not another, the analysis **merges** both
flows and reports the unsanitized data can reach the sink -- even when the
sanitized branch is actually taken.

## The Scenario

An HTTP request handler sanitizes GET parameters but passes POST data directly
to `system()`:

```c
void handle_request(int method) {
    char *user_input = getenv("QUERY_STRING"); /* taint source */
    char cmd[256];
    char safe_buf[256];

    if (method == 0) {
        /* GET: sanitize before use */
        char *safe = sanitize_input(user_input, safe_buf, sizeof(safe_buf));
        snprintf(cmd, sizeof(cmd), "echo %s", safe);
        system(cmd); /* Path-insensitive: reports taint here (FP) */
    } else {
        /* POST: raw input used directly */
        snprintf(cmd, sizeof(cmd), "process %s", user_input);
        system(cmd); /* Genuine taint flow: user_input -> system() */
    }
}
```

### What Path-Insensitive Analysis Sees

```
getenv("QUERY_STRING") --> both branches --> system()
```

It reports **2 findings** because it cannot distinguish the sanitized GET path
from the unsanitized POST path.

### What Z3 Refinement Discovers

Z3 extracts the branch guard `method == 0` and checks:
1. **GET path** (`method == 0`): `sanitize_input()` breaks the taint chain
2. **POST path** (`method != 0`): Raw input reaches `system()`

Result: **1 confirmed bug** (POST), **1 filtered false positive** (GET)

## How Z3 Refinement Works

1. **Find taint flows** -- Run standard path-insensitive taint analysis
2. **Extract path constraints** -- For each finding, collect branch conditions
   along the taint path
3. **Check feasibility** -- Ask Z3 "Is this path satisfiable given the constraints?"
4. **Filter results** -- Mark infeasible paths as false positives

## Run the Detector

```bash
python3 tutorials-new/information-flow/06-z3-refinement/detect.py
```

Expected output:

```
Step 1: Compiling C source to LLVM IR...
  Compiled: vulnerable.c -> vulnerable.ll

Step 2: Loading project...
  <Project info>

============================================================
ANALYSIS: Path-Insensitive Taint (baseline)
============================================================

Total findings: 2
  [high] taint: Tainted data from getenv flows to system (GET path)
  [high] taint: Tainted data from getenv flows to system (POST path)

============================================================
ANALYSIS: Path-Sensitive Taint (Z3-refined)
============================================================

<PathSensitiveResult summary>

Confirmed bugs (feasible): 1
  [high] taint: Tainted data from getenv flows to system (POST path)

False positives (infeasible): 1
  [high] taint: Tainted data from getenv flows to system (GET path)

Uncertain (unknown): 0

============================================================
COMPARISON
============================================================

  Path-insensitive findings: 2
  Path-sensitive confirmed:  1
  False positives filtered:  1

  False positive reduction: 1/2 (50%)

  Z3 solver calls: N
  Z3 timeouts:     0
```

## Understanding the Code

```python
#!/usr/bin/env python3
"""Z3-refined taint analysis: filter infeasible cross-branch taint flows."""

import subprocess
from pathlib import Path
import saf

def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g0",
         "-Xclang", "-disable-O0-optnone",
         "-fno-discard-value-names",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load project
    proj = saf.Project.open(str(llvm_ir))

    # Step 3: Path-insensitive baseline
    pi_findings = proj.check_all()

    # Step 4: Path-sensitive with Z3 refinement
    ps_result = proj.check_all_path_sensitive(
        z3_timeout_ms=2000,  # Timeout per Z3 query (milliseconds)
        max_guards=64,       # Maximum branch guards to track
    )

    # Step 5: Inspect results
    print(f"Confirmed bugs: {len(ps_result.feasible)}")
    print(f"False positives: {len(ps_result.infeasible)}")
    print(f"Uncertain: {len(ps_result.unknown)}")

    # Step 6: Check diagnostics
    diag = ps_result.diagnostics
    print(f"Z3 solver calls: {diag['z3_calls']}")
    print(f"Z3 timeouts: {diag['z3_timeouts']}")
```

### Key API Elements

| Method | Description |
|--------|-------------|
| `proj.check_all()` | Run all checkers (path-insensitive) |
| `proj.check_all_path_sensitive(...)` | Run checkers with Z3 refinement |
| `ps_result.feasible` | Findings confirmed as real bugs |
| `ps_result.infeasible` | Findings filtered as false positives |
| `ps_result.unknown` | Findings where Z3 timed out or was uncertain |
| `ps_result.diagnostics` | Statistics about Z3 usage |

### Configuration Options

- **`z3_timeout_ms`**: Maximum time per Z3 query (default: 2000ms). Longer
  timeouts increase precision but slow analysis.
- **`max_guards`**: Maximum number of branch conditions to track per path.
  Higher values increase precision but use more memory.

## Result Categories

| Category | Meaning | Action |
|----------|---------|--------|
| **Feasible** | Z3 found satisfying assignment | Investigate -- likely a real bug |
| **Infeasible** | Z3 proved path impossible | Safe to ignore -- false positive |
| **Unknown** | Z3 timed out or hit complexity limit | Needs manual review |

## When to Use Z3 Refinement

**Good candidates:**
- Programs with conditional sanitization
- Input validation that depends on flags/modes
- Security checks guarded by configuration

**Poor candidates:**
- Simple programs without complex branching
- Extremely large programs (Z3 may timeout)
- Programs with complex non-linear arithmetic

## Compilation Flags for Z3 Analysis

```bash
clang-18 -S -emit-llvm -O0 -g0 -Xclang -disable-O0-optnone -fno-discard-value-names ...
```

- `-Xclang -disable-O0-optnone`: Allow LLVM to analyze even at -O0
- `-g0`: No debug info (faster)
- `-fno-discard-value-names`: Preserve SSA names for readable output

## Summary

This tutorial demonstrated how Z3 SMT solver refinement can dramatically reduce
false positives in taint analysis by checking path feasibility. The technique
is particularly powerful for programs with conditional sanitization or
mode-dependent security checks.

| Analysis Type | Findings | False Positives |
|--------------|----------|-----------------|
| Path-insensitive | 2 | 1 (50%) |
| Z3-refined | 1 | 0 (0%) |

## Next Steps

Explore the other tutorial categories:
- [Memory Safety](../../memory-safety/README.md) -- Use-after-free, buffer overflow detection
- [Graph Exploration](../../graphs/README.md) -- Understanding CFG, call graph, ValueFlow
- [Custom Checkers](../../checkers/README.md) -- Writing your own security analyzers
