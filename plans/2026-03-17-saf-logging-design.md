# SAF Structured Debug Logging System

## Overview

A structured, runtime-controlled debug logging system for SAF, designed primarily for AI coding agents performing post-mortem debugging. The system provides module/phase/tag filtering via the `SAF_LOG` env var and outputs a human-readable DSL format optimized for token-efficient consumption and grep-based extraction.

## Goals

- **AI-agent-first**: every logged message earns its tokens — structured, extractable, concise
- **Zero-cost when off**: no recompilation; disabled by default; one atomic branch per call site
- **Modular and extensible**: adding logging to a new algorithm is one macro call + one registry line
- **Clean sweep**: replaces all debug `eprintln!` calls, ad-hoc env vars, and the compile-time `solver-stats` feature gate

## Non-Goals

- Production observability (metrics, alerting, dashboards)
- Persistent log storage or rotation
- User-facing progress reporting (existing `tracing::info!` in saf-bench handles this)

---

## Output DSL Format

```
[module::phase][tag] narrative | key=value key=value ...
```

### Rules

- `[module::phase]` — always present, always two levels
- `[tag]` — always present, one word, lowercase
- Narrative — short human summary before `|` (optional if keys are self-explanatory)
- `|` — delimiter, always present when there are key-value pairs. The first `|` separates narrative from key-values; narrative text must not contain `|`
- `key=value` — no spaces around `=`, space-separated
- No timestamps by default (agents don't need wall-clock time)
- No log levels in output (the filter already selected what to show)

### Value Types

| Type | Syntax | Example | Description |
|------|--------|---------|-------------|
| **id** | `0x` + hex | `val=0x1a2b3c` | Any SAF ID (value, node, object) |
| **set** | `{el,el,...}` | `delta={0x1a,0x2b}` | Set of IDs |
| **num** | bare integer | `iter=12` | Counts, sizes, depths |
| **name** | bare string | `func=main` | Function/block/variable names |
| **path** | `node->node->...` | `path=bb1->bb3->bb5` | CFG/callgraph/flow paths |
| **list** | `[el,el,...]` | `args=[0x1a,0x2b]` | Ordered sequence (unlike set) |
| **pair** | `lhs->rhs` | `edge=main->foo` | Directed relationship |
| **bool** | `true`/`false` | `changed=true` | Flags |
| **delta** | `+{...}` / `-{...}` | `delta=+{0x3c}` | Set additions/removals |
| **ratio** | `n/m` | `progress=12/50` | Progress or fractions |

### Examples

```
[pta::solve][worklist] pts grew | val=0x1a2b delta=+{0x3c4d,0x5e6f} pts_size=3
[pta::solve][convergence] iteration done | iter=12 worklist=342 changed=18 progress=12/50
[callgraph::refine][edge] indirect resolved | edge=main->foo site=0xab12 resolved=true
[checker::memleak][reasoning] suppressed | val=0x1a2b path=bb1->bb3->bb5 freed=true
[svfg::build][node] MemPhi added | node=0x1234 block=bb3 func=main preds={bb1,bb2}
[pta::solve][scc] cycle merged | rep=0x1a2b members={0x3c,0x4d,0x5e} merged=3
[absint::interproc][context] call enter | func=foo depth=3 args=[0x1a,0x2b] caller=main
[valueflow::build][edge] flow added | edge=0x1a->0x2b kind=Store func=main
```

Agents can extract values by syntax: `0x` prefix = id, `{` = set, `[` = list, `->` in bare value = pair, `+{`/`-{` = delta. No type annotations needed in output.

---

## Filter Grammar (`SAF_LOG` env var)

```
SAF_LOG="filter1,filter2,..."
```

Each filter has the form:

```
module::phase[tag1,tag2]
```

Any part can be `*` (wildcard) or omitted.

### Filter Examples

| Filter | Meaning |
|--------|---------|
| `pta` | All phases and tags in pta module |
| `pta::solve` | All tags in pta::solve phase |
| `pta::solve[worklist]` | Only worklist tag in pta::solve |
| `pta::solve[worklist,scc]` | worklist and scc tags in pta::solve |
| `pta[convergence]` | convergence tag across all pta phases |
| `*::*[convergence]` | convergence tag across all modules and phases |
| `*[stats]` | stats tag everywhere |
| `all` | Everything (equivalent to `*::*[*]`) |

### Combining and Negation

Comma-separated filters use union semantics. Prefix with `-` to exclude:

```bash
# PTA worklist + callgraph edges + all checker reasoning
SAF_LOG="pta::solve[worklist],callgraph::refine[edge],checker[reasoning]"

# Everything in PTA + just convergence info elsewhere
SAF_LOG="pta,*[convergence]"

# All PTA except worklist spam
SAF_LOG="pta,-pta::solve[worklist]"

# Everything except stats
SAF_LOG="all,-*[stats]"
```

**No env var set = no SAF logging output.** Silent by default.

---

## Macro API

### Primary Macro

```rust
saf_log!(module::phase, tag, "narrative" | key=expr, key=expr, ...);
```

### Call Site Examples

```rust
// Full form
saf_log!(pta::solve, worklist, "pts grew" | val=node_id, delta=&added, pts_size=pts.len());

// Narrative only (no key-values)
saf_log!(pta::solve, convergence, "fixpoint reached");

// Keys only (no narrative)
saf_log!(pta::solve, stats, | iter=12, worklist=342, changed=18);
```

### Type Mapping via `SafLogValue` Trait

| Rust Type | Output Type | Example Output |
|-----------|-------------|----------------|
| `u128` / ID types | id | `0x1a2b3c` |
| `BTreeSet<T: SafLogValue>` | set | `{0x1a,0x2b}` |
| `Vec<T: SafLogValue>` / `&[T]` | list | `[0x1a,0x2b]` |
| `usize`, `u32`, `i64`, etc. | num | `42` |
| `&str`, `String` | name | `main` |
| `bool` | bool | `true` |
| `SafPair<T>` (newtype) | pair | `main->foo` |
| `SafRatio` (newtype) | ratio | `12/50` |
| `PtsDelta` (custom) | delta | `+{0x3c}` |
| `CfgPath` / `&[BlockId]` | path | `bb1->bb3->bb5` |

Note: `SafPair<T>` and `SafRatio` are distinct newtypes to avoid ambiguity — bare `(T, T)` tuples are not used since both pair (`a->b`) and ratio (`a/b`) would collide.

### Compile-Time Validation

Module and phase are path-like identifiers validated against the registry. Typos produce compile errors:

```rust
saf_log!(pta::solve, worklist, "msg" | val=x);   // OK
saf_log!(pta::slove, worklist, "msg" | val=x);   // Compile error
```

**Mechanism**: `saf_log_module!` generates a hidden module hierarchy used for validation:

```rust
// saf_log_module! { pta { solve, constraint } } expands to:
mod __saf_log_registry {
    pub mod pta {
        pub struct solve;
        pub struct constraint;
    }
}
```

The `saf_log!` macro expands to include a reference like `let _ = __saf_log_registry::pta::solve;` (optimized away), which triggers a compile error if the module or phase doesn't exist. At runtime, the module/phase names are passed as `&str` to the tracing event — the struct reference is purely for validation.

Tags are open — any identifier works without registration.

---

## Module/Phase/Tag Registry

### Modules and Phases

| Module | Phase | Description |
|--------|-------|-------------|
| `pta` | `constraint` | Constraint extraction (addr, copy, load, store, gep) |
| `pta` | `solve` | Worklist solver iterations |
| `pta` | `hvn` | Hash value numbering preprocessing |
| `pta` | `scc` | Cycle detection and merging |
| `pta` | `lcd` | Lazy cycle detection |
| `callgraph` | `build` | Initial callgraph construction |
| `callgraph` | `refine` | CG refinement during PTA |
| `cfg` | `build` | CFG construction |
| `svfg` | `build` | SVFG node/edge construction |
| `svfg` | `optimize` | MemPhi reduction, cleanup |
| `valueflow` | `build` | Value-flow graph construction |
| `valueflow` | `query` | Flow/taint queries |
| `defuse` | `build` | Def-use chain construction |
| `mssa` | `build` | Memory SSA construction |
| `checker` | `memleak` | Memory leak checker |
| `checker` | `uaf` | Use-after-free checker |
| `checker` | `nullptr` | Null pointer checker |
| `checker` | `pathsens` | Path-sensitive runner |
| `absint` | `interproc` | Interprocedural analysis |
| `absint` | `transfer` | Transfer function execution |
| `absint` | `escape` | Escape analysis |
| `absint` | `nullness` | Nullness analysis |
| `frontend` | `ingest` | IR ingestion and AIR conversion |

### Semantic Tags (Cross-Cutting)

| Tag | Meaning | Typical Use |
|-----|---------|-------------|
| `worklist` | Worklist push/pop/size | Solver iterations |
| `convergence` | Fixpoint progress | Iteration counts, changes per round |
| `stats` | Summary statistics | Replaces `solver-stats` feature gate |
| `node` | Node added/removed/merged | Any graph builder |
| `edge` | Edge added/removed | Any graph builder |
| `constraint` | Constraint created/fired | PTA, IFDS |
| `pts` | Points-to set changes | PTA solver |
| `reasoning` | Why a decision was made | Checkers, path feasibility |
| `context` | Call context enter/exit/depth | Interprocedural, CS-PTA |
| `path` | Path explored/pruned | Path-sensitive analysis |
| `delta` | Incremental change | Any diff-based propagation |
| `merge` | Nodes/sets merged | SCC, LCD, widening |
| `filter` | Items filtered/pruned | Checker suppression |
| `result` | Final output/verdict | Checker findings, query results |

### Extensibility

```rust
// In crates/saf-core/src/logging/registry.rs
saf_log_module! {
    cspta {
        solve,
        context,
        inline,
    }
}
```

Modules/phases are closed (registered, compile-time checked). Tags are open (any identifier, no registration).

---

## Architecture

### Crate Layout

The logging infrastructure is split across two crates to keep `saf-core` dependency-light:

```
crates/saf-core/src/logging/
├── mod.rs          # Public API: saf_log! macro, SafLogValue trait
├── registry.rs     # Module/phase declarations (saf_log_module! macro)
├── value.rs        # SafLogValue trait + impls for core types (u128, BTreeSet, etc.)
└── formatter.rs    # DSL output formatting (format_saf_log_line())

crates/saf-core/src/logging/subscriber/
├── mod.rs          # init() function, subscriber composition
├── filter.rs       # SAF_LOG env var parser and matcher
└── layer.rs        # tracing::Layer implementation (SafLogLayer)
```

**Dependency split**: `saf-core` depends on `tracing` (the facade crate — lightweight). The `subscriber/` submodule depends on `tracing-subscriber` and is only used by binary crates at initialization. Library code only uses the `saf_log!` macro and `SafLogValue` trait, which depend only on the `tracing` facade. The `tracing-subscriber` dependency is gated behind a `logging-subscriber` feature on `saf-core`, which binary crates (`saf-cli`, `saf-bench`, `saf-trace`) enable.

### Data Flow

```
saf_log!(pta::solve, worklist, "pts grew" | val=id, delta=&added)
        │
        ▼
  macro expands to:
    tracing::event!(Level::DEBUG,
        saf_module = "pta",
        saf_phase = "solve",
        saf_tag = "worklist",
        saf_narrative = "pts grew",
        saf_kv = "val=0x1a2b delta=+{0x3c}")
        │                              ▲
        │                              │
        │                   SafLogValue::fmt() serializes
        │                   Rust values into DSL types
        ▼
  tracing subscriber receives event
        │
        ▼
  SafLogLayer::on_event()
    1. Extract saf_module, saf_phase, saf_tag fields
    2. Check against SafLogFilter (parsed from SAF_LOG env var)
    3. If matched: format as DSL line, write to stderr
    4. If not matched: discard
```

### Initialization

`init()` returns a `SafLogLayer` that binaries compose into their subscriber stack. This replaces the current inconsistent per-binary initialization:

```rust
// Before (each binary different):
tracing_subscriber::fmt().with_env_filter("info").init();

// After (all binaries, unified):
use tracing_subscriber::prelude::*;
let saf_layer = saf_core::logging::subscriber::init(); // reads SAF_LOG, returns Layer
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("info".parse().unwrap()))
        .with_writer(std::io::stderr))
    .with(saf_layer)
    .init();
```

The `init()` function:
1. Reads `SAF_LOG` env var (if absent, returns a no-op layer)
2. Optionally reads `SAF_LOG_FILE` env var for file output (see below)
3. Parses filter grammar into `SafLogFilter`
4. Returns a `SafLogLayer` that the binary composes into its subscriber

### Output Destination

By default, `SafLogLayer` writes to stderr. Set `SAF_LOG_FILE` to redirect to a file:

```bash
# stderr (default)
docker compose run --rm -e SAF_LOG="pta::solve[worklist]" dev sh -c '...'

# file output (clean separation from other stderr)
docker compose run --rm -e SAF_LOG="pta" -e SAF_LOG_FILE=/tmp/saf.log dev sh -c '...'
```

When `SAF_LOG_FILE` is set, SAF log output goes exclusively to the file, not stderr. This avoids mixing with other stderr output from tracing or the application.

### Zero-Cost Path

Two tiers of cost when logging is inactive:

1. **No `SAF_LOG` and `RUST_LOG` at default (`info`)**: Each `saf_log!` emits at `Level::TRACE`. Tracing's max-level filter rejects it before any field serialization — cost is one atomic load + predictable branch per call site. **Effectively zero.**

2. **No `SAF_LOG` but `RUST_LOG=trace` (user enabled verbose tracing)**: Events pass tracing's level filter and reach `SafLogLayer`, which checks its filter and discards. Cost includes field serialization via `SafLogValue::fmt()` before the layer can reject. Still fast, but not zero. This scenario is rare and intentional.

To minimize tier-2 cost, `saf_log!` events use a dedicated tracing target (`saf_debug`) so that `RUST_LOG=trace` can be narrowed: `RUST_LOG=trace,saf_debug=off` disables SAF events even at trace level.

### Relationship to Existing Tracing

- Existing `tracing::info!` / `tracing::debug!` calls continue to work via `RUST_LOG`
- `SAF_LOG` controls only `saf_log!` output (target `saf_debug`)
- Both can be active simultaneously via layered subscriber composition

---

## Static-Analysis-Specific Logging Content

### PTA Solver

| Phase | Tag | What to Log | Why It Matters |
|-------|-----|-------------|----------------|
| `pta::constraint` | `constraint` | Each constraint extracted | Missing constraints = silent unsoundness |
| `pta::solve` | `worklist` | Value/location popped, constraints fired | Traces causality of pts propagation |
| `pta::solve` | `pts` | Points-to set change with delta | Core data — "why does X point to Y?" |
| `pta::solve` | `convergence` | Iteration count, worklist size, changed count | Detects infinite loops, slow convergence |
| `pta::solve` | `stats` | Phase timings, op counts | Performance debugging without recompile |
| `pta::scc` | `merge` | Nodes merged, representative chosen | SCC bugs cause incorrect aliasing |
| `pta::hvn` | `merge` | Nodes collapsed during preprocessing | Over-merging = precision loss |

### Callgraph

| Phase | Tag | What to Log | Why It Matters |
|-------|-----|-------------|----------------|
| `callgraph::build` | `edge` | Direct call edges discovered | Missing edges = missing analysis scope |
| `callgraph::refine` | `edge` | Indirect calls resolved via PTA | CG refinement is where virtual dispatch bugs hide |
| `callgraph::refine` | `convergence` | Refinement iterations, new edges per round | Should converge — if not, something is wrong |

### SVFG / Value Flow

| Phase | Tag | What to Log | Why It Matters |
|-------|-----|-------------|----------------|
| `svfg::build` | `node` | MemPhi, formal/actual in/out nodes created | Missing nodes = missing flows |
| `svfg::build` | `edge` | Direct/indirect value-flow edges | Core connectivity |
| `svfg::optimize` | `filter` | Nodes removed during optimization | Over-aggressive removal = missed bugs |
| `valueflow::query` | `path` | Paths explored during flow queries | Explains query results |
| `valueflow::query` | `result` | Final query verdict with evidence | The answer + why |

### Checkers

| Phase | Tag | What to Log | Why It Matters |
|-------|-----|-------------|----------------|
| `checker::*` | `reasoning` | Why a finding was reported or suppressed | Most important for false positives/negatives |
| `checker::*` | `path` | Paths explored, feasibility checks | Path-sensitive decisions |
| `checker::*` | `result` | Final verdict per allocation/resource | Confirms checker output |
| `checker::pathsens` | `filter` | UAF filtering statistics, what was pruned | Currently in pathsens_runner tracing::debug |

### Abstract Interpretation

| Phase | Tag | What to Log | Why It Matters |
|-------|-----|-------------|----------------|
| `absint::interproc` | `context` | Function enter/exit, call depth, recursion | Replaces `SAF_INTERPROC_DEBUG` |
| `absint::interproc` | `convergence` | Widening applied, fixpoint reached | Replaces `SAF_RECURSION_DEBUG` |
| `absint::transfer` | `constraint` | Transfer function constraint application | Replaces `SAF_TRACE_STORE` |
| `absint::transfer` | `result` | Transfer function output per instruction | Largest eprintln concentration (~27 calls) |
| `absint::escape` | `result` | Escape classification per value | Explains who escapes and why |
| `absint::nullness` | `result` | Nullness verdict per value | Explains null pointer findings |

### Frontend

| Phase | Tag | What to Log | Why It Matters |
|-------|-----|-------------|----------------|
| `frontend::ingest` | `stats` | Functions/instructions/globals parsed | Sanity check — did we ingest everything? |
| `frontend::ingest` | `filter` | Skipped intrinsics, unsupported instructions | Explains missing coverage |

---

## Migration Plan

### What Gets Replaced

| Current Mechanism | Count | Scope | Replacement |
|---|---|---|---|
| Debug `eprintln!` in `saf-analysis` | ~56 | analysis/debug output | `saf_log!` with appropriate module/phase/tag |
| `SAF_INTERPROC_DEBUG` env var | 1 | absint | `SAF_LOG=absint::interproc` |
| `SAF_RECURSION_DEBUG` env var | 1 | absint | `SAF_LOG=absint::interproc[convergence]` |
| `SAF_TRACE_STORE` env var | 1 | absint | `SAF_LOG=absint::transfer` |
| `solver-stats` feature gate | 1 | PTA | `SAF_LOG=pta::solve[stats]` |
| `tracing::debug!` in analysis code | ~14 | various | `saf_log!` with appropriate tags |
| Inconsistent `tracing_subscriber` init | 3 | binaries | Unified layered subscriber with `SafLogLayer` |

**Not migrated** (operational output, not debug logging):
- `eprintln!` in `saf-bench` (~79) — user-facing error/progress messages
- `eprintln!` in `saf-cli` (~15) — CLI error output
- `eprintln!` in `saf-python` (~1) — Python binding error output

### What Stays

- `tracing::info!` in `saf-bench` for operational output (test discovery, runtime)
- `RUST_LOG` env var for tracing-level output
- `tracing_subscriber` setup in binaries (simplified, with `SafLogLayer` added)

### Migration Rules

1. Each `eprintln!` becomes `saf_log!` with structured key-values extracted from the format string
2. `SAF_INTERPROC_DEBUG` / `SAF_RECURSION_DEBUG` guards are removed — filtering handled by `SAF_LOG`
3. `SAF_TRACE_STORE` guard removed — replaced by `SAF_LOG=absint::transfer`
4. `solver_stats.rs` `print_summary()` is split into multiple `saf_log!` calls grouped by category. The current single function emits ~40 fields; these become grouped lines:
   ```
   [pta::solve][stats] timing | copy=1.2s load=0.8s store=0.5s gep=0.3s total=2.8s
   [pta::solve][stats] operations | value_pops=1234 loc_pops=567 scc_merges=12
   [pta::solve][stats] worklist | final_size=0 max_size=8921 iterations=45
   ```
   Each line covers one logical group. The `#[cfg(feature = "solver-stats")]` gates are removed.
5. `solver-stats` feature flag removed from `Cargo.toml`
6. Existing `tracing::debug!` in analysis code converted to `saf_log!`

---

## CLAUDE.md Addition

The following section will be added to CLAUDE.md to document the DSL for agents:

```markdown
### SAF Debug Logging (`SAF_LOG`)

SAF has a structured debug logging system controlled by the `SAF_LOG` env var.
Disabled by default — no output unless explicitly enabled.

**Enabling (inside Docker):**
  docker compose run --rm -e SAF_LOG="pta::solve[worklist,pts]" dev sh -c 'cargo run ...'
  docker compose run --rm -e SAF_LOG="pta::solve[worklist],checker[reasoning]" dev sh -c '...'
  docker compose run --rm -e SAF_LOG=all dev sh -c '...'
  docker compose run --rm -e SAF_LOG="all,-pta::solve[worklist]" dev sh -c '...'

**File output** (avoids mixing with other stderr):
  docker compose run --rm -e SAF_LOG="pta" -e SAF_LOG_FILE=/tmp/saf.log dev sh -c '...'

**Output DSL format:**
  [module::phase][tag] narrative | key=value key=value ...

**Value types in output:**
- 0x1a2b — SAF ID
- {0x1a,0x2b} — set
- [0x1a,0x2b] — ordered list
- bb1->bb3->bb5 — path
- main->foo — pair/edge
- +{0x3c} / -{0x3c} — set delta
- 12/50 — ratio
- Bare integers, strings, bools

**Extracting values:** grep for key= patterns, e.g.:
  grep '\[pta::solve\]\[pts\]' /tmp/saf.log | grep 'val=0x1a2b'
  grep '\[checker.*\]\[reasoning\]' /tmp/saf.log

**Adding logging to new code:**
  saf_log!(module::phase, tag, "narrative" | key=expr, key=expr);
Module and phase must be registered in saf-core/src/logging/registry.rs.
Tags are free-form — no registration needed.

**Common debug workflows:**
- Wrong PTA result → SAF_LOG=pta::solve[pts,worklist]
- Missing callgraph edge → SAF_LOG=callgraph[edge],pta::solve[pts]
- False positive/negative → SAF_LOG=checker[reasoning,path,result]
- Slow analysis → SAF_LOG=pta::solve[stats,convergence]
- Interprocedural bug → SAF_LOG=absint::interproc
```
