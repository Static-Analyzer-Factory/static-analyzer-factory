<!-- SAF Checker Dev Workflow v0.1.0 — from saf-dev-skills -->

## SAF Checker Development Workflow

> Workflow for creating SAF bug-finding checkers, taint rules, and resource leak detectors.
> For detailed references, see `skills/saf-checker-dev/core/references/`.

SAF checkers are **spec-driven**: define source, sink, sanitizer declaratively and the SVFG reachability solver does the work. Always try declarative composition before writing code.

### Tier Classification

| Tier | When to Use | Authoring Path |
|---|---|---|
| **1: Declarative** | Source/sink/sanitizer fit existing `SitePattern` variants | Compose a `CheckerSpec` in Rust or use Python `check_custom()` |
| **2: Typestate** | Bug involves state machine transitions (opened/closed, locked/unlocked) | Define a `TypestateSpec` via Python `typestate_custom()` |
| **3: Custom** | No existing `SitePattern` can express your source/sink/sanitizer | Add a new `SitePattern` variant in Rust, then compose into `CheckerSpec` |

Always start at Tier 1. Most checkers are Tier 1 (all 9 built-ins are).

### Reachability Modes

| Mode | Reports When | Typical Checkers |
|---|---|---|
| `MayReach` | Source reaches sink on SOME path without sanitizer | UAF, null-deref, uninit-use, stack-escape |
| `MustNotReach` | Source does NOT reach sanitizer on ALL paths before exit | FD leak, lock not released, resource leak |
| `MultiReach` | Source reaches 2+ distinct sink nodes | Double-free |
| `NeverReachSink` | Source does NOT reach ANY sink on any path | Memory leak (NEVERFREE) |

**Decision heuristic:** "Value must not reach operation" = `MayReach`; "Resource must be released before exit" = `MustNotReach`; "Resource must not be released twice" = `MultiReach`; "Resource is never released" = `NeverReachSink`.

### Phase 1: Understand the Bug Pattern

Extract four elements: **source** (where the bad value originates), **sink** (where the bug manifests), **sanitizer** (what prevents the bug), and **CWE mapping**. Common CWEs: 401 (memory leak), 415 (double-free), 416 (UAF), 476 (null-deref), 562 (stack escape), 772 (resource leak), 775 (FD leak), 908 (uninit use).

Exit: You can state "Source is X, sink is Y, sanitizer is Z, CWE is N."

### Phase 2: Classify Checker Tier

Use the tier table and reachability mode table above. Check whether existing `SitePattern` variants cover your source/sink/sanitizer. If yes, Tier 1. If state machine needed, Tier 2. Otherwise Tier 3.

### Phase 3: Explore Existing Checkers

Read `crates/saf-analysis/src/checkers/spec.rs` for the 9 built-in checkers. Pick the closest match by reachability mode and pattern structure. Trace resolution logic in `runner.rs`, `site_classifier.rs`, and `resource_table.rs`. Check whether needed functions are in the built-in resource table; if not, you need YAML specs.

**Built-in checkers:** `memory-leak` (NeverReachSink/401), `use-after-free` (MayReach/416), `double-free` (MultiReach/415), `null-deref` (MayReach/476), `file-descriptor-leak` (MustNotReach/775), `uninit-use` (MayReach/908), `stack-escape` (MayReach/562), `lock-not-released` (MustNotReach/764), `generic-resource-leak` (MustNotReach/772).

### Phase 4: Write the Spec

**YAML function specs** for library functions not in the built-in resource table. Discovery order (later overrides earlier): `<binary>/../share/saf/specs/` (shipped), `~/.saf/specs/` (user), `./saf-specs/` (project), `./share/saf/specs/` (workspace dev), `$SAF_SPECS_PATH` (explicit).

