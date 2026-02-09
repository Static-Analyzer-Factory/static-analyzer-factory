# Tutorial 4: Custom Resource Specifications

## What You'll Learn

- How to define typestate specifications for domain-specific resources
- When to use typestate analysis vs. SVFG-based reachability
- The `check_custom()` API for flexible resource leak detection
- Adding custom functions to the resource table

## Prerequisites

> Complete [Tutorial 3: Lock Safety](../03-lock-safety/README.md) first.

## The Problem

SAF provides built-in specifications for common resources like files and
mutexes, but your codebase likely has domain-specific resources:

- Database connections
- Network sockets
- Graphics contexts
- Hardware handles
- Session objects

These resources have similar lifecycle patterns (acquire, use, release)
but aren't covered by built-in checkers.

## Example: Database Connection API

```c
/* Database connection API */
db_connection_t *db_connect(const char *host, int port);
int db_query(db_connection_t *conn, const char *sql);
int db_disconnect(db_connection_t *conn);
```

### The Bugs

```c
/* BUG 1: Connection leak */
int process_data_leak(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    if (db_query(conn, "SELECT ...") != 0) {
        return -1;  /* Leak! Should call db_disconnect() */
    }
    db_disconnect(conn);
    return 0;
}

/* BUG 2: Use-after-disconnect */
int process_data_use_after_close(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    db_disconnect(conn);
    db_query(conn, "SELECT ...");  /* Bug! Using closed connection */
    return 0;
}

/* BUG 3: Double-disconnect */
int process_data_double_close(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    db_disconnect(conn);
    db_disconnect(conn);  /* Bug! Double-free */
    return 0;
}
```

## Approach 1: Custom Typestate Specification

Define a state machine that models the connection lifecycle:

```python
import saf

db_spec = saf.TypestateSpec(
    name="db_connection",
    states=["uninit", "connected", "disconnected", "error"],
    initial_state="uninit",
    error_states=["error"],
    accepting_states=["uninit", "disconnected"],
    transitions=[
        # Normal lifecycle
        ("uninit", "db_connect", "connected"),
        ("connected", "db_query", "connected"),
        ("connected", "db_disconnect", "disconnected"),

        # Error transitions
        ("disconnected", "db_query", "error"),       # Use-after-disconnect
        ("disconnected", "db_disconnect", "error"),  # Double-disconnect
    ],
    constructors=["db_connect"],
)

result = proj.typestate_custom(db_spec)
```

### Typestate Detection Capabilities

| Bug Type | Detected? | Finding Type |
|----------|-----------|--------------|
| Connection leak | Yes | Leak (non-accepting at exit) |
| Use-after-disconnect | Yes | Error state |
| Double-disconnect | Yes | Error state |
| Wrong operation order | Yes | Depends on spec |

## Approach 2: SVFG-Based Reachability (`check_custom`)

For simpler leak detection without full typestate tracking:

```python
# Add custom functions to resource table
table = proj.resource_table()
table.add("db_connect", saf.Allocator)
table.add("db_disconnect", saf.Deallocator)

# Run custom check
findings = proj.check_custom(
    "db-connection-leak",
    mode=saf.MustNotReach,
    source_role=saf.Allocator,
    source_match_return=True,
    sink_is_exit=True,
    sanitizer_role=saf.Deallocator,
    sanitizer_match_return=False,
    cwe=772,
    severity=saf.Warning,
)
```

### check_custom Parameters

| Parameter | Description |
|-----------|-------------|
| `name` | Identifier for the checker |
| `mode` | `MustReach` or `MustNotReach` |
| `source_role` | Resource role that generates the value to track |
| `source_match_return` | Track return value (True) or argument (False) |
| `sink_is_exit` | Use function exit as sink |
| `sanitizer_role` | Resource role that "cleans" the value |
| `cwe` | CWE number for the finding |
| `severity` | `Warning`, `Error`, etc. |

### Resource Roles

