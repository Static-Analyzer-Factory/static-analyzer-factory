# Spec Authoring Guide

Reference for authoring checker specifications in SAF: YAML function specs, `CheckerSpec` composition, `check_custom()`, `typestate_custom()`, and `ResourceTable` extension.

## 1. YAML Function Spec Format

Spec files use version `"1.0"`. Top-level: `version: "1.0"` and `specs: [...]`.

### FunctionSpec fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Exact name, or `glob:<pattern>` / `regex:<pattern>` prefix |
| `role` | enum | no | `allocator`, `reallocator`, `deallocator`, `source`, `sink`, `sanitizer`, `string_operation`, `io`, or custom string |
| `pure` | bool | no | No side effects |
| `noreturn` | bool | no | Never returns (e.g., `exit`) |
| `disabled` | bool | no | Suppress an inherited spec |
| `params` | list | no | Per-parameter specs (see below) |
| `returns` | object | no | Return value spec (see below) |
| `taint` | object | no | Taint propagation rules |

### ParamSpec fields

| Field | Type | Description |
|-------|------|-------------|
| `index` | u32 | Parameter position (0-indexed, required) |
| `name` | string | Human-readable name |
| `modifies` | bool | Pointee is modified |
| `reads` | bool | Pointee is read |
| `nullness` | enum | `not_null`, `maybe_null`, `required_nonnull`, `nullable` |
| `escapes` | bool | May escape function scope (default: true) |
| `callback` | bool | Is a function pointer that will be called |
| `semantic` | string | E.g., `allocation_size`, `byte_count` |
| `size_from` | string | Size bound relation (e.g., `"param.2"`) |
| `tainted` | bool | Becomes tainted after call (for sources) |

### ReturnSpec fields

| Field | Type | Description |
|-------|------|-------------|
| `nullness` | enum | `not_null`, `maybe_null`, `required_nonnull`, `nullable` |
| `pointer` | enum | `fresh_heap`, `fresh_stack`, `unknown`, `static_singleton` |
| `aliases` | string | Return aliases a parameter: `"param.0"`, `"param.1"`, etc. |
| `interval` | [i64, i64] | Numeric return interval [min, max] |
| `tainted` | bool | Return value is tainted |

### TaintSpec

```yaml
taint:
  propagates:
    - from: param.1        # TaintLocation: "param.N" or "return"
      to: [param.0, return]
```

### Complete example: database library

```yaml
version: "1.0"
specs:
  - name: db_connect
    role: allocator
    returns: { pointer: fresh_heap, nullness: maybe_null }
    params:
      - { index: 0, name: conn_str, reads: true, nullness: required_nonnull }

  - name: db_query
    role: source
    params:
      - { index: 0, name: conn, reads: true, nullness: required_nonnull }
      - { index: 1, name: sql, reads: true, nullness: required_nonnull }
    returns: { pointer: fresh_heap, nullness: maybe_null, tainted: true }
    taint:
      propagates:
        - from: param.1
          to: [return]

  - name: db_disconnect
    role: deallocator
    params:
      - { index: 0, modifies: true, nullness: required_nonnull }

  - name: db_escape
    role: sanitizer
    params: [{ index: 0, reads: true }]
    returns: { aliases: param.0, nullness: not_null }

  - name: "glob:db_fetch_*"
    role: io
    params: [{ index: 0, reads: true, nullness: required_nonnull }]
```

## 2. Spec Discovery Paths

The `SpecRegistry` loads specs in this order (later overrides earlier per-function):

1. `<binary>/../share/saf/specs/*.yaml` -- shipped defaults
2. `~/.saf/specs/*.yaml` -- user global
3. `./saf-specs/*.yaml` -- project local
4. `./share/saf/specs/*.yaml` -- workspace share directory
5. `$SAF_SPECS_PATH/*.yaml` -- explicit override (colon-separated)

Exact-name specs take priority over pattern-based specs. Within exact matches, later files merge into earlier specs (field-level override). Use `disabled: true` to suppress.

