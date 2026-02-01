//! Ascent-based Andersen's pointer analysis solver.
//!
//! Encodes the four core Andersen constraint rules (addr, copy, store, load)
//! as Datalog rules using Ascent's `ascent!` macro with lattice-based
//! fixpoint computation. The points-to sets use [`AscentPtsSet`] which
//! implements `Lattice` with set-union as join.
//!
//! When the `parallel` feature is enabled (and not targeting WASM),
//! [`ascent_solve`] dispatches to a parallel Ascent program using `ascent_par!`
//! for concurrent fixpoint evaluation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use ascent::ascent;
#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
use ascent::ascent_par;
use saf_core::ids::{LocId, ValueId};

use crate::facts::PtaFacts;
use crate::pta::AscentPtsSet;

/// Output type matching SAF's PTA result format.
pub type PointsToMap = BTreeMap<ValueId, BTreeSet<LocId>>;

ascent! {
    struct PtaProgram;

    // --- Input relations ---

    /// Address-of constraint: pointer `p` is assigned address of location `loc`.
    relation addr_of(ValueId, LocId);

    /// Copy constraint: `dst = src` (pointer assignment).
    relation copy_edge(ValueId, ValueId);

    /// Load constraint: `dst = *src_ptr` (pointer dereference read).
    relation load_edge(ValueId, ValueId);

    /// Store constraint: `*dst_ptr = src` (pointer dereference write).
    relation store_edge(ValueId, ValueId);

    // --- Derived: points-to as lattice ---

    /// Points-to lattice relation: maps each value to its points-to set.
    lattice points_to(ValueId, AscentPtsSet);

    // Addr rule: p = &x  =>  pts(p) ⊇ {x}
    points_to(*p, AscentPtsSet::singleton(*loc)) <--
        addr_of(p, loc);

    // Copy rule: dst = src  =>  pts(dst) ⊇ pts(src)
    points_to(*dst, pts.clone()) <--
        copy_edge(dst, src),
        points_to(src, pts);

    // Store intermediate: *dst_ptr = src  =>  for each loc ∈ pts(dst_ptr),
    // record that src flows into loc.
    relation store_target(LocId, ValueId);
    store_target(loc, *src) <--
        store_edge(dst_ptr, src),
        points_to(dst_ptr, pts),
        for loc in pts.iter();

    /// Per-location points-to: what each abstract location contains.
    lattice loc_pts(LocId, AscentPtsSet);
    loc_pts(*loc, src_pts.clone()) <--
        store_target(loc, src),
        points_to(src, src_pts);

    // Load rule: dst = *src_ptr  =>  for each loc ∈ pts(src_ptr): pts(dst) ⊇ loc_pts(loc)
    points_to(*dst, loc_set.clone()) <--
        load_edge(dst, src_ptr),
        points_to(src_ptr, ptr_pts),
        for loc in ptr_pts.iter(),
        loc_pts(loc, loc_set);
}

