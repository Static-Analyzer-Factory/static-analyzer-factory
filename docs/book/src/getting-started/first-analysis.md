# First Analysis

This guide walks you through analyzing a small C program from start to finish
using the SAF Python SDK.

## The Vulnerable Program

We will detect a command injection vulnerability (CWE-78) -- user-controlled
input flowing directly to a shell command:

```c
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;

    // SOURCE: argv[1] is user-controlled
    char *user_cmd = argv[1];

    // SINK: system() executes the string as a shell command
    return system(user_cmd);
}
```

If an attacker runs `./program "rm -rf /"`, the program executes that command.

## The Analysis Pipeline

```
vulnerable.c --> clang-18 --> LLVM IR (.ll) --> SAF --> Findings
                 (compile)    (.ll file)        (analyze)
```

SAF works on LLVM IR, not source code directly. The pipeline compiles C to LLVM
IR first, then runs analysis on the IR.

## Step 1: Write the Detection Script

Create a file called `detect.py`:

```python
import subprocess
from pathlib import Path
from saf import Project, sources, sinks

def main() -> None:
    # 1. Compile C to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", "vulnerable.ll", "vulnerable.c"],
        check=True,
    )
    print("Compiled: vulnerable.c -> vulnerable.ll")

    # 2. Load the LLVM IR into SAF
    proj = Project.open("vulnerable.ll")
    print("Project loaded")

    # 3. Create a query context
    q = proj.query()

    # 4. Run taint flow analysis
    findings = q.taint_flow(
        sources=sources.function_param("main", 1),  # argv
        sinks=sinks.call("system", arg_index=0),     # system()'s argument
    )

    # 5. Print results
    print(f"\nFound {len(findings)} taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"       trace steps: {len(f.trace.steps)}")

if __name__ == "__main__":
    main()
```

## Step 2: Run Inside Docker

```bash
make shell
python3 detect.py
```

Expected output:

```
Compiled: vulnerable.c -> vulnerable.ll
Project loaded

Found 1 taint flow(s):
  [0] finding_id=0x...
       trace steps: 3
```

SAF found one taint flow: data from `argv[1]` reaches `system()`.

## Understanding the Key Concepts

### Sources and Sinks

- **Source**: Where untrusted data enters. `sources.function_param("main", 1)`
  selects the second parameter of `main` (which is `argv`).
- **Sink**: Where untrusted data is dangerous. `sinks.call("system", arg_index=0)`
  selects the first argument to any call to `system()`.

### Taint Flow

`q.taint_flow(sources, sinks)` performs a breadth-first search over the
ValueFlow graph, looking for paths from any source to any sink. Each path found
is reported as a `Finding` with:

- `finding_id` -- a deterministic identifier
- `trace` -- the step-by-step data flow path

### Project.open()

`Project.open("vulnerable.ll")` does several things internally:
1. Detects the `.ll` extension and selects the LLVM frontend
2. Parses the LLVM IR into SAF's Analysis IR (AIR)
3. Builds the call graph, def-use chains, and points-to analysis
4. Constructs the ValueFlow graph

## Step 3: Explore the Graphs

You can also inspect the program's structure:

```python
# Export graphs
graphs = proj.graphs()

# Call graph
cg = graphs.export("callgraph")
print(f"Functions: {len(cg['nodes'])}")
print(f"Call edges: {len(cg['edges'])}")

# ValueFlow graph
vf = graphs.export("valueflow")
print(f"VF nodes: {len(vf['nodes'])}, edges: {len(vf['edges'])}")
```

## Step 4: Use the Checker Framework

Instead of manually specifying sources and sinks, use the built-in checkers for
common vulnerability patterns:

```python
import saf

proj = saf.Project.open("vulnerable.ll")

# Run all built-in checkers (memory-leak, use-after-free, double-free, etc.)
all_findings = proj.check_all()

for f in all_findings:
    print(f"[{f.severity}] {f.checker}: {f.message}")

# Or run a specific checker
leak_findings = proj.check("memory-leak")
```

## Next Steps

- [Playground Tour](playground-tour.md) -- Try the same analysis in the browser
- [Browser vs Full SAF](browser-vs-full.md) -- Understand the differences
- [Tutorials](/tutorials/) -- Detect UAF, double-free, leaks, and more
