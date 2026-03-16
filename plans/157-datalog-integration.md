# Plan 157: Datalog Integration via Ascent

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace SAF's PTA worklist solver with Ascent Datalog (lattice-based, BYODS union-find, offline SCC, parallel), migrate 9 SVFG checkers to Ascent rules, and add a unified query protocol. Old solver preserved behind `legacy-pta` feature flag.

**Architecture:** New `saf-datalog` crate contains all Ascent programs. Fact extraction is a single function (fixing the 3-entry-point sync bug). PTA outputs the same `PointsToMap` type. Checkers produce the same `CheckerFinding` type. SVFG builder stays as Rust. The `legacy-pta` feature flag on `saf-analysis` keeps the old solver available.

**Tech Stack:** Ascent 0.8+ (proc macro), ascent-byods-rels (union-find), Rayon (via ascent_par!)

**Design Doc:** `docs/plans/2026-02-23-datalog-integration-design.md`

---

## Task 1: Create saf-datalog Crate Skeleton

**Files:**
- Create: `crates/saf-datalog/Cargo.toml`
- Create: `crates/saf-datalog/src/lib.rs`
- Modify: `Cargo.toml` (workspace root — add member)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "saf-datalog"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
saf-core = { path = "../saf-core" }
ascent = "0.8"
ascent-byods-rels = "0.1"
tracing = { workspace = true }
rustc-hash = { workspace = true }

[dev-dependencies]
saf-test-utils = { path = "../saf-test-utils" }

[features]
default = []
parallel = ["ascent/par"]
```

**Step 2: Create src/lib.rs**

```rust
//! Datalog-based analysis engine for SAF.
//!
//! Uses Ascent (compile-time Datalog) for:
//! - Pointer analysis (Andersen's with lattice-based fixpoint)
//! - SVFG checker rules (reachability queries)
//!
//! Phase 2 (deferred): runtime Datalog interpreter for user-authored rules.

pub mod facts;
pub mod pta;
pub mod checkers;
```

**Step 3: Create stub modules**

- `crates/saf-datalog/src/facts.rs` — empty with doc comment
- `crates/saf-datalog/src/pta/mod.rs` — empty with doc comment
- `crates/saf-datalog/src/checkers/mod.rs` — empty with doc comment

**Step 4: Add to workspace**

In root `Cargo.toml`, add `"crates/saf-datalog"` to `workspace.members`.

**Step 5: Verify it compiles**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-datalog'`
Expected: Compiles with no errors (empty crate with ascent dependency).

**Step 6: Commit**

```bash
git add crates/saf-datalog/ Cargo.toml
git commit -m "feat: add saf-datalog crate skeleton with Ascent dependency"
```

---

## Task 2: Implement PointsToSet Lattice for Ascent

Ascent lattices require implementing the `ascent::Lattice` trait. SAF needs a `AscentPtsSet` that wraps `BTreeSet<LocId>` with set-union as the join operation.

**Files:**
- Create: `crates/saf-datalog/src/pta/pts_lattice.rs`
- Modify: `crates/saf-datalog/src/pta/mod.rs`
- Create: `crates/saf-datalog/tests/pts_lattice_test.rs`

**Step 1: Write the failing test**

```rust
// crates/saf-datalog/tests/pts_lattice_test.rs
use saf_datalog::pta::AscentPtsSet;
use saf_core::id::make_id;

#[test]
fn test_singleton_and_union() {
    let loc_a = make_id("loc_a");
    let loc_b = make_id("loc_b");

    let a = AscentPtsSet::singleton(loc_a);
    let b = AscentPtsSet::singleton(loc_b);
    let merged = a.join(b);

    assert!(merged.contains(loc_a));
    assert!(merged.contains(loc_b));
    assert_eq!(merged.len(), 2);
}

#[test]
fn test_lattice_partial_ord() {
    let loc_a = make_id("loc_a");
    let loc_b = make_id("loc_b");

    let a = AscentPtsSet::singleton(loc_a);
    let ab = AscentPtsSet::from_iter([loc_a, loc_b]);

    // a <= ab (subset)
    assert!(a <= ab);
    // ab is not <= a
    assert!(!(ab <= a));
}

#[test]
fn test_empty_is_bottom() {
    let empty = AscentPtsSet::empty();
    let a = AscentPtsSet::singleton(make_id("loc_a"));
    assert!(empty <= a);
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-datalog --test pts_lattice_test'`
Expected: FAIL — `AscentPtsSet` not defined.

