# LiveSafeRust POC Report

Date: 2026-04-29

## Reframed Claim

Claude's critique was right: the first version over-sold a toy benchmark. This
version frames the experiment more narrowly.

The claim under test is:

> LiveSafeRust is an orchestration layer above SAF: SAF computes taint facts and
> findings from LLVM IR, while LiveSafeRust caches those facts by function-level
> content, applies the Theorem 1 invalidation rule, and surfaces the result as
> LSP diagnostics.

This is an architecture claim, not yet a broad superiority claim over cargo,
rust-analyzer, Clippy, CodeQL, or SAF's analysis precision. The new experiment
does invoke the SAF Python SDK and patches SAF so host `rustc 1.95` LLVM IR for
`std::process::Command` reaches a real SAF taint finding.

## Setup

The runnable POC has two synthetic workloads, a `syn` fact adapter for real Rust
source files, and a SAF-backed adapter:

```text
Rust/C source -> LLVM IR -> SAF Python SDK -> SAF JSON facts
                                       -> LiveSafeRust cache/worklist -> LSP JSON
```

`flat` keeps the original many-independent-functions sanity case. It is useful
only to check cache mechanics.

`layered` is the main workload. It contains many unrelated helper functions and
five independent 24-hop command construction branches:

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
changes only that function, returning raw input. The analyzer should propagate
the changed summary through the wrapper chain and produce the same finding in
incremental and full modes.

The synthetic workloads still use the lightweight JavaScript source parser. The
real-crate syn run uses `syn-sidecar` to extract function facts. The SAF-backed
run invokes `Project.open(...).query().taint_flow(...)` inside the repository's
Docker environment and feeds SAF findings into the same summary cache and
reverse-call-graph invalidation engine.

## Result

Command:

```bash
node experiments/livesaferust/livesaferust_poc.mjs --workload layered --scale 5000 --chain-depth 24 --fanout 5
```

| Scenario | Functions | Changed | Recomputed | Summary Changed | Summary Stable | Cache Hits | Findings | Time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| clean cold | 5131 | 5131 | 5131 | 5007 | 130 | 0 | 0 | 362.471 ms |
| unrelated incremental | 5131 | 1 | 1 | 0 | 1 | 5130 | 0 | 13.864 ms |
| vulnerable incremental | 5131 | 1 | 122 | 121 | 1 | 5009 | 5 | 37.638 ms |
| vulnerable full | 5131 | 5131 | 5131 | 5128 | 9 | 0 | 5 | 368.320 ms |

Assertions from the run:

```text
ok - clean version has no taint findings
ok - unrelated edit has no taint findings
ok - vulnerable edit produces at least one finding
ok - incremental and full vulnerable findings agree
ok - incremental vulnerable analysis reuses most summaries
ok - layered workload propagates beyond the directly changed function
ok - fan-out workload reports one finding per tainted branch
```

The vulnerable finding is emitted as an LSP-style diagnostic under:

```text
experiments/livesaferust/out/layered/diagnostics.json
```

The SAF-oriented source facts and summaries are under:

```text
experiments/livesaferust/out/layered/semantic-facts.json
```

## Real-Crate Baseline: duct

To anchor the synthetic result, we ran the baseline harness on
`oconnor663/duct.rs`, using `src/lib.rs` as the source file inspected by the
sidecar.

Environment:

```text
cargo 1.95.0
rustc 1.95.0 (59807616e 2026-04-14)
rust-analyzer 1.95.0 (59807616 2026-04-14)
```

Source facts:

| Crate | File | LOC | Functions/Methods | Source Hints | Sink Hints |
| --- | --- | ---: | ---: | ---: | ---: |
| duct | `src/lib.rs` | 2134 | 94 | 0 | 1 (`ChildHandle::start` -> `Command::new`) |

Timing:

| System | Scenario | Time |
| --- | --- | ---: |
| cargo check | cold | 5.80 s |
| cargo check | warm | 0.05 s |
| cargo check | warm after Clippy | 0.28 s |
| cargo clippy | cold | 2.24 s |
| cargo clippy | warm | 0.10 s |
| syn-sidecar | warm parse/fact extraction | 0.04 s |

We then ran LiveSafeRust's summary engine over the `syn-sidecar` facts. The edit
is a compiling source-level change in `src/lib.rs` line 146:
`Vec::new()` to `Vec::<OsString>::new()`. This changes one function hash but
does not alter source/sink facts.

