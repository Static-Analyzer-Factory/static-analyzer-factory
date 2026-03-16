# Plan 025: Memory SSA

**Epic:** E11 — Memory SSA
**Status:** approved
**Depends on:** E10 (refined call graph), E4 (Andersen PTA), E3 (CFG/ICFG)

---

## Goal

Build a Memory SSA representation for SAF that provides precise def-use chains for address-taken (heap/stack) variables. This is the critical infrastructure that unlocks sparse value-flow (SVFG), flow-sensitive PTA, and SABER-style memory safety checkers.

## Design Summary

### Approach: Location-Partitioned, Demand-Driven, Interprocedural

SAF's Memory SSA uses a **hybrid approach** combining the best of LLVM's MemorySSA (compact single-chain skeleton) and SVF's interprocedural Memory SSA (per-location precision via PTA):

1. **Skeleton construction** — Build a single def chain per function (like LLVM): every store/call gets a `Def`, every load gets a `Use`, join points get `Phi`. The chain is ordered by dominance.

2. **Demand-driven clobber walking** — When a query asks "which store feeds this load for location L?", walk the def chain backward, consulting PTA to disambiguate. Cache results.

3. **Interprocedural mod/ref summaries** — For each function, compute which locations it may modify (mod) and read (ref), bottom-up on the call graph. Calls act as clobbers for locations in the callee's mod set.

### Why This Is Better Than SVF

- **No annotation explosion**: SVF inserts χ/μ annotations at every store/load for every aliasing location. With N locations and M stores, that's O(N×M). SAF's skeleton is O(M) — one access per instruction.
- **Demand-driven**: Only locations actually queried get their chains disambiguated. SVF eagerly materializes all memory versions.
- **Same interprocedural precision**: Mod/ref summaries give the same precision as SVF's interprocedural χ/μ placement, but lazily.
- **Clean integration**: Builds on SAF's existing PTA, CFG, and call graph infrastructure.

---

## Module Layout

```
crates/saf-analysis/src/
  mssa/
    mod.rs          — Public API: MemorySsa, MemAccessId, build()
    access.rs       — MemoryAccess enum (Def, Use, Phi, LiveOnEntry)
    builder.rs      — Skeleton construction: walk functions, place accesses and phis
    walker.rs       — ClobberWalker: demand-driven disambiguation via PTA
    modref.rs       — Per-function mod/ref summaries, bottom-up on call graph
    export.rs       — JSON export for debugging/tutorials
```

## Core Types

```rust
/// Unique identifier for a memory access
pub struct MemAccessId(u128);  // BLAKE3-derived, deterministic

/// A memory access in the Memory SSA form
pub enum MemoryAccess {
    /// Sentinel: memory state at function entry
    LiveOnEntry {
        id: MemAccessId,
        function: FunctionId,
    },
    /// Store or call that may modify memory
    Def {
        id: MemAccessId,
        inst: InstId,
        block: BlockId,
        /// Previous def in the skeleton chain (dominance order)
        defining: MemAccessId,
    },
    /// Load or read-only operation
    Use {
        id: MemAccessId,
        inst: InstId,
        block: BlockId,
        /// Reaching def (skeleton; refined by clobber walker)
        defining: MemAccessId,
    },
    /// Join point merging memory versions from predecessors
    Phi {
        id: MemAccessId,
        block: BlockId,
        /// Predecessor block → reaching def from that path
        operands: BTreeMap<BlockId, MemAccessId>,
    },
}

/// Mod/Ref summary for a function
pub struct ModRefSummary {
    /// Locations this function may modify (directly or transitively)
    pub may_mod: BTreeSet<LocId>,
    /// Locations this function may read (directly or transitively)
    pub may_ref: BTreeSet<LocId>,
}

/// The complete Memory SSA for a module
pub struct MemorySsa {
    /// All memory accesses indexed by ID
    accesses: BTreeMap<MemAccessId, MemoryAccess>,
    /// Instruction → memory access mapping
    inst_to_access: BTreeMap<InstId, MemAccessId>,
    /// Block → phi accesses at block entry
    block_phis: BTreeMap<BlockId, Vec<MemAccessId>>,
    /// Function → LiveOnEntry sentinel
    live_on_entry: BTreeMap<FunctionId, MemAccessId>,
    /// Function → mod/ref summary
    mod_ref: BTreeMap<FunctionId, ModRefSummary>,
    /// Clobber cache: (use_access, location) → clobbering def
    clobber_cache: BTreeMap<(MemAccessId, LocId), MemAccessId>,
}
```

