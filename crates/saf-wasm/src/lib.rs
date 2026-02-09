//! SAF WebAssembly entry point.
//!
//! Provides `analyze()` to run the full pipeline, `query()` to
//! interact with the resulting `ProgramDatabase` via JSON protocol,
//! and `resolve_display()` / `resolve_display_batch()` for
//! human-readable label resolution.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use saf_analysis::cfg::Cfg;
use saf_analysis::cg_refinement::RefinementConfig;
use saf_analysis::database::ProgramDatabase;
use saf_analysis::export::{PgEdge, PgNode, PropertyGraph};
use saf_analysis::pipeline::PipelineConfig;
use saf_analysis::{PtaConfig, ValueFlowConfig, ValueFlowMode, to_property_graph};

use saf_core::air::{AirBundle, AirModule, Operation};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

thread_local! {
    static DB: RefCell<Option<ProgramDatabase>> = const { RefCell::new(None) };
}

/// Runtime configuration parsed from `config_json`.
#[derive(Deserialize, Default)]
struct WasmConfig {
    /// Value-flow mode: "fast" or "precise" (default: "precise").
    #[serde(default)]
    vf_mode: Option<String>,

    /// PTA solver: "worklist" (default) or "datalog".
    #[serde(default)]
    pta_solver: Option<String>,

    /// PTA solver max iterations (default: 10,000).
    #[serde(default)]
    pta_max_iterations: Option<usize>,

    /// Max CG refinement iterations (default: 3).
    #[serde(default)]
    max_refinement_iters: Option<usize>,
}

/// Analyze an AIR JSON bundle and return results as JSON.
///
/// After analysis completes, the `ProgramDatabase` is stored in
/// thread-local storage for subsequent `query()` calls.
///
/// # Arguments
///
/// * `air_json` - Serialized `AirBundle` JSON string.
/// * `config_json` - JSON configuration string. Supported fields:
///   - `vf_mode`: "fast" or "precise" (default: "precise")
///   - `pta_solver`: "worklist" (default) or "datalog" — selects PTA solver
///   - `pta_max_iterations`: max PTA solver iterations (default: 10000)
///   - `max_refinement_iters`: max CG refinement iterations (default: 3)
///
/// # Returns
///
/// A JSON string containing an `AnalysisResults` object with `PropertyGraph`
/// exports for every analysis that ran successfully, or an `{"error": "..."}` object.
#[wasm_bindgen]
pub fn analyze(air_json: &str, config_json: &str) -> String {
    console_error_panic_hook::set_once();
    match run_analysis(air_json, config_json) {
        Ok(result) => result,
        Err(e) => {
            let error = serde_json::json!({ "error": e });
            serde_json::to_string(&error)
                .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
        }
    }
}

/// Query the `ProgramDatabase` via JSON protocol.
///
/// The database must have been created by a prior `analyze()` call.
/// Returns a JSON response string matching the SAF JSON protocol.
#[wasm_bindgen]
pub fn query(request_json: &str) -> String {
    DB.with(|cell| {
        let borrow = cell.borrow();
        let Some(db) = borrow.as_ref() else {
            return r#"{"status":"error","error":{"code":"NO_DATABASE","message":"No analysis has been run yet. Call analyze() first.","suggestions":[]}}"#.to_string();
        };
        match db.handle_request(request_json) {
            Ok(response) => response,
            Err(e) => {
                let resp = serde_json::json!({
                    "status": "error",
                    "error": {
                        "code": "SERIALIZATION_ERROR",
                        "message": format!("Failed to serialize response: {e}"),
                        "suggestions": []
                    }
                });
                serde_json::to_string(&resp)
                    .unwrap_or_else(|_| r#"{"status":"error","error":{"code":"SERIALIZATION_ERROR","message":"serialization failed","suggestions":[]}}"#.to_string())
            }
        }
    })
}

/// Resolve a single hex ID to a human-readable label.
///
/// Returns a JSON string containing a `HumanLabel` object, or an
/// `{"error": "..."}` object if the database has not been created yet
/// or the ID is malformed.
///
/// The `id` parameter should be a hex string like `"0x1a2b..."` or
/// a bare hex string like `"1a2b..."`.
///
/// # Example return value
///
/// ```json
/// {"kind":"function","short_name":"main","long_name":"function main (2 blocks, 5 instructions)"}
/// ```
#[wasm_bindgen]
pub fn resolve_display(id: &str) -> String {
    DB.with(|cell| {
        let borrow = cell.borrow();
        let Some(db) = borrow.as_ref() else {
            return r#"{"error":"No analysis has been run yet. Call analyze() first."}"#
                .to_string();
        };
        let raw = match parse_hex_id(id) {
            Ok(v) => v,
            Err(e) => {
                let resp = serde_json::json!({ "error": e });
                return serde_json::to_string(&resp)
                    .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string());
            }
        };
        let resolver = db.display_resolver();
        let label = resolver.resolve(raw);
        serde_json::to_string(&label)
            .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
    })
}

