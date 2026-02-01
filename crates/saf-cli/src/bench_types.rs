//! JSON data contract for `--bench-config` mode.
//!
//! `BenchConfig` is written by saf-bench, read by saf-cli.
//! `BenchResult` is written by saf-cli, read by saf-bench.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Input: saf-bench → saf-cli
// ---------------------------------------------------------------------------

/// Per-test configuration written by saf-bench.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchConfig {
    /// Alias pointer pairs to query (CI, CS, FS, path-sensitive).
    #[serde(default)]
    pub alias_queries: Vec<AliasQuery>,

    /// Pointer `ValueIds` to query for nullness.
    #[serde(default)]
    pub nullness_queries: Vec<NullnessQuery>,

    /// Interval queries for `svf_assert_eq` validation.
    #[serde(default)]
    pub interval_queries: Vec<IntervalQuery>,

    /// MTA interleaving queries (`INTERLEV_ACCESS` oracles).
    #[serde(default)]
    pub interleaving_queries: Vec<InterleavingQuery>,

    /// MTA TCT access queries (`TCT_ACCESS` oracles).
    #[serde(default)]
    pub tct_queries: Vec<TctQuery>,

    /// Which analysis passes to run.
    #[serde(default)]
    pub analyses: AnalysisFlags,

    /// PTA solver/sensitivity configuration overrides.
    #[serde(default)]
    pub pta_config: BenchPtaConfig,
}

/// A single alias query: check `alias(ptr_a, ptr_b)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasQuery {
    /// First pointer `ValueId` (hex string, e.g. "0xabc...").
    pub ptr_a: String,
    /// Second pointer `ValueId`.
    pub ptr_b: String,
    /// Block containing the oracle call (for path-sensitive refinement).
    #[serde(default)]
    pub oracle_block: Option<String>,
    /// Function containing the oracle call.
    #[serde(default)]
    pub oracle_function: Option<String>,
}

/// A nullness query for a specific load instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullnessQuery {
    /// Pointer `ValueId` to check.
    pub ptr: String,
    /// Call site (instruction) where the oracle appears.
    pub call_site: String,
}

/// An interval query for `svf_assert_eq(a, b)` — checks if intervals overlap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalQuery {
    /// Call site (`InstId` hex) of the `svf_assert_eq` oracle.
    pub call_site: String,
    /// Left operand `ValueId` (hex).
    pub left_value: String,
    /// Right operand `ValueId` (hex).
    pub right_value: String,
}

/// An MTA interleaving query: check concurrent threads at a call site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterleavingQuery {
    /// Thread ID from the `INTERLEV_ACCESS` oracle.
    pub thread_id: u64,
    /// Call site (`InstId` hex) of the oracle.
    pub call_site: String,
}

/// An MTA TCT access query: check thread accessibility at a call site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TctQuery {
    /// Thread ID from the `TCT_ACCESS` oracle.
    pub thread_id: u64,
    /// Call site (`InstId` hex) of the oracle.
    pub call_site: String,
}

/// Flags controlling which analyses to run.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnalysisFlags {
    #[serde(default)]
    pub checkers: bool,
    #[serde(default)]
    pub z3_prove: bool,
    #[serde(default)]
    pub nullness: bool,
    #[serde(default)]
    pub mta: bool,
    #[serde(default)]
    pub buffer_overflow: bool,
    #[serde(default)]
    pub absint: bool,
    #[serde(default)]
    pub cspta: bool,
    #[serde(default)]
    pub fspta: bool,
    /// Skip materializing `df_in`/`df_out` `BTreeMap`s after FS-PTA solve.
    /// Saves ~20 GB on large programs (tmux) where deduped VFS data would
    /// expand into per-entry `BTreeSet`s.  Safe when only `pts` and timing
    /// are needed (`CruxBC` benchmarks).
    #[serde(default)]
    pub fspta_skip_df: bool,
    #[serde(default)]
    pub ptaben_wrappers: bool,
}

/// PTA configuration overrides for bench mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchPtaConfig {
    #[serde(default = "default_field_depth")]
    pub field_depth: u32,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    #[serde(default = "default_true")]
    pub constant_indices: bool,
    #[serde(default = "default_true")]
    pub z3_index_refinement: bool,
    #[serde(default = "default_solver")]
    pub solver: String,
    #[serde(default = "default_entry_strategy")]
    pub entry_point_strategy: String,
    #[serde(default = "default_refinement_iters")]
    pub refinement_max_iterations: usize,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde skip_serializing_if requires &T
fn is_false(b: &bool) -> bool {
    !b
}

fn default_field_depth() -> u32 {
    10
}
fn default_max_iterations() -> usize {
    2_000_000
}
fn default_true() -> bool {
    true
}
fn default_solver() -> String {
    "worklist".to_string()
}
fn default_entry_strategy() -> String {
    "all_defined".to_string()
}
fn default_refinement_iters() -> usize {
    5
}

impl Default for BenchPtaConfig {
    fn default() -> Self {
        Self {
            field_depth: default_field_depth(),
            max_iterations: default_max_iterations(),
            constant_indices: default_true(),
            z3_index_refinement: default_true(),
            solver: default_solver(),
            entry_point_strategy: default_entry_strategy(),
            refinement_max_iterations: default_refinement_iters(),
        }
    }
}

// ---------------------------------------------------------------------------
// Output: saf-cli → saf-bench
// ---------------------------------------------------------------------------

