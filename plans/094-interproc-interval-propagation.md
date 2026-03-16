# Plan 094: ae_assert_fail Precision — Loop Narrowing, Phi-Path, Array Inline

## Investigation Summary

3 agents investigated the 35 unsound ae_assert_fail cases. **Key finding:** SAF
already has interprocedural return value propagation (4-phase: mod/ref → bottom-up
SCC summaries → two-pass re-analysis → context-sensitive refinement). Narrowing
(3 iterations) also exists. The remaining failures have distinct root causes:

| Root Cause | Count | Fixability |
|---|---|---|
| Recursive functions + global side effects | 13 | Deferred (research-grade) |
| Phi-path: unreachable predecessor not filtered | 8 | **Agent 2** |
| Loop narrowing not effective enough | 6 | **Agent 1** |
| Relational/parametric (undef == undef) | 4 | Infeasible (needs symbolic domain) |
| Array value through inline analysis | 2 | **Agent 3** |
| External API / UAF / other | 2 | Deferred |

## Team Structure

```
Leader (main agent)
  ├── Agent 1: Loop narrowing diagnosis + fix
  │     Files: interprocedural.rs (lines 2029-2154), fixpoint.rs, config.rs
  │     Tests: test_7, LOOP_for_break02, LOOP_while_nested
  │
  ├── Agent 2: Phi-path unreachable predecessor filtering
  │     Files: condition_prover.rs (lines 770-842)
  │     Tests: test_26, test_29, test_30, test_31
  │
  └── Agent 3: Array inline loc_memory diagnosis + fix
        Files: transfer.rs (lines 1550-1663)
        Tests: BASIC_array_func_1, BASIC_array_func_2
```

**Leader responsibilities:** Run `make fmt && make lint`, `make test`, PTABen
benchmarks. Agents MUST NOT run `make` commands.

---

## Agent 1: Loop Narrowing Effectiveness

### Problem

After widened fixpoint + 3 narrowing iterations, loop counter variables remain
imprecise. Example from test_7:

```c
int a = 0, b = 0;
while (a < 10) { a++; b += 2; }
svf_assert(a == 10);  // lhs=[0, 2147483647], rhs=[10,10]
svf_assert(b == 20);  // lhs=[0, 2147483647], rhs=[20,20]
```

The narrowing phase (interprocedural.rs:2029-2154) re-runs transfer functions
in RPO order using `narrow_state(old, old.join(propagated))`. This should tighten
bounds at ±∞ but currently doesn't produce `a=[10,10]` at loop exit.

### Files to Read (in order)

1. `crates/saf-analysis/src/absint/interprocedural.rs` lines 1990-2155 — widening
   section + narrowing phase. The narrowing loop is at line 2031.
2. `crates/saf-analysis/src/absint/fixpoint.rs` lines 1093-1155 — `narrow_state()`
   function that narrows point-wise.
3. `crates/saf-analysis/src/absint/interval.rs` lines 976-1004 — `Interval::narrow()`
   method: only refines bounds at ±∞.
4. `crates/saf-analysis/src/absint/config.rs` — `narrowing_iterations` default is 3.
5. Test files: `tests/benchmarks/ptaben/.compiled/ae_assert_tests_fail/INTERVAL_test_7-0.ll`
   and `LOOP_for_break02-0.ll`

### Diagnosis Steps

1. Read the narrowing phase code (interprocedural.rs:2029-2154) carefully.
2. Trace through test_7 mentally: `a` is widened from [0,0] → [0,1] → widen → [0,MAX].
   After widening fixpoint converges, the narrowing phase re-runs. At the loop exit
   edge (`a >= 10`), `refine_branch_condition` should narrow `a` to [10, MAX]. But
   at the NEXT block's phi or use, does the narrowed value persist?
3. Add `tracing::debug!` logging at key points in the narrowing phase to understand:
   - What `old_state` and `propagated_state` are at each narrowing iteration
   - Whether `narrow_state` actually changes anything
   - What the state is at the block containing `svf_assert` after narrowing
