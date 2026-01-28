//! AIR (Analysis Intermediate Representation) — SAF's canonical, frontend-agnostic IR.
//!
//! All analysis passes operate on AIR, never on frontend-specific representations.
//! This ensures that adding a new frontend does not require changes to analysis
//! algorithms (NFR-EXT-001, NFR-EXT-002).
//!
//! See FR-AIR-001 through FR-AIR-007 for full requirements.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ids::{BlockId, FunctionId, InstId, ModuleId, ObjId, TypeId, ValueId};
use crate::span::{SourceFile, Span, Symbol};

// =============================================================================
// Constants
// =============================================================================

/// Constant values in AIR.
///
/// Represents compile-time constant values that can appear as instruction operands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constant {
    /// Integer constant (signed, up to 128 bits).
    Int {
        /// The integer value (stored as i64 for JSON compatibility; use String variant for larger).
        value: i64,
        /// Bit width (e.g., 8, 16, 32, 64, 128).
        bits: u8,
    },

    /// Large integer constant that doesn't fit in i64 (stored as string).
    BigInt {
        /// The integer value as a string.
        value: String,
        /// Bit width.
        bits: u8,
    },

    /// Floating point constant.
    ///
    /// All floats are stored as `f64` regardless of source precision.
    /// This is lossless for f32 values because every IEEE 754 binary32
    /// value is exactly representable as a binary64 value. The `bits`
    /// field records the original source precision (32 or 64) for
    /// informational purposes, but analyses should treat the `value`
    /// field as the canonical representation and should not round-trip
    /// it back to f32.
    Float {
        /// The floating point value (stored as f64; lossless for f32 sources).
        value: f64,
        /// Original bit width (32 for f32, 64 for f64).
        bits: u8,
    },

    /// String constant (UTF-8).
    String {
        /// The string value.
        value: String,
    },

    /// Null pointer constant.
    Null,

    /// Undefined/poison value.
    Undef,

    /// Zero initializer (for aggregates).
    ZeroInit,

    /// Aggregate constant (struct, array).
    Aggregate {
        /// The aggregate elements.
        elements: Vec<Constant>,
    },

    /// Global reference (pointer to a global variable).
    ///
    /// Used for global pointer initializers like `@p = global ptr @target`.
    /// The `ValueId` identifies the target global's address.
    GlobalRef(crate::ids::ValueId),
}

impl Constant {
    /// Create an integer constant.
    #[must_use]
    pub fn int(value: i64, bits: u8) -> Self {
        Self::Int { value, bits }
    }

    /// Create a big integer constant (for values that don't fit in i64).
    #[must_use]
    pub fn big_int(value: i128, bits: u8) -> Self {
        Self::BigInt {
            value: value.to_string(),
            bits,
        }
    }

    /// Create a 32-bit integer constant.
    #[must_use]
    pub const fn i32(value: i32) -> Self {
        Self::Int {
            value: value as i64,
            bits: 32,
        }
    }

    /// Create a 64-bit integer constant.
    #[must_use]
    pub const fn i64(value: i64) -> Self {
        Self::Int { value, bits: 64 }
    }

    /// Create a floating point constant.
    #[must_use]
    pub const fn float(value: f64, bits: u8) -> Self {
        Self::Float { value, bits }
    }

    /// Create a string constant.
    #[must_use]
    pub fn string(value: impl Into<String>) -> Self {
        Self::String {
            value: value.into(),
        }
    }
}

// =============================================================================
// Types
// =============================================================================

/// Analysis-oriented type — describes memory layout, not source semantics.
///
/// Every language that compiles to machine code must resolve to concrete
/// memory layouts. This enum captures that lowering. Frontends map source
/// types to `AirType`; analyses use it for precision without coupling to
/// any language.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AirType {
    /// Pointer to memory (language-agnostic, like LLVM opaque ptr).
    Pointer,

    /// Non-null reference type (for Java, Rust, Kotlin).
    /// Distinct from `Pointer` in that it may carry nullability information.
    Reference {
        /// Whether this reference can be null.
        nullable: bool,
    },

    /// SIMD vector type.
    Vector {
        /// Element type.
        element: TypeId,
        /// Number of lanes (elements).
        lanes: u32,
    },

    /// Fixed-width integer.
    Integer {
        /// Bit width (e.g., 1, 8, 16, 32, 64, 128).
        bits: u16,
    },

    /// Floating-point.
    Float {
        /// Bit width (32 for f32, 64 for f64).
        bits: u16,
    },

    /// Fixed-size array.
    Array {
        /// Element type.
        element: TypeId,
        /// Element count. `None` for variable-length arrays.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u64>,
    },

    /// Struct/record with known field layout.
    Struct {
        /// Fields in declaration order.
        fields: Vec<StructField>,
        /// Total size in bytes (including tail padding).
        total_size: u64,
    },

    /// Function signature.
    Function {
        /// Parameter types.
        params: Vec<TypeId>,
        /// Return type.
        return_type: TypeId,
    },

    /// Void (no value / zero-sized).
    Void,

    /// Type is unknown or cannot be expressed.
    /// Analyses fall back to conservative behavior for `Opaque` types.
    Opaque,
}

/// A single field in a struct layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructField {
    /// Field type.
    pub field_type: TypeId,
    /// Byte offset from struct start. `None` if layout unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    /// Size in bytes. `None` if layout unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
    /// Optional field name from debug info or source language metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// =============================================================================
// Field paths (for GEP)
// =============================================================================

/// A single step in a field path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldStep {
    /// Array/pointer index (dynamic).
    Index,
    /// Struct field index (compile-time constant).
    Field { index: u32 },
}

