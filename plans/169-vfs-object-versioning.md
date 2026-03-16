# Plan 169: VFS Object Versioning for FS-PTA Solver

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the FS-PTA solver's per-node `BTreeMap<LocId, PtsSet>` dataflow maps with a global `VersionTable` + thin per-node version maps, reducing tmux memory from 23 GB (OOM) to ~90 MB.

**Architecture:** The solver currently stores full points-to sets per SVFG node per memory object (`df_in`/`df_out`). VFS replaces these with lightweight version IDs (u32) per node, storing actual PTS data once in a shared `VersionTable`. Stores create new versions only for modified objects; unmodified objects pass through the same version ID (no clone). Two growth controls: create-on-change (only allocate when PTS grows) and periodic compaction (GC unreferenced versions).

**Tech Stack:** Rust, existing `PtsSet` trait, `IndexMap`, `BTreeMap`/`BTreeSet` (NFR-DET)

**Design doc:** `docs/plans/2026-02-25-vfs-object-versioning-design.md`

**Key files:**
- `crates/saf-analysis/src/fspta/version_table.rs` — **NEW**
- `crates/saf-analysis/src/fspta/solver.rs` — **REWRITE** internals (lines 55-368)
- `crates/saf-analysis/src/fspta/mod.rs` — **MODIFY** (add module, config field)

**What stays unchanged:**
- `FlowSensitivePtaResult` public API (pts, df_in, df_out, diagnostics)
- `solve_flow_sensitive()` entry point signature
- `FsSvfg`, `FsSvfgBuilder`, `StrongUpdateInfo`
- Direct propagation (`propagate_direct`, `union_points_to`)
- All existing tests pass without modification

---

## Task 1: Create `VersionTable` with unit tests

**Files:**
- Create: `crates/saf-analysis/src/fspta/version_table.rs`

### Step 1: Write the `VersionTable` module

Create `crates/saf-analysis/src/fspta/version_table.rs`:

```rust
//! Version table for VFS (Versioned Flow-Sensitive) pointer analysis.
//!
//! Replaces per-node `BTreeMap<LocId, PtsSet>` dataflow maps with a global
//! version store. Each version ID is a lightweight u32 index into a dense
//! `Vec<P>`. Per-node state becomes `BTreeMap<LocId, VersionId>` (~20 bytes/entry)
//! instead of `BTreeMap<LocId, PtsSet>` (~1,600 bytes/entry).

use std::collections::{BTreeMap, BTreeSet};

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

/// Global version store for VFS pointer analysis.
///
/// Holds all points-to set data in a dense `Vec<P>`, indexed by `VersionId`.
/// Versions are created when stores modify objects or phi nodes merge
/// incoming versions. The create-on-change policy ensures versions are only
/// allocated when PTS actually grows (checked via `PtsSet::union` return value).
pub(crate) struct VersionTable<P: PtsSet> {
    /// Dense PTS storage, indexed by `VersionId.0`.
    pts: Vec<P>,
    /// Next available version ID.
    next_id: u32,
}

impl<P: PtsSet> VersionTable<P> {
    /// Create an empty version table.
    pub(crate) fn new() -> Self {
        Self {
            pts: Vec::new(),
            next_id: 0,
        }
    }

    /// Create a new version with the given points-to set.
    ///
    /// Returns the `VersionId` for the new entry. The caller is responsible
    /// for the create-on-change policy (only call this when PTS actually changed).
    pub(crate) fn new_version(&mut self, pts: P) -> VersionId {
        let vid = VersionId(self.next_id);
        self.next_id += 1;
        self.pts.push(pts);
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
    /// the internal `Vec<P>` with only live entries, and remaps all references
    /// in the version maps.
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
        for old_vid in &live {
            let new_vid = VersionId(new_pts.len() as u32);
            new_pts.push(self.pts[old_vid.0 as usize].clone());
            remap.insert(*old_vid, new_vid);
        }

        // 3. Replace internal state
        self.pts = new_pts;
        self.next_id = self.pts.len() as u32;

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
```

### Step 2: Wire module into `mod.rs`

