# Plan 190 — CruxBC Performance & Memory Parity vs SVF

**Status:** in-progress (investigation complete 2026-08-04; fixes 1.1, 2.2, 3.1,
3.2a, 4.1a/b implemented and verified same day — see "Implementation results";
fix 1.2 implemented but gated off pending 2.1; fix 2.1 designed, not implemented)

## Implementation results (2026-08-04)

Measured after landing fixes 1.1 + 2.2 + 3.1 + 3.2a + 4.1a/b (vs the same-day
baseline; SVF numbers unchanged):

| target | before | after | FS-PTA detail | RSS before → after |
|---|---|---|---|---|
| bash | 117.3s | **58.8s (−50%)** | 68.7s truncated → 13.8s converged (84,579 iters) | 4016 → 3386MB |
| tmux | 95.0s | **48.5s (−49%)** | 59.4s truncated → 14.4s converged (88,876 iters) | 3133 → 2618MB |
| curl | 5.81s | **2.31s (−60%)** | 4.28 → 0.94s; field locs 107,811 → 4,271 | 355 → 301MB |
| htop | 3.47s | 1.72s | 2.01 → 0.39s | 339 → 290MB |
| unrar | 4.77s | 3.72s | 1.44 → 0.48s | 470 → 436MB |

All 11 targets now converge (`fspta_limit_hit=false` everywhere). SAF is faster
than SVF on 10/11 (curl narrowed from 4.8x slower to 1.9x). Memory is near SVF
parity (bash 1.05x, tmux 1.08x). Gates: full suite 2,190/2,190; PTABen strictly
improved (exact 2269→2270, unsound 138→137).

**Fix 1.2 outcome:** implemented and validated (PTABen unchanged), but enabling
it WITHOUT fix 2.1 makes the newly-visible dispatch-table fan-outs explode under
field-insensitive stack/heap analysis: tmux 48.5→559s / 9.0GB (3.7x slower than
SVF), bash 58.8→199s / 6.3GB. Exactly the sequencing risk this plan predicted.
It is therefore **gated off by default** behind the `SAF_DECOMPOSE_POINTER_ARRAYS`
env var (`MappingContext::decompose_pointer_arrays`); flip the default when 2.1
lands.

### Adversarial review of the implementation (29-agent workflow, same day)

14 confirmed findings (9 refuted), all fixed:
- **Major/soundness:** the >512 aggregate cap left conflated cells
  strong-update-eligible — FS-PTA/DDA strong updates could kill live values
  (under-approximation), and `lookup_approx`'s parent fallback landed
  `[F0,Fk]` misses on element 0's exact cell (a sibling slot). Fixed with a
  **summary-object mechanism**: `LocationFactory::mark_summary_object` set on
  the capped path; `can_strong_update` (FS + DDA) blocks strong updates on
  summary-object cells (SVF `isFieldInsensitive` analog); `lookup_approx`
  routes summary-object path misses straight to the base cell. Unit tests
  added for both halves.
- **Major/soundness:** initializer decomposition dropped FORWARD global
  references (`global_value_ids` was populated lazily; affected vtables too).
  Fixed by pre-registering all global value IDs (name-derived, order-
  insensitive) before the conversion loop.
- `make_synthetic_value` keyed by local element index conflated sibling nested
  aggregates — now keyed by the full field path.
- Struct-wrapped pointer arrays (`{ [N x ptr], ... }`) now also trigger the
  (gated) decomposition.
- Dead `union_points_to` removed (would have failed `make lint`); stale doc
  comments corrected (the soundness-gate comment now states exactly which
  consumers fall back vs return empty).
- Python: `build_with_repr` returns `PyResult` and propagates the escalated
  truncation warning; stacklevel corrected to 1.
- Tutorials (4 call sites + 1 README snippet) updated for `Arc<PtaResult>`;
  PROGRESS.md plans-index row corruption repaired.
**Goal:** Close the remaining time/memory gaps vs SVF on the CruxBC corpus, bound peak
memory, and fix benchmark-validity/soundness-parity issues discovered during the
investigation.

## Context