**Step 3: Implement `AscentPtsSet`**

```rust
// crates/saf-datalog/src/pta/pts_lattice.rs
use std::collections::BTreeSet;
use ascent::lattice::Lattice;
use saf_core::air::LocId;

/// Points-to set for Ascent lattice-based PTA.
/// Uses `BTreeSet<LocId>` with set-union as join (least upper bound).
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct AscentPtsSet(BTreeSet<LocId>);

impl AscentPtsSet {
    pub fn empty() -> Self {
        Self(BTreeSet::new())
    }

    pub fn singleton(loc: LocId) -> Self {
        Self(BTreeSet::from([loc]))
    }

    pub fn from_iter(locs: impl IntoIterator<Item = LocId>) -> Self {
        Self(locs.into_iter().collect())
    }

    pub fn contains(&self, loc: LocId) -> bool {
        self.0.contains(&loc)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = LocId> + '_ {
        self.0.iter().copied()
    }

    /// Convert to the standard SAF `BTreeSet<LocId>` for output compatibility.
    pub fn into_btreeset(self) -> BTreeSet<LocId> {
        self.0
    }
}

impl PartialOrd for AscentPtsSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0 == other.0 {
            Some(std::cmp::Ordering::Equal)
        } else if self.0.is_subset(&other.0) {
            Some(std::cmp::Ordering::Less)
        } else if other.0.is_subset(&self.0) {
            Some(std::cmp::Ordering::Greater)
        } else {
            None // incomparable sets
        }
    }
}

impl Lattice for AscentPtsSet {
    fn meet(self, other: Self) -> Self {
        Self(self.0.intersection(&other.0).copied().collect())
    }

    fn join(self, other: Self) -> Self {
        let mut result = self.0;
        result.extend(other.0);
        Self(result)
    }
}
```

Note: The exact `Lattice` trait signature may differ in Ascent 0.8. Check `ascent::lattice::Lattice` docs — it may use `meet_mut`/`join_mut` instead. Adapt the impl to match the actual trait.

**Step 4: Wire into mod.rs**

```rust
// crates/saf-datalog/src/pta/mod.rs
//! Ascent-based pointer analysis solver.
pub mod pts_lattice;
pub use pts_lattice::AscentPtsSet;
```

**Step 5: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-datalog --test pts_lattice_test'`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/saf-datalog/src/pta/ crates/saf-datalog/tests/
git commit -m "feat(datalog): implement AscentPtsSet lattice for PTA"
```

---

## Task 3: Implement PTA Fact Extraction

Single `extract_facts()` function replacing the 3 extraction entry points. Takes a scope parameter to control which functions to process.

**Files:**
- Create: `crates/saf-datalog/src/facts.rs`
- Create: `crates/saf-datalog/tests/fact_extraction_test.rs`

**Step 1: Write the failing test**

```rust
// crates/saf-datalog/tests/fact_extraction_test.rs
use saf_datalog::facts::{extract_facts, AnalysisScope, PtaFacts};
use saf_test_utils::load_ll_fixture;
use saf_analysis::pta::LocationFactory;

#[test]
fn test_extract_facts_produces_addr_constraints() {
    let module = load_ll_fixture("simple_alias.ll");
    let mut factory = LocationFactory::new(Default::default());
    let facts = extract_facts(&module, &mut factory, AnalysisScope::WholeProgram);

    // Should produce at least one addr fact (any alloca or global)
    assert!(!facts.addr_of.is_empty(), "expected addr_of facts from simple_alias");
}