## Public API

```rust
impl MemorySsa {
    /// Build Memory SSA for a module
    pub fn build(
        module: &AirModule,
        cfgs: &BTreeMap<FunctionId, Cfg>,
        pta: &PtaResult,
        callgraph: &CallGraph,
    ) -> Self;

    /// Find the clobbering def for a Use at a specific location (demand-driven)
    pub fn clobber_for(&mut self, use_id: MemAccessId, loc: LocId) -> MemAccessId;

    /// Get the memory access for an instruction
    pub fn access_for(&self, inst: InstId) -> Option<&MemoryAccess>;

    /// Get all Phi accesses at block entry
    pub fn phis_at(&self, block: BlockId) -> &[MemAccessId];

    /// Get the LiveOnEntry sentinel for a function
    pub fn live_on_entry(&self, func: FunctionId) -> Option<MemAccessId>;

    /// Get mod/ref summary for a function
    pub fn mod_ref(&self, func: FunctionId) -> Option<&ModRefSummary>;

    /// Export to JSON
    pub fn export(&self) -> serde_json::Value;
}
```

## Algorithms

### Phase 2: Skeleton Construction

For each function:
1. Create `LiveOnEntry` sentinel
2. Walk blocks in dominance order (using CFG + DFS)
3. Within each block, walk instructions in order:
   - `Store` → create `Def`, chain to previous def
   - `CallDirect`/`CallIndirect` → create `Def` (may modify memory), chain to previous def
   - `Load` → create `Use`, point `defining` to current reaching def
   - Other → skip (no memory effect)
4. Track the "current def" at block exit for phi placement

### Phase 3: Phi Placement (Iterated Dominance Frontier)

Standard SSA phi-placement algorithm adapted for memory:
1. Compute dominance frontiers for the CFG
2. For each block that ends with a Def, insert Phi at its dominance frontier blocks
3. Iterate until no new Phis are needed (fixed point)
4. Rename: walk dominator tree, update `defining` pointers to use Phi results

### Phase 4: Clobber Walker

```
clobber_for(use_id, loc):
  if (use_id, loc) in clobber_cache:
    return clobber_cache[(use_id, loc)]

  current = accesses[use_id].defining
  result = walk_chain(current, loc)
  clobber_cache[(use_id, loc)] = result
  return result

walk_chain(access_id, loc):
  match accesses[access_id]:
    LiveOnEntry { .. } => return access_id
    Def { inst, defining, .. } =>
      if is_clobber(inst, loc):
        return access_id
      else:
        return walk_chain(defining, loc)
    Phi { operands, .. } =>
      // Check if all predecessors resolve to the same clobber
      results: BTreeSet = operands.values()
        .map(|pred| walk_chain(pred, loc))
        .collect()
      if results.len() == 1:
        return results.first()
      else:
        return access_id  // Phi itself is the "clobber" (multiple reaching defs)

is_clobber(inst, loc):
  match inst.op:
    Store { operands: [_, ptr] } =>
      pta.points_to(ptr).contains(loc)
    CallDirect { callee } =>
      mod_ref[callee].may_mod.contains(loc)
    CallIndirect { .. } =>
      true  // Conservative
    _ => false
```

### Phase 5: Mod/Ref Summaries

Bottom-up on call graph (reverse topological order):
1. For each function, scan instructions:
   - `Store { operands: [_, ptr] }` → `may_mod.extend(pta.points_to(ptr))`
   - `Load { operands: [ptr] }` → `may_ref.extend(pta.points_to(ptr))`
   - `CallDirect { callee }` → union callee's mod/ref into this function's
2. For SCCs (recursive functions): iterate to fixed point within the SCC

---

## E2E Test Programs

All compiled from source via clang-18 / rustc → LLVM IR → LLVM frontend → AIR → Memory SSA.

### 1. `mssa_store_load_simple.c` — Basic disambiguation

```c
#include <stdlib.h>
extern void sink(int);
extern int source(void);

void test(void) {
    int a, b;
    int *p = &a;
    int *q = &b;
    *p = source();  // S1: store to a
    *q = 99;        // S2: store to b
    int x = *p;     // L1: load from a — clobber should be S1, not S2
    sink(x);
}
int main(void) { test(); return 0; }
```

