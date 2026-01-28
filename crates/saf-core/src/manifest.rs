//! Cache manifest for tracking module fingerprints across analysis runs.
//!
//! The manifest records which modules were analyzed in a previous run
//! and their content fingerprints, enabling fast change detection on
//! subsequent runs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ids::{ModuleId, ProgramId};

/// Per-module entry in the manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// The module's content-derived ID.
    pub module_id: ModuleId,

    /// BLAKE3 fingerprint of the input file (hex-encoded).
    pub fingerprint: String,

    /// Original input file path (for display/debugging).
    pub input_path: String,

    /// BLAKE3 hash of the serialized `ModuleConstraints` (hex-encoded).
    ///
    /// `None` if constraints have not been computed yet for this module.
    /// Used to detect constraint-level staleness even when the input
    /// file fingerprint is unchanged (e.g., due to upstream module changes
    /// affecting cross-module references).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraint_hash: Option<String>,
}

/// Persisted manifest recording the state of a previous analysis run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CacheManifest {
    /// Program ID from the previous run.
    pub program_id: Option<ProgramId>,

    /// Per-module entries keyed by input file path.
    pub modules: BTreeMap<String, ManifestEntry>,
}

impl CacheManifest {
    /// Load manifest from a cache directory. Returns default if not found.
    pub fn load(cache_dir: &Path) -> Self {
        let path = Self::manifest_path(cache_dir);
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save manifest to a cache directory.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the cache directory cannot be created
    /// or the manifest file cannot be written.
    pub fn save(&self, cache_dir: &Path) -> Result<(), std::io::Error> {
        let path = Self::manifest_path(cache_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, json)
    }

    fn manifest_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("manifest.json")
    }

    /// Compare this manifest (previous run) against current fingerprints.
    ///
    /// Returns lists of unchanged, changed, added, and removed input paths.
    /// Does not check constraint-level staleness; use
    /// [`diff_with_constraints`](Self::diff_with_constraints) for that.
    pub fn diff(&self, current: &BTreeMap<String, String>) -> ManifestDiff {
        self.diff_with_constraints(current, &BTreeMap::new())
    }

    /// Compare this manifest against current fingerprints **and** constraint hashes.
    ///
    /// `current_fingerprints` maps input path to file fingerprint.
    /// `current_constraint_hashes` maps input path to constraint hash.
    ///
    /// A file is classified as `constraint_stale` when its fingerprint is
    /// unchanged but its constraint hash differs from the previous run.
    pub fn diff_with_constraints(
        &self,
        current_fingerprints: &BTreeMap<String, String>,
        current_constraint_hashes: &BTreeMap<String, String>,
    ) -> ManifestDiff {
        let mut unchanged = Vec::new();
        let mut changed = Vec::new();
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut constraint_stale = Vec::new();

        // Check current against previous
        for (path, fingerprint) in current_fingerprints {
            match self.modules.get(path) {
                Some(entry) if entry.fingerprint == *fingerprint => {
                    // Fingerprint unchanged — check constraint hash
                    if let Some(cur_hash) = current_constraint_hashes.get(path) {
                        if entry.constraint_hash.as_deref() != Some(cur_hash.as_str()) {
                            constraint_stale.push(path.clone());
                        }
                    }
                    unchanged.push(path.clone());
                }
                Some(_) => {
                    changed.push(path.clone());
                }
                None => {
                    added.push(path.clone());
                }
            }
        }

        // Check for removed files
        for path in self.modules.keys() {
            if !current_fingerprints.contains_key(path) {
                removed.push(path.clone());
            }
        }

        ManifestDiff {
            unchanged,
            changed,
            added,
            removed,
            constraint_stale,
        }
    }
}

