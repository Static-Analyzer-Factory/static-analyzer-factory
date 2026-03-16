# SAF Checker Development Workflow

This document defines the 8-phase workflow for authoring bug-finding checkers in SAF (Static Analyzer Factory). It is platform-agnostic: it describes *what* to do, not which tool to use. Any coding agent or human contributor can follow these phases.

SAF checkers are **spec-driven**: you define what to detect (source, sink, sanitizer) declaratively, and the framework's SVFG reachability solver does the work. Always try declarative composition before writing code.

---

## Phase 1: Understand the Bug Pattern

**Goal:** Extract the four elements that define every checker: source, sink, sanitizer, and CWE mapping.

**Entry criteria:** You have a bug class description, CWE reference, or a concrete example of the defect to detect.

### Steps

1. **Identify the source** -- where the "bad" value or state originates. Examples: `malloc` return (allocation that might leak), `free`'s argument (pointer that becomes dangling), `NULL` constant (value that must not be dereferenced).

2. **Identify the sink** -- where the bug manifests if the bad value reaches it. Examples: function exit without `free` (leak), `Load`/`Store` instruction after `free` (use-after-free), pointer dereference without null check (null-deref).

3. **Identify the sanitizer** -- what prevents the bug. Examples: `free` call (prevents leak), null check branch (prevents null-deref). Some checkers have no sanitizer (double-free, stack-escape).

4. **Map to a CWE.** Find the closest CWE identifier. Common mappings:
   - CWE-401: Memory leak
   - CWE-415: Double free
   - CWE-416: Use after free
   - CWE-476: NULL pointer dereference
   - CWE-562: Return of stack variable address
   - CWE-772: Missing release of resource
   - CWE-775: Missing release of file descriptor
   - CWE-908: Use of uninitialized resource

5. **For interactive sessions:** Ask the contributor these four questions explicitly and wait for confirmation before proceeding. For autonomous operation: derive all four from the bug description and state your derivation.

**Exit criteria:** You can state: "Source is X, sink is Y, sanitizer is Z, CWE is N" with confidence.

---

## Phase 2: Classify Checker Tier

**Goal:** Determine the authoring approach and select the reachability mode.

**Entry criteria:** Source, sink, sanitizer, and CWE are identified.

### Checker Tiers

Three tiers determine how much code you need to write:

| Tier | When to Use | Authoring Path |
|---|---|---|
| **Tier 1: Declarative** | Source/sink/sanitizer fit existing `SitePattern` variants and one of the four reachability modes | Compose a `CheckerSpec` in Rust (`spec.rs`) or use Python `check_custom()` |
| **Tier 2: Typestate** | Bug involves state machine transitions on a resource (opened/closed, locked/unlocked) | Define a `TypestateSpec` via Python `typestate_custom()` API |
| **Tier 3: Custom patterns** | None of the existing `SitePattern` variants can express your source/sink/sanitizer | Add a new `SitePattern` enum variant in Rust, then compose into `CheckerSpec` |

**Always start at Tier 1.** Only escalate if the existing patterns genuinely cannot express your checker. Most checkers are Tier 1.

### Reachability Mode Selection

The reachability mode defines how the SVFG solver evaluates source-sink relationships:

| Mode | Semantics | Reports When | Typical Checkers |
|---|---|---|---|
| `MayReach` | Source reaches sink on SOME path without passing through a sanitizer | Bad value flows to a dangerous operation | UAF, null-deref, uninit-use, stack-escape |
| `MustNotReach` | Source does NOT reach sanitizer on ALL paths before function exit | Resource acquired but not properly released on every path | File descriptor leak, lock not released, generic resource leak |
| `MultiReach` | Source reaches 2+ distinct sink nodes | Same resource hits multiple conflicting operations | Double-free |
| `NeverReachSink` | Source does NOT reach ANY sink on any path | Resource is allocated but never encounters its cleanup operation | Memory leak (SVF-style NEVERFREE) |

**Decision heuristic:**
- "Value X must not reach operation Y" => `MayReach` (with sanitizer if applicable)
- "Resource X must be released before exit" => `MustNotReach`
- "Resource X must not be released twice" => `MultiReach`
- "Resource X is never released at all" => `NeverReachSink`

**Python limitation:** `check_custom()` supports `may_reach`, `must_not_reach`, and `never_reach_sink` only. `MultiReach` is not available via Python — use the built-in `project.check("double-free")` or write a Rust `CheckerSpec` for custom multi-reach rules.