/// Resolve a batch of hex IDs to human-readable labels.
///
/// Accepts a JSON array of hex ID strings and returns a JSON array of
/// `HumanLabel` objects (same order). If the database has not been created,
/// returns an `{"error": "..."}` object.
///
/// # Example
///
/// ```json
/// // Input:  ["0x1a2b", "0x3c4d"]
/// // Output: [{"kind":"function","short_name":"main",...}, {"kind":"unknown","short_name":"%3c4d",...}]
/// ```
#[wasm_bindgen]
pub fn resolve_display_batch(ids_json: &str) -> String {
    DB.with(|cell| {
        let borrow = cell.borrow();
        let Some(db) = borrow.as_ref() else {
            return r#"{"error":"No analysis has been run yet. Call analyze() first."}"#
                .to_string();
        };

        let ids: Vec<String> = match serde_json::from_str(ids_json) {
            Ok(v) => v,
            Err(e) => {
                let resp = serde_json::json!({ "error": format!("Invalid JSON array: {e}") });
                return serde_json::to_string(&resp)
                    .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string());
            }
        };

        let resolver = db.display_resolver();
        let labels: Vec<saf_analysis::HumanLabel> = ids
            .iter()
            .map(|id_str| {
                let raw = parse_hex_id(id_str).unwrap_or(0);
                resolver.resolve(raw)
            })
            .collect();

        serde_json::to_string(&labels)
            .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
    })
}

/// Parse a hex ID string (with or without `0x` prefix) to `u128`.
fn parse_hex_id(id: &str) -> Result<u128, String> {
    let stripped = id.strip_prefix("0x").unwrap_or(id);
    u128::from_str_radix(stripped, 16).map_err(|e| format!("Invalid hex ID \"{id}\": {e}"))
}

/// Collected analysis results matching the TypeScript `AnalysisResults` interface.
#[derive(Serialize)]
struct AnalysisResults {
    cfg: PropertyGraph,
    callgraph: PropertyGraph,
    defuse: PropertyGraph,
    valueflow: PropertyGraph,
    pta: PtaResultExport,
    functions: Vec<String>,
}

/// Minimal PTA export matching the TypeScript `PTAResult` interface.
#[derive(Serialize)]
struct PtaResultExport {
    points_to: Vec<PtaEntryExport>,
    /// Human-readable labels for PTA value IDs (hex → label).
    value_labels: BTreeMap<String, String>,
    /// Human-readable labels for PTA location IDs (hex → label).
    location_labels: BTreeMap<String, String>,
}

/// Single points-to entry matching the TypeScript `PTAEntry` interface.
#[derive(Serialize)]
struct PtaEntryExport {
    value: String,
    locations: Vec<String>,
}

/// Merge multiple per-function `PropertyGraph`s into a single graph.
fn merge_property_graphs(graphs: Vec<PropertyGraph>) -> PropertyGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for g in graphs {
        nodes.extend(g.nodes);
        edges.extend(g.edges);
    }
    PropertyGraph {
        schema_version: "0.1.0".to_string(),
        graph_type: "cfg".to_string(),
        nodes,
        edges,
        metadata: BTreeMap::new(),
    }
}

