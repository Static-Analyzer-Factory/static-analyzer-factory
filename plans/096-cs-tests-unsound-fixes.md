# Plan 096: cs_tests Unsound Fixes

## Context

5 unsound PTABen cs_tests cases from 2 files:
- **cs18.ll** (4 unsound): MustAlias→Partial (x2), NoAlias→Partial (x2)
- **cs9.ll** (1 unsound): NoAlias→Partial

Both root causes are in a single file:
`crates/saf-analysis/src/pta/value_origin.rs` (~1427 lines)

## Root Cause Summary

1. **cs18** — `process_instruction_flow_sensitive()` skips nested `CallDirect` inside
   inlined callees (line ~1043). `foo→bar` chain: `bar`'s store never seen.
2. **cs9** — `solve_per_path_flow_sensitive()` uses an iterative fixpoint loop with
   strong updates. Strong updates are non-monotonic (replace, not union), causing
   infinite oscillation (confirmed: 100K iterations, never converges). Loads accumulate
   spurious values from intermediate states.

## Agent Task Assignments

Both fixes modify the **same file** in **different functions**, so they run
**sequentially**: Agent 1 first, then Agent 2.

---

### Task 1 — Agent 1: Recursive CallDirect inlining (fixes cs18, −4 unsound)

**What to change**: function `process_instruction_flow_sensitive` in
`crates/saf-analysis/src/pta/value_origin.rs` (starts at line ~759).

**Problem**: When processing a `CallDirect` inside the caller, the handler inlines
the callee's instructions — but SKIPS any `CallDirect` or `CallIndirect` found
inside the callee. This means nested calls (foo→bar) are never processed.

There are TWO places where this skip happens:

1. **CallDirect handler** (line ~1043):
   ```rust
   if !matches!(callee_inst.op, Operation::CallDirect { .. }) {
   ```
   Only processes non-CallDirect instructions from the callee.

2. **CallIndirect handler** (line ~1153):
   ```rust
   if !matches!(callee_inst.op, Operation::CallDirect { .. } | Operation::CallIndirect) {
   ```
   Same skip for both call types inside indirectly-called callees.

**Fix**:

1. Add an `inline_depth: usize` parameter to `process_instruction_flow_sensitive`.
   Add a constant `const MAX_INLINE_DEPTH: usize = 5;` at the top of the function.

2. In BOTH skip locations (CallDirect handler ~1043 and CallIndirect handler ~1153),
   **remove the skip guard** and instead process ALL callee instructions including
   calls, by passing the renamed instruction through to `process_instruction_flow_sensitive`
   recursively. Guard with `if inline_depth < MAX_INLINE_DEPTH` to prevent infinite
   recursion on recursive functions. When depth is exceeded, fall back to the current
   behavior (skip the nested call).

3. Pass `inline_depth + 1` to the recursive `process_instruction_flow_sensitive` call
   for callee instructions. Pass `inline_depth` (unchanged) when called from the
   top-level `solve_per_path_flow_sensitive` loop (initial depth = 0).

4. Update ALL call sites of `process_instruction_flow_sensitive` (there is one in
   `solve_per_path_flow_sensitive` at line ~753) to pass `inline_depth: 0`.

**Verification**: After this change, `cargo check -p saf-analysis` should compile.
Do NOT run tests (leader handles Docker commands).

**Important Rust conventions**:
- Use `thiserror` for errors, no `.unwrap()` in library code
- Wrap identifiers in backticks in doc comments (clippy `doc_markdown`)
- The function already has `#[allow(clippy::too_many_lines)]` — keep it

---

### Task 2 — Agent 2: Single-pass for acyclic paths (fixes cs9, −1 unsound)

**Depends on**: Task 1 must complete first (same file).

**What to change**: function `solve_per_path_flow_sensitive` in
`crates/saf-analysis/src/pta/value_origin.rs` (starts at line ~655).

**Problem**: The function has an iterative fixpoint loop:
```rust
for _iteration in 0..max_iterations {
    let mut changed = false;
    for &block_id in &topo_order {
        // process all instructions...
    }
    if !changed { break; }
}
```

With strong updates (store replaces content rather than union), the `changed` flag
oscillates forever: store A sets content to X, later store B sets it to Y, next
iteration store A sets it back to X → never converges.

Per-path analysis always operates on acyclic subgraphs (branch enumeration, not loop
unrolling). For acyclic CFGs, a single topological pass is **exact** — there are no
back-edges that require re-processing.

**Fix**:

1. After computing `topo_order`, detect whether the reachable subgraph has back-edges.
   A back-edge exists if any edge `(a, b)` in `active_edges` has `b` appearing
   before `a` in `topo_order`. Use this to set `let is_acyclic: bool`.

2. If `is_acyclic`: process blocks in topo order **exactly once** (no fixpoint loop).
   Remove the outer `for _iteration` loop and the `changed` tracking. Just iterate
   through `topo_order` once and process all instructions.

3. If NOT acyclic (loop present — unlikely but defensive): keep the current iterative
   fixpoint loop BUT change the strong update in `process_instruction_flow_sensitive`'s
   Store handler to a **weak update** (union instead of replace). This can be done by
   passing a `use_strong_updates: bool` parameter, or more simply: cap iterations at
   a small number (e.g., 10) instead of `max_iterations` (100K) to avoid the
   performance trap, and accept the imprecision.

   The simplest defensive approach: just cap at `max_iterations.min(10)` for the
   non-acyclic case and keep strong updates. This bounds the worst case while keeping
   the common acyclic path fast and precise.

**Verification**: After this change, `cargo check -p saf-analysis` should compile.
Do NOT run tests (leader handles Docker commands).

**Important Rust conventions**:
- Same as Task 1
- Prefer `let ... else { ... }` over `match ... { Some(v) => v, None => ... }`

---

### Task 3 — Leader: Build, format, lint, test

**After both agents complete**:

1. `make fmt && make lint` — fix any formatting or clippy issues
2. `make test` — run full Rust + Python test suite
3. Run PTABen cs_tests:
   ```
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben \
     --compiled-dir tests/benchmarks/ptaben/.compiled --filter "cs_tests/*" \
     -o /workspace/tests/benchmarks/ptaben/cs_results.json'
   ```
   Verify: cs18 4→0 unsound, cs9 1→0 unsound.

4. Run full PTABen (background, ~60s):
   ```
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben \
     --compiled-dir tests/benchmarks/ptaben/.compiled \
     -o /workspace/tests/benchmarks/ptaben/results.json'
   ```
   Verify: no regressions in other categories. Total unsound 75→70.

5. Update `plans/PROGRESS.md`.

## Target

- cs_tests unsound: 5 → 0
- Total unsound: 75 → 70
