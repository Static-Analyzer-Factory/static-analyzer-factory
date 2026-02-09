# Tutorial: Flow-Sensitive Pointer Analysis

## What You'll Learn

- How **flow-sensitive PTA** tracks pointer targets per program point
- Why Andersen's flow-insensitive analysis produces false positives
- How **strong updates** kill stale pointer targets at reassignment sites
- Using the `FlowSensitivePtaResult` API (Python and Rust)

## Background

Andersen's pointer analysis (flow-insensitive) computes a single points-to
set for each pointer that holds across the entire program.  When a pointer is
reassigned, Andersen conservatively merges both targets:

```c
conn = &secret_conn;  // Andersen adds secret_conn
conn = &pub_conn;     // Andersen adds pub_conn
// pts(conn) = {secret_conn, pub_conn}  — false alias!
```

**Flow-sensitive PTA** tracks points-to information at each program point.
When a pointer is stored to and the destination is a singleton (only one
target), the analysis performs a **strong update** — it kills the old targets
and replaces them with the new one:

```
Before S2: pts(conn) = {secret_conn}
After  S2: pts(conn) = {pub_conn}       ← strong update killed secret_conn
At load:   conn -> pub_conn only
```

## The Scenario

`vulnerable.c` models a connection pool where a pointer `conn` is first bound
to a secret-only connection, then reassigned to a public connection.  A
flow-insensitive analysis falsely reports that the public sink may receive
secret data (because `conn` might point to `secret_conn`).  Flow-sensitive
analysis eliminates this false positive.

## Running

```bash
# Inside the dev container (make shell)
python detect.py
```

## Expected Output

```
=== Andersen (flow-insensitive) PTA ===
  Total values with points-to info: <N>

=== Flow-Sensitive PTA ===
  Solver iterations: <N>
  Converged: True
  Strong updates: <N>
  Weak updates: <N>
  ...

=== Summary ===
  Andersen (flow-insensitive):
    Merges all program points — `conn` may point to
    {secret_conn, pub_conn} everywhere.
  Flow-sensitive:
    Tracks per-program-point. After `conn = &pub_conn`,
    the strong update kills secret_conn from the set.
    At the load site, conn -> {pub_conn} only.
```

## Algorithm Overview

SAF's flow-sensitive PTA implements the **SFS** (Sparse Flow-Sensitive)
algorithm from Hardekopf & Lin (CGO'11):

1. **Pre-analysis**: Run Andersen's CI analysis to get initial points-to sets
2. **Build SVFG**: Sparse value-flow graph with Memory SSA
3. **Build FsSvfg**: Annotate SVFG indirect edges with object (LocId) labels
4. **SFS solver**: Worklist propagation with dfIn/dfOut sets per node
   - **Strong update** at stores with singleton destination (kill + gen)
   - **Weak update** at stores with multiple possible destinations (gen only)
5. **Result**: Per-value flow-sensitive points-to sets

## API

### Python

```python
proj = Project.open("vulnerable.ll")

# Flow-sensitive PTA
fs = proj.flow_sensitive_pta()

# Diagnostics
diag = fs.diagnostics()
print(diag["strong_updates"], diag["weak_updates"])

# Export to dict
export = fs.export()
```

### Rust

```rust
use saf_analysis::fspta::{self, FsPtaConfig, FsSvfgBuilder};

// Build FsSvfg from SVFG + PTA + MSSA
let fs_svfg = FsSvfgBuilder::new(&module, &svfg, &pta, &mut mssa, &cg).build();

// Solve
let config = FsPtaConfig::default();
let result = fspta::solve_flow_sensitive(&module, &fs_svfg, &pta, &cg, &config);

// Query
let diag = result.diagnostics();
println!("strong updates: {}", diag.strong_updates);
```

## Next Steps

Continue to the **Context-Sensitive PTA** tutorial to see how k-call-site
sensitivity distinguishes allocations across different call sites.
