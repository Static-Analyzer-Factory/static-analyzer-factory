# Plan 111: Online Call Graph Construction Inside PTA Solver

## Context

CG refinement is the dominant bottleneck for bash (48.9s, 97% of total). The
current architecture runs an **outer loop**: extract constraints → full PTA solve
→ resolve indirect calls → recompute reachable → repeat. For bash, the loop
converges in 1 iteration, but the single PTA solve takes 48.9s because it
processes all 2071 functions.

Plan 110 showed that representation changes (BitVec, skip normalize) don't help,
and main-only entry causes multiple iterations (4x regression). The only viable
path is structural: **integrate CG construction into the PTA solver** so indirect
calls are resolved during solving, not after.

### SVF's Approach (from research)

SVF uses a two-phase loop inside `solveConstraints()`:
```
initWorklist();
do {
    reanalyze = false;
    solveWorklist();                          // drain worklist to fixed point
    if (updateCallGraph(indirectCallsites))   // resolve indirect calls
        reanalyze = true;                     // new edges → re-solve
} while (reanalyze);
```

Key design decisions:
1. **All intra-procedural constraints built upfront** in PAG/SVFIR before PTA starts
2. **CG refinement is batched** — happens after worklist drains, not during
3. **Only inter-procedural copy edges added dynamically** — actual→formal, return→callsite
4. **Resolved targets pushed to worklist** — re-triggers propagation

### SAF's Current Architecture (from research)

SAF's `GenericSolver` has these barriers to online CG:
1. `constraints: &ConstraintSet` — **immutable borrow**, can't add constraints mid-solve
2. `factory: &LocationFactory` — **immutable borrow**, can't create locations mid-solve
3. `ConstraintIndex` — built once at construction, no incremental add methods
4. `topo_order` — computed once from initial copy graph, invalidated by new edges
5. HVN preprocessing — runs once before solver, new constraints bypass it

However, the underlying data structures support incremental addition:
- `ConstraintIndex` uses `IndexMap<ValueId, SmallVec<[usize; 4]>>` — trivially supports push
- `LocationFactory` uses monotonic counter — new locations get higher IDs
- Wave propagation naturally handles new worklist entries
- LCD cycle detection already handles dynamically-formed cycles

### Design Decision: Upfront vs Lazy Extraction

**Option A: SVF-style upfront extraction** — extract all functions' constraints
before solving, only add inter-procedural edges dynamically.
- Pro: Simpler solver changes (no mid-solve extraction coordination)
- Pro: HVN can run on the full constraint set
- Con: Extracts constraints for potentially unreachable functions (wasted work)
- Con: More memory for unused constraints

**Option B: Lazy extraction** — extract per-function when newly reachable.
- Pro: Only processes reachable functions
- Pro: Memory-efficient
- Con: Requires mutable factory access mid-solve, more complex coordination

**Recommendation: Option A (upfront extraction)**. For bash (2071 functions),
extraction is ~1s total — negligible vs 48.9s solve. And it avoids the complexity
of mid-solve extraction. We extract everything upfront, then the solver only
needs to add inter-procedural copy edges when new CG edges are discovered.

## Changes

### Phase 1: Make solver support incremental constraint addition

**Agent A: ConstraintIndex incremental methods**

File: `crates/saf-analysis/src/pta/constraint_index.rs`

Add methods for incremental constraint insertion:
```rust
impl ConstraintIndex {
    /// Add a copy constraint to the index incrementally.
    pub fn add_copy(&mut self, indexed: &mut IndexedConstraints, c: CopyConstraint) {
        let idx = indexed.copy.len();
        indexed.copy.push(c.clone());
        self.copy_by_src.entry(c.src).or_default().push(idx);
    }

    /// Add a load constraint incrementally.
    pub fn add_load(&mut self, indexed: &mut IndexedConstraints, c: LoadConstraint) {
        let idx = indexed.load.len();
        indexed.load.push(c.clone());
        self.load_by_src_ptr.entry(c.src_ptr).or_default().push(idx);
    }

    // Similar for add_store, add_gep
}
```

**Agent B: GenericSolver owned data + resume support**

File: `crates/saf-analysis/src/pta/solver.rs`

