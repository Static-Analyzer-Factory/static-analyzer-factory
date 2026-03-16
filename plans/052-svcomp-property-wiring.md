# Plan 052: SV-COMP Property Analyzer Wiring

## Goal

Connect SAF's existing analysis infrastructure to SV-COMP property analyzers so benchmarks return meaningful TRUE/FALSE results instead of UNKNOWN.

## Properties to Wire

| Property | Analysis | Expected Outcome |
|----------|----------|------------------|
| `unreach-call` | Z3 path reachability | TRUE if `reach_error()` unreachable from main |
| `valid-memsafety` | SVFG checkers (null-deref, UAF, double-free) | TRUE if no memory safety violations |
| `no-overflow` | Interprocedural abstract interpretation | TRUE if no signed integer overflow |
| `no-data-race` | MTA MHP analysis | TRUE if no concurrent unprotected accesses |

## Implementation

### Phase 1: Add AnalysisContext (lazy caching)

**File:** `crates/saf-bench/src/svcomp/property.rs`

Add a struct to lazily build and cache analysis results:

```rust
use std::cell::OnceCell;

pub struct AnalysisContext<'a> {
    module: &'a AirModule,
    config: &'a PropertyAnalysisConfig,
    callgraph: OnceCell<CallGraph>,
    icfg: OnceCell<Icfg>,
    pta_result: OnceCell<PtaResult>,
    svfg: OnceCell<Svfg>,
    absint_result: OnceCell<InterproceduralResult>,
    mta_result: OnceCell<MtaResult>,
}
```

### Phase 2: Wire `unreach-call`

Use `z3_utils::reachability::check_path_reachable()`:

1. Find calls to `reach_error()` / `__VERIFIER_error()`
2. Fast-path: if no error calls → TRUE
3. Check callgraph reachability from main
4. For reachable error sites, use Z3 path reachability
5. If all paths UNSAT → TRUE, else FALSE with witness

**Key API:**
```rust
use saf_analysis::z3_utils::reachability::{check_path_reachable, PathReachability};

let result = check_path_reachable(
    entry_block, error_block, func_id, module,
    config.z3_timeout_ms, config.max_guards, config.max_paths
);
```

### Phase 3: Wire `valid-memsafety`

Use path-sensitive SVFG checkers:

1. Build SVFG (requires PTA)
2. Build ResourceTable
3. Run `run_checkers_path_sensitive()` with null-deref, UAF, double-free specs
4. If feasible findings → FALSE, else TRUE

**Key API:**
```rust
use saf_analysis::checkers::{
    run_checkers_path_sensitive, PathSensitiveConfig, ResourceTable,
    spec::{null_deref, use_after_free, double_free}
};

let table = ResourceTable::new(module);
let specs = vec![null_deref(), use_after_free(), double_free()];
let result = run_checkers_path_sensitive(&specs, module, &svfg, &table, &config);
```

### Phase 4: Wire `no-overflow`

Use interprocedural abstract interpretation:

1. Run `solve_interprocedural()` for cross-function interval tracking
2. Check Add/Sub/Mul operations against signed bounds
3. If definite overflow (Error severity) → FALSE, else TRUE

**Key API:**
```rust
use saf_analysis::absint::{
    check_integer_overflow, AbstractInterpConfig, NumericSeverity
};

let result = check_integer_overflow(module, &config);
let definite = result.findings.iter()
    .any(|f| f.severity == NumericSeverity::Error);
```

### Phase 5: Wire `no-data-race`

Use MTA may-happen-in-parallel analysis:

1. Run MtaAnalysis to discover threads
2. Fast-path: single thread → TRUE
3. Collect Load/Store accesses per thread
4. For concurrent threads, check if they access same memory (via PTA aliasing)
5. If racing pair found → FALSE, else TRUE

**Key API:**
```rust
use saf_analysis::mta::{MtaAnalysis, MtaConfig};

let mta = MtaAnalysis::new(module, &callgraph, &icfg, config);
let result = mta.analyze();
// Check result.mhp_result.concurrent_at() for interleaving
```

## Files to Modify

| File | Changes |
|------|---------|
| `crates/saf-bench/src/svcomp/property.rs` | Add `AnalysisContext`, implement all 4 property analyzers |

## Dependencies (already implemented)

- `saf_analysis::z3_utils::reachability` - path feasibility
- `saf_analysis::checkers::pathsens_runner` - SVFG memory safety
- `saf_analysis::absint::checker` - integer overflow detection
- `saf_analysis::mta` - MHP analysis

## Verification

1. **Unit tests in `property.rs`:**
   - Test each property with minimal fixtures

2. **Run SV-COMP benchmarks:**
   ```bash
   make compile-svcomp-category CAT=array-examples
   make test-svcomp-category CAT=array-examples
   ```

3. **Expected improvement:**
   - Current: 1 TRUE, 117 Unknown
   - Target: 50+ TRUE/FALSE, reduced Unknown

## Estimated Scope

~300-400 lines of Rust code in a single file. No new crates or dependencies needed.
