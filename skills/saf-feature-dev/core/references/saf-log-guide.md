# SAF Debug Logging (`SAF_LOG`) Guide

## Core Principle

`SAF_LOG` is a FIRST-CLASS debugging tool, not a last resort. When something goes wrong, trace with `SAF_LOG` BEFORE guessing at fixes. The system has zero runtime overhead when disabled and requires no recompilation to enable.

## Adding Logging to New Code

Import the macro and use one of three forms. The semicolon separates narrative text from key-value pairs (`;` is used instead of `|` because Rust macro rules disallow `|` after `expr` fragments).

```rust
use saf_core::saf_log;

// Form 1 — narrative + key-values
saf_log!(pta::solve, worklist, "pts grew"; val=node_id, delta=&added);

// Form 2 — narrative only
saf_log!(pta::solve, convergence, "fixpoint reached");

// Form 3 — keys only
saf_log!(pta::solve, stats; iter=12, worklist=342);
```

The macro validates `module::phase` at compile time. A typo in the module or phase name produces a compile error.

### Value types for key-value pairs

Values passed to `saf_log!` must implement `SafLogValue`. Built-in implementations:

| Rust type | DSL rendering | Example |
|---|---|---|
| `u128` / `EntityId` types | `0x` + 32 hex chars | `0x00000000...001a2b` |
| `BTreeSet<T>` | `{a,b,c}` | `{0x1a,0x2b}` |
| `Vec<T>` / `&[T]` | `[a,b,c]` | `[10,20,30]` |
| `SafPair(&a, &b)` | `a->b` | `main->foo` |
| `SafRatio(n, m)` | `n/m` | `12/50` |
| `PtsDelta::Added(&items)` | `+{a,b}` | `+{0x3c}` |
| `PtsDelta::Removed(&items)` | `-{a,b}` | `-{0x3c}` |
| `Duration` | `N.NNNs` | `1.234s` |
| `Option<T>` | value or `none` | `42` / `none` |
| integers, `bool`, `&str`, `String` | bare value | `42`, `true`, `main` |

Import helpers: `use saf_core::logging::{SafPair, SafRatio, PtsDelta};`

## Registering New Modules and Phases

Modules and phases must be registered in `crates/saf-core/src/lib.rs` via the `saf_log_module!` macro. Tags are free-form and need no registration.

```rust
// In saf-core/src/lib.rs
saf_log_module! {
    pta { constraint, solve, hvn, scc, lcd },
    callgraph { build, refine },
    cfg { build },
    svfg { build, optimize },
    valueflow { build, query },
    defuse { build },
    mssa { build },
    checker { memleak, uaf, nullptr, pathsens },
    absint { interproc, transfer, escape, nullness },
    frontend { ingest },
    pipeline { constraint, incremental, analysis },
}
```

To add a new module or phase, add it to this invocation. For example, to add a `solver` phase to a new `dataflow` module:

```rust
saf_log_module! {
    // ... existing entries ...
    dataflow { solver, merge },
}
```

Then use it: `saf_log!(dataflow::solver, worklist, "processing node"; id=node_id);`

## Enabling at Runtime

Set the `SAF_LOG` environment variable. All commands run inside Docker.

```bash
# Specific module + phase + tags
docker compose run --rm -e SAF_LOG="pta::solve[worklist,pts]" dev sh -c 'cargo run ...'

# Multiple filters (comma-separated, brackets respected)
docker compose run --rm -e SAF_LOG="pta::solve[worklist],checker[reasoning]" dev sh -c '...'

# Everything
docker compose run --rm -e SAF_LOG=all dev sh -c '...'

# Everything except noisy tags
docker compose run --rm -e SAF_LOG="all,-pta::solve[worklist]" dev sh -c '...'

# Module only (all phases and tags within it)
docker compose run --rm -e SAF_LOG="pta" dev sh -c '...'

# Module + phase, all tags
docker compose run --rm -e SAF_LOG="pta::solve" dev sh -c '...'

# Wildcard module with specific tag
docker compose run --rm -e SAF_LOG="*[convergence]" dev sh -c '...'
```

### Filter grammar

```
SAF_LOG  = filter1,filter2,...
filter   = [-]module[::phase][[tag1,tag2,...]]
module   = identifier | "*"
phase    = identifier | "*"
```

A `-` prefix excludes matching events. Excludes are checked after includes.

## File Output

By default, output goes to stderr. Set `SAF_LOG_FILE` to redirect to a file, which avoids mixing with other stderr output.

```bash
docker compose run --rm \
  -e SAF_LOG="pta" \
  -e SAF_LOG_FILE=/tmp/saf.log \
  dev sh -c 'cargo run ... && cat /tmp/saf.log'
```

## Output Format

Each log line follows this DSL:

```
[module::phase][tag] narrative | key=value key=value
```

Variations by form:
- **Full**: `[pta::solve][worklist] pts grew | val=0x1a2b pts_size=3`
- **Narrative only**: `[pta::solve][convergence] fixpoint reached`
- **Keys only**: `[pta::solve][stats] | iter=12 worklist=342`

## Extracting Values from Output

```bash
# All pts-change events for a specific value
grep '\[pta::solve\]\[pts\]' /tmp/saf.log | grep 'val=0x1a2b'

# All checker reasoning across any checker phase
grep '\[checker.*\]\[reasoning\]' /tmp/saf.log

# Convergence stats
grep '\[pta::solve\]\[convergence\]' /tmp/saf.log

# Count events by tag
grep -oP '\[pta::solve\]\[\K[^\]]+' /tmp/saf.log | sort | uniq -c | sort -rn
```

## Common Debug Workflows

| Symptom | SAF_LOG value |
|---|---|
| Wrong PTA result | `pta::solve[pts,worklist]` |
| Missing callgraph edge | `callgraph[edge],pta::solve[pts]` |
| False positive/negative | `checker[reasoning,path,result]` |
| Slow analysis | `pta::solve[stats,convergence]` |
| Interprocedural bug | `absint::interproc` |
| Constraint extraction issue | `pta::constraint` |
| SVFG construction problem | `svfg[build]` |
| Value-flow query wrong result | `valueflow::query` |

### Debugging workflow pattern

1. Reproduce the issue with the appropriate `SAF_LOG` filter and `-e SAF_LOG_FILE=/tmp/saf.log`
2. Read the log file (on host, same path works since project root is mounted at `/workspace`)
3. Search for the relevant IDs or function names in the log
4. Trace the data flow: constraints extracted, points-to sets computed, checker decisions made
5. Identify where the actual behavior diverges from expected
6. Fix the root cause, then re-run with logging to verify
