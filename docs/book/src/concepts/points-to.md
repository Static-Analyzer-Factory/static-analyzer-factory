# Points-To Analysis

**Points-to analysis** (PTA) determines what memory locations each pointer in a
program may refer to. This is fundamental to understanding aliasing, resolving
indirect calls, and building accurate data flow graphs.

## The Problem

Consider:

```c
int x = 10, y = 20;
int *p = &x;
int *q = p;     // q also points to x
*q = 30;        // modifies x through q
```

Without PTA, an analysis cannot determine that modifying `*q` affects `x`.
Points-to analysis computes that both `p` and `q` point to the same object `x`,
enabling the analysis to track data flow through memory.

## Andersen's Analysis

SAF implements **Andersen-style inclusion-based** points-to analysis. This is a
whole-program, flow-insensitive, context-insensitive analysis that computes a
conservative approximation of the points-to sets.

### Constraint Types

PTA works by extracting constraints from the program and solving them:

| Constraint | Meaning | Example |
|-----------|---------|---------|
| **Addr** | `p` points to object `o` | `p = &x` |
| **Copy** | `p` points to everything `q` points to | `p = q` |
| **Load** | `p` points to what `*q` points to | `p = *q` |
| **Store** | `*p` points to what `q` points to | `*p = q` |
| **GEP** | `p` points to a field of what `q` points to | `p = &q->field` |

### Solving

The solver iterates until a fixed point:

1. Initialize points-to sets from `Addr` constraints
2. Process `Copy` constraints: propagate points-to sets
3. Process `Load`/`Store` constraints: follow indirections
4. Repeat until no points-to set changes

### Field Sensitivity

SAF supports field-sensitive analysis, distinguishing different fields of a
struct:

```c
struct Pair { int *a; int *b; };
struct Pair s;
s.a = &x;   // s.a -> x
s.b = &y;   // s.b -> y (distinct from s.a)
```

Without field sensitivity, `s.a` and `s.b` would be conflated, causing false
aliasing.

## Querying Points-To Results

### Python SDK

```python
from saf import Project

proj = Project.open("program.ll")
q = proj.query()

# What does pointer p point to? (pass the hex ID of the pointer value)
pts = q.points_to("0x00000000000000000000000000000001")
print(f"p points to: {pts}")

# Do p and q alias? (pass hex IDs of both pointer values)
alias = q.may_alias("0x00000000000000000000000000000001",
                     "0x00000000000000000000000000000002")
print(f"p and q may alias: {alias}")
```

### Export Format

```python
pta = proj.pta_result().export()
# Returns: {"points_to": [{"value": "0x...", "locations": ["0x...", ...]}, ...]}
```

## Impact on Other Analyses

PTA results feed into multiple downstream analyses:

| Analysis | How PTA Helps |
|----------|---------------|
| **Call graph** | Resolves indirect call targets (function pointers) |
| **ValueFlow** | Determines which stores affect which loads |
| **Taint analysis** | Tracks taint through pointer indirections |
| **Alias analysis** | Identifies when two pointers may refer to the same memory |

Without PTA, these analyses must conservatively assume that any pointer could
point to any object, leading to excessive false positives.

## Configuration

PTA behavior can be tuned via configuration:

```json
{
  "pta": {
    "enabled": true,
    "field_sensitivity": "struct_fields",
    "max_objects": 100000
  }
}
```

- `field_sensitivity`: `"none"` or `"struct_fields"`
- `max_objects`: Safety cap; when exceeded, analysis collapses to a conservative
  approximation using `unknown_mem`

<div class="saf-widget">
  <iframe src="../../playground/?embed=true&split=true&example=pointer_alias&graph=pta" loading="lazy"></iframe>
</div>

## Next Steps

- [Value Flow](value-flow.md) -- How PTA enables precise data flow tracking
- [Taint Analysis](taint-analysis.md) -- Using PTA-aware taint propagation