// NOTE: This function orchestrates the full WASM analysis pipeline including
// parsing, PTA, value-flow, and graph export. Splitting would fragment the result assembly.
#[allow(clippy::too_many_lines)]
fn run_analysis(air_json: &str, config_json: &str) -> Result<String, String> {
    // 0. Parse configuration
    let config: WasmConfig = serde_json::from_str(config_json).unwrap_or_default();

    // 1. Parse AIR JSON into AirBundle
    let bundle: AirBundle =
        serde_json::from_str(air_json).map_err(|e| format!("Failed to parse AIR JSON: {e}"))?;

    // 2. Build PipelineConfig
    let vf_mode = match config.vf_mode.as_deref() {
        Some("fast") => ValueFlowMode::Fast,
        _ => ValueFlowMode::Precise,
    };
    let pipeline_config = PipelineConfig {
        refinement: RefinementConfig {
            max_iterations: config.max_refinement_iters.unwrap_or(3),
            pta_config: PtaConfig {
                max_iterations: config.pta_max_iterations.unwrap_or(10_000),
                ..PtaConfig::default()
            },
            ..RefinementConfig::default()
        },
        valueflow: ValueFlowConfig {
            mode: vf_mode,
            ..ValueFlowConfig::default()
        },
        specs: None,
        build_valueflow: true,
    };

    // 3. Build ProgramDatabase (runs the full pipeline)
    let db = ProgramDatabase::build(bundle.module, &pipeline_config);
    let module = db.module();

    // Run Ascent Datalog PTA only when explicitly requested
    let ascent_pta = if config.pta_solver.as_deref() == Some("datalog")
        || config.pta_solver.as_deref() == Some("ascent")
    {
        let pta_config = saf_analysis::PtaConfig {
            max_iterations: config.pta_max_iterations.unwrap_or(10_000),
            ..saf_analysis::PtaConfig::default()
        };
        let ascent_result = saf_datalog::pta::analyze_with_ascent(module, &pta_config, None);
        Some(saf_analysis::PtaResult::new(
            ascent_result.pts,
            Arc::new(ascent_result.factory),
            ascent_result.diagnostics,
        ))
    } else {
        None
    };

    // 4. Build per-function CFGs and merge into a single PropertyGraph
    let cfgs: Vec<(Cfg, &saf_core::air::AirFunction)> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| (Cfg::build(f), f))
        .collect();

    let cfg_pgs: Vec<PropertyGraph> = cfgs
        .iter()
        .map(|(cfg, func)| cfg.to_pg(func, &module.source_files, None))
        .collect();
    let cfg_merged = merge_property_graphs(cfg_pgs);

    // 5. Export def-use graph
    let defuse_pg = db.defuse().to_pg(module, None);

    // 6. Export call graph after refinement, then inject HeapAlloc/Memcpy edges
    let mut callgraph_pg = db.call_graph().to_pg(module, None);
    inject_special_call_edges(&mut callgraph_pg, module);

    // 7. Export value-flow graph
    let vf_pg = to_property_graph(db.valueflow(), module, None);

    // 8. Export PTA points-to sets with human-readable labels via DisplayResolver
    // Use Ascent PTA result if available, otherwise fall back to pipeline PTA
    let pta_ref = ascent_pta.as_ref().or(db.pta_result());
    let pta_export = if let Some(pta) = pta_ref {
        let resolver = db.display_resolver();
        let mut value_labels = BTreeMap::new();
        let mut location_labels = BTreeMap::new();

        // Resolve value labels from points-to map keys
        for value_id in pta.points_to_map().keys() {
            let label = resolver.resolve(value_id.raw());
            value_labels.insert(value_id.to_hex(), label.short_name.clone());
        }

        // Resolve location labels from all referenced location IDs
        for loc_set in pta.points_to_map().values() {
            for loc_id in loc_set {
                location_labels
                    .entry(loc_id.to_hex())
                    .or_insert_with(|| resolver.resolve(loc_id.raw()).short_name.clone());
            }
        }

        PtaResultExport {
            points_to: pta
                .points_to_map()
                .iter()
                .map(|(value, locs)| PtaEntryExport {
                    value: value.to_hex(),
                    locations: locs.iter().map(|l| l.to_hex()).collect(),
                })
                .collect(),
            value_labels,
            location_labels,
        }
    } else {
        PtaResultExport {
            points_to: Vec::new(),
            value_labels: BTreeMap::new(),
            location_labels: BTreeMap::new(),
        }
    };

    // 9. Collect function names
    let functions: Vec<String> = module
        .functions
        .iter()
        .filter(|f| !f.is_declaration)
        .map(|f| f.name.clone())
        .collect();

    // 10. Assemble results
    let results = AnalysisResults {
        cfg: cfg_merged,
        callgraph: callgraph_pg,
        defuse: defuse_pg,
        valueflow: vf_pg,
        pta: pta_export,
        functions,
    };

    let json =
        serde_json::to_string(&results).map_err(|e| format!("Failed to serialize results: {e}"))?;

    // 11. Store ProgramDatabase for subsequent query() calls
    DB.with(|cell| {
        *cell.borrow_mut() = Some(db);
    });

    Ok(json)
}