Load `references/checker-types-guide.md` for the full `SitePattern` catalog and decision tree.

**Exit criteria:** You know the tier (1, 2, or 3) and the reachability mode.

---

## Phase 3: Explore Existing Checkers

**Goal:** Find the closest built-in checker and use it as a template.

**Entry criteria:** Tier and reachability mode selected.

### Steps

1. **Read the built-in checker specs.** Open `crates/saf-analysis/src/checkers/spec.rs` and study the 9 built-in checkers:

   | Checker | Mode | CWE | Source | Sink | Sanitizer |
   |---|---|---|---|---|---|
   | `memory_leak` | NeverReachSink | 401 | Allocator return | Deallocator arg | (none) |
   | `use_after_free` | MayReach | 416 | Deallocator arg | Load/Store deref | (none) |
   | `double_free` | MultiReach | 415 | Allocator return | Deallocator arg | (none) |
   | `null_deref` | MayReach | 476 | NullSource return, NullConstant | Load/Store/GEP deref, Dereference role | NullCheckBranch |
   | `file_descriptor_leak` | MustNotReach | 775 | Acquire return | FunctionExit | Release arg |
   | `uninit_use` | MayReach | 908 | Allocator return | LoadDeref | StoreDeref |
   | `stack_escape` | MayReach | 562 | AllocaInst | FunctionExit | (none) |
   | `lock_not_released` | MustNotReach | 764 | Lock arg | FunctionExit | Unlock arg |
   | `generic_resource_leak` | MustNotReach | 772 | Allocator return | FunctionExit | Deallocator arg |

2. **Pick the closest match.** Select the built-in checker whose reachability mode and pattern structure most closely resemble your new checker. Read its `CheckerSpec` definition carefully.

3. **Trace how the runner resolves its patterns.** Read `crates/saf-analysis/src/checkers/runner.rs` to understand how `SitePattern` variants are resolved to concrete SVFG node sets. Then read `crates/saf-analysis/src/checkers/site_classifier.rs` to see how call sites are classified against the resource table.

4. **Check the resource table.** Read `crates/saf-analysis/src/checkers/resource_table.rs` to see which functions are already mapped to `ResourceRole` values. If your checker needs functions not in the built-in table, you will need YAML function specs.

**Exit criteria:** You have a template checker to base your work on, understand how patterns resolve, and know whether you need new YAML specs.

---

## Phase 4: Write the Spec

**Goal:** Define the checker specification using the appropriate tier.

**Entry criteria:** Template checker identified, resource table gaps known.

Load `references/spec-authoring-guide.md` for the full format and examples.

### Tier 1: Declarative Spec

1. **Check YAML function specs.** If your checker targets library functions not in the built-in `ResourceTable`, create YAML spec files. The spec registry discovers files from these paths (later overrides earlier):
   - `<binary>/../share/saf/specs/*.yaml` -- shipped defaults (installed)
   - `~/.saf/specs/*.yaml` -- user global
   - `./saf-specs/*.yaml` -- project local
   - `./share/saf/specs/*.yaml` -- workspace share (dev builds)
   - `$SAF_SPECS_PATH/*.yaml` -- explicit override (highest priority)

2. **Compose the `CheckerSpec`.**

   **In Rust** (for a permanent built-in checker), add a function to `crates/saf-analysis/src/checkers/spec.rs`:
   ```rust
   pub fn my_checker() -> CheckerSpec {
       CheckerSpec {
           name: "my-checker".to_string(),
           description: "Detects ...".to_string(),
           cwe: Some(NNN),
           severity: Severity::Warning,
           mode: ReachabilityMode::MustNotReach,
           sources: vec![SitePattern::Role {
               role: ResourceRole::Acquire,
               match_return: true,
           }],
           sinks: vec![SitePattern::FunctionExit],
           sanitizers: vec![SitePattern::Role {
               role: ResourceRole::Release,
               match_return: false,
           }],
       }
   }
   ```

   **In Python** (for ad-hoc or user-defined checkers), use `check_custom()`:
   ```python
   findings = project.check_custom(
       "my-checker",
       mode="must_not_reach",
       source_role="acquire",
       source_match_return=True,
       sink_is_exit=True,
       sanitizer_role="release",
       sanitizer_match_return=False,
       cwe=772,
       severity="warning",
   )
   ```

