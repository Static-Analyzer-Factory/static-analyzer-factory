# Plan 200: `termination` (loop-free ∧ acyclic-callgraph) TRUE — the first sound-TRUE slice (R7)

**Status: SCOPING / roadmap placement — NOT yet brainstormed. Do the brainstorm → de-risk → design →
approval → TDD in a NEW session** (this file documents the decision, the redlines, and the two big unknowns so
that session starts grounded). Branch `svcomp`. Laptop = git source of truth; all builds/experiments on the
VM (`ubuntu@cd-vm-15-ai-vm`); **commit only when the user asks.** Follows plan 199 (R6 `no-overflow`, DONE).
Implements roadmap **R7** (`plans/193` §4.4/§7).

## 1. The decision and why

`termination` (`CHECK( init(main()), LTL(F end) )`) is `unknown` on 100% today (`strategy_for` has no
`Termination` arm). R7 is the roadmap's **first sound-TRUE slice** — a strategic inflection point: R1–R6 are
all FALSE-only via runtime confirmers (replay / ASan / UBSan), whereas R7 emits **`true`** from a **static
structural proof**. That makes it qualitatively different and higher-risk: it is the first time SAF crosses
the CLAUDE.md redline-#1 "never prints `true`" gate, so the proof must be **provably sound**, not a
finding-absence heuristic.

**The sufficient condition (decidable, sound, incomplete):** a program whose every reachable function has a
**loop-free CFG** (no back-edge — the CFG is a DAG) **and** whose reachable **call graph is acyclic** (no
recursion) has only **finite executions**, so it **always terminates** ⇒ `termination = TRUE`. This is a
*sufficient* condition, not complete — most terminating programs have loops (which need ranking-function
reasoning, out of scope) — so recall is bounded to the loop-free ∧ non-recursive subset. It abstains
(`unknown`) otherwise; it **never** emits `false` (termination-FALSE = a concrete non-terminating execution,
a separate future confirmer).

## 2. Soundness redlines (CLAUDE.md — non-negotiable; R7 is where they first bind for TRUE)

- **#1 first `true`:** relax the hard "never prints `true`" gate ONLY for this proven path; every other
  TRUE-capable branch stays gated/deleted.
- **#6:** `termination` TRUE requires loop-free CFGs **AND** an acyclic call graph; **DELETE (do not merely
  gate) the aggressive `!conservative ⇒ TRUE` branch** in `analyze_termination` (`property.rs`).
- **#8 (the subtle one):** any sound TRUE must gate on **absint fixpoint convergence AND base-Andersen PTA
  convergence, fail-closed to `unknown` on truncation.** For R7 this bites on **call-graph completeness:**
  "acyclic call graph ⇒ no recursion" is sound only if the call graph is **complete** (over-approximate). Any
  **unresolved indirect call** (`CallIndirect`) can hide a recursive cycle, so R7 must either (a) require **no
  reachable `CallIndirect`**, or (b) treat a reachable `CallIndirect` as possibly-reaching any address-taken
  function and require *that* over-approximate graph to still be acyclic — else abstain. Likewise an **unknown
  external** call may not return (or may loop/recurse), so only an **allowlist of known-terminating externals**
  (`__VERIFIER_nondet_*`, `printf`-family, …) is permitted; any non-allowlisted external ⇒ abstain (mirrors the
  plan-194/R3 `is_known_returning_external` discipline). PTA/reachability truncation ⇒ abstain.

## 3. The TWO big unknowns to de-risk BEFORE any code (Slice 0, no production code)

1. **The correctness-witness pipeline (the real risk).** SV-COMP scores a TRUE verdict only with a **validated
   CORRECTNESS witness** — and *all* of SAF's committed witness machinery (`witness_yaml.rs`,
   `witness_lower.rs`, `validate_witness.sh`) emits/validates **VIOLATION** witnesses. R7 opens a **new
   correctness-witness workstream**: the YAML-2.0 *correctness*-witness schema (invariants; for termination, a
   ranking/termination argument — trivial for a loop-free program, but the format still has required shape),
   which validators confirm termination correctness witnesses (CPAchecker? UAutomizer? the termination
   validators), and whether a trivial loop-free witness is CONFIRMED. **De-risk: hand-craft a loop-free
   termination correctness witness and see if a validator confirms it** — this may be the hardest part of R7
   and could dwarf the verdict logic. (Mirror the plan-199 witness de-risk: `validate_witness.sh` is
   violation-only today; a correctness path likely needs a new config/flag.)