#[test]
fn test_extract_facts_scope_filtering() {
    let module = load_ll_fixture("simple_alias.ll");
    let mut factory = LocationFactory::new(Default::default());

    let whole = extract_facts(&module, &mut factory, AnalysisScope::WholeProgram);
    // Intraprocedural should have no interprocedural copy constraints
    let intra = extract_facts(&module, &mut factory, AnalysisScope::Intraprocedural);

    // Whole program should have >= intraprocedural facts
    assert!(whole.copy.len() >= intra.copy.len());
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-datalog --test fact_extraction_test'`
Expected: FAIL — `extract_facts` not defined.

**Step 3: Implement fact extraction**

The implementation wraps the existing internal helpers from `saf-analysis::pta::extract`. This initially delegates to the 3 existing functions for correctness. The sync bug fix comes from the fact that callers now use `extract_facts(scope)` — a single call site. In a follow-up, the internal logic can be truly unified.

```rust
// crates/saf-datalog/src/facts.rs
use std::collections::BTreeSet;
use saf_core::air::{AirModule, FunctionId, LocId, ValueId};
use saf_analysis::pta::{
    ConstraintSet, FieldPath, LocationFactory,
    extract_constraints, extract_constraints_reachable,
    extract_intraprocedural_constraints,
};

/// Scope for fact extraction — replaces the 3 separate functions.
pub enum AnalysisScope {
    /// Extract from all functions (replaces `extract_constraints`).
    WholeProgram,
    /// Extract from reachable functions only (replaces `extract_constraints_reachable`).
    Reachable(BTreeSet<FunctionId>),
    /// Extract without interprocedural constraints (replaces `extract_intraprocedural_constraints`).
    Intraprocedural,
}

/// Flat fact tuples for Ascent input relations.
pub struct PtaFacts {
    pub addr_of: Vec<(ValueId, LocId)>,
    pub copy: Vec<(ValueId, ValueId)>,
    pub load: Vec<(ValueId, ValueId)>,
    pub store: Vec<(ValueId, ValueId)>,
    pub gep: Vec<(ValueId, ValueId, FieldPath)>,
}

/// Single entry point for PTA fact extraction.
pub fn extract_facts(
    module: &AirModule,
    factory: &mut LocationFactory,
    scope: AnalysisScope,
) -> PtaFacts {
    let constraints = match scope {
        AnalysisScope::WholeProgram => extract_constraints(module, factory),
        AnalysisScope::Reachable(ref reachable) => {
            extract_constraints_reachable(module, reachable, factory)
        }
        AnalysisScope::Intraprocedural => {
            extract_intraprocedural_constraints(module, factory)
        }
    };
    constraint_set_to_facts(constraints)
}

fn constraint_set_to_facts(cs: ConstraintSet) -> PtaFacts {
    PtaFacts {
        addr_of: cs.addr.into_iter().map(|a| (a.ptr, a.loc)).collect(),
        copy: cs.copy.into_iter().map(|c| (c.dst, c.src)).collect(),
        load: cs.load.into_iter().map(|l| (l.dst, l.src_ptr)).collect(),
        store: cs.store.into_iter().map(|s| (s.dst_ptr, s.src)).collect(),
        gep: cs.gep.into_iter().map(|g| (g.dst, g.src_ptr, g.path)).collect(),
    }
}
```

**Step 4: Update saf-datalog Cargo.toml** — add `saf-analysis` dependency, `saf-frontends` to dev-deps.

**Step 5: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-datalog --test fact_extraction_test'`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/saf-datalog/
git commit -m "feat(datalog): implement unified fact extraction with AnalysisScope"
```

---

## Task 4: Core Ascent PTA Solver (Basic Andersen's)

Implement the core Andersen's rules in Ascent without SCC or BYODS.

**Files:**
- Create: `crates/saf-datalog/src/pta/solver.rs`
- Modify: `crates/saf-datalog/src/pta/mod.rs`
- Create: `crates/saf-datalog/tests/pta_basic_test.rs`

**Step 1: Write the failing test**

```rust
// crates/saf-datalog/tests/pta_basic_test.rs
use saf_datalog::pta::{ascent_solve, AscentPtsSet};
use saf_datalog::facts::PtaFacts;
use saf_core::id::make_id;

#[test]
fn test_addr_constraint() {
    // p = &x  =>  pts(p) = {x}
    let p = make_id("p");
    let x = make_id("x");

    let facts = PtaFacts {
        addr_of: vec![(p, x)],
        copy: vec![], load: vec![], store: vec![], gep: vec![],
    };

    let result = ascent_solve(&facts);
    let p_pts = result.get(&p).expect("p should have points-to set");
    assert!(p_pts.contains(&x));
}

#[test]
fn test_copy_propagation() {
    // p = &x; q = p  =>  pts(q) = {x}
    let p = make_id("p");
    let q = make_id("q");
    let x = make_id("x");

    let facts = PtaFacts {
        addr_of: vec![(p, x)],
        copy: vec![(q, p)],
        load: vec![], store: vec![], gep: vec![],
    };

    let result = ascent_solve(&facts);
    let q_pts = result.get(&q).expect("q should have points-to set");
    assert!(q_pts.contains(&x));
}
```

**Step 2: Run test to verify it fails**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-datalog --test pta_basic_test'`
Expected: FAIL — `ascent_solve` not defined.

**Step 3: Implement Ascent PTA solver**

```rust
// crates/saf-datalog/src/pta/solver.rs
use std::collections::{BTreeMap, BTreeSet};
use ascent::ascent;
use saf_core::air::{LocId, ValueId};
use crate::facts::PtaFacts;
use crate::pta::AscentPtsSet;

/// Output type matching SAF's `pta::PointsToMap`.
pub type PointsToMap = BTreeMap<ValueId, BTreeSet<LocId>>;

ascent! {
    struct PtaProgram;

    // --- Input facts ---
    relation addr_of(ValueId, LocId);
    relation copy_edge(ValueId, ValueId);  // (dst, src)
    relation load_edge(ValueId, ValueId);  // (dst, src_ptr)
    relation store_edge(ValueId, ValueId); // (dst_ptr, src)

    // --- Derived: points-to as lattice ---
    lattice points_to(ValueId, AscentPtsSet);

    // Addr: p = &x
    points_to(p, AscentPtsSet::singleton(*loc)) <-- addr_of(p, loc);

    // Copy: dst = src
    points_to(dst, pts.clone()) <-- copy_edge(dst, src), points_to(src, pts);

    // Store: *dst_ptr = src  (for each loc in pts(dst_ptr), store src's pts at loc)
    relation store_target(LocId, ValueId);
    store_target(loc, *src) <--
        store_edge(dst_ptr, src),
        points_to(dst_ptr, pts),
        for loc in pts.iter();

    lattice loc_pts(LocId, AscentPtsSet);
    loc_pts(loc, pts.clone()) <--
        store_target(loc, src),
        points_to(src, pts);

    // Load: dst = *src_ptr  (for each loc in pts(src_ptr), dst gets loc_pts(loc))
    points_to(dst, pts.clone()) <--
        load_edge(dst, src_ptr),
        points_to(src_ptr, ptr_pts),
        for loc in ptr_pts.iter(),
        loc_pts(loc, pts);
}

/// Run Ascent-based Andersen's pointer analysis.
pub fn ascent_solve(facts: &PtaFacts) -> PointsToMap {
    let mut prog = PtaProgram::default();

    prog.addr_of = facts.addr_of.iter().map(|(p, l)| (*p, *l)).collect();
    prog.copy_edge = facts.copy.iter().map(|(d, s)| (*d, *s)).collect();
    prog.load_edge = facts.load.iter().map(|(d, s)| (*d, *s)).collect();
    prog.store_edge = facts.store.iter().map(|(d, s)| (*d, *s)).collect();

    prog.run();

    let mut result = PointsToMap::new();
    for (val, pts) in &prog.points_to {
        if !pts.is_empty() {
            result.insert(*val, pts.iter().collect());
        }
    }
    result
}
```

Note: GEP handling deferred to Task 5. Store/load semantics may need adjustment based on how SAF bridges `LocId` and `ValueId` domains — debug with real fixtures.

**Step 4: Wire into mod.rs**

```rust
// crates/saf-datalog/src/pta/mod.rs
pub mod pts_lattice;
pub mod solver;
pub use pts_lattice::AscentPtsSet;
pub use solver::{ascent_solve, PointsToMap};
```

**Step 5: Run test to verify it passes**

Run: `docker compose run --rm dev sh -c 'cargo test -p saf-datalog --test pta_basic_test'`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/saf-datalog/
git commit -m "feat(datalog): implement core Ascent PTA solver (Andersen's)"
```

---

## Task 5: Add Field Sensitivity (GEP Rules)

Extend the Ascent PTA with GEP constraint handling for field-sensitive analysis.

**Files:**
- Modify: `crates/saf-datalog/src/pta/solver.rs`
- Create: `crates/saf-datalog/tests/pta_field_test.rs`

**Step 1: Write the failing test**

Test that GEP constraints produce field-specific points-to facts. Use a fixture with struct field access.

**Step 2: Add GEP rules to the Ascent program**

Add `relation gep_edge(ValueId, ValueId, FieldPath)` and a rule that resolves field locations from base locations. Reference `saf-analysis/src/pta/location.rs` for `Location::with_field_path()` or equivalent.

**Step 3: Verify existing tests still pass + new test passes**

**Step 4: Commit**

```bash
git commit -m "feat(datalog): add field-sensitive GEP rules to Ascent PTA"
```

---

## Task 6: Port HVN Preprocessing

Port the HVN (Hash-Value Numbering) preprocessing to work on `PtaFacts`.

**Files:**
- Create: `crates/saf-datalog/src/pta/hvn.rs`
- Create: `crates/saf-datalog/tests/hvn_test.rs`

**Step 1: Write the failing test** — verify HVN merges equivalent values and reduces fact count.

**Step 2: Implement**

Option B (simpler): Convert `PtaFacts` -> `ConstraintSet`, run existing `hvn_preprocess()`, convert back. Guarantees identical behavior.

```rust
// crates/saf-datalog/src/pta/hvn.rs
use saf_analysis::pta::hvn_preprocess;
use crate::facts::PtaFacts;

pub struct HvnResult {
    pub num_classes: usize,
    pub removed: usize,
}

pub fn preprocess_hvn(facts: &mut PtaFacts) -> HvnResult {
    let mut cs = facts_to_constraint_set(facts);
    let result = hvn_preprocess(&mut cs);
    *facts = constraint_set_to_facts(cs);
    HvnResult { num_classes: result.num_classes, removed: result.removed }
}
```

**Step 3: Run tests**

**Step 4: Commit**

```bash
git commit -m "feat(datalog): add HVN preprocessing for Ascent PTA facts"
```

---

## Task 7: Offline SCC Detection

Implement static cycle detection on the copy-constraint graph as a preprocessing step.

**Files:**
- Create: `crates/saf-datalog/src/pta/scc.rs`
- Create: `crates/saf-datalog/tests/scc_test.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_cycle_detection() {
    let a = make_id("a");
    let b = make_id("b");
    let c = make_id("c");

    // a -> b -> c -> a (cycle)
    let copy_edges = vec![(b, a), (c, b), (a, c)];
    let representatives = detect_scc(&copy_edges);

    // All three should map to same representative
    let rep_a = representatives.get(&a).copied().unwrap_or(a);
    let rep_b = representatives.get(&b).copied().unwrap_or(b);
    let rep_c = representatives.get(&c).copied().unwrap_or(c);
    assert_eq!(rep_a, rep_b);
    assert_eq!(rep_b, rep_c);
}
```

**Step 2: Implement SCC detection**

Use Tarjan's iterative SCC algorithm (same approach as existing `saf-analysis/src/pta/solver.rs` lines 352-425). SCC is one-shot preprocessing, not iterated fixpoint, so Rust is a better fit than Ascent here.

```rust
// crates/saf-datalog/src/pta/scc.rs
pub fn detect_scc(copy_edges: &[(ValueId, ValueId)]) -> BTreeMap<ValueId, ValueId> { ... }
pub fn rewrite_facts_with_scc(facts: &mut PtaFacts, reps: &BTreeMap<ValueId, ValueId>) { ... }
```

**Step 3: Run tests**

**Step 4: Commit**

```bash
git commit -m "feat(datalog): add offline SCC detection for copy-constraint graph"
```

---

## Task 8: Integrate Ascent Solver into PtaContext

Wire the Ascent solver into SAF's analysis pipeline. Add `legacy-pta` feature flag.

**Files:**
- Modify: `crates/saf-analysis/Cargo.toml` (add feature flag + saf-datalog dep)
- Modify: `crates/saf-analysis/src/pta/context.rs` (dispatch based on feature)

**Step 1: Add feature flag and dependency**

In `crates/saf-analysis/Cargo.toml`:

```toml
[dependencies]
saf-datalog = { path = "../saf-datalog", optional = true }

[features]
default = ["z3-solver", "ascent-pta"]
legacy-pta = []
ascent-pta = ["dep:saf-datalog"]
```

**Step 2: Add dispatch in PtaContext**

In `crates/saf-analysis/src/pta/context.rs`, modify `analyze_with_specs`:

```rust
#[cfg(feature = "ascent-pta")]
let (pts, iteration_limit_hit) = {
    use saf_datalog::facts::{extract_facts, AnalysisScope};
    use saf_datalog::pta::ascent_solve;
    let facts = extract_facts(module, &mut self.factory, AnalysisScope::WholeProgram);
    let result = ascent_solve(&facts);
    (result, false)
};

#[cfg(not(feature = "ascent-pta"))]
let (pts, iteration_limit_hit) = solve_with_index_config(
    &constraints, &self.factory, self.config.max_iterations,
    &self.config.pts_config, Some(module), self.config.index_sensitivity,
);
```

**Step 3: Run existing PTA tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test pta_verification_e2e'`

**Step 4: Verify legacy solver still works**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test pta_verification_e2e --no-default-features --features legacy-pta,z3-solver'`

**Step 5: Commit**

```bash
git commit -m "feat(datalog): integrate Ascent PTA into PtaContext with feature flag"
```

---

## Task 9: Validate PTA Correctness with PTABen

**Step 1: Run PTABen with Ascent solver (default)**

Run (background, 30-120s):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/ascent-results.json'
```

**Step 2: Run PTABen with legacy solver**

Run (background):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench --no-default-features --features legacy-pta,z3-solver -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/legacy-results.json'
```

**Step 3: Compare results** — same exact/sound/unsound counts per category. No new unsound.

**Step 4: Fix discrepancies** — common: missing GEP, store/load LocId bridging, HVN interaction.

**Step 5: Commit**

```bash
git commit -m "test(datalog): validate Ascent PTA correctness with PTABen"
```

---

## Task 10: Add ascent_par! for Parallel Evaluation

**Files:**
- Modify: `crates/saf-datalog/src/pta/solver.rs`
- Modify: `crates/saf-datalog/Cargo.toml`

**Step 1: Add cfg-gated parallel solver**

Use `ascent_par!` when `parallel` feature enabled and target is not WASM. Same rules, different macro.

**Step 2: Benchmark sequential vs parallel on PTABen**

**Step 3: Commit**

```bash
git commit -m "feat(datalog): add parallel Ascent PTA via ascent_par!"
```

---

## Task 11: Checker Base Facts Extraction from SVFG

Extract SVFG structure and classified sites into flat tuples for Ascent checker rules.

**Files:**
- Create: `crates/saf-datalog/src/checkers/base_facts.rs`
- Modify: `crates/saf-datalog/src/checkers/mod.rs`
- Create: `crates/saf-datalog/tests/checker_facts_test.rs`

**Step 1: Write the failing test** — extract facts from UAF fixture, verify non-empty edges/sources/sinks.

**Step 2: Implement**

```rust
// crates/saf-datalog/src/checkers/base_facts.rs
pub struct CheckerFacts {
    pub svfg_edges: Vec<(SvfgNodeId, SvfgNodeId)>,
    pub sources: Vec<SvfgNodeId>,
    pub sinks: Vec<SvfgNodeId>,
    pub sanitizers: Vec<SvfgNodeId>,
    pub exits: Vec<SvfgNodeId>,
}

pub fn extract_checker_facts(
    svfg: &Svfg,
    spec: &CheckerSpec,
    classified: &ClassifiedSites,
) -> CheckerFacts { ... }
```

Reuse existing `resolve_patterns` from `saf_analysis::checkers::runner` for source/sink/sanitizer resolution.

**Step 3: Run tests**

**Step 4: Commit**

```bash
git commit -m "feat(datalog): implement checker base facts extraction from SVFG"
```

---

## Task 12: Ascent Checker Reachability Rules

Implement the 3 reachability modes as Ascent rules.

**Files:**
- Create: `crates/saf-datalog/src/checkers/solver.rs`
- Create: `crates/saf-datalog/tests/checker_solver_test.rs`

**Step 1: Write the failing test** — test `may_reach` on a simple source->sink pattern.

**Step 2: Implement three Ascent programs**

```rust
// MayReach
ascent! {
    struct MayReachProgram;
    relation edge(SvfgNodeId, SvfgNodeId);
    relation source(SvfgNodeId);
    relation sink(SvfgNodeId);
    relation sanitizer(SvfgNodeId);

    relation flows(SvfgNodeId, SvfgNodeId);
    flows(a, b) <-- edge(a, b), !sanitizer(b);
    flows(a, c) <-- flows(a, b), edge(b, c), !sanitizer(c);

    relation finding(SvfgNodeId, SvfgNodeId);
    finding(src, snk) <-- source(src), flows(src, snk), sink(snk);
}

// MustNotReach — same but sinks are exit nodes
// MultiReach — uses agg count() for 2+ sinks
```

**Step 3: Implement `must_not_reach` and `multi_reach`**

**Step 4: Run tests**

**Step 5: Commit**

```bash
git commit -m "feat(datalog): implement Ascent checker reachability rules"
```

---

## Task 13: Wire Ascent Checkers into Runner

Replace BFS solver calls in `runner.rs` with Ascent solver calls.

**Files:**
- Modify: `crates/saf-analysis/src/checkers/runner.rs`
- Create: `crates/saf-datalog/src/checkers/mod.rs` (high-level API)

**Step 1: Create `run_ascent_checker` in saf-datalog**

```rust
pub fn run_ascent_checker(
    spec: &CheckerSpec,
    svfg: &Svfg,
    classified: &ClassifiedSites,
) -> Vec<(SvfgNodeId, SvfgNodeId)> {
    let facts = extract_checker_facts(svfg, spec, classified);
    match spec.mode {
        ReachabilityMode::MayReach => may_reach(&facts).findings,
        ReachabilityMode::MustNotReach => must_not_reach(&facts).findings,
        ReachabilityMode::MultiReach => multi_reach(&facts).findings,
    }
}
```

**Step 2: Modify runner.rs** — replace solver dispatch block with `run_ascent_checker`. Keep post-processing (location resolution, overlap expansion, suppression) unchanged.

**Step 3: Run checker E2E tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test checker_e2e'`

**Step 4: Commit**

```bash
git commit -m "feat(datalog): wire Ascent checkers into runner replacing BFS solvers"
```

---

## Task 14: Validate Checkers with E2E and Benchmarks

**Step 1: Run checker E2E tests**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test checker_e2e'`

**Step 2: Run Juliet for checker-relevant CWEs**

Run (background): CWE416 (UAF), CWE401 (memory leak), CWE415 (double free), CWE476 (null deref).

**Step 3: Run playground differential tests**

Run: `cd playground && npx playwright test e2e/differential.spec.ts`

**Step 4: Fix discrepancies** — common: depth limit differences (BFS vs Ascent transitive closure), sanitizer blocking semantics.

**Step 5: Commit**

```bash
git commit -m "test(datalog): validate Ascent checkers with E2E and Juliet"
```

---

## Task 15: Unified Query Protocol Types

**Files:**
- Modify: `crates/saf-analysis/src/database/protocol.rs` (or equivalent)

**Step 1: Add `QueryLanguage` enum and unified request type**

```rust
pub enum QueryLanguage { Builtin, Python, Datalog }
```

**Step 2: Add engine metadata to response**

**Step 3: Commit**

```bash
git commit -m "feat(datalog): add unified query protocol types"
```

---

## Task 16: Wire Unified Protocol into Handler

**Files:**
- Modify: `crates/saf-analysis/src/database/handler.rs`

**Step 1: Add `query` action routing** — dispatch to `handle_check` for `builtin`, error for `python`/`datalog` (Phase 2).

**Step 2: Backward compatibility** — `check` and `check_all` still work.

**Step 3: Commit**

```bash
git commit -m "feat(datalog): wire unified query protocol into handler"
```

---

## Task 17: CruxBC Performance Benchmark

**Step 1: Run CruxBC with Ascent** (background)
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc -o /workspace/tests/benchmarks/cruxbc/ascent-results.json'
```

**Step 2: Run CruxBC with legacy** (background)

**Step 3: Compare PTA phase timings** — document speedup/regression.

**Step 4: Commit**

```bash
git commit -m "bench(datalog): CruxBC performance comparison Ascent vs legacy PTA"
```

---

## Task 18: Rebuild WASM and Validate Playground

**Step 1: Check WASM compilation**
```bash
docker compose run --rm dev sh -c 'cargo check -p saf-wasm --target wasm32-unknown-unknown'
```

**Step 2: Build WASM** — `make wasm`

**Step 3: Run playground tests** — differential + suppression

**Step 4: Commit**

```bash
git commit -m "build(wasm): rebuild WASM with Ascent-based analysis"
```

---

## Task 19: Final Lint, Format, and Cleanup

**Step 1: Format and lint** — `make fmt && make lint`

**Step 2: Run full test suite** — `make test`

**Step 3: Update PROGRESS.md**

**Step 4: Commit**

```bash
git commit -m "chore: format, lint, and update progress for Datalog integration"
```

---

## Summary

| Task | Component | Effort |
|------|-----------|--------|
| 1 | Crate skeleton | 15 min |
| 2 | PointsToSet lattice | 30 min |
| 3 | Fact extraction | 45 min |
| 4 | Core Ascent PTA | 2-3 hrs |
| 5 | Field sensitivity (GEP) | 1-2 hrs |
| 6 | HVN preprocessing | 30 min |
| 7 | Offline SCC | 1-2 hrs |
| 8 | PtaContext integration | 1 hr |
| 9 | PTABen validation | 1-2 hrs |
| 10 | Parallel mode | 30 min |
| 11 | Checker facts extraction | 1 hr |
| 12 | Checker reachability rules | 2-3 hrs |
| 13 | Wire checkers into runner | 1 hr |
| 14 | Checker validation | 1-2 hrs |
| 15 | Query protocol types | 30 min |
| 16 | Handler integration | 30 min |
| 17 | CruxBC benchmark | 1 hr |
| 18 | WASM rebuild + playground | 1 hr |
| 19 | Cleanup | 30 min |

**Total estimated: ~4-5 working days**

**Critical path:** Tasks 1-4 (core PTA) must succeed before anything else. Task 9 (PTABen validation) is the go/no-go gate.
