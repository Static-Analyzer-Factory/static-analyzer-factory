# Plan 214 — Movement 2: the Anchored-Object `valid-memsafety` prover (+8..16 weighted)

**Status:** DESIGNED; the de-risking spike is ~70% already run in Python. Blocked on [`plans/213`](213-movement1-soundness-and-de-delegation.md) (it needs `universe.rs`). **Scope grew by one property — see the folded-in note in §0.**
**Branch:** cut `movement2/anchored-memsafety` off `movement1/sound-then-cut`.
**Track:** TRUE-side. Movement 2 of 4 — **the first offensive movement.**
**Design source:** `plans/211` §5.2, §5.5, §5.6. Memory: `saf-plan-211-design`, `svcomp-2027-witness-rules-50pts-at-risk`.

---

## 0. Goal and why this target

A sound, verdict-only `valid-memsafety` TRUE prover. **43 unsolved clusters, every one of
them verdict-only**, and SAF has *zero* memsafety TRUE capability today so every cluster
is additive.

`valid-memsafety` is the only property where no external validator can ever block a
point — `C.valid-memsafety.<any suffix>` is "not supported" in the 2027 rules,
**including Concurrency, uniquely among the five properties.** Contrast with
`no-overflow`, where 0 of 61 unsolved clusters are verdict-only and the bundled validator
confirmed **0 of 5** of SAF's existing proofs.

Per `plans/211` §5.2, 55 of the 180 unsolved TRUE clusters are verdict-only and this
property is 43 of them.

> ### FOLDED IN FROM MOVEMENT 0 — take `valid-memcleanup` in the same movement
>
> 2027 makes `C.valid-memcleanup.*` the **only** base category in the entire table that is
> "not supported" on BOTH columns: neither a TRUE nor a FALSE needs a witness. It is the
> highest points-per-witness-effort row on the board, and it is currently invisible because
> SAF's manifest omits the property.
>
> **It is opt-in, not excluded.** `scripts/svcomp_split.py`'s `DEFAULT_PROPERTIES` lists
> five properties and its comment says "`valid-memcleanup` is opt-in via `--properties`".
> So step one is re-running the split with it enabled — no code, just a flag. The corpus
> has **93 tasks across 10 group dirs** (`uthash-2.0.2` 27, `Juliet_Test` 24,
> `forester-heap` 15, `list-properties` 7, `heap-manipulation` 5, `list-ext3-properties` 4,
> `memsafety` 4, `verifythis` 4, +2 more), in base categories
> `C.valid-memcleanup.Main` (← Heap, Juliet, LinkedLists, VerifyThis-Loops,
> VerifyThis-Recursive) and `C.valid-memcleanup.SoftwareSystems-uthash`.
>
> **Anchored-Object gets it almost free, and for a principled reason.** §1's obligation 1
> rejects any reachable `malloc`/`calloc`/`realloc`/`free`/`alloca`, so the provable set is
> exactly the heap-free programs — and a program that never allocates cannot leak. The
> memcleanup proof is therefore the memsafety proof's own precondition, discharged by the
> same reachable-universe walk. Expect the incremental cost to be a property-routing arm
> plus tests, not a second prover.
>
> Two caveats, both from Movement 0's ledger: the denominator `398` quoted in `plans/211`
> §1 is understated because it omits this property, so re-derive it; and the 93 tasks'
> expected verdicts must be checked before claiming upside — a `valid-memcleanup` TRUE is
> only worth +2 where the task's expected verdict is actually `true`.

> ### FOLDED IN FROM MOVEMENT 0 — run the oracle probe before building
>
> Movement 0B's ranking-witness slice landed on the EXACT ceiling of CPAchecker's own
> producer+validator pipeline (10 of 19 clusters), and that ceiling was discoverable in
> ~45 minutes of probing *before* ~750 lines of Rust. Generalise the method: **before
> building a prover for a cluster set, check whether the best available tool can already
> do it, and let that set the estimate.**
>
> This movement is the one case where the probe's usual form does not apply — memsafety
> TRUE is verdict-only in 2027, so there is no validator to satisfy and no witness to
> reconfirm. That is precisely why it is the strongest target, and it should be stated as
> such rather than assumed. What IS worth probing cheaply: run a mature memsafety prover
> over the 43 unsolved clusters to bound how many are provable AT ALL by anyone. A cluster
> no tool can prove is one the Anchored-Object domain will not prove either, and knowing
> that before slicing converts the `+8..16` range into a measured number.

