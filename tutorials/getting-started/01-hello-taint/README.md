# Tutorial 1: Hello, Taint Analysis!

Welcome to SAF! This tutorial introduces you to static analysis by detecting
a real security vulnerability: command injection.

## What You Will Learn

- How SAF's analysis pipeline works
- What taint analysis is and why it matters
- How to define sources (user input) and sinks (dangerous functions)
- How to run your first vulnerability detector

## Prerequisites

Make sure you have completed the SAF setup:
```bash
make shell  # Enter the Docker development environment
```

## The Vulnerability

Command injection occurs when user-controlled data is passed to a function
that executes shell commands. Here is a simple vulnerable program:

```c
#include <stdlib.h>

int main(int argc, char **argv) {
    // SOURCE: argv[1] is user-controlled input
    if (argc > 1) {
        // SINK: system() executes argv[1] as a shell command
        system(argv[1]);
    }
    return 0;
}
```

If an attacker runs `./program "rm -rf /"`, this program will execute that
command. This is CWE-78: OS Command Injection.

## How SAF Detects It

SAF uses **taint analysis** to track data flow:

1. **Source**: Where untrusted data enters (here: `argv` parameter)
2. **Sink**: Where untrusted data is dangerous (here: `system()` argument)
3. **Flow**: SAF finds paths connecting sources to sinks

If a path exists, SAF reports a finding.

## The Analysis Pipeline

```
vulnerable.c --> clang-18 --> LLVM IR --> SAF --> Findings
                 (compile)    (.ll file)  (analyze)
```

## Run the Tutorial

Inside the Docker shell:

```bash
cd tutorials-new/getting-started/01-hello-taint
python detect.py
```

Expected output:
```
Step 1: Compiling C to LLVM IR...
  Created: vulnerable.ll

Step 2: Loading project...
  Project loaded successfully

Step 3: Running taint analysis...

Results:
  Found 1 taint flow(s)

  Finding 1:
    ID: 0x...
    Trace: 2 step(s)
    Path: argv -> ... -> system()

  VULNERABILITY DETECTED: Command-line input flows to system()!
  This is CWE-78: Improper Neutralization of Special Elements
  used in an OS Command (Command Injection)
```

## Understanding the Code

Here is the key part of `detect.py`:

```python
from saf import Project, sources, sinks

# Load compiled IR
proj = Project.open("vulnerable.ll")

# Create query context
q = proj.query()

# Find taint flows
findings = q.taint_flow(
    sources=sources.function_param("main", 1),  # argv (parameter 1)
    sinks=sinks.call("system", arg_index=0),    # First arg to system()
)
```

Key concepts:
- `Project.open()` loads LLVM IR and builds analysis graphs
- `sources.function_param(name, index)` selects function parameters as sources
- `sinks.call(name, arg_index=N)` selects call arguments as sinks
- `taint_flow()` searches for paths from sources to sinks

## Exercises

1. **Add a new sink**: Modify `detect.py` to also detect flows to `execve()`
2. **Inspect the trace**: Print each step in `finding.trace.steps` to see
   the full data flow path
3. **Try environment variables**: Create a new C file that uses `getenv()`
   and modify the detect script to use `sources.function_return("getenv")`

## What's Next?

Continue to [Tutorial 2: Call Graphs and CFGs](../02-call-graph-cfg/README.md)
to learn how SAF represents program structure.