| LiveSafeRust over duct syn facts | Functions | Changed | Recomputed | Cache Hits | Findings | Time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 94 | 94 | 94 | 0 | 0 | 1.802 ms |
| edit incremental | 94 | 1 | 1 | 93 | 0 | 0.615 ms |
| edit full | 94 | 94 | 94 | 0 | 0 | 0.967 ms |

The 0 findings are expected: duct exposes 0 source hints and 1
`Command::new` sink hint, so this experiment validates real-crate plumbing, not
detection efficacy. The synthetic workload stresses propagation and finding
refresh; the duct workload validates that real Rust syntax can feed the same
pipeline.

These times are the JavaScript summary engine over already extracted syn facts,
not the syn parse time and not a rust-analyzer/HIR implementation. The important
experimental closure is structural: the real-crate facts now go through the same
cache, summary equality, reverse call graph, diagnostic, and semantic-fact
pipeline as the synthetic workload.

The syn adapter resolves 44 of 368 call facts in duct and leaves 324 unresolved.
That number is the clearest ceiling of a syn-only frontend: trait methods,
standard-library calls, and external-crate calls need rust-analyzer HIR/name
resolution or conservative top summaries.

The rust-analyzer LSP probe remains inconclusive as a nonempty diagnostic
baseline in this minimal client. An append-comment edit produced an empty
diagnostics packet in 5.704 ms, but semantic-error edits with `didSave` did not
produce a nonempty `publishDiagnostics` within 60 s. We do not use the empty
packet as a strong latency claim.

## Minimal Diagnostic Case

To make the diagnostic payload concrete, we added a compiling 7-line Rust case:

```text
experiments/livesaferust/cases/minimal_command_injection.rs
```

It sends `env::args()` directly into `Command::new`. The `syn-sidecar` extracts
1 function, 1 source hint, and 1 sink hint; LiveSafeRust emits 1
`tainted-command` diagnostic in 0.257 ms. The output is under:

```text
experiments/livesaferust/out/syn/minimal-command-injection/diagnostics.json
```

This is a trace artifact, not a benchmark. It gives the paper a nonzero
diagnostic example while duct remains the real-crate plumbing baseline.

## SAF Integration

The latest prototype adds a real SAF invocation path:

```bash
python3 experiments/livesaferust/saf_integration/run_saf.py <source.rs-or-c>
node experiments/livesaferust/livesaferust_poc.mjs \
  --workload saf-backed --facts <saf-facts.json>
```

The host script compiles source to LLVM IR, invokes the SAF Python SDK inside
Docker, writes `livesaferust.saf-backed/0.1` JSON facts, and lets the existing
LiveSafeRust cache/worklist layer publish diagnostics.

### SAF Patch Notes

The R11 failure was real: `saf-dev:llvm22` could parse host `rustc 1.95` LLVM IR
syntax, but `Project.open(...)` crashed while lowering `std::process::Command`
IR. The crash site was the LLVM frontend constant mapping for Rust aggregate
constants such as:

```llvm
[7 x i8] undef
```

The frontend called string accessors on `undef`/`poison` array constants. The
patch conservatively lowers those arrays to SAF `Undef`. The value-flow builder
also needed Rust ABI modeling: aggregate `memcpy` moves now add direct def-use
edges, and `sret` calls connect the call result plus non-output operands to the
first out-pointer operand. This makes `env::args().nth(...).unwrap_or_default()`
flow to `Command::new` / `Command::arg`.

Regression coverage:

- `tests/fixtures/llvm/e2e/rust_std_command.ll`
- `crates/saf-frontends/tests/rust_std_command_frontend.rs`
- `crates/saf-analysis/tests/rust_std_command_e2e.rs`

Verification:

```text
CARGO_BUILD_JOBS=1 make test-llvm22
2193 Rust tests passed, 9 skipped
94 Python SDK tests passed, 1 skipped
```

The first high-concurrency build was killed by `SIGKILL` while compiling the
test binary; the single-job rerun passed and is the accepted gate result.

### Rust `std::process::Command` Through SAF

The primary Rust result now uses host `rustc 1.95` and the patched SAF LLVM-22
path.

