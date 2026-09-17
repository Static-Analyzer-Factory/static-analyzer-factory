# Plan 214 — Movement 2 RESULTS (2026-09-16)

**Status:** BUILT, measured, merge gate PASSED. Branch `movement5/submission`
(uncommitted, per the standing never-auto-commit rule).

**Headline: 132 → 142 dedup-weighted, +10. `false_alarms = 0`, `wrong_true = 0`.**
Every gate passed: the soundness sweep (0 of 10,087), the property-scoped merge gate
(20,570 tasks), the latency bar (+0.26% against +2%), and the `unreach-call`
blast-radius control (1 flip in 1,500, and it is a gain).
`valid-memsafety` goes 15 → 25, and **all 447 verdict transitions are
`Unknown → TrueCorrect`** — nothing was lost on the FALSE side. The re-cut estimate
in the SPIKE RESULT section was `+6..11`; the measured answer is **+10**, near the
top of it.

Getting there required something this plan explicitly ruled out: **four additive
frontend signals**, because without them the port is not merely lossy, it is
*unsound by construction*.

---

## 0. The correction this plan needs most

> ### ⚠️ "No frontend change is needed" is FALSE, and the failure mode is a wrong TRUE

The SPIKE RESULT section's Correction 1 is right that `compile_to_ir` already does
the right thing (`mem2reg` alone). But its conclusion — that no frontend work is
required — does not survive contact with the AIR.

**Four lines of C prove it.** These two programs produce **structurally identical
AIR**:

```c
int arr[100];  int main(void){ arr[5]   = 1; return arr[5];   }   /* in bounds */
int arr[100];  int main(void){ arr[500] = 1; return arr[500]; }   /* 2000 bytes past a 400-byte object */
```

Both become `store → @arr`. So do a store 4096 bytes past a 4-byte global
(`getelementptr inbounds (i8, ptr @g, i64 4096)`) and one 100,000 bytes past a
400-byte array (`inttoptr (add (ptrtoint @arr), 100000)`) — **all arrive as clean,
anchored, offset-0 stores, with no GEP, no cast, and no marker.** A faithful port
of §1's `Anchor { base, offset }` domain would read every one of them as in-bounds
at offset 0.

Five independent losses, each verified at `5f274d13`:

| # | loss | site |
|---|---|---|
| 1 | `AirGlobal.value_type` is never set by the LLVM frontend — a global has **no size** | `mapping.rs:621-674`; only `air_json.rs:564` sets it |
| 2 | Aggregate types are never interned; a module full of arrays/structs/unions has a type table of `{i32, i64, i8}` | `type_intern.rs:101-108` maps `%struct.S` → `Opaque` |
| 3 | `Operation::Gep` carries a `FieldPath` of type-descent steps — **no element type, no byte offset** | `air.rs:232-244`, `493-497`; GEP result type forced to `Pointer` at `mapping.rs:1250-1253` |
| 4 | Constant pointer expressions collapse to the base global's `ValueId`; `extract_at_name` takes the **first `@` in the printed text**, so even `inttoptr` collapses. When `decompose_constant_gep` *does* fire it **drops the pointer-level index** | `mapping.rs:345-370`, `426-440`, `508-514` |
| 5 | AIR **silently deletes** instructions it cannot model, with only a `tracing::warn!` | `mapping.rs:1221-1226` (catch-all), `1348` (`Skip`, covers `llvm.va_start`/`va_copy`) |

Plus two over-estimate hazards, both the wrong-proof direction:
`target_pointer_width` is hardcoded to `8` (`mapping.rs:612-614` TODO) while **100%
of the yield population is ILP32**; and `layout::alloc_size` returns `Some(0)` — a
*definite* answer — for a struct whose layout computation failed
(`type_intern.rs:263-270`).

Loss 5 is the deepest: a syntactic prover cannot be fail-closed on an IR that
deletes instructions without saying so. "No dereference appears in the AIR" is not
evidence that none occurs.

### But the damage is small, and the fix is additive

