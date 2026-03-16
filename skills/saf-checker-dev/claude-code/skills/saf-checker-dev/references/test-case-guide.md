# Test Case Guide

How to write test programs and e2e tests for SAF checkers.

---

## Good/Bad Variant Pattern

Every checker needs paired test programs:

- **Bad variant** (`<checker>_bad.c`): minimal C program that exhibits the bug. The checker MUST report findings.
- **Good variant** (`<checker>_good.c`): same structure but the bug is fixed. The checker MUST report zero findings.

Bad variants test recall (catches real bugs). Good variants test precision (no false positives). Always write both.
## C Test Program Guidelines

- Location: `tests/programs/c/<checker>_{bad,good}.c`
- Compiled fixtures: `tests/fixtures/llvm/e2e/<checker>_{bad,good}.ll`
- Keep minimal -- one bug pattern per file
- Mark key points: `// SOURCE`, `// SINK`, `// SANITIZER`, `// BUG`, `// OK`
- Use only standard library headers
- Compile inside Docker:
  ```bash
  docker compose run --rm dev sh -c \
    'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll'
  ```

---

## Worked Example 1: Resource Leak (NeverReachSink)

**Bad** -- `my_leak_bad.c`:
```c
// Memory leak: malloc without free. Expected: finding reported.
#include <stdlib.h>
int main() {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: allocation
    *p = 42;
    return *p;  // BUG: p is never freed
}
```

**Good** -- `my_leak_good.c`:
```c
// No leak: malloc paired with free. Expected: zero findings.
#include <stdlib.h>
int main() {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: allocation
    *p = 42;
    int val = *p;
    free(p);                               // SANITIZER: deallocation
    return val;
}
```

## Worked Example 2: Use-After-Free (MayReach)

**Bad** -- `my_uaf_bad.c`:
```c
#include <stdlib.h>
#include <string.h>
void process() {
    char *buf = (char *)malloc(64);
    strcpy(buf, "hello");
    free(buf);                  // SOURCE: deallocation
    char c = buf[0];            // SINK: dereference after free
}
int main() { process(); return 0; }
```

**Good** -- `my_uaf_good.c`:
```c
#include <stdlib.h>
#include <string.h>
void process() {
    char *buf = (char *)malloc(64);
    strcpy(buf, "hello");
    char c = buf[0];            // OK: use while still allocated
    free(buf);
}
int main() { process(); return 0; }
```

## Worked Example 3: Double-Free (MultiReach)

**Bad** -- `my_df_bad.c`:
```c
#include <stdlib.h>
void process() {
    int *p = (int *)malloc(sizeof(int));  // SOURCE: allocation
    *p = 42;
    free(p);                               // SINK 1: first free
    free(p);                               // SINK 2: second free (BUG)
}
int main() { process(); return 0; }
```

**Good** -- `my_df_good.c`:
```c
#include <stdlib.h>
void process() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
    p = NULL;  // OK: prevents accidental reuse
}
int main() { process(); return 0; }
```

---

## E2E Test Pattern (Rust)

Tests live in `crates/saf-analysis/tests/checker_e2e.rs`. The `build_svfg` helper (already in that file) builds the full pipeline: `DefUseGraph` + `CallGraph` + PTA + MSSA + SVFG.