## 1. The design — Anchored-Object

Thread-**insensitive**, syntactic, fail-closed. Modelled on `termination.rs` exactly as
`plans/211` §4 recommends: prove or abstain, never guess, whitelist everything.

**Domain.**
```rust
enum ObjBase { Global(ValueId), Stack(ValueId /* the Alloca */) }
struct Anchor { base: ObjBase, offset: i64 }   // flat lattice, explicit ⊥ = "not anchored"
```
Joining two different bases gives ⊥. **A `Load` result never anchors** — this single rule
is what makes the prover safe against dangling pointers, aliasing, and interference: a
pointer that came out of memory can never be the subject of a discharged obligation.

**Obligations,** over the Movement-1 `reachable_universe`:
1. Reject any reachable `malloc` / `calloc` / `realloc` / `free` / `alloca` /
   `__builtin_alloca`. The first increment proves only heap-free programs.
2. Reject any reachable libc call that could dereference a caller-supplied pointer —
   `memcpy`, `memset`, `strcpy`, `strcat`, `sprintf`, `scanf`, `read`, `qsort`, `bsearch`
   are **not** inert. `race_true.rs:145-149` already encodes this rule; reuse it. **This
   is the obligation whose absence caused the one soundness escape measured in the
   spike** — implement it before trusting any number.
3. Every reachable `Load` / `Store` / `GEP` / `memcpy` / `memset` address must be
   `base + constant`, with `offset + access_width <= size_of(base)` from the AIR's static
   type layout.
4. Abstain on anything else. No new abstract domain, no SMT, no interference reasoning.

**Why concurrency is free.** Every obligation is a syntactic property of the SSA pointer
graph and static layout; none mentions a runtime value. Interference can only change
values, so no interleaving, reordering or thread count can invalidate a discharged
obligation. The only concurrency-sensitive input is *which code runs*, and the universe
over-approximates that as `main-tree ∪ ⋃ thread-entry trees` — the same over-approximation
`race_true.rs:551-556` already uses, which has held FP=0 across the full 55,690-task run.

`valid-free` and `valid-memtrack` are discharged **vacuously** by obligation 1 (nothing is
ever allocated or freed). Only `valid-deref` does work.

**New code is small.** The universe gate comes from `plans/213`. The only genuinely new
pieces are `crates/saf-core/src/layout.rs` (`size_of` over `AirType` — the AIR already
carries `StructField::byte_offset` / `byte_size` and `total_size`, so this is mechanical;
`Opaque`, `Array{count: None}` and any missing offset return `None` ⇒ abstain) and the
anchor propagation itself.

## 2. Slice A — finish the spike (1 engineer-day, still zero Rust)

The decision procedure is already prototyped over LLVM IR:
`scripts/p211_anchor_prototype.py` and `scripts/p211_anchor_falsescan.py`.

**Measured 2026-09-12:**
- **Yield: ≥1 provable task in 10 of 16 concurrent clusters** — `pthread-theta` 13/13,
  `pthread-atomic` 8/8, `pthread-wmm` 20/25, `pthread-ext` 17/25, `weaver` 14/24,
  `pthread` 11/25, `goblint-regression` 9/25, `ldv-races` 4/11, `pthread-C-DAC` 2/4,
  `pthread-deagle` 2/4 — plus 3 of 27 sequential. Zero in `libvsync` / `pthread-divine`
  (`inttoptr`) and the heap-using clusters, as designed.
- **Soundness: 1 escape in 1,180** expected-FALSE tasks, and that escape was exactly
  obligation 2, which the prototype had not implemented. That is the good failure mode —
  a missing gate, not a broken argument.
- **Required IR recipe, itself a deliverable:** `clang-18 -O0 -Xclang -disable-O0-optnone`
  then `opt-18 -passes=sroa,mem2reg,instcombine`. Without `mem2reg` the `-O0` pointer
  spill defeats anchoring outright.

**Remaining spike work:** implement obligations 2 and 3 (so `memset(p,0,81)` into an
80-byte object is rejected), then re-run both sweeps — the yield run over all 43 unsolved
clusters, and the soundness run over **all 10,014** expected-FALSE `valid-memsafety`
tasks, not a sample.

