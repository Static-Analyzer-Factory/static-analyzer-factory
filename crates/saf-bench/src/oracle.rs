//! Oracle verification harness.
//!
//! Loads hand-crafted C programs (compiled to LLVM IR), runs SAF analysis,
//! and compares results against YAML expected results.
//!
//! Reports soundness failures (FAIL), imprecision (WARN), and correct results (PASS).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use saf_analysis::callgraph::{CallGraph, CallGraphNode};
use saf_analysis::cfg::Cfg;
use saf_analysis::{ConstraintSet, PtaConfig, PtaContext};
use saf_core::air::AirBundle;
use saf_core::config::Config;
use saf_core::ids::ValueId;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

/// A single oracle expectation from YAML.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OracleExpectation {
    /// Human-readable description of what this test verifies.
    pub description: String,
    /// Analysis layer (pta, callgraph, cfg, mssa, svfg).
    pub layer: String,
    /// Difficulty level (basic, intermediate, advanced).
    #[serde(default)]
    pub difficulty: String,
    /// The actual expectations to verify.
    #[serde(default)]
    pub expectations: ExpectationBlock,
}

/// Container for all expectation types.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ExpectationBlock {
    /// Points-to expectations.
    #[serde(default)]
    pub points_to: Vec<PointsToExpectation>,
    /// Alias expectations.
    #[serde(default)]
    pub alias: Vec<AliasExpectation>,
    /// Call graph expectations.
    #[serde(default)]
    pub calls: Vec<CallExpectation>,
    /// Indirect call target expectations.
    #[serde(default)]
    pub indirect_targets: Vec<IndirectTargetExpectation>,
    /// CFG edge expectations.
    #[serde(default)]
    pub cfg_edges: Vec<CfgEdgeExpectation>,
    /// MSSA reaching definition expectations.
    #[serde(default)]
    pub reaching_def: Vec<ReachingDefExpectation>,
    /// SVFG value flow expectations.
    #[serde(default)]
    pub value_flow: Vec<ValueFlowExpectation>,
}

/// Expected points-to set for a named pointer.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PointsToExpectation {
    /// Human-readable C variable name.
    pub pointer: String,
    /// Source line for disambiguation.
    #[serde(default)]
    pub at_line: Option<u32>,
    /// SOUNDNESS: these MUST be in `pts(p)` — missing = BUG.
    #[serde(default)]
    pub must_contain: Vec<String>,
    /// PRECISION: only these should be in `pts(p)` — extra = WARN.
    #[serde(default)]
    pub may_only_contain: Vec<String>,
}

/// Expected alias relation between two pointers.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AliasExpectation {
    /// The two pointer variable names.
    pub pair: (String, String),
    /// Expected relation: `must_alias`, `may_alias`, or `no_alias`.
    pub relation: String,
}

/// Expected call edges from a caller to its callees.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CallExpectation {
    /// Caller function name.
    pub caller: String,
    /// Expected callee function names.
    pub callees: Vec<String>,
}

/// Expected indirect call targets resolved by PTA.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct IndirectTargetExpectation {
    /// Source line where the indirect call occurs.
    pub call_site_line: u32,
    /// Expected resolved target function names.
    pub targets: Vec<String>,
}

/// Expected CFG structure for a function.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CfgEdgeExpectation {
    /// Function name.
    pub function: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Expected number of branch points (blocks with >1 successor).
    #[serde(default)]
    pub branch_count: Option<usize>,
}

/// Expected MSSA reaching definition.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReachingDefExpectation {
    /// Line where the value is used.
    pub use_line: u32,
    /// Line where the reaching definition originates.
    pub def_line: u32,
    /// Variable name.
    pub variable: String,
}

/// Expected SVFG value flow edge.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ValueFlowExpectation {
    /// Source line of the value flow.
    pub from_line: u32,
    /// Destination line of the value flow.
    pub to_line: u32,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
}

/// Result of a single oracle test.
#[derive(Debug)]
pub enum OracleVerdict {
    /// All expectations met exactly.
    Pass,
    /// Soundness OK but imprecise (extra items in points-to sets).
    Warn(String),
    /// Soundness failure — missing required items.
    Fail(String),
    /// Could not run (compilation error, missing file, etc.)
    Error(String),
}

