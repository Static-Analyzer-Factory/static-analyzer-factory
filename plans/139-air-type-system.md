# AIR Type System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an analysis-oriented type system to AIR with a centralized type table, enabling PTA filtering, field-sensitive precision, type-based CG pruning, and 9 other analysis improvements.

**Architecture:** `AirType` enum in saf-core with `TypeId` content-addressed via BLAKE3. Type table `BTreeMap<TypeId, AirType>` on `AirModule`. Values reference types via `Option<TypeId>`. LLVM frontend interns types via `TypeInterner`, computing struct layouts from a reimplemented LLVM `StructLayout` algorithm. All type fields are optional — analyses fall back to current behavior when absent.

**Tech Stack:** Rust (serde, BLAKE3, inkwell 0.8), Python (PyO3)

**Design doc:** `plans/139-air-type-system-design.md`

**Key project rules:**
- `saf-core` can be tested locally (`cargo test -p saf-core`)
- All other crates need Docker (`make test`)
- Always `make fmt && make lint` before committing
- Use `BTreeMap` for public APIs, `FxHashMap` for internal hot paths
- Prefer specific assertions over count assertions in tests
- Subagents must NEVER call `make` commands — main agent only

---

## Phase 1: Core Types (saf-core — testable locally)

### Task 1: Add `TypeId` newtype

**Files:**
- Modify: `crates/saf-core/src/ids.rs`

**Step 1: Add TypeId via existing macro**

In `crates/saf-core/src/ids.rs`, after the `LocId` definition (line ~133), add:

```rust
define_id_type!(
    /// Unique identifier for a type in the type table.
    TypeId,
    "type"
);
```

**Step 2: Write test**

Add to `ids.rs` `mod tests`:

```rust
#[test]
fn type_id_derive_is_deterministic() {
    let a = TypeId::derive(b"integer:32");
    let b = TypeId::derive(b"integer:32");
    assert_eq!(a, b);
}

#[test]
fn type_id_different_from_other_domains() {
    let type_id = TypeId::derive(b"test");
    let value_id = ValueId::derive(b"test");
    assert_ne!(type_id.raw(), value_id.raw());
}
```

**Step 3: Run tests locally**

```bash
cargo test -p saf-core -- ids::tests
```

**Step 4: Commit**

```
feat(core): add TypeId newtype for type table
```

---

### Task 2: Add `AirType` enum and `StructField`

**Files:**
- Modify: `crates/saf-core/src/air.rs`

**Step 1: Add types after the `Constant` section (before `FieldStep`)**

Add after line ~120 (after `Constant` impl block):

```rust
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
}
```

Also add `TypeId` to the import from `crate::ids` at the top of air.rs.

**Step 2: Write tests**

Add to `air.rs` `mod tests`:

```rust
#[test]
fn air_type_serialization_roundtrip() {
    use crate::ids::TypeId;

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
    // Opaque types should serialize minimally
    let json = serde_json::to_string(&AirType::Opaque).unwrap();
    assert!(json.contains("opaque"));
    let parsed: AirType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, AirType::Opaque);
}

#[test]
fn struct_field_optional_layout() {
    use crate::ids::TypeId;

    // StructField without layout info (byte_offset/byte_size = None)
    let field = StructField {
        field_type: TypeId::derive(b"integer:32"),
        byte_offset: None,
        byte_size: None,
    };
    let json = serde_json::to_string(&field).unwrap();
    // Optional fields should be omitted
    assert!(!json.contains("byte_offset"));
    assert!(!json.contains("byte_size"));
    let parsed: StructField = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, field);
}
```

**Step 3: Run tests locally**

```bash
cargo test -p saf-core -- air::tests
```

**Step 4: Commit**

```
feat(core): add AirType enum and StructField for type system
```

---

### Task 3: Add type table to `AirModule` and value type references

**Files:**
- Modify: `crates/saf-core/src/air.rs`

**Step 1: Add `types` field to `AirModule`**

Add after the `constants` field (~line 828):

```rust
    /// Type table: maps `TypeId` to `AirType` definition.
    ///
    /// Frontends intern types here during ingestion. Analyses look up
    /// types by `TypeId` for precision improvements. Deterministic
    /// ordering via `BTreeMap` ensures reproducible JSON output.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub types: std::collections::BTreeMap<TypeId, AirType>,
```

Initialize in `AirModule::new()`:

```rust
    types: std::collections::BTreeMap::new(),
```