/// Path through nested structures for GEP operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FieldPath {
    /// Sequence of field steps.
    pub steps: Vec<FieldStep>,
}

impl FieldPath {
    /// Create an empty field path.
    #[must_use]
    pub const fn empty() -> Self {
        Self { steps: Vec::new() }
    }

    /// Create a field path with a single field index.
    #[must_use]
    pub fn field(index: u32) -> Self {
        Self {
            steps: vec![FieldStep::Field { index }],
        }
    }

    /// Create a field path with a single array index.
    #[must_use]
    pub fn index() -> Self {
        Self {
            steps: vec![FieldStep::Index],
        }
    }
}

// =============================================================================
// Cast and binary operation kinds
// =============================================================================

/// Kind of cast operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CastKind {
    /// Truncate to smaller integer type.
    Trunc,
    /// Zero-extend to larger integer type.
    ZExt,
    /// Sign-extend to larger integer type.
    SExt,
    /// Float to unsigned integer.
    FPToUI,
    /// Float to signed integer.
    FPToSI,
    /// Unsigned integer to float.
    UIToFP,
    /// Signed integer to float.
    SIToFP,
    /// Float truncation.
    FPTrunc,
    /// Float extension.
    FPExt,
    /// Pointer to integer.
    PtrToInt,
    /// Integer to pointer.
    IntToPtr,
    /// Bitcast (reinterpret bits).
    Bitcast,
    /// Address space cast (pointer conversion).
    AddrSpaceCast,
}

/// Kind of binary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    // Integer arithmetic
    /// Integer addition.
    Add,
    /// Integer subtraction.
    Sub,
    /// Integer multiplication.
    Mul,
    /// Unsigned integer division.
    UDiv,
    /// Signed integer division.
    SDiv,
    /// Unsigned integer remainder.
    URem,
    /// Signed integer remainder.
    SRem,

    // Floating point arithmetic
    /// Floating point addition.
    FAdd,
    /// Floating point subtraction.
    FSub,
    /// Floating point multiplication.
    FMul,
    /// Floating point division.
    FDiv,
    /// Floating point remainder.
    FRem,

    // Bitwise operations
    /// Bitwise AND.
    And,
    /// Bitwise OR.
    Or,
    /// Bitwise XOR.
    Xor,
    /// Shift left.
    Shl,
    /// Logical shift right.
    LShr,
    /// Arithmetic shift right.
    AShr,

    // Comparisons
    /// Integer equality.
    ICmpEq,
    /// Integer inequality.
    ICmpNe,
    /// Unsigned greater than.
    ICmpUgt,
    /// Unsigned greater or equal.
    ICmpUge,
    /// Unsigned less than.
    ICmpUlt,
    /// Unsigned less or equal.
    ICmpUle,
    /// Signed greater than.
    ICmpSgt,
    /// Signed greater or equal.
    ICmpSge,
    /// Signed less than.
    ICmpSlt,
    /// Signed less or equal.
    ICmpSle,

    // Float comparisons (ordered)
    /// Ordered equal.
    FCmpOeq,
    /// Ordered not equal.
    FCmpOne,
    /// Ordered greater than.
    FCmpOgt,
    /// Ordered greater or equal.
    FCmpOge,
    /// Ordered less than.
    FCmpOlt,
    /// Ordered less or equal.
    FCmpOle,
}

// =============================================================================
// Heap allocation kinds
// =============================================================================

/// Kind of heap allocation operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeapAllocKind {
    /// `malloc`-family allocation.
    Malloc,
    /// C++ `new` operator.
    New,
    /// `calloc` (zero-initialized).
    Calloc,
    /// `realloc` (resize).
    Realloc,
    /// Other/custom allocator (name preserved as string).
    Other(String),
}

impl HeapAllocKind {
    /// Get the kind as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Malloc => "malloc",
            Self::New => "new",
            Self::Calloc => "calloc",
            Self::Realloc => "realloc",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for HeapAllocKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for HeapAllocKind {
    fn from(s: &str) -> Self {
        match s {
            "malloc" => Self::Malloc,
            "new"
            | "operator_new"
            | "operator_new_array"
            | "operator_new_nothrow"
            | "operator_new_array_nothrow" => Self::New,
            "calloc" => Self::Calloc,
            "realloc" => Self::Realloc,
            other => Self::Other(other.to_string()),
        }
    }
}

// =============================================================================
// Operations (flat enum, pattern-match friendly)
// =============================================================================

/// AIR operation kind.
///
/// This is a flat enum for easy pattern matching. Additional metadata
/// is carried in optional fields where needed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    // =========================================================================
    // Allocation operations
    // =========================================================================
    /// Stack allocation. Result is a pointer to allocated memory.
    ///
    /// The `size_bytes` field contains the allocation size when known statically.
    /// For fixed-size allocas like `int x[10]`, this is the total size in bytes.
    /// For variable-length arrays (VLAs), this is `None`.
    Alloca {
        /// Size of allocation in bytes, if known statically.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size_bytes: Option<u64>,
    },

    /// Global variable/constant reference.
    Global {
        /// The global object being referenced.
        obj: ObjId,
    },

    /// Heap allocation (malloc, new, etc.).
    HeapAlloc {
        /// Kind of heap allocation (e.g., `Malloc`, `New`, `Calloc`).
        kind: HeapAllocKind,
    },

    // =========================================================================
    // Memory access operations
    // =========================================================================
    /// Load from memory. Operand[0] is the pointer.
    Load,

    /// Store to memory. Operand[0] is value, operand[1] is pointer.
    Store,