| Case | Expected | Compile | SAF wall | SAF query | Functions | Findings | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| A: `env::args -> Command::new` | bug | 383.9 ms | 955.4 ms | 10.860 ms | 166 | 1 | TP |
| B: `env::args -> Command::arg` | bug | 44.9 ms | 649.3 ms | 10.987 ms | 171 | 1 | TP |
| E: `env::args -> File::open` | bug | 380.8 ms | 789.0 ms | measured | 115 | 1 | TP |
| F: RUSTSEC-2024-0350 minimized `PathBuf::join -> File::create` | bug | 376.2 ms | 811.8 ms | measured | 135 | 1 | TP |
| C: `env::args -> println!` | clean | 45.0 ms | 578.2 ms | 2.089 ms | 54 | 0 | TN |
| D: `env::args -> sanitize_cmd -> Command::new` | clean | 71.6 ms | 715.6 ms | 38.590 ms | 222 | 0 | TN |
| G: `env::args -> sanitize_path -> File::open` | clean | 52.3 ms | 654.3 ms | measured | 129 | 0 | TN |
| Semantic no-op edit | unchanged bug | 383.9 ms | 0 ms | reused | 166 | 1 | cache hit |
| Sanitizer edit | clean | 77.1 ms | 709.5 ms | 25.703 ms | 222 | 0 | finding removed |

Five-run medians for the six non-cache minimal cases:

| Case | Runs | Findings | Functions | Compile median | Compile IQR | SAF median | SAF IQR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `Command::new` | 5 | 1 | 166 | 47.4 ms | 1.6 ms | 593.0 ms | 7.6 ms |
| `Command::arg` | 5 | 1 | 171 | 49.4 ms | 2.8 ms | 615.6 ms | 60.7 ms |
| `File::open` | 5 | 1 | 115 | 44.5 ms | 2.7 ms | 614.5 ms | 51.7 ms |
| clean `println!` | 5 | 0 | 54 | 40.4 ms | 2.3 ms | 534.2 ms | 10.0 ms |
| sanitized command | 5 | 0 | 222 | 52.1 ms | 0.7 ms | 634.2 ms | 22.5 ms |
| sanitized path | 5 | 0 | 129 | 48.1 ms | 7.7 ms | 589.3 ms | 19.2 ms |

LiveSafeRust then consumes the Rust SAF JSON:

| SAF-backed workload | Functions | Changed | Recomputed | Cache Hits | Findings | Orchestration | SAF invoked |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Command cold | 166 | 166 | 166 | 0 | 1 | 0.694 ms | yes |
| Semantic no-op incremental | 166 | 0 | 0 | 166 | 1 | 0.571 ms | no |
| Semantic no-op full | 166 | 166 | 166 | 0 | 1 | 0.224 ms | no |
| Sanitizer incremental | 222 | 218 | 218 | 4 | 0 | 0.706 ms | yes |
| Sanitizer full | 222 | 222 | 222 | 0 | 0 | 0.331 ms | yes |

Sanity assertions now covered on the Rust `std::process::Command` path:

```text
ok - SAF produces at least one finding on rustc-emitted Rust IR
ok - LiveSafeRust orchestration agrees with SAF direct verdict
ok - semantic-LLVM-identical edit does not re-invoke SAF
ok - sanitizer edit re-invokes SAF and removes the finding
```

Confusion matrix:

| Case | Expected | SAF | LiveSafeRust | Verdict |
| --- | --- | --- | --- | --- |
| A | bug | finding | finding | TP |
| B | bug | finding | finding | TP |
| E | bug | finding | finding | TP |
| F | bug | finding | finding | TP |
| C | clean | none | none | TN |
| D | clean | none | none | TN |
| G | clean | none | none | TN |

Aggregate: TP=4, FP=0, FN=0, TN=3.

