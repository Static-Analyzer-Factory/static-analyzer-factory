//! SAF Benchmark Suite Runner
#![allow(clippy::doc_markdown)]
//!
//! This crate provides infrastructure for running pointer analysis and
//! bug-detection benchmarks against SAF. Each benchmark suite implements
//! the [`BenchmarkSuite`] trait with its own oracle format and validation logic.
//!
//! # Supported Suites
//!
//! - **PTABen** - SVF Test-Suite with ~400 micro-benchmarks
//!   (alias checks, memory leaks, double-frees)
//!
//! # Usage
//!
//! ```bash
//! # Compile test programs first (one-time setup)
//! make compile-ptaben
//!
//! # Run benchmarks
//! saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled
//! ```

pub mod cruxbc;
pub mod juliet;
pub mod oracle;
pub mod ptaben;
pub mod report;
pub mod runner;
pub mod svcomp;

use anyhow::Result;
use saf_core::air::AirBundle;
use saf_core::ids::{InstId, ValueId};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// A benchmark suite with its own oracle format and validation logic.
///
/// Each suite knows how to:
/// 1. Discover test cases from a compiled directory
/// 2. Extract expectations (oracles) from the bitcode
/// 3. Run SAF analysis and compare against expectations
pub trait BenchmarkSuite {
    /// Human-readable name of this suite (e.g., "PTABen").
    fn name(&self) -> &str;

    /// Discover all test cases in the given directory.
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal fails.
    fn discover_tests(&self, compiled_dir: &Path) -> Result<Vec<TestCase>>;

    /// Extract expectations from a test case's compiled bitcode.
    ///
    /// # Errors
    ///
    /// Returns an error if oracle extraction fails.
    fn extract_expectations(&self, bundle: &AirBundle) -> Result<Vec<Expectation>>;

    /// Validate a single test case against SAF analysis results.
    ///
    /// # Errors
    ///
    /// Returns an error if validation encounters an unrecoverable error.
    fn validate(&self, test: &TestCase, bundle: &AirBundle) -> Result<TestResult>;
}

/// A single test case discovered from a benchmark suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Path to the compiled bitcode file (relative to compiled dir).
    pub path: String,

    /// Category/directory the test belongs to (e.g., "basic_c_tests").
    pub category: String,

    /// Test name (usually the filename without extension).
    pub name: String,
}

