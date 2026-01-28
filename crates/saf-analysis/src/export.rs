//! Unified property graph export format.
//!
//! Provides a single export format for all SAF graph types that maps directly
//! to the property graph model (Neo4j-compatible). Supports DOT and interactive
//! HTML (Cytoscape.js) visualization.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;

use saf_core::air::{AirModule, AirType};
use saf_core::ids::TypeId;
use saf_core::span::{SourceFile, Span};

use crate::display::DisplayResolver;

/// Unified property graph export format.
///
/// Maps directly to the property graph model: labeled nodes with properties,
/// typed directed edges with properties. Compatible with Neo4j, Cytoscape.js,
/// and NetworkX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyGraph {
    /// Schema version (currently "0.1.0").
    pub schema_version: String,
    /// Graph type identifier (e.g., "callgraph", "cfg", "valueflow").
    pub graph_type: String,
    /// Graph-level metadata (stats, diagnostics, etc.).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, serde_json::Value>,
    /// All nodes in the graph.
    pub nodes: Vec<PgNode>,
    /// All edges in the graph.
    pub edges: Vec<PgEdge>,
}

/// A node in the property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgNode {
    /// Node ID (hex string).
    pub id: String,
    /// Node labels (e.g., `["Function"]`, `["Block", "Entry"]`).
    pub labels: Vec<String>,
    /// Node properties (name, kind, function, etc.).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, serde_json::Value>,
}

/// An edge in the property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgEdge {
    /// Source node ID (hex string).
    pub src: String,
    /// Destination node ID (hex string).
    pub dst: String,
    /// Edge type (e.g., "CALLS", "FLOWS_TO", "POINTS_TO").
    pub edge_type: String,
    /// Edge properties.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, serde_json::Value>,
}

/// Convert a `Span` into a JSON property value for `PgNode.properties`.
///
/// Returns a `serde_json::Value::Object` with keys: `file`, `line_start`,
/// `col_start`, `line_end`, `col_end`. The `file` is looked up from
/// `source_files` by `file_id`; if not found, it's omitted.
#[must_use]
pub fn span_to_property(span: &Span, source_files: &[SourceFile]) -> serde_json::Value {
    let mut map = serde_json::Map::new();

    if let Some(sf) = source_files.iter().find(|sf| sf.id == span.file_id) {
        map.insert("file".to_string(), serde_json::json!(sf.path));
    }

    map.insert("line_start".to_string(), serde_json::json!(span.line_start));
    map.insert("col_start".to_string(), serde_json::json!(span.col_start));
    map.insert("line_end".to_string(), serde_json::json!(span.line_end));
    map.insert("col_end".to_string(), serde_json::json!(span.col_end));

    serde_json::Value::Object(map)
}

/// Format an `AirType` as a human-readable string for export.
///
/// Produces compact type names suitable for node properties in exported
/// graphs: `ptr`, `i32`, `float`, `double`, `void`, etc.
#[must_use]
pub fn format_type_name(ty: &AirType) -> String {
    match ty {
        AirType::Pointer => "ptr".to_string(),
        AirType::Reference { nullable } => {
            if *nullable {
                "ref?".to_string()
            } else {
                "ref".to_string()
            }
        }
        AirType::Vector { lanes, .. } => format!("vec<{lanes}>"),
        AirType::Integer { bits } => format!("i{bits}"),
        AirType::Float { bits: 32 } => "float".to_string(),
        AirType::Float { bits: 64 } => "double".to_string(),
        AirType::Float { bits } => format!("f{bits}"),
        AirType::Array { count: Some(n), .. } => format!("[{n} x ...]"),
        AirType::Array { count: None, .. } => "[? x ...]".to_string(),
        AirType::Struct {
            fields, total_size, ..
        } => format!("struct({} fields, {total_size}B)", fields.len()),
        AirType::Function { params, .. } => format!("fn({} params)", params.len()),
        AirType::Void => "void".to_string(),
        AirType::Opaque => "opaque".to_string(),
    }
}

/// Build a lookup from `ValueId` (raw u128) to `TypeId` for all values in the module.
///
/// Covers instruction results (via `result_type`) and function parameters
/// (via `param_type`). The returned map can be used by graph exporters to
/// annotate nodes with human-readable type names.
#[must_use]
pub fn build_value_type_lookup(module: &AirModule) -> BTreeMap<u128, TypeId> {
    let mut map = BTreeMap::new();
    for func in &module.functions {
        for param in &func.params {
            if let Some(tid) = param.param_type {
                map.insert(param.id.raw(), tid);
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let (Some(dst), Some(tid)) = (inst.dst, inst.result_type) {
                    map.insert(dst.raw(), tid);
                }
            }
        }
    }
    map
}

