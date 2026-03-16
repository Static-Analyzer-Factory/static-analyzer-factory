# Analysis IR (AIR)

AIR (Analysis Intermediate Representation) is SAF's canonical, frontend-agnostic
intermediate representation. All analysis passes operate on AIR, never on
frontend-specific types like LLVM IR or Clang ASTs.

## Why a Separate IR?

SAF supports multiple frontends (LLVM bitcode, AIR JSON, and potentially
source-level frontends in the future). Rather than coupling analysis algorithms
to any specific input format, SAF defines AIR as a common target:

```
LLVM bitcode (.bc/.ll)  -->  LLVM Frontend  -->  AIR
AIR JSON (.air.json)    -->  JSON Frontend  -->  AIR
(future: Clang AST)     -->  AST Frontend   -->  AIR
                                                   |
                                                   v
                                              Analysis passes
                                         (CFG, PTA, ValueFlow, ...)
```

This design means adding a new frontend requires only implementing the mapping
to AIR -- no changes to analysis algorithms.

## Structure

An AIR module contains:

| Entity | Description |
|--------|-------------|
| **Module** | Top-level container with a fingerprint and metadata |
| **Functions** | Named functions with parameters, return types, and basic blocks |
| **Basic Blocks** | Sequences of instructions with a terminator |
| **Instructions** | Individual operations (alloc, load, store, call, etc.) |
| **Values** | SSA registers, function parameters, constants, and globals |
| **Objects** | Memory objects (stack allocas, heap allocations, globals) |

## Operations

AIR supports the following operation types:

| Category | Operations |
|----------|-----------|
| **Allocation** | `Alloca` (stack), `Global`, `HeapAlloc` (malloc/calloc) |
| **Memory** | `Load`, `Store`, `GEP` (field/element access), `Memcpy`, `Memset` |
| **Control** | `Br` (branch), `Switch`, `Ret` (return) |
| **SSA** | `Phi`, `Select` |
| **Calls** | `CallDirect`, `CallIndirect` |
| **Transforms** | `Cast`, `BinaryOp` (arithmetic, bitwise) |

## Deterministic IDs

Every AIR entity has a deterministic `u128` ID derived from BLAKE3 hashes.
IDs are serialized as `0x` followed by 32 lowercase hex characters (e.g.,
`0x1a2b3c4d5e6f...`).

The ID derivation hierarchy:

```
ModuleFingerprint = hash(FrontendId, input_fingerprint_bytes)
  FunctionId = hash(ModuleFingerprint, "fn", function_key)
    BlockId = hash(FunctionId, "bb", block_index)
      InstId = hash(BlockId, "inst", inst_index, opcode_tag)
        ValueId = derived from instruction results, args, globals, constants
    ObjId = derived from allocas, heap allocators, globals
      LocId = hash(ObjId, "loc", field_path)
```

This means identical inputs always produce identical IDs, regardless of when
or where the analysis runs. Debug information does not affect structural IDs
by default.

## Source Metadata

AIR instructions can carry optional source-level metadata:

- **Span**: File, line, column, byte offsets for source location
- **Symbol**: Display name, mangled name, namespace path
- **Type representation**: Frontend-specific type string

This metadata enables source-level error reporting without coupling the analysis
to any particular frontend.

<div class="saf-widget">
  <iframe src="../../playground/?embed=true&split=true&example=taint_flow&graph=cfg" loading="lazy"></iframe>
</div>

## Next Steps

- [Control Flow Graphs](cfg-icfg.md) -- How AIR is organized into CFGs
- [Points-To Analysis](points-to.md) -- How AIR objects are analyzed for aliasing