/// Inject call-graph edges for `HeapAlloc`, `Memcpy`, and `Memset` instructions.
///
/// The core call graph builder only scans `CallDirect`/`CallIndirect`. Special
/// operations like `HeapAlloc { kind: "malloc" }` are invisible to it, even though
/// they represent calls to library functions. This function adds those edges so the
/// visualization shows `main → malloc`, `duplicate → strcpy`, etc.
fn inject_special_call_edges(pg: &mut PropertyGraph, module: &AirModule) {
    // Build a lookup from function name → node ID in the call graph
    let name_to_id: BTreeMap<&str, &str> = pg
        .nodes
        .iter()
        .filter_map(|n| {
            let name = n.properties.get("name")?.as_str()?;
            Some((name, n.id.as_str()))
        })
        .collect();

    // Set of function node IDs in the call graph (used to verify a caller exists)
    let func_node_ids: std::collections::BTreeSet<&str> = pg
        .nodes
        .iter()
        .filter_map(|n| {
            let kind = n.properties.get("kind")?.as_str()?;
            if kind == "function" {
                Some(n.id.as_str())
            } else {
                None
            }
        })
        .collect();

    let mut new_nodes: Vec<PgNode> = Vec::new();
    let mut new_edges: Vec<PgEdge> = Vec::new();
    let existing_node_ids: std::collections::BTreeSet<&str> =
        pg.nodes.iter().map(|n| n.id.as_str()).collect();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let caller_hex = func.id.to_hex();
        if !func_node_ids.contains(caller_hex.as_str()) {
            continue;
        }
        let caller_id = caller_hex.as_str();

        for block in &func.blocks {
            for inst in &block.instructions {
                let target_name = match &inst.op {
                    Operation::HeapAlloc { kind } => kind.as_str(),
                    Operation::Memcpy => "memcpy",
                    Operation::Memset => "memset",
                    _ => continue,
                };

                // Find or create the target node
                let target_id = if let Some(&id) = name_to_id.get(target_name) {
                    id.to_string()
                } else {
                    // Create an external node for this library function
                    let node_id = format!("ext:{target_name}");
                    if !existing_node_ids.contains(node_id.as_str())
                        && !new_nodes.iter().any(|n| n.id == node_id)
                    {
                        let mut props = BTreeMap::new();
                        props.insert(
                            "name".to_string(),
                            serde_json::Value::String(target_name.to_string()),
                        );
                        props.insert(
                            "kind".to_string(),
                            serde_json::Value::String("external".to_string()),
                        );
                        new_nodes.push(PgNode {
                            id: node_id.clone(),
                            labels: vec!["Function".to_string()],
                            properties: props,
                        });
                    }
                    node_id
                };

                // Add edge if not already present
                let already = pg
                    .edges
                    .iter()
                    .chain(new_edges.iter())
                    .any(|e| e.src == caller_id && e.dst == target_id);
                if !already {
                    new_edges.push(PgEdge {
                        src: caller_id.to_string(),
                        dst: target_id,
                        edge_type: "CALLS".to_string(),
                        properties: BTreeMap::new(),
                    });
                }
            }
        }
    }

    pg.nodes.extend(new_nodes);
    pg.edges.extend(new_edges);
}
