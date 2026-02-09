# Tutorial: Typestate Analysis for Memory Allocation

## Overview

This tutorial demonstrates typestate analysis for memory allocation lifecycle tracking. Typestate analysis uses the IDE framework (Sagiv/Reps/Horwitz TCS'96) to track per-resource state machines across program paths, detecting violations like leaks, double-free, and use-after-free.

## What is Typestate Analysis?

Typestate analysis tracks resources through a formal state machine:

```
             malloc()              free()
[Untracked] ---------> [Allocated] --------> [Freed] (accepting)
                            |                    |
                            |                    +--free()--> [Error] (double-free)
                            |                    |
                            +-----use after free-+--use-----> [Error] (UAF)
```

Key concepts:
- **States**: Allocated, Freed, Error
- **Transitions**: API calls (malloc, free) change state
- **Accepting state**: Freed (resource properly cleaned up)
- **Error state**: Invalid operation detected
- **Leak**: Resource in non-accepting state at function exit

## The Vulnerability

```c
/* BUG 1: Memory leak - malloc without free */
void alloc_then_leak(void) {
    char *ptr = (char *)malloc(256);
    if (ptr) {
        strcpy(ptr, "This memory will leak");
    }
    /* Missing: free(ptr) - resource in Allocated state at exit */
}

/* BUG 2: Double-free - free called twice */
void alloc_double_free(void) {
    char *ptr = (char *)malloc(128);
    free(ptr);
    free(ptr);  /* Transition: Freed -> Error */
}

/* BUG 3: Use-after-free */
void alloc_use_after_free(void) {
    int *ptr = (int *)malloc(sizeof(int));
    free(ptr);
    int x = *ptr;  /* Use in Freed state -> Error */
}

/* CORRECT: Proper lifecycle */
void alloc_correct(void) {
    char *ptr = (char *)malloc(64);
    if (ptr) {
        strcpy(ptr, "Correct usage");
        free(ptr);  /* Allocated -> Freed (accepting) */
    }
}
```

## Run the Tutorial

```bash
cd tutorials-new/memory-safety/04-typestate-memory
python detect.py
```

Expected output:
```
Step 3: Running memory_alloc typestate analysis...
  Result: TypestateResult(findings=N, spec='memory_alloc')

Step 4: Inspecting findings by type...

  Error-state findings (double-free, UAF): N
    [error_state] state=Error
      Resource: ...
      Function: alloc_double_free

  Leak findings (non-accepting at exit): N
    [non_accepting_at_exit] state=Allocated
      Resource: ...
      Function: alloc_then_leak
```

## Key API Calls

```python
import saf

# Load the project
proj = saf.Project.open("vulnerable.ll")

# Run the memory_alloc typestate analysis
result = proj.typestate("memory_alloc")

# Get error-state findings (double-free, UAF)
errors = result.error_findings()
for f in errors:
    print(f"[{f.kind}] state={f.state}, resource={f.resource}")

# Get leak findings (non-accepting at exit)
leaks = result.leak_findings()
for f in leaks:
    print(f"[{f.kind}] state={f.state}, resource={f.resource}")

# All findings combined
for f in result.findings():
    print(f.to_dict())
```

## Built-in Typestate Specifications

SAF includes three built-in typestate specs:

| Spec | Resource | States | Detecting |
|------|----------|--------|-----------|
| `memory_alloc` | Heap pointers | Allocated, Freed, Error | Leak, double-free, UAF |
| `file_io` | FILE* handles | Opened, Closed, Error | File leak, double-close, use-after-close |
| `mutex_lock` | pthread_mutex_t | Unlocked, Locked, Error | Lock leak, double-lock, unlock-before-lock |

## Typestate vs. Checker Framework

| Aspect | Typestate (IDE) | Checker Framework (SVFG) |
|--------|-----------------|--------------------------|
| Foundation | IDE dataflow solver | SVFG reachability |
| Precision | Path-sensitive via edge functions | Path-insensitive |
| State tracking | Full state machine | Source/sink pairs |
| Multiple violations | All in one pass | Separate checker per bug type |
| Custom specs | Yes (declarative) | Yes (resource table) |

## Creating Custom Typestate Specs

```python
# Define a custom spec for database connections
custom_spec = {
    "name": "db_connection",
    "states": ["Disconnected", "Connected", "Error"],
    "initial": "Disconnected",
    "accepting": ["Disconnected"],
    "error": "Error",
    "transitions": {
        "db_connect": {"Disconnected": "Connected"},
        "db_query": {"Connected": "Connected", "Disconnected": "Error"},
        "db_close": {"Connected": "Disconnected", "Disconnected": "Error"},
    }
}

# Use custom spec
result = proj.typestate_custom(custom_spec)
```

## Next Steps

- [05-path-sensitive](../05-path-sensitive/) - Reduce false positives with Z3-based path sensitivity
- [../resource-safety/](../../resource-safety/) - Apply typestate to files, locks, and other resources
