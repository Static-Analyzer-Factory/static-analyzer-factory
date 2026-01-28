//! Demand-driven clobber walker for Memory SSA.
//!
//! Given a Use access and a memory location, walks the def chain backward
//! consulting PTA to find the clobbering Def (the most recent store that
//! may modify that location).

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{InstId, LocId};

use crate::PtaResult;

use super::access::{MemAccessId, MemoryAccess};
use super::{InstInfo, MemorySsa, ModRefSummary};

/// Find the clobbering def for a Use at a specific location.
///
/// This is the main entry point for demand-driven disambiguation.
/// Results are cached in `mssa.clobber_cache` at `(access_id, loc)` granularity
/// so that different Uses with the same reaching def reuse walk results.
pub fn clobber_for(mssa: &mut MemorySsa, use_id: MemAccessId, loc: LocId) -> MemAccessId {
    // Check cache first
    if let Some(&cached) = mssa.clobber_cache.get(&(use_id, loc)) {
        return cached;
    }

    // Get the Use's reaching def in the skeleton
    let defining = match mssa.accesses.get(&use_id) {
        Some(MemoryAccess::Use { defining, .. }) => *defining,
        _ => return use_id, // Not a Use — return self
    };

    // Check if the reaching def's result is already cached (from a different Use)
    if let Some(&cached) = mssa.clobber_cache.get(&(defining, loc)) {
        mssa.clobber_cache.insert((use_id, loc), cached);
        return cached;
    }

    // Walk the chain with split borrows: walk_chain reads from accesses/inst_info/pta
    // and writes intermediate results to clobber_cache.
    let mut visited = BTreeSet::new();
    let result = walk_chain(
        &mssa.accesses,
        &mssa.inst_to_access,
        &mssa.inst_info,
        &mssa.pta,
        &mssa.mod_ref_summaries,
        &mut mssa.clobber_cache,
        defining,
        loc,
        &mut visited,
    );

    mssa.clobber_cache.insert((use_id, loc), result);
    result
}

/// Walk the def chain backward looking for a clobber of the given location.
///
/// Uses PTA to determine if a Def actually affects the queried location.
/// For Phi nodes, checks if all predecessor paths resolve to the same clobber
/// (returns the Phi itself if they diverge).
///
/// Takes split borrows of `MemorySsa` fields so it can both read analysis data
/// and write to the clobber cache. Every non-clobbering Def on the walk path
/// gets cached with the final result, so future queries starting from any
/// point on the same chain get O(1) answers.
// NOTE: This function takes many parameters because it uses split borrows
// of MemorySsa fields to enable simultaneous reads and cache writes.
#[allow(clippy::too_many_arguments)]
fn walk_chain(
    accesses: &BTreeMap<MemAccessId, MemoryAccess>,
    inst_to_access: &BTreeMap<InstId, MemAccessId>,
    inst_info: &BTreeMap<InstId, InstInfo>,
    pta: &PtaResult,
    mod_ref_summaries: &BTreeMap<saf_core::ids::FunctionId, ModRefSummary>,
    cache: &mut BTreeMap<(MemAccessId, LocId), MemAccessId>,
    access_id: MemAccessId,
    loc: LocId,
    visited: &mut BTreeSet<MemAccessId>,
) -> MemAccessId {
    // Cycle detection: if we've already visited this access, return it
    if !visited.insert(access_id) {
        return access_id;
    }

    // Check cache — a previous query may have already resolved this (access, loc)
    if let Some(&cached) = cache.get(&(access_id, loc)) {
        return cached;
    }

    let access = match accesses.get(&access_id) {
        Some(a) => a.clone(), // Clone to avoid borrow conflicts
        None => return access_id,
    };

    let result = match access {
        MemoryAccess::LiveOnEntry { .. } => {
            // Reached function entry — no clobber found
            access_id
        }

        MemoryAccess::Def { inst, defining, .. } => {
            if is_clobber(
                accesses,
                inst_to_access,
                inst_info,
                pta,
                mod_ref_summaries,
                inst,
                loc,
            ) {
                // This def clobbers the location
                access_id
            } else {
                // Skip this def, continue walking
                walk_chain(
                    accesses,
                    inst_to_access,
                    inst_info,
                    pta,
                    mod_ref_summaries,
                    cache,
                    defining,
                    loc,
                    visited,
                )
            }
        }

        MemoryAccess::Use { defining, .. } => {
            // Skip Uses in the chain (shouldn't normally appear in the
            // defining chain, but handle gracefully)
            walk_chain(
                accesses,
                inst_to_access,
                inst_info,
                pta,
                mod_ref_summaries,
                cache,
                defining,
                loc,
                visited,
            )
        }

        MemoryAccess::Phi { operands, .. } => {
            // Check if all predecessor paths resolve to the same clobber
            let mut results = BTreeSet::new();
            for &pred_def in operands.values() {
                let r = walk_chain(
                    accesses,
                    inst_to_access,
                    inst_info,
                    pta,
                    mod_ref_summaries,
                    cache,
                    pred_def,
                    loc,
                    visited,
                );
                results.insert(r);
            }

            if results.len() == 1 {
                // All paths agree — return the single clobber
                *results.iter().next().expect("non-empty results")
            } else {
                // Multiple reaching defs — the Phi itself is the "clobber"
                access_id
            }
        }
    };

    // Cache intermediate result so future queries from any point
    // on this chain get O(1) answers
    cache.insert((access_id, loc), result);

    result
}

/// Check if an instruction clobbers (may modify) a given location.
///
/// Consults PTA for stores and mod/ref summaries for calls.
/// For stores and memcpy/memset, checks if the destination pointer's
/// points-to set contains the queried location (O(log n) in a `BTreeSet`).
fn is_clobber(
    accesses: &BTreeMap<MemAccessId, MemoryAccess>,
    inst_to_access: &BTreeMap<InstId, MemAccessId>,
    inst_info: &BTreeMap<InstId, InstInfo>,
    pta: &PtaResult,
    mod_ref_summaries: &BTreeMap<saf_core::ids::FunctionId, ModRefSummary>,
    inst_id: InstId,
    loc: LocId,
) -> bool {
    // Get the access for this instruction
    let access = match inst_to_access.get(&inst_id) {
        Some(&acc_id) => accesses.get(&acc_id),
        None => return false,
    };

    // The access is always a Def (stores and calls are Defs)
    let Some(MemoryAccess::Def { .. }) = access else {
        return false;
    };

    if let Some(info) = inst_info.get(&inst_id) {
        match info {
            InstInfo::Store { ptr } => pta.points_to(*ptr).contains(&loc),
            InstInfo::Call { callee } => {
                // Check mod/ref: does callee modify loc?
                if let Some(callee_fid) = callee {
                    if let Some(summary) = mod_ref_summaries.get(callee_fid) {
                        return summary.may_mod.contains(loc);
                    }
                }
                // Unknown callee (indirect call) — conservatively clobbers
                true
            }
            InstInfo::Memcpy { dst_ptr } | InstInfo::Memset { dst_ptr } => {
                pta.points_to(*dst_ptr).contains(&loc)
            }
        }
    } else {
        // No info — conservatively assume clobber
        true
    }
}
