---
name: saf-feature-dev
description: Use when developing new SAF features — frontends, core analysis improvements, Python SDK extensions, or CLI features — to follow the project's structured development workflow
---

# SAF Feature Development Skill

Guides you through SAF's 8-phase development workflow for four feature types: Frontend, Core Analysis, Python SDK, and CLI. Enforces SAF-specific invariants (determinism, AIR-only analysis, BLAKE3 IDs) that generic coding misses.

## Feature Type Routing

| Type | Primary Crates | Key Entry Points |
|---|---|---|
| **Frontend** | `saf-frontends`, `saf-core` | `Frontend` trait in `api.rs`, AIR types in `air.rs` |
| **Core Analysis** | `saf-analysis`, `saf-core` | PTA in `pta/extract.rs`, pipeline in `pipeline.rs` |
| **Python SDK** | `saf-python` + exposed crate | Registration in `lib.rs`, tests in `python/tests/` |
| **CLI** | `saf-cli` | `commands.rs`, `driver.rs` |

## Workflow Phases

### Phase 1: Context Loading
Read `plans/PROGRESS.md` and relevant SRS sections (`docs/static_analyzer_factory_srs.md`) with the Read tool. Classify feature type from the table above and confirm.

### Phase 2: Codebase Exploration
Launch 2-3 Agent subagents (`subagent_type: Explore`) for parallel discovery. Load `references/feature-type-guides.md` for type-specific prompts. No code yet -- build a mental model first.

### Phase 3: Clarifying Questions
Load `references/saf-invariants.md` for the concern checklist. Evaluate: determinism, cross-crate impact, Python exposure, benchmarks, AIR compatibility, IDs, constraint extraction, no-SVF. Present all questions and wait for answers.

### Phase 4: Plan and Design
Launch Agent (`subagent_type: feature-dev:code-architect`) to explore 2-3 approaches against SAF constraints. Write plan to `plans/NNN-<topic>.md`, update `PROGRESS.md`.

### Phase 5: Test First
Load `references/e2e-testing-guide.md`. Write C programs, compile to IR inside Docker via Bash, write failing tests using `load_ll_fixture()`. Use specific assertions, not counts. Verify failure with `make test`.

### Phase 6: Implementation
Load `references/saf-log-guide.md` for `SAF_LOG` instrumentation. Implement incrementally; run `make fmt && make lint` via Bash after each unit (always fmt before lint). Add `saf_log!` at decision points. Docker builds only.

### Phase 7: Validation
Launch Agent (`subagent_type: feature-dev:code-reviewer`) checking: determinism violations, missing constraint entry points, `.unwrap()` in libraries, missing docs, unguarded casts. Run `make test 2>&1 | tee /tmp/test-output.txt` via Bash. Run benchmarks if analysis changed. Never re-run expensive commands to grep differently.

### Phase 8: Wrap-up
Update `plans/PROGRESS.md` (status, session log, next steps) via Read/Edit tools. Summarize deliverables, decisions, and remaining work.

## Tool Mappings

| Action | Tool |
|---|---|
| Search code | Grep, Glob |
| Read files | Read |
| Builds/tests | Bash (`make test`, `make fmt`, `docker compose run ...`) |
| Parallel exploration | Agent (`subagent_type: Explore`) |
| Architecture | Agent (`subagent_type: feature-dev:code-architect`) |
| Code review | Agent (`subagent_type: feature-dev:code-reviewer`) |

## Key Reminders

- **Docker only**: LLVM 18 is in the container. Only `saf-core` builds locally.
- **Determinism**: `BTreeMap`/`BTreeSet` always. Never `HashMap`/`HashSet`. Exception: `IndexMap`/`FxHashMap` permitted in documented PTA hot paths where output order is normalized afterward.
- **PTA triple-update**: New constraint steps must appear in all three extraction functions.
- **`cargo-nextest`**: Grep for `Summary` or `passed`, not `test result:`.
- Load detailed references from `references/` on demand.