In `crates/saf-analysis/src/fspta/mod.rs`, add after line 14 (`mod strong_update;`):

```rust
mod version_table;
```

### Step 3: Build check

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis 2>&1' | tail -20`

Expected: compilation succeeds (new module compiles, tests discoverable).

### Step 4: Run version_table unit tests

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis version_table 2>&1' | tail -20`

Expected: 4 tests pass.

### Step 5: Commit

```bash
git add crates/saf-analysis/src/fspta/version_table.rs crates/saf-analysis/src/fspta/mod.rs
git commit -m "feat(fspta): add VersionTable for VFS object versioning (Plan 169 Task 1)"
```

---

## Task 2: Add `compact_interval` to `FsPtaConfig`

**Files:**
- Modify: `crates/saf-analysis/src/fspta/mod.rs:210-228`

### Step 1: Add field to `FsPtaConfig`

In `crates/saf-analysis/src/fspta/mod.rs`, add after the `pts_config` field (line 218):

```rust
    /// Version table compaction interval (VFS solver).
    ///
    /// Every `compact_interval` worklist iterations, unreferenced versions
    /// are garbage-collected from the version table. Set to 0 to disable.
    pub compact_interval: usize,
```

### Step 2: Update `Default` impl

In the `Default` impl (lines 221-228), add `compact_interval: 10_000` to the struct literal.

### Step 3: Add VFS diagnostics fields to `FsPtaDiagnostics`

In `crates/saf-analysis/src/fspta/mod.rs`, add two fields to `FsPtaDiagnostics` (after line 486, before the closing `}`):

```rust
    /// Number of versions in the version table at solver completion.
    pub version_count: usize,
    /// Number of compaction passes performed.
    pub compactions: usize,
```

### Step 4: Update existing config test

The test `fs_pta_config_default` at line 610 should still pass (no assertion on new field).

### Step 5: Build check

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis 2>&1' | tail -20`

Expected: compiles cleanly.

### Step 6: Commit

```bash
git add crates/saf-analysis/src/fspta/mod.rs
git commit -m "feat(fspta): add compact_interval config and VFS diagnostics fields (Plan 169 Task 2)"
```

---

## Task 3: Rewrite `solve_flow_sensitive_generic` to use VFS

This is the core task. Replace `df_in`/`df_out` with `ver_in`/`ver_out`/`ver_table` throughout `solver.rs`.

**Files:**
- Modify: `crates/saf-analysis/src/fspta/solver.rs` (lines 55-368)

### Step 1: Update imports

At the top of `solver.rs` (lines 16-31), add the `version_table` import. Replace `super::GenericDfPointsTo` with the VFS types:

Replace lines 28-31:
```rust
use super::GenericDfPointsTo;
use super::StoreInfo;
use super::strong_update::StrongUpdateInfo;
use super::{FlowSensitivePtaResult, FsPtaConfig, FsPtaDiagnostics, FsSvfg};
```

With:
```rust
use super::StoreInfo;
use super::strong_update::StrongUpdateInfo;
use super::version_table::{NodeVersionMap, VersionTable};
use super::{FlowSensitivePtaResult, FsPtaConfig, FsPtaDiagnostics, FsSvfg};
```

### Step 2: Rewrite `solve_flow_sensitive_generic`

Replace the entire function (lines 55-147) with:

```rust
fn solve_flow_sensitive_generic<P: PtsSet>(
    module: &AirModule,
    fs_svfg: &FsSvfg,
    pta_result: &PtaResult,
    callgraph: &CallGraph,
    config: &FsPtaConfig,
) -> FlowSensitivePtaResult {
    let su_info = StrongUpdateInfo::new(callgraph);
    let inst_to_func = build_inst_to_func_map(module);

    // Step 1: Initialize points_to from Andersen results
    let mut points_to: IndexMap<ValueId, P> = IndexMap::new();
    for (vid, locs) in pta_result.points_to_map() {
        if !locs.is_empty() {
            points_to.insert(*vid, P::from_btreeset(locs));
        }
    }

    // VFS state: version table + thin per-node version maps
    let mut ver_table: VersionTable<P> = VersionTable::new();
    let mut ver_in: IndexMap<SvfgNodeId, NodeVersionMap> = IndexMap::new();
    let mut ver_out: IndexMap<SvfgNodeId, NodeVersionMap> = IndexMap::new();

    let mut diagnostics = FsPtaDiagnostics {
        fs_svfg_nodes: fs_svfg.node_count(),
        fs_svfg_edges: fs_svfg.edge_count(),
        store_nodes: fs_svfg.store_count(),
        load_nodes: fs_svfg.load_count(),
        ..FsPtaDiagnostics::default()
    };

    // Seed worklist with all nodes that have non-empty points_to (value nodes only)
    let mut worklist: BTreeSet<SvfgNodeId> = BTreeSet::new();
    for node in fs_svfg.nodes() {
        if let SvfgNodeId::Value(vid) = node {
            if points_to.get(vid).is_some_and(|s| !s.is_empty()) {
                worklist.insert(*node);
            }
        }
    }

    // Step 2: Process worklist
    let mut iterations = 0usize;
    while let Some(node) = worklist.pop_first() {
        if iterations >= config.max_iterations {
            diagnostics.iteration_limit_hit = true;
            break;
        }
        iterations += 1;

        // Periodic compaction
        if config.compact_interval > 0 && iterations % config.compact_interval == 0 {
            ver_table.compact(&mut ver_in, &mut ver_out);
            diagnostics.compactions += 1;
        }

        // Process the node's instruction semantics
        process_node(
            node,
            fs_svfg,
            pta_result,
            &su_info,
            &inst_to_func,
            &mut points_to,
            &mut ver_in,
            &mut ver_out,
            &mut ver_table,
            &mut diagnostics,
        );

        // Propagate to successors
        for edge in fs_svfg.successors_of(node) {
            let changed = if edge.kind.is_direct() {
                propagate_direct(node, edge.target, &points_to).is_some_and(|ref new_pt_set| {
                    union_points_to(&mut points_to, edge.target, new_pt_set)
                })
            } else {
                propagate_indirect(
                    node,
                    edge.target,
                    &edge.objects,
                    &ver_out,
                    &mut ver_in,
                    &mut ver_table,
                )
            };

            if changed {
                worklist.insert(edge.target);
            }
        }
    }

    diagnostics.iterations = iterations;
    diagnostics.version_count = ver_table.len();

    // Convert VFS state to public BTreeMap/BTreeSet representation
    let pts_btree: BTreeMap<ValueId, BTreeSet<LocId>> = points_to
        .into_iter()
        .map(|(k, v)| (k, v.to_btreeset()))
        .collect();
    let df_in_btree = convert_ver_map(&ver_in, &ver_table);
    let df_out_btree = convert_ver_map(&ver_out, &ver_table);

    FlowSensitivePtaResult {
        pts: pts_btree,
        df_in: df_in_btree,
        df_out: df_out_btree,
        diagnostics,
    }
}
```

### Step 3: Replace `convert_df_map` with `convert_ver_map`

Replace the `convert_df_map` function (lines 149-163) with:

```rust
/// Convert VFS version maps to the public `BTreeMap` representation.
fn convert_ver_map<P: PtsSet>(
    ver_map: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &VersionTable<P>,
) -> BTreeMap<SvfgNodeId, BTreeMap<LocId, BTreeSet<LocId>>> {
    ver_map
        .iter()
        .map(|(node, versions)| {
            let converted: BTreeMap<LocId, BTreeSet<LocId>> = versions
                .iter()
                .filter_map(|(loc, vid)| {
                    let pts = ver_table.get(*vid).to_btreeset();
                    if pts.is_empty() { None } else { Some((*loc, pts)) }
                })
                .collect();
            (*node, converted)
        })
        .collect()
}
```

### Step 4: Rewrite `process_node`

Replace lines 170-210 with:

```rust
#[allow(clippy::too_many_arguments)]
fn process_node<P: PtsSet>(
    node: SvfgNodeId,
    fs_svfg: &FsSvfg,
    pta_result: &PtaResult,
    su_info: &StrongUpdateInfo,
    inst_to_func: &IndexMap<ValueId, FunctionId>,
    points_to: &mut IndexMap<ValueId, P>,
    ver_in: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_out: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &mut VersionTable<P>,
    diagnostics: &mut FsPtaDiagnostics,
) {
    // Store node: create new versions for stored-to objects
    let store_infos = fs_svfg.store_infos(node);
    if !store_infos.is_empty() {
        process_stores(
            node,
            store_infos,
            pta_result,
            su_info,
            inst_to_func,
            points_to,
            ver_in,
            ver_out,
            ver_table,
            diagnostics,
        );
        return;
    }

    // Load node: read from version table into points_to
    if let Some(load_info) = fs_svfg.load_info(node) {
        process_load(node, load_info.ptr, load_info.dst, points_to, ver_in, ver_table);
        return;
    }

    // Pass-through: ver_out[node] = ver_in[node]
    if let Some(in_versions) = ver_in.get(&node).cloned() {
        ver_out.insert(node, in_versions);
    }
}
```

### Step 5: Rewrite `process_stores`

Replace lines 222-267 with:

```rust
#[allow(clippy::too_many_arguments)]
fn process_stores<P: PtsSet>(
    node: SvfgNodeId,
    store_infos: &[StoreInfo],
    pta_result: &PtaResult,
    su_info: &StrongUpdateInfo,
    inst_to_func: &IndexMap<ValueId, FunctionId>,
    points_to: &IndexMap<ValueId, P>,
    ver_in: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_out: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &mut VersionTable<P>,
    diagnostics: &mut FsPtaDiagnostics,
) {
    let empty_ver: NodeVersionMap = BTreeMap::new();
    let in_versions = ver_in.get(&node).unwrap_or(&empty_ver);
    // Clone version map (~20 bytes/entry) instead of full PTS map (~1600 bytes/entry)
    let mut out_versions = in_versions.clone();

    for store_info in store_infos {
        let empty_pts = P::empty();
        let pointer_pt_set = points_to.get(&store_info.ptr).unwrap_or(&empty_pts);
        let val_pt_set = points_to.get(&store_info.val).unwrap_or(&empty_pts);

        let func_id = inst_to_func
            .get(&store_info.ptr)
            .or_else(|| inst_to_func.get(&store_info.val))
            .copied()
            .unwrap_or(FunctionId::new(0));

        // Convert to BTreeSet for strong update check
        let pointer_btree = pointer_pt_set.to_btreeset();
        let can_strong =
            su_info.can_strong_update(store_info.ptr, func_id, &pointer_btree, pta_result);

        for loc in pointer_pt_set.iter() {
            if can_strong {
                diagnostics.strong_updates += 1;
                // Strong update: new version = just the stored value's PTS
                let vid = ver_table.new_version(val_pt_set.clone());
                out_versions.insert(loc, vid);
            } else {
                diagnostics.weak_updates += 1;
                // Weak update: merge old version PTS with stored value's PTS
                let mut merged = in_versions
                    .get(&loc)
                    .map(|vid| ver_table.get(*vid).clone())
                    .unwrap_or_else(P::empty);
                if merged.union(val_pt_set) {
                    // PTS actually grew — create new version
                    let vid = ver_table.new_version(merged);
                    out_versions.insert(loc, vid);
                } else if let Some(&vid) = in_versions.get(&loc) {
                    // PTS unchanged — reuse existing version
                    out_versions.insert(loc, vid);
                }
            }
        }
    }

    ver_out.insert(node, out_versions);
}
```

### Step 6: Rewrite `process_load`

Replace lines 270-292 with:

```rust
fn process_load<P: PtsSet>(
    node: SvfgNodeId,
    pointer: ValueId,
    dst: ValueId,
    points_to: &mut IndexMap<ValueId, P>,
    ver_in: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &VersionTable<P>,
) {
    let pointer_pt_set = points_to.get(&pointer).cloned().unwrap_or_else(P::empty);
    let empty_ver: NodeVersionMap = BTreeMap::new();
    let in_versions = ver_in.get(&node).unwrap_or(&empty_ver);

    let mut new_pt_set = P::empty();
    for loc in pointer_pt_set.iter() {
        if let Some(vid) = in_versions.get(&loc) {
            new_pt_set.union(ver_table.get(*vid));
        }
    }

    if !new_pt_set.is_empty() {
        let dst_pt_set = points_to.entry(dst).or_insert_with(P::empty);
        dst_pt_set.union(&new_pt_set);
    }
}
```

### Step 7: Rewrite `propagate_indirect`

Replace lines 328-368 with:

```rust
fn propagate_indirect<P: PtsSet>(
    src: SvfgNodeId,
    target: SvfgNodeId,
    objects: &BTreeSet<LocId>,
    ver_out: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_in: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &mut VersionTable<P>,
) -> bool {
    let Some(src_versions) = ver_out.get(&src) else {
        return false;
    };

    let mut changed = false;

    // Determine which objects to propagate
    let propagate_all = objects.is_empty();

    let obj_iter: Box<dyn Iterator<Item = &LocId>> = if propagate_all {
        // PhiFlow or unlabeled indirect edge: propagate all objects
        Box::new(src_versions.keys())
    } else {
        // Labeled indirect edge: propagate only specified objects
        Box::new(objects.iter())
    };

    for obj in obj_iter {
        let Some(&src_vid) = src_versions.get(obj) else {
            continue;
        };

        let target_in = ver_in.entry(target).or_default();
        match target_in.get(obj).copied() {
            None => {
                // First arrival: just copy the VersionId (4 bytes, no PTS clone!)
                target_in.insert(*obj, src_vid);
                changed = true;
            }
            Some(target_vid) if target_vid == src_vid => {
                // Same version — no work needed (common case in converged state)
            }
            Some(target_vid) => {
                // Different versions — merge their PTS
                let mut merged = ver_table.get(target_vid).clone();
                if merged.union(ver_table.get(src_vid)) {
                    // PTS grew — create new merged version
                    let new_vid = ver_table.new_version(merged);
                    target_in.insert(*obj, new_vid);
                    changed = true;
                }
                // If union didn't grow, target already subsumes src — no change
            }
        }
    }

    changed
}
```

### Step 8: Remove unused `GenericDfPointsTo` import

The `use super::GenericDfPointsTo;` line was already removed in Step 1.

### Step 9: Build check

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis 2>&1' | tail -30`

Expected: compiles. If there are unused import warnings for `GenericDfPointsTo` in mod.rs, that's OK — it's only used by the public type alias `DfPointsTo` and the `GenericDfPointsTo` type alias. The `GenericDfPointsTo` type alias in mod.rs may now be unused since the solver no longer references it — remove it if clippy complains but keep the public `DfPointsTo` alias.

### Step 10: Commit

```bash
git add crates/saf-analysis/src/fspta/solver.rs
git commit -m "feat(fspta): rewrite solver to use VFS version table (Plan 169 Task 3)"
```

---

## Task 4: Clean up unused code

**Files:**
- Modify: `crates/saf-analysis/src/fspta/mod.rs`

### Step 1: Check if `GenericDfPointsTo` is still used

Run: `grep -rn 'GenericDfPointsTo' crates/saf-analysis/src/`

If the only occurrence is the type alias definition at line 271, remove lines 267-271:
```rust
/// Generic per-location dataflow points-to map: `LocId` → `P`.
///
/// Used internally by the generic solver. Converted to `DfPointsTo`
/// at the solver boundary for API stability.
pub(crate) type GenericDfPointsTo<P> = BTreeMap<LocId, P>;
```

### Step 2: Build check

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis 2>&1' | tail -20`

### Step 3: Commit (if changes made)

```bash
git add crates/saf-analysis/src/fspta/mod.rs
git commit -m "refactor(fspta): remove unused GenericDfPointsTo alias (Plan 169 Task 4)"
```

---

