# E1: Frontend API + AIR JSON Frontend — Implementation Plan

## Summary

Implement the full AIR (Analysis Intermediate Representation) data model and the AIR JSON frontend that parses `.air.json` files into `AirBundle`. This establishes the test contract for all downstream analysis work.

**Key design decisions (from brainstorming):**
- **Flat enum** for AIR instructions (pattern-match friendly, extensible via optional metadata)
- **Hierarchical/nested JSON schema** (module → functions → blocks → instructions)
- **Newtypes** for IDs (ValueId, InstId, BlockId, FunctionId, ObjId, ModuleId)
- **Golden tests** (implement, snapshot outputs as regression baselines)

---

## Files to Modify/Create

### saf-core (AIR data model)

| File | Action | Description |
|------|--------|-------------|
| `crates/saf-core/src/air.rs` | **Expand** | Full AIR types: Module, Function, Block, Instruction, Value, etc. |
| `crates/saf-core/src/ids.rs` | **New** | Newtype wrappers: ModuleId, FunctionId, BlockId, InstId, ValueId, ObjId |
| `crates/saf-core/src/span.rs` | **New** | Source span types for optional metadata |
| `crates/saf-core/src/lib.rs` | **Edit** | Export new modules |

### saf-frontends (AIR JSON frontend)

| File | Action | Description |
|------|--------|-------------|
| `crates/saf-frontends/src/air_json.rs` | **Expand** | Implement `Frontend` trait: parse JSON, build AirBundle |
| `crates/saf-frontends/src/air_json_schema.rs` | **New** | JSON schema types (serde structs for deserialization) |

### Test fixtures

| File | Action | Description |
|------|--------|-------------|
| `tests/fixtures/air_json/minimal.air.json` | **New** | Single function, Ret only |
| `tests/fixtures/air_json/memory_ops.air.json` | **New** | Alloca, Load, Store, GEP |
| `tests/fixtures/air_json/control_flow.air.json` | **New** | CondBr, Phi, multiple blocks |
| `tests/fixtures/air_json/calls.air.json` | **New** | CallDirect, CallIndirect |
| `tests/fixtures/air_json/constants.air.json` | **New** | Int, Float, String, Null, Aggregate constants |
| `tests/fixtures/expected/` | **Generated** | Golden test outputs (snapshots) |

### Tests

| File | Action | Description |
|------|--------|-------------|
| `crates/saf-core/tests/air_model.rs` | **New** | Unit tests for AIR types, ID derivation |
| `crates/saf-frontends/tests/air_json_frontend.rs` | **New** | Integration tests for parsing fixtures |
| `crates/saf-frontends/tests/golden/` | **New** | Golden test snapshots |

---

## Implementation Steps

### Step 1: Define ID newtypes (`saf-core/src/ids.rs`)

Newtype wrappers for type-safe ID handling:
- ModuleId, FunctionId, BlockId, InstId, ValueId, ObjId
- Implement Display (hex format), Debug, From<u128>, PartialEq, Eq, Hash, Ord, PartialOrd
- Implement Serialize/Deserialize with hex string format
- Implement deterministic ID derivation using `saf_core::id::make_id`

### Step 2: Define source span types (`saf-core/src/span.rs`)

- Span: file_id, byte_start, byte_end, line_start, col_start, line_end, col_end
- Symbol: display_name, mangled_name, namespace_path

### Step 3: Define constants and AIR instruction enum (`saf-core/src/air.rs`)

Constant enum:
- Int { value: i128, bits: u8 }
- Float { value: f64, bits: u8 }
- String(String)
- Null
- Undef
- ZeroInit
- Aggregate(Vec<Constant>)

Operation enum (flat, pattern-match friendly):
- Allocation: Alloca, Global, HeapAlloc
- Memory: Load, Store, GEP, Memcpy, Memset
- Control flow: Br, CondBr, Switch, Ret
- SSA: Phi, Select
- Calls: CallDirect, CallIndirect
- Transforms: Cast, BinaryOp

### Step 4: Define AIR module structure

- AirModule: id, functions, globals
- AirFunction: id, name, params, blocks, entry_block, span
- AirBlock: id, instructions
- AirParam: id, name, index
- AirGlobal: id, obj, name, span
- Instruction: id, op, operands, dst, span, symbol
- Value: InstResult, Param, Global, Const

### Step 5: Create JSON schema types

Separate serde structs optimized for JSON parsing (may differ from internal types).

### Step 6: Implement AIR JSON frontend

Implement Frontend trait:
- ingest(): read JSON, validate schema, convert types, derive IDs
- input_fingerprint_bytes(): hash normalized JSON content
- supported_features(): report spans, symbols, heap_alloc support

### Step 7: Create test fixtures

- minimal.air.json: single function with Ret
- memory_ops.air.json: Alloca, Store, Load, GEP
- control_flow.air.json: multiple blocks, CondBr, Phi
- calls.air.json: CallDirect, CallIndirect
- constants.air.json: all constant types

### Step 8: Write tests and generate golden snapshots

Use insta crate for snapshot testing.

---

## Dependencies to Add

```toml
# In workspace Cargo.toml [workspace.dependencies]
insta = { version = "1.40", features = ["json"] }

# In saf-frontends/Cargo.toml [dev-dependencies]
insta.workspace = true
```

---

## Verification

After implementation, verify:

1. **`make test`** — All tests pass (Rust + Python)
2. **`make lint`** — clippy + rustfmt clean
3. **Determinism check** — Parse same fixture twice, verify identical output
4. **Golden snapshots committed** — insta review shows no unexpected changes

---

## Acceptance Criteria (from SRS)

- [ ] AT-FE-01: `air-json` frontend loads fixture and produces valid AIR + schema
- [ ] FR-AIR-001: AIR data model sufficient for pointer/value-flow analyses
- [ ] FR-AIR-002: Deterministic u128 IDs serialized as hex with 0x prefix
- [ ] FR-AIR-006: Minimum operation set (Alloca, Load, Store, GEP, Br, Call, etc.)
- [ ] NFR-DET-001: Byte-identical outputs for identical inputs