**Step 2: Add `result_type` to `Instruction`, `param_type` to `AirParam`, `value_type` to `AirGlobal`**

On `Instruction` (add after `is_pointer` field):

```rust
    /// Type of the result value, if known.
    /// Populated by frontends with type info (e.g., LLVM).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_type: Option<TypeId>,
```

Initialize as `result_type: None` in `Instruction::new()`.

On `AirParam` (add after `is_pointer` field):

```rust
    /// Type of this parameter, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_type: Option<TypeId>,
```

Initialize as `param_type: None` in `AirParam::new()` and `AirParam::named()`.

On `AirGlobal` (add after `span` field):

```rust
    /// Type of the global's value (the global itself is always a pointer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<TypeId>,
```

Initialize as `value_type: None` in `AirGlobal::new()`.

**Step 3: Add `expected_signature` to `CallIndirect`**

Change `CallIndirect` from a unit variant to:

```rust
    /// Indirect function call through pointer. Operand[0] is function pointer, rest are arguments.
    CallIndirect {
        /// Expected function signature at this call site, if known.
        /// Used for type-based call graph pruning.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_signature: Option<TypeId>,
    },
```

Update all references to `Operation::CallIndirect` in the codebase to use `Operation::CallIndirect { expected_signature: None }` pattern.

**Important:** Search all crates for `CallIndirect` pattern matches and update them. Key files:
- `crates/saf-core/src/air.rs` (tests)
- `crates/saf-frontends/src/llvm/mapping.rs` (~3 occurrences)
- `crates/saf-frontends/src/air_json.rs`
- `crates/saf-analysis/src/pta/extract.rs`
- `crates/saf-analysis/src/callgraph.rs`
- `crates/saf-analysis/src/cg_refinement.rs`
- Any match arms with `CallIndirect` pattern

Use `Operation::CallIndirect { .. }` for match patterns that don't need the field.

**Step 4: Add utility methods to `AirModule`**

```rust
impl AirModule {
    /// Look up a type by `TypeId`.
    #[must_use]
    pub fn get_type(&self, id: TypeId) -> Option<&AirType> {
        self.types.get(&id)
    }

    /// Check if a `TypeId` resolves to `AirType::Pointer`.
    #[must_use]
    pub fn is_pointer_type(&self, id: TypeId) -> bool {
        matches!(self.types.get(&id), Some(AirType::Pointer))
    }

    /// Get the type of an instruction's result, if available.
    #[must_use]
    pub fn instruction_type(&self, inst: &Instruction) -> Option<&AirType> {
        inst.result_type.and_then(|id| self.types.get(&id))
    }
}
```

**Step 5: Update `pointer_value_count` to use type table when available**

Replace the existing method:

```rust
    /// Count all values with pointer type in the module.
    ///
    /// Uses the type table when available, falling back to `is_pointer`
    /// flag for backwards compatibility during migration.
    #[must_use]
    pub fn pointer_value_count(&self) -> usize {
        let mut count = self.globals.len(); // globals are always pointers
        for func in &self.functions {
            count += func
                .params
                .iter()
                .filter(|p| {
                    p.param_type
                        .map_or(p.is_pointer, |id| self.is_pointer_type(id))
                })
                .count();
            for block in &func.blocks {
                count += block
                    .instructions
                    .iter()
                    .filter(|i| {
                        i.dst.is_some()
                            && i.result_type
                                .map_or(i.is_pointer, |id| self.is_pointer_type(id))
                    })
                    .count();
            }
        }
        count
    }
```

**Step 6: Write tests**

```rust
#[test]
fn module_type_table_roundtrip() {
    use crate::ids::TypeId;

    let mut module = AirModule::new(ModuleId::derive(b"type_test"));
    let ptr_type_id = TypeId::derive(b"pointer");
    module.types.insert(ptr_type_id, AirType::Pointer);
    let i32_type_id = TypeId::derive(b"integer:32");
    module.types.insert(i32_type_id, AirType::Integer { bits: 32 });

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
    use crate::ids::TypeId;

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
    use crate::ids::TypeId;

    let ptr_type = TypeId::derive(b"pointer");
    let i32_type = TypeId::derive(b"integer:32");

    let mut module = AirModule::new(ModuleId::derive(b"count_test"));
    module.types.insert(ptr_type, AirType::Pointer);
    module.types.insert(i32_type, AirType::Integer { bits: 32 });

    let mut func = AirFunction::new(FunctionId::derive(b"main"), "main");
    // One pointer param, one int param
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
        Operation::BinaryOp { kind: BinaryOp::Add },
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
    use crate::ids::TypeId;

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
```

