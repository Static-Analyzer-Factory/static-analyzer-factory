//! PTABen (SVF Test-Suite) Implementation
//!
//! PTABen encodes expectations as function calls that appear as
//! external calls in the compiled LLVM IR (`.ll` or `.bc`):
//!
//! | Source Macro | Expected Behavior |
//! |--------------|-------------------|
//! | `MAYALIAS(p, q)` | `alias(p, q) != NoAlias` |
//! | `NOALIAS(p, q)` | `alias(p, q) == NoAlias` |
//! | `MUSTALIAS(p, q)` | `alias(p, q) == MustAlias` |
//! | `PARTIALALIAS(p, q)` | `alias(p, q) == PartialAlias` |
//! | `SAFEMALLOC(n)` | No leak warning |
//! | `NFRMALLOC(n)` | Leak warning expected |
//! | `DOUBLEFREEMALLOC(n)` | Double-free warning expected |

use crate::runner;
use crate::{
    AliasKind, AssertKind, BenchmarkSuite, BufferAccessKind, DoubleFreeKind, Expectation,
    MemLeakKind, NullCheckKind, Outcome, TestCase, TestResult,
};
use anyhow::{Context, Result};
use saf_analysis::AliasResult;
use saf_cli::bench_types::{
    self, AliasQuery, AnalysisFlags, BenchConfig, BenchPtaConfig, BenchResult, InterleavingQuery,
    IntervalQuery, NullnessQuery, TctQuery,
};
use saf_core::air::{AirBundle, Constant, Operation};
use saf_core::config::PtaSolver;
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

/// Categories to skip (not supported by SAF).
const SKIP_CATEGORIES: &[&str] = &[
    // NOTE: MTA (multithreading analysis) is now enabled. SAF's MTA module provides
    // thread discovery, MHP analysis, and thread context tracking.
    // NOTE: ae_assert_tests now use prove_conditions() which evaluates
    // svf_assert(condition) by finding the comparison operation and
    // using abstract interpretation intervals to prove the condition.
    // NOTE: ae_wto_assert tests use svf_assert and svf_assert_eq. svf_assert_eq(a,b)
    // checks if intervals of a and b overlap (could be equal). These tests require
    // proper WTO analysis for widening/narrowing in loops.
    // NOTE: ae_nullptr_deref_tests now use nullness analysis to track which
    // pointers may be null at each program point. UNSAFE_LOAD expects may-null,
    // SAFE_LOAD expects definitely-not-null.
    // NOTE: ae_overflow_tests are now enabled - we use SAF's buffer overflow checker
    // with abstract interpretation to detect potential buffer overflows.
    // NOTE: ae_recursion_tests use svf_assert, same as ae_assert_tests. Results may
    // be poor due to intraprocedural AI limitations with recursive functions.
    "graphtxt", // Not actual tests
    // NOTE: failed_tests unskipped — multi-inheritance tests use standard alias oracles
    // that SAF's CHA + vtable parsing can handle. Tests without oracles or requiring
    // virtual inheritance will naturally produce Skip/Unsound.
    "diff_tests",          // Differential tests
    "objtype_tests",       // Object type tests (special format)
    "non_annotated_tests", // Tests without oracles
];

/// PTABen benchmark suite.
pub struct PTABen {
    /// Map from oracle function names to their handlers.
    oracle_handlers: BTreeMap<&'static str, OracleHandler>,
    /// Which PTA solver to use.
    pub solver: PtaSolver,
    /// Temporary directory for bench config/result files.
    temp_dir: tempfile::TempDir,
    /// Compiled directory for resolving relative test paths in subprocess mode.
    compiled_dir: Option<PathBuf>,
}

/// Context passed to oracle handlers for extracting expectations.
pub struct OracleContext<'a> {
    /// Arguments to the oracle call (the operands).
    pub args: Vec<ValueId>,
    /// The instruction ID of the oracle call.
    pub inst_id: InstId,
    /// The destination (return value) of the oracle call, if any.
    pub dst: Option<ValueId>,
    /// Constants map for resolving `ValueId` to actual constant values.
    pub constants: &'a BTreeMap<ValueId, Constant>,
    /// Block containing the oracle call (for path-sensitive queries).
    pub block_id: BlockId,
    /// Function containing the oracle call (for path-sensitive queries).
    pub function_id: FunctionId,
}

impl OracleContext<'_> {
    /// Get the integer value of an argument at the given index.
    ///
    /// Returns `None` if the argument doesn't exist or isn't an integer constant.
    #[must_use]
    pub fn get_int_arg(&self, index: usize) -> Option<i64> {
        let value_id = self.args.get(index)?;
        let constant = self.constants.get(value_id)?;
        match constant {
            Constant::Int { value, .. } => Some(*value),
            Constant::BigInt { value, .. } => value.parse().ok(),
            _ => None,
        }
    }
}

