# Plan 134: Pointer-Keyed Value Cache

**Goal:** Eliminate redundant `print_to_string()` FFI calls in the LLVM→AIR conversion by using `LLVMValueRef` pointer identity as the value cache key, replacing the string-keyed `BTreeMap`.

**Baseline (Plan 133):**
```
bash Total: ~27.2s
  Load:   17-18s (LLVM parsing + AIR conversion)
  Ander:  ~10.0s
```

---

## Problem

`get_or_create_value_id` calls `value.print_to_string()` on every invocation — even cache hits — because the string repr is computed BEFORE the cache lookup. With ~500-700k calls across bash's 141k instructions (2-5 operands per instruction), this dominates the 17-18s load time.

Each `print_to_string` call:
1. Crosses the Rust→C++ FFI boundary
2. LLVM serializes the IR node to text
3. Allocates a C string, then copies to a Rust `String`

## Solution

Replace `value_ids: BTreeMap<String, ValueId>` with `ptr_cache: FxHashMap<usize, ValueId>` keyed by `value.as_value_ref() as usize`. LLVM SSA values have stable pointer identity within a module context — the same `%x` referenced as an operand in different instructions returns the same `LLVMValueRef`.

### Part A: Pointer Cache in `get_or_create_value_id`

**Before:**
```rust
let repr = value.print_to_string().to_string();  // ALWAYS called
if let Some(&existing_id) = self.value_ids.get(&repr) {
    return existing_id;  // Cache hit, but FFI cost already paid
}
```

**After:**
```rust
let ptr_key = value.as_value_ref() as usize;
if let Some(&existing_id) = self.ptr_cache.get(&ptr_key) {
    return existing_id;  // NO print_to_string on hit
}
let repr = value.print_to_string().to_string();  // Only on miss
// ... existing global detection, constant GEP, BLAKE3 derivation ...
self.ptr_cache.insert(ptr_key, value_id);
```

### Part B: Instruction Result Cache

**Before (line 826):**
```rust
let repr = inst.print_to_string().to_string();
let dst_id = if let Some(&existing_id) = ctx.value_ids.get(&repr) {
    existing_id  // phi forward-reference
} else { ... };
```

**After:**
```rust
let ptr_key = inst.as_value_ref() as usize;
let dst_id = if let Some(&existing_id) = ctx.ptr_cache.get(&ptr_key) {
    existing_id  // phi forward-reference (pointer identity)
} else {
    let id = ctx.create_inst_result_id(inst_id);
    ctx.ptr_cache.insert(ptr_key, id);
    id
};
```

Phi forward references work because: phi conversion calls `get_or_create_value_id(incoming_value)` which uses the same `LLVMValueRef` pointer as the later instruction definition.

### Part C: Update Remaining Call Sites

- Lines 905, 916 (intrinsic PassThrough/External): use `ptr_cache.insert` instead of `value_ids.insert`
- `clear()` call at function boundary (line 621): change to `ptr_cache.clear()`

---

## Files Modified

- `crates/saf-frontends/src/llvm/mapping.rs` — replace `value_ids` with `ptr_cache`, update all usages
- `crates/saf-frontends/Cargo.toml` — add `rustc-hash` dependency

## Validation

1. `make fmt && make lint`
2. `cargo nextest run --release -p saf-analysis`
3. PTABen regression (69 Unsound expected)
4. CruxBC benchmark (load time improvement)

---

## Results

### CruxBC Benchmark (clean, no solver-stats)

| Phase | Plan 133 | Plan 134 | Change |
|-------|----------|----------|--------|
| **Load** | 17-18s | **3.0-3.2s** | **-82%** |
| Ander | ~10.0s | 9.5-9.8s | ~flat |
| **Total** | ~27.2s | **12.5-13.0s** | **-52%** |

The pointer-keyed cache eliminated the `print_to_string()` FFI bottleneck. The vast majority of `get_or_create_value_id` calls now hit the pointer cache (O(1) `FxHashMap` lookup with `usize` key), completely skipping the LLVM C++ serialization + string allocation + `BTreeMap<String>` comparison that previously dominated the load phase.

### Validation

| Check | Result |
|-------|--------|
| Tests | 1421 pass, 5 skip |
| PTABen | 69 Unsound (no regression) |
| `make fmt && make lint` | clean |
