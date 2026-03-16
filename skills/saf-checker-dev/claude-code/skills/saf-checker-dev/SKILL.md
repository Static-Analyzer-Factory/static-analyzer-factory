---
name: saf-checker-dev
description: Use when creating new SAF bug-finding checkers, taint analysis rules, resource leak detectors, or typestate specifications
---

# SAF Checker Development Skill

Guides spec-first development of bug-finding checkers in SAF. Checkers are classified into three tiers -- try declarative first, escalate only when needed.

## Checker Tier Routing

| Tier | When to Use | What You Write |
|---|---|---|
| **1 - Declarative** | Source/sink/sanitizer expressible with existing `SitePattern` variants | `CheckerSpec` only (no Rust code) |
| **2 - Typestate** | Bug involves ordered state transitions (open->use->close) | `TypestateSpec` with state machine |
| **3 - Custom** | Need new `SitePattern` variant or `CustomPredicate` | Rust code changes in `spec.rs`, `site_classifier.rs`, solver |

All 9 built-in checkers are Tier 1. Start there.

## Workflow Phases

### Phase 1: Understand the Bug Pattern
Ask the user for: source (where bad values originate), sink (where the bug manifests), sanitizer (what prevents it), CWE number, and severity.

### Phase 2: Classify Tier
Load `references/checker-types-guide.md` with Read. Use the decision tree and `SitePattern` catalog to determine if existing patterns suffice (Tier 1), a state machine is needed (Tier 2), or new Rust code is required (Tier 3).

### Phase 3: Explore Existing Checkers
Read `crates/saf-analysis/src/checkers/spec.rs` to find the closest built-in checker. Use Grep to search for similar `SitePattern` usage across `checkers/`. Reuse patterns where possible.

### Phase 4: Write the Spec
Load `references/spec-authoring-guide.md` with Read. For Tier 1: compose a `CheckerSpec` with the right `ReachabilityMode`, sources, sinks, and sanitizers. For Tier 2: define a `TypestateSpec`. For Tier 3: implement new `SitePattern` variants in Rust.

### Phase 5: Create Test Cases
Load `references/test-case-guide.md` with Read. Write paired C programs (bad variant with the bug, good variant without). Compile to LLVM IR inside Docker via Bash:
```
docker compose run --rm dev sh -c 'clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll'
```
Write e2e tests asserting bad variant produces findings and good variant produces none.

### Phase 6: Run and Debug
Run tests via Bash: `make fmt && make test 2>&1 | tee /tmp/test-output.txt`. On failures, enable debug logging:
```
docker compose run --rm -e SAF_LOG="checker[reasoning,path,result]" dev sh -c 'cargo nextest run <test_name>'
```
Load `references/saf-log-guide.md` for log DSL details.

### Phase 7: Refine
Iterate: fix false positives by adding sanitizers, fix false negatives by broadening sources/sinks. Re-run tests after each change. Use `SAF_LOG=pta::solve[pts]` for pointer analysis issues.

### Phase 8: Export and Register
For built-in checkers: Edit `spec.rs` to add to `builtin_checkers()` and `builtin_checker_names()`. For YAML-based: write spec file to `share/saf/specs/`. Update any relevant documentation.

## Key Reminders

- **Spec-first**: Always try Tier 1 declarative before writing Rust code.
- **Docker only**: All builds and tests run inside Docker (`make test`, `make shell`).
- **SAF_LOG**: Use `checker[reasoning,path,result]` to trace checker decisions.
- **Paired tests**: Every checker needs both bad and good variant test programs.
- **Shared references** available in `references/`: `saf-invariants.md`, `saf-log-guide.md`, `e2e-testing-guide.md`.