### Spike gate

- **PASS:** ≥7 of the 16 concurrent clusters yield a clean task with the bounds obligation
  on, **AND** 0 of the 10,014 expected-FALSE tasks survive.
- **KILL (a):** fewer than 7 ⇒ the binding constraint is the universe gate (non-inert
  externals, indirect calls, dropped terminators), not the anchoring. The estimate falls
  under +10 and `plans/215` (no-overflow) dominates instead — go there.
- **KILL (b), absolute:** any expected-FALSE survivor that needs a second, third and
  fourth ad-hoc gate. That is evidence the syntactic argument does not close, and one
  uncaught wrong proof is −32 uncapped.

## 3. Slice B — the Rust prover

Only on a PASS. Mirror `prove_no_overflow_cmd` / `prove_unreachable_cmd`:

1. `crates/saf-core/src/layout.rs` — `size_of` / `gep_byte_offset`.
2. `crates/saf-svcomp/src/memsafe.rs` — `prove_memsafe(&AirModule) -> MemSafeProof`
   (`Proven | Abstain(String)`), consuming `universe::reachable_universe` with the
   memsafety `ExternalPolicy`.
3. `saf memsafe-prove` dev subcommand printing one line `PROVE` / `ABSTAIN:<reason>`,
   so `scripts/p211_probe_unsolved.py` and `scripts/p211_sweep_soundness.py` work on it
   unchanged. **Add the subcommand to both scripts' `SUBCMD` maps.**
4. Only then wire a TRUE arm into `memsafety_strategy`. It emits **no witness** —
   verdict-only — so the path is `VerdictOutcome { verdict: "true", correctness: None, .. }`,
   like `termination_strategy`.

## 4. Merge gate

- [ ] `p211_sweep_soundness.py memsafety` = **0 PROVEs** over all 10,014 expected-FALSE.
- [ ] Full 55,690-task run: `false_alarms == 0`, `wrong_true == 0`, weighted **+8 or
      better** over the post-Movement-1 baseline.
- [ ] Latency: memsafety is 20,570 tasks; keep the added CPU under +2% (Lever #1's bar was
      +0.4%). The prover is syntactic and PTA-bounded, so this should be easy — measure it
      anyway, per `saf-lever1-cbmc-loop-free`.
- [ ] `make lint` clean, TDD-green.

## 5. Risks and honest caveats

- **The +8..16 range rests on a Python prototype**, not the Rust prover. SAF's real
  universe gate is stricter than the prototype's (external whitelist, no reachable
  indirect calls), so expect attrition. The discount is believed small because
  `race_true` already clears that gate on `pthread-wmm`, `weaver`, `pthread` and
  `goblint-regression` — but it is a belief until Slice B measures it.
- **Do not let this become a shape analysis.** The moment obligation 1 is relaxed to admit
  heap, the syntactic soundness argument in §1 collapses and every number here is void.
  Heap memsafety is a separate plan, not a slice of this one.
- **`plans/211` §5.2 previously reported 95 concurrent clusters; the real figure is 50.**
  Classify by `Concurrency.set` membership, never by grepping sources for `pthread_create`
  (that sweeps in `Sequentialized.set` and ldv drivers scored as sequential).

---

# SPIKE RESULT (2026-09-16) — **PASS**, and two corrections to this plan

Artifacts on the VM: `scripts/m2_anchor_prototype.py` (the finished prototype),
`m2-FINAL-yield.jsonl`, `m2-FINAL-false.jsonl`. Every number below was re-verified
directly from those files, not taken from a report.

## The gate

| §2 criterion | required | **measured** |
|---|---|---|
| concurrent clusters yielding, bounds obligation ON | ≥ 7 of 16 | **10 of 16** |
| expected-FALSE `valid-memsafety` tasks surviving | 0 | **0 of 10,087** |

Yielding concurrent clusters: `pthread-wmm` 225/283, `weaver` 76/174, `goblint-regression`
35/116, `pthread` 22/41, `pthread-theta` 13/13, `pthread-ext` 9/34, `pthread-atomic` 8/8,
`ldv-races` 4/11, `pthread-deagle` 2/4, `pthread-C-DAC` 1/4. Plus one sequential = 11
clusters, 408 of 1,099 expected-TRUE tasks proved.