The user report was "SAF is slow on some CruxBC targets (slower than SVF by a lot)."
A fresh head-to-head (2026-08-04, LLVM 18 mem2reg corpus, process-isolated, same host)
shows the picture has inverted since the plan-102/103/105 era on most targets — but
one target (curl) is genuinely ~7x slower, memory is worse than SVF on the two
largest targets, and two benchmark-validity bugs were flattering SAF's numbers.

### Benchmark infrastructure reconstructed (was missing from the repo)

`scripts/prepare-cruxbc.sh`, `scripts/run-svf-cruxbc.sh`, `scripts/compare-cruxbc.py`
were referenced by the Makefile but never committed. They are now (re)created:
- `prepare-cruxbc.sh` — converts the 11 `.bc` from
  `tests/benchmarks/ptaben/test_cases_bc/crux-bc/` to mem2reg `.ll` under
  `tests/benchmarks/cruxbc/.compiled/{small,big,extra}/`
  (small = bc, dc, bunzip2, bzip2recover, htop, libbz2.so, curl, unrar;
  big = bash, libcurl.so; extra = tmux).
- `run-svf-cruxbc.sh` — runs `wpa -fspta -stat` from the `svftools/svf` Docker image
  on the same `.ll` corpus, polling VmHWM for peak RSS; full stat output saved to
  `tests/benchmarks/cruxbc/svf-logs/<name>.txt`.
- `compare-cruxbc.py` — SAF-vs-SVF table + per-phase breakdown.

### Measured head-to-head (wall clock; SVF wall includes ~0.5s docker startup)

| target | SAF total | SVF total | SAF RSS | SVF RSS | verdict |
|---|---|---|---|---|---|
| big/bash | 117.3s | 527.0s | 4016MB | 3239MB | SAF 4.5x faster; 1.24x more memory |
| extra/tmux | 95.0s (truncated) / ~112s converged | 151.0s | 3133MB | 2432MB | SAF 1.3x faster converged; 1.29x more memory |
| small/curl | 5.81s | 1.22s (0.83s internal) | 355MB | 260MB | **SAF ~7x slower** |
| small/unrar | 4.77s | 31.28s | 470MB | 876MB | SAF faster (soundness caveat V8b) |
| big/libcurl.so | 3.39s | 11.91s | 565MB | 1243MB | SAF faster (soundness caveat V8a) |
| small/htop | 3.47s | 4.65s | 339MB | 375MB | parity |
| others (bc, dc, bunzip2, bzip2recover, libbz2) | ≤0.5s | ≤0.8s | ≤173MB | ≤155MB | parity or SAF faster |

### Per-phase (internal times; SVF from `-stat`, SAF from bench phases)

| target | SAF ander | SVF ander | SAF mssa+svfg | SVF mssa+svfg | SAF fs (converged) | SVF fs total |
|---|---|---|---|---|---|---|
| bash | 34.4 | 33.9 | 12.5 | 34.8 | 68.1 (103,258 iters) | 516.8 |
| tmux | 26.4 | 17.4 | 7.8 | ~4.0 | 75.1 (125,151 iters) | 146.5 |
| curl | 0.38 | 0.17 | 0.89 | 0.13 | 4.28 (18,433 iters) | 0.53 |
| unrar | 0.58 | 1.96 | 2.39 | ~0.66 | 1.44 (57,138 iters) | 29.5 |

Precision proxy (avg/max points-to size):
- bash: SAF 345.6/730 (51,371 of 92,564 pointers >256!) vs SVF 191.7/326
- curl: SAF 69.8/349 (6,026 of 15,683 pointers at 65–256) vs SVF 2.05/109 (~13x mass)
- unrar: SAF 42.0/4031 (14,619 pointers at 65–256) vs SVF 47.6/171

## Root causes (verified against code + targeted experiments)

### V1 — FS-PTA silently truncates at 100K pops (benchmark validity, soundness) — CONFIRMED
`FsPtaConfig::default().max_iterations = 100_000` (fspta/mod.rs) counts worklist pops;
`solve_flow_sensitive` breaks with only a silent `iteration_limit_hit` flag that the
driver discarded. Measured: bash needs 103,258 pops (3% over), tmux 125,151 (25% over).
Both big-target results in any prior benchmark run were non-converged. Bench plumbing
now surfaces `fspta_iterations` / `fspta_limit_hit` (implemented in this session:
`bench_types.rs`, `driver.rs`, `cruxbc.rs`, plus a `fspta_max_iterations` knob).

