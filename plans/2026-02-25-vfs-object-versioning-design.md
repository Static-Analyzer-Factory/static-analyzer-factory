# VFS Object Versioning for Flow-Sensitive PTA

**Date**: 2026-02-25
**Status**: Approved
**Epic**: Scalability
**Addresses**: FS-PTA memory explosion (tmux: 1.5 GB → 23.3 GB OOM kill)

## Problem

The FS-PTA solver maintains `df_in` and `df_out` maps per SVFG node:
```
IndexMap<SvfgNodeId, BTreeMap<LocId, P>>
```

Three compounding issues cause unbounded memory growth:

1. **Full map cloning per store**: Every store node clones its entire `df_in[node]` (BTreeMap with hundreds of objects, each containing hundreds of PTS entries) before modifying 1-2 objects.
2. **Weak update accumulation**: Each weak update unions into existing PTS. With large Andersen pre-analysis PTS, each store fans out to many locations, each getting unioned PTS.
3. **No state pruning**: Entries are added to `df_in`/`df_out` but never reclaimed, even after convergence.

RSS profiling on tmux confirms the FS-PTA solve phase alone grows memory from 1,488 MB to 23,301 MB (~600 MB/s) before OOM kill.

## Solution: Version Table (Classic VFS)

Replace heavy per-node `BTreeMap<LocId, PtsSet>` with lightweight per-node `BTreeMap<LocId, VersionId>` (version numbers) plus a global `VersionTable` holding actual PTS data.

### Core Insight

Most SVFG nodes don't modify most objects. In the current approach, every store clones ALL objects' PTS even though it only modifies 1-2. With VFS, unmodified objects just pass through the same version ID (a 4-byte integer, not a PTS copy).

### Data Structures

```rust
// crates/saf-analysis/src/fspta/version_table.rs

/// Globally unique version identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VersionId(u32);

/// Global version store — holds all PTS data, indexed by VersionId
pub(crate) struct VersionTable<P: PtsSet> {
    /// Dense PTS storage, indexed by VersionId.0
    pts: Vec<P>,
    /// Next available version ID
    next_id: u32,
}

/// Thin per-node state — version numbers only
/// ~20 bytes/entry (LocId u128 + VersionId u32)
/// vs current ~1,600 bytes/entry (LocId + full PtsSet)
pub(crate) type NodeVersionMap = BTreeMap<LocId, VersionId>;
```

Solver state changes from:
```rust
// BEFORE
let mut df_in:  IndexMap<SvfgNodeId, BTreeMap<LocId, P>>;
let mut df_out: IndexMap<SvfgNodeId, BTreeMap<LocId, P>>;

// AFTER
let mut ver_in:    IndexMap<SvfgNodeId, NodeVersionMap>;
let mut ver_out:   IndexMap<SvfgNodeId, NodeVersionMap>;
let mut ver_table: VersionTable<P>;
```

### Algorithm Changes

#### Store Processing

The biggest win. Clone a version map (~10 KB) instead of a PTS map (~800 KB):