**Step 7: Run tests locally**

```bash
cargo test -p saf-core
```

**Step 8: Commit**

```
feat(core): add type table to AirModule with value type references
```

---

### Task 4: Struct layout algorithm

**Files:**
- Create: `crates/saf-core/src/layout.rs`
- Modify: `crates/saf-core/src/lib.rs` (add `pub mod layout;`)

**Step 1: Create layout module**

Create `crates/saf-core/src/layout.rs` with the struct layout algorithm reimplemented from LLVM's `StructLayout`. The module provides:

- `abi_alignment(ty: &AirType, types: &BTreeMap<TypeId, AirType>) -> Option<u64>` — ABI alignment for a type
- `alloc_size(ty: &AirType, types: &BTreeMap<TypeId, AirType>) -> Option<u64>` — allocation size in bytes
- `compute_struct_layout(fields: &[StructField], types: &BTreeMap<TypeId, AirType>) -> Option<(Vec<u64>, u64)>` — returns (field_offsets, total_size)

The `types` parameter is the type table for resolving `TypeId` references in nested types.

System V AMD64 ABI defaults:
- `Integer { bits: N }`: size = ceil(N/8), align = min(size, 16).next_power_of_two()
- `Float { bits: 32 }`: size = 4, align = 4
- `Float { bits: 64 }`: size = 8, align = 8
- `Pointer`: size = 8, align = 8
- `Array`: size = count * element_size, align = element_align
- `Struct`: size = computed, align = max(field_align)
- `Void`: size = 0, align = 1
- `Opaque`: returns `None`

**Step 2: Write comprehensive tests**

Test cases:
- Simple struct `{ i32, ptr }` → offsets [0, 8], total 16 (i32 at 0, 4 bytes padding, ptr at 8)
- Packed-like struct `{ i8, i8, i8 }` → offsets [0, 1, 2], total 3
- Nested arrays `{ [10 x i32], ptr }` → offsets [0, 40], total 48
- Empty struct → total 0
- Struct with Opaque field → returns None
- Single pointer field → offsets [0], total 8
- `{ i8, i32 }` → offsets [0, 4], total 8 (3 bytes padding after i8)
- `{ i64, i8 }` → offsets [0, 8], total 16 (7 bytes tail padding)
- Integer sizes: i1=1 byte, i8=1, i16=2, i32=4, i64=8, i128=16

**Step 3: Run tests locally**

```bash
cargo test -p saf-core -- layout::tests
```

**Step 4: Commit**

```
feat(core): add struct layout algorithm (reimplemented from LLVM StructLayout)
```

---

## Phase 2: Frontend Population (needs Docker)

### Task 5: LLVM frontend — `TypeInterner` and type string parsing

**Files:**
- Create: `crates/saf-frontends/src/llvm/type_intern.rs`
- Modify: `crates/saf-frontends/src/llvm/mod.rs` (add `pub(crate) mod type_intern;`)
- Modify: `crates/saf-frontends/src/llvm/mapping.rs`

**Step 1: Create `TypeInterner`**

Create `crates/saf-frontends/src/llvm/type_intern.rs`:

```rust
//! Type interning for LLVM-to-AIR type conversion.

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;

use saf_core::air::{AirType, StructField};
use saf_core::ids::TypeId;
use saf_core::layout;

/// Interns `AirType` values into a deduplicated type table.
///
/// Uses content-addressed `TypeId` derivation for determinism.
/// Maintains a reverse map for O(1) dedup lookups.
pub(crate) struct TypeInterner {
    /// Forward map: TypeId → AirType (becomes `AirModule.types`).
    forward: BTreeMap<TypeId, AirType>,
    /// Reverse map: AirType → TypeId (dedup lookup).
    reverse: FxHashMap<AirType, TypeId>,
}
```

Methods:
- `new() -> Self`
- `intern(&mut self, ty: AirType) -> TypeId` — interns a type, returns its ID
- `intern_pointer(&mut self) -> TypeId` — shorthand for `Pointer`
- `intern_integer(&mut self, bits: u16) -> TypeId` — shorthand
- `intern_float(&mut self, bits: u16) -> TypeId` — shorthand
- `intern_void(&mut self) -> TypeId`
- `into_table(self) -> BTreeMap<TypeId, AirType>` — consumes and returns the forward map
- `parse_llvm_type_string(&mut self, type_str: &str) -> TypeId` — parses LLVM IR type strings like `"ptr"`, `"i32"`, `"float"`, `"double"`, `"[10 x i32]"`, `"{ i32, ptr }"`, `"void"`

