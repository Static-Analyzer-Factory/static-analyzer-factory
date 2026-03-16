# Plan 180 — Structured Debug Logging System

**Design spec:** `docs/superpowers/specs/2026-03-17-saf-logging-design.md`

## Summary

Build a structured, runtime-controlled debug logging system for AI agents. The `saf_log!` macro emits tracing events formatted as a human-readable DSL (`[module::phase][tag] narrative | key=value`). Filtering via `SAF_LOG` env var. Clean-sweep migration of all debug eprintln, ad-hoc env vars, and compile-time `solver-stats` feature gate.

## Codebase Inspection Findings

### Migration Targets (saf-analysis only)

| File | Count | Guard Mechanism | Notes |
|------|-------|----------------|-------|
| `absint/transfer.rs` | 27 | `debug_trace` fn param (25), `SAF_TRACE_STORE` (1), conditional (1) | Largest concentration |
| `absint/interprocedural.rs` | 12 | `SAF_INTERPROC_DEBUG` (2), `SAF_RECURSION_DEBUG` (10) | |
| `cg_refinement.rs` | 9 | `#[cfg(feature = "solver-stats")]` | Stats output |
| `pta/context.rs` | 4 | None (test code) | Skip — test-only |
| `pta/value_origin.rs` | 2 | `SAF_PS_DEBUG` | Not in original spec — discovered during inspection |
| **Subtotal eprintln** | **54** | | 50 non-test migration targets |

| File | Count | Type | Notes |
|------|-------|------|-------|
| `pipeline.rs` | 11 | tracing info/debug/warn | Incremental pipeline logging |
| `pta/solver_stats.rs` | 4 | tracing debug | Solver profiling summary |
| `pta/solver.rs` | 2 | tracing debug | HVN preprocessing |
| `checkers/pathsens_runner.rs` | 2 | tracing debug | UAF/multi-reach filtering |
| `checkers/site_classifier.rs` | 1 | tracing debug | Null source classification |
| `svfg/optimize.rs` | 1 | tracing debug | MemPhi removal count |
| **Subtotal tracing** | **21** | | |

**Grand total: 71 call sites to migrate** (50 eprintln + 21 tracing)

### Feature Gates to Remove

| File | `#[cfg(feature = "solver-stats")]` count |
|------|------------------------------------------|
| `pta/solver.rs` | ~60 |
| `cg_refinement.rs` | ~14 |
| `pta/mod.rs` | 2 |
| `Cargo.toml` (saf-analysis) | 1 |
| `Cargo.toml` (saf-bench) | 1 |
| **Total** | **~78** |

### Env Vars to Remove

| Env Var | File | Calls |
|---------|------|-------|
| `SAF_INTERPROC_DEBUG` | absint/interprocedural.rs | 2 |
| `SAF_RECURSION_DEBUG` | absint/interprocedural.rs | 10 |
| `SAF_TRACE_STORE` | absint/transfer.rs | 1 |
| `SAF_PS_DEBUG` | pta/value_origin.rs | 2 |
| **Total** | | **15** |

### Key Types for SafLogValue

All defined via `define_id_type!` macro in `saf-core/src/ids.rs` (u128 newtypes):
- `ValueId`, `LocId`, `BlockId`, `FunctionId`, `InstId`, `ObjId`, `ModuleId`, `TypeId`, `FileId`, `ProgramId`
- `MemAccessId` (u128 newtype in `saf-analysis/src/mssa/access.rs`)
- `SvfgNodeId` (enum: `Value(ValueId)` | `MemPhi(MemAccessId)`)
- `CallGraphNode` (enum: `Function(FunctionId)` | `External { name, func }` | `IndirectPlaceholder { site }`)
- `BTreeSet<LocId>` (points-to sets in normalized form)

### Tracing Init (3 binaries to update)

| Binary | Current Init | Writer |
|--------|-------------|--------|
| `saf-cli/src/main.rs` | `fmt().with_env_filter("info").init()` (+ json variant) | stdout |
| `saf-bench/src/main.rs` | `fmt().with_env_filter(EnvFilter::from_default_env()...).with_target(false).with_writer(stderr).init()` | stderr |
| `saf-trace/src/main.rs` | `fmt().with_env_filter(try_from_default_env().unwrap_or("info")).with_writer(stderr).init()` | stderr |

