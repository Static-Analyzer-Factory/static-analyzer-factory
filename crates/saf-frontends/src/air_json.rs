//! AIR JSON frontend — ingests `.air.json` files (FR-FE-004).
//!
//! This frontend serves as:
//! - A test contract for unit/integration testing without LLVM
//! - A stable interface for future source-level frontends

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use saf_core::air::{
    AirBlock, AirBundle, AirFunction, AirGlobal, AirModule, AirParam, AirType, BinaryOp, CastKind,
    Constant, FieldPath, FieldStep, HeapAllocKind, Instruction, Operation, StructField,
};
use saf_core::config::Config;
use saf_core::ids::{BlockId, FileId, FunctionId, InstId, ModuleId, ObjId, TypeId, ValueId};
use saf_core::span::{SourceFile, Span, Symbol};

use crate::air_json_schema::{
    JsonAirBundle, JsonAirType, JsonBlock, JsonConstant, JsonFieldStep, JsonFunction, JsonGlobal,
    JsonInstruction, JsonModule, JsonParam, JsonSourceFile, JsonSpan, JsonSymbol,
};
use crate::api::Frontend;
use crate::error::FrontendError;

/// Frontend that ingests SAF's AIR JSON contract format (`.air.json`).
pub struct AirJsonFrontend;

impl AirJsonFrontend {
    /// Current supported schema version.
    pub const SCHEMA_VERSION: &'static str = "0.1.0";
}

impl Frontend for AirJsonFrontend {
    fn ingest(&self, inputs: &[&Path], _config: &Config) -> Result<AirBundle, FrontendError> {
        if inputs.is_empty() {
            return Err(FrontendError::Parse("no input files provided".to_string()));
        }

        if inputs.len() > 1 {
            return Err(FrontendError::Parse(
                "air-json frontend currently only supports single file input".to_string(),
            ));
        }

        let path = inputs[0];
        let content = fs::read_to_string(path)?;
        let json_bundle: JsonAirBundle =
            serde_json::from_str(&content).map_err(|e| FrontendError::Parse(e.to_string()))?;

        // Validate schema version
        if json_bundle.schema_version != Self::SCHEMA_VERSION {
            return Err(FrontendError::Parse(format!(
                "unsupported schema version '{}', expected '{}'",
                json_bundle.schema_version,
                Self::SCHEMA_VERSION
            )));
        }

        let module = convert_module(&json_bundle.module, path)?;

        Ok(AirBundle {
            frontend_id: self.frontend_id().to_string(),
            schema_version: json_bundle.schema_version,
            module,
        })
    }

    fn input_fingerprint_bytes(
        &self,
        inputs: &[&Path],
        _config: &Config,
    ) -> Result<Vec<u8>, FrontendError> {
        // Hash the normalized content of all input files
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"air-json:");

        for path in inputs {
            let content = fs::read_to_string(path)?;
            // Normalize by re-parsing and re-serializing (removes formatting differences)
            let json: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| FrontendError::Parse(e.to_string()))?;
            let normalized =
                serde_json::to_string(&json).map_err(|e| FrontendError::Parse(e.to_string()))?;
            hasher.update(normalized.as_bytes());
        }

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    fn supported_features(&self) -> BTreeMap<String, bool> {
        let mut features = BTreeMap::new();
        features.insert("spans".to_string(), true);
        features.insert("symbols".to_string(), true);
        features.insert("heap_alloc".to_string(), true);
        features.insert("constants".to_string(), true);
        features
    }

    fn frontend_id(&self) -> &'static str {
        "air-json"
    }
}

// =============================================================================
// Conversion helpers
// =============================================================================

/// Parse a hex ID string, returning an error if invalid.
fn parse_hex_id(s: &str) -> Result<u128, FrontendError> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(hex_str, 16)
        .map_err(|e| FrontendError::Parse(format!("invalid hex ID '{s}': {e}")))
}

/// Context for generating deterministic IDs during conversion.
struct ConversionContext {
    module_path: String,
    inst_counter: u64,
    block_counter: u64,
}

