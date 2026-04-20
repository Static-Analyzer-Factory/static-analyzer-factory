//! Analysis session state for incremental analysis.
//!
//! [`AnalysisSession`] tracks analysis state across incremental runs,
//! including per-function staleness information and constraint hashes.
//! It is persisted as `session.json` inside the cache directory.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use saf_core::ids::{FunctionId, InstId, ModuleId, ProgramId};

use crate::callgraph::CallGraph;
use crate::defuse::DefUseGraph;
use crate::pta::module_constraints::ProgramConstraints;
use crate::pta::{IncrementalPtaState, LocationFactory};
use crate::{PtaResult, ValueFlowGraph};

/// Per-function staleness information for incremental analysis.
///
/// Tracks which analysis passes have been invalidated for a given
/// function, allowing the solver to skip unchanged functions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StalenessInfo {
    /// Version counter for the PTA pass that last analyzed this function.
    pub pta_version: u64,

    /// Version counter for the value-flow pass that last analyzed this function.
    pub vf_version: u64,

    /// Whether this function's PTA results are stale and need recomputation.
    pub is_pta_stale: bool,

    /// Whether this function's value-flow results are stale and need recomputation.
    pub is_vf_stale: bool,
}

impl Default for StalenessInfo {
    fn default() -> Self {
        Self {
            pta_version: 0,
            vf_version: 0,
            is_pta_stale: true,
            is_vf_stale: true,
        }
    }
}

/// Persistent analysis session tracking incremental state.
///
/// Stored as `{cache_dir}/session.json` between analysis runs. Tracks
/// per-function staleness, constraint hashes for change detection, and
/// a run counter for diagnostics.
#[derive(Clone, Serialize, Deserialize)]
pub struct AnalysisSession {
    /// Directory where cache artifacts (session, manifest, summaries) are stored.
    #[serde(skip)]
    cache_dir: PathBuf,

    /// Program ID from the most recent completed analysis run.
    pub program_id: Option<ProgramId>,

    /// Per-function staleness tracking.
    pub function_staleness: BTreeMap<FunctionId, StalenessInfo>,

    /// Constraint hashes from the previous run, keyed by module ID.
    ///
    /// Used to detect constraint-level changes even when the source
    /// fingerprint is unchanged.
    pub previous_constraint_hashes: BTreeMap<ModuleId, String>,

    /// Number of analysis runs completed with this session.
    pub run_count: u64,

    /// Constraint state from the most recent analysis run.
    ///
    /// This is in-memory only (not serialized) — it is rebuilt from the
    /// per-module constraint cache on disk during each incremental run.
    #[serde(skip)]
    pub previous_constraints: Option<ProgramConstraints>,

    // -- In-memory incremental state (not serialized) -----------------------
    // These fields are populated after each pipeline run and consumed on the
    // next incremental run within the same process. Between process restarts
    // the system falls back to full analysis.
    /// Incremental PTA solver state from the previous run.
    #[serde(skip)]
    pub incremental_pta_state: Option<IncrementalPtaState>,

    /// Previous PTA result for constructing the next `PtaResult`.
    #[serde(skip)]
    pub previous_pta_result: Option<PtaResult>,

    /// Previous call graph for incremental CG refinement.
    #[serde(skip)]
    pub previous_call_graph: Option<CallGraph>,

    /// Previous def-use graph for selective VF rebuild.
    #[serde(skip)]
    pub previous_defuse: Option<DefUseGraph>,

    /// Previous value-flow graph for selective VF rebuild.
    #[serde(skip)]
    pub previous_valueflow: Option<ValueFlowGraph>,

    /// Previous indirect call target resolution, keyed by call-site instruction.
    #[serde(skip)]
    pub previous_indirect_targets: BTreeMap<InstId, BTreeSet<FunctionId>>,

    /// Location factory shared across incremental runs.
    #[serde(skip)]
    pub location_factory: Option<LocationFactory>,
}

#[allow(clippy::missing_fields_in_debug)] // In-memory transient state omitted for clarity
impl std::fmt::Debug for AnalysisSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisSession")
            .field("cache_dir", &self.cache_dir)
            .field("program_id", &self.program_id)
            .field("run_count", &self.run_count)
            .field("function_staleness_count", &self.function_staleness.len())
            .field(
                "has_previous_constraints",
                &self.previous_constraints.is_some(),
            )
            .field(
                "has_incremental_state",
                &self.incremental_pta_state.is_some(),
            )
            .finish_non_exhaustive()
    }
}

