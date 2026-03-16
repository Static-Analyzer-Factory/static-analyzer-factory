# Function Specs Design

## Overview

A pluggable function specification system for SAF that enables precise modeling of external/library function behavior across all analyses (PTA, nullness, taint, interval, escape, etc.).

## Goals

1. **Model external functions** — Provide semantic knowledge for libc, POSIX, and system APIs
2. **Enable taint analysis** — Define sources, sinks, sanitizers, and propagation rules
3. **Support custom APIs** — Allow users to define specs for their own libraries
4. **Pluggable design** — Specs loaded from external YAML files, not hardcoded

## Spec Format

### File Structure

YAML format with layered schema (minimal required, detailed optional):

```yaml
version: "1.0"
specs:
  # Minimal: just a role
  - name: printf
    role: sink

  # Medium: role + key properties
  - name: malloc
    role: allocator
    returns:
      nullness: maybe_null
      pointer: fresh_heap
    params:
      - index: 0
        semantic: allocation_size

  # Detailed: full behavioral spec
  - name: strcpy
    params:
      - index: 0
        name: dst
        modifies: true
        size_from: param.1_strlen_plus_1
        nullness: required_nonnull
      - index: 1
        name: src
        reads: true
        nullness: required_nonnull
    returns:
      aliases: param.0
      nullness: not_null
    taint:
      propagates:
        - from: param.1
          to: [param.0, return]

  # Pattern matching
  - name: "glob:str*"
    role: string_operation
    params:
      - index: 0
        nullness: required_nonnull
```

### Name Matching

- **Exact match** (default): `name: malloc`
- **Glob pattern**: `name: "glob:str*"`
- **Regex pattern**: `name: "regex:^mem(cpy|set|move)$"`

### Schema Fields

All fields are optional except `name`:

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Function name or pattern |
| `role` | enum | allocator, deallocator, source, sink, sanitizer, pure |
| `pure` | bool | No side effects, result depends only on inputs |
| `noreturn` | bool | Function never returns (exit, abort, longjmp) |
| `disabled` | bool | Suppress an inherited spec |
| `params[]` | array | Per-parameter specifications |
| `returns` | object | Return value specification |
| `taint` | object | Taint propagation rules |

#### Parameter Fields

| Field | Type | Description |
|-------|------|-------------|
| `index` | u32 | Parameter position (0-indexed) |
| `name` | string | Optional human-readable name |
| `modifies` | bool | Parameter's pointee is modified |
| `reads` | bool | Parameter's pointee is read |
| `nullness` | enum | required_nonnull, nullable |
| `escapes` | bool | Pointer may escape function scope |
| `callback` | bool | Function pointer that will be called |
| `semantic` | string | allocation_size, byte_count, element_count, etc. |
| `size_from` | string | Bounds relation (e.g., "param.2") |
| `tainted` | bool | Becomes tainted after call (for sources) |

#### Return Fields

| Field | Type | Description |
|-------|------|-------------|
| `nullness` | enum | not_null, maybe_null |
| `pointer` | enum | fresh_heap, fresh_stack, aliases_param.N, unknown |
| `aliases` | string | "param.0" if return aliases a parameter |
| `interval` | array | [min, max] numeric range (use "infinity" for unbounded) |
| `tainted` | bool | Return value is tainted (for sources) |

#### Taint Fields

```yaml
taint:
  propagates:
    - from: param.1      # or "return"
      to: [param.0, return]
```

## Discovery & Loading

### Search Order (later overrides earlier, per-function)

```
1. ./saf-specs/*.yaml                    # project local (highest priority)
2. ~/.saf/specs/*.yaml                   # user global
3. <binary>/../share/saf/specs/*.yaml    # shipped defaults
4. $SAF_SPECS_PATH/*.yaml                # explicit override (if set)
```

### Merge Semantics

- All YAML files in each directory are loaded and merged
- Later directories override earlier ones **per function name**
- Within a directory, files loaded alphabetically (deterministic)
- Exact matches take precedence over glob/regex patterns
- Use `disabled: true` to suppress an inherited spec

### Binary-Relative Path Resolution

```rust
let binary_path = std::env::current_exe()?;
let specs_dir = binary_path
    .parent()           // /usr/local/bin/
    .parent()           // /usr/local/
    .join("share/saf/specs");
```

Benefits:
- `cargo install saf` → specs at `~/.cargo/share/saf/specs/`
- Docker → specs at `/usr/local/share/saf/specs/`
- No environment variables required for normal operation

## Analysis Integration

### Consumer Robustness

All spec fields are optional. Analyses use safe defaults when fields are missing:

| Field | Default | Rationale |
|-------|---------|-----------|
| `pure` | `false` | Assume side effects |
| `noreturn` | `false` | Assume function returns |
| `escapes` | `true` | Assume argument may escape |
| `modifies` | unspecified | Skip, no false claims |
| `reads` | unspecified | Skip, no false claims |
| `nullness` (param) | no requirement | Don't report false null errors |
| `nullness` (return) | `maybe_null` | Assume may be null |
| `interval` | `[-∞, +∞]` | Unbounded |
| `size_from` | `None` | Skip bounds checking |
| `callback` | `false` | Don't assume called |
| `taint.*` | `None` | No taint propagation assumed |

### Missing Spec Handling

When a function has no spec:
1. Use conservative assumptions (sound)
2. Emit warning: `warning: no spec for 'custom_alloc', using conservative assumptions`
3. Suppression via config or `--quiet-spec-warnings`

### PTA Integration

```rust
fn extract_call_constraints(..., specs: &SpecRegistry) {
    if let Some(spec) = specs.lookup(&callee_name) {
        match spec.returns.pointer {
            Pointer::FreshHeap => {
                constraints.add(AllocConstraint { dst: ret_val, site: call_site });
            }
            Pointer::AliasesParam(i) => {
                constraints.add(CopyConstraint { dst: ret_val, src: args[i] });
            }
        }
        for param in &spec.params {
            if param.modifies {
                constraints.add(StoreConstraint { ptr: args[param.index], ... });
            }
        }
        return;
    }
    // No spec: conservative
}
```

### Nullness Integration

```rust
fn transfer_call(..., specs: &SpecRegistry) -> NullnessState {
    if let Some(spec) = specs.lookup(&callee_name) {
        // Check preconditions
        for param in &spec.params {
            if param.nullness == RequiredNonnull {
                if state.get(args[param.index]).may_be_null() {
                    report_potential_null_arg(call_site, param.index);
                }
            }
        }
        // Apply return nullness
        match spec.returns.nullness {
            NotNull => state.set(ret_val, Nullness::NotNull),
            MaybeNull => state.set(ret_val, Nullness::MaybeNull),
        }
        return state;
    }
    // No spec: conservative
}
```

### Taint Integration

```rust
fn flow_call(..., specs: &SpecRegistry) -> TaintFacts {
    if let Some(spec) = specs.lookup(&callee_name) {
        if spec.role == Role::Source {
            facts.add_taint(ret_val);
        }
        if spec.role == Role::Sink {
            for (i, arg) in args.iter().enumerate() {
                if facts.is_tainted(*arg) {
                    report_taint_reaches_sink(call_site, i);
                }
            }
        }
        for prop in &spec.taint.propagates {
            if facts.is_tainted(resolve(prop.from, args, ret_val)) {
                for target in &prop.to {
                    facts.add_taint(resolve(target, args, ret_val));
                }
            }
        }
    }
}
```

## Shipped Default Specs

### Directory Structure

```
share/saf/specs/
  libc/
    alloc.yaml      # malloc, calloc, realloc, free
    string.yaml     # strlen, strcpy, strcat, strcmp, memcpy, memset
    stdio.yaml      # printf, fopen, fclose, fread, fwrite
    stdlib.yaml     # atoi, getenv, system, exit, abort
  posix/
    unistd.yaml     # read, write, open, close, fork, exec*
    socket.yaml     # socket, bind, listen, accept, recv, send
    mmap.yaml       # mmap, munmap, mprotect
  taint/
    sources.yaml    # getenv, read, recv, fgets, scanf
    sinks.yaml      # system, exec*, SQL APIs
    sanitizers.yaml # escape functions, validators
```

### Example: libc/alloc.yaml

```yaml
version: "1.0"
specs:
  - name: malloc
    role: allocator
    returns:
      pointer: fresh_heap
      nullness: maybe_null
    params:
      - index: 0
        semantic: allocation_size

  - name: calloc
    role: allocator
    returns:
      pointer: fresh_heap
      nullness: maybe_null
    params:
      - index: 0
        semantic: element_count
      - index: 1
        semantic: element_size

  - name: realloc
    role: reallocator
    returns:
      pointer: fresh_heap
      nullness: maybe_null
    params:
      - index: 0
        reads: true
        semantic: old_pointer
      - index: 1
        semantic: new_size

  - name: free
    role: deallocator
    params:
      - index: 0
        semantic: freed_pointer
        nullness: nullable  # free(NULL) is valid
```

### Example: taint/sources.yaml