impl ConversionContext {
    fn new(path: &Path) -> Self {
        Self {
            module_path: path.to_string_lossy().to_string(),
            inst_counter: 0,
            block_counter: 0,
        }
    }

    fn derive_module_id(&self) -> ModuleId {
        ModuleId::derive(self.module_path.as_bytes())
    }

    fn derive_function_id(&self, name: &str) -> FunctionId {
        let data = format!("{}:{}", self.module_path, name);
        FunctionId::derive(data.as_bytes())
    }

    fn derive_block_id(&mut self, func_name: &str, label: Option<&str>) -> BlockId {
        let data = if let Some(label) = label {
            format!("{}:{}:{}", self.module_path, func_name, label)
        } else {
            self.block_counter += 1;
            format!(
                "{}:{}:block{}",
                self.module_path, func_name, self.block_counter
            )
        };
        BlockId::derive(data.as_bytes())
    }

    fn derive_inst_id(&mut self, func_name: &str) -> InstId {
        self.inst_counter += 1;
        let data = format!(
            "{}:{}:inst{}",
            self.module_path, func_name, self.inst_counter
        );
        InstId::derive(data.as_bytes())
    }

    fn derive_param_id(&self, func_name: &str, index: u32) -> ValueId {
        let data = format!("{}:{}:param{}", self.module_path, func_name, index);
        ValueId::derive(data.as_bytes())
    }

    fn derive_obj_id(&self, name: &str) -> ObjId {
        let data = format!("{}:obj:{}", self.module_path, name);
        ObjId::derive(data.as_bytes())
    }

    fn derive_global_value_id(&self, name: &str) -> ValueId {
        let data = format!("{}:global:{}", self.module_path, name);
        ValueId::derive(data.as_bytes())
    }
}

fn convert_module(json: &JsonModule, path: &Path) -> Result<AirModule, FrontendError> {
    let mut ctx = ConversionContext::new(path);

    let id = if let Some(ref id_str) = json.id {
        ModuleId::new(parse_hex_id(id_str)?)
    } else {
        ctx.derive_module_id()
    };

    let mut module = AirModule::new(id);
    module.name.clone_from(&json.name);

    // Convert globals first (so we have their IDs for instructions)
    for json_global in &json.globals {
        module.globals.push(convert_global(json_global, &mut ctx)?);
    }

    // Convert functions
    for json_func in &json.functions {
        module.add_function(convert_function(json_func, &mut ctx)?);
    }

    // Convert source files
    for json_file in &json.source_files {
        module.source_files.push(convert_source_file(json_file));
    }

    // Convert types
    for json_type_entry in &json.types {
        let type_id = TypeId::new(parse_hex_id(&json_type_entry.id)?);
        let air_type = convert_json_air_type(&json_type_entry.ty)?;
        module.types.insert(type_id, air_type);
    }

    Ok(module)
}

fn convert_function(
    json: &JsonFunction,
    ctx: &mut ConversionContext,
) -> Result<AirFunction, FrontendError> {
    let id = if let Some(ref id_str) = json.id {
        FunctionId::new(parse_hex_id(id_str)?)
    } else {
        ctx.derive_function_id(&json.name)
    };

    let mut func = AirFunction::new(id, &json.name);
    func.is_declaration = json.is_declaration;

    // Convert parameters
    for (i, json_param) in json.params.iter().enumerate() {
        // INVARIANT: Functions have at most hundreds of parameters; index fits in u32.
        #[allow(clippy::cast_possible_truncation)]
        let idx = i as u32;
        func.params
            .push(convert_param(json_param, &json.name, idx, ctx)?);
    }

    // Convert blocks
    for json_block in &json.blocks {
        func.add_block(convert_block(json_block, &json.name, ctx)?);
    }

    // Set entry block if specified
    if let Some(ref entry_str) = json.entry_block {
        func.entry_block = Some(BlockId::new(parse_hex_id(entry_str)?));
    }

    // Convert span and symbol
    func.span = json.span.as_ref().map(convert_span);
    func.symbol = json.symbol.as_ref().map(convert_symbol);

    Ok(func)
}