    /// Get element pointer. Computes address offset.
    /// Operand[0] is base pointer, remaining operands are indices.
    Gep {
        /// Field path for struct/array traversal.
        #[serde(default, skip_serializing_if = "FieldPath::is_default")]
        field_path: FieldPath,
    },

    /// Memory copy. Operand[0] is dest, operand[1] is src, operand[2] is size.
    Memcpy,

    /// Memory set. Operand[0] is dest, operand[1] is value, operand[2] is size.
    Memset,

    // =========================================================================
    // Control flow operations
    // =========================================================================
    /// Unconditional branch.
    Br {
        /// Target block.
        target: BlockId,
    },

    /// Conditional branch. Operand[0] is the condition.
    CondBr {
        /// Target if condition is true.
        then_target: BlockId,
        /// Target if condition is false.
        else_target: BlockId,
    },

    /// Switch statement. Operand[0] is the discriminant.
    Switch {
        /// Default target if no case matches.
        default: BlockId,
        /// (value, target) pairs.
        cases: Vec<(i64, BlockId)>,
    },

    /// Return from function. Operand[0] is return value if present.
    Ret,

    /// Unreachable code marker.
    Unreachable,

    // =========================================================================
    // SSA operations
    // =========================================================================
    /// Phi node for SSA. Merges values from predecessor blocks.
    Phi {
        /// (predecessor block, value from that block) pairs.
        incoming: Vec<(BlockId, ValueId)>,
    },

    /// Select (ternary). Operand[0] is condition, operand[1] is true value, operand[2] is false value.
    Select,

    // =========================================================================
    // Call operations
    // =========================================================================
    /// Direct function call. Operands are arguments.
    CallDirect {
        /// The function being called.
        callee: FunctionId,
    },

    /// Indirect function call through pointer. The last operand is the function
    /// pointer; all preceding operands are arguments (callee-LAST convention).
    CallIndirect {
        /// Expected function signature at this call site, if known.
        /// Used for type-based call graph pruning.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_signature: Option<TypeId>,
    },

    // =========================================================================
    // Transform operations
    // =========================================================================
    /// Type cast. Operand[0] is the value to cast.
    Cast {
        /// Kind of cast.
        kind: CastKind,
        /// Target type bit-width (e.g., 8 for `trunc i64 to i8`).
        /// `None` for backward compatibility with older AIR JSON.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_bits: Option<u8>,
    },

    /// Binary operation. Operand[0] and operand[1] are the operands.
    BinaryOp {
        /// Kind of binary operation.
        kind: BinaryOp,
    },

    // =========================================================================
    // Miscellaneous
    // =========================================================================
    /// Copy/move value (identity operation, used for clarity in IR).
    Copy,

    /// Freeze undefined value to determinate but unknown value.
    Freeze,
}

impl FieldPath {
    /// Check if the field path is the default (empty).
    fn is_default(&self) -> bool {
        self.steps.is_empty()
    }
}

// =============================================================================
// Values
// =============================================================================

/// A value in AIR — either an instruction result, parameter, or constant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Value {
    /// Result of an instruction.
    InstResult {
        /// The instruction that produces this value.
        inst: InstId,
    },

    /// Function parameter.
    Param {
        /// The function this parameter belongs to.
        func: FunctionId,
        /// Parameter index (0-based).
        index: u32,
    },

    /// Global variable/constant address.
    Global {
        /// The global object.
        id: ObjId,
    },

    /// Inline constant value.
    Const(Constant),
}

// =============================================================================
// Instructions
// =============================================================================

/// An AIR instruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    /// Unique instruction identifier.
    pub id: InstId,

    /// The operation performed.
    #[serde(flatten)]
    pub op: Operation,

    /// Input values (operands).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operands: Vec<ValueId>,

    /// Output value (if the instruction produces one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dst: Option<ValueId>,

    /// Optional source location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,

    /// Optional symbol/name information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<Symbol>,

    /// Type of the result value, if known.
    /// Populated by frontends with type info (e.g., LLVM).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_type: Option<TypeId>,

    /// Frontend-specific extension data.
    /// Analyses that don't understand these extensions should ignore them.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Instruction {
    /// Create a new instruction.
    #[must_use]
    pub fn new(id: InstId, op: Operation) -> Self {
        Self {
            id,
            op,
            operands: Vec::new(),
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    /// Add operands to the instruction.
    #[must_use]
    pub fn with_operands(mut self, operands: Vec<ValueId>) -> Self {
        self.operands = operands;
        self
    }

    /// Set the destination value.
    #[must_use]
    pub fn with_dst(mut self, dst: ValueId) -> Self {
        self.dst = Some(dst);
        self
    }

    /// Set the source span.
    #[must_use]
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Set the symbol.
    #[must_use]
    pub fn with_symbol(mut self, symbol: Symbol) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// Check if this is a terminator instruction.
    #[must_use]
    pub fn is_terminator(&self) -> bool {
        matches!(
            self.op,
            Operation::Br { .. }
                | Operation::CondBr { .. }
                | Operation::Switch { .. }
                | Operation::Ret
                | Operation::Unreachable
        )
    }
}

// =============================================================================
// Basic blocks
// =============================================================================

/// A basic block — a sequence of instructions with single entry/exit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirBlock {
    /// Unique block identifier.
    pub id: BlockId,

    /// Optional block label/name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Instructions in this block (last must be terminator).
    pub instructions: Vec<Instruction>,
}

impl AirBlock {
    /// Create a new block.
    #[must_use]
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            label: None,
            instructions: Vec::new(),
        }
    }

    /// Create a block with a label.
    #[must_use]
    pub fn with_label(id: BlockId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: Some(label.into()),
            instructions: Vec::new(),
        }
    }

    /// Get the terminator instruction if present.
    #[must_use]
    pub fn terminator(&self) -> Option<&Instruction> {
        self.instructions.last().filter(|i| i.is_terminator())
    }
}

