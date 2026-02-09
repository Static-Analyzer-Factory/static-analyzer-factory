# Tutorial: Batch Scanning

**Difficulty:** Advanced | **Time:** 30 minutes | **Category:** Integration

## What You Will Learn

- How to scan multiple programs in a single pipeline
- How to compile C, C++, and Rust sources to LLVM IR for analysis
- How to scan for multiple vulnerability classes (SQLi, XSS, log injection)
- How to handle large codebases and analyze PTA performance at scale
- How to aggregate findings across programs for a unified SARIF report

## Prerequisites

Complete [Tutorial 03: SARIF Reporting](../03-sarif-reporting/README.md) before starting this one.

## Why Batch Scanning?

Real-world codebases contain multiple programs, libraries, and services written
in different languages. A practical scanning pipeline needs to:

1. **Compile** each source to LLVM IR using the appropriate compiler
2. **Analyze** each program independently
3. **Aggregate** findings across all programs into a unified report
4. **Handle failures** gracefully (some programs may not compile)
5. **Scale** to large codebases with hundreds of functions

This tutorial demonstrates a comprehensive batch scanning pipeline that processes
programs of varying complexity.

## The Programs

This tutorial includes 5 programs of increasing complexity:

### Simple Programs

#### `injection.c` -- Command Injection (C)

```c
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    return system(argv[1]);
}
```

Direct `argv` to `system()` flow -- the simplest command injection.

#### `dangling.cpp` -- Use-After-Free (C++)

```cpp
int *create_value(int x) {
    int *p = (int *)malloc(sizeof(int));
    *p = x;
    return p;
}

void use_after_free(int *p) {
    free(p);
    printf("value = %d\n", *p);  // dangling
}
```

This program has a use-after-free bug. The batch scanner applies a command
injection query, so it will return 0 findings for it (demonstrating that
findings are query-specific).

#### `unsafe_ffi.rs` -- Unsafe FFI (Rust)

```rust
extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn system(cmd: *const c_char) -> i32;
}

fn main() {
    let key = CString::new("USER_CMD").unwrap();
    unsafe {
        let val = getenv(key.as_ptr());
        if !val.is_null() {
            system(val);
        }
    }
}
```

Rust `unsafe` FFI calling `getenv` and passing the result to `system()`.

### Large Programs

#### `http_handler.c` -- HTTP Request Handler (~200 lines)

A realistic web application with multiple vulnerability classes:

| CWE | Vulnerability | Source | Sink |
|-----|--------------|--------|------|
| CWE-89 | SQL Injection | `argv[1]` (query params) | `db_query()` |
| CWE-79 | Cross-Site Scripting | `argv[1]` (query params) | `render_html()` |
| CWE-117 | Log Injection | `argv[1]` (path) | `log_access()` |

#### `plugin_system.cpp` -- Plugin System (~330 lines)

A stress test for PTA with multiple OOP patterns:

- **Virtual dispatch** through plugin class hierarchy
- **Factory pattern** with function pointer registry
- **Event bus** with callback function pointers
- **Health monitor** with checker function pointers
- **Dynamic allocation** with `new`/`delete`

## The Pipeline

```
For each source file:
  source -> compiler -> LLVM IR -> SAF analysis -> findings

For large codebases:
  LLVM IR -> PTA analysis -> scale metrics

Aggregate:
  All findings -> unified SARIF report
```

### Compilation Commands

| Language | Compiler | Command |
|----------|----------|---------|
| C | clang-18 | `clang-18 -S -emit-llvm -O0 -g -o out.ll source.c` |
| C++ | clang-18 | `clang-18 -S -emit-llvm -O0 -g -o out.ll source.cpp` |
| Rust | rustc | `rustc --emit=llvm-ir -C debuginfo=0 -o out.ll source.rs` |

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
======================================================================
SAF Batch Scanning Pipeline
======================================================================

Phase 1: Multi-Language Compilation and Scanning
----------------------------------------------------------------------

[injection (C)]
  Compiling injection.c...
  Loading into SAF...
  command_injection: 1 finding(s)

[dangling (C++)]
  Compiling dangling.cpp...
  Loading into SAF...
  command_injection: 0 findings

[unsafe_ffi (Rust)]
  Compiling unsafe_ffi.rs...
  Loading into SAF...
  command_injection: <N> finding(s)