fn convert_param(
    json: &JsonParam,
    func_name: &str,
    position: u32,
    ctx: &ConversionContext,
) -> Result<AirParam, FrontendError> {
    let index = json.index.unwrap_or(position);

    let id = if let Some(ref id_str) = json.id {
        ValueId::new(parse_hex_id(id_str)?)
    } else {
        ctx.derive_param_id(func_name, index)
    };

    Ok(AirParam {
        id,
        name: json.name.clone(),
        index,
        param_type: json
            .param_type
            .as_ref()
            .map(|s| parse_hex_id(s))
            .transpose()?
            .map(TypeId::new),
    })
}

fn convert_block(
    json: &JsonBlock,
    func_name: &str,
    ctx: &mut ConversionContext,
) -> Result<AirBlock, FrontendError> {
    let id = if let Some(ref id_str) = json.id {
        BlockId::new(parse_hex_id(id_str)?)
    } else {
        ctx.derive_block_id(func_name, json.label.as_deref())
    };

    let mut block = AirBlock::new(id);
    block.label.clone_from(&json.label);

    for json_inst in &json.instructions {
        block
            .instructions
            .push(convert_instruction(json_inst, func_name, ctx)?);
    }

    Ok(block)
}

fn convert_instruction(
    json: &JsonInstruction,
    func_name: &str,
    ctx: &mut ConversionContext,
) -> Result<Instruction, FrontendError> {
    let id = if let Some(ref id_str) = json.id {
        InstId::new(parse_hex_id(id_str)?)
    } else {
        ctx.derive_inst_id(func_name)
    };

    let op = convert_operation(json)?;

    let mut inst = Instruction::new(id, op);

    // Convert operands
    for operand_str in &json.operands {
        inst.operands.push(ValueId::new(parse_hex_id(operand_str)?));
    }

    // Convert destination
    if let Some(ref dst_str) = json.dst {
        inst.dst = Some(ValueId::new(parse_hex_id(dst_str)?));
    }

    // Convert span and symbol
    inst.span = json.span.as_ref().map(convert_span);
    inst.symbol = json.symbol.as_ref().map(convert_symbol);

    // Pass through result type
    if let Some(ref type_str) = json.result_type {
        inst.result_type = Some(TypeId::new(parse_hex_id(type_str)?));
    }

    Ok(inst)
}

