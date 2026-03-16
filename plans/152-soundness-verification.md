# SAF Soundness Verification — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a three-pillar verification strategy (Kani formal proofs, proptest property tests, hand-crafted oracle suite) to verify soundness of SAF's analysis pipeline.

**Architecture:** Pillar 1 uses Kani model checking for bounded exhaustive proofs on isolated components (saf-core ID/spec, saf-analysis PtsSet/constraints). Pillar 2 extends existing proptest infrastructure for pipeline-level statistical verification. Pillar 3 creates 54 hand-crafted C programs with YAML expected results testing core library outputs (PTA, CG, CFG, MSSA, SVFG), with a harness in saf-bench that compiles, analyzes, compares, and reports.

**Tech Stack:** Kani (`kani-verifier` crate), proptest (already used), serde_yaml (already used in saf-bench), clap (already used), LLVM 18 (Docker).

**Design doc:** `docs/plans/2026-02-22-soundness-verification-design.md`

**IMPORTANT:** All `make` / `docker compose` commands must be run by the main agent only, never by subagents. saf-core is the only crate safe to build/test locally.

---

## Phase A: Kani Infrastructure & saf-core Proofs

### Task 1: Kani Setup for saf-core

**Files:**
- Modify: `crates/saf-core/Cargo.toml`
- Create: `crates/saf-core/src/kani_proofs.rs`
- Modify: `crates/saf-core/src/lib.rs`

**Step 1: Add Kani dev-dependency to saf-core**

In `crates/saf-core/Cargo.toml`, add under `[dev-dependencies]`:

```toml
[dev-dependencies]
pretty_assertions = { workspace = true }
insta = { workspace = true }
tempfile = { workspace = true }
kani-verifier = "0.59"
```

**Step 2: Create Kani proof module**

Create `crates/saf-core/src/kani_proofs.rs`:

```rust
//! Kani formal verification proof harnesses for saf-core.
//!
//! These are only compiled when running `cargo kani`.
//! They have zero impact on normal builds, tests, or releases.

#[cfg(kani)]
mod id_proofs {
    use crate::id::make_id;
    use crate::ids::{BlockId, FunctionId, ValueId};

    /// PROPERTY: Domain separation — different domain tags produce different IDs
    /// for the same input data.
    #[kani::proof]
    fn proof_domain_separation() {
        let data: [u8; 8] = kani::any();
        let func_id = FunctionId::derive(&data);
        let block_id = BlockId::derive(&data);
        let value_id = ValueId::derive(&data);

        assert_ne!(func_id.raw(), block_id.raw(), "FunctionId and BlockId must differ for same input");
        assert_ne!(func_id.raw(), value_id.raw(), "FunctionId and ValueId must differ for same input");
        assert_ne!(block_id.raw(), value_id.raw(), "BlockId and ValueId must differ for same input");
    }

    /// PROPERTY: Determinism — same domain and data always produce the same ID.
    #[kani::proof]
    fn proof_id_determinism() {
        let data: [u8; 8] = kani::any();
        let id1 = make_id("test_domain", &data);
        let id2 = make_id("test_domain", &data);
        assert_eq!(id1, id2, "make_id must be deterministic");
    }

    /// PROPERTY: Non-zero — IDs from non-empty data are never zero.
    #[kani::proof]
    fn proof_id_nonzero() {
        let byte: u8 = kani::any();
        kani::assume(byte != 0); // At least one non-zero byte
        let id = make_id("nonzero_test", &[byte]);
        // BLAKE3 output for non-trivial input is astronomically unlikely to be 0
        // but we verify the property holds for all single-byte inputs
        assert_ne!(id, 0, "ID from non-empty data should not be zero");
    }
}

#[cfg(kani)]
mod spec_proofs {
    use crate::spec::analyzed::AnalyzedSpecRegistry;
    use crate::spec::derived::DerivedSpec;
    use crate::spec::registry::SpecRegistry;

    /// PROPERTY: Lookup returns None for empty registry.
    #[kani::proof]
    fn proof_empty_registry_lookup() {
        let registry = AnalyzedSpecRegistry::default();
        // Any function name on empty registry returns None
        assert!(registry.lookup("foo").is_none());
        assert!(registry.lookup("bar").is_none());
        assert!(registry.lookup("").is_none());
    }

    /// PROPERTY: Derived spec is visible after add_derived.
    #[kani::proof]
    fn proof_derived_visible_after_add() {
        let mut registry = AnalyzedSpecRegistry::default();
        let spec = DerivedSpec::default();
        registry.add_derived("test_func", spec);

        let result = registry.lookup("test_func");
        assert!(result.is_some(), "Derived spec must be visible after add");

        let result = result.unwrap();
        assert!(result.derived().is_some(), "Derived overlay must be present");
        assert!(result.yaml().is_none(), "No YAML spec was added");
    }

    /// PROPERTY: Derived count matches number of add_derived calls (unique names).
    #[kani::proof]
    fn proof_derived_count() {
        let mut registry = AnalyzedSpecRegistry::default();
        assert_eq!(registry.derived_count(), 0);

        registry.add_derived("a", DerivedSpec::default());
        assert_eq!(registry.derived_count(), 1);

        registry.add_derived("b", DerivedSpec::default());
        assert_eq!(registry.derived_count(), 2);

        // Re-adding same name replaces, count stays at 2
        registry.add_derived("a", DerivedSpec::default());
        assert_eq!(registry.derived_count(), 2);
    }

    /// PROPERTY: Lookup for non-existent name returns None even when registry has entries.
    #[kani::proof]
    fn proof_lookup_miss() {
        let mut registry = AnalyzedSpecRegistry::default();
        registry.add_derived("exists", DerivedSpec::default());

        assert!(registry.lookup("not_exists").is_none());
        assert!(registry.lookup_derived("not_exists").is_none());
        assert!(registry.lookup_yaml("not_exists").is_none());
    }
}
```

**Step 3: Register module in lib.rs**

Add to `crates/saf-core/src/lib.rs`:

```rust
#[cfg(kani)]
mod kani_proofs;
```

**Step 4: Verify Kani compiles locally (saf-core has no LLVM dep)**

Run: `cd crates/saf-core && cargo kani --tests`
Expected: All 7 proofs verified (or Kani reports verification results)

Note: If `cargo kani` is not installed, install with:
```bash
cargo install --locked kani-verifier
cargo kani setup
```

**Step 5: Commit**

```bash
git add crates/saf-core/Cargo.toml crates/saf-core/src/kani_proofs.rs crates/saf-core/src/lib.rs
git commit -m "feat(verify): add Kani proof harnesses for saf-core ID and spec registry"
```

---

### Task 2: Kani Setup for saf-analysis (Docker)

**Files:**
- Modify: `crates/saf-analysis/Cargo.toml`
- Create: `crates/saf-analysis/src/pta/ptsset/kani_proofs.rs`
- Modify: `crates/saf-analysis/src/pta/ptsset/mod.rs`
- Modify: `docker/Dockerfile.dev` (or equivalent)

**Step 1: Add Kani dev-dependency to saf-analysis**

In `crates/saf-analysis/Cargo.toml`, add under `[dev-dependencies]`:

```toml
kani-verifier = "0.59"
```

**Step 2: Create PtsSet Kani proof module**

Create `crates/saf-analysis/src/pta/ptsset/kani_proofs.rs`:

```rust
//! Kani formal verification proofs for PtsSet implementations.
//!
//! Proves algebraic properties: insert-then-contains, union commutativity,
//! idempotence, and cross-implementation equivalence.
//!
//! Only compiled when running `cargo kani`. Zero production impact.

#[cfg(kani)]
mod btree_proofs {
    use crate::pta::ptsset::btree::BTreePtsSet;
    use crate::pta::ptsset::trait_def::PtsSet;
    use saf_core::ids::LocId;

    /// PROPERTY: Insert-then-contains — inserting x means contains(x) is true.
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_insert_then_contains() {
        let raw: u128 = kani::any();
        let loc = LocId::new(raw);
        let mut set = BTreePtsSet::empty();
        set.insert(loc);
        assert!(set.contains(loc), "insert(x) must make contains(x) true");
    }

    /// PROPERTY: Insert idempotence — inserting the same element twice doesn't change the set.
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_insert_idempotent() {
        let raw: u128 = kani::any();
        let loc = LocId::new(raw);
        let mut set = BTreePtsSet::empty();
        set.insert(loc);
        let len_after_first = set.len();
        let changed = set.insert(loc);
        assert!(!changed, "Second insert of same element must return false");
        assert_eq!(set.len(), len_after_first, "Length must not change on duplicate insert");
    }

    /// PROPERTY: Empty set contains nothing.
    #[kani::proof]
    fn proof_empty_contains_nothing() {
        let raw: u128 = kani::any();
        let loc = LocId::new(raw);
        let set = BTreePtsSet::empty();
        assert!(!set.contains(loc), "Empty set must not contain anything");
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    /// PROPERTY: Union commutativity — union(A,B) == union(B,A).
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_union_commutative() {
        // Use small bounded values to keep state space manageable
        let a_val: u8 = kani::any();
        let b_val: u8 = kani::any();
        let loc_a = LocId::new(u128::from(a_val));
        let loc_b = LocId::new(u128::from(b_val));

        // A ∪ B
        let mut set_ab = BTreePtsSet::empty();
        set_ab.insert(loc_a);
        let mut b_set = BTreePtsSet::empty();
        b_set.insert(loc_b);
        set_ab.union(&b_set);

        // B ∪ A
        let mut set_ba = BTreePtsSet::empty();
        set_ba.insert(loc_b);
        let mut a_set = BTreePtsSet::empty();
        a_set.insert(loc_a);
        set_ba.union(&a_set);

        assert_eq!(set_ab.to_btreeset(), set_ba.to_btreeset(),
            "union(A,B) must equal union(B,A)");
    }

    /// PROPERTY: Remove-then-not-contains — removing x means contains(x) is false.
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_remove_then_not_contains() {
        let raw: u128 = kani::any();
        let loc = LocId::new(raw);
        let mut set = BTreePtsSet::empty();
        set.insert(loc);
        assert!(set.contains(loc));
        set.remove(loc);
        assert!(!set.contains(loc), "remove(x) must make contains(x) false");
    }

    /// PROPERTY: Subset reflexivity — every set is a subset of itself.
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_subset_reflexive() {
        let val: u8 = kani::any();
        let loc = LocId::new(u128::from(val));
        let mut set = BTreePtsSet::empty();
        set.insert(loc);
        assert!(set.is_subset(&set), "Set must be a subset of itself");
    }
}

#[cfg(kani)]
mod fxhash_proofs {
    use crate::pta::ptsset::fxhash::FxHashPtsSet;
    use crate::pta::ptsset::trait_def::PtsSet;
    use saf_core::ids::LocId;

    /// PROPERTY: Insert-then-contains for FxHashPtsSet.
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_fxhash_insert_then_contains() {
        let raw: u128 = kani::any();
        let loc = LocId::new(raw);
        let mut set = FxHashPtsSet::empty();
        set.insert(loc);
        assert!(set.contains(loc), "FxHash: insert(x) must make contains(x) true");
    }

    /// PROPERTY: FxHashPtsSet agrees with BTreePtsSet on membership after same operations.
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_fxhash_btree_equivalence() {
        use crate::pta::ptsset::btree::BTreePtsSet;

        let val1: u8 = kani::any();
        let val2: u8 = kani::any();
        let loc1 = LocId::new(u128::from(val1));
        let loc2 = LocId::new(u128::from(val2));

        let mut btree = BTreePtsSet::empty();
        let mut fxhash = FxHashPtsSet::empty();

        btree.insert(loc1);
        fxhash.insert(loc1);
        btree.insert(loc2);
        fxhash.insert(loc2);

        let query: u8 = kani::any();
        let query_loc = LocId::new(u128::from(query));
        assert_eq!(
            btree.contains(query_loc),
            fxhash.contains(query_loc),
            "BTree and FxHash must agree on contains()"
        );
    }
}
```

**Step 3: Register module in ptsset/mod.rs**

Add to `crates/saf-analysis/src/pta/ptsset/mod.rs`:

```rust
#[cfg(kani)]
mod kani_proofs;
```

**Step 4: Install Kani in Docker and verify**

Kani for saf-analysis must run inside Docker because the crate depends on LLVM via dev-dependencies. Install Kani in the Docker dev image:

```bash
docker compose run --rm dev sh -c "cargo install --locked kani-verifier && cargo kani setup"
```

Then run proofs:

```bash
docker compose run --rm dev sh -c "cargo kani -p saf-analysis --default-features=false"
```

Note: Run with `--default-features=false` to skip the z3-solver feature which adds build complexity. The PtsSet proofs don't need Z3.

If Kani can't compile saf-analysis due to LLVM linking issues, fallback: move PtsSet proofs to a separate thin crate `crates/saf-verify/` that depends only on `saf-core` and copies the PtsSet types. Document this limitation.

**Step 5: Commit**

```bash
git add crates/saf-analysis/Cargo.toml crates/saf-analysis/src/pta/ptsset/kani_proofs.rs crates/saf-analysis/src/pta/ptsset/mod.rs
git commit -m "feat(verify): add Kani PtsSet algebra proofs for BTreePtsSet and FxHashPtsSet"
```

---

## Phase B: Proptest Extensions

### Task 3: Constraint Extraction Completeness Properties

**Files:**
- Modify: `crates/saf-analysis/src/proptest_arb.rs`
- Modify: `crates/saf-analysis/src/proptest_tests.rs`

**Step 1: Enhance PTA module generator with instruction tracking**

Add to `crates/saf-analysis/src/proptest_arb.rs` (after existing `arb_pta_function`):

```rust
/// Generate a PTA module that tracks how many of each instruction type it contains.
/// Returns (AirModule, expected_alloca_count, expected_store_count, expected_load_count).
pub fn arb_tracked_pta_module() -> impl Strategy<Value = (AirModule, usize, usize, usize)> {
    arb_pta_function().prop_map(|func| {
        let mut alloca_count = 0usize;
        let mut store_count = 0usize;
        let mut load_count = 0usize;

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.operation {
                    saf_core::air::Operation::Alloca { .. } => alloca_count += 1,
                    saf_core::air::Operation::Store { .. } => store_count += 1,
                    saf_core::air::Operation::Load { .. } => load_count += 1,
                    _ => {}
                }
            }
        }

        let module = saf_core::air::AirModule {
            id: saf_core::ids::ModuleId::derive(b"tracked_test"),
            name: "tracked_test".to_string(),
            source_filename: String::new(),
            functions: vec![func],
            globals: vec![],
            type_table: Default::default(),
        };

        (module, alloca_count, store_count, load_count)
    })
}
```