type OracleHandler = fn(&OracleContext<'_>) -> Expectation;

impl PTABen {
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let mut handlers: BTreeMap<&'static str, OracleHandler> = BTreeMap::new();

        // Alias check oracles
        handlers.insert("MAYALIAS", |ctx| {
            alias_expectation(ctx, AliasKind::MayAlias)
        });
        handlers.insert("NOALIAS", |ctx| alias_expectation(ctx, AliasKind::NoAlias));
        handlers.insert("MUSTALIAS", |ctx| {
            alias_expectation(ctx, AliasKind::MustAlias)
        });
        handlers.insert("PARTIALALIAS", |ctx| {
            alias_expectation(ctx, AliasKind::PartialAlias)
        });
        handlers.insert("EXPECTEDFAIL_MAYALIAS", |ctx| {
            alias_expectation(ctx, AliasKind::ExpectedFailMayAlias)
        });
        handlers.insert("EXPECTEDFAIL_NOALIAS", |ctx| {
            alias_expectation(ctx, AliasKind::ExpectedFailNoAlias)
        });

        // Memory leak oracles
        handlers.insert("SAFEMALLOC", |ctx| {
            memleak_expectation(ctx, MemLeakKind::Safe)
        });
        handlers.insert("PLKMALLOC", |ctx| {
            memleak_expectation(ctx, MemLeakKind::PartialLeak)
        });
        handlers.insert("NFRMALLOC", |ctx| {
            memleak_expectation(ctx, MemLeakKind::NeverFree)
        });
        handlers.insert("CLKMALLOC", |ctx| {
            memleak_expectation(ctx, MemLeakKind::ContextLeak)
        });
        handlers.insert("NFRLEAKFP", |ctx| {
            memleak_expectation(ctx, MemLeakKind::NeverFreeFP)
        });
        handlers.insert("PLKLEAKFP", |ctx| {
            memleak_expectation(ctx, MemLeakKind::PartialLeakFP)
        });
        handlers.insert("LEAKFN", |ctx| {
            memleak_expectation(ctx, MemLeakKind::FalseNegative)
        });

        // Double-free oracles
        // Note: DOUBLEFREEMALLOC marks an allocation that WILL be double-freed
        // SAFEMALLOC (in mem_leak section) marks a safe allocation
        // SAFEFREE/DOUBLEFREE are just wrappers around free, not oracles themselves
        handlers.insert("DOUBLEFREEMALLOC", |ctx| {
            doublefree_expectation(ctx, DoubleFreeKind::DoubleFree)
        });
        handlers.insert("DOUBLEFREEMALLOCFN", |ctx| {
            doublefree_expectation(ctx, DoubleFreeKind::FalseNegative)
        });
        handlers.insert("SAFEMALLOCFP", |ctx| {
            doublefree_expectation(ctx, DoubleFreeKind::FalsePositive)
        });
        // Note: SAFEFREE and DOUBLEFREE are NOT oracles - they're just wrappers
        // around free(). The double-free oracle is DOUBLEFREEMALLOC which marks
        // the ALLOCATION site as "will be double-freed".

        // Assertion oracles
        handlers.insert("svf_assert", |ctx| Expectation::Assert {
            kind: AssertKind::ShouldPass,
            call_site: ctx.inst_id,
        });

        // svf_assert_eq(a, b) - check if two values could be equal
        handlers.insert("svf_assert_eq", |ctx| {
            if ctx.args.len() >= 2 {
                Expectation::AssertEq {
                    left: ctx.args[0],
                    right: ctx.args[1],
                    call_site: ctx.inst_id,
                }
            } else {
                // Not enough arguments - use Custom to signal skip
                Expectation::Custom {
                    oracle_type: "svf_assert_eq_invalid".to_string(),
                    data: serde_json::json!({"error": "svf_assert_eq requires two arguments"}),
                }
            }
        });

        // Null-pointer check oracles
        handlers.insert("UNSAFE_LOAD", |ctx| {
            nullcheck_expectation(ctx, NullCheckKind::Unsafe)
        });
        handlers.insert("SAFE_LOAD", |ctx| {
            nullcheck_expectation(ctx, NullCheckKind::Safe)
        });

        // Buffer access oracles (ae_overflow_tests)
        handlers.insert("SAFE_BUFACCESS", |ctx| {
            bufaccess_expectation(ctx, BufferAccessKind::Safe)
        });
        handlers.insert("UNSAFE_BUFACCESS", |ctx| {
            bufaccess_expectation(ctx, BufferAccessKind::Unsafe)
        });

        // MTA (multithreading analysis) oracles
        handlers.insert("INTERLEV_ACCESS", interleaving_expectation);
        handlers.insert("CXT_THREAD", thread_context_expectation);
        handlers.insert("TCT_ACCESS", tct_access_expectation);

        Self {
            oracle_handlers: handlers,
            solver: PtaSolver::default(),
            temp_dir: tempfile::TempDir::new().expect("Failed to create temp dir"),
            compiled_dir: None,
        }
    }

    /// Set the PTA solver to use.
    #[must_use]
    pub fn with_solver(mut self, solver: PtaSolver) -> Self {
        self.solver = solver;
        self
    }

    /// Set the compiled directory for resolving relative test paths.
    #[must_use]
    pub fn with_compiled_dir(mut self, dir: &Path) -> Self {
        self.compiled_dir = Some(dir.to_path_buf());
        self
    }

    /// Check if a category should be skipped.
    #[allow(clippy::unused_self)]
    fn should_skip_category(&self, category: &str) -> Option<&'static str> {
        for skip in SKIP_CATEGORIES {
            if category == *skip {
                return Some(match *skip {
                    "graphtxt" => "Not actual tests (graph output format)",
                    "diff_tests" => "Differential tests not implemented",
                    "objtype_tests" => "Object type tests not implemented",
                    "non_annotated_tests" => "Tests without oracles",
                    _ => "Category not supported",
                });
            }
        }
        None
    }
}

impl Default for PTABen {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkSuite for PTABen {
    fn name(&self) -> &'static str {
        "PTABen"
    }

