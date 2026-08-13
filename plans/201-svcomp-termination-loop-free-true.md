# Plan 201: `termination` (loop-free ∧ acyclic-callgraph) TRUE — SAF's first sound-TRUE slice (R7)

**Status: DONE + REVIEW-HARDENED (Slices 1–3 + a pre-commit adversarial review, TDD, VM-green, UNCOMMITTED
2026-08-13). Acceptance MET** — SAF emits its first sound `true`. **A high-effort pre-commit adversarial review
(§15) found & fixed 3 real −32 bugs the scaled audit had missed** (global constructors, global destructors, and
`indirectbr`/computed-goto — all cases of code/control-flow the frontend drops or that runs outside `main`). The
−32 audit passed at scale (WRONG_TRUE=0 over 964 non-terminating tasks, FALSE_EMITTED=0), recall ~16%,
byte-deterministic; full `make test` (**2291 nextest** + 94 pytest) + clippy `-D warnings` + fmt clean.
**Commit only when the user asks.** Branch `svcomp`. Laptop = git source of truth; all
builds/experiments on the VM (`ubuntu@cd-vm-15-ai-vm`). Follows plan 199 (R6 `no-overflow`, committed `5afa17e`).
Implements roadmap **R7** (`plans/193` §4.4/§7); supersedes the scoping stub `plans/200`.

Grounded by a this-session Slice-0 de-risk (§13): a primary-source + adversarially-verified SV-COMP scoring
resolution, a 4-agent code recon, and a **full-reservoir** structural-prevalence measurement on the VM.
References: `plans/199` (the confirmer-first template + the FP/premise-check discipline), `plans/200` (scoping),
CLAUDE.md redlines #1/#6/#8, [[svcomp-termination-witness-not-required]], [[saf-svcomp-capability-findings]],
[[saf-svcomp-workflow]], [[saf-svcomp-vm-env]], [[svcomp-yaml-witness-2.0-format]].

---

## 0. The one-paragraph shape

`saf verify --property <termination.prp> ...` currently prints `unknown` on 100% (`strategy_for` has no
`Termination` arm). R7 adds a `termination_strategy` that emits **`true`** — SAF's **first-ever TRUE verdict** —
**iff** a purely-static, deterministic **structural proof** holds: the program has a defined `main`, every
function reachable from `main` has a **loop-free (DAG) CFG**, the reachable **call graph is acyclic** (no
recursion), there is **no reachable unresolved indirect call**, and every reachable **external** callee is a
**known-terminating** one. That is a *sufficient* (decidable, sound, incomplete) condition for "always
terminates". Otherwise it abstains (`unknown`); it **never** emits `false`. Unlike R1–R6 (FALSE-only via runtime
confirmers), R7 asserts TRUE from a static proof — so soundness is the whole game (a wrong TRUE is **−32**). The
de-risk (§13) resolved the two big unknowns: **(1)** termination TRUE is **witness-not-required** in SV-COMP
2026 (scored on the verdict alone; **no correctness-witness pipeline needed** — this *inverts* `plans/200`
§3.1), and **(2)** the loop-free∧acyclic subset is **190/1434 = 13.25%** of terminating tasks (→ ~15.7% with a
libc-allowlist extension) — a cheap, sound, nonzero footprint, well clear of the R4 "~nil → stop" bar.

---

## 1. The decision and why (Slice-0-evidence-backed)

`termination` is the roadmap's **first sound-TRUE slice** and a strategic inflection: R1–R6 all emit FALSE via a
concrete runtime confirmer (replay/ASan/UBSan); R7 emits **TRUE from a static structural proof**. The sufficient
condition:

> **A program whose every reachable function has a loop-free CFG (a DAG — no back-edge) AND whose reachable call
> graph is acyclic (no recursion) AND whose reachable externals all terminate-and-return has only finite
> executions ⇒ it always terminates ⇒ `termination = TRUE`.**

Sound, **incomplete** — most terminating programs loop (needing ranking functions, out of scope) — so recall is
bounded to the loop-free∧non-recursive subset; it abstains (`unknown`) otherwise and **never** emits `false`
(non-termination FALSE = a lasso/cycle counterexample, a separate future confirmer, §12). The de-risk (§13)
falsifies plan 200 §3.1's premise and sizes the payoff → **GO**.

---

## 2. The scoring reframe — CONFIRMED (corrects `plans/200` §3.1)

`plans/200` §3.1 called "the correctness-witness pipeline … the real risk … [it] may dwarf the verdict logic."
**The de-risk (§13a) falsifies that.** From SV-COMP 2026 `rules.php` (primary source, 3/3 adversarial verifiers
CONFIRMED, independent WebFetch agreed): termination **correctness** witnesses are *"2.1\* (demo mode)"*, and the
rule states verbatim *"No witnesses for the TRUE results are required in the base categories where the table says
that the correctness witnesses are not supported or supported only in a demo mode."* Stable across 2022–2026.