/// Insert a `"type"` property on a node's property map, if the value's type is
/// known in the module's type table.
///
/// This is a convenience helper for graph exporters. It looks up the raw u128
/// ID in `value_types`, resolves the `TypeId` to an `AirType` in
/// `module.types`, and inserts a formatted string like `"ptr"` or `"i32"`.
pub fn insert_type_property(
    properties: &mut BTreeMap<String, serde_json::Value>,
    raw_id: u128,
    value_types: &BTreeMap<u128, TypeId>,
    module: &AirModule,
) {
    if let Some(tid) = value_types.get(&raw_id) {
        if let Some(air_type) = module.get_type(*tid) {
            properties.insert(
                "type".to_string(),
                serde_json::Value::String(format_type_name(air_type)),
            );
        }
    }
}

/// Enrich a [`PgNode`]'s properties with human-readable display information
/// from a [`DisplayResolver`].
///
/// Parses the node's hex `id` back to a `u128`, resolves it through the
/// resolver, and inserts `display_name`, `source_file`, `source_line`, and
/// `source_col` properties when available. No-op if `resolver` is `None` or
/// the hex ID cannot be parsed.
pub fn enrich_node(node: &mut PgNode, resolver: Option<&DisplayResolver<'_>>) {
    let Some(resolver) = resolver else {
        return;
    };

    // Parse hex ID — nodes use "0x<hex>" format.
    let raw_id = parse_hex_id(&node.id);
    let Some(id) = raw_id else {
        return;
    };

    let label = resolver.resolve(id);

    node.properties.insert(
        "display_name".to_string(),
        serde_json::Value::String(label.short_name),
    );

    if let Some(loc) = label.source_loc {
        node.properties.insert(
            "source_file".to_string(),
            serde_json::Value::String(loc.file),
        );
        node.properties
            .insert("source_line".to_string(), serde_json::json!(loc.line));
        node.properties
            .insert("source_col".to_string(), serde_json::json!(loc.col));
    }
}

/// Parse a hex ID string (e.g. `"0x001a2b"`) to a `u128`.
///
/// Returns `None` if the string does not start with `"0x"` or cannot be
/// parsed as hexadecimal.
fn parse_hex_id(hex: &str) -> Option<u128> {
    let stripped = hex.strip_prefix("0x")?;
    u128::from_str_radix(stripped, 16).ok()
}