4. Check if the issue is that `Interval::narrow()` only refines bounds at signed ±∞.
   If `a = [0, 2147483647]` (hi = i32::MAX = signed_max(32)), then `narrow([0, MAX],
   [0, 10])` should produce [0, 10] because hi == signed_max → use other.hi = 10.
   But if `a = [0, 2147483646]` (hi ≠ MAX), narrowing preserves self.hi. Check what
   the actual widened interval is.

### Possible Fixes

**Fix A: Generalized narrowing** — Instead of only refining bounds at ±∞, also
refine when the narrowed bound is strictly tighter:
```rust
// In Interval::narrow():
// Current: only refine at extremes
let hi = if self.hi == max_bound { other.hi } else { self.hi };
// Proposed: refine when other is tighter (standard narrowing)
let hi = if self.hi == max_bound || other.hi < self.hi {
    other.hi
} else {
    self.hi
};
```
**CAUTION:** This may not terminate. The standard narrowing guarantee is that
narrowing is monotonically decreasing (self ⊇ narrow(self, other)). The proposed
change satisfies this since `other.hi < self.hi` means we're shrinking.

**Fix B: Increase narrowing iterations** — Change default from 3 to 5 or 8 in
config.rs. This is a one-line change.

**Fix C: Branch-condition narrowing at loop exits** — After the main narrowing
loop, add a final pass that specifically applies negated loop conditions at exit
edges. This ensures the exit condition (e.g., `a >= 10`) is reflected in the
post-loop state.

### Acceptance Criteria

- test_7: `a` interval at assertion site is [10, 10] or [10, MAX] (not [0, MAX])
- LOOP_for_break02: interval at assertion tighter than [0, 5]
- No regressions in other ae_assert tests (run full ae_assert_tests + ae_assert_tests_fail)

### Files to Edit

- `crates/saf-analysis/src/absint/interval.rs` — `narrow()` method (~line 976)
- `crates/saf-analysis/src/absint/config.rs` — `narrowing_iterations` if needed
- `crates/saf-analysis/src/absint/interprocedural.rs` — narrowing phase (~line 2029)
  ONLY the section between lines 2029-2154. Do NOT touch the widening worklist
  loop above (lines 1807-2027).

---

## Agent 2: Phi-Path Unreachable Predecessor Filtering

### Problem

The condition prover evaluates phi nodes by checking ALL incoming values, but
ignores the `block_id` of each incoming. When a phi has `[false, %entry]` and
`[comparison_result, %cond_block]`, the prover can't prove the phi is always true
because the `false` incoming COULD be selected.

Error pattern:
```
Phi path may fail: Phi with 2 incoming: [constant false (value=0), ICmpSlt always true (...)]
```

The fix: pass block reachability information to the condition prover and skip
phi incoming from unreachable blocks. The absint's fixpoint already tracks which
blocks are reached — if a block's entry state is `bottom` (unreachable), its phi
incoming can be ignored.

### Files to Read (in order)

1. `crates/saf-analysis/src/z3_utils/condition_prover.rs` lines 770-842 — the phi
   evaluation handler. Note `_block_id` is IGNORED at line 790.
2. Same file, lines 144-220 — `prove_conditions_generic()` entry point and how it
   builds data structures. Note the `interval_source` parameter.
3. Same file, lines 413-470 — `evaluate_condition_recursive()` to understand the
   call chain that reaches the phi handler.
4. `crates/saf-analysis/src/absint/result.rs` — `AbstractInterpResult` and
   `InterproceduralResult` types. Look for what reachability info they expose.
5. Test file: `tests/benchmarks/ptaben/.compiled/ae_assert_tests_fail/INTERVAL_test_26-0.ll`
   — short-circuit `&&` pattern: `phi i1 [false, %entry], [%cmp1, %land.rhs]`
6. Test file: `INTERVAL_test_29-0.ll` and `INTERVAL_test_30-0.ll` — similar patterns.

### Diagnosis Steps

1. For test_26: trace the CFG. `entry` branches to `land.rhs` (if `a >= 0`) or
   `land.end` (if `a < 0`). At `land.end`, the phi gets `false` from `entry` or
   `%cmp1` from `land.rhs`. The assertion is on this phi.
