# Tutorial: Path-Sensitive Memory Safety Analysis

## Overview

This tutorial demonstrates path-sensitive analysis using Z3 SMT solver to filter false positives in memory safety checking. Path-insensitive analysis may report bugs on execution paths that are actually infeasible due to branch conditions. Path-sensitive analysis verifies the feasibility of each finding's path using SMT solving.

## The Problem with Path-Insensitive Analysis

Consider this null-guarded code:

```c
void config_add(Config *cfg, const char *key, const char *value) {
    // Null guard: only dereference cfg if non-null
    if (!cfg) return;

    // This dereference is safe - guarded by null check above
    cfg->entries[cfg->count].key = strdup(key);
}
```

A path-insensitive null-dereference checker might flag `cfg->entries[cfg->count]` as a potential null dereference. But the `if (!cfg) return;` guard makes this path infeasible - if `cfg` were NULL, the function would have returned early.

## How Path-Sensitive Analysis Works

1. **Stage 1**: Run path-insensitive checkers to find potential bugs
2. **Stage 2**: For each finding:
   - Extract branch conditions (guards) along the SVFG trace
   - Translate guards to Z3 formulas
   - Check satisfiability:
     - **SAT** (satisfiable) -> Path is feasible, finding is a real bug
     - **UNSAT** (unsatisfiable) -> Path is infeasible, finding is a false positive
     - **Unknown/timeout** -> Keep finding conservatively

## The Vulnerable Code

```c
int config_add(Config *cfg, const char *key, const char *value) {
    // Null guard makes null-deref infeasible
    if (!cfg) return -1;
    cfg->entries[cfg->count].key = strdup(key);  // Safe!
    ...
}

int process_config(const char *filename) {
    Config *cfg = config_create(16);
    config_add(cfg, "host", "localhost");
    const char *host = config_get(cfg, "host");

    config_free(cfg);  // Free the config

    // BUG: Use-after-free - host points into freed memory
    if (host) {
        printf("Connecting to: %s\n", host);
    }
    return 0;
}
```

The program has:
- **False positive**: Null dereference guarded by `if (!cfg) return`
- **True positive**: Use-after-free accessing `host` after `config_free(cfg)`

## Run the Tutorial

```bash
cd tutorials-new/memory-safety/05-path-sensitive
python detect.py
```

Expected output:
```
ANALYSIS MODE: Path-Insensitive (baseline)
==================================================
Total findings: N
  [warning] null-dereference: ...
  [warning] use-after-free: ...

ANALYSIS MODE: Path-Sensitive (Z3-filtered)
==================================================
Feasible findings (CONFIRMED BUGS): M
  [warning] use-after-free: accessing freed config entry

Infeasible findings (FALSE POSITIVES filtered): K
  [warning] null-dereference: guarded by null check

COMPARISON: Path-Insensitive vs Path-Sensitive
==================================================
  Metric                                   PI     PS
  ---------------------------------------- ------ ------
  Total findings reported                     N      M
  Confirmed bugs (feasible)                 N/A      M
  False positives (infeasible)              N/A      K
```

## Key API Calls

```python
import saf

proj = saf.Project.open("vulnerable.ll")

# Path-insensitive analysis (baseline)
pi_findings = proj.check_all()

# Path-sensitive analysis (Z3-filtered)
ps_result = proj.check_all_path_sensitive()

# Access classified findings
for f in ps_result.feasible:       # Real bugs
    print(f"BUG: {f.message}")
for f in ps_result.infeasible:     # False positives
    print(f"FP: {f.message}")
for f in ps_result.unknown:        # Uncertain
    print(f"UNKNOWN: {f.message}")

# Get diagnostics
diag = ps_result.diagnostics
print(f"Guards extracted: {diag['guards_extracted']}")
print(f"Z3 solver calls: {diag['z3_calls']}")

# Post-filter existing findings
filtered = proj.filter_infeasible(pi_findings)
```

## When Path-Sensitivity Helps

Path-sensitive analysis is most valuable when code has:

| Pattern | Example | PI Result | PS Result |
|---------|---------|-----------|-----------|
| Null guards | `if (!p) return; *p = x;` | False positive | Filtered |
| Correlated branches | Same flag guards alloc and dealloc | False positive | Filtered |
| Conditional cleanup | `if (allocated) free(p);` | May miss or false positive | Precise |
| Error handling | Multiple return paths with cleanup | May over-report | Precise |

## Limitations

Path-sensitive analysis has costs:
- **Performance**: Z3 solving adds overhead
- **Timeouts**: Complex conditions may time out (kept conservatively)
- **Approximation**: Some conditions cannot be precisely modeled

Use path-sensitive mode when:
- Reducing false positives is important
- Code has complex control flow
- Triaging a large number of findings

## Next Steps

- [../buffer-overflow/](../../buffer-overflow/) - Apply path-sensitive analysis to buffer overflows
- [../../advanced-techniques/](../../advanced-techniques/) - Explore Z3-enhanced taint and typestate analysis