### V2 — Per-element field-location materialization for aggregate globals — CONFIRMED
`create_aggregate_field_locations` (pta/extract.rs:196-240) creates one location per
initializer element *recursively*, plus a duplicate `[Field{0}, Field{i}]` variant,
with no cap and no pointer-relevance filter. On curl, `@hugehelpgz = [51770 x i8]`
(gzipped help text) alone produces 103,541 of the 107,811 field locations — locations
that can never hold pointers. This inflates the location universe consumed by
mod/ref dense bitsets, GEP handling, and MSSA. Controlled experiments showed
`field_depth` and `constant_indices`/Z3 knobs have **zero** effect on this
(depth histogram is 1–2; the blowup is breadth).

### V3 — Fat points-to sets: the benchmarked pipeline is FIELD-INSENSITIVE for all stack/heap objects — CONFIRMED (corrected mechanism)
Adversarial verification refuted the "constant tables" theory and found the real,
larger mechanism: `refine_prepare` (cg_refinement.rs:185-268) never calls
`precompute_indexed_locations`; extraction creates only empty-path base locations for
Alloca/HeapAlloc (extract.rs:681-700); and the solver holds the `LocationFactory`
immutably so it can never create field locations. Consequently in
`handle_gep_constraints` (solver.rs:1243-1258) `lookup_approx` falls through to the
**base object** for every GEP on a stack/heap struct — all pointer fields of e.g.
`Curl_easy`/`connectdata` share ONE cell. Stores pile every field's targets into that
cell; loads return the union. This is exactly the observed 65–349-element sets of
whole-object (empty-path) locations on curl and the >256-element sets on bash/unrar.
SVF creates `GepObjVar` per accessed field on demand during solving. This is a
first-order precision defect that multiplies every downstream phase.

### V4 — FS solver constant factors — CONFIRMED
`solve_flow_sensitive` hardcodes `BTreePtsSet`; `propagate_direct` deep-clones the
source set per outgoing edge visit; the worklist is a `BTreeSet<SvfgNodeId>` popped in
BLAKE3-random ID order (not SVFG topological order); top-level sets have no sharing
(only address-taken IN/OUT use the VersionTable). SVF survives identical set mass via
hash-consed persistent points-to sets (SAS'21: ≥4.93x memory, 1.69x time on SFS;
<1% of unions executed concretely) + SCC/topo ordering.

### V5 — Andersen regressions vs the plans-125–134 era — CONFIRMED (details per verification)
Current bash Andersen phase is 34.4s vs ~10s recorded after plan 134. SAF_LOG stats:
`process_location = 18.66s` with **zero** early exits and zero empty diffs (the
plan-129 early-exit path never fires); `cg_loop = 16.0s` (refinement waves nearly as
expensive as the initial solve); `lcd_invocations = 95,350` with `lcd_merges = 0`
(pure overhead); `scc_invocations = 0` despite 1.34M pops (the every-50K-pops Tarjan
cadence appears dead). `store_locs_iterated = 13.1M` (5x the plan-130-era count).
Also: CG refinement hardcodes `GenericSolver<FxHashPtsSet>` (cg_refinement.rs:286),
so plan 170's Roaring/frozen-indexer memory win is inactive on this path.

### V6 — Memory residency (bash 4.0GB vs SVF 3.2GB) — CONFIRMED
At peak, multiple full copies of the points-to relation coexist: db `PtaResult`
(BTreeMap/BTreeSet of u128), the full `pta.clone()` owned by MSSA (driver.rs
`pta_clone` phase), the FS seed copy, the MSSA clobber cache
(`BTreeMap<(MemAccessId, LocId), MemAccessId>`, ~400MB-class on bash), and
SVFG + FsSvfg each storing every edge twice with cloned object sets. The SVFG is
donated to the db and stays resident through the FS solve even with checkers off.

### V7 — mod/ref universal clobber for unresolved indirect calls — CONFIRMED
`modref.rs:195-206`: any unresolved CallIndirect gets may_mod/may_ref = ALL locations
(dense per-function bitsets over the loc universe). Interacts multiplicatively with
V2's location blowup on curl. `PtaResult::points_to` allocates a fresh Vec per call
and `is_clobber` does linear `Vec::contains` per Def per walk.

