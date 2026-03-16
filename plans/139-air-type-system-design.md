# Plan 139 — Analysis-Oriented Type System for AIR

**Status:** approved
**Created:** 2026-02-20

## Motivation

AIR currently has no type system. The only type hint is a single `is_pointer: bool` flag on instructions and parameters (added in Plan 138). This creates cascading limitations:

- PTA processes non-pointer values, inflating constraint sets
- No struct layout for precise field-sensitive analysis (byte offsets, field overlap)
- No function signatures for type-based indirect call resolution
- Benchmark counting must approximate pointer-typed value counts
- Error reports and exports lack type annotations
- AI agents using the Python SDK cannot query types

## Design Principles

1. **Analysis-oriented, not source-faithful.** The type system describes memory layout, not source-language semantics. Every compiled language resolves to concrete layouts; AIR captures that lowering.
2. **Optional and graceful.** All type fields are `Option`. Analyses fall back to current behavior (sound over-approximation) when types are unavailable.
3. **Deterministic.** Type IDs are content-addressed via BLAKE3. Identical types produce identical IDs across runs.
4. **Untyped pointers.** `Pointer` has no pointee type, matching LLVM 15+ opaque pointers. PTA already tracks what pointers point to via points-to sets.

## Core Type Representation

### `AirType` enum

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AirType {
    /// Pointer to memory (language-agnostic, like LLVM opaque ptr).
    Pointer,

    /// Fixed-width integer.
    Integer { bits: u16 },

    /// Floating-point.
    Float { bits: u16 },

    /// Fixed-size array.
    Array {
        element: TypeId,
        /// None for variable-length arrays.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<u64>,
    },

    /// Struct/record with known field layout.
    Struct {
        fields: Vec<StructField>,
        /// Total size in bytes (including tail padding).
        total_size: u64,
    },

    /// Function signature.
    Function {
        params: Vec<TypeId>,
        return_type: TypeId,
    },

    /// Void (no value / zero-sized).
    Void,

    /// Type is unknown or cannot be expressed.
    /// Analyses fall back to conservative behavior.
    Opaque,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructField {
    pub field_type: TypeId,
    /// Byte offset from struct start. None if layout unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    /// Size in bytes. None if layout unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
}
```

### `TypeId`

New ID type via the existing `define_id_type!` macro:

```rust
define_id_type!(
    /// Unique identifier for a type in the type table.
    TypeId,
    "type"
);
```

`TypeId` is derived from the structural content of the type (e.g., `TypeId::derive(b"integer:32")` for `i32`). This makes it content-addressed and deterministic.

## Storage: Type Table on `AirModule`

```rust
pub struct AirModule {
    // ... existing fields ...

    /// Type table: maps TypeId to AirType definition.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub types: BTreeMap<TypeId, AirType>,
}
```

- **Public/serialized**: `BTreeMap<TypeId, AirType>` for deterministic JSON output (NFR-DET).
- **Analysis internals**: Passes may build `FxHashMap<TypeId, &AirType>` for O(1) lookups.
- **Frontend interning**: `FxHashMap<AirType, TypeId>` reverse-lookup for deduplication during construction.

## Value Type References

Each value-defining construct gets an `Option<TypeId>`:

```rust
pub struct Instruction {
    // ... existing fields ...
    /// Type of the result value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_type: Option<TypeId>,
    // is_pointer: REMOVED (subsumed by result_type)
}

pub struct AirParam {
    // ... existing fields ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_type: Option<TypeId>,
    // is_pointer: REMOVED
}

pub struct AirGlobal {
    // ... existing fields ...
    /// Type of the global's value (globals themselves are always pointers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_type: Option<TypeId>,
}
```

`Operation::CallIndirect` gains an explicit expected function signature:

```rust
CallIndirect {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected_signature: Option<TypeId>,
}
```

## Struct Layout Algorithm

Reimplementation of LLVM's `StructLayout` constructor, adapted for `AirType`:

```
fn compute_struct_layout(fields: &[AirType]) -> Option<(Vec<u64>, u64)>:
    offset = 0
    offsets = []
    max_align = 1
    for each field:
        align = abi_alignment(field)?
        size = alloc_size(field)?
        offset = align_up(offset, align)
        offsets.push(offset)
        offset += size
        max_align = max(max_align, align)
    offset = align_up(offset, max_align)  // tail padding
    return (offsets, offset)
```

ABI alignment rules (System V AMD64 defaults, covers ~99% of x86-64 code):

| Type | Size | Alignment |
|------|------|-----------|
| `Integer { bits: N }` | ceil(N/8) | min(ceil(N/8), 16).next_power_of_two() |
| `Float { bits: 32 }` | 4 | 4 |
| `Float { bits: 64 }` | 8 | 8 |
| `Pointer` | 8 | 8 |
| `Array { element, count }` | count * size(element) | align(element) |
| `Struct { fields }` | computed | max(align(field_i)) |

Edge cases:
- **Packed structs** (`<{ ... }>`): detect `<` prefix in LLVM IR string, force alignment = 1
- **Named/opaque structs**: resolve body if possible, else `byte_offset = None`
- **Vectors** (`<4 x i32>`): treat as arrays

Layout computed once during frontend interning, stored in `StructField.byte_offset`/`byte_size`.

## Frontend Population

### LLVM Frontend

The `MappingContext` gains a `TypeInterner`:

```rust
struct TypeInterner {
    forward: BTreeMap<TypeId, AirType>,   // becomes module.types
    reverse: FxHashMap<AirType, TypeId>,  // dedup lookup
}
```

Type sources:
- **Instruction results**: `inst.get_type().print_to_string()` (already called for `is_pointer`)
- **Function parameters**: `FunctionValue::get_params()` → inspect each `BasicValueEnum`
- **Globals**: `global.get_value_type()` via `print_to_string()`
- **Struct fields**: parse type strings, then compute layout via the algorithm above
- **Arrays**: parse `[N x TYPE]` from `print_to_string()`
- **Function signatures**: `FunctionValue::get_params()` + return type from `print_to_string()`

### AIR-JSON Frontend

Optional fields in the JSON schema:

```json
{
  "types": {
    "0x...": { "kind": "pointer" },
    "0x...": { "kind": "integer", "bits": 32 },
    "0x...": { "kind": "struct", "fields": [...], "total_size": 16 }
  }
}
```

All type fields use `#[serde(default)]`. JSON without types deserializes cleanly.

## Analysis Consumers

All consumers follow the pattern: use types for precision when available, fall back to current behavior when `None`/`Opaque`.

### High priority

1. **PTA constraint filtering** — Skip non-pointer values in constraint extraction. Reduces constraint set size.
2. **Field-sensitive precision** — Byte-offset-aware locations via `StructField.byte_offset`. Detect field overlap.
3. **Type-based CG pruning** — Filter indirect call targets by function signature compatibility via `expected_signature`.
4. **Benchmark counting** — `pointer_value_count()` becomes exact via type table lookup.

### Medium priority

5. **Value-flow graph** — Type-annotate flow edges; filter non-pointer flows for pointer-focused queries.
6. **Taint source/sink selectors** — Type-based matching (e.g., "all `char*` params are injection sinks").
7. **Typestate analysis** — Associate state machines with typed allocations.
8. **Error reporting / SARIF** — Type names in diagnostics ("null dereference of `struct Node*`").
9. **Python SDK / AI agent API** — Expose type queries for custom analyzer authoring.

### Lower priority (still in initial scope)

10. **Abstract interpretation** — Precise integer overflow/wrap semantics from `Integer { bits }`.
11. **DDA pruning** — Filter backward search by type.
12. **Export enrichment** — Type annotations on PropertyGraph nodes.

## Migration

### `is_pointer` removal

`is_pointer` on `Instruction` and `AirParam` is replaced by `result_type`/`param_type` resolving to `AirType::Pointer`. Steps:

1. Add `result_type`/`param_type` alongside `is_pointer`
2. Populate both in LLVM frontend
3. Migrate all consumers to use type table
4. Remove `is_pointer`

### Backwards compatibility

- `is_pointer` was added in Plan 138 (in-progress), no external consumers
- All new type fields are `Option` with `#[serde(default)]`
- Schema version stays `0.1.0` (pre-1.0)

### Utility methods on `AirModule`

```rust
impl AirModule {
    fn get_type(&self, id: TypeId) -> Option<&AirType>;
    fn is_pointer_type(&self, id: TypeId) -> bool;
    fn instruction_type(&self, inst: &Instruction) -> Option<&AirType>;
    fn pointer_value_count(&self) -> usize;  // updated to use type table
}
```

## Estimated Scope

| Component | Lines (est.) |
|-----------|-------------|
| `AirType` enum + `StructField` + `TypeId` | ~100 |
| Type table on `AirModule` + utility methods | ~80 |
| Layout algorithm (`compute_struct_layout`) | ~120 |
| LLVM frontend `TypeInterner` + population | ~300 |
| LLVM type string parsing (structs, arrays, functions) | ~150 |
| AIR-JSON schema + frontend updates | ~80 |
| `is_pointer` migration + removal | ~50 |
| PTA constraint filtering | ~60 |
| Field-sensitive precision (byte-offset locations) | ~100 |
| Type-based CG pruning | ~80 |
| Benchmark counting update | ~30 |
| Value-flow type annotations | ~60 |
| Taint selector type matching | ~50 |
| Typestate type association | ~40 |
| Error reporting / SARIF types | ~40 |
| Python SDK type queries | ~60 |
| Abstract interpretation bit-width | ~40 |
| DDA type-based pruning | ~30 |
| Export enrichment | ~30 |
| **Total** | **~1,500** |
