# E2: LLVM Frontend — Implementation Plan

## Summary

Implement the LLVM Bitcode frontend that reads `.bc` files and converts them to AIR, enabling SAF to analyze real-world C/C++/Rust programs compiled with LLVM.

**Key design decisions:**
- **LLVM versions:** 17 + 18 supported, extensible architecture for future versions
- **Version abstraction:** Adapter trait with feature flags (`llvm-17`, `llvm-18`)
- **`invoke` handling:** Flatten to `CallDirect`/`CallIndirect` (exception edges deferred)
- **Intrinsic handling:** Pattern-based allowlist (not full table)
- **Debug info:** Best-effort extraction when present
- **Test fixtures:** Hybrid approach — `.ll` for precision, C sources for realism

---

## Architecture

### Module Structure

```
crates/saf-frontends/src/
  llvm/
    mod.rs              # LlvmFrontend (implements Frontend trait)
    adapter.rs          # LlvmAdapter trait (version-agnostic interface)
    llvm17.rs           # LLVM 17 adapter (feature = "llvm-17")
    llvm18.rs           # LLVM 18 adapter (feature = "llvm-18")
    mapping.rs          # LLVM IR → AIR conversion (shared logic)
    intrinsics.rs       # Pattern-based intrinsic handling
    debug_info.rs       # Debug metadata → Span/Symbol extraction
    error.rs            # LLVM-specific error types
```

### Feature Flags

```toml
# saf-frontends/Cargo.toml
[features]
default = ["llvm-18"]
llvm-17 = ["inkwell/llvm17-0"]
llvm-18 = ["inkwell/llvm18-0"]

[dependencies]
inkwell = { workspace = true, optional = true }
```

### Trait Hierarchy

```
┌─────────────────────────────────────────────────────┐
│ Frontend (saf-frontends::api)                       │
│   fn ingest() -> AirBundle                          │
│   fn input_fingerprint_bytes() -> Vec<u8>           │
│   fn supported_features() -> BTreeMap               │
│   fn frontend_id() -> &'static str                  │
└─────────────────────────────────────────────────────┘
                        ▲
                        │ implements
┌─────────────────────────────────────────────────────┐
│ LlvmFrontend                                        │
│   adapter: Box<dyn LlvmAdapter>                     │
└─────────────────────────────────────────────────────┘
                        │ delegates to
                        ▼
┌─────────────────────────────────────────────────────┐
│ LlvmAdapter (internal trait)                        │
│   fn parse_bitcode(bytes) -> LlvmModule            │
│   fn get_functions(module) -> Vec<LlvmFunction>    │
│   fn get_instructions(block) -> Vec<LlvmInst>      │
│   ...                                               │
└─────────────────────────────────────────────────────┘
        ▲                               ▲
        │                               │
┌───────────────┐             ┌───────────────┐
│ Llvm17Adapter │             │ Llvm18Adapter │
└───────────────┘             └───────────────┘
```

---

## LLVM → AIR Mapping

### Instruction Mapping

| LLVM Instruction | AIR Operation | Notes |
|------------------|---------------|-------|
| `alloca` | `Alloca` | Creates stack object |
| `load` | `Load` | operands[0] = pointer |
| `store` | `Store` | operands[0] = value, operands[1] = pointer |
| `getelementptr` | `Gep { field_path }` | Extract indices → FieldPath |
| `call` (direct) | `CallDirect { callee }` | Resolve function → FunctionId |
| `call` (indirect) | `CallIndirect` | operands[0] = function pointer |
| `invoke` | `CallDirect`/`CallIndirect` | **Flatten** — ignore exception edges |
| `ret` | `Ret` | operands[0] = return value (if any) |
| `br` (uncond) | `Br { target }` | Single target |
| `br` (cond) | `CondBr { then_target, else_target }` | operands[0] = condition |
| `switch` | `Switch { default, cases }` | operands[0] = discriminant |
| `phi` | `Phi { incoming }` | (BlockId, ValueId) pairs |
| `select` | `Select` | operands = [cond, true_val, false_val] |
| `trunc`, `zext`, `sext`, ... | `Cast { kind }` | CastKind enum variant |
| `add`, `sub`, `mul`, ... | `BinaryOp { kind }` | BinaryOp enum variant |
| `icmp`, `fcmp` | `BinaryOp { kind }` | ICmpEq, FCmpOlt, etc. |
| `unreachable` | `Unreachable` | Direct map |

### Value Mapping

| LLVM Value | AIR Value |
|------------|-----------|
| Instruction result | `Value::InstResult { inst }` |
| Function argument | `Value::Param { func, index }` |
| Global variable | `Value::Global { id }` |
| ConstantInt | `Value::Const(Constant::Int { .. })` |
| ConstantFP | `Value::Const(Constant::Float { .. })` |
| ConstantArray/Struct | `Value::Const(Constant::Aggregate { .. })` |
| ConstantPointerNull | `Value::Const(Constant::Null)` |
| UndefValue | `Value::Const(Constant::Undef)` |

---

## Intrinsic Handling

Pattern-based recognition with allowlist:

```rust
pub enum IntrinsicMapping {
    MapTo(Operation),  // Map to specific AIR operation
    Skip,              // No AIR instruction emitted
    External,          // Treat as external call
}

pub fn classify_intrinsic(name: &str) -> IntrinsicMapping {
    match name {
        // Memory operations
        n if n.starts_with("llvm.memcpy.") => MapTo(Operation::Memcpy),
        n if n.starts_with("llvm.memmove.") => MapTo(Operation::Memcpy),
        n if n.starts_with("llvm.memset.") => MapTo(Operation::Memset),

        // Skip - no pointer/value-flow effect
        n if n.starts_with("llvm.lifetime.") => Skip,
        n if n.starts_with("llvm.dbg.") => Skip,
        n if n.starts_with("llvm.assume") => Skip,
        n if n.starts_with("llvm.annotation.") => Skip,
        n if n.starts_with("llvm.invariant.") => Skip,
        n if n.starts_with("llvm.experimental.") => Skip,

        // Pass-through (value identity)
        n if n.starts_with("llvm.expect.") => /* emit Copy */,

        // Everything else → external call
        _ => External,
    }
}
```

