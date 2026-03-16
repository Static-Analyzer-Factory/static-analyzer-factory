# SAF End-to-End Testing Guide

E2E tests follow a consistent pipeline: C source -> LLVM IR -> analysis -> assertions. This guide uses a use-after-free detector as a running example through every step.

## 1. Write a C Test Program

Location: `tests/programs/c/<name>.c`

Keep it minimal -- one behavior per file. Mark key points with `// SOURCE:` / `// SINK:` comments.

```c
// tests/programs/c/use_after_free.c
// CWE-416: Simple use-after-free
// malloc result flows to free, then to a dereference.
#include <stdlib.h>

int main(void) {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: heap allocation
    *p = 42;
    free(p);                               // free deallocates
    return *p;                             // SINK: use after free
}
```

## 2. Compile to LLVM IR (Inside Docker)

LLVM 18 is only available inside the Docker container. Never compile locally.

```bash
# Interactive
make shell
clang -S -emit-llvm -g -O0 tests/programs/c/use_after_free.c \
  -o tests/fixtures/llvm/e2e/use_after_free.ll

# One-shot
docker compose run --rm dev sh -c \
  'clang -S -emit-llvm -g -O0 tests/programs/c/use_after_free.c \
   -o tests/fixtures/llvm/e2e/use_after_free.ll'
```

Flags: `-S -emit-llvm` emits textual IR, `-g` adds debug info, `-O0` prevents optimization. The compiled `.ll` goes in `tests/fixtures/llvm/e2e/` and is checked into the repo.

## 3. Write the Rust E2E Test

Location: `crates/saf-analysis/tests/<name>_e2e.rs`

### Test Utilities (`saf-test-utils`)

| Function | Returns | Use case |
|----------|---------|----------|
| `load_ll_fixture("name")` | `AirModule` | Most common -- single-module tests |
| `load_ll_bundle("name")` | `AirBundle` | When you need module + metadata |
| `load_ll_from_path(path)` | `AirBundle` | Non-standard fixture locations |
| `load_air_json_fixture("name")` | `AirModule` | AIR-JSON format fixtures |
| `load_verification_fixture(cat, name)` | `AirModule` | `tests/fixtures/<category>/` |

### Complete Test File

```rust
//! E2E test: use-after-free detection via value flow.

use std::collections::BTreeSet;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::selector::Selector;
use saf_analysis::{QueryLimits, ValueFlowBuilder, ValueFlowConfig};
use saf_core::air::AirModule;
use saf_core::ids::ValueId;
use saf_test_utils::load_ll_fixture;

fn build_fast_vf(module: &AirModule) -> saf_analysis::ValueFlowGraph {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let config = ValueFlowConfig::fast();
    let builder = ValueFlowBuilder::new(&config, module, &defuse, &callgraph, None);
    builder.build()
}

fn resolve(selector: &Selector, module: &AirModule) -> BTreeSet<ValueId> {
    selector.resolve(module).expect("selector resolution failed")
}

#[test]
fn use_after_free_malloc_to_free_flow() {
    let module = load_ll_fixture("use_after_free");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::call_to("malloc"), &module);
    let sinks = resolve(&Selector::arg_to("free", Some(0)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "malloc source should resolve");
    assert!(!sinks.is_empty(), "free sink should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find flow from malloc result to free arg"
    );
}

#[test]
fn use_after_free_graph_has_structure() {
    let module = load_ll_fixture("use_after_free");
    let graph = build_fast_vf(&module);

    assert!(!module.functions.is_empty(), "should have functions");
    assert!(graph.node_count() > 0, "value flow graph should have nodes");
}
```

### Other Analysis Pipelines

**PTA (pointer analysis):**

```rust
use saf_analysis::pta::context::PtaContext;
use saf_analysis::PtaConfig;

let pta_config = PtaConfig::default();
let mut pta_ctx = PtaContext::new(pta_config);
let result = pta_ctx.analyze(&module);
// Use result.pts for points-to queries, result.factory for location lookups
```

**Constraint extraction:**

```rust
use saf_analysis::{FieldSensitivity, LocationFactory, extract_constraints};

let mut factory = LocationFactory::new(
    FieldSensitivity::StructFields { max_depth: 2 },
);
let constraints = extract_constraints(&module, &mut factory);
```

