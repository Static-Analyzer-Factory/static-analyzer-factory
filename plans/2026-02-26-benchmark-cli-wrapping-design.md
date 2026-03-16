# Benchmark CLI Wrapping Design

**Date:** 2026-02-26
**Status:** Approved
**Plan:** 171

## Problem

saf-bench duplicates the analysis pipeline that saf-cli already abstracts:
- 5 copies of frontend loading (`LlvmFrontend::default()` + `ingest()`)
- 2 identical copies of CG refinement config + `refine()` calls
- 3 copies of PTA config construction with solver branching
- 2 copies of `DefUseGraph::build()` + `Cfg::build()`
- 2 copies of MSSA + SVFG construction

Meanwhile saf-cli encapsulates all of this behind `AnalysisDriver::build()` and `ProgramDatabase`.

## Design

### Architecture: Subprocess Wrapping

saf-bench invokes `saf run` as a subprocess per test file. No batch mode — the 90 PTABen tests are small enough that per-process overhead is negligible.

```
saf-bench (thin harness)               saf CLI (analysis engine)
┌──────────────────────────┐            ┌──────────────────────────┐
│ 1. Discover .ll files    │            │                          │
│ 2. Extract oracles       │            │                          │
│    from IR               │  spawn     │ saf run                  │
│ 3. Write bench-config    │──────────→ │   --input test.ll        │
│    (queries + flags)     │            │   --bench-config q.json  │
│ 4. Parse result JSON     │←───json──  │   --output result.json   │
│ 5. Validate oracles      │            │                          │
│    vs CLI results        │            │                          │
│ 6. Aggregate report      │            │                          │
│ 7. Clean up temp files   │            │                          │
└──────────────────────────┘            └──────────────────────────┘
```

### Separation of Concerns

- **Oracle extraction** stays in saf-bench (benchmark-specific logic)
- **Analysis execution** moves to saf-cli (general-purpose tool)
- **Communication** via JSON config/result files (clean data contract)
- **Validation** stays in saf-bench (oracle comparison logic)

### Bench Config Schema (saf-bench → saf-cli)

Written by saf-bench per test, based on extracted oracle types:

```json
{
  "alias_queries": [
    {
      "ptr_a": "0xabc...",
      "ptr_b": "0xdef...",
      "oracle_block": "0x111...",
      "oracle_function": "0x222..."
    }
  ],
  "nullness_queries": [
    {"ptr": "0xddd...", "call_site": "0xeee..."}
  ],
  "analyses": {
    "checkers": true,
    "z3_prove": true,
    "nullness": true,
    "mta": true,
    "buffer_overflow": true,
    "absint": true,
    "cspta": true,
    "fspta": true
  },
  "pta_config": {
    "field_depth": 10,
    "max_iterations": 2000000,
    "constant_indices": true,
    "z3_index_refinement": true,
    "solver": "worklist",
    "entry_point_strategy": "all_defined",
    "refinement_max_iterations": 5
  }
}
```

Fields are all optional — saf-bench only includes what's needed based on detected oracle types. The CLI skips analyses not requested.

### Result Schema (saf-cli → saf-bench)

```json
{
  "success": true,
  "error": null,

  "stats": {
    "pta_solve_secs": 0.123,
    "total_secs": 0.456,
    "refinement_iterations": 3,
    "defuse_build_secs": 0.01,
    "valueflow_build_secs": 0.05
  },

  "alias_results": [
    {
      "ptr_a": "0xabc...",
      "ptr_b": "0xdef...",
      "ci": "MayAlias",
      "cs": "NoAlias",
      "fs": null,
      "ps": null,
      "best": "NoAlias"
    }
  ],

  "checker_findings": [
    {
      "check": "memory-leak",
      "alloc_site": "0x123...",
      "severity": "high",
      "call_sites": ["0x456..."],
      "path": [{"location": "...", "event": "..."}]
    }
  ],

  "assertion_results": [
    {"call_site": "0x789...", "kind": "svf_assert", "proved": true}
  ],

  "interval_results": [
    {
      "call_site": "0xaaa...",
      "left": "0xbbb...",
      "right": "0xccc...",
      "left_interval": "[3,3]",
      "right_interval": "[3,3]",
      "overlap": true
    }
  ],

  "nullness_results": [
    {"ptr": "0xddd...", "call_site": "0xeee...", "may_null": false}
  ],

  "buffer_findings": [
    {
      "ptr": "0xfff...",
      "function": "test_func",
      "kind": "buffer-overflow",
      "description": "..."
    }
  ],

  "mta_results": {
    "interleaving": [
      {"thread_id": 1, "call_site": "0x111...", "interleaved": true}
    ],
    "thread_contexts": [
      {"thread_id": 1, "exists": true}
    ],
    "tct_access": [
      {"thread_id": 1, "call_site": "0x222...", "accessible": true}
    ]
  }
}
```