/// Result of comparing previous manifest against current fingerprints.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ManifestDiff {
    /// Files whose fingerprint matches the previous run.
    pub unchanged: Vec<String>,

    /// Files whose fingerprint changed since the previous run.
    pub changed: Vec<String>,

    /// Files present now but not in the previous run.
    pub added: Vec<String>,

    /// Files in the previous run but not present now.
    pub removed: Vec<String>,

    /// Files whose fingerprint is unchanged but whose constraint hash
    /// differs from the previous run (constraint-level staleness).
    pub constraint_stale: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest_diff_shows_all_added() {
        let manifest = CacheManifest::default();
        let mut current = BTreeMap::new();
        current.insert("a.ll".to_string(), "aaa".to_string());
        current.insert("b.ll".to_string(), "bbb".to_string());

        let diff = manifest.diff(&current);
        assert_eq!(diff.added.len(), 2);
        assert!(diff.unchanged.is_empty());
        assert!(diff.changed.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn same_fingerprints_show_unchanged() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "aaa".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: None,
            },
        );

        let mut current = BTreeMap::new();
        current.insert("a.ll".to_string(), "aaa".to_string());

        let diff = manifest.diff(&current);
        assert_eq!(diff.unchanged, vec!["a.ll"]);
        assert!(diff.changed.is_empty());
    }

    #[test]
    fn changed_fingerprint_detected() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "old".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: None,
            },
        );

        let mut current = BTreeMap::new();
        current.insert("a.ll".to_string(), "new".to_string());

        let diff = manifest.diff(&current);
        assert_eq!(diff.changed, vec!["a.ll"]);
        assert!(diff.unchanged.is_empty());
    }

    #[test]
    fn removed_file_detected() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "deleted.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "xxx".to_string(),
                input_path: "deleted.ll".to_string(),
                constraint_hash: None,
            },
        );

        let current = BTreeMap::new(); // empty — file was deleted

        let diff = manifest.diff(&current);
        assert_eq!(diff.removed, vec!["deleted.ll"]);
    }

    #[test]
    fn manifest_roundtrip_through_filesystem() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "test.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(42),
                fingerprint: "deadbeef".to_string(),
                input_path: "test.ll".to_string(),
                constraint_hash: Some("abc123".to_string()),
            },
        );

        manifest.save(tmp.path()).unwrap();
        let loaded = CacheManifest::load(tmp.path());
        assert_eq!(manifest, loaded);
    }

    #[test]
    fn same_fingerprint_same_constraint_hash_is_unchanged() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "aaa".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: Some("hash1".to_string()),
            },
        );

        let mut fingerprints = BTreeMap::new();
        fingerprints.insert("a.ll".to_string(), "aaa".to_string());

        let mut constraint_hashes = BTreeMap::new();
        constraint_hashes.insert("a.ll".to_string(), "hash1".to_string());

        let diff = manifest.diff_with_constraints(&fingerprints, &constraint_hashes);
        assert_eq!(diff.unchanged, vec!["a.ll"]);
        assert!(diff.constraint_stale.is_empty());
    }

    #[test]
    fn same_fingerprint_different_constraint_hash_is_constraint_stale() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "aaa".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: Some("old_hash".to_string()),
            },
        );

        let mut fingerprints = BTreeMap::new();
        fingerprints.insert("a.ll".to_string(), "aaa".to_string());

        let mut constraint_hashes = BTreeMap::new();
        constraint_hashes.insert("a.ll".to_string(), "new_hash".to_string());

        let diff = manifest.diff_with_constraints(&fingerprints, &constraint_hashes);
        // File is still in unchanged (fingerprint matches)
        assert_eq!(diff.unchanged, vec!["a.ll"]);
        // But also flagged as constraint-stale
        assert_eq!(diff.constraint_stale, vec!["a.ll"]);
    }

    #[test]
    fn no_previous_constraint_hash_with_current_is_stale() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "aaa".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: None,
            },
        );

        let mut fingerprints = BTreeMap::new();
        fingerprints.insert("a.ll".to_string(), "aaa".to_string());

        let mut constraint_hashes = BTreeMap::new();
        constraint_hashes.insert("a.ll".to_string(), "new_hash".to_string());

        let diff = manifest.diff_with_constraints(&fingerprints, &constraint_hashes);
        assert_eq!(diff.unchanged, vec!["a.ll"]);
        // No previous hash means it differs from the current hash
        assert_eq!(diff.constraint_stale, vec!["a.ll"]);
    }

    #[test]
    fn constraint_hash_not_checked_when_fingerprint_changed() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "old_fp".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: Some("old_hash".to_string()),
            },
        );

        let mut fingerprints = BTreeMap::new();
        fingerprints.insert("a.ll".to_string(), "new_fp".to_string());

        let mut constraint_hashes = BTreeMap::new();
        constraint_hashes.insert("a.ll".to_string(), "new_hash".to_string());

        let diff = manifest.diff_with_constraints(&fingerprints, &constraint_hashes);
        assert_eq!(diff.changed, vec!["a.ll"]);
        // No constraint staleness — the file itself changed
        assert!(diff.constraint_stale.is_empty());
    }

    #[test]
    fn constraint_hash_skipped_when_not_in_current() {
        let mut manifest = CacheManifest::default();
        manifest.modules.insert(
            "a.ll".to_string(),
            ManifestEntry {
                module_id: ModuleId::new(1),
                fingerprint: "aaa".to_string(),
                input_path: "a.ll".to_string(),
                constraint_hash: Some("old_hash".to_string()),
            },
        );

        let mut fingerprints = BTreeMap::new();
        fingerprints.insert("a.ll".to_string(), "aaa".to_string());

        // No constraint hashes provided — skip constraint check
        let diff = manifest.diff_with_constraints(&fingerprints, &BTreeMap::new());
        assert_eq!(diff.unchanged, vec!["a.ll"]);
        assert!(diff.constraint_stale.is_empty());
    }
}
