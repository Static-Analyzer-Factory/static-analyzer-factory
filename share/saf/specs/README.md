# SAF Function Specifications

This directory contains function specifications for external/library functions.
These specs enable SAF's analyses to precisely model function behavior without
requiring source code.

## Directory Structure

```
specs/
  libc/
    alloc.yaml     # Memory allocation (malloc, free, etc.)
    string.yaml    # String operations (strlen, strcpy, etc.)
    stdio.yaml     # Standard I/O (printf, fopen, etc.)
    stdlib.yaml    # Standard library (atoi, exit, etc.)
  posix/
    unistd.yaml    # POSIX I/O and process (read, write, fork, etc.)
    socket.yaml    # Socket operations (socket, recv, send, etc.)
  taint/
    sources.yaml   # Taint sources (getenv, recv, etc.)
    sinks.yaml     # Taint sinks (system, exec, etc.)
    sanitizers.yaml # Sanitizers (escape functions, validators)
```

## Spec Format

Specs are YAML files with version "1.0":

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
```

## Discovery Order

SAF loads specs from multiple paths (later overrides earlier per-function):

1. `<binary>/../share/saf/specs/*.yaml` - Shipped defaults (this directory)
2. `~/.saf/specs/*.yaml` - User global overrides
3. `./saf-specs/*.yaml` - Project-local specs
4. `$SAF_SPECS_PATH/*.yaml` - Explicit override path

## Name Matching

- **Exact match** (default): `name: malloc`
- **Glob pattern**: `name: "glob:str*"`
- **Regex pattern**: `name: "regex:^mem(cpy|set|move)$"`

## Key Fields

### Function-Level

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Function name or pattern (required) |
| `role` | enum | allocator, deallocator, source, sink, sanitizer, etc. |
| `pure` | bool | No side effects, result depends only on inputs |
| `noreturn` | bool | Function never returns (exit, abort, etc.) |
| `disabled` | bool | Suppress an inherited spec |

### Parameter Fields (`params[]`)

| Field | Type | Description |
|-------|------|-------------|
| `index` | u32 | Parameter position (0-indexed) |
| `modifies` | bool | Parameter's pointee is modified |
| `reads` | bool | Parameter's pointee is read |
| `nullness` | enum | required_nonnull, nullable |
| `escapes` | bool | Pointer may escape function scope |
| `callback` | bool | Function pointer that will be called |
| `semantic` | string | allocation_size, byte_count, etc. |
| `tainted` | bool | Becomes tainted after call (for sources) |

### Return Fields (`returns`)

| Field | Type | Description |
|-------|------|-------------|
| `nullness` | enum | not_null, maybe_null |
| `pointer` | enum | fresh_heap, fresh_stack, unknown |
| `aliases` | string | "param.0" if return aliases a parameter |
| `tainted` | bool | Return value is tainted (for sources) |

### Taint Fields (`taint`)

```yaml
taint:
  propagates:
    - from: param.1
      to: [param.0, return]
```

## Adding Custom Specs

Create a `./saf-specs/` directory in your project root:

```bash
mkdir -p saf-specs
cat > saf-specs/mylib.yaml << 'EOF'
version: "1.0"
specs:
  - name: my_alloc
    role: allocator
    returns:
      pointer: fresh_heap
      nullness: maybe_null
EOF
```

## Validation

Use the CLI to validate spec files:

```bash
saf specs validate ./saf-specs/
saf specs lookup my_alloc
saf specs list --verbose
```