```yaml
version: "1.0"
specs:
  - name: getenv
    role: source
    returns:
      nullness: maybe_null
      tainted: true
    params:
      - index: 0
        reads: true

  - name: "glob:recv*"
    role: source
    returns:
      tainted: true
    params:
      - index: 1
        modifies: true
        tainted: true

  - name: fgets
    role: source
    returns:
      nullness: maybe_null
    params:
      - index: 0
        modifies: true
        tainted: true
```

## CLI Interface

```bash
# Use default spec discovery
saf analyze program.bc

# Add extra spec paths
saf analyze --specs ./my-specs.yaml program.bc
saf analyze --specs ./proj/ --specs ./extra.yaml program.bc

# Disable spec warnings
saf analyze --quiet-spec-warnings program.bc

# List loaded specs
saf specs list
saf specs list --verbose

# Validate spec files
saf specs validate ./my-specs.yaml

# Show which spec matches a function
saf specs lookup malloc
saf specs lookup my_custom_alloc
```

## Validation & Error Handling

### Load-Time Validation

```
error[E0001]: invalid nullness value 'invalid_value'
  --> ./saf-specs/custom.yaml:15:17
   |
15 |       nullness: invalid_value
   |                 ^^^^^^^^^^^^^ expected one of: not_null, maybe_null, required_nonnull, nullable

warning[W0001]: duplicate spec for 'malloc'
  --> ./saf-specs/custom.yaml:30:3
   |
30 |   - name: malloc
   |     ^^^^^^^^^^^^ this definition shadows the one at line 10
```

### Runtime Warnings

```
warning[W0100]: no spec for 'custom_alloc'
  --> src/memory.c:42:10
   |
42 |     void *p = custom_alloc(size);
   |               ^^^^^^^^^^^^ using conservative assumptions
```

### Warning Suppression

```yaml
# In ./saf-specs/config.yaml
warnings:
  missing_specs:
    level: ignore  # ignore | warn | error
    except: ["my_*"]
  suppress:
    - custom_alloc
    - "glob:internal_*"
```

## Future Integrations

Schema includes fields now, analysis integration deferred:

| Integration | Schema Fields | Priority | Notes |
|-------------|---------------|----------|-------|
| Mod/Ref Analysis | `modifies`, `reads` | High | Phase 2 (PTA) |
| Interval Analysis | `returns.interval`, `pure` | Medium | AbsInt refactor |
| Escape Analysis | `escapes` | Medium | New analysis |
| Purity Analysis | `pure` | Medium | Needs escape analysis |
| Bounds Checking | `size_from` | Low | New analysis |
| Call Graph (callbacks) | `callback` | Low | PTA indirect resolution |
| Termination/CFG | `noreturn` | High | Trivial, Phase 1 |
| Resource Checkers | `role`, `semantic` | High | Existing checker infra |

### Implementation Notes

- **Mod/Ref**: Extend `pta/mod_ref.rs` to read `spec.modifies`/`reads` instead of `conservative()`
- **Interval**: Extend `absint/interval.rs` to use `spec.returns.interval` for externals
- **Escape**: New analysis pass (`absint/escape.rs`) using `spec.escapes`
- **Purity**: Combine `pure` + no escapes + no modifies = cacheable
- **Bounds**: New checker using `size_from` relations for buffer overflow detection
- **Callbacks**: Extend call graph to add edges for `spec.callback` params
- **Noreturn**: Mark noreturn calls as terminators in CFG, prune unreachable successors

## Rust API

```rust
// crates/saf-core/src/spec/mod.rs
pub struct SpecRegistry { ... }
pub struct FunctionSpec { ... }
pub enum Role { Allocator, Deallocator, Source, Sink, Sanitizer, Pure }
pub enum Nullness { NotNull, MaybeNull, RequiredNonnull, Nullable }
pub enum Pointer { FreshHeap, FreshStack, AliasesParam(u32), Unknown }

impl SpecRegistry {
    pub fn load() -> Result<Self, SpecError>;
    pub fn load_from(paths: &[PathBuf]) -> Result<Self, SpecError>;
    pub fn lookup(&self, name: &str) -> Option<&FunctionSpec>;
    pub fn iter(&self) -> impl Iterator<Item = &FunctionSpec>;
    pub fn insert(&mut self, spec: FunctionSpec);
}
```

## Python API

```python
from saf import SpecRegistry, FunctionSpec

specs = SpecRegistry.load()
specs = SpecRegistry.load_from(["./my-specs.yaml"])

spec = specs.lookup("malloc")
if spec:
    print(f"malloc role: {spec.role}")
    print(f"returns null: {spec.returns.nullness}")

# Use in analysis
pta_result = saf.analyze_pta(module, specs=specs)
nullness = saf.analyze_nullness(module, specs=specs)
```