impl PropertyGraph {
    /// Create a new empty `PropertyGraph`.
    #[must_use]
    pub fn new(graph_type: impl Into<String>) -> Self {
        Self {
            schema_version: "0.1.0".to_string(),
            graph_type: graph_type.into(),
            metadata: BTreeMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Generate a Graphviz DOT representation of this property graph.
    ///
    /// Nodes are shaped based on their labels and labeled with the `name`
    /// property when present. Edges are labeled with their `edge_type`.
    /// When nodes have a `function` property, they are grouped into
    /// subgraph clusters.
    #[must_use]
    pub fn to_dot(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "digraph {{");
        let _ = writeln!(out, "  graph [fontname=\"Helvetica\"];");
        let _ = writeln!(out, "  node [fontname=\"Helvetica\"];");
        let _ = writeln!(out, "  edge [fontname=\"Helvetica\"];");

        // Group nodes by function property for subgraph clustering
        let mut clustered: BTreeMap<String, Vec<&PgNode>> = BTreeMap::new();
        let mut unclustered: Vec<&PgNode> = Vec::new();

        for node in &self.nodes {
            if let Some(serde_json::Value::String(func)) = node.properties.get("function") {
                clustered.entry(func.clone()).or_default().push(node);
            } else {
                unclustered.push(node);
            }
        }

        // Emit unclustered nodes
        for node in &unclustered {
            let _ = writeln!(out, "  {};", format_dot_node(node));
        }

        // Emit clustered subgraphs
        for (i, (func_name, nodes)) in clustered.iter().enumerate() {
            let _ = writeln!(out, "  subgraph cluster_{i} {{");
            let _ = writeln!(out, "    label=\"{}\";", escape_dot(func_name));
            let _ = writeln!(out, "    style=dashed;");
            let _ = writeln!(out, "    color=\"#666666\";");
            for node in nodes {
                let _ = writeln!(out, "    {};", format_dot_node(node));
            }
            let _ = writeln!(out, "  }}");
        }

        // Emit edges
        for edge in &self.edges {
            let _ = writeln!(
                out,
                "  \"{}\" -> \"{}\" [label=\"{}\"];",
                escape_dot(&edge.src),
                escape_dot(&edge.dst),
                escape_dot(&edge.edge_type)
            );
        }

        let _ = writeln!(out, "}}");
        out
    }

    /// Generate a self-contained HTML page with an interactive Cytoscape.js
    /// visualization of this property graph.
    ///
    /// The HTML loads Cytoscape.js, cytoscape-dagre, and cytoscape-cola from
    /// CDN. It includes a toolbar with layout selection, zoom-to-fit, search,
    /// edge type filters, hover tooltips, and click-to-highlight neighbors.
    #[must_use]
    pub fn to_html(&self) -> String {
        let template = include_str!("export_html_template.html");
        // Serialize the graph to JSON — serialization of a valid PropertyGraph
        // cannot fail since all fields are serializable types.
        let json = serde_json::to_string(self).unwrap_or_else(|_| "null".to_string());
        template.replace("/*GRAPH_DATA*/null", &json)
    }
}

/// Format a single node as a DOT node statement (without trailing semicolon/newline).
fn format_dot_node(node: &PgNode) -> String {
    let label = node_display_label(node);
    let shape = node_dot_shape(node);
    format!(
        "\"{}\" [label=\"{}\" shape={}]",
        escape_dot(&node.id),
        escape_dot(&label),
        shape
    )
}

/// Determine the display label for a node.
///
/// Prefers the `name` property, then the first label, then a truncated id.
fn node_display_label(node: &PgNode) -> String {
    if let Some(serde_json::Value::String(name)) = node.properties.get("name") {
        return name.clone();
    }
    if let Some(first_label) = node.labels.first() {
        return first_label.clone();
    }
    if node.id.len() > 12 {
        format!("{}...", &node.id[..12])
    } else {
        node.id.clone()
    }
}

/// Map node labels to DOT shapes.
fn node_dot_shape(node: &PgNode) -> &'static str {
    for label in &node.labels {
        match label.as_str() {
            "Function" => return "box",
            "Block" => return "rectangle",
            "Value" => return "ellipse",
            "MemPhi" => return "diamond",
            "Location" => return "hexagon",
            "Type" => return "component",
            "Thread" => return "parallelogram",
            _ => {}
        }
    }
    "ellipse"
}