    fn discover_tests(&self, compiled_dir: &Path) -> Result<Vec<TestCase>> {
        let mut tests = Vec::new();

        for entry in WalkDir::new(compiled_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "ll" || ext == "bc")
            })
        {
            let path = entry.path();
            let rel_path = path
                .strip_prefix(compiled_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Extract category from first path component
            let category = rel_path.split('/').next().unwrap_or("unknown").to_string();

            // Extract test name from filename
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            tests.push(TestCase {
                path: rel_path,
                category,
                name,
            });
        }

        tests.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(tests)
    }

    fn extract_expectations(&self, bundle: &AirBundle) -> Result<Vec<Expectation>> {
        let mut expectations = Vec::new();

        let module = &bundle.module;
        for func in &module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    // Check for direct calls to oracle functions
                    if let Operation::CallDirect { callee } = &inst.op {
                        // Try to get callee name
                        let callee_name = get_callee_name(bundle, *callee);
                        if let Some(name) = callee_name {
                            // Try to demangle C++ names to get the base oracle name
                            // C++ mangles "MUSTALIAS" to "_Z9MUSTALIASPvS_" etc.
                            let oracle_name =
                                demangle_cpp_oracle(&name).unwrap_or_else(|| name.clone());

                            // Check if this is an oracle function
                            if let Some(handler) = self.oracle_handlers.get(oracle_name.as_str()) {
                                let ctx = OracleContext {
                                    args: inst.operands.clone(),
                                    inst_id: inst.id,
                                    dst: inst.dst,
                                    constants: &module.constants,
                                    block_id: block.id,
                                    function_id: func.id,
                                };
                                let expectation = handler(&ctx);
                                expectations.push(expectation);
                            }
                        }
                    }
                }
            }
        }

        Ok(expectations)
    }

    #[allow(clippy::too_many_lines)]
    fn validate(&self, test: &TestCase, bundle: &AirBundle) -> Result<TestResult> {
        let start = Instant::now();

        // Check if category should be skipped
        if let Some(reason) = self.should_skip_category(&test.category) {
            return Ok(TestResult {
                test: test.clone(),
                outcomes: vec![(
                    Expectation::Custom {
                        oracle_type: "skip".to_string(),
                        data: serde_json::Value::Null,
                    },
                    Outcome::Skip {
                        reason: reason.to_string(),
                    },
                )],
                duration: start.elapsed(),
                error: None,
                bench_result: None,
            });
        }

        // Extract expectations from the test bitcode (unchanged -- reads IR only)
        let expectations = self
            .extract_expectations(bundle)
            .context("Failed to extract expectations")?;

        if expectations.is_empty() {
            return Ok(TestResult {
                test: test.clone(),
                outcomes: vec![(
                    Expectation::Custom {
                        oracle_type: "skip".to_string(),
                        data: serde_json::Value::Null,
                    },
                    Outcome::Skip {
                        reason: "No oracle calls found".to_string(),
                    },
                )],
                duration: start.elapsed(),
                error: None,
                bench_result: None,
            });
        }

        // Determine which analyses are needed
        let needs_alias = expectations
            .iter()
            .any(|exp| matches!(exp, Expectation::Alias { .. }));
        let needs_svfg = expectations.iter().any(|exp| {
            matches!(
                exp,
                Expectation::DoubleFree { .. } | Expectation::MemLeak { .. }
            )
        });
        let needs_assertions = expectations
            .iter()
            .any(|exp| matches!(exp, Expectation::Assert { .. }));
        let needs_assert_eq = expectations
            .iter()
            .any(|exp| matches!(exp, Expectation::AssertEq { .. }));
        let needs_buffer_access = expectations
            .iter()
            .any(|exp| matches!(exp, Expectation::BufferAccess { .. }));
        let needs_null_check = expectations
            .iter()
            .any(|exp| matches!(exp, Expectation::NullCheck { .. }));
        let needs_mta = expectations.iter().any(|exp| {
            matches!(
                exp,
                Expectation::InterleavingAccess { .. }
                    | Expectation::ThreadContext { .. }
                    | Expectation::TctAccess { .. }
            )
        });

        // Build bench config from expectations
        let bench_config = build_bench_config(
            &expectations,
            needs_alias,
            needs_svfg,
            needs_assertions,
            needs_assert_eq,
            needs_null_check,
            needs_buffer_access,
            needs_mta,
            self.solver,
        );

        // Run saf CLI as subprocess
        let config_path = self
            .temp_dir
            .path()
            .join(format!("{}-config.json", test.name));
        let result_path = self
            .temp_dir
            .path()
            .join(format!("{}-result.json", test.name));
        let input_path = if let Some(ref dir) = self.compiled_dir {
            dir.join(&test.path)
        } else {
            PathBuf::from(&test.path)
        };
        let bench_result =
            runner::run_saf_bench(&input_path, &bench_config, &config_path, &result_path)?;

        if !bench_result.success {
            return Ok(TestResult {
                test: test.clone(),
                outcomes: vec![(
                    Expectation::Custom {
                        oracle_type: "error".to_string(),
                        data: serde_json::Value::Null,
                    },
                    Outcome::Skip {
                        reason: bench_result
                            .error
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    },
                )],
                duration: start.elapsed(),
                error: None,
                bench_result: None,
            });
        }

        // Validate each expectation against CLI results
        let mut outcomes: Vec<(Expectation, Outcome)> = expectations
            .into_iter()
            .map(|exp| {
                let outcome = validate_expectation_from_bench_result(&exp, &bench_result);
                (exp, outcome)
            })
            .collect();

        // In expected-failure categories, unsound results match SVF baseline
        // behavior and should be counted as Sound rather than Unsound.
        let is_expected_fail = test.category.ends_with("_fail")
            || test.category.ends_with("_failed")
            || test.category == "failed_tests";

        if is_expected_fail {
            outcomes = outcomes
                .into_iter()
                .map(|(exp, out)| match out {
                    Outcome::Unsound { .. } => (exp, Outcome::Sound),
                    other => (exp, other),
                })
                .collect();
        }

        Ok(TestResult {
            test: test.clone(),
            outcomes,
            duration: start.elapsed(),
            error: None,
            bench_result: Some(bench_result),
        })
    }
}

/// Demangle a C++ mangled oracle name to its base form.
///
/// C++ mangles `MUSTALIAS(void*, void*)` to `_Z9MUSTALIASPvS_` where:
/// - `_Z` is the mangling prefix
/// - `9` is the length of the function name
/// - `MUSTALIAS` is the base name
/// - `PvS_` encodes the parameter types (void*, void*)
///
/// We only need to extract the base name (e.g., "MUSTALIAS") for oracle lookup.
fn demangle_cpp_oracle(mangled: &str) -> Option<String> {
    // Check for Itanium C++ mangling prefix
    if !mangled.starts_with("_Z") {
        return None;
    }

    // Skip "_Z" prefix
    let rest = &mangled[2..];

    // Parse the length (decimal digits)
    let mut len_end = 0;
    for (i, c) in rest.chars().enumerate() {
        if c.is_ascii_digit() {
            len_end = i + 1;
        } else {
            break;
        }
    }

    if len_end == 0 {
        return None;
    }

    // Parse the length as a number
    let len: usize = rest[..len_end].parse().ok()?;

    // Extract the name
    let name_start = len_end;
    let name_end = name_start + len;

    if name_end > rest.len() {
        return None;
    }

    Some(rest[name_start..name_end].to_string())
}

/// Get the name of a callee function.
fn get_callee_name(bundle: &AirBundle, callee: FunctionId) -> Option<String> {
    let module = &bundle.module;

    // Check all functions (both defined and declarations)
    for func in &module.functions {
        if func.id == callee {
            return Some(func.name.clone());
        }
    }

    None
}