/// An expectation (oracle) extracted from a test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Expectation {
    /// Alias relationship between two pointers.
    Alias {
        kind: AliasKind,
        /// `ValueId` of pointer A (the first argument to the alias oracle).
        ptr_a: ValueId,
        /// `ValueId` of pointer B (the second argument to the alias oracle).
        ptr_b: ValueId,
        /// Block containing the oracle call (for path-sensitive queries).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        oracle_block: Option<saf_core::ids::BlockId>,
        /// Function containing the oracle call (for path-sensitive queries).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        oracle_function: Option<saf_core::ids::FunctionId>,
    },

    /// Memory leak at an allocation site.
    MemLeak {
        /// Allocation kind: safe, partial-leak, never-free, etc.
        kind: MemLeakKind,
        /// The allocation site `ValueId` (return value of the oracle malloc wrapper).
        alloc_site: ValueId,
        /// The call instruction ID for location tracking.
        call_site: InstId,
    },

    /// Double-free of an allocation.
    DoubleFree {
        /// Double-free kind: safe, double-free, etc.
        kind: DoubleFreeKind,
        /// The allocation site `ValueId` (return value of the oracle malloc wrapper).
        alloc_site: ValueId,
        /// The call instruction ID for location tracking.
        call_site: InstId,
    },

    /// Race condition access (MTA tests).
    RaceAccess {
        /// Pair ID for matched accesses.
        pair_id: i32,
        /// Flags indicating expected race conditions.
        flags: i32,
    },

    /// Thread interleaving access oracle (MTA tests).
    ///
    /// `INTERLEV_ACCESS(thread_id, context, interleaving_threads)` validates
    /// which threads may execute concurrently at a specific program point.
    InterleavingAccess {
        /// Thread ID making the access.
        thread_id: u32,
        /// Thread context string (e.g., "cs1.foo1,cs2.foo2").
        context: String,
        /// Expected concurrent thread IDs as comma-separated string (e.g., "0,1,2").
        expected_interleaving: String,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Thread context declaration oracle (MTA tests).
    ///
    /// `CXT_THREAD(thread_id, context)` declares that a thread with the given ID
    /// exists at the specified context.
    ThreadContext {
        /// Thread ID being declared.
        thread_id: u32,
        /// Expected context string (e.g., "cs1.foo1").
        context: String,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Thread-Context-Tree access oracle (MTA tests).
    ///
    /// `TCT_ACCESS(thread_id, accessible_threads)` validates which threads
    /// can access a location from a specific thread context.
    TctAccess {
        /// Thread ID of the access context.
        thread_id: u32,
        /// Expected accessible thread IDs as comma-separated string.
        accessible_threads: String,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Assertion that should be proven true or false.
    Assert {
        /// The assertion kind.
        kind: AssertKind,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Equality assertion `svf_assert_eq(a, b)` - checks if two values could be equal.
    ///
    /// Used by ae_wto_assert tests. The assertion passes if the abstract intervals
    /// of the two values overlap (i.e., they could potentially be equal).
    AssertEq {
        /// Left operand value.
        left: ValueId,
        /// Right operand value.
        right: ValueId,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Null-pointer check oracle from PTABen.
    NullCheck {
        /// The kind of null check (safe or unsafe).
        kind: NullCheckKind,
        /// The pointer being checked.
        ptr: ValueId,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Buffer access oracle from PTABen ae_overflow_tests.
    ///
    /// Indicates whether a buffer access at a specific pointer should
    /// trigger an overflow warning.
    BufferAccess {
        /// The kind of buffer access (safe or unsafe).
        kind: BufferAccessKind,
        /// The pointer being accessed.
        ptr: ValueId,
        /// The size of the access in bytes.
        size: ValueId,
        /// The call instruction ID.
        call_site: InstId,
    },

    /// Catch-all for future oracle types.
    Custom {
        oracle_type: String,
        data: serde_json::Value,
    },
}

/// Assertion expectation kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertKind {
    /// Assertion that should be proven true (svf_assert).
    ShouldPass,
    /// Assertion expected to fail.
    ShouldFail,
}

/// Expected alias relationship kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AliasKind {
    /// Pointers may alias (conservative).
    MayAlias,
    /// Pointers must not alias.
    NoAlias,
    /// Pointers must alias (same location).
    MustAlias,
    /// Pointers partially alias (overlapping).
    PartialAlias,
    /// Expected false positive for MayAlias.
    ExpectedFailMayAlias,
    /// Expected false positive for NoAlias.
    ExpectedFailNoAlias,
}

/// Memory leak oracle kinds from PTABen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemLeakKind {
    /// Safe allocation (should not be flagged).
    Safe,
    /// Partial leak (some paths leak).
    PartialLeak,
    /// Never freed (definite leak).
    NeverFree,
    /// Context-dependent leak.
    ContextLeak,
    /// Expected false positive (never-free).
    NeverFreeFP,
    /// Expected false positive (partial-leak).
    PartialLeakFP,
    /// Expected false negative.
    FalseNegative,
}

/// Double-free oracle kinds from PTABen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoubleFreeKind {
    /// Safe allocation (should not be flagged).
    Safe,
    /// Double-free detected.
    DoubleFree,
    /// Expected false negative (should be double-free but missed).
    FalseNegative,
    /// Expected false positive (safe but flagged).
    FalsePositive,
}

/// Null-pointer check oracle kinds from PTABen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullCheckKind {
    /// Pointer is safe (not null) at this point - `SAFE_LOAD(ptr)`.
    Safe,
    /// Pointer may be null at this point - `UNSAFE_LOAD(ptr)`.
    Unsafe,
    /// Expected false positive (safe but flagged as unsafe).
    FalsePositive,
    /// Expected false negative (unsafe but missed).
    FalseNegative,
}

/// Buffer access oracle kinds from PTABen ae_overflow_tests.
///
/// These oracles are placed after memory operations (like memcpy) to indicate
/// whether the buffer access at that point should have caused an overflow warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BufferAccessKind {
    /// Buffer access is safe (no overflow expected) - `SAFE_BUFACCESS(ptr, size)`.
    Safe,
    /// Buffer access is unsafe (overflow expected) - `UNSAFE_BUFACCESS(ptr, size)`.
    Unsafe,
}