The `TypeId` derivation uses a canonical string representation:
- `Pointer` → `TypeId::derive(b"pointer")`
- `Integer { bits: 32 }` → `TypeId::derive(b"integer:32")`
- `Float { bits: 64 }` → `TypeId::derive(b"float:64")`
- `Void` → `TypeId::derive(b"void")`
- `Opaque` → `TypeId::derive(b"opaque")`
- `Array { element, count }` → `TypeId::derive(format!("array:{element_id}:{count}").as_bytes())`
- `Struct { fields, total_size }` → `TypeId::derive(format!("struct:{field_types_and_offsets}:{total_size}").as_bytes())`
- `Function { params, ret }` → `TypeId::derive(format!("function:{param_ids}:{ret_id}").as_bytes())`

**Step 2: Implement `parse_llvm_type_string`**

This parses LLVM IR type string representations:
- `"ptr"` → `AirType::Pointer`
- `"i1"`, `"i8"`, `"i32"`, `"i64"`, `"i128"` → `AirType::Integer { bits }`
- `"float"` → `AirType::Float { bits: 32 }`
- `"double"` → `AirType::Float { bits: 64 }`
- `"void"` → `AirType::Void`
- `"[N x TYPE]"` → `AirType::Array { element, count }`
- `"{ TYPE, TYPE, ... }"` → `AirType::Struct { fields, total_size }` (using `layout::compute_struct_layout`)
- `"<{ TYPE, TYPE }>"` → packed struct (alignment=1)
- Anything else → `AirType::Opaque`

**Step 3: Write tests for type string parsing**

Test each LLVM type string pattern. Use unit tests within the module.

**Step 4: Commit**

```
feat(frontends): add TypeInterner with LLVM type string parsing
```

---

### Task 6: LLVM frontend — populate types on instructions, params, globals

**Files:**
- Modify: `crates/saf-frontends/src/llvm/mapping.rs`

**Step 1: Add `TypeInterner` to `MappingContext`**

Add `pub type_interner: TypeInterner` field. Initialize in `MappingContext::new()`.

**Step 2: Populate instruction result types**

In `convert_instruction()` (line ~828-855), where `has_result` is true:
- The code already computes `type_str` (line 848). Instead of just checking `"ptr"`, parse it via `ctx.type_interner.parse_llvm_type_string(&type_str)` and set `air_inst.result_type = Some(type_id)`.
- Keep `is_pointer` assignment for backwards compat during migration.

Similarly update intrinsic handlers (`convert_intrinsic_call`, lines ~918, 932).

For `Alloca` instructions: type is always `Pointer` (alloca returns a pointer).

For `HeapAlloc`: type is always `Pointer`.

For `Gep`: type is always `Pointer` (already set via `is_pointer = true`).

**Step 3: Populate parameter types**

In `convert_function()` (line ~631-638), where params are created:
- Use `matches!(param, BasicValueEnum::PointerValue(_))` → `Pointer`
- `BasicValueEnum::IntValue(v)` → `Integer { bits: v.get_type().get_bit_width() }`
- `BasicValueEnum::FloatValue(v)` → parse float type
- Others → `Opaque`

**Step 4: Populate global value types**

In `convert_global()` (line ~446-479):
- Parse the global's value type from `global.get_value_type().print_to_string()`

**Step 5: Populate function signatures and `CallIndirect.expected_signature`**

In `convert_function()`:
- Build `AirType::Function` from param types + return type
- Store the function's signature TypeId

In `convert_call_instruction()` for indirect calls:
- Parse the function type from the call instruction's IR string
- Set `expected_signature` on `Operation::CallIndirect`

**Step 6: Wire type table to module**

In `convert_module()` (line ~434-442):
- After conversion, set `air_module.types = ctx.type_interner.into_table()`

**Step 7: Run full test suite in Docker**

```bash
make fmt && make test
```

**Step 8: Commit**

```
feat(frontends): populate AIR type table from LLVM frontend
```

---

### Task 7: AIR-JSON frontend — type table support

**Files:**
- Modify: `crates/saf-frontends/src/air_json_schema.rs`
- Modify: `crates/saf-frontends/src/air_json.rs`

**Step 1: Add type fields to JSON schema**