/// Summary of oracle suite run.
#[derive(Debug, Default)]
pub struct OracleSummary {
    /// Number of tests that passed.
    pub pass: usize,
    /// Number of tests with imprecision warnings.
    pub warn: usize,
    /// Number of tests with soundness failures.
    pub fail: usize,
    /// Number of tests with errors.
    pub error: usize,
    /// Per-test results.
    pub results: Vec<(String, OracleVerdict)>,
}

/// A single oracle test case.
#[derive(Debug)]
pub struct OracleTestCase {
    /// Display name (e.g., `pta/01_simple_alias`).
    pub name: String,
    /// Analysis layer (pta, callgraph, cfg, mssa, svfg).
    pub layer: String,
    /// Path to the C source file.
    pub c_source: PathBuf,
    /// Path to the expected results YAML file.
    pub expected_yaml: PathBuf,
    /// Path to the compiled LLVM IR file.
    pub compiled_ll: PathBuf,
}

/// Discover oracle test cases in a directory.
///
/// Scans `oracle_dir` for layer subdirectories (pta, callgraph, etc.) and
/// finds `.c` files with matching `.expected.yaml` files.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
///
/// # Panics
///
/// Panics if a directory entry or file lacks a name (should never happen
/// with valid filesystem entries).
pub fn discover_tests(
    oracle_dir: &Path,
    layer_filter: Option<&str>,
) -> Result<Vec<OracleTestCase>> {
    let mut tests = Vec::new();

    for entry in std::fs::read_dir(oracle_dir)? {
        let entry = entry?;
        let layer_path = entry.path();
        if !layer_path.is_dir() {
            continue;
        }
        let layer = layer_path
            .file_name()
            .expect("directory entry has a file name")
            .to_string_lossy()
            .to_string();

        // Skip non-layer directories
        if layer == ".compiled" || layer == "harness" {
            continue;
        }

        // Apply layer filter
        if let Some(filter) = layer_filter {
            if layer != filter {
                continue;
            }
        }

        for file_entry in std::fs::read_dir(&layer_path)? {
            let file_entry = file_entry?;
            let path = file_entry.path();
            if path.extension().is_none_or(|e| e != "c") {
                continue;
            }
            let stem = path
                .file_stem()
                .expect("file has a stem")
                .to_string_lossy()
                .to_string();
            let yaml_path = layer_path.join(format!("{stem}.expected.yaml"));
            let ll_path = oracle_dir
                .join(".compiled")
                .join(&layer)
                .join(format!("{stem}.ll"));

            if !yaml_path.exists() {
                eprintln!("  WARNING: {layer}/{stem}.c has no .expected.yaml — skipping");
                continue;
            }

            tests.push(OracleTestCase {
                name: format!("{layer}/{stem}"),
                layer: layer.clone(),
                c_source: path.clone(),
                expected_yaml: yaml_path,
                compiled_ll: ll_path,
            });
        }
    }

    tests.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tests)
}

impl OracleTestCase {
    /// Run verification for this test case.
    pub fn verify(&self) -> OracleVerdict {
        // Check compiled LLVM IR exists
        if !self.compiled_ll.exists() {
            return OracleVerdict::Error(format!(
                "Compiled IR not found: {}. Run `make compile-oracle` first.",
                self.compiled_ll.display()
            ));
        }

        // Parse expected results
        let yaml_content = match std::fs::read_to_string(&self.expected_yaml) {
            Ok(c) => c,
            Err(e) => return OracleVerdict::Error(format!("Cannot read YAML: {e}")),
        };
        let expectation: OracleExpectation = match serde_yaml::from_str(&yaml_content) {
            Ok(e) => e,
            Err(e) => return OracleVerdict::Error(format!("Cannot parse YAML: {e}")),
        };

        // Load LLVM IR through SAF
        let config = Config::default();
        let frontend = LlvmFrontend::default();
        let ll_path = &self.compiled_ll;
        let bundle = match frontend.ingest(&[ll_path.as_path()], &config) {
            Ok(b) => b,
            Err(e) => return OracleVerdict::Error(format!("Cannot load IR: {e}")),
        };

        match self.layer.as_str() {
            "pta" => verify_pta(&bundle, &expectation),
            "callgraph" => verify_callgraph(&bundle, &expectation),
            "cfg" => verify_cfg(&bundle, &expectation),
            "mssa" => verify_mssa(),
            "svfg" => verify_svfg(),
            other => OracleVerdict::Error(format!("Unknown layer: {other}")),
        }
    }
}