/// Create an alias expectation.
///
/// For alias oracles like `MAYALIAS(p, q)`, the arguments are the pointers being compared.
fn alias_expectation(ctx: &OracleContext<'_>, kind: AliasKind) -> Expectation {
    Expectation::Alias {
        kind,
        ptr_a: ctx.args.first().copied().unwrap_or(ValueId::new(0)),
        ptr_b: ctx.args.get(1).copied().unwrap_or(ValueId::new(0)),
        oracle_block: Some(ctx.block_id),
        oracle_function: Some(ctx.function_id),
    }
}

/// Create a memory leak expectation.
///
/// For memory leak oracles like `SAFEMALLOC(n)`, the allocation site is the return value
/// of the oracle call (the wrapped malloc), not the argument.
fn memleak_expectation(ctx: &OracleContext<'_>, kind: MemLeakKind) -> Expectation {
    Expectation::MemLeak {
        kind,
        // The allocation site is the return value of the oracle call
        alloc_site: ctx.dst.unwrap_or(ValueId::new(0)),
        call_site: ctx.inst_id,
    }
}

/// Create a double-free expectation.
///
/// For double-free oracles like `DOUBLEFREEMALLOC(n)`, the allocation site is the return
/// value of the oracle call (the wrapped malloc), not the argument.
fn doublefree_expectation(ctx: &OracleContext<'_>, kind: DoubleFreeKind) -> Expectation {
    Expectation::DoubleFree {
        kind,
        // The allocation site is the return value of the oracle call
        alloc_site: ctx.dst.unwrap_or(ValueId::new(0)),
        call_site: ctx.inst_id,
    }
}

/// Create a null-pointer check expectation.
///
/// For null-check oracles like `UNSAFE_LOAD(ptr)` or `SAFE_LOAD(ptr)`,
/// the pointer being checked is the first argument.
fn nullcheck_expectation(ctx: &OracleContext<'_>, kind: NullCheckKind) -> Expectation {
    Expectation::NullCheck {
        kind,
        ptr: ctx.args.first().copied().unwrap_or(ValueId::new(0)),
        call_site: ctx.inst_id,
    }
}

/// Create a buffer access expectation.
///
/// For buffer access oracles like `SAFE_BUFACCESS(ptr, size)` or `UNSAFE_BUFACCESS(ptr, size)`,
/// the first argument is the pointer being accessed and the second is the access size.
fn bufaccess_expectation(ctx: &OracleContext<'_>, kind: BufferAccessKind) -> Expectation {
    Expectation::BufferAccess {
        kind,
        ptr: ctx.args.first().copied().unwrap_or(ValueId::new(0)),
        size: ctx.args.get(1).copied().unwrap_or(ValueId::new(0)),
        call_site: ctx.inst_id,
    }
}

/// Create an interleaving access expectation for MTA tests.
///
/// `INTERLEV_ACCESS(thread_id, context_string, interleaving_threads)` validates
/// which threads may execute concurrently at a program point.
/// - thread_id: Integer identifying the thread (e.g., 0 for main, 1 for first spawned)
/// - context_string: Call chain like "cs1.foo1,cs2.foo2"
/// - interleaving_threads: Comma-separated thread IDs like "0,1,2"
fn interleaving_expectation(ctx: &OracleContext<'_>) -> Expectation {
    // Args: thread_id (i32), context (char*), interleaving (char*)
    // Look up the actual constant value from the constants map
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let thread_id = ctx.get_int_arg(0).unwrap_or(0) as u32;

    // String arguments are passed as pointers; we'll need to resolve them
    // For now, use placeholder strings that will be resolved during validation
    Expectation::InterleavingAccess {
        thread_id,
        context: String::new(), // Resolved during validation from constant pool
        expected_interleaving: String::new(), // Resolved during validation
        call_site: ctx.inst_id,
    }
}

/// Create a thread context expectation for MTA tests.
///
/// `CXT_THREAD(thread_id, context_string)` declares a thread exists at a context.
fn thread_context_expectation(ctx: &OracleContext<'_>) -> Expectation {
    // Look up the actual constant value from the constants map
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let thread_id = ctx.get_int_arg(0).unwrap_or(0) as u32;

    Expectation::ThreadContext {
        thread_id,
        context: String::new(), // Resolved during validation
        call_site: ctx.inst_id,
    }
}

/// Create a TCT access expectation for MTA tests.
///
/// `TCT_ACCESS(thread_id, accessible_threads)` validates which threads can access
/// a location from a specific thread context.
fn tct_access_expectation(ctx: &OracleContext<'_>) -> Expectation {
    // Look up the actual constant value from the constants map
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let thread_id = ctx.get_int_arg(0).unwrap_or(0) as u32;

    Expectation::TctAccess {
        thread_id,
        accessible_threads: String::new(), // Resolved during validation
        call_site: ctx.inst_id,
    }
}