### Cargo Dependencies

| Crate | Has `tracing` | Has `tracing-subscriber` |
|-------|---------------|--------------------------|
| saf-core | Yes | **No** (needs `logging-subscriber` feature) |
| saf-analysis | Yes | No |
| saf-bench | Yes | Yes |
| saf-cli | Yes | Yes (implicit) |
| saf-trace | Yes | Yes |

---

## Implementation Plan

### Phase 1: Core Infrastructure (saf-core/src/logging/)

**Task 1.1 — Create `registry.rs`** with `saf_log_module!` macro

The macro generates a hidden module hierarchy for compile-time validation:

```rust
// saf_log_module! { pta { constraint, solve, hvn, scc, lcd } }
// expands to:
mod __saf_log_registry {
    pub mod pta {
        pub struct constraint;
        pub struct solve;
        // ...
    }
}
```

Register all modules/phases from the spec:
- `pta` → `constraint`, `solve`, `hvn`, `scc`, `lcd`
- `callgraph` → `build`, `refine`
- `cfg` → `build`
- `svfg` → `build`, `optimize`
- `valueflow` → `build`, `query`
- `defuse` → `build`
- `mssa` → `build`
- `checker` → `memleak`, `uaf`, `nullptr`, `pathsens`
- `absint` → `interproc`, `transfer`, `escape`, `nullness`
- `frontend` → `ingest`
- `pipeline` → `constraint`, `incremental`, `analysis`

**Files:** `+crates/saf-core/src/logging/registry.rs`

---

**Task 1.2 — Create `value.rs`** with `SafLogValue` trait + impls

```rust
pub trait SafLogValue {
    fn fmt_saf_log(&self, buf: &mut String);
}
```

Implement for:
- Primitives: `u128`, `u64`, `u32`, `usize`, `i64`, `i32`, `isize`, `f64`, `bool`
- Strings: `&str`, `String`
- All ID newtypes via blanket impl on `EntityId` trait → `0x` + hex
- Collections: `BTreeSet<T>` → `{a,b,c}`, `Vec<T>` / `&[T]` → `[a,b,c]`
- Newtype wrappers: `SafPair<T>` → `a->b`, `SafRatio` → `n/m`, `PtsDelta<T>` → `+{a,b}` / `-{a,b}`
- `Duration` → seconds with 3dp (`1.234s`)
- `Option<T>` → value or `none`

Note: `SvfgNodeId`, `CallGraphNode`, `MemAccessId` live in saf-analysis and cannot be in saf-core. These impls will be added in Phase 4 (in saf-analysis, since the trait is defined in saf-core and types are in saf-analysis — orphan rule satisfied).

**Files:** `+crates/saf-core/src/logging/value.rs`

---

**Task 1.3 — Create `formatter.rs`** for DSL line formatting

Takes module, phase, tag, narrative, and pre-formatted key-value string → outputs:
```
[module::phase][tag] narrative | key=value key=value
```

Handles the three variants: full form, narrative-only, keys-only.

**Files:** `+crates/saf-core/src/logging/formatter.rs`

---

**Task 1.4 — Create `mod.rs`** with `saf_log!` macro

The macro:
1. References `__saf_log_registry::$module::$phase` for compile-time validation
2. Serializes key-value pairs via `SafLogValue::fmt_saf_log()`
3. Emits a `tracing::event!(Level::TRACE, target: "saf_debug", ...)` with structured fields

Three forms:
```rust
saf_log!(pta::solve, worklist, "pts grew" | val=node_id, delta=&added);
saf_log!(pta::solve, convergence, "fixpoint reached");
saf_log!(pta::solve, stats, | iter=12, worklist=342);
```

Export the macro, `SafLogValue` trait, formatter, and registry from the module.

**Files:** `+crates/saf-core/src/logging/mod.rs`, `~crates/saf-core/src/lib.rs` (add `pub mod logging;`)

---

