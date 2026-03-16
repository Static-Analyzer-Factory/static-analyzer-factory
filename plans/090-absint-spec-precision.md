# Plan 090: Absint Spec Precision + Switch Refinement

## Context

Plan 089 done: **2238 Exact, 81 Unsound.** 10 are ae_assert. Investigation of the 10 unsound ae_assert tests reveals 6 are fixable with spec improvements and transfer function enhancements:

| Test | Root Cause | Fix |
|------|-----------|-----|
| `INTERVAL_test_20` | `rand()` spec interval not applied to return value | A: Spec return interval lookup |
| `BASIC_array_int_0` | Constant global not propagated through memcpy | B: Memcpy constant propagation |
| `BASIC_arraycopy2` | Same as above | B: Memcpy constant propagation |
| `INTERVAL_test_19` | Switch case doesn't narrow switched-on variable | C: Switch case refinement |
| `INTERVAL_test_2` | scanf out-param + branch refinement | D: Scanf spec + invalidation |
| (infrastructure) | realloc missing `aliases: param.0` | E: Realloc spec fix |

4 remaining ae_assert tests (`BASIC_array_func_6`, `BASIC_arraycopy1`, `BASIC_ptr_func_0`, `INTERVAL_test_49`) require deeper interprocedural store-through-pointer work — deferred.

## Dispatch Prompt (for new session)

Implement Plan 090: Absint Spec Precision + Switch Refinement. Use **1 agent** that implements 5 fixes sequentially.

**Current state:** 2238 Exact, 81 Unsound. 10 ae_assert unsound.

---

### Agent: absint-spec-precision

**Scope:** 5 fixes across `transfer.rs`, spec YAML files, and fixpoint/block-entry logic. ~80 lines total.

**Read these files before starting:**
- `crates/saf-analysis/src/absint/transfer.rs` lines 460-490 (CallDirect handler, TOP fallback)
- `crates/saf-analysis/src/absint/transfer.rs` lines 1060-1190 (`compute_call_return_with_summaries`)
- `crates/saf-analysis/src/absint/transfer.rs` memcpy/intrinsic handling section (search for `Memcpy` or `llvm.memcpy`)
- `crates/saf-analysis/src/absint/fixpoint.rs` block entry processing (where `CondBr` refinement happens)
- `share/saf/specs/libc/stdlib.yaml` lines 185-195 (rand spec with `returns.interval`)
- `share/saf/specs/libc/alloc.yaml` lines 24-34 (realloc spec)
- `share/saf/specs/libc/stdio.yaml` lines 195-210 (scanf spec)
- `crates/saf-core/src/spec/types.rs` — `ReturnSpec`, `interval()` method, `alias_param_index()`
- Test files (first 60 lines each): `INTERVAL_test_20-0.ll`, `BASIC_array_int_0-0.ll`, `BASIC_arraycopy2-0.ll`, `INTERVAL_test_19-0.ll`, `INTERVAL_test_2-0.ll` in `tests/benchmarks/ptaben/.compiled/ae_assert_tests/`

---

### Fix A: Spec Return Interval for External Calls (~15 lines)

**Problem:** `rand()` has `returns.interval: [0, 2147483647]` in `stdlib.yaml`, but the abstract interpreter never consults specs for external function return intervals. It always returns TOP.

**Where:** `transfer.rs`, in `compute_call_return_with_summaries()` or the CallDirect handler.

**Fix:** After checking the pre-computed `return_intervals` map and before falling through to TOP, look up the callee in the spec registry. If the spec has `returns.interval`, use that instead of TOP.

```rust
// Pseudocode — adapt to actual API:
// In the code path where callee has no body and no summary:
if let Some(pta) = ctx.pta {
    let callee_name = /* get callee name from module/AirBundle */;
    if let Some(spec) = pta.spec_registry().lookup(&callee_name) {
        if let Some(ref ret_spec) = spec.returns {
            if let Some(iv) = ret_spec.interval() {
                return (iv, false);
            }
        }
    }
}
// Fall through to TOP
```