### V8 — Soundness-parity gaps that flatter SAF's numbers — CONFIRMED (corrected mechanism)
(a) libcurl.so: initial "0 indirect sites" reading was wrong (that was curl, which
genuinely has 0; libcurl resolves 946/1068 sites, matching SVF). The real gap: **the
LLVM frontend drops initializers of non-vtable constant arrays** — `convert_global`
sets `decompose_arrays=true` only for `_ZTV/_ZTC/_ZTT` globals (mapping.rs:619-623),
and struct elements inside arrays become `Constant::Null` (mapping.rs:732-749), so
arrays-of-structs are always dropped. libcurl's dispatch table
`@protocols = [20 x ptr]` and bash's `@static_shell_builtins [77 x %struct...]` are
invisible → `conn->handler` loads are ∅ → SAF never explores the fan-out that costs
SVF 5.6s Andersen + 9.6s FS on libcurl (SAF avg pts 2.72 vs SVF 94.3 — ~35x less
points-to mass at comparable constraint counts). This under-analysis inflates SAF's
apparent wins on libcurl, bash, and tmux.
(b) unrar: frontend warns `Unsupported LLVM instruction: LandingPad/Resume` — C++
exception paths are skipped (missing constraints → smaller sets → faster).
(c) External functions without summaries contribute no constraints (under-approx)
where SVF applies extapi summaries.

## Fixes (verified designs, ranked by expected value)

### Phase 1 — Correctness of the comparison + soundness (do first)

**Fix 1.1 (V1): FS truncation soundness gate + scaled cap.**
On `iteration_limit_hit`, discard mid-flight df state (re-seed `pts` from Andersen,
clear `ver_in`/`ver_out`) so consumers fall back to the sound flow-insensitive
answer — a partially-populated `df_in` currently makes `may_alias_at` return false
`NoAlias` (fspta/mod.rs:328-366) and `points_to_at` return empty sets. Add
`converged: bool` to `FsPtaDiagnostics`; scale the default cap with graph size
(`max(100_000, K × fs_svfg.node_count())`, K≈25–50); warn via `saf_log!`; print a
`fs:TRUNC` marker in cruxbc output; make `compare-cruxbc.py` read `fspta_limit_hit`;
emit a Python `UserWarning` in `Project.flow_sensitive_pta()`.
Measured: bash converges at 103,258 pops (68.1s), tmux at 125,151 (75.1s) — the flat
100K cap truncated both. (Bench plumbing for `fspta_iterations`/`fspta_limit_hit` and
the `fspta_max_iterations` knob was already implemented in this session.)

**Fix 1.2 (V8a): Frontend — decompose pointer-bearing constant-array initializers.**
In `convert_global`, stop gating array decomposition on vtable names; decompose
whenever the element type is/contains pointers (keep the int fast path). Recurse into
`StructValue`/nested arrays instead of emitting `Constant::Null`. Keep the
`has_named_ref` guard. `extract_global_initializers` already handles nested
aggregates. Expected: SAF starts analyzing the dispatch-table fan-outs (libcurl
`@protocols`, bash builtins, tmux cmd-table) — honest numbers will be slower but
comparable to SVF's model; bash gains ~1,300 indirect call edges (SVF: 1,375 vs
SAF's 48 today).

**Fix 1.3 (V8b/c): C++ EH + externals policy.** LandingPad → synthetic load from a
per-module exception channel; `__cxa_throw` stores into it. Externals: keep ∅ default
but count and surface unmodeled pointer-returning external calls in bench stats;
extend spec coverage (~30 specs: OpenSSL EVP/SHA, zlib, inet_ntop, gmtime_r, ...).

### Phase 2 — Precision (fixes both speed and memory; biggest single lever)

**Fix 2.1 (V3): On-demand field-location creation in the solver.
IMPLEMENTED (same day) — scale-gated.** Final shape after iterating through
two unsound/two pathological intermediate designs (see below):
- `FieldMinting` overlay in `GenericSolver` (pta/solver.rs): mints
  content-addressed field cells (`make_id("loc_field", obj‖path)`, NFR-DET)
  in `handle_gep_constraints`; merged into the `LocationFactory` by
  `refine_legacy` after solving.