## 3. CheckerSpec Composition (Rust)

Source: `crates/saf-analysis/src/checkers/spec.rs`

```rust
pub struct CheckerSpec {
    pub name: String,
    pub description: String,
    pub cwe: Option<u32>,
    pub severity: Severity,     // Info | Warning | Error | Critical
    pub mode: ReachabilityMode,
    pub sources: Vec<SitePattern>,
    pub sinks: Vec<SitePattern>,
    pub sanitizers: Vec<SitePattern>,
}
```

### ReachabilityMode

| Mode | Use case | Reports when... |
|------|----------|-----------------|
| `MayReach` | UAF, null-deref | Source reaches sink on SOME path without sanitizer |
| `MustNotReach` | Resource leak | Source does NOT reach sanitizer on ALL paths before exit |
| `MultiReach` | Double-free | Source reaches 2+ distinct sink nodes |
| `NeverReachSink` | Memory leak | Source does NOT reach any sink on any path |

### SitePattern variants

`Role { role, match_return }`, `FunctionName { name, match_return }`, `FunctionExit`, `AnyUseOf`, `AllocaInst`, `LoadDeref`, `StoreDeref`, `GepDeref`, `NullConstant`, `DirectNullDeref`, `NullCheckBranch`, `CustomPredicate { name }`.

### Example: database connection leak checker

```rust
use saf_analysis::checkers::{CheckerSpec, ReachabilityMode, Severity, SitePattern};

pub fn db_connection_leak() -> CheckerSpec {
    CheckerSpec {
        name: "db-connection-leak".to_string(),
        description: "Database connection opened but not closed".to_string(),
        cwe: Some(772),
        severity: Severity::Warning,
        mode: ReachabilityMode::MustNotReach,
        sources: vec![SitePattern::FunctionName {
            name: "db_connect".to_string(), match_return: true,
        }],
        sinks: vec![SitePattern::FunctionExit],
        sanitizers: vec![SitePattern::FunctionName {
            name: "db_disconnect".to_string(), match_return: false,
        }],
    }
}
```

Register by adding to `builtin_checkers()` and `builtin_checker_names()` in `spec.rs`.

## 4. Python check_custom() API

```python
project.check_custom(
    name: str,
    *,
    mode: str = "must_not_reach",        # "may_reach" | "must_not_reach" | "never_reach_sink"
    source_role: str = "allocator",       # ResourceRole string
    source_match_return: bool = True,     # True = return value, False = first arg
    sink_role: str | None = None,         # ResourceRole string, or None
    sink_is_exit: bool = True,            # True = sinks are function exits
    sanitizer_role: str | None = None,
    sanitizer_match_return: bool = False,
    cwe: int | None = None,
    severity: str = "warning",            # "info" | "warning" | "error" | "critical"
) -> list[CheckerFinding]
```

Role strings: `"allocator"`, `"deallocator"`, `"reallocator"`, `"acquire"`, `"release"`, `"lock"`, `"unlock"`, `"null_source"`, `"dereference"`.

### Example: file descriptor leak

```python
import saf

project = saf.Project("program.ll")
findings = project.check_custom(
    "fd-leak",
    mode="must_not_reach",
    source_role="acquire",
    source_match_return=True,
    sink_is_exit=True,
    sanitizer_role="release",
    sanitizer_match_return=False,
    cwe=775, severity="warning",
)
for f in findings:
    print(f"[{f.severity}] {f.message} (source={f.source}, sink={f.sink})")
```

### CheckerFinding properties

`checker` (str), `severity` (str), `cwe` (int|None), `message` (str), `source` (hex str), `sink` (hex str), `trace` (list[str]), `sink_traces` (list[dict] for MultiReach).

## 5. Python typestate_custom() API

