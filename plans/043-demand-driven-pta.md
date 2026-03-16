# Plan 043: Demand-Driven Pointer Analysis (DDA)

**Epic**: E24 — Demand-Driven PTA
**Status**: approved
**Created**: 2026-01-31

## Overview

Implement demand-driven pointer analysis (DDA) using CFL-reachability for context-sensitive backward traversal on SVFG. DDA computes points-to information only for explicitly queried pointers, enabling scalable analysis of large codebases.

### References

- [SVF SUPA (TSE'18)](https://yuleisui.github.io/publications/tse18.pdf) — Value-flow-based demand-driven PTA with strong updates
- [Heintze & Tardieu (PLDI'01)](https://dl.acm.org/doi/10.1145/378795.378802) — First on-demand Andersen-style for C
- [Zheng & Rugina (POPL'08)](https://www.cs.cornell.edu/~xinz/papers/alias-popl08.pdf) — CFL-reachability alias analysis
- SVF source: `svf/include/DDA/` (DDAClient, DDAVFSolver, FlowDDA, ContextDDA)

## Design Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| Query model | Points-to + alias + refined reachability | All three tiers; points-to is foundation |
| Algorithm | CFL-reachability | Matched call/return parentheses for context sensitivity |
| Budget | Steps/time/depth with CI fallback | Sound over-approximation when budget exhausted |
| Caching | Persistent cross-query cache | Amortized cost for batch analysis |
| API | Standalone `DdaPta` type | Clean separation from whole-program analyses |
| Python | Full API with diagnostics | Matches SAF's Python-first philosophy |
| Tutorials | Deferred to FUTURE.md | Per user instruction |

## Dependencies

All dependencies are implemented:
- SVFG (E12) — backward traversal graph
- MemorySsa (E11) — clobber resolution for indirect edges
- PtaResult (E4) — CI fallback
- CallGraph (E3) — SCC detection for recursion
- AirModule (E1) — instruction inspection

## Core Data Structures

### CallString (CFL Context)

```rust
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CallString {
    sites: Vec<InstId>,  // Stack of call-site IDs (most recent last)
}

impl CallString {
    pub fn empty() -> Self;
    pub fn push(&self, call_site: InstId) -> Self;      // Enter callee: push ⟨i
    pub fn pop(&self) -> Option<(Self, InstId)>;        // Exit callee: pop ⟩i
    pub fn matches(&self, return_site: InstId) -> bool; // Check top matches
    pub fn is_empty(&self) -> bool;
    pub fn depth(&self) -> usize;
}
```

### Dpm (Demand-Driven Points-To Message)

```rust
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Dpm {
    pub node: SvfgNodeId,    // Current SVFG node being traced
    pub context: CallString, // Calling context for CFL matching
}
```

### DdaCache

```rust
pub struct DdaCache {
    /// Top-level pointer cache: (value, context) → points-to set
    tl_cache: BTreeMap<Dpm, BTreeSet<LocId>>,

    /// Address-taken object cache: (mem_access, context) → points-to set
    at_cache: BTreeMap<(MemAccessId, CallString), BTreeSet<LocId>>,

    /// Visited set for cycle detection (per-query)
    visited: BTreeSet<Dpm>,
}
```

### DdaConfig and Budget

```rust
pub struct DdaConfig {
    pub max_steps: usize,           // 0 = unlimited
    pub max_context_depth: usize,
    pub timeout_ms: u64,            // 0 = unlimited
    pub enable_strong_updates: bool,
}

pub struct Budget {
    steps_remaining: usize,
    start_time: Instant,
    timeout_ms: u64,
    exhausted_reason: Option<ExhaustionReason>,
}

pub enum ExhaustionReason {
    StepsExceeded,
    TimeoutExceeded,
    ContextDepthExceeded,
}
```

## Algorithm

### CFL Grammar

```
# Context-sensitive matching (Dyck language)
ContextPath → ε                           # Empty (same function)
            | ⟨i ContextPath ⟩i           # Matched call-return pair
            | ContextPath ContextPath      # Concatenation
```

### Backward Traversal

```rust
fn find_points_to(&mut self, query: ValueId, budget: &mut Budget) -> BTreeSet<LocId> {
    let mut worklist: VecDeque<Dpm> = VecDeque::new();
    let mut result: BTreeSet<LocId> = BTreeSet::new();

    // Seed: start from query node
    worklist.push_back(Dpm::new(query, CallString::empty()));

    while let Some(dpm) = worklist.pop_front() {
        if budget.exhausted() {
            return self.fallback_to_ci(query);
        }

        if self.cache.mark_visited(&dpm) {
            continue;  // Already visited in this query
        }

        if let Some(cached) = self.cache.get(&dpm) {
            result.extend(cached);
            continue;
        }

        match self.classify_node(dpm.node) {
            NodeKind::Addr(loc) => { result.insert(loc); }
            NodeKind::Direct => self.propagate_backward_direct(&mut worklist, &dpm),
            NodeKind::CallArg => self.handle_call_entry(&mut worklist, &dpm),
            NodeKind::Return => self.handle_return_exit(&mut worklist, &dpm),
            NodeKind::Load => self.handle_load(&mut worklist, &dpm, budget),
            NodeKind::MemPhi => self.handle_phi(&mut worklist, &dpm),
        }

        budget.tick();
    }

    self.cache.insert_tl(Dpm::new(query, CallString::empty()), result.clone());
    result
}
```

### Call/Return Matching

- **At call edge (entering callee)**: `context.push(call_site_id)`
- **At return edge (exiting callee)**: Check `context.matches(return_site)`, then `context.pop()`
- **Mismatched return**: Skip edge (spurious interprocedural path)
- **Recursion**: Detect via CallGraph SCC; collapse recursive call sites

### Strong Update Conditions

```rust
fn can_strong_update(&self, loc: LocId, all_locs: &BTreeSet<LocId>) -> bool {
    all_locs.len() == 1                    // Singleton target
        && !self.is_array_location(loc)    // Not array element
        && !self.is_in_recursive_function(loc)  // Not in recursive function
        && !self.has_escaped(loc)          // Not escaped
}
```

## Query API

```rust
pub struct DdaPta<'a> {
    svfg: &'a Svfg,
    mssa: &'a MemorySsa,
    ci_pta: &'a PtaResult,
    cache: DdaCache,
    config: DdaConfig,
    diagnostics: DdaDiagnostics,
}

impl<'a> DdaPta<'a> {
    pub fn new(svfg: &'a Svfg, mssa: &'a MemorySsa, ci_pta: &'a PtaResult, config: DdaConfig) -> Self;

    // Primary queries
    pub fn points_to(&mut self, ptr: ValueId) -> Vec<LocId>;
    pub fn may_alias(&mut self, p: ValueId, q: ValueId) -> AliasResult;
    pub fn reachable(&mut self, src: ValueId, sink: ValueId) -> bool;
    pub fn reachable_refined(&mut self, src: ValueId, sink: ValueId) -> ReachabilityResult;

    // Introspection
    pub fn cache_stats(&self) -> CacheStats;
    pub fn diagnostics(&self) -> &DdaDiagnostics;
    pub fn export(&self) -> DdaExport;
}
```

## Python API

```python
# Creation
dda = project.demand_pta(
    max_steps=100000,
    max_context_depth=10,
    timeout_ms=5000,
    enable_strong_updates=True
)

# Queries
locs = dda.points_to(ptr_id)              # -> List[str]
alias = dda.may_alias(p_id, q_id)         # -> "may" | "no" | "unknown"
reaches = dda.reachable(src_id, sink_id)  # -> bool
result = dda.reachable_refined(src, sink) # -> ReachabilityResult

# Introspection
stats = dda.cache_stats()   # -> dict
diag = dda.diagnostics()    # -> dict
export = dda.export()       # -> dict
```

## Implementation Phases

### Phase 1: Core Data Structures
- [ ] `CallString` type with push/pop/matches operations
- [ ] `Dpm` (demand-driven points-to message) type
- [ ] `DdaConfig` and `Budget` types
- [ ] `DdaCache` with two-level storage
- [ ] Unit tests for all types
- **Files**: `crates/saf-analysis/src/dda/types.rs`

### Phase 2: Backward Traversal Engine
- [ ] `DdaPta` struct with SVFG/MSSA/CI-PTA references
- [ ] `find_points_to()` core algorithm with worklist
- [ ] Direct edge propagation (DirectDef, DirectTransform, Copy, Phi)
- [ ] Call/return CFL matching (push/pop context)
- [ ] Cycle detection via visited set
- [ ] Unit tests for traversal
- **Files**: `crates/saf-analysis/src/dda/solver.rs`

### Phase 3: Indirect Edge Handling
- [ ] `handle_load()` with recursive points-to query
- [ ] MSSA clobber integration for store→load resolution
- [ ] Strong update conditions (`can_strong_update()`)
- [ ] MemPhi handling for control-flow merges
- [ ] Unit tests for indirect edges
- **Files**: Updates to `solver.rs`

### Phase 4: Budget and Fallback
- [ ] Budget exhaustion detection (steps, time, depth)
- [ ] CI-PTA fallback mechanism
- [ ] `DdaDiagnostics` for query tracking
- [ ] Cache statistics collection
- [ ] Unit tests for budget behavior
- **Files**: `crates/saf-analysis/src/dda/budget.rs`

### Phase 5: Query API and Export
- [ ] `points_to()`, `may_alias()`, `reachable()` public methods
- [ ] `reachable_refined()` combining SVFG + DDA precision
- [ ] JSON export for cache and diagnostics
- [ ] Integration with `AnalysisError` types
- **Files**: `crates/saf-analysis/src/dda/mod.rs`, `export.rs`

### Phase 6: E2E Tests
- [ ] 6 C/C++ test programs:
  - `dda_basic_query.c` — single function pointer query
  - `dda_context_sensitive.c` — wrapper function disambiguation
  - `dda_strong_update.c` — singleton store precision
  - `dda_budget_fallback.c` — complex program triggering fallback
  - `dda_cache_reuse.c` — multiple related queries
  - `dda_recursion.cpp` — recursive function handling
- [ ] Compile to LLVM IR
- [ ] Rust E2E tests
- **Files**: `crates/saf-analysis/tests/dda_e2e.rs`, `tests/fixtures/dda/`

### Phase 7: Python Bindings
- [ ] `PyDdaPta` class with all query methods
- [ ] `Project.demand_pta()` factory method
- [ ] Python E2E tests
- **Files**: `crates/saf-python/src/dda.rs`, `python/tests/test_dda.py`

### Phase 8: Documentation
- [ ] Update `docs/tool-comparison.md` (mark DDA as implemented)
- [ ] Add tutorial entries to `plans/FUTURE.md`
- [ ] Update `plans/PROGRESS.md`

## Test Programs

| Program | Tests |
|---------|-------|
| `dda_basic_query.c` | Single-function pointer, verify DDA matches CI |
| `dda_context_sensitive.c` | Wrapper functions, DDA more precise than CI |
| `dda_strong_update.c` | Singleton store kills previous, DDA filters spurious |
| `dda_budget_fallback.c` | Deep recursion/loops, verify graceful fallback |
| `dda_cache_reuse.c` | Query p then q (aliasing), second query faster |
| `dda_recursion.cpp` | Recursive linked list, SCC collapse works |

## Success Criteria

1. All 8 phases complete with passing tests
2. DDA produces same or more precise results than CI-PTA
3. Budget fallback is sound (never less precise than CI)
4. Cache amortization measurable (second query faster)
5. Python API fully functional with diagnostics
6. Documentation updated

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| CFL matching complexity | Start with simple call-string, upgrade if needed |
| Performance on large programs | Budget mechanism ensures bounded cost |
| Indirect edge explosion | Strong updates + MSSA clobber filtering |
| Cache memory growth | Consider LRU eviction in future if needed |