3. **Map source/sink/sanitizer to `SitePattern` variants.** Available variants:
   - `Role { role, match_return }` -- match by `ResourceRole` (Allocator, Deallocator, Acquire, Release, Lock, Unlock, NullSource, Dereference, Reallocator)
   - `FunctionName { name, match_return }` -- match a specific function by name
   - `FunctionExit` -- any function return/exit point
   - `AnyUseOf` -- any SVFG successor of the source
   - `AllocaInst` -- stack allocation instruction
   - `LoadDeref`, `StoreDeref`, `GepDeref` -- pointer dereference instructions
   - `NullConstant` -- explicit NULL assignment
   - `DirectNullDeref` -- instruction dereferencing a literal NULL
   - `NullCheckBranch` -- value guarded by a null check
   - `CustomPredicate { name }` -- runtime-resolved predicate

### Tier 2: Typestate Spec

1. **Define the state machine.** Identify states, transitions, initial state, error states, and accepting states.

2. **Use Python `typestate_custom()` API:**
   ```python
   spec = saf.TypestateSpec(
       name="my-resource",
       states=["opened", "closed", "error"],
       initial="opened",
       error_states=["error"],
       accepting=["closed"],
       transitions=[
           {"from": "opened", "to": "closed", "function": "close_resource"},
           {"from": "closed", "to": "error", "function": "close_resource"},
       ],
   )
   result = project.typestate_custom(spec)
   ```

3. **Or use a built-in typestate spec** if one matches: `"file_io"`, `"mutex_lock"`, `"memory_alloc"`:
   ```python
   result = project.typestate("file_io")
   ```

### Tier 3: Custom Patterns

1. **Add a new `SitePattern` variant** in `crates/saf-analysis/src/checkers/spec.rs`. Follow the existing variant style with doc comments and serde attributes.

2. **Add resolution logic** in `crates/saf-analysis/src/checkers/site_classifier.rs`. The classifier must populate the node sets for your new pattern during AIR instruction scanning.

3. **Update the solver if needed** in `crates/saf-analysis/src/checkers/solver.rs`. Most new patterns work with existing solver modes. If yours requires a fundamentally new reachability algorithm, escalate to the `saf-feature-dev` workflow.

4. **Then compose into a `CheckerSpec`** exactly as in Tier 1.

**Exit criteria:** The checker spec is written (Rust function, Python call, or typestate spec) and compiles without errors.

---

## Phase 5: Create Test Cases

**Goal:** Write minimal C programs that exercise the bug pattern, compile them, and write e2e tests.

**Entry criteria:** Checker spec defined.

Load `references/test-case-guide.md` for detailed patterns and examples.

### Steps

1. **Write the "bad" variant.** Create `tests/programs/c/<checker>_bad.c` -- a minimal C program that exhibits the bug. Include a comment at the top explaining what should be detected:
   ```c
   // Expected: <checker-name> finding — <description of the bug>
   #include <stdlib.h>
   int main() {
       int *p = malloc(sizeof(int));
       // Missing free — should trigger memory-leak finding
       return 0;
   }
   ```

2. **Write the "good" variant.** Create `tests/programs/c/<checker>_good.c` -- the same structure with the bug fixed (sanitizer present, resource released, null check added):
   ```c
   // Expected: no findings — resource properly released
   #include <stdlib.h>
   int main() {
       int *p = malloc(sizeof(int));
       free(p);  // Sanitizer present
       return 0;
   }
   ```

3. **Compile both inside Docker.** LLVM 18 is only available in the container:
   ```bash
   docker compose run --rm dev sh -c \
     'clang -S -emit-llvm -g -O0 tests/programs/c/<checker>_bad.c \
       -o tests/fixtures/llvm/e2e/<checker>_bad.ll && \
      clang -S -emit-llvm -g -O0 tests/programs/c/<checker>_good.c \
       -o tests/fixtures/llvm/e2e/<checker>_good.ll'
   ```