In `air_json_schema.rs`:
- Add `JsonAirType` enum (mirrors `AirType`)
- Add `JsonStructField` struct
- Add `types: Option<BTreeMap<String, JsonAirType>>` to `JsonModule`
- Add `result_type: Option<String>` to `JsonInstruction`
- Add `param_type: Option<String>` to `JsonParam`
- Add `value_type: Option<String>` to `JsonGlobal`
- Add `expected_signature: Option<String>` to `JsonInstruction` (for `call_indirect` op)

**Step 2: Update conversion in `air_json.rs`**

In `convert_module()`:
- Convert `json.types` entries to `AirType` + populate `module.types`

In `convert_instruction()`:
- Pass through `json.result_type` as `Option<TypeId>`

In `convert_param()`:
- Pass through `json.param_type`

In `convert_global()`:
- Pass through `json.value_type`

In `convert_operation()` for `"call_indirect"`:
- Parse `expected_signature` from JSON

**Step 3: Write test for JSON roundtrip with types**

Write a small `.air.json` test fixture with a type table and type references on instructions/params, verify it round-trips correctly.

**Step 4: Run tests**

```bash
make fmt && make test
```

**Step 5: Commit**

```
feat(frontends): add type table support to AIR-JSON frontend
```

---

### Task 8: Remove `is_pointer` flag

**Files:**
- Modify: `crates/saf-core/src/air.rs`
- Modify: `crates/saf-frontends/src/llvm/mapping.rs`
- Modify: `crates/saf-frontends/src/air_json.rs`
- Modify: `crates/saf-frontends/src/air_json_schema.rs`
- Modify: `crates/saf-analysis/src/mssa/modref.rs`
- Modify: `crates/saf-analysis/src/mssa/builder.rs`
- Modify: `crates/saf-analysis/src/dda/solver.rs`
- Modify: `crates/saf-analysis/src/pta/value_origin.rs`
- Modify: `crates/saf-bench/src/cruxbc.rs`

**Step 1: Remove `is_pointer` from `Instruction` and `AirParam`**

Remove the field, its serde annotation, and its initialization in constructors.

**Step 2: Update all synthetic instruction creation sites**

Every place that creates an `Instruction` with `is_pointer: false` (found in mssa/modref.rs, mssa/builder.rs, dda/solver.rs) — remove the field.

**Step 3: Update `value_origin.rs`**

Line 618 copies `is_pointer` from an existing instruction — remove.

**Step 4: Update frontends**

- `mapping.rs`: Remove all `air_inst.is_pointer = ...` assignments, `air_param.is_pointer = ...`
- `air_json.rs`: Remove `is_pointer` pass-through
- `air_json_schema.rs`: Remove `is_pointer` fields from `JsonParam` and `JsonInstruction`

**Step 5: Update benchmark counting**

In `cruxbc.rs`, update any direct `is_pointer` references to use the type table.

**Step 6: Update `pointer_value_count` to type-table only**

Remove the `is_pointer` fallback in the method since the field no longer exists.

**Step 7: Run full test suite**

```bash
make fmt && make test
```

**Step 8: Commit**

```
refactor(core): remove is_pointer flag, subsumed by type table
```

---

## Phase 3: Analysis Consumers (needs Docker)

### Task 9: PTA constraint filtering

**Files:**
- Modify: `crates/saf-analysis/src/pta/extract.rs`

**Step 1: Add type-aware filtering**

In constraint extraction, when generating `CopyConstraint` for operations like `Phi`, `Select`, and `Cast`:
- If `result_type` is available and resolves to a non-pointer type, skip the constraint
- If `result_type` is `None` or `Opaque`, conservatively generate the constraint (current behavior)

Add a helper:

```rust
fn is_pointer_typed(vid: ValueId, module: &AirModule) -> bool {
    // Check if any instruction or param with this ValueId has a pointer result_type
    // Conservative: return true if type info unavailable
    // Implementation scans the module's instructions/params for the ValueId
}
```

Or better: pass a prebuilt `FxHashMap<ValueId, TypeId>` into the extraction context.

**Step 2: Write test**

Test that non-pointer operations (e.g., integer phi) don't generate Copy constraints when types are available.

**Step 3: Run tests**

```bash
make fmt && make test
```

**Step 4: Commit**

```
feat(pta): filter non-pointer values from constraint extraction using type table
```

---

### Task 10: Field-sensitive byte-offset locations

**Files:**
- Modify: `crates/saf-analysis/src/pta/location.rs`

**Step 1: Enhance `LocationFactory` with type-aware field resolution**