**⇒ R7 emits a bare `true` and needs NO correctness-witness code.** SAF's existing verdict seam already handles
this: the witness-write gate is `if outcome.verdict.starts_with("false")` (so a `true` writes no witness), and
BenchExec `saf.py` already maps a bare `"true"` line → `RESULT_TRUE_PROP`. A correct `termination=true` scores
**+2 on the verdict alone**. (A termination correctness witness, if ever needed, is a **YAML 2.1** `invariant_set`
with `loop_transition_invariant`s — SAF's committed `witness_yaml.rs` is 2.0-only; for a loop-free program it
would be an empty `invariant_set` shell. Deferred, §12.)

**Strategic caveat (documented, non-blocking — the user chose "verdict-only R7" with this in view):** the
**C.FalseOverall** meta-category (SAF's stated home as a FALSE-only bug-finder) **excludes the whole Termination
meta-category and ignores all TRUE results.** So R7's +2s land in the **Termination category** and **C.Overall**,
**not** in C.FalseOverall. R7's value is therefore **(a)** SAF's first *sound TRUE* + a **reusable sound-TRUE
verdict spine** that R8 (`no-data-race` no-threading TRUE, also witness-not-required) inherits, and **(b)** a
partial Termination-category / C.Overall entry sized by §13b.

---

## 3. Soundness redlines (CLAUDE.md — R7 is where they FIRST bind for a TRUE verdict) and how R7 satisfies each

- **#1 (never `true` except on a proven path):** R7 relaxes the hard "never prints `true`" gate **ONLY** for the
  §4 structural proof. `termination_strategy` returns `true` only when `program_structurally_terminates` holds,
  else `unknown_outcome()`. Every other TRUE-capable branch stays gated/deleted (see #6).
- **#6 (delete the unsound branch):** `analyze_termination` (`property.rs:1441`, **dead** — zero live callers,
  §13c) has an unsound `!conservative ⇒ True` branch (`:1462`) **and** an unsound loop-free-only ⇒ True branch
  (`:1455`, no acyclic-callgraph check → a loop-free *recursive* program → wrong TRUE). Per the approved decision,
  **neutralize the whole dead function to always-`Unknown`** (remove *both* TRUE-capable branches), and remove any
  helpers thereby orphaned (`find_nonterminating_loops`, and `program_is_loop_free` if it has no other live
  caller) to keep clippy `-D warnings` clean. R7's real logic lives in the fresh `termination_strategy`, not the
  dead engine.
- **#8 (call-graph completeness / convergence-gating):** "acyclic ⇒ no recursion" is sound **only if the call
  graph is complete** over the reachable set. R7 guarantees this WITHOUT PTA by **abstaining on any reachable
  unresolved indirect call** — so on the paths R7 does *not* abstain, the direct-call graph is complete, and
  `tarjan_scc`/`toposort` acyclicity is sound. Because R7 abstains on indirect calls it **never runs PTA/absint**,
  so the "fail-closed on PTA/absint fixpoint truncation" clause is satisfied vacuously (there is no fixpoint to
  truncate). Any reachable **non-allowlisted external** (which may loop / not return / invoke a callback) →
  abstain (mirrors the R3/plan-194 `is_known_returning_external` discipline).
- **Never `false`** (non-termination FALSE is a separate future confirmer, §12). **Byte-deterministic** (all
  `BTreeMap`/`BTreeSet`; the verdict is the constant string `"true"`; no witness, no timestamps). **Bounded**
  (one static pass over the already-ingested AIR — no compile-of-original, no execution, no watchdog needed).

---

## 4. The structural proof (D1–D6)

**D1 — The sufficient condition `program_structurally_terminates(module)` (T∧L∧A∧I∧E).** Emit `true` iff ALL:
- **(T)** a **defined** `main` exists (`name == "main" && !is_declaration`); else abstain(`no-main`).
- **(I) — no reachable unresolved indirect call.** Abstain if any function reachable-from-`main` contains an
  `Operation::CallIndirect`, OR any reachable call-graph node is a `CallGraphNode::IndirectPlaceholder`. (Primary
  form: scan reachable defined-function bodies for `CallIndirect`; backstop: any reachable `IndirectPlaceholder`
  node.) This is the redline-#8 gate — it makes the direct call graph complete over the reachable set and avoids
  PTA entirely.
- **(E) — allowlisted externals only.** Every reachable **external declaration** callee's name must satisfy
  `is_known_returning_external` (D2); else abstain(`non-allowlisted-external`).
- **(A) — acyclic reachable call graph.** Over the **reachable-restricted** subgraph, `tarjan_scc` shows no SCC of
  size > 1 and no self-loop (equivalently `toposort(...).is_some()`); else abstain(`recursion`). Externals are
  edge-less sinks (no bodies), so they cannot form a cycle.
- **(L) — loop-free reachable CFGs.** Every reachable **defined** function has a loop-free CFG
  (`reachable_is_loop_free` over the reachable defined set, using `cfg_has_loops` back-edge detection); else
  abstain(`loop`).

**Reachability is NODE-level (fidelity-critical):** compute the reachable set by DFS over the `CallGraph` **from
`main`'s node** (`graph_algo::dfs(main_node, &cg)`), collecting `CallGraphNode`s — not just `FunctionId`s — so
`External` and `IndirectPlaceholder` nodes on reachable paths are visible to (I)/(E). (The §13 probe validated
that fid-only reachability would miss them.) The reachability refinement is load-bearing: §13b confirmed real
WOULD_TRUE tasks whose *only* loop lives in a function **unreachable from `main`** (e.g. `memleaks_test9_2`).

**Soundness sketch:** (I)+(E) ⇒ the reachable call graph is complete and its leaves (externals) terminate-and-
return; (A) ⇒ finite call depth (no recursion); (L) ⇒ each function body has finitely many intra-procedural paths
(a DAG). A finite-depth tree of finite-path bodies with terminating leaves has only finite executions ⇒ the
program always halts ⇒ TRUE. A genuinely non-terminating C program must contain a loop, infinite recursion, or a
non-returning/looping external — each of which forces an abstain — so **R7 can never emit `true` on a
non-terminating task** (the −32 surface is structurally absent; §9 audits it empirically).

**D2 — `is_known_returning_external` allowlist + its soundness criterion.** An external name is allowlisted iff it
**provably (a) always returns or halts the program, (b) contains no unbounded loop, and (c) never invokes a
program callback** (which could re-enter user code and loop/recurse). Seed set:
`__VERIFIER_nondet_*` (all typed variants), `__VERIFIER_assume`, `__VERIFIER_assert`, `__assert_fail`, `abort`,
`exit`, `_exit`, `reach_error`/`__VERIFIER_error` (halt = terminates), and the callback-free, always-terminating
libc: `printf`/`fprintf`/`sprintf`/`snprintf`/`puts`/`putchar`, `malloc`/`calloc`/`realloc`/`free`,
`memcpy`/`memset`/`memmove`/`memcmp`, `strcpy`/`strncpy`/`strlen`/`strcmp`/`strncmp`, `sqrt`. **Explicitly NOT
allowlisted:** any callback-taking libc (`qsort`/`bsearch`/`atexit`/`scandir` — and they arrive via function
pointers ⇒ already caught by (I)), and any potentially-blocking/looping syscall (`read`/`recv`/`poll`/`scanf`
family — `read` can block indefinitely). §13b's non-allowlisted-external histogram justifies the libc extension
(reclaims 35 tasks: 13.25%→15.7%) and confirms the residual opaque blockers (`__startrek_*`, `unknown_0x…`,
`read`) must stay abstained. **The allowlist is the single most soundness-sensitive surface in R7** — each
addition needs the (a)+(b)+(c) argument, unit-tested.

**D3 — No PTA, no compile-of-original, no execution.** R7 is a pure static pass over the module already produced
by the shared compile+ingest (`clang-18 -O0 -disable-O0-optnone` + `opt -passes=mem2reg`, the same IR every other
strategy sees). Unlike `memsafety_strategy`/`overflow_strategy` it does **not** re-compile or run the program (it
asserts TRUE, not a confirmed FALSE) — so it is the simplest strategy: no driver, no sanitizer, no mini-fuzz, no
watchdog. It ignores the `conservative` config flag (the proof is always the sound path).

**D4 — Verdict string = bare `"true"`, witness `None`.** `termination_verdict()` → the constant `"true"` (NOT
`true(termination)` — BenchExec's `RESULT_TRUE_PROP` is bare `"true"`, and `saf.py` already matches it). The
existing witness-write gate (`verdict.starts_with("false")`) writes nothing for a `true`, so **no witness path,
no `witness_yaml`/`validate_witness` change**. `Property::Termination.name()` is already `"termination"`.

**D5 — Neutralize the dead `analyze_termination` (redline #6).** Reduce it to always-`Unknown`; delete orphaned
helpers. This removes SAF's last latent unsound-TRUE branch. (The `analyze_property` engine family remains dead;
R7 does not resurrect it.)

**D6 — Determinism.** All reachability/SCC/CFG structures already use `BTreeMap`/`BTreeSet` (NFR-DET). The verdict
carries no file/line/uuid/timestamp — it is the constant `"true"` — so byte-stability is trivial and must be
asserted (re-run ⇒ identical stdout).

---

## 5. Code seam (mirror R5/R6; the seam is already TRUE-ready; re-confirm line numbers — they drift)

- `crates/saf-svcomp/src/property_kind.rs` — `Termination` variant (`:35`), `from_prp` ("F end", `:124`),
  `name()` → `"termination"` (`:79`) **already exist and are correct**. **No edit.**
- `crates/saf-svcomp/src/termination.rs` (**NEW**, pure, mirrors `overflow.rs`/`memsafety.rs`):
  `is_known_returning_external(name: &str) -> bool` (D2 allowlist); `program_structurally_terminates(module:
  &AirModule) -> bool` (the D1 T∧L∧A∧I∧E composition, node-level reachability); `termination_verdict() -> &'static
  str` (`"true"`). Re-export from `lib.rs`. Reuses `saf_analysis::{callgraph::CallGraph, cfg::Cfg,
  graph_algo::{dfs, tarjan_scc, Successors}}` and `saf_svcomp::fast_paths::{reachable_is_loop_free, cfg_has_loops}`.
- `crates/saf-svcomp/src/fast_paths.rs` — add one reusable structural helper
  `reachable_callgraph_is_acyclic(cg, &reachable_nodes) -> bool` next to `reachable_spawns_threads` /
  `reachable_is_loop_free` (a reachable-restricted `tarjan_scc`/self-loop check). (Or inline in `termination.rs`
  — confirm at TDD; keep the pure logic where it reads best.)
- `crates/saf-cli/src/commands.rs`:
  - `strategy_for` (`:987`) — add arm `Termination => Some(termination_strategy)` (4th arm on the plan-194 spine).
  - `termination_strategy(ctx: &VerifyCtx) -> VerdictOutcome` — the simplest strategy: `if
    saf_svcomp::program_structurally_terminates(ctx.module) { VerdictOutcome { verdict:
    saf_svcomp::termination_verdict().to_string(), witness: None } } else { unknown_outcome() }`. No compile-of-
    original, no driver, no watchdog, no witness. (Confirm `VerifyCtx` exposes the ingested `module`; the shared
    compile+ingest at `~:972` builds it before dispatch — `overflow_strategy` reads it at `:1557`.)
  - Update the `verify` docstring (`:744`, "never `true`") to describe the proven-`true` path.
- `crates/saf-svcomp/src/property.rs` — neutralize `analyze_termination` (`:1441`) to always-`Unknown`; remove
  orphaned helpers (D5).
- **`benchexec/tools/saf.py` — NO change** (bare `"true"` → `RESULT_TRUE_PROP` already, `test_saf.py` covers it).
- **`Dockerfile` — NO change.** **No witness machinery change.**

---

## 6. File structure (new + touched)

```
crates/saf-svcomp/src/
  termination.rs       (NEW) is_known_returning_external; program_structurally_terminates (T∧L∧A∧I∧E); termination_verdict
  fast_paths.rs        (edit) + reachable_callgraph_is_acyclic (reachable-restricted tarjan_scc/self-loop)
  property.rs          (edit) neutralize dead analyze_termination → Unknown; drop orphaned helpers (redline #6)
  lib.rs               (edit) pub mod termination; re-exports
crates/saf-cli/src/
  commands.rs          (edit) strategy_for arm Termination; termination_strategy(); verify docstring
tests/programs/c/svcomp/ (NEW fixtures) termination_true_straightline.c; _true_allowlisted_extern.c;
                        _true_loop_in_unreachable.c; _unknown_loop.c; _unknown_selfrec.c; _unknown_mutualrec.c;
                        _unknown_indirect.c; _unknown_bad_extern.c; termination.prp
crates/saf-cli/tests/   (edit) verify_termination.rs (#[ignore] Docker e2e) — mirror verify_overflow.rs
scripts/                (Slice-0 de-risk, measurement-only, already on VM, UNCOMMITTED) r7_prevalence_targets.py,
                        r7_prevalence_compile.py, r7_prevalence_run.py (+ throwaway r7_classify_one example /
                        r7_prevalence test — removed by the next crates/ rsync)
```

---

## 7. TDD slices

### Slice 0 — de-risk spike (NO production code) — **DONE this session → GO** (§13)

### Slice 1 (GO) — the pure structural check (`termination.rs`, RED→GREEN units)
- [ ] **1a `is_known_returning_external`** — allowlisted names (incl. `__VERIFIER_nondet_*` prefix, libc pure set)
  → true; `read`/`qsort`/`__startrek_*`/opaque → false. Unit table.
- [ ] **1b `program_structurally_terminates`** on hand-built `AirModule`s (no LLVM needed): straight-line `main`
  ⇒ true; `main` with a back-edge loop ⇒ false(loop); self-recursion ⇒ false(recursion); mutual recursion A↔B ⇒
  false(recursion); a reachable `CallIndirect` ⇒ false(indirect); a reachable non-allowlisted external ⇒
  false(non-allowlisted-external); a reachable **allowlisted** external (printf/nondet) ⇒ true; a loop in a
  function **unreachable** from `main` ⇒ true (reachability-scoped); missing `main` ⇒ false(no-main). Byte-stable.
- [ ] **1c `reachable_callgraph_is_acyclic`** (if factored into `fast_paths`) — direct unit tests for SCC>1,
  self-loop, and DAG.

### Slice 2 (GO) — wire the strategy + neutralize the dead branch + e2e
- [ ] **2a `commands.rs`** — `strategy_for` arm `Termination => termination_strategy`; `termination_strategy`
  (pure static check on `ctx.module`, no confirmer). Docstring update.
- [ ] **2b `property.rs`** — neutralize `analyze_termination` → `Unknown`; drop orphaned helpers; `make lint`
  clean (`-D warnings`).
- [ ] **2c e2e (`verify_termination.rs`, `#[ignore]` Docker)** — fixtures → verdict at **LP64 AND ILP32**:
  loop-free straight-line ⇒ `true`; allowlisted-extern loop-free ⇒ `true`; loop-in-unreachable-fn ⇒ `true`;
  looping ⇒ `unknown`; self/mutual recursion ⇒ `unknown`; indirect-call ⇒ `unknown`; safe non-allowlisted-extern
  ⇒ `unknown`; **byte-identical stdout across re-runs; no witness file written for `true`.**

### Slice 3 (GO) — blind `saf verify` reservoir eval (the −32 audit + recall)
- [ ] **3a** `scripts/r7_verify_termination_eval.py` (clone of `r6_verify_overflow_eval.py`): run blind `saf
  verify --property termination.prp` over the termination reservoir (the **996 expected-false** in full + a large
  stratified slice of the **1434 expected-true**), at ILP32+LP64. **Assert 0 wrong TRUE** (the −32 audit — R7
  must emit `unknown`, never `true`, on every non-terminating task) **and 0 FALSE** (R7 never emits false).
  Report recall = TRUE/1434 (expect ≈ the §13b 13.25–15.7%), and byte-determinism.
- [ ] **3b** Re-run `make fmt && make lint` (`--workspace -D warnings`) + `make test` (nextest + pytest); the
  plan-192/194 unreach and R5/R6 memsafety/overflow e2e suites **unregressed**.

### Slice 4 (GATED, later — measure-before-build)
- Libc-allowlist tuning only if 3a shows a material, provably-sound gain beyond the seed set (§13b: the audited
  extension reclaims 35; each new name needs the D2 (a)+(b)+(c) argument). No PTA-based indirect resolution
  (deferred; abstaining is the sound default). No termination FALSE, no 2.1 correctness witness (§12).

---

## 8. Soundness & determinism invariants (mapped to slices)

| Invariant | Where enforced |
|---|---|
| Never `true` except on the §4 proof | `termination_strategy` returns `unknown_outcome()` unless `program_structurally_terminates` (2a) |
| Call graph complete over reachable set (⇒ acyclic ⇒ no recursion is sound) | (I) abstain on any reachable `CallIndirect`/`IndirectPlaceholder` (1b) |
| No PTA/absint truncation risk | R7 runs no PTA/absint — abstaining on indirect calls removes the need (D3) |
| Reachable externals terminate-and-return, no callbacks | (E) `is_known_returning_external`, soundness criterion D2 (1a) |
| No recursion | (A) reachable `tarjan_scc` no SCC>1 / no self-loop (1b/1c) |
| Finite intra-procedural paths | (L) `reachable_is_loop_free` back-edge detection (1b) |
| Reachability scoped to `main` (unreachable loops ignored soundly) | node-level `dfs(main_node, &cg)` (1b) |
| Never `false` | `termination_strategy` has no `false` path (2a) |
| Byte-deterministic | constant `"true"`, no witness/uuid/timestamp; `BTree*` structures (1b/2c) |
| Bounded | one static pass over the ingested AIR; no compile-of-original/exec/watchdog (D3) |
| Last latent unsound-TRUE removed | neutralize dead `analyze_termination` (2b, redline #6) |

---

## 9. Acceptance criteria & evidence

- **0 wrong verdicts at scale — the −32 audit is THE bar.** Blind `saf verify` over the termination reservoir
  (996 expected-false in full + a large expected-true slice, ILP32+LP64): **0 TRUE on any non-terminating task,
  0 FALSE emitted**, byte-deterministic (3a). (A wrong `true` is −32 ≈ 16× a right one — this dominates.)
- **Blind `true` recall > 0**, ≈ the §13b measured **13.25%** (190/1434), sub-property N/A, no witness written.
- **Extensibility proven:** `termination` added as a `strategy_for` arm + one pure module, **the FIRST
  TRUE-capable arm**, with **no seam rewrite** — a 4th property on the plan-194 spine, and the sound-TRUE spine
  R8 reuses.
- **Redline hygiene:** the dead `analyze_termination` unsound branch(es) are **deleted** (redline #6), verified
  by grep + `make lint`.
- Gates: full `make test` (Rust nextest + pytest) + clippy `--workspace -D warnings` + fmt; plan-192/194 unreach
  and R5/R6 e2e suites unregressed.

---

## 10. VM workflow

```
rsync -az --delete .../static-analyzer-factory/crates/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/crates/
rsync -az .../scripts/ .../tests/programs/ ubuntu@cd-vm-15-ai-vm:~/static-analyzer-factory/   # no --delete
# no image rebuild needed — R7 adds no toolchain/runtime deps
ssh ubuntu@cd-vm-15-ai-vm 'cd ~/static-analyzer-factory && make fmt && make test 2>&1 | tee /tmp/t.txt'
docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo nextest run -p saf-cli --run-ignored all -E "test(verify_termination)"'
```
(The `crates/ --delete` rsync also removes the Slice-0 throwaway `crates/saf-svcomp/{tests/r7_prevalence.rs,
examples/r7_classify_one.rs}` + the throwaway `Cargo.toml` dev-dep from the VM, restoring the laptop's clean
tree.)

---

## 11. Risks & open questions

- **Allowlist soundness (the one real risk).** Every `is_known_returning_external` entry is load-bearing: a
  wrongly-allowlisted external that loops/blocks/recurses/calls-back ⇒ a wrong TRUE ⇒ −32. Mitigation: the D2
  (a)+(b)+(c) criterion, a conservative seed set, per-name unit tests, and the 3a −32 audit at scale. Blocking
  syscalls (`read`) and callback-libc (`qsort`) are explicitly excluded.
- **Reachability fidelity.** Node-level `dfs` from `main`'s node is required (fid-only misses External/Indirect
  nodes). Covered by 1b (loop-in-unreachable ⇒ true) and the §13b LLVM cross-check.
- **Compile-recipe abstains (not soundness).** §13b saw 101/1434 ILP32 compile-fails (a bundled-stub
  `__VERIFIER_assert`-macro collision) — all in loop/recursion families that abstain anyway; R7 sees the same
  compile and soundly abstains. A recall lever, not a −32 risk.
- **Payoff lands in Termination-category/C.Overall, not C.FalseOverall** (§2 caveat) — accepted by the user.
- **Non-termination FALSE and hard-benchmark TRUE are out of reach** (ranking functions / lasso witnesses) — §12.

---

## 12. Out of scope (deferred)

- Any `termination` **FALSE** (non-termination): needs a concrete lasso/recurrent-set counterexample + a **YAML
  2.1** violation witness with `action: cycle` — a separate future confirmer.
- **2.1 correctness witness** (`invariant_set` + `loop_transition_invariant`): witness-not-required in 2026
  (§2); build ONLY if a future cycle flips termination to witness-required. CPAchecker's
  `correctness-witness-validation--termination.properties` (on the VM) makes a future feasibility spot-check
  cheap.
- **Ranking-function / lexicographic / lasso termination proving** for looping programs (the ~65% loop reservoir
  and the dedicated `termination-*` families, ~0% caught by R7) — a greenfield prover SAF does not have.
- PTA-based indirect-call resolution (abstaining is the sound default); the Slice-4 allowlist tuning.
- After R7: resume the roadmap at **R8 `no-data-race` no-threading TRUE** (`plans/193` §7) — which reuses R7's
  sound-TRUE verdict spine (a `strategy_for` arm returning a proven bare `true`, witness-not-required).

---

## 13. De-risk record (Slice 0, this session, 2026-08-13, VM `ubuntu@cd-vm-15-ai-vm`) — **GO**

### 13a — Scoring (de-risk unknown #1) — RESOLVED: termination TRUE is **witness-not-required**
Primary-source + adversarially-verified (SV-COMP 2026 & 2025 `rules.php`, 2025/2026 reports; a 7-agent workflow
with 3/3 adversarial verifiers CONFIRMED; an independent WebFetch of 2026 `rules.php` agreed). Termination
correctness witnesses are *"2.1\* (demo mode)"* ⇒ *not required*; a correct `termination=true` scores +2 on the
verdict alone (same bucket as memsafety/no-data-race/memcleanup; only unreach-call/no-overflow require a
correctness witness). **This inverts `plans/200` §3.1** — there is **no correctness-witness pipeline to build**.
The termination correctness-witness *format* is YAML 2.1 (`invariant_set` + `loop_transition_invariant`; empty
shell for a loop-free program), not 2.0. Recorded in [[svcomp-termination-witness-not-required]].

### 13b — Reservoir prevalence (de-risk unknown #2) — **190/1434 = 13.25%** (→ ~15.7% with a libc allowlist)
A **throwaway** `#[ignore]` Rust probe (uncommitted) ingested each task via SAF's **real** LLVM frontend and
applied R7's exact T∧L∧A∧I∧E using SAF's own `CallGraph::build`/`dfs`/`tarjan_scc`/`Cfg::build`+`cfg_has_loops`/
`reachable_is_loop_free`, over **all 1434** termination-TRUE tasks (full set, ~31s; not a sample). Faithful
compile recipe (`clang-18 -O0 -disable-O0-optnone` + `opt-18 mem2reg`, per-file isolation so a frontend SIGSEGV
becomes a conservative abstain). **WOULD_TRUE = 190 (13.25%)**, a sound lower bound (every uncertainty abstains).
Abstain histogram: **loop 928 (64.7%), recursion 136 (9.5%), compile-fail 101 (7.0%, faithful stub-macro quirk,
all in loop/recursion families), non-allowlisted-external 56 (3.9%), indirect-call 13 (0.9%), missing-src 9,
ingest-crash 1.** Per-family: **~0%** across every dedicated `termination-*`/`loops`/`recursive*`/`product-lines`
family (correctly — they need ranking functions); the wins concentrate in loop-free programs cross-listed under
termination from the memsafety/overflow/float/ldv pools (`ldv-regression` 71/79, `ldv-memsafety` 40/55,
`floats-cdfpl` 32/40, `floats-cbmc-regression` 16/16, `signedintegeroverflow-regression` 5/5, `memsafety` 5/5).
**Adversarial cross-check:** 3-case validation correct; 6 WOULD_TRUE examples verified loop-free + non-recursive
against LLVM ground truth (`opt -passes=print<loops>` + an independent `.ll` call-graph cycle check); the
reachability refinement genuinely works (`memleaks_test9_2` ⇒ WOULD_TRUE because its only loop is in a function
unreachable from `main`). Non-allowlisted-external histogram (Slice-4 lever): `memcmp`(25), `__startrek_*`(48
opaque), `strncmp`/`strncpy`/`strcmp`, singletons `sqrt`/`sprintf`/`snprintf`/`read`, 8 `unknown_0x…` — a pure-
libc extension reclaims 35 → 225 (15.7%); `__startrek_*`/`unknown_0x…`/`read` must stay abstained.

### 13c — Recon (deliverable #3)
- `analyze_termination` (`property.rs:1441`) is **dead** (zero live callers — the whole `analyze_property` family
  is dead on both `verify` and bench surfaces); the unsound `!conservative ⇒ True` branch (`:1462`) **and** the
  loop-free-only ⇒ True branch (`:1455`, no acyclic check) both violate redline #6 → neutralize (D5).
- The verdict seam is **already TRUE-ready**: `strategy_for` (`:987`) needs one arm; `VerdictOutcome`
  (`:959`) carries `verdict: String` + `Option<ViolationWitness>`; the witness-write gate is
  `if outcome.verdict.starts_with("false")` (`:827`) ⇒ a `true` writes no witness; `unknown_outcome()` (`:965`).
- BenchExec `saf.py determine_result` already maps a bare `"true"` → `RESULT_TRUE_PROP` (no change).
- **All structural machinery exists** (no re-implementation): `fast_paths::{reachable_functions:517,
  reachable_is_loop_free:652, cfg_has_loops:83}`; `graph_algo::{dfs:25, tarjan_scc:137, toposort:265, Successors}`
  with **`CallGraph` implementing `Successors`** (so callgraph acyclicity is `toposort(&cg.nodes,&cg).is_some()`);
  `callgraph::CallGraph::build` (nodes `Function`/`External{name}`/`IndirectPlaceholder`); `cfg::Cfg::build`;
  `AirFunction::is_declaration`; `Operation::CallIndirect`. No `petgraph`; all `BTree*` (deterministic).
- Witness/validate path is violation-only (2.0) — untouched by R7 (no correctness-witness needed, 13a).

**GO decision:** scoring needs no witness pipeline (13a), the proof is sound and reuses existing machinery with a
one-arm seam addition (13c), and it soundly recalls 13.25–15.7% of terminating tasks with 0 wrong-TRUE by
construction (13b, audited at scale in Slice 3). Proceed to Slice 1.

---

## 14. Implementation record (Slices 1–3, TDD, 2026-08-13, VM `ubuntu@cd-vm-15-ai-vm`, `saf-dev:llvm18`) — DONE, UNCOMMITTED

**Slice 1 — pure structural check (`crates/saf-svcomp/src/termination.rs`, RED→GREEN).** New pure module:
`is_known_returning_external` (D2 allowlist: `__VERIFIER_nondet_*` prefix + verifier/halt + callback-free pure
libc; `read`/`qsort`/`__startrek_*` rejected), `program_structurally_terminates` (the T∧L∧A∧I∧E proof, node-level
`dfs` reachability from `main`, reusing `CallGraph::build`/`graph_algo::tarjan_scc`/`cfg_has_loops`), a private
`reachable_callgraph_is_acyclic` (tarjan SCC>1 + explicit self-loop), and `termination_verdict()`→`"true"`.
**16 unit tests** on hand-built `AirModule`s (straight-line/allowlisted-extern/loop-in-unreachable ⇒ true;
loop/self-rec/mutual-rec/reachable-indirect/non-allowlisted-extern/no-main/decl-main ⇒ abstain) — verified RED
(13 logic tests panic on `unimplemented!()`), then GREEN (16/16). Re-exported from `lib.rs`.

**Slice 2 — wire the strategy + neutralize the dead branch + e2e.** `commands.rs`: `strategy_for` arm
`Termination => termination_strategy`; `termination_strategy` (a pure static check on `ctx.module` — no
compile-of-original/confirmer/driver/watchdog/witness, the simplest strategy; verdict = bare `true`, witness
`None`); the `verify` docstring updated (no longer "never `true`"). `property.rs`: `analyze_termination`
neutralized to always-`Unknown` (both unsound TRUE branches DELETED, redline #6; `program_is_loop_free` +
`find_nonterminating_loops` imports dropped). **NO** witness/Docker/benchexec change (bare `true` already maps
via `saf.py`). **10 e2e** (`smoke.rs`, `#[ignore]` Docker) + 7 fixtures + `termination.prp`: straight-line ⇒
`true` at **LP64 AND ILP32**; allowlisted-externs ⇒ `true`; loop-in-unreachable ⇒ `true` (reachability
refinement works end-to-end); loop/recursion/indirect/bad-external ⇒ `unknown`; `true` writes **no witness**;
byte-stable. **10/10 GREEN.** Gates: **2288 nextest + 94 pytest**, clippy `--workspace -D warnings` + fmt clean;
unreach/R5/R6 e2e unregressed.

**Review hardening (pre-eval, −32 defense-in-depth).** An adversarial self-review of the allowlist +
`program_structurally_terminates` held (reachability-scoped; abstain-on-indirect keeps the call graph complete;
self-loop caught separately from SCC>1; `__VERIFIER_assume` sound because SV-COMP termination counts only
feasible executions). One hardening: the dead `None` reachability arm (main defined but absent from the call
graph — unreachable in practice) now **abstains** (`return false`) instead of an optimistic `!cfg_has_loops`
that skipped the (I)/(E) checks.

**Slice 3 — blind `saf verify` reservoir eval (`scripts/r7_verify_termination_eval.py`, the −32 audit).** Ran
the release binary over the termination reservoir (2395 tasks: 1425 terminating / 970 non-terminating), at each
task's declared data model. **−32 audit: WRONG_TRUE=0 over 964 non-terminating tasks, FALSE_EMITTED=0**, recall
**49/299 = 16.4%** (≥ the 13.25% structural prevalence), **determinism 15/15**. **A first eval run reported a
false WRONG_TRUE=61 — all ILP32 — which a diagnostic showed to be a MEASUREMENT ARTIFACT** (the plan-198 shape):
the eval's `.yml` parser scanned a 5-line window and kept the *last* `expected_verdict`, grabbing a neighboring
property's `false` on multi-property tasks, so correct TPs on genuinely-terminating `true-termination` tasks
(e.g. `newton_1_4`, `memleaks_test1-1`, `test02` — all `termination expected_verdict: true`, verified) were
mislabeled. Fixed the parser (take termination's own first `expected_verdict`); the corrected re-run is the
result above. **R7's verdict logic was correct throughout; the bug was in the measurement harness.**

**Acceptance MET (§9):** 0 wrong verdicts at scale (the −32 audit), recall > 0 (~16%), byte-deterministic,
extensibility proven (a 4th `strategy_for` arm — the FIRST TRUE-capable — with no seam rewrite; the sound-TRUE
spine R8 reuses), redline hygiene (the dead unsound `analyze_termination` branches deleted), all gates green.
Throwaway Slice-0 prevalence artifacts on the VM were cleaned by the `crates/` rsync.

---

## 15. Pre-commit adversarial soundness review — 3 −32 bugs found & fixed (2026-08-13)

A high-effort adversarial review (an 8-angle workflow + a completeness critic, each hunting a non-terminating
program R7 would call `true`, code-grounded in the real frontend/AIR/CFG/callgraph) **found 3 REAL −32 bugs that
the scaled Slice-3 audit had missed** (the reservoir happens not to contain these patterns). This is the payoff
of reviewing beyond the empirical audit for a first sound-TRUE. All 3 were reproduced on the real `saf verify`
pipeline (`true` on a non-terminating program), then fixed TDD and re-verified (`unknown`):

- **B1 — global constructors** (`__attribute__((constructor))` → `@llvm.global_ctors`): run BEFORE `main`, so a
  looping constructor is outside main's call graph and invisible to R7. **B2 — global destructors**
  (`@llvm.global_dtors`): run AFTER `main`. **Fix:** abstain if `module.globals` contains `llvm.global_ctors` /
  `llvm.global_dtors` (the frontend preserves those globals). Sound + conservative (ctors/dtors are rare in the
  reservoir).
- **B3 — `indirectbr` (computed goto / labels-as-values)** and any unsupported terminator (`callbr`, …): the LLVM
  frontend's catch-all (`mapping.rs:1222`) **silently drops** them (`warn! + Ok(None)`), leaving the block with
  **no terminator** → `extract_successors` returns empty → the block looks like an exit → the back-edge is
  invisible → `cfg_has_loops` says loop-free. **Fix (general):** abstain if any reachable defined block has no
  recognized terminator (`block.terminator().is_none()`) — the CFG is incompletely modeled. Catches indirectbr,
  callbr, and any future dropped terminator.
- **Bundled contract fix:** the frontend's `WARN: Unsupported LLVM instruction` was going to **stdout** (`main.rs`
  wired `fmt::layer()` with the default stdout writer), polluting `saf verify`'s verdict-only-stdout contract for
  ANY task with an unsupported instruction (all properties). **Fix:** route tracing to **stderr**
  (`.with_writer(std::io::stderr)`).

**Verified sound by the review (no fix needed):** function **aliases** (`alias_recursion` → `unknown`),
**`setjmp`/`longjmp`** (externals, not allowlisted → abstain), determinism (`BTreeMap`/`BTreeSet` throughout),
the verdict string (bare `true`), and the `__VERIFIER_nondet_` prefix (a *defined* looping fn is not a
declaration, so (E) doesn't allowlist it and (L) catches its loop).

**Tests added:** 3 unit tests (`unterminated_block_abstains`, `global_constructors_abstain`,
`global_destructors_abstain`) — verified RED→GREEN; the test builders now emit terminated blocks (real IR always
does). 3 e2e regression guards (`termination_unknown_{ctor_loop,dtor_loop,computed_goto}.c` → `unknown`). Probes
kept under `scripts/r7probes/` (uncommitted). Final gates: **2291 nextest + 94 pytest + 13/13 termination e2e**,
clippy `--workspace -D warnings` + fmt clean; the −32 audit re-run (hardened binary) holds WRONG_TRUE=0.

**Commit when the user asks; then R8 `no-data-race` no-threading TRUE** (reuses this sound-TRUE verdict spine).