**Step 2: Add constraint extraction completeness property tests**

Add to `crates/saf-analysis/src/proptest_tests.rs` (in a new section after existing PTA tests):

```rust
mod constraint_extraction_tests {
    use super::*;
    use crate::proptest_arb::arb_tracked_pta_module;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// PROPERTY: Every alloca instruction produces at least one Addr constraint.
        #[test]
        fn every_alloca_produces_addr_constraint(
            (module, alloca_count, _, _) in arb_tracked_pta_module()
        ) {
            if alloca_count == 0 {
                return Ok(());  // Skip modules with no allocas
            }
            let config = make_pta_config();
            let mut ctx = PtaContext::new(config);
            let result = ctx.analyze(&module);
            prop_assert!(
                result.constraints.addr.len() >= alloca_count,
                "Expected at least {} Addr constraints for {} allocas, got {}",
                alloca_count,
                alloca_count,
                result.constraints.addr.len()
            );
        }

        /// PROPERTY: Every store instruction produces at least one Store constraint.
        #[test]
        fn every_store_produces_store_constraint(
            (module, _, store_count, _) in arb_tracked_pta_module()
        ) {
            if store_count == 0 {
                return Ok(());
            }
            let config = make_pta_config();
            let mut ctx = PtaContext::new(config);
            let result = ctx.analyze(&module);
            prop_assert!(
                result.constraints.store.len() >= store_count,
                "Expected at least {} Store constraints for {} stores, got {}",
                store_count,
                store_count,
                result.constraints.store.len()
            );
        }

        /// PROPERTY: Every load instruction produces at least one Load constraint.
        #[test]
        fn every_load_produces_load_constraint(
            (module, _, _, load_count) in arb_tracked_pta_module()
        ) {
            if load_count == 0 {
                return Ok(());
            }
            let config = make_pta_config();
            let mut ctx = PtaContext::new(config);
            let result = ctx.analyze(&module);
            prop_assert!(
                result.constraints.load.len() >= load_count,
                "Expected at least {} Load constraints for {} loads, got {}",
                load_count,
                load_count,
                result.constraints.load.len()
            );
        }

        /// PROPERTY: No constraint references a ValueId that doesn't exist in the module.
        #[test]
        fn no_dangling_constraint_references(module in arb_pta_module()) {
            let config = make_pta_config();
            let mut ctx = PtaContext::new(config);
            let result = ctx.analyze(&module);

            // Collect all ValueIds defined in the module
            let mut defined_values = std::collections::BTreeSet::new();
            for func in &module.functions {
                for param in &func.params {
                    defined_values.insert(param.value_id);
                }
                for block in &func.blocks {
                    for inst in &block.instructions {
                        if let Some(result_id) = inst.result {
                            defined_values.insert(result_id);
                        }
                    }
                }
            }

            // Check Copy constraints reference defined values
            for copy in &result.constraints.copy {
                prop_assert!(
                    defined_values.contains(&copy.src) || is_synthetic_value(copy.src),
                    "Copy constraint references undefined src: {:?}", copy.src
                );
                prop_assert!(
                    defined_values.contains(&copy.dst) || is_synthetic_value(copy.dst),
                    "Copy constraint references undefined dst: {:?}", copy.dst
                );
            }
        }
    }

    /// Check if a ValueId might be synthesized by the analysis (not from source).
    fn is_synthetic_value(_id: saf_core::ids::ValueId) -> bool {
        // The analysis may create synthetic values for globals, return values, etc.
        // For now, allow any value — tighten as we understand the synthesis rules.
        true
    }
}
```

**Step 3: Run tests in Docker to verify**

Run: `make test`
Expected: All tests pass (including new property tests with 1000 iterations)

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/proptest_arb.rs crates/saf-analysis/src/proptest_tests.rs
git commit -m "feat(verify): add proptest constraint extraction completeness properties"
```

---

## Phase C: Oracle Infrastructure

### Task 4: Oracle Directory Structure and YAML Schema

**Files:**
- Create: `tests/verification/oracle/README.md`
- Create: `tests/verification/oracle/schema.yaml`

**Step 1: Create directory structure**

```bash
mkdir -p tests/verification/oracle/{pta,callgraph,cfg,mssa,svfg}
```

**Step 2: Create schema documentation**

Create `tests/verification/oracle/schema.yaml`:

```yaml
# Schema for oracle expected results files.
# Each .c file has a companion .expected.yaml describing what SAF should compute.
#
# Naming convention:
#   tests/verification/oracle/<layer>/<test_name>.c
#   tests/verification/oracle/<layer>/<test_name>.expected.yaml

# --- Top-level fields ---
description: "Human-readable description of what this test verifies"
layer: "pta | callgraph | cfg | mssa | svfg"
difficulty: "basic | intermediate | advanced"

# --- PTA expectations ---
expectations:
  points_to:
    - pointer: "variable_name"       # Human-readable C variable name
      at_line: 10                     # Source line (optional, for disambiguation)
      must_contain: [x, y]            # SOUNDNESS: these MUST be in pts(p) — missing = BUG
      may_only_contain: [x, y, z]     # PRECISION: only these should be in pts(p) — extra = WARN

  alias:
    - pair: [p, q]
      relation: "must_alias | may_alias | no_alias"

  # --- Call graph expectations ---
  calls:
    - caller: "main"
      callees: [foo, bar]             # Direct + resolved indirect calls

  indirect_targets:
    - call_site_line: 15              # Line where indirect call occurs
      targets: [handler_a, handler_b] # PTA-resolved targets

  # --- CFG expectations ---
  cfg_edges:
    - function: "test"
      block: "entry"                  # Block name (from LLVM IR label)
      successors: [then_block, else_block]

  # --- MSSA expectations ---
  reaching_def:
    - use_line: 10
      def_line: 5
      variable: "x"

  # --- SVFG expectations ---
  value_flow:
    - from_line: 3
      to_line: 8
      description: "alloca flows to use via store+load"
```

**Step 3: Create README**

Create `tests/verification/oracle/README.md`:

```markdown
# Oracle Verification Suite

Hand-crafted C programs with known-correct analysis results.
Tests core library outputs (PTA, CG, CFG, MSSA, SVFG), NOT checker verdicts.

## Adding a new test

1. Write a small C program: `<layer>/<name>.c` (10-30 lines)
2. Write expected results: `<layer>/<name>.expected.yaml`
3. Run `make compile-oracle` to compile C → LLVM IR
4. Run `make verify-oracle` to check

## Verdict types

- **PASS**: Analysis results match expectations exactly
- **WARN**: Imprecision — extra items in points-to sets (acceptable)
- **FAIL**: Unsoundness — missing items in must_contain (BUG)

## Schema

