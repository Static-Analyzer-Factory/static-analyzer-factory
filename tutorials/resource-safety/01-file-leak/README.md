# Tutorial 1: Detecting File Descriptor Leaks (CWE-775)

## What You'll Learn

- What file descriptor leaks are and why they matter
- How SAF's SVFG-based reachability analysis detects leaks
- The difference between `must_reach` and `must_not_reach` checkers
- How to use the built-in resource table

## Prerequisites

> Complete [SETUP.md](../../SETUP.md) before starting this tutorial.

## The Vulnerability

A file descriptor leak occurs when a program opens a file (or other resource)
but fails to close it on all execution paths. Over time, leaked descriptors
exhaust the process's file descriptor limit, causing subsequent opens to fail.

### Why It Matters

File descriptor leaks cause:
- **Resource exhaustion**: Process hits ulimit and can't open new files
- **Denial of service**: Server becomes unresponsive after handling many requests
- **Data loss**: Unflushed buffers may lose data if descriptors aren't closed
- **Security issues**: Open handles to sensitive files persist longer than needed

### Vulnerable Code

```c
char *read_config_value(const char *filepath, const char *key) {
    FILE *f = fopen(filepath, "r");  /* Opens file */
    if (!f) {
        return NULL;
    }

    char *buffer = (char *)malloc(1024);
    if (!buffer) {
        /* BUG: File handle leaked here!
         * fclose(f) should be called before return */
        return NULL;
    }

    /* ... rest of function properly closes f ... */
    fclose(f);
    return buffer;
}
```

The bug is subtle: the happy path closes the file, but the error path
(malloc failure) returns without calling `fclose()`.

## How SAF Detects It

SAF's file-descriptor-leak checker uses **SVFG reachability analysis**:

1. **Build the Sparse Value Flow Graph (SVFG)**: Tracks how values flow
   through the program, including through pointer assignments and function
   calls.

2. **Identify Sources**: The return value of `fopen()` is marked as a
   source (an opened file handle).

3. **Identify Sinks**: Function exit points (return statements) are sinks.

4. **Identify Sanitizers**: Calls to `fclose()` with the file handle as
   an argument are sanitizers (they "clean" the taint).

5. **Check Reachability**: The checker uses `must_not_reach` mode:
   - If a source reaches a sink WITHOUT passing through a sanitizer,
     that's a leak.

### Reachability Modes

SAF supports two reachability modes:

| Mode | Meaning | Example |
|------|---------|---------|
| `must_reach` | Source must flow to sink | Taint analysis (data must reach dangerous function) |
| `must_not_reach` | Source must NOT reach sink unsanitized | Leak detection (handle must not reach exit without close) |

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
Step 1: Compiling C source to LLVM IR...
  Compiled: vulnerable.c -> vulnerable.ll

Step 2: Loading via LLVM frontend...
  Project loaded: <saf.Project object>

Step 3: Running file descriptor leak checker (CWE-775)...
  Findings: 1

  File descriptor leak details:
    [1] Severity: warning
        Message: File handle opened by fopen may reach exit without fclose
        CWE-775

Step 4: Understanding the checker...
  The 'file-descriptor-leak' checker uses SVFG reachability:
    - Source: Return value of fopen()
    - Sink: Function exit points
    - Sanitizer: Arguments to fclose()
    - Mode: must_not_reach (leak if source reaches sink)

Step 5: Checker configuration...
  Name: file-descriptor-leak
  Mode: must_not_reach
  CWE: 775
  Description: File handle reaches exit without fclose()

============================================================
SUMMARY: File Descriptor Leak Detection (CWE-775)
  File descriptor leaks found: 1

  VULNERABILITY DETECTED in vulnerable.c
  The read_config_value() function has a path where:
    1. fopen() opens a file successfully
    2. malloc() fails, triggering early return
    3. fclose() is never called
  This leaks the file descriptor.
============================================================
```

## Understanding the Code

```python
import saf

# Load the LLVM IR
proj = saf.Project.open(str(llvm_ir))

# Run the specific file-descriptor-leak checker
fd_findings = proj.check("file-descriptor-leak")

# Each finding has:
#   - severity: "warning", "error", etc.
#   - message: Human-readable description
#   - cwe: CWE number (775 for file descriptor leaks)
#   - checker: Name of the checker that produced it

# View available checkers
schema = proj.checker_schema()
for c in schema["checkers"]:
    print(f"{c['name']}: {c['mode']} (CWE-{c.get('cwe', 'N/A')})")

# Run all checkers at once
all_findings = proj.check_all()
```

### The Resource Table

SAF maintains a **resource table** that maps function names to roles:

```python
table = proj.resource_table()

# Query built-in roles
table.has_role('fopen', 'file_opener')   # True
table.has_role('fclose', 'file_closer')  # True
table.has_role('malloc', 'allocator')    # True
table.has_role('free', 'deallocator')    # True

# Add custom resources
table.add('my_open', saf.FileOpener)
table.add('my_close', saf.FileCloser)
```

## The Fix

```c
char *read_config_value(const char *filepath, const char *key) {
    FILE *f = fopen(filepath, "r");
    if (!f) {
        return NULL;
    }

    char *buffer = (char *)malloc(1024);
    if (!buffer) {
        fclose(f);  /* FIX: Close file before returning */
        return NULL;
    }

    /* ... */
    fclose(f);
    return buffer;
}
```

Or use a cleanup pattern:

```c
char *read_config_value(const char *filepath, const char *key) {
    FILE *f = NULL;
    char *buffer = NULL;

    f = fopen(filepath, "r");
    if (!f) goto cleanup;

    buffer = malloc(1024);
    if (!buffer) goto cleanup;

    /* ... */

cleanup:
    if (f) fclose(f);
    return buffer;
}
```

## Exercises

1. **Fix the bug**: Modify `vulnerable.c` to close the file on the error
   path. Re-run the detector to verify the warning disappears.

2. **Add another leak**: Create a function that opens two files and only
   closes one. Does SAF detect both leaks?

3. **Test the fix patterns**: Implement the `goto cleanup` pattern. Does
   SAF correctly recognize it as safe?

4. **Explore other checkers**: Run `proj.check("memory-leak")`. What does
   it find in the same code?

## Next Steps

Continue to [Tutorial 2: Typestate File I/O](../02-typestate-file-io/README.md)
to learn how typestate analysis provides richer lifecycle tracking beyond
simple leak detection.