// NOTE: This function is intentionally long because it exhaustively maps all
// JSON operation names to their AIR `Operation` equivalents. The 1:1 mapping
// makes this the authoritative operation dispatch table.
#[allow(clippy::too_many_lines)]
fn convert_operation(json: &JsonInstruction) -> Result<Operation, FrontendError> {
    match json.op.as_str() {
        // Allocation
        "alloca" => Ok(Operation::Alloca {
            size_bytes: json.size_bytes,
        }),
        "global" => {
            let obj_str = json
                .obj
                .as_ref()
                .ok_or_else(|| FrontendError::Parse("global requires 'obj' field".to_string()))?;
            Ok(Operation::Global {
                obj: ObjId::new(parse_hex_id(obj_str)?),
            })
        }
        "heap_alloc" => {
            let kind = json.heap_kind.clone().ok_or_else(|| {
                FrontendError::Parse("heap_alloc requires 'heap_kind'".to_string())
            })?;
            Ok(Operation::HeapAlloc {
                kind: HeapAllocKind::from(kind.as_str()),
            })
        }

        // Memory
        "load" => Ok(Operation::Load),
        "store" => Ok(Operation::Store),
        "gep" => {
            let field_path = if let Some(ref fp) = json.field_path {
                FieldPath {
                    steps: fp.steps.iter().map(convert_field_step).collect(),
                }
            } else {
                FieldPath::empty()
            };
            Ok(Operation::Gep { field_path })
        }
        "memcpy" => Ok(Operation::Memcpy),
        "memset" => Ok(Operation::Memset),

        // Control flow
        "br" => {
            let target_str = json
                .target
                .as_ref()
                .ok_or_else(|| FrontendError::Parse("br requires 'target' field".to_string()))?;
            Ok(Operation::Br {
                target: BlockId::new(parse_hex_id(target_str)?),
            })
        }
        "cond_br" => {
            let then_str = json.then_target.as_ref().ok_or_else(|| {
                FrontendError::Parse("cond_br requires 'then_target' field".to_string())
            })?;
            let else_str = json.else_target.as_ref().ok_or_else(|| {
                FrontendError::Parse("cond_br requires 'else_target' field".to_string())
            })?;
            Ok(Operation::CondBr {
                then_target: BlockId::new(parse_hex_id(then_str)?),
                else_target: BlockId::new(parse_hex_id(else_str)?),
            })
        }
        "switch" => {
            let default_str = json.default.as_ref().ok_or_else(|| {
                FrontendError::Parse("switch requires 'default' field".to_string())
            })?;
            let mut cases = Vec::new();
            for (val, block_str) in &json.cases {
                cases.push((*val, BlockId::new(parse_hex_id(block_str)?)));
            }
            Ok(Operation::Switch {
                default: BlockId::new(parse_hex_id(default_str)?),
                cases,
            })
        }
        "ret" => Ok(Operation::Ret),
        "unreachable" => Ok(Operation::Unreachable),

        // SSA
        "phi" => {
            let mut incoming = Vec::new();
            for (block_str, value_str) in &json.incoming {
                incoming.push((
                    BlockId::new(parse_hex_id(block_str)?),
                    ValueId::new(parse_hex_id(value_str)?),
                ));
            }
            Ok(Operation::Phi { incoming })
        }
        "select" => Ok(Operation::Select),

        // Calls
        "call_direct" => {
            let callee_str = json.callee.as_ref().ok_or_else(|| {
                FrontendError::Parse("call_direct requires 'callee' field".to_string())
            })?;
            Ok(Operation::CallDirect {
                callee: FunctionId::new(parse_hex_id(callee_str)?),
            })
        }
        "call_indirect" => Ok(Operation::CallIndirect {
            expected_signature: None,
        }),

        // Transforms
        "cast" => {
            let kind_str = json.cast_kind.as_ref().ok_or_else(|| {
                FrontendError::Parse("cast requires 'cast_kind' field".to_string())
            })?;
            let kind = parse_cast_kind(kind_str)?;
            Ok(Operation::Cast {
                kind,
                target_bits: None,
            })
        }
        "binary_op" => {
            let op_str = json.binary_op.as_ref().ok_or_else(|| {
                FrontendError::Parse("binary_op requires 'binary_op' field".to_string())
            })?;
            let kind = parse_binary_op(op_str)?;
            Ok(Operation::BinaryOp { kind })
        }

        // Misc
        "copy" => Ok(Operation::Copy),
        "freeze" => Ok(Operation::Freeze),

        other => Err(FrontendError::Parse(format!("unknown operation: {other}"))),
    }
}

fn convert_field_step(json: &JsonFieldStep) -> FieldStep {
    match json {
        JsonFieldStep::Index => FieldStep::Index,
        JsonFieldStep::Field { index } => FieldStep::Field { index: *index },
    }
}

fn parse_cast_kind(s: &str) -> Result<CastKind, FrontendError> {
    match s {
        "trunc" => Ok(CastKind::Trunc),
        "z_ext" => Ok(CastKind::ZExt),
        "s_ext" => Ok(CastKind::SExt),
        "fp_to_ui" => Ok(CastKind::FPToUI),
        "fp_to_si" => Ok(CastKind::FPToSI),
        "ui_to_fp" => Ok(CastKind::UIToFP),
        "si_to_fp" => Ok(CastKind::SIToFP),
        "fp_trunc" => Ok(CastKind::FPTrunc),
        "fp_ext" => Ok(CastKind::FPExt),
        "ptr_to_int" => Ok(CastKind::PtrToInt),
        "int_to_ptr" => Ok(CastKind::IntToPtr),
        "bitcast" => Ok(CastKind::Bitcast),
        "addr_space_cast" => Ok(CastKind::AddrSpaceCast),
        other => Err(FrontendError::Parse(format!("unknown cast kind: {other}"))),
    }
}