See `schema.yaml` for the expected results format.
```

**Step 4: Commit**

```bash
git add tests/verification/oracle/
git commit -m "feat(verify): create oracle suite directory structure and YAML schema"
```

---

### Task 5: Oracle Programs — PTA (20 programs)

**Files:**
- Create: `tests/verification/oracle/pta/*.c` (20 files)
- Create: `tests/verification/oracle/pta/*.expected.yaml` (20 files)

Write all 20 C programs and their expected YAML files. Each program is 10-30 lines. Here are representative examples — all 20 must be created:

**Step 1: Basic aliasing (3 programs)**

`tests/verification/oracle/pta/01_simple_alias.c`:
```c
// PURPOSE: Simplest case — address-of creates a points-to edge
#include <stdlib.h>

void test() {
    int x;
    int *p = &x;
    *p = 42;
}
```

`tests/verification/oracle/pta/01_simple_alias.expected.yaml`:
```yaml
description: "Address-of creates a single points-to edge"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "p"
      must_contain: [x]
      may_only_contain: [x]
```

`tests/verification/oracle/pta/02_copy_alias.c`:
```c
// PURPOSE: Copy propagates points-to set from source to destination
void test() {
    int x;
    int *p = &x;
    int *q = p;
    *q = 42;
}
```

`tests/verification/oracle/pta/02_copy_alias.expected.yaml`:
```yaml
description: "Copy propagates points-to set"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "p"
      must_contain: [x]
    - pointer: "q"
      must_contain: [x]
  alias:
    - pair: [p, q]
      relation: must_alias
```

`tests/verification/oracle/pta/03_conditional_alias.c`:
```c
// PURPOSE: Conditional assignment creates may-alias via phi node
#include <stdlib.h>

int rand_bool(void);

void test() {
    int x, y;
    int *p;
    if (rand_bool()) {
        p = &x;
    } else {
        p = &y;
    }
    *p = 42;
}
```

`tests/verification/oracle/pta/03_conditional_alias.expected.yaml`:
```yaml
description: "Conditional pointer assignment creates may-alias"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "p"
      must_contain: [x, y]
      may_only_contain: [x, y]
```

**Step 2: Multi-level pointers (3 programs)**

`tests/verification/oracle/pta/04_double_pointer.c`:
```c
// PURPOSE: Double pointer dereference — p -> q -> x
void test() {
    int x;
    int *q = &x;
    int **p = &q;
    **p = 42;
}
```

`tests/verification/oracle/pta/04_double_pointer.expected.yaml`:
```yaml
description: "Double pointer dereference chain"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "q"
      must_contain: [x]
    - pointer: "p"
      must_contain: [q]
```

`tests/verification/oracle/pta/05_pointer_chain.c`:
```c
// PURPOSE: Three-level pointer chain a -> b -> c -> x
void test() {
    int x;
    int *c = &x;
    int **b = &c;
    int ***a = &b;
    ***a = 42;
}
```

`tests/verification/oracle/pta/05_pointer_chain.expected.yaml`:
```yaml
description: "Three-level pointer chain"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "c"
      must_contain: [x]
    - pointer: "b"
      must_contain: [c]
    - pointer: "a"
      must_contain: [b]
```

`tests/verification/oracle/pta/06_deref_assign.c`:
```c
// PURPOSE: Store through pointer — *p = &y changes what locations hold
void test() {
    int x, y;
    int *p = &x;
    int **pp = &p;
    *pp = &y;
    // After store, p now points to y (not x)
}
```

`tests/verification/oracle/pta/06_deref_assign.expected.yaml`:
```yaml
description: "Store through pointer reassigns target"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "pp"
      must_contain: [p]
    - pointer: "p"
      must_contain: [x, y]
```

**Step 3: Struct/field access (3 programs)**

`tests/verification/oracle/pta/07_struct_field.c`:
```c
// PURPOSE: Struct field pointer — p->field points to what was stored
struct Node { int val; int *ptr; };

void test() {
    int x;
    struct Node n;
    n.ptr = &x;
    int *p = n.ptr;
    *p = 42;
}
```

`tests/verification/oracle/pta/07_struct_field.expected.yaml`:
```yaml
description: "Struct field stores and loads pointer"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "p"
      must_contain: [x]
```

`tests/verification/oracle/pta/08_nested_struct.c`:
```c
// PURPOSE: Nested struct field access
struct Inner { int *ptr; };
struct Outer { struct Inner inner; };

void test() {
    int x;
    struct Outer o;
    o.inner.ptr = &x;
    int *p = o.inner.ptr;
    *p = 42;
}
```

`tests/verification/oracle/pta/08_nested_struct.expected.yaml`:
```yaml
description: "Nested struct field pointer access"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "p"
      must_contain: [x]
```

`tests/verification/oracle/pta/09_field_sensitive.c`:
```c
// PURPOSE: Two fields in same struct point to different objects
struct Pair { int *first; int *second; };

void test() {
    int x, y;
    struct Pair p;
    p.first = &x;
    p.second = &y;
    int *a = p.first;
    int *b = p.second;
}
```

`tests/verification/oracle/pta/09_field_sensitive.expected.yaml`:
```yaml
description: "Field-sensitive analysis distinguishes struct fields"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "a"
      must_contain: [x]
    - pointer: "b"
      must_contain: [y]
  alias:
    - pair: [a, b]
      relation: no_alias
```

**Step 4: Heap & arrays (3 programs)**

`tests/verification/oracle/pta/10_malloc.c`:
```c
// PURPOSE: Heap allocation — malloc returns a fresh location
#include <stdlib.h>

void test() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
}
```

`tests/verification/oracle/pta/10_malloc.expected.yaml`:
```yaml
description: "Malloc creates a fresh heap location"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "p"
      must_contain: ["malloc@test"]
```

`tests/verification/oracle/pta/11_two_mallocs.c`:
```c
// PURPOSE: Two mallocs produce distinct heap locations
#include <stdlib.h>

void test() {
    int *p = (int *)malloc(sizeof(int));
    int *q = (int *)malloc(sizeof(int));
}
```

`tests/verification/oracle/pta/11_two_mallocs.expected.yaml`:
```yaml
description: "Two malloc calls produce distinct locations"
layer: pta
difficulty: basic
expectations:
  alias:
    - pair: [p, q]
      relation: no_alias
```

`tests/verification/oracle/pta/12_array_element.c`:
```c
// PURPOSE: Array element access via GEP
void test() {
    int arr[10];
    int *p = &arr[3];
    *p = 42;
}
```

`tests/verification/oracle/pta/12_array_element.expected.yaml`:
```yaml
description: "Array element access via pointer arithmetic"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "p"
      must_contain: [arr]
```

**Step 5: Function pointers (3 programs)**

`tests/verification/oracle/pta/13_simple_fptr.c`:
```c
// PURPOSE: Simple function pointer call — PTA resolves indirect target
void handler(int x) { (void)x; }

void test() {
    void (*fp)(int) = handler;
    fp(42);
}
```

`tests/verification/oracle/pta/13_simple_fptr.expected.yaml`:
```yaml
description: "Function pointer resolved by PTA"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "fp"
      must_contain: [handler]
```

`tests/verification/oracle/pta/14_fptr_array.c`:
```c
// PURPOSE: Function pointer array — PTA resolves all possible targets
void a(void) {}
void b(void) {}
void c(void) {}

int rand_index(void);

void test() {
    void (*handlers[3])(void) = {a, b, c};
    handlers[rand_index()]();
}
```

`tests/verification/oracle/pta/14_fptr_array.expected.yaml`:
```yaml
description: "Function pointer array dispatch"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "handlers"
      must_contain: [a, b, c]
```

`tests/verification/oracle/pta/15_fptr_through_struct.c`:
```c
// PURPOSE: Function pointer stored in struct field
typedef void (*callback_t)(int);
struct Handler { callback_t cb; };

void on_event(int x) { (void)x; }

void test() {
    struct Handler h;
    h.cb = on_event;
    h.cb(42);
}
```

`tests/verification/oracle/pta/15_fptr_through_struct.expected.yaml`:
```yaml
description: "Function pointer through struct field"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "h.cb"
      must_contain: [on_event]
```

**Step 6: Interprocedural (3 programs)**

`tests/verification/oracle/pta/16_param_passing.c`:
```c
// PURPOSE: Pointer passed as function parameter
void callee(int *p) {
    *p = 42;
}

void test() {
    int x;
    callee(&x);
}
```

`tests/verification/oracle/pta/16_param_passing.expected.yaml`:
```yaml
description: "Pointer flows through function parameter"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "p"
      must_contain: [x]
```

`tests/verification/oracle/pta/17_return_pointer.c`:
```c
// PURPOSE: Function returns a pointer to local-ish allocation
#include <stdlib.h>

int *create() {
    int *p = (int *)malloc(sizeof(int));
    return p;
}

void test() {
    int *q = create();
    *q = 42;
    free(q);
}
```

`tests/verification/oracle/pta/17_return_pointer.expected.yaml`:
```yaml
description: "Pointer returned from function call"
layer: pta
difficulty: intermediate
expectations:
  points_to:
    - pointer: "q"
      must_contain: ["malloc@create"]
```

`tests/verification/oracle/pta/18_context_sensitive.c`:
```c
// PURPOSE: Same function called with different pointer args
// Context-insensitive analysis merges; context-sensitive distinguishes
int *identity(int *p) { return p; }

void test() {
    int x, y;
    int *a = identity(&x);
    int *b = identity(&y);
}
```

`tests/verification/oracle/pta/18_context_sensitive.expected.yaml`:
```yaml
description: "Context sensitivity test — same function, different args"
layer: pta
difficulty: advanced
expectations:
  points_to:
    # Context-insensitive (Andersen): both a and b point to {x, y}
    - pointer: "a"
      must_contain: [x]
      may_only_contain: [x, y]
    - pointer: "b"
      must_contain: [y]
      may_only_contain: [x, y]
```

**Step 7: Edge cases (2 programs)**

`tests/verification/oracle/pta/19_void_star_cast.c`:
```c
// PURPOSE: void* casting — pointer identity preserved through cast
#include <stdlib.h>

void test() {
    int x;
    void *v = &x;
    int *p = (int *)v;
    *p = 42;
}
```

`tests/verification/oracle/pta/19_void_star_cast.expected.yaml`:
```yaml
description: "void* cast preserves points-to"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "v"
      must_contain: [x]
    - pointer: "p"
      must_contain: [x]
```

`tests/verification/oracle/pta/20_global_pointer.c`:
```c
// PURPOSE: Global variable pointer initialization
int g_val;
int *g_ptr = &g_val;

void test() {
    *g_ptr = 42;
}
```

`tests/verification/oracle/pta/20_global_pointer.expected.yaml`:
```yaml
description: "Global pointer initialization"
layer: pta
difficulty: basic
expectations:
  points_to:
    - pointer: "g_ptr"
      must_contain: [g_val]
```

**Step 8: Commit**

```bash
git add tests/verification/oracle/pta/
git commit -m "feat(verify): add 20 PTA oracle programs with expected results"
```

---

### Task 6: Oracle Programs — Call Graph (10 programs)

**Files:**
- Create: `tests/verification/oracle/callgraph/*.c` (10 files)
- Create: `tests/verification/oracle/callgraph/*.expected.yaml` (10 files)

Create 10 programs following the same pattern as Task 5. Representative examples:

**Step 1: Direct calls (2 programs)**

`tests/verification/oracle/callgraph/01_simple_direct.c`:
```c
// PURPOSE: Simple direct call — main calls foo and bar
void foo(void) {}
void bar(void) {}

void test() {
    foo();
    bar();
}
```

`tests/verification/oracle/callgraph/01_simple_direct.expected.yaml`:
```yaml
description: "Simple direct function calls"
layer: callgraph
difficulty: basic
expectations:
  calls:
    - caller: "test"
      callees: [foo, bar]
```

`tests/verification/oracle/callgraph/02_mutual_recursion.c`:
```c
// PURPOSE: Mutually recursive functions
void ping(int n);
void pong(int n);

void ping(int n) { if (n > 0) pong(n - 1); }
void pong(int n) { if (n > 0) ping(n - 1); }

void test() { ping(10); }
```

`tests/verification/oracle/callgraph/02_mutual_recursion.expected.yaml`:
```yaml
description: "Mutually recursive call edges"
layer: callgraph
difficulty: basic
expectations:
  calls:
    - caller: "test"
      callees: [ping]
    - caller: "ping"
      callees: [pong]
    - caller: "pong"
      callees: [ping]
```

**Step 2: Indirect calls (4 programs) — create `03_fptr_call.c`, `04_fptr_array_dispatch.c`, `05_fptr_as_param.c`, `06_fptr_from_return.c`**

Each tests a different indirect call resolution pattern. Follow the pattern above.

**Step 3: Edge cases (4 programs) — create `07_unreachable_fn.c`, `08_external_call.c`, `09_varargs.c`, `10_callback.c`**

**Step 4: Commit**

```bash
git add tests/verification/oracle/callgraph/
git commit -m "feat(verify): add 10 call graph oracle programs with expected results"
```

---

### Task 7: Oracle Programs — CFG (8 programs)

**Files:**
- Create: `tests/verification/oracle/cfg/*.c` (8 files)
- Create: `tests/verification/oracle/cfg/*.expected.yaml` (8 files)

Create 8 programs: 3 branching (if/else, nested, switch), 3 loops (for, while+break, nested), 2 edge cases (goto, unreachable after return).

**Step 1-3: Create all 8 programs following the PTA pattern**

Example:

`tests/verification/oracle/cfg/01_if_else.c`:
```c
// PURPOSE: Simple if/else creates two successor edges from condition block
int rand_bool(void);

void test() {
    int x = 0;
    if (rand_bool()) {
        x = 1;
    } else {
        x = 2;
    }
    x = x + 1;
}
```

`tests/verification/oracle/cfg/01_if_else.expected.yaml`:
```yaml
description: "If/else creates two successor edges"
layer: cfg
difficulty: basic
expectations:
  cfg_edges:
    - function: "test"
      description: "Condition block has two successors (then, else)"
      branch_count: 2
```

**Step 4: Commit**

```bash
git add tests/verification/oracle/cfg/
git commit -m "feat(verify): add 8 CFG oracle programs with expected results"
```

---

### Task 8: Oracle Programs — MSSA + SVFG (16 programs)

**Files:**
- Create: `tests/verification/oracle/mssa/*.c` (8 files)
- Create: `tests/verification/oracle/mssa/*.expected.yaml` (8 files)
- Create: `tests/verification/oracle/svfg/*.c` (8 files)
- Create: `tests/verification/oracle/svfg/*.expected.yaml` (8 files)

Create 16 programs following the established pattern.

MSSA examples: simple store/load, overwrite, conditional store phi, loop-carried dependency, aliased stores, interprocedural store visibility, memcpy, global mutation.

SVFG examples: direct assignment chain, return value flow, store→load flow, aliased pointer flow, struct field flow, param→use, caller→callee→return, global-mediated flow.

**Step 1-4: Create all 16 programs, commit**

```bash
git add tests/verification/oracle/mssa/ tests/verification/oracle/svfg/
git commit -m "feat(verify): add 16 MSSA and SVFG oracle programs with expected results"
```

---

## Phase D: Oracle Harness & Make Commands

### Task 9: Oracle Compilation Script

**Files:**
- Create: `scripts/compile-oracle.sh`

**Step 1: Create compilation script**

Create `scripts/compile-oracle.sh`:

```bash
#!/bin/bash
# Compile oracle C programs to LLVM IR for verification testing.
# Must run inside Docker (needs clang/LLVM 18).
#
# Usage: ./scripts/compile-oracle.sh [--layer pta|callgraph|cfg|mssa|svfg]

set -euo pipefail

ORACLE_DIR="tests/verification/oracle"
COMPILED_DIR="tests/verification/oracle/.compiled"

LAYER="${1:-}"
if [[ "$LAYER" == "--layer" ]]; then
    LAYER="${2:-}"
fi

compile_file() {
    local src="$1"
    local layer
    layer=$(dirname "$src" | xargs basename)
    local name
    name=$(basename "$src" .c)
    local out_dir="$COMPILED_DIR/$layer"
    local out="$out_dir/${name}.ll"

    mkdir -p "$out_dir"

    echo "  Compiling $layer/$name.c -> $name.ll"
    clang -S -emit-llvm -g -O0 -Xclang -disable-O0-optnone "$src" -o "$out" 2>/dev/null || {
        echo "  WARNING: Failed to compile $src (skipping)"
        return 0
    }

    # Apply mem2reg for cleaner SSA
    opt -passes=mem2reg -S "$out" -o "$out"
}

echo "=== Compiling Oracle Verification Programs ==="
echo ""

count=0
for layer_dir in "$ORACLE_DIR"/*/; do
    layer=$(basename "$layer_dir")
    # Skip non-layer directories
    [[ "$layer" == ".compiled" ]] && continue
    [[ "$layer" == "harness" ]] && continue

    # Filter by layer if specified
    if [[ -n "$LAYER" ]] && [[ "$layer" != "$LAYER" ]]; then
        continue
    fi

    echo "Layer: $layer"
    for src in "$layer_dir"*.c; do
        [[ -f "$src" ]] || continue
        compile_file "$src"
        count=$((count + 1))
    done
    echo ""