4. **Write the e2e test.** Create or extend a test file in `crates/saf-analysis/tests/`. Use `saf_test_utils::load_ll_fixture()` to load the compiled IR:
   ```rust
   #[test]
   fn my_checker_bad_variant() {
       let module = saf_test_utils::load_ll_fixture("my_checker_bad");
       // ... build SVFG, run checker ...
       assert!(!result.findings.is_empty(), "bad variant should produce findings");
       assert!(result.findings.iter().any(|f| f.checker_name == "my-checker"));
   }

   #[test]
   fn my_checker_good_variant() {
       let module = saf_test_utils::load_ll_fixture("my_checker_good");
       // ... build SVFG, run checker ...
       assert!(result.findings.is_empty(), "good variant should produce no findings");
   }
   ```

5. **For specialized checkers, add more variants:**
   - **Taint checkers:** Test both tainted and sanitized data flows
   - **Typestate checkers:** Test each error transition separately (double-close, use-after-close, etc.)
   - **MustNotReach checkers:** Test both "all paths release" (good) and "some path misses release" (bad)
   - **MultiReach checkers:** Test single-operation (good) vs. duplicate-operation (bad)

6. **Use specific assertions, not count assertions:**
   ```rust
   // GOOD: survives when unrelated changes add legitimate findings
   assert!(result.findings.iter().any(|f| f.checker_name == "my-checker"));

   // FRAGILE: breaks if the framework adds a new check
   assert_eq!(result.findings.len(), 1);
   ```

**Exit criteria:** Bad variant compiles and produces findings. Good variant compiles and produces zero findings. Tests are written and fail meaningfully (if the checker is not yet integrated).

---

## Phase 6: Run and Debug

**Goal:** Execute the checker and verify it produces correct results.

**Entry criteria:** Test cases and checker spec exist.

### Steps

1. **Run the test suite.** Capture output for inspection:
   ```bash
   make test 2>&1 | tee /tmp/test-output.txt
   ```
   SAF uses `cargo-nextest`. Look for `Summary` lines, not `test result:`.

2. **Use `SAF_LOG` for targeted debugging.** SAF has a structured debug logging system controlled by environment variables. Common checker debug workflows:

   | Symptom | SAF_LOG Filter | What to Look For |
   |---|---|---|
   | Checker misses a finding (false negative) | `checker[reasoning,path,result]` | Solver's reachability decision for each source-sink pair |
   | Source/sink not classified | `checker[reasoning]` | Missing source/sink nodes in classification output |
   | SVFG edges missing | `svfg[edge]` | Whether value-flow edges connect source to sink |
   | PTA imprecision causes missed flow | `checker[reasoning],pta::solve[pts]` | Points-to sets for relevant pointers |
   | Wrong reachability verdict | `checker[reasoning,path]` | BFS traversal path and sanitizer encounters |

   Enable logging inside Docker:
   ```bash
   docker compose run --rm -e SAF_LOG="checker[reasoning,path,result]" dev sh -c \
     'cargo nextest run -p saf-analysis --test my_checker_e2e'
   ```

   Write log to file to avoid mixing with other stderr output:
   ```bash
   docker compose run --rm \
     -e SAF_LOG="checker[reasoning],svfg[edge]" \
     -e SAF_LOG_FILE=/tmp/saf-checker.log \
     dev sh -c 'cargo nextest run -p saf-analysis --test my_checker_e2e'
   ```

3. **Compare findings against expected results.** For each test case, verify:
   - Bad variant: at least one finding with the correct `checker_name` and `cwe`
   - Good variant: zero findings from this checker
   - Finding traces make sense (source node is the expected allocation/call, sink node is the expected dereference/exit)

4. **For Python checkers:** Use `print()` probes inside Docker to inspect intermediate results:
   ```bash
   docker compose run --rm dev sh -c 'python3 my_checker_script.py'
   ```

**Exit criteria:** Checker produces correct results on both bad and good variants.

---

## Phase 7: Refine

**Goal:** Iteratively improve precision and recall through targeted spec adjustments.

**Entry criteria:** Checker runs but may have false positives or false negatives.

This phase is an iterative loop with Phase 6: diagnose a problem, hypothesize a fix, modify the spec, re-run, repeat.

### Fixing False Positives (checker reports bugs that are not real)

- **Add sanitizers.** If the checker fires on code that properly handles the resource, add a sanitizer pattern that matches the handling function.
- **Tighten site patterns.** Use `FunctionName` instead of `Role` if the role is too broad. Use a more specific `ResourceRole` if available.
- **Enable path-sensitive mode.** For `MayReach` checkers, use `check_path_sensitive()` (Python) or the guarded solver (Rust) to prune infeasible paths via Z3 guard contradiction detection:
  ```python
  result = project.check_path_sensitive("my-checker", z3_timeout_ms=1000)
  # result.feasible — real findings
  # result.infeasible — false positives pruned by Z3
  ```