**Task 1.5 — Create `subscriber/filter.rs`** for `SAF_LOG` env var parsing

Parse the filter grammar:
- `pta::solve[worklist,scc]` → match module="pta", phase="solve", tag in {"worklist","scc"}
- `pta` → match module="pta", any phase, any tag
- `*[convergence]` → any module, any phase, tag="convergence"
- `all` → match everything
- `-prefix` → exclude
- Comma-separated union

Data structure: `SafLogFilter` with `includes: Vec<FilterRule>` and `excludes: Vec<FilterRule>`.

`FilterRule`:
```rust
struct FilterRule {
    module: Option<String>,  // None = wildcard
    phase: Option<String>,
    tags: Option<Vec<String>>,  // None = wildcard
}
```

`matches(module: &str, phase: &str, tag: &str) -> bool`:
1. Check if any include rule matches
2. Check if any exclude rule matches
3. Return `included && !excluded`

**Files:** `+crates/saf-core/src/logging/subscriber/filter.rs`

---

**Task 1.6 — Create `subscriber/layer.rs`** with `SafLogLayer`

A `tracing_subscriber::Layer` that:
1. Extracts `saf_module`, `saf_phase`, `saf_tag`, `saf_narrative`, `saf_kv` fields from events with target `"saf_debug"`
2. Checks against `SafLogFilter`
3. If matched: writes formatted DSL line to writer (stderr or file)
4. If not matched: discards

Writer is `Box<dyn Write + Send>` — stderr by default, file if `SAF_LOG_FILE` is set.

**Files:** `+crates/saf-core/src/logging/subscriber/layer.rs`

---

**Task 1.7 — Create `subscriber/mod.rs`** with `init()`

```rust
pub fn init() -> SafLogLayer {
    let filter = match std::env::var("SAF_LOG") {
        Ok(spec) => SafLogFilter::parse(&spec),
        Err(_) => SafLogFilter::none(),  // no-op
    };
    let writer = match std::env::var("SAF_LOG_FILE") {
        Ok(path) => Box::new(File::create(path).expect("...")),
        Err(_) => Box::new(std::io::stderr()),
    };
    SafLogLayer::new(filter, writer)
}
```

**Files:** `+crates/saf-core/src/logging/subscriber/mod.rs`

---

**Task 1.8 — Update `saf-core/Cargo.toml`**

Add optional `logging-subscriber` feature:
```toml
[features]
logging-subscriber = ["tracing-subscriber"]

[dependencies]
tracing-subscriber = { workspace = true, optional = true }
```

**Files:** `~crates/saf-core/Cargo.toml`

---

### Phase 2: Binary Integration

**Task 2.1 — Update saf-cli/src/main.rs**

Replace current init with layered subscriber:
```rust
use tracing_subscriber::prelude::*;
let saf_layer = saf_core::logging::subscriber::init();
tracing_subscriber::registry()
    .with(if cli.json_errors {
        tracing_subscriber::fmt::layer().json().with_filter(EnvFilter::new("info")).boxed()
    } else {
        tracing_subscriber::fmt::layer().with_filter(EnvFilter::new("info")).boxed()
    })
    .with(saf_layer)
    .init();
```

**Files:** `~crates/saf-cli/src/main.rs`, `~crates/saf-cli/Cargo.toml` (add `saf-core/logging-subscriber`)

---

**Task 2.2 — Update saf-bench/src/main.rs**

Replace current init with layered subscriber incorporating `SafLogLayer`.

**Files:** `~crates/saf-bench/src/main.rs`, `~crates/saf-bench/Cargo.toml` (add `saf-core/logging-subscriber`)

---

**Task 2.3 — Update saf-trace/src/main.rs**

Same pattern as Task 2.2.

**Files:** `~crates/saf-trace/src/main.rs`, `~crates/saf-trace/Cargo.toml` (add `saf-core/logging-subscriber`)

---

### Phase 3: Migrate absint Module (39 calls)

**Task 3.1 — Migrate `absint/interprocedural.rs`** (12 calls)

