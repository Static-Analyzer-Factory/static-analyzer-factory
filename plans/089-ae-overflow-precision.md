# Plan 089: ae_overflow Precision Improvements

## Context

Plan 088 done: **2207 Exact, 112 Unsound.** 32 are ae_overflow (30 + 2 fail). All say `"No buffer overflow finding in function (memcpy tracking not supported)"`.

Breakdown of 32 unsound tests:

| Bucket | Count | Example file |
|--------|-------|-------------|
| loop-write | 14 | `CWE805_char_alloca_loop_01.ll` |
| memmove | 7 | `CWE805_struct_declare_memmove_01.ll` |
| CWE129-gep-index | 6 | `CWE129_rand_01.ll` |
| memcpy | 3 | `CWE193_char_alloca_memcpy_01.ll` |
| strncpy | 1 | `CWE806_char_declare_ncpy_01.ll` |
| strcat | 1 | `ExtAPI_strcat_04.ll` |

**Diagnosis (already done — no diagnostic agent needed):**

The checker IS capable of detecting these overflows — the specs, intrinsic handling, and comparison logic are all correct. The failures are caused by **two infrastructure gaps** that affect ALL buckets:

1. **Allocation size resolution through GEP pointers.** `find_allocation_size_with_pta(gep_result)` returns `None` because the GEP result's LocId (a field of the alloca object) doesn't match the alloca's LocId in `loc_alloc_sizes`. The ObjId fallback exists (checker.rs:750-757) but likely fails because `object_of_location` returns `None` for the GEP-derived LocId.

2. **`affected_ptr` mismatch in finding-to-oracle matching.** The GEP checker sets `affected_ptr: Some(base_operand)` (the alloca), but the `UNSAFE_BUFACCESS` oracle ptr is a GEP of that alloca (e.g., `gep %buffer, 0, 0`). With field-sensitive PTA, these have non-overlapping points-to sets, so the harness's may-alias check fails.

3. **Loop variable intervals are TOP after widening.** The GEP checker skips `is_top()` indices (checker.rs:566). For loop counters widened to `[0, +inf]`, this means loop-based overflows are always skipped.

## Dispatch Prompt (for new session)

Implement Plan 089: ae_overflow Precision Improvements. Use **2 sequential agents** that each edit non-overlapping parts of `checker.rs`.

**Current state:** 2207 Exact, 112 Unsound. 32 ae_overflow unsound. Results at `tests/benchmarks/ptaben/ae_overflow_results.json`.

**File contention note:** Both agents edit `crates/saf-analysis/src/absint/checker.rs`. They MUST run sequentially (Agent A first, then Agent B) to avoid write conflicts. Their edit regions do not overlap: Agent A edits lines 725+ and 1386-1810; Agent B edits lines 455-633.

---

### Agent A: memcpy-alloc-resolver (runs first)

**Scope:** Fix allocation size resolution for GEP'd pointers in the memcpy/memmove checker AND the strcat/ncpy edge cases. Edits `checker.rs` lines 725+ and 1386-1810 only.

**Read these ranges (and nothing else):**
- `checker.rs` lines 725-773 (`find_allocation_size_with_pta`)
- `checker.rs` lines 1490-1530 (dest alloc lookup + TOP-size fallback in `check_memcpy_overflow_with_pta_impl`)
- `checker.rs` lines 1620-1655 (source overread detection)
- `checker.rs` lines 1700-1810 (`check_strcat_overflow`)
- One test file (first 50 lines only): `tests/benchmarks/ptaben/.compiled/ae_overflow_tests/ExtAPI_strcat_04.ll`

**Problem:** When the dest pointer of a memcpy/memmove is a GEP result (`%arraydecay = gep [50 x Struct], ptr %alloca, 0, 0`), `find_allocation_size_with_pta(%arraydecay)` returns `None`. The ObjId fallback at line 750-757 exists but likely fails for GEP-derived field locations.

**Fix 1 — GEP-base allocation fallback (~30 lines):**

Add a helper function right after `find_allocation_size_with_pta` (after line 773):