- **Scope to specific functions.** Use `SolverConfig.reachable_functions` to restrict analysis to functions of interest, avoiding cross-function false positives.

### Fixing False Negatives (checker misses real bugs)

- **Broaden source/sink patterns.** Add multiple `SitePattern` variants to sources or sinks. The null-deref checker, for example, has 2 source patterns and 4 sink patterns.
- **Check YAML function specs.** If your checker targets library functions, verify they are modeled in YAML specs. Missing specs make functions invisible to the resource table. Check `share/saf/specs/` for shipped defaults and add project-local specs to `saf-specs/` if needed.
- **Verify SVFG connectivity.** Use `SAF_LOG=svfg[edge]` to check whether value-flow edges exist between source and sink. Missing edges indicate an SVFG construction gap (escalate to `saf-feature-dev`).
- **Check PTA precision.** Use `SAF_LOG=pta::solve[pts]` to verify that the pointer analysis resolves relevant indirect calls. Missing callgraph edges cause missing SVFG edges.

### Performance Issues

- **Too many source-sink pairs.** If the checker creates an excessive number of source-sink combinations on large programs, restrict sources or sinks to specific roles or functions.
- **Solver depth.** If the BFS hits the `max_depth` limit (default 5000), consider whether the depth limit is appropriate or if the checker's pattern is too broad.

### Benchmark Validation

When the checker is stable, run it against benchmark suites if the bug class maps to a supported category:

- **PTABen** (pointer analysis benchmarks, 30-120s):
  ```bash
  docker compose run --rm dev sh -c \
    'cargo run --release -p saf-bench -- ptaben \
      --compiled-dir tests/benchmarks/ptaben/.compiled \
      -o /workspace/tests/benchmarks/ptaben/results.json'
  ```
  Read results on the host from `tests/benchmarks/ptaben/results.json`.

- **Juliet** (NIST CWE test suite, if your CWE is among the 15 supported categories):
  ```bash
  make test-juliet CWE=CWE<NNN>
  ```
  Juliet runs in aggressive mode and reports precision/recall/F1 per CWE.

Always use `-o <path>` for PTABen to write results to a file rather than capturing JSON from Docker stdout. PTABen benchmarks should be run in the background due to their duration.

**Exit criteria:** False positives and false negatives are at acceptable levels. Benchmark results (if applicable) show no regressions.

---

## Phase 8: Export and Document

**Goal:** Make the checker discoverable and document its characteristics.

**Entry criteria:** Checker is refined and stable.

### Steps