Convert 12 `eprintln!` calls to `saf_log!`:
- 2 guarded by `SAF_INTERPROC_DEBUG` → `absint::interproc` with tags `context`, `stats`
- 10 guarded by `SAF_RECURSION_DEBUG` → `absint::interproc` with tags `convergence`, `context`, `delta`

Remove `SAF_INTERPROC_DEBUG` and `SAF_RECURSION_DEBUG` env var checks.

Mapping:
| Line(s) | Current Tag | New saf_log! |
|---------|-------------|-------------|
| 942 | `[INTERPROC] Analyzing function` | `saf_log!(absint::interproc, context, "analyzing function" \| func=name, summaries=count)` |
| 972 | `[INTERPROC] Computed summary` | `saf_log!(absint::interproc, result, "computed summary" \| func=name, ret_interval=interval)` |
| 1438 | `[RECURSION] caller callee arg` | `saf_log!(absint::interproc, context, "recursive arg extraction" \| caller=name, arg_idx=i, vid=vid, final_val=val)` |
| 1628 | `[RECURSION] iter func param` | `saf_log!(absint::interproc, convergence, "SCC iteration" \| iter=n, func=name)` |
| 1661 | `[RECURSION] ret_operand` | `saf_log!(absint::interproc, convergence, "return tracking" \| iter=n, func=name)` |
| 1674 | `[RECURSION] new_summary` | `saf_log!(absint::interproc, convergence, "SCC summary" \| iter=n, func=name)` |
| 1853 | `[RECURSION] unreachable` | `saf_log!(absint::interproc, filter, "block unreachable" \| block=block_id)` |
| 1859-1919 | `[RECURSION] block processing` | `saf_log!(absint::interproc, delta, ...)` for block entry/exit/instruction |

**Files:** `~crates/saf-analysis/src/absint/interprocedural.rs`

---

**Task 3.2 — Migrate `absint/transfer.rs`** (27 calls)

The `debug_trace` parameter pattern: currently a `bool` passed through function signatures. After migration, remove the parameter from all functions that use it — `saf_log!` handles filtering at runtime.

Functions that take `debug_trace: bool` (need signature changes):
- `transfer_store()`
- `transfer_gep_with_summaries()`
- `transfer_store_with_pta()`
- `transfer_load_with_pta()`
- `inline_function_for_summary()`
- And all callers of these functions (trace the call chain upward)

Convert 27 calls:
- 1 `SAF_TRACE_STORE` → `saf_log!(absint::transfer, constraint, ...)`
- 25 `debug_trace` → `saf_log!(absint::transfer, ...)` with tags: `constraint` (stores), `pts` (loads), `context` (GEP resolution), `delta` (inline propagation)
- 1 conditional → `saf_log!(absint::transfer, ...)`

Remove `SAF_TRACE_STORE` env var check. Remove `debug_trace` parameter from all function signatures + call sites.

**Files:** `~crates/saf-analysis/src/absint/transfer.rs`, plus callers (likely `interprocedural.rs`, `escape.rs`, `nullness.rs` or wherever `debug_trace` is passed)

---

### Phase 4: Migrate PTA + CG + Remove solver-stats (19 calls + ~78 feature gates)

**Task 4.1 — Migrate `pta/solver_stats.rs`** (4 tracing::debug)

Convert `print_summary()` method's 4 `tracing::debug!` events into grouped `saf_log!` calls:

```
[pta::solve][stats] overview | copy=N load=N store=N gep=N value_pops=N loc_pops=N
[pta::solve][stats] timing | copy=1.2s load=0.8s store=0.5s gep=0.3s total=2.8s
[pta::solve][stats] deep | proc_value=1.5s handler_sum=1.2s proc_loc=0.8s
[pta::solve][stats] find_rep | calls=N hops=N max_chain=N avg=0.5
```

Remove `#[cfg(feature = "solver-stats")]` from `print_summary()` — it's now runtime-gated.

**BUT**: Keep `SolverStats` struct and its `start_section()`/`end_section()` instrumentation methods. The struct fields still need to be populated at runtime. The question is whether to keep the feature gate on the *collection* code (the ~60 gates in solver.rs).