2. **Reservoir recall (the premise check).** The `termination` reservoir (`termination-crafted*`,
   `termination-restricted*`, `termination-nla`, `nla-digbench`, `loop-*`, …) is **loop-DOMINATED** by
   construction — loop-free ∧ non-recursive termination tasks may be a **small** subset. **De-risk: measure how
   many `termination=true` tasks are loop-free ∧ acyclic ∧ allowlisted-externals-only** (a pure structural scan
   over the AIR, no verdict needed). If it is ~nil, R7's TRUE recall is ~0 and the payoff is the *pipeline*
   (the first sound-TRUE + the correctness-witness spine that R8/R10 reuse), not the score — decide scope
   accordingly (the R4 measure-before-build discipline; R4 was STOPPED on exactly this shape).

## 4. Code seam (recon starting points — verify before editing)

- `crates/saf-cli/src/commands.rs` — `strategy_for` (add `Termination => termination_strategy`); the seam
  (`VerdictOutcome`/`run_verdict`/`build_witness`) is TRUE-capable already (it carries a verdict string +
  optional witness) but has only ever emitted `false`/`unknown` — the first `true` flows here.
- `crates/saf-svcomp/src/property.rs` — `analyze_termination` (the DEAD over-approx engine + the unsound
  `!conservative ⇒ TRUE` branch to **delete**).
- `crates/saf-svcomp/src/fast_paths.rs` — `reachable_functions` (DFS from `main`), the callgraph plumbing
  (`CallGraph::build`), and `Operation::CallDirect`/`CallIndirect` — the pieces a `reachable_is_loop_free`
  + `reachable_callgraph_is_acyclic` structural check would reuse (mirror `reachable_spawns_threads`).
- CFG back-edge / loop detection: check `saf-analysis` for existing CFG + dominator/back-edge machinery (a
  natural-loop or back-edge finder) rather than re-implementing.
- `property_kind.rs` — `Termination` variant + `from_prp` ("F end") already exist; `name()`→`"termination"`.
- `crates/saf-svcomp/src/witness_yaml.rs` / `validate_witness.sh` — the VIOLATION-only witness path; R7 needs
  a **correctness**-witness addition (the §3.1 workstream).

## 5. First steps for the new session

1. `superpowers:brainstorming` → clarify scope + the §3 unknowns (esp. correctness-witness feasibility +
   loop-free recall) with the user.
2. Recon: `analyze_termination` (the branch to delete), `fast_paths` (reachability/callgraph), the CFG
   back-edge machinery in `saf-analysis`, the violation-witness path (what a correctness path must add).
3. Run the §3 de-risk (correctness-witness confirmation + loop-free∧acyclic reservoir prevalence) — GO/NO-GO
   before any production code (the R4/R5/R6 discipline). A NO-GO here (no validator confirms a loop-free
   correctness witness, OR ~nil loop-free reservoir) reshapes or defers R7.
4. Design → user approval → TDD on the VM. Then the roadmap continues at **R8 `no-data-race` no-threading
   TRUE** (`plans/193` §7) — which reuses the sound-TRUE + correctness-witness spine R7 builds.

Context: `plans/193` §4.4/§7 (R7), `plans/199` (R6 — the confirmer-first template + the plan-198 FP-surface /
premise-check discipline), CLAUDE.md redlines #1/#6/#8, [[saf-svcomp-199-r6-overflow-derisk-go]],
[[saf-svcomp-capability-findings]], [[saf-svcomp-workflow]], [[saf-svcomp-vm-env]], [[svcomp-yaml-witness-2.0-format]].