fn parse_binary_op(s: &str) -> Result<BinaryOp, FrontendError> {
    match s {
        "add" => Ok(BinaryOp::Add),
        "sub" => Ok(BinaryOp::Sub),
        "mul" => Ok(BinaryOp::Mul),
        "u_div" => Ok(BinaryOp::UDiv),
        "s_div" => Ok(BinaryOp::SDiv),
        "u_rem" => Ok(BinaryOp::URem),
        "s_rem" => Ok(BinaryOp::SRem),
        "f_add" => Ok(BinaryOp::FAdd),
        "f_sub" => Ok(BinaryOp::FSub),
        "f_mul" => Ok(BinaryOp::FMul),
        "f_div" => Ok(BinaryOp::FDiv),
        "f_rem" => Ok(BinaryOp::FRem),
        "and" => Ok(BinaryOp::And),
        "or" => Ok(BinaryOp::Or),
        "xor" => Ok(BinaryOp::Xor),
        "shl" => Ok(BinaryOp::Shl),
        "l_shr" => Ok(BinaryOp::LShr),
        "a_shr" => Ok(BinaryOp::AShr),
        "i_cmp_eq" => Ok(BinaryOp::ICmpEq),
        "i_cmp_ne" => Ok(BinaryOp::ICmpNe),
        "i_cmp_ugt" => Ok(BinaryOp::ICmpUgt),
        "i_cmp_uge" => Ok(BinaryOp::ICmpUge),
        "i_cmp_ult" => Ok(BinaryOp::ICmpUlt),
        "i_cmp_ule" => Ok(BinaryOp::ICmpUle),
        "i_cmp_sgt" => Ok(BinaryOp::ICmpSgt),
        "i_cmp_sge" => Ok(BinaryOp::ICmpSge),
        "i_cmp_slt" => Ok(BinaryOp::ICmpSlt),
        "i_cmp_sle" => Ok(BinaryOp::ICmpSle),
        "f_cmp_oeq" => Ok(BinaryOp::FCmpOeq),
        "f_cmp_one" => Ok(BinaryOp::FCmpOne),
        "f_cmp_ogt" => Ok(BinaryOp::FCmpOgt),
        "f_cmp_oge" => Ok(BinaryOp::FCmpOge),
        "f_cmp_olt" => Ok(BinaryOp::FCmpOlt),
        "f_cmp_ole" => Ok(BinaryOp::FCmpOle),
        other => Err(FrontendError::Parse(format!("unknown binary op: {other}"))),
    }
}

fn convert_global(
    json: &JsonGlobal,
    ctx: &mut ConversionContext,
) -> Result<AirGlobal, FrontendError> {
    let id = if let Some(ref id_str) = json.id {
        ValueId::new(parse_hex_id(id_str)?)
    } else {
        ctx.derive_global_value_id(&json.name)
    };

    let obj = if let Some(ref obj_str) = json.obj {
        ObjId::new(parse_hex_id(obj_str)?)
    } else {
        ctx.derive_obj_id(&json.name)
    };

    let mut global = AirGlobal::new(id, obj, &json.name);
    global.init = json.init.as_ref().map(convert_constant).transpose()?;
    global.is_constant = json.is_constant;
    global.span = json.span.as_ref().map(convert_span);

    if let Some(ref type_str) = json.value_type {
        global.value_type = Some(TypeId::new(parse_hex_id(type_str)?));
    }

    Ok(global)
}