// --- Parallel Ascent program (feature-gated) ---

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
ascent_par! {
    struct PtaProgramPar;

    // --- Input relations (same as sequential) ---

    /// Address-of constraint: pointer `p` is assigned address of location `loc`.
    relation addr_of(ValueId, LocId);

    /// Copy constraint: `dst = src` (pointer assignment).
    relation copy_edge(ValueId, ValueId);

    /// Load constraint: `dst = *src_ptr` (pointer dereference read).
    relation load_edge(ValueId, ValueId);

    /// Store constraint: `*dst_ptr = src` (pointer dereference write).
    relation store_edge(ValueId, ValueId);

    // --- Derived: points-to as lattice ---

    /// Points-to lattice relation: maps each value to its points-to set.
    lattice points_to(ValueId, AscentPtsSet);

    // Addr rule: p = &x  =>  pts(p) ⊇ {x}
    points_to(*p, AscentPtsSet::singleton(*loc)) <--
        addr_of(p, loc);

    // Copy rule: dst = src  =>  pts(dst) ⊇ pts(src)
    points_to(*dst, pts.clone()) <--
        copy_edge(dst, src),
        points_to(src, pts);

    // Store intermediate: *dst_ptr = src  =>  for each loc ∈ pts(dst_ptr),
    // record that src flows into loc.
    relation store_target(LocId, ValueId);
    store_target(loc, *src) <--
        store_edge(dst_ptr, src),
        points_to(dst_ptr, pts),
        for loc in pts.iter();

    /// Per-location points-to: what each abstract location contains.
    lattice loc_pts(LocId, AscentPtsSet);
    loc_pts(*loc, src_pts.clone()) <--
        store_target(loc, src),
        points_to(src, src_pts);

    // Load rule: dst = *src_ptr  =>  for each loc ∈ pts(src_ptr): pts(dst) ⊇ loc_pts(loc)
    points_to(*dst, loc_set.clone()) <--
        load_edge(dst, src_ptr),
        points_to(src_ptr, ptr_pts),
        for loc in ptr_pts.iter(),
        loc_pts(loc, loc_set);
}

/// Run Ascent-based Andersen's pointer analysis.
///
/// Takes flat [`PtaFacts`] and returns a [`PointsToMap`] compatible with SAF's
/// existing PTA result format.
///
/// When the `parallel` feature is enabled (and not targeting WASM), this
/// dispatches to the parallel `ascent_par!` solver for concurrent evaluation.
///
/// # Arguments
///
/// * `facts` — The extracted PTA facts (addr, copy, load, store constraints).
///
/// # Returns
///
/// A `BTreeMap` mapping each `ValueId` to its points-to set of `LocId`s.
pub fn ascent_solve(facts: &PtaFacts) -> PointsToMap {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        ascent_solve_par(facts)
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        ascent_solve_seq(facts)
    }
}

/// Sequential solver using `ascent!`.
// NOTE: Ascent-generated struct only exposes Default::default();
// field assignment is the intended way to populate input relations.
#[allow(clippy::field_reassign_with_default)]
fn ascent_solve_seq(facts: &PtaFacts) -> PointsToMap {
    let mut prog = PtaProgram::default();

    // Load input relations from facts
    prog.addr_of = facts.addr_of.iter().map(|(p, l)| (*p, *l)).collect();
    prog.copy_edge = facts.copy.iter().map(|(d, s)| (*d, *s)).collect();
    prog.load_edge = facts.load.iter().map(|(d, s)| (*d, *s)).collect();
    prog.store_edge = facts.store.iter().map(|(d, s)| (*d, *s)).collect();

    // Run fixpoint computation
    prog.run();

    // Extract results into PointsToMap
    let mut result = PointsToMap::new();
    for (val, pts) in &prog.points_to {
        if !pts.is_empty() {
            result.insert(*val, pts.iter().collect());
        }
    }
    result
}

/// Parallel solver using `ascent_par!`.
///
/// Requires the `parallel` feature and is not available on WASM targets.
#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
// NOTE: Ascent-generated struct only exposes Default::default();
// field assignment is the intended way to populate input relations.
#[allow(clippy::field_reassign_with_default)]
fn ascent_solve_par(facts: &PtaFacts) -> PointsToMap {
    let mut prog = PtaProgramPar::default();

    // Load input relations from facts
    prog.addr_of = facts.addr_of.iter().map(|(p, l)| (*p, *l)).collect();
    prog.copy_edge = facts.copy.iter().map(|(d, s)| (*d, *s)).collect();
    prog.load_edge = facts.load.iter().map(|(d, s)| (*d, *s)).collect();
    prog.store_edge = facts.store.iter().map(|(d, s)| (*d, *s)).collect();

    // Run parallel fixpoint computation
    prog.run();

    // Extract results into PointsToMap
    let mut result = PointsToMap::new();
    for (val, pts) in &prog.points_to {
        if !pts.is_empty() {
            result.insert(*val, pts.iter().collect());
        }
    }
    result
}