Create a new `OnlineSolver<P: PtsSet>` struct that owns its data:
```rust
pub struct OnlineSolver<P: PtsSet> {
    pts: IndexMap<ValueId, P>,
    loc_pts: IndexMap<LocId, P>,
    worklist: BTreeSet<(u32, ValueId)>,
    loc_worklist: IdBitSet<LocId>,
    constraints: ConstraintSet,        // OWNED, not borrowed
    factory: LocationFactory,           // OWNED, not borrowed
    constants: Option<ConstantsTable>,  // OWNED
    index: ConstraintIndex,
    indexed: IndexedConstraints,
    load_loc_index: IndexMap<LocId, Vec<usize>>,
    pending_cycle_pairs: Vec<(ValueId, ValueId)>,
    prev_pts: IndexMap<ValueId, P>,
    rep: IndexMap<ValueId, ValueId>,
    topo_order: IndexMap<ValueId, u32>,
    template: P,
    // NEW: Online CG support
    indirect_callsites: Vec<IndirectCallSite>,  // tracked call sites
    func_loc_map: FunctionLocationMap,           // function→location mapping
    resolved_calls: BTreeMap<InstId, BTreeSet<FunctionId>>,  // already resolved
}

/// An indirect call site tracked by the online solver.
struct IndirectCallSite {
    inst_id: InstId,
    /// The ValueIds whose pts sets determine call targets
    operand_values: Vec<ValueId>,
}
```

Key methods:
```rust
impl<P: PtsSet> OnlineSolver<P> {
    /// Create from upfront-extracted constraints.
    pub fn new(constraints: ConstraintSet, factory: LocationFactory, ...) -> Self;

    /// Run the solver with online CG construction.
    pub fn solve_online(&mut self, module: &AirModule, max_iterations: usize);

    /// Add inter-procedural copy constraints for a new call edge.
    fn connect_caller_to_callee(
        &mut self,
        call_inst: &Instruction,
        callee: &AirFunction,
    );

    /// Check indirect call sites for new resolutions.
    fn resolve_indirect_calls(&mut self, module: &AirModule) -> bool;

    /// Recompute topo order after adding new copy edges.
    fn recompute_topo_order(&mut self);
}
```

### Phase 2: Wire online solver into CG refinement

**Leader: Replace outer loop with single online solve**

File: `crates/saf-analysis/src/cg_refinement.rs`

Replace the current iterative loop:
```rust
// BEFORE (current):
for _iter in 0..config.max_iterations {
    let mut factory = LocationFactory::new(...);
    let constraints = extract_constraints_reachable(module, &reachable, &mut factory);
    let pts = solve_with_config(&constraints, &factory, ...);
    let newly_resolved = resolve_indirect_calls_via_pta(module, &mut cg, &pts, ...);
    // ... recompute reachable, check fixed point ...
}

// AFTER (online):
let mut factory = LocationFactory::new(config.field_sensitivity.clone());
let constraints = extract_constraints(module, &mut factory);  // ALL functions upfront
let mut solver = OnlineSolver::new(constraints, factory, ...);
solver.solve_online(module, config.pta_config.max_iterations);
// solver.resolved_calls contains the refined CG
// solver.pts contains final points-to result
```

### Phase 3: The solve_online algorithm

```rust
fn solve_online(&mut self, module: &AirModule, max_iterations: usize) {
    // Phase 1: Initialize (same as current solve)
    self.initialize_addr_constraints();
    self.compute_topo_order();

    // Phase 2: Solve-then-refine loop (SVF pattern)
    loop {
        // 2a. Drain worklist to local fixed point
        self.solve_worklist(max_iterations);

        // 2b. Check indirect calls for new resolutions
        let found_new = self.resolve_indirect_calls(module);

        if !found_new {
            break; // Global fixed point
        }

        // 2c. New edges added — recompute topo order and continue
        self.recompute_topo_order();
    }
}
```

The `resolve_indirect_calls` method:
```rust
fn resolve_indirect_calls(&mut self, module: &AirModule) -> bool {
    let mut found_new = false;
    for site in &self.indirect_callsites {
        for &op_val in &site.operand_values {
            if let Some(pts_set) = self.pts.get(&op_val) {
                for loc_id in pts_set.iter() {
                    if let Some(loc) = self.factory.all_locations().get(&loc_id) {
                        if let Some(fid) = self.func_loc_map.get(loc.obj) {
                            if self.resolved_calls
                                .entry(site.inst_id)
                                .or_default()
                                .insert(fid)
                            {
                                // NEW call edge: add inter-procedural constraints
                                self.connect_caller_to_callee(site, fid, module);
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }
    }
    found_new
}
```