[http_handler (C)]
  Compiling http_handler.c...
  Loading into SAF...
  sqli: <N> finding(s)
  xss: <N> finding(s)
  log_injection: <N> finding(s)
  --- Stress Test Stats ---
    Module: <N> functions, <N> globals
    CallGraph: <N> nodes, <N> edges
    PTA: <N> values, <N> locations

[plugin_system (C++)]
  Compiling plugin_system.cpp...
  Loading into SAF...
  --- Stress Test Stats ---
    Module: <N> functions, <N> globals
    CallGraph: <N> nodes, <N> edges
    PTA: <N> values, <N> locations
    PTA set sizes: {1: N, 2: N, ...}...

======================================================================
Batch Scan Summary
======================================================================
  Programs scanned: 5
  Programs failed:  0
  Total findings:   <N>

  Findings by CWE:
    CWE-78: <N>
    CWE-79: <N>
    CWE-89: <N>
    CWE-117: <N>

  SARIF report: batch_report.sarif.json
```

## Understanding the Code

### Multi-Language Compilation

Each language has a dedicated compile function:

```python
def compile_c(src: Path, ll: Path) -> bool:
    result = subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(ll), str(src)],
        capture_output=True,
    )
    return result.returncode == 0

def compile_rust(src: Path, ll: Path) -> bool:
    result = subprocess.run(
        ["rustc", "--emit=llvm-ir", "-C", "debuginfo=0",
         "-o", str(ll), str(src)],
        capture_output=True,
    )
    return result.returncode == 0
```

### Multi-Vulnerability Scanning

The same project can be scanned with different source/sink configurations:

```python
# SQL Injection
sqli_findings = q.taint_flow(
    sources=sources.function_param("main", 1),
    sinks=sinks.call("db_query"),
)

# XSS
xss_findings = q.taint_flow(
    sources=sources.function_param("main", 1),
    sinks=sinks.call("render_html"),
)
```

### Scale Metrics

For large codebases, we collect PTA statistics to understand analysis performance:

```python
pta = proj.pta_result()
export = pta.export()

# Points-to set size distribution
size_counts = Counter()
for entry in export.get("points_to", []):
    sz = len(entry.get("locations", []))
    size_counts[sz] += 1
```

The size distribution indicates analysis precision:
- **Size 1**: The pointer targets exactly one location (ideal precision)
- **Size 2-4**: Expected for polymorphic pointers
- **Size 5+**: May indicate imprecision or genuinely polymorphic code

## Error Handling

The scanner is defensive about failures:

- **Compilation failure**: Skip the program and increment `programs_failed`
- **Analysis failure**: Caught by `try/except`, prints a warning and returns
  an empty list

This is important for batch mode where some programs may use unsupported
language features or have missing dependencies.

## Scale Considerations

| Metric | Simple Programs | Large C | Large C++ |
|--------|-----------------|---------|-----------|
| Functions | 1-5 | ~20 | ~30+ |
| LLVM IR lines | 50-200 | ~500 | ~1000+ |
| PTA iterations | < 100 | < 1,000 | < 10,000 |
| Query time | < 100ms | < 1s | < 5s |

For even larger codebases, SAF's PTA and value flow analyses scale to programs
with hundreds of thousands of LLVM IR instructions. The `PtaConfig` limits
provide safety bounds for very large programs.

## SARIF Aggregation

All findings from all programs are combined into a single SARIF report:

```python
for f in all_findings:
    results.append({
        "ruleId": f["cwe"],
        "level": "error",
        "message": {"text": f"{f['type']} in {f['program']}"},
        "fingerprints": {"safFindingId": f["finding_id"]},
    })
```

This produces a single `batch_report.sarif.json` file with results categorized
by CWE ID and tagged with the source program.

## What You Have Learned

This tutorial demonstrated:

1. **Multi-language scanning** - C, C++, and Rust in one pipeline
2. **Multi-vulnerability scanning** - Different source/sink pairs for different CWEs
3. **Large codebase handling** - HTTP handler with 20+ functions
4. **PTA stress testing** - C++ plugin system with complex pointer patterns
5. **Scale metrics** - Understanding analysis performance through statistics
6. **Unified reporting** - Aggregating findings into a single SARIF report

## Summary

You have completed the Integration tutorials. You have learned:

1. **Schema Discovery** -- runtime API introspection
2. **JSON Export** -- portable graph data for external tools
3. **SARIF Reporting** -- standards-compliant vulnerability reports
4. **Batch Scanning** -- multi-language, multi-vulnerability scanning at scale

Return to the [Tutorial Index](../../README.md) for the full list of tutorials
across all categories.
