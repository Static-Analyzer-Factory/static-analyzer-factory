# Information Flow 03: Detecting Format String Vulnerabilities (CWE-134)

**Category:** Information Flow Analysis
**Difficulty:** Beginner
**Estimated Time:** 15 minutes

## What You'll Learn

- How to use **call-return selectors** to taint the return value of a function
- How different CWEs reuse the same taint analysis technique with different
  source/sink configurations
- The difference between `sources.function_param()` and `sources.call()`

## Prerequisites

> Complete [SETUP.md](../../../SETUP.md) before starting this tutorial.

## The Vulnerability

A format string vulnerability occurs when user-controlled input is used as
the format string argument to a function like `printf()`. The format string
controls how subsequent arguments are interpreted -- if an attacker supplies
format specifiers like `%x` (read stack memory) or `%n` (write to memory),
they can leak data or gain code execution.

In this program, `gets()` reads arbitrary user input into a buffer, and that
buffer is passed directly as `printf()`'s format string.

### Vulnerable Code

```c
// CWE-134: Uncontrolled Format String

#include <stdio.h>

extern char *gets(char *);

int main(void) {
    char buf[256];

    // SOURCE: gets() returns a pointer to the tainted buffer.
    char *input = gets(buf);

    // SINK: printf()'s argument 0 is the format string.
    // If user-controlled, this enables format string attacks.
    printf(input);

    return 0;
}
```

The taint flows from the return value of `gets()` to the first argument
(index 0) of `printf()` -- the format string position. We capture `gets()`'s
return value explicitly so that SAF can trace the SSA data flow.

## How SAF Detects It

This tutorial uses the same **taint analysis** engine as Tutorial 01, but with
different selectors:

- **Source**: The return value of `gets()`. Unlike Tutorial 01 where the source
  was a function parameter, here the source is a **call return value** -- the
  data produced by calling `gets()`.
- **Sink**: Argument 0 of `printf()` -- the format string position.
- **Flow**: `gets()` writes user input into `buf`, which is then passed as
  `printf()`'s first argument.

The key insight: **different CWEs use the same analysis engine**. You only
change which functions you mark as sources and sinks.

## The Pipeline

```
vulnerable.c -> clang-18 -> LLVM IR (.ll) -> LLVM frontend -> AIR -> analysis -> findings
```

## Run the Detector

```bash
python3 tutorials-new/information-flow/03-format-string/detect.py
```

Expected output:

```
Found 1 format string taint flow(s):
  [0] finding_id=<hex id>
```

## Understanding the Code

```python
import subprocess
from pathlib import Path
from saf import Project, sources, sinks

def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Load via LLVM frontend
    proj = Project.open(str(llvm_ir))
    q = proj.query()

    # SOURCE: return value of gets() -- tainted user input
    # SINK:   argument 0 of printf() -- the format string position
    findings = q.taint_flow(
        sources=sources.call("gets"),
        sinks=sinks.call("printf", arg_index=0),
    )

    print(f"Found {len(findings)} format string taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")

if __name__ == "__main__":
    main()
```

Compared to Tutorial 01:

| | Tutorial 01 (CWE-78) | Tutorial 03 (CWE-134) |
|---|---|---|
| **Source** | `sources.function_param("main", 1)` | `sources.call("gets")` |
| **Sink** | `sinks.call("system", arg_index=0)` | `sinks.call("printf", arg_index=0)` |
| **Source type** | Function parameter | Call return value |

`sources.call(name)` selects the **return value** of every call to the named
function as a taint source. This is useful when the untrusted data is produced
by a function (like `gets()`, `recv()`, `read()`) rather than arriving as a
parameter.

## Using the Rust API

The Rust `detect.rs` compiles the source and uses `LlvmFrontend` directly:

```rust
use std::process::Command;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    // Compile C source to LLVM IR
    Command::new("clang-18")
        .args(["-S", "-emit-llvm", "-O0", "-g", "-o", ...])
        .status().expect("failed to run clang-18");

    // Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let bundle = frontend.ingest(&[llvm_ir.as_path()], &config).unwrap();
    // ... build graphs, run query

    // SOURCE: return value of gets() calls
    let sources = Selector::call_to("gets")
        .resolve(&module).expect("source resolve failed");
    // SINK: argument 0 of printf() -- the format string
    let sinks = Selector::arg_to("printf", Some(0))
        .resolve(&module).expect("sink resolve failed");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
}
```

In the Rust API, `Selector::call_to(name)` is the equivalent of Python's
`sources.call(name)` -- it resolves to the return-value node of each call to
that function. The pipeline setup (compilation, LlvmFrontend, DefUse,
CallGraph, PTA, ValueFlow) follows the same pattern as Tutorial 01.

## Next Steps

Continue to [Tutorial 04: Cross-Module Taint](../04-cross-module/README.md)
to see how SAF traces taint flows across multiple source files using
whole-program analysis.
