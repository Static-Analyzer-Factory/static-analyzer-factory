# LiveSafeRust Salsa Adapter Design

This document records the method contribution that the runnable POC is meant to
support: SAF-derived summaries and findings as rust-analyzer/Salsa queries.

The prototype in `livesaferust_poc.mjs` is a sidecar. In R8 it can also consume
facts emitted by the real SAF Python SDK. A paper should sell the design below,
then use the prototype as preliminary evidence that the SAF orchestration,
summary shape, and invalidation policy are plausible.

## Design Goal

Use rust-analyzer's live semantic database as the frontend for security-oriented
SAF analyses:

```text
LSP didChange
  -> AnalysisHost::apply_change(...)
  -> rust-analyzer/Salsa invalidates parse, macro, HIR, type queries
  -> SAF summary queries recompute only affected functions
  -> publishDiagnostics with SAF traces
```

The novelty is not incremental dataflow by itself. The novelty is making
rust-analyzer's existing edit-local semantic recomputation the frontend boundary
for heavyweight SAF security facts, so the LLVM/batch path is paid on semantic
cache misses instead of every edit.

## Theorem 1: Diagnostic Refresh By Summary Inequality

Let each function summary live in a finite monotone lattice, and let local
findings for function `f` depend only on:

1. `f`'s current body facts; and
2. the summaries of functions directly called by `f`.

Then, after an edit, it is sufficient for diagnostic completeness to:

1. recompute summaries and local findings for edited functions;
2. enqueue callers only when a recomputed callee summary changes; and
3. stop propagation at a caller whose recomputed summary is equal to its
   previous summary.

Completeness here means that every diagnostic whose truth value could change
because of the edit is recomputed before diagnostics are published.

**Soundness sketch.** A function's ancestors can observe that function only
through its summary. If the summary is unchanged, all transfer effects visible
to ancestors are unchanged, so no ancestor's local finding can change through
that path. Local findings are not stored in summaries because they are physically
emitted at the sink-containing function; they are recomputed whenever that
function's body facts or any directly called callee summary changes. The
monotone finite lattice assumption gives a deterministic fixed point for summary
recomputation, and summary equality is therefore a valid cutoff for ancestors.

This theorem is the paper's central method claim. The POC implements the same
policy with `summaryChangedFunctions` and `summaryStableFunctions` counters.

## Query Sketch

