# SAF Feature Development Skill

A coding-agent skill that guides AI assistants (and human contributors) through an 8-phase workflow for adding features to SAF.

## What It Does

When activated, the skill provides structured guidance for developing SAF features across four categories:

| Feature Type | Primary Crates | Examples |
|---|---|---|
| **Frontend** | `saf-frontends`, `saf-core` | New IR frontend, AIR extension |
| **Core Analysis** | `saf-analysis`, `saf-core` | New graph builder, PTA variant, solver |
| **Python SDK** | `saf-python` | New Python binding, query API |
| **CLI** | `saf-cli` | New command, output format |

## The 8 Phases

1. **Context Loading** — Read progress, classify feature type
2. **Codebase Exploration** — Locate relevant files and patterns
3. **Design & Planning** — Write implementation plan
4. **Test First** — Create E2E test with C fixture
5. **Implementation** — Write the feature code
6. **SAF_LOG Instrumentation** — Add structured debug logging
7. **Verification** — Run tests, lint, benchmarks
8. **Documentation** — Update docs, plans, PROGRESS.md

## Installation

### Claude Code

```bash
claude plugin add /path/to/static-analyzer-factory/skills/saf-feature-dev/claude-code
```

The skill activates automatically when you work on SAF features.

### Codex / Other Agents

Read `codex/saf-feature-dev-workflow.md` as context — it contains the full workflow in a single file.

## Included References

- **Feature type guides** — crate-specific patterns and file locations
- **E2E testing guide** — how to compile C fixtures and write integration tests
- **SAF_LOG guide** — structured debug logging DSL and common workflows
- **SAF invariants** — coding conventions, determinism requirements, clippy rules

## Updating

After editing files in `core/`, run `build.sh` to sync changes to all platform-specific formats:

```bash
./skills/saf-feature-dev/build.sh
```