Add a method that, given a base location + GEP field path + type info, creates locations using byte offsets instead of abstract field indices. This enables detecting field overlap (e.g., unions).

**Step 2: Write test**

Test that two GEP paths to the same byte offset resolve to the same location.

**Step 3: Run tests**

```bash
make fmt && make test
```

**Step 4: Commit**

```
feat(pta): byte-offset-aware field locations using type table
```

---

### Task 11: Type-based CG pruning

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs`

**Step 1: Add signature compatibility check**

When resolving indirect calls in `resolve_and_connect`, if the callsite has `expected_signature`:
- Filter PTA-resolved targets to only those whose function signature is compatible (matching param count and compatible param types)
- Preserve all targets if signature is `None` (current behavior)

**Step 2: Write test**

Test that indirect call resolution filters incompatible function signatures.

**Step 3: Run tests**

```bash
make fmt && make test
```

**Step 4: Commit**

```
feat(cg): type-based pruning of indirect call targets via function signatures
```

---

### Task 12: Benchmark counting update

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs`

**Step 1: Update stats computation**

Replace any `is_pointer`-based counting with type-table-based counting. Verify `total_pointer_values` metric uses `module.pointer_value_count()` which now uses the type table.

**Step 2: Run benchmarks**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small/*" -o /workspace/tests/benchmarks/cruxbc/results-types.json'
```

Verify metrics are populated and reasonable.

**Step 3: Commit**

```
feat(bench): update benchmark counting to use type table
```

---

### Task 13: Remaining analysis consumers (batch)

This task covers the 8 remaining consumers. Each is a small change (~30-60 lines). They can be implemented as subagent tasks in parallel.

**Consumers:**
1. **Value-flow typing** (`crates/saf-analysis/src/valueflow/builder.rs`): Annotate VF edges with type info from source/dest instructions.
2. **Taint selector type matching** (`crates/saf-analysis/src/selector/`): Add type-based selector predicates.
3. **Typestate type association** (`crates/saf-analysis/src/typestate/` or checker code): Use allocation types to select state machines.
4. **Error reporting / SARIF** (`crates/saf-analysis/src/export.rs`): Add type names to PropertyGraph node properties.
5. **Python SDK** (`crates/saf-python/`): Expose `module.types`, `module.get_type()`, `module.is_pointer_type()` via PyO3.
6. **Abstract interpretation** (`crates/saf-analysis/src/absint/`): Use `Integer { bits }` for precise overflow/wrap semantics in interval analysis.
7. **DDA pruning** (`crates/saf-analysis/src/dda/solver.rs`): Skip backward search through non-pointer typed values.
8. **Export enrichment** (`crates/saf-analysis/src/export.rs`): Add `type` property to PropertyGraph nodes when type info available.

Each consumer follows the same pattern:
1. Accept `&AirModule` or a type lookup table
2. Check type info availability
3. Use it for precision/annotation
4. Fall back to current behavior when absent

**Commit per consumer or batch:**

```
feat(analysis): wire type system into value-flow, taint, typestate, SARIF, SDK, absint, DDA, export
```

---

## Phase 4: Validation

### Task 14: End-to-end validation

**Step 1: Run full test suite**

```bash
make fmt && make lint && make test
```

**Step 2: Run PTABen benchmarks**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-types.json'
```

Verify no regression from baseline (68 Unsound). Expect potential improvement from PTA constraint filtering.

**Step 3: Run CruxBC benchmarks**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small/*" -o /workspace/tests/benchmarks/cruxbc/results-types.json'
```

Compare constraint counts and timing with pre-type-system baseline.

**Step 4: Update PROGRESS.md**

Record results, constraint reduction %, timing changes, any precision improvements.

**Step 5: Final commit**

```
docs: update PROGRESS.md with Plan 139 results
```

---

## Dependency Graph

```
Task 1 (TypeId)
  → Task 2 (AirType enum)
    → Task 3 (type table + value refs)
      → Task 4 (layout algorithm)
        → Task 5 (LLVM TypeInterner)
          → Task 6 (LLVM population)
            → Task 7 (AIR-JSON support)
              → Task 8 (remove is_pointer)
                → Tasks 9-13 (analysis consumers, parallelizable)
                  → Task 14 (validation)
```

Tasks 9-13 are independent of each other and can run in parallel once Task 8 completes.

## Estimated Total

~1,500 lines across ~25 files. Phase 1 (Tasks 1-4) is testable locally. Phases 2-4 need Docker.