// =============================================================================
// Function parameters
// =============================================================================

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirParam {
    /// Value ID for this parameter.
    pub id: ValueId,

    /// Optional parameter name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Parameter index (0-based).
    pub index: u32,

    /// Type of this parameter, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_type: Option<TypeId>,
}

impl AirParam {
    /// Create a new parameter.
    #[must_use]
    pub fn new(id: ValueId, index: u32) -> Self {
        Self {
            id,
            name: None,
            index,
            param_type: None,
        }
    }

    /// Create a named parameter.
    #[must_use]
    pub fn named(id: ValueId, index: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: Some(name.into()),
            index,
            param_type: None,
        }
    }
}

// =============================================================================
// Functions
// =============================================================================

/// An AIR function.
#[derive(Debug, Clone, Serialize)]
pub struct AirFunction {
    /// Unique function identifier.
    pub id: FunctionId,

    /// Function name.
    pub name: String,

    /// Function parameters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<AirParam>,

    /// Basic blocks (first is entry block unless `entry_block` is specified).
    pub blocks: Vec<AirBlock>,

    /// Entry block ID (defaults to first block if not specified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_block: Option<BlockId>,

    /// Whether this function is a declaration (no body).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_declaration: bool,

    /// Optional source location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,

    /// Optional symbol information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<Symbol>,

    /// Pre-computed index: `BlockId` -> position in `blocks` vec.
    /// Skipped during serialization; rebuilt on deserialization.
    #[serde(skip)]
    pub block_index: BTreeMap<BlockId, usize>,
}

impl PartialEq for AirFunction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.params == other.params
            && self.blocks == other.blocks
            && self.entry_block == other.entry_block
            && self.is_declaration == other.is_declaration
            && self.span == other.span
            && self.symbol == other.symbol
    }
}

impl<'de> Deserialize<'de> for AirFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// Helper struct for deserialization (all fields except index).
        #[derive(Deserialize)]
        struct AirFunctionData {
            id: FunctionId,
            name: String,
            #[serde(default)]
            params: Vec<AirParam>,
            blocks: Vec<AirBlock>,
            #[serde(default)]
            entry_block: Option<BlockId>,
            #[serde(default)]
            is_declaration: bool,
            #[serde(default)]
            span: Option<Span>,
            #[serde(default)]
            symbol: Option<Symbol>,
        }

        let data = AirFunctionData::deserialize(deserializer)?;
        let block_index = data
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect();
        Ok(AirFunction {
            id: data.id,
            name: data.name,
            params: data.params,
            blocks: data.blocks,
            entry_block: data.entry_block,
            is_declaration: data.is_declaration,
            span: data.span,
            symbol: data.symbol,
            block_index,
        })
    }
}

impl AirFunction {
    /// Create a new function.
    #[must_use]
    pub fn new(id: FunctionId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    /// Rebuild the `block_index` from the current `blocks` vec.
    ///
    /// Call this after bulk-modifying `blocks` directly (e.g., assigning
    /// a whole `Vec<AirBlock>`). Not needed when using `add_block`.
    pub fn rebuild_block_index(&mut self) {
        self.block_index = self
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect();
    }

    /// Look up a block by `BlockId` using the pre-computed index (O(log n)).
    #[must_use]
    pub fn block(&self, id: BlockId) -> Option<&AirBlock> {
        self.block_index
            .get(&id)
            .and_then(|&idx| self.blocks.get(idx))
    }

    /// Add a block, updating the index.
    pub fn add_block(&mut self, block: AirBlock) {
        let idx = self.blocks.len();
        self.block_index.insert(block.id, idx);
        self.blocks.push(block);
    }

    /// Get the entry block.
    #[must_use]
    pub fn entry(&self) -> Option<&AirBlock> {
        if let Some(entry_id) = self.entry_block {
            self.block(entry_id)
        } else {
            self.blocks.first()
        }
    }
}

// =============================================================================
// Globals
// =============================================================================

/// A global variable or constant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirGlobal {
    /// Value ID for this global's address.
    pub id: ValueId,

    /// Object ID for the allocation.
    pub obj: ObjId,

    /// Global name.
    pub name: String,

    /// Initial value (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init: Option<Constant>,

    /// Whether this is a constant (immutable).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_constant: bool,

    /// Optional source location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,

    /// Type of the global's value (the global itself is always a pointer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<TypeId>,
}

impl AirGlobal {
    /// Create a new global variable.
    #[must_use]
    pub fn new(id: ValueId, obj: ObjId, name: impl Into<String>) -> Self {
        Self {
            id,
            obj,
            name: name.into(),
            init: None,
            is_constant: false,
            span: None,
            value_type: None,
        }
    }
}

// =============================================================================
// Type hierarchy (for CHA / virtual dispatch resolution)
// =============================================================================

/// A virtual method slot in a class's vtable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualMethodSlot {
    /// Slot index in the vtable.
    pub index: usize,
    /// Function ID that occupies this slot, or `None` for pure virtual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionId>,
}

/// Type hierarchy entry for a class/struct with virtual methods.
///
/// Frontend-agnostic: LLVM frontend extracts from `_ZTV*`/`_ZTI*` globals;
/// future frontends populate from their own metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeHierarchyEntry {
    /// Demangled class/struct name.
    pub type_name: String,
    /// Direct base class names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_types: Vec<String>,
    /// Virtual method slots from the vtable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub virtual_methods: Vec<VirtualMethodSlot>,
}

// =============================================================================
// Module
// =============================================================================