**Decision**: Remove the feature gate entirely. The collection overhead is trivial (a few `Instant::now()` + `Duration += elapsed` per constraint handler invocation). The solver processes millions of constraints; the overhead of timing instrumentation is <1% based on prior profiling (Plan 128). Making it always-on means `SAF_LOG=pta::solve[stats]` works without recompilation.

**Files:** `~crates/saf-analysis/src/pta/solver_stats.rs`

---

**Task 4.2 — Remove solver-stats feature gates from `pta/solver.rs`** (~60 gates)

Remove all `#[cfg(feature = "solver-stats")]` annotations. The `SolverStats` field, timing calls, and counter increments become unconditional. This is the largest single edit in the plan.

Strategy: `stats` field becomes non-optional, `stats_mut()` / `stats_borrow_mut()` accessors become plain methods.

**Files:** `~crates/saf-analysis/src/pta/solver.rs`

---

**Task 4.3 — Migrate `cg_refinement.rs`** (9 eprintln + ~14 feature gates)

Convert 9 `eprintln!` stats lines (behind `solver-stats` gate) into:
```
[callgraph::refine][stats] profile | init=0.5s solve=1.2s cg_loop=0.3s normalize=0.1s hvn_expand=0.2s total=2.3s iterations=5
```

Remove `#[cfg(feature = "solver-stats")]` from all 14 gates. Timer variables become unconditional.

**Files:** `~crates/saf-analysis/src/cg_refinement.rs`

---

**Task 4.4 — Migrate `pta/value_origin.rs`** (2 eprintln)

Convert 2 `SAF_PS_DEBUG` calls:
```
saf_log!(pta::solve, reasoning, "path-sensitive rename" | caller=name, callee=name, ...);
saf_log!(pta::solve, result, "path-sensitive result" | alias=result, paths_total=n, paths_feasible=n);
```

Remove `SAF_PS_DEBUG` env var check.

**Files:** `~crates/saf-analysis/src/pta/value_origin.rs`

---

**Task 4.5 — Add SafLogValue impls for saf-analysis types**

Implement `SafLogValue` for types only available in saf-analysis:
- `SvfgNodeId` → `val:0x1a2b` or `memphi:0x3c4d`
- `CallGraphNode` → `func:main` or `ext:malloc` or `indirect:0x1a2b`
- `MemAccessId` → `0x1a2b`

**Files:** `~crates/saf-analysis/src/svfg/mod.rs` or a new `logging_impls.rs`

---

**Task 4.6 — Remove solver-stats feature from Cargo.toml**

- Remove `solver-stats = []` from `crates/saf-analysis/Cargo.toml`
- Remove `solver-stats = ["saf-analysis/solver-stats"]` from `crates/saf-bench/Cargo.toml`
- Remove from `pta/mod.rs`: `#[cfg(feature = "solver-stats")]` on module declaration and re-export

**Files:** `~crates/saf-analysis/Cargo.toml`, `~crates/saf-bench/Cargo.toml`, `~crates/saf-analysis/src/pta/mod.rs`

---

### Phase 5: Migrate Remaining Analysis Calls (5 tracing calls)

**Task 5.1 — Migrate `checkers/pathsens_runner.rs`** (2 tracing::debug)

```
saf_log!(checker::pathsens, filter, "temporal filter" | removed=count);
saf_log!(checker::pathsens, filter, "joint feasibility filter" | removed=count);
```

**Files:** `~crates/saf-analysis/src/checkers/pathsens_runner.rs`

---

**Task 5.2 — Migrate `checkers/site_classifier.rs`** (1 tracing::debug)

```
saf_log!(checker::pathsens, stats, | null_sources=count);
```

**Files:** `~crates/saf-analysis/src/checkers/site_classifier.rs`

---

**Task 5.3 — Migrate `svfg/optimize.rs`** (1 tracing::debug)

```
saf_log!(svfg::optimize, stats, "MemPhi reduction" | removed=count);
```

**Files:** `~crates/saf-analysis/src/svfg/optimize.rs`

---

**Task 5.4 — Migrate `pta/solver.rs` HVN calls** (2 tracing::debug)

```
saf_log!(pta::hvn, stats, "HVN preprocessing" | classes=n, removed=n, before=n, after=n);
```