**Key:** The spec registry is accessible via `PtaIntegration` or `TransferContext`. The `ReturnSpec` type in `saf-core/src/spec/types.rs` has an `interval()` method — verify its signature and return type. The interval may need conversion to the right bit-width.

**Verification:** After this fix, `rand()` should return `[0, 2147483647]`. Then `srem %rand, 128` should narrow to `[0, 127]` (verify the srem transfer function handles this). Then `trunc i8` + `zext i32` should preserve `[0, 127]`. Then branch refinement narrows to `[97, 122]`. INTERVAL_test_20 should become Exact.

---

### Fix B: Memcpy Constant Propagation (~20 lines)

**Problem:** `llvm.memcpy(dest_alloca, @const_global, len)` doesn't propagate constant values from the global to the alloca's loc_memory slots. Loads from dest return TOP.

**Where:** `transfer.rs`, in the memcpy/memmove intrinsic transfer handling.

**Fix:** When processing a memcpy intrinsic, check if the source operand's PTA points-to set includes locations that have known intervals in `loc_memory` (from global constant initialization). If so, copy those intervals to the destination's locations.

```rust
// Pseudocode:
// In memcpy transfer handling, after existing logic:
if let Some(pta) = ctx.pta {
    // Get source and dest points-to sets
    let src_pts = pta.points_to_set(src_operand);
    let dst_pts = pta.points_to_set(dest_operand);

    // Copy loc_memory entries from source locations to dest locations
    let mut copies = Vec::new();
    for src_loc in src_pts.iter() {
        if let Some(interval) = state.loc_memory.get(&src_loc).cloned() {
            for dst_loc in dst_pts.iter() {
                copies.push((dst_loc, interval.clone()));
            }
        }
        // Also check child locations (array elements, struct fields)
        for (child_loc, child_iv) in state.loc_memory_children(src_loc) {
            // Map child offset from src to corresponding dst child
            // ... (depends on location factory API)
        }
    }
    for (loc, iv) in copies {
        state.loc_memory.insert(loc, iv);
    }
}
```

