//! Version table for VFS (Versioned Flow-Sensitive) pointer analysis.
//!
//! Replaces per-node `BTreeMap<LocId, PtsSet>` dataflow maps with a global
//! version store. Each version ID is a lightweight u32 index into a dense
//! `Vec<P>`. Per-node state becomes `BTreeMap<LocId, VersionId>` (~20 bytes/entry)
//! instead of `BTreeMap<LocId, PtsSet>` (~1,600 bytes/entry).
//!
//! Content-based deduplication ensures that PTS with identical contents share
//! the same `VersionId`, capping memory at the number of truly distinct PTS
//! states rather than the number of version operations.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

use indexmap::IndexMap;

use saf_core::ids::LocId;

use crate::pta::ptsset::PtsSet;
use crate::svfg::SvfgNodeId;

/// Globally unique version identifier.
///
/// Indexes into `VersionTable.pts`. Cheap to copy (4 bytes) compared to
/// cloning an entire `PtsSet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VersionId(u32);

/// Thin per-node state — version numbers only.
///
/// ~20 bytes per entry (`LocId` u128 + `VersionId` u32) vs ~1,600 bytes per
/// entry for the old `BTreeMap<LocId, PtsSet>`.
pub(crate) type NodeVersionMap = BTreeMap<LocId, VersionId>;

/// Compute a hash of a `PtsSet` for deduplication lookup.
fn pts_hash<P: PtsSet>(pts: &P) -> u64 {
    let mut hasher = DefaultHasher::new();
    pts.hash(&mut hasher);
    hasher.finish()
}

/// Global version store for VFS pointer analysis.
///
/// Holds all points-to set data in a dense `Vec<P>`, indexed by `VersionId`.
/// Content-based deduplication ensures that identical PTS share the same
/// `VersionId`. Combined with create-on-change at call sites, this bounds
/// memory at the number of truly distinct PTS states.
pub(crate) struct VersionTable<P: PtsSet> {
    /// Dense PTS storage, indexed by `VersionId.0`.
    pts: Vec<P>,
    /// Next available version ID.
    next_id: u32,
    /// Content dedup index: hash(PTS) → candidate VersionIds.
    /// On collision, equality check confirms the match.
    dedup: HashMap<u64, Vec<VersionId>>,
    /// Number of dedup hits (reused existing version).
    pub(crate) dedup_hits: u64,
}

impl<P: PtsSet> VersionTable<P> {
    /// Create an empty version table.
    pub(crate) fn new() -> Self {
        Self {
            pts: Vec::new(),
            next_id: 0,
            dedup: HashMap::new(),
            dedup_hits: 0,
        }
    }

    /// Create a new version with the given points-to set, or return an existing
    /// `VersionId` if an identical PTS already exists (content deduplication).
    ///
    /// The caller is responsible for the create-on-change policy (only call
    /// this when PTS actually changed). Dedup handles the case where the
    /// "changed" PTS happens to be identical to a previously created version.
    pub(crate) fn new_version(&mut self, pts: P) -> VersionId {
        let hash = pts_hash(&pts);

        // Check for existing version with identical content
        if let Some(candidates) = self.dedup.get(&hash) {
            for &vid in candidates {
                if self.pts[vid.0 as usize] == pts {
                    self.dedup_hits += 1;
                    return vid;
                }
            }
        }

        // No match — allocate new version
        let vid = VersionId(self.next_id);
        self.next_id += 1;
        self.pts.push(pts);
        self.dedup.entry(hash).or_default().push(vid);
        vid
    }

    /// Get the points-to set for a version.
    ///
    /// # Panics
    /// Panics if the `VersionId` is out of bounds (indicates a bug).
    pub(crate) fn get(&self, vid: VersionId) -> &P {
        &self.pts[vid.0 as usize]
    }

    /// Number of versions currently stored.
    pub(crate) fn len(&self) -> usize {
        self.pts.len()
    }