Over the 408 tasks the prototype proved, 388 compile and **387 — all 11 clusters —
contain zero `getelementptr`, zero `inttoptr`/`ptrtoint`, zero `llvm.mem*`**. The
yield population does no pointer arithmetic at all, so the losses cost ~1 task
once they are made *visible*. Four additive signals do that — three that make the
losses visible, and one that supplies a size the AIR never had:

1. **`AirGlobal.value_type` populated** (`convert_global`), giving a global a size
   for the first time. Aggregate types now intern as a side effect. (This is the
   one with a blast radius — see §2.)
2. **`IngestFidelity::collapsed_const_ptr_expr`** — set at the single point where an
   unnamed constant mentioning a global reaches the text path. A bare `@g` cannot
   trip it: named globals return earlier via the typed fast path.
3. **`IngestFidelity::dropped_instruction`** — set by the unsupported-opcode
   catch-all and by `skip_is_lossy` intrinsics (`llvm.va_start`/`va_copy`,
   `llvm.experimental.*`, `llvm.stackrestore`). Pure-metadata skips (`llvm.dbg.*`,
   `llvm.lifetime.*`, `llvm.assume`) deliberately do **not** set it.
4. **`ALLOCA_EXACT_SIZE_KEY`** — an
   `Instruction::extensions` entry carrying an alloca's size *only when exact*.
   `Operation::Alloca { size_bytes }` could not be used — it reports 8 bytes for
   every float and pointer, over-stating a 4-byte `float` and every ILP32 pointer
   slot. Needed once stack slots were allowed to anchor; see §1.

`IngestFidelity` lives on `AirBundle`, not `AirModule`: it describes the
*conversion*, like `frontend_id`, and `AirBundle` has 2 literal construction sites
against `AirModule`'s 150. It serializes to nothing when faithful
(`skip_serializing_if`), so every existing snapshot and consumer is byte-identical.

---

## 1. What the domain actually became

§1 specifies `Anchor { base: ObjBase, offset: i64 }` propagated through GEPs. That
domain is not implementable on AIR (losses 3 and 4). The sound residue:

> **An address is anchored only when it IS a global or a stack slot — offset zero
> by construction — and every instruction that could move an address off its base
> is rejected outright.**

Obligations as built:

1. **Heap-free.** `HeapAlloc` rejected in-IR; `malloc`/`calloc`/`realloc`/`free`
   rejected by the external policy. This discharges `valid-free` and
   `valid-memtrack` vacuously.
2. **Non-inert libc rejected.** A tight allowlist; `memcpy`/`memset`/`strcpy`/
   `sprintf`/`scanf`/`read`/`qsort`/`bsearch` and `printf` are all absent.
3. **Bounds.** Every reachable `Load`/`Store` address is a global's or a stack
   slot's `ValueId`, and `access_width <= exact_size_of(object)`.
4. **Abstain on everything else** — `Gep`, `Memcpy`, `Memset`, `IntToPtr`,
   `PtrToInt`, indirect calls, and any unfaithful ingestion.

### Two corrections to §1 that the measurement forced

**`ObjBase::Stack` should NOT be dropped — only barred from *deriving*.** The SPIKE
RESULT's GLOBALS_ONLY correction is right that a stack *anchor* is unsound in
general, but implementing it as "reject any reachable `alloca`" cost **378 of the
408 provable tasks** for no soundness gain. The scope hazard
(`{ int y; p = &y; } *p = 1;`) requires the address to escape through a pointer
variable — a store then a load — and **a load result never anchors**, so the
dereference is rejected before scope is ever in question. A *direct* access to a
slot's own `ValueId` can only appear inside the function whose activation owns the
frame. Yield: 13 tasks / 1 cluster → 408 / 11.

