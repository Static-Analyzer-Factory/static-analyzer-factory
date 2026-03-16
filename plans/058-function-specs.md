# Plan 058: Function Specs

**Epic**: E33 (Function Specs)
**Status**: approved
**Design**: `docs/plans/2026-02-02-function-specs-design.md`

## Overview

Implement a pluggable function specification system for SAF that enables precise modeling of external/library function behavior across all analyses.

## Phases

### Phase 1: Core Infrastructure (~400 LOC)

**Goal**: Define spec types and implement loading/discovery.

**Files**:
- `crates/saf-core/src/spec/mod.rs` — module root, re-exports
- `crates/saf-core/src/spec/types.rs` — `FunctionSpec`, `Role`, `Nullness`, `Pointer`, etc.
- `crates/saf-core/src/spec/registry.rs` — `SpecRegistry`, loading, merging
- `crates/saf-core/src/spec/pattern.rs` — glob/regex matching
- `crates/saf-core/src/spec/schema.rs` — YAML parsing, validation

**Tasks**:
1. Define `FunctionSpec` struct with all optional fields
2. Define enums: `Role`, `Nullness`, `Pointer`, `ParamSpec`, `ReturnSpec`, `TaintSpec`
3. Implement `SpecRegistry::load()` with discovery order:
   - `./saf-specs/*.yaml`
   - `~/.saf/specs/*.yaml`
   - `<binary>/../share/saf/specs/*.yaml`
   - `$SAF_SPECS_PATH/*.yaml`
4. Implement name matching: exact, `glob:`, `regex:`
5. Implement per-function merging (later overrides earlier)
6. Add validation with actionable error messages
7. Unit tests for loading, merging, pattern matching

**Dependencies**: Add `serde_yaml`, `glob` crates to saf-core

### Phase 2: PTA Integration (~200 LOC)

**Goal**: Use specs for constraint generation on external functions.

**Files**:
- `crates/saf-analysis/src/pta/extract.rs` — extend `extract_call_constraints()`
- `crates/saf-analysis/src/pta/mod_ref.rs` — use specs for mod/ref

**Tasks**:
1. Add `specs: &SpecRegistry` parameter to `extract_interprocedural()`
2. In call constraint extraction:
   - `pointer: fresh_heap` → `AllocConstraint`
   - `aliases: param.N` → `CopyConstraint`
   - `modifies: true` → `StoreConstraint`
3. Update `compute_mod_ref()` to use `spec.params[].modifies/reads`
4. Integration tests with malloc/free/strcpy

**Ship**: `share/saf/specs/libc/alloc.yaml`

### Phase 3: Nullness Integration (~150 LOC)

**Goal**: Use specs for nullness analysis of external calls.

**Files**:
- `crates/saf-analysis/src/absint/nullness.rs` — extend transfer function

**Tasks**:
1. Add `specs: &SpecRegistry` parameter to `analyze_nullness()`
2. In `transfer_call()`:
   - Check `required_nonnull` preconditions → report potential null arg
   - Apply `returns.nullness` to return value
3. Handle `noreturn: true` → mark successors unreachable
4. Integration tests for malloc null check, string function preconditions

**Ship**: `share/saf/specs/libc/string.yaml`

### Phase 4: Taint Integration (~200 LOC)

**Goal**: Use specs for taint source/sink/propagation.

**Files**:
- `crates/saf-analysis/src/ifds/taint.rs` — extend flow functions

**Tasks**:
1. Add `specs: &SpecRegistry` parameter to `TaintIfdsProblem::new()`
2. In flow functions:
   - `role: source` → add taint to return/params marked `tainted: true`
   - `role: sink` → report if tainted value reaches sink
   - `role: sanitizer` → kill taint
   - `taint.propagates` → apply propagation rules
3. Integration tests for getenv→system, recv→SQL paths

**Ship**: `share/saf/specs/taint/sources.yaml`, `sinks.yaml`, `sanitizers.yaml`

### Phase 5: CLI & Python Bindings (~200 LOC)

**Goal**: Expose spec system via CLI and Python.

**Files**:
- `crates/saf-cli/src/main.rs` — add `saf specs` subcommand
- `crates/saf-python/src/spec.rs` — Python bindings

**Tasks**:
1. CLI commands:
   - `saf specs list [--verbose]`
   - `saf specs validate <path>`
   - `saf specs lookup <name>`
   - `saf analyze --specs <path>` flag
   - `saf analyze --quiet-spec-warnings` flag
2. Python bindings:
   - `SpecRegistry.load()`, `load_from(paths)`
   - `SpecRegistry.lookup(name)` → `FunctionSpec`
   - `FunctionSpec` properties: `role`, `pure`, `noreturn`, `params`, `returns`, `taint`
   - Pass `specs=` to analysis functions

### Phase 6: Complete Default Specs (~300 LOC YAML)

**Goal**: Ship comprehensive libc/POSIX specs.

**Files**:
- `share/saf/specs/libc/alloc.yaml` — malloc, calloc, realloc, free
- `share/saf/specs/libc/string.yaml` — strlen, strcpy, strcat, strcmp, memcpy, memset
- `share/saf/specs/libc/stdio.yaml` — printf, fprintf, fopen, fclose, fread, fwrite
- `share/saf/specs/libc/stdlib.yaml` — atoi, getenv, system, exit, abort
- `share/saf/specs/posix/unistd.yaml` — read, write, open, close, fork
- `share/saf/specs/posix/socket.yaml` — socket, bind, listen, accept, recv, send
- `share/saf/specs/taint/sources.yaml` — getenv, read, recv, fgets, scanf
- `share/saf/specs/taint/sinks.yaml` — system, exec*, SQL
- `share/saf/specs/taint/sanitizers.yaml` — escape functions

**Tasks**:
1. Write specs for ~100 common functions
2. Validate against real programs
3. Document spec format in `share/saf/specs/README.md`

### Phase 7: Dockerfile & Distribution (~50 LOC)

**Goal**: Ensure specs are properly distributed.

**Files**:
- `Dockerfile` — COPY specs
- `Makefile` — install target
- `Cargo.toml` — include in package

**Tasks**:
1. Add `COPY share/ /usr/local/share/` to Dockerfile
2. Add `make install` target that copies specs
3. Ensure `cargo install` includes specs via build.rs or similar

## Future Integrations (tracked, not in this plan)

| Integration | Priority | Notes |
|-------------|----------|-------|
| Interval Analysis | Medium | Use `returns.interval` for numeric bounds |
| Escape Analysis | Medium | New analysis using `escapes` field |
| Purity Analysis | Medium | Combine `pure` + escape + modifies |
| Bounds Checking | Low | Use `size_from` for buffer overflow |
| Call Graph Callbacks | Low | Use `callback` field for indirect edges |

## Estimated LOC

| Phase | LOC |
|-------|-----|
| Phase 1: Core | ~400 |
| Phase 2: PTA | ~200 |
| Phase 3: Nullness | ~150 |
| Phase 4: Taint | ~200 |
| Phase 5: CLI/Python | ~200 |
| Phase 6: Default specs | ~300 YAML |
| Phase 7: Distribution | ~50 |
| **Total** | **~1500** |

## Success Criteria

1. `saf specs list` shows loaded specs from all discovery paths
2. `saf specs validate` catches malformed YAML with helpful errors
3. PTA generates correct constraints for malloc/free/strcpy from specs
4. Nullness analysis reports null pointer dereference after malloc without check
5. Taint analysis detects getenv→system flow using specs
6. Users can add project-local specs that override defaults
7. Missing spec warnings help users identify functions needing specs