/// Outcome of validating a single expectation using dual ground truth.
///
/// PTABen's original annotations may be conservative (e.g., `MAYALIAS` where a more
/// precise analysis could prove `NoAlias`). We use a dual ground truth system:
///
/// - **Sound floor**: The minimum precision any correct analysis must achieve
/// - **Precise ceiling**: The most precise correct answer (if known)
///
/// This avoids misleading results where a more precise analysis appears to "fail"
/// when it's actually providing better answers than the baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome")]
pub enum Outcome {
    /// Analysis result matches the precise ceiling (optimal).
    /// The analysis achieved the best possible precision for this test.
    Exact,

    /// Analysis is sound (meets floor) but less precise than ceiling.
    /// This is a correct result, just not maximally precise.
    Sound,

    /// Analysis reported more precise than the sound floor.
    /// This needs manual verification to determine if:
    /// 1. SAF correctly proved a tighter bound (e.g., via context-sensitivity)
    /// 2. SAF has a soundness bug
    ///
    /// These cases are NOT counted as failures since they may be correct.
    ToVerify {
        /// The sound floor that was expected
        floor: String,
        /// What SAF actually reported
        actual: String,
        /// Explanation of why this needs verification
        note: String,
    },

    /// Analysis is unsound (failed to meet the sound floor).
    /// This is a definite error - the analysis missed something it should have caught.
    Unsound {
        /// The expected sound floor
        expected: String,
        /// What SAF actually reported
        actual: String,
    },

    /// Test was skipped (unsupported feature, etc.).
    Skip { reason: String },
}

impl Outcome {
    /// Returns true if the outcome is a success (Exact or Sound).
    pub fn is_pass(&self) -> bool {
        matches!(self, Outcome::Exact | Outcome::Sound)
    }

    /// Returns true if the outcome is a definite failure (Unsound).
    pub fn is_unsound(&self) -> bool {
        matches!(self, Outcome::Unsound { .. })
    }

    /// Returns true if the outcome needs manual verification (ToVerify).
    pub fn is_to_verify(&self) -> bool {
        matches!(self, Outcome::ToVerify { .. })
    }

    /// Returns true if the test was skipped.
    pub fn is_skip(&self) -> bool {
        matches!(self, Outcome::Skip { .. })
    }

    /// Returns true if this is the optimal result (Exact).
    pub fn is_exact(&self) -> bool {
        matches!(self, Outcome::Exact)
    }

    /// Returns true if this is sound but not exact.
    pub fn is_sound_only(&self) -> bool {
        matches!(self, Outcome::Sound)
    }

    // Legacy compatibility methods
    pub fn is_fail(&self) -> bool {
        self.is_unsound()
    }

    pub fn is_more_precise(&self) -> bool {
        self.is_to_verify()
    }
}

/// Result of validating a single test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// The test case that was validated.
    pub test: TestCase,

    /// Outcomes for each expectation in the test.
    pub outcomes: Vec<(Expectation, Outcome)>,

    /// Time taken to analyze this test.
    pub duration: Duration,

    /// Optional error message if analysis failed entirely.
    pub error: Option<String>,

    /// Raw bench result for debug inspection (not serialized in summary).
    #[serde(skip)]
    pub bench_result: Option<saf_cli::bench_types::BenchResult>,
}

impl TestResult {
    /// Returns true if all expectations passed (Exact or Sound) or need verification.
    pub fn passed(&self) -> bool {
        self.error.is_none()
            && self
                .outcomes
                .iter()
                .all(|(_, o)| o.is_pass() || o.is_to_verify())
    }

    /// Returns true if any expectation is unsound (definite failure).
    pub fn failed(&self) -> bool {
        self.error.is_some() || self.outcomes.iter().any(|(_, o)| o.is_unsound())
    }

    /// Returns true if all expectations were skipped.
    pub fn skipped(&self) -> bool {
        self.error.is_none()
            && !self.outcomes.is_empty()
            && self.outcomes.iter().all(|(_, o)| o.is_skip())
    }

    /// Count of exact matches (optimal precision).
    pub fn exact_count(&self) -> usize {
        self.outcomes.iter().filter(|(_, o)| o.is_exact()).count()
    }