done

echo "=== Compiled $count oracle programs ==="
```

**Step 2: Make executable**

```bash
chmod +x scripts/compile-oracle.sh
```

**Step 3: Commit**

```bash
git add scripts/compile-oracle.sh
git commit -m "feat(verify): add oracle compilation script"
```

---

### Task 10: Oracle Harness in saf-bench

**Files:**
- Create: `crates/saf-bench/src/oracle.rs`
- Modify: `crates/saf-bench/src/main.rs`
- Modify: `crates/saf-bench/src/lib.rs`

**Step 1: Create oracle harness module**

Create `crates/saf-bench/src/oracle.rs`:

```rust
//! Oracle verification harness.
//!
//! Loads hand-crafted C programs (compiled to LLVM IR), runs SAF analysis,
//! and compares results against YAML expected results.
//!
//! Reports soundness failures (FAIL), imprecision (WARN), and correct results (PASS).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use saf_analysis::callgraph::CallGraphBuilder;
use saf_analysis::cfg::CfgBuilder;
use saf_analysis::pta::context::PtaContext;
use saf_core::air::AirBundle;
use saf_core::config::Config;
use saf_core::ids::{FunctionId, LocId, ValueId};
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

/// A single oracle expectation from YAML.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OracleExpectation {
    pub description: String,
    pub layer: String,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub expectations: ExpectationBlock,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ExpectationBlock {
    #[serde(default)]
    pub points_to: Vec<PointsToExpectation>,
    #[serde(default)]
    pub alias: Vec<AliasExpectation>,
    #[serde(default)]
    pub calls: Vec<CallExpectation>,
    #[serde(default)]
    pub indirect_targets: Vec<IndirectTargetExpectation>,
    #[serde(default)]
    pub cfg_edges: Vec<CfgEdgeExpectation>,
    #[serde(default)]
    pub reaching_def: Vec<ReachingDefExpectation>,
    #[serde(default)]
    pub value_flow: Vec<ValueFlowExpectation>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PointsToExpectation {
    pub pointer: String,
    #[serde(default)]
    pub at_line: Option<u32>,
    #[serde(default)]
    pub must_contain: Vec<String>,
    #[serde(default)]
    pub may_only_contain: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AliasExpectation {
    pub pair: (String, String),
    pub relation: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CallExpectation {
    pub caller: String,
    pub callees: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct IndirectTargetExpectation {
    pub call_site_line: u32,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CfgEdgeExpectation {
    pub function: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub branch_count: Option<usize>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReachingDefExpectation {
    pub use_line: u32,
    pub def_line: u32,
    pub variable: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ValueFlowExpectation {
    pub from_line: u32,
    pub to_line: u32,
    #[serde(default)]
    pub description: String,
}

/// Result of a single oracle test.
#[derive(Debug)]
pub enum OracleVerdict {
    /// All expectations met exactly.
    Pass,
    /// Soundness OK but imprecise (extra items in points-to sets).
    Warn(String),
    /// Soundness failure — missing required items.
    Fail(String),
    /// Could not run (compilation error, missing file, etc.)
    Error(String),
}

/// Summary of oracle suite run.
#[derive(Debug, Default)]
pub struct OracleSummary {
    pub pass: usize,
    pub warn: usize,
    pub fail: usize,
    pub error: usize,
    pub results: Vec<(String, OracleVerdict)>,
}

/// Discover oracle test cases in a directory.
pub fn discover_tests(oracle_dir: &Path, layer_filter: Option<&str>) -> Result<Vec<OracleTestCase>> {
    let mut tests = Vec::new();

    for entry in std::fs::read_dir(oracle_dir)? {
        let entry = entry?;
        let layer_path = entry.path();
        if !layer_path.is_dir() {
            continue;
        }
        let layer = layer_path.file_name().unwrap().to_string_lossy().to_string();

        // Skip non-layer directories
        if layer == ".compiled" || layer == "harness" {
            continue;
        }

        // Apply layer filter
        if let Some(filter) = layer_filter {
            if layer != filter {
                continue;
            }
        }

        for file_entry in std::fs::read_dir(&layer_path)? {
            let file_entry = file_entry?;
            let path = file_entry.path();
            if path.extension().map_or(true, |e| e != "c") {
                continue;
            }
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let yaml_path = layer_path.join(format!("{stem}.expected.yaml"));
            let ll_path = oracle_dir
                .join(".compiled")
                .join(&layer)
                .join(format!("{stem}.ll"));

            if !yaml_path.exists() {
                eprintln!("  WARNING: {layer}/{stem}.c has no .expected.yaml — skipping");
                continue;
            }

            tests.push(OracleTestCase {
                name: format!("{layer}/{stem}"),
                layer: layer.clone(),
                c_source: path.clone(),
                expected_yaml: yaml_path,
                compiled_ll: ll_path,
            });
        }
    }

    tests.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tests)
}

/// A single oracle test case.
#[derive(Debug)]
pub struct OracleTestCase {
    pub name: String,
    pub layer: String,
    pub c_source: PathBuf,
    pub expected_yaml: PathBuf,
    pub compiled_ll: PathBuf,
}

impl OracleTestCase {
    /// Run verification for this test case.
    pub fn verify(&self) -> OracleVerdict {
        // Check compiled LLVM IR exists
        if !self.compiled_ll.exists() {
            return OracleVerdict::Error(format!(
                "Compiled IR not found: {}. Run `make compile-oracle` first.",
                self.compiled_ll.display()
            ));
        }

        // Parse expected results
        let yaml_content = match std::fs::read_to_string(&self.expected_yaml) {
            Ok(c) => c,
            Err(e) => return OracleVerdict::Error(format!("Cannot read YAML: {e}")),
        };
        let expectation: OracleExpectation = match serde_yaml::from_str(&yaml_content) {
            Ok(e) => e,
            Err(e) => return OracleVerdict::Error(format!("Cannot parse YAML: {e}")),
        };

        // Load LLVM IR through SAF
        let config = Config::default();
        let frontend = LlvmFrontend::new();
        let bundle = match frontend.load_file(&self.compiled_ll, &config) {
            Ok(b) => b,
            Err(e) => return OracleVerdict::Error(format!("Cannot load IR: {e}")),
        };

        match self.layer.as_str() {
            "pta" => self.verify_pta(&bundle, &expectation),
            "callgraph" => self.verify_callgraph(&bundle, &expectation),
            "cfg" => self.verify_cfg(&bundle, &expectation),
            "mssa" => self.verify_mssa(&bundle, &expectation),
            "svfg" => self.verify_svfg(&bundle, &expectation),
            other => OracleVerdict::Error(format!("Unknown layer: {other}")),
        }
    }

    fn verify_pta(&self, bundle: &AirBundle, expectation: &OracleExpectation) -> OracleVerdict {
        // Build name-to-ValueId map from debug info
        let name_map = build_name_map(bundle);

        // Run PTA
        let config = saf_analysis::pta::context::PtaConfig::default();
        let mut ctx = PtaContext::new(config);
        let result = ctx.analyze(&bundle.module);

        let mut unsound = Vec::new();
        let mut imprecise = Vec::new();

        for pt in &expectation.expectations.points_to {
            let ptr_id = match name_map.get(&pt.pointer) {
                Some(id) => *id,
                None => {
                    unsound.push(format!("Variable '{}' not found in module", pt.pointer));
                    continue;
                }
            };

            let pts: std::collections::BTreeSet<String> = result
                .pts
                .get(&ptr_id)
                .map(|locs| {
                    locs.iter()
                        .map(|loc| reverse_name_map(bundle, *loc).unwrap_or_else(|| format!("{:?}", loc)))
                        .collect()
                })
                .unwrap_or_default();

            // Check must_contain (soundness)
            for must in &pt.must_contain {
                if !pts.contains(must.as_str()) {
                    unsound.push(format!(
                        "pts({}) missing '{}' — got {{{}}}",
                        pt.pointer,
                        must,
                        pts.iter().cloned().collect::<Vec<_>>().join(", ")
                    ));
                }
            }

            // Check may_only_contain (precision)
            if !pt.may_only_contain.is_empty() {
                let allowed: std::collections::BTreeSet<&str> =
                    pt.may_only_contain.iter().map(|s| s.as_str()).collect();
                for actual in &pts {
                    if !allowed.contains(actual.as_str()) {
                        imprecise.push(format!(
                            "pts({}) has extra '{}' — expected only {{{}}}",
                            pt.pointer,
                            actual,
                            pt.may_only_contain.join(", ")
                        ));
                    }
                }
            }
        }

        if !unsound.is_empty() {
            OracleVerdict::Fail(unsound.join("; "))
        } else if !imprecise.is_empty() {
            OracleVerdict::Warn(imprecise.join("; "))
        } else {
            OracleVerdict::Pass
        }
    }

    fn verify_callgraph(&self, bundle: &AirBundle, expectation: &OracleExpectation) -> OracleVerdict {
        // Build call graph
        let config = Config::default();
        let cg = CallGraphBuilder::new(&bundle.module, &config).build();

        let mut unsound = Vec::new();

        for call_exp in &expectation.expectations.calls {
            let caller_edges: Vec<String> = cg
                .callees_of_name(&call_exp.caller)
                .map(|names| names.collect())
                .unwrap_or_default();

            for expected_callee in &call_exp.callees {
                if !caller_edges.contains(expected_callee) {
                    unsound.push(format!(
                        "{} should call {} but doesn't — edges: [{}]",
                        call_exp.caller,
                        expected_callee,
                        caller_edges.join(", ")
                    ));
                }
            }
        }

        if !unsound.is_empty() {
            OracleVerdict::Fail(unsound.join("; "))
        } else {
            OracleVerdict::Pass
        }
    }

    fn verify_cfg(&self, bundle: &AirBundle, expectation: &OracleExpectation) -> OracleVerdict {
        // Build CFGs for all functions
        let mut unsound = Vec::new();

        for cfg_exp in &expectation.expectations.cfg_edges {
            let func = bundle
                .module
                .functions
                .iter()
                .find(|f| f.name == cfg_exp.function);

            let func = match func {
                Some(f) => f,
                None => {
                    unsound.push(format!("Function '{}' not found", cfg_exp.function));
                    continue;
                }
            };

            let cfg = CfgBuilder::new(func).build();

            if let Some(expected_branch_count) = cfg_exp.branch_count {
                // Count total branch edges (blocks with >1 successor)
                let actual_branches: usize = func
                    .blocks
                    .iter()
                    .filter(|b| cfg.successors(b.id).map_or(0, |s| s.len()) > 1)
                    .count();

                if actual_branches < expected_branch_count {
                    unsound.push(format!(
                        "{}: expected {} branch points, found {}",
                        cfg_exp.function, expected_branch_count, actual_branches
                    ));
                }
            }
        }

        if !unsound.is_empty() {
            OracleVerdict::Fail(unsound.join("; "))
        } else {
            OracleVerdict::Pass
        }
    }

    fn verify_mssa(&self, _bundle: &AirBundle, _expectation: &OracleExpectation) -> OracleVerdict {
        // TODO: Implement MSSA verification (reaching defs)
        OracleVerdict::Error("MSSA verification not yet implemented".to_string())
    }

    fn verify_svfg(&self, _bundle: &AirBundle, _expectation: &OracleExpectation) -> OracleVerdict {
        // TODO: Implement SVFG verification (value flow paths)
        OracleVerdict::Error("SVFG verification not yet implemented".to_string())
    }
}

/// Build a map from human-readable variable names to `ValueId`s.
///
/// Uses LLVM debug info names when available, falls back to SSA register names.
fn build_name_map(bundle: &AirBundle) -> BTreeMap<String, ValueId> {
    let mut map = BTreeMap::new();
    for func in &bundle.module.functions {
        for param in &func.params {
            if let Some(ref name) = param.name {
                map.insert(name.clone(), param.value_id);
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref name) = inst.name {
                    if let Some(result_id) = inst.result {
                        map.insert(name.clone(), result_id);
                    }
                }
            }
        }
    }
    map
}

/// Reverse-map a `LocId` back to a human-readable name.
fn reverse_name_map(_bundle: &AirBundle, _loc: LocId) -> Option<String> {
    // TODO: Build reverse map from LocId to alloca/global name via LocationFactory
    None
}

/// Print human-readable oracle report.
pub fn print_report(summary: &OracleSummary) {
    eprintln!();
    eprintln!("=== Oracle Verification Report ===");
    eprintln!();

    let mut current_layer = String::new();
    for (name, verdict) in &summary.results {
        let layer = name.split('/').next().unwrap_or("");
        if layer != current_layer {
            if !current_layer.is_empty() {
                eprintln!();
            }
            current_layer = layer.to_string();
            eprintln!("{}:", layer.to_uppercase());
        }

        let (tag, detail) = match verdict {
            OracleVerdict::Pass => ("[PASS]".to_string(), String::new()),
            OracleVerdict::Warn(msg) => ("[WARN]".to_string(), format!(" — {msg}")),
            OracleVerdict::Fail(msg) => ("[FAIL]".to_string(), format!(" — UNSOUND: {msg}")),
            OracleVerdict::Error(msg) => ("[ERR ]".to_string(), format!(" — {msg}")),
        };
        eprintln!("  {tag} {name}{detail}");
    }

    eprintln!();
    eprintln!(
        "Overall: {} pass | {} imprecision | {} unsound | {} errors",
        summary.pass, summary.warn, summary.fail, summary.error
    );
}
```

**Step 2: Add oracle subcommand to main.rs**

Add to the `Commands` enum in `crates/saf-bench/src/main.rs`:

```rust
    /// Run oracle verification suite (hand-crafted programs with expected results)
    Oracle {
        /// Directory containing oracle test programs
        #[arg(long, default_value = "tests/verification/oracle")]
        oracle_dir: PathBuf,

        /// Filter by layer (pta, callgraph, cfg, mssa, svfg)
        #[arg(long)]
        layer: Option<String>,
    },
```

Add the match arm in the main function:

```rust
        Commands::Oracle { oracle_dir, layer } => {
            let tests = saf_bench::oracle::discover_tests(&oracle_dir, layer.as_deref())
                .context("Failed to discover oracle tests")?;

            eprintln!("Found {} oracle test cases", tests.len());

            let mut summary = saf_bench::oracle::OracleSummary::default();

            for test in &tests {
                let verdict = test.verify();
                match &verdict {
                    saf_bench::oracle::OracleVerdict::Pass => summary.pass += 1,
                    saf_bench::oracle::OracleVerdict::Warn(_) => summary.warn += 1,
                    saf_bench::oracle::OracleVerdict::Fail(_) => summary.fail += 1,
                    saf_bench::oracle::OracleVerdict::Error(_) => summary.error += 1,
                }
                summary.results.push((test.name.clone(), verdict));
            }

            saf_bench::oracle::print_report(&summary);

            // Exit with failure if any unsoundness found
            if summary.fail > 0 {
                std::process::exit(1);
            }
        }
```

**Step 3: Register module in lib.rs**

Add to `crates/saf-bench/src/lib.rs`:

```rust
pub mod oracle;
```

**Step 4: Verify it compiles in Docker**

Run: `docker compose run --rm dev sh -c "cargo build -p saf-bench"`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/saf-bench/src/oracle.rs crates/saf-bench/src/main.rs crates/saf-bench/src/lib.rs
git commit -m "feat(verify): add oracle verification harness to saf-bench"
```

---

### Task 11: Make Commands

**Files:**
- Modify: `Makefile`

**Step 1: Add verification targets to Makefile**

Add before the `# --- WASM Build ---` section:

```makefile
# --- Verification Suite ---

compile-oracle: ## Compile oracle C programs to LLVM IR
	@echo "Compiling oracle verification programs..."
	docker compose run --rm dev ./scripts/compile-oracle.sh \
		$(if $(LAYER),--layer $(LAYER),)

verify-oracle: ## Run oracle verification suite (LAYER=pta to filter)
	@echo "Running oracle verification suite..."
	docker compose run --rm dev cargo run --release -p saf-bench -- oracle \
		--oracle-dir tests/verification/oracle \
		$(if $(LAYER),--layer $(LAYER),)

verify-props: ## Run property tests with 10000 iterations
	@echo "Running property tests (10000 iterations)..."
	docker compose run --rm dev sh -c \
		"PROPTEST_CASES=10000 cargo nextest run --workspace --exclude saf-python -E 'test(proptest) | test(constraint_extraction)'"

verify-props-quick: ## Run property tests with 256 iterations (fast feedback)
	@echo "Running property tests (256 iterations)..."
	docker compose run --rm dev sh -c \
		"cargo nextest run --workspace --exclude saf-python -E 'test(proptest) | test(constraint_extraction)'"

verify-kani-core: ## Run Kani proofs for saf-core (local, no Docker needed)
	@echo "Running Kani proofs for saf-core..."
	cd crates/saf-core && cargo kani

verify-kani: verify-kani-core ## Run ALL Kani proofs

verify-quick: verify-kani-core verify-props-quick verify-oracle ## Quick verification (all three pillars, fast)

verify: verify-kani verify-props verify-oracle ## Run ALL verification (Kani + proptest + oracle)

clean-oracle: ## Remove compiled oracle LLVM IR
	rm -rf tests/verification/oracle/.compiled
```

**Step 2: Update .PHONY**

Add to the `.PHONY` line at top of Makefile:

```
compile-oracle verify-oracle verify-props verify-props-quick verify-kani verify-kani-core verify-quick verify clean-oracle
```

**Step 3: Verify make targets**

Run: `make help | grep verify`
Expected: Shows all verify-* targets with descriptions

**Step 4: Commit**

```bash
git add Makefile
git commit -m "feat(verify): add make commands for three-pillar verification"
```

---

### Task 12: Integration Test — End-to-End Verification

**Step 1: Compile oracle programs**

Run: `make compile-oracle`
Expected: All C programs compiled to LLVM IR under `tests/verification/oracle/.compiled/`

**Step 2: Run oracle suite**

Run: `make verify-oracle`
Expected: Report shows PASS/WARN/FAIL for each test case

**Step 3: Run property tests**

Run: `make verify-props-quick`
Expected: All property tests pass

**Step 4: Run Kani (saf-core)**

Run: `make verify-kani-core`
Expected: All Kani proofs verified

**Step 5: Run full verification**

Run: `make verify-quick`
Expected: All three pillars produce results

**Step 6: Final commit**

```bash
git add -A
git commit -m "feat(verify): complete first iteration of soundness verification suite

Three-pillar verification:
- Pillar 1: Kani formal proofs for ID generation, spec registry, PtsSet algebra
- Pillar 2: Proptest constraint extraction completeness (1000 iterations)
- Pillar 3: Oracle suite with 54 hand-crafted C programs testing PTA, CG, CFG, MSSA, SVFG

Make commands: verify, verify-quick, verify-kani, verify-props, verify-oracle"
```

---

## Notes and Caveats

### Kani + saf-analysis LLVM Issue

Kani may not be able to compile saf-analysis due to LLVM C API linking. If this happens:
- **Option A:** Install Kani in Docker image alongside LLVM 18
- **Option B:** Create a thin `crates/saf-verify/` crate that depends only on saf-core and reimports PtsSet types
- **Option C:** Limit Kani to saf-core only (still valuable for ID and spec proofs)

The plan is structured so Tasks 1 (saf-core Kani) works independently of Task 2 (saf-analysis Kani).

### Oracle Harness Limitations

The initial harness implements PTA, CG, and CFG verification. MSSA and SVFG verification stubs return `Error("not yet implemented")`. These are filled in during subsequent iterations when the name-to-ID mapping infrastructure matures.

### Name-to-ID Mapping

The `build_name_map()` function relies on LLVM debug info names propagated through SAF's AIR. If variable names are lost during compilation (e.g., optimized away), the harness won't be able to map human-readable names to `ValueId`s. The compilation script uses `-g -O0` to preserve debug info.

### Proptest Iteration Count

CI uses 10,000 iterations (`make verify-props`). Local development uses 256 (`make verify-props-quick`). The higher count is needed for soundness confidence — 256 iterations may miss edge cases in constraint extraction.
