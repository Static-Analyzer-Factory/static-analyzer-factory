# Advanced Techniques

This category covers SAF's analysis internals and advanced techniques. These tutorials help you understand the underlying algorithms and when to use different analysis configurations.

## Learning Objectives

After completing these tutorials, you will be able to:
- Understand pointer aliasing and Andersen's algorithm
- Configure field sensitivity for struct analysis
- Apply flow-sensitive PTA for temporal precision
- Use context-sensitive PTA for interprocedural precision
- Explore Memory SSA representation
- Navigate the Sparse Value-Flow Graph (SVFG)
- Compare different analysis techniques on the same program

## Tutorials

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Pointer Aliasing](01-pointer-aliasing/) | Intermediate | Andersen PTA basics + indirect calls |
| 02 | [Field Sensitivity](02-field-sensitivity/) | Intermediate | Field-sensitive struct analysis |
| 03 | [Flow-Sensitive PTA](03-flow-sensitive-pta/) | Intermediate | Per-program-point precision |
| 04 | [Context-Sensitive](04-context-sensitive/) | Advanced | k-CFA + virtual dispatch + CG refinement |
| 05 | [Memory SSA](05-memory-ssa/) | Advanced | Memory SSA internals |
| 06 | [SVFG Exploration](06-svfg-exploration/) | Advanced | Sparse value-flow graph |
| 07 | [Analysis Comparison](07-analysis-comparison/) | Advanced | All techniques on one program |

## Difficulty Progression

- **Intermediate (01-03)**: Single analysis technique with clear examples
- **Advanced (04-07)**: Multiple techniques or complex internal structures

## Prerequisites

- Complete [Getting Started](../getting-started/) tutorials first
- Familiarity with pointer analysis concepts helps

## Key Concepts

### Pointer Analysis Hierarchy
```
Context-Insensitive (CI) Andersen
    ↓
Field-Sensitive Andersen
    ↓
Flow-Sensitive PTA (SFS)
    ↓
Context-Sensitive PTA (k-CFA)
```

Each level adds precision at the cost of analysis time.

### When to Use Each Technique

| Technique | Use Case | Trade-off |
|-----------|----------|-----------|
| CI Andersen | Quick alias queries | Fast, less precise |
| Field-Sensitive | Struct member tracking | Moderate cost, better for structs |
| Flow-Sensitive | Pointer reassignment | Higher cost, temporal precision |
| Context-Sensitive | Wrapper functions, factories | Highest cost, best precision |

### Memory SSA
Memory SSA extends SSA to memory operations, enabling sparse analysis:
- `MemDef`: Memory write creating a new version
- `MemUse`: Memory read consuming a version
- `MemPhi`: Merge point for memory versions

### SVFG (Sparse Value-Flow Graph)
Unifies direct (SSA) and indirect (memory) value flow:
- Direct edges: SSA def-use chains
- Indirect edges: Store → Load through aliasing

## Related Categories

- [Memory Safety](../memory-safety/) applies these techniques to find real bugs
- [Buffer Overflow](../buffer-overflow/) uses PTA for complex overflow patterns