fn verify_pta(bundle: &AirBundle, expectation: &OracleExpectation) -> OracleVerdict {
    // Build name-to-ValueId map from debug info / SSA names
    let name_map = build_name_map(bundle);

    // Run PTA
    let config = PtaConfig::default();
    let mut ctx = PtaContext::new(config);
    let result = ctx.analyze(&bundle.module);

    // Build LocId-to-name reverse map using constraints and name map
    let loc_name_map = build_loc_name_map(&result.constraints, &name_map);

    let mut unsound = Vec::new();
    let mut imprecise = Vec::new();

    for pt in &expectation.expectations.points_to {
        let Some(ptr_id) = name_map.get(&pt.pointer).copied() else {
            unsound.push(format!("Variable '{}' not found in module", pt.pointer));
            continue;
        };

        let pts: std::collections::BTreeSet<String> = result
            .pts
            .get(&ptr_id)
            .map(|locs| {
                locs.iter()
                    .map(|loc| {
                        loc_name_map
                            .get(loc)
                            .cloned()
                            .unwrap_or_else(|| format!("{loc:?}"))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Check must_contain (soundness)
        for must in &pt.must_contain {
            if !pts.contains(must.as_str()) {
                unsound.push(format!(
                    "pts({}) missing '{}' — got {{{}}}",
                    pt.pointer,
                    must,
                    pts.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }

        // Check may_only_contain (precision)
        if !pt.may_only_contain.is_empty() {
            let allowed: std::collections::BTreeSet<&str> =
                pt.may_only_contain.iter().map(String::as_str).collect();
            for actual in &pts {
                if !allowed.contains(actual.as_str()) {
                    imprecise.push(format!(
                        "pts({}) has extra '{}' — expected only {{{}}}",
                        pt.pointer,
                        actual,
                        pt.may_only_contain.join(", ")
                    ));
                }
            }
        }
    }

    if unsound.is_empty() {
        if imprecise.is_empty() {
            OracleVerdict::Pass
        } else {
            OracleVerdict::Warn(imprecise.join("; "))
        }
    } else {
        OracleVerdict::Fail(unsound.join("; "))
    }
}

fn verify_callgraph(bundle: &AirBundle, expectation: &OracleExpectation) -> OracleVerdict {
    // Build call graph
    let cg = CallGraph::build(&bundle.module);

    // Build function name lookup: FunctionId -> name
    let func_names: BTreeMap<saf_core::ids::FunctionId, &str> = bundle
        .module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    // Build reverse: name -> CallGraphNode
    let name_to_node: BTreeMap<&str, &CallGraphNode> = cg
        .nodes
        .iter()
        .filter_map(|n| match n {
            CallGraphNode::Function(fid) => func_names.get(fid).map(|name| (*name, n)),
            CallGraphNode::External { name, .. } => Some((name.as_str(), n)),
            CallGraphNode::IndirectPlaceholder { .. } => None,
        })
        .collect();

    let mut unsound = Vec::new();

    for call_exp in &expectation.expectations.calls {
        let Some(caller_node) = name_to_node.get(call_exp.caller.as_str()).copied() else {
            unsound.push(format!(
                "Caller '{}' not found in call graph",
                call_exp.caller
            ));
            continue;
        };

        let callee_names: Vec<String> = cg
            .callees_of(caller_node)
            .map(|callees| {
                callees
                    .iter()
                    .filter_map(|c| match c {
                        CallGraphNode::Function(fid) => {
                            func_names.get(fid).map(|s| (*s).to_string())
                        }
                        CallGraphNode::External { name, .. } => Some(name.clone()),
                        CallGraphNode::IndirectPlaceholder { .. } => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        for expected_callee in &call_exp.callees {
            if !callee_names.contains(expected_callee) {
                unsound.push(format!(
                    "{} should call {} but doesn't — edges: [{}]",
                    call_exp.caller,
                    expected_callee,
                    callee_names.join(", ")
                ));
            }
        }
    }

    if unsound.is_empty() {
        OracleVerdict::Pass
    } else {
        OracleVerdict::Fail(unsound.join("; "))
    }
}

fn verify_cfg(bundle: &AirBundle, expectation: &OracleExpectation) -> OracleVerdict {
    let mut unsound = Vec::new();

    for cfg_exp in &expectation.expectations.cfg_edges {
        let func = bundle
            .module
            .functions
            .iter()
            .find(|f| f.name == cfg_exp.function);

        let Some(func) = func else {
            unsound.push(format!("Function '{}' not found", cfg_exp.function));
            continue;
        };

        let cfg = Cfg::build(func);

        if let Some(expected_branch_count) = cfg_exp.branch_count {
            // Count branch points (blocks with >1 successor)
            let actual_branches: usize = func
                .blocks
                .iter()
                .filter(|b| {
                    cfg.successors
                        .get(&b.id)
                        .map_or(0, std::collections::BTreeSet::len)
                        > 1
                })
                .count();

            if actual_branches < expected_branch_count {
                unsound.push(format!(
                    "{}: expected at least {} branch points, found {}",
                    cfg_exp.function, expected_branch_count, actual_branches
                ));
            }
        }
    }

    if unsound.is_empty() {
        OracleVerdict::Pass
    } else {
        OracleVerdict::Fail(unsound.join("; "))
    }
}

fn verify_mssa() -> OracleVerdict {
    // TODO: Implement MSSA verification (reaching defs)
    OracleVerdict::Error("MSSA verification not yet implemented".to_string())
}

fn verify_svfg() -> OracleVerdict {
    // TODO: Implement SVFG verification (value flow paths)
    OracleVerdict::Error("SVFG verification not yet implemented".to_string())
}

/// Build a map from human-readable variable names to `ValueId`s.
///
/// Uses SSA register names (from `symbol.display_name` on instructions)
/// and parameter names.
fn build_name_map(bundle: &AirBundle) -> BTreeMap<String, ValueId> {
    let mut map = BTreeMap::new();
    for func in &bundle.module.functions {
        for param in &func.params {
            if let Some(ref name) = param.name {
                map.insert(name.clone(), param.id);
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref sym) = inst.symbol {
                    if let Some(dst) = inst.dst {
                        map.insert(sym.display_name.clone(), dst);
                    }
                }
            }
        }
    }
    map
}

/// Build a reverse map from `LocId` to human-readable names.
///
/// Uses Addr constraints to map `LocId` to name: for each `Addr(ptr, loc)`,
/// the human-readable name of `loc` is derived from the SSA name of `ptr`
/// (which is the alloca instruction's destination). For globals and functions,
/// uses the module's global/function names.
fn build_loc_name_map(
    constraints: &ConstraintSet,
    name_map: &BTreeMap<String, ValueId>,
) -> BTreeMap<saf_core::ids::LocId, String> {
    let mut map = BTreeMap::new();

    // Reverse the name_map: ValueId -> name
    let value_to_name: BTreeMap<ValueId, &str> = name_map
        .iter()
        .map(|(name, vid)| (*vid, name.as_str()))
        .collect();

    // For each Addr constraint, map loc -> the name of the pointer value
    for addr in &constraints.addr {
        if let Some(name) = value_to_name.get(&addr.ptr) {
            map.insert(addr.loc, (*name).to_string());
        }
    }

    map
}

/// Print human-readable oracle report.
pub fn print_report(summary: &OracleSummary) {
    eprintln!();
    eprintln!("=== Oracle Verification Report ===");
    eprintln!();

    let mut current_layer = String::new();
    for (name, verdict) in &summary.results {
        let layer = name.split('/').next().unwrap_or("");
        if layer != current_layer {
            if !current_layer.is_empty() {
                eprintln!();
            }
            current_layer = layer.to_string();
            eprintln!("{}:", layer.to_uppercase());
        }

        let (tag, detail) = match verdict {
            OracleVerdict::Pass => ("[PASS]".to_string(), String::new()),
            OracleVerdict::Warn(msg) => ("[WARN]".to_string(), format!(" — {msg}")),
            OracleVerdict::Fail(msg) => ("[FAIL]".to_string(), format!(" — UNSOUND: {msg}")),
            OracleVerdict::Error(msg) => ("[ERR ]".to_string(), format!(" — {msg}")),
        };
        eprintln!("  {tag} {name}{detail}");
    }

    eprintln!();
    eprintln!(
        "Overall: {} pass | {} imprecision | {} unsound | {} errors",
        summary.pass, summary.warn, summary.fail, summary.error
    );
}
