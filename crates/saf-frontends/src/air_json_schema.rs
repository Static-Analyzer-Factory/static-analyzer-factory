//! JSON schema types for AIR JSON frontend.
//!
//! These types are optimized for JSON parsing ergonomics. They may differ
//! slightly from the internal AIR types and are converted during ingestion.

use serde::{Deserialize, Serialize};

/// JSON representation of an AIR type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JsonAirType {
    /// Pointer type.
    Pointer,
    /// Non-null reference type.
    Reference {
        /// Whether this reference can be null.
        nullable: bool,
    },
    /// SIMD vector type.
    Vector {
        /// Element type ID (hex string).
        element: String,
        /// Number of lanes.
        lanes: u32,
    },
    /// Fixed-width integer type.
    Integer {
        /// Bit width.
        bits: u16,
    },
    /// Floating-point type.
    Float {
        /// Bit width.
        bits: u16,
    },
    /// Array type.
    Array {
        /// Element type ID (hex string).
        element: String,
        /// Element count (`None` for variable-length).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u64>,
    },
    /// Struct type with field layout.
    Struct {
        /// Fields in declaration order.
        fields: Vec<JsonStructField>,
        /// Total size in bytes (including tail padding).
        total_size: u64,
    },
    /// Function signature type.
    Function {
        /// Parameter type IDs (hex strings).
        params: Vec<String>,
        /// Return type ID (hex string).
        return_type: String,
    },
    /// Void (no value / zero-sized).
    Void,
    /// Opaque/unknown type.
    Opaque,
}

/// JSON representation of a struct field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStructField {
    /// Field type ID (hex string).
    pub field_type: String,
    /// Byte offset from struct start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    /// Size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
    /// Optional field name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A type entry in the JSON type table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTypeEntry {
    /// Type ID (hex string).
    pub id: String,
    /// The type definition.
    #[serde(flatten)]
    pub ty: JsonAirType,
}

/// Root of an AIR JSON file.
#[derive(Debug, Deserialize)]
pub struct JsonAirBundle {
    /// Frontend identifier (should be "air-json").
    pub frontend_id: String,

    /// Schema version for compatibility checking.
    pub schema_version: String,

    /// The module definition.
    pub module: JsonModule,
}

/// A module in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonModule {
    /// Optional module ID (hex string). If not provided, will be derived.
    #[serde(default)]
    pub id: Option<String>,

    /// Optional module name.
    #[serde(default)]
    pub name: Option<String>,

    /// Functions in the module.
    pub functions: Vec<JsonFunction>,

    /// Global variables/constants.
    #[serde(default)]
    pub globals: Vec<JsonGlobal>,

    /// Source files referenced by spans.
    #[serde(default)]
    pub source_files: Vec<JsonSourceFile>,

    /// Type table entries.
    #[serde(default)]
    pub types: Vec<JsonTypeEntry>,
}

/// A function in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonFunction {
    /// Optional function ID (hex string). If not provided, will be derived from name.
    #[serde(default)]
    pub id: Option<String>,

    /// Function name (required).
    pub name: String,

    /// Function parameters.
    #[serde(default)]
    pub params: Vec<JsonParam>,

    /// Basic blocks.
    pub blocks: Vec<JsonBlock>,

    /// Entry block ID. If not specified, first block is entry.
    #[serde(default)]
    pub entry_block: Option<String>,

    /// Whether this is a declaration (no body).
    #[serde(default)]
    pub is_declaration: bool,

    /// Optional source span.
    #[serde(default)]
    pub span: Option<JsonSpan>,

    /// Optional symbol information.
    #[serde(default)]
    pub symbol: Option<JsonSymbol>,
}

/// A function parameter in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonParam {
    /// Optional value ID (hex string). If not provided, will be derived.
    #[serde(default)]
    pub id: Option<String>,

    /// Optional parameter name.
    #[serde(default)]
    pub name: Option<String>,

    /// Parameter index (0-based). If not provided, will use position in array.
    #[serde(default)]
    pub index: Option<u32>,

    /// Type ID of this parameter (hex string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_type: Option<String>,
}

/// A basic block in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonBlock {
    /// Optional block ID (hex string). If not provided, will be derived.
    #[serde(default)]
    pub id: Option<String>,

    /// Optional block label.
    #[serde(default)]
    pub label: Option<String>,

    /// Instructions in this block.
    pub instructions: Vec<JsonInstruction>,
}