/// Default pointer width (64-bit / 8 bytes).
fn default_pointer_width() -> u32 {
    8
}

/// An AIR module — the top-level container for a compilation unit.
#[derive(Debug, Clone, Serialize)]
pub struct AirModule {
    /// Unique module identifier.
    pub id: ModuleId,

    /// Optional module name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Functions in this module.
    pub functions: Vec<AirFunction>,

    /// Global variables/constants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub globals: Vec<AirGlobal>,

    /// Source files referenced by spans.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_files: Vec<SourceFile>,

    /// Type hierarchy entries for CHA (Class Hierarchy Analysis).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_hierarchy: Vec<TypeHierarchyEntry>,

    /// Inline constants: maps `ValueId` to constant value.
    ///
    /// When a constant (like `i32 0`) appears as an instruction operand,
    /// the frontend records the mapping here so analyses can look up
    /// the constant value by `ValueId`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub constants: BTreeMap<ValueId, Constant>,

    /// Type table: maps `TypeId` to `AirType` definition.
    ///
    /// Frontends intern types here during ingestion. Analyses look up
    /// types by `TypeId` for precision improvements. Deterministic
    /// ordering via `BTreeMap` ensures reproducible JSON output.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub types: BTreeMap<TypeId, AirType>,

    /// Target pointer width in bytes (4 for 32-bit, 8 for 64-bit).
    /// Used by layout computation. Defaults to 8.
    #[serde(default = "default_pointer_width")]
    pub target_pointer_width: u32,

    /// Pre-computed index: `FunctionId` -> position in `functions` vec.
    /// Skipped during serialization; rebuilt on deserialization.
    #[serde(skip)]
    pub function_index: BTreeMap<FunctionId, usize>,

    /// Pre-computed index: function name -> position in `functions` vec.
    /// Skipped during serialization; rebuilt on deserialization.
    #[serde(skip)]
    pub name_index: BTreeMap<String, usize>,
}

impl PartialEq for AirModule {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.functions == other.functions
            && self.globals == other.globals
            && self.source_files == other.source_files
            && self.type_hierarchy == other.type_hierarchy
            && self.constants == other.constants
            && self.types == other.types
            && self.target_pointer_width == other.target_pointer_width
    }
}

impl<'de> Deserialize<'de> for AirModule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// Helper struct for deserialization (all fields except index).
        #[derive(Deserialize)]
        struct AirModuleData {
            id: ModuleId,
            #[serde(default)]
            name: Option<String>,
            functions: Vec<AirFunction>,
            #[serde(default)]
            globals: Vec<AirGlobal>,
            #[serde(default)]
            source_files: Vec<SourceFile>,
            #[serde(default)]
            type_hierarchy: Vec<TypeHierarchyEntry>,
            #[serde(default)]
            constants: BTreeMap<ValueId, Constant>,
            #[serde(default)]
            types: BTreeMap<TypeId, AirType>,
            #[serde(default = "default_pointer_width")]
            target_pointer_width: u32,
        }

        let data = AirModuleData::deserialize(deserializer)?;
        let function_index = data
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id, i))
            .collect();
        let name_index = data
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();
        Ok(AirModule {
            id: data.id,
            name: data.name,
            functions: data.functions,
            globals: data.globals,
            source_files: data.source_files,
            type_hierarchy: data.type_hierarchy,
            constants: data.constants,
            types: data.types,
            target_pointer_width: data.target_pointer_width,
            function_index,
            name_index,
        })
    }
}

impl AirModule {
    /// Create a new module.
    #[must_use]
    pub fn new(id: ModuleId) -> Self {
        Self {
            id,
            name: None,
            functions: Vec::new(),
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: BTreeMap::new(),
            types: BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    /// Rebuild the `function_index` from the current `functions` vec.
    ///
    /// Call this after bulk-modifying `functions` directly (e.g., assigning
    /// a whole `Vec<AirFunction>`). Not needed when using `add_function`.
    pub fn rebuild_function_index(&mut self) {
        self.function_index = self
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id, i))
            .collect();
        self.name_index = self
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();
    }

    /// Add a function, updating the index.
    pub fn add_function(&mut self, func: AirFunction) {
        let idx = self.functions.len();
        self.function_index.insert(func.id, idx);
        self.name_index.insert(func.name.clone(), idx);
        self.functions.push(func);
    }

    /// Find a function by ID using the pre-computed index (O(log n)).
    ///
    /// Falls back to linear scan if the index is empty (e.g., when the
    /// module was constructed via struct literal without rebuilding).
    #[must_use]
    pub fn function(&self, id: FunctionId) -> Option<&AirFunction> {
        if self.function_index.is_empty() {
            self.functions.iter().find(|f| f.id == id)
        } else {
            self.function_index
                .get(&id)
                .and_then(|&idx| self.functions.get(idx))
        }
    }

    /// Find a function by ID (mutable) using the pre-computed index (O(log n)).
    ///
    /// Falls back to linear scan if the index is empty.
    #[must_use]
    pub fn function_mut(&mut self, id: FunctionId) -> Option<&mut AirFunction> {
        if self.function_index.is_empty() {
            self.functions.iter_mut().find(|f| f.id == id)
        } else {
            self.function_index
                .get(&id)
                .copied()
                .and_then(|idx| self.functions.get_mut(idx))
        }
    }

    /// Find a function by name using the pre-computed index (O(log n)).
    ///
    /// Falls back to linear scan if the index is empty (e.g., when the
    /// module was constructed via struct literal without rebuilding).
    #[must_use]
    pub fn function_by_name(&self, name: &str) -> Option<&AirFunction> {
        if self.name_index.is_empty() {
            self.functions.iter().find(|f| f.name == name)
        } else {
            self.name_index
                .get(name)
                .and_then(|&idx| self.functions.get(idx))
        }
    }

