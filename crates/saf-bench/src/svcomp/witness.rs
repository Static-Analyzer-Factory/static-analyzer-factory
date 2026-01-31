//! SV-COMP witness generation.
//!
//! Generates GraphML witness files according to the SV-COMP 2.0 witness format.
//! These witnesses can be validated by external witness validators and are required
//! for scoring FALSE verdicts.
//!
//! # Witness Types
//!
//! - **Violation witness**: Demonstrates a path to property violation (for FALSE verdicts)
//! - **Correctness witness**: Provides invariants proving property holds (for TRUE verdicts)
//!
//! # Format
//!
//! Witnesses use GraphML format with SV-COMP-specific key definitions for:
//! - Graph-level metadata (producer, specification, program file, etc.)
//! - Node attributes (entry, sink, violation)
//! - Edge attributes (source code, line numbers, assumptions)
//!
//! See: <https://sv-comp.sosy-lab.org/2024/witnesses.php>

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use std::time::SystemTime;

use saf_core::air::AirModule;
use saf_core::ids::BlockId;
use sha2::{Digest, Sha256};

/// Type of SV-COMP witness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessType {
    /// Violation witness for FALSE verdicts.
    /// Demonstrates a path leading to property violation.
    Violation,

    /// Correctness witness for TRUE verdicts.
    /// Provides invariants proving the property holds.
    Correctness,
}

impl WitnessType {
    /// Get the witness type string for GraphML output.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Violation => "violation_witness",
            Self::Correctness => "correctness_witness",
        }
    }
}

/// A node in the witness graph.
#[derive(Clone, Debug)]
pub struct WitnessNode {
    /// Unique node identifier.
    pub id: String,

    /// True if this is the entry node.
    pub entry: bool,

    /// True if this is a sink (terminal) node.
    pub sink: bool,

    /// True if this node represents a violation.
    pub violation: bool,

    /// Optional invariant at this node (for correctness witnesses).
    pub invariant: Option<String>,
}

impl WitnessNode {
    /// Create a new node with the given ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entry: false,
            sink: false,
            violation: false,
            invariant: None,
        }
    }

    /// Mark this node as the entry node.
    #[must_use]
    pub fn with_entry(mut self) -> Self {
        self.entry = true;
        self
    }

    /// Mark this node as a sink node.
    #[must_use]
    pub fn with_sink(mut self) -> Self {
        self.sink = true;
        self
    }

    /// Mark this node as a violation node.
    #[must_use]
    pub fn with_violation(mut self) -> Self {
        self.violation = true;
        self
    }

    /// Add an invariant to this node.
    #[must_use]
    pub fn with_invariant(mut self, invariant: impl Into<String>) -> Self {
        self.invariant = Some(invariant.into());
        self
    }
}

/// An edge in the witness graph.
#[derive(Clone, Debug)]
pub struct WitnessEdge {
    /// Source node ID.
    pub source: String,

    /// Target node ID.
    pub target: String,

    /// Source code on this edge (optional).
    pub source_code: Option<String>,

    /// Starting line number (optional).
    pub start_line: Option<u32>,

    /// Ending line number (optional).
    pub end_line: Option<u32>,

    /// Assumption made on this edge (optional).
    pub assumption: Option<String>,

    /// Function entry (optional).
    pub enter_function: Option<String>,

    /// Function exit (optional).
    pub return_from_function: Option<String>,
}

