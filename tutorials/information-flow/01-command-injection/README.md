# Information Flow 01: Detecting Command Injection (CWE-78)

**Category:** Information Flow Analysis
**Difficulty:** Beginner
**Estimated Time:** 15 minutes

## What You'll Learn

- What taint analysis is and how it detects security vulnerabilities
- How to define **sources** (user input) and **sinks** (dangerous functions)
- How to run a taint flow query using the SAF Python SDK

## Prerequisites

> Complete [SETUP.md](../../../SETUP.md) before starting this tutorial.

## The Vulnerability

Command injection occurs when user-controlled input is passed directly to a
function that executes operating system commands. If an attacker controls the
argument to `system()`, they can execute arbitrary commands on the host machine.

For example, a program that runs `system(argv[1])` allows anyone who invokes it
to pass a shell command as the first argument -- the program will execute whatever
the user provides.

### Vulnerable Code

```c
// CWE-78: OS Command Injection

#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;

    // SOURCE: argv[1] is controlled by the user at runtime
    char *user_cmd = argv[1];

    // SINK: system() executes the string as a shell command.
    // If user_cmd is attacker-controlled, this is command injection.
    return system(user_cmd);
}
```

The data flows from `argv[1]` (user-controlled) directly to `system()`
(command execution) with no sanitization in between.

## How SAF Detects It

SAF uses **taint analysis** to find this bug. Taint analysis tracks how data
flows through a program:

1. **Source** -- where "tainted" (untrusted) data enters the program. Here, that
   is `main`'s second parameter (`argv`, parameter index 1).
2. **Sink** -- a dangerous function that should never receive tainted data. Here,
   that is the first argument to `system()` (argument index 0).
3. **Flow** -- the path data takes from source to sink. SAF's ValueFlow graph
   captures this as a chain of edges: the parameter value is copied to
   `user_cmd`, which is passed as the argument to `system()`.

If SAF finds a path from source to sink, it reports a **finding** -- a potential
vulnerability.

## The Pipeline

```
vulnerable.c -> clang-18 -> LLVM IR (.ll) -> LLVM frontend -> AIR -> analysis -> findings
```

The detection script handles the full pipeline automatically.

## Run the Detector

```bash
python3 tutorials-new/information-flow/01-command-injection/detect.py
```

Expected output:

```
Found 1 taint flow(s):
  [0] finding_id=<hex id>
       trace steps: 3
```

SAF found exactly one taint flow connecting `argv` to `system()`.

## Understanding the Code

Here is the full `detect.py` with a line-by-line walkthrough:

```python
import subprocess
from pathlib import Path
from saf import Project, sources, sinks

def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # 1. Compile the C source to LLVM IR using clang-18.
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # 2. Load the LLVM IR via the LLVM frontend.
    #    Project.open() detects the .ll extension and uses the
    #    LLVM frontend automatically, then builds internal graphs.
    proj = Project.open(str(llvm_ir))

    # 3. Create a query context -- this gives access to taint_flow().
    q = proj.query()

    # 4. Define the taint flow query.
    #    sources.function_param("main", 1) selects main()'s parameter at
    #    index 1 (argv). sinks.call("system", arg_index=0) selects the
    #    first argument passed to any call to system().
    findings = q.taint_flow(
        sources=sources.function_param("main", 1),
        sinks=sinks.call("system", arg_index=0),
    )

    # 5. Print the results.
    print(f"Found {len(findings)} taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"       trace steps: {len(f.trace.steps)}")

if __name__ == "__main__":
    main()
```

Key points:

- **Compilation step** -- `clang-18 -S -emit-llvm -O0 -g` compiles C to
  human-readable LLVM IR (`.ll`). The `-O0` flag disables optimizations
  (preserving data flow structure) and `-g` adds debug info.
- `Project.open()` detects the `.ll` extension and automatically uses the
  LLVM frontend. It then builds the full analysis pipeline (DefUse graph,
  call graph, points-to analysis, ValueFlow graph) internally.
- `sources.function_param(name, index)` selects a function parameter as a
  taint source by function name and parameter position.
- `sinks.call(name, arg_index=N)` selects a call argument as a taint sink
  by callee name and argument position.
- `q.taint_flow(sources, sinks)` runs a BFS over the ValueFlow graph to find
  paths from any source to any sink. It returns a list of `Finding` objects.
- Each `Finding` has a deterministic `finding_id` and an optional `trace`
  showing the step-by-step data flow path.

## Using the Rust API

The equivalent Rust code in `detect.rs` compiles the source and builds the
analysis pipeline explicitly:

```rust
use std::process::Command;
use saf_frontends::llvm::LlvmFrontend;
use saf_frontends::api::Frontend;
use saf_core::config::Config;

fn main() {
    // Compile C source to LLVM IR
    let status = Command::new("clang-18")
        .args(["-S", "-emit-llvm", "-O0", "-g",
               "-o", "tutorials-new/information-flow/01-command-injection/vulnerable.ll",
               "tutorials-new/information-flow/01-command-injection/vulnerable.c"])
        .status().expect("failed to run clang-18");
    assert!(status.success());

    // Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let bundle = frontend.ingest(
        &[Path::new("tutorials-new/information-flow/01-command-injection/vulnerable.ll")],
        &Config::default(),
    ).unwrap();
    let module = bundle.module;

    // Build graphs manually (Python SDK does this inside Project.open)
    let defuse = DefUseGraph::build(&module);
    let callgraph = CallGraph::build(&module);
    // ... PTA, ValueFlow, selectors, taint_flow (same as before)
}
```

The main difference from the Python version:

- **Compilation step**: Uses `std::process::Command` to invoke `clang-18`.
- **Explicit pipeline**: You build DefUse, CallGraph, PTA, and ValueFlow
  graphs yourself, giving full control over configuration.
- **LlvmFrontend**: Replaces `AirJsonFrontend` -- reads `.ll` files directly.
- **Selectors**: `Selector::function_param` and `Selector::arg_to` are the
  Rust equivalents of `sources.function_param` and `sinks.call`.
- **Sanitizers**: The third argument to `taint_flow` is a `BTreeSet` of
  sanitizer node IDs (empty here -- no sanitizers for this query).

## Next Steps

Continue to [Tutorial 02: Cross-Language Rust](../02-cross-language-rust/README.md)
to see how SAF detects vulnerabilities across language boundaries (Rust to C FFI).
