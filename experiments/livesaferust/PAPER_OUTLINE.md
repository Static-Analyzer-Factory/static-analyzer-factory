# LiveSafeRust Proposal Outline

## Working Title

LiveSafeRust: LSP-Guided Incremental Security Analysis for Rust

## Gap

Rust developers already get fast compiler and rust-analyzer feedback, but
security-oriented cross-procedure analyses usually live outside the edit loop.
Batch analyzers can find deeper issues, but they are poorly matched to the
sub-second feedback expectations of an IDE. Existing live tooling focuses on
syntax, types, borrow checking, and lint-like patterns; it does not expose a
general path for SAF/SVF-style value-flow and typestate summaries to run inside
the LSP feedback loop.

The paper now has a first quantitative anchor:

- on duct, `cargo check` is 5.80 s cold / 0.05 s warm and Clippy is 2.24 s cold
  / 0.10 s warm;
- after patching SAF, SDK calls on minimized `std::process::Command` Rust cases
  take 0.58-0.96 s wall time, while a semantic no-op edit reuses cached SAF JSON
  in 0.57 ms without re-invoking SAF;
- rust-analyzer nonempty diagnostic latency remains inconclusive in the current
  probe and should not be over-claimed.

## Core Idea

Treat rust-analyzer's Salsa database as a live invalidation frontend for
SAF-derived summaries and findings:

```text
Editor edit
  -> rust-analyzer applies change
  -> Salsa invalidates parse/HIR/type queries
  -> SAF body facts and summaries recompute as downstream queries
  -> LSP diagnostics include source-to-sink traces
```

The method contribution is the query boundary and invalidation invariant, not
the basic existence of function summaries.

LiveSafeRust is an orchestration layer above SAF. SAF computes per-function
taint summaries or findings from IR facts; LiveSafeRust caches them, invalidates
them via Theorem 1, and surfaces them as LSP diagnostics. The prototype actually
invokes the SAF Python SDK on Rust-generated LLVM IR. R11 patches SAF's LLVM-22
path so `std::process::Command` cases produce real SAF findings.

See `SALSA_ADAPTER.md` for the concrete query sketch.

## Method

1. Lower rust-analyzer HIR/type facts to SAF body facts:
   function parameters, source ranges, call sites, local source/sink/sanitizer
   events, and conservative unresolved-call facts.
2. Compute tracked summaries:
   `returnDeps`, `sinkParams`, and later typestate/resource summaries.
3. Emit local diagnostics as tracked queries that depend on local body facts and
   callee summaries.
4. Convert findings to LSP diagnostics with trace metadata and related
   information.

Important invariant:

> Local findings are refreshed when body facts or callee summaries change. Only
> summary changes need to propagate to ancestors.

This should be promoted to the central theorem in the paper:

**Theorem 1 (Diagnostic refresh by summary inequality).** Under a finite
monotone summary lattice and finding locality, caller-only invalidation triggered
by callee summary inequality is sufficient for diagnostic completeness.

Proof sketch: a caller's ancestors can observe the caller only through its
summary. If that summary is unchanged after recomputing local findings, no
ancestor-visible transfer behavior changed, so propagation may stop. The local
finding query still depends on body facts and direct callee summaries, so
diagnostics physically emitted in the caller are refreshed even when the caller
summary is stable.

Unresolved Rust calls should be handled conservatively. Trait-object calls,
function pointers, unknown closure targets, and unmodeled FFI calls receive a
top summary: the return may depend on unknown taint and all arguments may reach
an unknown sink. This is noisy, but gives formal coverage beyond direct-call toy
graphs.

## Preliminary Result

Abstract lead:

> We patched SAF to support `rustc`-emitted LLVM IR for
> `std::process::Command` and file-opening sinks. On seven minimized Rust cases
> across command injection and path traversal, including a RUSTSEC-2024-0350 /
> CVE-2024-35186-style case, SAF reports 4 TP / 0 FP / 0 FN / 3 TN;
> LiveSafeRust agrees with SAF's direct verdict and serves a semantic no-op edit
> from cached SAF JSON in 0.57 ms without re-invoking SAF. The synthetic fan-out
> workload is secondary: it validates Theorem 1's summary-equality cutoff over a
> deep call graph.

The current runnable POC uses a source sidecar rather than rust-analyzer. It is
therefore preliminary architecture evidence, not a full Rust-analysis result.

Main synthetic workload:

- 5000 unrelated helper functions;
- 5 independent tainted command-construction branches;
- 24 wrapper functions per branch;
- one edit removes sanitizer behavior in the shared `normalize_command`.