## 4. Assertion Style

Prefer specific assertions over count-based ones. Counts break when unrelated changes add legitimate constraints.

```rust
// GOOD: assert a specific property exists
assert!(
    constraints.addr.iter().any(|a| a.ptr == expected_id),
    "should have Addr constraint for variable x"
);

// GOOD: assert a flow exists
assert!(!flows.is_empty(), "should find flow from malloc to free");

// GOOD: relative comparison when counts matter
assert!(full.copy.len() > intra.copy.len(),
    "full extraction should include interprocedural constraints");

// BAD: exact count -- breaks when new optimizations add constraints
assert_eq!(constraints.addr.len(), 2);
```

## 5. Running Tests

SAF uses `cargo-nextest`, not `cargo test`. Grep for `Summary` or `passed`, never `test result:`.

```bash
# Full suite (Rust + Python)
make test

# Single test by name
docker compose run --rm dev sh -c \
  'cargo nextest run -p saf-analysis use_after_free'

# All e2e tests
docker compose run --rm dev sh -c \
  'cargo nextest run -p saf-analysis --test "*_e2e"'
```

Capture output once rather than re-running expensive commands:

```bash
make test 2>&1 | tee /tmp/test-output.txt
grep "FAIL\|Summary" /tmp/test-output.txt
```

Expected nextest output: `Summary [   0.XXXs] 2 tests run: 2 passed, 0 skipped`

## 6. Python E2E Tests

Location: `python/tests/test_<feature>.py`

Python tests use the `saf` PyO3 bindings and load fixtures by path:

```python
"""E2E test for use-after-free detection via Python SDK."""
import os
import unittest
import saf

FIXTURE_DIR = os.path.join(
    os.path.dirname(__file__), "..", "..",
    "tests", "fixtures", "llvm", "e2e",
)

def fixture(name: str) -> str:
    return os.path.join(FIXTURE_DIR, f"{name}.ll")

class TestUafSimple(unittest.TestCase):
    def test_project_opens(self) -> None:
        proj = saf.Project.open(fixture("use_after_free"))
        self.assertIsNotNone(proj)

    def test_malloc_to_free_flow(self) -> None:
        proj = saf.Project.open(fixture("use_after_free"))
        q = proj.query()
        findings = q.taint_flow(
            sources=saf.sources.call("malloc"),
            sinks=saf.sinks.call("free", arg_index=0),
        )
        self.assertGreater(len(findings), 0,
                           "should find flow from malloc to free")
```

Run inside Docker:

```bash
docker compose run --rm dev sh -c \
  'cd /workspace && python3 -m pytest python/tests/test_use_after_free.py -v'
```

## 7. Non-LLVM Frontend Testing

For frontends that don't use C/LLVM as input (e.g., Java bytecode, custom IR formats), the C-to-LLVM pipeline doesn't apply. Instead:

- **Test AIR output directly:** Use `load_air_json_fixture("name")` to load hand-written `.air.json` files that represent the expected AIR output of your frontend
- **Location:** `tests/fixtures/air_json/<name>.air.json`
- **Frontend isolation tests** go in `crates/saf-frontends/tests/` (test that the frontend produces correct `AirBundle`)
- **Full pipeline tests** go in `crates/saf-analysis/tests/` (test that analysis works on the AIR your frontend produces)

## Quick Reference

| Item | Location |
|------|----------|
| C source programs | `tests/programs/c/<name>.c` |
| Compiled LLVM IR fixtures | `tests/fixtures/llvm/e2e/<name>.ll` |
| Rust E2E tests | `crates/saf-analysis/tests/<name>_e2e.rs` |
| Python E2E tests | `python/tests/test_<feature>.py` |
| Test utility crate | `crates/saf-test-utils/src/lib.rs` |
| Fixture loader | `saf_test_utils::load_ll_fixture("<name>")` |
| Run all tests | `make test` |
| Run one Rust test | `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis <name>'` |
| Run Python tests | `docker compose run --rm dev sh -c 'cd /workspace && python3 -m pytest python/tests/<file> -v'` |