impl WitnessEdge {
    /// Create a new edge between two nodes.
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            source_code: None,
            start_line: None,
            end_line: None,
            assumption: None,
            enter_function: None,
            return_from_function: None,
        }
    }

    /// Add source code to this edge.
    #[must_use]
    pub fn with_source_code(mut self, code: impl Into<String>) -> Self {
        self.source_code = Some(code.into());
        self
    }

    /// Add line number to this edge.
    #[must_use]
    pub fn with_line(mut self, line: u32) -> Self {
        self.start_line = Some(line);
        self
    }

    /// Add line range to this edge.
    #[must_use]
    pub fn with_line_range(mut self, start: u32, end: u32) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Add an assumption to this edge.
    #[must_use]
    pub fn with_assumption(mut self, assumption: impl Into<String>) -> Self {
        self.assumption = Some(assumption.into());
        self
    }

    /// Mark function entry on this edge.
    #[must_use]
    pub fn with_enter_function(mut self, func: impl Into<String>) -> Self {
        self.enter_function = Some(func.into());
        self
    }

    /// Mark function return on this edge.
    #[must_use]
    pub fn with_return_from_function(mut self, func: impl Into<String>) -> Self {
        self.return_from_function = Some(func.into());
        self
    }
}

/// An SV-COMP witness in GraphML format.
#[derive(Clone, Debug)]
pub struct Witness {
    /// Type of witness (violation or correctness).
    pub witness_type: WitnessType,

    /// Producer tool name and version.
    pub producer: String,

    /// Property specification (e.g., "CHECK( init(main()), LTL(G ! call(reach_error())))").
    pub specification: String,

    /// Path to the source program file.
    pub program_file: String,

    /// SHA-256 hash of the program file.
    pub program_hash: String,

    /// Target architecture (e.g., "32bit" or "64bit").
    pub architecture: String,

    /// Witness creation time in ISO 8601 format.
    pub creation_time: String,

    /// Nodes in the witness graph.
    pub nodes: Vec<WitnessNode>,

    /// Edges in the witness graph.
    pub edges: Vec<WitnessEdge>,
}