**The external allowlist must be WIDER than `race_true`'s, not narrower.**
`race_true` excludes `pthread_cond_*`, `pthread_rwlock_*`, `pthread_mutex_trylock`
and the barrier/semaphore family because it does not model their happens-before
effect. This prover is thread-*insensitive* and never reasons about ordering, so a
primitive's scheduling semantics are irrelevant — only its memory effect is, and
that is confined to its handle. Admitting them recovered `weaver`,
`pthread-atomic` and `pthread-deagle`.

---

## 2. Measured

### Yield — 408 tasks, 11 clusters

Run over the identical expected-TRUE population the prototype was measured on
(`m2-FINAL-yield.jsonl`, 1,099 tasks), so the porting attrition is a number rather
than an estimate.

```
prototype PROVE (raw LLVM IR): 408  in 11 clusters
RUST      PROVE (AIR)        : 408  in 11 clusters

pthread-wmm 225, weaver 75, goblint-regression 28, pthread 22, locks 13,
pthread-theta 13, pthread-ext 9, termination-memory-alloca 9, pthread-atomic 8,
ldv-races 4, pthread-deagle 2
```

Attrition is **10 tasks**: `non-inert-external:printf` 8, the one
constant-expression-GEP task, and one pointer-width load. The Rust port also proves
**10 tasks the prototype did not** (8 `store-unanchored`, 2 `load-unanchored` in
the prototype's own histogram), in `termination-memory-alloca` — a cluster the
prototype missed entirely. Net cluster set differs by one in each direction:
gained `termination-memory-alloca`, lost `pthread-C-DAC` (1 task, to `printf`).

### Soundness — 0 of 10,087, the KILL(b) gate

`scripts/p211_sweep_soundness.py valid-memsafety` over the **complete**
expected-FALSE population:

```
PROVE (would-be WRONG TRUE) = 0

ABSTAIN:ingest-collapsed-const-ptr-expr  9894
ABSTAIN:non-inert-external:free            55
(no source / compile-fail)                 51
ABSTAIN:gep                                32
ABSTAIN:ingest-dropped-instruction         14
ABSTAIN:heap-alloc                          9
ABSTAIN:load-unanchored                     9
ABSTAIN:non-inert-external:printf           8
```

Run twice: once with the strict prover, once after the alloca/allowlist
loosening that tripled the yield. Both 0. The 51 unparsed rows are clang compile
failures on ldv CIL files — a pre-existing ingestion limit shared with the other
provers, and not a PROVE.

**`plans/214` §4 names the sweep key `memsafety`, which is the wrong string.** The
per-task dump's `property` field is the `.prp` stem — `valid-memsafety`. Keying `SUBCMD` on
`memsafety` matches zero rows and prints a vacuous `PROVE = 0` pass. Both
`p211_sweep_soundness.py` and `p211_probe_unsolved.py` now use the correct key.

### Latency — the +2% bar is 69.7 ms/task, and the staged gate clears it

§4's "+2%" is tighter than it sounds: `valid-memsafety` is **19.92 CPU-hours over
20,570 tasks** (mean 3.49 s, median 4.00 s), so 2% is **1,434 s total — 69.7 ms per
task**. A per-task Andersen PTA would blow that several times over.

It is never paid. Staged over the corpus:

| stage | tasks | share |
|---|---:|---:|
| rejected before any call graph (heap / non-inert libc) | 19,890 | **96.59%** |
| cheap universe gate only (`CallGraph` + BFS, `MainOnly`) | 187 | 0.91% |
| **escalates to Andersen PTA + ICFG + MTA** | **515** | **2.50%** |

`fast_paths::reachable_spawns_threads` already existed and is sound in the needed
direction (it returns `true` on any reachable indirect call), so no new pre-check
was written. The 515 escalating tasks are exactly the yielding clusters
(`pthread-wmm` 283, `weaver` 76, `goblint-regression` 66…) — the PTA cost lands
only where it can be paid for in points.

### Blast radius of the frontend change

Interning a global's type adds entries to `AirModule.types`, and
`absint::build_obj_type_map` maps an alloca to a struct type **only when exactly
one struct shares its size** — so a new entry can *drop* an existing mapping as
well as add one. That is a precision change, not an additive one. Only an
**anonymous** struct global can trigger it (`%struct.S` interns as `Opaque`).
Sampled, 300 tasks per property:

| property | tasks with an anonymous struct global |
|---|---:|
| `no-overflow` | **0 of 276 (0.00%)** |
| `valid-memsafety` | 3 of 295 (1.02%) |
| `unreach-call` | **48 of 268 (17.91%)** |

`no-overflow` is unaffected outright. `valid-memsafety` is covered by the merge
gate. `unreach-call` got its own control — at 22,631 tasks and a 21.4 s mean the
full property is ~17 h, so `scripts/m2_unreach_control.sh` ran a deterministic
1,500-task stride sample and `scripts/m2_ab.py` diffed it task-for-task against the
55,690-task baseline. **Result: 1 transition in 1,500 (0.07%)**, and it is a *gain*
(`Unknown -> FalseCorrect`, one `eca-rers2012` task), with `false_alarms = 0` and
`wrong_true = 0` on both sides.

`eca-rers2012` is the cluster `plans/213` §"FINAL MEASUREMENT" already identified as
`unreach-call`'s timeout-tail noise, so the honest reading is **inert, not
improved** — one flip at 0.07% is inside the noise floor, and it happens to have
landed on the favourable side. The frontend change does not perturb `unreach-call`
in practice despite 17.9% of its tasks being structurally exposed to it.

### The merge gate — PASSED

`scripts/m2_memsafety_gate.sh`, the same flags as the Movement 1 run so the rows
splice into the same baseline. 20,570 tasks, 4 h 05 m.

```
BASELINE    saf-alone-20260913                  120   {ndr 9, nov 22, term 26, unreach 48, mem 15}
MOVEMENT 1  (4 properties measured)             132   {ndr 9, nov 33, term 27, unreach 48, mem 15}
MOVEMENT 2  (+ Anchored-Object memsafety)       142   {ndr 9, nov 33, term 27, unreach 48, mem 25}

  Movement 1 delta : +12   (120 -> 132)
  Movement 2 delta : +10   (132 -> 142)

  valid-memsafety transitions: {'Unknown -> TrueCorrect': 447}   <- and NOTHING else
```

The splice reproduces both published baselines exactly, which is what licenses the
comparison. Per-property: `TP` unchanged at 6,008, `FP` 0, `unknown` 14,509 →
14,062 — down by exactly the 447 that became TRUE. The TRUE arm took nothing from
the FALSE side.

447 > the 408 of the yield run because the yield run was scored over the
prototype's 1,099-task sample; the gate covers all 20,570. Clusters gained beyond
that sample: `termination-crafted` 48, `ldv-regression` 6.

### The gate was run on a binary two commits old — re-verified

Two changes landed after the gate started: the empty-thread-discovery guard (which
can only *remove* proofs) and the precomputed global-size map (a pure refactor).
Re-proving the gate's exact TRUE set with the final binary:
**447 / 447 still PROVE**, and the full 10,087-task soundness sweep re-run on that
same binary is again **0 PROVEs**. The +10 stands as measured.

---

## 3. Where the soundness argument rests on something other than a computation

Stated plainly, because it is the one place this prover is not purely syntactic.

An admitted synchronisation primitive (`pthread_mutex_lock`, `pthread_create`, …)
writes `sizeof(handle_type)` bytes through its handle argument. The prover requires
that argument to be an anchored global or stack slot, but does **not** size-check
it — there is no `sizeof(pthread_mutex_t)` available in AIR, and hardcoding a
glibc table would be a platform assumption that silently breaks on musl or a
different arch.

What discharges it instead is the source's type discipline: the argument's declared
C type *is* the parameter type unless the program casts, an instruction cast is
rejected, and a constant-expression cast sets `collapsed_const_ptr_expr`. The
residual hole is LLVM 18's no-op pointer-to-pointer cast, which is invisible — but
it is invisible identically for a global and for a stack slot, so admitting the
stack added no risk class that admitting globals did not already carry.

Settled empirically rather than by argument: **0 PROVEs over all 10,087
expected-FALSE tasks**. If a survivor ever appears here, the fix is a size check,
not another allowlist entry.

---

## 4. Incidental findings

* **`saf index --output` is broken for any module with a function-pointer
  constant** — `Error: cannot serialize tagged newtype variant Constant::GlobalRef
  containing a string`. It hit 375 of 408 tasks when used as an inspection tool.
  Pre-existing, unrelated to this movement, and worth its own fix: it makes AIR-JSON
  export unusable on exactly the concurrent programs.
* **`p211_probe_unsolved.py` cannot run at this commit.** It hard-codes
  `lever1-pertask.jsonl` *and* `lever1-true99-pertask.jsonl`; the latter does not
  exist and there is no CLI or env override.
* **`rsync -a` + `cargo` is a trap.** `-a` preserves mtimes, so syncing sources
  whose mtime predates a cached artifact leaves cargo convinced it is up to date —
  it compiled `saf-cli` against a stale `saf-core` rlib and reported
  `no field 'fidelity' on type 'AirBundle'` against source that plainly had one.
  `touch` the synced files.

---

## 5. Quality gates

* `cargo nextest run --workspace --exclude saf-python` — **2728 passed, 0 failed**
  (unchanged from the branch baseline)
* `cargo clippy --workspace -- -D warnings` — clean
* `cargo fmt --check` — clean
* `--run-ignored all` — **12 failures, byte-for-byte the pre-existing set**,
  verified against a control run at `5f274d13` in the same tree (which showed those
  same 12 plus the 10 new `memsafe_prover` tests failing only because the
  subcommand was stashed away). **Zero new failures.**
* 10 new `#[ignore]`d end-to-end regressions in
  `crates/saf-cli/tests/memsafe_prover.rs`, all passing. Each negative fixture is a
  program the prover answers TRUE on if its obligation is removed:

| fixture | pins | without it |
|---|---|---|
| `memsafe_bounds_oob_width.c` | obligation 3, globals | 4-byte store into a 1-byte global proves |
| `memsafe_bounds_oob_stack.c` | obligation 3, stack | same, via `ALLOCA_EXACT_SIZE_KEY` |
| `memsafe_noninert_libc.c` | obligation 2 | `puts` walks a non-NUL-terminated `char[4]` |
| `memsafe_heap_alloc.c` | obligation 1 | a leak and a heap store go unmodelled |
| `memsafe_collapsed_constexpr.c` | ingest fidelity | a 2000-byte overrun proves |
| `memsafe_thread_body_unsafe.c` | thread bodies are scanned | a vacuous proof over main alone |
| `memsafe_prove_threaded_globals.c` | the positive case | — |

---

## 6. Wiring

`memsafety_strategy` runs the ASan FALSE confirmer **first**; the TRUE arm only
sees what it could not reproduce. The two are disjoint by construction — obligation
1 rejects every allocator and every ASan-confirmable class needs either an
allocation or an unanchorable address — but that is an argument, not a mechanism,
so the ordering makes the argument being wrong cost recall rather than −32.

The TRUE arm emits `VerdictOutcome { verdict: "true", witness: None, graphml: None,
correctness: None }` — verdict-only. **`plans/214` §3 says "like
`termination_strategy`"; that is wrong.** `termination` needs a 2.1+ correctness
witness under the 2027 rules and emits `correctness: Some(..)`. The verdict-only
exemplar is `race_true_outcome`, and `svcomp_witness_rules.py:173` confirms
`TRUE_WITNESS_RULE["valid-memsafety"] = {_DEFAULT: None}`.

End to end through `saf verify --property valid-memsafety.prp`:

```
memsafe_prove_threaded_globals.c  ->  true
memsafe_collapsed_constexpr.c     ->  false(valid-deref)   (ASan, before the TRUE arm)
memsafe_bounds_oob_width.c        ->  false(valid-deref)   (ASan, before the TRUE arm)
```