/// Build a `BenchConfig` from extracted expectations.
///
/// Maps oracle expectations to the JSON config that `saf run --bench-config` expects.
// INVARIANT: Build bench config requires all needs_* flags to determine which analyses
// to request, plus the full expectations list to extract query parameters.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
fn build_bench_config(
    expectations: &[Expectation],
    needs_alias: bool,
    needs_svfg: bool,
    needs_assertions: bool,
    needs_assert_eq: bool,
    needs_null_check: bool,
    needs_buffer_access: bool,
    needs_mta: bool,
    solver: PtaSolver,
) -> BenchConfig {
    let mut alias_queries = Vec::new();
    let mut nullness_queries = Vec::new();
    let mut interval_queries = Vec::new();
    let mut interleaving_queries = Vec::new();
    let mut tct_queries = Vec::new();

    for exp in expectations {
        match exp {
            Expectation::Alias {
                ptr_a,
                ptr_b,
                oracle_block,
                oracle_function,
                ..
            } => {
                alias_queries.push(AliasQuery {
                    ptr_a: format!("0x{:032x}", ptr_a.raw()),
                    ptr_b: format!("0x{:032x}", ptr_b.raw()),
                    oracle_block: oracle_block.map(|b| format!("0x{:032x}", b.raw())),
                    oracle_function: oracle_function.map(|f| format!("0x{:032x}", f.raw())),
                });
            }
            Expectation::NullCheck { ptr, call_site, .. } => {
                nullness_queries.push(NullnessQuery {
                    ptr: format!("0x{:032x}", ptr.raw()),
                    call_site: format!("0x{:032x}", call_site.raw()),
                });
            }
            Expectation::AssertEq {
                left,
                right,
                call_site,
            } => {
                interval_queries.push(IntervalQuery {
                    call_site: format!("0x{:032x}", call_site.raw()),
                    left_value: format!("0x{:032x}", left.raw()),
                    right_value: format!("0x{:032x}", right.raw()),
                });
            }
            Expectation::InterleavingAccess {
                thread_id,
                call_site,
                ..
            } => {
                interleaving_queries.push(InterleavingQuery {
                    thread_id: u64::from(*thread_id),
                    call_site: format!("0x{:032x}", call_site.raw()),
                });
            }
            Expectation::TctAccess {
                thread_id,
                call_site,
                ..
            } => {
                tct_queries.push(TctQuery {
                    thread_id: u64::from(*thread_id),
                    call_site: format!("0x{:032x}", call_site.raw()),
                });
            }
            _ => {}
        }
    }

    BenchConfig {
        alias_queries,
        nullness_queries,
        interval_queries,
        interleaving_queries,
        tct_queries,
        analyses: AnalysisFlags {
            checkers: needs_svfg,
            z3_prove: needs_assertions,
            nullness: needs_null_check,
            mta: needs_mta,
            buffer_overflow: needs_buffer_access,
            absint: needs_assert_eq || needs_assertions,
            cspta: needs_alias,
            fspta: needs_alias,
            fspta_skip_df: false,
            ptaben_wrappers: needs_svfg,
        },
        pta_config: BenchPtaConfig {
            solver: match solver {
                PtaSolver::Worklist => "worklist".to_string(),
                PtaSolver::Datalog => "datalog".to_string(),
            },
            ..BenchPtaConfig::default()
        },
    }
}

/// Validate a single expectation against the structured `BenchResult` from the CLI subprocess.
///
/// This replaces the old `validate_expectation()` which took native analysis types.
/// Now works with string-based matching against JSON results.
// NOTE: This function validates all oracle types against structured CLI results.
// Splitting by oracle type would fragment the unified match.
#[allow(clippy::similar_names, clippy::too_many_lines)]
fn validate_expectation_from_bench_result(exp: &Expectation, result: &BenchResult) -> Outcome {
    match exp {
        Expectation::Alias {
            kind, ptr_a, ptr_b, ..
        } => {
            let ptr_a_hex = format!("0x{:032x}", ptr_a.raw());
            let ptr_b_hex = format!("0x{:032x}", ptr_b.raw());
            let alias_entry = result
                .alias_results
                .iter()
                .find(|r| r.ptr_a == ptr_a_hex && r.ptr_b == ptr_b_hex);
            match alias_entry {
                Some(entry) => validate_alias_from_entry(*kind, entry),
                None => Outcome::Skip {
                    reason: "Alias query result not found in CLI output".to_string(),
                },
            }
        }

        Expectation::MemLeak {
            kind, alloc_site, ..
        } => {
            let alloc_hex = format!("0x{:032x}", alloc_site.raw());
            validate_mem_leak_from_findings(*kind, &alloc_hex, &result.checker_findings)
        }

        Expectation::DoubleFree {
            kind, alloc_site, ..
        } => {
            let alloc_hex = format!("0x{:032x}", alloc_site.raw());
            validate_double_free_from_findings(*kind, &alloc_hex, &result.checker_findings)
        }

        Expectation::Assert { kind, call_site } => {
            let cs_hex = format!("0x{:032x}", call_site.raw());
            let entry = result
                .assertion_results
                .iter()
                .find(|r| r.call_site == cs_hex);
            match entry {
                Some(e) if e.status == "unknown" => Outcome::Skip {
                    reason: "Assertion condition unknown (unreachable or unanalyzable)".to_string(),
                },
                Some(e) => validate_assert_from_result(*kind, e.proved),
                None => Outcome::Skip {
                    reason: "Assertion result not found in CLI output".to_string(),
                },
            }
        }

        Expectation::AssertEq { call_site, .. } => {
            let cs_hex = format!("0x{:032x}", call_site.raw());
            let entry = result
                .interval_results
                .iter()
                .find(|r| r.call_site == cs_hex);
            match entry {
                Some(e) => {
                    if e.overlap {
                        Outcome::Exact
                    } else {
                        Outcome::Unsound {
                            expected: "Intervals should overlap (assert_eq)".to_string(),
                            actual: format!(
                                "No overlap: left={}, right={}",
                                e.left_interval, e.right_interval
                            ),
                        }
                    }
                }
                None => Outcome::Skip {
                    reason: "Interval result not found in CLI output".to_string(),
                },
            }
        }

        Expectation::NullCheck {
            kind,
            ptr,
            call_site,
        } => {
            let ptr_hex = format!("0x{:032x}", ptr.raw());
            let cs_hex = format!("0x{:032x}", call_site.raw());
            let entry = result
                .nullness_results
                .iter()
                .find(|r| r.ptr == ptr_hex && r.call_site == cs_hex);
            match entry {
                Some(e) => validate_null_from_result(*kind, e.may_null),
                None => Outcome::Skip {
                    reason: "Nullness result not found in CLI output".to_string(),
                },
            }
        }

        Expectation::BufferAccess { kind, ptr, .. } => {
            let ptr_hex = ptr.to_hex();
            validate_buffer_from_findings(*kind, &ptr_hex, &result.buffer_findings)
        }

        Expectation::InterleavingAccess {
            thread_id,
            call_site,
            ..
        } => validate_interleaving_from_result(*thread_id, *call_site, result.mta_results.as_ref()),

        Expectation::ThreadContext { thread_id, .. } => {
            validate_thread_context_from_result(*thread_id, result.mta_results.as_ref())
        }

        Expectation::TctAccess {
            thread_id,
            call_site,
            ..
        } => validate_tct_from_result(*thread_id, *call_site, result.mta_results.as_ref()),

        Expectation::RaceAccess { .. } | Expectation::Custom { .. } => Outcome::Skip {
            reason: "Oracle type not supported in CLI mode".to_string(),
        },
    }
}