/// An instruction in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonInstruction {
    /// Optional instruction ID (hex string). If not provided, will be derived.
    #[serde(default)]
    pub id: Option<String>,

    /// Operation type (e.g., "alloca", "load", "store", "ret").
    pub op: String,

    /// Input operands (value IDs as hex strings).
    #[serde(default)]
    pub operands: Vec<String>,

    /// Output value ID (hex string).
    #[serde(default)]
    pub dst: Option<String>,

    /// Optional source span.
    #[serde(default)]
    pub span: Option<JsonSpan>,

    /// Optional symbol information.
    #[serde(default)]
    pub symbol: Option<JsonSymbol>,

    // Operation-specific fields
    /// Target block for Br.
    #[serde(default)]
    pub target: Option<String>,

    /// Then target for `CondBr`.
    #[serde(default)]
    pub then_target: Option<String>,

    /// Else target for `CondBr`.
    #[serde(default)]
    pub else_target: Option<String>,

    /// Default target for Switch.
    #[serde(default)]
    pub default: Option<String>,

    /// Cases for Switch: `[[value, block_id], ...]`.
    #[serde(default)]
    pub cases: Vec<(i64, String)>,

    /// Incoming values for Phi: `[[block_id, value_id], ...]`.
    #[serde(default)]
    pub incoming: Vec<(String, String)>,

    /// Callee function ID for `CallDirect`.
    #[serde(default)]
    pub callee: Option<String>,

    /// Global object ID for Global operation.
    #[serde(default)]
    pub obj: Option<String>,

    /// Cast kind for Cast operation.
    #[serde(default)]
    pub cast_kind: Option<String>,

    /// Binary operation kind for `BinaryOp`.
    #[serde(default)]
    pub binary_op: Option<String>,

    /// Heap allocation kind for `HeapAlloc`.
    #[serde(default)]
    pub heap_kind: Option<String>,

    /// Field path for GEP.
    #[serde(default)]
    pub field_path: Option<JsonFieldPath>,

    /// Allocation size in bytes for `Alloca`.
    #[serde(default)]
    pub size_bytes: Option<u64>,

    /// Type ID of the result value (hex string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_type: Option<String>,
}

/// Field path for GEP operations.
#[derive(Debug, Deserialize)]
pub struct JsonFieldPath {
    /// Steps in the field path.
    pub steps: Vec<JsonFieldStep>,
}

/// A single step in a field path.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JsonFieldStep {
    /// Array/pointer index.
    Index,
    /// Struct field access.
    Field { index: u32 },
}

/// A global variable/constant in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonGlobal {
    /// Value ID for the global's address.
    #[serde(default)]
    pub id: Option<String>,

    /// Object ID for the allocation.
    #[serde(default)]
    pub obj: Option<String>,

    /// Global name.
    pub name: String,

    /// Initial value.
    #[serde(default)]
    pub init: Option<JsonConstant>,

    /// Whether this is a constant.
    #[serde(default)]
    pub is_constant: bool,

    /// Optional source span.
    #[serde(default)]
    pub span: Option<JsonSpan>,

    /// Type ID of the global's value (hex string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
}

/// A constant value in JSON format.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JsonConstant {
    /// Integer constant.
    Int { value: i64, bits: u8 },
    /// Big integer constant (for values > i64).
    BigInt { value: String, bits: u8 },
    /// Floating point constant.
    Float { value: f64, bits: u8 },
    /// String constant.
    String { value: String },
    /// Null pointer.
    Null,
    /// Undefined value.
    Undef,
    /// Zero initializer.
    ZeroInit,
    /// Aggregate (struct/array).
    Aggregate { elements: Vec<JsonConstant> },
    /// Global reference (function pointer or global variable).
    GlobalRef {
        #[serde(rename = "0")]
        value: String,
    },
}

/// Source span in JSON format.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonSpan {
    /// File ID (u128 or hex string).
    pub file_id: u128,
    /// Start byte offset.
    pub byte_start: u32,
    /// End byte offset.
    pub byte_end: u32,
    /// Start line (1-based).
    pub line_start: u32,
    /// Start column (1-based).
    pub col_start: u32,
    /// End line (1-based).
    pub line_end: u32,
    /// End column (1-based).
    pub col_end: u32,
}

/// Symbol information in JSON format.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonSymbol {
    /// Display name.
    pub display_name: String,
    /// Mangled name (optional).
    #[serde(default)]
    pub mangled_name: Option<String>,
    /// Namespace path.
    #[serde(default)]
    pub namespace_path: Vec<String>,
}

/// Source file entry in JSON format.
#[derive(Debug, Deserialize)]
pub struct JsonSourceFile {
    /// File ID.
    pub id: u128,
    /// File path.
    pub path: String,
    /// Optional checksum.
    #[serde(default)]
    pub checksum: Option<String>,
}