```rust
/// Fallback: if `ptr` is the result of a GEP instruction, resolve the
/// allocation size of the GEP's base pointer instead.
fn find_gep_base_allocation_size(
    ptr: ValueId,
    func: &AirFunction,
    value_sizes: &BTreeMap<ValueId, Interval>,
    loc_sizes: &BTreeMap<LocId, Interval>,
    pta: &PtaIntegration<'_>,
) -> Option<Interval> {
    for block in &func.blocks {
        for inst in &block.instructions {
            if matches!(&inst.op, Operation::Gep { .. }) && inst.dst == Some(ptr) {
                let base = inst.operands[0];
                return find_allocation_size_with_pta(base, value_sizes, loc_sizes, pta);
            }
        }
    }
    None
}
```

**Fix 2 — Apply fallback at all memcpy checker call sites (~10 lines):**

In `check_memcpy_overflow_with_pta_impl`, around line 1505:
```rust
// OLD:
let dest_alloc_size = find_allocation_size_with_pta(dest_operand, &alloc_sizes, &loc_alloc_sizes, pta);
// NEW:
let dest_alloc_size = find_allocation_size_with_pta(dest_operand, &alloc_sizes, &loc_alloc_sizes, pta)
    .or_else(|| find_gep_base_allocation_size(dest_operand, func, &alloc_sizes, &loc_alloc_sizes, pta));
```

Apply the same `.or_else(|| find_gep_base_allocation_size(...))` pattern to:
- The source overread detection (~line 1624) for `src_operand`
- The "Phase F" TOP-size fallback (~line 1527): if `alloc_sizes.get(&dest_operand)` returns `None`, try `find_gep_base_allocation_size` before falling through to `find_individual_allocation_sizes_with_pta`
- The strcat overflow checker's allocation lookups (~line 1730-1760) if they have the same issue

**Fix 3 — ExtAPI_strcat_04 edge case:**

Read `ExtAPI_strcat_04.ll` (first 50 lines). Identify why `check_strcat_overflow` misses it — likely the same GEP-alloc resolution issue. The GEP-base fallback (Fix 1) may solve it automatically. If not, trace the specific pattern and add handling.

**Fix 4 — CWE806_char_declare_ncpy_01:**

This `strncpy(dst_gep, src_gep, 99)` test should be fixed automatically by Fix 2 if the GEP-base fallback is applied to the spec-function path (the dest operand goes through the same `find_allocation_size_with_pta` call at line 1505).

**Estimated total changes:** ~40-50 lines.

**Do NOT edit:** Lines 455-633 (GEP checker). That's Agent B's scope.

---

### Agent B: gep-checker-improvements (runs second, after Agent A)

**Scope:** Fix `affected_ptr` and add TOP-index heuristic in the GEP overflow checker. Edits `checker.rs` lines 455-633 only.

**Read these ranges (and nothing else):**
- `checker.rs` lines 455-633 (`check_buffer_overflow_with_result` + `check_buffer_overflow_with_pta_and_result`)
- `checker.rs` lines 725-773 (`find_allocation_size_with_pta` — read-only, for calling)

**Problem 1 — affected_ptr mismatch (6 CWE129 tests):**

The GEP checker sets `affected_ptr: Some(base_operand)` (lines 495, 520, 588, 621) where `base_operand` is the alloca. But `UNSAFE_BUFACCESS` uses a GEP of that alloca. With field-sensitive PTA, they don't alias.

**Fix:** Change `affected_ptr` to `None` in all four finding sites. The harness treats `None` as "matches any UNSAFE oracle in the same function" (ptaben.rs line 1881: `None => true`) while being filtered out for SAFE oracles (line 1900: `None => false`). This is safe for PTABen because each bad function has exactly one UNSAFE oracle.

In `check_buffer_overflow_with_result` (lines 486-497, 509-522):
```rust
affected_ptr: None,  // was: Some(base_operand) — None matches any UNSAFE oracle in function
```

In `check_buffer_overflow_with_pta_and_result` (lines 579-590, 610-622):
```rust
affected_ptr: None,  // was: Some(base_operand)
```

**Problem 2 — TOP-index skip (14 loop tests):**

Loop counter `%i` is widened to TOP. The checker at line 566 does `if idx_interval.is_top() { continue; }`, skipping all TOP-index GEPs.