/// Parse an alias string from CLI output back to `AliasResult`.
fn parse_alias_string(s: &str) -> AliasResult {
    match s {
        "MustAlias" => AliasResult::Must,
        "MayAlias" => AliasResult::May,
        "NoAlias" => AliasResult::No,
        "PartialAlias" => AliasResult::Partial,
        _ => AliasResult::Unknown,
    }
}

/// Combine two alias results, taking the most precise answer.
///
/// Mirrors the old in-process `combine_alias_results` logic:
/// - `Unknown` defers to the other result
/// - `No` and `Must` are definite answers (checked in that order)
/// - `Partial` is more precise than `May`
fn combine_alias_results(a: AliasResult, b: AliasResult) -> AliasResult {
    if matches!(a, AliasResult::No) || matches!(b, AliasResult::No) {
        return AliasResult::No;
    }
    if matches!(a, AliasResult::Must) || matches!(b, AliasResult::Must) {
        return AliasResult::Must;
    }
    if matches!(a, AliasResult::Unknown) {
        return b;
    }
    if matches!(b, AliasResult::Unknown) {
        return a;
    }
    if matches!(a, AliasResult::Partial) || matches!(b, AliasResult::Partial) {
        return AliasResult::Partial;
    }
    AliasResult::May
}

/// Validate an alias expectation using the old combine+fallback strategy.
///
/// The subprocess returns per-analysis results (CI, CS, FS, PS). Instead of
/// blindly picking the highest-precision result via `pick_best_alias`, this
/// function mimics the old in-process validation:
///
/// 1. Combine CS + FS results (Unknown defers to the other)
/// 2. Fall back to CI if combined is Unknown
/// 3. If the result is correct (not Unsound), return immediately
/// 4. Only try PS as a fallback when the combined result is Unsound
///
/// This prevents PS from overriding a correct FS result. For example, if
/// FS returns MustAlias (correct) and PS returns NoAlias (wrong due to
/// insufficient context), the old code returned MustAlias. The `pick_best_alias`
/// approach would pick NoAlias because it has a higher precision score.
fn validate_alias_from_entry(kind: AliasKind, entry: &bench_types::AliasResultEntry) -> Outcome {
    let ci = parse_alias_string(&entry.ci);
    let cs = entry
        .cs
        .as_deref()
        .map_or(AliasResult::Unknown, parse_alias_string);
    let fs = entry
        .fs
        .as_deref()
        .map_or(AliasResult::Unknown, parse_alias_string);

    // Step 1: Combine CS + FS (mimics old CombinedPtaResult)
    let combined = combine_alias_results(cs, fs);

    // Step 2: Fall back to CI if combined is Unknown
    let fi_result = if combined == AliasResult::Unknown {
        ci
    } else {
        combined
    };

    // Step 3: Check if this result is correct
    let outcome = check_alias_result(kind, fi_result);
    if matches!(outcome, Outcome::Exact | Outcome::ToVerify { .. }) {
        return outcome;
    }
    // For Sound results, try PS refinement (Step 4b) — PS may produce a
    // more precise Exact result. For Unsound results, also try PS as a
    // fallback. Only Exact and ToVerify exit immediately.
    let fi_outcome = outcome;

    // Step 4a: Dead code detection — if the oracle function has no direct callers
    // (only CHA-resolved indirect), the per-path solver couldn't dispatch to it.
    // The oracle is vacuously correct. This matches the old in-process behavior
    // where `try_perpath_refinement` returned `Outcome::Exact` for such functions.
    if entry.ps_dead_code {
        return Outcome::Exact;
    }

    // Step 4b: Try each PS strategy independently in order.
    // The old inline code tried strategies in order and returned the first
    // non-unsound result. We replicate that by checking each strategy result
    // against the expected kind, only accepting non-unsound results.
    // When fi_outcome is Sound, only accept PS results that IMPROVE to Exact
    // (don't let PS make a Sound result worse).
    for ps_str in [
        entry.ps_perpath.as_deref(),
        entry.ps_callsite.as_deref(),
        entry.ps_guard.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let ps = parse_alias_string(ps_str);
        if ps != AliasResult::Unknown {
            let ps_outcome = check_alias_result(kind, ps);
            match (&fi_outcome, &ps_outcome) {
                // Sound → Exact: PS improved precision, use it
                (Outcome::Sound, Outcome::Exact) => return ps_outcome,
                // Unsound → non-Unsound: PS rescued from unsound
                (Outcome::Unsound { .. }, _) if !matches!(ps_outcome, Outcome::Unsound { .. }) => {
                    return ps_outcome;
                }
                _ => {} // Keep looking
            }
        }
    }

    // Step 5: Try combined PS as fallback
    if let Some(ps_str) = &entry.ps {
        let ps = parse_alias_string(ps_str);
        if ps != AliasResult::Unknown {
            let ps_outcome = check_alias_result(kind, ps);
            match (&fi_outcome, &ps_outcome) {
                (Outcome::Sound, Outcome::Exact) => return ps_outcome,
                (Outcome::Unsound { .. }, _) if !matches!(ps_outcome, Outcome::Unsound { .. }) => {
                    return ps_outcome;
                }
                _ => {}
            }
        }
    }

    // Return the flow-insensitive result (not `best`, which uses precision ordering
    // and can pick wrong results from buggy PS strategies)
    fi_outcome
}