- **Sound-by-construction guards**: mint only pure-`Field` paths (array
  indices smash — sibling index cells would produce false NoAlias, PTABen
  `array-varIdx2`); mint only single-hop from empty-path bases (chained GEPs
  produce inconsistent path depths); canonicalize the leading pointer-deref
  `Field(0)`; per-object cap 512 → summary object; base-set fan-out cap 16
  (fat bases gain nothing and multiply cells); global ceiling 500K.
- **Read-side visibility, zero duplication**: loads from a minted cell also
  read the base cell; loads from a base also read its minted cells, with
  registrations mirrored (and inherited at mint time) so diffs re-trigger.
  Store paths stay single-target. Earlier write-side designs (mint-time set
  seeding + store fan-out) measured 18.8GB/timeout on bash.
- **OPT-IN (`SAF_PTA_FIELD_MINTING` env var), default off.** Two measured
  blockers: (1) on bash/tmux-class inputs the field-sensitive fixpoint needs
  several times the pops of the collapsed one (memory cycles run through
  minted cells; the solver's cycle collapsing is currently ineffective —
  V5: `scc_invocations=0`); (2) even where the solve converges fast, minted
  cells inflate the location universe that MSSA clobber walks and the SVFG
  multiply over — measured 4-9x END-TO-END regressions on the minting-enabled
  cruxbc targets (curl 2.3→14.1s, htop 1.7→8.7s, unrar 3.7→34.8s, RSS up to
  4x) despite improved Andersen precision. Benchmarks run real defaults, so
  the default stays off. Unblocking work: memory-region clobber partitioning
  (fix 3.2c/d), cycle machinery (3.3), dense-ID sets (phase 4).
- Measured with minting ON: PTABen exact 2270→2275/2276, unsound 137→133,
  zero category regressions; 4 dedicated solver unit tests (field precision,
  both memcpy-visibility directions, legacy fallback); determinism
  double-run byte-identical.

Original design notes (2026-08-04, pre-implementation):
- Solver-local overlay instead of `&mut LocationFactory`: the solver holds
  `field_overlay: FxHashMap<Location, LocId>` plus a per-object index
  `overlay_by_obj: FxHashMap<ObjId, SmallVec<LocId>>`, merged into the factory
  at finalize. Avoids re-borrowing `RefinementPrepared`.
- In `handle_gep_constraints` (pta/solver.rs:1218-1274): when
  `lookup_approx` would fall back past the exact/truncated path, mint the field
  cell in the overlay instead. Honor the factory's depth truncation via a new
  public `LocationFactory::effective_path()` helper.
- `LocId` = BLAKE3 `make_id("loc_field", obj.raw ‖ canonical-path-bytes)` —
  content-addressed, so IDs are independent of (FxHash-driven) creation order.
  This is REQUIRED for NFR-DET: the sequential `next_id` counter would assign
  IDs in nondeterministic order when GEP diffs iterate FxHash sets.
- Per-object cap 512 (mirror `MAX_AGGREGATE_FIELD_ELEMENTS`); past it, route to
  the base cell.
- Base↔field bridging for whole-struct ops (memcpy is modeled as Load+Store
  through the base cell): store-to-base(obj) also weak-updates every overlay
  field cell of obj; load-from-overlay-field-cell also unions the base cell's
  pts. Apply the bridge ONLY to overlay (stack/heap) cells — global aggregate
  field cells keep today's semantics to avoid PTABen churn.
- Expected: curl avg pts 69.8 → single digits, total → ~1.5s-class; makes
  fix 1.2 affordable (flip `SAF_DECOMPOSE_POINTER_ARRAYS` default after).
- Gates: PTABen (esp. memcpy/struct-copy categories), Juliet spot-check,
  determinism double-run, cruxbc with and without fix 1.2 enabled.

**Fix 2.2 (V2): Aggregate-initializer location cap + pointer-relevance gate.**
In `create_aggregate_field_locations`: recursive element count > 512 → materialize
only pointer-relevant elements (GlobalRef / function-ID Int); drop the duplicate
`[Field{0},Field{i}]` variant by canonicalizing the leading GEP index in
`resolve_gep_path`/`merge_gep_with_base_path` (must be paired or lookups miss).
Expected: curl field locations 107,811 → ~4,300; mssa 0.89 → ~0.1–0.2s; −60–70MB RSS.