impl Witness {
    /// Create a new empty witness.
    #[must_use]
    pub fn new(witness_type: WitnessType) -> Self {
        let creation_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or_else(
                |_| "1970-01-01T00:00:00Z".to_string(),
                |d| {
                    // Format as ISO 8601: 2024-01-15T10:30:00Z
                    let secs = d.as_secs();
                    let days = secs / 86400;
                    let time = secs % 86400;
                    // Simplified date calculation (approximate)
                    let years = 1970 + days / 365;
                    let remaining_days = days % 365;
                    let months = remaining_days / 30 + 1;
                    let day = remaining_days % 30 + 1;
                    format!(
                        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                        years,
                        months,
                        day,
                        time / 3600,
                        (time % 3600) / 60,
                        time % 60
                    )
                },
            );

        Self {
            witness_type,
            producer: "SAF (Static Analyzer Factory)".to_string(),
            specification: String::new(),
            program_file: String::new(),
            program_hash: String::new(),
            architecture: "64bit".to_string(),
            creation_time,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Create a violation witness from a path trace.
    ///
    /// The trace is a sequence of block IDs representing the path from
    /// program entry to the violation.
    #[must_use]
    pub fn from_violation_trace(
        trace: &[BlockId],
        module: &AirModule,
        specification: &str,
        program_path: &Path,
    ) -> Self {
        let mut witness = Self::new(WitnessType::Violation);
        witness.specification = specification.to_string();
        witness.program_file = program_path.display().to_string();
        witness.program_hash = compute_file_hash(program_path);

        if trace.is_empty() {
            return witness;
        }

        // Build block-to-function and block-to-line maps
        let block_info = build_block_info(module);

        // Create nodes for each block in the trace
        for (i, _block_id) in trace.iter().enumerate() {
            let node_id = format!("N{i}");
            let mut node = WitnessNode::new(&node_id);

            if i == 0 {
                node = node.with_entry();
            }
            if i == trace.len() - 1 {
                node = node.with_violation().with_sink();
            }

            witness.nodes.push(node);
        }

        // Create edges between consecutive blocks
        for i in 0..trace.len().saturating_sub(1) {
            let block_id = trace[i];
            let source = format!("N{i}");
            let target = format!("N{}", i + 1);

            let mut edge = WitnessEdge::new(source, target);

            // Add debug info if available
            if let Some(info) = block_info.get(&block_id) {
                if let Some(line) = info.first_line {
                    edge = edge.with_line(line);
                }
                if let Some(func) = &info.function_name {
                    // Mark function entry on first block of a function
                    if i == 0
                        || block_info
                            .get(&trace[i - 1])
                            .and_then(|prev| prev.function_name.as_ref())
                            != Some(func)
                    {
                        edge = edge.with_enter_function(func.clone());
                    }
                }
            }

            witness.edges.push(edge);
        }

        witness
    }

    /// Create a correctness witness from loop invariants.
    ///
    /// Invariants map block IDs (typically loop headers) to invariant strings.
    #[must_use]
    pub fn from_invariants(
        invariants: &BTreeMap<BlockId, String>,
        _module: &AirModule,
        specification: &str,
        program_path: &Path,
    ) -> Self {
        let mut witness = Self::new(WitnessType::Correctness);
        witness.specification = specification.to_string();
        witness.program_file = program_path.display().to_string();
        witness.program_hash = compute_file_hash(program_path);

        // Create entry node
        witness.nodes.push(WitnessNode::new("N0").with_entry());

        // Create nodes for each invariant location
        let mut node_idx = 1;
        for (block_id, invariant) in invariants {
            let node_id = format!("N{node_idx}");
            let node = WitnessNode::new(&node_id).with_invariant(format!(
                "block_{}: {}",
                block_id.raw(),
                invariant
            ));
            witness.nodes.push(node);

            // Connect from entry to invariant node
            witness.edges.push(WitnessEdge::new("N0", &node_id));

            node_idx += 1;
        }

        // Add sink node
        let sink_id = format!("N{node_idx}");
        witness.nodes.push(WitnessNode::new(&sink_id).with_sink());

        // Connect all invariant nodes to sink
        for i in 1..node_idx {
            witness
                .edges
                .push(WitnessEdge::new(format!("N{i}"), &sink_id));
        }

        witness
    }

    /// Create a violation witness from a simple trace (list of block/location strings).
    ///
    /// This is used to convert the simple `Vec<String>` witness format used in
    /// `PropertyResult` to a full GraphML witness.
    #[must_use]
    pub fn from_simple_trace(trace: &[String], specification: &str, program_path: &Path) -> Self {
        let mut witness = Self::new(WitnessType::Violation);
        witness.specification = specification.to_string();
        witness.program_file = program_path.display().to_string();
        witness.program_hash = compute_file_hash(program_path);

        if trace.is_empty() {
            return witness;
        }

        // Create nodes for each location in the trace
        for (i, location) in trace.iter().enumerate() {
            let node_id = format!("N{i}");
            let mut node = WitnessNode::new(&node_id);

            if i == 0 {
                node = node.with_entry();
            }
            if i == trace.len() - 1 {
                node = node.with_violation().with_sink();
            }

            witness.nodes.push(node);

            // Create edge from previous node
            if i > 0 {
                let source = format!("N{}", i - 1);
                let mut edge = WitnessEdge::new(source, &node_id);

                // Try to extract line number from location string if present
                // Format could be "block_0x123" or "line:42" or "func:main:line:10"
                if let Some(line) = extract_line_number(location) {
                    edge = edge.with_line(line);
                }

                witness.edges.push(edge);
            }
        }

        witness
    }

    /// Export the witness to GraphML format.
    #[must_use]
    pub fn to_graphml(&self) -> String {
        let mut xml = String::new();

        // XML header and key declarations
        write_graphml_header(&mut xml);
        write_graphml_keys(&mut xml);

        // Graph element with metadata
        xml.push_str(r#"  <graph edgedefault="directed">"#);
        xml.push('\n');
        self.write_graph_metadata(&mut xml);

        // Write nodes and edges
        for node in &self.nodes {
            write_node(&mut xml, node);
        }
        for (i, edge) in self.edges.iter().enumerate() {
            write_edge(&mut xml, i, edge);
        }

        xml.push_str("  </graph>\n");
        xml.push_str("</graphml>\n");
        xml
    }

    /// Write graph-level metadata to the XML output.
    fn write_graph_metadata(&self, xml: &mut String) {
        xml.push_str(&format!(
            "    <data key=\"witness-type\">{}</data>\n",
            self.witness_type.as_str()
        ));
        xml.push_str(&format!(
            "    <data key=\"producer\">{}</data>\n",
            escape_xml(&self.producer)
        ));
        xml.push_str(&format!(
            "    <data key=\"specification\">{}</data>\n",
            escape_xml(&self.specification)
        ));
        xml.push_str(&format!(
            "    <data key=\"programfile\">{}</data>\n",
            escape_xml(&self.program_file)
        ));
        xml.push_str(&format!(
            "    <data key=\"programhash\">{}</data>\n",
            escape_xml(&self.program_hash)
        ));
        xml.push_str(&format!(
            "    <data key=\"architecture\">{}</data>\n",
            escape_xml(&self.architecture)
        ));
        xml.push_str(&format!(
            "    <data key=\"creationtime\">{}</data>\n",
            escape_xml(&self.creation_time)
        ));
    }

    /// Write the witness to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_to_file(&self, path: &Path) -> io::Result<()> {
        let graphml = self.to_graphml();
        let mut file = std::fs::File::create(path)?;
        file.write_all(graphml.as_bytes())?;
        Ok(())
    }
}

/// Write the GraphML XML header.
fn write_graphml_header(xml: &mut String) {
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns""#);
    xml.push_str(r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#);
    xml.push('\n');
}

/// Write all GraphML key declarations.
fn write_graphml_keys(xml: &mut String) {
    // Graph-level keys
    for key in &[
        "witness-type",
        "producer",
        "specification",
        "programfile",
        "programhash",
        "architecture",
        "creationtime",
    ] {
        xml.push_str(&format!(
            "  <key id=\"{key}\" for=\"graph\" attr.type=\"string\"/>\n"
        ));
    }

    // Node-level keys with defaults
    for key in &["entry", "sink", "violation"] {
        xml.push_str(&format!(
            "  <key id=\"{key}\" for=\"node\" attr.type=\"boolean\"><default>false</default></key>\n"
        ));
    }
    xml.push_str("  <key id=\"invariant\" for=\"node\" attr.type=\"string\"/>\n");

    // Edge-level keys
    for key in &[
        "sourcecode",
        "assumption",
        "enterFunction",
        "returnFromFunction",
    ] {
        xml.push_str(&format!(
            "  <key id=\"{key}\" for=\"edge\" attr.type=\"string\"/>\n"
        ));
    }
    for key in &["startline", "endline"] {
        xml.push_str(&format!(
            "  <key id=\"{key}\" for=\"edge\" attr.type=\"int\"/>\n"
        ));
    }
}

/// Write a node element to the XML output.
fn write_node(xml: &mut String, node: &WitnessNode) {
    xml.push_str(&format!("    <node id=\"{}\">\n", escape_xml(&node.id)));

    if node.entry {
        xml.push_str("      <data key=\"entry\">true</data>\n");
    }
    if node.sink {
        xml.push_str("      <data key=\"sink\">true</data>\n");
    }
    if node.violation {
        xml.push_str("      <data key=\"violation\">true</data>\n");
    }
    if let Some(inv) = &node.invariant {
        xml.push_str(&format!(
            "      <data key=\"invariant\">{}</data>\n",
            escape_xml(inv)
        ));
    }

    xml.push_str("    </node>\n");
}

/// Write an edge element to the XML output.
fn write_edge(xml: &mut String, index: usize, edge: &WitnessEdge) {
    xml.push_str(&format!(
        "    <edge id=\"E{}\" source=\"{}\" target=\"{}\">\n",
        index,
        escape_xml(&edge.source),
        escape_xml(&edge.target)
    ));

    if let Some(code) = &edge.source_code {
        xml.push_str(&format!(
            "      <data key=\"sourcecode\">{}</data>\n",
            escape_xml(code)
        ));
    }
    if let Some(line) = edge.start_line {
        xml.push_str(&format!("      <data key=\"startline\">{line}</data>\n"));
    }
    if let Some(line) = edge.end_line {
        xml.push_str(&format!("      <data key=\"endline\">{line}</data>\n"));
    }
    if let Some(assumption) = &edge.assumption {
        xml.push_str(&format!(
            "      <data key=\"assumption\">{}</data>\n",
            escape_xml(assumption)
        ));
    }
    if let Some(func) = &edge.enter_function {
        xml.push_str(&format!(
            "      <data key=\"enterFunction\">{}</data>\n",
            escape_xml(func)
        ));
    }
    if let Some(func) = &edge.return_from_function {
        xml.push_str(&format!(
            "      <data key=\"returnFromFunction\">{}</data>\n",
            escape_xml(func)
        ));
    }

    xml.push_str("    </edge>\n");
}

/// Information about a basic block for witness generation.
struct BlockInfo {
    /// First source line in the block (if available).
    first_line: Option<u32>,
    /// Function containing this block.
    function_name: Option<String>,
}

/// Build a map from block IDs to their debug information.
fn build_block_info(module: &AirModule) -> BTreeMap<BlockId, BlockInfo> {
    let mut info = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            let mut first_line = None;

            // Find the first instruction with span info
            for inst in &block.instructions {
                if let Some(span) = &inst.span {
                    first_line = Some(span.line_start);
                    break;
                }
            }

            info.insert(
                block.id,
                BlockInfo {
                    first_line,
                    function_name: Some(func.name.clone()),
                },
            );
        }
    }

    info
}

/// Compute SHA-256 hash of a file.
fn compute_file_hash(path: &Path) -> String {
    match std::fs::read(path) {
        Ok(contents) => {
            let mut hasher = Sha256::new();
            hasher.update(&contents);
            let result = hasher.finalize();
            format!("{result:064x}")
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Try to extract a line number from a location string.
///
/// Supports formats like "line:42", "func:main:line:10", etc.
fn extract_line_number(location: &str) -> Option<u32> {
    // Try "line:N" format
    if let Some(pos) = location.find("line:") {
        let rest = &location[pos + 5..];
        let num_str: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(line) = num_str.parse::<u32>() {
            return Some(line);
        }
    }
    None
}

/// Escape special XML characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_type_as_str() {
        assert_eq!(WitnessType::Violation.as_str(), "violation_witness");
        assert_eq!(WitnessType::Correctness.as_str(), "correctness_witness");
    }

    #[test]
    fn test_witness_node_builder() {
        let node = WitnessNode::new("N0")
            .with_entry()
            .with_violation()
            .with_invariant("x >= 0");

        assert_eq!(node.id, "N0");
        assert!(node.entry);
        assert!(node.violation);
        assert!(!node.sink);
        assert_eq!(node.invariant, Some("x >= 0".to_string()));
    }

    #[test]
    fn test_witness_edge_builder() {
        let edge = WitnessEdge::new("N0", "N1")
            .with_source_code("x = 1;")
            .with_line(42)
            .with_enter_function("main");

        assert_eq!(edge.source, "N0");
        assert_eq!(edge.target, "N1");
        assert_eq!(edge.source_code, Some("x = 1;".to_string()));
        assert_eq!(edge.start_line, Some(42));
        assert_eq!(edge.enter_function, Some("main".to_string()));
    }

    #[test]
    fn test_empty_violation_witness() {
        let witness = Witness::new(WitnessType::Violation);
        let graphml = witness.to_graphml();

        assert!(graphml.contains("<?xml version=\"1.0\""));
        assert!(graphml.contains("<graphml"));
        assert!(graphml.contains("violation_witness"));
        assert!(graphml.contains("SAF (Static Analyzer Factory)"));
        assert!(graphml.contains("</graphml>"));
    }

    #[test]
    fn test_graphml_xml_escaping() {
        let mut witness = Witness::new(WitnessType::Violation);
        witness.specification = "CHECK(x < 5 && y > 0)".to_string();

        let graphml = witness.to_graphml();
        assert!(graphml.contains("&lt;"));
        assert!(graphml.contains("&gt;"));
        assert!(graphml.contains("&amp;"));
    }

    #[test]
    fn test_witness_with_nodes_and_edges() {
        let mut witness = Witness::new(WitnessType::Violation);
        witness.nodes.push(WitnessNode::new("N0").with_entry());
        witness
            .nodes
            .push(WitnessNode::new("N1").with_violation().with_sink());
        witness.edges.push(
            WitnessEdge::new("N0", "N1")
                .with_line(10)
                .with_enter_function("main"),
        );

        let graphml = witness.to_graphml();

        assert!(graphml.contains(r#"<node id="N0">"#));
        assert!(graphml.contains(r#"<data key="entry">true</data>"#));
        assert!(graphml.contains(r#"<node id="N1">"#));
        assert!(graphml.contains(r#"<data key="violation">true</data>"#));
        assert!(graphml.contains(r#"<data key="sink">true</data>"#));
        assert!(graphml.contains(r#"source="N0" target="N1">"#));
        assert!(graphml.contains(r#"<data key="startline">10</data>"#));
        assert!(graphml.contains(r#"<data key="enterFunction">main</data>"#));
    }

    #[test]
    fn test_from_simple_trace() {
        let trace = vec![
            "block_0x100".to_string(),
            "block_0x200".to_string(),
            "block_0x300".to_string(),
        ];
        let witness = Witness::from_simple_trace(
            &trace,
            "CHECK( init(main()), LTL(G ! call(reach_error())))",
            Path::new("test.c"),
        );

        assert_eq!(witness.witness_type, WitnessType::Violation);
        assert_eq!(witness.nodes.len(), 3);
        assert_eq!(witness.edges.len(), 2);

        // First node is entry
        assert!(witness.nodes[0].entry);
        assert!(!witness.nodes[0].violation);

        // Last node is violation and sink
        assert!(witness.nodes[2].violation);
        assert!(witness.nodes[2].sink);

        let graphml = witness.to_graphml();
        assert!(graphml.contains("violation_witness"));
        assert!(graphml.contains("reach_error"));
    }

    #[test]
    fn test_extract_line_number() {
        assert_eq!(extract_line_number("line:42"), Some(42));
        assert_eq!(extract_line_number("func:main:line:10"), Some(10));
        assert_eq!(extract_line_number("block_0x100"), None);
        assert_eq!(extract_line_number("line:"), None);
    }

    #[test]
    fn test_correctness_witness_with_invariants() {
        let mut invariants = BTreeMap::new();
        invariants.insert(BlockId::new(100), "i >= 0".to_string());
        invariants.insert(BlockId::new(200), "i < n".to_string());

        // We can't easily create an AirModule, so test the basic structure
        let mut witness = Witness::new(WitnessType::Correctness);
        witness.nodes.push(WitnessNode::new("N0").with_entry());
        witness
            .nodes
            .push(WitnessNode::new("N1").with_invariant("i >= 0"));
        witness
            .nodes
            .push(WitnessNode::new("N2").with_invariant("i < n"));
        witness.nodes.push(WitnessNode::new("N3").with_sink());

        let graphml = witness.to_graphml();

        assert!(graphml.contains("correctness_witness"));
        assert!(graphml.contains(r#"<data key="invariant">i &gt;= 0</data>"#));
        assert!(graphml.contains(r#"<data key="invariant">i &lt; n</data>"#));
    }
}