2. The prover sees `Phi [false, cmp_always_true]` → "may fail" because false is
   possible. But if the entry→land.end edge is only taken when `a < 0`, and the
   prover could check "is `entry → land.end` reachable?", it could prune the false.
3. **HOWEVER**: for test_26, `srem undef, 5 = [-4, 4]`, so `a < 0` IS possible.
   The `false` incoming IS reachable. This test may be genuinely imprecise.
4. Check test_29 and test_30 differently — the phi `false` incoming may come from
   a block that the absint marked unreachable.

### Implementation Approach

**Step 1: Add `IntervalQuery::is_block_reachable(block_id)` method.**

The `IntervalQuery` trait is defined in condition_prover.rs or result.rs. Add:
```rust
fn is_block_reachable(&self, block_id: BlockId) -> bool { true } // default: all reachable
```

Implement it on `AbstractInterpResult` and `InterproceduralResult` using their
`inst_states` or `block_entry_states` — a block is reachable if its entry state
is NOT bottom.

**Step 2: Use reachability in phi evaluation.**

In the phi handler (condition_prover.rs ~line 790), change:
```rust
for (_block_id, value_id) in incoming {
```
to:
```rust
for (block_id, value_id) in incoming {
    // Skip phi incoming from unreachable blocks
    if !interval_source.is_block_reachable(*block_id) {
        descriptions.push("unreachable predecessor (skipped)".to_string());
        continue;
    }
```

**Step 3: Pass the function's block reachability info.**

The condition prover currently doesn't know which function a phi belongs to. The
prover scans all functions (line 199: `for func in &module.functions`). The
`interval_source` has the analysis result for all functions. The
`InterproceduralResult` already stores per-instruction states — if a block has
no instructions in `inst_states`, it's unreachable.

### Acceptance Criteria

- At least 2-3 of the phi-path tests change from Unsound to Exact
- No regressions in any other test category
- Tests that genuinely have reachable `false` paths should remain Unsound (correct)

### Files to Edit

- `crates/saf-analysis/src/z3_utils/condition_prover.rs` — phi handler and
  `IntervalQuery` trait. ONLY this file.
- `crates/saf-analysis/src/absint/result.rs` — implement `is_block_reachable()`
  on result types. ONLY this file.

---

## Agent 3: Array Inline loc_memory Propagation

### Problem

`BASIC_array_func_1` calls `getValue(arr, 0, 0)` where `arr[0][0] = 10`. The
function loads from `arr[x][y]` and returns it. Expected return: `[10, 10]`.
Actual: `[-MAX, MAX]` (TOP).

Error:
```
lhs=[-9223372036854775808,9223372036854775807], rhs=[10,10]
```

### Files to Read (in order)

1. `tests/benchmarks/ptaben/.compiled/ae_assert_tests_fail/BASIC_array_func_1-0.ll`
   — the test IR. Identify the function `getValue` and its parameters.
2. `crates/saf-analysis/src/absint/transfer.rs` lines 1550-1663 — `analyze_callee_inline()`.
   Focus on how `caller_loc_memory` is propagated (lines 1576-1619) and how the
   return value is computed (lines 1624-1649).
3. `crates/saf-analysis/src/absint/transfer.rs` lines 1087-1205 —
   `compute_call_return_with_summaries()`. Check the inline eligibility conditions
   and how `arg_pts` (PTA points-to sets for arguments) are collected.
4. `crates/saf-analysis/src/absint/state.rs` — `AbstractState` loc_memory methods:
   `store_loc()`, `load_loc()`, `loc_memory_entries()`.

### Diagnosis Steps

1. Read the test .ll to understand the call: what are the arguments to `getValue`?
   What GEP chain does the callee use to access `arr[x][y]`?
2. In `analyze_callee_inline()`, trace what happens:
   - Are the argument intervals correct? (x=[0,0], y=[0,0], arr=pointer)
   - Does `arg_pts` contain the PTA locations for `arr`?
   - Does `caller_loc_memory` have `arr[0][0] = [10, 10]`?
   - When the callee does `GEP arr, x, y` → `Load`, does the GEP resolve to the
     right location? Does `load_loc()` find the stored value?