All IDs are `0x` + 32 hex chars (u128), matching SAF's standard ID format.

### Multi-Resolution Alias Chain

For alias queries, the CLI runs the chain CI → CS → FS → path-sensitive and returns per-level results. saf-bench's validator applies its own precision logic (e.g., use `best` or inspect individual levels).

The CLI only runs higher levels if lower levels return MayAlias and the corresponding analysis is enabled in `analyses`.

### Per-Suite Adaptation

**PTABen** (oracle-based validation):
- Full bench-config with alias queries, nullness queries, analysis flags
- Full result JSON with all sections
- saf-bench validates each oracle against results

**CruxBC** (scalability / performance):
- Minimal bench-config: `{"analyses": {"checkers": true, "fspta": true}, "pta_config": {...}}`
- Result: mainly `stats` + `success/error`
- saf-bench measures wall-clock time and peak RSS externally (via `wait4` / `getrusage` on the child process)

**SV-COMP / Juliet** (verdict-based):
- bench-config: `{"analyses": {"absint": true, "buffer_overflow": true, "nullness": true}}`
- Result: `numeric_findings` + `nullness_results`
- saf-bench compares against expected verdicts from YAML task definitions

### Temp File Management

saf-bench creates a temp directory (via `tempfile::TempDir`) at suite start. All bench-config and result JSON files are written into this directory. The `TempDir` is dropped (cleaned up) when the suite finishes or on Ctrl-C (via RAII).

```rust
let temp_dir = tempfile::TempDir::new()?;
for test in tests {
    let config_path = temp_dir.path().join(format!("{}-config.json", test.name));
    let result_path = temp_dir.path().join(format!("{}-result.json", test.name));
    write_bench_config(&config_path, &oracles)?;
    spawn_saf_run(&test.input, &config_path, &result_path)?;
    let result = read_bench_result(&result_path)?;
    validate(test, result);
}
// temp_dir dropped here — all files cleaned up
```

### Required CLI Changes

| Change | Complexity | Description |
|--------|-----------|-------------|
| `--bench-config <path>` arg | Small | New arg in `commands.rs` |
| `BenchConfig` serde struct | Medium | Parse bench-config JSON |
| `BenchResult` serde struct | Medium | Serialize result JSON |
| `constant_indices` CLI arg | Small | Expose existing `PtaConfig` field |
| `z3_index_refinement` CLI arg | Small | Expose existing field |
| `field_depth` as numeric arg | Small | Currently enum, needs number |
| `entry_point_strategy` CLI arg | Small | Expose `RefinementConfig` field |
| Alias query execution | Medium | Run `query_alias(p, q)` per pair at CI/CS/FS/PS levels |
| **Wire nullness analysis** | Medium-Large | `NullnessResult` exists in saf-analysis, needs driver wiring |
| **Wire MTA analysis** | Large | Thread analysis exists in saf-analysis, needs driver wiring |
| **Wire interval queries** | Medium | Expose per-value abstract interp intervals |
| Assertion result output | Medium | Per-call-site Z3 verdicts |
| Raw ValueId in output | Medium | All findings include hex ValueId, not just names |

### saf-bench Code Reduction

After migration, saf-bench removes:
- All `LlvmFrontend` + `ingest()` calls (5 locations)
- All `RefinementConfig` + `refine()` calls (2 identical copies)
- All `PtaConfig` construction + solver dispatch (3 copies)
- All `DefUseGraph::build()` + `Cfg::build()` (2 copies)
- All MSSA + SVFG construction (2 copies)
- All direct analysis invocations

Estimated reduction: ~60-70% of ptaben.rs and cruxbc.rs.

saf-bench retains:
- Test discovery (directory scanning)
- Oracle extraction from IR
- Bench-config generation
- Subprocess management + parallelism
- Result validation (oracle comparison)
- Reporting / aggregation (`SuiteSummary`, JSON, tables)
- Timing/memory measurement (CruxBC)
- Temp file lifecycle

### Risks and Mitigations

**Debuggability loss:** When a test fails, the harness only sees the JSON result, not raw intermediate structures. Mitigation: include a `diagnostics` section in the result JSON with additional context (e.g., PTA stats, SVFG node counts) that aids debugging.

**Configuration drift:** bench-config and CLI driver config may diverge over time. Mitigation: the `BenchConfig` struct is a subset of `DriverConfig` — changes to one should be reflected in the other. Tests validate round-trip.

**Docker build ordering:** saf-bench spawns `saf` binary which must be compiled first. Mitigation: the Docker entrypoint already builds all binaries. `cargo build --release -p saf-cli` before running benchmarks.