/// Structured result written by saf-cli for saf-bench to parse.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub stats: BenchStats,

    /// Peak RSS of the analysis process in megabytes (`VmHWM` on Linux).
    #[serde(default)]
    pub peak_rss_mb: usize,

    /// Analysis statistics (constraints, PTA, CG metrics).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_stats: Option<BenchAnalysisStats>,
    /// IR statistics (function/instruction/global counts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir_stats: Option<BenchIrStats>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alias_results: Vec<AliasResultEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checker_findings: Vec<BenchCheckerFinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertion_results: Vec<AssertionResultEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interval_results: Vec<IntervalResultEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nullness_results: Vec<NullnessResultEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buffer_findings: Vec<BenchBufferFinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mta_results: Option<BenchMtaResults>,
}

/// IR statistics for a loaded program.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BenchIrStats {
    /// Number of defined (non-declaration) functions.
    pub functions: usize,
    /// Total instructions across all defined functions.
    pub instructions: usize,
    /// Number of global variables.
    pub globals: usize,
}

/// Analysis statistics carried across the subprocess boundary.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BenchAnalysisStats {
    // -- Constraints (pre-HVN) --
    pub addr_constraints: usize,
    pub copy_constraints: usize,
    pub load_constraints: usize,
    pub store_constraints: usize,
    pub gep_constraints: usize,
    // -- Constraints (post-HVN) --
    pub post_hvn_total_constraints: usize,
    // -- Solver --
    pub solve_iterations: usize,
    // -- Pointers --
    pub pta_pointers: usize,
    pub total_pointer_values: usize,
    // -- Locations --
    pub obj_count: usize,
    pub field_location_count: usize,
    // -- Points-to sizes --
    pub avg_pts_size: f64,
    pub avg_pts_size_svf: f64,
    pub max_pts_size: usize,
    // -- Call graph --
    pub cg_nodes: usize,
    pub cg_edges: usize,
    pub cg_callsite_edges: usize,
    pub indirect_calls_resolved: usize,
    /// Number of `CallIndirect` sites in the AIR (before resolution).
    #[serde(default)]
    pub ind_call_sites: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BenchStats {
    pub total_secs: f64,
    pub pta_solve_secs: f64,
    pub refinement_iterations: u32,
    pub defuse_build_secs: f64,
    pub valueflow_build_secs: f64,
    /// Time spent on context-sensitive PTA (seconds), if run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cspta_secs: Option<f64>,
    /// Time spent building MSSA + SVFG (seconds), if run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mssa_svfg_secs: Option<f64>,
    /// Time spent on flow-sensitive PTA solve (seconds), if run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fspta_secs: Option<f64>,
    /// Time spent on frontend loading (seconds).
    #[serde(default)]
    pub frontend_secs: f64,
    /// Time spent building CFGs for FS-PTA (seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfg_build_secs: Option<f64>,
    /// Time spent cloning `PtaResult` for MSSA (seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pta_clone_secs: Option<f64>,
    /// Time spent building `DefUseGraph` for SVFG (seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defuse_local_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasResultEntry {
    pub ptr_a: String,
    pub ptr_b: String,
    /// CI-PTA result: "Must", "May", "No", "Partial", "Unknown"
    pub ci: String,
    /// CS-PTA result (if run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cs: Option<String>,
    /// FS-PTA result (if run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<String>,
    /// Path-sensitive result (if run) — combined best of all PS strategies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ps: Option<String>,
    /// Per-path interprocedural refinement result (PS Strategy 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ps_perpath: Option<String>,
    /// Callsite argument mapping result (PS Strategy 2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ps_callsite: Option<String>,
    /// Guard-based path-sensitive result (PS Strategy 3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ps_guard: Option<String>,
    /// True if the oracle function has no direct callers (only CHA-resolved
    /// indirect callers). The oracle is vacuously correct (dead code).
    #[serde(default, skip_serializing_if = "is_false")]
    pub ps_dead_code: bool,
    /// Best (most precise non-Unknown) result across all levels.
    pub best: String,
    /// Debug: CI-PTA points-to set size for `ptr_a`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_pts_a_size: Option<usize>,
    /// Debug: CI-PTA points-to set size for `ptr_b`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_pts_b_size: Option<usize>,
    /// Debug: CI-PTA uniqueness info for the singleton target (if singleton).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_unique: Option<bool>,
    /// Debug: FS-PTA points-to set size for `ptr_a`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_pts_a_size: Option<usize>,
    /// Debug: FS-PTA points-to set size for `ptr_b`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_pts_b_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchCheckerFinding {
    pub check: String,
    /// Allocation site `ValueId` (hex).
    pub alloc_site: String,
    pub severity: String,
    pub call_sites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResultEntry {
    /// Call site (`InstId` hex) of the assertion oracle.
    pub call_site: String,
    /// `svf_assert` or `svf_assert_eq`
    pub kind: String,
    /// Whether the assertion was proved.
    pub proved: bool,
    /// Status: `proven`, `may_fail`, or `unknown`.
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalResultEntry {
    pub call_site: String,
    pub left: String,
    pub right: String,
    pub left_interval: String,
    pub right_interval: String,
    pub overlap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullnessResultEntry {
    pub ptr: String,
    pub call_site: String,
    /// true if may be null, false if definitely not null.
    pub may_null: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchBufferFinding {
    pub ptr: String,
    pub function: String,
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BenchMtaResults {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interleaving: Vec<BenchInterleavingEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thread_contexts: Vec<BenchThreadContextEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tct_access: Vec<BenchTctEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchInterleavingEntry {
    pub thread_id: u64,
    pub call_site: String,
    pub interleaved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchThreadContextEntry {
    pub thread_id: u64,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchTctEntry {
    pub thread_id: u64,
    pub call_site: String,
    pub accessible: bool,
}