fn convert_constant(json: &JsonConstant) -> Result<Constant, FrontendError> {
    match json {
        JsonConstant::Int { value, bits } => Ok(Constant::Int {
            value: *value,
            bits: *bits,
        }),
        JsonConstant::BigInt { value, bits } => Ok(Constant::BigInt {
            value: value.clone(),
            bits: *bits,
        }),
        JsonConstant::Float { value, bits } => Ok(Constant::Float {
            value: *value,
            bits: *bits,
        }),
        JsonConstant::String { value } => Ok(Constant::String {
            value: value.clone(),
        }),
        JsonConstant::Null => Ok(Constant::Null),
        JsonConstant::Undef => Ok(Constant::Undef),
        JsonConstant::ZeroInit => Ok(Constant::ZeroInit),
        JsonConstant::Aggregate { elements } => Ok(Constant::Aggregate {
            elements: elements
                .iter()
                .map(convert_constant)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        JsonConstant::GlobalRef { value } => {
            let id = parse_hex_id(value)?;
            Ok(Constant::GlobalRef(ValueId::new(id)))
        }
    }
}

fn convert_span(json: &JsonSpan) -> Span {
    Span {
        file_id: FileId::new(json.file_id),
        byte_start: json.byte_start,
        byte_end: json.byte_end,
        line_start: json.line_start,
        col_start: json.col_start,
        line_end: json.line_end,
        col_end: json.col_end,
    }
}

fn convert_symbol(json: &JsonSymbol) -> Symbol {
    Symbol {
        display_name: json.display_name.clone(),
        mangled_name: json.mangled_name.clone(),
        namespace_path: json.namespace_path.clone(),
    }
}

fn convert_source_file(json: &JsonSourceFile) -> SourceFile {
    SourceFile {
        id: FileId::new(json.id),
        path: json.path.clone(),
        checksum: json.checksum.clone(),
    }
}

fn convert_json_air_type(json: &JsonAirType) -> Result<AirType, FrontendError> {
    match json {
        JsonAirType::Pointer => Ok(AirType::Pointer),
        JsonAirType::Reference { nullable } => Ok(AirType::Reference {
            nullable: *nullable,
        }),
        JsonAirType::Vector { element, lanes } => Ok(AirType::Vector {
            element: TypeId::new(parse_hex_id(element)?),
            lanes: *lanes,
        }),
        JsonAirType::Integer { bits } => Ok(AirType::Integer { bits: *bits }),
        JsonAirType::Float { bits } => Ok(AirType::Float { bits: *bits }),
        JsonAirType::Array { element, count } => Ok(AirType::Array {
            element: TypeId::new(parse_hex_id(element)?),
            count: *count,
        }),
        JsonAirType::Struct { fields, total_size } => {
            let mut air_fields = Vec::with_capacity(fields.len());
            for f in fields {
                air_fields.push(StructField {
                    field_type: TypeId::new(parse_hex_id(&f.field_type)?),
                    byte_offset: f.byte_offset,
                    byte_size: f.byte_size,
                    name: f.name.clone(),
                });
            }
            Ok(AirType::Struct {
                fields: air_fields,
                total_size: *total_size,
            })
        }
        JsonAirType::Function {
            params,
            return_type,
        } => {
            let mut air_params = Vec::with_capacity(params.len());
            for p in params {
                air_params.push(TypeId::new(parse_hex_id(p)?));
            }
            Ok(AirType::Function {
                params: air_params,
                return_type: TypeId::new(parse_hex_id(return_type)?),
            })
        }
        JsonAirType::Void => Ok(AirType::Void),
        JsonAirType::Opaque => Ok(AirType::Opaque),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_id() {
        let frontend = AirJsonFrontend;
        assert_eq!(frontend.frontend_id(), "air-json");
    }

    #[test]
    fn supported_features() {
        let frontend = AirJsonFrontend;
        let features = frontend.supported_features();
        assert_eq!(features.get("spans"), Some(&true));
        assert_eq!(features.get("symbols"), Some(&true));
        assert_eq!(features.get("heap_alloc"), Some(&true));
    }

    #[test]
    fn parse_hex_id_with_prefix() {
        let id = parse_hex_id("0x123").unwrap();
        assert_eq!(id, 0x123);
    }

    #[test]
    fn parse_hex_id_without_prefix() {
        let id = parse_hex_id("abc").unwrap();
        assert_eq!(id, 0xabc);
    }

    #[test]
    fn parse_cast_kinds() {
        assert!(matches!(parse_cast_kind("trunc"), Ok(CastKind::Trunc)));
        assert!(matches!(parse_cast_kind("bitcast"), Ok(CastKind::Bitcast)));
        assert!(parse_cast_kind("invalid").is_err());
    }

    #[test]
    fn parse_binary_ops() {
        assert!(matches!(parse_binary_op("add"), Ok(BinaryOp::Add)));
        assert!(matches!(parse_binary_op("i_cmp_eq"), Ok(BinaryOp::ICmpEq)));
        assert!(parse_binary_op("invalid").is_err());
    }
}