    /// Count of sound results (meets floor but not ceiling).
    pub fn sound_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|(_, o)| o.is_sound_only())
            .count()
    }

    /// Count of results needing verification (more precise than floor).
    pub fn to_verify_count(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|(_, o)| o.is_to_verify())
            .count()
    }

    /// Count of unsound results (definite failures).
    pub fn unsound_count(&self) -> usize {
        if self.error.is_some() {
            return 1;
        }
        self.outcomes.iter().filter(|(_, o)| o.is_unsound()).count()
    }

    /// Count of skipped expectations.
    pub fn skip_count(&self) -> usize {
        self.outcomes.iter().filter(|(_, o)| o.is_skip()).count()
    }

    // Legacy compatibility
    pub fn pass_count(&self) -> usize {
        self.exact_count() + self.sound_count()
    }

    pub fn fail_count(&self) -> usize {
        self.unsound_count()
    }
}

/// Summary of running a benchmark suite with dual ground truth outcomes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuiteSummary {
    /// Name of the suite.
    pub suite: String,

    /// Total number of tests.
    pub total: usize,

    /// Exact matches (optimal precision, matches precise ceiling).
    pub exact: usize,

    /// Sound results (meets floor, may be conservative).
    pub sound: usize,

    /// Results needing verification (more precise than floor).
    /// These may be correct (SAF is better) or incorrect (soundness bug).
    pub to_verify: usize,

    /// Unsound results (failed to meet sound floor).
    pub unsound: usize,

    /// Number of skipped tests.
    pub skip: usize,

    /// Results by category.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub by_category: Vec<CategorySummary>,

    /// Unsound test details (definite failures).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsound_details: Vec<FailureDetail>,

    /// Tests needing verification.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to_verify_details: Vec<ToVerifyDetail>,

    /// Sound results details (meets floor but not optimal).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sound_details: Vec<SoundDetail>,

    /// Skipped category reasons.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<SkipDetail>,

    /// Total time taken.
    pub timing_secs: f64,

    /// Total number of `EXPECTEDFAIL` oracles (where SVF has known imprecision).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub expectedfail_total: usize,

    /// Number of `EXPECTEDFAIL` oracles where SAF got the correct answer (beat SVF).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub expectedfail_exact: usize,

    /// Details of expected-imprecision oracles where SAF beat SVF.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expectedfail_details: Vec<ExpectedFailDetail>,

    // Legacy fields for backwards compatibility
    #[serde(default, skip_serializing_if = "is_zero")]
    pub pass: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fail: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<FailureDetail>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub precision_improvements: usize,
}

/// Helper for serde skip_serializing_if
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &usize) -> bool {
    *v == 0
}

/// Summary for a single category with dual ground truth outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: String,
    /// Exact matches (optimal precision).
    pub exact: usize,
    /// Sound results (meets floor).
    pub sound: usize,
    /// Results needing verification.
    pub to_verify: usize,
    /// Unsound results.
    pub unsound: usize,
    /// Skipped tests.
    pub skip: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    /// Total time spent on tests in this category (seconds).
    #[serde(default)]
    pub timing_secs: f64,
    // Legacy fields
    #[serde(default, skip_serializing_if = "is_zero")]
    pub pass: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fail: usize,
}

/// Details about a test failure (unsound result).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetail {
    pub file: String,
    pub expectation: String,
    pub expected: String,
    pub actual: String,
}

/// Details about a test needing verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToVerifyDetail {
    pub file: String,
    pub expectation: String,
    pub floor: String,
    pub actual: String,
    pub note: String,
}

/// Details about a Sound result (meets floor but not optimal precision).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundDetail {
    pub file: String,
    pub expectation: String,
    /// CI result string.
    pub ci: String,
    /// CS result string (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cs: Option<String>,
    /// FS result string (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<String>,
    /// CI-PTA pts sizes and uniqueness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_pts_a: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_pts_b: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_unique: Option<bool>,
    /// FS-PTA pts sizes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs_pts_a: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs_pts_b: Option<usize>,
    /// PS combined result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ps: Option<String>,
    /// PS per-path result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ps_perpath: Option<String>,
    /// PS callsite result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ps_callsite: Option<String>,
    /// PS guard-based result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ps_guard: Option<String>,
    /// PS dead code flag.
    #[serde(default)]
    pub ps_dead_code: bool,
}

/// Details about an expected-imprecision oracle where SAF beat SVF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedFailDetail {
    /// Test file containing the oracle.
    pub file: String,
    /// The oracle expectation (e.g., `EXPECTEDFAIL_MAYALIAS`).
    pub expectation: String,
    /// What SAF correctly determined.
    pub result: String,
}

/// Details about skipped tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipDetail {
    pub category: String,
    pub reason: String,
    pub count: usize,
}