```rust
use saf_analysis::checkers::{self, ResourceTable, SolverConfig, run_checker};
use saf_test_utils::load_ll_fixture;

#[test]
fn my_checker_bad_reports_finding() {
    let module = load_ll_fixture("my_checker_bad");
    let svfg = build_svfg(&module);  // reuse helper from checker_e2e.rs
    let table = ResourceTable::new();
    let config = SolverConfig::default();
    let spec = checkers::builtin_checker("memory-leak").expect("unknown checker");
    let result = run_checker(&spec, &module, &svfg, &table, &config);

    assert!(
        !result.findings.is_empty(),
        "bad variant must produce at least one finding"
    );
}

#[test]
fn my_checker_good_reports_nothing() {
    let module = load_ll_fixture("my_checker_good");
    let svfg = build_svfg(&module);
    let table = ResourceTable::new();
    let config = SolverConfig::default();
    let spec = checkers::builtin_checker("memory-leak").expect("unknown checker");
    let result = run_checker(&spec, &module, &svfg, &table, &config);

    assert!(
        result.findings.is_empty(),
        "good variant must produce zero findings, got: {:?}",
        result.findings.iter().map(|f| &f.message).collect::<Vec<_>>()
    );
}
```

Key points:
- `load_ll_fixture("name")` loads `tests/fixtures/llvm/e2e/name.ll`
- `builtin_checker("name")` returns a built-in spec; for custom checkers build a `CheckerSpec` directly
- `run_checker()` signature: `(&spec, &module, &svfg, &table, &config)`
- Bad variant: `!findings.is_empty()`; good variant: `findings.is_empty()`
- Always print finding details in assertion failure messages

---

## E2E Test Pattern (Python)

For custom checkers via `saf.Project.check_custom()`:

```python
import os, unittest, saf
FIXTURE = lambda n: os.path.join(os.path.dirname(__file__),
    "..", "..", "tests", "fixtures", "llvm", "e2e", f"{n}.ll")

class TestMyChecker(unittest.TestCase):
    def _run(self, fixture: str) -> list:
        proj = saf.Project.open(FIXTURE(fixture))
        return proj.check_custom("my-checker", mode="never_reach_sink",
            source_role="allocator", source_match_return=True,
            sink_is_exit=True, sanitizer_role="deallocator",
            sanitizer_match_return=False)

    def test_bad_variant(self) -> None:
        self.assertGreater(len(self._run("my_checker_bad")), 0)

    def test_good_variant(self) -> None:
        self.assertEqual(len(self._run("my_checker_good")), 0)
```

Run: `docker compose run --rm dev sh -c 'uv run pytest python/tests/test_my_checker.py -v'`

---

## Taint-Specific Test Patterns

**Bad** -- tainted input reaches dangerous sink without sanitization:
```c
#include <stdlib.h>
int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *cmd = argv[1];       // SOURCE: user-controlled input
    return system(cmd);        // SINK: OS command execution (BUG)
}
```

**Good** -- sanitizer function blocks the taint flow:
```c
#include <stdlib.h>
#include <string.h>
char *sanitize(const char *in) { /* strip dangerous chars */ }
int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *safe = sanitize(argv[1]);  // SANITIZER
    return system(safe);             // SINK (safe after sanitization)
}
```

See `tests/programs/c/command_injection.c` and `taint_sanitized.c` for full examples.

## Typestate-Specific Test Patterns

**Bad** -- exits in non-accepting state:
```c
#include <stdio.h>
void leak_file(void) {
    FILE *fp = fopen("data.txt", "r");  // init -> opened
    fread(NULL, 1, 1, fp);
    // BUG: missing fclose -- exits in "opened" (non-accepting)
}
```

**Good** -- all transitions valid, ends in accepting state:
```c
#include <stdio.h>
void correct_usage(void) {
    FILE *fp = fopen("data.txt", "r");  // init -> opened
    fread(NULL, 1, 1, fp);
    fclose(fp);                         // opened -> closed (accepting)
}
```

See `tests/programs/c/typestate_file_leak.c` and `typestate_correct.c` for full examples.

---

## Checklist

Before submitting a checker with tests:

1. Both C files compile with `clang -S -emit-llvm -g -O0`
2. Bad variant e2e test asserts `!findings.is_empty()`
3. Good variant e2e test asserts `findings.is_empty()`
4. Assertion messages include diagnostic detail on failure
5. `make test` passes with both new tests
6. Both `.c` source files and `.ll` compiled fixtures are committed
