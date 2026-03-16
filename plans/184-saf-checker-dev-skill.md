# Plan 184: SAF Checker Dev Skill

## Goal

Create a distributable coding-agent skill (`saf-checker-dev`) that guides agents through creating SAF bug-finding checkers, taint rules, resource leak detectors, and typestate specifications. Spec-first workflow with three tiers: declarative (YAML + CheckerSpec), typestate (TypestateSpec), and custom patterns (new SitePattern variants).

## Distribution

Same as `saf-feature-dev` — lives under `skills/saf-checker-dev/` in the SAF monorepo. Multi-format: Claude Code plugin + Codex AGENTS.md fragment. Shares references with sibling skill (`saf-invariants.md`, `saf-log-guide.md`, `e2e-testing-guide.md`).

## Design Decisions

- **Audience**: Both AI agents (autonomous checker authoring via Python SDK) and human developers guided by coding agents
- **Scope**: Tier 1 (declarative specs) and Tier 2 (typestate) as primary targets. Tier 3 (custom patterns) with awareness, escalating to `saf-feature-dev` for framework changes.
- **Knowledge approach**: Hybrid — embed the SitePattern catalog, reachability mode decision tree, and YAML spec format. Point to code for solver internals and API details that evolve.
- **Workflow**: Spec-first (YAML/Python before Rust), iterative test-debug-refine loop, no plans/ unless Tier 3.

## Checker Tier Classification

| Tier | Description | Authoring Path | Example |
|---|---|---|---|
| Tier 1: Declarative | Fits existing SitePattern + reachability modes | YAML specs + CheckerSpec (Rust) or check_custom() (Python) | Memory leak, UAF for custom allocator |
| Tier 2: Typestate | State machine transitions on a resource | TypestateSpec via Python API or YAML | DB connection lifecycle, lock ordering |
| Tier 3: Custom patterns | Needs new SitePattern enum variant | New variant + classifier logic in Rust, then compose | Crypto key use after zeroing |

## Workflow (8 Phases)

### Phase 1: Understand the Bug Pattern
- Extract: source (where bad value originates), sink (where bug manifests), sanitizer (what prevents it)
- Map to CWE if applicable
- For human-guided: ask these questions explicitly
- For autonomous agents: derive from bug description

### Phase 2: Classify Checker Tier
- Map source/sink/sanitizer to a reachability mode (MayReach, MustNotReach, MultiReach, NeverReachSink)
- Check if existing SitePattern variants cover the pattern
- Check if typestate is a better fit
- Load checker-types-guide.md for decision tree

### Phase 3: Explore Existing Checkers
- Find closest built-in among SAF's 9 checkers in `crates/saf-analysis/src/checkers/spec.rs`
- Read its spec and trace how runner resolves its patterns
- Use as template

### Phase 4: Write the Spec
- Tier 1: create YAML function specs if needed, compose CheckerSpec
- Tier 2: define TypestateSpec with states and transitions
- Tier 3: implement new SitePattern variant first, then compose
- Python path: use check_custom() or typestate_custom()

### Phase 5: Create Test Cases
- Write bad variant: `tests/programs/c/<checker>_bad.c` (bug present)
- Write good variant: `tests/programs/c/<checker>_good.c` (bug absent, sanitizer present)
- Compile both inside Docker to .ll fixtures
- Write e2e tests: bad produces findings, good produces zero findings

### Phase 6: Run and Debug
- Execute checker, use SAF_LOG=checker[reasoning,path,result]
- For PTA precision issues: SAF_LOG=checker[reasoning],pta::solve[pts]
- Compare findings against expected results

### Phase 7: Refine (iterative loop with Phase 6)
- False positives → add sanitizers, tighten patterns, consider may_reach_guarded
- False negatives → broaden patterns, check YAML specs cover library functions, verify SVFG edges
- When stable, run PTABen/Juliet if CWE category is supported

### Phase 8: Export and Document
- Place YAML specs in discovery path (saf-specs/ for project-local, share/saf/specs/ for bundled)
- For Rust built-ins: add to builtin_checkers() in spec.rs
- For Python: create reusable script or tutorial
- Document: what it detects, limitations, FP/FN characteristics

## File Structure

```
skills/saf-checker-dev/
  core/
    workflow.md                      # 8-phase checker development workflow
    references/
      checker-types-guide.md         # Reachability modes, SitePattern catalog, decision tree
      spec-authoring-guide.md        # YAML spec format, CheckerSpec composition, Python API
      test-case-guide.md             # Good/bad variant C programs, assertion patterns
  claude-code/
    .claude-plugin/plugin.json
    skills/saf-checker-dev/
      SKILL.md
      references/                    # 3 own + 3 shared from saf-feature-dev
  codex/
    saf-checker-dev-workflow.md
  build.sh
```

## Implementation Steps

### Step 1: Create directory structure
- Initialize skills/saf-checker-dev/ with core/, claude-code/, codex/ dirs

### Step 2: Write core/workflow.md
- 8-phase checker workflow in platform-agnostic language
- Spec-first emphasis, iterative test-debug-refine loop

### Step 3: Write core/references/checker-types-guide.md
- Full SitePattern catalog (all variants with descriptions)
- Reachability mode decision tree (flowchart: source/sink/sanitizer → mode)
- All 9 built-in checkers as reference examples
- Tier classification decision tree

### Step 4: Write core/references/spec-authoring-guide.md
- YAML spec format (version "1.0", FunctionSpec fields, discovery paths)
- CheckerSpec composition in Rust (with complete examples)
- Python check_custom() API (parameters, example)
- Python typestate_custom() API (TypestateSpec fields, example)
- ResourceTable extension (adding custom roles)

### Step 5: Write core/references/test-case-guide.md
- Good/bad variant pattern with concrete examples
- How to write C programs that exercise specific checker patterns
- Assertion patterns for checker e2e tests (findings.len() > 0 for bad, == 0 for good)
- Taint-specific test patterns (tainted vs sanitized paths)
- Typestate-specific test patterns (each error transition)

### Step 6: Write Claude Code plugin
- plugin.json manifest
- SKILL.md (under 500 words, defers to references)
- Copy own references + shared references from saf-feature-dev

### Step 7: Write Codex format
- Self-contained saf-checker-dev-workflow.md (under 15 KiB)
- Inline critical content, point to saf-dev-skills repo for detail

### Step 8: Write build script
- Syncs core/ → claude-code/references/
- Copies shared references from ../saf-feature-dev/core/references/

### Step 9: Test with subagent
- Scenario: "Create a checker that detects database connections opened but never closed"
- Verify: correct tier classification (Tier 1 or Tier 2), correct mode selection, correct test patterns

### Step 10: Update CLAUDE.md and AGENTS.md
- Add saf-checker-dev to the Development Skills section

### Step 11: Final verification and commit
