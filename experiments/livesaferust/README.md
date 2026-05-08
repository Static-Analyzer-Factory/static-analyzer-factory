# LiveSafeRust POC

This directory contains a small architecture-validation experiment for a
SAF-enhanced Rust LSP:

```text
Rust edit event
  -> source sidecar facts
  -> SAF-backed facts or summaries
  -> LSP diagnostics
```

The synthetic experiment uses a lightweight JavaScript source sidecar. The
real-crate experiment uses `syn-sidecar/` to parse Rust syntax and then feeds
those facts into the same summary/invalidation engine. The latest prototype adds a SAF-backed path
that invokes the repository's SAF Python SDK on LLVM IR and feeds SAF JSON into
the LiveSafeRust cache/worklist layer.

## What This Is, And Is Not

This is not evidence that a full Rust analyzer is already implemented. It is a
small check of the proposed integration contract:

- function-level facts and summaries can be cached by source content hash;
- changed summaries can invalidate callers through a reverse call graph;
- diagnostics can be emitted in an LSP-compatible shape;
- the parser/source of facts is replaceable (`lightweight-js` now, `syn` or
  rust-analyzer/HIR later).

The synthetic performance table is a scalability sanity check. The duct run adds
one real-crate anchor against `cargo check`, Clippy, and syn fact extraction.

## Run

```bash
node experiments/livesaferust/livesaferust_poc.mjs --workload layered --scale 5000 --chain-depth 24 --fanout 5
```

This is the paper-scale synthetic run. The script's default scale is smaller
(`--scale 800`), which produces 931 functions for the same layered structure.
The paper-scale command above uses 5000 unrelated helpers and produces 5131
functions: 5000 helpers, 5 x (24 wrappers + 1 dispatch), five shared utility
functions, and the modeled `extern system` sink.

| Scenario | Functions | Changed | Recomputed | Summary Changed | Summary Stable | Cache Hits | Findings | Time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| clean cold | 5131 | 5131 | 5131 | 5007 | 130 | 0 | 0 | 362.471 ms |
| unrelated incremental | 5131 | 1 | 1 | 0 | 1 | 5130 | 0 | 13.864 ms |
| vulnerable incremental | 5131 | 1 | 122 | 121 | 1 | 5009 | 5 | 37.638 ms |
| vulnerable full | 5131 | 5131 | 5131 | 5128 | 9 | 0 | 5 | 368.320 ms |

Useful variants:

```bash
# Original mostly-independent helper workload.
node experiments/livesaferust/livesaferust_poc.mjs --workload flat --scale 800

# Run both workloads and assertions.
node experiments/livesaferust/livesaferust_poc.mjs --workload all --scale 800 --chain-depth 24 --fanout 5
```

Outputs are written to `experiments/livesaferust/out/<workload>/`:

- `clean.rs`, `vulnerable.rs`, `unrelated_edit.rs`: generated Rust workloads
- `results.json`: timing, cache, finding, and assertion metrics
- `diagnostics.json`: LSP-style diagnostics for the vulnerable version
- `semantic-facts.json`: SAF-oriented function/call/summary facts

`experiments/livesaferust/out/summary.json` summarizes all workloads from the
latest run.

## Workloads

`flat` is the original sanity workload: many unrelated helper functions plus a
short taint path.

`layered` is the more useful workload. It contains many unrelated helper
functions plus five independent 24-hop wrapper branches:

```text
read_user()
  -> normalize_command(...)
  -> b00_wrap_00(...) / ... / b04_wrap_00(...)
  -> ...
  -> b00_wrap_23(...) / ... / b04_wrap_23(...)
  -> b00_dispatch(...) / ... / b04_dispatch(...)
  -> run_command(...)
```

The clean version sanitizes input in `normalize_command`. The vulnerable edit
changes only `normalize_command`, returning raw input. A correct incremental
summary engine must propagate that summary change through all fan-out branches
and recompute `main`, not merely the directly edited function.

## R13 Edit Sequence

The paper-facing live-LSP experiment is generated and run with:

```bash
python3 experiments/livesaferust/run_r13_edit_sequence.py
```

It expands the RUSTSEC-2024-0350 / CVE-2024-35186-style path-traversal case,
generates 12 realistic edits, runs each edit incrementally and cold through SAF
LLVM-22, and writes:

- `cases/rustsec_2024_0350_expanded.rs`
- `cases/r13_edits/*.rs`
- `out/saf/r13/edit-sequence/summary.json`
- `out/saf/r13/edit-sequence/NOTES.md`

The cache key is a semantic LLVM-IR hash: it drops comments, `source_filename`,
and debug metadata, then alpha-normalizes local named SSA value identifiers
within each function. Global function identifiers and call structure are
retained, so a local-variable rename can hit cache while helper renames and
call-graph changes remain visible.

Current result:

| Metric | Value |
| --- | ---: |
| edits | 12 |
| cache-hit ratio | 0.333 |
| all incremental findings match cold findings | true |
| median incremental orchestration | 3139.633 ms |
| p95 incremental orchestration | 3320.066 ms |

All five R13 gates pass: no-op edits avoid SAF re-invocation, body-only edits
rerun SAF with stable summaries, summary-changing edits cut off within five
source-level hops, incremental/cold findings agree, and the sanitizer edit
removes then restores the vulnerability finding.

## Method Design Notes

The actual paper claim needs the rust-analyzer/Salsa integration story, not just
the synthetic timing table. See:

- `SALSA_ADAPTER.md`: proposed rust-analyzer query integration and invalidation
  invariants.