```rust
fn process_stores_vfs<P: PtsSet>(...) {
    let in_versions = ver_in.get(&node).cloned().unwrap_or_default();
    let mut out_versions = in_versions.clone();  // Clone BTreeMap<LocId, VersionId>

    for store_info in store_infos {
        for loc in pts(store_info.ptr) {
            if can_strong_update(loc) {
                // Strong: new version = just stored value's PTS
                let vid = ver_table.new_version(val_pts.clone());
                out_versions.insert(loc, vid);
            } else {
                // Weak: merge old version PTS with stored value's PTS
                let mut merged = match in_versions.get(&loc) {
                    Some(vid) => ver_table.get(*vid).clone(),
                    None => P::empty(),
                };
                if merged.union(&val_pts) {
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

#### Load Processing

Reads from version table instead of `df_in`:

```rust
fn process_load_vfs<P: PtsSet>(...) {
    let in_versions = ver_in.get(&node);
    let mut new_pt_set = P::empty();
    for loc in pts(pointer) {
        if let Some(vid) = in_versions.and_then(|v| v.get(&loc)) {
            new_pt_set.union(ver_table.get(*vid));
        }
    }
    if !new_pt_set.is_empty() {
        points_to.entry(dst).or_insert_with(P::empty).union(&new_pt_set);
    }
}
```

#### Indirect Propagation

Compares version IDs first. Same version = skip entirely (the common case):

```rust
fn propagate_indirect_vfs<P: PtsSet>(...) -> bool {
    let src_versions = ver_out.get(&src);
    let target_in = ver_in.entry(target).or_default();
    let mut changed = false;

    for obj in relevant_objects(objects, src_versions) {
        let src_vid = src_versions[obj];
        match target_in.get(obj) {
            None => {
                // First arrival: copy VersionId (4 bytes, not a PTS)
                target_in.insert(*obj, src_vid);
                changed = true;
            }
            Some(&target_vid) if target_vid == src_vid => {
                // Same version — NO WORK (common case in converged state)
            }
            Some(&target_vid) => {
                // Different versions — merge PTS
                let mut merged = ver_table.get(target_vid).clone();
                if merged.union(ver_table.get(src_vid)) {
                    let new_vid = ver_table.new_version(merged);
                    target_in.insert(*obj, new_vid);
                    changed = true;
                }
            }
        }
    }
    changed
}
```

### Version Table Growth Control

Two complementary mitigations:

**1. Create-on-change (essential)**: Never create a new version if the PTS didn't actually grow. The `PtsSet::union()` method returns `bool` — only allocate when it returns `true`. This bounds total versions to `|objects| x |distinct PTS states per object|`.

**2. Periodic compaction (safety net)**: Every `compact_interval` iterations (configurable, default 10,000), scan `ver_in`/`ver_out` for referenced `VersionId`s and rebuild the version table with only live entries. Unreferenced intermediate versions are dropped.

```rust
impl<P: PtsSet> VersionTable<P> {
    fn compact(
        &mut self,
        ver_in: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
        ver_out: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ) {
        // 1. Collect all referenced VersionIds
        // 2. Build new Vec<P> with only live versions
        // 3. Create old→new VersionId remap
        // 4. Update all ver_in/ver_out entries with remapped IDs
    }
}
```

### Memory Estimate for tmux

| Component | Current | VFS |
|-----------|---------|-----|
| Per-node state (1000 nodes x 500 objects) | 500 x 1.6 KB x 1000 = 800 MB (cloned per iteration) | 500 x 20B x 1000 = 10 MB |
| Version table | N/A | ~50K versions x 1.6 KB = 80 MB |
| Total during solve | 23,301 MB (OOM) | ~90 MB |
| Reduction | — | ~250x |

### Convergence

Same convergence proof as current solver: PTS are monotone (only grow via union). Once an object's PTS stabilizes at a node, `union` returns `false`, no new version is created, and successors are not re-added to the worklist.

## Integration

### Files Modified

| File | Change |
|------|--------|
| `fspta/version_table.rs` | **New** — `VersionTable<P>`, `VersionId`, `NodeVersionMap`, compaction |
| `fspta/solver.rs` | Rewrite internals to use VFS state; replace `df_in`/`df_out` with `ver_in`/`ver_out`/`ver_table` |
| `fspta/mod.rs` | Add `mod version_table`, add `compact_interval` to `FsPtaConfig` |

### What Stays the Same

- **Public API**: `FlowSensitivePtaResult` keeps `BTreeMap<SvfgNodeId, DfPointsTo>` for `df_in`/`df_out`
- **Direct propagation**: `propagate_direct` untouched (top-level pointers, not memory objects)
- **Strong update detection**: `StrongUpdateInfo` untouched
- **FsSvfg construction**: Builder unchanged
- **PtsSet trait**: All existing representations work unmodified
- **Entry point**: `solve_flow_sensitive()` signature unchanged

### Result Materialization (VFS → public API)

At solver completion, convert version maps to the existing `DfPointsTo` format:
```rust
let df_in_result: BTreeMap<SvfgNodeId, DfPointsTo> = ver_in
    .into_iter()
    .map(|(node, versions)| {
        let df: DfPointsTo = versions
            .into_iter()
            .filter_map(|(loc, vid)| {
                let pts = ver_table.get(vid).to_btreeset();
                if pts.is_empty() { None } else { Some((loc, pts)) }
            })
            .collect();
        (node, df)
    })
    .collect();
```

### Config Extension

```rust
pub struct FsPtaConfig {
    pub max_iterations: usize,    // existing (default: 100_000)
    pub pts_config: PtsConfig,    // existing
    pub compact_interval: usize,  // NEW (default: 10_000)
}
```

## Testing

1. **Unit tests** (`version_table.rs`): CRUD, create-on-change dedup, compaction correctness
2. **Equivalence tests**: VFS solver produces identical `FlowSensitivePtaResult` to old solver for existing test fixtures
3. **Existing integration tests**: All current `fspta` tests pass (zero regression)
4. **CruxBC benchmark**: Compare RSS (instrumentation already present) and runtime
5. **tmux stress test**: Verify no longer OOMs, measure peak RSS

## Non-Goals

- **Semi-sparse flow sensitivity**: Orthogonal optimization (only track FS for address-taken vars). Can be layered on top of VFS later.
- **im-rc persistent maps**: VFS eliminates the need for persistent data structures — version numbers are already O(1) to copy.
- **Solver strategy flag**: No Classic/VFS toggle. Replace the old approach directly.