    /// Find a global by name.
    #[must_use]
    pub fn global_by_name(&self, name: &str) -> Option<&AirGlobal> {
        self.globals.iter().find(|g| g.name == name)
    }

    /// Look up a type by `TypeId`.
    #[must_use]
    pub fn get_type(&self, id: TypeId) -> Option<&AirType> {
        self.types.get(&id)
    }

    /// Check if a `TypeId` resolves to a pointer-like type (`Pointer` or `Reference`).
    #[must_use]
    pub fn is_pointer_type(&self, id: TypeId) -> bool {
        matches!(
            self.types.get(&id),
            Some(AirType::Pointer | AirType::Reference { .. })
        )
    }

    /// Get the type of an instruction's result, if available.
    #[must_use]
    pub fn instruction_type(&self, inst: &Instruction) -> Option<&AirType> {
        inst.result_type.and_then(|id| self.types.get(&id))
    }

    /// Count all values with pointer type in the module.
    ///
    /// Uses the type table to determine pointer types. If no type entry exists
    /// for a value, it is conservatively not counted as a pointer.
    #[must_use]
    pub fn pointer_value_count(&self) -> usize {
        let mut count = self.globals.len(); // globals are always pointers
        for func in &self.functions {
            count += func
                .params
                .iter()
                .filter(|p| p.param_type.is_some_and(|id| self.is_pointer_type(id)))
                .count();
            for block in &func.blocks {
                count += block
                    .instructions
                    .iter()
                    .filter(|i| {
                        i.dst.is_some() && i.result_type.is_some_and(|id| self.is_pointer_type(id))
                    })
                    .count();
            }
        }
        count
    }
}

// =============================================================================
// Bundle
// =============================================================================

/// The bundle produced by a frontend's `ingest()` call.
///
/// Contains the full AIR module plus metadata needed for caching and
/// schema discoverability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirBundle {
    /// The frontend that produced this bundle (e.g., `"llvm"`, `"air-json"`).
    pub frontend_id: String,

    /// Schema version for forward/backward compatibility checks.
    pub schema_version: String,

    /// The AIR module.
    pub module: AirModule,
}

impl AirBundle {
    /// Current schema version.
    pub const SCHEMA_VERSION: &'static str = "0.1.0";