- `PAPER_OUTLINE.md`: proposal-shaped gap, method, preliminary result, threats,
  and next evaluation steps.
- `baseline_harness.sh`: a cargo-facing baseline helper to run on a real Rust
  crate.
- `lsp_latency_probe.py`: a minimal rust-analyzer LSP client that measures
  `didChange` to `publishDiagnostics` latency on a real Rust machine.
- `run_r13_edit_sequence.py`: the 12-edit SAF-backed live experiment.
- `saf_integration/run_saf.py`: compiles Rust/C to LLVM IR, invokes SAF in
  Docker, and emits LiveSafeRust-compatible SAF facts.
- `paper/live_saferust_nit.tex`: a LaTeX-ready NIT draft skeleton with Theorem 1
  centered in the method section.

## Baseline Probes

On a machine with `cargo`, `rust-analyzer`, and optionally `hyperfine`:

```bash
experiments/livesaferust/baseline_harness.sh /path/to/duct \
  experiments/livesaferust/out/baseline/duct

python3 experiments/livesaferust/lsp_latency_probe.py \
  --crate /path/to/duct \
  --file src/lib.rs \
  --wait-initial \
  --out experiments/livesaferust/out/baseline/duct/ra-lsp-latency.json
```

For the current local run on `duct`, see:

```text
experiments/livesaferust/out/baseline/duct/baseline-summary.json
```

The cargo and syn-sidecar rows are usable. The rust-analyzer nonempty
`publishDiagnostics` latency is currently inconclusive in the minimal client.

## syn Sidecar

Parse a Rust file into syn facts:

```bash
cd experiments/livesaferust/syn-sidecar
cargo run -- ../out/layered/vulnerable.rs > ../out/layered/syn-facts.json
```

Feed syn facts into LiveSafeRust's summary engine:

```bash
node experiments/livesaferust/livesaferust_poc.mjs \
  --syn-facts experiments/livesaferust/out/baseline/duct/syn-facts.json \
  --syn-facts-edited experiments/livesaferust/out/baseline/duct/syn-facts-edited.json \
  --syn-name duct
```

That path parses real Rust syntax with `syn` and emits function/call facts in
the same spirit as `semantic-facts.json`. The next implementation step is to
swap this for rust-analyzer/HIR facts while keeping the incremental summary
engine stable.

## Minimal Diagnostic Case

For a nonzero diagnostic payload:

```bash
experiments/livesaferust/syn-sidecar/target/debug/livesaferust-syn-sidecar \
  experiments/livesaferust/cases/minimal_command_injection.rs \
  > experiments/livesaferust/out/cases/minimal-command-injection/syn-facts.json

node experiments/livesaferust/livesaferust_poc.mjs \
  --syn-facts experiments/livesaferust/out/cases/minimal-command-injection/syn-facts.json \
  --syn-name minimal-command-injection
```

The case compiles and produces one `tainted-command` diagnostic from
`env::args()` to `Command::new`.

## SAF-Backed Path

**Recommended invocation** for the `std::env::args() -> Command::new` and
`std::env::args() -> File::open` Rust cases that R11/R12 enabled:

```bash
python3 experiments/livesaferust/saf_integration/run_saf.py \
  experiments/livesaferust/cases/minimal_command_injection.rs \
  --saf-image llvm22 \
  --ensure-saf-sdk \
  --out-dir experiments/livesaferust/out/saf/minimal-command-injection
```

The two flags matter:

- `--saf-image llvm22` selects the SAF Docker image with the LLVM-22 frontend.
  The default is `llvm18`, which cannot parse `rustc` 1.95+ LLVM IR
  (`trunc nuw`, `getelementptr inbounds nuw`).
- `--ensure-saf-sdk` rebuilds the SAF Python wheel inside the container with
  `--features llvm-22` before analysis. It only adds cost on the first run per
  fresh container image; subsequent runs reuse the maturin cache.

Without these flags the script silently drops to `dev` (LLVM 18) and reports
`status: failed` with a `trunc nuw` parse error. This is the single most common
gotcha for reviewers re-running the experiment.

Current result on `minimal_command_injection.rs` (R12, after SAF patches in
`crates/saf-frontends/src/llvm/mapping.rs` and
`crates/saf-analysis/src/valueflow/builder.rs`): SAF reports one real taint
finding from `env::args()` to `Command::new`, and a semantic-IR-identical edit
hits the SAF cache without re-invoking SAF.

The earlier minimized Rust FFI case is still useful as a cross-check that does
not depend on the LLVM-22 image:

```bash
python3 experiments/livesaferust/saf_integration/run_saf.py \
  experiments/livesaferust/cases/minimal_rust_ffi_command_injection.rs \
  --rustc 1.79.0 \
  --rustc-in-docker \
  --saf-image llvm18 \
  --out-dir experiments/livesaferust/out/saf/minimal-rust-ffi-fixed
```

It reports one real SAF finding from `getenv` to `system`, validating the same
SAF-backed pipeline against a simpler IR shape.

Run the C equivalent as a cross-check:

```bash
python3 experiments/livesaferust/saf_integration/run_saf.py \
  experiments/livesaferust/cases/c_command_injection.c \
  --c-line-tables \
  --out-dir experiments/livesaferust/out/saf/c-command-injection

node experiments/livesaferust/livesaferust_poc.mjs \
  --workload saf-backed \
  --facts experiments/livesaferust/out/saf/c-command-injection/saf-facts.json \
  --saf-name c-command-injection
```

This path produces one real SAF finding from `getenv` to `system`, then emits
the same finding as an LSP-shaped LiveSafeRust diagnostic.