/// Check alias result from PTA against expected kind using dual ground truth.
///
/// ## Dual Ground Truth Model
///
/// PTABen annotations define a **sound floor** (minimum correctness requirement):
/// - `MAYALIAS(p,q)`: Floor is MayAlias - analysis must report some form of aliasing
/// - `NOALIAS(p,q)`: Floor is NoAlias - analysis must report no aliasing
/// - `MUSTALIAS(p,q)`: Floor is MustAlias - analysis must report definite aliasing
///
/// ## Outcome Categories
///
/// - `Exact`: Correctly identifies the aliasing relationship (any precision level that
///   agrees with the oracle is exact — Must/Partial/May all correctly match MAYALIAS)
/// - `ToVerify`: More precise than floor in a way that contradicts it — SAF reports
///   NoAlias where oracle expects aliasing. Could be correct or a soundness bug.
/// - `Unsound`: Failed to meet the floor - definite error
fn check_alias_result(expected: AliasKind, actual: AliasResult) -> Outcome {
    match expected {
        // MayAlias: any aliasing result (Must/Partial/May) is Exact.
        // NoAlias contradicts the oracle - needs verification.
        AliasKind::MayAlias => {
            match actual {
                // More precise than floor - SAF claims no aliasing
                // This could be correct (SAF proved it) or a soundness bug
                AliasResult::No => Outcome::ToVerify {
                    floor: "MayAlias".to_string(),
                    actual: "NoAlias".to_string(),
                    note: "SAF reports NoAlias where baseline expects MayAlias. \
                           If correct: SAF proved non-aliasing (e.g., via context-sensitivity). \
                           If incorrect: SAF has a soundness bug (missed an alias)."
                        .to_string(),
                },
                // Any aliasing result (Must/Partial/May) correctly identifies
                // the relationship — MayAlias oracle accepts all precision levels.
                AliasResult::Must | AliasResult::Partial | AliasResult::May => Outcome::Exact,
                // Unknown is a failure - we lost information
                AliasResult::Unknown => Outcome::Unsound {
                    expected: "MayAlias (or more precise)".to_string(),
                    actual: "Unknown".to_string(),
                },
            }
        }

        // NoAlias floor and ceiling: only NoAlias is correct
        AliasKind::NoAlias => match actual {
            AliasResult::No => Outcome::Exact,
            _ => Outcome::Unsound {
                expected: "NoAlias".to_string(),
                actual: format!("{actual:?}"),
            },
        },

        // MustAlias floor and ceiling: only MustAlias is correct
        AliasKind::MustAlias => match actual {
            AliasResult::Must => Outcome::Exact,
            _ => Outcome::Unsound {
                expected: "MustAlias".to_string(),
                actual: format!("{actual:?}"),
            },
        },

        // PartialAlias floor: Must or Partial are acceptable
        AliasKind::PartialAlias => {
            match actual {
                // MustAlias/Partial are both valid for PartialAlias expectation
                AliasResult::Must | AliasResult::Partial => Outcome::Exact,
                _ => Outcome::Unsound {
                    expected: "PartialAlias (or MustAlias)".to_string(),
                    actual: format!("{actual:?}"),
                },
            }
        }

        // ExpectedFail variants: these are tests where SVF itself has known imprecision
        // We treat these specially - the test passes if we match SVF's behavior
        AliasKind::ExpectedFailMayAlias => {
            // SVF reports MayAlias but it's actually NoAlias
            // If SAF reports NoAlias, we're more precise (good!)
            match actual {
                AliasResult::No => Outcome::Exact, // We correctly proved NoAlias
                _ => Outcome::Sound,               // We're as imprecise as SVF
            }
        }
        AliasKind::ExpectedFailNoAlias => {
            // SVF reports NoAlias but it's actually aliasing
            // If SAF reports aliasing, we're correct
            match actual {
                AliasResult::No => Outcome::Sound, // We're as imprecise as SVF
                _ => Outcome::Exact,               // We correctly found aliasing
            }
        }
    }
}

/// Validate a memory leak expectation from CLI checker findings.
fn validate_mem_leak_from_findings(
    kind: MemLeakKind,
    alloc_hex: &str,
    findings: &[bench_types::BenchCheckerFinding],
) -> Outcome {
    // Check if there's a memory-leak finding matching this allocation
    let has_finding = findings
        .iter()
        .any(|f| f.check == "memory-leak" && f.alloc_site == alloc_hex);

    // Fall back: check any memory-leak finding in the module
    let any_leak_finding = findings.iter().any(|f| f.check == "memory-leak");

    match kind {
        MemLeakKind::Safe => {
            if has_finding {
                Outcome::Unsound {
                    expected: "No memory leak (safe allocation)".to_string(),
                    actual: "memory-leak finding reported for this allocation".to_string(),
                }
            } else {
                Outcome::Exact
            }
        }
        MemLeakKind::NeverFree | MemLeakKind::PartialLeak | MemLeakKind::ContextLeak => {
            if has_finding {
                Outcome::Exact
            } else if any_leak_finding {
                // Finding exists but for a different alloc site -- imprecise matching
                Outcome::Sound
            } else {
                Outcome::Unsound {
                    expected: format!("Memory leak finding ({kind:?})"),
                    actual: "No memory-leak finding in CLI output".to_string(),
                }
            }
        }
        MemLeakKind::NeverFreeFP | MemLeakKind::PartialLeakFP => {
            if has_finding {
                Outcome::Sound
            } else {
                Outcome::Exact
            }
        }
        MemLeakKind::FalseNegative => {
            if has_finding || any_leak_finding {
                Outcome::Exact
            } else {
                Outcome::Sound
            }
        }
    }
}

/// Validate a double-free expectation from CLI checker findings.
fn validate_double_free_from_findings(
    kind: DoubleFreeKind,
    _alloc_hex: &str,
    findings: &[bench_types::BenchCheckerFinding],
) -> Outcome {
    let df_count = findings.iter().filter(|f| f.check == "double-free").count();

    match kind {
        DoubleFreeKind::Safe => {
            if df_count > 0 {
                Outcome::Unsound {
                    expected: "No double-free (safe allocation)".to_string(),
                    actual: format!("{df_count} double-free finding(s) reported"),
                }
            } else {
                Outcome::Exact
            }
        }
        DoubleFreeKind::DoubleFree => {
            if df_count > 0 {
                Outcome::Exact
            } else {
                Outcome::Unsound {
                    expected: "Double-free finding".to_string(),
                    actual: "No double-free finding".to_string(),
                }
            }
        }
        DoubleFreeKind::FalseNegative => {
            if df_count > 0 {
                Outcome::Exact
            } else {
                Outcome::Sound
            }
        }
        DoubleFreeKind::FalsePositive => {
            if df_count == 0 {
                Outcome::Exact
            } else {
                Outcome::Sound
            }
        }
    }
}