**Files:** `~crates/saf-analysis/src/pta/solver.rs` (already touched in Task 4.2)

---

**Task 5.5 — Migrate `pipeline.rs`** (11 tracing calls)

Decide per-call: some are operational (`info!` for incremental progress) and some are debug.

Migrate to `saf_log!`:
- `debug!` constraint cache hit/miss → `saf_log!(pipeline::constraint, stats, ...)`
- `info!` incremental diff/solve/rebuild → `saf_log!(pipeline::incremental, stats, ...)`

Keep as `tracing::warn!`:
- Pool creation failure (operational warning, not debug)
- Constraint cache save failure (operational warning)

**Files:** `~crates/saf-analysis/src/pipeline.rs`

---

### Phase 6: Documentation + Cleanup

**Task 6.1 — Update CLAUDE.md**

Add the SAF Debug Logging section from the spec (Section 8). Include:
- `SAF_LOG` env var syntax
- `SAF_LOG_FILE` for file output
- Output DSL format with value types
- Common debug workflows
- How to add logging to new code

Also update the spec's migration table to reflect the accurate count of 50 eprintln + 21 tracing = 71 total (not 56+14=70 as estimated).

**Files:** `~CLAUDE.md`

---

**Task 6.2 — Update design spec**

Add `SAF_PS_DEBUG` to migration table and `pta::solve` `reasoning` tag for path-sensitive alias. Add `debug_trace` function parameter pattern to migration notes.

**Files:** `~docs/superpowers/specs/2026-03-17-saf-logging-design.md`

---

### Phase 7: Verification

**Task 7.1 — Format and lint**

```bash
make fmt && make lint
```

**Files:** none (verification only)

---

**Task 7.2 — Run all tests**

```bash
make test 2>&1 | tee /tmp/test-output.txt
```

**Files:** none (verification only)

---

**Task 7.3 — Smoke test SAF_LOG**

Run a simple analysis with logging enabled to verify output format:

```bash
docker compose run --rm -e SAF_LOG=all dev sh -c \
  'cargo run --release -p saf-cli -- run tests/fixtures/llvm/e2e/simple_alias.ll 2>/tmp/saf-all.log && head -50 /tmp/saf-all.log'
```

Verify:
1. Output follows DSL format: `[module::phase][tag] narrative | key=value`
2. No output when `SAF_LOG` is not set
3. Filtering works: `SAF_LOG=pta::solve[stats]` only shows solver stats
4. `SAF_LOG_FILE` writes to file

**Files:** none (verification only)

---

## Task Dependency Graph

```
Phase 1 (Tasks 1.1-1.8) — Core infrastructure, all independent except:
  1.4 (mod.rs/macro) depends on 1.1 (registry), 1.2 (value), 1.3 (formatter)
  1.6 (layer) depends on 1.5 (filter), 1.3 (formatter)
  1.7 (subscriber/mod) depends on 1.5, 1.6

Phase 2 (Tasks 2.1-2.3) — depends on Phase 1 complete

Phase 3 (Tasks 3.1-3.2) — depends on Phase 1 (needs saf_log! macro available)
Phase 4 (Tasks 4.1-4.6) — depends on Phase 1
Phase 5 (Tasks 5.1-5.5) — depends on Phase 1

  Phase 3, 4, 5 are independent of each other (parallelizable)

Phase 6 (Tasks 6.1-6.2) — depends on Phase 3-5 (needs final migration counts)
Phase 7 (Tasks 7.1-7.3) — depends on all prior phases

Total: 25 tasks across 7 phases
```

## Estimated Scope

- **New files:** 7 (logging module: mod.rs, registry.rs, value.rs, formatter.rs, subscriber/mod.rs, subscriber/filter.rs, subscriber/layer.rs)
- **Modified files:** ~16 (3 binaries + 3 Cargo.toml + 8 analysis files + CLAUDE.md + spec)
- **Lines added:** ~800-1000 (logging infrastructure)
- **Lines removed:** ~200 (eprintln calls, env var guards, feature gates)
- **Net change:** ~600-800 lines added
- **Feature gates removed:** ~78
