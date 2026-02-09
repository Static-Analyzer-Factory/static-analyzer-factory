# Tutorial 4: Your First Checker

This tutorial introduces SAF's checker framework, which provides built-in
detectors for common bug patterns like memory leaks, use-after-free, and more.

## What You Will Learn

- How SAF's checker framework works
- How to run built-in checkers
- How to interpret checker findings
- The resource table and checker diagnostics

## The Vulnerability

A memory leak occurs when allocated memory is not freed before it becomes
unreachable:

```c
char *create_greeting(const char *name) {
    char *greeting = (char *)malloc(256);
    // ... build greeting ...
    return greeting;  // Caller must free this!
}

int main(int argc, char *argv[]) {
    char *msg = create_greeting(argv[1]);

    if (msg) {
        // ... use msg ...
        // OOPS: We forgot to free(msg)!
    }

    return 0;
    // LEAK: msg is never freed
}
```

This is CWE-401: Missing Release of Memory after Effective Lifetime.

## How the Checker Framework Works

SAF's checker framework uses the ValueFlow graph to find bug patterns:

1. **Classify sites**: Identify allocators, deallocators, and other relevant
   function calls using the resource table
2. **Find sources**: Nodes where resources are acquired (e.g., `malloc` return)
3. **Find sinks**: Nodes where resources should be released (e.g., function exit)
4. **Search paths**: Use graph reachability to find paths from source to sink
   without going through a sanitizer (e.g., `free`)
5. **Report findings**: Each reachable path is a potential bug

## Run the Tutorial

```bash
cd tutorials-new/getting-started/04-your-first-checker
python detect.py
```

Expected output:
```
Step 1: Compiling C to LLVM IR...
  Compiled: vulnerable.c

Step 2: Loading project...
  Project loaded successfully

Step 3: Available checkers...
  SAF has 9 built-in checkers:
    - memory-leak: Detects allocated memory that is never freed (CWE-401)
    - use-after-free: Detects use of memory after it was freed (CWE-416)
    - double-free: Detects memory freed more than once (CWE-415)
    ...

==================================================
Step 4: Running memory-leak checker
==================================================

Findings: 1

  Finding 1:
    Checker: memory-leak
    Severity: high
    Message: Memory allocated here may not be freed
    Source: malloc call in create_greeting
    Sink: function exit
    Trace length: 3 step(s)
```

## Understanding the Code

### Running a Specific Checker

```python
import saf

proj = saf.Project.open("vulnerable.ll")

# Run a single checker by name
findings = proj.check("memory-leak")

for finding in findings:
    print(f"Severity: {finding.severity}")
    print(f"Message: {finding.message}")
    print(f"Source: {finding.source}")
    print(f"Sink: {finding.sink}")
```

### Running All Checkers

```python
# Run all available checkers at once
all_findings = proj.check_all()

# Group by checker name
by_checker = {}
for f in all_findings:
    by_checker.setdefault(f.checker, []).append(f)
```

### Exploring Available Checkers

```python
# Get the checker schema
schema = proj.checker_schema()

print(f"Available checkers: {schema['count']}")
for checker in schema["checkers"]:
    print(f"  {checker['name']}: {checker['description']}")
```

### The Resource Table

The resource table tells SAF which functions allocate, deallocate, or
modify resources:

```python
table = proj.resource_table()

# Check if a function has a specific role
table.has_role("malloc", "allocator")   # True
table.has_role("free", "deallocator")   # True
table.has_role("fopen", "allocator")    # True (file handles)
table.has_role("fclose", "deallocator") # True
```

## Available Checkers

| Checker | CWE | Description |
|---------|-----|-------------|
| memory-leak | 401 | Allocated memory never freed |
| use-after-free | 416 | Memory accessed after being freed |
| double-free | 415 | Memory freed more than once |
| null-deref | 476 | Null pointer dereference |
| uninitialized | 457 | Use of uninitialized memory |
| buffer-overflow | 120 | Write beyond buffer bounds |
| file-leak | 775 | Opened file never closed |
| format-string | 134 | Uncontrolled format string |
| command-injection | 78 | Tainted data to command execution |

## Exercises

1. **Fix the bug**: Add `free(msg);` to `vulnerable.c` and verify the checker
   no longer reports a finding
2. **Add more bugs**: Add a use-after-free or double-free to `vulnerable.c`
   and run the corresponding checker
3. **Custom resource**: Add a function that allocates a custom resource and
   see if the checker detects leaks (hint: it may need resource table updates)

## Understanding the Analysis

The memory leak checker works by:

1. Finding all `malloc`/`calloc`/`realloc` calls (sources)
2. Finding all function exits and unreachable code points (sinks)
3. Checking if there is a path from source to sink that does not pass
   through a `free` call (sanitizer)
4. Reporting paths where allocated memory escapes without being freed

This is why it catches our bug: the `malloc` in `create_greeting` returns
its result, which flows to `msg` in `main`, and `main` exits without
calling `free(msg)`.

## What's Next?

You have completed the Getting Started tutorials! You now understand:

- How to run taint analysis (Tutorial 1)
- How call graphs and CFGs represent program structure (Tutorial 2)
- How def-use chains and ValueFlow track data flow (Tutorial 3)
- How the checker framework finds bugs (Tutorial 4)

Continue to the **Taint Analysis** category for deeper exploration of
vulnerability detection, or explore the **Checkers** category for more
advanced checker patterns.
