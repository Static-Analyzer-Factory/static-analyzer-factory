//! Tests for HVN preprocessing on Ascent PTA facts.

use std::sync::Arc;

use saf_core::id::make_id;
use saf_core::ids::{LocId, ValueId};
use saf_datalog::facts::PtaFacts;
use saf_datalog::pta::hvn::preprocess_hvn;
use saf_datalog::pta::{LocIdRegistry, ascent_solve, with_registry};

fn val(name: &str) -> ValueId {
    ValueId::new(make_id("val", name.as_bytes()))
}

fn loc(name: &str) -> LocId {
    LocId::new(make_id("loc", name.as_bytes()))
}

fn solve_with_registry(facts: &PtaFacts) -> saf_datalog::pta::PointsToMap {
    let registry = Arc::new(LocIdRegistry::from_facts(facts));
    with_registry(registry, || ascent_solve(facts))
}

#[test]
fn test_hvn_merges_addr_only_equivalent_values() {
    // p = &x; q = &x  =>  p and q have identical AddrOnly signatures
    let p = val("p");
    let q = val("q");
    let x = loc("x");

    let mut facts = PtaFacts {
        addr_of: vec![(p, x), (q, x)],
        ..Default::default()
    };

    let original_total = facts.total();
    let result = preprocess_hvn(&mut facts);

    // HVN should merge p and q (same AddrOnly sig)
    assert!(result.num_classes() > 0);
    assert!(facts.total() < original_total);
    // Only one addr_of fact should remain (the representative)
    assert_eq!(facts.addr_of.len(), 1);
}

#[test]
fn test_hvn_empty_facts() {
    let mut facts = PtaFacts::default();
    let result = preprocess_hvn(&mut facts);
    assert_eq!(result.removed(), 0);
    assert_eq!(result.num_classes(), 0);
    assert!(facts.is_empty());
}

#[test]
fn test_hvn_does_not_merge_different_signatures() {
    // p = &x; q = &y  =>  different AddrOnly signatures, no merge
    let p = val("p");
    let q = val("q");
    let x = loc("x");
    let y = loc("y");

    let mut facts = PtaFacts {
        addr_of: vec![(p, x), (q, y)],
        ..Default::default()
    };

    let result = preprocess_hvn(&mut facts);
    assert_eq!(result.num_classes(), 0);
    assert_eq!(result.removed(), 0);
    assert_eq!(facts.addr_of.len(), 2);
}

#[test]
fn test_hvn_preserves_complex_constraints() {
    // Load/store constraints make values Complex — they should not be merged
    let p = val("p");
    let q = val("q");
    let r = val("r");
    let s = val("s");

    let mut facts = PtaFacts {
        load: vec![(p, q)],
        store: vec![(r, s)],
        ..Default::default()
    };

    let original_total = facts.total();
    let result = preprocess_hvn(&mut facts);

    assert_eq!(result.num_classes(), 0);
    assert_eq!(facts.total(), original_total);
}

#[test]
fn test_hvn_result_expansion() {
    let p = val("p");
    let q = val("q");
    let x = loc("x");

    let mut facts = PtaFacts {
        addr_of: vec![(p, x), (q, x)],
        ..Default::default()
    };

    let hvn_result = preprocess_hvn(&mut facts);

    let mut pts = solve_with_registry(&facts);

    let has_both_before = pts.contains_key(&p) && pts.contains_key(&q);

    hvn_result.expand_results(&mut pts);

    let p_pts = pts
        .get(&p)
        .expect("p should have points-to set after expansion");
    let q_pts = pts
        .get(&q)
        .expect("q should have points-to set after expansion");
    assert!(p_pts.contains(&x));
    assert!(q_pts.contains(&x));

    if !has_both_before {
        assert!(hvn_result.num_classes() > 0);
    }
}

#[test]
fn test_hvn_copy_target_merge() {
    // p = &x; q = p; r = p  =>  q and r have same CopyTarget signature
    let p = val("p");
    let q = val("q");
    let r = val("r");
    let x = loc("x");

    let mut facts = PtaFacts {
        addr_of: vec![(p, x)],
        copy: vec![(q, p), (r, p)],
        ..Default::default()
    };

    let original_total = facts.total();
    let result = preprocess_hvn(&mut facts);

    // q and r should be merged (same CopyTarget{p} signature)
    assert!(result.num_classes() > 0);
    assert!(facts.total() < original_total);
}

#[test]
fn test_hvn_roundtrip_preserves_semantics() {
    let p = val("p");
    let q = val("q");
    let r = val("r");
    let x = loc("x");

    let facts_orig = PtaFacts {
        addr_of: vec![(p, x), (q, x)],
        copy: vec![(r, p)],
        ..Default::default()
    };

    let result_no_hvn = solve_with_registry(&facts_orig);

    let mut facts_hvn = facts_orig.clone();
    let hvn_result = preprocess_hvn(&mut facts_hvn);
    let mut result_hvn = solve_with_registry(&facts_hvn);
    hvn_result.expand_results(&mut result_hvn);

    for (vid, pts_no_hvn) in &result_no_hvn {
        let pts_hvn = result_hvn
            .get(vid)
            .unwrap_or_else(|| panic!("HVN result missing entry for {vid:?}"));
        assert_eq!(pts_no_hvn, pts_hvn, "Points-to sets differ for {vid:?}");
    }
}