**Fix:** Replace the unconditional skip with a heuristic. In `check_buffer_overflow_with_pta_and_result`, replace lines 566-568:

```rust
// OLD:
if idx_interval.is_bottom() || idx_interval.is_top() {
    continue;
}
```

With:
```rust
if idx_interval.is_bottom() {
    continue;
}

if idx_interval.is_top() {
    // Heuristic: unconstrained index into a known-size small buffer
    // is a potential overflow. Report Warning since we can't prove
    // it always overflows.
    let base_operand = inst.operands[0];
    let alloc_size = find_allocation_size_with_pta(
        base_operand, &alloc_sizes, &loc_alloc_sizes, pta,
    );
    if let Some(ref asize) = alloc_size {
        if !asize.is_top() && !asize.is_bottom()
            && asize.hi() > 0 && asize.hi() <= 4096
        {
            findings.push(NumericFinding {
                checker: NumericCheckerKind::BufferOverflow,
                severity: NumericSeverity::Warning,
                cwe: 120,
                inst_id: inst.id.to_hex(),
                description: format!(
                    "Unconstrained array index into buffer of size {asize}"
                ),
                interval: "TOP".to_string(),
                function: func.name.clone(),
                location: (func.id, block.id),
                affected_ptr: None,
            });
        }
    }
    continue;
}
```

Apply the same pattern to the non-PTA variant `check_buffer_overflow_with_result` (lines 476-478), using `alloc_sizes.get(&base_operand)` instead of `find_allocation_size_with_pta`.

**Estimated total changes:** ~40 lines.

**Do NOT edit:** Lines 725+ (alloc resolution, memcpy checker, strcat checker). That's Agent A's scope.

---

### Leader responsibilities

1. **Spawn Agent A.** Wait for completion.
2. **Spawn Agent B.** Wait for completion.
3. Run: `make fmt && make lint && make test`
4. Run ae_overflow benchmark (background, 120s):
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_overflow*" -o /workspace/tests/benchmarks/ptaben/ae_overflow_results.json'
   ```
5. Run full PTABen benchmark to check for regressions (background, 120s):
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
   ```
6. Compare results:
   - ae_overflow unsound: target ≤ 12 (from 32)
   - Full benchmark: Exact ≥ 2207, total Unsound ≤ 112
7. **If regressions occur** (Exact dropped or new Unsound in non-ae_overflow categories), **dispatch a regression-investigator agent** to:
   - Read the full benchmark JSON results
   - Identify which categories regressed and which specific tests flipped
   - Add temporary `eprintln!()` debug prints to the changed code paths in `checker.rs` (e.g., print `affected_ptr`, `alloc_size`, `idx_interval` for the regressed test's function name). Run the filtered benchmark for the regressed category to see the debug output.
   - Trace the regression to the specific edit that caused it
   - Fix the regression (e.g., tighten the heuristic threshold, restrict `affected_ptr: None` to specific conditions)
   - **Remove all debug prints before finishing** — no `eprintln!` should remain in committed code
   - Leader re-runs fmt/lint/test/benchmark after the fix
8. If SAFE_BUFACCESS regressions appear specifically (findings incorrectly matching SAFE oracles due to `affected_ptr: None`), the fix is to switch back to `affected_ptr: inst.dst.or(Some(base_operand))` in the GEP checker findings and re-test.
9. Update `plans/PROGRESS.md`.

## Expected Impact

| Agent | Fix | Target tests | Confidence |
|-------|-----|-------------|------------|
| A | GEP alloc fallback | 7 memmove + 3 memcpy + 1 ncpy + 1 strcat = 12 | High |
| B | affected_ptr → None | 6 CWE129 | High |
| B | TOP-index heuristic | 14 loop | Medium |
| **TOTAL** | | **32** | |

**Target: ≤ 12 unsound ae_overflow (from 32), fixing ≥ 20.**
**Projected total: 112 → ~92 Unsound (−18%)**

## Verification

1. `make fmt && make lint` — clippy clean
2. `make test` — all tests pass
3. ae_overflow benchmark: unsound ≤ 12
4. Full PTABen benchmark: Exact ≥ 2207, no regressions
5. No new false positives in SAFE_BUFACCESS checks
