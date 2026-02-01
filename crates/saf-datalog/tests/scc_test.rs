use saf_core::id::make_id;
use saf_core::ids::{LocId, ValueId};
use saf_datalog::facts::PtaFacts;
use saf_datalog::pta::scc::{detect_scc, rewrite_facts_with_scc};

fn val(name: &str) -> ValueId {
    ValueId::new(make_id("val", name.as_bytes()))
}

fn loc(name: &str) -> LocId {
    LocId::new(make_id("loc", name.as_bytes()))
}

#[test]
fn test_cycle_detection() {
    let a = val("a");
    let b = val("b");
    let c = val("c");

    // a -> b -> c -> a (cycle): copy edges are (dst, src)
    let copy_edges = vec![(b, a), (c, b), (a, c)];
    let result = detect_scc(&copy_edges);

    // All three should map to same representative
    let rep_a = result.representatives.get(&a).copied().unwrap_or(a);
    let rep_b = result.representatives.get(&b).copied().unwrap_or(b);
    let rep_c = result.representatives.get(&c).copied().unwrap_or(c);
    assert_eq!(rep_a, rep_b);
    assert_eq!(rep_b, rep_c);
    assert_eq!(result.num_sccs, 1);
    assert_eq!(result.collapsed_count, 2);
}

#[test]
fn test_no_cycles() {
    let a = val("a");
    let b = val("b");
    let c = val("c");

    // a -> b -> c (no cycle): copy edges are (dst, src)
    let copy_edges = vec![(b, a), (c, b)];
    let result = detect_scc(&copy_edges);

    assert!(result.representatives.is_empty());
    assert_eq!(result.num_sccs, 0);
    assert_eq!(result.collapsed_count, 0);
}

#[test]
fn test_two_separate_sccs() {
    let a = val("a");
    let b = val("b");
    let c = val("c");
    let d = val("d");

    // a <-> b, c <-> d (two 2-cycles)
    let copy_edges = vec![(b, a), (a, b), (d, c), (c, d)];
    let result = detect_scc(&copy_edges);

    assert_eq!(result.num_sccs, 2);

    let rep_a = result.representatives.get(&a).copied().unwrap_or(a);
    let rep_b = result.representatives.get(&b).copied().unwrap_or(b);
    assert_eq!(rep_a, rep_b);

    let rep_c = result.representatives.get(&c).copied().unwrap_or(c);
    let rep_d = result.representatives.get(&d).copied().unwrap_or(d);
    assert_eq!(rep_c, rep_d);

    // The two SCCs should have different representatives
    assert_ne!(rep_a, rep_c);
}

#[test]
fn test_rewrite_facts_removes_self_copies() {
    let a = val("a");
    let b = val("b");
    let x = loc("x");

    let mut facts = PtaFacts {
        addr_of: vec![(a, x)],
        copy: vec![(b, a), (a, b)], // cycle: a <-> b
        ..Default::default()
    };

    let scc_result = detect_scc(&facts.copy);
    assert_eq!(scc_result.num_sccs, 1);

    rewrite_facts_with_scc(&mut facts, &scc_result.representatives);

    // Self-copies should be removed after rewriting
    assert!(
        facts.copy.iter().all(|(d, s)| d != s),
        "self-copies should be removed"
    );

    // addr_of should be rewritten to use the representative
    let rep = *[a, b].iter().min().unwrap();
    assert!(facts.addr_of.iter().any(|(ptr, _)| *ptr == rep));
}

#[test]
fn test_rewrite_facts_rewrites_load_store_gep() {
    let a = val("a");
    let b = val("b");
    let c = val("c");
    let x = loc("x");

    let mut facts = PtaFacts {
        addr_of: vec![(a, x)],
        copy: vec![(b, a), (a, b)], // cycle: a <-> b
        load: vec![(c, a)],
        store: vec![(b, c)],
        gep: vec![],
    };

    let scc_result = detect_scc(&facts.copy);
    let rep = *[a, b].iter().min().unwrap();
    rewrite_facts_with_scc(&mut facts, &scc_result.representatives);

    // load src should be rewritten to rep
    assert_eq!(facts.load.len(), 1);
    assert_eq!(facts.load[0].1, rep);

    // store dst_ptr should be rewritten to rep
    assert_eq!(facts.store.len(), 1);
    assert_eq!(facts.store[0].0, rep);
}

#[test]
fn test_empty_edges() {
    let result = detect_scc(&[]);
    assert!(result.representatives.is_empty());
    assert_eq!(result.num_sccs, 0);
    assert_eq!(result.collapsed_count, 0);
}

#[test]
fn test_self_loop() {
    let a = val("a");

    // a -> a (self-loop is a trivial SCC, should be ignored)
    let copy_edges = vec![(a, a)];
    let result = detect_scc(&copy_edges);

    // Self-loops form a trivial SCC (size 1), not collapsed
    assert!(result.representatives.is_empty());
    assert_eq!(result.num_sccs, 0);
    assert_eq!(result.collapsed_count, 0);
}

#[test]
fn test_deterministic_representative() {
    let a = val("a");
    let b = val("b");
    let c = val("c");

    // Run twice to verify determinism
    let copy_edges = vec![(b, a), (c, b), (a, c)];
    let result1 = detect_scc(&copy_edges);
    let result2 = detect_scc(&copy_edges);

    assert_eq!(result1.representatives, result2.representatives);
    assert_eq!(result1.num_sccs, result2.num_sccs);
    assert_eq!(result1.collapsed_count, result2.collapsed_count);

    // Representative should be the minimum ValueId
    let min_val = *[a, b, c].iter().min().unwrap();
    for &v in &[a, b, c] {
        let rep = result1.representatives.get(&v).copied().unwrap_or(v);
        assert_eq!(rep, min_val);
    }
}

#[test]
fn test_rewrite_empty_reps_is_noop() {
    let a = val("a");
    let b = val("b");
    let x = loc("x");

    let mut facts = PtaFacts {
        addr_of: vec![(a, x)],
        copy: vec![(b, a)],
        ..Default::default()
    };

    let original_facts = facts.clone();
    let empty_reps = std::collections::BTreeMap::new();
    rewrite_facts_with_scc(&mut facts, &empty_reps);

    assert_eq!(facts.addr_of, original_facts.addr_of);
    assert_eq!(facts.copy, original_facts.copy);
}

#[test]
fn test_chain_with_back_edge() {
    // a -> b -> c -> d -> b (SCC is {b, c, d}, a is outside)
    let a = val("a");
    let b = val("b");
    let c = val("c");
    let d = val("d");

    let copy_edges = vec![(b, a), (c, b), (d, c), (b, d)];
    let result = detect_scc(&copy_edges);

    assert_eq!(result.num_sccs, 1);

    // a should NOT be in the SCC
    assert!(!result.representatives.contains_key(&a));

    // b, c, d should share a representative
    let rep_b = result.representatives.get(&b).copied().unwrap_or(b);
    let rep_c = result.representatives.get(&c).copied().unwrap_or(c);
    let rep_d = result.representatives.get(&d).copied().unwrap_or(d);
    assert_eq!(rep_b, rep_c);
    assert_eq!(rep_c, rep_d);
}