The RustSec-inspired case models the essence of
RUSTSEC-2024-0350 / CVE-2024-35186: untrusted repository path data is joined
with a worktree path and used to create a file outside the intended tree
(https://rustsec.org/advisories/RUSTSEC-2024-0350.html).

### R13 Per-Edit Live Sequence

Raw outputs are under `experiments/livesaferust/out/saf/r13/edit-sequence/`.
The harness is `experiments/livesaferust/run_r13_edit_sequence.py`.

The sequence expands the RUSTSEC-2024-0350-style path traversal into an
approximately 80-line Rust case and applies 12 realistic edits. Each edit is run
incrementally against the previous SAF facts and cold from a clean cache.
The cache key is a semantic LLVM-IR hash that drops comments,
`source_filename`, and debug metadata, then alpha-normalizes local named SSA
value identifiers within each function. Global function identifiers and call
structure are retained, so the local-variable rename is a cache hit while helper
renames and call-graph changes remain cache misses.

All gates pass:

```text
PASS - G1 no-op edits avoid SAF re-invocation
PASS - G2 body-changed summary-stable edits rerun SAF but keep summaries stable
PASS - G3 summary-changing edits cut off within five source-level hops
PASS - G4 incremental and cold findings agree
PASS - G5 sanitizer removes and restore reintroduces finding
```

Per-edit result:

| Edit | Class | SAF rerun? | Findings | Summary changed | Cutoff | Incremental |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| add comment | comment | no | 1 | 0 | 0 | 136.036 ms |
| reformat whitespace | whitespace | no | 1 | 0 | 0 | 135.317 ms |
| rename local | rename-local | no | 1 | 0 | 0 | 133.780 ms |
| rename helper | rename-helper | yes | 1 | 0 | 0 | 3248.532 ms |
| change string literal | body-changed-summary-stable | yes | 1 | 0 | 0 | 3348.115 ms |
| add debug println | body-changed-summary-stable | yes | 1 | 0 | 0 | 3138.675 ms |
| add unrelated helper | new-unrelated-function | yes | 1 | 0 | 0 | 3320.066 ms |
| add sanitizer | summary-changed-finding-removed | yes | 0 | 1 | 4 | 3140.592 ms |
| remove sanitizer | summary-changed-finding-restored | yes | 1 | 1 | 4 | 3240.066 ms |
| add second sink | new-sink-new-finding | yes | 2 | 1 | 3 | 2929.671 ms |
| remove second sink | revert-toward-base | yes | 1 | 1 | 1 | 3299.820 ms |
| revert to base | full-revert | no | 1 | 0 | 0 | 130.406 ms |

Aggregate: 12 edits, cache-hit ratio 0.333, all incremental findings match cold
findings, median incremental orchestration 3139.633 ms, p95 3320.066 ms.

### Existing-Tool Comparison

Raw outputs are under `experiments/livesaferust/out/comparison/`.

| Tool | Ran? | Result on minimal `Command::new` case |
| --- | --- | --- |
| Clippy (`all`, `pedantic`, `nursery`) | yes | no warnings; stderr only shows successful check |
| rust-analyzer LSP probe | yes | initialized; append-comment edit produced 0 diagnostics after 1226.688 ms |
| CodeQL 2.24.2 Rust security-extended | yes | explicit `rust-security-extended.qls` suite (27 rules) produced 0 SARIF results on command and path cases |
| MIRAI | no | `cargo-mirai` / `mirai` not installed locally |
| LiveSafeRust + patched SAF | yes | 1 SAF finding, 1 LiveSafeRust diagnostic |

Paper table framing:

| Tool | Cross-procedure | Live (IDE) | Security-specific | Incremental | Found bug |
| --- | ---: | ---: | ---: | ---: | ---: |
| rust-analyzer | - | yes | - | yes | - |
| Clippy | - | yes | - | yes | - |
| CodeQL Rust security-extended | yes | - | yes | partial | - |
| MIRAI | yes | - | partial | - | not measured |
| LiveSafeRust | yes | yes | yes | yes | yes |

### C Cross-Check

To cross-check that SAF sees the same pattern in a simpler IR shape, we also run
a C equivalent of the command-injection pattern:

```c
char *cmd = getenv("SAF_INPUT");
return system(cmd);
```

SAF reports one real taint finding:

| Case | Compile | SAF wall | SAF query | Functions | Findings | Source -> Sink |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| C command injection | 699.6 ms | 729.5 ms | 0.126 ms | 3 | 1 | `c_command_injection.c:4:17` -> `c_command_injection.c:5:19` |
| C + `sanitize_input` | 721.1 ms | 728.4 ms | 0.095 ms | 4 | 0 | finding removed |
| C comment edit, semantic LLVM hash unchanged | 744.8 ms | 0 ms | 0 ms | 3 | 1 | SAF cache hit |

LiveSafeRust then consumes the SAF JSON:

| SAF-backed workload | Functions | Changed | Recomputed | Cache Hits | Findings | Orchestration |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| C vulnerable cold | 3 | 3 | 3 | 0 | 1 | 0.207 ms |
| C sanitizer edit incremental | 4 | 2 | 2 | 2 | 0 | 0.035 ms |
| C sanitizer edit full | 4 | 4 | 4 | 0 | 0 | 0.051 ms |
| C comment edit incremental | 3 | 0 | 0 | 3 | 1 | 0.025 ms |

Sanity assertions on the C path:

```text
ok - SAF produces at least one finding on the C equivalent
ok - LiveSafeRust orchestration agrees with SAF direct verdict
ok - semantic-LLVM-identical edit does not re-invoke SAF
ok - sanitizer edit re-invokes SAF and removes the finding
```

Experiment roles should stay explicit in the paper. The synthetic fan-out
benchmark validates Theorem 1's summary-equality cutoff over a deep call graph.
The SAF-backed Rust `std::process::Command` benchmarks validate real SAF
invocation, content-hash caching, and LSP diagnostic transport. They are not yet
one unified demonstration because the current SAF SDK exposes final findings,
not per-function intermediate summaries. A complete LiveSafeRust implementation
should feed SAF per-function summaries into the same invalidation layer.

## Takeaway

The improved experiment fixes two weak points in the first version:

- the main workload is no longer just isolated helpers or a single path; it
  requires multi-hop propagation through five independent fan-out branches;
- the script now asserts clean/vulnerable behavior and full/incremental finding
  agreement.
- the real-crate duct facts now run end-to-end through the LiveSafeRust summary
  engine, rather than only being used as an external baseline.
- the minimized Rust cases produce concrete LSP diagnostic payloads.
- the SAF Python SDK is now actually invoked on Rust-generated LLVM IR; after a
  small SAF patch, `std::process::Command` produces real SAF findings and the
  expanded matrix is TP=4, FP=0, FN=0, TN=3 across command injection and path
  traversal.

The result is still only a preliminary architecture signal. It says the sidecar
contract and invalidation policy are plausible, and that SAF can be patched to
cover a meaningful Rust command-execution subset. It does not yet say the system
handles arbitrary Rust crates, real IDE latency, or broad false-positive /
false-negative rates.

## Threats To Validity

- The synthetic parser is still a lightweight JavaScript source parser. The
  real-crate path uses `syn`, but the fact adapter is intentionally shallow and
  does not yet model argument-level value flow, macros, traits, generics,
  lifetimes, MIR, async lowering, match exhaustiveness, closure captures, or
  ownership semantics.
- On duct, the syn adapter resolves only 44 of 368 call facts. The remaining 324
  unresolved calls should be treated as conservative `Top` in a sound
  implementation, which is why rust-analyzer HIR integration matters.
- Summary orchestration timings are JavaScript microbenchmark timings. SAF wall
  times are measured separately from the SAF Python SDK inside Docker.
- The patched SAF LLVM-22 path accepts minimized `std::process::Command` IR and
  finds the intended flows. This is still a small Rust subset; async lowering,
  macro-heavy crates, trait objects, richer standard-library protocols, and
  ownership-sensitive unsafe behavior remain untested.
- `rustc -> LLVM IR` compilation is still an edit-path cost. The R11 minimal
  cases compile in 44.9-383.9 ms on the host, while SAF itself costs
  578.2-955.4 ms per cold call. Future work should use incremental rustc IR
  generation or a direct rust-analyzer HIR -> SAF AIR lowering.
- The generated workload is synthetic. It now has non-trivial propagation, but
  not real crate structure.
- We now have `cargo check`, Clippy, rust-analyzer empty-diagnostic, CodeQL Rust,
  syn-sidecar, and SAF-SDK measurements, but not PhASAR or a reliable
  rust-analyzer nonempty diagnostic latency.
- The FP/FN matrix has seven minimized cases across two vulnerability families,
  but it is still not a broad precision/recall study.
- The rust-analyzer LSP diagnostic latency probe needs a fuller client/config
  before it can be used as a reliable baseline.

## Paper Narrative Adjustment

The paper should not claim "incremental analysis is faster" as the main novelty.
That is known. The stronger new-idea framing is:

> Treat rust-analyzer/Salsa re-computation events as the live invalidation layer
> for SAF-derived security facts, so expensive SAF calls are paid on semantic
> cache misses and cached findings can still appear in the IDE at edit latency.

## Next Steps

1. Replace the syn fact adapter with rust-analyzer HIR/source-map facts.
2. Add real-crate case studies with command execution, FFI, and unsafe resource
   protocol patterns.
3. Add remaining baselines: rust-analyzer nonempty diagnostic latency and
   PhASAR.
4. Report TP/FP/FN over additional real RustSec or hand-minimized cases beyond
   command execution and path traversal.