---

## ID Derivation

Deterministic ID generation following FR-AIR-005:

| Entity | Derivation |
|--------|------------|
| Module | `hash("llvm", bitcode_bytes)` |
| Function | `hash(module_id, "fn", function_name)` |
| Block | `hash(function_id, "bb", block_index)` |
| Instruction | `hash(block_id, "inst", inst_index)` |
| Value (inst result) | `hash(inst_id, "result")` |
| Value (param) | `hash(function_id, "param", param_index)` |
| Value (named global) | `hash(module_id, "global", global_name)` |
| Value (unnamed global) | `hash(module_id, "anon_global", decl_index)` |
| Object (alloca) | `hash(inst_id, "obj")` |
| Object (heap) | `hash(inst_id, "heap")` |
| Object (global) | `hash(module_id, "gobj", global_name)` |

**Known limitation:** Unnamed globals use declaration order index. See `plans/FUTURE.md` for details.

---

## Debug Info Extraction

Best-effort extraction from LLVM debug metadata:

| LLVM Metadata | AIR Type | Extraction |
|---------------|----------|------------|
| `DILocation` | `Span` | line, column, file reference |
| `DIFile` | `SourceFile` | filename, directory |
| `DISubprogram` | `Symbol` | function name, linkage name |

**Semantics:**
- If no debug info: `span` and `symbol` fields are `None`
- If partial debug info: extract what's available
- Never fail ingestion due to missing/malformed debug info

---

## Test Fixtures

### Hand-written `.ll` (precise control)

```
tests/fixtures/llvm/
  minimal.ll           # Single function, ret void
  memory_ops.ll        # alloca, load, store, gep
  control_flow.ll      # br, cond_br, phi, switch
  calls.ll             # direct call, indirect call
  globals.ll           # named globals, unnamed globals
  intrinsics.ll        # memcpy, memset, lifetime
  debug_info.ll        # DILocation, DISubprogram metadata
  constants.ll         # all constant types
  casts.ll             # all cast kinds
  binary_ops.ll        # arithmetic, bitwise, comparisons
```

### Realistic C sources

```
tests/fixtures/c_sources/
  basic/
    pointers_basic.c   # &x, *p, p->field
    structs.c          # struct access, nested
    arrays.c           # indexing, pointer arithmetic
    globals.c          # global vars, static locals

  memory/
    heap_alloc.c       # malloc, calloc, realloc, free
    stack_arrays.c     # VLAs, alloca

  control/
    control_flow.c     # loops, switch, nested ifs
    func_pointers.c    # indirect calls, vtable patterns
    interprocedural.c  # call chains, recursion

  taint/
    taint_argv_system.c      # argv → system()
    taint_getenv_fopen.c     # getenv() → fopen()
    taint_read_write.c       # read() → write()
    taint_sanitized.c        # flow blocked by sanitizer
    taint_format_string.c    # argv → printf format

  edge_cases/
    varargs.c          # printf, va_list
    setjmp_longjmp.c   # non-local jumps
    volatile.c         # volatile qualifier
    bitfields.c        # struct bitfields
    unions.c           # union types

  build.sh             # Compile all .c → .bc
```

---

## Implementation Phases

| Phase | Deliverable | Tests First | Depends On |
|-------|-------------|-------------|------------|
| **2a** | LLVM adapter skeleton | Feature flags work, inkwell links | — |
| **2b** | Basic module ingestion | Function names extracted from minimal.ll | 2a |
| **2c** | Instruction mapping | memory_ops, control_flow, calls mapped | 2b |
| **2d** | Value/ID derivation | Same input twice → identical IDs | 2c |
| **2e** | Intrinsic handling | memcpy mapped, dbg skipped | 2c |
| **2f** | Debug info extraction | Spans populated when present | 2c |
| **2g** | Realistic test suite | All C fixtures ingest | 2c-2f |
| **2h** | Cross-frontend equivalence | AT-EXT-01 passes | 2g |

---

## Dependencies

```toml
# workspace Cargo.toml
[workspace.dependencies]
inkwell = { version = "0.5", default-features = false }

# saf-frontends/Cargo.toml
[features]
default = ["llvm-18"]
llvm-17 = ["inkwell/llvm17-0"]
llvm-18 = ["inkwell/llvm18-0"]

[dependencies]
inkwell = { workspace = true, optional = true }
```

---

## Definition of Done

1. `make test` passes with both `--features llvm-17` and `--features llvm-18`
2. `make lint` clean
3. All `.ll` fixtures produce correct AIR
4. All C fixtures ingest successfully
5. Determinism: same `.bc` parsed twice → byte-identical JSON export
6. AT-EXT-01: equivalent program via LLVM and AIR-JSON → matching analysis results
7. Documentation updated

---

## Acceptance Criteria (from SRS)

- [ ] FR-FE-002: Frontend implements common contract (ingest, fingerprint, features)
- [ ] FR-FE-004: LLVM Bitcode Frontend reads `.bc` files
- [ ] FR-FE-005: Best-effort source mapping metadata (Span, Symbol)
- [ ] FR-AIR-005: Deterministic ID derivation
- [ ] NFR-DET-001: Byte-identical outputs for identical inputs
- [ ] NFR-EXT-001: Analysis engine operates only on AIR
- [ ] AT-EXT-01: Cross-frontend equivalence test passes