    /// Create a new bundle.
    #[must_use]
    pub fn new(frontend_id: impl Into<String>, module: AirModule) -> Self {
        Self {
            frontend_id: frontend_id.into(),
            schema_version: Self::SCHEMA_VERSION.to_string(),
            module,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_serialization_roundtrip() {
        let constants = vec![
            Constant::i32(42),
            Constant::i64(-1),
            Constant::big_int(i128::MAX, 128),
            Constant::float(3.15, 64),
            Constant::string("hello"),
            Constant::Null,
            Constant::Undef,
            Constant::ZeroInit,
            Constant::Aggregate {
                elements: vec![Constant::i32(1), Constant::i32(2)],
            },
        ];

        for constant in constants {
            let json = serde_json::to_string(&constant).expect("serialize");
            let parsed: Constant = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(constant, parsed);
        }
    }

    #[test]
    fn operation_serialization_roundtrip() {
        let ops = vec![
            Operation::Alloca { size_bytes: None },
            Operation::Load,
            Operation::Store,
            Operation::Ret,
            Operation::Br {
                target: BlockId::new(1),
            },
            Operation::CondBr {
                then_target: BlockId::new(1),
                else_target: BlockId::new(2),
            },
            Operation::CallDirect {
                callee: FunctionId::new(42),
            },
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            Operation::Cast {
                kind: CastKind::Bitcast,
                target_bits: None,
            },
            Operation::HeapAlloc {
                kind: HeapAllocKind::Malloc,
            },
            Operation::HeapAlloc {
                kind: HeapAllocKind::Other("custom_alloc".to_string()),
            },
        ];

        for op in ops {
            let json = serde_json::to_string(&op).expect("serialize");
            let parsed: Operation = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(op, parsed);
        }
    }

    #[test]
    fn heap_alloc_kind_from_str() {
        assert_eq!(HeapAllocKind::from("malloc"), HeapAllocKind::Malloc);
        assert_eq!(HeapAllocKind::from("new"), HeapAllocKind::New);
        assert_eq!(HeapAllocKind::from("operator_new"), HeapAllocKind::New);
        assert_eq!(
            HeapAllocKind::from("operator_new_array"),
            HeapAllocKind::New
        );
        assert_eq!(HeapAllocKind::from("calloc"), HeapAllocKind::Calloc);
        assert_eq!(HeapAllocKind::from("realloc"), HeapAllocKind::Realloc);
        assert_eq!(
            HeapAllocKind::from("my_allocator"),
            HeapAllocKind::Other("my_allocator".to_string())
        );
    }

    #[test]
    fn heap_alloc_kind_as_str() {
        assert_eq!(HeapAllocKind::Malloc.as_str(), "malloc");
        assert_eq!(HeapAllocKind::New.as_str(), "new");
        assert_eq!(HeapAllocKind::Calloc.as_str(), "calloc");
        assert_eq!(HeapAllocKind::Realloc.as_str(), "realloc");
        assert_eq!(
            HeapAllocKind::Other("custom".to_string()).as_str(),
            "custom"
        );
    }

    #[test]
    fn heap_alloc_kind_display() {
        assert_eq!(HeapAllocKind::Malloc.to_string(), "malloc");
        assert_eq!(
            HeapAllocKind::Other("zmalloc".to_string()).to_string(),
            "zmalloc"
        );
    }

    #[test]
    fn heap_alloc_kind_serialization_roundtrip() {
        let kinds = vec![
            HeapAllocKind::Malloc,
            HeapAllocKind::New,
            HeapAllocKind::Calloc,
            HeapAllocKind::Realloc,
            HeapAllocKind::Other("custom_alloc".to_string()),
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).expect("serialize");
            let parsed: HeapAllocKind = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(kind, parsed);
        }

        // Verify exact JSON format for known variants (backward-compatible strings).
        assert_eq!(
            serde_json::to_string(&HeapAllocKind::Malloc).unwrap(),
            "\"malloc\""
        );
        assert_eq!(
            serde_json::to_string(&HeapAllocKind::New).unwrap(),
            "\"new\""
        );
    }

    #[test]
    fn instruction_is_terminator() {
        let br = Instruction::new(
            InstId::new(1),
            Operation::Br {
                target: BlockId::new(2),
            },
        );
        assert!(br.is_terminator());

        let load = Instruction::new(InstId::new(2), Operation::Load);
        assert!(!load.is_terminator());
    }

    #[test]
    fn air_bundle_creation() {
        let module = AirModule::new(ModuleId::derive(b"test"));
        let bundle = AirBundle::new("test-frontend", module);

        assert_eq!(bundle.frontend_id, "test-frontend");
        assert_eq!(bundle.schema_version, AirBundle::SCHEMA_VERSION);
    }

    #[test]
    fn air_bundle_serialization_roundtrip() {
        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.name = Some("test_module".to_string());

        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let bundle = AirBundle::new("air-json", module);

        let json = serde_json::to_string_pretty(&bundle).expect("serialize");
        let parsed: AirBundle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(bundle, parsed);
    }

    #[test]
    fn type_hierarchy_serialization_roundtrip() {
        let mut module = AirModule::new(ModuleId::derive(b"cha_test"));
        module.type_hierarchy.push(TypeHierarchyEntry {
            type_name: "Derived".to_string(),
            base_types: vec!["Base".to_string()],
            virtual_methods: vec![
                VirtualMethodSlot {
                    index: 0,
                    function: Some(FunctionId::derive(b"Derived::process")),
                },
                VirtualMethodSlot {
                    index: 1,
                    function: None, // pure virtual
                },
            ],
        });

        let bundle = AirBundle::new("test", module);
        let json = serde_json::to_string_pretty(&bundle).expect("serialize");
        let parsed: AirBundle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(bundle, parsed);
    }

    #[test]
    fn type_hierarchy_empty_is_omitted_in_json() {
        let module = AirModule::new(ModuleId::derive(b"empty_cha"));
        let bundle = AirBundle::new("test", module);
        let json = serde_json::to_string(&bundle).expect("serialize");
        assert!(
            !json.contains("type_hierarchy"),
            "empty type_hierarchy should be omitted"
        );
    }

    #[test]
    fn type_hierarchy_deserializes_without_field() {
        // JSON from older version without type_hierarchy should still parse
        let json = r#"{"frontend_id":"test","schema_version":"0.1.0","module":{"id":"0x00000000000000000000000000000001","functions":[],"globals":[]}}"#;
        let bundle: AirBundle = serde_json::from_str(json).expect("deserialize");
        assert!(bundle.module.type_hierarchy.is_empty());
    }

    #[test]
    fn air_type_serialization_roundtrip() {
        let types = vec![
            AirType::Pointer,
            AirType::Integer { bits: 32 },
            AirType::Float { bits: 64 },
            AirType::Void,
            AirType::Opaque,
            AirType::Array {
                element: TypeId::derive(b"integer:32"),
                count: Some(10),
            },
            AirType::Struct {
                fields: vec![StructField {
                    field_type: TypeId::derive(b"pointer"),
                    byte_offset: Some(0),
                    byte_size: Some(8),
                    name: None,
                }],
                total_size: 8,
            },
            AirType::Function {
                params: vec![TypeId::derive(b"pointer")],
                return_type: TypeId::derive(b"void"),
            },
        ];

        for ty in types {
            let json = serde_json::to_string(&ty).expect("serialize");
            let parsed: AirType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(ty, parsed);
        }
    }

    #[test]
    fn air_type_opaque_is_default_friendly() {
        let json = serde_json::to_string(&AirType::Opaque).unwrap();
        assert!(json.contains("opaque"));
        let parsed: AirType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, AirType::Opaque);
    }

    #[test]
    fn struct_field_optional_layout() {
        let field = StructField {
            field_type: TypeId::derive(b"integer:32"),
            byte_offset: None,
            byte_size: None,
            name: None,
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(!json.contains("byte_offset"));
        assert!(!json.contains("byte_size"));
        let parsed: StructField = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, field);
    }

    #[test]
    fn struct_field_with_name_roundtrip() {
        let field = StructField {
            field_type: TypeId::derive(b"integer:32"),
            byte_offset: Some(0),
            byte_size: Some(4),
            name: Some("x".to_string()),
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("\"name\":\"x\""));
        let parsed: StructField = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some("x".to_string()));

