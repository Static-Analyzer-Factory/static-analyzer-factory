# Plan 183: SAF Feature Development Skill

## Goal

Create a distributable coding-agent skill (`saf-feature-dev`) that guides external contributors through SAF feature development. The skill provides a structured 8-phase workflow for adding frontends, core analysis improvements, Python SDK extensions, and CLI features. It does NOT cover checker/analyzer authoring (separate future skill).

## Distribution

Multi-format from the start — single source of truth with platform-specific wrappers:
- **Claude Code**: Plugin with `SKILL.md` + references
- **Codex**: Supplementary markdown referenced from the repo's existing `AGENTS.md`

The skill provides *workflow only*. Coding conventions stay in the repo's `CLAUDE.md` / `AGENTS.md`.

## Design Decisions

- **Audience**: External contributors using coding agents (Claude Code, Codex, etc.)
- **Interaction model**: Agent does implementation, contributor provides direction and approvals at key decision points
- **Knowledge approach**: Hybrid — embed structural invariants that rarely change (crate graph, Docker requirement, determinism, IDs), point to code for implementation details that evolve
- **Testing philosophy**: E2e tests preferred (C → .ll → analysis → assertions), unit tests when e2e doesn't make sense
- **Debugging**: `SAF_LOG` is a first-class debugging tool, not a last resort — instrument during development
- **Scope boundary**: Frontends, core analysis, Python SDK, CLI. Not checkers/analyzers.

## Feature Type Routing

Early in the workflow, the agent classifies the feature and tailors exploration/validation accordingly:

| Feature Type | Primary Crates | Exploration Focus | Validation |
|---|---|---|---|
| Frontend | `saf-frontends`, `saf-core` | `Frontend` trait, `AirBundle` construction, existing LLVM frontend as reference | E2e: new input format → AIR → verify round-trip |
| Core Analysis | `saf-analysis`, `saf-core` | Graph builders, PTA solver, value-flow, constraint extraction entry points | E2e tests + PTABen/Juliet benchmark regression check |
| Python SDK | `saf-python` + underlying crate | PyO3 binding patterns, existing `#[pyfunction]`/`#[pymethods]`, PropertyGraph export | Python e2e tests in `python/tests/` + probe real API output before writing consumer code |
| CLI | `saf-cli` | Clap arg patterns, output format handling, existing subcommands | Integration tests with fixture inputs |

## Workflow (8 Phases)

### Phase 1: Context Loading
- Read `plans/PROGRESS.md` to understand current project state
- Read relevant sections of `docs/static_analyzer_factory_srs.md` based on the feature request
- Identify feature type using the routing table
- Announce classification to contributor and confirm

### Phase 2: Codebase Exploration
- Launch 2-3 explorer agents with SAF-aware prompts based on feature type:
  - Frontend: "Trace how the LLVM frontend implements the `Frontend` trait, constructs `AirBundle`, and how tests exercise it"
  - Core analysis: "Trace the PTA constraint extraction pipeline — `extract_constraints`, `extract_constraints_reachable`, `extract_intraprocedural_constraints` — and how the solver consumes them"
  - Python SDK: "Trace existing PyO3 bindings in `saf-python`, how they wrap Rust types, and how Python tests exercise them"
  - CLI: "Trace existing CLI subcommands, how they wire frontends to analysis, and how integration tests work"
- Read all key files identified by explorers
- Present findings summary to contributor

### Phase 3: Clarifying Questions
- Review codebase findings and feature request
- Identify underspecified aspects with SAF-specific prompts:
  - Determinism: any new data structures need `BTreeMap`/`BTreeSet`?
  - Cross-crate impact: does this change ripple through the dependency graph?
  - Python exposure: should this be accessible via the SDK?
  - Benchmark impact: could this regress PTA precision or performance?
  - AIR compatibility: does this operate only on AIR, not frontend-specific types?
- Present all questions, wait for answers before proceeding