The query names below are proposal-level pseudocode. They are intentionally
small enough to fit into a rust-analyzer extension crate or sidecar layer.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SafConfig {
    pub model_version: u32,
    pub source_specs_hash: Hash,
    pub sink_specs_hash: Hash,
    pub sanitizer_specs_hash: Hash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SafFunctionId {
    pub crate_id: CrateId,
    pub def: DefWithBodyId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodyFacts {
    pub source_range: TextRange,
    pub params: Vec<ParamFact>,
    pub call_sites: Vec<CallSiteFact>,
    pub local_sources: Vec<ValueFact>,
    pub local_sinks: Vec<SinkFact>,
    pub sanitizers: Vec<SanitizerFact>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaintSummary {
    pub return_deps: SmallVec<[TaintDep; 4]>,
    pub sink_params: SmallVec<[ParamIdx; 4]>,
    pub content_hash: Hash,
}

#[salsa::tracked]
fn saf_body_facts(db: &dyn HirDatabase, f: SafFunctionId, cfg: SafConfig) -> BodyFacts {
    let body = db.body(f.def);
    let source_map = db.body_with_source_map(f.def).1;
    let infer = db.infer(f.def);
    lower_hir_to_saf_facts(body, source_map, infer, cfg)
}

#[salsa::tracked]
fn saf_callees(db: &dyn HirDatabase, f: SafFunctionId, cfg: SafConfig) -> Vec<SafFunctionId> {
    saf_body_facts(db, f, cfg)
        .call_sites
        .iter()
        .filter_map(|call| resolve_to_def_with_body(db, call))
        .map(|def| SafFunctionId { crate_id: f.crate_id, def })
        .collect()
}

#[salsa::tracked]
fn taint_summary(db: &dyn HirDatabase, f: SafFunctionId, cfg: SafConfig) -> TaintSummary {
    let facts = saf_body_facts(db, f, cfg);
    let callee_summaries = saf_callees(db, f, cfg)
        .into_iter()
        .map(|callee| (callee, taint_summary(db, callee, cfg)))
        .collect();
    summarize_function(facts, callee_summaries)
}

#[salsa::tracked]
fn taint_diagnostics(db: &dyn HirDatabase, file: FileId, cfg: SafConfig) -> Vec<Diagnostic> {
    functions_in_file(db, file)
        .into_iter()
        .flat_map(|f| local_taint_findings(db, f, cfg))
        .map(to_lsp_diagnostic)
        .collect()
}

#[salsa::tracked]
fn local_taint_findings(db: &dyn HirDatabase, f: SafFunctionId, cfg: SafConfig) -> Vec<Finding> {
    let facts = saf_body_facts(db, f, cfg);
    let callee_summaries = saf_callees(db, f, cfg)
        .into_iter()
        .map(|callee| (callee, taint_summary(db, callee, cfg)))
        .collect();
    find_local_sinks(facts, callee_summaries)
}
```

## Dependency Alignment

The SAF facts query should depend on rust-analyzer queries at the highest
semantic layer needed by a checker:

| SAF need | rust-analyzer dependency |
| --- | --- |
| Function identity | `DefWithBodyId`, crate graph |
| Source ranges | `body_with_source_map` |
| Macro-expanded body | `body` / HIR body query |
| Call resolution | `infer`, path resolution, method resolution |
| Trait/generic target set | type inference plus conservative unresolved-call summary |
| Diagnostic spans | source map back to original file ranges |

This keeps invalidation natural: file edits already invalidate the parse/HIR/type
queries in rust-analyzer; SAF summaries are downstream tracked queries.

## Indirect Calls And Rust-Specific Conservatism

The IDE-mode analyzer should be optimistic only when rust-analyzer can resolve a
call target with enough confidence. For unresolved trait-object calls, function
pointers, closure captures whose target set is not available, macro-generated
calls without a stable source mapping, or FFI calls without a model, use a
conservative top summary:

```rust
fn unresolved_call_summary(arity: usize) -> TaintSummary {
    TaintSummary {
        return_deps: smallvec![TaintDep::Top],
        sink_params: (0..arity).map(ParamIdx).collect(),
        content_hash: Hash::model("unresolved-call-top"),
    }
}
```

For a security checker this means:

- returned values are considered possibly tainted;
- every argument is treated as possibly reaching an unknown sink;
- diagnostics should be marked `confidence = low` or `precision = conservative`;
- the same policy can later be refined by crate-specific summaries or SAF's
  LLVM/MIR-level facts.

This policy is intentionally noisy but prevents the method from being limited to
direct calls in the presence of Rust traits, closures, and FFI.

## Cache Key Strategy

There are two cache layers:

1. **Salsa query cache**: keyed by `(SafFunctionId, SafConfig)` and invalidated
   by normal rust-analyzer dependencies.
2. **Optional SAF persistent cache**: keyed by
   `(crate_id, def_path_hash, body_hash, cfg_hash, model_version)`.

The persistent cache is optional for IDE mode. It helps warm-start large
workspaces, but correctness should not rely on it. The Salsa cache is the source
of truth after `AnalysisHost::apply_change`.

## Finding Refresh Invariant In The Query Graph

The POC stores two products per function:

- `TaintSummary`: what callers need to know.
- `Finding[]`: diagnostics whose physical sink is in the function body.

Findings are not placed inside summaries. This is safe under the following
invariant:

> If a function body changes, recompute its summary and local findings. If a
> callee summary changes, recompute the caller's local findings and summary. If
> the caller summary is unchanged, do not propagate further, because ancestors
> depend on the caller summary, not on diagnostics physically emitted in the
> caller body.

This is exactly the `local_taint_findings(db, f, cfg)` dependency above: a
finding is a tracked query depending on local body facts and callee summaries.
It is refreshed whenever either changes, even when the refreshed summary is
equal to the old one.

## Open Research Questions

- How conservative should unresolved trait-object or function-pointer calls be
  in live IDE mode?
- Which summary products should be shown immediately, and which should be
  deferred until rust-analyzer finishes higher-cost inference?
- Can diagnostics carry a stable trace across edits without stale spans?
- How should SAF merge rust-analyzer facts with LLVM/MIR-level facts for unsafe
  blocks and FFI boundaries?