```python
spec = saf.TypestateSpec(
    name="db_connection",
    states=["uninit", "connected", "disconnected", "error"],
    initial_state="connected",              # State when resource is created
    error_states=["error"],                 # States indicating a bug
    accepting_states=["uninit", "disconnected"],  # OK at program exit
    transitions=[                           # (from_state, call_name, to_state)
        ("connected", "db_query", "connected"),
        ("connected", "db_execute", "connected"),
        ("connected", "db_disconnect", "disconnected"),
        ("disconnected", "db_query", "error"),       # use-after-disconnect
        ("disconnected", "db_disconnect", "error"),   # double-disconnect
        ("error", "db_query", "error"),               # absorbing
        ("error", "db_disconnect", "error"),
    ],
    constructors=["db_connect"],            # Functions that create resources
)
result = project.typestate_custom(spec)
```

The `call_name` in transitions supports glob patterns (e.g., `"db_fetch_*"`). Built-in specs: `project.typestate("file_io")`, `"mutex_lock"`, `"memory_alloc"`.

### Finding types

- **`error_state`**: Resource reached a bug state (e.g., double-close)
- **`non_accepting_at_exit`**: Resource not in accepting state at exit (e.g., leak)

### TypestateResult methods

```python
result.findings()        # list[TypestateFinding] -- all violations
result.error_findings()  # error-state violations only
result.leak_findings()   # non-accepting-at-exit violations only
result.has_findings()    # bool
result.diagnostics()     # dict with IDE solver stats
```

### TypestateFinding properties

`resource` (hex str), `state` (str), `inst` (hex str), `kind` (`"error_state"` | `"non_accepting_at_exit"`), `spec_name` (str).

### Complete usage

```python
result = project.typestate_custom(spec)
for f in result.error_findings():
    print(f"[error] resource={f.resource} state={f.state} at {f.inst}")
for f in result.leak_findings():
    print(f"[leak] resource={f.resource} state={f.state} at {f.inst}")
```

## 6. ResourceTable Extension

Source: `crates/saf-analysis/src/checkers/resource_table.rs`

Maps function names to `ResourceRole` values. Checkers use it to classify SVFG sites.

Roles: `allocator`, `deallocator`, `reallocator`, `acquire`, `release`, `lock`, `unlock`, `null_source`, `dereference`.

### Python API

```python
table = project.resource_table()  # Includes built-in C/C++/POSIX entries
table.add("pool_alloc", "allocator")
table.add("pool_alloc", "null_source")  # Multiple roles per function
table.add("pool_free", "deallocator")
table.has_role("pool_alloc", "allocator")  # True
table.function_names()  # Sorted list
table.export()          # [{"name": ..., "roles": [...]}, ...]
```

### Rust API

```rust
let mut table = ResourceTable::new();  // Built-in entries included
table.add("pool_alloc", ResourceRole::Allocator);
table.add("pool_alloc", ResourceRole::NullSource);
table.add("pool_free", ResourceRole::Deallocator);
table.has_role("pool_alloc", ResourceRole::Allocator); // true
table.lookup("pool_alloc"); // Some(&BTreeSet{Allocator, NullSource})
```

### Spec-driven population

`ResourceTable::from_specs(registry)` auto-populates from a `SpecRegistry`:

- `Role::Allocator` -> `ResourceRole::Allocator`
- `Role::Deallocator` -> `ResourceRole::Deallocator`
- `Role::Reallocator` -> `ResourceRole::Reallocator`
- `returns.nullness: maybe_null` -> `ResourceRole::NullSource`
- Any param with `reads: true` -> `ResourceRole::Dereference`

YAML specs with `role` and `returns.nullness` fields automatically populate the resource table -- no separate registration needed.

**Important:** Only `allocator`, `deallocator`, and `reallocator` YAML roles map to `ResourceRole` entries. There are no `acquire`/`release` values in the YAML `Role` enum. For non-memory resources (files, locks, DB connections), either use `role: allocator`/`role: deallocator` as semantic approximations, use `FunctionName` site patterns directly in the `CheckerSpec`, or add entries via `resource_table.add()` in code.