Obligations 2 and 3 are demonstrably live in that run: `llvm.memset` 1,026,
`llvm.memcpy` 208, `llvm.memmove` 208 (obligation 2), `gep-out-of-bounds` 330
(obligation 3), `non-inert-external` 1,005, `reachable-indirect-call` 112.

**KILL(a) does not fire** (10 > 7). **KILL(b) does not fire**: the escapes that existed
were closed by ONE domain restriction, not by stacking ad-hoc gates.

**Verdict-only confirmed independently.** All 10 base categories the yield population
occupies return `witness_required=False` from `svcomp_witness_rules::true_witness_requirement`
— Concurrency included. There is no validator to satisfy and no witness to emit.

## ⚠️ CORRECTION 1 — §2's prescribed IR recipe is ACTIVELY HARMFUL

§2 mandates `opt-18 -passes=sroa,mem2reg,instcombine`. Measured survivors
(= would-be wrong TRUEs) by pipeline, obligations 2+3 on, globals gate off:

```
  sroa,mem2reg,instcombine   11
  sroa,mem2reg                8
  mem2reg                     2   <- SAF's own production pipeline
  (no opt)                    0
```

`sroa` and `instcombine` EXPLOIT the undefined behaviour the prover exists to detect:
they rewrite a genuinely unsafe access into one that looks safe, turning an
expected-FALSE task into a wrong TRUE. **Use `mem2reg` alone — which is already what
`compile_to_ir` does.** No frontend change is required; this plan's §2 recipe should
simply be deleted.

## ⚠️ CORRECTION 2 — "without mem2reg the -O0 pointer spill defeats anchoring outright"

§2 states this as a required deliverable. It is **false for the concurrent clusters**:
concurrent-cluster yield is **10 of 16 under all four pipelines above**, because those
clusters anchor on GLOBALS and on `pthread_t` slots, not on promoted stack pointers. It
is true for the sequential clusters, which drop 3 → 1 without the passes. Since the
concurrent clusters are the prize, the premise does not bind.

## The one real design change: GLOBALS_ONLY

The zero-survivor result requires restricting anchor BASES to globals
(`M2_GLOBALS_ONLY=1`, the prototype's default). Reason: clang hoists every block-scoped C
local to a function-entry `alloca`, and without lifetime intrinsics the C scope is simply
gone from the IR — so `{ int y; p = &y; } *p = 1;` is indistinguishable from an in-scope
access, and the prover would answer TRUE on a use-after-scope FALSE task. A global is live
for the whole program, so restricting the base kind removes the question entirely.

This is a **tightening of obligation 1** — from "proves only heap-free programs" to
"heap-free AND stack-anchor-free" — not a new ad-hoc gate, which is why it does not trip
KILL(b). §1's `ObjBase` should drop the `Stack(ValueId)` variant in the first increment.
Verified: all 5 `memsafety-ext3/scopes*` expected-FALSE tasks are rejected under
production `mem2reg` + GLOBALS_ONLY.

## Re-cut the estimate: +6..11, not +8..16

The measured prototype ceiling in the sound configuration is **11 clusters = +11
weighted**, BEFORE any SAF-side attrition (the real universe gate is stricter than the
prototype's, the AIR is lossier than raw LLVM IR, and the Rust port will lose more).
§5's own caveat applies. `+8..16` is above the evidence at the top end.

## A weaker denominator than the headline suggests

Of the 10,087 expected-FALSE rows, only **2,422 are in a 2027 base category**; 7,656 are
`Unused_Juliet`-only and 9 have no `.set` at all. Survivors among the live 2,422 are also
**0**, so the verdict stands — but the statistical power against the corpus that actually
scores is 2,422, not 10,087. (The population is 10,087 rather than §2's 10,014 because the
prototype's `parse_yml` was fixed: it had matched only *quoted* `input_files` and was
silently skipping tasks.)

## Latency, unbudgeted by this plan

§4 sets a +2% CPU bar. A spawn-admitting universe requires Andersen PTA + ICFG + MTA per
task (`race_true.rs:519-539`). `race_true` pays that on 1,031 `no-data-race` tasks; this
prover would pay it on **20,570**. Gate it behind a cheap syntactic spawn pre-check so the
majority-sequential population skips PTA entirely, or the bar is missed on plumbing rather
than on proving.
