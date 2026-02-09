# Tutorial 2: Typestate Analysis for File I/O

## What You'll Learn

- What typestate analysis is and how it differs from simple leak detection
- How SAF's IDE solver tracks per-resource state machines
- Detecting file leaks, double-close, and use-after-close bugs
- The difference between error states and non-accepting states

## Prerequisites

> Complete [Tutorial 1: File Leak Detection](../01-file-leak/README.md) first.

## The Vulnerability Classes

Typestate analysis detects three classes of file I/O bugs:

### 1. File Leak (Non-Accepting at Exit)
```c
void read_config_leak(void) {
    FILE *fp = fopen("config.ini", "r");
    fread(NULL, 1, 1, fp);
    /* Missing fclose(fp) - resource leaked */
}
```
State at exit: `Open` (not an accepting state)

### 2. Double-Close (Error State)
```c
void read_config_double_close(void) {
    FILE *fp = fopen("config.ini", "r");
    fread(NULL, 1, 1, fp);
    fclose(fp);
    fclose(fp);  /* BUG: double-close */
}
```
Second `fclose()` transitions from `Closed` to `Error`

### 3. Use-After-Close (Error State)
```c
void read_config_use_after_close(void) {
    FILE *fp = fopen("config.ini", "r");
    fclose(fp);
    fread(NULL, 1, 1, fp);  /* BUG: use-after-close */
}
```
`fread()` on a `Closed` file transitions to `Error`

## What is Typestate Analysis?

Typestate analysis tracks the **state** of a resource through its lifecycle.
Unlike simple leak detection, it can detect bugs where operations are
performed in the wrong order or wrong state.

### The File I/O State Machine

```
                    fopen()
     [Uninit] -----------------> [Open]
                                   |
                     fread()       | fclose()
                     fwrite()      |
                     (stay Open)   v
                                [Closed]
                                   |
                    fclose()       | fread()/fwrite()
                    (to Error)     | (to Error)
                                   v
                                [Error]

Accepting states: Uninit, Closed
Error states: Error
```

### IDE Framework

SAF uses the **IDE** (Interprocedural Distributive Environment) framework
from Sagiv, Reps, and Horwitz (1996). This framework enables:

- **Precise tracking**: Each resource instance is tracked separately
- **Context-sensitivity**: Function calls are analyzed with caller context
- **Path-sensitivity**: Different branches can have different states
- **Efficient computation**: Polynomial-time algorithm via graph reachability

## How SAF Detects It

1. **Define the typestate specification**: States, transitions, accepting
   states, and error states.

2. **Run the IDE solver**: Propagates state information through the SVFG,
   computing which states are possible at each program point.

3. **Check for violations**:
   - **Error findings**: Any path reaches an error state
   - **Leak findings**: Any path exits in a non-accepting state

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

Step 3: Running file_io typestate analysis...
  Result: <saf.TypestateResult object>

Step 4: Inspecting findings...

  Error-state findings: 2
    [error] state=Error, resource=0x1234...
      spec: file_io, at instruction: call @fclose
    [error] state=Error, resource=0x5678...
      spec: file_io, at instruction: call @fread

  Leak findings (non-accepting at exit): 1
    [leak] state=Open, resource=0x9abc...

Step 5: Finding details as dictionaries...
  {'kind': 'error', 'state': 'Error', 'resource': '0x1234...', ...}

Step 6: IDE solver diagnostics...
  Jump function updates: 45
  Value propagations: 120

Step 7: Available typestate specs...
  - file_io
  - mutex_lock
  - memory_alloc

============================================================
SUMMARY: Found 3 typestate violations
  Error states (double-close, use-after-close): 2
  File leaks (non-accepting at exit): 1
  -> Double-close or use-after-close DETECTED
  -> File leak DETECTED
============================================================
```

## Understanding the Code

```python
import saf

proj = saf.Project.open(str(llvm_ir))

# Run typestate analysis with the built-in file_io spec
result = proj.typestate("file_io")

# Get error-state findings (double-close, use-after-close)
errors = result.error_findings()

# Get leak findings (non-accepting at exit)
leaks = result.leak_findings()

# Get all findings
all_findings = result.findings()

# Each finding has:
#   - kind: "error" or "leak"
#   - state: The state at the violation point
#   - resource: The resource ID
#   - spec_name: Which typestate spec detected it
#   - inst: The instruction where the violation occurred

# Solver diagnostics
diag = result.diagnostics()
print(f"Jump function updates: {diag['jump_fn_updates']}")
print(f"Value propagations: {diag['value_propagations']}")
```

### Finding Types

| Finding Type | Condition | Example |
|--------------|-----------|---------|
| Error | Path reaches error state | Double-close, use-after-close |
| Leak | Path exits in non-accepting state | fopen without fclose |

## Correct Code

```c
void read_config_correct(void) {
    FILE *fp = fopen("config.ini", "r");
    fread(NULL, 1, 1, fp);
    fclose(fp);  /* Proper close - state goes to Closed (accepting) */
}
```

This function:
1. Opens file: `Uninit` -> `Open`
2. Reads: stays `Open`
3. Closes: `Open` -> `Closed` (accepting)
4. Exits in accepting state: no leak

## Comparison with Simple Leak Detection

| Feature | Simple Leak (Tutorial 1) | Typestate (Tutorial 2) |
|---------|-------------------------|------------------------|
| Leak detection | Yes | Yes |
| Double-close | No | Yes |
| Use-after-close | No | Yes |
| Order violations | No | Yes |
| Analysis cost | Lower | Higher |
| Precision | Path-insensitive | Path-sensitive |

Use simple leak detection for quick checks; use typestate for thorough
lifecycle analysis.

## Exercises

1. **Add a new bug pattern**: Create a function that reads after close.
   Does SAF detect it as use-after-close?

2. **Test conditional paths**: Create a function where one branch leaks
   and one doesn't. Does SAF report a leak?

3. **Multiple files**: Open two files and close only one. How many leaks
   does SAF report?

4. **Examine diagnostics**: What do the `jump_fn_updates` and
   `value_propagations` numbers tell you about analysis complexity?

## Next Steps

Continue to [Tutorial 3: Lock Safety](../03-lock-safety/README.md)
to see how the same typestate framework applies to mutex lock/unlock
tracking.