**Rust CheckerSpec example** (permanent built-in):
```rust
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

**Python check_custom()** (ad-hoc checkers):
```python
findings = project.check_custom(
    "fd-leak", mode="must_not_reach",
    source_role="acquire", source_match_return=True,
    sink_is_exit=True, sanitizer_role="release",
    sanitizer_match_return=False, cwe=775, severity="warning",
)
```

**SitePattern variants:** `Role { role, match_return }`, `FunctionName { name, match_return }`, `FunctionExit`, `AnyUseOf`, `AllocaInst`, `LoadDeref`, `StoreDeref`, `GepDeref`, `NullConstant`, `DirectNullDeref`, `NullCheckBranch`, `CustomPredicate { name }`.

**ResourceRole values:** `Allocator`, `Deallocator`, `Reallocator`, `Acquire`, `Release`, `Lock`, `Unlock`, `NullSource`, `Dereference`.

**Tier 2 (Typestate):** Use `saf.TypestateSpec` with states, transitions, initial/error/accepting states. Built-in specs: `"file_io"`, `"mutex_lock"`, `"memory_alloc"`.

**Tier 3:** Add a `SitePattern` variant in `spec.rs`, resolution logic in `site_classifier.rs`, then compose into `CheckerSpec` as Tier 1.

### Phase 5: Create Test Cases

Write paired C programs: `tests/programs/c/<checker>_bad.c` (exhibits bug) and `<checker>_good.c` (bug fixed). Mark key points with `// SOURCE`, `// SINK`, `// SANITIZER`, `// BUG`, `// OK`.

Compile inside Docker:
```bash
docker compose run --rm dev sh -c \
  'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll'
```

Write e2e tests in `crates/saf-analysis/tests/`. Use `load_ll_fixture()` to load compiled IR. Assert bad variant has findings (`!findings.is_empty()`), good variant has none. Use specific assertions (`any(|f| f.checker_name == "...")`) over count assertions.

### Phase 6: Run and Debug

Run tests: `make test 2>&1 | tee /tmp/test-output.txt`. SAF uses `cargo-nextest` -- look for `Summary` lines.

Debug with `SAF_LOG`:
```bash
docker compose run --rm -e SAF_LOG="checker[reasoning,path,result]" dev sh -c '...'
```

| Symptom | SAF_LOG Filter |
|---|---|
| Checker misses a finding | `checker[reasoning,path,result]` |
| Source/sink not classified | `checker[reasoning]` |
| SVFG edges missing | `svfg[edge]` |
| PTA imprecision | `pta::solve[pts]` |

Use `-e SAF_LOG_FILE=/tmp/saf.log` to write logs to a file.

### Phase 7: Refine

**False positives:** Add sanitizers, tighten site patterns (`FunctionName` over `Role`), enable path-sensitive mode with Z3, scope to specific functions.

**False negatives:** Broaden source/sink patterns (e.g., null-deref has 2 sources, 4 sinks), add YAML function specs for missing libraries, verify SVFG connectivity with `SAF_LOG=svfg[edge]`, check PTA with `SAF_LOG=pta::solve[pts]`.

**Benchmarks:** PTABen (30-120s, always use `-o <path>`):
```bash
docker compose run --rm dev sh -c \
  'cargo run --release -p saf-bench -- ptaben \
    --compiled-dir tests/benchmarks/ptaben/.compiled \
    -o /workspace/tests/benchmarks/ptaben/results.json'
```
Juliet (NIST CWE suite): `make test-juliet CWE=CWE<NNN>`.

### Phase 8: Export and Document

1. Place YAML specs in the appropriate discovery path
2. Register the checker: add to `builtin_checkers()` and `builtin_checker_names()` in `spec.rs`
3. Document: bug class, CWE, source/sink/sanitizer, mode, limitations, example
4. Run `make fmt && make lint` then `make test`

### Key File Paths

| What | Path |
|---|---|
| Checker specs (built-in) | `crates/saf-analysis/src/checkers/spec.rs` |
| Checker runner | `crates/saf-analysis/src/checkers/runner.rs` |
| Reachability solver | `crates/saf-analysis/src/checkers/solver.rs` |
| Site classifier | `crates/saf-analysis/src/checkers/site_classifier.rs` |
| Resource table | `crates/saf-analysis/src/checkers/resource_table.rs` |
| Path-sensitive runner | `crates/saf-analysis/src/checkers/pathsens_runner.rs` |
| Python checker bindings | `crates/saf-python/src/checkers.rs` |
| YAML spec registry | `crates/saf-core/src/spec/registry.rs` |
| Shipped default specs | `share/saf/specs/` |
| C test programs | `tests/programs/c/` |
| Compiled IR fixtures | `tests/fixtures/llvm/e2e/` |
| E2E tests | `crates/saf-analysis/tests/*_e2e.rs` |

### Commands

| Action | Command |
|---|---|
| Run all tests | `make test` |
| Format + lint | `make fmt && make lint` |
| Compile C to LLVM IR | `docker compose run --rm dev sh -c 'clang -S -emit-llvm -g -O0 <src> -o <dst>'` |
| Run specific test | `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test <name>'` |
| Debug checker | `docker compose run --rm -e SAF_LOG="checker[reasoning]" dev sh -c '...'` |