Command:

```bash
node experiments/livesaferust/livesaferust_poc.mjs --workload layered --scale 5000 --chain-depth 24 --fanout 5
```

Result:

| Scenario | Functions | Changed | Recomputed | Summary Changed | Summary Stable | Findings | Time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| clean cold | 5131 | 5131 | 5131 | 5007 | 130 | 0 | 362.471 ms |
| unrelated incremental | 5131 | 1 | 1 | 0 | 1 | 0 | 13.864 ms |
| vulnerable incremental | 5131 | 1 | 122 | 121 | 1 | 5 | 37.638 ms |
| vulnerable full | 5131 | 5131 | 5131 | 5128 | 9 | 5 | 368.320 ms |

Assertions:

- clean version has no taint findings;
- unrelated edit has no taint findings;
- vulnerable edit reports one finding per tainted branch;
- incremental and full vulnerable findings agree;
- changed summary propagates beyond the directly edited function.

Bridge to SAF experiment:

Synthetic fan-out stresses Theorem 1's summary-equality cutoff over deep call
graphs. SAF-backed `std::process::Command` Rust cases validate real SAF engine
invocation, content-hash caching, and LSP diagnostic transport. The current SAF
SDK returns final findings rather than per-function intermediate summaries, so
the experiments are complementary rather than a single unified demonstration. A
complete implementation should expose SAF per-function summaries to the same
LiveSafeRust orchestration layer.

## Evaluation Plan

Minimum credible next evaluation:

1. Replace the `syn` fact adapter with rust-analyzer/HIR facts.
2. Improve baseline latency on one small real crate:
   rust-analyzer nonempty diagnostics and PhASAR.
3. Evaluate 3-5 real or minimized Rust cases:
   command execution, FFI input-to-sink, unsafe resource protocol misuse.
4. Report TP/FP/FN and one trace screenshot or LSP diagnostic payload.

Candidate crates or cases should be selected for actual command/FFI/unsafe
behavior, not just for size.

Real-crate baseline collected so far:

| System | Scenario | Target crate | Time |
| --- | --- | --- | ---: |
| cargo check | cold | duct | 5.80 s |
| cargo check | warm | duct | 0.05 s |
| cargo check | warm after Clippy | duct | 0.28 s |
| cargo clippy | cold | duct | 2.24 s |
| cargo clippy | warm | duct | 0.10 s |
| syn-sidecar | warm fact extraction from `src/lib.rs` | duct | 0.04 s |
| LiveSafeRust over syn facts | cold summary analysis | duct | 1.8 ms |
| LiveSafeRust over syn facts | edit incremental | duct | 0.6 ms |
| LiveSafeRust over syn facts | edit full | duct | 1.0 ms |

The `syn-sidecar` extracted 94 functions/methods and one `Command::new` sink
from duct's 2134-line `src/lib.rs`. A compiling source edit
(`Vec::new()` to `Vec::<OsString>::new()`) changes one function hash; the
incremental summary engine recomputes that one function and agrees with full
reanalysis. The rust-analyzer LSP latency probe is not yet a reliable baseline:
an append-comment edit produced an empty diagnostics packet in 5.704 ms, but
nonempty diagnostics for semantic-error edits did not arrive within 60 s in the
minimal client.

Interpretation:

- synthetic fan-out stresses propagation and finding refresh;
- duct validates real-crate plumbing and external latency anchors;
- duct has 0 source hints and 1 sink hint, so 0 findings is expected;
- the syn adapter resolves 44 of 368 duct call facts, leaving 324 unresolved and
  motivating rust-analyzer/HIR integration.

Minimal diagnostic artifact:

| Case | Functions | Source Hints | Sink Hints | Findings | Time |
| --- | ---: | ---: | ---: | ---: | ---: |
| `minimal_command_injection.rs` | 1 | 1 | 1 | 1 | 0.257 ms |

The diagnostic payload is under
`experiments/livesaferust/out/syn/minimal-command-injection/diagnostics.json`.

SAF-backed integration result:

| Target | SAF status | Compile | SAF wall | Functions | Findings |
| --- | --- | ---: | ---: | ---: | ---: |
| Rust `Command::new` vuln | ok, TP | 383.9 ms | 955.4 ms | 166 | 1 |
| Rust `Command::arg` vuln | ok, TP | 44.9 ms | 649.3 ms | 171 | 1 |
| Rust `File::open` path traversal | ok, TP | 380.8 ms | 789.0 ms | 115 | 1 |
| RUSTSEC-2024-0350 minimized gix-fs | ok, TP | 376.2 ms | 811.8 ms | 135 | 1 |
| Rust clean print | ok, TN | 45.0 ms | 578.2 ms | 54 | 0 |
| Rust sanitized command | ok, TN | 71.6 ms | 715.6 ms | 222 | 0 |
| Rust sanitized path | ok, TN | 52.3 ms | 654.3 ms | 129 | 0 |
| Rust semantic no-op edit | cache hit | 383.9 ms | 0 ms | 166 | 1 |
| C equivalent `getenv -> system` | ok | 699.6 ms | 729.5 ms | 3 | 1 |
| C equivalent + sanitizer | ok | 721.1 ms | 728.4 ms | 4 | 0 |
| C comment edit, semantic LLVM unchanged | cache hit | 744.8 ms | 0 ms | 3 | 1 |

LiveSafeRust over SAF JSON:

| Workload | Functions | Changed | Recomputed | Cache Hits | Findings | Orchestration |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rust command vulnerable cold | 166 | 166 | 166 | 0 | 1 | 0.694 ms |
| Rust command cache-hit incremental | 166 | 0 | 0 | 166 | 1 | 0.571 ms |
| Rust command sanitizer incremental | 222 | 218 | 218 | 4 | 0 | 0.706 ms |
| C vulnerable cold | 3 | 3 | 3 | 0 | 1 | 0.207 ms |
| C sanitizer edit incremental | 4 | 2 | 2 | 2 | 0 | 0.035 ms |
| C comment edit incremental | 3 | 0 | 0 | 3 | 1 | 0.025 ms |

Interpretation: SAF is a seconds-scale external analyzer in this harness; the
LiveSafeRust contribution is keeping SAF results usable across edits by caching
semantic IR/function content and refreshing only the affected diagnostics. The
positive Rust evidence now includes `std::process::Command` flows from
`env::args` to both `Command::new` and `Command::arg`, plus path traversal flows
to `File::open` and `File::create`. The confusion matrix is TP=4, FP=0, FN=0,
TN=3 over seven minimized cases. Full crate coverage remains future work: lower
rust-analyzer HIR to SAF AIR or continue hardening Rust LLVM IR support.

Existing-tool comparison:

| Tool | Cross-procedure | Live | Security | Incremental | Found bug |
| --- | ---: | ---: | ---: | ---: | ---: |
| rust-analyzer | - | yes | - | yes | - |
| Clippy | - | yes | - | yes | - |
| CodeQL Rust security-extended | yes | - | yes | partial | - |
| MIRAI | yes | - | partial | - | n/m |
| LiveSafeRust | yes | yes | yes | yes | yes |

## Related Work Positioning

- rust-analyzer/Salsa: our work adds downstream security queries to an existing
  incremental Rust IDE database.
- Infer: compositional summaries, but focused on code review/CI rather than
  LSP edit latency.
- SVF/PhASAR/SAF-over-LLVM: powerful batch value-flow engines; LiveSafeRust is
  a source/HIR-side live frontend, not a replacement.
- CodeQL/incremental CodeQL: query-based security analysis with reuse across PR
  changes; LiveSafeRust targets a finer editor-loop granularity.
- Souffle/Doop/Datalog analysis: declarative program facts and scalable solvers;
  LiveSafeRust borrows the fact/summary separation while aligning invalidation
  with Salsa.

## Threats

- Current real-crate parser is `syn`, not rust-analyzer/HIR.
- Current summary timings are JavaScript microbenchmarks.
- SAF is now invoked through the Python SDK on Rust-generated LLVM IR, and the
  patched LLVM-22 path handles minimized `std::process::Command` cases. This is
  still a subset, not full Rust semantic coverage.
- `rustc -> LLVM IR` generation remains an edit-path cost; LiveSafeRust's cache
  avoids SAF re-invocation on semantic IR cache hits, but first analysis still
  pays compile/frontend cost.
- Synthetic fan-out still lacks traits, closures, generics, macros, async, and
  ownership-sensitive unsafe behavior.
- duct resolves only 44 of 368 call facts through the syn adapter.
- External baselines cover one real crate and minimized command/path traversal
  cases only.

## Contribution Shape

For a New Idea paper, the honest contribution package is:

1. A live-SAF architecture over rust-analyzer/Salsa queries.
2. A summary/finding invalidation invariant for IDE diagnostics.
3. A preliminary sidecar prototype showing the interface and invalidation shape.
4. A concrete plan and early harness for real-crate and baseline evaluation.