1. **Place YAML function specs** in the appropriate discovery path:
   - `share/saf/specs/*.yaml` -- for specs that should ship with SAF (commit to repo)
   - `saf-specs/*.yaml` -- for project-local specs (user's project)
   - `~/.saf/specs/*.yaml` -- for user-global specs

2. **Register the checker.**

   **For Rust built-in checkers:**
   - Add your checker function to `builtin_checkers()` in `crates/saf-analysis/src/checkers/spec.rs`
   - Add the name string to `builtin_checker_names()`
   - Update the test `all_builtin_checkers_have_names` count if it uses an exact count

   **For Python checkers:**
   - Create a reusable script in `tutorials/` or `examples/` demonstrating usage
   - Document the `check_custom()` or `typestate_custom()` call with all parameters

3. **Document the checker.** Write a brief description covering:
   - What bug class it detects and the CWE mapping
   - Source, sink, and sanitizer definitions
   - Reachability mode and rationale
   - Known limitations (e.g., does not handle wrapper functions, requires YAML specs for library X)
   - False positive / false negative characteristics
   - Example invocation (Rust and/or Python)

4. **Run formatting and lint checks** to ensure all new code is clean:
   ```bash
   make fmt && make lint
   ```

5. **Verify tests pass:**
   ```bash
   make test 2>&1 | tee /tmp/test-output.txt
   ```

**Exit criteria:** Checker is registered, YAML specs are placed, documentation exists, tests pass, lint is clean.

---

## Quick Reference: Commands

All builds and tests run inside Docker. Never run `cargo build` or `cargo test` locally for LLVM-dependent crates.

| Action | Command |
|---|---|
| Run all tests | `make test` |
| Format code | `make fmt` |
| Lint code | `make lint` (always run `make fmt` first) |
| Format + lint | `make fmt && make lint` |
| Compile C to LLVM IR | `docker compose run --rm dev sh -c 'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll'` |
| Run specific test | `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test <test_name>'` |
| Debug checker reasoning | `docker compose run --rm -e SAF_LOG="checker[reasoning,path,result]" dev sh -c '...'` |
| Debug SVFG edges | `docker compose run --rm -e SAF_LOG="svfg[edge]" dev sh -c '...'` |
| Debug PTA precision | `docker compose run --rm -e SAF_LOG="pta::solve[pts,worklist]" dev sh -c '...'` |
| Log to file | `docker compose run --rm -e SAF_LOG="checker" -e SAF_LOG_FILE=/tmp/saf.log dev sh -c '...'` |
| Run PTABen benchmarks | `docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'` |
| Run Juliet (single CWE) | `make test-juliet CWE=CWE<NNN>` |

## Quick Reference: Key File Paths

| What | Path |
|---|---|
| Checker specs (built-in definitions) | `crates/saf-analysis/src/checkers/spec.rs` |
| Checker runner (orchestration) | `crates/saf-analysis/src/checkers/runner.rs` |
| Reachability solvers | `crates/saf-analysis/src/checkers/solver.rs` |
| Site classifier | `crates/saf-analysis/src/checkers/site_classifier.rs` |
| Resource table (function-to-role mapping) | `crates/saf-analysis/src/checkers/resource_table.rs` |
| Finding types and export | `crates/saf-analysis/src/checkers/finding.rs` |
| Path-sensitive runner | `crates/saf-analysis/src/checkers/pathsens_runner.rs` |
| Path-sensitive solver (Z3/BDD) | `crates/saf-analysis/src/checkers/pathsens.rs` |
| Z3 solver integration | `crates/saf-analysis/src/checkers/z3solver.rs` |
| Checker diagnostics/summary | `crates/saf-analysis/src/checkers/summary.rs` |
| Python checker bindings | `crates/saf-python/src/checkers.rs` |
| Python project API (check/check_custom) | `crates/saf-python/src/project.rs` |
| YAML spec registry | `crates/saf-core/src/spec/registry.rs` |
| Spec types and schema | `crates/saf-core/src/spec/` |
| Shipped default specs | `share/saf/specs/` |
| C test programs | `tests/programs/c/` |
| Compiled LLVM IR fixtures | `tests/fixtures/llvm/e2e/` |
| E2E Rust tests | `crates/saf-analysis/tests/*_e2e.rs` |
| PTABen compiled benchmarks | `tests/benchmarks/ptaben/.compiled/` |

## Quick Reference: Reachability Mode Selection

| Question | Answer | Mode |
|---|---|---|
| Does bad value flow to a dangerous operation? | Yes | `MayReach` |
| Must a resource be released on all paths before exit? | Yes | `MustNotReach` |
| Can the same resource hit a conflicting operation twice? | Yes | `MultiReach` |
| Is the resource never cleaned up at all? | Yes | `NeverReachSink` |

## Quick Reference: SitePattern Variants

| Variant | Matches | Common Use |
|---|---|---|
| `Role { role, match_return }` | Call sites by `ResourceRole` | Most checkers (allocator, deallocator, acquire, release, lock, unlock) |
| `FunctionName { name, match_return }` | Specific function by name | Targeting a particular API |
| `FunctionExit` | Function return/exit points | Leak/release checkers (sink = exit) |
| `AnyUseOf` | Any SVFG successor | Broad taint tracking |
| `AllocaInst` | Stack allocation (`alloca`) | Stack escape detection |
| `LoadDeref` | Load instruction (pointer deref) | UAF, null-deref, uninit-use sinks |
| `StoreDeref` | Store instruction (pointer deref) | UAF sinks, uninit-use sanitizer |
| `GepDeref` | GEP instruction (base pointer) | Null-deref sinks |
| `NullConstant` | Explicit NULL assignment | Null-deref source |
| `DirectNullDeref` | Literal NULL used as deref pointer | Definite null-deref sink |
| `NullCheckBranch` | Value guarded by null check | Null-deref sanitizer |
| `CustomPredicate { name }` | Runtime-resolved predicate | Extensibility |