/// Escape a string for use inside DOT quoted labels.
fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> PropertyGraph {
        let mut pg = PropertyGraph::new("callgraph");

        let mut props_main = BTreeMap::new();
        props_main.insert(
            "name".to_string(),
            serde_json::Value::String("main".to_string()),
        );

        let mut props_foo = BTreeMap::new();
        props_foo.insert(
            "name".to_string(),
            serde_json::Value::String("foo".to_string()),
        );

        pg.nodes.push(PgNode {
            id: "0x0001".to_string(),
            labels: vec!["Function".to_string()],
            properties: props_main,
        });
        pg.nodes.push(PgNode {
            id: "0x0002".to_string(),
            labels: vec!["Function".to_string()],
            properties: props_foo,
        });

        pg.edges.push(PgEdge {
            src: "0x0001".to_string(),
            dst: "0x0002".to_string(),
            edge_type: "CALLS".to_string(),
            properties: BTreeMap::new(),
        });

        pg
    }

    #[test]
    fn test_new_creates_empty_graph() {
        let pg = PropertyGraph::new("cfg");
        assert_eq!(pg.schema_version, "0.1.0");
        assert_eq!(pg.graph_type, "cfg");
        assert!(pg.nodes.is_empty());
        assert!(pg.edges.is_empty());
        assert!(pg.metadata.is_empty());
    }

    #[test]
    fn test_to_dot_contains_digraph_wrapper() {
        let pg = sample_graph();
        let dot = pg.to_dot();
        assert!(dot.starts_with("digraph {"));
        assert!(dot.trim_end().ends_with('}'));
    }

    #[test]
    fn test_to_dot_node_labels() {
        let pg = sample_graph();
        let dot = pg.to_dot();
        assert!(dot.contains("label=\"main\""));
        assert!(dot.contains("label=\"foo\""));
    }

    #[test]
    fn test_to_dot_node_shapes() {
        let pg = sample_graph();
        let dot = pg.to_dot();
        assert!(dot.contains("shape=box"));
    }

    #[test]
    fn test_to_dot_edge_labels() {
        let pg = sample_graph();
        let dot = pg.to_dot();
        assert!(dot.contains("label=\"CALLS\""));
    }

    #[test]
    fn test_to_dot_escapes_special_chars() {
        let mut pg = PropertyGraph::new("test");
        let mut props = BTreeMap::new();
        props.insert(
            "name".to_string(),
            serde_json::Value::String("say \"hello\"\nworld".to_string()),
        );
        pg.nodes.push(PgNode {
            id: "n1".to_string(),
            labels: vec!["Value".to_string()],
            properties: props,
        });
        let dot = pg.to_dot();
        assert!(dot.contains(r#"say \"hello\"\nworld"#));
    }

    #[test]
    fn test_to_dot_subgraph_clustering() {
        let mut pg = PropertyGraph::new("cfg");

        let mut props_b1 = BTreeMap::new();
        props_b1.insert(
            "name".to_string(),
            serde_json::Value::String("entry".to_string()),
        );
        props_b1.insert(
            "function".to_string(),
            serde_json::Value::String("main".to_string()),
        );

        let mut props_b2 = BTreeMap::new();
        props_b2.insert(
            "name".to_string(),
            serde_json::Value::String("exit".to_string()),
        );
        props_b2.insert(
            "function".to_string(),
            serde_json::Value::String("main".to_string()),
        );

        pg.nodes.push(PgNode {
            id: "b1".to_string(),
            labels: vec!["Block".to_string()],
            properties: props_b1,
        });
        pg.nodes.push(PgNode {
            id: "b2".to_string(),
            labels: vec!["Block".to_string()],
            properties: props_b2,
        });

        let dot = pg.to_dot();
        assert!(dot.contains("subgraph cluster_0"));
        assert!(dot.contains("label=\"main\""));
    }

    #[test]
    fn test_to_dot_fallback_label_uses_first_label() {
        let mut pg = PropertyGraph::new("test");
        pg.nodes.push(PgNode {
            id: "n1".to_string(),
            labels: vec!["Location".to_string()],
            properties: BTreeMap::new(),
        });
        let dot = pg.to_dot();
        assert!(dot.contains("label=\"Location\""));
    }

    #[test]
    fn test_to_dot_fallback_label_truncates_long_id() {
        let mut pg = PropertyGraph::new("test");
        pg.nodes.push(PgNode {
            id: "0x0123456789abcdef0123456789abcdef".to_string(),
            labels: vec![],
            properties: BTreeMap::new(),
        });
        let dot = pg.to_dot();
        assert!(dot.contains("label=\"0x0123456789...\""));
    }

    #[test]
    fn test_to_html_contains_cytoscape() {
        let pg = sample_graph();
        let html = pg.to_html();
        assert!(html.contains("cytoscape"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_to_html_embeds_graph_data() {
        let pg = sample_graph();
        let html = pg.to_html();
        assert!(html.contains("\"graph_type\":\"callgraph\""));
        assert!(html.contains("\"schema_version\":\"0.1.0\""));
        assert!(html.contains("0x0001"));
        assert!(html.contains("0x0002"));
    }

    #[test]
    fn test_to_html_does_not_contain_placeholder() {
        let pg = sample_graph();
        let html = pg.to_html();
        assert!(!html.contains("/*GRAPH_DATA*/null"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let pg = sample_graph();
        let json = serde_json::to_string(&pg).expect("serialize");
        let pg2: PropertyGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(pg2.graph_type, "callgraph");
        assert_eq!(pg2.nodes.len(), 2);
        assert_eq!(pg2.edges.len(), 1);
        assert_eq!(pg2.edges[0].edge_type, "CALLS");
    }

    #[test]
    fn test_serde_skips_empty_metadata() {
        let pg = PropertyGraph::new("test");
        let json = serde_json::to_string(&pg).expect("serialize");
        assert!(!json.contains("metadata"));
    }

    #[test]
    fn test_serde_skips_empty_properties() {
        let mut pg = PropertyGraph::new("test");
        pg.nodes.push(PgNode {
            id: "n1".to_string(),
            labels: vec!["Value".to_string()],
            properties: BTreeMap::new(),
        });
        let json = serde_json::to_string(&pg).expect("serialize");
        // The node should not have a "properties" key since it's empty
        assert!(!json.contains("\"properties\""));
    }

    #[test]
    fn test_format_type_name_primitives() {
        assert_eq!(format_type_name(&AirType::Pointer), "ptr");
        assert_eq!(format_type_name(&AirType::Integer { bits: 32 }), "i32");
        assert_eq!(format_type_name(&AirType::Integer { bits: 1 }), "i1");
        assert_eq!(format_type_name(&AirType::Integer { bits: 64 }), "i64");
        assert_eq!(format_type_name(&AirType::Float { bits: 32 }), "float");
        assert_eq!(format_type_name(&AirType::Float { bits: 64 }), "double");
        assert_eq!(format_type_name(&AirType::Float { bits: 16 }), "f16");
        assert_eq!(format_type_name(&AirType::Void), "void");
        assert_eq!(format_type_name(&AirType::Opaque), "opaque");
    }

    #[test]
    fn test_format_type_name_composites() {
        use saf_core::air::StructField;

        let arr = AirType::Array {
            element: TypeId::derive(b"i32"),
            count: Some(10),
        };
        assert_eq!(format_type_name(&arr), "[10 x ...]");

        let arr_vla = AirType::Array {
            element: TypeId::derive(b"i32"),
            count: None,
        };
        assert_eq!(format_type_name(&arr_vla), "[? x ...]");

        let st = AirType::Struct {
            fields: vec![
                StructField {
                    field_type: TypeId::derive(b"ptr"),
                    byte_offset: Some(0),
                    byte_size: Some(8),
                    name: None,
                },
                StructField {
                    field_type: TypeId::derive(b"i32"),
                    byte_offset: Some(8),
                    byte_size: Some(4),
                    name: None,
                },
            ],
            total_size: 16,
        };
        assert_eq!(format_type_name(&st), "struct(2 fields, 16B)");

        let func = AirType::Function {
            params: vec![TypeId::derive(b"ptr"), TypeId::derive(b"i32")],
            return_type: TypeId::derive(b"void"),
        };
        assert_eq!(format_type_name(&func), "fn(2 params)");
    }

    #[test]
    fn test_build_value_type_lookup() {
        use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction, Operation};
        use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

        let ptr_type = TypeId::derive(b"pointer");
        let i32_type = TypeId::derive(b"integer:32");

        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.types.insert(ptr_type, AirType::Pointer);
        module.types.insert(i32_type, AirType::Integer { bits: 32 });

        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut p0 = AirParam::new(ValueId::derive(b"p0"), 0);
        p0.param_type = Some(ptr_type);
        func.params = vec![p0.clone()];

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        let mut load = Instruction::new(InstId::derive(b"load"), Operation::Load);
        let load_dst = ValueId::derive(b"v1");
        load.dst = Some(load_dst);
        load.result_type = Some(i32_type);
        block.instructions.push(load);
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let lookup = build_value_type_lookup(&module);
        assert_eq!(lookup.get(&p0.id.raw()), Some(&ptr_type));
        assert_eq!(lookup.get(&load_dst.raw()), Some(&i32_type));
    }

    #[test]
    fn test_insert_type_property() {
        use saf_core::ids::ModuleId;

        let ptr_type = TypeId::derive(b"pointer");
        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.types.insert(ptr_type, AirType::Pointer);

        let mut value_types = BTreeMap::new();
        let raw_id: u128 = 42;
        value_types.insert(raw_id, ptr_type);

        let mut properties = BTreeMap::new();
        insert_type_property(&mut properties, raw_id, &value_types, &module);

        assert_eq!(
            properties.get("type"),
            Some(&serde_json::Value::String("ptr".to_string()))
        );

        // No-op when raw ID is not in the lookup
        let mut properties2 = BTreeMap::new();
        insert_type_property(&mut properties2, 999, &value_types, &module);
        assert!(properties2.get("type").is_none());
    }

    #[test]
    fn test_parse_hex_id() {
        assert_eq!(parse_hex_id("0x0001"), Some(1));
        assert_eq!(
            parse_hex_id("0x00000000000000000000000000001234"),
            Some(0x1234)
        );
        assert_eq!(parse_hex_id("0xff"), Some(255));
        assert_eq!(parse_hex_id("not_hex"), None);
        assert_eq!(parse_hex_id(""), None);
        assert_eq!(parse_hex_id("0x"), None);
    }

    #[test]
    fn test_enrich_node_with_resolver() {
        use saf_core::air::{AirBlock, AirFunction, AirModule, Instruction, Operation};
        use saf_core::ids::{BlockId, FileId, FunctionId, InstId, ModuleId};
        use saf_core::span::{SourceFile, Span};

        // Build a minimal module with a function that has a span
        let mut module = AirModule::new(ModuleId::derive(b"test-module"));

        let fid = FunctionId::derive(b"test-func-main");
        let bid = BlockId::derive(b"entry-block");
        let iid = InstId::derive(b"ret-inst");

        let sfid = FileId::derive(b"test.c");
        module.source_files.push(SourceFile {
            id: sfid,
            path: "test.c".to_string(),
            checksum: None,
        });

        let mut func = AirFunction::new(fid, "main");
        func.span = Some(Span {
            file_id: sfid,
            byte_start: 0,
            byte_end: 50,
            line_start: 5,
            col_start: 1,
            line_end: 10,
            col_end: 1,
        });

        let mut block = AirBlock::new(bid);
        block
            .instructions
            .push(Instruction::new(iid, Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);
        module.rebuild_function_index();

        let resolver = crate::display::DisplayResolver::from_module(&module);

        // Test enrichment: function node should get display_name + source info
        let mut node = PgNode {
            id: fid.to_hex(),
            labels: vec!["Function".to_string()],
            properties: BTreeMap::new(),
        };
        enrich_node(&mut node, Some(&resolver));

        assert!(
            node.properties.contains_key("display_name"),
            "enriched node should have display_name property"
        );
        let display_name = node.properties["display_name"].as_str().unwrap();
        assert_eq!(display_name, "main");

        assert!(
            node.properties.contains_key("source_file"),
            "enriched node should have source_file property"
        );
        assert_eq!(node.properties["source_file"].as_str().unwrap(), "test.c");

        assert!(
            node.properties.contains_key("source_line"),
            "enriched node should have source_line property"
        );
        assert_eq!(node.properties["source_line"].as_u64().unwrap(), 5);

        assert!(
            node.properties.contains_key("source_col"),
            "enriched node should have source_col property"
        );
        assert_eq!(node.properties["source_col"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_enrich_node_without_resolver_is_noop() {
        let mut node = PgNode {
            id: "0x0001".to_string(),
            labels: vec!["Function".to_string()],
            properties: BTreeMap::new(),
        };
        enrich_node(&mut node, None);

        assert!(
            !node.properties.contains_key("display_name"),
            "node should NOT have display_name when resolver is None"
        );
        assert!(
            !node.properties.contains_key("source_file"),
            "node should NOT have source_file when resolver is None"
        );
        assert!(
            !node.properties.contains_key("source_line"),
            "node should NOT have source_line when resolver is None"
        );
        assert!(
            !node.properties.contains_key("source_col"),
            "node should NOT have source_col when resolver is None"
        );
    }

    #[test]
    fn test_enrich_node_non_hex_id_is_noop() {
        use saf_core::air::AirModule;
        use saf_core::ids::ModuleId;

        let module = AirModule::new(ModuleId::derive(b"test-module"));
        let resolver = crate::display::DisplayResolver::from_module(&module);

        // Node with a non-hex ID (e.g., CHA class name)
        let mut node = PgNode {
            id: "MyClass".to_string(),
            labels: vec!["Type".to_string()],
            properties: BTreeMap::new(),
        };
        enrich_node(&mut node, Some(&resolver));

        // Should not crash, and should not add properties
        assert!(
            !node.properties.contains_key("display_name"),
            "non-hex ID node should NOT gain display_name"
        );
    }

    #[test]
    fn test_enrich_node_unknown_id_gets_fallback_name() {
        use saf_core::air::AirModule;
        use saf_core::ids::ModuleId;

        let module = AirModule::new(ModuleId::derive(b"test-module"));
        let resolver = crate::display::DisplayResolver::from_module(&module);

        // Node with a valid hex ID that doesn't match any entity
        let mut node = PgNode {
            id: "0x00000000000000000000000000001234".to_string(),
            labels: vec!["Value".to_string()],
            properties: BTreeMap::new(),
        };
        enrich_node(&mut node, Some(&resolver));

        // Should still get a display_name (the fallback short hex)
        assert!(
            node.properties.contains_key("display_name"),
            "unknown-ID node should get a fallback display_name"
        );
        // No source location for unknown IDs
        assert!(
            !node.properties.contains_key("source_file"),
            "unknown-ID node should NOT have source_file"
        );
    }
}