        // Without name - should be omitted from JSON
        let field_no_name = StructField {
            field_type: TypeId::derive(b"integer:32"),
            byte_offset: None,
            byte_size: None,
            name: None,
        };
        let json2 = serde_json::to_string(&field_no_name).unwrap();
        assert!(!json2.contains("name"));
    }

    #[test]
    fn module_type_table_roundtrip() {
        let mut module = AirModule::new(ModuleId::derive(b"type_test"));
        let ptr_type_id = TypeId::derive(b"pointer");
        module.types.insert(ptr_type_id, AirType::Pointer);
        let i32_type_id = TypeId::derive(b"integer:32");
        module
            .types
            .insert(i32_type_id, AirType::Integer { bits: 32 });

        assert!(module.is_pointer_type(ptr_type_id));
        assert!(!module.is_pointer_type(i32_type_id));
        assert!(module.get_type(ptr_type_id).is_some());

        let bundle = AirBundle::new("test", module);
        let json = serde_json::to_string_pretty(&bundle).expect("serialize");
        let parsed: AirBundle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.module.types.len(), 2);
        assert!(parsed.module.is_pointer_type(ptr_type_id));
    }

    #[test]
    fn module_type_table_empty_omitted_in_json() {
        let module = AirModule::new(ModuleId::derive(b"empty_types"));
        let bundle = AirBundle::new("test", module);
        let json = serde_json::to_string(&bundle).expect("serialize");
        assert!(!json.contains("\"types\""), "empty types should be omitted");
    }

    #[test]
    fn instruction_result_type() {
        let ptr_type = TypeId::derive(b"pointer");
        let mut module = AirModule::new(ModuleId::derive(b"inst_type_test"));
        module.types.insert(ptr_type, AirType::Pointer);

        let mut inst = Instruction::new(InstId::derive(b"load1"), Operation::Load);
        inst.result_type = Some(ptr_type);

        assert!(matches!(
            module.instruction_type(&inst),
            Some(AirType::Pointer)
        ));
    }

    #[test]
    fn pointer_value_count_uses_type_table() {
        let ptr_type = TypeId::derive(b"pointer");
        let i32_type = TypeId::derive(b"integer:32");

        let mut module = AirModule::new(ModuleId::derive(b"count_test"));
        module.types.insert(ptr_type, AirType::Pointer);
        module.types.insert(i32_type, AirType::Integer { bits: 32 });

        let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
        let mut p0 = AirParam::new(ValueId::derive(b"p0"), 0);
        p0.param_type = Some(ptr_type);
        let mut p1 = AirParam::new(ValueId::derive(b"p1"), 1);
        p1.param_type = Some(i32_type);
        func.params = vec![p0, p1];

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        let mut load = Instruction::new(InstId::derive(b"load"), Operation::Load);
        load.dst = Some(ValueId::derive(b"v1"));
        load.result_type = Some(ptr_type);
        let mut add = Instruction::new(
            InstId::derive(b"add"),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
        );
        add.dst = Some(ValueId::derive(b"v2"));
        add.result_type = Some(i32_type);
        block.instructions.push(load);
        block.instructions.push(add);
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        // 0 globals + 1 pointer param + 1 pointer instruction = 2
        assert_eq!(module.pointer_value_count(), 2);
    }

    #[test]
    fn call_indirect_with_signature() {
        let sig = TypeId::derive(b"fn(ptr)->void");
        let op = Operation::CallIndirect {
            expected_signature: Some(sig),
        };
        let json = serde_json::to_string(&op).expect("serialize");
        assert!(json.contains("expected_signature"));
        let parsed: Operation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(op, parsed);
    }

    #[test]
    fn call_indirect_no_signature_omitted() {
        let op = Operation::CallIndirect {
            expected_signature: None,
        };
        let json = serde_json::to_string(&op).expect("serialize");
        assert!(!json.contains("expected_signature"));
    }

    #[test]
    fn instruction_extensions_roundtrip() {
        let mut inst = Instruction::new(InstId::new(1), Operation::Ret);
        inst.extensions.insert(
            "llvm.landingpad".to_string(),
            serde_json::json!({"cleanup": true}),
        );

        let json = serde_json::to_string(&inst).unwrap();
        let roundtripped: Instruction = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.extensions.len(), 1);
        assert_eq!(
            roundtripped.extensions["llvm.landingpad"],
            serde_json::json!({"cleanup": true})
        );

        // Empty extensions should not appear in JSON
        let inst2 = Instruction::new(InstId::new(2), Operation::Ret);
        let json2 = serde_json::to_string(&inst2).unwrap();
        assert!(!json2.contains("extensions"));
    }

    #[test]
    fn air_type_reference_serialization_roundtrip() {
        let types = vec![
            AirType::Reference { nullable: false },
            AirType::Reference { nullable: true },
        ];
        for ty in types {
            let json = serde_json::to_string(&ty).expect("serialize");
            let parsed: AirType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(ty, parsed);
        }
    }

    #[test]
    fn air_type_vector_serialization_roundtrip() {
        let ty = AirType::Vector {
            element: TypeId::derive(b"float:32"),
            lanes: 4,
        };
        let json = serde_json::to_string(&ty).expect("serialize");
        let parsed: AirType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ty, parsed);
    }

    #[test]
    fn target_pointer_width_default() {
        let module = AirModule::new(ModuleId::derive(b"test"));
        assert_eq!(module.target_pointer_width, 8);
    }

    #[test]
    fn target_pointer_width_serialization_roundtrip() {
        let mut module = AirModule::new(ModuleId::derive(b"test32"));
        module.target_pointer_width = 4;
        let bundle = AirBundle::new("test", module);
        let json = serde_json::to_string(&bundle).expect("serialize");
        let parsed: AirBundle = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.module.target_pointer_width, 4);
    }

    #[test]
    fn target_pointer_width_default_when_absent() {
        // JSON without `target_pointer_width` should default to 8
        let json = r#"{"frontend_id":"test","schema_version":"0.1.0","module":{"id":"0x00000000000000000000000000000001","functions":[]}}"#;
        let bundle: AirBundle = serde_json::from_str(json).expect("deserialize");
        assert_eq!(bundle.module.target_pointer_width, 8);
    }
}
