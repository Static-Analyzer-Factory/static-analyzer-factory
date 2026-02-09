# Tutorial: Memory Leak Detection (CWE-401)

## Overview

This tutorial introduces memory leak detection using SAF's SVFG-based checker framework. Memory leaks occur when dynamically allocated memory is never freed, leading to gradual memory exhaustion and eventual program failure.

## The Vulnerability

The vulnerable code is a simplified HTTP header parser with a partial cleanup bug:

```c
Header *parse_header(const char *line) {
    Header *h = (Header *)malloc(sizeof(Header));
    if (!h) return NULL;

    h->name = (char *)malloc(64);
    h->value = (char *)malloc(256);

    if (!h->name || !h->value) {
        // BUG: Partial cleanup - if value alloc fails, name leaks
        free(h);
        return NULL;
    }
    // ...
}
```

When `h->value` allocation fails but `h->name` succeeded, only `h` is freed. The memory allocated for `h->name` is leaked.

## Detection Technique

SAF's memory leak checker uses **must_not_reach reachability** on the SVFG:

| Component | Description |
|-----------|-------------|
| Source | `malloc()` return value (allocated pointer) |
| Sink/Sanitizer | `free()` argument (deallocation point) |
| Mode | `must_not_reach` - pointer must reach free on ALL paths |
| Finding | Pointer reaches function exit without passing through `free()` |

Unlike `may_reach` (used for UAF/double-free), `must_not_reach` reports when a resource is NOT properly cleaned up on every execution path.

## Run the Tutorial

```bash
cd tutorials-new/memory-safety/01-leak-detection
python detect.py
```

Expected output:
```
Step 1: Compiling C source to LLVM IR...
  Compiled: vulnerable.c -> vulnerable.ll

Step 2: Loading via LLVM frontend...
  Project loaded: ...

Step 3: Running memory leak checker (CWE-401)...
  Findings: N

  [0] Memory leak: allocated memory may not be freed
      Severity: warning
      Checker:  memory-leak
      ...
```

## Key API Calls

```python
import saf

# Load the project
proj = saf.Project.open("vulnerable.ll")

# Run the memory-leak checker
findings = proj.check("memory-leak")

# Each finding has:
for f in findings:
    print(f.message)     # Human-readable description
    print(f.severity)    # "warning" or "error"
    print(f.source)      # Where allocation happens
    print(f.sink)        # Where leak is detected (function exit)
    print(f.trace)       # Path from allocation to leak point
```

## Understanding the Checker Framework

SAF's checker framework includes 9 built-in checkers:

| Checker | CWE | Mode | Description |
|---------|-----|------|-------------|
| memory-leak | 401 | must_not_reach | Allocation without deallocation |
| use-after-free | 416 | may_reach | Use of freed memory |
| double-free | 415 | may_reach | Freeing memory twice |
| null-dereference | 476 | may_reach | Dereferencing NULL |
| file-descriptor-leak | 403 | must_not_reach | Open file not closed |
| uninit-use | 457 | may_reach | Using uninitialized memory |
| stack-escape | 562 | may_reach | Returning stack address |
| lock-not-released | 764 | must_not_reach | Mutex not unlocked |
| generic-resource | N/A | configurable | Custom resource tracking |

## Next Steps

- [02-use-after-free](../02-use-after-free/) - Detect dangling pointer dereference
- [03-double-free](../03-double-free/) - Detect freeing memory twice
