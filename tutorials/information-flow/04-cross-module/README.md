# Information Flow 04: Cross-Module Taint Analysis (CWE-78)

**Category:** Information Flow Analysis
**Difficulty:** Intermediate
**Estimated Time:** 20 minutes

## What You'll Learn

- How taint flows propagate across translation unit (module) boundaries
- How to use `llvm-link` to combine multiple LLVM IR modules for whole-program analysis
- Why interprocedural analysis is essential for real-world vulnerability detection

## Prerequisites

> Complete [SETUP.md](../../../SETUP.md) before starting this tutorial.

## The Vulnerability

In real-world code, the source of tainted data and the dangerous sink often
live in different files. A security audit that only examines one file at a time
would miss the vulnerability entirely.

In this example:
- **Module A** (`module_a.c`) reads a user-controlled environment variable via `getenv()`
- **Module B** (`module_b.c`) receives that data and passes it directly to `system()`

Neither file is obviously dangerous in isolation. The vulnerability only
becomes visible when you analyze the combined data flow.

### Module A -- Input

```c
#include <stdlib.h>
#include <stdio.h>

extern void module_b_process(const char *data);

int main(void) {
    const char *user_input = getenv("USER_CMD");  // SOURCE
    if (!user_input) {
        printf("Set USER_CMD environment variable\n");
        return 1;
    }
    module_b_process(user_input);  // passes tainted data
    return 0;
}
```

### Module B -- Sink

```c
#include <stdlib.h>

void module_b_process(const char *data) {
    system(data);  // SINK: command injection
}
```

## How SAF Detects It

SAF performs **whole-program analysis** on the combined LLVM IR:

1. **Link** -- `llvm-link-18` merges both modules into a single IR file, resolving the `extern` declaration to the actual function body.
2. **Interprocedural flow** -- SAF's ValueFlow graph tracks data across function boundaries: `getenv()` return -> `user_input` -> `module_b_process` parameter -> `system()` argument.
3. **Taint query** -- `taint_flow(sources=call("getenv"), sinks=call("system", arg_index=0))` finds the cross-module path.

## The Pipeline

```
module_a.c --+
             +-> clang-18 -> module_a.ll -+
module_b.c --+                            +-> llvm-link-18 -> combined.ll -> LLVM frontend -> AIR -> analysis -> findings
             +-> clang-18 -> module_b.ll -+
```

## Run the Detector

```bash
python3 tutorials-new/information-flow/04-cross-module/detect.py
```

Expected output:

```
Cross-module taint flows found: 1
  [0] finding_id=<hex id>
       trace steps: N
         ...
```

SAF traced the tainted data from `getenv()` in module_a through the function
call boundary into `system()` in module_b.

## Understanding the Code

The key difference from earlier tutorials is the **multi-file compilation and linking** step:

```python
# Compile each module separately
for src, ll in [(module_a_src, module_a_ll), (module_b_src, module_b_ll)]:
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(ll), str(src)],
        check=True,
    )

# Link into a single module for whole-program analysis
subprocess.run(
    ["llvm-link-18", "-S", "-o", str(combined_ll),
     str(module_a_ll), str(module_b_ll)],
    check=True,
)
```

After linking, the analysis proceeds identically to single-file tutorials.

## Using the Rust API

The Rust version follows the same pattern -- compile, link, load, analyze:

```rust
// Compile both modules
for (src, ll) in [(&module_a_src, &module_a_ll), (&module_b_src, &module_b_ll)] {
    Command::new("clang-18")
        .args(["-S", "-emit-llvm", "-O0", "-g",
               "-o", ll.to_str().unwrap(), src.to_str().unwrap()])
        .status().expect("clang-18 failed");
}

// Link
Command::new("llvm-link-18")
    .args(["-S", "-o", combined_ll.to_str().unwrap(),
           module_a_ll.to_str().unwrap(), module_b_ll.to_str().unwrap()])
    .status().expect("llvm-link-18 failed");

// Load and analyze (same as single-file tutorials)
let frontend = LlvmFrontend::new();
let bundle = frontend.ingest(&[combined_ll.as_path()], &Config::default()).unwrap();
```

## Next Steps

Continue to [Tutorial 05: IFDS Precision](../05-ifds-precision/README.md)
to learn about the IFDS algorithm and how it provides more precise
interprocedural taint tracking compared to BFS-based analysis.
