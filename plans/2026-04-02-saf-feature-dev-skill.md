# SAF Feature Dev Skill — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a distributable coding-agent skill that guides external contributors through SAF feature development (frontends, core analysis, Python SDK, CLI).

**Architecture:** Single source of truth in `core/` with platform-agnostic workflow + references. Thin wrappers for Claude Code (plugin with SKILL.md) and Codex (self-contained AGENTS.md fragment that users append to their project). Build script syncs core → platform dirs.

**Tech Stack:** Markdown, shell script, Claude Code plugin format (`.claude-plugin/plugin.json` + `skills/`), Codex AGENTS.md conventions.

**Repo location:** `/home/yuekang-li/projects/saf-dev-skills/`

---

## Task 1: Initialize Repo Structure

**Files:**
- Create: `README.md` (placeholder)
- Create: `LICENSE`
- Create: `core/` directory
- Create: `claude-code/` directory
- Create: `codex/` directory
- Create: `scripts/` directory

**Step 1: Create repo and directory skeleton**

```bash
cd /home/yuekang-li/projects
mkdir -p saf-dev-skills/{core/references,claude-code/.claude-plugin,claude-code/skills/saf-feature-dev/references,codex,scripts}
cd saf-dev-skills
git init
```

**Step 2: Create LICENSE (MIT)**

Create `LICENSE` with MIT license, copyright 2026, author from git config.

**Step 3: Create placeholder README**

Create `README.md`:
```markdown
# SAF Dev Skills

Coding-agent skills for developing features in the [SAF (Static Analyzer Factory)](https://github.com/nickel-lang/saf) project.

## Available Skills

- **saf-feature-dev** — Guided workflow for adding frontends, core analysis improvements, Python SDK extensions, and CLI features to SAF.

## Installation

### Claude Code

```bash
claude mcp add-plugin saf-dev-skills /path/to/saf-dev-skills/claude-code
```

### Codex

Append the contents of `codex/saf-feature-dev-workflow.md` to your project's `AGENTS.md`.

## Building from Source

```bash
./scripts/build.sh
```

This syncs shared content from `core/` into platform-specific directories.
```

**Step 4: Initial commit**

```bash
git add -A
git commit -m "chore: initialize repo structure"
```

---

## Task 2: Write Core Workflow

**Files:**
- Create: `core/workflow.md`

This is the platform-agnostic 8-phase workflow. Uses generic language ("search the codebase for X", "read file Y") instead of tool-specific names. Each phase has clear entry/exit criteria and decision points.

**Step 1: Write `core/workflow.md`**

The file should contain:

```markdown
# SAF Feature Development Workflow

## Scope

This workflow covers adding or modifying SAF infrastructure:
- **Frontends** — new input formats (e.g., beyond LLVM IR)
- **Core analysis** — graph builders, PTA, value-flow, abstract interpretation
- **Python SDK** — PyO3 bindings exposing Rust functionality to Python
- **CLI** — new commands, output formats, flags

This workflow does NOT cover writing checkers/analyzers (use saf-checker-dev for that).

---

## Phase 1: Context Loading

**Goal:** Understand current project state and classify the feature.

**Actions:**
1. Read `plans/PROGRESS.md` to understand what's been built and what's in flight
2. Read relevant sections of `docs/static_analyzer_factory_srs.md` for requirements context
3. Classify the feature into one of four types:

| Feature Type | Primary Crates | Key Files to Find |
|---|---|---|
| Frontend | `saf-frontends`, `saf-core` | `Frontend` trait in `saf-frontends/src/api.rs`, `AirBundle` in `saf-core` |
| Core Analysis | `saf-analysis`, `saf-core` | Graph builders in `saf-analysis/src/`, PTA in `saf-analysis/src/pta/` |
| Python SDK | `saf-python` + underlying crate | PyO3 bindings in `saf-python/src/`, Python tests in `python/tests/` |
| CLI | `saf-cli` | Commands in `saf-cli/src/commands.rs`, driver in `saf-cli/src/driver.rs` |

4. Announce classification to the contributor: "This looks like a **[type]** feature touching `[crates]`. I'll focus exploration on [area]. Sound right?"
5. Wait for confirmation before proceeding.

---

## Phase 2: Codebase Exploration

**Goal:** Deeply understand relevant existing code and patterns.

**Actions:**
1. Launch 2-3 parallel exploration tasks, tailored by feature type:

   **Frontend:**
   - "Trace how the LLVM frontend implements the `Frontend` trait, how it constructs `AirBundle`, and how `saf-frontends/tests/` exercises it"
   - "Map the AIR types in `saf-core/src/air/` — what does `AirBundle` contain, what are the instruction variants, how are IDs generated"

   **Core Analysis:**
   - "Trace the PTA constraint extraction pipeline — find all functions named `extract_constraints`, `extract_constraints_reachable`, and `extract_intraprocedural_constraints` — and how the solver consumes them"
   - "Map the graph builder pipeline: CFG → CallGraph → DefUse → ValueFlow → SVFG. What trait does each implement? Where are they built?"
   - "Trace existing e2e tests in `crates/saf-analysis/tests/*_e2e.rs` — how do they load fixtures, run analysis, and assert results?"

   **Python SDK:**
   - "Trace existing PyO3 bindings in `saf-python/src/` — how do `#[pyfunction]` and `#[pymethods]` wrap Rust types? What patterns are used for error conversion?"
   - "Map the Python test suite in `python/tests/` — what does the test infrastructure look like?"

   **CLI:**
   - "Trace the CLI command dispatch: how does `main()` → `commands.rs` → `driver.rs` wire frontends to analysis?"
   - "Map existing integration tests and how they exercise CLI commands"

2. Read all key files identified by exploration
3. Present a findings summary to the contributor covering:
   - Relevant patterns and abstractions
   - Extension points for the new feature
   - Similar existing implementations to use as reference

---

## Phase 3: Clarifying Questions

**Goal:** Fill in all gaps before designing.

**CRITICAL: Do not skip this phase.** Present all questions before proceeding to design.

**Actions:**
1. Review codebase findings and original feature request
2. Check for SAF-specific concerns:
   - **Determinism:** Will new data structures need `BTreeMap`/`BTreeSet` for deterministic iteration?
   - **Cross-crate impact:** Does this change ripple through the dependency graph (`saf-core` → `saf-frontends` → `saf-analysis` → `saf-cli`/`saf-python`)?
   - **Python exposure:** Should this feature be accessible via the Python SDK?
   - **Benchmark impact:** Could this change affect PTA precision or performance? Should PTABen/Juliet be run?
   - **AIR compatibility:** Does this operate only on AIR types, not frontend-specific types?
   - **ID system:** Does this introduce new entity types needing BLAKE3-based IDs?
3. Present all questions in an organized list
4. Wait for answers before proceeding

---

## Phase 4: Plan & Design

**Goal:** Design the implementation approach and write a plan.

**Actions:**
1. Explore 2-3 implementation approaches with different tradeoffs, keeping SAF constraints in mind:
   - All analysis must operate on AIR only (never frontend-specific types)
   - No SVF code reuse — independent implementations only
   - Determinism required (byte-identical outputs for identical inputs)
   - All IDs are u128, BLAKE3-derived
2. Present approaches to contributor with:
   - Brief summary of each
   - Tradeoffs comparison
   - Your recommendation with reasoning
3. After contributor picks an approach, write the plan:
   - Save to `plans/NNN-<topic>.md` (increment NNN from the last existing plan)
   - Update `plans/PROGRESS.md` with new plan entry in Plans Index (status: "approved")
   - Update "Next Steps" section if appropriate

---

## Phase 5: Test First (E2E Preferred)

**Goal:** Create failing tests before writing implementation code.

**Actions:**
1. Choose the test strategy based on feature type:

   **Frontend / Core Analysis (E2E preferred):**
   - Write a C test program in `tests/programs/c/<name>.c` that exercises the expected behavior
   - Compile to LLVM IR inside Docker: `clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll`
   - Write an e2e test in `crates/saf-analysis/tests/<name>_e2e.rs` using `load_ll_fixture("<name>.ll")`
   - The test should assert the expected behavior and FAIL because it doesn't exist yet

   **Python SDK:**
   - Write a Python test in `python/tests/test_<feature>.py` that imports the new functionality
   - Test should fail because the binding doesn't exist yet

   **CLI:**
   - Write an integration test that invokes the new CLI command/flag with fixture input
   - Test should fail because the command doesn't exist yet

2. Run the test to verify it fails: `make test` (inside Docker)
3. If e2e doesn't make sense for this feature (e.g., pure config, internal refactoring), write a unit test instead and document why e2e was skipped

**Preference hierarchy:** e2e with real C programs > e2e with hand-written .ll > integration test > unit test. Default to the highest level that makes sense.

---

## Phase 6: Implementation

**Goal:** Build the feature.

**DO NOT START without contributor approval of the plan from Phase 4.**

**Actions:**
1. All builds and tests run inside Docker — never run `cargo build` or `cargo test` locally for LLVM-dependent crates (`saf-frontends`, `saf-analysis`, `saf-python`, `saf-cli`). Only `saf-core` is safe to build locally.
2. Instrument new code with `saf_log!` calls during development:
   ```rust
   use saf_core::saf_log;
   saf_log!(module::phase, tag, "narrative"; key=value);
   ```
   Register new modules/phases in `saf-core/src/lib.rs` if needed.
3. Run `make fmt && make lint` frequently — don't accumulate lint debt
4. When something goes wrong, use `SAF_LOG` to trace BEFORE guessing at fixes:
   ```bash
   docker compose run --rm -e SAF_LOG="module::phase[tag]" dev sh -c 'cargo test ...'
   ```
5. Follow SAF coding conventions:
   - `thiserror` for library errors, `anyhow` only in binaries
   - No `.unwrap()` in library code — use `Result` or `expect("reason")`
   - `BTreeMap`/`BTreeSet` for deterministic iteration
   - Doc comments on all public items
   - Pedantic clippy (wrap code identifiers in backticks in doc comments)
6. Commit after each meaningful unit of progress

---

## Phase 7: Validation

**Goal:** Verify correctness and catch regressions.

**Actions:**
1. Run the full test suite: `make test`
   - Capture output: `make test 2>&1 | tee /tmp/test-output.txt`
   - SAF uses `cargo-nextest` — grep for `Summary` or `passed`, not `test result`
2. Run `make fmt && make lint` for final check
3. **If core analysis was changed:** Run PTABen benchmarks to check for regressions:
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
   ```
   Read results on host and compare against known baselines.
4. **If touching checkers indirectly:** Run relevant Juliet CWE categories:
   ```bash
   make test-juliet CWE=CWE476  # example
   ```
5. Review code for SAF-specific issues:
   - Determinism violations (`HashMap`/`HashSet` usage)
   - Missing constraint extraction entry points (if PTA-related: update `extract_constraints`, `extract_constraints_reachable`, AND `extract_intraprocedural_constraints`)
   - `.unwrap()` in library code
   - Missing doc comments on public items
   - Cast lints without documented invariants
6. Present findings to contributor, ask what to fix

---

## Phase 8: Wrap-up

**Goal:** Document what was accomplished.

**Actions:**
1. Update `plans/PROGRESS.md`:
   - Set plan status to "done" (or "in-progress" with notes if partial)
   - Update "Next Steps" for the next session
   - Append to Session Log with summary of work done
2. Summarize to contributor:
   - What was built
   - Key decisions made
   - Files modified/created
   - Any remaining work or follow-ups
3. If benchmarks were run, include before/after comparison
```

**Step 2: Commit**

```bash
git add core/workflow.md
git commit -m "feat: add core workflow — 8-phase SAF feature development guide"
```

---

## Task 3: Write Core References — SAF Invariants

**Files:**
- Create: `core/references/saf-invariants.md`

**Step 1: Write `core/references/saf-invariants.md`**

This file contains structural knowledge that rarely changes — the stable anchors an agent needs regardless of feature type. Extract from SAF's CLAUDE.md but only the invariants, not the full conventions (those live in the repo's CLAUDE.md/AGENTS.md).

Content should cover:
- **Crate dependency graph** — which crate depends on which, and the rule that changes propagate downward
- **Docker requirement** — LLVM 18 is only inside Docker; list which crates need it
- **Determinism requirement** — `BTreeMap`/`BTreeSet`, BLAKE3 IDs, byte-identical output (NFR-DET-001)
- **AIR-only rule** — analysis operates only on AIR, never on frontend-specific types (NFR-EXT-001)
- **No SVF reuse** — independent implementations only (REQ-IP-001)
- **ID system** — all IDs are u128, BLAKE3-derived, serialized as `0x` + 32 hex chars
- **Data flow pipeline** — Input → Frontend → AirBundle → Graphs → PTA → ValueFlow → Queries → Export
- **Key constraint**: PTA constraint extraction triple-update rule

Keep it under 300 lines. Point to code paths, don't duplicate code.

**Step 2: Commit**

```bash
git add core/references/saf-invariants.md
git commit -m "feat: add saf-invariants reference — structural knowledge for agents"
```

---

## Task 4: Write Core References — Feature Type Guides

**Files:**
- Create: `core/references/feature-type-guides.md`

**Step 1: Write `core/references/feature-type-guides.md`**

Per-feature-type checklists for what to explore, what to watch out for, and what to validate. Organized as four sections:

**Frontend Guide:**
- Must implement `Frontend` trait from `saf-frontends/src/api.rs`
- Must produce `AirBundle` — never leak frontend-specific types
- Reference implementation: LLVM frontend in `saf-frontends/src/llvm/`
- Tests: compile input format → ingest → verify AIR round-trip
- Watch out: instruction coverage, metadata preservation, debug info mapping

**Core Analysis Guide:**
- Understand the graph builder pipeline (CFG → CG → DefUse → VFG → SVFG)
- If touching PTA: the triple-update rule for constraint extraction
- If adding a new graph type: follow the PropertyGraph export format
- Run PTABen/Juliet benchmarks before and after
- Watch out: determinism, IndexMap in hot paths only, BTreeMap in public API

**Python SDK Guide:**
- PyO3 patterns: `#[pyfunction]` for free functions, `#[pymethods]` for struct methods
- Required clippy allows: `unnecessary_wraps`, `unused_self`, `needless_pass_by_value`
- Probe real API output before writing consumer code (`print(type(x), x)`)
- PropertyGraph export format is shared across all graph types
- Test in `python/tests/` with real analysis pipelines

**CLI Guide:**
- Commands defined in `saf-cli/src/commands.rs` with clap arg structs
- `AnalysisDriver` in `saf-cli/src/driver.rs` orchestrates the pipeline
- Output formats: JSON, SARIF, PropertyGraph
- Test with fixture inputs and expected outputs

Keep each section focused on what's unique to that feature type.

**Step 2: Commit**

```bash
git add core/references/feature-type-guides.md
git commit -m "feat: add feature-type-guides reference — per-type exploration checklists"
```

---

## Task 5: Write Core References — E2E Testing Guide

**Files:**
- Create: `core/references/e2e-testing-guide.md`

**Step 1: Write `core/references/e2e-testing-guide.md`**

Step-by-step recipe for creating e2e tests in SAF, with concrete examples.

Content:
1. **Write a C test program** — where to put it (`tests/programs/c/<name>.c`), what to include (minimal program exercising the behavior, with comments marking expected results)
2. **Compile to LLVM IR** — must be inside Docker: `clang -S -emit-llvm -g -O0 tests/programs/c/<name>.c -o tests/fixtures/llvm/e2e/<name>.ll`
3. **Write the e2e test** — in `crates/saf-analysis/tests/<name>_e2e.rs`, using `load_ll_fixture("<name>.ll")`. Show the pattern:
   ```rust
   use saf_analysis::test_utils::load_ll_fixture;
   
   #[test]
   fn test_feature_name() {
       let bundle = load_ll_fixture("name.ll");
       // Build analysis artifacts
       // Assert expected behavior
   }
   ```
4. **Assertion style** — prefer specific assertions (`assert!(results.iter().any(|r| ...))`) over count assertions (`assert_eq!(results.len(), N)`)
5. **Running tests** — `make test` runs everything; for targeted runs: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis test_feature_name'`
6. **Python e2e tests** — equivalent pattern for `python/tests/` with `import saf`

Include one complete worked example (a simple null-pointer-related test case).

**Step 2: Commit**

```bash
git add core/references/e2e-testing-guide.md
git commit -m "feat: add e2e-testing-guide reference — C-to-LLVM-to-assertion recipe"
```

---

## Task 6: Write Core References — SAF Log Guide

**Files:**
- Create: `core/references/saf-log-guide.md`

**Step 1: Write `core/references/saf-log-guide.md`**

How to instrument SAF code with `saf_log!` and use `SAF_LOG` for debugging during development.

Content:
1. **Adding logging to new code:**
   ```rust
   use saf_core::saf_log;
   // Full form: narrative + key-values
   saf_log!(pta::solve, worklist, "pts grew"; val=node_id, delta=&added);
   // Narrative only
   saf_log!(pta::solve, convergence, "fixpoint reached");
   // Keys only
   saf_log!(pta::solve, stats; iter=12, worklist=342);
   ```
2. **Registering new modules/phases** — must be registered in `saf-core/src/lib.rs` via `saf_log_module!` invocation
3. **Enabling at runtime:**
   ```bash
   docker compose run --rm -e SAF_LOG="module::phase[tag]" dev sh -c 'cargo run ...'
   docker compose run --rm -e SAF_LOG="all" dev sh -c '...'
   docker compose run --rm -e SAF_LOG="all,-pta::solve[worklist]" dev sh -c '...'
   ```
4. **File output** to avoid mixing with stderr:
   ```bash
   docker compose run --rm -e SAF_LOG="pta" -e SAF_LOG_FILE=/tmp/saf.log dev sh -c '...'
   ```
5. **Output format:** `[module::phase][tag] narrative | key=value key=value`
6. **Value types:** `0x1a2b` (ID), `{0x1a,0x2b}` (set), `[0x1a,0x2b]` (list), `bb1->bb3` (path), `+{0x3c}`/`-{0x3c}` (delta), `12/50` (ratio)
7. **Common debug workflows by symptom:**
   - Wrong PTA result → `SAF_LOG=pta::solve[pts,worklist]`
   - Missing callgraph edge → `SAF_LOG=callgraph[edge],pta::solve[pts]`
   - False positive/negative → `SAF_LOG=checker[reasoning,path,result]`
   - Slow analysis → `SAF_LOG=pta::solve[stats,convergence]`
8. **Key principle:** Use `SAF_LOG` to trace BEFORE guessing at fixes

**Step 2: Commit**

```bash
git add core/references/saf-log-guide.md
git commit -m "feat: add saf-log-guide reference — debugging with SAF_LOG"
```

---

## Task 7: Write Claude Code Plugin

**Files:**
- Create: `claude-code/.claude-plugin/plugin.json`
- Create: `claude-code/skills/saf-feature-dev/SKILL.md`
- Copy: `claude-code/skills/saf-feature-dev/references/` (from `core/references/`)

**Step 1: Create plugin manifest**

Create `claude-code/.claude-plugin/plugin.json`:
```json
{
  "name": "saf-dev-skills",
  "description": "Coding-agent skills for developing features in the SAF (Static Analyzer Factory) project",
  "author": {
    "name": "SAF Team"
  },
  "version": "0.1.0",
  "license": "MIT",
  "keywords": ["saf", "static-analysis", "rust", "feature-development"]
}
```

**Step 2: Write SKILL.md**

Create `claude-code/skills/saf-feature-dev/SKILL.md` with:
- YAML frontmatter: `name: saf-feature-dev`, `description: Use when developing new SAF features — frontends, core analysis improvements, Python SDK extensions, or CLI features — to follow the project's structured development workflow`
- Wrap `core/workflow.md` content but with Claude Code tool names:
  - "Search the codebase" → "Use Grep/Glob to find..."
  - "Launch parallel exploration" → "Launch Agent with `subagent_type: Explore`"
  - "Read file" → "Use Read tool"
  - "Run tests" → "Use Bash tool: `make test`"
  - "Launch architect agents" → "Launch Agent with `subagent_type: feature-dev:code-architect`"
  - "Launch code review" → "Launch Agent with `subagent_type: feature-dev:code-reviewer`"
- Reference supporting files: "Load `references/saf-invariants.md` for structural knowledge", etc.
- Include the feature-type routing table inline (small enough)
- Keep SKILL.md under 500 words (per best practices), defer detail to references

**Step 3: Copy references**

```bash
cp core/references/*.md claude-code/skills/saf-feature-dev/references/
```

**Step 4: Commit**

```bash
git add claude-code/
git commit -m "feat: add Claude Code plugin — saf-feature-dev skill"
```

---

## Task 8: Write Codex Format

**Files:**
- Create: `codex/saf-feature-dev-workflow.md`

**Step 1: Write the Codex workflow file**

Create `codex/saf-feature-dev-workflow.md` — a self-contained markdown section that users append to their project's `AGENTS.md`.

Since Codex doesn't support file references, this must be a single document that inlines all content. Structure:

```markdown
## SAF Feature Development Workflow

> Append this section to your project's AGENTS.md for guided feature development.

[Full workflow from core/workflow.md, adapted:]
- No tool-specific names (Codex uses shell commands natively)
- "Search for X" instead of "Use Grep tool"  
- "Read file Y" instead of "Use Read tool"
- "Run `make test`" (same — Codex uses shell directly)
- Inline the most critical reference content (invariants, feature-type routing table)
- For detailed references, point to the files in the saf-dev-skills repo: "For detailed e2e testing instructions, see saf-dev-skills/core/references/e2e-testing-guide.md"
```

Keep under 32 KiB (Codex's AGENTS.md limit) — the user's existing AGENTS.md is ~13 KiB, so this fragment should be under ~15 KiB to fit alongside it.

**Step 2: Commit**

```bash
git add codex/
git commit -m "feat: add Codex format — self-contained AGENTS.md workflow section"
```

---

## Task 9: Write Build Script

**Files:**
- Create: `scripts/build.sh`

**Step 1: Write `scripts/build.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."

echo "=== Syncing core references to Claude Code plugin ==="
mkdir -p "$ROOT/claude-code/skills/saf-feature-dev/references"
cp "$ROOT/core/references/"*.md "$ROOT/claude-code/skills/saf-feature-dev/references/"

echo "=== Rebuilding Codex workflow ==="
# The Codex file is manually maintained (it inlines + adapts content)
# Just verify it exists
if [ ! -f "$ROOT/codex/saf-feature-dev-workflow.md" ]; then
    echo "WARNING: codex/saf-feature-dev-workflow.md not found"
    exit 1
fi

echo "=== Checking for stale references ==="
# Verify no core reference is newer than its Claude Code copy
STALE=0
for f in "$ROOT/core/references/"*.md; do
    base=$(basename "$f")
    cc="$ROOT/claude-code/skills/saf-feature-dev/references/$base"
    if [ "$f" -nt "$cc" ]; then
        echo "STALE: $base (core is newer than claude-code copy)"
        STALE=1
    fi
done

if [ $STALE -eq 1 ]; then
    echo "References synced. Don't forget to review Codex file for manual sync."
else
    echo "All references up to date."
fi

echo "=== Done ==="
```

**Step 2: Make executable and commit**

```bash
chmod +x scripts/build.sh
git add scripts/
git commit -m "feat: add build script — syncs core references to platform dirs"
```

---

## Task 10: Write README with Installation Instructions

**Files:**
- Modify: `README.md` (replace placeholder from Task 1)

**Step 1: Write full README**

Replace the placeholder with complete documentation:
- Project description
- Available skills
- Installation instructions for Claude Code (with exact commands)
- Installation instructions for Codex (with copy-paste steps)
- Building from source
- Contributing guide (how to update core → rebuild)
- Link back to SAF repo

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add full README with installation instructions"
```

---

## Task 11: Test with Subagent — Claude Code Format

**Goal:** Verify the skill works by running a simulated scenario with a subagent.

**Step 1: Run a test scenario**

Launch a subagent (in the SAF repo, not saf-dev-skills) that has the skill loaded, with a prompt like:

> "I want to add a new AIR-JSON export format that writes the AirBundle to a compact binary format. Use the saf-feature-dev skill to guide me through the first 3 phases (Context Loading, Codebase Exploration, Clarifying Questions)."

**Step 2: Evaluate the result**

Check that the agent:
- Correctly classified the feature type (Frontend)
- Read PROGRESS.md and relevant SRS sections
- Launched appropriate exploration tasks
- Asked SAF-specific clarifying questions (determinism, AIR-only, Python exposure)
- Did NOT skip Phase 3

**Step 3: Document gaps and fix**

If the agent missed steps or made wrong assumptions, update the SKILL.md or references to close the gap.

**Step 4: Commit fixes**

```bash
git add -A
git commit -m "fix: address gaps found in subagent testing"
```

---

## Task 12: Test with Subagent — Codex Format

**Goal:** Verify the Codex workflow fragment is self-contained and usable.

**Step 1: Create a temporary AGENTS.md**

In a temp directory, concatenate the SAF project's AGENTS.md with the Codex workflow fragment:
```bash
cat /home/yuekang-li/projects/static-analyzer-factory/AGENTS.md codex/saf-feature-dev-workflow.md > /tmp/test-agents.md
```

**Step 2: Verify**

- Check total size is under 32 KiB
- Read through and verify no Claude Code-specific tool names leaked in
- Verify all critical information is self-contained (doesn't require loading separate files)

**Step 3: Document and commit any fixes**

```bash
git add -A
git commit -m "fix: address Codex format issues found in testing"
```

---

## Task 13: Final Verification and Tag

**Step 1: Run build script**

```bash
./scripts/build.sh
```

Verify no stale references.

**Step 2: Review all files**

Quick scan of every file for consistency:
- Core workflow phases match both platform wrappers
- No platform-specific language in core/
- No SAF convention duplication (conventions live in CLAUDE.md/AGENTS.md)
- References are accurate and point to real SAF code paths

**Step 3: Tag v0.1.0**

```bash
git tag -a v0.1.0 -m "Initial release: saf-feature-dev skill for Claude Code and Codex"
```