**Key challenge:** The constant global's element values need to be in `loc_memory` already (from global initializer analysis). Verify this by checking how global constants are initialized in the abstract interpreter. If they're not in `loc_memory`, a different approach may be needed (e.g., reading the constant directly from the AirBundle's global definitions).

**Alternative simpler approach:** If the source is a constant global, extract the constant values directly from the `AirBundle.globals` and store them into the dest's loc_memory. This avoids depending on global init already being in loc_memory.

**Verification:** After this fix, `memcpy(a, @const_array, 40)` followed by `load a[9]` should return `[9, 9]` (or whatever the constant is). BASIC_array_int_0 and BASIC_arraycopy2 should become Exact.

---

### Fix C: Switch Case Interval Refinement (~15 lines)

**Problem:** When a `switch %val [i32 0 → bb0, i32 1 → bb1, default → bbN]` branches to a case target, the abstract interpreter doesn't narrow `%val` to the case constant in that block. Unlike `CondBr` which refines via `refine_branch_condition()`, switch cases get no refinement.

**Where:** `fixpoint.rs` or `transfer.rs`, in the block entry processing where `CondBr` predecessor refinement is applied.

**Fix:** When entering a block, check if any predecessor terminates with a `Switch` and this block is one of its case targets. If so, narrow the switched-on value to `[case_val, case_val]`.

```rust
// Pseudocode:
// In block entry refinement (same place CondBr refinement happens):
for pred_block in predecessors(current_block) {
    if let Some(switch_inst) = pred_block.terminator() {
        if let Operation::Switch { operand, cases, .. } = &switch_inst.op {
            for (case_val, target_block) in cases {
                if target_block == current_block.id {
                    // Narrow switched-on operand to this case constant
                    state.set(*operand, Interval::from_constant(*case_val));
                }
            }
            // For default block: narrow to exclude all case values
            // (optional, harder — skip for now)
        }
    }
}
```

**Key:** Find the exact location where CondBr refinement is applied. The switch refinement should follow the same pattern. The `Operation::Switch` variant's structure needs to be checked (field names, how cases are stored).

**Verification:** After this fix, in INTERVAL_test_19, the switch on `%rem = srem %i, 2` should narrow `%rem` to `[0, 0]` in case 0 and `[1, 1]` in case 1. After processing through `foo()`, `%i` should be `[2, 2]`, and `2 % 2 == 0` should be provably true.

---

### Fix D: Scanf Parameter Modification (~10 lines)

**Problem:** After `scanf("%d", &a)`, the variable `a` should be invalidated (set to TOP) because scanf writes through the pointer. Currently, the abstract interpreter may leave `a` at its pre-call value (bottom/uninitialized), preventing branch refinement from working.

**Where:** `stdio.yaml` spec + `transfer.rs` external call handling.

**Fix:** Two parts:
1. Update `scanf` spec in `stdio.yaml` to indicate variadic params are modified (or add a generic `writes_params: variadic` flag).
2. In the CallDirect handler for external functions, when a spec indicates a parameter `modifies`/`writes` its pointee, set the pointed-to location to TOP in loc_memory.

**Simpler alternative:** Since scanf's primary effect is writing TOP to its out-params, and the abstract interpreter already invalidates memory for unknown calls (the `modifies_unknown` flag in summaries), verify that this invalidation is happening. If `a` is already TOP after scanf, then branch refinement (`if a > 5`) should narrow it to `[6, MAX]`. Debug which step fails: is `a` TOP? Is refinement working?

**Investigation before coding:** Before implementing, add a temporary `eprintln!()` in the CallDirect handler for INTERVAL_test_2's function to see what `a`'s interval is before and after the scanf call, and after the branch refinement. This tells us exactly what's broken.

**Verification:** After this fix, `scanf` should leave `a` as TOP, then `if a > 5` narrows to `[6, MAX]`, then `a + 1 > 6` is provably true. INTERVAL_test_2 should become Exact.

---

### Fix E: Realloc `aliases: param.0` (~1 line)

**Problem:** `realloc(ptr, size)` returns a pointer that may alias `ptr`, but the spec doesn't create a copy constraint.

**Where:** `share/saf/specs/libc/alloc.yaml` line 27.

**Fix:**
```yaml
# In alloc.yaml, realloc spec:
returns:
  pointer: fresh_heap
  nullness: maybe_null
  aliases: param.0       # ADD THIS LINE
```

**Verification:** After this fix, `realloc(p, n)` returns a value that aliases `p` in PTA, so `*realloc(p, n)` may alias `*p`. No specific PTABen test to verify, but improves correctness.

---

### Leader responsibilities

1. **Spawn agent.** Wait for completion.
2. Run: `make fmt && make lint && make test`
3. Run ae_assert benchmark (background, 120s):
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_assert_tests/*" -o /workspace/tests/benchmarks/ptaben/ae_assert_results.json'
   ```
4. Run full PTABen benchmark (background, 120s):
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
   ```
5. Compare results:
   - ae_assert unsound: target ≤ 4 (from 10)
   - Full benchmark: Exact ≥ 2238, total Unsound ≤ 81
6. **If regressions occur**, dispatch a regression-investigator agent to trace and fix.
7. Update `plans/PROGRESS.md`.

## Expected Impact

| Fix | Target tests | Confidence |
|-----|-------------|------------|
| A: Spec return interval | INTERVAL_test_20 | High |
| B: Memcpy constant prop | BASIC_array_int_0, BASIC_arraycopy2 | Medium |
| C: Switch refinement | INTERVAL_test_19 | High |
| D: Scanf invalidation | INTERVAL_test_2 | Medium |
| E: Realloc alias | (infrastructure) | High |
| **TOTAL** | **5 ae_assert + 1 infra** | |

**Target: ae_assert ≤ 4 unsound (from 10), fixing 6.**
**Projected total: 81 → ~75 Unsound (−7%)**

## Verification

1. `make fmt && make lint` — clippy clean
2. `make test` — all tests pass
3. ae_assert benchmark: unsound ≤ 4
4. Full PTABen benchmark: Exact ≥ 2238, no regressions