### Phase 4: Plan & Design
- Launch architect agents with SAF constraints baked in (AIR-only, determinism, no-SVF)
- Present 2-3 approaches with tradeoffs, recommend one
- After contributor picks approach, write plan to `plans/NNN-<topic>.md`
- Update `plans/PROGRESS.md` with new plan entry

### Phase 5: Test First (E2E Preferred)
- Before implementation, create the test harness:
  - Write C test program in `tests/programs/c/<name>.c` if applicable
  - Compile to `.ll` inside Docker: `clang -S -emit-llvm -g -O0 ...`
  - Write e2e test in `crates/saf-analysis/tests/<name>_e2e.rs` using `load_ll_fixture`
  - Test should fail (asserts expected behavior that doesn't exist yet)
- Python SDK: write Python test in `python/tests/` first
- CLI: write integration test with fixture input
- Default to e2e; explain if choosing unit tests instead

### Phase 6: Implementation
- All builds inside Docker (`make` commands or `docker compose run`)
- Instrument new code with `saf_log!` calls for debuggability
- Run `make fmt && make lint` frequently
- When something goes wrong: use `SAF_LOG` to trace before guessing at fixes
- Follow SAF conventions: `thiserror`, no `.unwrap()`, doc comments, pedantic clippy, `BTreeMap`/`BTreeSet`

### Phase 7: Validation
- Run full test suite: `make test`
- If core analysis changed: run PTABen benchmarks with `-o` for clean JSON, check regressions
- If touching checkers indirectly: run relevant Juliet CWE categories
- Launch code review agent focused on SAF-specific concerns:
  - Determinism violations (HashMap/HashSet)
  - Missing constraint extraction entry points (triple-update rule)
  - `.unwrap()` in library code
  - Missing doc comments on public items
  - Cast lints without documented invariants
- Present review findings, ask what to fix

### Phase 8: Wrap-up
- Update `plans/PROGRESS.md`: mark plan status, update "Next Steps", append session log
- Summarize: what was built, key decisions, files modified, remaining work
- Include benchmark before/after if applicable

## File Structure

```
saf-dev-skills/
  README.md                             # Overview, installation for each platform
  LICENSE

  # Shared content (platform-agnostic)
  core/
    workflow.md                         # The 8 phases in tool-agnostic language
    references/
      saf-invariants.md                 # Crate graph, Docker req, determinism, IDs, no-SVF, AIR-only
      feature-type-guides.md            # Per-type exploration checklists & watch-outs
      e2e-testing-guide.md              # C → .ll → e2e test recipe with examples
      saf-log-guide.md                  # saf_log! macro, SAF_LOG env var, common debug workflows

  # Claude Code plugin format
  claude-code/
    plugin.json                         # Plugin manifest
    skills/
      saf-feature-dev/
        SKILL.md                        # Wraps core/workflow.md with Claude Code tool names
        references/                     # Copies/symlinks from core/references/

  # Codex format
  codex/
    saf-feature-dev.md                  # Wraps core/workflow.md with Codex conventions

  # Build tooling
  scripts/
    build.sh                            # Generates platform-specific files from core/
```

## Implementation Steps

### Step 1: Create repo and shared core
- Initialize `saf-dev-skills` repo
- Write `core/workflow.md` — the 8 phases in platform-agnostic language
- Write all four reference files from SAF's CLAUDE.md and codebase knowledge

### Step 2: Claude Code plugin
- Create `plugin.json` manifest
- Write `SKILL.md` that wraps the core workflow with Claude Code tool names (Grep, Read, Agent with subagent_type, etc.)
- Copy references into the skill directory

### Step 3: Codex format
- Write `codex/saf-feature-dev.md` that wraps the core workflow with Codex conventions (shell commands instead of tool names)
- Add instructions for referencing from SAF repo's `AGENTS.md`

### Step 4: Build script
- `scripts/build.sh` that syncs core/ content into platform-specific directories
- Ensures references stay in sync

### Step 5: Documentation
- Write README.md with installation instructions for each platform
- Add usage examples