SAF provides these built-in resource roles:

| Role | Examples |
|------|----------|
| `saf.Allocator` | malloc, calloc, realloc |
| `saf.Deallocator` | free |
| `saf.FileOpener` | fopen, open |
| `saf.FileCloser` | fclose, close |
| `saf.LockAcquire` | pthread_mutex_lock |
| `saf.LockRelease` | pthread_mutex_unlock |

You can add your own functions to any role.

## When to Use Each Approach

| Feature | Typestate | check_custom |
|---------|-----------|--------------|
| Leak detection | Yes | Yes |
| Use-after-release | Yes | No |
| Double-release | Yes | No |
| Order violations | Yes | No |
| Setup complexity | Higher | Lower |
| Analysis cost | Higher | Lower |

**Use typestate** when you need to detect state-dependent bugs like
use-after-close or double-close.

**Use check_custom** when you only need leak detection and want simpler
setup or faster analysis.

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

Step 3: Defining custom typestate spec for database connections...
  Spec name: db_connection
  States: ['uninit', 'connected', 'disconnected', 'error']
  Error states: ['error']
  Accepting states: ['uninit', 'disconnected']
  Transitions: 5

Step 4: Running typestate analysis with custom spec...
  Result: <saf.TypestateResult object>

Step 5: Inspecting findings...

  Leak findings (connection not closed): 1
    [leak] state=connected, resource=0x1234...

  Error findings (use-after-disconnect, double-disconnect): 2
    [error] state=error, resource=0x5678...
      at instruction: call @db_query
    [error] state=error, resource=0x9abc...
      at instruction: call @db_disconnect

Step 6: Using check_custom() for SVFG-based leak detection...
  Built-in resource table size: 24
  After adding db_connect/db_disconnect: 26

  check_custom findings: 1
    [warning] Resource opened by db_connect may reach exit without release
      CWE: 772

Step 7: Comparing approaches...
  Typestate analysis (typestate_custom):
    - Detects leaks: 1
    - Detects use-after-disconnect/double-disconnect: 2
  SVFG reachability (check_custom):
    - Detects leaks: 1
    - Cannot detect use-after-disconnect (no state tracking)

============================================================
SUMMARY: Custom Resource Analysis
  Typestate violations found: 3
    Connection leaks: 1
    Use-after-disconnect / double-disconnect: 2
  SVFG-based findings: 1

  VULNERABILITIES DETECTED in vulnerable.c
  The code has the following bugs:
    - Connection leak in process_data_leak()
    - Use-after-disconnect in process_data_use_after_close()
    - Double-disconnect in process_data_double_close()
============================================================
```

## Correct Implementation

```c
int process_data_correct(const char *host) {
    db_connection_t *conn = db_connect(host, 5432);
    if (!conn) {
        return -1;
    }

    /* First query */
    if (db_query(conn, "SELECT * FROM users") != 0) {
        db_disconnect(conn);  /* Properly close on error */
        return -1;
    }

    /* Second query */
    if (db_query(conn, "UPDATE users SET active=1") != 0) {
        db_disconnect(conn);  /* Properly close on error */
        return -1;
    }

    db_disconnect(conn);  /* Properly close on success */
    return 0;
}
```

## Exercises

1. **Add a new resource**: Define a typestate spec for network sockets
   (`socket`, `connect`, `send`, `recv`, `close`).

2. **Complex state machine**: Add a "prepared" state for database prepared
   statements with separate `prepare` and `execute` operations.

3. **Multiple resources**: Modify the code to open two connections. Can
   SAF track them independently?

4. **Compare overhead**: Time both approaches on larger programs. How do
   they scale?

## Next Steps

This concludes the Resource Safety category. Continue to:
- [Information Flow](../../information-flow/) for taint analysis
- [Memory Safety](../../memory-safety/) for use-after-free detection
- [Advanced Techniques](../../advanced-techniques/) for Z3 refinement