## Task 5: Run all existing tests (zero regression)

**Files:** None (validation only)

### Step 1: Run unit tests

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis fspta 2>&1' | tail -30`

Expected: All existing tests pass — `solver_converges_on_simple_store_load`, `solver_diagnostics_populated`, `fs_svfg_*` tests, `strong_update_*` tests, `builder_*` tests.

### Step 2: Run E2E tests

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test fspta_e2e 2>&1' | tail -30`

Expected: All 8 E2E tests pass:
- `fspta_strong_update_builds_and_converges`
- `fspta_strong_update_export`
- `fspta_branch_merge_builds`
- `fspta_loop_weak_update_builds`
- `fspta_interproc_builds`
- `fspta_cpp_field_builds`
- `fspta_rust_unsafe_builds`
- `fspta_export_is_deterministic`
- `fspta_strong_update_no_worse_than_andersen`

### Step 3: Run full test suite

Run (in background): `docker compose run --rm dev sh -c 'cargo nextest run 2>&1' | tail -40`

Expected: All ~2000+ Rust tests pass. No regressions from VFS changes.

### Step 4: Run lint

Run: `make fmt && make lint`

Expected: Clean. Fix any clippy warnings (likely `clippy::too_many_arguments` allows are correct, `doc_markdown` on new doc comments).

### Step 5: Commit lint fixes if any

```bash
git add -u
git commit -m "style(fspta): fix clippy/fmt warnings (Plan 169 Task 5)"
```

---

## Task 6: CruxBC memory benchmark

**Files:** None (validation only)

### Step 1: Run CruxBC on curl (baseline comparison)

The benchmark harness already has RSS tracking from Plan 168. Run:

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc-single --program-path tests/benchmarks/sv-benchmarks/.compiled-cruxbc/curl.ll --solver fspta 2>&1' | tail -40
```

Compare RSS values with the baseline from Plan 168:
- Before VFS (curl): FS-PTA solve consumed ~923 MB (166 MB → 1089 MB)
- After VFS: Expect significantly lower delta (maybe 30-80 MB)

### Step 2: Run CruxBC on tmux (the OOM test)

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc-single --program-path tests/benchmarks/sv-benchmarks/.compiled-cruxbc/tmux.ll --solver fspta 2>&1' | tail -60
```

Before VFS: OOM kill at 23+ GB.
After VFS: Should complete successfully with peak RSS well under 4 GB.

### Step 3: Run full CruxBC suite

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc 2>&1' | tail -30
```

Expected: All 11 programs pass (including tmux).

### Step 4: Document results

Record RSS measurements in the commit message and update `plans/PROGRESS.md`.

---

## Task 7: Update `plans/PROGRESS.md`

**Files:**
- Modify: `plans/PROGRESS.md`

### Step 1: Add plan to Plans Index

Add row to the Plans Index table:
```
| 169 | vfs-object-versioning | scalability | done | Notes: VFS object versioning for FS-PTA solver. Replaced per-node BTreeMap<LocId, PtsSet> with global VersionTable + thin NodeVersionMap. Create-on-change dedup + periodic compaction. [RSS results from Task 6]. Plan: `plans/169-vfs-object-versioning.md`. Design: `docs/plans/2026-02-25-vfs-object-versioning-design.md`. |
```

### Step 2: Update Scalability roadmap in Next Steps

Change the line:
```
- **Scalability roadmap — Phase 2 remaining:** Object Versioning VFS (5-26x fspta — biggest single win still pending), im-rc persistent DfPointsTo (dependency added, needs solver wiring for O(log n) clone).
```

To:
```
- **Scalability roadmap — Phase 2 remaining:** im-rc persistent DfPointsTo (dependency added, superseded by VFS for fspta — may still benefit other subsystems).
```

### Step 3: Append session log entry

Add session log entry with date, epic=scalability, and summary of work done including RSS numbers.

### Step 4: Commit

```bash
git add plans/PROGRESS.md plans/169-vfs-object-versioning.md
git commit -m "docs: update PROGRESS.md for Plan 169 VFS object versioning"
```
