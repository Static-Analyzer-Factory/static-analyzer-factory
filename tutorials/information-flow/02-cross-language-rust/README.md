# Information Flow 02: Cross-Language Detection in Rust (CWE-78)

**Category:** Information Flow Analysis
**Difficulty:** Intermediate
**Estimated Time:** 20 minutes

## What You'll Learn

- How SAF detects vulnerabilities across language boundaries (Rust to C FFI)
- Why a language-agnostic intermediate representation (AIR) enables this
- How `unsafe` Rust code can introduce the same injection vulnerabilities as C

## Prerequisites

> Complete [SETUP.md](../../../SETUP.md) before starting this tutorial.

## The Vulnerability

Rust's ownership system and borrow checker prevent many classes of bugs at
compile time. However, `unsafe` blocks bypass these checks -- and Rust's
safety guarantees say nothing about **injection vulnerabilities**. A Rust
program that passes user input to C's `system()` via FFI is just as
vulnerable to command injection as a C program.

### Vulnerable Code

```rust
// CWE-78: Command Injection in Rust via unsafe FFI

use std::ffi::{c_char, c_int, CString};

extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn system(command: *const c_char) -> c_int;
}

fn main() {
    unsafe {
        // SOURCE: getenv() returns user-controlled data from the environment.
        let key = CString::new("USER_CMD").unwrap();
        let cmd = getenv(key.as_ptr());

        // SINK: system() executes the string as a shell command.
        if !cmd.is_null() {
            system(cmd);
        }
    }
}
```

This is the same vulnerability as Tutorial 01 (CWE-78), but in Rust. The
taint source is `getenv()` (user-controlled environment variable), and the
sink is the same `system()` function, reached through an `unsafe` FFI call.

## How SAF Detects It

SAF operates on **LLVM IR**, a language-agnostic intermediate representation.
When Rust code is compiled to LLVM IR:

- `getenv()` is a standard C library function -- the Rust `extern "C"` block
  emits a direct call to it.
- `system()` is likewise a direct C library call.
- The return value of `getenv()` flows directly to `system()`'s argument
  as an SSA value -- no intermediate memory operations.

Because SAF operates on LLVM IR (converted to AIR), it traces the taint
flow identically regardless of the source language. This is the power of a
language-agnostic analysis framework.

- **Source**: Return value of `getenv()` -- user-controlled environment data.
- **Sink**: Argument 0 of `system()` -- libc command execution.
- **Flow**: The return value of `getenv()` is passed directly to
  `system()` as its argument -- a zero-hop direct flow.

## The Pipeline

```
vulnerable.rs -> rustc --emit=llvm-ir -> LLVM IR (.ll) -> LLVM frontend -> AIR -> analysis -> findings
```

Note the only difference from C tutorials: `rustc` replaces `clang-18`.

## Run the Detector

```bash
python3 tutorials-new/information-flow/02-cross-language-rust/detect.py
```

Expected output:

```
Found 1 cross-language taint flow(s):
  [0] finding_id=<hex id>
       trace: <pretty-printed trace>
```

## Understanding the Code

```python
import subprocess
from pathlib import Path
from saf import Project, sources, sinks

def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.rs"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Compile Rust source to LLVM IR (rustc instead of clang-18)
    subprocess.run(
        ["rustc", "--emit=llvm-ir", "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Load via LLVM frontend (same as C tutorials)
    proj = Project.open(str(llvm_ir))
    q = proj.query()

    # SOURCE: return value of getenv() -- user-controlled environment variable
    # SINK:   argument 0 of system() -- libc command execution
    findings = q.taint_flow(
        sources=sources.call("getenv"),
        sinks=sinks.call("system", arg_index=0),
    )

    print(f"Found {len(findings)} cross-language taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"       trace: {f.trace.pretty()}")

if __name__ == "__main__":
    main()
```

New concepts in this tutorial:

- **`rustc --emit=llvm-ir`** -- Compiles Rust to LLVM IR, just as `clang-18
  -S -emit-llvm` does for C. The resulting `.ll` file is loaded by the same
  LLVM frontend.
- **`f.trace.pretty()`** -- Produces a human-readable string representation
  of the trace, showing each step in the data flow path. This is useful for
  debugging and understanding how data moves through the program.
- **Cross-language selectors** -- `sources.call("getenv")` and
  `sinks.call("system", arg_index=0)` reference C library functions called
  from Rust via FFI. SAF treats them identically because AIR is
  language-agnostic.
- **Language-agnostic detection** -- The same `taint_flow()` API that detected
  C vulnerabilities in Tutorial 01 works unchanged for Rust. No
  language-specific configuration is needed.

## Using the Rust API

The Rust `detect.rs` compiles the source with `rustc` and uses `LlvmFrontend`:

```rust
use std::process::Command;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    // Compile Rust source to LLVM IR
    Command::new("rustc")
        .args(["--emit=llvm-ir", "-o", ...])
        .status().expect("failed to run rustc");

    // Load via LLVM frontend (same as C tutorials)
    let frontend = LlvmFrontend::new();
    let bundle = frontend.ingest(&[llvm_ir.as_path()], &config).unwrap();
    // ... build graphs, run query

    // SOURCE: getenv() return -- user-controlled environment variable
    let sources = Selector::call_to("getenv")
        .resolve(&module).expect("source resolve failed");
    // SINK: system() arg 0 -- libc command execution
    let sinks = Selector::arg_to("system", Some(0))
        .resolve(&module).expect("sink resolve failed");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
}
```

The Rust API is identical to previous tutorials -- only the compiler
(`rustc` vs `clang-18`) and function names in the selectors change. This
uniformity is a direct consequence of SAF's language-agnostic design.

## Key Takeaway: Language-Agnostic Analysis

This tutorial demonstrates SAF's most powerful feature for cross-language
codebases: **any language that compiles to LLVM IR can be analyzed with the
same tools and APIs**. This includes:

- C and C++
- Rust
- Swift
- Objective-C
- And many more via LLVM frontends

The security analyst writes one detector, and it works across all these languages.

## Next Steps

Continue to [Tutorial 03: Format String Vulnerabilities](../03-format-string/README.md)
to see how the same taint analysis technique detects a completely different
vulnerability class by changing the source and sink selectors.