    /// Compact the version table by removing unreferenced versions.
    ///
    /// Scans `ver_in` and `ver_out` to find all live `VersionId`s, rebuilds
    /// the internal `Vec<P>` with only live entries, remaps all references
    /// in the version maps, and rebuilds the dedup index.
    pub(crate) fn compact(
        &mut self,
        ver_in: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
        ver_out: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ) {
        // 1. Collect all referenced VersionIds
        let mut live = BTreeSet::new();
        for map in ver_in.values() {
            for vid in map.values() {
                live.insert(*vid);
            }
        }
        for map in ver_out.values() {
            for vid in map.values() {
                live.insert(*vid);
            }
        }

        if live.len() == self.pts.len() {
            // All versions are live — nothing to compact
            return;
        }

        // 2. Build new Vec<P> with only live versions + remap table
        let mut new_pts = Vec::with_capacity(live.len());
        let mut remap: BTreeMap<VersionId, VersionId> = BTreeMap::new();
        let mut new_dedup: HashMap<u64, Vec<VersionId>> = HashMap::new();

        for old_vid in &live {
            // INVARIANT: version count < 2^32 (4B unique PTS versions infeasible)
            #[allow(clippy::cast_possible_truncation)]
            let new_vid = VersionId(new_pts.len() as u32);
            let pts = &self.pts[old_vid.0 as usize];
            let hash = pts_hash(pts);
            new_pts.push(pts.clone());
            remap.insert(*old_vid, new_vid);
            new_dedup.entry(hash).or_default().push(new_vid);
        }

        // 3. Replace internal state
        self.pts = new_pts;
        // INVARIANT: version count < 2^32
        #[allow(clippy::cast_possible_truncation)]
        let len = self.pts.len() as u32;
        self.next_id = len;
        self.dedup = new_dedup;

        // 4. Remap all ver_in/ver_out entries
        for map in ver_in.values_mut() {
            for vid in map.values_mut() {
                *vid = remap[vid];
            }
        }
        for map in ver_out.values_mut() {
            for vid in map.values_mut() {
                *vid = remap[vid];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pta::ptsset::BTreePtsSet;

    #[test]
    fn new_version_returns_sequential_ids() {
        let mut table = VersionTable::<BTreePtsSet>::new();
        let v0 = table.new_version(BTreePtsSet::singleton(LocId::new(1)));
        let v1 = table.new_version(BTreePtsSet::singleton(LocId::new(2)));
        assert_eq!(v0, VersionId(0));
        assert_eq!(v1, VersionId(1));
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn get_returns_correct_pts() {
        let mut table = VersionTable::<BTreePtsSet>::new();
        let loc_a = LocId::new(10);
        let loc_b = LocId::new(20);
        let v0 = table.new_version(BTreePtsSet::singleton(loc_a));
        let v1 = table.new_version(BTreePtsSet::singleton(loc_b));
        assert!(table.get(v0).contains(loc_a));
        assert!(!table.get(v0).contains(loc_b));
        assert!(table.get(v1).contains(loc_b));
    }

    #[test]
    fn dedup_returns_same_vid_for_identical_pts() {
        let mut table = VersionTable::<BTreePtsSet>::new();
        let v0 = table.new_version(BTreePtsSet::singleton(LocId::new(42)));
        let v1 = table.new_version(BTreePtsSet::singleton(LocId::new(42)));
        assert_eq!(v0, v1, "identical PTS should return same VersionId");
        assert_eq!(table.len(), 1, "should have only 1 stored version");
        assert_eq!(table.dedup_hits, 1);
    }

    #[test]
    fn dedup_distinguishes_different_pts() {
        let mut table = VersionTable::<BTreePtsSet>::new();
        let v0 = table.new_version(BTreePtsSet::singleton(LocId::new(1)));
        let v1 = table.new_version(BTreePtsSet::singleton(LocId::new(2)));
        assert_ne!(v0, v1);
        assert_eq!(table.len(), 2);
        assert_eq!(table.dedup_hits, 0);
    }

    #[test]
    fn compact_removes_unreferenced_versions() {
        let mut table = VersionTable::<BTreePtsSet>::new();
        let v0 = table.new_version(BTreePtsSet::singleton(LocId::new(1)));
        let _v1 = table.new_version(BTreePtsSet::singleton(LocId::new(2))); // unreferenced
        let v2 = table.new_version(BTreePtsSet::singleton(LocId::new(3)));
        assert_eq!(table.len(), 3);

        // Only v0 and v2 are referenced
        let node_a = SvfgNodeId::Value(saf_core::ids::ValueId::new(100));
        let node_b = SvfgNodeId::Value(saf_core::ids::ValueId::new(200));
        let mut ver_in = IndexMap::new();
        let mut ver_out = IndexMap::new();
        let mut map_a = NodeVersionMap::new();
        map_a.insert(LocId::new(1), v0);
        ver_in.insert(node_a, map_a);
        let mut map_b = NodeVersionMap::new();
        map_b.insert(LocId::new(3), v2);
        ver_out.insert(node_b, map_b);

        table.compact(&mut ver_in, &mut ver_out);

        // Should have 2 versions now
        assert_eq!(table.len(), 2);

        // Remapped IDs should still retrieve correct PTS
        let new_v0 = ver_in[&node_a][&LocId::new(1)];
        let new_v2 = ver_out[&node_b][&LocId::new(3)];
        assert!(table.get(new_v0).contains(LocId::new(1)));
        assert!(table.get(new_v2).contains(LocId::new(3)));
    }

    #[test]
    fn compact_noop_when_all_live() {
        let mut table = VersionTable::<BTreePtsSet>::new();
        let v0 = table.new_version(BTreePtsSet::singleton(LocId::new(1)));

        let node = SvfgNodeId::Value(saf_core::ids::ValueId::new(100));
        let mut ver_in = IndexMap::new();
        let mut ver_out = IndexMap::new();
        let mut map = NodeVersionMap::new();
        map.insert(LocId::new(1), v0);
        ver_in.insert(node, map);

        table.compact(&mut ver_in, &mut ver_out);
        assert_eq!(table.len(), 1);
        assert_eq!(ver_in[&node][&LocId::new(1)], VersionId(0));
    }
}