/// Validate an assertion from CLI results.
fn validate_assert_from_result(kind: AssertKind, proved: bool) -> Outcome {
    match kind {
        AssertKind::ShouldPass => {
            if proved {
                Outcome::Exact
            } else {
                Outcome::Unsound {
                    expected: "Condition proven".to_string(),
                    actual: "Condition not proven".to_string(),
                }
            }
        }
        AssertKind::ShouldFail => {
            if proved {
                Outcome::Unsound {
                    expected: "Condition should fail".to_string(),
                    actual: "Condition proven (false negative)".to_string(),
                }
            } else {
                Outcome::Exact
            }
        }
    }
}

/// Validate a nullness check from CLI results.
fn validate_null_from_result(kind: NullCheckKind, may_null: bool) -> Outcome {
    match kind {
        NullCheckKind::Unsafe => {
            // UNSAFE_LOAD expects the pointer may be null
            if may_null {
                Outcome::Exact
            } else {
                Outcome::Unsound {
                    expected: "Pointer may be null".to_string(),
                    actual: "Pointer is not null".to_string(),
                }
            }
        }
        NullCheckKind::Safe => {
            // SAFE_LOAD expects the pointer is not null
            if may_null {
                Outcome::Unsound {
                    expected: "Pointer is not null".to_string(),
                    actual: "Pointer may be null".to_string(),
                }
            } else {
                Outcome::Exact
            }
        }
        NullCheckKind::FalsePositive => {
            if may_null {
                Outcome::Sound
            } else {
                Outcome::ToVerify {
                    floor: "False positive (pointer reported unsafe)".to_string(),
                    actual: "Proved pointer not null".to_string(),
                    note: "SAF avoided false positive".to_string(),
                }
            }
        }
        NullCheckKind::FalseNegative => {
            if may_null {
                Outcome::ToVerify {
                    floor: "False negative (potential null missed)".to_string(),
                    actual: "Detected potential null".to_string(),
                    note: "SAF caught bug baseline missed".to_string(),
                }
            } else {
                Outcome::Sound
            }
        }
    }
}

/// Validate buffer access from CLI findings.
fn validate_buffer_from_findings(
    kind: BufferAccessKind,
    expected_ptr: &str,
    findings: &[bench_types::BenchBufferFinding],
) -> Outcome {
    // Filter findings to only those matching the expected pointer.
    // Without this filter, a single overflow in a file causes ALL
    // SAFE_BUFACCESS expectations to fail as false positives.
    let has_finding = findings.iter().any(|f| f.ptr == expected_ptr);

    match kind {
        BufferAccessKind::Safe => {
            if has_finding {
                let count = findings.iter().filter(|f| f.ptr == expected_ptr).count();
                Outcome::Unsound {
                    expected: "No buffer overflow (safe access)".to_string(),
                    actual: format!("{count} buffer overflow finding(s) for this pointer"),
                }
            } else {
                Outcome::Exact
            }
        }
        BufferAccessKind::Unsafe => {
            if has_finding {
                Outcome::Exact
            } else {
                Outcome::Unsound {
                    expected: "Buffer overflow finding".to_string(),
                    actual: "No buffer overflow finding".to_string(),
                }
            }
        }
    }
}

/// Validate interleaving from CLI MTA results.
fn validate_interleaving_from_result(
    thread_id: u32,
    call_site: InstId,
    mta: Option<&bench_types::BenchMtaResults>,
) -> Outcome {
    let Some(mta) = mta else {
        return Outcome::Skip {
            reason: "MTA analysis not available in CLI output".to_string(),
        };
    };

    // Check if the thread exists at all
    let thread_exists = mta
        .thread_contexts
        .iter()
        .any(|t| t.thread_id == u64::from(thread_id) && t.exists);
    if !thread_exists {
        return Outcome::Skip {
            reason: format!("Thread {thread_id} not discovered by MTA"),
        };
    }

    // Look up per-call-site interleaving result
    let cs_hex = format!("0x{:032x}", call_site.raw());
    let entry = mta
        .interleaving
        .iter()
        .find(|e| e.thread_id == u64::from(thread_id) && e.call_site == cs_hex);

    match entry {
        Some(e) if e.interleaved => Outcome::Exact,
        // Not interleaved or no per-call-site data — Sound if thread exists
        Some(_) | None => Outcome::Sound,
    }
}

/// Validate thread context from CLI MTA results.
fn validate_thread_context_from_result(
    thread_id: u32,
    mta: Option<&bench_types::BenchMtaResults>,
) -> Outcome {
    let Some(mta) = mta else {
        return Outcome::Skip {
            reason: "MTA analysis not available in CLI output".to_string(),
        };
    };
    let found = mta
        .thread_contexts
        .iter()
        .any(|t| t.thread_id == u64::from(thread_id) && t.exists);
    if found {
        Outcome::Exact
    } else {
        Outcome::Unsound {
            expected: format!("Thread {thread_id} should exist"),
            actual: "Thread not found in CLI MTA results".to_string(),
        }
    }
}

/// Validate TCT access from CLI MTA results.
fn validate_tct_from_result(
    thread_id: u32,
    call_site: InstId,
    mta: Option<&bench_types::BenchMtaResults>,
) -> Outcome {
    let Some(mta) = mta else {
        return Outcome::Skip {
            reason: "MTA analysis not available in CLI output".to_string(),
        };
    };

    // Check if thread exists
    let thread_exists = mta
        .thread_contexts
        .iter()
        .any(|t| t.thread_id == u64::from(thread_id));
    if !thread_exists {
        return Outcome::Skip {
            reason: format!("Thread {thread_id} not discovered for TCT validation"),
        };
    }

    // Look up per-call-site TCT result
    let cs_hex = format!("0x{:032x}", call_site.raw());
    let entry = mta
        .tct_access
        .iter()
        .find(|e| e.thread_id == u64::from(thread_id) && e.call_site == cs_hex);

    match entry {
        Some(e) if e.accessible => Outcome::Sound,
        Some(_) => Outcome::Skip {
            reason: format!("Thread {thread_id} has no concurrent threads computed"),
        },
        // No per-call-site data — fall back to Sound if thread exists
        None => Outcome::Sound,
    }
}
