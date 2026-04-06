# SAF Checker Development Skill

A coding-agent skill that guides AI assistants (and human contributors) through creating bug-finding checkers in SAF using a spec-first workflow.

## What It Does

SAF checkers are **spec-driven**: you define what to detect (source, sink, sanitizer) declaratively, and the SVFG reachability solver does the heavy lifting. This skill guides you through authoring checkers across three tiers of complexity:

| Tier | Approach | When to Use | Examples |
|---|---|---|---|
| **1. Declarative** | YAML/Python spec only | Source → sink reachability | Memory leak, null-deref, double-free |
| **2. Typestate** | State machine spec | Multi-state resource lifecycle | File open/close, lock/unlock |
| **3. Custom** | Rust code | Complex patterns beyond specs | Cross-function temporal reasoning |

## The 8 Phases

1. **Understand the Bug Pattern** — Identify source, sink, sanitizer, CWE mapping
2. **Classify the Tier** — Declarative, typestate, or custom
3. **Explore Existing Checkers** — Find similar patterns to reuse
4. **Write the Spec** — Author the checker specification
5. **Create Test Cases** — Write C programs with known bugs and clean variants
6. **Run & Debug** — Execute with `SAF_LOG=checker[reasoning,path,result]`
7. **Refine** — Iterative precision/recall improvement
8. **Export & Document** — SARIF output, documentation

## Installation

### Claude Code

```bash
claude plugin add /path/to/static-analyzer-factory/skills/saf-checker-dev/claude-code
```

The skill activates automatically when you work on SAF checkers or analyzers.

### Codex / Other Agents

Read `codex/saf-checker-dev-workflow.md` as context — it contains the full workflow in a single file.

## Included References

- **Checker types guide** — all 9 built-in checker categories with specs
- **Spec authoring guide** — YAML/Python spec format and solver modes
- **Test case guide** — writing good/bad C test variants for precision/recall
- **E2E testing guide** — compilation, Docker, benchmark validation
- **SAF_LOG guide** — debug logging for checker reasoning traces

## Updating

After editing files in `core/`, run `build.sh` to sync changes to all platform-specific formats:

```bash
./skills/saf-checker-dev/build.sh
```
