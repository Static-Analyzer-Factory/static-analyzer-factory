# Plan 214 — Movement 2: the Anchored-Object `valid-memsafety` prover (+8..16 weighted)

**Status:** DESIGNED; the de-risking spike is ~70% already run in Python. Blocked on [`plans/213`](213-movement1-soundness-and-de-delegation.md) (it needs `universe.rs`).
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