**Asserts**: `clobber_for(L1, loc_a) == S1` (S2 skipped because q→{loc_b} doesn't alias p→{loc_a})

### 2. `mssa_phi_merge.c` — Control flow join

```c
#include <stdlib.h>
extern void sink(int);

void test(int *p, int cond) {
    if (cond) {
        *p = 1;   // S1
    } else {
        *p = 2;   // S2
    }
    int x = *p;   // L1: should see Phi(S1, S2)
    sink(x);
}
int main(void) { int v; test(&v, 1); return 0; }
```

**Asserts**: `clobber_for(L1, loc_p)` returns a Phi access (both S1 and S2 reach)

### 3. `mssa_interproc.c` — Interprocedural mod/ref

```c
#include <stdlib.h>
extern void sink(int);
extern int source(void);

void modify(int *p) {
    *p = 100;  // Modifies *p
}

void test(void) {
    int a;
    int *p = &a;
    *p = source();   // S1
    modify(p);       // Call: mod_ref says modify() modifies loc_a
    int x = *p;      // L1: clobber should be the call, not S1
    sink(x);
}
int main(void) { test(); return 0; }
```

**Asserts**: `mod_ref(modify).may_mod` contains `loc_a`; `clobber_for(L1, loc_a)` is the call Def, not S1

### 4. `mssa_field_sensitive.cpp` — C++ struct field disambiguation

```cpp
#include <cstdlib>
extern "C" void sink(int);
extern "C" int source();

struct Pair {
    int a;
    int b;
};

void test() {
    Pair s;
    s.a = source();  // S1: GEP field 0 + store
    s.b = 20;        // S2: GEP field 1 + store
    int x = s.a;     // L1: GEP field 0 + load — clobber should be S1
    sink(x);
}
int main() { test(); return 0; }
```

**Asserts**: With field-sensitive PTA, `clobber_for(L1, loc_s_field0) == S1` (S2 is to field 1, different location)

### 5. `mssa_loop.c` — Loop memory Phi

```c
#include <stdlib.h>
extern void sink(int);

void test(int n) {
    int acc;
    int *p = &acc;
    *p = 0;                 // S1: initial
    for (int i = 0; i < n; i++) {
        int x = *p;         // L1: Phi(S1, S2) — loop header
        *p = x + 1;         // S2: loop body
    }
    int result = *p;        // L2: Phi(S1, S2) — loop exit
    sink(result);
}
int main(void) { test(10); return 0; }
```

**Asserts**: `clobber_for(L1, loc_acc)` and `clobber_for(L2, loc_acc)` return Phi accesses (both S1 and S2 can reach)

### 6. `mssa_rust_unsafe.rs` — Rust unsafe pointer operations

```rust
extern "C" {
    fn source() -> i32;
    fn sink(x: i32);
}

unsafe fn test() {
    let mut a: i32 = 0;
    let mut b: i32 = 0;
    let p: *mut i32 = &mut a;
    let q: *mut i32 = &mut b;
    *p = source();   // S1
    *q = 99;         // S2
    let x = *p;      // L1: clobber should be S1
    sink(x);
}

fn main() {
    unsafe { test(); }
}
```

**Asserts**: Same as test 1 but compiled from Rust — validates Memory SSA with Rust-generated LLVM IR

---

## Python Bindings

### New types in `saf-python`

```python
class MemorySsa:
    """Memory SSA result for a module."""
    def clobber_for(self, use_inst: str, location: str) -> dict:
        """Find clobbering def for a load at a location.
        Returns dict with 'kind' (def/phi/live_on_entry), 'inst' (if def), 'block'."""

    def access_for(self, inst_id: str) -> dict:
        """Get memory access for an instruction."""

    def mod_ref(self, function: str) -> dict:
        """Get mod/ref summary: {'may_mod': [...], 'may_ref': [...]}"""

    def export(self) -> dict:
        """Export full Memory SSA as JSON."""

# On Project:
class Project:
    def memory_ssa(self) -> MemorySsa:
        """Build Memory SSA for the loaded module."""
```

### Python E2E tests

One test per E2E program, verifying:
- Memory SSA builds without error
- Export produces valid JSON with expected structure
- Clobber queries return expected results (where deterministic)
- Mod/ref summaries contain expected locations

---

## Tutorial

**Location**: `tutorials/pta/06-memory-ssa/`

Contents:
- `vulnerable.c` — Program with aliased stores/loads where Memory SSA disambiguates
- `detect.py` — Python script demonstrating Memory SSA API
- `detect.rs` — Rust equivalent
- `README.md` — Explanation of Memory SSA concepts and API usage

---

## Implementation Phases

### Phase 1: Core types
- [ ] `mssa/access.rs`: `MemoryAccess` enum, `MemAccessId` newtype
- [ ] `mssa/mod.rs`: `MemorySsa` struct, public API signatures (stubs)
- [ ] Tests: type compilation, basic construction

### Phase 2: Skeleton builder
- [ ] `mssa/builder.rs`: Walk functions in dominance order, create Def/Use/LiveOnEntry
- [ ] Chain Defs in instruction order within blocks
- [ ] Track current reaching def per block exit
- [ ] Tests: correct Def/Use/LiveOnEntry placement for simple functions

### Phase 3: Phi placement
- [ ] Compute dominance frontiers from CFG
- [ ] Insert Phi accesses at dominance frontier blocks
- [ ] SSA-style renaming: walk dominator tree, update `defining` pointers
- [ ] Tests: Phi at if/else join, loop header, multi-predecessor blocks

### Phase 4: Clobber walker
- [ ] `mssa/walker.rs`: `clobber_for()` implementation
- [ ] PTA consultation via `is_clobber()`
- [ ] Clobber cache
- [ ] Phi handling: check if all paths resolve to same clobber
- [ ] Tests: disambiguation with aliasing/non-aliasing PTA results

### Phase 5: Mod/ref summaries
- [ ] `mssa/modref.rs`: Bottom-up on call graph
- [ ] Transitive inclusion of callee effects
- [ ] SCC fixed-point for recursive functions
- [ ] Tests: direct mod/ref, transitive through calls, recursive SCC

### Phase 6: Interprocedural clobber
- [ ] Extend `is_clobber()` to consult mod/ref for CallDirect
- [ ] Conservative handling for CallIndirect
- [ ] Tests: call as clobber when callee modifies location

### Phase 7: E2E tests
- [ ] Compile 6 source programs (4 C, 1 C++, 1 Rust) to LLVM IR in Docker
- [ ] Write 6+ Rust E2E tests in `crates/saf-analysis/tests/mssa_e2e.rs`
- [ ] Write 6+ Python E2E tests in `python/tests/test_mssa_e2e.py`
- [ ] Verify all tests pass in Docker

### Phase 8: Python bindings
- [ ] `saf-python/src/mssa.rs`: `PyMemorySsa` wrapper
- [ ] `Project.memory_ssa()` method
- [ ] Python E2E tests

### Phase 9: Tutorial
- [ ] `tutorials/pta/06-memory-ssa/vulnerable.c`
- [ ] `tutorials/pta/06-memory-ssa/detect.py`
- [ ] `tutorials/pta/06-memory-ssa/detect.rs`
- [ ] `tutorials/pta/06-memory-ssa/README.md`
- [ ] Verify tutorial end-to-end in Docker

### Phase 10: Documentation updates
- [ ] Update `docs/tool-comparison.md`: Mark Memory SSA as implemented (no longer gapped)
- [ ] Update `plans/PROGRESS.md`: E11 status, session log
- [ ] Update `plans/FUTURE.md`: Move Memory SSA from deferred to done, add SVFG/flow-sens PTA extension points

---

## Success Criteria

1. `MemorySsa::build()` completes for all 6 E2E test programs
2. Clobber walker correctly disambiguates store→load for non-aliasing pointers
3. Phi nodes placed correctly at control flow joins and loop headers
4. Mod/ref summaries correctly capture transitive call effects
5. Interprocedural clobber: calls recognized as clobbers when callee modifies queried location
6. All Rust and Python E2E tests pass in Docker
7. Tutorial runs end-to-end in Docker
8. Deterministic: identical inputs produce byte-identical Memory SSA output

## Sources

- [LLVM MemorySSA Documentation](https://llvm.org/docs/MemorySSA.html)
- [SVF: Interprocedural Static Value-Flow Analysis in LLVM (CC'16)](https://dl.acm.org/doi/10.1145/2892208.2892235)
- [SILVA: Scalable Incremental Layered Sparse Value-Flow Analysis (TOSEM'25)](https://dl.acm.org/doi/10.1145/3725214)
- [SVF GitHub](https://github.com/SVF-tools/SVF)
- [SVF Technical Documentation](https://github.com/svf-tools/SVF/wiki/Technical-documentation)