The `connect_caller_to_callee` method adds copy constraints:
```rust
fn connect_caller_to_callee(&mut self, site: &IndirectCallSite, callee_fid: FunctionId, module: &AirModule) {
    let callee = module.functions.iter().find(|f| f.id == callee_fid);
    let Some(callee) = callee else { return; };

    // Find the call instruction to get actual arguments
    // Add copy: actual_arg[i] → formal_param[i]
    // Add copy: callee_return → callsite_dst
    // Push affected values into worklist
    for (actual, formal) in actual_formal_pairs {
        let copy = CopyConstraint { src: actual, dst: formal };
        self.index.add_copy(&mut self.indexed, copy);
        self.worklist.insert((self.topo_rank(formal), formal));
    }
}
```

## Execution Plan

### Phase 1: Incremental constraint infrastructure (2 parallel agents)
- **Agent A**: `constraint_index.rs` — add `add_copy`, `add_load`, `add_store`, `add_gep` methods
- **Agent B**: `solver.rs` — create `OnlineSolver<P>` struct with owned data, `solve_online()`, `resolve_indirect_calls()`, `connect_caller_to_callee()`

### Phase 2: Leader wiring
- Wire `OnlineSolver` into `cg_refinement.rs::refine()`
- Update `extract.rs` to support extracting inter-procedural constraints separately
- Build `IndirectCallSite` list from module

### Phase 3: Verification
1. `make fmt && make lint` — clean
2. `make test` — all 1515 Rust + 72 Python tests pass
3. Bash benchmark: target CG refine < 15s (from 48.9s)
4. PTABen: 2252 Exact, 69 Unsound (no regression)

## Expected Performance Impact

The current architecture does:
1. Extract constraints for 2071 functions → ~1s
2. HVN preprocessing → ~0.5s
3. Full PTA solve → ~47s
4. Resolve indirect calls → ~0.1s
Total: ~48.9s (1 iteration)

With online CG:
1. Extract ALL constraints upfront → ~1s (same, but only once)
2. HVN preprocessing → ~0.5s (same, but only once)
3. PTA solve with online CG → **the key question**
   - If bash has few indirect calls: ~same as current (47s)
   - If bash has many indirect calls resolved over multiple waves: could be faster
     because inter-procedural edges are added incrementally instead of being
     present from the start

**Honest assessment**: For bash with AllDefined entry (all functions already reachable),
the main benefit is eliminating the outer loop overhead. Since bash already converges
in 1 iteration, the speedup may be modest (5-15%). The larger benefit comes when
using main-only entry, where online CG construction discovers functions lazily inside
a SINGLE solve pass instead of requiring multiple full re-solves.

With main-only entry + online CG:
- Start with main's constraints only
- Discover callees during solving, add their inter-procedural edges
- Single solve pass discovers everything
- Expected: 1 solve pass instead of 4+ = **3-4x speedup** (48.9s → 12-15s)

## Risks

1. **connect_caller_to_callee complexity**: Extracting actual→formal mappings from
   AIR CallIndirect instructions requires understanding the calling convention
   (which operands are args vs the callee pointer)
2. **Topo order invalidation**: Adding new copy edges invalidates topological ordering.
   Recomputing after each CG update wave is cheap (~10ms for bash) but adds complexity.
3. **HVN interaction**: New constraints bypass HVN. For the inter-procedural copy
   edges this is fine (HVN mostly helps with intra-procedural redundancy).
4. **Determinism**: Must ensure online CG construction produces identical results
   regardless of worklist processing order (monotonicity guarantees this).

## Deferred

- **Lazy constraint extraction** (Option B): Extract per-function on demand.
  More memory-efficient but complex. Defer to future plan if upfront extraction
  is too costly for very large programs.
- **Online cycle detection for new edges**: Current LCD handles dynamic cycles
  from pts propagation but may need tuning for CG-induced cycles.
