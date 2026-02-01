//! Basic tests for the Ascent-based Andersen's PTA solver.

use std::sync::Arc;

use saf_core::id::make_id;
use saf_core::ids::{LocId, ValueId};
use saf_datalog::facts::PtaFacts;
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
fn test_addr_constraint() {
    let p = val("p");
    let x = loc("x");

    let facts = PtaFacts {
        addr_of: vec![(p, x)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let p_pts = result.get(&p).expect("p should have points-to set");
    assert!(p_pts.contains(&x));
    assert_eq!(p_pts.len(), 1);
}

#[test]
fn test_copy_propagation() {
    let p = val("p");
    let q = val("q");
    let x = loc("x");

    let facts = PtaFacts {
        addr_of: vec![(p, x)],
        copy: vec![(q, p)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let q_pts = result.get(&q).expect("q should have points-to set");
    assert!(q_pts.contains(&x));
}

#[test]
fn test_store_load_cycle() {
    let p = val("p");
    let q = val("q");
    let r = val("r");
    let x = loc("x");
    let y = loc("y");

    let facts = PtaFacts {
        addr_of: vec![(p, x), (q, y)],
        store: vec![(q, p)],
        load: vec![(r, q)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let r_pts = result.get(&r).expect("r should have points-to set");
    assert!(r_pts.contains(&x), "r should transitively point to x");
}

#[test]
fn test_transitive_copy() {
    let p = val("p");
    let q = val("q");
    let r = val("r");
    let x = loc("x");

    let facts = PtaFacts {
        addr_of: vec![(p, x)],
        copy: vec![(q, p), (r, q)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let r_pts = result.get(&r).expect("r should have points-to set");
    assert!(r_pts.contains(&x));
}

#[test]
fn test_multiple_addr_of() {
    let p = val("p");
    let x = loc("x");
    let y = loc("y");

    let facts = PtaFacts {
        addr_of: vec![(p, x), (p, y)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let p_pts = result.get(&p).expect("p should have points-to set");
    assert!(p_pts.contains(&x));
    assert!(p_pts.contains(&y));
    assert_eq!(p_pts.len(), 2);
}

#[test]
fn test_empty_facts() {
    let facts = PtaFacts::default();
    let result = solve_with_registry(&facts);
    assert!(result.is_empty());
}

#[test]
fn test_copy_merges_multiple_sources() {
    let p = val("p");
    let q = val("q");
    let r = val("r");
    let x = loc("x");
    let y = loc("y");

    let facts = PtaFacts {
        addr_of: vec![(p, x), (q, y)],
        copy: vec![(r, p), (r, q)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let r_pts = result.get(&r).expect("r should have points-to set");
    assert!(r_pts.contains(&x));
    assert!(r_pts.contains(&y));
    assert_eq!(r_pts.len(), 2);
}

#[test]
fn test_store_load_diamond() {
    let a = val("a");
    let b = val("b");
    let ptr = val("ptr");
    let c = val("c");
    let obj1 = loc("obj1");
    let obj2 = loc("obj2");
    let slot = loc("slot");

    let facts = PtaFacts {
        addr_of: vec![(a, obj1), (b, obj2), (ptr, slot)],
        store: vec![(ptr, a), (ptr, b)],
        load: vec![(c, ptr)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let c_pts = result.get(&c).expect("c should have points-to set");
    assert!(c_pts.contains(&obj1));
    assert!(c_pts.contains(&obj2));
    assert_eq!(c_pts.len(), 2);
}

#[test]
fn test_chain_store_load_store_load() {
    let a = val("a");
    let b = val("b");
    let c = val("c");
    let p1 = val("p1");
    let p2 = val("p2");
    let obj = loc("obj");
    let slot1 = loc("slot1");
    let slot2 = loc("slot2");

    let facts = PtaFacts {
        addr_of: vec![(a, obj), (p1, slot1), (p2, slot2)],
        store: vec![(p1, a), (p2, b)],
        load: vec![(b, p1), (c, p2)],
        ..Default::default()
    };

    let result = solve_with_registry(&facts);
    let c_pts = result.get(&c).expect("c should have points-to set");
    assert!(
        c_pts.contains(&obj),
        "c should transitively point to obj through two store/load pairs"
    );
}
