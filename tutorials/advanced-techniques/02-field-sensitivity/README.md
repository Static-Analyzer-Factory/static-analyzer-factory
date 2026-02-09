# Tutorial: Field Sensitivity

## What You'll Learn

- What **field sensitivity** means in pointer analysis
- How SAF tracks individual struct fields separately for precision
- How the **max_depth** parameter controls precision vs. cost for nested structs
- How linked lists behave when depth exceeds the configured limit

## Prerequisites

Complete the setup instructions in the main tutorials README before starting.

## Background: Field Sensitivity

Consider a struct with two pointer fields:

```c
struct Pair {
    int *first;
    int *second;
};

struct Pair pair;
pair.first = &a;
pair.second = &b;
```

A **field-insensitive** analysis treats the struct as a single object, merging
all fields:

```
points_to(pair) = {a, b}    // imprecise: pair.first and pair.second are mixed
may_alias(pair.first, pair.second) = true   // spurious alias!
```

A **field-sensitive** analysis tracks each field separately:

```
points_to(pair.first) = {a}     // precise
points_to(pair.second) = {b}    // precise
no_alias(pair.first, pair.second) = true   // correct: different fields
```

Field sensitivity dramatically improves precision for programs that use
structs, classes, and data structures.

## The Program

The tutorial program uses two data structures:

### Struct Pair (flat fields)

```c
struct Pair pair;
pair.first = &a;     // field-sensitive: only first -> a
pair.second = &b;    // field-sensitive: only second -> b
```

With field sensitivity, PTA correctly determines that `pair.first` and
`pair.second` do not alias. Without it, both would appear to alias through
the merged `pair` object.

### Linked List (nested fields)

```c
struct Node { int data; struct Node *next; };

n1->next = n2;
n2->next = n3;
n3->next = NULL;
```

Linked lists test the **depth** of field sensitivity. Each `->next` access
adds one level of indirection:

- Depth 1: `n1->next` is tracked precisely
- Depth 2: `n1->next->next` is tracked precisely
- Depth 3+: Beyond `max_depth`, fields may be merged

## The `max_depth` Parameter

SAF's PTA is configured with `FieldSensitivity::StructFields { max_depth: N }`:

```rust
let pta_config = PtaConfig {
    field_sensitivity: FieldSensitivity::StructFields { max_depth: 3 },
    // ...
};
```

| max_depth | Behavior |
|-----------|----------|
| 0         | Field-insensitive (all fields merged) |
| 1         | Top-level fields only |
| 2         | Two levels of nesting (default) |
| 3+        | Deeper nesting, more precise but slower |

For most programs, `max_depth: 2` provides a good balance. Programs with
deep data structures (trees, nested linked lists) may benefit from
`max_depth: 3` at the cost of increased memory and time.

## What Happens Beyond max_depth

When a struct field access exceeds `max_depth`, the analysis "collapses" the
remaining levels into a single abstract location. This means:

- All nodes beyond the depth limit may appear to alias each other
- Points-to sets for deep accesses may be larger than necessary
- The analysis remains **sound** (no missed aliases) but loses precision

For the linked list example with `max_depth: 2`:

```
n1->next = n2        // depth 1: precise
n2->next = n3        // depth 2: precise
n3->next = NULL      // depth 3: may be collapsed into depth-2 node
```

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
PTA Statistics:
  Values tracked: <N>
  Abstract locations: <N>
  Points-to entries: <N>

Points-to sets:
  0x1234...  -> 1 loc(s)
  0x5678...  -> 1 loc(s)
  ...

Values pointing to multiple locations (<N>):
  ...
```

Points-to entries with exactly 1 location indicate precise field-sensitive
tracking. Entries with multiple locations may indicate merged fields (either
due to actual aliasing or depth-limit collapse).

## Understanding the Code

```python
proj = Project.open(str(llvm_ir))
pta = proj.pta_result()

# Export the full PTA map
export = pta.export()
pts_data = export.get("points_to", {})

# Find values with multiple targets (potential merging)
multi_target = {k: v for k, v in pts_data.items()
                if isinstance(v, list) and len(v) > 1}
```

Key points:

- Each struct field access in the LLVM IR generates a GEP instruction,
  which PTA interprets as a field access to track separately.
- The `export()` dict maps value IDs to lists of location IDs. Single-element
  lists indicate precise tracking; multi-element lists may indicate field
  merging or genuine multi-target pointers.
- `malloc` calls create fresh abstract locations. Each `malloc` site gets a
  unique location ID.

## Using the Rust API

In Rust, you can directly control the field sensitivity depth:

```rust
// Try different depths to see the effect on precision
let pta_config = PtaConfig {
    enabled: true,
    field_sensitivity: FieldSensitivity::StructFields { max_depth: 3 },
    max_objects: 100_000,
    max_iterations: 1_000_000,
};

let mut pta_ctx = PtaContext::new(pta_config);
let pta_analysis = pta_ctx.analyze(&module);
let pta_result = PtaResult::new(
    pta_analysis.pts,
    &pta_analysis.factory,
    pta_analysis.diagnostics,
);

// Count how many values have multiple targets
let multi_count = pta_result.export()
    .points_to.values()
    .filter(|v| v.len() > 1)
    .count();
```

Try running with different `max_depth` values (1, 2, 3) to see how precision
changes. With depth 1, more values will have multiple targets due to field
merging. With depth 3, the linked list should be tracked more precisely.

## Next Steps

Continue to the **Flow-Sensitive PTA** tutorial to see how flow-sensitive
analysis tracks points-to sets per program point for even higher precision.