3. Add `tracing::debug!` at:
   - The parameter binding loop (line 1568) to print arg intervals and PTA sets
   - The loc_memory propagation (line 1608) to print what's propagated
   - Before the GEP/Load handling inside the callee to see if the location resolves
4. The likely issue: the callee's `GEP` uses its OWN parameter `ValueId` as the base
   pointer, not the caller's. PTA resolves the callee's parameter to the same
   locations as the caller's argument. But the GEP inside the callee creates a NEW
   location that doesn't match any location in `caller_loc_memory`.

### Possible Fixes

**Fix A: Pre-resolve callee GEP locations.** In `analyze_callee_inline()`, after
binding parameters, scan the callee for GEP instructions. For each GEP whose base
pointer has known PTA targets, pre-compute the resulting location and register it
in the callee state's GEP targets. This way, when the callee does `Load` from a
GEP result, the load can find the loc_memory entry.

**Fix B: Propagate ALL loc_memory from caller.** Currently, only locations matching
`arg_reachable_locs` are propagated (lines 1598-1616). If the callee accesses a
sub-element (e.g., arr[0][0] when we only passed arr[0]), the specific location
may not be in `arg_reachable_locs`. Fix: also include child locations (locations
with the same base object but extended field path).

### Acceptance Criteria

- BASIC_array_func_1: return interval is [10, 10] (not TOP)
- BASIC_array_func_2: return interval includes [21, 21] (currently [10, 21])
- No regressions in other tests

### Files to Edit

- `crates/saf-analysis/src/absint/transfer.rs` — `analyze_callee_inline()` function
  ONLY (lines 1550-1663). Do NOT touch `compute_call_return_with_summaries` or
  the CallDirect transfer function above it.

---

## Execution Protocol

### Phase 0: Leader creates team + tasks, spawns all 3 agents in parallel.

### Phase 1: All agents work simultaneously on their files (no overlap).

- Agent 1 edits: `interval.rs`, `config.rs`, `interprocedural.rs` (lines 2029-2154 ONLY)
- Agent 2 edits: `condition_prover.rs`, `result.rs`
- Agent 3 edits: `transfer.rs` (lines 1550-1663 ONLY)

**No file overlap between agents.**

### Phase 2: Leader runs verification after each agent completes.

```bash
make fmt && make lint    # Must pass
make test                # All 1447 tests must pass
# PTABen regression check:
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben \
  --compiled-dir tests/benchmarks/ptaben/.compiled \
  --filter "ae_assert_tests_fail/*" \
  -o /workspace/tests/benchmarks/ptaben/094-results.json'
```

### Phase 3: Leader runs full PTABen to check for cross-category regressions.

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben \
  --compiled-dir tests/benchmarks/ptaben/.compiled \
  -o /workspace/tests/benchmarks/ptaben/094-full-results.json'
```

Expected: 2240+ Exact, ≤ 80 Unsound (no regression from current 2240/80).

## Expected Impact

- Current: 35 ae_assert_fail unsound, 2240 Exact total, 80 Unsound total
- Agent 1 (loop narrowing): −2 to −4 ae_assert_fail unsound
- Agent 2 (phi-path): −2 to −4 ae_assert_fail unsound
- Agent 3 (array inline): −1 to −2 ae_assert_fail unsound
- **Total: 27-30 ae_assert_fail unsound (from 35), 73-75 total unsound (from 80)**

## Deferred (Not in This Plan)

- 13 recursive global accumulation tests (requires research-grade recursive effect analysis)
- 4 relational/symbolic tests (requires affine/polyhedra domain)
- 2 external API / UAF tests (different analysis domains)

## Investigation Reports

- `docs/debug/interproc-ll-analysis.md` — per-test .ll pattern classification
- `docs/debug/interproc-absint-trace.md` — absint code path trace
- `docs/debug/interproc-callgraph-analysis.md` — callgraph ordering analysis