impl AnalysisSession {
    /// Create a new empty session for the given cache directory.
    #[must_use]
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            program_id: None,
            function_staleness: BTreeMap::new(),
            previous_constraint_hashes: BTreeMap::new(),
            run_count: 0,
            previous_constraints: None,
            incremental_pta_state: None,
            previous_pta_result: None,
            previous_call_graph: None,
            previous_defuse: None,
            previous_valueflow: None,
            previous_indirect_targets: BTreeMap::new(),
            location_factory: None,
        }
    }

    /// Load a session from the cache directory. Returns a fresh session if
    /// the file does not exist or cannot be parsed.
    #[must_use]
    pub fn load(cache_dir: &Path) -> Self {
        let path = Self::session_path(cache_dir);
        match std::fs::read_to_string(&path) {
            Ok(json) => {
                let mut session: Self =
                    serde_json::from_str(&json).unwrap_or_else(|_| Self::new(cache_dir.to_owned()));
                // Restore the non-serialized cache_dir
                cache_dir.clone_into(&mut session.cache_dir);
                session
            }
            Err(_) => Self::new(cache_dir.to_owned()),
        }
    }

    /// Persist the session to `{cache_dir}/session.json`.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the cache directory cannot be created
    /// or the session file cannot be written.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::session_path(&self.cache_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, json)
    }

    /// Increment the run counter. Call this at the start of each analysis run.
    pub fn record_run(&mut self) {
        self.run_count += 1;
    }

    /// Return the cache directory path.
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    fn session_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("session.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_has_default_values() {
        let session = AnalysisSession::new(PathBuf::from("/tmp/test"));
        assert_eq!(session.program_id, None);
        assert!(session.function_staleness.is_empty());
        assert!(session.previous_constraint_hashes.is_empty());
        assert_eq!(session.run_count, 0);
        assert_eq!(session.cache_dir(), Path::new("/tmp/test"));
    }

    #[test]
    fn record_run_increments_count() {
        let mut session = AnalysisSession::new(PathBuf::from("/tmp/test"));
        assert_eq!(session.run_count, 0);
        session.record_run();
        assert_eq!(session.run_count, 1);
        session.record_run();
        assert_eq!(session.run_count, 2);
    }

    #[test]
    fn load_missing_file_returns_fresh_session() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let session = AnalysisSession::load(tmp.path());
        assert_eq!(session.run_count, 0);
        assert_eq!(session.cache_dir(), tmp.path());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let tmp = tempfile::tempdir().expect("create temp dir");

        let mut session = AnalysisSession::new(tmp.path().to_owned());
        session.program_id = Some(ProgramId::new(42));
        session.run_count = 5;
        session.function_staleness.insert(
            FunctionId::new(100),
            StalenessInfo {
                pta_version: 3,
                vf_version: 2,
                is_pta_stale: false,
                is_vf_stale: true,
            },
        );
        session
            .previous_constraint_hashes
            .insert(ModuleId::new(1), "abc123".to_string());

        session.save().expect("save session");

        let loaded = AnalysisSession::load(tmp.path());
        assert_eq!(loaded.program_id, Some(ProgramId::new(42)));
        assert_eq!(loaded.run_count, 5);
        assert_eq!(loaded.cache_dir(), tmp.path());

        let staleness = loaded
            .function_staleness
            .get(&FunctionId::new(100))
            .expect("staleness entry");
        assert_eq!(staleness.pta_version, 3);
        assert_eq!(staleness.vf_version, 2);
        assert!(!staleness.is_pta_stale);
        assert!(staleness.is_vf_stale);

        assert_eq!(
            loaded.previous_constraint_hashes.get(&ModuleId::new(1)),
            Some(&"abc123".to_string()),
        );
    }

    #[test]
    fn new_then_load_roundtrip() {
        let tmp = tempfile::tempdir().expect("create temp dir");

        // Create and save a fresh session
        let session = AnalysisSession::new(tmp.path().to_owned());
        session.save().expect("save session");

        // Load it back
        let loaded = AnalysisSession::load(tmp.path());
        assert_eq!(loaded.run_count, 0);
        assert_eq!(loaded.program_id, None);
        assert!(loaded.function_staleness.is_empty());
        assert!(loaded.previous_constraint_hashes.is_empty());
    }

    #[test]
    fn staleness_info_default_is_stale() {
        let info = StalenessInfo::default();
        assert_eq!(info.pta_version, 0);
        assert_eq!(info.vf_version, 0);
        assert!(info.is_pta_stale);
        assert!(info.is_vf_stale);
    }

    #[test]
    fn load_corrupt_json_returns_fresh_session() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let session_path = tmp.path().join("session.json");
        std::fs::write(&session_path, "not valid json {{{").expect("write corrupt file");

        let loaded = AnalysisSession::load(tmp.path());
        assert_eq!(loaded.run_count, 0);
        assert_eq!(loaded.cache_dir(), tmp.path());
    }
}