### Phase 3 — FS/MSSA engine costs

**Fix 3.1 (V4): FS solver on generic PtsSet + subset pre-check + topo worklist.**
Dispatch `solve_flow_sensitive` on `pts_config.representation` (FxHash first;
Roaring after indexer registration is fixed — frozen mode panics on unregistered
`LocId`, roaring_pts.rs:144-147); skip `propagate_direct`'s unconditional clone with
an `is_subset` pre-check; Tarjan-condense the FsSvfg and pop worklist in
`(topo_rank, node)` order. Expected: curl fs → ~0.5–1.0s; bash fs pops get 3–5x
cheaper. Long-term option: hash-consed persistent PTS pool with memoized unions
(SVF's single biggest lever; determinism preserved by content-addressed IDs +
sorted iteration at boundaries).

**Fix 3.2 (V7): MSSA/modref.** (a) `points_to_contains`/`points_to_ref` on
`PtaResult` — kill the fresh-Vec-per-call + linear `contains` in `is_clobber`
(~30 lines, est. bash mssa 12.2 → 4–6s); (b) modref uses resolved indirect-call
targets, `modifies_unknown` flag instead of materializing the universe;
(c) memory-region partitioning (SVF `intra-disjoint` analog; SVF curl: 1,191 regions
vs SAF's 107,811 loc keys) — clobber cache keyed `(access, RegionId)`; bash cache
1–1.6GB → <100MB; (d) bounded/anchor-only clobber caching as the cheap alternative.

**Fix 3.3 (V5): Andersen cg_loop + cycle machinery.** Per-site examined-sets so
refinement waves scan only new pts elements; memoize `signature_compatible` per
`(TypeId, FunctionId)`; `resolve_indirect` on delta only; indexed `module.function()`
lookup; split `scc_calls`/`sccs_found` stats and fix the dead 50K-pop Tarjan cadence;
delete the structurally-dead plan-129 early-exit. Note: verification showed
process_location's 18.66s is genuine propagation (scales with V3's fat sets — Fix 2.1
is the real lever), and ~90% of cg_loop is legitimate drain_worklist re-propagation.

### Phase 4 — Memory bounding (target: bash ≤2GB, tmux ≤1.5GB)

**Fix 4.1 (V6):** `Arc<PtaResult>` into `MemorySsa` (removes the ~0.95GB deep clone;
`pta_clone` phase → ~0); don't donate SVFG to the db when `checkers=false` (drop
before FS solve); activate Roaring/frozen-indexer on both hot paths (`refine_legacy`
dispatch on `pts_config` instead of hardcoded `FxHashPtsSet`; FS solver per Fix 3.1).
Projected bash peak: 4.0GB → ~1.7–2.0GB (below SVF's 3.2GB); remaining floor is the
db's BTreeMap `PointsToMap` (~0.95GB) — interned-PtaResult follow-up → ~1.0–1.3GB.
Optional enforcement: a `memory_budget_mb` config that switches representations /
degrades to collapsed fields when exceeded, with the degradation recorded in
diagnostics.

## Sequencing note
Fix 1.2 (initializer decomposition) makes bash/tmux/libcurl slower-but-honest, and
Fix 2.1 (field sensitivity) then claws the cost back with precision. Land 1.1 first
(cheap, unblocks honest measurement), then 2.1+2.2 together with 1.2 so the corpus
numbers never regress net, then 3.x, then 4.x.

## Verification gates for any fix
- `make test` (2188 Rust + 94 pytest), PTABen exact/unsound counts unchanged
  (`make compile-ptaben && cargo run -p saf-bench -- ptaben --compiled-dir
  tests/benchmarks/ptaben/.compiled -o ...`), Juliet unchanged on affected CWEs.
- `make test-cruxbc && make compare-cruxbc` before/after (now that svf baselines are
  in `tests/benchmarks/cruxbc/svf-mem2reg-results.json`).
- Determinism: two consecutive runs byte-identical (NFR-DET-001).
- FS convergence: `fspta_limit_hit == false` on all 11 targets.
