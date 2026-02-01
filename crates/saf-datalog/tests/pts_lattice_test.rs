use std::sync::Arc;

use ascent::lattice::Lattice;
use saf_core::id::make_id;
use saf_core::ids::{LocId, ValueId};
use saf_datalog::facts::PtaFacts;
use saf_datalog::pta::{AscentPtsSet, LocIdRegistry, with_registry};

fn loc(name: &str) -> LocId {
    LocId::new(make_id("loc", name.as_bytes()))
}

fn setup_registry(locs: &[LocId]) -> Arc<LocIdRegistry> {
    let val = ValueId::new(make_id("val", b"ptr"));
    let facts = PtaFacts {
        addr_of: locs.iter().map(|l| (val, *l)).collect(),
        ..Default::default()
    };
    Arc::new(LocIdRegistry::from_facts(&facts))
}

#[test]
fn test_singleton_and_union() {
    let loc_a = loc("a");
    let loc_b = loc("b");
    let reg = setup_registry(&[loc_a, loc_b]);

    with_registry(reg, || {
        let a = AscentPtsSet::singleton(loc_a);
        let b = AscentPtsSet::singleton(loc_b);
        let merged = a.join(b);

        assert!(merged.contains(loc_a));
        assert!(merged.contains(loc_b));
        assert_eq!(merged.len(), 2);
    });
}

#[test]
fn test_lattice_partial_ord() {
    let loc_a = loc("a");
    let loc_b = loc("b");
    let reg = setup_registry(&[loc_a, loc_b]);

    with_registry(reg, || {
        let a = AscentPtsSet::singleton(loc_a);
        let ab = AscentPtsSet::collect_from([loc_a, loc_b]);

        assert!(a <= ab);
        assert!(!(ab <= a));
    });
}

#[test]
fn test_empty_is_bottom() {
    let la = loc("a");
    let reg = setup_registry(&[la]);

    with_registry(reg, || {
        let empty = AscentPtsSet::empty();
        let a = AscentPtsSet::singleton(la);
        assert!(empty <= a);
    });
}

#[test]
fn test_join_mut_returns_changed() {
    let loc_a = loc("a");
    let loc_b = loc("b");
    let reg = setup_registry(&[loc_a, loc_b]);

    with_registry(reg, || {
        let mut a = AscentPtsSet::singleton(loc_a);
        let b = AscentPtsSet::singleton(loc_b);
        assert!(a.join_mut(b)); // changed

        let c = AscentPtsSet::singleton(loc_a);
        assert!(!a.join_mut(c)); // not changed, loc_a already in set
    });
}

#[test]
fn test_meet_mut_returns_changed() {
    let loc_a = loc("a");
    let loc_b = loc("b");
    let reg = setup_registry(&[loc_a, loc_b]);

    with_registry(reg, || {
        let mut ab = AscentPtsSet::collect_from([loc_a, loc_b]);
        let a_only = AscentPtsSet::singleton(loc_a);
        assert!(ab.meet_mut(a_only)); // changed (removed loc_b)
        assert_eq!(ab.len(), 1);
        assert!(ab.contains(loc_a));
    });
}

#[test]
fn test_incomparable_sets() {
    let loc_a = loc("a");
    let loc_b = loc("b");
    let reg = setup_registry(&[loc_a, loc_b]);

    with_registry(reg, || {
        let a = AscentPtsSet::singleton(loc_a);
        let b = AscentPtsSet::singleton(loc_b);
        assert!(a.partial_cmp(&b).is_none());
    });
}

#[test]
fn test_into_btreeset() {
    let loc_a = loc("a");
    let loc_b = loc("b");
    let reg = setup_registry(&[loc_a, loc_b]);

    with_registry(reg, || {
        let pts = AscentPtsSet::collect_from([loc_a, loc_b]);
        let set = pts.into_btreeset();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&loc_a));
        assert!(set.contains(&loc_b));
    });
}

#[test]
fn test_default_is_empty() {
    let la = loc("a");
    let reg = setup_registry(&[la]);

    with_registry(reg, || {
        let pts = AscentPtsSet::default();
        assert!(pts.is_empty());
        assert_eq!(pts.len(), 0);
    });
}

#[test]
fn test_meet_with_empty_produces_empty() {
    let loc_a = loc("a");
    let reg = setup_registry(&[loc_a]);

    with_registry(reg, || {
        let mut pts = AscentPtsSet::singleton(loc_a);
        let empty = AscentPtsSet::empty();
        assert!(pts.meet_mut(empty));
        assert!(pts.is_empty());
    });
}

#[test]
fn test_join_with_empty_unchanged() {
    let loc_a = loc("a");
    let reg = setup_registry(&[loc_a]);

    with_registry(reg, || {
        let mut pts = AscentPtsSet::singleton(loc_a);
        let empty = AscentPtsSet::empty();
        assert!(!pts.join_mut(empty)); // no change
        assert_eq!(pts.len(), 1);
    });
}
